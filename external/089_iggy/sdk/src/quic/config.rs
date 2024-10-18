use crate::client::AutoLogin;
use crate::utils::duration::IggyDuration;
use std::str::FromStr;

/// Configuration for the QUIC client.
#[derive(Debug, Clone)]
pub struct QuicClientConfig {
    /// The address to bind the QUIC client to.
    pub client_address: String,
    /// The address of the QUIC server to connect to.
    pub server_address: String,
    /// The server name to use.
    pub server_name: String,
    /// Whether to automatically login user after establishing connection.
    pub auto_login: AutoLogin,
    // Whether to automatically reconnect when disconnected.
    pub reconnection: QuicClientReconnectionConfig,
    /// The size of the response buffer.
    pub response_buffer_size: u64,
    /// The maximum number of concurrent bidirectional streams.
    pub max_concurrent_bidi_streams: u64,
    /// The size of the datagram send buffer.
    pub datagram_send_buffer_size: u64,
    /// The initial MTU.
    pub initial_mtu: u16,
    /// The send window.
    pub send_window: u64,
    /// The receive window.
    pub receive_window: u64,
    /// The keep alive interval.
    pub keep_alive_interval: u64,
    /// The maximum idle timeout.
    pub max_idle_timeout: u64,
    /// Whether to validate the server certificate.
    pub validate_certificate: bool,
    /// Interval of heartbeats sent by the client
    pub heartbeat_interval: IggyDuration,
}

#[derive(Debug, Clone)]
pub struct QuicClientReconnectionConfig {
    pub enabled: bool,
    pub max_retries: Option<u32>,
    pub interval: IggyDuration,
    pub reestablish_after: IggyDuration,
}

impl Default for QuicClientReconnectionConfig {
    fn default() -> QuicClientReconnectionConfig {
        QuicClientReconnectionConfig {
            enabled: true,
            max_retries: None,
            interval: IggyDuration::from_str("1s").unwrap(),
            reestablish_after: IggyDuration::from_str("5s").unwrap(),
        }
    }
}

impl Default for QuicClientConfig {
    fn default() -> QuicClientConfig {
        QuicClientConfig {
            client_address: "127.0.0.1:0".to_string(),
            server_address: "127.0.0.1:8080".to_string(),
            server_name: "localhost".to_string(),
            auto_login: AutoLogin::Disabled,
            heartbeat_interval: IggyDuration::from_str("5s").unwrap(),
            reconnection: QuicClientReconnectionConfig::default(),
            response_buffer_size: 1000 * 1000 * 10,
            max_concurrent_bidi_streams: 10000,
            datagram_send_buffer_size: 100_000,
            initial_mtu: 1200,
            send_window: 100_000,
            receive_window: 100_000,
            keep_alive_interval: 5000,
            max_idle_timeout: 10000,
            validate_certificate: false,
        }
    }
}

/// Builder for the QUIC client configuration.
///
/// Allows configuring the QUIC client with custom settings or using defaults:
/// - `client_address`: Default is "127.0.0.1:0" (binds to any available port).
/// - `server_address`: Default is "127.0.0.1:8080".
/// - `server_name`: Default is "localhost".
/// - `auto_login`: Default is AutoLogin::Disabled.
/// - `reconnection`: Default is enabled unlimited retries and 1 second interval.
/// - `response_buffer_size`: Default is 10MB (10,000,000 bytes).
/// - `max_concurrent_bidi_streams`: Default is 10,000 streams.
/// - `datagram_send_buffer_size`: Default is 100,000 bytes.
/// - `initial_mtu`: Default is 1200 bytes.
/// - `send_window`: Default is 100,000 bytes.
/// - `receive_window`: Default is 100,000 bytes.
/// - `keep_alive_interval`: Default is 5000 milliseconds.
/// - `max_idle_timeout`: Default is 10,000 milliseconds.
/// - `validate_certificate`: Default is false (certificate validation is disabled).
#[derive(Debug, Default)]
pub struct QuicClientConfigBuilder {
    config: QuicClientConfig,
}

impl QuicClientConfigBuilder {
    /// Creates a new builder instance with default configuration values.
    pub fn new() -> Self {
        QuicClientConfigBuilder::default()
    }

    /// Sets the client address. Defaults to "127.0.0.1:0".
    pub fn with_client_address(mut self, client_address: String) -> Self {
        self.config.client_address = client_address;
        self
    }

    /// Sets the server address. Defaults to "127.0.0.1:8080".
    pub fn with_server_address(mut self, server_address: String) -> Self {
        self.config.server_address = server_address;
        self
    }

    /// Sets the auto sign in during connection.
    pub fn with_auto_sign_in(mut self, auto_sign_in: AutoLogin) -> Self {
        self.config.auto_login = auto_sign_in;
        self
    }

    /// Sets the server name. Defaults to "localhost".
    pub fn with_server_name(mut self, server_name: String) -> Self {
        self.config.server_name = server_name;
        self
    }

    pub fn with_enabled_reconnection(mut self) -> Self {
        self.config.reconnection.enabled = true;
        self
    }

    /// Sets the number of retries when connecting to the server.
    pub fn with_reconnection_max_retries(mut self, max_retries: Option<u32>) -> Self {
        self.config.reconnection.max_retries = max_retries;
        self
    }

    /// Sets the interval between retries when connecting to the server.
    pub fn with_reconnection_interval(mut self, interval: IggyDuration) -> Self {
        self.config.reconnection.interval = interval;
        self
    }

    /// Sets the response buffer size in bytes. Defaults to 10MB (10,000,000 bytes).
    pub fn with_response_buffer_size(mut self, response_buffer_size: u64) -> Self {
        self.config.response_buffer_size = response_buffer_size;
        self
    }

    /// Sets the maximum number of concurrent bidirectional streams. Defaults to 10,000.
    pub fn with_max_concurrent_bidi_streams(mut self, max_concurrent_bidi_streams: u64) -> Self {
        self.config.max_concurrent_bidi_streams = max_concurrent_bidi_streams;
        self
    }

    /// Sets the datagram send buffer size in bytes. Defaults to 100,000 bytes.
    pub fn with_datagram_send_buffer_size(mut self, datagram_send_buffer_size: u64) -> Self {
        self.config.datagram_send_buffer_size = datagram_send_buffer_size;
        self
    }

    /// Sets the initial MTU (Maximum Transmission Unit) in bytes. Defaults to 1200 bytes.
    pub fn with_initial_mtu(mut self, initial_mtu: u16) -> Self {
        self.config.initial_mtu = initial_mtu;
        self
    }

    /// Sets the send window size in bytes. Defaults to 100,000 bytes.
    pub fn with_send_window(mut self, send_window: u64) -> Self {
        self.config.send_window = send_window;
        self
    }

    /// Sets the receive window size in bytes. Defaults to 100,000 bytes.
    pub fn with_receive_window(mut self, receive_window: u64) -> Self {
        self.config.receive_window = receive_window;
        self
    }

    /// Sets the keep-alive interval in milliseconds. Defaults to 5000ms.
    pub fn with_keep_alive_interval(mut self, keep_alive_interval: u64) -> Self {
        self.config.keep_alive_interval = keep_alive_interval;
        self
    }

    /// Sets the maximum idle timeout in milliseconds. Defaults to 10,000ms.
    pub fn with_max_idle_timeout(mut self, max_idle_timeout: u64) -> Self {
        self.config.max_idle_timeout = max_idle_timeout;
        self
    }

    /// Enables or disables certificate validation. Defaults to false (disabled).
    pub fn with_validate_certificate(mut self, validate_certificate: bool) -> Self {
        self.config.validate_certificate = validate_certificate;
        self
    }

    /// Sets the heartbeat interval. Defaults to 5000ms.
    pub fn with_heartbeat_interval(mut self, interval: IggyDuration) -> Self {
        self.config.heartbeat_interval = interval;
        self
    }

    /// Finalizes the builder and returns the `QuicClientConfig`.
    pub fn build(self) -> QuicClientConfig {
        self.config
    }
}
