use crate::client::Client;
use crate::clients::client::IggyClient;
use crate::error::IggyError;
use crate::http::client::HttpClient;
use crate::http::config::HttpClientConfigBuilder;
use crate::partitioner::Partitioner;
use crate::quic::client::QuicClient;
use crate::quic::config::QuicClientConfigBuilder;
use crate::tcp::client::TcpClient;
use crate::tcp::config::TcpClientConfigBuilder;
use crate::utils::crypto::Encryptor;
use crate::utils::duration::IggyDuration;
use std::sync::Arc;
use tracing::error;

/// The builder for the `IggyClient` instance, which allows to configure and provide custom implementations for the partitioner, encryptor or message handler.
#[derive(Debug, Default)]
pub struct IggyClientBuilder {
    client: Option<Box<dyn Client>>,
    partitioner: Option<Arc<dyn Partitioner>>,
    encryptor: Option<Arc<dyn Encryptor>>,
}

impl IggyClientBuilder {
    /// Creates a new `IggyClientBuilder`.
    /// This is not enough to build the `IggyClient` instance. You need to provide the client configuration or the client implementation for the specific transport.
    pub fn new() -> Self {
        IggyClientBuilder::default()
    }

    pub fn from_connection_string(connection_string: &str) -> Result<Self, IggyError> {
        let mut builder = Self::new();
        builder.client = Some(Box::new(TcpClient::from_connection_string(
            connection_string,
        )?));
        Ok(builder)
    }

    /// Apply the provided client implementation for the specific transport. Setting client clears the client config.
    pub fn with_client(mut self, client: Box<dyn Client>) -> Self {
        self.client = Some(client);
        self
    }

    /// Use the custom partitioner implementation.
    pub fn with_partitioner(mut self, partitioner: Arc<dyn Partitioner>) -> Self {
        self.partitioner = Some(partitioner);
        self
    }

    /// Use the custom encryptor implementation.
    pub fn with_encryptor(mut self, encryptor: Arc<dyn Encryptor>) -> Self {
        self.encryptor = Some(encryptor);
        self
    }

    /// This method provides fluent API for the TCP client configuration.
    /// It returns the `TcpClientBuilder` instance, which allows to configure the TCP client with custom settings or using defaults.
    /// This should be called after the non-protocol specific methods, such as `with_partitioner`, `with_encryptor` or `with_message_handler`.
    pub fn with_tcp(self) -> TcpClientBuilder {
        TcpClientBuilder {
            config: TcpClientConfigBuilder::default(),
            parent_builder: self,
        }
    }

    /// This method provides fluent API for the QUIC client configuration.
    /// It returns the `QuicClientBuilder` instance, which allows to configure the QUIC client with custom settings or using defaults.
    /// This should be called after the non-protocol specific methods, such as `with_partitioner`, `with_encryptor` or `with_message_handler`.
    pub fn with_quic(self) -> QuicClientBuilder {
        QuicClientBuilder {
            config: QuicClientConfigBuilder::default(),
            parent_builder: self,
        }
    }

    /// This method provides fluent API for the HTTP client configuration.
    /// It returns the `HttpClientBuilder` instance, which allows to configure the HTTP client with custom settings or using defaults.
    /// This should be called after the non-protocol specific methods, such as `with_partitioner`, `with_encryptor` or `with_message_handler`.
    pub fn with_http(self) -> HttpClientBuilder {
        HttpClientBuilder {
            config: HttpClientConfigBuilder::default(),
            parent_builder: self,
        }
    }

    /// Build the `IggyClient` instance.
    /// This method returns an error if the client is not provided.
    /// If the client is provided, it creates the `IggyClient` instance with the provided configuration.
    /// To provide the client configuration, use the `with_tcp`, `with_quic` or `with_http` methods.
    pub fn build(self) -> Result<IggyClient, IggyError> {
        let Some(client) = self.client else {
            error!("Client is not provided");
            return Err(IggyError::InvalidConfiguration);
        };

        Ok(IggyClient::create(client, self.partitioner, self.encryptor))
    }
}

#[derive(Debug, Default)]
pub struct TcpClientBuilder {
    config: TcpClientConfigBuilder,
    parent_builder: IggyClientBuilder,
}

impl TcpClientBuilder {
    /// Sets the server address for the TCP client.
    pub fn with_server_address(mut self, server_address: String) -> Self {
        self.config = self.config.with_server_address(server_address);
        self
    }

    /// Sets the number of max retries when connecting to the server.
    pub fn with_reconnection_max_retries(mut self, reconnection_retries: Option<u32>) -> Self {
        self.config = self
            .config
            .with_reconnection_max_retries(reconnection_retries);
        self
    }

    /// Sets the interval between retries when connecting to the server.
    pub fn with_reconnection_interval(mut self, reconnection_interval: IggyDuration) -> Self {
        self.config = self
            .config
            .with_reconnection_interval(reconnection_interval);
        self
    }

    /// Sets whether to use TLS when connecting to the server.
    pub fn with_tls_enabled(mut self, tls_enabled: bool) -> Self {
        self.config = self.config.with_tls_enabled(tls_enabled);
        self
    }

    /// Sets the domain to use for TLS when connecting to the server.
    pub fn with_tls_domain(mut self, tls_domain: String) -> Self {
        self.config = self.config.with_tls_domain(tls_domain);
        self
    }

    /// Builds the parent `IggyClient` with TCP configuration.
    pub fn build(self) -> Result<IggyClient, IggyError> {
        let client = TcpClient::create(Arc::new(self.config.build()))?;
        let client = self.parent_builder.with_client(Box::new(client)).build()?;
        Ok(client)
    }
}

#[derive(Debug, Default)]
pub struct QuicClientBuilder {
    config: QuicClientConfigBuilder,
    parent_builder: IggyClientBuilder,
}

impl QuicClientBuilder {
    /// Sets the server address for the QUIC client.
    pub fn with_server_address(mut self, server_address: String) -> Self {
        self.config = self.config.with_server_address(server_address);
        self
    }

    /// Sets the number of retries when connecting to the server.
    pub fn with_reconnection_max_retries(mut self, reconnection_retries: Option<u32>) -> Self {
        self.config = self
            .config
            .with_reconnection_max_retries(reconnection_retries);
        self
    }

    /// Sets the interval between retries when connecting to the server.
    pub fn with_reconnection_interval(mut self, reconnection_interval: IggyDuration) -> Self {
        self.config = self
            .config
            .with_reconnection_interval(reconnection_interval);
        self
    }

    /// Sets the server name for the QUIC client.
    pub fn with_server_name(mut self, server_name: String) -> Self {
        self.config = self.config.with_server_name(server_name);
        self
    }

    /// Builds the parent `IggyClient` with QUIC configuration.
    pub fn build(self) -> Result<IggyClient, IggyError> {
        let client = QuicClient::create(Arc::new(self.config.build()))?;
        let client = self.parent_builder.with_client(Box::new(client)).build()?;
        Ok(client)
    }
}

#[derive(Debug, Default)]
pub struct HttpClientBuilder {
    config: HttpClientConfigBuilder,
    parent_builder: IggyClientBuilder,
}

impl HttpClientBuilder {
    /// Sets the server address for the HTTP client.
    pub fn with_api_url(mut self, api_url: String) -> Self {
        self.config = self.config.with_api_url(api_url);
        self
    }

    /// Sets the number of retries for the HTTP client.
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.config = self.config.with_retries(retries);
        self
    }

    /// Builds the parent `IggyClient` with HTTP configuration.
    pub fn build(self) -> Result<IggyClient, IggyError> {
        let client = HttpClient::create(Arc::new(self.config.build()))?;
        let client = self.parent_builder.with_client(Box::new(client)).build()?;
        Ok(client)
    }
}
