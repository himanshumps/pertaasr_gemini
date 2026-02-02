use std::io::{self, Write};
use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::client::conn::http1;
use hyper::{http, Request};
use hyper_util::rt::TokioIo;
use quanta::Clock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Barrier;
use tokio_util::sync::CancellationToken;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

macro_rules! diag {
    ($($arg:tt)*) => {{
        print!($($arg)*);
        let _ = io::stdout().flush();
    }};
}

fn main() {
    let num_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2);
    let total_conns = 40; // Increased connections to saturate network pipeline
    let barrier = Arc::new(Barrier::new(total_conns));
    let token = CancellationToken::new();
    let clock = Clock::new();
    diag!("[1/5] Initialising Maximum-Throughput Benchmark with os threads: {}", num_threads);
    let mut thread_handles = vec![];
    let mut tasks_spawned = 0;

    for i in 0..num_threads {
        let b = Arc::clone(&barrier);
        let t = token.clone();

        let tasks_on_thread = if i == num_threads - 1 { total_conns - tasks_spawned } else { total_conns / num_threads };
        tasks_spawned += tasks_on_thread;

        thread_handles.push(std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

            rt.block_on(async move {
                let mut local_tasks = vec![];
                for _ in 0..tasks_on_thread {
                    let b = Arc::clone(&b);
                    let t = t.clone();

                    local_tasks.push(tokio::spawn(async move {
                        let target = "rust-server.himanshumps-1-dev.svc.cluster.local:8080";
                        let host = "rust-server.himanshumps-1-dev.svc.cluster.local";

                        // Raw TCP connection setup
                        let stream = TcpStream::connect(target).await.expect("TCP Connect Failed");
                        stream.set_nodelay(true).ok();
                        // Low-level HTTP/1 handshake with vectored writes enabled
                        let (mut sender, conn) = http1::Builder::new()
                            .max_buf_size(8192 * 4)
                            .handshake(TokioIo::new(stream))
                            .await
                            .unwrap();

                        // Drive connection on this core's executor
                        tokio::spawn(async move { let _ = conn.await; });

                        let req_template = Request::builder()
                            .version(http::Version::HTTP_11)
                            .uri("/")
                            .header("Host", host)
                            .header("Connection", "keep-alive")
                            .body(Empty::<Bytes>::new())
                            .unwrap();

                        b.wait().await;

                        let mut local_count = 0u64;
                        // ATOMIC-FREE HOT LOOP
                        loop {
                            if t.is_cancelled() {
                                break;
                            }
                            for _ in 0..128 { // check every 128 request
                                let req = req_template.clone();
                                if let Ok(resp) = sender.send_request(req).await {
                                    // Efficient body draining
                                    let mut body = resp.into_body();
                                    while let Some(frame_res) = body.frame().await {
                                        if let Ok(frame) = frame_res { drop(frame); }
                                    }
                                    local_count += 1;
                                } else { break; }
                            }
                        }
                        local_count
                    }));
                }

                let mut thread_total = 0u64;
                for h in local_tasks {
                    thread_total += h.await.unwrap_or(0);
                }
                thread_total
            })
        }));
    }

    diag!("[2/5] Warmup complete. Starting 120s blast...\n");
    let start_time = clock.now();

    // PROGRESS THREAD (Non-blocking)

    std::thread::sleep(Duration::from_secs(120));
    token.cancel();

    let total_elapsed = clock.now() - start_time;
    diag!("\n[3/5] Test completed. Aggregating results...");

    let total_requests: u64 = thread_handles.into_iter().map(|h| h.join().unwrap()).sum();

    diag!("\n[4/5] Results Summary:\n");
    println!("--------------------------------------------------");
    println!("Total Requests:  {}", total_requests);
    println!("Actual Duration: {:.2?}", total_elapsed);
    println!("Throughput:      {:.2} req/sec", total_requests as f64 / total_elapsed.as_secs_f64());
    println!("--------------------------------------------------");
    diag!("[5/5] Benchmark finished.\n");
}
