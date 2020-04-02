// Подключаем сгенерированный ProtoBuf файлик и оборачиваем в модуль
pub mod simple_hello {
    tonic::include_proto!("helloworld");
}

use simple_hello::greeter_client::GreeterClient;
use simple_hello::HelloRequest;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.hello_command(request).await?;

    println!("RESPONSE = {:?}", response);

    Ok(())
}