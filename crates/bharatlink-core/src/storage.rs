use crate::types::*;

use iroh::SecretKey;
use std::collections::HashMap;
use std::path::Path;

/// Load settings, trusted peers, and transfer history from disk
pub(crate) fn load_state(
    config_dir: &Path,
) -> (BharatLinkSettings, HashMap<String, String>, Vec<TransferHistoryEntry>) {
    let settings = {
        let path = config_dir.join("settings.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    };

    let trusted_peers = {
        let path = config_dir.join("trusted_peers.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    };

    let history = {
        let path = config_dir.join("transfer_history.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    };

    (settings, trusted_peers, history)
}

/// Save settings, trusted peers, and transfer history to disk
pub(crate) fn save_state(
    config_dir: &Path,
    settings: &BharatLinkSettings,
    trusted_peers: &tokio::sync::Mutex<HashMap<String, String>>,
    transfer_history: &tokio::sync::Mutex<Vec<TransferHistoryEntry>>,
) -> Result<(), String> {
    std::fs::create_dir_all(config_dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;

    let data = serde_json::to_string_pretty(settings).map_err(|e| format!("{}", e))?;
    std::fs::write(config_dir.join("settings.json"), data)
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    // Save trusted peers from shared state — use try_lock to avoid async in sync context
    if let Ok(trusted) = trusted_peers.try_lock() {
        let data = serde_json::to_string_pretty(&*trusted).map_err(|e| format!("{}", e))?;
        std::fs::write(config_dir.join("trusted_peers.json"), data)
            .map_err(|e| format!("Failed to save trusted peers: {}", e))?;
    }

    // Save history from shared state
    let history: Vec<_> = if let Ok(h) = transfer_history.try_lock() {
        h.iter().rev().take(MAX_HISTORY).rev().cloned().collect()
    } else {
        Vec::new()
    };
    let data = serde_json::to_string_pretty(&history).map_err(|e| format!("{}", e))?;
    std::fs::write(config_dir.join("transfer_history.json"), data)
        .map_err(|e| format!("Failed to save history: {}", e))?;

    Ok(())
}

/// Load or create a persistent Ed25519 secret key
pub(crate) fn load_or_create_secret_key(config_dir: &Path) -> Result<SecretKey, String> {
    std::fs::create_dir_all(config_dir)
        .map_err(|e| format!("Failed to create config dir: {}", e))?;

    let key_path = config_dir.join("secret.key");
    if key_path.exists() {
        let bytes = std::fs::read(&key_path)
            .map_err(|e| format!("Failed to read secret key: {}", e))?;
        if bytes.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            return Ok(SecretKey::from_bytes(&arr));
        }
    }

    let key = SecretKey::generate(&mut rand::rng());
    std::fs::write(&key_path, key.to_bytes())
        .map_err(|e| format!("Failed to save secret key: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        let _ = std::fs::set_permissions(&key_path, perms);
    }

    Ok(key)
}
