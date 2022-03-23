use crate::{
    addr_type::AddrType,
    error::{MerinoError, ResponseCode},
    sock_command::SockCommand,
    SOCKS_VERSION,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Сообщение от пользователя
#[allow(dead_code)]
pub struct SOCKSReq {
    // Версия
    pub version: u8,
    // Непосредственно команда
    pub command: SockCommand,
    // Тип адреса
    pub addr_type: AddrType,
    // Непосредственно целевой адрес
    pub addr: Vec<u8>,
    // Целевой порт
    pub port: u16,
}

impl SOCKSReq {
    /// Парсим данные из Tcp потока
    pub async fn from_stream<T>(stream: &mut T) -> Result<Self, MerinoError>
    where
        T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        // From rfc 1928 (S4), the SOCKS request is formed as follows:
        //
        //    +----+-----+-------+------+----------+----------+
        //    |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
        //    +----+-----+-------+------+----------+----------+
        //    | 1  |  1  | X'00' |  1   | Variable |    2     |
        //    +----+-----+-------+------+----------+----------+
        //
        // Where:
        //
        //      o  VER    protocol version: X'05'
        //      o  CMD
        //         o  CONNECT X'01'
        //         o  BIND X'02'
        //         o  UDP ASSOCIATE X'03'
        //      o  RSV    RESERVED
        //      o  ATYP   address type of following address
        //         o  IP V4 address: X'01'
        //         o  DOMAINNAME: X'03'
        //         o  IP V6 address: X'04'
        //      o  DST.ADDR       desired destination address
        //      o  DST.PORT desired destination port in network octet
        //         order
        trace!("Server waiting for connect");

        // Буффер для пакета
        let mut packet = [0u8; 4];

        // Читаем байты из потока и определяем версию прокси
        stream.read_exact(&mut packet).await?;
        trace!("Server received {:?}", packet);

        // Если версия прокси совершенно не та, тогда закрываем сокет
        if packet[0] != SOCKS_VERSION {
            warn!("from_stream Unsupported version: SOCKS{}", packet[0]);
            stream.shutdown().await?;
        }

        // Получаем комманду
        let command = match SockCommand::from(packet[1] as usize) {
            Some(com) => Ok(com),
            None => {
                warn!("Invalid Command");
                stream.shutdown().await?;
                Err(MerinoError::Socks(ResponseCode::CommandNotSupported))
            }
        }?;

        // Целевой адрес пакета
        let addr_type = match AddrType::from(packet[3] as usize) {
            Some(addr) => Ok(addr),
            None => {
                error!("No Addr");
                stream.shutdown().await?;
                Err(MerinoError::Socks(ResponseCode::AddrTypeNotSupported))
            }
        }?;

        // Получаем адрес из типа и сокета
        trace!("Getting Addr");
        let addr: Vec<u8> = match addr_type {
            AddrType::Domain => {
                // Длина домена
                let mut dlen = [0u8; 1];
                stream.read_exact(&mut dlen).await?;
                // Сам домен
                let mut domain = vec![0u8; dlen[0] as usize];
                stream.read_exact(&mut domain).await?;

                domain
            }
            AddrType::V4 => {
                let mut addr = [0u8; 4];
                stream.read_exact(&mut addr).await?;
                addr.to_vec()
            }
            AddrType::V6 => {
                let mut addr = [0u8; 16];
                stream.read_exact(&mut addr).await?;
                addr.to_vec()
            }
        };

        // Целевой порт
        let mut port = [0u8; 2];
        stream.read_exact(&mut port).await?;

        // Merge two u8s into u16
        // Собираем из двух байтов одно значение u16
        let port = u16::from_be_bytes(port);
        // let port = (u16::from(port[0]) << 8) | u16::from(port[1]);

        // Return parsed request
        Ok(SOCKSReq {
            version: packet[0],
            command,
            addr_type,
            addr,
            port,
        })
    }
}
