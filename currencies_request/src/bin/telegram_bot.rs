use std::{
    env,
    time::Duration,
    //collections::HashMap
    //pin::Pin,
    //future::Future
};
use futures::{
    Stream,
    StreamExt,
    //FutureExt
};
use serde::{
    Deserialize, 
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
    sync::{
        Semaphore,
        SemaphorePermit
    },
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

async fn check_proxy_addr<S>(addr: S, semaphore: &Semaphore) -> Option<S>
where S: std::fmt::Display + std::string::ToString // addr_str
{
    // Ограничение максимального количества обработок
    let _lock = semaphore.acquire().await;

    // TODO: копирование
    let addr_str: String = addr.to_string();
    let proxy = reqwest::Proxy::all(&addr_str).unwrap();
    let client: Client = reqwest::ClientBuilder::new()
        .proxy(proxy)
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let req = client.get("https://api.telegram.org")
        .build()
        .unwrap();
    let res = client.execute(req).await;
    
    //println!("Result: {:?}", res);

    if res.is_ok() {
        println!("Valid addr: {}", addr);
        Some(addr)
    }else{
        println!("Invalid addr: {}", addr);
        None
    }
}

/*async fn check_proxy_addr(addr: String) -> Option<String>
{
    let proxy = reqwest::Proxy::all(addr.as_str()).unwrap();
    let client: Client = reqwest::ClientBuilder::new()
        .proxy(proxy)
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let req = client.get("https://api.telegram.org")
        .build()
        .unwrap();
    let res = client.execute(req).await;
    
    //println!("Result: {:?}", res);

    if res.is_ok() {
        println!("Valid addr: {}", addr);
        Some(addr)
    }else{
        println!("Invalid addr: {}", addr);
        None
    }
}*/

#[derive(Deserialize, Debug)]
struct ProxyInfo{
    #[serde(rename(deserialize = "ipPort"))]
    addr: String,
}

#[derive(Deserialize, Debug)]
struct ProxyResponse{
    data: Vec<ProxyInfo>,
    count: i32
}

/*async fn get_http_proxies() -> Result<Vec<String>, reqwest::Error>{
    let valid_addresses = loop {
        let result: ProxyResponse  = reqwest::get("http://pubproxy.com/api/proxy?type=http&limit=5?level=anonymous?post=true") // ?not_country=RU,BY,UA
            .await?
            .json()
            .await?;

        //println!("{:?}", result);
        let http_addresses_array: Vec<String> = result
            .data
            .into_iter()
            .map(|info|{
                format!("http://{}", info.addr)
            })
            .collect();

        let check_futures_iter = http_addresses_array
            .into_iter()
            .map(|addr|{
                check_proxy_addr(addr)
            });
        let check_results = futures::future::join_all(check_futures_iter).await;
        
        let valid_address: Vec<String> = check_results
            .into_iter()
            .filter_map(|addr_option|{
                addr_option
            })
            // .map(|addr|{
            //     addr
            // })
            .collect();

        if valid_address.len() > 0 {
            break valid_address;
        }
    };

    Ok(valid_addresses)
}*/

async fn async_main(){
    // TODO: 
    // - добавить пример работы с прокси в библиотеку
    // - проверять доступность нескольких проксей, добавлять только доступные
    // - запрашивать откуда-то список проксей, затем по очереди проверять через прокси доступность телеграма, периодически обновлять активный прокси

    let proxy: Box<dyn Connector> = {
        // https://www.firexproxy.com/
        // http://free-proxy.cz/ru/
        // http://spys.one/proxylist/
        // https://free-proxy-list.net/
        // http://pubproxy.com/
        // http://pubproxy.com/api/proxy?type=http
        // http://pubproxy.com/api/proxy?type=http&limit=5
       
        const PROXIES: &[&str] = &[
            "http://185.134.23.196:80",
            "http://191.96.42.80:3128",
            "http://12.139.101.101:80",
            "http://54.37.103.99:3128",
            "http://138.197.133.143:8080",
            "http://192.41.71.204:3128",
            "http://192.41.71.199:3128",
            "http://12.139.101.97:80",
            "http://34.222.244.181:3129",
            "http://93.190.253.50:80",
            "http://46.4.96.87:80",
            "http://209.141.49.11:8080",
            "http://77.73.241.154:80",
            "http://91.134.24.240:3128",
            "http://138.68.240.218:8080",
            "http://185.134.23.198:80",
            "http://5.226.141.223:80",
            "http://68.185.57.66:80",
            "http://192.41.13.71:3128",
            "http://151.80.199.89:3128",
            "http://212.83.163.112:5836",
            "http://188.226.141.211:3128",
            "http://66.232.217.154:8080",
            "http://185.21.217.8:3128",
            "http://198.98.56.71:8080",
            "http://51.158.107.202:8811",
            "http://207.148.25.145:8080",
            "http://46.4.96.67:80",
            "http://93.174.94.80:8080",
            "http://159.203.44.177:3128",
            "http://85.26.146.169:80",
            "http://217.61.0.167:8080",
            "http://165.227.15.78:3128",
            "http://3.10.190.54:80",
            "http://185.156.173.154:80",
            "http://144.217.83.160:8080",
            "http://192.99.244.148:8080",
            "http://178.128.159.243:5836",
            "http://159.203.170.62:80",
            "http://149.28.44.208:8080",
            "http://45.76.10.181:8080",
            "http://174.138.42.112:8080",
            "http://159.203.2.130:80",
            "http://167.172.254.5:8080",
            "http://70.110.31.20:8080",
            "http://68.188.59.198:80",
            "http://50.206.25.108:80",
            "http://50.206.25.106:80",
            "http://50.206.25.111:80",
            "http://50.206.25.110:80",
            "http://198.98.54.241:8080",
            "http://71.13.131.142:80",
            "http://138.197.32.120:3128",
            "http://52.179.231.206:80",
            "http://54.156.164.61:80",
            "http://50.206.25.107:80",
            "http://52.161.188.147:80",
            "http://64.235.204.107:8080",
            "http://165.22.41.190:80",
            "http://52.161.188.146:80",
            "http://162.251.61.196:3128",
            "http://52.161.188.149:80",
            "http://52.161.188.145:80",
            "http://96.113.201.134:3128",
            "http://198.255.114.82:3128",
            "http://138.68.53.220:5836",
            "http://161.35.50.98:3128",
            "http://168.169.146.12:8080",
            "http://12.139.101.98:80",
            "http://185.134.23.197:80",
            "http://52.2.120.147:3128",
            "http://174.138.42.112:8080",
            "http://52.179.231.206:80",
            "http://80.187.140.26:8080",
            "http://95.179.167.232:8080",
            "http://95.179.130.83:8080",
            "http://82.119.170.106:8080",
            "http://54.37.131.45:3128",
            "http://51.158.68.68:8811",
            "http://51.91.212.159:3128",
        ];

        let semaphore = Semaphore::new(16);

        // Оптимальное хранилище для большого количества футур
        let check_futures_iter: futures::stream::FuturesUnordered<_> = PROXIES
            .into_iter()
            .map(|addr|{
                check_proxy_addr(*addr, &semaphore)
            })
            .collect();

        let proxy_stream = check_futures_iter
            .filter_map(|addr_option| async move {
                addr_option
            })
            .take(5)
            .map(|addr| {
                let proxy_uri: Uri = addr.parse().unwrap();
                let proxy: Proxy = Proxy::new(Intercept::All, proxy_uri);
                // proxy.set_authorization(Credentials::bearer(Token68::new("").unwrap()));
                // proxy.set_authorization(Credentials::basic("John Doe", "Agent1234").unwrap());
                proxy
            });
        futures::pin_mut!(proxy_stream);

        //let val: Option<&str> = proxy_stream.next().await;
        
        /*let check_futures_iter = PROXIES
            .into_iter()
            .map(|addr|{
                check_proxy_addr(*addr, &semaphore)
            });
        let proxies_iter = futures::future::join_all(check_futures_iter)
            .await
            .into_iter()
            .filter_map(|addr_option|{
                addr_option
            })
            .map(|addr|{
                let proxy_uri: Uri = addr.parse().unwrap();
                let proxy: Proxy = Proxy::new(Intercept::All, proxy_uri);
                // proxy.set_authorization(Credentials::bearer(Token68::new("").unwrap()));
                // proxy.set_authorization(Credentials::basic("John Doe", "Agent1234").unwrap());
                proxy
            });*/

        /*let addresses = get_http_proxies()
            .await
            .unwrap();
        let proxies_iter = addresses
            .into_iter()
            .map(|addr|{
                let proxy_uri: Uri = addr.parse().unwrap();
                let proxy: Proxy = Proxy::new(Intercept::All, proxy_uri);
                // proxy.set_authorization(Credentials::bearer(Token68::new("").unwrap()));
                // proxy.set_authorization(Credentials::basic("John Doe", "Agent1234").unwrap());
                proxy
            });*/

        let connector: HttpConnector<GaiResolver> = HttpConnector::new();
        let mut proxy_connector = ProxyConnector::new(connector).unwrap();
        while let Some(sock) = proxy_stream.next().await{
            proxy_connector.add_proxy(sock);
        } 
        // proxy_connector.extend_proxies(proxies_iter);

        if proxy_connector.proxies().is_empty() {
            panic!("No valid proxies");
        }

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