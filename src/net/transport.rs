// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.
//
// MODULE: Transport Layer (QUIC)
// DESCRIPTION: Asynchronous, multiplexed transport over UDP.
// SECURITY: TLS 1.3 enforced by default via rustls.

use std::sync::Arc;
use quinn::{Endpoint, ServerConfig, ClientConfig};
use std::net::SocketAddr;
use std::error::Error;

/// The HTP Transport Engine.
/// Handles connection pooling and stream multiplexing.
pub struct QuicTransport {
    endpoint: Endpoint,
}

impl QuicTransport {
    /// Starts a Prover Node (Server) listening on a specific port.
    pub async fn bind_server(addr: SocketAddr, cert_path: &str, key_path: &str) -> Result<Self, Box<dyn Error>> {
        // [Showcase Detail]: Loading TLS certificates for QUIC
        let (cert, key) = Self::load_certs(cert_path, key_path)?;
        let server_crypto = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert, key)?;
            
        let mut server_config = ServerConfig::with_crypto(Arc::new(server_crypto));
        
        // [Optimization]: Tuning transport parameters for high throughput accumulator data
        let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
        transport_config.max_concurrent_uni_streams(0_u8.into()); // We only use bi-directional
        transport_config.max_concurrent_bidi_streams(1024_u8.into()); // High concurrency
        
        let endpoint = Endpoint::server(server_config, addr)?;
        println!("[Net] Prover Service listening on QUIC {}", addr);
        
        Ok(Self { endpoint })
    }

    /// Starts a Verifier Node (Client).
    pub fn bind_client() -> Result<Self, Box<dyn Error>> {
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())?;
        let client_crypto = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_native_roots()
            .with_no_client_auth();
            
        endpoint.set_default_client_config(ClientConfig::new(Arc::new(client_crypto)));
        
        Ok(Self { endpoint })
    }
    
    // ... Internal helper: load_certs ...
    fn load_certs(cert: &str, key: &str) -> Result<(Vec<rustls::Certificate>, rustls::PrivateKey), Box<dyn Error>> {
        // Implementation redacted for showcase brevity
        Ok((vec![], rustls::PrivateKey(vec![])))
    }

    pub fn get_endpoint(&self) -> &Endpoint {
        &self.endpoint
    }
}
