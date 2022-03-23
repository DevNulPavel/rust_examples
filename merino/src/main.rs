#![forbid(unsafe_code)]
#[macro_use]
extern crate log;

use merino::*;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use clap::{ArgGroup, Parser};

/// Logo to be printed at when merino is run
const LOGO: &str = r"
                      _
  _ __ ___   ___ _ __(_)_ __   ___
 | '_ ` _ \ / _ \ '__| | '_ \ / _ \
 | | | | | |  __/ |  | | | | | (_) |
 |_| |_| |_|\___|_|  |_|_| |_|\___/

 A SOCKS5 Proxy server written in Rust
";

#[derive(Parser, Debug)]
#[clap(version)]
#[clap(group(
    ArgGroup::new("auth")
        .required(true)
        .args(&["no-auth", "users"]),
))]
struct Opt {
    #[clap(short, long, default_value_t = 1080)]
    /// Set port to listen on
    port: u16,

    #[clap(short, long, default_value = "127.0.0.1")]
    /// Set ip to listen on
    ip: String,

    #[clap(long)]
    /// Allow unauthenticated connections
    no_auth: bool,

    #[clap(short, long)]
    /// CSV File with username/password pairs
    users: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", LOGO);

    // Парсим параметры
    let opt = Opt::parse();

    // Выставляем переменные окружения `RUST_LOG` если не установлено
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "merino=INFO");
    }

    // Инициализируем систему логирования
    pretty_env_logger::init_timed();

    // Выставляем способы аутентификации
    let mut auth_methods: Vec<u8> = Vec::new();

    // Если есть флаг отсутствия аутентификации
    if opt.no_auth {
        // Добавляем нулевую аутентификацию 
        auth_methods.push(merino::AuthMethods::NoAuth as u8);
    }

    // Если надо, то включаем аутентификацию через юзера и пароль
    let authed_users: Result<Vec<User>, Box<dyn Error>> = match opt.users {
        Some(users_file) => {
            // Добавляем аутентификацию по паролю и логину
            auth_methods.push(AuthMethods::UserPass as u8);

            // Читаем файлик
            let file = std::fs::File::open(users_file)?;

            // Массив пользователей
            let mut users: Vec<User> = Vec::new();

            // Читаем файлик CSV
            let mut rdr = csv::Reader::from_reader(file);

            // Читаем из файлика
            for result in rdr.deserialize() {
                let record: User = result?;

                trace!("Loaded user: {}", record.username);
                users.push(record);
            }

            Ok(users)
        }
        _ => Ok(Vec::new()),
    };

    // Список юзеров
    let authed_users = authed_users?;

    // Создаем прокси-сервер
    let mut merino = Merino::new(opt.port, &opt.ip, auth_methods, authed_users, None).await?;

    // Start Proxies
    merino.serve().await;

    Ok(())
}
