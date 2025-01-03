////////////////////////////////////////////////////////////////////////////////

use crate::{
    codec::{CodecProtocolName, FileCodec},
    error::P2PError,
    transport::create_transport,
};
use async_trait::async_trait;
use libp2p::{
    identity,
    multiaddr::Protocol,
    request_response::{Codec, Config as RequestResponseConfig},
    swarm::handler::ProtocolSupport,
    PeerId, Swarm, Transport,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////

pub struct SwarmCreateResult {
    pub swarm: Swarm<()>,
    pub peer_id: PeerId,
}

////////////////////////////////////////////////////////////////////////////////

/// Общая часть по созданию swarm для клиента или сервера
pub fn create_swarm() -> Result<SwarmCreateResult, P2PError> {
    // Генерируем пару открытых и закрытых ключей с помощью OpenSSL алгоритмом ed25519
    let local_key = identity::Keypair::generate_ed25519();

    // Создаем теперь на основании этой самой пары ключей непосредственно идентификатор текущего пира
    let local_peer_id = {
        // Создавать идентификатор пира будем с
        // помощью публичного ключа из пары ключей
        let public_key = local_key.public();

        // Создаем идентификатор пира
        PeerId::from_public_key(&public_key)
    };

    // Сразу же выведем для мониторинга идентификатор нашего пира
    // println!("Local peer id: {:?}", local_peer_id);

    // Создаём транспорт
    let transport = create_transport(&local_key);

    // Настраиваем протокол обмена файлами
    let request_response_behaviour = {
        let codec = FileCodec::new();

        // TODO: Можно ли как-то получать имя из типа сразу же?
        let protocol_name = codec.get_protocol_name();

        let protocols = std::iter::once((protocol_name, ProtocolSupport::Full));

        let cfg = RequestResponseConfig::default();

        libp2p::request_response::Behaviour::with_codec(codec, protocols, cfg)
    };

    // Создаём Swarm
    let mut swarm = Swarm::new(transport, request_response_behaviour, local_peer_id);
}

// ////////////////////////////////////////////////////////////////////////////////

// use crate::{error::P2PError, transport::create_transport};
// use async_trait::async_trait;
// use libp2p::{
//     identity, request_response::Config as RequestResponseConfig, swarm::handler::ProtocolSupport,
//     PeerId, Swarm, Transport,
// };
// use serde::{Deserialize, Serialize};
// use std::path::Path;

// ////////////////////////////////////////////////////////////////////////////////

// /// Начало работы приложения
// fn main() {
//     // Генерируем пару открытых и закрытых ключей с помощью OpenSSL алгоритмом ed25519
//     let local_key = identity::Keypair::generate_ed25519();

//     // Создаем теперь на основании этой самой пары ключей непосредственно идентификатор текущего пира
//     let local_peer_id = {
//         // Создавать идентификатор пира будем с
//         // помощью публичного ключа из пары ключей
//         let public_key = local_key.public();

//         // Создаем идентификатор пира
//         PeerId::from_public_key(&public_key)
//     };

//     // Сразу же выведем для мониторинга идентификатор нашего пира
//     println!("Local peer id: {:?}", local_peer_id);

//     // Создаём транспорт
//     let transport = create_transport(&local_key);

//     // Настраиваем протокол обмена файлами
//     let request_response_behaviour = {
//         let protocols = std::iter::once((FileProtocolName, ProtocolSupport::Full));
//         let codec = FileCodec();
//         let cfg = RequestResponseConfig::default();
//         libp2p::request_response::Behaviour::with_codec(codec, protocols, cfg)
//     };

//     // Создаём Swarm
//     let mut swarm = Swarm::new(transport, request_response_behaviour, local_peer_id);

//     // Слушаем на всех доступных интерфейсах и портах
//     swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
//     println!("Swarm запущен. Ожидаем входящего подключения...");

//     // Подключаемся к другому узлу
//     // let remote: Multiaddr = "/ip4/127.0.0.1/tcp/8080/p2p/12D3KooW...".parse()?;
//     // swarm.dial(remote)?;
//     // println!("Подключение к {}", remote)?;

//     // Создаем теперь уже асинхронный рантайм
//     // tokio::runtime::Builder::new_multi_thread()
//     //     .enable_io()
//     //     .build()
//     //     .expect("Tokio runtime build failed")
//     //     .block_on(async_main(swarm));
// }

// ////////////////////////////////////////////////////////////////////////////////

// /// Непосредственно наша исполняема асинхронная часть
// async fn async_loop(swarm: Swarm<_>) -> Result<(), P2PError> {
//     // Основной цикл
//     loop {
//         match swarm.next().await.unwrap() {
//             RequestResponseEvent::Message { peer, message } => {
//                 match message {
//                     RequestResponseMessage::Request {
//                         request, channel, ..
//                     } => {
//                         println!("Получен запрос на файл: {}", request.filename);
//                         // Здесь должна быть логика поиска и чтения файла
//                         // Для примера отправим заглушку
//                         let response = match std::fs::read(&request.filename) {
//                             Ok(content) => FileResponse::Found { content },
//                             Err(_) => FileResponse::NotFound,
//                         };

//                         swarm.respond(channel, response)?;
//                     }
//                     RequestResponseMessage::Response { response, .. } => {
//                         println!(
//                             "Получен ответ с содержимым файла: {} байт",
//                             response.content.len()
//                         );
//                         // Здесь можно обработать полученные данные
//                     }
//                 }
//             }
//             RequestResponseEvent::OutboundFailure { peer, error, .. } => {
//                 eprintln!("Ошибка при отправке запроса к {}: {:?}", peer, error);
//             }
//             RequestResponseEvent::InboundFailure { peer, error, .. } => {
//                 eprintln!("Ошибка при получении запроса от {}: {:?}", peer, error);
//             }
//             RequestResponseEvent::ResponseSent { peer, .. } => {
//                 println!("Ответ отправлен к {}", peer);
//             }
//             _ => {}
//         }
//     }
// }

// ////////////////////////////////////////////////////////////////////////////////

// #[derive(Debug, Clone)]
// struct FileProtocol;
