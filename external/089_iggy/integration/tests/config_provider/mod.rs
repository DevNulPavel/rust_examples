use integration::file::{file_exists, get_root_path};
use serial_test::serial;
use server::configs::config_provider::{ConfigProvider, FileConfigProvider};
use std::env;

async fn scenario_parsing_from_file(extension: &str) {
    let mut config_path = get_root_path().join("../configs/server");
    assert!(config_path.set_extension(extension), "Cannot set extension");
    let config_path = config_path.as_path().display().to_string();
    let config_provider = FileConfigProvider::new(config_path.clone());
    assert!(
        file_exists(&config_path),
        "Config file not found: {}",
        config_path
    );
    assert!(
        config_provider.load_config().await.is_ok(),
        "ConfigProvider failed to parse config from {}",
        config_path
    );
}

#[tokio::test]
async fn validate_server_config_toml_from_repository() {
    scenario_parsing_from_file("toml").await;
}

#[tokio::test]
async fn validate_server_config_json_from_repository() {
    scenario_parsing_from_file("json").await;
}

// This test needs to be run in serial because it modifies the environment variables
// which are shared, since all tests run in parallel by default.
#[serial]
#[tokio::test]
async fn validate_custom_env_provider() {
    let expected_datagram_send_buffer_size = "1.00 KB";
    let expected_quic_certificate_self_signed = false;
    let expected_http_enabled = false;
    let expected_tcp_enabled = "false";
    let expected_message_saver_enabled = false;
    let expected_message_expiry = "10s";

    env::set_var(
        "IGGY_QUIC_DATAGRAM_SEND_BUFFER_SIZE",
        expected_datagram_send_buffer_size,
    );
    env::set_var(
        "IGGY_QUIC_CERTIFICATE_SELF_SIGNED",
        expected_quic_certificate_self_signed.to_string(),
    );
    env::set_var("IGGY_HTTP_ENABLED", expected_http_enabled.to_string());
    env::set_var("IGGY_TCP_ENABLED", expected_tcp_enabled);
    env::set_var(
        "IGGY_MESSAGE_SAVER_ENABLED",
        expected_message_saver_enabled.to_string(),
    );
    env::set_var("IGGY_SYSTEM_SEGMENT_MESSAGE_EXPIRY", "10s");

    let config_path = get_root_path().join("../configs/server.toml");
    let file_config_provider = FileConfigProvider::new(config_path.as_path().display().to_string());
    let config = file_config_provider
        .load_config()
        .await
        .expect("Failed to load default server.toml config");

    assert_eq!(
        config.quic.datagram_send_buffer_size.to_string(),
        expected_datagram_send_buffer_size
    );
    assert_eq!(
        config.quic.certificate.self_signed,
        expected_quic_certificate_self_signed
    );
    assert_eq!(config.http.enabled, expected_http_enabled);
    assert_eq!(config.tcp.enabled.to_string(), expected_tcp_enabled);
    assert_eq!(config.message_saver.enabled, expected_message_saver_enabled);
    assert_eq!(
        config.system.segment.message_expiry.to_string(),
        expected_message_expiry
    );

    env::remove_var("IGGY_QUIC_DATAGRAM_SEND_BUFFER_SIZE");
    env::remove_var("IGGY_QUIC_CERTIFICATE_SELF_SIGNED");
    env::remove_var("IGGY_HTTP_ENABLED");
    env::remove_var("IGGY_TCP_ENABLED");
    env::remove_var("IGGY_MESSAGE_SAVER_ENABLED");
    env::remove_var("IGGY_SYSTEM_RETENTION_POLICY_MESSAGE_EXPIRY");
}
