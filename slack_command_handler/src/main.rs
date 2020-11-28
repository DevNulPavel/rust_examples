mod application_data;
mod jenkins;
mod slack;
mod windows;
mod handlers;

use actix_web::{
    client,
    web,
    guard,
    middleware,
    App,
    HttpServer
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
        ApplicationData
    },
    handlers::{
        jenkins_command_handler,
        window_handler
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
                                        .to(jenkins_command_handler)))
                    .service(web::resource("/window")
                                .route(web::route()
                                        .guard(guard::Post())
                                        .guard(guard::Header("Content-type", "application/x-www-form-urlencoded"))
                                        .to(window_handler))));
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    // Активируем логирование и настраиваем уровни вывода
    // https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info,slack_command_handler=trace");
    env_logger::init();

    info!("Application setup");

    // Создание веб-приложения
    let build_web_application = || {
        // Slack api token
        let slack_api_token = std::env::var("SLACK_API_TOKEN")
            .expect("SLACK_API_TOKEN environment variable is missing");

        // Jenkins user
        let jenkins_user = std::env::var("JENKINS_USER")
            .expect("JENKINS_USER environment variable is missing");

        // Jenkins api token
        let jenkins_api_token = std::env::var("JENKINS_API_TOKEN")
            .expect("JENKINS_API_TOKEN environment variable is missing");

        // Создаем общие данные приложения
        let app_data = ApplicationData{
            jenkins_client: jenkins::JenkinsClient::new(&jenkins_user, &jenkins_api_token),
            slack_client: slack::SlackClient::new(&slack_api_token)
        };

        // Создаем приложение
        App::new()
            .data(app_data)
            .wrap(middleware::Logger::default()) // Включаем логирование запросов с помощью middleware
            .configure(configure_server)
    };

    // Создаем слушателя, чтобы просто переподключаться к открытому сокету при быстром рестарте
    let server = match ListenFd::from_env().take_tcp_listener(0)? {
        Some(listener) => {
            info!("Reuse server socket");
            
            // Создаем сервер с уже имеющимся листнером
            HttpServer::new(build_web_application)
                .listen(listener)?
        },
        None => {
            info!("New server socket");

            // Создаем новый сервер
            HttpServer::new(build_web_application)
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