
use std::{
    time::Duration,
};
use log::{
    info,
    // warn,
    // debug,
    error
};
use futures::{
    //FutureExt,
    stream::FuturesUnordered,
    StreamExt,
};
use hyper_proxy::{
    Proxy, 
    ProxyConnector, 
    Intercept
};
use hyper::{
    Uri,
    client::{
        HttpConnector,
        connect::dns::GaiResolver
    }
};
use telegram_bot::{
    connector::Connector,
    connector::hyper::HyperConnector,
};
use tokio::{
    sync::{
        Semaphore,
        //SemaphorePermit
    },
    /*sync::{
        mpsc::{
            Receiver,
            channel
        }
    }*/
};
use reqwest::{
    Client,
    //ClientBuilder,
};
use serde::{
    Deserialize
};
use crate::{
    constants::{
        PROXIES
    },
    error::{
        TelegramBotError
    }
};

/// Проверяем адрес на валидность
async fn check_proxy_addr<S>(addr: S) -> Option<S>
    where S: std::fmt::Display + AsRef<str> {

    // TODO: копирование
    let addr_str: &str = addr.as_ref();
    let proxy = match reqwest::Proxy::all(addr_str){
        Ok(proxy) => {
            proxy
        },
        Err(e)=>{
            error!("Proxy create error: {}", e);
            return None;      
        }
    };
    let client: Client = reqwest::ClientBuilder::new()
        .proxy(proxy)
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(30))
        .build()
        .expect("Proxy client build failed");
    let req = client.get("https://api.telegram.org")
        .build()
        .expect("Proxy request build failed");
    let res = client.execute(req).await;
    
    //info!("Result: {:?}", res);

    if res.is_ok() {
        info!("Valid addr: {}", addr);
        Some(addr)
    }else{
        //info!("Invalid addr: {}", addr);
        None
    }
}

/// Запрашивает из HTTP API адреса проксей
async fn get_http_proxies_1() -> Result<Vec<String>, TelegramBotError>{
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

    // ?not_country=RU,BY,UA
    const URL: &str = "http://pubproxy.com/api/proxy?type=http&limit=5?level=anonymous?post=true";
    let result: ProxyResponse  = reqwest::get(URL)
        .await?
        .json()
        .await?;

    //info!("{:?}", result);
    let http_addresses_array: Vec<String> = result
        .data
        .into_iter()
        .map(|info|{
            format!("http://{}", info.addr)
        })
        .collect();

    Ok(http_addresses_array)
}

// Вариант с параллельным получением результатов
/*fn build_http_1_proxies_stream() -> Receiver<Option<String>> {
    let (mut tx, rx) = channel(32);
    tokio::spawn(async move{
        let http_1_proxies_res = get_http_proxies_1().await;
        if let Ok(http_1_proxies) = http_1_proxies_res {
            info!("Http 1 proxies request resolved");
            for addr in http_1_proxies {
                let result: Option<String> = check_proxy_addr(addr.to_string()).await;
                if tx.send(result).await.is_err(){
                    info!("Http 1 proxies request exit");
                    return;
                }
            }
        }
        info!("Http 1 proxies request finished");
    });
    rx
}

fn get_static_proxies_stream() -> Receiver<Option<String>>{
    let (mut tx, rx) = channel(32);
    tokio::spawn(async move {
        let semaphore = Semaphore::new(32);

        let mut static_futures_stream: FuturesUnordered<_> = PROXIES
            .into_iter()
            .zip(std::iter::repeat(&semaphore))
            .map(|(addr, sem)| async move {
                // Ограничение максимального количества обработок
                let _lock = sem.acquire();
                let result: Option<String> = check_proxy_addr(addr.to_string()).await;
                result
            })
            .collect();

        while let Some(addr) = static_futures_stream.next().await {
            if tx.send(addr).await.is_err(){
                return;
            }
        }
    });
    rx
}*/

/// Получаем вектор из валидных адресов
async fn get_first_valid_addresses<I, T>(addresses: I, max_count: usize, max_connections: usize) -> Vec<T>
where I: std::iter::IntoIterator<Item=T>,
      T: AsRef<str> + std::fmt::Display
{
    let semaphore = Semaphore::new(max_connections);

    let futures_stream: FuturesUnordered<_> = addresses
        .into_iter()
        .zip(std::iter::repeat(&semaphore))
        .map(|(addr, sem)| async move {
            // Ограничение максимального количества обработок
            let _lock = sem.acquire();
            let result: Option<T> = check_proxy_addr(addr).await;
            result
        })
        .collect();

    let valid_proxy_addresses: Vec<T> = futures_stream
        .filter_map(|addr_option| async move {
            addr_option
        })
        .take(max_count)
        .collect()
        .await;

    valid_proxy_addresses
}

/// Получаем немного валидных прокси-адресов
pub async fn get_valid_proxy_addresses() -> Option<Vec<String>>{
    // Вариант с параллельным получением результатов
    /*let mut streams: [Receiver<Option<String>>; 2] = [
        get_static_proxies_stream(),  // Стрим из статически сохраненных проксей
        build_http_1_proxies_stream() // Стрим из http адресов
    ];

    // Получаем из стрима 10 валидных адресов проксей
    let streams_iter = streams.iter_mut();
    let valid_proxy_addresses: Vec<String> = futures::stream::select_all(streams_iter)
        .filter_map(|addr_option| async move {
            addr_option
        })
        .take(3)
        .collect()
        .await;
    */

    // Сначала проверяем статические адреса
    {
        let valid_proxy_addresses: Vec<&&str> = get_first_valid_addresses(PROXIES, 3, 8).await;

        if !valid_proxy_addresses.is_empty(){
            let vec = valid_proxy_addresses
                .into_iter()
                .map(|val| val.to_string())
                .collect();
            return Some(vec);
        }
    }

    // Потом проверяем HTTP адреса из запроса
    {
        let http_1_proxies_res = get_http_proxies_1() .await;
        if let Ok(http_1_proxies) = http_1_proxies_res {
            info!("Http 1 proxies request resolved");

            let valid_proxy_addresses: Vec<String> = get_first_valid_addresses(http_1_proxies, 3, 8).await;
            
            if !valid_proxy_addresses.is_empty(){
                return Some(valid_proxy_addresses);
            }
        }
    }

    None
}

/// Проверяем, что все прокси-адреса из списка доступны
pub async fn check_all_proxy_addresses_accessible(proxies: &[String]) -> bool {
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

/// Создаем прокси из валидных адресов
pub fn build_proxy_for_addresses(valid_proxy_addresses: &[String]) -> Box<dyn Connector> {
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
