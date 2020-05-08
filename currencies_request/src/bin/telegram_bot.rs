use std::{
    env,
    //pin::Pin,
    //future::Future
};
use futures::StreamExt;
use hyper_proxy::{
    Proxy, 
    ProxyConnector, 
    Intercept
};
use hyper::{
    Client, 
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
    connector::Connector,
    connector::hyper::HyperConnector,
    Api,
    UpdatesStream,
    UpdateKind,
    MessageKind,
    Error,
    CanReplySendMessage,
    Update,
};

#[tokio::main(max_threads = 1)]
async fn main() -> Result<(), Error> {
    // TODO: 
    // - добавить пример работы с прокси в библиотеку
    // - проверять доступность нескольких проксей

    let proxy: Box<dyn Connector> = {
        // https://www.firexproxy.com/
        
        const PROXIES: [&str; 4] = [
            "http://95.179.167.232:8080",
            "http://95.179.130.83:8080",
            "http://82.119.170.106:8080",
            "http://54.37.131.45:3128"
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
        
        let client = Client::builder()
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
        let update: Update = update?;
        //println!("Update: {:?}", update);

        match update.kind {
            // Тип обновления - сообщение
            UpdateKind::Message(message) => {
                match message.kind {
                    // Сообщение с текстом, ref нужен для получения ссылки на data из message, вместо переноса владения 
                    MessageKind::Text{ ref data, .. } =>  {
                        // Полученное сообщение
                        println!("<{}>: {}", &message.from.first_name, data);
                        
                        let text = format!("Hi, {}! You just wrote '{}'", &message.from.first_name, data);
                        let reply = message.text_reply(text);

                        // Answer message with "Hi".
                        api.send(reply).await?;
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
    Ok(())
}