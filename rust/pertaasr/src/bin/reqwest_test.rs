use mimalloc::MiMalloc;
use quanta::Clock;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Barrier;
use futures_util::StreamExt;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    let barrier = Arc::new(Barrier::new(20));
    let clock = Clock::new();
    let duration = Duration::from_secs(120);

    let start_time = clock.now();
    let mut handles = vec![];

    for _ in 0..20 {
        let b = Arc::clone(&barrier);
        let c = clock.clone();

        let handle = tokio::spawn(async move {
            let mut local_count: u64 = 0; // Thread-local counter
            let client = Client::builder()
                .tcp_nodelay(true)
                .http1_only()
                .pool_max_idle_per_host(1)
                .no_brotli()
                .no_gzip()
                .no_zstd()
                .no_deflate()
                .build()
                .unwrap();

            b.wait().await;

            while c.now() - start_time < duration {
                let url = "http://rust-server.himanshumps-1-dev.svc.cluster.local";
                if let Ok(resp) = client.get(url).send().await {
                    let mut body = resp.bytes_stream();
                    while let Some(item) = body.next().await {
                        if let Ok(chunk) = item {
                            std::mem::drop(chunk);
                        } else {
                            break;
                        }
                    }
                    local_count += 1; // Zero overhead increment
                }
            }
            local_count // Return count to main
        });
        handles.push(handle);
    }

    // Collect results from all handles
    let mut total_count: u64 = 0;
    for handle in handles {
        if let Ok(count) = handle.await {
            total_count += count;
        }
    }
    let total_elapsed = clock.now() - start_time;
    let rps = total_count as f64 / total_elapsed.as_secs_f64();

    println!("--- Benchmark Results ---");
    println!("Total Requests: {}", total_count);
    println!("Actual Duration: {:.2?}", total_elapsed);
    println!("Requests/sec:   {:.2}", rps);
}
