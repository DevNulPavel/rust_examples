use std::{
    net::{
        IpAddr
    },
    str::{
        FromStr
    }
};
use uuid::{
    Uuid
};
use crate::{
    auth::{
        SigKey
    }
};

fn get_port(var: &'static str, default: u16) -> u16 {
    if let Ok(port) = std::env::var(var) {
        port
            .parse()
            .unwrap_or_else(|_| {
                panic!("invalid port ENV {}={}", var, port);
            })
    } else {
        default
    }
}

/// Глобальные настройки сервиса
pub struct Config {
    /// На каких хостах разрешаютс туннели:
    /// i.e:    baz.com => *.baz.com
    ///         foo.bar => *.foo.bar
    pub allowed_hosts: Vec<String>,

    /// Какие поддомены мы всегда блокируем
    /// i.e:    dashboard.tunnelto.dev
    pub blocked_sub_domains: Vec<String>,

    /// Порт для удаленных потоков (конечных пользователей)
    pub remote_port: u16,

    /// Порт управления сервером
    pub control_port: u16,

    /// Внутренний порт для сообщений от сервера к серверу
    pub internal_network_port: u16,

    /// Наш ключ подписи
    pub master_sig_key: SigKey,

    /// Днс адрес для обнаружения внутренних связей
    pub gossip_dns_host: Option<String>,

    // Ключ для мониторинга Honeycomb
    pub honeycomb_api_key: Option<String>,

    /// Идентификатор данного экземпляра сервера
    pub instance_id: String,

    /// Заблокированные адреса
    pub blocked_ips: Vec<IpAddr>,
}

impl Config {
    pub fn from_env() -> Config {
        let allowed_hosts = std::env::var("ALLOWED_HOSTS")
            .map(|s| s.split(",").map(String::from).collect())
            .unwrap_or(vec![]);

        let blocked_sub_domains = std::env::var("BLOCKED_SUB_DOMAINS")
            .map(|s| s.split(",").map(String::from).collect())
            .unwrap_or(vec![]);

        let master_sig_key = if let Ok(key) = std::env::var("MASTER_SIG_KEY") {
            SigKey::from_hex(&key).expect("invalid master key: not hex or length incorrect")
        } else {
            tracing::warn!("WARNING! generating ephemeral signature key!");
            SigKey::generate()
        };

        let gossip_dns_host = std::env::var("FLY_APP_NAME")
            .map(|app_name| format!("global.{}.internal", app_name))
            .ok();

        let honeycomb_api_key = std::env::var("HONEYCOMB_API_KEY").ok();
        let instance_id = std::env::var("FLY_ALLOC_ID").unwrap_or(Uuid::new_v4().to_string());
        let blocked_ips = std::env::var("BLOCKED_IPS")
            .map(|s| {
                s.split(",")
                    .map(IpAddr::from_str)
                    .filter_map(Result::ok)
                    .collect()
            })
            .unwrap_or(vec![]);

        Config {
            allowed_hosts,
            blocked_sub_domains,
            control_port: get_port("CTRL_PORT", 5000),
            remote_port: get_port("PORT", 8080),
            internal_network_port: get_port("NET_PORT", 6000),
            master_sig_key,
            gossip_dns_host,
            honeycomb_api_key,
            instance_id,
            blocked_ips,
        }
    }
}
