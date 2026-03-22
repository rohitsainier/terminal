use std::time::{SystemTime, UNIX_EPOCH};

pub fn epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn short_id(id: &str) -> String {
    if id.len() >= 8 {
        id[..8].to_string()
    } else {
        id.to_string()
    }
}
