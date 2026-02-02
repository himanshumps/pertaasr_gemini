use std::io::{self, Write};
use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::client::conn::http1;
use hyper::{http, Request};
use hyper_util::rt::TokioIo;
use quanta::Clock;
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
        println!($($arg)*);
        let _ = io::stdout().flush();
    }};
}

fn main() {
    diag!("[1/5] Initialising Maximum-Throughput Benchmark...");

    // Respect OpenShift quotas: 2 cores = 2 threads
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let total_conns = 20;
    let barrier = Arc::new(Barrier::new(total_conns));
    let token = CancellationToken::new();
    let clock = Clock::new();

    let mut thread_handles = vec![];
    let mut tasks_spawned = 0;

    for i in 0..num_threads {
        let b = Arc::clone(&barrier);
        let t = token.clone();

        let tasks_on_this_thread = if i == num_threads - 1 {
            total_conns - tasks_spawned
        } else {
            total_conns / num_threads
        };
        tasks_spawned += tasks_on_this_thread;

        thread_handles.push(std::thread::spawn(move || {
            // Each thread gets its own current_thread runtime to avoid cross-core theft
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let mut conn_handles = vec![];
                for _ in 0..tasks_on_this_thread {
                    let b = Arc::clone(&b);
                    let t = t.clone();

                    conn_handles.push(tokio::spawn(async move {
                        let target = "rust-server.himanshumps-1-dev.svc.cluster.local:8080";
                        let host = "rust-server.himanshumps-1-dev.svc.cluster.local";

                        // 1. Establish raw TCP stream
                        let stream = TcpStream::connect(target).await.expect("Connect failed");
                        stream.set_nodelay(true).ok();

                        // 2. Perform Raw Handshake (Zero pooling overhead)
                        let (mut sender, conn) = http1::Builder::new()
                            .writev(true) // Reduces syscalls by batching writes
                            .handshake(TokioIo::new(stream))
                            .await
                            .unwrap();

                        // 3. Drive connection on the local executor
                        tokio::spawn(async move {
                            let _ = conn.await;
                        });

                        // PRE-BUILD: Clone() is O(1) and bypasses builder overhead
                        let req_template = Request::builder()
                            .version(http::Version::HTTP_11)
                            .uri("/")
                            .header("Host", host)
                            .header("Connection", "keep-alive")
                            .body(Empty::<Bytes>::new())
                            .unwrap();

                        // Sync all tasks at the starting line
                        b.wait().await;

                        let mut local_count = 0u64;
                        while !t.is_cancelled() {
                            let req = req_template.clone();

                            // RAW SENDER: Faster than high-level Client
                            if let Ok(resp) = sender.send_request(req).await {
                                let mut body = resp.into_body();
                                // FAST DRAIN: Avoids collect() allocations
                                while let Some(frame_res) = body.frame().await {
                                    if let Ok(frame) = frame_res {
                                        std::mem::drop(frame);
                                    }
                                }
                                local_count += 1;
                            } else {
                                break;
                            }
                        }
                        local_count
                    }));
                }

                let mut total = 0;
                for h in conn_handles { total += h.await.unwrap_or(0); }
                total
            })
        }));
    }

    diag!("[2/5] Warmup complete. Starting 120s blast...");
    let start_time = clock.now();

    std::thread::sleep(Duration::from_secs(120));
    token.cancel();

    let total_requests: u64 = thread_handles.into_iter().map(|h| h.join().unwrap()).sum();
    let total_elapsed = clock.now() - start_time;

    diag!("[3/5] Test completed.");
    println!("\n--- Results ---");
    println!("Total Requests:  {}", total_requests);
    println!("Actual Elapsed:  {:.2?}", total_elapsed);
    println!("Requests/sec:    {:.2}", total_requests as f64 / total_elapsed.as_secs_f64());
}
