use std::{
    env,
    time::Duration,
    //collections::HashMap
    //pin::Pin,
    //future::Future
};
use futures::{
    stream::FuturesUnordered,
    //Stream,
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
        //SemaphorePermit
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
    PROXIES,
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

async fn check_proxy_addr<S>(addr: S) -> Option<S>
where S: std::fmt::Display + std::string::ToString // addr_str
{
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


async fn get_valid_proxy_addresses<'a>(all_proxies: &[&'a str]) -> Option<Vec<&'a str>>{
    let semaphore = Semaphore::new(32);

    // Оптимальное хранилище для большого количества футур
    let check_futures_container: FuturesUnordered<_> = all_proxies
        .into_iter()
        .zip(std::iter::repeat(&semaphore))
        .map(|(addr, sem)| async move {
            // Ограничение максимального количества обработок
            let _lock = sem.acquire();
            let result = check_proxy_addr(*addr).await;
            result
        })
        .collect();
    
    // futures::pin_mut!(proxy_stream);

    // Получаем из стрима 10 валидных адресов проксей
    let valid_proxy_addresses: Vec<&str> = check_futures_container
        .filter_map(|addr_option| async move {
            addr_option
        })
        .take(10)
        .collect()
        .await;
    
    if !valid_proxy_addresses.is_empty(){
        Some(valid_proxy_addresses)
    }else{
        None
    }
}

async fn check_all_proxy_addresses_accessible<'a>(proxies: &[&'a str]) -> bool {
    let all_futures_iter = proxies
        .iter()
        .map(|addr|{
            check_proxy_addr(addr)
        });
    let result = futures::future::join_all(all_futures_iter).await;
    result
        .iter()
        .all(|res|{
            res.is_some()
        })
}

fn build_proxy_for_addresses(valid_proxy_addresses: &[&str]) -> Box<dyn Connector> {
    let proxy_iter = valid_proxy_addresses
        .iter()
        .map(|addr| {
            let proxy_uri: Uri = addr.parse().unwrap();
            let proxy: Proxy = Proxy::new(Intercept::All, proxy_uri);
            // proxy.set_authorization(Credentials::bearer(Token68::new("").unwrap()));
            // proxy.set_authorization(Credentials::basic("John Doe", "Agent1234").unwrap());
            proxy
        });

    let connector: HttpConnector<GaiResolver> = HttpConnector::new();
    let mut proxy_connector = ProxyConnector::new(connector).unwrap();
    proxy_connector.extend_proxies(proxy_iter);

    let client = hyper::Client::builder()
        .build(proxy_connector);

    Box::new(HyperConnector::new(client))
}

async fn process_update(api: &Api, update: Update){
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

async fn async_main(){
    // TODO: 
    // - добавить пример работы с прокси в библиотеку
    // - проверять доступность нескольких проксей, добавлять только доступные
    // - запрашивать откуда-то список проксей, затем по очереди проверять через прокси доступность телеграма, периодически обновлять активный прокси
    // - сортировать прокси по пингу
    // - добавить систему логирования

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

    // Таймер проверки проксей
    let mut proxy_check_timer = tokio::time::interval(Duration::from_secs(60*2));
    proxy_check_timer.tick().await; // Первый тик сбрасываем

    // Таймер проверки проксей
    let mut send_message_timer = tokio::time::interval(Duration::from_secs(60*5));
    //proxy_check_timer.tick().await; // Первый тик сбрасываем

    loop {
        // Получаем валидные адреса проксей
        let valid_proxy_addresses = get_valid_proxy_addresses(PROXIES)
            .await
            .expect("No valid proxies");

        // Создаем прокси
        let proxy: Box<dyn Connector> = build_proxy_for_addresses(&valid_proxy_addresses);

        // Подключаемся с использованием прокси
        // let api: Api = Api::new(token);
        let api: Api = Api::with_connector(token.clone(), proxy);

        // Дергаем новые обновления через long poll метод
        let mut stream: UpdatesStream = api.stream();

        println!("Stream created\n");

        'select_loop: loop {
            tokio::select! {
                // Таймер проверки проксей
                _ = proxy_check_timer.tick() => {
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
                _ = send_message_timer.tick() => {
                    //let messaage = telegram_bot::;
                    //api.send(messaage);
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
                    process_update(&api, update).await;
                }
            }
        }

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