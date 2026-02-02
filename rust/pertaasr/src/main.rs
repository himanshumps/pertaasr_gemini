use std::io::{self, Write};
use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::client::conn::http1;
use hyper::{http, Request};
use hyper_util::rt::{TokioExecutor, TokioIo};
use quanta::Clock;
use std::sync::Arc;
use std::time::Duration;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use tokio::net::TcpStream;
use tokio::sync::Barrier;
use tokio_util::sync::CancellationToken;


macro_rules! diag {
    ($($arg:tt)*) => {{
        println!($($arg)*);
        let _ = io::stdout().flush();
    }};
}

fn main() {
    diag!("[1/5] Initialising benchmark (Cancellation Mode)...");

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let total_conns = 20;
    let barrier = Arc::new(Barrier::new(total_conns));
    let token = CancellationToken::new();
    let clock = Clock::new();
    let start_time = clock.now();

    diag!("[2/5] Using {} OS threads for {} connections", num_threads, total_conns);

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

                        // 1. Establish raw connection once per task
                        let stream = TcpStream::connect(target).await.expect("Connect failed");
                        stream.set_nodelay(true).ok();
                        // 2. Perform raw handshake
                        let (mut sender, conn) = http1::Builder::new()
                            .allow_obsolete_multiline_headers_in_responses(true)
                            .allow_spaces_after_header_name_in_responses(true)
                            .ignore_invalid_headers_in_responses(true)
                            .writev(true) // Reduce syscalls
                            .handshake(TokioIo::new(stream))
                            .await
                            .unwrap();

                        // 3. Drive the connection on the local executor
                        tokio::spawn(async move {
                            if let Err(e) = conn.await { /* handle/ignore */ }
                        });

                        b.wait().await;

                        let req_template = Request::builder()
                            .uri("/")
                            .header("Host", "rust-server.himanshumps-1-dev.svc.cluster.local")
                            .header("Connection", "keep-alive")
                            .body(Empty::<Bytes>::new())
                            .unwrap();

                        let mut local_count = 0u64;
                        while !t.is_cancelled() {
                            let req = req_template.clone();
                            // send_request on a raw handshake sender is faster than legacy client
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

    diag!("[3/5] Connections established. Starting 120s blast...");

    // Main thread controls the timer
    std::thread::sleep(Duration::from_secs(120));
    token.cancel();

    diag!("[4/5] Timer expired. Collecting results...");

    let total_requests: u64 = thread_handles.into_iter().map(|h| h.join().unwrap()).sum();
    let total_elapsed = clock.now() - start_time;

    diag!("[5/5] Done.");
    println!("\n--- Results ---");
    println!("Total Requests:  {}", total_requests);
    println!("Actual Elapsed:  {:.2?}", total_elapsed);
    println!("Requests/sec:    {:.2}", total_requests as f64 / total_elapsed.as_secs_f64());
}
