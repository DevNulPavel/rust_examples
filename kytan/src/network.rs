// Copyright 2016-2017 Chang Lan
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::device;
use crate::utils;
use bincode::{deserialize, serialize};
use dns_lookup;
use log::{info, warn};
use mio;
use rand::{thread_rng, Rng};
use ring::{aead, pbkdf2};
use serde_derive::{Deserialize, Serialize};
use snap;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::num::NonZeroU32;
use std::os::unix::io::AsRawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time;
use transient_hashmap::TransientHashMap;

pub static INTERRUPTED: AtomicBool = AtomicBool::new(false);
static CONNECTED: AtomicBool = AtomicBool::new(false);
static LISTENING: AtomicBool = AtomicBool::new(false);
const KEY_LEN: usize = 32;

type Id = u8;
type Token = u64;

fn generate_add_nonce() -> (aead::Aad<[u8; 0]>, aead::Nonce) {
    let nonce = aead::Nonce::assume_unique_for_key([0; 12]);
    let aad = aead::Aad::empty();
    (aad, nonce)
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum Message {
    Request,
    Response { id: Id, token: Token, dns: String },
    Data { id: Id, token: Token, data: Vec<u8> },
}

const TUN: mio::Token = mio::Token(0);
const SOCK: mio::Token = mio::Token(1);

fn resolve(host: &str) -> Result<IpAddr, String> {
    let ip_list = dns_lookup::lookup_host(host).map_err(|_| "dns_lookup::lookup_host")?;
    Ok(ip_list.first().unwrap().clone())
}

fn create_tun_attempt() -> device::Tun {
    // Описываем локальную функцию для повторных попыток
    fn attempt(id: u8) -> device::Tun {
        match id {
            255 => panic!("Unable to create TUN device."),
            // Пытаемся создать туннель с определенным ID
            // Если не удалось - делаем новую попытку с id + 1
            _ => match device::Tun::create(id) {
                Ok(tun) => tun,
                Err(_) => attempt(id + 1),
            },
        }
    }
    attempt(0)
}

fn derive_keys(password: &str) -> aead::LessSafeKey {
    // Буффер для ключа
    let mut key = [0; KEY_LEN];

    // Соль для данных
    let salt = vec![0; 64];
    // 1024 итерации
    let pbkdf2_iterations: NonZeroU32 = NonZeroU32::new(1024).unwrap();

    // Формируем ключ шифрования на основе пароля
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        pbkdf2_iterations,
        &salt,
        password.as_bytes(),
        &mut key,
    );

    // Оборачиваем данный сгенерированный ключ
    let less_safe_key =
        aead::LessSafeKey::new(aead::UnboundKey::new(&aead::AES_256_GCM, &key).unwrap());
    less_safe_key
}

fn initiate(
    socket: &UdpSocket,
    addr: &SocketAddr,
    secret: &str,
) -> Result<(Id, Token, String), String> {
    let key = derive_keys(secret);
    let req_msg = Message::Request;
    let encoded_req_msg: Vec<u8> = serialize(&req_msg).map_err(|e| e.to_string())?;
    let mut encrypted_req_msg = encoded_req_msg.clone();
    encrypted_req_msg.resize(encoded_req_msg.len() + key.algorithm().tag_len(), 0);
    let (aad, nonce) = generate_add_nonce();
    key.seal_in_place_append_tag(nonce, aad, &mut encrypted_req_msg)
        .unwrap();

    let mut remaining_len = encrypted_req_msg.len();
    while remaining_len > 0 {
        let sent_bytes = socket
            .send_to(&encrypted_req_msg, addr)
            .map_err(|e| e.to_string())?;
        remaining_len -= sent_bytes;
    }
    info!("Request sent to {}.", addr);

    let mut buf = [0u8; 1600];
    let (len, recv_addr) = socket.recv_from(&mut buf).map_err(|e| e.to_string())?;
    assert_eq!(&recv_addr, addr);
    info!("Response received from {}.", addr);

    let (aad, nonce) = generate_add_nonce();
    let decrypted_buf = key.open_in_place(nonce, aad, &mut buf[0..len]).unwrap();

    let dlen = decrypted_buf.len();
    let resp_msg: Message = deserialize(&decrypted_buf[0..dlen]).map_err(|e| e.to_string())?;
    match resp_msg {
        Message::Response { id, token, dns } => Ok((id, token, dns)),
        _ => Err(format!("Invalid message {:?} from {}", resp_msg, addr)),
    }
}

pub fn connect(host: &str, port: u16, default: bool, secret: &str) {
    info!("Working in client mode.");
    let remote_ip = resolve(host).unwrap();
    let remote_addr = SocketAddr::new(remote_ip, port);
    info!("Remote server: {}", remote_addr);

    let local_addr: SocketAddr = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
    let socket = UdpSocket::bind(&local_addr).unwrap();

    let key = derive_keys(secret);

    let (id, token, dns) = initiate(&socket, &remote_addr, &secret).unwrap();
    info!(
        "Session established with token {}. Assigned IP address: 10.10.10.{}. dns: {}",
        token, id, dns
    );

    info!("Bringing up TUN device.");
    let mut tun = create_tun_attempt();
    let tun_rawfd = tun.as_raw_fd();
    tun.up(id);
    let mut tunfd = mio::unix::SourceFd(&tun_rawfd);
    info!(
        "TUN device {} initialized. Internal IP: 10.10.10.{}/24.",
        tun.name(),
        id
    );

    info!("setting dns to {}", dns);
    utils::set_dns(&dns).unwrap();

    let mut poll = mio::Poll::new().unwrap();
    info!("Setting up TUN device for polling.");
    poll.registry()
        .register(
            &mut tunfd,
            TUN,
            mio::Interest::READABLE | mio::Interest::WRITABLE,
        )
        .unwrap();

    info!("Setting up socket for polling.");
    let mut sockfd = mio::net::UdpSocket::from_std(socket);
    poll.registry()
        .register(&mut sockfd, SOCK, mio::Interest::READABLE)
        .unwrap();

    let mut events = mio::Events::with_capacity(1024);
    let mut buf = [0u8; 1600];

    // RAII so ignore unused variable warning
    let _gw =
        utils::DefaultGateway::create("10.10.10.1", &format!("{}", remote_addr.ip()), default);

    let mut encoder = snap::raw::Encoder::new();
    let mut decoder = snap::raw::Decoder::new();

    CONNECTED.store(true, Ordering::Relaxed);
    info!("Ready for transmission.");

    loop {
        if INTERRUPTED.load(Ordering::Relaxed) {
            break;
        }
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                SOCK => {
                    let (len, addr) = sockfd.recv_from(&mut buf).unwrap();
                    let (aad, nonce) = generate_add_nonce();

                    let decrypted_buf = key.open_in_place(nonce, aad, &mut buf[0..len]).unwrap();
                    let dlen = decrypted_buf.len();
                    let msg: Message = deserialize(&decrypted_buf[0..dlen]).unwrap();
                    match msg {
                        Message::Request
                        | Message::Response {
                            id: _,
                            token: _,
                            dns: _,
                        } => {
                            warn!("Invalid message {:?} from {}", msg, addr);
                        }
                        Message::Data {
                            id: _,
                            token: server_token,
                            data,
                        } => {
                            if token == server_token {
                                let decompressed_data = decoder.decompress_vec(&data).unwrap();
                                let data_len = decompressed_data.len();
                                let mut sent_len = 0;
                                while sent_len < data_len {
                                    sent_len +=
                                        tun.write(&decompressed_data[sent_len..data_len]).unwrap();
                                }
                            } else {
                                warn!(
                                    "Token mismatched. Received: {}. Expected: {}",
                                    server_token, token
                                );
                            }
                        }
                    }
                }
                TUN => {
                    let len: usize = tun.read(&mut buf).unwrap();
                    let data = &buf[0..len];
                    let msg = Message::Data {
                        id: id,
                        token: token,
                        data: encoder.compress_vec(data).unwrap(),
                    };
                    let encoded_msg = serialize(&msg).unwrap();
                    let mut encrypted_msg = encoded_msg.clone();
                    encrypted_msg.resize(encoded_msg.len() + key.algorithm().tag_len(), 0);
                    let (aad, nonce) = generate_add_nonce();
                    key.seal_in_place_append_tag(nonce, aad, &mut encrypted_msg)
                        .unwrap();
                    let mut sent_len = 0;
                    while sent_len < encrypted_msg.len() {
                        sent_len += sockfd
                            .send_to(&encrypted_msg[sent_len..encrypted_msg.len()], remote_addr)
                            .unwrap();
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

pub fn serve(port: u16, secret: &str, dns: IpAddr) {
    if cfg!(not(target_os = "linux")) {
        panic!("Server mode is only available in Linux!");
    }

    info!("Working in server mode.");

    // Получаем наш публичный ip
    let public_ip = utils::get_public_ip().unwrap();
    info!("Public IP: {}", public_ip);

    // Включаем ip форвардинг, чтобы можно было определять маршрут по которому должен идти наш пакет
    info!("Enabling kernel's IPv4 forwarding.");
    utils::enable_ipv4_forwarding().unwrap();

    // Пытаемся создать туннель и запустить его
    info!("Bringing up TUN device.");
    let mut tun = create_tun_attempt();
    tun.up(1);

    // Получаем файловый дескриптор туннеля
    let tun_rawfd = tun.as_raw_fd();
    // Оборачиваем наш туннель в mio источник событий
    let mut tunfd = mio::unix::SourceFd(&tun_rawfd);
    info!(
        "TUN device {} initialized. Internal IP: 10.10.10.1/24.",
        tun.name()
    );

    // Создаем UDP сокет ожидающих подключений на порту туннеля
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
    let mut sockfd = mio::net::UdpSocket::bind(addr).expect("UDP listen socket bind failed");
    info!("Listening on: 0.0.0.0:{}.", port);

    // Создаем mio пулер
    let mut poll = mio::Poll::new().unwrap();
    // Регистрируем UDP сокет на новые входящие подключения
    poll.registry()
        .register(&mut sockfd, SOCK, mio::Interest::READABLE)
        .expect("UDP socket read listen failed");
    // Регистрируем сокет туннеля на события чтения
    poll.registry()
        .register(&mut tunfd, TUN, mio::Interest::READABLE)
        .expect("Tunnel socket read listen failed");

    // Пулл для событий
    let mut events = mio::Events::with_capacity(1024);

    let mut rng = thread_rng();
    // Список созможных ID от [2 до 253]
    // 
    let mut available_ids: Vec<Id> = (2..254).collect();
    // Хеш мапа клиентов со временем жизни в 60 секунд
    let mut client_info: TransientHashMap<Id, (Token, SocketAddr)> = TransientHashMap::new(60);

    // Буффер для данных
    let mut buf = [0u8; 1600];

    // Snap сжатие и расжатие
    let mut encoder = snap::raw::Encoder::new();
    let mut decoder = snap::raw::Decoder::new();

    // Генерация ключа шифрования на основе переданного секрета
    let key = derive_keys(secret);

    // Выставляем флаг активности цикла обработки событий
    // Нужен для прекращения работы цикла выше
    LISTENING.store(true, Ordering::Relaxed);
    info!("Ready for transmission.");

    loop {
        // Не была ли прервана работа?
        if INTERRUPTED.load(Ordering::Relaxed) {
            break;
        }

        // К списку достапных ID снова добавляем клиентов, у которых истекло время активности последней
        available_ids.append(&mut client_info.prune());

        // Полим возможные события в массив событий
        poll.poll(&mut events, None).expect("Event loop poll failed");

        // Обходим теперь прилетевшие нам события
        for event in events.iter() {
            // Проверяем на каком именно сокете прилетели события
            match event.token() {
                // Событие на нашем UDP сокете входящих соединений
                SOCK => {
                    // Читаем данные из сокета, получаем длину и адрес, от которого данные прилетели
                    let (len, addr) = sockfd.recv_from(&mut buf).unwrap();

                    // Дешифруем данные из буффера
                    // TODO: ???
                    let (aad, nonce) = generate_add_nonce();
                    let decrypted_buf = key.open_in_place(nonce, aad, &mut buf[0..len]).unwrap();

                    // Десереализуем наши бинарные данные в сообщение
                    let dlen = decrypted_buf.len();
                    let msg: Message = deserialize(&decrypted_buf[0..dlen]).unwrap();

                    // Смотрим что за сообщение прилетело
                    match msg {
                        // Запрос на новое подключение?
                        Message::Request => {
                            // Берем доступный ID
                            let client_id: Id = match available_ids.pop(){
                                Some(id) => id,
                                None => {
                                    warn!("All available IDs from 10.10.10.0/24 pool are complete, can't connect new client from {}", addr);
                                    continue;
                                }
                            };
                            // Генерируем токен подключения для данного клиента
                            let client_token: Token = rng.gen::<Token>();

                            // Сохраняем данного клиента (его токен и адрес) под указанным ID
                            client_info.insert(client_id, (client_token, addr));

                            // Выводим инфу о подключении
                            info!(
                                "Got request from {}. Assigning IP address: 10.10.10.{}.",
                                addr, client_id
                            );

                            // Формируем ответ к серверу, указываем его id, токен и используемый DNS сервер
                            let reply = Message::Response {
                                id: client_id,
                                token: client_token,
                                dns: dns.to_string(),
                            };
                            
                            // Кодируем в бинарный вид
                            let mut encrypted_reply = serialize(&reply).unwrap();

                            // Добавляем в конец еще места для описания алгоритма шифрования
                            encrypted_reply
                                .resize(encrypted_reply.len() + key.algorithm().tag_len(), 0);

                            // Шифруем наши данные, добавляя в конец алгоритм шифрования
                            let (aad, nonce) = generate_add_nonce();
                            key.seal_in_place_append_tag(nonce, aad, &mut encrypted_reply)
                                .unwrap();

                            // Запускаем цикл отправки данных
                            let mut sent_len = 0;
                            while sent_len < encrypted_reply.len() {
                                let data = &encrypted_reply.get(sent_len..encrypted_reply.len()).unwrap();
                                sent_len += sockfd.send_to(data, addr).unwrap();
                            }
                        }

                        // Такого сообщения не может быть
                        Message::Response {
                            id: _,
                            token: _,
                            dns: _,
                        } => warn!("Invalid message {:?} from {}", msg, addr),

                        // Прилетели какие-то данные от существующего клиента
                        Message::Data { id, token, data } => {
                            // Получаем для указанного id клиента непосредственно самого клиента
                            match client_info.get(&id) {
                                // Нет такого клиента
                                None => {
                                    warn!("Unknown data with token {} from id {}.", token, id)
                                },
                                // Клиента нашли
                                Some(&(t, _)) => {
                                    // Может быть токен не совпадает?
                                    if t != token {
                                        warn!(
                                            "Unknown data with mismatched token {} from id {}. \
                                                Expected: {}",
                                            token, id, t
                                        );
                                    } else {
                                        // Разжимаем данные
                                        let decompressed_data = decoder.decompress_vec(&data).unwrap();
                                        let data_len = decompressed_data.len();

                                        // Записываем эти данные в хендлер туннеля
                                        let mut sent_len = 0;
                                        while sent_len < data_len {
                                            sent_len += tun
                                                .write(&decompressed_data[sent_len..data_len])
                                                .unwrap();
                                        }
                                    }
                                }
                            }
                        },
                    }
                }

                // Прилетели данные в туннель
                TUN => {
                    // Вычитываем данные из туннеля
                    let len: usize = tun.read(&mut buf).unwrap();
                    let data = &buf[0..len];

                    // Берем id целевого клиента без парсинга, просто как 19й байт
                    let client_id: u8 = data[19];

                    // Ищем нужного нам клиента
                    match client_info.get(&client_id) {
                        // Такого клиента нету у нас
                        None => warn!("Unknown IP packet from TUN for client {}.", client_id),
                        
                        // Нашли нужного нам клиента
                        Some(&(token, addr)) => {
                            // Данные для отправки
                            let msg = Message::Data {
                                id: client_id,
                                token: token,
                                data: encoder.compress_vec(data).unwrap(), // Заново сжимаем данные для отправки
                            };

                            // Сериализуем данные
                            let mut encrypted_msg = serialize(&msg).unwrap();
                            encrypted_msg.resize(encrypted_msg.len() + key.algorithm().tag_len(), 0);

                            // Шифруем наши данные для ключа
                            let (aad, nonce) = generate_add_nonce();
                            key.seal_in_place_append_tag(nonce, aad, &mut encrypted_msg)
                                .unwrap();

                            // Пишем данные теперь уже в обычный сокет
                            let mut sent_len = 0;
                            while sent_len < encrypted_msg.len() {
                                sent_len += sockfd
                                    .send_to(&encrypted_msg[sent_len..encrypted_msg.len()], addr)
                                    .unwrap();
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::network::*;
    use std::net::Ipv4Addr;

    #[cfg(target_os = "linux")]
    use std::thread;

    #[test]
    fn resolve_test() {
        assert_eq!(
            resolve("127.0.0.1").unwrap(),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn integration_test() {
        assert!(utils::is_root());
        let _server =
            thread::spawn(move || serve(8964, "password", "8.8.8.8".parse::<IpAddr>().unwrap()));

        thread::sleep(time::Duration::from_secs(1));
        assert!(LISTENING.load(Ordering::Relaxed));

        let remote_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8964);
        let local_addr: SocketAddr = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
        let local_socket = UdpSocket::bind(&local_addr).unwrap();

        let (id, _, _) = initiate(&local_socket, &remote_addr, "password").unwrap();
        assert_eq!(id, 253);

        let _client = thread::spawn(move || connect("127.0.0.1", 8964, false, "password"));

        thread::sleep(time::Duration::from_secs(1));
        assert!(CONNECTED.load(Ordering::Relaxed));

        INTERRUPTED.store(true, Ordering::Relaxed);
    }
}
