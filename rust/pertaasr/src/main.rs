use bytes::Bytes;
use hickory_resolver::TokioAsyncResolver;
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use quanta::Clock;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Barrier;
use tower_service::Service;
use hyper_util::client::legacy::connect::dns::Name;

// Removed MiMalloc to stop "Illegal Instruction"
// #[global_allocator]
// static GLOBAL: MiMalloc = MiMalloc;
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
#[derive(Clone)]
struct HickoryResolver(TokioAsyncResolver);

impl Service<Name> for HickoryResolver {
    type Response = std::vec::IntoIter<SocketAddr>;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn call(&mut self, name: Name) -> Self::Future {
        let resolver = self.0.clone();
        Box::pin(async move {
            let lookup = resolver.lookup_ip(name.as_str()).await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            let addrs: Vec<SocketAddr> = lookup.into_iter().map(|ip| SocketAddr::new(ip, 0)).collect();
            Ok(addrs.into_iter())
        })
    }
}

fn main() {
    println!("[Main] Starting benchmark setup...");
    let core_ids = core_affinity::get_core_ids().unwrap_or_else(|| vec![]);
    let num_cores = core_ids.len().max(1);
    let total_conns = 20;

    let barrier = Arc::new(Barrier::new(total_conns));
    let clock = Clock::new();
    let duration = Duration::from_secs(120);
    let start_time = clock.now();
    let mut thread_handles = vec![];
    let mut tasks_spawned = 0;

    // Use a simpler core distribution
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
                core_affinity::set_for_current(id);
            }

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let (config, mut opts) = hickory_resolver::system_conf::read_system_conf().expect("resolv.conf");
                opts.ndots = 5;
                let resolver = TokioAsyncResolver::tokio(config, opts);
                let mut connector = HttpConnector::new_with_resolver(HickoryResolver(resolver));
                connector.set_nodelay(true);
                connector.enforce_http(true);

                let client = Arc::new(Client::builder(TokioExecutor::new())
                    .pool_max_idle_per_host(total_conns)
                    .pool_idle_timeout(None)
                    .build::<_, Empty<Bytes>>(connector));

                let mut conn_handles = vec![];
                for _ in 0..tasks_on_this_thread {
                    let b = Arc::clone(&b);
                    let c = c.clone();
                    let client = Arc::clone(&client);

                    conn_handles.push(tokio::spawn(async move {
                        let url: hyper::Uri = "http://rust-server.himanshumps-1-dev.svc.cluster.local:8080/".parse().unwrap();
                        let mut count = 0u64;

                        b.wait().await;

                        while c.now() - start_time < duration {
                            let req = Request::builder()
                                .uri(&url)
                                .header("Host", "rust-server.himanshumps-1-dev.svc.cluster.local")
                                .body(Empty::<Bytes>::new())
                                .unwrap();

                            if let Ok(resp) = client.request(req).await {
                                let mut body = resp.into_body();
                                while let Some(Ok(frame)) = body.frame().await {
                                    std::mem::drop(frame);
                                }
                                count += 1;
                            }
                        }
                        count
                    }));
                }
                let mut total = 0;
                for h in conn_handles { total += h.await.unwrap_or(0); }
                total
            })
        }));
    }

    println!("[Main] Benchmark running for 120s...");
    let total_requests: u64 = thread_handles.into_iter().map(|h| h.join().unwrap()).sum();
    let total_elapsed = clock.now() - start_time;

    println!("\n--- Benchmark Results ---");
    println!("Total Requests: {}", total_requests);
    println!("Requests/sec:   {:.2}", total_requests as f64 / total_elapsed.as_secs_f64());
}
