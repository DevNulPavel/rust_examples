use arachne_api::{BitcodeCodec, RpcClient, UserPayload, rpc_dto};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Display;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::net::ToSocketAddrs;

use tarpc::client;
use tarpc::context;

pub use arachne_api::{
    PlatformCreateStatus, PlatformDeleteStatus, SelectorCreateStatus, SelectorDeleteStatus,
    SelectorSize, UserAddStatus, UserDeleteStatus, UserId, UserReleaseStatus, UserUpdateStatus,
};
pub use smol_str::*;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum SelectUser<T> {
    Found(User<T>),
    Empty,
    Finished,
    PlatformDoesNotExist,
    SelectorDoesNotExist,
}

impl<T: DeserializeOwned> TryInto<SelectUser<T>> for rpc_dto::RpcSelectUserResponse {
    type Error = serde_json::Error;

    fn try_into(self) -> Result<SelectUser<T>, Self::Error> {
        match self {
            rpc_dto::RpcSelectUserResponse::Found(v) => Ok(SelectUser::Found(User {
                id: v.id.into(),
                payload: v.payload,
                meta: serde_json::from_value(v.meta)?,
            })),
            rpc_dto::RpcSelectUserResponse::Finished => Ok(SelectUser::Finished),
            rpc_dto::RpcSelectUserResponse::Empty => Ok(SelectUser::Empty),
            rpc_dto::RpcSelectUserResponse::PlatformDoesNotExist => {
                Ok(SelectUser::PlatformDoesNotExist)
            }
            rpc_dto::RpcSelectUserResponse::SelectorDoesNotExist => {
                Ok(SelectUser::SelectorDoesNotExist)
            }
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct User<T> {
    pub id: UserId,
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
    pub meta: T,
}

// Удобная обработка ошибок (соединяет сетевые ошибки tarpc и бизнес-логику базы)
#[derive(Debug, thiserror::Error)]
pub enum ArachneError {
    #[error("Rpc network error: {0}")]
    Rpc(#[from] tarpc::client::RpcError),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Server error: {0}")]
    Server(String),
    #[error("Connection error: {0}")]
    Connection(std::io::Error),
}

// Хелперы для распаковки результатов
fn handle_res<T>(
    res: Result<Result<T, String>, tarpc::client::RpcError>,
) -> Result<T, ArachneError> {
    match res {
        Ok(Ok(val)) => Ok(val),
        Ok(Err(e)) => Err(ArachneError::Server(e)),
        Err(e) => Err(ArachneError::Rpc(e)),
    }
}

fn handle_rpc<T>(res: Result<T, tarpc::client::RpcError>) -> Result<T, ArachneError> {
    res.map_err(ArachneError::Rpc)
}

#[derive(Clone)]
/// Клиент арахне
pub struct ArachneClient {
    clients: Arc<Vec<RpcClient>>,
    next_client_idx: Arc<AtomicUsize>,
}

impl ArachneClient {
    /// Значение пула лучше указывать от 1 до 4, больше особого смысла нет
    ///
    /// Чем выше значение, тем чуть хуже скорость вставки, но выше select
    pub async fn new<A: ToSocketAddrs + Display + Clone + Send + Sync + 'static>(
        addr: A,
        pool_size: usize,
    ) -> Result<Self, ArachneError> {
        let mut clients = Vec::with_capacity(pool_size);

        for _ in 0..pool_size {
            // Подключаемся к серверу
            let mut transport =
                tarpc::serde_transport::tcp::connect(addr.clone(), BitcodeCodec::default);
            transport.config_mut().max_frame_length(usize::MAX);
            let transport = transport.await.map_err(ArachneError::Connection)?;
            let _ = transport.get_ref().set_nodelay(true);
            let mut config = client::Config::default();
            config.max_in_flight_requests = 10_000;
            config.pending_request_buffer = 10_000;
            // Создаем и запускаем tarpc клиента в фоне
            let client = RpcClient::new(config, transport).spawn();
            clients.push(client);
        }

        Ok(Self {
            clients: Arc::new(clients),
            next_client_idx: Arc::new(AtomicUsize::new(0)),
        })
    }

    #[inline]
    fn client(&self) -> &RpcClient {
        let idx = self.next_client_idx.fetch_add(1, Ordering::Relaxed) % self.clients.len();
        &self.clients[idx]
    }

    /// Создание платформы
    pub async fn platform_create(
        &self,
        platform_id: u8,
    ) -> Result<PlatformCreateStatus, ArachneError> {
        handle_res(
            self.client()
                .platform_create(context::current(), platform_id)
                .await,
        )
    }

    /// Удаление платформы
    pub async fn platform_delete(
        &self,
        platform_id: u8,
    ) -> Result<PlatformDeleteStatus, ArachneError> {
        handle_res(
            self.client()
                .platform_delete(context::current(), platform_id)
                .await,
        )
    }

    /// Список платформ (их id)
    pub async fn platforms_list(&self) -> Result<Vec<u8>, ArachneError> {
        // Требует handle_rpc, так как этот метод в трейте не возвращает кастомную String-ошибку
        handle_rpc(self.client().platforms_list(context::current()).await)
    }

    /// Создание селектора
    ///
    /// Для изменения селектора используется этот же метод
    pub async fn selector_create(
        &self,
        platform_id: u8,
        selector_id: u8,
        request: &str,
        duration_sec: u64,
    ) -> Result<SelectorCreateStatus, ArachneError> {
        handle_rpc(
            self.client()
                .selector_create(
                    context::current(),
                    platform_id,
                    selector_id,
                    request.to_string(),
                    duration_sec,
                )
                .await,
        )
    }

    /// Удаление селектора
    pub async fn selector_delete(
        &self,
        platform_id: u8,
        selector_id: u8,
    ) -> Result<SelectorDeleteStatus, ArachneError> {
        handle_res(
            self.client()
                .selector_delete(context::current(), platform_id, selector_id)
                .await,
        )
    }

    /// Размер очереди селектора
    pub async fn selector_queue_size(
        &self,
        platform_id: u8,
        selector_id: u8,
    ) -> Result<SelectorSize, ArachneError> {
        self.client()
            .selector_queue_size(context::current(), platform_id, selector_id)
            .await
            .map_err(ArachneError::Rpc)
    }

    /// Размер очереди ожидания селектора
    pub async fn selector_pending_size(
        &self,
        platform_id: u8,
        selector_id: u8,
    ) -> Result<SelectorSize, ArachneError> {
        self.client()
            .selector_pending_size(context::current(), platform_id, selector_id)
            .await
            .map_err(ArachneError::Rpc)
    }

    /// Размер очереди заблокированных пользователей селектора
    pub async fn selector_locked_size(
        &self,
        platform_id: u8,
        selector_id: u8,
    ) -> Result<SelectorSize, ArachneError> {
        self.client()
            .selector_locked_size(context::current(), platform_id, selector_id)
            .await
            .map_err(ArachneError::Rpc)
    }

    /// Селект юзера
    pub async fn select_user<T: DeserializeOwned>(
        &self,
        platform_id: u8,
        selector_id: u8,
    ) -> Result<SelectUser<T>, ArachneError> {
        handle_res(
            self.client()
                .select_user(context::current(), platform_id, selector_id)
                .await,
        )
        .map(|v| v.try_into().map_err(Into::into))?
    }

    /// Добавление юзера
    pub async fn user_add<T: Serialize>(
        &self,
        platform_id: u8,
        payload: User<T>,
    ) -> Result<UserAddStatus, ArachneError> {
        handle_res(
            self.client()
                .user_add(
                    context::current(),
                    platform_id,
                    UserPayload {
                        id: payload.id,
                        payload: payload.payload,
                        meta: serde_json::to_value(payload.meta).unwrap(),
                    }
                    .into(),
                )
                .await,
        )
    }

    /// Добавление юзеров батчем
    pub async fn user_add_batch<T: Serialize>(
        &self,
        platform_id: u8,
        payloads: Vec<User<T>>,
    ) -> Result<UserAddStatus, ArachneError> {
        handle_res(
            self.client()
                .users_add_batch(
                    context::current(),
                    platform_id,
                    payloads
                        .into_iter()
                        .map(|payload| {
                            UserPayload {
                                id: payload.id,
                                payload: payload.payload,
                                meta: serde_json::to_value(payload.meta).unwrap(),
                            }
                            .into()
                        })
                        .collect(),
                )
                .await,
        )
    }

    /// Апдейт юзера
    pub async fn user_update<T: Serialize>(
        &self,
        platform_id: u8,
        payload: User<T>,
    ) -> Result<UserUpdateStatus, ArachneError> {
        handle_res(
            self.client()
                .user_update(
                    context::current(),
                    platform_id,
                    UserPayload {
                        id: payload.id,
                        payload: payload.payload,
                        meta: serde_json::to_value(payload.meta).unwrap(),
                    }
                    .into(),
                )
                .await,
        )
    }
    /// Удаление юзера
    pub async fn user_delete(
        &self,
        platform_id: u8,
        user_id: UserId,
    ) -> Result<UserDeleteStatus, ArachneError> {
        handle_res(
            self.client()
                .user_delete(context::current(), platform_id, user_id.into())
                .await,
        )
    }

    /// Освобождение блокировки юзера
    pub async fn user_release(
        &self,
        platform_id: u8,
        user_id: UserId,
    ) -> Result<UserReleaseStatus, ArachneError> {
        handle_res(
            self.client()
                .user_release(context::current(), platform_id, user_id.into())
                .await,
        )
    }

    /// Поиск юзера по id (без блокировки)
    pub async fn user_search<T: DeserializeOwned>(
        &self,
        platform_id: u8,
        user_id: UserId,
    ) -> Result<Option<User<T>>, ArachneError> {
        handle_res(
            self.client()
                .user_search(context::current(), platform_id, user_id.into())
                .await,
        )
        .map(|v| {
            v.map(|v| {
                Ok(User {
                    id: v.id.into(),
                    payload: v.payload,
                    meta: serde_json::from_value(v.meta)?,
                })
            })
            .transpose()
        })?
    }

    /// Общее кол-во юзеров на всех платформах
    pub async fn total_users_count(&self) -> Result<u64, ArachneError> {
        handle_res(self.client().total_users_count(context::current()).await)
    }

    /// Кол-во юзеров на платформе
    ///
    /// ВАЖНО: Ограничение юзера на платформу - 4 миллиарда
    pub async fn total_platformed_users_count(&self, platform_id: u8) -> Result<u64, ArachneError> {
        handle_res(
            self.client()
                .total_platformed_users_count(context::current(), platform_id)
                .await,
        )
    }
}
