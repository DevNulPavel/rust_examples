use std::{
    env,
    time::Duration
    //pin::Pin,
    //future::Future
};
use futures::{
    StreamExt,
    //FutureExt
};
use hyper_proxy::{
    Proxy, 
    ProxyConnector, 
    Intercept
};
use hyper::{
    //Client, 
    //Request, 
    Uri,
    client::{
        HttpConnector,
        connect::dns::GaiResolver
    }
};
// use typed_headers::{
//     Credentials,
//     Token68
// };
use telegram_bot::{
    //prelude::*,
    connector::Connector,
    connector::hyper::HyperConnector,
    Api,
    UpdatesStream,
    UpdateKind,
    MessageKind,
    MessageEntityKind,
    Error,
    //CanReplySendMessage,
    Update,
    Message,
    //MessageChat,
    CanSendMessage
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
use currencies_request::{
    CurrencyError,
    CurrencyResult,
    //CurrencyChange,
    get_all_currencies
};

async fn process_currencies_command(api: &Api, message: &Message) -> Result<(), Error> {
    // Создаем клиента для запроса
    let client: Client = ClientBuilder::new()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();

    let mut text = String::new();

    // Выводим текст, используем into_iter для потребляющего итератора
    for info in get_all_currencies(&client).await {
        let info: Result<CurrencyResult, CurrencyError> = info;
        match info {
            Ok(info) =>{
                let info: CurrencyResult = info;

                let time_str: String = match info.update_time {
                    Some(time) => time.format("%H:%M:%S %Y-%m-%d").to_string(),
                    None => "No time".into()
                };

                let bank_text = format!("{} ({}):\nUSD: buy = {}, sell = {}\nEUR: buy = {}, sell = {}\n\n",
                        info.bank_name,
                        time_str,
                        info.usd.buy,
                        info.usd.sell,
                        info.eur.buy,
                        info.eur.sell
                    );
                
                text.push_str(bank_text.as_str())
            },
            Err(_e) => {
                // TODO: Вывод ошибок
                /*let row = Row::new(vec![
                    Cell::new(format!("{:?}", e).as_str()),
                ]);
                table.add_row(row);*/
                println!("{:?}", _e);
            }
        }
    }

    let private_messaage = message.from.text(text);    
    api.send(private_messaage).await?;

    Ok(())
}

async fn async_main(){
    // TODO: 
    // - добавить пример работы с прокси в библиотеку
    // - проверять доступность нескольких проксей, добавлять только доступные

    let proxy: Box<dyn Connector> = {
        // https://www.firexproxy.com/
        // http://free-proxy.cz/ru/
        // http://spys.one/proxylist/
        
        const PROXIES: &[&str] = &[
            // "http://174.138.42.112:8080",
            // "http://52.179.231.206:80",
            // "http://82.119.170.106:8080",
            // "http://80.187.140.26:8080",
            "http://95.179.167.232:8080",
            // "http://95.179.130.83:8080",
            // "http://82.119.170.106:8080",
            "http://54.37.131.45:3128",
            "http://51.158.68.68:8811",
            "http://51.91.212.159:3128",
            "http://95.179.130.83:8080",
        ];
        let proxies_iter = PROXIES.iter()
            .map(|addr|{
                let proxy_uri: Uri = addr.parse().unwrap();
                let proxy: Proxy = Proxy::new(Intercept::All, proxy_uri);
                // proxy.set_authorization(Credentials::bearer(Token68::new("").unwrap()));
                // proxy.set_authorization(Credentials::basic("John Doe", "Agent1234").unwrap());
                proxy
            });

        let connector: HttpConnector<GaiResolver> = HttpConnector::new();

        let mut proxy_connector = ProxyConnector::new(connector).unwrap();
        proxy_connector.extend_proxies(proxies_iter);
        
        let client = hyper::Client::builder()
            .build(proxy_connector);

        Box::new(HyperConnector::new(client))
    };

    // Получаем токен нашего бота
    let token: String = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");
    println!("Token: {}", token);

    // Подключаемся с использованием прокси
    // let api: Api = Api::new(token);
    let api: Api = Api::with_connector(token, proxy);

    // TODO: Обернуть в cfg
    // Трассировка
    // tracing::subscriber::set_global_default(
    //     tracing_subscriber::FmtSubscriber::builder()
    //         .with_env_filter("telegram_bot=trace")
    //         .finish(),
    // )
    // .unwrap();

    // Дергаем новые обновления через long poll метод
    let mut stream: UpdatesStream = api.stream();

    // Идем по новым событиям
    while let Some(update) = stream.next().await {
        // Получаем новое обновление
        let update: Update = update.unwrap();
        println!("Update: {:?}\n", update);

        match update.kind {
            // Тип обновления - сообщение
            UpdateKind::Message(ref message) => {
                match message.kind {
                    // Сообщение с текстом, ref нужен для получения ссылки на data из message, вместо переноса владения 
                    MessageKind::Text{ ref data, ref entities } =>  {
                        for command in entities {
                            match command.kind {
                                MessageEntityKind::BotCommand => {
                                    if data.starts_with("/currencies") {
                                        process_currencies_command(&api, message).await.unwrap();
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