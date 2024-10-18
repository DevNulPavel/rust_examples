use serde::{
    Deserialize, 
    Serialize
};
use warp::{
    Filter
};
use crate::{
    connected_clients::{
        Connections
    },
    ClientId
};
use super::{
    SocketAddr
};

///////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostQueryResponse {
    pub client_id: Option<ClientId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostQuery {
    pub host: String
}

// Возвращает идентификатор клиента
fn handle_query(query: HostQuery) -> HostQueryResponse {
    tracing::debug!(host=%query.host, "got query");
    HostQueryResponse {
        client_id: Connections::client_for_host(&query.host),
    }
}

///////////////////////////////////////////////////////////////

pub fn spawn<A: Into<SocketAddr>>(addr: A) {
    // Маршрут проверки доступности сервера
    let health_check = warp::get()
        .and(warp::path("health_check")).map(|| {
            tracing::debug!("Net svc health check triggered");
            "ok"
        });

    // Коренной маршрут обработчика запроса
    let query_svc = warp::path::end()
        .and(warp::get())
        .and(warp::query::<HostQuery>())
        .map(|query| {
            warp::reply::json(&handle_query(query))
        });

    let routes = query_svc
        .or(health_check);

    // Запускаем наш сервер общения между серверами
    tokio::spawn(warp::serve(routes).run(addr.into()));
}