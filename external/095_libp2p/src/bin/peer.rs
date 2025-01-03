use async_trait::async_trait;
use libp2p::{
    identity, request_response::Config as RequestResponseConfig, swarm::handler::ProtocolSupport,
    PeerId, Swarm, Transport,
};
use libp2p_research::{
    create_swarm, error::P2PError, transport::create_transport, SwarmCreateResult,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////

/// Начало работы приложения
fn main() {
    // Создаем Swarm
    let SwarmCreateResult { mut swarm, peer_id } = create_swarm().expect("swarm_create");

    // Сразу же выведем для мониторинга идентификатор нашего пира
    println!("Current peer id: {:?}", peer_id);

    // Слушаем на всех доступных интерфейсах и портах
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    println!("Swarm запущен. Ожидаем входящего подключения...");

    // Подключаемся к другому узлу
    // let remote: Multiaddr = "/ip4/127.0.0.1/tcp/8080/p2p/12D3KooW...".parse()?;
    // swarm.dial(remote)?;
    // println!("Подключение к {}", remote)?;

    // Создаем теперь уже асинхронный рантайм
    tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .expect("Tokio runtime build failed")
        .block_on(async_main(swarm));
}

////////////////////////////////////////////////////////////////////////////////

/// Непосредственно наша исполняема асинхронная часть
async fn async_loop(swarm: Swarm<Behaviour<FileCodec>>) -> Result<(), P2PError> {
    // Основной цикл
    loop {
        match swarm.next().await? {
            RequestResponseEvent::Message { peer, message } => {
                match message {
                    RequestResponseMessage::Request {
                        request, channel, ..
                    } => {
                        println!("Получен запрос на файл: {}", request.filename);
                        // Здесь должна быть логика поиска и чтения файла
                        // Для примера отправим заглушку
                        let response = match std::fs::read(&request.filename) {
                            Ok(content) => FileResponse::Found { content },
                            Err(_) => FileResponse::NotFound,
                        };

                        swarm.respond(channel, response)?;
                    }
                    RequestResponseMessage::Response { response, .. } => {
                        println!(
                            "Получен ответ с содержимым файла: {} байт",
                            response.content.len()
                        );
                        // Здесь можно обработать полученные данные
                    }
                }
            }
            RequestResponseEvent::OutboundFailure { peer, error, .. } => {
                eprintln!("Ошибка при отправке запроса к {}: {:?}", peer, error);
            }
            RequestResponseEvent::InboundFailure { peer, error, .. } => {
                eprintln!("Ошибка при получении запроса от {}: {:?}", peer, error);
            }
            RequestResponseEvent::ResponseSent { peer, .. } => {
                println!("Ответ отправлен к {}", peer);
            }
            _ => {}
        }
    }
}
