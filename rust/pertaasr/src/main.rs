mod utils;

use bytes::Bytes;
use http_body_util::Full;
use http_wire::WireEncodeAsync;
use metrics_lib::{AsyncMetricBatch, init, metrics};
use quanta::Clock;
use std::env;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Barrier;
use tokio_util::sync::CancellationToken;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

macro_rules! diag {
    ($($arg:tt)*) => {{ print!($($arg)*); let _ = io::stdout().flush(); }};
}

fn main() {
    init();
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
    let total_connections = args[3].parse::<usize>().expect("Invalid connection count");

    let core_ids = core_affinity::get_core_ids().expect("Failed to get core IDs");
    let num_threads = core_ids.len();

    println!(
        "[INIT] Using {} logical cores via core_affinity.",
        num_threads
    );

    let barrier = Arc::new(Barrier::new(total_connections));
    let token = CancellationToken::new();
    let clock = Clock::new();

    diag!(
        "[1/4] Starting {} OS threads for {} connections for a duration of {}s...\n",
        num_threads,
        total_connections,
        duration_secs
    );

    let mut thread_handles = vec![];
    let mut tasks_spawned = 0;

    for i in 0..num_threads {
        let b = Arc::clone(&barrier);
        let t = token.clone();
        let addr = Arc::clone(&target_addr);
        let c_thread = clock.clone();
        let core_id = core_ids[i];

        let tasks_on_thread = if i == num_threads - 1 {
            total_connections - tasks_spawned
        } else {
            total_connections / num_threads
        };
        tasks_spawned += tasks_on_thread;

        thread_handles.push(std::thread::spawn(move || {
            // Pin OS thread to specific core
            core_affinity::set_for_current(core_id);
            //let timer_metric = metrics().timer("request_latency");

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let host = addr.split(':').next().unwrap_or("localhost");
                // Pre-serialize the request bytes once per thread to avoid allocation in the loop
                let raw_req_base = http::Request::builder()
                    .method("GET")
                    .uri(format!("http://{}/", addr.clone()))
                    .header("host", host)
                    .header("connection", "keep-alive")
                    .body(Full::new(Bytes::from("")))
                    .unwrap()
                    .encode_async()
                    .await
                    .unwrap(); // This is now a 'Bytes' object
                let mut tasks = vec![];
                for _ in 0..tasks_on_thread {
                    let b_inner = Arc::clone(&b);
                    let t_inner = t.clone();
                    let addr_inner = Arc::clone(&addr);
                    let c_inner = c_thread.clone();
                    let raw_req = raw_req_base.clone();
                    //let timer_metric = metrics().timer("request_latency");

                    tasks.push(tokio::spawn(async move {
                        // Nanosecond precision for the histogram
                        //let mut hist = Histogram::<u64>::new_with_bounds(1, 10_000_000_000, 3).unwrap();

                        // Raw TCP Stream bypasses Hyper framework overhead
                        let mut stream = TcpStream::connect(&*addr_inner)
                            .await
                            .expect("Connect failed");
                        stream.set_nodelay(true).ok();

                        let mut read_buf = [0u8; 1024]; // Buffer to drain the response

                        // Wait for all connections to be ready
                        b_inner.wait().await;

                        loop {
                            if t_inner.is_cancelled() {
                                break;
                            }
                            let mut batch_metrics = AsyncMetricBatch::new();
                            for _ in 0..128 {
                                // Larger batch
                                let start = c_inner.now();

                                // 1. ATTEMPT SYSCALL DIRECTLY
                                if let Err(_) = stream.try_write(&raw_req) {
                                    stream.write_all(&raw_req).await.ok();
                                }
                                match stream.read(&mut read_buf).await {
                                    Ok(0) | Err(_) => break,
                                    Ok(_) => {
                                        // Record latency in nanoseconds
                                        metrics().timer("request_latency").record_ns((c_inner.now() - start).as_nanos() as u64);
                                    }
                                }
                            }
                            batch_metrics.flush(metrics());
                        }
                        //hist
                    }));
                }

                for h in tasks {
                    if let Ok(_lh) = h.await {}
                }
                //thread_hist
            })
        }));
    }

    diag!("[2/4] Starting the test and aligning threads...\n");
    let start_time = clock.now();
    std::thread::sleep(Duration::from_secs(duration_secs));
    token.cancel();

    //let mut final_hist = Histogram::<u64>::new_with_bounds(1, 10_000_000_000, 3).unwrap();
    for h in thread_handles {
        h.join().unwrap();
    }

    let total_elapsed = clock.now() - start_time;
    let total_reqs = metrics().timer("request_latency").count();

    println!("\n[RESULTS]");
    println!("Total requests:  {}", total_reqs);
    println!(
        "Throughput:      {:.2} req/sec",
        total_reqs as f64 / total_elapsed.as_secs_f64()
    );
    println!(
        "Latency P50:     {:.2} µs",
        metrics().timer("request_latency").average().as_micros()
    );
    println!(
        "Latency Max:     {:.2} µs",
        metrics().timer("request_latency").max().as_micros()
    );
}
