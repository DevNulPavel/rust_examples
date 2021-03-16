use hyper::{
    client::{
        response::{
            Response
        }
    },
    status::{
        StatusCode
    }
};

/// Трейт, дающий возможность дополнительно проверять статус в ответе
pub trait CheckResponseStatus {
    /// Функция для проверки, что статуc `PartialContent` содержится в ответе HTTP
    fn check_partialcontent_status(&self) -> bool;
    /// Валидный ли ответ?
    fn is_ok(&self) -> bool;
}

/// Делаем реализацию трейта для ответа из hyper, то есть для сторонней структуры
impl CheckResponseStatus for Response {
    fn check_partialcontent_status(&self) -> bool {
        self.status == StatusCode::PartialContent
    }
    fn is_ok(&self) -> bool {
        self.status.is_success()
    }
}
