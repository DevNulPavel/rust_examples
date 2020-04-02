// Proto файлы оборачиваем в подмодуль
pub mod hello_world {
    tonic::include_proto!("simple_hello"); // Тут надо указать имя нашего файлика
}

use tonic::{transport::Server, Request, Response, Status};

// ProtoBuf генерирует пространство имен для сервера вида: UpperConverter -> <нижний регистр, _ разделитель>_server
// в пространстве имен upper_converter_server появляются классы *Server
use hello_world::upper_converter_server::{UpperConverter, UpperConverterServer};
// Команды и ответы - импортируются как есть
use hello_world::{HelloReply, HelloRequest};

// Так как UpperConverter - это всего лишь trait, нужно создать собственный класс
#[derive(Debug, Default)]
pub struct UpperConverterSystem {}

#[tonic::async_trait]
impl UpperConverter for UpperConverterSystem {
    // Описанные команды в proto конвертируются в нижний регистр c _ разделителем
    // HelloCommand -> hello_command
    async fn hello_command(&self, request: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        // Текст запроса
        let request_data: HelloRequest = request.into_inner();
        let response_text = format!("Hello {}!", request_data.name);

        // Формируем ответ
        let reply: HelloReply = hello_world::HelloReply {
            message: response_text.into(),
        };

        // Отдаем ответ
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Адрес сервера
    let addr = "127.0.0.1:50051".parse()?; // V6 localhost: "[::1]:50051"
    // Наш объект сервиса
    let system = UpperConverterSystem::default();
    // Сам сервер
    let server = UpperConverterServer::new(system);

    // Зарускаем в работу сервер
    Server::builder()
        .add_service(server)
        .serve(addr)
        .await?;

    Ok(())
}