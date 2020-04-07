// Импортируем результат работы Proto - имя пакета
pub mod info {
    tonic::include_proto!("info");
}

use std::time::Duration;
use clap::{Arg, App, ArgMatches};
use tonic::{Request, Response};
use tokio::time::delay_for;
use info::computer_info_client::ComputerInfoClient; // route_guide_server::RouteGuideServer генерируется protobuf
use info::{Stats, InfoRequest};
use jenkins_stat;
//use futures::join;

#[derive(Debug)]
struct StatRequestResult<'a>{
    stats: Stats,
    server_name: &'a str
}

fn get_app_parameters() -> ArgMatches<'static> {
    // Parse parameters
    App::new("stats_receiver")
        .version("1.0")
        .author("Pavel Ershov")
        .about("Jenkins stats receiver + slack posting")
        // Channel
        .arg(Arg::with_name("slack_channel")
            .long("slack_channel")
            .help("Slack channel")
            .required(true)
            .takes_value(true))
        // Servers
        .arg(Arg::with_name("stat_servers")
            .long("stat_servers")
            .help("Servers")
            .multiple(true)
            .required(true)
            .takes_value(true))            
        .get_matches()
}

type Client = info::computer_info_client::ComputerInfoClient<tonic::transport::channel::Channel>;

async fn get_client(addr: &str) -> Result<Client, Box<dyn std::error::Error>> {
    let mut ip_addr_iter = tokio::net::lookup_host(addr).await?;
    loop{
        if let Some(ip_addr) = ip_addr_iter.next(){
            let ip_addr: std::net::SocketAddr = ip_addr;
            println!("Ip address found: {}", ip_addr);

            let ip_str = format!("http://{}:{}", ip_addr.ip(), ip_addr.port());

            if let Ok(client) = ComputerInfoClient::connect(ip_str).await {
                return Ok(client);
            }else{
                continue;
            }
        }else{
            break;   
        }
    }
    return Err("Connection failed".into());
}

async fn get_stats_from_addres<'a>(addr: &'a str) -> Result<StatRequestResult<'a>, Box<dyn std::error::Error>>{
    let request_future = async {

        // Создаем клиента
        let mut client = get_client(addr).await?;

        // Запрашиваем с сервера особенности в данной точке
        let response: Response<Stats> = client
            .get_stats(Request::new(InfoRequest{}))
            .await?;

        // println!("RESPONSE = {:?}", response);
        let stats: Stats = response.into_inner();

        Ok(StatRequestResult{
            stats,
            server_name: addr
        })  
    };
    // Создаем фьючу с задержкой для таймаута
    let delay_future = delay_for(Duration::from_secs(5));
    // Смотрим, что сработает первое
    tokio::select! {
        result = request_future => {
            result
        },
        _ = delay_future => {
            Err("Timeout".into())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let params: ArgMatches = get_app_parameters();
    // Канал слака, куда постим все
    let slack_channel = params.value_of("slack_channel")
        .unwrap();
    // Вектор из серверов
    let servers: Vec<&str> = params.values_of("stat_servers")
        .unwrap()
        .collect();

    // Api token
    let api_token = std::env::var("SLACK_API_TOKEN")
        .expect("SLACK_API_TOKEN environment variable is missing");


    // Итератор по фьючам запросов к серверу
    let stat_futures = servers
        .into_iter()
        .map(|addr|{
            get_stats_from_addres(addr) // 
        });

    // Выполняем параллельно все запросы и собираем результаты
    let all_stats: Vec<Result<StatRequestResult, _>>  = futures::future::join_all(stat_futures)
        .await;

    // Собираем результаты от серверов в кучу
    let stat_info: String = all_stats
        .into_iter()
        .filter_map(|stat|{
            // Используем только успешные результаты
            stat.ok()
        })
        .fold(String::new(), |prev, stat|{
            // Создаем строку с информацией о сервере
            let formatted = format!("Server {}: free space = {}, total size = {}\n", 
                stat.server_name, 
                bytesize::ByteSize(stat.stats.free_space), 
                bytesize::ByteSize(stat.stats.total_space));

            prev + formatted.as_str()
        });

    if stat_info.is_empty() {
        return Ok(());
    }

    println!("{:?}", stat_info);        

    // Создаем клиента для переиспользования
    // let client = reqwest::Client::new();

    // Пишем в слак
    // jenkins_stat::send_message_to_channel(&client, &api_token, &slack_channel, &stat_info).await?;

    Ok(())
}