use std::env;

pub fn coordinator_uri() -> String {
    match env::var("COORDINATOR_URI") {
        Ok(addr) => addr,
        Err(_) => "http://localhost:8081".to_string(),
    }
}