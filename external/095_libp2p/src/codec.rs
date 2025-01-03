use async_trait::async_trait;
use futures::{AsyncBufRead, AsyncRead, AsyncReadExt};
use libp2p::request_response::Codec;
use serde::{Deserialize, Serialize};
use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind},
    path::{Path, PathBuf},
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct FileProtocolName;

impl AsRef<str> for FileProtocolName {
    fn as_ref(&self) -> &str {
        "/p2pfile/1.0.0"
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

#[derive(Clone)]
pub struct FileCodec {
    buffer: Vec<u8>,
}

#[async_trait]
impl Codec for FileCodec {
    type Protocol = FileProtocolName;
    type Request = FileRequest;
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
