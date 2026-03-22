mod events;

use bharatlink_core::BharatLinkManager;
use clap::{Parser, Subcommand};
use events::CliEventSink;
use std::sync::Arc;

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
    /// Start the BharatLink node (foreground daemon)
    Start,

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

    /// Send a file to a peer
    Send {
        /// Peer's node ID
        peer_id: String,
        /// Path to the file
        path: String,
    },

    /// Send a text message to a peer
    Text {
        /// Peer's node ID
        peer_id: String,
        /// Message text
        message: String,
    },

    /// Show transfer history
    History,

    /// Show current settings
    Settings,
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
    // CLI uses ~/.config/bharatlink/ by default (separate from Flux Terminal)
    // The manager expects a parent dir and appends /bharatlink/ itself,
    // so we pass the parent to keep the config at the right path.
    let config_dir = get_config_dir(cli.config_dir.as_deref());

    match cli.command {
        Commands::Start => cmd_start(config_dir).await,
        Commands::Info => cmd_info(config_dir).await,
        Commands::Peers => cmd_peers(config_dir).await,
        Commands::Trust { peer_id, nickname } => cmd_trust(config_dir, peer_id, nickname).await,
        Commands::Untrust { peer_id } => cmd_untrust(config_dir, peer_id).await,
        Commands::Send { peer_id, path } => cmd_send(config_dir, peer_id, path).await,
        Commands::Text { peer_id, message } => cmd_text(config_dir, peer_id, message).await,
        Commands::History => cmd_history(config_dir),
        Commands::Settings => cmd_settings(config_dir),
    }
}

async fn cmd_start(config_dir: std::path::PathBuf) {
    eprintln!("🔗 Starting BharatLink node...");
    let mut manager = BharatLinkManager::new(config_dir);
    let events: Arc<dyn bharatlink_core::EventSink> = Arc::new(CliEventSink::new());

    match manager.start(events).await {
        Ok(info) => {
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
            println!("  Share your Node ID with peers to connect.");
            println!("  Press Ctrl+C to stop.\n");

            // Block until SIGINT
            tokio::signal::ctrl_c().await.ok();
            eprintln!("\n🛑 Shutting down...");
            if let Err(e) = manager.stop().await {
                eprintln!("Error during shutdown: {}", e);
            }
            eprintln!("Goodbye!");
        }
        Err(e) => {
            eprintln!("❌ Failed to start: {}", e);
            std::process::exit(1);
        }
    }
}

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
    let mut manager = BharatLinkManager::new(config_dir);
    let events: Arc<dyn bharatlink_core::EventSink> = Arc::new(CliEventSink::new());

    eprintln!("🔗 Starting node for file transfer...");
    if let Err(e) = manager.start(events).await {
        eprintln!("❌ Failed to start node: {}", e);
        std::process::exit(1);
    }

    // Wait for mDNS/DNS discovery to find the peer
    eprintln!("🔍 Discovering peer {}...", &peer_id[..8.min(peer_id.len())]);
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    eprintln!("📤 Sending {} to {}...", path, &peer_id[..8.min(peer_id.len())]);
    match manager.send_file(peer_id, path).await {
        Ok(id) => {
            eprintln!("✅ Transfer {} complete", &id[..8]);
        }
        Err(e) => {
            eprintln!("❌ Send failed: {}", e);
        }
    }

    let _ = manager.stop().await;
}

async fn cmd_text(config_dir: std::path::PathBuf, peer_id: String, message: String) {
    let mut manager = BharatLinkManager::new(config_dir);
    let events: Arc<dyn bharatlink_core::EventSink> = Arc::new(CliEventSink::new());

    eprintln!("🔗 Starting node for text transfer...");
    if let Err(e) = manager.start(events).await {
        eprintln!("❌ Failed to start node: {}", e);
        std::process::exit(1);
    }

    // Wait for mDNS/DNS discovery to find the peer
    eprintln!("🔍 Discovering peer {}...", &peer_id[..8.min(peer_id.len())]);
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    eprintln!("💬 Sending text to {}...", &peer_id[..8.min(peer_id.len())]);
    match manager.send_text(peer_id, message).await {
        Ok(id) => {
            eprintln!("✅ Message {} sent", &id[..8]);
        }
        Err(e) => {
            eprintln!("❌ Send failed: {}", e);
        }
    }

    let _ = manager.stop().await;
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
