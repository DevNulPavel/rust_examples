use async_trait::async_trait;
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::request_response::Codec;
use serde::{Deserialize, Serialize};
use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind},
    path::PathBuf,
};

////////////////////////////////////////////////////////////////////////////////

/// Отдельная структура, которая будет выдывать имя конкретного протокола
#[derive(Debug, Clone, Default)]
pub(super) struct FileProtocolName;

/// Возвращаем имя этого самого протокола
impl AsRef<str> for FileProtocolName {
    fn as_ref(&self) -> &str {
        "/p2pfile/0.1.0"
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Какой-то запрос к нашему сервису
#[derive(Debug, Serialize, Deserialize)]
struct FileRequest {
    /// По какому пути читаем файлик
    file_path: PathBuf,
}

/// Какой-то ответ нашего сервиса
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum FileResponse {
    /// Файлик найден
    #[serde(rename = "found")]
    Found { content: Box<[u8]> },

    /// Такого файла нету или нельзя прочитать
    #[serde(rename = "not_available")]
    NotAvailable,
}

////////////////////////////////////////////////////////////////////////////////

/// Отдельный трейт для возможности получения имени протокола для кодеков
pub(super) trait CodecProtocolName: Codec {
    fn get_protocol_name(&self) -> &str;
}

////////////////////////////////////////////////////////////////////////////////

// TODO: Стратечия уменьшения размера буфера

#[derive(Clone)]
pub(super) struct FileCodec {
    buffer: Vec<u8>,
    protocol_obj: <Self as Codec>::Protocol,
}

impl FileCodec {
    pub(super) fn new() -> FileCodec {
        FileCodec {
            buffer: Vec::with_capacity(128),
            protocol_obj: <Self as Codec>::Protocol::default(),
        }
    }
}

impl CodecProtocolName for FileCodec {
    fn get_protocol_name(&self) -> &str {
        self.protocol_obj.as_ref()

        // let protocol_name = <FileCodec as Codec>::Protocol::default().as_ref();

        // TODO: Какая-то фигня
        // <Self as Codec>::Protocol::default().as_ref()
    }
}

#[async_trait]
impl Codec for FileCodec {
    /// Отдельный тип, который четко идентифицирует наш протокол и его версию
    type Protocol = FileProtocolName;

    /// Тип запроса
    type Request = FileRequest;

    /// Тип ответа
    type Response = FileResponse;

    /// Чтение запроса из потока
    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> Result<Self::Request, IoError>
    where
        T: AsyncRead + Unpin + Send,
    {
        // TODO: Надо ли сравнивать протокол?

        // Очистим буфер в плане размера, но не емкости
        self.buffer.clear();

        // TODO: Защита от переполнения входящего потока
        // Вычитываем входящие данные полностью в этот буфер
        io.read_to_end(&mut self.buffer).await?;

        // Сконвертируем данные в запрос
        let req: Self::Request = bincode::deserialize(self.buffer.as_slice())
            .map_err(|e| IoError::new(IoErrorKind::InvalidData, e))?;

        Ok(req)
    }

    /// Обработка какого-то ответа
    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> Result<Self::Response, IoError>
    where
        T: AsyncRead + Unpin + Send,
    {
        // TODO: Надо ли сравнивать протокол?

        // Очистим буфер в плане размера, но не емкости
        self.buffer.clear();

        // TODO: Защита от переполнения входящего потока
        // Вычитываем входящие данные полностью в этот буфер
        io.read_to_end(&mut self.buffer).await?;

        // Сконвертируем данные в запрос
        let req: Self::Response = bincode::deserialize(self.buffer.as_slice())
            .map_err(|e| IoError::new(IoErrorKind::InvalidData, e))?;

        Ok(req)
    }

    /// Пишем непосредственно запрос
    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> Result<(), IoError>
    where
        T: AsyncWrite + Unpin + Send,
    {
        // TODO: Надо ли сравнивать протокол?

        // Очистим буфер в плане размера, но не емкости
        self.buffer.clear();

        // Сериализуем теперь запрос
        bincode::serialize_into(&mut self.buffer, &req)
            .map_err(|e| IoError::new(IoErrorKind::InvalidData, e))?;

        // Пишем результат
        io.write_all(&self.buffer).await?;

        Ok(())
    }

    /// Пишем теперь ответ
    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> Result<(), IoError>
    where
        T: AsyncWrite + Unpin + Send,
    {
        // TODO: Надо ли сравнивать протокол?

        // Очистим буфер в плане размера, но не емкости
        self.buffer.clear();

        // Сериализуем теперь запрос
        bincode::serialize_into(&mut self.buffer, &res)
            .map_err(|e| IoError::new(IoErrorKind::InvalidData, e))?;

        // Пишем результат
        io.write_all(&self.buffer).await?;

        Ok(())
    }
}
