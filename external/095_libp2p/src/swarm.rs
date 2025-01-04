use crate::{
    codec::{CodecProtocolName, FileCodec},
    error::P2PError,
    transport::create_transport,
};
use libp2p::{
    identity,
    request_response::{Behaviour, Config as RequestResponseConfig, ProtocolSupport},
    swarm::Config as SwarmConfig,
    PeerId, Swarm,
};

////////////////////////////////////////////////////////////////////////////////

/// Публичный удобный тип
pub type SwarmP2PType = Swarm<Behaviour<FileCodec>>;

////////////////////////////////////////////////////////////////////////////////

/// Результат создания Swarm
pub struct SwarmCreateResult {
    /// Сам Swarm
    pub swarm: SwarmP2PType,

    /// С каким идентификатором пира было создано
    pub peer_id: PeerId,
}

////////////////////////////////////////////////////////////////////////////////

/// Общая часть по созданию swarm для клиента или сервера
pub fn create_swarm() -> Result<SwarmCreateResult, P2PError> {
    // Генерируем пару открытых и закрытых ключей с помощью OpenSSL алгоритмом ed25519
    let local_key = identity::Keypair::generate_ed25519();

    // Создаем теперь на основании этой самой пары ключей непосредственно идентификатор текущего пира
    let peer_id = {
        // Создавать идентификатор пира будем с
        // помощью публичного ключа из пары ключей
        let public_key = local_key.public();

        // Создаем идентификатор пира
        PeerId::from_public_key(&public_key)
    };

    // Сразу же выведем для мониторинга идентификатор нашего пира
    // println!("Local peer id: {:?}", local_peer_id);

    // Создаём транспорт
    let transport = create_transport(&local_key)?;

    // Настраиваем протокол обмена файлами
    let request_response_behaviour = {
        // Создаем кодек для протокола нашего
        let codec = FileCodec::new();

        // Пробуем получить сразу же имя протокола
        let protocol = codec.get_protocol();

        // TODO: Список протоколов, правда идея не до конца понятна
        let protocols = std::iter::once((protocol, ProtocolSupport::Full));

        // Отдельный конфиг для запросов и ответов
        let cfg = RequestResponseConfig::default();

        // Теперь собираем все в кучу
        Behaviour::with_codec(codec, protocols, cfg)
    };

    // Будем использовать Tokio для исполнения
    let config = SwarmConfig::with_tokio_executor();

    // Создаём Swarm
    let swarm = Swarm::new(transport, request_response_behaviour, peer_id, config);

    Ok(SwarmCreateResult { swarm, peer_id })
}
