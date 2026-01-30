use std::sync::OnceLock;

pub static CONNECTION_COUNT: OnceLock<usize> = OnceLock::new();
pub static RUN_DURATION: OnceLock<u64> = OnceLock::new();

pub static JAVA_HOME: OnceLock<String> = OnceLock::new();