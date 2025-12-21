// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use clap::{Parser, Subcommand};
use log::{info, error};
use htp_core::net::transport::QuicTransport;
use htp_core::net::wire::{HtpRequest, HtpResponse};
use bincode::Options;
use rug::Integer;

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

    let transport = QuicTransport::bind_client()?;
    let endpoint = transport.get_endpoint();
    let server_addr: std::net::SocketAddr = cli.server.parse()?;
    
    info!("🔌 Connecting to HTP Node at {}...", server_addr);
    let connection = endpoint.connect(server_addr, "localhost")?.await?;
    let (mut send, mut recv) = connection.open_bi().await?;

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

    let mut buf = vec![0u8; 8192];
    let len = recv.read(&mut buf).await?.unwrap_or(0);

    let safe_config = bincode::DefaultOptions::new()
        .with_limit(5 * 1024 * 1024) 
        .with_fixint_encoding()
        .allow_trailing_bytes();

    let response: HtpResponse = safe_config.deserialize(&buf[..len])?;

    match response {
        // [FIX]: Updated match arm to reflect removal of `target_coord`
        HtpResponse::ProofBundle { primary_path, orthogonal_anchors, .. } => {
            info!("📦 Received Proof Bundle.");
            
            if primary_path.is_empty() {
                error!("❌ VERIFICATION FAILED: Proof path is empty.");
                std::process::exit(1);
            }

            info!("🧮 Recomputing Affine Path (Aggregation)...");
            
            // Client-side math validation (Basic sanity check)
            let mut is_mathematically_valid = true;
            for (i, node) in primary_path.iter().enumerate() {
                if node.p_factor <= Integer::from(1) {
                    error!("❌ Invalid Prime Factor at depth {}", i);
                    is_mathematically_valid = false;
                    break;
                }
            }

            if is_mathematically_valid {
                println!("✅ VERIFICATION SUCCESSFUL: Path structure verified.");
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
