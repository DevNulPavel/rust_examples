// Импортируем результат работы Proto - имя пакета
pub mod info {
    tonic::include_proto!("info");
}

use tonic::transport::Server;
use tonic::{Request, Response, Status};
use info::computer_info_server::{ComputerInfo, ComputerInfoServer}; // route_guide_server::RouteGuideServer генерируется protobuf
use info::{Stats, InfoRequest};

// Описываем класс сервиса
#[derive(Debug)]
pub struct ComputerInfoService {
}

// Реализация трейта-обработчика
#[tonic::async_trait]
impl ComputerInfo for ComputerInfoService {
    // Вызов получения особенности, конверируется из GetFeature
    async fn get_stats(&self, request: Request<InfoRequest>) -> Result<Response<Stats>, Status> {
        println!("GetFeature = {:?}", request);

        // TODO: Параметром для запуска сервера должен быть диск с Jenkins
        // https://doc.rust-lang.org/reference/conditional-compilation.html
        let free_space = if cfg!(target_family = "unix") {
            fs2::available_space("/").unwrap_or(0)
        }else if cfg!(target_family = "windows"){
            // TODO: All disks
            fs2::available_space("C:").unwrap_or(0) + fs2::available_space("D:").unwrap_or(0)
        }else{
            0
        };

        let total_space = if cfg!(target_family = "unix") {
            fs2::total_space("/").unwrap_or(0)
        }else if cfg!(target_family = "windows"){
            // TODO: All disks
            fs2::total_space("C:").unwrap_or(0) + fs2::total_space("D:").unwrap_or(0)
        }else{
            0
        };

        let result = Stats{
            total_space: total_space as u64,
            free_space: free_space as u64,
            server_name: String::new()
        };

        // Если нет - пустой вариаант
        Ok(Response::new(result))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:10000".parse().unwrap(); // V6 - [::1]:10000
    println!("Computer info listening on: {}", addr);

    // Создаем наш сервис
    let info_service = ComputerInfoService {
    };

    // Создаем непосредственно сервер
    let server = ComputerInfoServer::new(info_service);

    Server::builder()
        .add_service(server)
        .serve(addr)
        .await?;

    Ok(())
}