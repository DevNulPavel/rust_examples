mod constants;
mod proxy;
mod currency;
mod app_context;
mod bot_context;

use std::{
    env,
    time::Duration,
    //collections::HashSet
    //collections::HashMap
    //pin::Pin,
    //future::Future
};
use futures::{
    StreamExt,
};
use telegram_bot::{
    //prelude::*,
    connector::Connector,
    Api,
    UpdatesStream,
    UpdateKind,
    MessageKind,
    MessageEntityKind,
    //Error,
    //CanReplySendMessage,
    CanSendMessage,
    Update,
    Message,
    //MessageChat,
    //CanSendMessage,
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
use sqlx::{
    Connect,
    sqlite::{
        SqliteConnection
    }
};
use crate::{
    // constants::PROXIES,
    app_context::AppContext,
    bot_context::BotContext,
    proxy::{
        get_valid_proxy_addresses,
        build_proxy_for_addresses,
        check_all_proxy_addresses_accessible
    },
    currency::{
        CurrencyUsersStorrage,
        check_currencies_update,
        process_currencies_command
    }
};

async fn process_bot_command(bot_context: &mut BotContext, data: &String, message: &Message){
/*
currencies - Receive currencies
currencies_monitoring_on - Start monitoring
currencies_monitoring_off - Stop monitoring
currencies_monitoring_reset - Reset monitoring from current time
habr - Habr news
*/ 

    // TODO: match
    if data.eq("/start") {
    }
    if data.eq("/currencies") {
        process_currencies_command(bot_context, message).await.unwrap();
    }
    if data.eq("/currencies_monitoring_on") {
        println!("Start monitoring for: {:?}", message.from);
        (*bot_context).users_for_push.add_user(&message.from.id);
        let private_messaage = message.from.text("Enabled");
        bot_context.api.send(private_messaage).await.ok();

        // После нового юзера - стартуем обновление для всех
        check_currencies_update(bot_context).await;
    }
    if data.eq("/currencies_monitoring_reset") {
    }
    if data.eq("/currencies_monitoring_off") {
        println!("Stop monitoring for: {:?}", message.from);
        (*bot_context).users_for_push.remove_user(&message.from.id);
        let private_messaage = message.from.text("Disabled");
        bot_context.api.send(private_messaage).await.ok();
    }
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
                                process_bot_command(bot_context, data, message).await;
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
    // TODO: 
    // - добавить пример работы с прокси в библиотеку
    // - проверять доступность нескольких проксей, добавлять только доступные
    // - запрашивать откуда-то список проксей, затем по очереди проверять через прокси доступность телеграма, периодически обновлять активный прокси
    // - сортировать прокси по пингу
    // - добавить систему логирования c временем 
    // - добавить юнит тесты
    // - добавить тесты обновления платежки
    // - информация о боте на /start
    // - еще прокси
    // - банк ВТБ
    // - сохранение в базу юзеров
    // - рестарт мониторинга
    // - оборачивать приложение в docker
    // - прокси в файлике контейнера

    // TODO: Обернуть в cfg
    // Трассировка
    // tracing::subscriber::set_global_default(
    //     tracing_subscriber::FmtSubscriber::builder()
    //         .with_env_filter("telegram_bot=trace")
    //         .finish(),
    // )
    // .unwrap();

    // Получаем токен нашего бота
    let token: String = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");
    println!("Token: {}", token);

    // Создаем клиента для запросов
    let client: Client = ClientBuilder::new()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();

    // База данных
    let db_conn = SqliteConnection::connect("telegram_bot.sqlite")
        .await
        .unwrap();

    // Таймер проверки проксей
    let mut proxy_check_timer = tokio::time::interval(Duration::from_secs(60*5));
    proxy_check_timer.tick().await; // Первый тик сбрасываем

    // Таймер проверки проксей
    let mut send_message_timer = tokio::time::interval(Duration::from_secs(60*10));
    send_message_timer.tick().await; // Первый тик сбрасываем

    let mut app_context = AppContext{
        token,
        proxy_check_timer,
        send_message_timer,
        client,
        db_conn,
        users_for_push: CurrencyUsersStorrage::new(), // Хранилище пользователей
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

        println!("Stream created\n");

        'select_loop: loop {
            tokio::select! {
                // Таймер проверки проксей
                _ = bot_context.app_context.proxy_check_timer.tick() => {
                    println!("Repeat proxy check");
                    let accessible = check_all_proxy_addresses_accessible(&valid_proxy_addresses).await;
                    if accessible {
                        println!("All proxies are valid, continue");
                    }else{
                        println!("Some proxy is invalid, break");
                        break 'select_loop;
                    }
                },
                
                // Таймер периодических сообщений
                _ = bot_context.app_context.send_message_timer.tick() => {
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
                            println!("No updates - break");
                            break 'select_loop;
                        }
                    };

                    // Получаем новое обновление, падая при ошибке
                    let update: Update = update.unwrap();
                    println!("Update: {:?}\n", update);

                    // Обработка обновления
                    process_update(&mut bot_context, update).await;
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
        .unwrap();
    
    runtime.block_on(async_main());
}