mod events;

use bharatlink_core::BharatLinkManager;
use clap::{Parser, Subcommand};
use events::CliEventSink;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

#[derive(Parser)]
#[command(name = "bharatlink", version, about = "P2P file & text sharing from your terminal")]
struct Cli {
    /// Config directory (default: ~/.config/bharatlink)
    #[arg(long, global = true)]
    config_dir: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the BharatLink node (interactive daemon)
    Start,

    /// Wait for one incoming transfer, save it, and exit
    Receive {
        /// Save directory (default: current directory)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Show node info (ID, relay, local addresses)
    Info,

    /// List discovered and trusted peers
    Peers,

    /// Trust a peer
    Trust {
        /// Peer's node ID
        peer_id: String,
        /// Nickname for the peer
        nickname: String,
    },

    /// Untrust a peer
    Untrust {
        /// Peer's node ID
        peer_id: String,
    },

    /// Send a file or text to a peer
    Send {
        #[command(subcommand)]
        what: SendWhat,
    },

    /// Show transfer history
    History,

    /// Show current settings
    Settings,
}

#[derive(Subcommand)]
enum SendWhat {
    /// Send a file
    File {
        /// Peer's node ID
        peer_id: String,
        /// Path to the file
        path: String,
    },
    /// Send a text message
    Text {
        /// Peer's node ID
        peer_id: String,
        /// Message text
        message: String,
    },
}

fn get_config_dir(custom: Option<&str>) -> std::path::PathBuf {
    if let Some(dir) = custom {
        std::path::PathBuf::from(dir)
    } else {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("bharatlink")
    }
}

#[tokio::main]
async fn main() {
    // Only show logs if RUST_LOG is explicitly set; otherwise keep output clean
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("off")),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let config_dir = get_config_dir(cli.config_dir.as_deref());

    match cli.command {
        Commands::Start => cmd_start(config_dir).await,
        Commands::Receive { output } => cmd_receive(config_dir, output).await,
        Commands::Info => cmd_info(config_dir).await,
        Commands::Peers => cmd_peers(config_dir).await,
        Commands::Trust { peer_id, nickname } => cmd_trust(config_dir, peer_id, nickname).await,
        Commands::Untrust { peer_id } => cmd_untrust(config_dir, peer_id).await,
        Commands::Send { what } => match what {
            SendWhat::File { peer_id, path } => cmd_send(config_dir, peer_id, path).await,
            SendWhat::Text { peer_id, message } => cmd_text(config_dir, peer_id, message).await,
        },
        Commands::History => cmd_history(config_dir),
        Commands::Settings => cmd_settings(config_dir),
    }
}

// ── Start: interactive daemon ──────────────────────────────────────────

async fn cmd_start(config_dir: std::path::PathBuf) {
    eprintln!("🔗 Starting BharatLink node...");

    let manager = Arc::new(TokioMutex::new(BharatLinkManager::new(config_dir)));
    let events_sink = Arc::new(CliEventSink::new());
    let events: Arc<dyn bharatlink_core::EventSink> = events_sink.clone();

    // Enable auto-accept for CLI by default
    {
        let mut mgr = manager.lock().await;
        let mut settings = mgr.get_settings();
        settings.auto_accept_from_trusted = true;
        settings.auto_accept_text = true;
        let _ = mgr.update_settings(settings).await;
    }

    // Start the node
    let info = {
        let mut mgr = manager.lock().await;
        match mgr.start(events).await {
            Ok(info) => info,
            Err(e) => {
                eprintln!("❌ Failed to start: {}", e);
                std::process::exit(1);
            }
        }
    };

    println!();
    println!("  ✅ BharatLink Node Started");
    println!("  ─────────────────────────────────────────────");
    println!("  Node ID:  {}", info.node_id);
    println!("  Short ID: {}", info.node_id_short);
    if let Some(relay) = &info.relay_url {
        println!("  Relay:    {}", relay);
    }
    for addr in &info.local_addrs {
        println!("  Local:    {}", addr);
    }
    println!("  ─────────────────────────────────────────────");
    println!();
    println!("  Auto-accept from trusted peers: ON");
    println!("  Files saved to: {}", {
        let mgr = manager.lock().await;
        mgr.get_settings().download_dir
    });
    println!();
    println!("  Type 'help' for commands. Examples:");
    println!("    send file <peer_id> /path/to/file.txt");
    println!("    send text <peer_id> hello world");
    println!();

    // Run interactive input loop + ctrl_c handler concurrently
    let manager_for_input = manager.clone();
    let pending_for_input = events_sink.pending_requests.clone();

    tokio::select! {
        _ = interactive_loop(manager_for_input, pending_for_input) => {}
        _ = tokio::signal::ctrl_c() => {
            eprintln!("\n🛑 Shutting down...");
        }
    }

    // Shutdown
    let mut mgr = manager.lock().await;
    if let Err(e) = mgr.stop().await {
        eprintln!("Error during shutdown: {}", e);
    }
    eprintln!("Goodbye!");
}

async fn interactive_loop(
    manager: Arc<TokioMutex<BharatLinkManager>>,
    pending: Arc<std::sync::Mutex<Vec<bharatlink_core::TransferRequest>>>,
) {
    let stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut lines = tokio::io::AsyncBufReadExt::lines(stdin);

    loop {
        eprint!("bharatlink> ");
        // Flush stderr prompt
        use std::io::Write;
        let _ = std::io::stderr().flush();

        let line = match lines.next_line().await {
            Ok(Some(line)) => line.trim().to_string(),
            Ok(None) => break, // EOF
            Err(_) => break,
        };

        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let cmd = parts[0].to_lowercase();
        let arg = parts.get(1).map(|s| s.trim().to_string());

        match cmd.as_str() {
            "quit" | "exit" | "q" => break,

            "accept" | "a" => {
                if let Some(id_prefix) = arg {
                    let full_id = resolve_request_id(&pending, &id_prefix);
                    match full_id {
                        Some(id) => {
                            let mut mgr = manager.lock().await;
                            match mgr.accept_transfer(id.clone()).await {
                                Ok(()) => eprintln!("  ✅ Accepted {}", &id[..8.min(id.len())]),
                                Err(e) => eprintln!("  ❌ {}", e),
                            }
                        }
                        None => eprintln!("  ❌ No pending request matching '{}'", id_prefix),
                    }
                } else {
                    eprintln!("  Usage: accept <id>");
                }
            }

            "reject" | "r" => {
                if let Some(id_prefix) = arg {
                    let full_id = resolve_request_id(&pending, &id_prefix);
                    match full_id {
                        Some(id) => {
                            let mut mgr = manager.lock().await;
                            match mgr.reject_transfer(id.clone()).await {
                                Ok(()) => {
                                    eprintln!("  ✅ Rejected {}", &id[..8.min(id.len())]);
                                    if let Ok(mut p) = pending.lock() {
                                        p.retain(|r| r.id != id);
                                    }
                                }
                                Err(e) => eprintln!("  ❌ {}", e),
                            }
                        }
                        None => eprintln!("  ❌ No pending request matching '{}'", id_prefix),
                    }
                } else {
                    eprintln!("  Usage: reject <id>");
                }
            }

            "list" | "l" | "pending" => {
                let reqs = if let Ok(p) = pending.lock() { p.clone() } else { vec![] };
                if reqs.is_empty() {
                    eprintln!("  No pending requests.");
                } else {
                    eprintln!("  Pending requests:");
                    for r in &reqs {
                        let from = r.from_nickname.as_deref()
                            .unwrap_or(&r.from_peer[..8.min(r.from_peer.len())]);
                        let what = r.filename.as_deref().unwrap_or("text");
                        let short_id = &r.id[..8.min(r.id.len())];
                        eprintln!("    [{}] {} from {}", short_id, what, from);
                    }
                }
            }

            "peers" | "p" => {
                let mgr = manager.lock().await;
                let peers = mgr.get_peers();
                if peers.is_empty() {
                    eprintln!("  No peers.");
                } else {
                    for p in &peers {
                        let status = if p.is_connected { "●" } else { "○" };
                        let name = p.nickname.as_deref().unwrap_or("-");
                        eprintln!("  {} {} ({})", status, name, p.node_id_short);
                    }
                }
            }

            "history" | "h" => {
                let mgr = manager.lock().await;
                let history = mgr.get_history();
                if history.is_empty() {
                    eprintln!("  No transfer history.");
                } else {
                    for e in history.iter().rev().take(10) {
                        let dir = if e.direction == "send" { "↑" } else { "↓" };
                        let name = e.filename.as_deref()
                            .or(e.text_content.as_deref().map(|t| if t.len() > 30 { &t[..30] } else { t }))
                            .unwrap_or("-");
                        let peer = e.peer_nickname.as_deref()
                            .unwrap_or(&e.peer_id[..8.min(e.peer_id.len())]);
                        eprintln!("  {} {} {} ({})", dir, name, e.status, peer);
                    }
                }
            }

            "send" => {
                if let Some(args) = arg {
                    let parts: Vec<&str> = args.splitn(3, ' ').collect();
                    match parts.get(0).map(|s| s.to_lowercase()).as_deref() {
                        Some("file") if parts.len() == 3 => {
                            let peer_id = parts[1].to_string();
                            let file_path = strip_quotes(parts[2]);
                            let short = &peer_id[..8.min(peer_id.len())];
                            eprintln!("  📤 Sending {} to {}...", file_path, short);
                            let result = send_with_retry(&manager, |mgr| {
                                let pid = peer_id.clone();
                                let fp = file_path.clone();
                                Box::pin(async move { mgr.send_file(pid, fp).await })
                            }).await;
                            match result {
                                Ok(id) => eprintln!("  ✅ Transfer {} complete", &id[..8]),
                                Err(e) => eprintln!("  ❌ {}", e),
                            }
                        }
                        Some("text") if parts.len() >= 3 => {
                            let peer_id = parts[1].to_string();
                            let after_text_and_peer = args.splitn(3, ' ').nth(2).unwrap_or("");
                            let message = after_text_and_peer.to_string();
                            let short = &peer_id[..8.min(peer_id.len())];
                            eprintln!("  💬 Sending text to {}...", short);
                            let result = send_with_retry(&manager, |mgr| {
                                let pid = peer_id.clone();
                                let msg = message.clone();
                                Box::pin(async move { mgr.send_text(pid, msg).await })
                            }).await;
                            match result {
                                Ok(id) => eprintln!("  ✅ Message {} sent", &id[..8]),
                                Err(e) => eprintln!("  ❌ {}", e),
                            }
                        }
                        _ => {
                            eprintln!("  Usage:");
                            eprintln!("    send file <peer_id> <file_path>");
                            eprintln!("    send text <peer_id> <message>");
                        }
                    }
                } else {
                    eprintln!("  Usage:");
                    eprintln!("    send file <peer_id> <file_path>");
                    eprintln!("    send text <peer_id> <message>");
                }
            }

            "help" | "?" => {
                eprintln!("  Commands:");
                eprintln!("    send file <peer> <path>  — Send a file");
                eprintln!("    send text <peer> <msg>   — Send a text message");
                eprintln!("    accept <id>              — Accept a pending transfer");
                eprintln!("    reject <id>              — Reject a pending transfer");
                eprintln!("    list                     — Show pending requests");
                eprintln!("    peers                    — Show connected peers");
                eprintln!("    history                  — Show recent transfers");
                eprintln!("    quit                     — Stop the node and exit");
            }

            _ => {
                eprintln!("  Unknown command: '{}'. Type 'help' for commands.", cmd);
            }
        }
    }
}

/// Retry a send operation with backoff — handles cross-network relay resolution delays
async fn send_with_retry<F>(
    manager: &Arc<TokioMutex<BharatLinkManager>>,
    make_future: F,
) -> Result<String, String>
where
    F: Fn(&mut BharatLinkManager) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + '_>>,
{
    let delays = [0, 3, 5]; // seconds: try immediately, wait 3s, wait 5s
    let mut last_err = String::new();

    for (attempt, delay) in delays.iter().enumerate() {
        if *delay > 0 {
            eprintln!("  ⏳ Retrying in {}s (attempt {}/{})...", delay, attempt + 1, delays.len());
            tokio::time::sleep(std::time::Duration::from_secs(*delay)).await;
        }

        let mut mgr = manager.lock().await;
        match make_future(&mut mgr).await {
            Ok(id) => return Ok(id),
            Err(e) => {
                if e.contains("No addressing information") || e.contains("Failed to connect") {
                    last_err = e;
                    continue; // Retry — peer might not be discovered yet
                }
                return Err(e); // Non-retryable error
            }
        }
    }

    Err(format!("{}\n  Hint: For cross-network transfers, make sure the peer's node is running and both sides have trusted each other.", last_err))
}

/// Resolve a short ID prefix to the full request ID
fn resolve_request_id(
    pending: &std::sync::Mutex<Vec<bharatlink_core::TransferRequest>>,
    prefix: &str,
) -> Option<String> {
    if let Ok(reqs) = pending.lock() {
        // Exact match first
        if let Some(r) = reqs.iter().find(|r| r.id == prefix) {
            return Some(r.id.clone());
        }
        // Prefix match
        if let Some(r) = reqs.iter().find(|r| r.id.starts_with(prefix)) {
            return Some(r.id.clone());
        }
    }
    None
}

/// Strip surrounding single or double quotes from a string
fn strip_quotes(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('\'') && s.ends_with('\'')) || (s.starts_with('"') && s.ends_with('"')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

// ── Receive: one-shot mode ─────────────────────────────────────────────

async fn cmd_receive(config_dir: std::path::PathBuf, output_dir: Option<String>) {
    let mut manager = BharatLinkManager::new(config_dir);
    let events_sink = Arc::new(CliEventSink::new());
    let events: Arc<dyn bharatlink_core::EventSink> = events_sink.clone();
    let complete_flag = events_sink.transfer_complete.clone();

    // Set download directory to output or current dir
    let save_dir = output_dir.unwrap_or_else(|| {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .to_string_lossy()
            .to_string()
    });

    // Enable auto-accept for everything
    let mut settings = manager.get_settings();
    settings.auto_accept_from_trusted = true;
    settings.auto_accept_text = true;
    settings.download_dir = save_dir.clone();
    let _ = manager.update_settings(settings).await;

    eprintln!("🔗 Starting node...");
    match manager.start(events).await {
        Ok(info) => {
            println!();
            println!("  📥 Waiting to receive...");
            println!("  Node ID: {}", info.node_id);
            println!("  Saving to: {}", save_dir);
            println!();
            println!("  Send a file from another device to this Node ID.");
            println!("  Press Ctrl+C to cancel.\n");
        }
        Err(e) => {
            eprintln!("❌ Failed to start: {}", e);
            std::process::exit(1);
        }
    }

    // Wait for a transfer to complete or ctrl_c
    tokio::select! {
        _ = async {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                if let Ok(flag) = complete_flag.lock() {
                    if *flag {
                        break;
                    }
                }
            }
        } => {
            eprintln!("\n  ✅ Transfer complete!");
        }
        _ = tokio::signal::ctrl_c() => {
            eprintln!("\n  Cancelled.");
        }
    }

    let _ = manager.stop().await;
}

// ── Simple commands (no daemon needed) ─────────────────────────────────

async fn cmd_info(config_dir: std::path::PathBuf) {
    let manager = BharatLinkManager::new(config_dir);
    match manager.get_node_info() {
        Ok(info) => {
            if info.is_running {
                println!("Node ID:  {}", info.node_id);
                println!("Status:   Running");
                if let Some(relay) = info.relay_url {
                    println!("Relay:    {}", relay);
                }
                for addr in &info.local_addrs {
                    println!("Local:    {}", addr);
                }
            } else {
                println!("Status: Not running");
                println!("Use 'bharatlink start' to start the node");
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn cmd_peers(config_dir: std::path::PathBuf) {
    let manager = BharatLinkManager::new(config_dir);
    let peers = manager.get_peers();
    if peers.is_empty() {
        println!("No peers found. Use 'bharatlink trust <peer_id> <nickname>' to add a peer.");
        return;
    }
    println!("{:<10} {:<20} {:<10} {}", "STATUS", "NAME", "TRUSTED", "NODE ID");
    println!("{}", "-".repeat(70));
    for p in peers {
        let status = if p.is_connected { "● online" } else { "○ offline" };
        let name = p.nickname.as_deref().unwrap_or("-");
        let trusted = if p.is_trusted { "yes" } else { "no" };
        println!("{:<10} {:<20} {:<10} {}", status, name, trusted, p.node_id_short);
    }
}

async fn cmd_trust(config_dir: std::path::PathBuf, peer_id: String, nickname: String) {
    let mut manager = BharatLinkManager::new(config_dir);
    match manager.trust_peer(peer_id.clone(), nickname.clone()).await {
        Ok(()) => println!("✅ Trusted peer {} as '{}'", &peer_id[..8.min(peer_id.len())], nickname),
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn cmd_untrust(config_dir: std::path::PathBuf, peer_id: String) {
    let mut manager = BharatLinkManager::new(config_dir);
    match manager.untrust_peer(peer_id.clone()).await {
        Ok(()) => println!("✅ Untrusted peer {}", &peer_id[..8.min(peer_id.len())]),
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn cmd_send(config_dir: std::path::PathBuf, peer_id: String, path: String) {
    let manager = Arc::new(TokioMutex::new(BharatLinkManager::new(config_dir)));
    let events: Arc<dyn bharatlink_core::EventSink> = Arc::new(CliEventSink::new());

    eprintln!("🔗 Starting node for file transfer...");
    {
        let mut mgr = manager.lock().await;
        if let Err(e) = mgr.start(events).await {
            eprintln!("❌ Failed to start node: {}", e);
            std::process::exit(1);
        }
    }

    eprintln!("📤 Sending {} to {}...", path, &peer_id[..8.min(peer_id.len())]);
    let result = send_with_retry(&manager, |mgr| {
        let pid = peer_id.clone();
        let p = path.clone();
        Box::pin(async move { mgr.send_file(pid, p).await })
    }).await;

    match result {
        Ok(id) => eprintln!("✅ Transfer {} complete", &id[..8]),
        Err(e) => eprintln!("❌ Send failed: {}", e),
    }

    let mut mgr = manager.lock().await;
    let _ = mgr.stop().await;
}

async fn cmd_text(config_dir: std::path::PathBuf, peer_id: String, message: String) {
    let manager = Arc::new(TokioMutex::new(BharatLinkManager::new(config_dir)));
    let events: Arc<dyn bharatlink_core::EventSink> = Arc::new(CliEventSink::new());

    eprintln!("🔗 Starting node for text transfer...");
    {
        let mut mgr = manager.lock().await;
        if let Err(e) = mgr.start(events).await {
            eprintln!("❌ Failed to start node: {}", e);
            std::process::exit(1);
        }
    }

    eprintln!("💬 Sending text to {}...", &peer_id[..8.min(peer_id.len())]);
    let result = send_with_retry(&manager, |mgr| {
        let pid = peer_id.clone();
        let msg = message.clone();
        Box::pin(async move { mgr.send_text(pid, msg).await })
    }).await;

    match result {
        Ok(id) => eprintln!("✅ Message {} sent", &id[..8]),
        Err(e) => eprintln!("❌ Send failed: {}", e),
    }

    let mut mgr = manager.lock().await;
    let _ = mgr.stop().await;
}

fn cmd_history(config_dir: std::path::PathBuf) {
    let manager = BharatLinkManager::new(config_dir);
    let history = manager.get_history();
    if history.is_empty() {
        println!("No transfer history.");
        return;
    }
    println!("{:<5} {:<10} {:<30} {:<10} {}", "DIR", "TYPE", "NAME", "STATUS", "PEER");
    println!("{}", "-".repeat(80));
    for e in history.iter().rev().take(20) {
        let dir = if e.direction == "send" { "↑" } else { "↓" };
        let name = e.filename.as_deref()
            .or(e.text_content.as_deref().map(|t| if t.len() > 28 { &t[..28] } else { t }))
            .unwrap_or("-");
        let peer = e.peer_nickname.as_deref().unwrap_or(&e.peer_id[..8.min(e.peer_id.len())]);
        println!("{:<5} {:<10} {:<30} {:<10} {}", dir, e.transfer_type, name, e.status, peer);
    }
}

fn cmd_settings(config_dir: std::path::PathBuf) {
    let manager = BharatLinkManager::new(config_dir);
    let settings = manager.get_settings();
    println!("{}", serde_json::to_string_pretty(&settings).unwrap());
}
