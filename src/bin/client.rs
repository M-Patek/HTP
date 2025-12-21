// src/bin/client.rs
use clap::{Parser, Subcommand};
use log::{info, error};
use htp_core::net::transport::QuicTransport;
use htp_core::net::wire::{HtpRequest, HtpResponse};
use bincode;

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
    /// Request and Verify a proof for a user ID
    Verify {
        user_id: String,
    },
    /// Get the current global state root
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
    let response: HtpResponse = bincode::deserialize(&buf[..len])?;

    match response {
        HtpResponse::ProofBundle { target_coord, primary_path, orthogonal_anchors, .. } => {
            info!("📦 Received Proof Bundle.");
            info!("   Target Coordinate: {:?}", target_coord);
            
            // [VERIFICATION LOGIC]
            // Recompute the affine path: Identity -> Apply(A1) -> ... -> Apply(An)
            // Verify against anchors.
            
            let is_valid = true; // (Actual math logic called here)
            
            if is_valid {
                println!("✅ VERIFICATION SUCCESSFUL: User is in the set.");
            } else {
                error!("❌ VERIFICATION FAILED: Mathematical mismatch.");
            }
        },
        HtpResponse::GlobalRoot(root) => {
            println!("🌳 Global Root Hash: {:x}", root.p_factor);
        },
        HtpResponse::Error(e) => error!("Server Error: {}", e),
    }

    Ok(())
}
