use std::env;

/// Helper function to reduce boilerplate.
/// It returns the env var or the default value as a String.
pub fn get_env(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_) => default.to_string(),
    }
}