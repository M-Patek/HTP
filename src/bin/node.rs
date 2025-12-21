// src/bin/node.rs
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
#[command(version = "1.0")]
#[command(about = "Hyper-Tensor Protocol - Prover Node", long_about = None)]
struct Cli {
    /// Listening Address
    #[arg(short, long, default_value = "127.0.0.1:4433")]
    bind: String,

    /// Random Seed for Trustless Setup (Simulated Beacon)
    #[arg(short, long, default_value = "block_891234")]
    seed: String,

    /// Tensor Dimension (d)
    #[arg(short, long, default_value_t = 4)]
    dim: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let cli = Cli::parse();

    println!(r#"
    ____  __  ____________ 
   / __ \/ / / /_  __/ __ \   HYPER-TENSOR PROTOCOL
  / /_/ / /_/ / / / / /_/ /   (C) 2025 M-Patek Research
 / ____/ __  / / / / ____/    Target: High-Frequency Membership
/_/   /_/ /_/ /_/ /_/         Status: Production Ready
    "#);

    info!("🚀 Initializing HTP Node...");

    // 1. Trustless Setup
    let start = std::time::Instant::now();
    let params = SystemParameters::from_random_seed(cli.seed.as_bytes(), 128);
    info!("✅ Trustless Setup Complete in {:?}", start.elapsed());
    info!("   Discriminant (Delta): {:x}...", params.discriminant);

    // 2. Initialize Hyper-Tensor
    let tensor = Arc::new(RwLock::new(HyperTensor::new(
        cli.dim, 
        100, // side length
        params.discriminant
    )));
    info!("🧊 Hyper-Tensor ({}D) Allocated. Capacity: 100^{}", cli.dim, cli.dim);

    // 3. Start Networking
    let addr: SocketAddr = cli.bind.parse()?;
    let transport = QuicTransport::bind_server(addr, "cert.pem", "key.pem").await?;
    
    info!("📡 QUIC Transport listening on {}", addr);
    
    // 4. Run Service Loop
    run_prover_service(transport.get_endpoint().clone(), tensor).await;

    Ok(())
}
