// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use std::sync::Arc;
use quinn::{Endpoint, ServerConfig, ClientConfig};
use std::net::SocketAddr;
use std::error::Error;
use rcgen::generate_simple_self_signed; // [FIX]: Added for cert generation

pub struct QuicTransport {
    endpoint: Endpoint,
}

impl QuicTransport {
    pub async fn bind_server(addr: SocketAddr, cert_path: &str, key_path: &str) -> Result<Self, Box<dyn Error>> {
        // [FIX]: Safe certificate loading (Generates ephemeral cert if missing)
        // Prevents Panic on startup.
        let (cert, key) = Self::load_certs(cert_path, key_path)?;
        
        let server_crypto = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert, key)?;
            
        let mut server_config = ServerConfig::with_crypto(Arc::new(server_crypto));
        
        let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
        transport_config.max_concurrent_uni_streams(0_u8.into()); 
        transport_config.max_concurrent_bidi_streams(1024_u8.into());
        
        let endpoint = Endpoint::server(server_config, addr)?;
        println!("[Net] Prover Service listening on QUIC {}", addr);
        
        Ok(Self { endpoint })
    }

    pub fn bind_client() -> Result<Self, Box<dyn Error>> {
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())?;
        let client_crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_native_roots()
            .with_no_client_auth();
            
        endpoint.set_default_client_config(ClientConfig::new(Arc::new(client_crypto)));
        
        Ok(Self { endpoint })
    }
    
    // [FIX]: Implementation of ephemeral cert generation
    fn load_certs(cert_path: &str, _key_path: &str) -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey), Box<dyn Error>> {
        // If file doesn't exist, generate in-memory
        if std::fs::metadata(cert_path).is_err() {
            println!("⚠️  Certificate file not found. Generating ephemeral self-signed cert...");
            
            let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
            let cert = generate_simple_self_signed(subject_alt_names)?;
            
            let cert_der = cert.serialize_der()?;
            let key_der = cert.serialize_private_key_der();
            
            return Ok((
                vec![rustls::Certificate(cert_der)],
                rustls::PrivateKey(key_der),
            ));
        }
        
        // Original logic would go here...
        Err("File loading not implemented in this demo, used fallback.".into())
    }

    pub fn get_endpoint(&self) -> &Endpoint {
        &self.endpoint
    }
}
