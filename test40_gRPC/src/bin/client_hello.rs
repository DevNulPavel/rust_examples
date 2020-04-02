// Подключаем сгенерированный ProtoBuf файлик и оборачиваем в модуль
pub mod simple_hello {
    tonic::include_proto!("simple_hello"); // Тут надо указать имя нашего файлика
}

// ProtoBuf генерирует пространство имен для сервера вида: UpperConverter -> <нижний регистр, _ разделитель>_client
// в пространстве имен upper_converter_client появляются классы *Client
use simple_hello::upper_converter_client::UpperConverterClient;
use simple_hello::HelloRequest;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Подключаемся к серверу
    let mut client = UpperConverterClient::connect("http://127.0.0.1:50051").await?; // V6 localhost: "[::1]:50051"

    // Объект данных запроса
    let request_object = HelloRequest {
        name: "Tonic".into(),
    };
    // Запрос
    let request = tonic::Request::new(request_object);

    // Получаем ответ
    let response = client
        .hello_command(request)
        .await?;

    println!("RESPONSE = {:?}", response);

    Ok(())
}

// Вместо оборачивания в #[tokio::main] - запускаем
/*fn main(){
    // Создаем Tokio Runtime
    let mut runtime = tokio::runtime::Builder::new()
        .basic_scheduler() // Single-threaded
        //.threaded_scheduler() // Multi-threade
        .on_thread_start(|| {
            println!("thread started");
        })
        .thread_name("Tokio thread")
        .build()
        .unwrap();

    runtime
        .block_on(async {
            println!("Async started");
            tokio_loop().await
        })
        .unwrap();
}*/