use std::collections::HashMap;
use std::env;
use fory::Fory;
use reqwest::Method;
use crate::structs::ForyRequest;

/// Helper function to reduce boilerplate.
/// It returns the env var or the default value as a String.
pub fn get_env(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_) => default.to_string(),
    }
}

pub fn init_fory() -> anyhow::Result<Fory> {
    let mut fory = Fory::default().xlang(true).compatible(true);
    fory.register_by_namespace::<ForyRequest>("com.google.gemini", "fory_request")?;
    let _warm_up_fory = fory.serialize(&ForyRequest{
        label: "test".to_string(),
        absolute_url: "https://google.com".to_string(),
        host: "google.com".to_string(),
        port: 443,
        method: "GET".to_string(),
        path: "/".to_string(),
        valid_status_codes: vec![200],
        max_response_time_sla_micros: 1000,
        insecure: false,
        headers: HashMap::new(),
        query_params: HashMap::new(),
        body: "".to_string(),
    })?;
    for _ in 0..1000 {
        let _: ForyRequest = fory.deserialize(&_warm_up_fory)?;
    }
    Ok(fory)
}

pub fn reqwest_method(method_name: String) -> reqwest::Method {
    match(method_name) {
        ref m if m.eq_ignore_ascii_case("GET") => reqwest::Method::GET,
        ref m if m.eq_ignore_ascii_case("POST") => reqwest::Method::POST,
        ref m if m.eq_ignore_ascii_case("PUT") => reqwest::Method::PUT,
        ref m if m.eq_ignore_ascii_case("DELETE") => reqwest::Method::DELETE,
        ref m if m.eq_ignore_ascii_case("PATCH") => reqwest::Method::PATCH,
        ref m if m.eq_ignore_ascii_case("HEAD") => reqwest::Method::HEAD,
        ref m if m.eq_ignore_ascii_case("OPTIONS") => reqwest::Method::OPTIONS,
        ref m if m.eq_ignore_ascii_case("TRACE") => reqwest::Method::TRACE,
        ref m if m.eq_ignore_ascii_case("CONNECT") => reqwest::Method::CONNECT,
        _ => reqwest::Method::GET,
    }
}