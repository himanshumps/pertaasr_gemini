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

macro_rules! diag {
    ($($arg:tt)*) => {{
        println!($($arg)*);
        let _ = io::stdout().flush();
    }};
}

fn main() {
    diag!("[1/6] Process starting...");

    let core_ids = core_affinity::get_core_ids().unwrap_or_else(|| {
        diag!("[!] Core detection failed; using soft-threading");
        vec![]
    });

    let num_cores = core_ids.len().max(1);
    let total_conns = 20;
    let barrier = Arc::new(Barrier::new(total_conns));
    let clock = Clock::new();
    let duration = Duration::from_secs(120);
    let start_time = clock.now();

    diag!("[2/6] Detected {} cores. Distributing {} connections...", num_cores, total_conns);

    let mut thread_handles = vec![];
    let mut tasks_spawned = 0;

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
            if let Some(id) = core_id {
                // If this hangs, OpenShift SCC is blocking affinity
                core_affinity::set_for_current(id);
            }

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let mut conn_handles = vec![];
                for t_idx in 0..tasks_on_this_thread {
                    let b = Arc::clone(&b);
                    let c = c.clone();

                    conn_handles.push(tokio::spawn(async move {
                        let target = "rust-server.himanshumps-1-dev.svc.cluster.local:8080";
                        let host = "rust-server.himanshumps-1-dev.svc.cluster.local";

                        let stream = match tokio::time::timeout(Duration::from_secs(10), TcpStream::connect(target)).await {
                            Ok(Ok(s)) => s,
                            Ok(Err(e)) => panic!("Connect error: {}", e),
                            Err(_) => panic!("Connect timeout"),
                        };

                        stream.set_nodelay(true).ok();
                        let (mut sender, conn) = http1::handshake(TokioIo::new(stream)).await.unwrap();
                        tokio::spawn(async move { let _ = conn.await; });

                        // PRE-BUILD TEMPLATE (Saves ~20% CPU vs building in loop)
                        let req_template = Request::builder()
                            .version(http::Version::HTTP_11)
                            .uri("/")
                            .header("Host", host)
                            .header("Connection", "keep-alive")
                            .body(Empty::<Bytes>::new())
                            .unwrap();

                        diag!("[3/6] Task ready on core {:?}. Waiting at barrier...", core_id);
                        b.wait().await;

                        // Milestones for the first task only
                        if i == 0 && t_idx == 0 { diag!("[4/6] Barrier released. Entering tight loop..."); }

                        let mut local_count = 0u64;
                        while c.now() - start_time < duration {
                            // Cheap O(1) clone of the request
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

    diag!("[5/6] All threads spawned. Benchmark in progress...");

    let total_requests: u64 = thread_handles.into_iter().map(|h| h.join().unwrap()).sum();
    let total_elapsed = clock.now() - start_time;

    diag!("[6/6] Benchmark finished.");
    println!("\n--- Results ---");
    println!("Total Requests:  {}", total_requests);
    println!("Actual Elapsed:  {:.2?}", total_elapsed);
    println!("Requests/sec:    {:.2}", total_requests as f64 / total_elapsed.as_secs_f64());
}
