mod buffer_pool;
mod config;
mod directive;
mod error;
mod file_server;
mod request;
mod response;
mod reverse_proxy;
mod server;

////////////////////////////////////////////////////////////////////////////////

use crate::{
    config::{build_config, Directive},
    error::CbltError,
    server::{server_init, Server},
};
use anyhow::Context;
use clap::Parser;
use kdl::KdlDocument;
use log::{debug, error, info};
use server::Cert;
use std::{collections::HashMap, num::NonZeroU16, str};
use tokio::fs;
use tokio::runtime::Builder;
use tracing::{instrument, Level};
use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

////////////////////////////////////////////////////////////////////////////////

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Путь к файлику конфигурации
    // default_value = "./Cbltfile"
    #[arg(long)]
    cfg: String,

    /// Ограничение на максимум подключений
    // default_value_t = 10000
    #[arg(long)]
    max_connections: usize,
}

////////////////////////////////////////////////////////////////////////////////

fn main() -> anyhow::Result<()> {
    // Разный код для релизной и тестовой сборки
    {
        #[cfg(debug_assertions)]
        only_in_debug();

        #[cfg(not(debug_assertions))]
        only_in_production();
    }

    // Распарсим сразу аргументы
    let args = Args::parse();

    // Определяем доступную нам многопоточность
    let num_cpus = std::thread::available_parallelism()?.get();
    info!("Workers amount: {}", num_cpus);

    // Создаем рантайм для tokio
    let runtime = Builder::new_multi_thread()
        .worker_threads(num_cpus)
        .enable_all()
        .build()?;

    // Запускаем теперь нашу футуру с сервером непосредственно уже
    // и ждем завершения
    runtime.block_on(async {
        server(args).await?;
        Ok(())
    })
}

////////////////////////////////////////////////////////////////////////////////

/// Непосредственно сама асинхронная часть кода
async fn server(args: Args) -> anyhow::Result<()> {
    // Количество коннектов
    let max_connections: usize = args.max_connections;
    info!("Max connections: {}", max_connections);

    let config = {
        // Вычитываем файлик с контентом конфига асинхронно
        // TODO: Хотя можно было бы еще при инициализации это сделать
        let cbltfile_content = fs::read_to_string(&args.cfg)
            .await
            .context("Failed to read Cbltfile")?;

        // Парсим теперь конфиг
        let doc: KdlDocument = cbltfile_content.parse()?;

        // Текст конфига нам уже не нужен, так что удалим его
        drop(cbltfile_content);

        // После парсинга можем уже создать конфиг
        build_config(&doc)?
    };

    // Маппинг портов в серверы
    let mut servers: HashMap<u16, Server> = HashMap::new(); // Port -> Server

    // Перебираем теперь хосты и настройки из конфига
    for (host_str, directives) in config {
        // Просматриваем все директивы для определения
        // необходимости TLS режима
        let cert = directives.iter().find_map(|d| {
            // Есть ли там директива для TLS ?
            if let Directive::Tls { cert, key } = d {
                // Параметры для TLS
                let params = Cert {
                    cert_path: cert.to_string(),
                    key_path: key.to_string(),
                };

                Some(params)
            } else {
                None
            }
        });

        // Пробуем распарсить информацию о том, для какого это хоста у нас сделано?
        let parsed_host = ParsedHost::from_str(&host_str);

        // Определяем точный порт, который будем использовать.
        // Используем стандартные если не было указано в конфиге.
        let port = parsed_host.port.unwrap_or_else(|| match &cert {
            None => 80,
            Some(_) => 443,
        });

        // Чисто для отладки
        debug!("Host: {}, Port: {}", host_str, port);

        // Находим в хешмапе нужный элемент по комеру порта
        servers
            .entry(port)
            // Затем делаем модификацию параметров уже имеющихся
            // если что-то уже было
            .and_modify(|s| {
                // Берем мутабельную ссылку на текущий список хостов
                let hosts = &mut s.hosts;

                // TODO: Проверка дублей?
                // TODO: Не добавлять ли сюда по host_str?
                // Добавляем туда еще список директив на память
                hosts.insert(parsed_host.host.clone(), directives.clone());

                // Обновляем данные по сертификату
                s.cert = cert.clone();
            })
            // Либо создаем запись с нуля если нету такой еще
            .or_insert({
                // Создаем мап с хостами
                let mut hosts = HashMap::new();

                // Делаем запись с хостом
                hosts.insert(parsed_host.host, directives);

                Server { port, hosts, cert }
            });
    }

    // Выведем получившийся конфиг конечный для проверки
    debug!("{:#?}", servers);

    for (_, server) in servers {
        tokio::spawn(async move {
            match server_init(&server, max_connections).await {
                Ok(_) => {}
                Err(err) => {
                    error!("Error: {}", err);
                }
            }
        });
    }
    info!("CBLT started");
    tokio::signal::ctrl_c().await?;
    info!("CBLT stopped");

    Ok(())
}

#[allow(dead_code)]
pub fn only_in_debug() {
    let _ =
        env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("debug")).try_init();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE) // Set the maximum log level
        .with_span_events(FmtSpan::CLOSE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

#[allow(dead_code)]
fn only_in_production() {
    let _ =
        env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("info")).try_init();
}

#[cfg_attr(debug_assertions, instrument(level = "trace", skip_all))]
fn matches_pattern(pattern: &str, path: &str) -> bool {
    if pattern == "*" {
        true
    } else if pattern.ends_with("*") {
        let prefix = &pattern[..pattern.len() - 1];
        path.starts_with(prefix)
    } else {
        pattern == path
    }
}

pub struct ParsedHost {
    pub host: String,
    pub port: Option<u16>,
}

impl ParsedHost {
    fn from_str(host_str: &str) -> Self {
        if let Some((host_part, port_part)) = host_str.split_once(':') {
            let port = port_part.parse().ok();
            ParsedHost {
                host: host_part.to_string(),
                port,
            }
        } else {
            ParsedHost {
                host: host_str.to_string(),
                port: None,
            }
        }
    }
}
