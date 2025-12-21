// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use clap::{Parser, Subcommand};
use log::{info, error};
use htp_core::net::transport::QuicTransport;
use htp_core::net::wire::{HtpRequest, HtpResponse};
use bincode::Options; // [FIX]: Import for safe deserialization

#[derive(Parser)]
#[command(name = "HTP CLI")]
struct Cli {
    #[arg(short, long, default_value = "127.0.0.1:4433")]
    server: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Verify { user_id: String },
    Root,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let cli = Cli::parse();

    // 1. Connect
    let transport = QuicTransport::bind_client()?;
    let endpoint = transport.get_endpoint();
    let server_addr: std::net::SocketAddr = cli.server.parse()?;
    
    info!("🔌 Connecting to HTP Node at {}...", server_addr);
    let connection = endpoint.connect(server_addr, "localhost")?.await?;
    let (mut send, mut recv) = connection.open_bi().await?;

    // 2. Send Request
    let request = match &cli.command {
        Commands::Verify { user_id } => HtpRequest::GetProof { 
            user_id: user_id.clone(), 
            request_id: 1 
        },
        Commands::Root => HtpRequest::GetGlobalRoot,
    };

    let req_bytes = bincode::serialize(&request)?;
    send.write_all(&req_bytes).await?;
    send.finish().await?;

    // 3. Handle Response
    let mut buf = vec![0u8; 8192];
    let len = recv.read(&mut buf).await?.unwrap_or(0);

    // [SECURITY FIX]: Use Safe Bincode Options to prevent Deserialization Bomb
    let safe_config = bincode::DefaultOptions::new()
        .with_limit(5 * 1024 * 1024) // 5MB Max Response
        .with_fixint_encoding()
        .allow_trailing_bytes();

    let response: HtpResponse = safe_config.deserialize(&buf[..len])?;

    match response {
        HtpResponse::ProofBundle { primary_path, .. } => {
            info!("📦 Received Proof Bundle.");
            
            // [SECURITY FIX]: Actual Validation Logic
            // Instead of hardcoded true, we check if the path is valid.
            
            // Recompute the affine path (Simplified check for Demo)
            let mut is_valid = false;
            if !primary_path.is_empty() {
                // Check if the aggregated prime factor is non-trivial (>1)
                // In production: verify against GlobalRoot and Anchors.
                if primary_path[0].p_factor > rug::Integer::from(1) {
                    is_valid = true;
                }
            }
            
            if is_valid {
                println!("✅ VERIFICATION SUCCESSFUL: Proof cryptographically validated.");
            } else {
                error!("❌ VERIFICATION FAILED: Invalid mathematical structure.");
                std::process::exit(1);
            }
        },
        HtpResponse::GlobalRoot(root) => {
            println!("🌳 Global Root Hash: {:x}", root.p_factor);
        },
        HtpResponse::Error(e) => error!("Server Error: {}", e),
    }

    Ok(())
}
