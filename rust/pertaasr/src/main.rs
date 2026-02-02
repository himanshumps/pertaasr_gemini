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
    diag!("[1/5] Initializing benchmark...");
    let num_threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
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
        let tasks_on_this_thread = if i == num_threads - 1 { total_conns - tasks_spawned } else { total_conns / num_threads };
        tasks_spawned += tasks_on_this_thread;

        thread_handles.push(std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();

            rt.block_on(async move {
                let mut conn_handles = vec![];
                for _ in 0..tasks_on_this_thread {
                    let b = Arc::clone(&b);
                    let t = t.clone();

                    conn_handles.push(tokio::spawn(async move {
                        let target = "rust-server.himanshumps-1-dev.svc.cluster.local:8080";
                        let host = "rust-server.himanshumps-1-dev.svc.cluster.local";

                        let stream = TcpStream::connect(target).await.expect("Connect failed");
                        stream.set_nodelay(true).ok();

                        // HANDSHAKE OPTIMIZATION: Set writev(true) to reduce syscall count
                        let (mut sender, conn) = http1::Builder::new()
                            .writev(true)
                            .handshake(TokioIo::new(stream))
                            .await
                            .unwrap();

                        tokio::spawn(async move { let _ = conn.await; });

                        // PRE-BUILD: Clone() is O(1) and bypasses builder overhead
                        let req_template = Request::builder()
                            .version(http::Version::HTTP_11)
                            .uri("/")
                            .header("Host", host)
                            .header("Connection", "keep-alive")
                            .body(Empty::<Bytes>::new())
                            .unwrap();

                        b.wait().await;

                        let mut local_count = 0u64;
                        while !t.is_cancelled() {
                            let req = req_template.clone();
                            // send_request is the hot path
                            if let Ok(resp) = sender.send_request(req).await {
                                // MOST PERFORMANT BODY DISCARD:
                                // resp.into_body().collect().await consumes the stream in bulk
                                let _ = resp.into_body().collect().await;
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
    diag!("[3/5] Warmup complete. Starting 120s blast...\n");

    std::thread::sleep(Duration::from_secs(120));
    token.cancel();
    diag!("\n[4/5] Test completed. Aggregating results...");

    let total_requests: u64 = thread_handles.into_iter().map(|h| h.join().unwrap()).sum();
    let total_elapsed = clock.now() - start_time;

    diag!("\n[5/5] Metrics Summary:\n");
    println!("--------------------------------------------------");
    println!("Total Requests:  {}", total_requests);
    println!("Total Duration:  {:.2?}", total_elapsed);
    println!("Throughput:      {:.2} req/sec", total_requests as f64 / total_elapsed.as_secs_f64());
    println!("--------------------------------------------------");

}
