use bytes::Bytes;
use hickory_resolver::TokioAsyncResolver;
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use mimalloc::MiMalloc;
use quanta::Clock;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Barrier;
use tower_service::Service;
use hyper_util::client::legacy::connect::dns::Name;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// --- DNS RESOLVER SHIM ---
// Bridge Hickory (IpAddr) to Hyper (SocketAddr)
#[derive(Clone)]
struct HickoryResolver(TokioAsyncResolver);

impl Service<Name> for HickoryResolver {
    type Response = std::vec::IntoIter<SocketAddr>;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, name: Name) -> Self::Future {
        let resolver = self.0.clone();
        Box::pin(async move {
            let lookup = resolver
                .lookup_ip(name.as_str())
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            // Map to SocketAddr with port 0 (Hyper fills actual port later)
            let addrs: Vec<SocketAddr> = lookup
                .into_iter()
                .map(|ip| SocketAddr::new(ip, 0))
                .collect();
            Ok(addrs.into_iter())
        })
    }
}

fn main() {
    println!("Starting the test");
    let core_ids = core_affinity::get_core_ids().unwrap_or_default();
    let num_cores = core_ids.len().max(1);
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
            println!("tokio::runtime::Builder::new_current_thread");
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                println!("hickory_resolver::system_conf::read_system_conf");
                // Read system conf properly for .svc.cluster.local support
                let (config, mut opts) = hickory_resolver::system_conf::read_system_conf()
                    .expect("Failed to read resolv.conf");

                // Kubernetes typically uses ndots:5
                opts.ndots = 5;
                println!("TokioAsyncResolver::tokio");
                let resolver = TokioAsyncResolver::tokio(config, opts);
                println!("After TokioAsyncResolver::tokio");
                let mut connector = HttpConnector::new_with_resolver(HickoryResolver(resolver));
                connector.set_nodelay(true);
                connector.enforce_http(true);
                println!("Creating client");
                let client = Arc::new(
                    Client::builder(TokioExecutor::new())
                        .pool_max_idle_per_host(total_conns)
                        .pool_idle_timeout(None)
                        .build::<_, Empty<Bytes>>(connector),
                );
                println!("After creating client");
                let mut conn_handles = vec![];

                for _ in 0..conns_per_core {
                    let b = Arc::clone(&b);
                    let c = c.clone();
                    let client = Arc::clone(&client);

                    conn_handles.push(tokio::spawn(async move {
                        let url: hyper::Uri = "http://rust-server.himanshumps-1-dev.svc.cluster.local:8080/".parse().unwrap();
                        let mut local_count = 0u64;

                        b.wait().await;

                        while c.now() - start_time < duration {
                            println!("Creating request");
                            let req = Request::builder()
                                .uri(&url)
                                .header("Host", "rust-server.himanshumps-1-dev.svc.cluster.local")
                                .body(Empty::<Bytes>::new())
                                .unwrap();
                            println!("Sending request");
                            if let Ok(resp) = client.request(req).await {
                                println!("Response code: {}", resp.status().as_u16());
                                let mut body = resp.into_body();
                                while let Some(frame_res) = body.frame().await {
                                    if let Ok(frame) = frame_res {
                                        std::mem::drop(frame);
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

    let total_requests: u64 = thread_handles.into_iter().map(|h| h.join().unwrap()).sum();
    let total_elapsed = clock.now() - start_time;

    println!("--- Benchmark Results ---");
    println!("Total Requests: {}", total_requests);
    println!("Requests/sec:   {:.2}", total_requests as f64 / total_elapsed.as_secs_f64());
}
