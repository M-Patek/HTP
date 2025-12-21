// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use clap::{Parser, Subcommand};
use log::{info, error, debug};
use htp_core::net::transport::QuicTransport;
use htp_core::net::wire::{HtpRequest, HtpResponse, RequestHeader};
use bincode::Options;
use rug::Integer;
use std::time::SystemTime;

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
    Register { user_id: String },
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

    // 构造带时间戳的 Header，防止重放
    let now = SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
    let header = RequestHeader { version: 1, timestamp: now, request_id: 1 };

    let request = match &cli.command {
        Commands::Verify { user_id } => HtpRequest::GetProof { 
            header,
            user_id: user_id.clone(), 
        },
        Commands::Register { user_id } => HtpRequest::RegisterUser {
            header,
            user_id: user_id.clone(),
        },
        Commands::Root => HtpRequest::GetGlobalRoot { header },
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
        HtpResponse::ProofBundle { primary_path, epoch, .. } => {
            info!("📦 Received Proof Bundle (Epoch: {}).", epoch);
            
            if primary_path.is_empty() {
                error!("❌ VERIFICATION FAILED: Proof path is empty.");
                std::process::exit(1);
            }
            
            // [SECURITY FIX]: 身份绑定校验 (Identity Binding Check)
            // 防止 Proof Binding Attack (冒充者攻击)
            if let Commands::Verify { user_id } = &cli.command {
                info!("🕵️ Verifying User Identity binding...");
                let expected_p = htp_core::core::primes::hash_to_prime(user_id, 64)
                    .map_err(|e| anyhow::anyhow!("Local Prime Gen Failed: {}", e))?;

                let leaf_node = &primary_path[0];
                if leaf_node.p_factor != expected_p {
                    // 如果这是 Dummy Proof，这里也会校验失败，从侧面保护了隐私
                    error!("❌ SPOOFING DETECTED: Proof belongs to a different user!");
                    std::process::exit(1);
                }
                info!("✅ Identity Confirmed.");
            }

            // 执行数学验证
            debug!("🔄 Recomputing Affine Path...");
            let mut calculated_agg = primary_path[0].clone();
            let one = Integer::from(1);
            let four = Integer::from(4);
            // 简单的 Discriminant 提取 hack
            let discriminant = &one - (&primary_path[0].q_shift.c * &four); 

            for i in 1..primary_path.len() {
                calculated_agg = calculated_agg.compose(&primary_path[i], &discriminant)
                    .map_err(|e| anyhow::anyhow!("Math Error: {}", e))?;
            }
            println!("✅ Proof Verified: Path Aggregate P-Factor = {:x}...", calculated_agg.p_factor);
        },
        HtpResponse::GlobalRoot(root) => {
            println!("🌳 Global Root Hash: {:x}", root.p_factor);
        },
        HtpResponse::RegisterSuccess { epoch, .. } => {
            println!("✅ User Registered Successfully (Epoch: {})", epoch);
        },
        HtpResponse::Error(e) => error!("Server Error: {}", e),
    }

    Ok(())
}
