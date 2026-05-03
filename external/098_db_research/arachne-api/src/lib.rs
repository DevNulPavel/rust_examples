use arachne_parser::evaluator::UserMeta;
use bytes::{Bytes, BytesMut};
use std::pin::Pin;
use tarpc::tokio_serde::{Deserializer, Serializer};

////////////////////////////////////////////////////////////////////////////////

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum PlatformCreateStatus {
    /// Платформа создана
    Created,
    /// Платформа уже существует
    Exists,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum SelectorSize {
    /// Размер очереди
    Size(u64),
    /// Платформы не существует
    PlatformDoesNotExist,
    /// Селектора не существует
    SelectorDoesNotExist,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum PlatformDeleteStatus {
    /// Успешно удалено
    Deleted,
    /// Платформы не существует
    NotFound,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum SelectorCreateStatus {
    /// Селектор создан успешно
    Created,
    /// Уже существует и запрос совпадает
    Exists,
    /// Селектор изменен (новый запрос)
    Modified,
    /// Платформы не существует
    PlatformDoesNotExist,
    /// Ошибка парсинга запроса
    ParseError(String),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum SelectorDeleteStatus {
    /// Селектор удален успешно
    Deleted,
    /// Селектор не найден
    NotFound,
    /// Платформы не существует
    PlatformDoesNotExist,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum UserAddStatus {
    /// Платформы не существует
    PlatformDoesNotExist,
    /// Юзер успешно добавлен
    Success,
    /// Юзер уже существует
    Exists,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum UserReleaseStatus {
    /// Платформы не существует
    PlatformDoesNotExist,
    /// Юзер успешно освобожден
    Released,
    /// Юзер не существует
    NotFound,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum UserUpdateStatus {
    /// Платформы не существует
    PlatformDoesNotExist,
    /// Юзер успешно обновлен
    Success,
    /// Юзер не существует
    NotFound,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum UserDeleteStatus {
    /// Юзер успешно удален
    Success,
    /// Юзер не существует
    NotFound,
    /// Платформы не существует
    PlatformDoesNotExist,
}

////////////////////////////////////////////////////////////////////////////////

// TODO: Целых 24 байта на идентификатор пользователя без возможности настройки
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum UserId {
    /// Числовой id
    Int(i64),
    /// Строковый id
    Str(smol_str::SmolStr),
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserId::Int(i) => write!(f, "{}", i),
            UserId::Str(s) => write!(f, "{}", s),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct UserPayload {
    #[serde(rename = "i")]
    pub id: UserId,

    // TODO: ОГРОМНАЯ проблема с фрагментацией оперативной памяти из-за
    // тонны мелких аллокаций для каждого пользователя
    #[serde(rename = "p")]
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,

    // TODO: Тут вообще у нас динамическое представление данных о пользователе в виде json::Value
    #[serde(rename = "m")]
    pub meta: UserMeta,
}

////////////////////////////////////////////////////////////////////////////////

pub mod rpc_dto {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    pub enum RpcUserId {
        Int(i64),
        Str(smol_str::SmolStr),
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    pub struct RpcUserPayload {
        pub id: RpcUserId,

        #[serde(with = "serde_bytes")]
        pub payload: Vec<u8>,

        #[serde(with = "json_bytes_adapter")]
        pub meta: UserMeta,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
    pub enum RpcSelectUserResponse {
        Found(RpcUserPayload),
        Finished,
        Empty,
        PlatformDoesNotExist,
        SelectorDoesNotExist,
    }

    mod json_bytes_adapter {
        use arachne_parser::evaluator::UserMeta;
        use serde::{Deserialize, Deserializer, Serializer};

        pub fn serialize<S>(meta: &UserMeta, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let bytes = serde_json::to_vec(meta).map_err(serde::ser::Error::custom)?;
            serializer.serialize_bytes(&bytes)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<UserMeta, D::Error>
        where
            D: Deserializer<'de>,
        {
            let bytes = <Vec<u8>>::deserialize(deserializer)?;
            serde_json::from_slice(&bytes).map_err(serde::de::Error::custom)
        }
    }
}

impl From<UserId> for rpc_dto::RpcUserId {
    fn from(id: UserId) -> Self {
        match id {
            UserId::Int(i) => Self::Int(i),
            UserId::Str(s) => Self::Str(s),
        }
    }
}

impl From<rpc_dto::RpcUserId> for UserId {
    fn from(id: rpc_dto::RpcUserId) -> Self {
        match id {
            rpc_dto::RpcUserId::Int(i) => Self::Int(i),
            rpc_dto::RpcUserId::Str(s) => Self::Str(s),
        }
    }
}

impl From<UserPayload> for rpc_dto::RpcUserPayload {
    fn from(p: UserPayload) -> Self {
        Self {
            id: p.id.into(),
            payload: p.payload,
            meta: p.meta,
        }
    }
}

impl From<rpc_dto::RpcUserPayload> for UserPayload {
    fn from(p: rpc_dto::RpcUserPayload) -> Self {
        Self {
            id: p.id.into(),
            payload: p.payload,
            meta: p.meta,
        }
    }
}

#[tarpc::service]
pub trait Rpc {
    async fn platform_create(platform_id: u8) -> Result<PlatformCreateStatus, String>;
    async fn platform_delete(platform_id: u8) -> Result<PlatformDeleteStatus, String>;
    async fn platforms_list() -> Vec<u8>;

    async fn selector_create(
        platform_id: u8,
        selector_id: u8,
        request: String,
        lock_duration_sec: u64,
    ) -> SelectorCreateStatus;
    async fn selector_delete(
        platform_id: u8,
        selector_id: u8,
    ) -> Result<SelectorDeleteStatus, String>;
    async fn selector_queue_size(platform_id: u8, selector_id: u8) -> SelectorSize;
    async fn selector_pending_size(platform_id: u8, selector_id: u8) -> SelectorSize;
    async fn selector_locked_size(platform_id: u8, selector_id: u8) -> SelectorSize;

    async fn select_user(
        platform_id: u8,
        selector_id: u8,
    ) -> Result<rpc_dto::RpcSelectUserResponse, String>;
    async fn user_add(
        platform_id: u8,
        payload: rpc_dto::RpcUserPayload,
    ) -> Result<UserAddStatus, String>;
    async fn users_add_batch(
        platform_id: u8,
        payloads: Vec<rpc_dto::RpcUserPayload>,
    ) -> Result<UserAddStatus, String>;
    async fn user_update(
        platform_id: u8,
        payload: rpc_dto::RpcUserPayload,
    ) -> Result<UserUpdateStatus, String>;
    async fn user_delete(
        platform_id: u8,
        user_id: rpc_dto::RpcUserId,
    ) -> Result<UserDeleteStatus, String>;
    async fn user_release(
        platform_id: u8,
        user_id: rpc_dto::RpcUserId,
    ) -> Result<UserReleaseStatus, String>;
    async fn user_search(
        platform_id: u8,
        user_id: rpc_dto::RpcUserId,
    ) -> Result<Option<rpc_dto::RpcUserPayload>, String>;

    async fn total_users_count() -> Result<u64, String>;
    async fn total_platformed_users_count(platform_id: u8) -> Result<u64, String>;
}

#[derive(Clone, Default)]
pub struct BitcodeCodec;

impl<T> Serializer<T> for BitcodeCodec
where
    T: serde::Serialize,
{
    type Error = std::io::Error;

    fn serialize(self: Pin<&mut Self>, item: &T) -> Result<Bytes, Self::Error> {
        let encoded = bitcode::serialize(item)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Bytes::from(encoded))
    }
}

impl<T> Deserializer<T> for BitcodeCodec
where
    T: serde::de::DeserializeOwned,
{
    type Error = std::io::Error;

    fn deserialize(self: Pin<&mut Self>, src: &BytesMut) -> Result<T, Self::Error> {
        bitcode::deserialize(src)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
