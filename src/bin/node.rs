// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use clap::Parser;
use log::{info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::net::SocketAddr;

use htp_core::core::param::SystemParameters;
use htp_core::topology::tensor::HyperTensor;
use htp_core::net::transport::QuicTransport;
use htp_core::net::service::run_prover_service;

#[derive(Parser)]
#[command(name = "HTP Node")]
struct Cli {
    #[arg(short, long, default_value = "127.0.0.1:4433")]
    bind: String,

    // [SECURITY FIX]: Removed default_value. Seed is REQUIRED.
    /// Random Seed for Trustless Setup (MUST be unique)
    #[arg(short, long)]
    seed: String,

    #[arg(short, long, default_value_t = 4)]
    dim: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let cli = Cli::parse();

    // [SECURITY WARNING]
    if cli.seed == "block_891234" {
        warn!("⚠️  WARNING: You are using a known test seed. ASSETS ARE AT RISK.");
    }

    info!("🚀 Initializing HTP Node (Secure Edition)...");

    // 1. Trustless Setup
    // [SECURITY FIX]: Increased parameters to 2048-bit for production security.
    let start = std::time::Instant::now();
    let params = SystemParameters::from_random_seed(cli.seed.as_bytes(), 2048);
    info!("✅ Trustless Setup Complete (2048-bit) in {:?}", start.elapsed());

    // 2. Initialize Hyper-Tensor
    let tensor = Arc::new(RwLock::new(HyperTensor::new(
        cli.dim, 
        100,
        params.discriminant
    )));
    info!("🧊 Hyper-Tensor ({}D) Allocated.", cli.dim);

    // 3. Start Networking
    let addr: SocketAddr = cli.bind.parse()?;
    // Certificate generation handled inside bind_server
    let transport = QuicTransport::bind_server(addr, "cert.pem", "key.pem").await?;
    
    info!("📡 QUIC Transport listening on {}", addr);
    
    // 4. Run Service Loop
    run_prover_service(transport.get_endpoint().clone(), tensor).await;

    Ok(())
}
