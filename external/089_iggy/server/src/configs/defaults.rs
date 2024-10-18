use crate::configs::http::{
    HttpConfig, HttpCorsConfig, HttpJwtConfig, HttpMetricsConfig, HttpTlsConfig,
};
use crate::configs::quic::{QuicCertificateConfig, QuicConfig};
use crate::configs::server::{
    ArchiverConfig, DataMaintenanceConfig, HeartbeatConfig, MessageSaverConfig,
    MessagesMaintenanceConfig, PersonalAccessTokenCleanerConfig, PersonalAccessTokenConfig,
    ServerConfig, StateMaintenanceConfig, TelemetryConfig, TelemetryLogsConfig,
    TelemetryTracesConfig,
};
use crate::configs::system::{
    BackupConfig, CacheConfig, CompatibilityConfig, CompressionConfig, EncryptionConfig,
    LoggingConfig, MessageDeduplicationConfig, PartitionConfig, RecoveryConfig, RuntimeConfig,
    SegmentConfig, StateConfig, StreamConfig, SystemConfig, TopicConfig,
};
use crate::configs::tcp::{TcpConfig, TcpTlsConfig};
use std::sync::Arc;

static_toml::static_toml! {
    // static_toml crate always starts from CARGO_MANIFEST_DIR (in this case iggy-server root directory)
    static SERVER_CONFIG = include_toml!("../configs/server.toml");
}

impl Default for ServerConfig {
    fn default() -> ServerConfig {
        ServerConfig {
            data_maintenance: DataMaintenanceConfig::default(),
            heartbeat: HeartbeatConfig::default(),
            message_saver: MessageSaverConfig::default(),
            personal_access_token: PersonalAccessTokenConfig::default(),
            system: Arc::new(SystemConfig::default()),
            quic: QuicConfig::default(),
            tcp: TcpConfig::default(),
            http: HttpConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

impl Default for ArchiverConfig {
    fn default() -> ArchiverConfig {
        ArchiverConfig {
            enabled: false,
            kind: SERVER_CONFIG
                .data_maintenance
                .archiver
                .kind
                .parse()
                .unwrap(),
            disk: None,
            s3: None,
        }
    }
}

impl Default for MessagesMaintenanceConfig {
    fn default() -> MessagesMaintenanceConfig {
        MessagesMaintenanceConfig {
            archiver_enabled: SERVER_CONFIG.data_maintenance.messages.archiver_enabled,
            cleaner_enabled: SERVER_CONFIG.data_maintenance.messages.cleaner_enabled,
            interval: SERVER_CONFIG
                .data_maintenance
                .messages
                .interval
                .parse()
                .unwrap(),
        }
    }
}

impl Default for StateMaintenanceConfig {
    fn default() -> StateMaintenanceConfig {
        StateMaintenanceConfig {
            archiver_enabled: SERVER_CONFIG.data_maintenance.state.archiver_enabled,
            overwrite: SERVER_CONFIG.data_maintenance.state.overwrite,
            interval: SERVER_CONFIG
                .data_maintenance
                .state
                .interval
                .parse()
                .unwrap(),
        }
    }
}

impl Default for QuicConfig {
    fn default() -> QuicConfig {
        QuicConfig {
            enabled: SERVER_CONFIG.quic.enabled,
            address: SERVER_CONFIG.quic.address.parse().unwrap(),
            max_concurrent_bidi_streams: SERVER_CONFIG.quic.max_concurrent_bidi_streams as u64,
            datagram_send_buffer_size: SERVER_CONFIG
                .quic
                .datagram_send_buffer_size
                .parse()
                .unwrap(),
            initial_mtu: SERVER_CONFIG.quic.initial_mtu.parse().unwrap(),
            send_window: SERVER_CONFIG.quic.send_window.parse().unwrap(),
            receive_window: SERVER_CONFIG.quic.receive_window.parse().unwrap(),
            keep_alive_interval: SERVER_CONFIG.quic.keep_alive_interval.parse().unwrap(),
            max_idle_timeout: SERVER_CONFIG.quic.max_idle_timeout.parse().unwrap(),
            certificate: QuicCertificateConfig::default(),
        }
    }
}

impl Default for QuicCertificateConfig {
    fn default() -> QuicCertificateConfig {
        QuicCertificateConfig {
            self_signed: SERVER_CONFIG.quic.certificate.self_signed,
            cert_file: SERVER_CONFIG.quic.certificate.cert_file.parse().unwrap(),
            key_file: SERVER_CONFIG.quic.certificate.key_file.parse().unwrap(),
        }
    }
}

impl Default for TcpConfig {
    fn default() -> TcpConfig {
        TcpConfig {
            enabled: SERVER_CONFIG.tcp.enabled,
            address: SERVER_CONFIG.tcp.address.parse().unwrap(),
            tls: TcpTlsConfig::default(),
        }
    }
}

impl Default for TcpTlsConfig {
    fn default() -> TcpTlsConfig {
        TcpTlsConfig {
            enabled: SERVER_CONFIG.tcp.tls.enabled,
            certificate: SERVER_CONFIG.tcp.tls.certificate.parse().unwrap(),
            password: SERVER_CONFIG.tcp.tls.password.parse().unwrap(),
        }
    }
}

impl Default for HttpConfig {
    fn default() -> HttpConfig {
        HttpConfig {
            enabled: SERVER_CONFIG.http.enabled,
            address: SERVER_CONFIG.http.address.parse().unwrap(),
            max_request_size: SERVER_CONFIG.http.max_request_size.parse().unwrap(),
            cors: HttpCorsConfig::default(),
            jwt: HttpJwtConfig::default(),
            metrics: HttpMetricsConfig::default(),
            tls: HttpTlsConfig::default(),
        }
    }
}

impl Default for HttpCorsConfig {
    fn default() -> HttpCorsConfig {
        HttpCorsConfig {
            enabled: SERVER_CONFIG.http.cors.enabled,
            allowed_methods: SERVER_CONFIG
                .http
                .cors
                .allowed_methods
                .iter()
                .map(|s| s.parse().unwrap())
                .collect(),
            allowed_origins: SERVER_CONFIG
                .http
                .cors
                .allowed_origins
                .iter()
                .map(|s| s.parse().unwrap())
                .collect(),
            allowed_headers: SERVER_CONFIG
                .http
                .cors
                .allowed_headers
                .iter()
                .map(|s| s.parse().unwrap())
                .collect(),
            exposed_headers: SERVER_CONFIG
                .http
                .cors
                .exposed_headers
                .iter()
                .map(|s| s.parse().unwrap())
                .collect(),
            allow_credentials: SERVER_CONFIG.http.cors.allow_credentials,
            allow_private_network: SERVER_CONFIG.http.cors.allow_private_network,
        }
    }
}

impl Default for HttpJwtConfig {
    fn default() -> HttpJwtConfig {
        HttpJwtConfig {
            algorithm: SERVER_CONFIG.http.jwt.algorithm.parse().unwrap(),
            issuer: SERVER_CONFIG.http.jwt.issuer.parse().unwrap(),
            audience: SERVER_CONFIG.http.jwt.audience.parse().unwrap(),
            valid_issuers: SERVER_CONFIG
                .http
                .jwt
                .valid_issuers
                .iter()
                .map(|s| s.parse().unwrap())
                .collect(),
            valid_audiences: SERVER_CONFIG
                .http
                .jwt
                .valid_audiences
                .iter()
                .map(|s| s.parse().unwrap())
                .collect(),
            access_token_expiry: SERVER_CONFIG.http.jwt.access_token_expiry.parse().unwrap(),
            clock_skew: SERVER_CONFIG.http.jwt.clock_skew.parse().unwrap(),
            not_before: SERVER_CONFIG.http.jwt.not_before.parse().unwrap(),
            encoding_secret: SERVER_CONFIG.http.jwt.encoding_secret.parse().unwrap(),
            decoding_secret: SERVER_CONFIG.http.jwt.decoding_secret.parse().unwrap(),
            use_base64_secret: SERVER_CONFIG.http.jwt.use_base_64_secret,
        }
    }
}

impl Default for HttpMetricsConfig {
    fn default() -> HttpMetricsConfig {
        HttpMetricsConfig {
            enabled: SERVER_CONFIG.http.metrics.enabled,
            endpoint: SERVER_CONFIG.http.metrics.endpoint.parse().unwrap(),
        }
    }
}

impl Default for HttpTlsConfig {
    fn default() -> HttpTlsConfig {
        HttpTlsConfig {
            enabled: SERVER_CONFIG.http.tls.enabled,
            cert_file: SERVER_CONFIG.http.tls.cert_file.parse().unwrap(),
            key_file: SERVER_CONFIG.http.tls.key_file.parse().unwrap(),
        }
    }
}

impl Default for MessageSaverConfig {
    fn default() -> MessageSaverConfig {
        MessageSaverConfig {
            enabled: SERVER_CONFIG.message_saver.enabled,
            enforce_fsync: SERVER_CONFIG.message_saver.enforce_fsync,
            interval: SERVER_CONFIG.message_saver.interval.parse().unwrap(),
        }
    }
}

impl Default for PersonalAccessTokenConfig {
    fn default() -> PersonalAccessTokenConfig {
        PersonalAccessTokenConfig {
            max_tokens_per_user: SERVER_CONFIG.personal_access_token.max_tokens_per_user as u32,
            cleaner: PersonalAccessTokenCleanerConfig::default(),
        }
    }
}

impl Default for PersonalAccessTokenCleanerConfig {
    fn default() -> PersonalAccessTokenCleanerConfig {
        PersonalAccessTokenCleanerConfig {
            enabled: SERVER_CONFIG.personal_access_token.cleaner.enabled,
            interval: SERVER_CONFIG
                .personal_access_token
                .cleaner
                .interval
                .parse()
                .unwrap(),
        }
    }
}

impl Default for SystemConfig {
    fn default() -> SystemConfig {
        SystemConfig {
            path: SERVER_CONFIG.system.path.parse().unwrap(),
            backup: BackupConfig::default(),
            database: None,
            runtime: RuntimeConfig::default(),
            logging: LoggingConfig::default(),
            cache: CacheConfig::default(),
            stream: StreamConfig::default(),
            encryption: EncryptionConfig::default(),
            topic: TopicConfig::default(),
            partition: PartitionConfig::default(),
            segment: SegmentConfig::default(),
            state: StateConfig::default(),
            compression: CompressionConfig::default(),
            message_deduplication: MessageDeduplicationConfig::default(),
            recovery: RecoveryConfig::default(),
        }
    }
}

impl Default for BackupConfig {
    fn default() -> BackupConfig {
        BackupConfig {
            path: SERVER_CONFIG.system.backup.path.parse().unwrap(),
            compatibility: CompatibilityConfig::default(),
        }
    }
}

impl Default for CompatibilityConfig {
    fn default() -> Self {
        CompatibilityConfig {
            path: SERVER_CONFIG
                .system
                .backup
                .compatibility
                .path
                .parse()
                .unwrap(),
        }
    }
}

impl Default for HeartbeatConfig {
    fn default() -> HeartbeatConfig {
        HeartbeatConfig {
            enabled: SERVER_CONFIG.heartbeat.enabled,
            interval: SERVER_CONFIG.heartbeat.interval.parse().unwrap(),
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> RuntimeConfig {
        RuntimeConfig {
            path: SERVER_CONFIG.system.runtime.path.parse().unwrap(),
        }
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        CompressionConfig {
            allow_override: SERVER_CONFIG.system.compression.allow_override,
            default_algorithm: SERVER_CONFIG
                .system
                .compression
                .default_algorithm
                .parse()
                .unwrap(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> LoggingConfig {
        LoggingConfig {
            path: SERVER_CONFIG.system.logging.path.parse().unwrap(),
            level: SERVER_CONFIG.system.logging.level.parse().unwrap(),
            max_size: SERVER_CONFIG.system.logging.max_size.parse().unwrap(),
            retention: SERVER_CONFIG.system.logging.retention.parse().unwrap(),
            sysinfo_print_interval: SERVER_CONFIG
                .system
                .logging
                .sysinfo_print_interval
                .parse()
                .unwrap(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> CacheConfig {
        CacheConfig {
            enabled: SERVER_CONFIG.system.cache.enabled,
            size: SERVER_CONFIG.system.cache.size.parse().unwrap(),
        }
    }
}

impl Default for EncryptionConfig {
    fn default() -> EncryptionConfig {
        EncryptionConfig {
            enabled: SERVER_CONFIG.system.encryption.enabled,
            key: SERVER_CONFIG.system.encryption.key.parse().unwrap(),
        }
    }
}

impl Default for StreamConfig {
    fn default() -> StreamConfig {
        StreamConfig {
            path: SERVER_CONFIG.system.stream.path.parse().unwrap(),
        }
    }
}

impl Default for TopicConfig {
    fn default() -> TopicConfig {
        TopicConfig {
            path: SERVER_CONFIG.system.topic.path.parse().unwrap(),
            max_size: SERVER_CONFIG.system.topic.max_size.parse().unwrap(),
            delete_oldest_segments: SERVER_CONFIG.system.topic.delete_oldest_segments,
        }
    }
}

impl Default for PartitionConfig {
    fn default() -> PartitionConfig {
        PartitionConfig {
            path: SERVER_CONFIG.system.partition.path.parse().unwrap(),
            messages_required_to_save: SERVER_CONFIG.system.partition.messages_required_to_save
                as u32,
            enforce_fsync: SERVER_CONFIG.system.partition.enforce_fsync,
            validate_checksum: SERVER_CONFIG.system.partition.validate_checksum,
        }
    }
}

impl Default for SegmentConfig {
    fn default() -> SegmentConfig {
        SegmentConfig {
            size: SERVER_CONFIG.system.segment.size.parse().unwrap(),
            cache_indexes: SERVER_CONFIG.system.segment.cache_indexes,
            cache_time_indexes: SERVER_CONFIG.system.segment.cache_time_indexes,
            message_expiry: SERVER_CONFIG.system.segment.message_expiry.parse().unwrap(),
            archive_expired: SERVER_CONFIG.system.segment.archive_expired,
        }
    }
}

impl Default for StateConfig {
    fn default() -> StateConfig {
        StateConfig {
            enforce_fsync: SERVER_CONFIG.system.state.enforce_fsync,
        }
    }
}

impl Default for MessageDeduplicationConfig {
    fn default() -> MessageDeduplicationConfig {
        MessageDeduplicationConfig {
            enabled: SERVER_CONFIG.system.message_deduplication.enabled,
            max_entries: SERVER_CONFIG.system.message_deduplication.max_entries as u64,
            expiry: SERVER_CONFIG
                .system
                .message_deduplication
                .expiry
                .parse()
                .unwrap(),
        }
    }
}

impl Default for RecoveryConfig {
    fn default() -> RecoveryConfig {
        RecoveryConfig {
            recreate_missing_state: SERVER_CONFIG.system.recovery.recreate_missing_state,
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> TelemetryConfig {
        TelemetryConfig {
            enabled: SERVER_CONFIG.telemetry.enabled,
            service_name: SERVER_CONFIG.telemetry.service_name.parse().unwrap(),
            logs: TelemetryLogsConfig::default(),
            traces: TelemetryTracesConfig::default(),
        }
    }
}

impl Default for TelemetryLogsConfig {
    fn default() -> TelemetryLogsConfig {
        TelemetryLogsConfig {
            transport: SERVER_CONFIG.telemetry.logs.transport.parse().unwrap(),
            endpoint: SERVER_CONFIG.telemetry.logs.endpoint.parse().unwrap(),
        }
    }
}

impl Default for TelemetryTracesConfig {
    fn default() -> TelemetryTracesConfig {
        TelemetryTracesConfig {
            transport: SERVER_CONFIG.telemetry.traces.transport.parse().unwrap(),
            endpoint: SERVER_CONFIG.telemetry.traces.endpoint.parse().unwrap(),
        }
    }
}
