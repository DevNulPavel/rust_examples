mod slack_window;
mod slack_command;

use actix_web::{ 
    web,
    guard,
    middleware,
    App,
    HttpServer,
    Responder,
    HttpResponse
};
use serde::{
    Serialize
};
use log::{
    debug,
    info,
    error
};
use listenfd::{
    ListenFd
};
use crate::{
    slack_window::{
        SlackWindow,
        SlackWindowParameters,
        SlackWindowParametersPayload
    },
    slack_command::{
        SlackCommandParameters
    }
};

//////////////////////////////////////////////////////////////////////////////////////////////////////////

// https://api.slack.com/
// https://api.slack.com/apps/A01BSSSHB36/slash-commands?

//////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Serialize)]
struct SlackCommandResponse{
    response_type: &'static str,
    text: String
}


async fn jenkins_command(parameters: web::Form<SlackCommandParameters>, app_data: web::Data<ApplicationData>) -> impl Responder {
    debug!("Index parameters: {:?}", parameters);

    // Описываем наше окно
    let window = serde_json::json!(
        {
            "trigger_id": parameters.trigger_id,
            "view": {
                "type": "modal",
                "callback_id": "build_jenkins_id",
                "title": {
                    "type": "plain_text",
                    "text": "Build jenkins target"
                },
                "blocks": [
                    {
                        "type": "section",
                        "block_id": "section-identifier",
                        "text": {
                            "type": "mrkdwn",
                            "text": "*Welcome* to ~my~ Block Kit _modal_!"
                        },
                        "accessory": {
                            "type": "button",
                            "action_id": "test_button_id",
                            "text": {
                                "type": "plain_text",
                                "text": "Test button"
                            }
                        }
                    }
                ]
            }
        }
    );

    // Выполняем наш запрос
    let response = app_data
        .slack_client
        .post("https://slack.com/api/views.open")
        .bearer_auth(app_data.slack_api_token.as_str())
        .header("Content-type", "application/json")
        .body(serde_json::to_string(&window).unwrap())
        .send()
        .await;
        
    match response {
        Ok(res) => {
            debug!("Window open result: {:?}", res);
            HttpResponse::Ok()
                .finish()
            // let response = SlackCommandResponse{
            //     response_type: "ephemeral",
            //     text: String::from("test")
            // };
        
            // HttpResponse::Ok()
            //     .json(response)            
        },
        Err(err) =>{
            error!("Window open error: {:?}", err);
            HttpResponse::Ok()
                .body(format!("{:?}", err))
        }
    }
}

// std::collections::HashMap<String, serde_json::Value>
async fn jenkins_window(parameters: web::Form<SlackWindowParameters>, app_data: web::Data<ApplicationData>) -> impl Responder {
    //debug!("Jenkins window parameters: {:?}", parameters);

    match serde_json::from_str::<SlackWindowParametersPayload>(parameters.payload.as_str()){
        Ok(payload) => {
            debug!("Parsed payload: {:?}", payload);

            // Описываем обновление нашего окна
            // https://api.slack.com/surfaces/modals/using#interactions
            // TODO: Hash??
            /*let window_update = serde_json::json!(
                {
                    "view_id": payload.view.id,
                    "hash": "156772938.1827394",
                    "view": {
                        "type": "modal",
                        "callback_id": "view-helpdesk",
                        "title": {
                            "type": "plain_text",
                            "text": "Submit an issue"
                        },
                        "submit": {
                            "type": "plain_text",
                            "text": "Submit"
                        },
                        "blocks": [
                            {
                                "type": "input",
                                "block_id": "ticket-title",
                                "label": {
                                    "type": "plain_text",
                                    "text": "Ticket title"
                                },
                                "element": {
                                    "type": "plain_text_input",
                                    "action_id": "ticket-title-value"
                                }
                            },
                            {
                                "type": "input",
                                "block_id": "ticket-desc",
                                "label": {
                                    "type": "plain_text",
                                    "text": "Ticket description"
                                },
                                "element": {
                                    "type": "plain_text_input",
                                    "multiline": true,
                                    "action_id": "ticket-desc-value"
                                }
                            }
                        ]
                    }
                }
            );

            // Выполняем запрос обновления вьюшки
            let response = app_data
                .slack_client
                .post("https://slack.com/api/views.update")
                .bearer_auth(app_data.slack_api_token.as_str())
                .header("Content-type", "application/json")
                .body(serde_json::to_string(&window_update).unwrap())
                .send()
                .await;
            
            match response {
                Ok(res) => {
                    debug!("Window modify response: {:?}", res);
                    HttpResponse::Ok()
                        .finish()    
                },
                Err(err) => {
                    error!("Window modify error: {:?}", err);
                    // TODO: Error
                    HttpResponse::Ok()
                        .body(format!("Window modify error: {}", err))
                }
            }*/


            let new_window = serde_json::json!(
                {
                    "trigger_id": payload.trigger_id,
                    "view": {
                      "type": "modal",
                      "callback_id": "edit-task",
                      "title": {
                        "type": "plain_text",
                        "text": "Edit task details"
                      },
                      "submit": {
                          "type": "plain_text",
                          "text": "Create"
                      },
                      "blocks": [
                        {
                          "type": "input",
                          "block_id": "edit-task-title",
                          "label": {
                            "type": "plain_text",
                            "text": "Task title"
                          },
                          "element": {
                            "type": "plain_text_input",
                            "action_id": "task-title-value",
                            "initial_value": "Block Kit documentation"
                          },
                        },
                        {
                          "type": "input",
                          "block_id": "edit-ticket-desc",
                          "label": {
                            "type": "plain_text",
                            "text": "Ticket description"
                          },
                          "element": {
                            "type": "plain_text_input",
                            "multiline": true,
                            "action_id": "ticket-desc-value",
                            "initial_value": "Update Block Kit documentation to include Block Kit in new surface areas (like modals)."
                          }
                        }
                      ]
                    }
                  }
            );

            // Выполняем запрос обновления вьюшки
            let response = app_data
                .slack_client
                .post("https://slack.com/api/views.push")
                .bearer_auth(app_data.slack_api_token.as_str())
                .header("Content-type", "application/json")
                .body(serde_json::to_string(&new_window).unwrap())
                .send()
                .await;
            
            match response {
                Ok(res) => {
                    debug!("Window create response: {:?}", res);
                    HttpResponse::Ok()
                        .finish()    
                },
                Err(err) => {
                    error!("Window create error: {:?}", err);
                    // TODO: Error
                    HttpResponse::Ok()
                        .body(format!("Window create error: {}", err))
                }
            }
        },
        Err(err) => {
            error!("Payload parse error: {:?}", err);
            // TODO: Error
            HttpResponse::Ok()
                .body(format!("Payload parse error: {}", err))
        }
    }
}

// Настройка путей веб сервера
fn configure_server(cfg: &mut web::ServiceConfig) {
    let jenkins_build_scope = web::scope("/jenkins")
        .service(web::resource("/command")
                    .route(web::route()
                            .guard(guard::Post())
                            .guard(guard::Header("Content-type", "application/x-www-form-urlencoded"))
                            .to(jenkins_command)))
        .service(web::resource("/window")
                    .route(web::route()
                            .to(jenkins_window)));
    
    cfg
        .service(jenkins_build_scope);
}

#[derive(Clone)]
struct ApplicationData{
    slack_api_token: String,
    slack_client: reqwest::Client
}

#[actix_rt::main]
async fn main() -> std::io::Result<()>{
    // Активируем логирование и настраиваем уровни вывода
    // https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info,slack_command_handler=trace");
    env_logger::init();

    info!("Application setup");

    // Создание веб-приложения
    let build_web_application = || {
        // Api token
        let api_token = std::env::var("SLACK_API_TOKEN")
            .expect("SLACK_API_TOKEN environment variable is missing");

        let app_data = ApplicationData{
            slack_api_token: api_token,
            slack_client: reqwest::Client::new()
        };

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