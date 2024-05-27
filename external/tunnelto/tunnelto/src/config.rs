use super::*;
use structopt::StructOpt;

const HOST_ENV: &'static str = "CTRL_HOST";
const PORT_ENV: &'static str = "CTRL_PORT";
const TLS_OFF_ENV: &'static str = "CTRL_TLS_OFF";

const DEFAULT_HOST: &'static str = "tunnelto.dev";
const DEFAULT_CONTROL_HOST: &'static str = "wormhole.tunnelto.dev";
const DEFAULT_CONTROL_PORT: &'static str = "443";

const SETTINGS_DIR: &'static str = ".tunnelto";
const SECRET_KEY_FILE: &'static str = "key.token";

/// Command line arguments
#[derive(Debug, StructOpt)]
#[structopt(
    name = "tunnelto",
    author = "Alex Grinman <alex@tunnelto.dev>",
    about = "Expose your local web server to the internet with a public url."
)]
struct Opts {
    /// A level of verbosity, and can be used multiple times
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    #[structopt(subcommand)]
    command: Option<SubCommand>,

    /// Sets an API authentication key to use for this tunnel
    #[structopt(short = "k", long = "key")]
    key: Option<String>,

    /// Specify a sub-domain for this tunnel
    #[structopt(short = "s", long = "subdomain")]
    sub_domain: Option<String>,

    /// Sets the HOST (i.e. localhost) to forward incoming tunnel traffic to
    #[structopt(long = "host", default_value = "localhost")]
    local_host: String,

    /// Sets the SCHEME (i.e. http or https) to forward incoming tunnel traffic to
    #[structopt(long = "scheme", default_value = "http")]
    scheme: String,

    /// Sets the port to forward incoming tunnel traffic to on the target host
    #[structopt(short = "p", long = "port", default_value = "8000")]
    port: u16,

    /// Sets the address of the local introspection dashboard
    #[structopt(long = "dashboard-port")]
    dashboard_port: Option<u16>,
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    /// Store the API Authentication key
    SetAuth {
        /// Sets an API authentication key on disk for future use
        #[structopt(short = "k", long = "key")]
        key: String,
    },
}

/// Config
#[derive(Debug, Clone)]
pub struct Config {
    pub client_id: ClientId,
    pub control_url: String,
    pub local_host: String,
    pub scheme: String,
    pub host: String,
    pub local_port: u16,
    pub sub_domain: Option<String>,
    pub secret_key: Option<SecretKey>,
    pub tls_off: bool,
    pub first_run: bool,
    pub dashboard_port: u16,
    pub verbose: bool,
}

impl Config {
    /// Parse the URL to use to connect to the wormhole control server
    pub fn get() -> Result<Config, ()> {
        // Парсим сначала параметры из коммандной строки
        let opts: Opts = Opts::from_args();

        // Если включен подробный вывод, значит выставляем переменную окружения для отладочного вывода
        if opts.verbose {
            std::env::set_var("RUST_LOG", "tunnelto=debug");
        }

        // Инициалазируем логирование
        // Лучше бы инициализировать логи отдельно после конфига
        pretty_env_logger::init();

        let (secret_key, sub_domain) = match opts.command {
            // Если был передан ключ аутентификации для сохранения
            Some(SubCommand::SetAuth { key }) => {
                let key = opts.key.unwrap_or(key);

                // Директория в домашней папке для файлика с настройками
                let settings_dir = match dirs::home_dir().map(|h| { h.join(SETTINGS_DIR) }) {
                    Some(path) => path,
                    None => {
                        panic!("Could not find home directory to store token.")
                    }
                };
                // Создаем директорию, пишем туда значение ключа
                std::fs::create_dir_all(&settings_dir)
                    .expect("Fail to create file in home directory");
                std::fs::write(settings_dir.join(SECRET_KEY_FILE), key)
                    .expect("Failed to save authentication key file.");

                eprintln!("Authentication key stored successfully!");
                
                // Завершаем приложение
                std::process::exit(0);
            }
            None => {
                // Был ли передан файлик ключа?
                let key = match opts.key {
                    Some(key) => {
                        Some(key)
                    },
                    None => {
                        // Если нет, пробуем читать из настроек
                        dirs::home_dir()
                            .map(|h| {
                                h.join(SETTINGS_DIR).join(SECRET_KEY_FILE)
                            })
                            .map(|path| {
                                if path.exists() {
                                    std::fs::read_to_string(path)
                                        .map_err(|e| {
                                            error!("Error reading authentication token: {:?}", e)
                                        })
                                        .ok()
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(None)
                    },
                };
                let sub_domain = opts.sub_domain;
                (key, sub_domain)
            }
        };

        // Включен ли TLS?
        let tls_off = env::var(TLS_OFF_ENV)
            .is_ok();
        
        // Хост
        let host = env::var(HOST_ENV)
            .unwrap_or(format!("{}", DEFAULT_HOST));

        // Хост управления
        let control_host = env::var(HOST_ENV)
            .unwrap_or(format!("{}", DEFAULT_CONTROL_HOST));

        // Порт
        let port = env::var(PORT_ENV)
            .unwrap_or(format!("{}", DEFAULT_CONTROL_PORT));

        // Определяем какие использовать вебсокеты, безопасные или нет?
        let scheme = if tls_off { 
            "ws" 
        } else { 
            "wss" 
        };

        // Урл для контроля
        let control_url = format!("{}://{}:{}/wormhole", scheme, control_host, port);

        info!("Control Server URL: {}", &control_url);

        Ok(Config {
            client_id: ClientId::generate(),
            local_host: opts.local_host,
            scheme: opts.scheme,
            control_url,
            host,
            local_port: opts.port,
            sub_domain,
            dashboard_port: opts.dashboard_port.unwrap_or(0),
            verbose: opts.verbose,
            secret_key: secret_key.map(|s| SecretKey(s)),
            tls_off,
            first_run: true,
        })
    }

    pub fn activation_url(&self, server_chosen_sub_domain: &str) -> String {
        format!(
            "{}://{}",
            if self.tls_off { "http" } else { "https" },
            self.activation_host(server_chosen_sub_domain)
        )
    }

    pub fn activation_host(&self, server_chosen_sub_domain: &str) -> String {
        format!("{}.{}", &server_chosen_sub_domain, &self.host)
    }

    pub fn forward_url(&self) -> String {
        format!(
            "{}://{}:{}",
            &self.scheme, &self.local_host, &self.local_port
        )
    }
    pub fn ws_forward_url(&self) -> String {
        let ws_scheme = if &self.scheme == "https" { "wss" } else { "ws" };
        format!("{}://{}:{}", ws_scheme, &self.local_host, &self.local_port)
    }
}
