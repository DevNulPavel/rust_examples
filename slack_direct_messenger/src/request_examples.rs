/*use tokio::prelude::*;
use tokio::net::TcpListener;
use hyper::Client;

async fn test_tokio_server() -> Result<(), Box<dyn std::error::Error>>{
    // Создаем серверный сокет
    let mut listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        // В цикле получаем новые подключения
        let (mut socket, _) = listener.accept().await?;

        // Закидываем на обработку новые асинхронные функции для обработки подключения
        tokio::spawn(async move {
            // Создаем буффер
            let mut buf = [0; 1024];

            // Начинаем читать в цикле из сокета
            loop {
                // Пробуем прочитать в сокет    
                let n = match socket.read(&mut buf).await {
                    // Сокет у нас закрыт, прочитали 0 данных - значит выходим из данного обработчика сокета
                    Ok(n) if (n == 0) => {
                        return;
                    },
                    // Если у нас ненулевое значение прочитано, значит все ок
                    Ok(n) => {
                        n
                    },
                    // Если ошибка, выводим ее и выходим из обработчика сокета
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Пишем данные назад, в случае ошибки - выводим ее и выходим
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
    //Ok(())
}

async fn perform_get_request_with_hyper() -> Result<(), Box<dyn std::error::Error>> {
    // Still inside `async fn main`...
    let client = Client::new();

    // Parse an `http::Uri`...
    let uri = "http://httpbin.org/ip".parse()?;

    // Await the response...
    let resp = client.get(uri).await?;

    println!("Response status: {}", resp.status());

    // And now...
    // while let Some(chunk) = resp.body_mut().data().await {
    //     stdout().write_all(&chunk?).await?;
    // }

    Ok(())
}

async fn perform_get_request_with_reqwest() -> Result<(), Box<dyn std::error::Error>> {
    // let resp: HashMap<String, String> = reqwest::get("https://httpbin.org/ip")
    //     .await?
    //     .json()
    //     .await?;
    // println!("{:#?}", resp);

    // let body = reqwest::get("https://www.rust-lang.org")
    //     .await?
    //     .text()
    //     .await?;
    //println!("body = {:?}", body);

    //https://docs.rs/reqwest/0.10.0-alpha.2/reqwest/

    // This will POST a body of `{"lang":"rust","body":"json"}`
    let mut map = HashMap::new();
    map.insert("lang", "rust");
    map.insert("body", "json");
    let client = reqwest::Client::new();
    let res = client.post("http://httpbin.org/post")
        .json(&map)
        .send()
        .await?;
    println!("result = {:?}", res);

    Ok(())
}

// let result = test_tokio_server();
// result.await?;
//perform_get_request_with_hyper().await?;
//perform_get_request_with_reqwest().await?;*/