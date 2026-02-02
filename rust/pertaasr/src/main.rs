use bytes::Bytes;
use hdrhistogram::Histogram;
use http_body_util::{BodyExt, Empty};
use hyper::client::conn::http1;
use hyper::{Request, http};
use hyper_util::rt::TokioIo;
use quanta::Clock;
use std::env;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{Barrier, mpsc};
use tokio_util::sync::CancellationToken;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

macro_rules! diag {
    ($($arg:tt)*) => {{ print!($($arg)*); let _ = io::stdout().flush(); }};
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Usage: {} <host:port> <duration_secs> <connections>",
            args[0]
        );
        std::process::exit(1);
    }

    let target_addr = Arc::new(args[1].clone());
    let duration_secs = args[2].parse::<u64>().expect("Invalid duration");
    let total_conns = args[3].parse::<usize>().expect("Invalid connection count");

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    let barrier = Arc::new(Barrier::new(total_conns));
    let token = CancellationToken::new();
    let clock = Clock::new();
    diag!("[1/4] Running test for {}s with {} os threads and {} connections...\n", duration_secs, num_threads, total_conns);
    let mut thread_handles = vec![];
    let mut tasks_spawned = 0;

    for i in 0..num_threads {
        let b = Arc::clone(&barrier);
        let t = token.clone();
        let addr = Arc::clone(&target_addr);
        let c_thread = clock.clone();
        let tasks_on_thread = if i == num_threads - 1 {
            total_conns - tasks_spawned
        } else {
            total_conns / num_threads
        };
        tasks_spawned += tasks_on_thread;

        thread_handles.push(std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                // 1. BUSY-SPIN WARMUP: Keep this worker thread "hot" immediately

                let mut tasks = vec![];
                for _ in 0..tasks_on_thread {
                    let b_inner = Arc::clone(&b);
                    let t_inner = t.clone();
                    let addr_inner = Arc::clone(&addr);
                    let c_inner = c_thread.clone();

                    tasks.push(tokio::spawn(async move {
                        let host = addr_inner.split(':').next().unwrap_or("localhost");
                        let mut hist = Histogram::<u64>::new_with_bounds(1, 10_000_000, 1).unwrap();
                        // 3. BARRIER: Wait for ALL tasks on ALL threads to be fully connected
                        let req_base = Request::builder()
                            .version(http::Version::HTTP_11)
                            .uri("/")
                            .header("Host", host)
                            .header("Connection", "keep-alive")
                            .body(Empty::<Bytes>::new())
                            .unwrap();
                        b_inner.wait().await;

                        // 2. CONNECT PHASE (Cold): Do this before the barrier
                        let stream = TcpStream::connect(&*addr_inner)
                            .await
                            .expect("Connect failed");
                        stream.set_nodelay(true).ok();
                        let (mut sender, conn) = http1::Builder::new()
                            .writev(true)
                            .handshake(TokioIo::new(stream))
                            .await
                            .unwrap();
                        tokio::spawn(async move {
                            let _ = conn.await;
                        });


                        // 4. MEASUREMENT LOOP: Optimized count-based check
                        //let mut loop_count: u8 = 0;
                        loop {
                            if t_inner.is_cancelled() {
                                break;
                            }
                            for _ in 0..128 {
                                let start = c_inner.now();

                                if let Ok(resp) = sender.send_request(req_base.clone()).await {
                                    let mut body = resp.into_body();
                                    while let Some(Ok(frame)) = body.frame().await {
                                        let _ = drop(frame);
                                    }
                                    hist += (c_inner.now() - start).as_micros() as u64;
                                    //let _ = hist.record((c_inner.now() - start).as_micros() as u64);
                                } else {
                                    break;
                                }
                            }
                        }
                        hist
                    }));
                }

                // Wait for all benchmark tasks to finish
                let mut thread_hist = Histogram::<u64>::new_with_bounds(1, 10_000_000, 3).unwrap();
                for h in tasks {
                    if let Ok(lh) = h.await {
                        thread_hist.add(lh).unwrap();
                    }
                }
                // Finally, stop the busy-spin task
                thread_hist
            })
        }));
    }

    diag!("[2/4] Connections established and threads hot. Starting blast...\n");
    let start_time = clock.now();
    std::thread::sleep(Duration::from_secs(duration_secs));
    token.cancel();

    let mut final_hist = Histogram::<u64>::new_with_bounds(1, 10_000_000, 3).unwrap();
    for h in thread_handles {
        final_hist.add(h.join().unwrap()).unwrap();
    }
    let total_elapsed = clock.now() - start_time;
    println!("Total requests: {}", final_hist.len());
    println!(
        "\nThroughput: {:.2} req/sec",
        final_hist.len() as f64 / total_elapsed.as_secs_f64()
    );
    println!("P99: {} µs", final_hist.value_at_quantile(0.99));
}
