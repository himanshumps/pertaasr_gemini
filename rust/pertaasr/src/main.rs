use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::client::conn::http1;
use hyper::Request;
use hyper_util::rt::TokioIo; // The fix for E0277
use quanta::Clock;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Barrier;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    println!("[Main] Starting high-perf raw handshake benchmark...");
    let core_ids = core_affinity::get_core_ids().unwrap_or_else(|| vec![]);
    let num_cores = core_ids.len().max(1);
    let total_conns = 20;

    let barrier = Arc::new(Barrier::new(total_conns));
    let clock = Clock::new();
    let duration = Duration::from_secs(120);
    let start_time = clock.now();
    let mut thread_handles = vec![];
    let mut tasks_spawned = 0;
    println!("[Main] Starting 0..num_cores...");
    for i in 0..num_cores {
        let b = Arc::clone(&barrier);
        let c = clock.clone();
        let core_id = core_ids.get(i).cloned();

        let tasks_on_this_thread = if i == num_cores - 1 {
            total_conns - tasks_spawned
        } else {
            total_conns / num_cores
        };
        tasks_spawned += tasks_on_this_thread;

        thread_handles.push(std::thread::spawn(move || {
            if let Some(id) = core_id { core_affinity::set_for_current(id); }

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let mut conn_handles = vec![];

                for _ in 0..tasks_on_this_thread {
                    let b = Arc::clone(&b);
                    let c = c.clone();

                    conn_handles.push(tokio::spawn(async move {
                        let target = "rust-server.himanshumps-1-dev.svc.cluster.local:8080";
                        let host = "rust-server.himanshumps-1-dev.svc.cluster.local";
                        let mut local_count = 0u64;
                        println!("Trying connection to {}...", target);
                        // 1. TCP Connect
                        let stream = TcpStream::connect(target).await.expect("Connect failed");
                        println!("Trying connection to {} succeeded...", target);
                        stream.set_nodelay(true).ok();

                        // 2. Wrap in TokioIo to satisfy hyper::rt::Read/Write
                        let io = TokioIo::new(stream);

                        // 3. HTTP/1 Handshake
                        let (mut sender, conn) = http1::handshake(io).await.expect("Handshake failed");

                        // 4. Drive connection
                        tokio::spawn(async move {
                            if let Err(err) = conn.await {
                                // Ignore errors after test duration finishes
                            }
                        });

                        b.wait().await;
                        println!("Starting the test...");
                        while c.now() - start_time < duration {
                            let req = Request::builder()
                                .uri("/")
                                .header("Host", host)
                                .header("Connection", "keep-alive")
                                .body(Empty::<Bytes>::new())
                                .unwrap();

                            if let Ok(resp) = sender.send_request(req).await {
                                let mut body = resp.into_body();
                                while let Some(frame_res) = body.frame().await {
                                    if let Ok(frame) = frame_res {
                                        std::mem::drop(frame);
                                    }
                                }
                                local_count += 1;
                            } else {
                                break; // Connection closed
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

    let total_requests: u64 = thread_handles.into_iter().map(|h| h.join().unwrap()).sum();
    let total_elapsed = clock.now() - start_time;

    println!("\n--- Benchmark Results ---");
    println!("Total Requests: {}", total_requests);
    println!("Requests/sec:   {:.2}", total_requests as f64 / total_elapsed.as_secs_f64());
}
