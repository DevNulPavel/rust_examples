use std::net::SocketAddr;

// Трейт эндпоинта подключения
pub trait Endpoint: Send + 'static {
    fn from_address(addr: SocketAddr) -> Self;
    fn into_address(&self) -> SocketAddr;
    fn clear_src(&mut self);
}
