use async_trait::async_trait;
use futures::{AsyncRead, AsyncReadExt};
use libp2p::request_response::Codec;
use serde::{Deserialize, Serialize};
use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind},
    path::PathBuf,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub(super) struct FileProtocolName;

impl AsRef<str> for FileProtocolName {
    fn as_ref(&self) -> &str {
        "/p2pfile/0.1.0"
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Какой-то запрос к нашему сервису
#[derive(Debug, Serialize, Deserialize)]
struct FileRequest {
    file_path: PathBuf,
}

/// Какой-то ответ нашего сервиса
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum FileResponse {
    #[serde(rename = "found")]
    Found { content: Box<[u8]> },

    #[serde(rename = "not_found")]
    NotFound,
}

////////////////////////////////////////////////////////////////////////////////

pub(super) trait CodecProtocolName: Codec {
    fn get_protocol_name(&self) -> &str;
}

////////////////////////////////////////////////////////////////////////////////

// TODO: Стратечия уменьшения размера буфера

#[derive(Clone)]
pub(super) struct FileCodec {
    buffer: Vec<u8>,
}

impl FileCodec {
    pub(super) fn new() -> FileCodec {
        FileCodec {
            buffer: Vec::with_capacity(128),
        }
    }
}

impl CodecProtocolName for FileCodec {
    fn get_protocol_name(&self) -> &str {
        // let protocol_name = <FileCodec as Codec>::Protocol::default().as_ref();
        // TODO: Какая-то фигня
        <Self as Codec>::Protocol::as_ref(&self)
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

    async fn read_request<T>(
        &mut self,
        protocol: &Self::Protocol,
        io: &mut T,
    ) -> Result<Self::Request, IoError>
    where
        T: AsyncRead + Unpin + Send,
    {
        self.buffer.clear();

        io.read_to_end(&mut self.buffer)?;

        let req: FileRequest = bincode::deserialize(self.buffer.as_slice())
            .map_err(|e| IoError::new(IoErrorKind::InvalidData, e))?;

        Ok(req)
    }

    async fn read_response<T>(
        &mut self,
        protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
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
