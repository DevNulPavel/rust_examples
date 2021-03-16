use hyper::{
    version::{
        HttpVersion
    }
};

/// Данный трейт используется для валидации, что данная HTTP версия соответствует нужной
pub trait ValidateHttpVersion {
    /// Подтверждаем, что текущая версия HTTP как минимум больше или равна 1.1,
    /// Это нужно для поддержки загрузки чанков
    fn greater_than_http_11(&self) -> bool;
}

/// Реализация для HttpVersion из Hyper
impl ValidateHttpVersion for HttpVersion {
    /// Проверяем данную версию HTTP
    /// Данная версия должна быть как минимум 1.1
    fn greater_than_http_11(&self) -> bool {
        self >= &HttpVersion::Http11
    }
}
