use std::io;
use std::io::Write;
use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::client::conn::http1;
use hyper::{http, Request};
use hyper_util::rt::TokioIo; // The fix for E0277
use quanta::Clock;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Barrier;

fn main() {
    println!("[Main] Starting high-perf raw handshake benchmark...");
    io::stdout().flush().unwrap();
    let core_ids = core_affinity::get_core_ids().unwrap_or_else(|| {
        println!("[WARN] Could not detect cores, defaulting to soft-parallelism");
        io::stdout().flush().unwrap();
        vec![]
    });
    let num_cores = core_ids.len().max(1);
    let total_conns = 20;
    let barrier = Arc::new(Barrier::new(total_conns));
    let clock = Clock::new();
    let duration = Duration::from_secs(120);
    let start_time = clock.now();

    let mut thread_handles = vec![];
    let mut tasks_spawned = 0;
    println!("0..num_cores");
    io::stdout().flush().unwrap();
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
            // 2. Safer pinning: some Docker runtimes hang on set_for_current
            if let Some(id) = core_id {
                core_affinity::set_for_current(id);
            }

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
                        println!("Connecting");
                        io::stdout().flush().unwrap();
                        // TCP Connect with timeout to prevent hanging
                        let stream = tokio::time::timeout(
                            Duration::from_secs(10),
                            TcpStream::connect(target)
                        ).await.expect("Connect timed out").expect("Connect failed");
                        println!("connected");
                        stream.set_nodelay(true).ok();

                        let (mut sender, conn) = http1::handshake(TokioIo::new(stream)).await.unwrap();

                        tokio::spawn(async move { let _ = conn.await; });

                        // SYNC POINT
                        b.wait().await;
                        let target_url: hyper::Uri = "/".parse().unwrap();
                        let req_template = Request::builder()
                            .version(http::Version::HTTP_11)
                            .uri(target_url)
                            .header("Host", host)
                            .header("Connection", "keep-alive")
                            .body(Empty::<Bytes>::new())
                            .unwrap();
                        let mut local_count = 0u64;
                        while c.now() - start_time < duration {
                            let req = req_template.clone();
                            if let Ok(resp) = sender.send_request(req).await {
                                let mut body = resp.into_body();
                                while let Some(Ok(frame)) = body.frame().await {
                                    drop(frame);
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

    println!("[RUN] All threads started. Waiting 120s...");
    io::stdout().flush().unwrap();

    let total_requests: u64 = thread_handles.into_iter().map(|h| h.join().unwrap()).sum();
    let total_elapsed = clock.now() - start_time;

    println!("\n--- Results ---");
    println!("Elapsed {:?}", total_elapsed);
    println!("Count {}", total_requests);
    println!("Requests/sec: {:.2}", total_requests as f64 / total_elapsed.as_secs_f64());
}