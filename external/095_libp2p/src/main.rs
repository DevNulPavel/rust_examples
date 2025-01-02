use async_trait::async_trait;
use libp2p::{
    core::upgrade,
    identity, mplex, noise,
    request_response::{
        ProtocolName, ProtocolSupport, RequestResponse, RequestResponseCodec,
        RequestResponseConfig, RequestResponseEvent, RequestResponseMessage,
        RequestResponseProtocol,
    },
    tcp::TokioTcpConfig,
    Multiaddr, PeerId, Swarm, Transport,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, thiserror::Error)]
enum P2PError {}

////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<(), P2PError> {
    // Генерируем ключи и PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Создаём транспорт
    let transport = create_transport(&local_key);

    // Настраиваем протокол обмена файлами
    let protocols = std::iter::once((FileProtocolName, ProtocolSupport::Full));
    let codec = FileCodec();
    let cfg = RequestResponseConfig::default();
    let request_response = RequestResponse::new(codec, protocols, cfg);

    // Создаём Swarm
    let mut swarm = Swarm::new(transport, request_response, local_peer_id);

    // Слушаем на всех доступных интерфейсах и портах
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    println!("Swarm запущен. Ожидаем входящего подключения...");

    // Подключаемся к другому узлу
    // let remote: Multiaddr = "/ip4/127.0.0.1/tcp/8080/p2p/12D3KooW...".parse()?;
    // swarm.dial(remote)?;
    // println!("Подключение к {}", remote)?;

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

////////////////////////////////////////////////////////////////////////////////

fn create_transport(
    local_key: &identity::Keypair,
) -> impl Transport<Output = impl libp2p::swarm::ConnectionHandler, Error = impl std::error::Error> + Clone
{
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(local_key)
        .expect("Signing libp2p-noise static DH keypair failed.");

    TokioTcpConfig::new()
        .nodelay(true)
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed()
}
