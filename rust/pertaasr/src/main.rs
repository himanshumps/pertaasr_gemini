use std::sync::Arc;
use std::time::Duration;
use hyper::Request;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use http_body_util::{BodyExt, Empty};
use bytes::Bytes;
use quanta::Clock;
use tokio::sync::Barrier;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    println!("Running the test using hyper and tokio");
    let core_ids = core_affinity::get_core_ids().unwrap_or_default();
    // Docker restricted us to 2 cores; this will capture cores 0 and 1
    let num_cores = core_ids.len();
    let total_conns = 20;
    let conns_per_core = total_conns / num_cores;

    let barrier = Arc::new(Barrier::new(total_conns));
    let clock = Clock::new();
    let duration = Duration::from_secs(120);
    let start_time = clock.now();

    let mut thread_handles = vec![];

    for core_id in core_ids {
        let b = Arc::clone(&barrier);
        let c = clock.clone();

        thread_handles.push(std::thread::spawn(move || {
            core_affinity::set_for_current(core_id);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let mut conn_handles = vec![];

                // Construct the client once per core-thread for maximum reuse
                let mut connector = HttpConnector::new();
                connector.set_nodelay(true);
                connector.enforce_http(true);

                let client = Arc::new(
                    Client::builder(TokioExecutor::new())
                        .build::<_, Empty<Bytes>>(connector)
                );

                for _ in 0..conns_per_core {
                    let b = Arc::clone(&b);
                    let c = c.clone();
                    let client = Arc::clone(&client);

                    conn_handles.push(tokio::spawn(async move {
                        let url: hyper::Uri = "http://rust-server.himanshumps-1-dev.svc.cluster.local:8080/".parse().unwrap();
                        let mut local_count = 0u64;

                        b.wait().await;

                        while c.now() - start_time < duration {
                            // Use http_body_util::Empty to fix the "private" error
                            let req = Request::builder()
                                .uri(&url)
                                .header("Host", "rust-server-ffi")
                                .body(Empty::<Bytes>::new())
                                .unwrap();

                            if let Ok(mut resp) = client.request(req).await {
                                let body = resp.body_mut();
                                // Efficiently stream and discard
                                while let Some(frame) = body.frame().await {
                                    if let Ok(f) = frame {
                                        std::mem::drop(f);
                                    }
                                }
                                local_count += 1;
                            }
                        }
                        local_count
                    }));
                }

                let mut core_total = 0;
                for h in conn_handles {
                    core_total += h.await.unwrap_or(0);
                }
                core_total
            })
        }));
    }

    let total_requests: u64 = thread_handles.into_iter()
        .map(|h| h.join().unwrap_or(0))
        .sum();

    let total_elapsed = clock.now() - start_time;
    println!("--- Final Results ---");
    println!("Total Requests: {}", total_requests);
    println!("Requests/sec: {:.2}", total_requests as f64 / total_elapsed.as_secs_f64());
}
