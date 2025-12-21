// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use std::sync::Arc;
use quinn::{Endpoint, ServerConfig, ClientConfig};
use std::net::SocketAddr;
use std::error::Error;
use rcgen::generate_simple_self_signed;

pub struct QuicTransport {
    endpoint: Endpoint,
}

impl QuicTransport {
    pub async fn bind_server(addr: SocketAddr, cert_path: &str, key_path: &str) -> Result<Self, Box<dyn Error>> {
        let (cert, key) = Self::load_or_generate_certs(cert_path, key_path)?;
        
        let server_crypto = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert, key)?;
            
        let mut server_config = ServerConfig::with_crypto(Arc::new(server_crypto));
        
        // [FIX]: 安全解包，防止多线程环境下 unwrap 导致 Panic
        if let Some(transport_config) = Arc::get_mut(&mut server_config.transport) {
            transport_config.max_concurrent_uni_streams(0_u8.into()); 
            transport_config.max_concurrent_bidi_streams(1024_u8.into());
        } else {
            return Err("Failed to configure transport: concurrent access detected.".into());
        }
        
        let endpoint = Endpoint::server(server_config, addr)?;
        Ok(Self { endpoint })
    }

    pub fn bind_client() -> Result<Self, Box<dyn Error>> {
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())?;
        let client_crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_native_roots()
            .with_no_client_auth(); // 开发环境跳过验证
        endpoint.set_default_client_config(ClientConfig::new(Arc::new(client_crypto)));
        Ok(Self { endpoint })
    }
    
    fn load_or_generate_certs(cert_path: &str, key_path: &str) -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey), Box<dyn Error>> {
        let c_path = std::path::Path::new(cert_path);
        let k_path = std::path::Path::new(key_path);

        if c_path.exists() && k_path.exists() {
            println!("🔐 Found certificates at {:?}. (Mocking load logic for demo)", c_path);
            // 生产环境应使用 rustls-pemfile 加载，Demo 仍回退
            println!("⚠️  PEM parser not linked. Generating ephemeral certs.");
        } 

        println!("🛠️  Generating ephemeral self-signed cert...");
        let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
        let cert = generate_simple_self_signed(subject_alt_names)?;
        let cert_der = cert.serialize_der()?;
        let key_der = cert.serialize_private_key_der();
        Ok((vec![rustls::Certificate(cert_der)], rustls::PrivateKey(key_der)))
    }

    pub fn get_endpoint(&self) -> &Endpoint {
        &self.endpoint
    }
}
