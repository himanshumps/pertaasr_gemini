use reqwest::Client;

/// This is to create a defaukt reqwest client which will create a pool of vusers count
pub(crate) fn build_request_client(vusers: usize) -> Client{
    Client::builder()
        // Enforce using Rustls (implied by Cargo features, but good to know)
        .use_rustls_tls()
        // Force HTTP 1.1 (disables HTTP/2)
        .http1_only()
        // Accept invalid/self-signed certificates
        .danger_accept_invalid_certs(true)
        .pool_max_idle_per_host(vusers)
        .build()
        .unwrap()
}