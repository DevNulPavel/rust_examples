use crate::client::{AutoLogin, Client, Credentials};
use crate::client_error::ClientError;
#[allow(deprecated)]
use crate::clients::client::IggyClient;
use crate::http::client::HttpClient;
use crate::http::config::HttpClientConfig;
use crate::quic::client::QuicClient;
use crate::quic::config::{QuicClientConfig, QuicClientReconnectionConfig};
use crate::tcp::client::TcpClient;
use crate::tcp::config::{TcpClientConfig, TcpClientReconnectionConfig};
use crate::utils::duration::IggyDuration;
use std::str::FromStr;
use std::sync::Arc;

const QUIC_TRANSPORT: &str = "quic";
const HTTP_TRANSPORT: &str = "http";
const TCP_TRANSPORT: &str = "tcp";

/// Configuration for the `ClientProvider`.
/// It consists of the following fields:
/// - `transport`: the transport to use. Valid values are `quic`, `http` and `tcp`.
/// - `http`: the optional configuration for the HTTP transport.
/// - `quic`: the optional configuration for the QUIC transport.
/// - `tcp`: the optional configuration for the TCP transport.
#[derive(Debug)]
pub struct ClientProviderConfig {
    /// The transport to use. Valid values are `quic`, `http` and `tcp`.
    pub transport: String,
    /// The optional configuration for the HTTP transport.
    pub http: Option<Arc<HttpClientConfig>>,
    /// The optional configuration for the QUIC transport.
    pub quic: Option<Arc<QuicClientConfig>>,
    /// The optional configuration for the TCP transport.
    pub tcp: Option<Arc<TcpClientConfig>>,
}

impl Default for ClientProviderConfig {
    fn default() -> ClientProviderConfig {
        ClientProviderConfig {
            transport: TCP_TRANSPORT.to_string(),
            http: Some(Arc::new(HttpClientConfig::default())),
            quic: Some(Arc::new(QuicClientConfig::default())),
            tcp: Some(Arc::new(TcpClientConfig::default())),
        }
    }
}

impl ClientProviderConfig {
    /// Create a new `ClientProviderConfig` from the provided `Args`.
    pub fn from_args(args: crate::args::Args) -> Result<Self, ClientError> {
        Self::from_args_set_autologin(args, true)
    }

    /// Create a new `ClientProviderConfig` from the provided `Args` with possibility to enable or disable
    /// auto login option for TCP or QUIC protocols.
    pub fn from_args_set_autologin(
        args: crate::args::Args,
        auto_login: bool,
    ) -> Result<Self, ClientError> {
        let transport = args.transport;
        let mut config = Self {
            transport,
            http: None,
            quic: None,
            tcp: None,
        };
        match config.transport.as_str() {
            QUIC_TRANSPORT => {
                config.quic = Some(Arc::new(QuicClientConfig {
                    client_address: args.quic_client_address,
                    server_address: args.quic_server_address,
                    server_name: args.quic_server_name,
                    heartbeat_interval: IggyDuration::from_str(&args.quic_heartbeat_interval)
                        .unwrap(),
                    reconnection: QuicClientReconnectionConfig {
                        enabled: args.quic_reconnection_enabled,
                        max_retries: args.quic_reconnection_max_retries,
                        interval: IggyDuration::from_str(&args.quic_reconnection_interval).unwrap(),
                        reestablish_after: IggyDuration::from_str(
                            &args.quic_reconnection_reestablish_after,
                        )
                        .unwrap(),
                    },
                    auto_login: if auto_login {
                        AutoLogin::Enabled(Credentials::UsernamePassword(
                            args.username,
                            args.password,
                        ))
                    } else {
                        AutoLogin::Disabled
                    },
                    response_buffer_size: args.quic_response_buffer_size,
                    max_concurrent_bidi_streams: args.quic_max_concurrent_bidi_streams,
                    datagram_send_buffer_size: args.quic_datagram_send_buffer_size,
                    initial_mtu: args.quic_initial_mtu,
                    send_window: args.quic_send_window,
                    receive_window: args.quic_receive_window,
                    keep_alive_interval: args.quic_keep_alive_interval,
                    max_idle_timeout: args.quic_max_idle_timeout,
                    validate_certificate: args.quic_validate_certificate,
                }));
            }
            HTTP_TRANSPORT => {
                config.http = Some(Arc::new(HttpClientConfig {
                    api_url: args.http_api_url,
                    retries: args.http_retries,
                }));
            }
            TCP_TRANSPORT => {
                config.tcp = Some(Arc::new(TcpClientConfig {
                    server_address: args.tcp_server_address,
                    tls_enabled: args.tcp_tls_enabled,
                    tls_domain: args.tcp_tls_domain,
                    heartbeat_interval: IggyDuration::from_str(&args.tcp_heartbeat_interval)
                        .unwrap(),
                    reconnection: TcpClientReconnectionConfig {
                        enabled: args.tcp_reconnection_enabled,
                        max_retries: args.tcp_reconnection_max_retries,
                        interval: IggyDuration::from_str(&args.tcp_reconnection_interval).unwrap(),
                        reestablish_after: IggyDuration::from_str(
                            &args.tcp_reconnection_reestablish_after,
                        )
                        .unwrap(),
                    },
                    auto_login: if auto_login {
                        AutoLogin::Enabled(Credentials::UsernamePassword(
                            args.username,
                            args.password,
                        ))
                    } else {
                        AutoLogin::Disabled
                    },
                }));
            }
            _ => return Err(ClientError::InvalidTransport(config.transport.clone())),
        }

        Ok(config)
    }
}

/// Create a default `IggyClient` with the default configuration.
pub async fn get_default_client_() -> Result<IggyClient, ClientError> {
    get_client(Arc::new(ClientProviderConfig::default())).await
}

/// Create a `IggyClient` for the specific transport based on the provided configuration.
pub async fn get_client(config: Arc<ClientProviderConfig>) -> Result<IggyClient, ClientError> {
    let client = get_raw_connected_client(config).await?;
    Ok(IggyClient::builder().with_client(client).build()?)
}

/// Create a `Client` for the specific transport based on the provided configuration.
pub async fn get_raw_connected_client(
    config: Arc<ClientProviderConfig>,
) -> Result<Box<dyn Client>, ClientError> {
    get_raw_client(config, true).await
}

/// Create a `Client` for the specific transport based on the provided configuration.
pub async fn get_raw_client(
    config: Arc<ClientProviderConfig>,
    establish_connection: bool,
) -> Result<Box<dyn Client>, ClientError> {
    let transport = config.transport.clone();
    match transport.as_str() {
        QUIC_TRANSPORT => {
            let quic_config = config.quic.as_ref().unwrap();
            let client = QuicClient::create(quic_config.clone())?;
            if establish_connection {
                Client::connect(&client).await?
            };
            Ok(Box::new(client))
        }
        HTTP_TRANSPORT => {
            let http_config = config.http.as_ref().unwrap();
            let client = HttpClient::create(http_config.clone())?;
            Ok(Box::new(client))
        }
        TCP_TRANSPORT => {
            let tcp_config = config.tcp.as_ref().unwrap();
            let client = TcpClient::create(tcp_config.clone())?;
            if establish_connection {
                Client::connect(&client).await?
            };
            Ok(Box::new(client))
        }
        _ => Err(ClientError::InvalidTransport(transport)),
    }
}
