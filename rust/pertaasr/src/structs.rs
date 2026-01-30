use std::collections::HashMap;
use fory::ForyObject;

#[derive(Debug, Clone, PartialEq, ForyObject)]
pub struct ForyRequest {
    pub label: String,
    pub absolute_url: String,
    pub host: String,
    pub port: i32,
    pub method: String,
    pub path: String,
    pub valid_status_codes: Vec<i32>,
    pub max_response_time_sla_micros: u64,
    pub insecure: bool,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: String,
}