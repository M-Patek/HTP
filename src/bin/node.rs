// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use clap::Parser;
use log::{info, warn, error};
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

    #[arg(short, long)]
    seed: String,

    #[arg(short, long, default_value_t = 4)]
    dim: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let cli = Cli::parse();

    // [SECURITY FIX]: Enforce Seed Strength
    if cli.seed.len() < 32 {
        error!("❌ CRITICAL ERROR: Seed is too short! Must be >32 chars.");
        std::process::exit(1);
    }
    if cli.seed == "block_891234" {
        warn!("⚠️  WARNING: Using known test seed.");
    }

    info!("🚀 Initializing HTP Node (Secure Edition)...");

    let start = std::time::Instant::now();
    let params = SystemParameters::from_random_seed(cli.seed.as_bytes(), 2048);
    info!("✅ Trustless Setup Complete in {:?}", start.elapsed());

    let tensor = Arc::new(RwLock::new(HyperTensor::new(
        cli.dim, 
        100,
        params.discriminant
    )));
    info!("🧊 Hyper-Tensor ({}D) Allocated.", cli.dim);

    let addr: SocketAddr = cli.bind.parse()?;
    // [FIX]: Cert loading with fallback
    let transport = QuicTransport::bind_server(addr, "cert.pem", "key.pem").await?;
    
    info!("📡 QUIC Transport listening on {}", addr);
    run_prover_service(transport.get_endpoint().clone(), tensor).await;

    Ok(())
}
