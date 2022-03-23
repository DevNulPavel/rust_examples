#![forbid(unsafe_code)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

mod addr_type;
mod error;
mod helpers;
mod sock_command;
mod sock_req;

use crate::{
    addr_type::AddrType,
    error::{MerinoError, ResponseCode},
    helpers::{addr_to_socket, pretty_print_addr},
    sock_command::SockCommand,
    sock_req::SOCKSReq,
};
use snafu::Snafu;
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

/// Version of socks
pub const SOCKS_VERSION: u8 = 0x05;

const RESERVED: u8 = 0x00;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct User {
    pub username: String,
    password: String,
}

pub struct SocksReply {
    // From rfc 1928 (S6),
    // the server evaluates the request, and returns a reply formed as follows:
    //
    //    +----+-----+-------+------+----------+----------+
    //    |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
    //    +----+-----+-------+------+----------+----------+
    //    | 1  |  1  | X'00' |  1   | Variable |    2     |
    //    +----+-----+-------+------+----------+----------+
    //
    // Where:
    //
    //      o  VER    protocol version: X'05'
    //      o  REP    Reply field:
    //         o  X'00' succeeded
    //         o  X'01' general SOCKS server failure
    //         o  X'02' connection not allowed by ruleset
    //         o  X'03' Network unreachable
    //         o  X'04' Host unreachable
    //         o  X'05' Connection refused
    //         o  X'06' TTL expired
    //         o  X'07' Command not supported
    //         o  X'08' Address type not supported
    //         o  X'09' to X'FF' unassigned
    //      o  RSV    RESERVED
    //      o  ATYP   address type of following address
    //         o  IP V4 address: X'01'
    //         o  DOMAINNAME: X'03'
    //         o  IP V6 address: X'04'
    //      o  BND.ADDR       server bound address
    //      o  BND.PORT       server bound port in network octet order
    //
    buf: [u8; 10],
}

impl SocksReply {
    pub fn new(status: ResponseCode) -> Self {
        let buf = [
            // VER
            SOCKS_VERSION,
            // REP
            status as u8,
            // RSV
            RESERVED,
            // ATYP
            1,
            // BND.ADDR
            0,
            0,
            0,
            0,
            // BND.PORT
            0,
            0,
        ];
        Self { buf }
    }

    pub async fn send<T>(&self, stream: &mut T) -> io::Result<()>
    where
        T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    {
        stream.write_all(&self.buf[..]).await?;
        Ok(())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

/////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Способы аутентификации
pub enum AuthMethods {
    /// Аутентификация не нужна
    NoAuth = 0x00,
    // GssApi = 0x01,
    /// Юзер и пароль
    UserPass = 0x02,
    /// Без аутентификации
    NoMethods = 0xFF,
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Merino {
    // TCP листенер
    listener: TcpListener,
    // Список разрешенных пользователей
    users: Arc<Vec<User>>,
    // Метод аутентификации
    auth_methods: Arc<Vec<u8>>,
    // Таймаут для соединений
    timeout: Option<Duration>,
}

impl Merino {
    /// Создаем инстанс
    pub async fn new(
        port: u16,
        ip: &str,
        auth_methods: Vec<u8>,
        users: Vec<User>,
        timeout: Option<Duration>,
    ) -> io::Result<Self> {
        info!("Listening on {}:{}", ip, port);
        Ok(Merino {
            listener: TcpListener::bind((ip, port)).await?,
            auth_methods: Arc::new(auth_methods),
            users: Arc::new(users),
            timeout,
        })
    }

    /// Запускаем в работу сервер
    pub async fn serve(&mut self) {
        info!("Serving Connections...");

        // Принимаем новое входящее подключение
        while let Ok((stream, client_addr)) = self.listener.accept().await {
            // Получем Arc на список пользователей
            let users = self.users.clone();
            // Получаем Arc на методы аутентификации
            let auth_methods = self.auth_methods.clone();
            // Создаем копию таймаута
            let timeout = self.timeout;

            // Запускаем отдельную корутину tokio
            tokio::spawn(async move {
                // Создаем клиента из параметров
                let mut client = SOCKClient::new(stream, users, auth_methods, timeout);
                // Инициализируем клиента
                match client.init().await {
                    Ok(_) => {}
                    Err(error) => {
                        // В клиенте произошла ошибка
                        error!("Error! {:?}, client: {:?}", error, client_addr);

                        // Создаем в ответ ответ с ошибкой
                        if let Err(e) = SocksReply::new(error.into()).send(&mut client.stream).await
                        {
                            warn!("Failed to send error code: {:?}", e);
                        }

                        // Вырубаем клиента
                        if let Err(e) = client.shutdown().await {
                            warn!("Failed to shutdown TcpStream: {:?}", e);
                        };
                    }
                };
            });
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Непосредственно клиент прокси
pub struct SOCKClient<T: AsyncRead + AsyncWrite + Send + Unpin + 'static> {
    // TCP stream
    stream: T,
    // Методы аутентификации
    auth_nmethods: u8,
    auth_methods: Arc<Vec<u8>>,
    // Список юзеров аутентификации
    authed_users: Arc<Vec<User>>,
    // Версия
    socks_version: u8,
    // Таймаут
    timeout: Option<Duration>,
}

impl<T> SOCKClient<T>
where
    T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    /// Создаем новый SOCKClient
    pub fn new(
        stream: T,
        authed_users: Arc<Vec<User>>,
        auth_methods: Arc<Vec<u8>>,
        timeout: Option<Duration>,
    ) -> Self {
        SOCKClient {
            stream,
            auth_nmethods: 0,
            socks_version: 0,
            authed_users,
            auth_methods,
            timeout,
        }
    }

    /// Создаем клиента без аутентификации
    pub fn new_no_auth(stream: T, timeout: Option<Duration>) -> Self {
        // FIXME: use option here
        let authed_users: Arc<Vec<User>> = Arc::new(Vec::new());
        let no_auth: Vec<u8> = vec![AuthMethods::NoAuth as u8];
        let auth_methods: Arc<Vec<u8>> = Arc::new(no_auth);

        SOCKClient {
            stream,
            auth_nmethods: 0,
            socks_version: 0,
            authed_users,
            auth_methods,
            timeout,
        }
    }

    /// Мутабельный геттер для внутреннего потока
    pub fn stream_mut(&mut self) -> &mut T {
        &mut self.stream
    }

    /// Делаем проверку аутентификации данного юзера
    fn authed(&self, user: &User) -> bool {
        self.authed_users.contains(user)
    }

    /// Вырубаем клиента
    pub async fn shutdown(&mut self) -> io::Result<()> {
        self.stream.shutdown().await?;
        Ok(())
    }

    /// Выполняем инициализацию
    pub async fn init(&mut self) -> Result<(), MerinoError> {
        debug!("New connection");

        // Заголовок из 2х байт
        let mut header = [0u8; 2];

        // Читаем из потока первые 2 байта для определения версии
        self.stream.read_exact(&mut header).await?;

        // Версия сокетов и аутентификации
        self.socks_version = header[0];
        self.auth_nmethods = header[1];

        trace!(
            "Version: {} Auth nmethods: {}",
            self.socks_version,
            self.auth_nmethods
        );

        // Определяем версию
        match self.socks_version {
            // V5 используется
            SOCKS_VERSION => {
                // Аутентификация клиента
                self.auth().await?;

                // Запускаем в работу клиента
                self.handle_client().await?;
            }
            _ => {
                warn!("Init: Unsupported version: SOCKS{}", self.socks_version);
                // Вырубаем TCP поток
                self.shutdown().await?;
            }
        }

        Ok(())
    }

    // Выполняем аутентификацию
    async fn auth(&mut self) -> Result<(), MerinoError> {
        debug!("Authenticating");

        // Получаем валидные методы аутентификации
        let methods = self.get_avalible_methods().await?;
        trace!("methods: {:?}", methods);

        // Массив ответа на запрос
        let mut response = [0u8; 2];

        // Выставляем в первый байт версию 5
        response[0] = SOCKS_VERSION;

        // TODO: Использовать буфферизацию сокета для логина и пароля тоже?

        // Методы содержат пароль и пользователя
        if methods.contains(&(AuthMethods::UserPass as u8)) {
            // Устанавливаем стандартный метод аутентификации
            response[1] = AuthMethods::UserPass as u8;

            // Отправляем в стрим необходимость юзера и пароль
            debug!("Sending USER/PASS packet");
            self.stream.write_all(&response).await?;

            // Массив для заголовка
            let mut header = [0u8; 2];

            // Читаем из ответа и версию
            self.stream.read_exact(&mut header).await?;

            // debug!("Auth Header: [{}, {}]", header[0], header[1]);

            // Парсим длину имени
            let ulen = header[1] as usize;

            // Создаем буффер для имени
            let mut username = vec![0; ulen];

            // Читаем имя из TCP
            self.stream.read_exact(&mut username).await?;

            // Получаем длину пароля
            let mut plen = [0u8; 1];
            self.stream.read_exact(&mut plen).await?;

            // Получаем пароль
            let mut password = vec![0; plen[0] as usize];
            self.stream.read_exact(&mut password).await?;

            // Перегоняем юзера и пароль в UTF-8
            let username = String::from_utf8_lossy(&username).to_string();
            let password = String::from_utf8_lossy(&password).to_string();

            // Создаем объект с юзером и паролем
            let user = User { username, password };

            // Проверяем данного пользователя
            if self.authed(&user) {
                debug!("Access Granted. User: {}", user.username);
                // Пишем в ответ успешность
                let response = [1, ResponseCode::Success as u8];
                self.stream.write_all(&response).await?;
            } else {
                debug!("Access Denied. User: {}", user.username);
                // Выдаем в ответ ошибку
                let response = [1, ResponseCode::Failure as u8];
                self.stream.write_all(&response).await?;

                // Закрываем сокет
                self.shutdown().await?;
            }

            Ok(())
        } else if methods.contains(&(AuthMethods::NoAuth as u8)) {
            // Говорим клиенту, что нам не нужна аутентификация
            response[1] = AuthMethods::NoAuth as u8;
            debug!("Sending NOAUTH packet");
            self.stream.write_all(&response).await?;
            debug!("NOAUTH sent");
            Ok(())
        } else {
            // У сервера нет вообще методов аутентификации
            warn!("Client has no suitable Auth methods!");
            // Выдаем ответ
            response[1] = AuthMethods::NoMethods as u8;
            self.stream.write_all(&response).await?;

            // Вырубаем сокет
            self.shutdown().await?;

            Err(MerinoError::Socks(ResponseCode::Failure))
        }
    }

    /// Обрабатываем клиента
    pub async fn handle_client(&mut self) -> Result<usize, MerinoError> {
        debug!("Starting to relay data");

        // Заголовок пакета
        let req = SOCKSReq::from_stream(&mut self.stream).await?;

        // Печатаем адрес
        let displayed_addr = pretty_print_addr(&req.addr_type, &req.addr);
        info!(
            "New Request: Command: {:?} Addr: {}, Port: {}",
            req.command, displayed_addr, req.port
        );

        // Отвечаем на переданную комманду
        match req.command {
            // Используем наш прокси для соединения к конкретному адресу и порту
            SockCommand::Connect => {
                debug!("Handling CONNECT Command");

                // По адресу из запроса получаем непосредственно список адресов
                let sock_addr = addr_to_socket(&req.addr_type, &req.addr, req.port).await?;

                trace!("Connecting to: {:?}", sock_addr);

                // Определяем таймаут для подключения
                let time_out = if let Some(time_out) = self.timeout {
                    time_out
                } else {
                    Duration::from_millis(50)
                };

                // Оборачиваем в таймаут попытку подключения к каждому из полученных адресов
                let mut target =
                    timeout(
                        time_out,
                        async move { TcpStream::connect(&sock_addr[..]).await },
                    )
                    .await
                    .map_err(|_| MerinoError::Socks(ResponseCode::AddrTypeNotSupported))
                    .map_err(|_| MerinoError::Socks(ResponseCode::AddrTypeNotSupported))??;

                trace!("Connected!");
                
                // Если смогли подключиться нормально, тогда отсылаем в ответ подтверждение успешности подключения
                SocksReply::new(ResponseCode::Success)
                    .send(&mut self.stream)
                    .await?;

                // Запускаем двухстороннее копирование данных между открытыми сокетами
                trace!("Copy bidirectional");
                match tokio::io::copy_bidirectional(&mut self.stream, &mut target).await {
                    // Если уже закрыто, то просто игнорируем
                    Err(e) if e.kind() == std::io::ErrorKind::NotConnected => {
                        trace!("already closed");
                        Ok(0)
                    }
                    Err(e) => Err(MerinoError::Io(e)),
                    Ok((_s_to_t, t_to_s)) => Ok(t_to_s as usize),
                }
            }
            SockCommand::Bind => Err(MerinoError::Io(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Bind not supported",
            ))),
            SockCommand::UdpAssosiate => Err(MerinoError::Io(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "UdpAssosiate not supported",
            ))),
        }
    }

    /// Получаем список доступных методов
    async fn get_avalible_methods(&mut self) -> io::Result<Vec<u8>> {
        let mut methods: Vec<u8> = Vec::with_capacity(self.auth_nmethods as usize);
        for _ in 0..self.auth_nmethods {
            let mut method = [0u8; 1];
            self.stream.read_exact(&mut method).await?;
            if self.auth_methods.contains(&method[0]) {
                methods.append(&mut method.to_vec());
            }
        }
        Ok(methods)
    }
}
