mod application_data;
mod jenkins;
mod slack;
mod windows;
mod session;
mod handlers;

use std::{
    sync::{
        Mutex,
        Arc
    },
    path::{
        Path
    },
    fs::{
        File
    },
    io::{
        BufReader
    }
};
use actix_web::{
    web::{
        self,
        Data
    },
    guard,
    middleware,
    App,
    HttpServer
};
use rustls::{
    internal::{
        pemfile::{
            certs, 
            pkcs8_private_keys
        }
    },
    NoClientAuth, 
    ServerConfig
};
use reqwest::{
    Client
};
use log::{
    // debug,
    info,
    // error
};
use listenfd::{
    ListenFd
};
use crate::{
    application_data::{
        ApplicationData,
        ViewsHandlersMap
    },
    handlers::{
        jenkins_slash_command_handler,
        jenkins_events_handler,
        jenkins_window_handler
    }
};

//////////////////////////////////////////////////////////////////////////////////////////////////////////

// https://api.slack.com/
// https://api.slack.com/apps/A01BSSSHB36/slash-commands?
// https://app.slack.com/block-kit-builder/

//////////////////////////////////////////////////////////////////////////////////////////////////////////

// Настройка путей веб сервера
fn configure_server(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/jenkins")
                    .service(web::resource("/command")
                                .route(web::route()
                                        .guard(guard::Post())
                                        .guard(guard::Header("Content-type", "application/x-www-form-urlencoded"))
                                        .to(jenkins_slash_command_handler)))
                    .service(web::resource("/events")
                                .route(web::route()
                                        .guard(guard::Post())
                                        .to(jenkins_events_handler)))                                        
                    .service(web::resource("/window")
                                .route(web::route()
                                        .guard(guard::Post())
                                        .guard(guard::Header("Content-type", "application/x-www-form-urlencoded"))
                                        .to(jenkins_window_handler))));
}

fn load_https_certificate(certificate_path: &Path, key_path: &Path) -> ServerConfig {
    // load ssl keys
    let mut config = ServerConfig::new(NoClientAuth::new());
    let cert_file = &mut BufReader::new(File::open("cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("key.pem").unwrap());
    let cert_chain = certs(cert_file).unwrap();
    let mut keys = pkcs8_private_keys(key_file).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

    config
}

fn parse_application_arguments(){
    let matches = clap::App::new("My Super Program")
        .version("1.0")
        .author("Kevin K. <kbknapp@gmail.com>")
        .about("Does awesome things")
        .arg(clap::Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .arg(clap::Arg::with_name("INPUT")
            .help("Sets the input file to use")
            .required(true)
            .index(1))
        .arg(clap::Arg::with_name("v")
            .short("v")
            .multiple(true)
            .help("Sets the level of verbosity"))
        .subcommand(clap::SubCommand::with_name("test")
                    .about("controls testing features")
                    .version("1.3")
                    .author("Someone E. <someone_else@other.com>")
                    .arg(clap::Arg::with_name("debug")
                        .short("d")
                        .help("print debug information verbosely")))
        .get_matches();
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    // Активируем логирование и настраиваем уровни вывода
    // https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info,slack_command_handler=trace");
    env_logger::init();

    info!("Application setup");

    // Slack api token
    let slack_api_token = std::env::var("SLACK_API_TOKEN")
        .expect("SLACK_API_TOKEN environment variable is missing");

    // Jenkins user
    let jenkins_user = std::env::var("JENKINS_USER")
        .expect("JENKINS_USER environment variable is missing");

    // Jenkins api token
    let jenkins_api_token = std::env::var("JENKINS_API_TOKEN")
        .expect("JENKINS_API_TOKEN environment variable is missing");

    // Jenkins api token
    let http_port: u16 = 8888;
    let https_port: u16 = 8443;

    // Общий менеджер запросов с пулом соединений
    // TODO: Configure
    let request_client = Client::new();

    // Контейнер для вьюшек, общий для всех инстансов приложения
    let active_views_container = Arc::new(Mutex::new(ViewsHandlersMap::new()));

    // Создание веб-приложения, таких приложений может быть создано много за раз
    // Данный коллбек может вызываться несколько раз
    let web_application_factory = move || {
        // Создаем данные приложения для текущего треда
        let app_data = Data::new(ApplicationData::new(
            slack::SlackClient::new(request_client.clone(), &slack_api_token),
            jenkins::JenkinsClient::new(request_client.clone(), &jenkins_user, &jenkins_api_token),
            active_views_container.clone()
        ));

        // Создаем приложение
        App::new()
            .app_data(app_data)
            .wrap(middleware::Logger::default()) // Включаем логирование запросов с помощью middleware
            .configure(configure_server)
    };


    // Создаем слушателя, чтобы просто переподключаться к открытому сокету при быстром рестарте
    let server = match ListenFd::from_env().take_tcp_listener(0)? {
        Some(listener) => {
            info!("Reuse server socket");
            
            // Создаем сервер с уже имеющимся листнером
            HttpServer::new(web_application_factory)
                .listen(listener)?
        },
        None => {
            info!("New server socket");

            // Создаем новый сервер
            HttpServer::new(web_application_factory)
                .bind("0.0.0.0:8888")?
        }
    };

    // Запускаем сервер
    server
        .keep_alive(75_usize) // 75 секунд
        .workers(1_usize) // Можно задать конкретное количество потоков
        .run()
        .await
}