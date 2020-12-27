mod application_data;
mod jenkins;
mod windows;
mod session;
mod handlers;
mod helpers;
mod qr;
mod response_awaiter_holder;
mod active_views_holder;


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
/*use rustls::{
    internal::{
        pemfile::{
            certs, 
            pkcs8_private_keys
        }
    },
    NoClientAuth, 
    ServerConfig
};*/
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
use slack_client_lib::{
    SlackRequestBuilder,
    SlackClient
};
use crate::{
    application_data::{
        ApplicationData
    },
    handlers::{
        slack_handlers::{
            slack_slash_command_handler,
            slack_events_handler,
            slack_window_handler
        },
        jenkins_handlers::{
            jenkins_build_finished_handler
        }
    },
    active_views_holder::{
        ViewsHandlersHolder
    },
    response_awaiter_holder::{
        ResponseAwaiterHolder
    },
    jenkins::{
        JenkinsRequestBuilder
    }
};

//////////////////////////////////////////////////////////////////////////////////////////////////////////

// https://api.slack.com/
// https://api.slack.com/apps/A01BSSSHB36/slash-commands?
// https://app.slack.com/block-kit-builder/

//////////////////////////////////////////////////////////////////////////////////////////////////////////

/*fn load_https_certificate(certificate_path: &Path, key_path: &Path) -> ServerConfig {
    // load ssl keys
    let mut config = ServerConfig::new(NoClientAuth::new());
    let cert_file = &mut BufReader::new(File::open("cert.pem").unwrap());
    let key_file = &mut BufReader::new(File::open("key.pem").unwrap());
    let cert_chain = certs(cert_file).unwrap();
    let mut keys = pkcs8_private_keys(key_file).unwrap();
    config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

    config
}*/

fn parse_application_arguments() -> clap::ArgMatches<'static> {
    let matches = clap::App::new("Slack bot")
        .version("1.0")
        .author("Pavel Ershov")
        .about("Slack bot http server")
        .arg(clap::Arg::with_name("http_port")
            .short("p")
            .long("http_port")
            .value_name("http_port")
            .help("Sets HTTP port value, 8888 in case of empty")
            .takes_value(true))
        .get_matches();

    matches
}

// Настройка путей веб сервера
fn configure_server(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/slack")
                    .service(web::resource("/command")
                                .route(web::route()
                                        .guard(guard::Post())
                                        .guard(guard::Header("Content-type", "application/x-www-form-urlencoded"))
                                        .to(slack_slash_command_handler)))
                    .service(web::resource("/events")
                                .route(web::route()
                                        .guard(guard::Post())
                                        .to(slack_events_handler)))                                        
                    .service(web::resource("/window")
                                .route(web::route()
                                        .guard(guard::Post())
                                        .guard(guard::Header("Content-type", "application/x-www-form-urlencoded"))
                                        .to(slack_window_handler))));
    cfg.service(web::scope("/jenkins")
                    .service(web::resource("/build_finished")
                            .route(web::route()
                                    .guard(guard::Post())
                                    .guard(guard::Header("Content-type", "application/x-www-form-urlencoded"))
                                    .to(jenkins_build_finished_handler))));
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    // Активируем логирование и настраиваем уровни вывода
    // https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html
    if !std::env::var("RUST_LOG").is_ok() {
        std::env::set_var("RUST_LOG", "actix_server=trace,actix_web=trace,slack_command_handler=trace");
    }
    env_logger::init();

    info!("Application setup");

    const DEFAULT_PORT: u16 = 8888;

    // Slack api token
    let slack_api_token = std::env::var("SLACK_API_TOKEN")
        .expect("SLACK_API_TOKEN environment variable is missing");

    // Jenkins user
    let jenkins_user = std::env::var("JENKINS_USER")
        .expect("JENKINS_USER environment variable is missing");

    // Jenkins api token
    let jenkins_api_token = std::env::var("JENKINS_API_TOKEN")
        .expect("JENKINS_API_TOKEN environment variable is missing");

    // Port from environment
    let http_port = std::env::var("SLACK_BOT_HTTP_PORT")
        .ok()
        .and_then(|val|{
            val.parse::<u16>().ok()
        })
        .unwrap_or(DEFAULT_PORT);

    // Парсинг аргументов приложения
    let arguments = parse_application_arguments();

    // HTTP Port from parameters
    let http_port: u16 = arguments
        .value_of("http_port")
        .and_then(|val|{
            val.parse::<u16>().ok()
        })
        .unwrap_or(http_port);

    // Общий менеджер запросов с пулом соединений
    let request_client = Client::new(); // TODO: Configure

    // Контейнер для вьюшек, общий для всех инстансов приложения
    let active_views_container = Data::new(ViewsHandlersHolder::default());

    // Специальный контейнер, который позволяет дождаться прихода ответа
    let response_awaiter = ResponseAwaiterHolder::new();

    // Обработчик запросов Jenkins
    let jenkins_request_builder = JenkinsRequestBuilder::new(request_client.clone(), jenkins_user, jenkins_api_token);

    // Обработчик запросов Slack
    let slack_request_builder = SlackRequestBuilder::new(request_client, slack_api_token);

    // Создание веб-приложения, таких приложений может быть создано много за раз
    // Данный коллбек может вызываться несколько раз
    let web_application_factory = move || {
        // Создаем данные приложения для текущего треда
        let app_data = Data::new(ApplicationData::new(
            SlackClient::new(slack_request_builder.clone()),
            jenkins::JenkinsClient::new(jenkins_request_builder.clone())
        ));

        // Создаем приложение
        App::new()
            .app_data(app_data)
            .app_data(active_views_container.clone())
            .app_data(response_awaiter.clone())
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
                .bind(format!("0.0.0.0:{}", http_port))?
        }
    };

    // Запускаем сервер
    server
        .keep_alive(75_usize) // 75 секунд
        .workers(1_usize) // Можно задать конкретное количество потоков
        .run()
        .await
}