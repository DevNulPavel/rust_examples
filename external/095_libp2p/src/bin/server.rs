////////////////////////////////////////////////////////////////////////////////

use async_trait::async_trait;
use libp2p::{
    identity, request_response::Config as RequestResponseConfig, swarm::handler::ProtocolSupport,
    PeerId, Swarm, Transport,
};
use libp2p_research::{error::P2PError, transport::create_transport};
use serde::{Deserialize, Serialize};
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////

/// Начало работы приложения
fn main() {


    // Слушаем на всех доступных интерфейсах и портах
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    println!("Swarm запущен. Ожидаем входящего подключения...");

    // Подключаемся к другому узлу
    // let remote: Multiaddr = "/ip4/127.0.0.1/tcp/8080/p2p/12D3KooW...".parse()?;
    // swarm.dial(remote)?;
    // println!("Подключение к {}", remote)?;

    // Создаем теперь уже асинхронный рантайм
    // tokio::runtime::Builder::new_multi_thread()
    //     .enable_io()
    //     .build()
    //     .expect("Tokio runtime build failed")
    //     .block_on(async_main(swarm));
}

////////////////////////////////////////////////////////////////////////////////

/// Непосредственно наша исполняема асинхронная часть
async fn async_loop(swarm: Swarm<_>) -> Result<(), P2PError> {
    // Основной цикл
    loop {
        match swarm.next().await.unwrap() {
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

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
struct FileRequest<'a> {
    file_path: &'a Path,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum FileResponse<'a> {
    #[serde(rename = "found")]
    Found { content: &'a [u8] },

    #[serde(rename = "not_found")]
    NotFound,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileProtocolName;

impl ProtocolName for FileProtocolName {
    fn protocol_name(&self) -> &[u8] {
        b"/p2pfile/1.0.0"
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
struct FileProtocol;

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct FileCodec;

#[async_trait]
impl RequestResponseCodec for FileCodec {
    type Protocol = FileProtocolName;
    type Request = FileRequest;
    type Response = FileResponse;

    async fn read_request<T>(
        &mut self,
        _: &FileProtocolName,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    where
        T: async_std::io::Read + Unpin,
    {
        let req: FileRequest = serde_json::from_reader(io)
            .map_err(|e| async_std::io::Error::new(async_std::io::ErrorKind::InvalidData, e))?;
        Ok(req)
    }

    async fn read_response<T>(
        &mut self,
        _: &FileProtocolName,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: async_std::io::Read + Unpin,
    {
        let res: FileResponse = serde_json::from_reader(io)
            .map_err(|e| async_std::io::Error::new(async_std::io::ErrorKind::InvalidData, e))?;
        Ok(res)
    }

    async fn write_request<T>(
        &mut self,
        _: &FileProtocolName,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: async_std::io::Write + Unpin,
    {
        serde_json::to_writer(io, &req)
            .map_err(|e| async_std::io::Error::new(async_std::io::ErrorKind::InvalidData, e))
    }

    async fn write_response<T>(
        &mut self,
        _: &FileProtocolName,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: async_std::io::Write + Unpin,
    {
        serde_json::to_writer(io, &res)
            .map_err(|e| async_std::io::Error::new(async_std::io::ErrorKind::InvalidData, e))
    }
}
