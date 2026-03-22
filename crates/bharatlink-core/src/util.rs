use std::time::{SystemTime, UNIX_EPOCH};

use iroh::{Endpoint, PublicKey};

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

// ─── Connection helper ───────────────────────────────────────────────────────

/// Connect to a peer with retry — handles cross-network relay/DNS resolution delays.
/// Tries up to 4 times with increasing backoff. Per-attempt timeout is 20s to allow
/// for relay-assisted QUIC handshakes which can be slow on the first attempt.
pub(crate) async fn connect_with_retry(
    endpoint: &Endpoint,
    peer_key: PublicKey,
    alpn: &[u8],
) -> Result<iroh::endpoint::Connection, String> {
    // Delays before each attempt (seconds): immediate, then 5s, 10s, 20s
    let delays_secs: &[u64] = &[0, 5, 10, 20];
    const TIMEOUT_SECS: u64 = 20;
    let mut last_err = String::new();

    for (attempt, delay) in delays_secs.iter().enumerate() {
        if *delay > 0 {
            tracing::info!(
                "[BharatLink] connect_with_retry: attempt {}/{}, backing off {}s...",
                attempt + 1,
                delays_secs.len(),
                delay
            );
            tokio::time::sleep(std::time::Duration::from_secs(*delay)).await;
        }

        match tokio::time::timeout(
            std::time::Duration::from_secs(TIMEOUT_SECS),
            endpoint.connect(peer_key, alpn),
        )
        .await
        {
            Ok(Ok(conn)) => {
                tracing::info!(
                    "[BharatLink] connect_with_retry: connected on attempt {}",
                    attempt + 1
                );
                return Ok(conn);
            }
            Ok(Err(e)) => {
                let err = format!("{}", e);
                tracing::warn!(
                    "[BharatLink] connect_with_retry: attempt {} failed: {}",
                    attempt + 1,
                    err
                );
                last_err = err.clone();
                // Retry on any address-resolution, relay, or transient network error
                let is_retryable = err.contains("No addressing information")
                    || err.contains("relay")
                    || err.contains("timed out")
                    || err.contains("timeout")
                    || err.contains("connection reset")
                    || err.contains("unreachable")
                    || err.contains("refused");
                if is_retryable {
                    continue;
                }
                // Non-retryable error (e.g. bad ALPN, auth failure)
                return Err(err);
            }
            Err(_) => {
                last_err = format!("Connection timed out after {}s", TIMEOUT_SECS);
                tracing::warn!(
                    "[BharatLink] connect_with_retry: attempt {} timed out",
                    attempt + 1
                );
                // Always retry on timeout
            }
        }
    }

    Err(last_err)
}
