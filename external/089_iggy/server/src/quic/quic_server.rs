use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use quinn::{Endpoint, IdleTimeout, VarInt};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tracing::info;

use crate::configs::quic::QuicConfig;
use crate::quic::listener;
use crate::streaming::systems::system::SharedSystem;

/// Starts the QUIC server.
/// Returns the address the server is listening on.
pub fn start(config: QuicConfig, system: SharedSystem) -> SocketAddr {
    info!("Initializing Iggy QUIC server...");
    let address = config.address.parse().unwrap();
    let quic_config = configure_quic(config);
    if let Err(error) = quic_config {
        panic!("Error when configuring QUIC: {:?}", error);
    }

    let endpoint = Endpoint::server(quic_config.unwrap(), address).unwrap();
    let addr = endpoint.local_addr().unwrap();
    listener::start(endpoint, system);
    info!("Iggy QUIC server has started on: {:?}", addr);
    addr
}

fn configure_quic(config: QuicConfig) -> Result<quinn::ServerConfig, Box<dyn Error>> {
    let (certificate, key) = match config.certificate.self_signed {
        true => generate_self_signed_cert()?,
        false => load_certificates(&config.certificate.cert_file, &config.certificate.key_file)?,
    };

    let mut server_config = quinn::ServerConfig::with_single_cert(certificate, key)?;
    let mut transport = quinn::TransportConfig::default();
    transport.initial_mtu(config.initial_mtu.as_bytes_u64() as u16);
    transport.send_window(config.send_window.as_bytes_u64());
    transport.receive_window(VarInt::try_from(config.receive_window.as_bytes_u64())?);
    transport.datagram_send_buffer_size(config.datagram_send_buffer_size.as_bytes_u64() as usize);
    transport.max_concurrent_bidi_streams(VarInt::try_from(config.max_concurrent_bidi_streams)?);
    if !config.keep_alive_interval.is_zero() {
        transport.keep_alive_interval(Some(config.keep_alive_interval.get_duration()));
    }
    if !config.max_idle_timeout.is_zero() {
        let max_idle_timeout = IdleTimeout::try_from(config.max_idle_timeout.get_duration())?;
        transport.max_idle_timeout(Some(max_idle_timeout));
    }

    server_config.transport_config(Arc::new(transport));
    Ok(server_config)
}

fn generate_self_signed_cert<'a>(
) -> Result<(Vec<CertificateDer<'a>>, PrivateKeyDer<'a>), Box<dyn Error>> {
    let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let certificate_der = certificate.cert.der();
    let private_key = certificate.key_pair.serialize_der();
    let private_key = PrivateKeyDer::try_from(private_key)?;
    let cert_chain = vec![certificate_der.clone()];
    Ok((cert_chain, private_key))
}

fn load_certificates(
    cert_file: &str,
    key_file: &str,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), Box<dyn Error>> {
    let mut cert_chain_reader = BufReader::new(File::open(cert_file)?);
    let certs = rustls_pemfile::certs(&mut cert_chain_reader)
        .map(|x| CertificateDer::from(x.unwrap().to_vec()))
        .collect();
    let mut key_reader = BufReader::new(File::open(key_file)?);
    let mut keys = rustls_pemfile::rsa_private_keys(&mut key_reader)
        .filter(|key| key.is_ok())
        .map(|key| PrivateKeyDer::try_from(key.unwrap().secret_pkcs1_der().to_vec()))
        .collect::<Result<Vec<_>, _>>()?;
    let key = keys.remove(0);
    Ok((certs, key))
}
