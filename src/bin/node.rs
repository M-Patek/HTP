// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use clap::Parser;
use log::{info, error};
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

    // [SECURITY FIX]: 输入参数校验，防止零维黑洞 (Zero-Dimension Singularity)
    if cli.dim == 0 || cli.dim > 20 {
        error!("❌ Invalid dimension. Must be between 1 and 20.");
        std::process::exit(1);
    }

    info!("🚀 Initializing HTP Node (Secure Edition)...");

    // [FIX]: 健忘节点修复 - 启用持久化加载
    let db_path = "htp_tensor.db";
    let tensor = if std::path::Path::new(db_path).exists() {
        info!("💾 Found existing database. Loading...");
        match HyperTensor::load_from_disk(db_path) {
            Ok(t) => {
                info!("✅ Database loaded successfully.");
                Arc::new(RwLock::new(t))
            },
            Err(e) => {
                error!("❌ Failed to load database: {}. Starting fresh.", e);
                let params = SystemParameters::from_random_seed(cli.seed.as_bytes(), 2048);
                Arc::new(RwLock::new(HyperTensor::new(cli.dim, 100, params.discriminant)))
            }
        }
    } else {
        info!("✨ Creating new Hyper-Tensor.");
        let params = SystemParameters::from_random_seed(cli.seed.as_bytes(), 2048);
        Arc::new(RwLock::new(HyperTensor::new(cli.dim, 100, params.discriminant)))
    };

    let addr: SocketAddr = cli.bind.parse()?;
    let transport = QuicTransport::bind_server(addr, "cert.pem", "key.pem").await?;
    
    info!("📡 QUIC Transport listening on {}", addr);
    run_prover_service(transport.get_endpoint().clone(), tensor).await;

    Ok(())
}
