use futures::StreamExt;
use libp2p::{
    multiaddr::Protocol,
    request_response::{Event, Message},
    swarm::SwarmEvent,
    Multiaddr,
};
use libp2p_research::{create_swarm, FileResponse, P2PError, SwarmCreateResult, SwarmP2PType};
use std::net::Ipv4Addr;

////////////////////////////////////////////////////////////////////////////////

/// Начало работы приложения
fn main() {
    // Создаем Swarm
    let SwarmCreateResult { mut swarm, peer_id } = create_swarm().expect("swarm_create");

    // Сразу же выведем для мониторинга идентификатор нашего пира
    println!("Current peer id: {:?}", peer_id);

    // Слушаем на определенном адресе и порте
    {
        // Пример с парсингом адреса
        // "/ip4/0.0.0.0/tcp/0".parse()?

        // TODO: Не совсем понятна здесь идея протоколов и слушающих адресов
        let listen_address = Multiaddr::with_capacity(2)
            .with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Tcp(9999));

        // Будем слушать на определенных адресах и протоколах
        swarm.listen_on(listen_address).expect("swarm_listen");

        println!("Swarm запущен. Ожидаем входящего подключения...");
    }

    // Подключаемся к другому узлу
    // let remote: Multiaddr = "/ip4/127.0.0.1/tcp/8080/p2p/12D3KooW...".parse()?;
    // swarm.dial(remote)?;
    // println!("Подключение к {}", remote)?;

    // Создаем теперь уже асинхронный рантайм
    tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .expect("Tokio runtime build failed")
        .block_on(async_loop(swarm))
        .expect("main_swarm_loop");
}

////////////////////////////////////////////////////////////////////////////////

/// Непосредственно наша исполняема асинхронная часть
async fn async_loop(mut swarm: SwarmP2PType) -> Result<(), P2PError> {
    // Основной цикл
    while let Some(next_event) = swarm.next().await {
        // Обработка остальных событий
        #[allow(clippy::single_match)]
        match next_event {
            // Какие-то конкретные события протокола и кодека
            SwarmEvent::Behaviour(behaviour_event) => {
                // Смотрим что там за событие вообще
                match behaviour_event {
                    // Сообщение какое-то от какого-то пира
                    Event::Message { peer, message } => {
                        // Смотрим на сообщения теперь
                        match message {
                            // Это какой-то запрос
                            Message::Request {
                                request, channel, ..
                            } => {
                                // Содержимое
                                println!(
                                    "File content request from peer '{}': '{}'",
                                    peer,
                                    request.file_path.display()
                                );

                                // TODO: Асинхронное чтение файлика
                                let response = match std::fs::read(request.file_path.as_path()) {
                                    Ok(content) => FileResponse::Found {
                                        content: content.into_boxed_slice(),
                                    },
                                    Err(_) => {
                                        // TODO: Залогировать ошибку
                                        FileResponse::NotAvailable
                                    }
                                };

                                // Пробуем поставить в очередь на отправку, при ошибке
                                // нам вернется объект ответа назад
                                let send_response_res =
                                    swarm.behaviour_mut().send_response(channel, response);

                                // Проверяем статус ответа
                                if send_response_res.is_err() {
                                    // TODO: Залогировать ошибку постановки в очередь отправки
                                }
                            }
                            // Получен ответ
                            Message::Response { response, .. } => {
                                // Смотрим на наш ответ
                                match response {
                                    FileResponse::Found { content } => {
                                        // Здесь можно обработать полученные данные
                                        println!(
                                            "File content response length: {} bytes",
                                            content.len()
                                        );
                                    }
                                    FileResponse::NotAvailable => {
                                        eprintln!("File not available");
                                    }
                                }
                            }
                        }
                    }
                    // Успешно смогли отправить сообщение
                    Event::ResponseSent { peer, .. } => {
                        println!("Answer sent to '{}'", peer);
                    }
                    // Проблемы с отправкой сообщения
                    Event::OutboundFailure { peer, error, .. } => {
                        eprintln!("Request send to '{}' failed: {:?}", peer, error);
                    }
                    // Проблема с входящим сообщением
                    Event::InboundFailure { peer, error, .. } => {
                        eprintln!("Request receive from '{}' failed: {:?}", peer, error);
                    }
                }
            }
            // TODO: Обработка остальных событий, например, отвала клиента и тд
            _ => {}
        }
    }

    Ok(())
}
