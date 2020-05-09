
use std::{
    time::Duration,
};
use futures::{
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
    }
};
use reqwest::{
    Client,
    //ClientBuilder,
};

async fn check_proxy_addr<S>(addr: S) -> Option<S>
    where S: std::fmt::Display + std::string::ToString {

    // TODO: копирование
    let addr_str: String = addr.to_string();
    let proxy = reqwest::Proxy::all(&addr_str).unwrap();
    let client: Client = reqwest::ClientBuilder::new()
        .proxy(proxy)
        .timeout(Duration::from_secs(25))
        .connect_timeout(Duration::from_secs(25))
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

/*
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

async fn get_http_proxies() -> Result<Vec<String>, reqwest::Error>{
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


pub async fn get_valid_proxy_addresses<'a>(all_proxies: &[&'a str]) -> Option<Vec<&'a str>>{
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

pub async fn check_all_proxy_addresses_accessible<'a>(proxies: &[&'a str]) -> bool {
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

pub fn build_proxy_for_addresses(valid_proxy_addresses: &[&str]) -> Box<dyn Connector> {
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
