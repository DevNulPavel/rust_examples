use crate::args::common::IggyBenchArgs;
use integration::test_server::{IpAddrKind, TestServer, Transport, SYSTEM_PATH_ENV_VAR};
use serde::Deserialize;
use std::net::SocketAddr;
use std::{collections::HashMap, time::Instant};
use tokio::net::{TcpStream, UdpSocket};
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
struct ServerConfig {
    http: ConfigAddress,
    tcp: ConfigAddress,
    quic: ConfigAddress,
}

#[derive(Debug, Deserialize)]
struct ConfigAddress {
    address: String,
}

pub async fn start_server_if_needed(args: &mut IggyBenchArgs) -> Option<TestServer> {
    if args.skip_server_start {
        info!("Skipping iggy-server start");
        return None;
    }

    let default_config: ServerConfig =
        toml::from_str(include_str!("../../configs/server.toml")).unwrap();
    let (should_start, mut envs) = match &args.transport() {
        Transport::Http => {
            let args_http_address = args.server_address().parse::<SocketAddr>().unwrap();
            let config_http_address = default_config.http.address.parse::<SocketAddr>().unwrap();
            let envs = HashMap::from([
                (
                    "IGGY_HTTP_ADDRESS".to_owned(),
                    default_config.http.address.to_owned(),
                ),
                ("IGGY_TCP_ENABLED".to_owned(), "false".to_owned()),
                ("IGGY_QUIC_ENABLED".to_owned(), "false".to_owned()),
            ]);
            (
                addresses_are_equivalent(&args_http_address, &config_http_address)
                    && !is_tcp_addr_in_use(&args_http_address).await,
                envs,
            )
        }
        Transport::Tcp => {
            let args_tcp_address = args.server_address().parse::<SocketAddr>().unwrap();
            let config_tcp_address = default_config.tcp.address.parse::<SocketAddr>().unwrap();

            let envs = HashMap::from([
                (
                    "IGGY_TCP_ADDRESS".to_owned(),
                    default_config.tcp.address.to_owned(),
                ),
                ("IGGY_HTTP_ENABLED".to_owned(), "false".to_owned()),
                ("IGGY_QUIC_ENABLED".to_owned(), "false".to_owned()),
            ]);
            (
                addresses_are_equivalent(&args_tcp_address, &config_tcp_address)
                    && !is_tcp_addr_in_use(&args_tcp_address).await,
                envs,
            )
        }
        Transport::Quic => {
            let args_quic_address = args.server_address().parse::<SocketAddr>().unwrap();
            let config_quic_address = default_config.quic.address.parse::<SocketAddr>().unwrap();
            let envs = HashMap::from([
                (
                    "IGGY_QUIC_ADDRESS".to_owned(),
                    default_config.quic.address.to_owned(),
                ),
                ("IGGY_HTTP_ENABLED".to_owned(), "false".to_owned()),
                ("IGGY_TCP_ENABLED".to_owned(), "false".to_owned()),
            ]);

            (
                addresses_are_equivalent(&args_quic_address, &config_quic_address)
                    && !is_udp_addr_in_use(&args_quic_address).await,
                envs,
            )
        }
    };

    if should_start {
        envs.insert(
            SYSTEM_PATH_ENV_VAR.to_owned(),
            args.server_system_path.clone(),
        );

        if args.verbose {
            envs.insert("IGGY_TEST_VERBOSE".to_owned(), "true".to_owned());
            info!("Enabling verbose output - iggy-server will logs print to stdout")
        } else {
            info!("Disabling verbose output - iggy-server will print logs to files")
        }

        info!(
            "Starting test server, transport: {}, data path: {}, cleanup: {}, verbosity: {}",
            args.transport(),
            args.server_system_path,
            args.cleanup,
            args.verbose
        );
        let mut test_server = TestServer::new(
            Some(envs),
            args.cleanup,
            args.server_executable_path.clone(),
            IpAddrKind::V4,
        );
        let now = Instant::now();
        test_server.start();
        let elapsed = now.elapsed();
        if elapsed.as_millis() > 1000 {
            warn!("Test iggy-server started, pid: {}, startup took {} ms because it had to load messages from disk to cache", test_server.pid(), elapsed.as_millis());
        } else {
            info!(
                "Test iggy-server started, pid: {}, startup time: {} ms",
                test_server.pid(),
                elapsed.as_millis()
            )
        }

        Some(test_server)
    } else {
        info!("Skipping iggy-server start");
        None
    }
}

async fn is_tcp_addr_in_use(addr: &SocketAddr) -> bool {
    TcpStream::connect(addr).await.is_ok()
}

async fn is_udp_addr_in_use(addr: &SocketAddr) -> bool {
    UdpSocket::bind(addr).await.is_err()
}

fn addresses_are_equivalent(first: &SocketAddr, second: &SocketAddr) -> bool {
    if first.ip().is_unspecified() || second.ip().is_unspecified() {
        first.port() == second.port()
    } else {
        first == second
    }
}
