mod constants;
mod proxy;
mod currency;
mod app_context;
mod bot_context;
mod database;
mod error;

use std::{
    env,
    time::Duration,
};
use futures::{
    StreamExt,
};
use telegram_bot::{
    connector::Connector,
    Api,
    UpdatesStream,
    UpdateKind,
    MessageKind,
    MessageEntityKind,
    CanSendMessage,
    Update,
    Message,
    ParseMode,
    MessageOrChannelPost,
};
use tokio::{
    runtime::{
        Builder,
        Runtime
    }
};
use reqwest::{
    Client,
    ClientBuilder,
};
use log::{
    info,
    warn,
    error
};
use crate::{
    app_context::AppContext,
    bot_context::BotContext,
    error::{
        TelegramBotError,
        TelegramBotResult
    },
    proxy::{
        get_valid_proxy_addresses,
        build_proxy_for_addresses,
        check_all_proxy_addresses_accessible
    },
    currency::{
        CurrencyUsersStorrage,
        check_currencies_update,
        process_currencies_command,
        process_currencies_status
    },
    database::{
        get_database
    }
};

async fn send_tutorial(api: &Api, message: &Message) -> Result<MessageOrChannelPost, TelegramBotError>{
    //telegram_bot::types::requests::Messa
    const FAQ_TEXT: &str = "\
        Information: /start or /help \n/
        Currencies: /currencies \n/
        Start minimum monitoring: /currencies_monitoring_on \n/
        Stop minimum monitoring with reset: /currencies_monitoring_off \n/
        Reset minimum monitoring values: /currencies_monitoring_reset \n/
        Information: /start or /help \n/";
    let mut req = message.from.text(FAQ_TEXT);
    let send_result = api.send(req.parse_mode(ParseMode::Markdown)).await?;
    Ok(send_result)
}

async fn start_user_monitoring(bot_context: &mut BotContext, message: &Message) -> TelegramBotResult {
    info!("Start monitoring for: {:?}", message.from);

    // https://doc.rust-lang.org/book/ch18-03-pattern-syntax.html#destructuring-structs
    // Разворачиваем структуру в поля
    let AppContext{
        db_conn: db,
        users_for_push: users,
        ..
    } = &mut bot_context.app_context;

    // Добавляем пользователя к мониторингу
    let result = users
        .add_user(&message.from.id, db)
        .await;
    
    // Если добавили юзера - хорошо
    let message = if (&result).is_ok() {
        message.from.text("Enabled")
    }else{
        message.from.text("Enable failed")
    };
    bot_context.api.send(message).await?;

    // Если было все ок - запрашиваем валюты принудительно
    if result.is_ok() {
        info!("Check currencies");
        check_currencies_update(bot_context).await;
    }

    Ok(())
}

async fn stop_user_monitoring(bot_context: &mut BotContext, message: &Message) -> TelegramBotResult {
    info!("Stop monitoring for: {:?}", message.from);

    // https://doc.rust-lang.org/book/ch18-03-pattern-syntax.html#destructuring-structs
    // Разворачиваем структуру в поля
    let AppContext{
        db_conn: db,
        users_for_push: users,
        ..
    } = &mut bot_context.app_context;

    // Добавляем пользователя из монитринга
    let remove_result = users.remove_user(&message.from.id, db)
        .await;

    // TODO: Осмысленные сообщения
    // Если было все ок - сообщение успешности
    let message = if remove_result.is_ok() {
        message.from.text("Disabled")
    }else{
        message.from.text("Disable failed")
    };
    bot_context.api.send(message).await?;

    Ok(())
}

async fn process_bot_command(bot_context: &mut BotContext, data: &String, message: &Message) -> TelegramBotResult {
    /*
    currencies - Receive currencies
    currencies_monitoring_on - Start monitoring
    currencies_monitoring_off - Stop monitoring
    currencies_monitoring_reset - Reset monitoring from current time
    currencies_monitoring_status - Get current monitoring status and values
    habr - Habr news
    */ 

    match data.as_str() {
        "/start" =>{
            send_tutorial(&bot_context.api, message).await?;
        },
        "/currencies" => {
            process_currencies_command(bot_context, message).await?;
        },
        "/currencies_monitoring_on" => {
            start_user_monitoring(bot_context, message).await?;
        }
        "/currencies_monitoring_off" => {
            stop_user_monitoring(bot_context, message).await?;
        },
        "/currencies_monitoring_status" => {
            process_currencies_status(bot_context, message).await?;
        },
        "/currencies_monitoring_reset" => {
            // TODO: ???
            //reset_user_monitoring(bot_context, message).await?;
        },
        text => {
            return Err(TelegramBotError::CustomError(format!("Invalid bot command: {}", text)));
        }
    }

    Ok(())
}

async fn process_update(bot_context: &mut BotContext, update: Update){
    match update.kind {
        // Тип обновления - сообщение
        UpdateKind::Message(ref message) => {
            match message.kind {
                // Сообщение с текстом, ref нужен для получения ссылки на data из message, вместо переноса владения 
                MessageKind::Text{ ref data, ref entities } =>  {
                    for command in entities {
                        match command.kind {
                            MessageEntityKind::BotCommand => {
                                if let Err(e) = process_bot_command(bot_context, data, message).await{
                                    error!("Process command error: {}", e);
                                }
                            },
                            _ => {
                            }
                        }
                    }
                },
                _ => {
    
                }
            }
        },
    
        // Остальные не обрабатываем
        _ => {
    
        }
    }
}

async fn async_main(){
    // pretty_env_logger::init_timed();
    std::env::set_var("RUST_LOG", "telegram_bot=trace");
    pretty_env_logger::init();

    // TODO: Обернуть в cfg
    // Трассировка
    // tracing::subscriber::set_global_default(
    //     tracing_subscriber::FmtSubscriber::builder()
    //         .with_env_filter("telegram_bot=trace")
    //         .finish(),
    // )
    // .unwrap();

    // Получаем токен нашего бота
    let token: String = env::var("TELEGRAM_TOKEN")
        .expect("TELEGRAM_TOKEN not set");
    info!("Token: {}", token);

    // Создаем клиента для запросов
    let client: Client = ClientBuilder::new()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(3))
        .build()
        .expect("Request build failed");

    // База данных
    let mut db_conn = get_database().await;

    // Таймер проверки проксей
    let mut proxy_check_timer = tokio::time::interval(Duration::from_secs(60*3));
    proxy_check_timer.tick().await; // Первый тик сбрасываем

    // Таймер проверки проксей
    let mut check_updates_timer = tokio::time::interval(Duration::from_secs(60*5));
    check_updates_timer.tick().await; // Первый тик сбрасываем

    // Хранилище юзеров
    let users_storrage = CurrencyUsersStorrage::new(&mut db_conn).await;

    // Данные всего приложения
    let mut app_context = AppContext{
        token,
        proxy_check_timer,
        check_updates_timer,
        client,
        db_conn,
        users_for_push: users_storrage, // Хранилище пользователей
    };

    loop {
        // Получаем валидные адреса проксей
        let valid_proxy_addresses = get_valid_proxy_addresses()
            .await
            .expect("No valid proxies");

        // Создаем прокси
        let proxy: Box<dyn Connector> = build_proxy_for_addresses(&valid_proxy_addresses);

        // Создаем хранилище данных бота
        let token = app_context.token.clone();
        let mut bot_context = BotContext::new(app_context, Api::with_connector(token, proxy)); // Подключаемся с использованием прокси
            
        // Дергаем новые обновления через long poll метод
        let mut stream: UpdatesStream = bot_context.api.stream();

        info!("Stream created\n");

        'select_loop: loop {
            tokio::select! {
                // Таймер проверки проксей
                _ = bot_context.app_context.proxy_check_timer.tick() => {
                    info!("Repeat proxy check");
                    let accessible = check_all_proxy_addresses_accessible(&valid_proxy_addresses).await;
                    if accessible {
                        info!("All proxies are valid, continue");
                    }else{
                        warn!("Some proxy is invalid, break");
                        break 'select_loop;
                    }
                },
                
                // Таймер периодических сообщений
                _ = bot_context.app_context.check_updates_timer.tick() => {
                    check_currencies_update(&mut bot_context).await;
                },

                // Обработка обновлений
                update = stream.next() => {
                    // Обновлений нету - выходим
                    let update = match update {
                        Some(update) => {
                            update
                        },
                        None =>{
                            error!("No updates - break");
                            break 'select_loop;
                        }
                    };

                    // Получаем новое обновление, падая при ошибке
                    match update {
                        Ok(update) => {
                            let update: Update = update;
                            info!("Update: {:?}\n", update);

                            // Обработка обновления
                            process_update(&mut bot_context, update).await;
                        },
                        Err(e) => {
                            info!("Update receive failed: {}", e);
                            // Перед новым подключением - подождем немного
                            //tokio::time::delay_for(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        }

        // Возвращаем владение в переменные вне цикла
        app_context = bot_context.into(); // bot_context.app_context;

        // Перед новым подключением - подождем немного
        tokio::time::delay_for(Duration::from_secs(15)).await;
    }
}

fn main() {
    // Создаем однопоточный рантайм, здесь нет нужды в многопоточном
    let mut runtime: Runtime = Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .expect("Tokio runtime build failed");
    
    runtime.block_on(async_main());
}