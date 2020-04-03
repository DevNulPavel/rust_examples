// Импортируем результат работы Proto - имя пакета
pub mod info {
    tonic::include_proto!("info");
}

//use std::collections::hash_map::HashMap;
use clap::{Arg, App, ArgMatches};
use tonic::{Request, Response};
use info::computer_info_client::ComputerInfoClient; // route_guide_server::RouteGuideServer генерируется protobuf
use info::{Stats, InfoRequest};
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

async fn get_stats_from_addres<'a>(addr: &'a str) -> Result<StatRequestResult<'a>, Box<dyn std::error::Error>>{
    // Создаем клиента
    let mut client = ComputerInfoClient::connect(String::from(addr))
        .await?; // "http://[::1]:10000"

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let params: ArgMatches = get_app_parameters();
    let slack_channel = params.value_of("slack_channel")
        .unwrap();
    let servers: Vec<&str> = params.values_of("stat_servers")
        .unwrap()
        .collect();

    let stat_futures: Vec<_> = servers
        .into_iter()
        .map(|addr|{
            get_stats_from_addres(addr) // 
        })
        .collect();

    let all_stats: Vec<Result<StatRequestResult, _>>  = futures::future::join_all(stat_futures.into_iter())
        .await;

    let stat_info: String = all_stats
        .into_iter()
        .filter_map(|stat|{
            stat.ok()
        })
        .fold(String::new(), |prev, stat|{
            //println!("{:?}", stat); 
            //use std::ops::Add;

            let stat: StatRequestResult = stat;
            let prev: String = prev;

            let formatted = format!("Server {}: free space = {}, total size = {}\n\n", 
                stat.server_name, 
                bytesize::ByteSize(stat.stats.free_space), 
                bytesize::ByteSize(stat.stats.total_space));

            prev + formatted.as_str()
        });

    println!("{:?}", stat_info);        

    // for stat in all_stats{}

    // let total_size = bytesize::ByteSize(stats.total_space);
    // let free_space = bytesize::ByteSize(stats.free_space);
    // println!("Stats = {:?}", stats);
    // println!("Stats = {}", total_size);
    // println!("Stats = {}", free_space);

    Ok(())
}