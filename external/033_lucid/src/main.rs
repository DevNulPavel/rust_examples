#[macro_use]
extern crate clap;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

extern crate block_modes;
extern crate hex;
extern crate serpent;

mod configuration;
mod kvstore;
mod lucid;
mod server;

use self::lucid::Lucid;
use configuration::{Claims, Configuration, LogOutput};

use std::{
    fmt,
    fs::{self, File},
    path::Path,
};

use app_dirs::AppDirsError;
use chrono::{DateTime, Duration, Utc};
use clap::{App, ArgMatches};
use fern::colors::{Color, ColoredLevelConfig};
use fern::Dispatch;
use jsonwebtoken::Header;
use rand::Rng;
use ring::digest;
use snafu::{ResultExt, Snafu};

const BANNER: &'static str = r###"
 ██╗    ██╗   ██╗ ██████╗██╗██████╗     ██╗  ██╗██╗   ██╗
 ██║    ██║   ██║██╔════╝██║██╔══██╗    ██║ ██╔╝██║   ██║
 ██║    ██║   ██║██║     ██║██║  ██║    ██╔═██╗ ╚██╗ ██╔╝
 ██████╗╚██████╔╝╚██████╗██║██████╔╝    ██║  ██╗ ╚████╔╝
 ╚═════╝ ╚═════╝  ╚═════╝╚═╝╚═════╝     ╚═╝  ╚═╝  ╚═══╝

A Fast, Secure and Distributed KV store with an HTTP API.
Written in Rust, Fork us on GitHub (https://github.com/lucid-kv)
"###;

const CREDITS: &'static str = "\
                               +-----------------+-----------------------+--------------------+\n\
                               |               Lucid KV Development Credits                   |\n\
                               +-----------------+-----------------------+--------------------+\n\
                               | Clint Mourlevat | me@clint.network      | Lucid Founder      |\n\
                               | Jonathan Serra  | jonathan@blocs.fr     | Core Development   |\n\
                               | CephalonRho     | CephalonRho@gmail.com | Core Development   |\n\
                               | Rigwild         | me@rigwild.dev        | Web UI Development |\n\
                               +-----------------+-----------------------+--------------------+";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let long_version = format!(
        "{}\n{}\n\nYou can send a tips here: 3BxEYn4RZ3iYETcFpN7nA6VqCY4Hz1tSUK",
        crate_version!(),
        CREDITS
    );

    // Подгружаем yaml на этапе компиляции для описания параметров
    let cli_yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(&cli_yaml)
        .version(crate_version!())
        .long_version(long_version.as_str());
    let matches = match app.get_matches_safe() {
        Ok(x) => x,
        Err(clap::Error {
            kind: clap::ErrorKind::HelpDisplayed,
            message,
            ..
        })
        | Err(clap::Error {
            kind: clap::ErrorKind::VersionDisplayed,
            message,
            ..
        })
        | Err(clap::Error {
            kind: clap::ErrorKind::MissingArgumentOrSubcommand,
            message,
            ..
        }) => {
            println!("{}", message);
            return Ok(());
        }
        Err(e) => return Err(Error::ParseCli { source: e }),
    };

    // Получаем путь к файлику конфига
    let config_path = {
        if let Some(config) = matches.value_of("config") {
            Path::new(config).to_path_buf()
        } else {
            Configuration::get_path().context(GetConfigDir)?
        }
    };

    // Парсим файлик конфига, либо стандартный конфиг если нет файлика
    let config = if config_path.exists() {
        serde_yaml::from_reader(File::open(&config_path).context(OpenConfigFile)?)
            .context(ReadConfigFile)?
    } else {
        Configuration::default()
    };

    // Если у нас не отключен показ стартового баннера, тогда
    if !matches.is_present("no-banner") && config.general.show_banner {
        println!("{}", BANNER);
    }

    // Конфиг цветного вывода логов
    let logging_colors = ColoredLevelConfig::new()
        .debug(Color::BrightMagenta)
        .info(Color::BrightCyan)
        .warn(Color::BrightYellow)
        .error(Color::BrightRed);

    // Настраиваем цепочку вывода логов в разные таргеты
    let mut dispatch = Dispatch::new();
    for output in &config.logging.outputs {
        dispatch = match output {
            LogOutput::File { path } => {
                dispatch.chain(create_format_dispatch(None).chain(fern::log_file(path).unwrap()))
            }
            LogOutput::Stdout { colored } => {
                if *colored {
                    dispatch.chain(
                        create_format_dispatch(Some(logging_colors)).chain(std::io::stdout()),
                    )
                } else {
                    dispatch.chain(create_format_dispatch(None).chain(std::io::stdout()))
                }
            }
            LogOutput::Stderr { colored } => {
                if *colored {
                    dispatch.chain(
                        create_format_dispatch(Some(logging_colors)).chain(std::io::stderr()),
                    )
                } else {
                    dispatch.chain(create_format_dispatch(None).chain(std::io::stderr()))
                }
            }
        };
    }

    dispatch.apply().expect("Couldn't start logger.");
    log::set_max_level(config.logging.level);
    if let Err(e) = start(matches, config, &config_path).await {
        error!("fatal: {}", e);
    }
    Ok(())
}

async fn start(
    matches: ArgMatches<'_>,
    config: Configuration,
    config_path: &Path,
) -> Result<(), Error> {
    if let Some(init_matches) = matches.subcommand_matches("init") {
        if config_path.exists() && !init_matches.is_present("force") {
            return Err(Error::AlreadyInitialized);
        } else {
            // Стандартная конфигурация
            let mut config = Configuration::default();
            // Генерируем секретный ключ для работы
            let secret_key = generate_secret_key();
            // Создаем JWT токен на основе секретного ключа, подписывая токен секретным ключем
            // Время жизни делаем бесконечным, но по факту - 3 года для клиентов
            config.authentication.root_token = issue_jwt(&secret_key, None)?;
            // Сохраняем ключ секретный на потом
            config.authentication.secret_key = secret_key;
            // Создаем директорию по пути конфига
            fs::create_dir_all(config_path.parent().unwrap()).context(CreateConfigDir)?;
            // Пишем файлик конфига туда, сохраняя наши секретные данные
            serde_yaml::to_writer(
                File::create(&config_path).context(CreateConfigFile)?,
                &config,
            )
            .context(WriteConfigFile)?;
            info!(
                "Lucid successfully initialized in {}",
                config_path.to_string_lossy()
            );
        }
    }
    // Нужен ли нам сервер?
    if let Some(_) = matches.subcommand_matches("server") {
        if config_path.exists() {
            Lucid::new(config).run().await.context(RunServer)?;
        } else {
            return Err(Error::ConfigurationNotFound);
        }
    }
    if let Some(_) = matches.subcommand_matches("settings") {
        if config_path.exists() {
            println!(
                "Configuration location: {}\n\n{}",
                &config_path.to_str().unwrap(),
                fs::read_to_string(&config_path).context(OpenConfigFile)?
            );
        } else {
            return Err(Error::ConfigurationNotFound);
        }
    }
    Ok(())
}

fn generate_secret_key() -> String {
    // Рандомный хеш SHA256 от рандомных данных
    let secret_key_bytes = digest::digest(&digest::SHA256, &rand::thread_rng().gen::<[u8; 32]>());
    secret_key_bytes.as_ref().iter().fold(
        // Создаем строку в двое большего размера
        String::with_capacity(secret_key_bytes.as_ref().len() * 2),

        |mut acc, x| {
            // Записываем шестнадцетиричные символы дополняемые нулями в начале
            acc.push_str(&format!("{:0>2x}", x));
            acc
        },
    )
}

/// Создаем JWT токен с секретным ключем
fn issue_jwt(secret_key: &str, expiration: Option<DateTime<Utc>>) -> Result<String, Error> {
    // Создаем токен с данными
    jsonwebtoken::encode(
        // Хедер делаем стандартный со стандартным алгоритмом шифрования
        &Header::default(),
        // Данные
        &Claims {
            // Кто выпустил токен?
            sub: String::from("Lucid Root Token"),
            // Адрес
            iss: String::from("http://localhost:7021/"), // TODO: check issuer, maybe set the proper uri
            // Время выпуска токена
            iat: Utc::now().timestamp(),
            // Время жизни токена в зависимости от параметра
            exp: match expiration {
                Some(exp) => exp.timestamp(),
                // Либо на 3 года
                None => (Utc::now() + Duration::weeks(52 * 3)).timestamp(),
            },
        },
        // Подписываем токен секретным ключем
        secret_key.as_ref(),
    )
    .context(EncodeJwt)
}

fn create_format_dispatch(colors: Option<ColoredLevelConfig>) -> Dispatch {
    Dispatch::new().format(move |out, message, record| {
        if let Some(colors) = colors {
            out.finish(format_args!(
                "{} {} [{}] {}",
                Utc::now().format("%Y/%m/%d %H:%M:%S"),
                colors.color(record.level()),
                record.target(),
                message
            ))
        } else {
            out.finish(format_args!(
                "{} {} [{}] {}",
                Utc::now().format("%Y/%m/%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        }
    })
}

#[derive(Snafu)]
pub enum Error {
    #[snafu(display("{}", source))]
    ParseCli { source: clap::Error },
    #[snafu(display("{}", source))]
    RunServer { source: std::io::Error },
    #[snafu(display("Configuration file not found."))]
    ConfigurationNotFound,
    #[snafu(display("The Lucid node has already been initialized."))]
    AlreadyInitialized,
    #[snafu(display("Unable to get the Lucid configuration directory: {}", source))]
    GetConfigDir { source: AppDirsError },
    #[snafu(display("Unable to create the Lucid configuration directory: {}", source))]
    CreateConfigDir { source: std::io::Error },
    #[snafu(display("Unable to create the Lucid configuration file: {}", source))]
    CreateConfigFile { source: std::io::Error },
    #[snafu(display("Unable to write the Lucid configuration file: {}", source))]
    WriteConfigFile { source: serde_yaml::Error },
    #[snafu(display("Unable to open the Lucid configuration file: {}", source))]
    OpenConfigFile { source: std::io::Error },
    #[snafu(display("Unable to read the Lucid configuration file: {}", source))]
    ReadConfigFile { source: serde_yaml::Error },
    #[snafu(display("Error while encoding the JWT root token: {}", source))]
    EncodeJwt { source: jsonwebtoken::errors::Error },
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
