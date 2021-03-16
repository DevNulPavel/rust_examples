use std::{
    ops::{
        Deref
    }
};
use hyper::{
    header::{
        ContentLength, 
        Headers
    }
};

use BytesLength;

/// Трейт для расширения функциональности типа `Headers` из `hyper`
pub trait GetContentLength {
    /// Функция для получения длины контента удаленного документа
    /// Возвращаемый тип - `Option<Bytes>`
    fn get_content_length(&self) -> Option<BytesLength>;
}

/// Расширяем хедеры реализацией трейта
impl GetContentLength for Headers {
    /// Функция для получения `content-length` контейнера из данного хедера
    /// Данная функция возвращает `Option`, который содержит размер
    fn get_content_length(&self) -> Option<BytesLength> {
        if self.has::<ContentLength>() {
            return Some(*self.get::<ContentLength>().unwrap().deref());
        }
        None
    }
}
