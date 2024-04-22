use std::{
    io::{
        self,
        Read,
        Write
    },
    error::Error,
    net::SocketAddr,
    collections::HashMap
};
use mio::{
    net::{
        TcpListener, 
        TcpStream
    },
    Events, 
    Interest, 
    Poll, 
    Token
};


// Токены позволяют нам идентифицировать, какой ивент на каком сокете произошел
const SERVER: Token = Token(0);

struct ConnectionInfo{
    sock: TcpStream,
    write_data: Option<Vec<u8>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Создаем пулинг
    let mut poll: Poll = Poll::new()?;
    // Создаем хранилище для ивентов
    let mut events: Events = Events::with_capacity(128);

    // Адрес серверного сокета
    let addr = "127.0.0.1:12345".parse()?;
    // Создаем листнер для сокета
    let mut server = TcpListener::bind(addr)?;
    // Регистрируем наш серверный сокет с токеном на события чтения
    poll.registry()
        .register(&mut server, SERVER, Interest::READABLE)?;

    // Маса с активными соединениями
    let mut next_token_id: usize = 1;
    let mut connections_map: HashMap<Token, ConnectionInfo> = HashMap::new();

    // Стуртуем цикл
    loop {
        // Блокируемся на пулинге событий пока мы не получи новый ивент на каком-то сокете
        poll.poll(&mut events, None)?;

        // Обрабатываем каждый ивент, который мы получили
        for event in events.iter() {
            // Мы можем посмотреть токен, который мы указали до этого при регистрации каждого из сокетов
            match event.token() {
                SERVER => {
                    // Если у нас событие на сокете сервера - это новое соединение
                    // получая новое соединение и закрывая его сразу же - мы говорим клиенту,
                    // что мы больше не работаем и ему придет EOF
                    let connection: io::Result<(TcpStream, SocketAddr)> = server.accept();
                    if let Ok((conn, addr)) = connection {
                        println!("Connection accepted for address: {}", addr);

                        // Регистрируем сокет на события возможности чтения
                        // TODO: Упадет на переполнении
                        next_token_id = next_token_id.checked_add(next_token_id).unwrap();
                        let token = Token(next_token_id);

                        let mut conn_info = ConnectionInfo{
                            sock: conn,
                            write_data: None
                        };

                        poll.registry()
                            .register(&mut conn_info.sock, token, Interest::READABLE)?;

                        connections_map.insert(token, conn_info);
                    }
                }
                token => {
                    let conn_info = connections_map.get_mut(&token);
                    if let Some(conn_info) = conn_info {    
                        if event.is_writable() {
                            println!("Socket writable");

                            if let Some(data) = conn_info.write_data.take() {
                                // Мы скорее всего можем записать в сокет без блокировки
                                let sock: &mut TcpStream = &mut conn_info.sock;
                                sock.write_all(data.as_slice())
                                    .unwrap();
                            }

                            poll.registry()
                                .reregister(&mut conn_info.sock, token, Interest::READABLE)
                                .unwrap();
                        }    

                        if event.is_readable() {
                            println!("Socket readable");
                            // Мы скорее всего можем читать из сокета без блокировки

                            // Читаем тестовые данные
                            let mut test_data: [u8; 4] = [76; 4];

                            // Смотрим сколько данных у нас прилетело, если недостаточно - продолжаем ждать
                            let peek_res = conn_info.sock.peek(&mut test_data);
                            match peek_res {
                                Ok(size) => {
                                    println!("Peek size: {}", size);
                                    if size != test_data.len(){
                                        println!("Wait more data...");
                                        continue;
                                    }
                                },
                                Err(e) => {
                                    println!("Peek err: {}", e);
                                }
                            }

                            // Данные все есть - вычитываем
                            // Данная функция возвращает ошибку если было отправлено меньше данных
                            // Данные придут - но тут будет ошибка
                            // TODO: !!!
                            // Опасность данной функции в том, что остальные данные отбрасываются!
                            // Связано вроде как с неблокирующей природой сокетов, в Tokio вроде бы нет такой проблемы
                            // https://github.com/tokio-rs/mio/issues/634#issuecomment-316241765
                            // Да и вообще нету реализации в mio https://github.com/tokio-rs/mio/blob/master/src/net/tcp/stream.rs
                            let read_result = conn_info.sock
                               .read_exact(&mut test_data);

                            //let read_result = conn_info.sock
                            //    .read(&mut test_data);

                            // Не прочитали - клиент отвалился
                            match read_result {
                                Ok(_size)=>{
                                    //println!("Received size: {}", _size);

                                    // TODO: Test
                                    conn_info.write_data = Some(Vec::from(test_data));

                                    if conn_info.write_data.is_some() {
                                        poll.registry()
                                            .reregister(&mut conn_info.sock, token, Interest::WRITABLE | Interest::READABLE)
                                            .unwrap();
                                    }else{
                                        poll.registry()
                                            .reregister(&mut conn_info.sock, token, Interest::READABLE)
                                            .unwrap();
                                    }
                                },
                                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                                    println!("Socket disconnect: {}", e);
    
                                    poll.registry()
                                        .deregister(&mut conn_info.sock)
                                        .unwrap();
    
                                    connections_map.remove(&token);
                                    continue;
                                },
                                Err(_e) => {
                                    println!("Socket read error: {}", _e);
                                }
                            }
                        }

                    }else{
                        // TODO: ???
                        //poll.registry().deregister(&mut token);
                    }

                    // Мы просто выходим из цикла после работы
                    //return Ok(());
                }
            }
        }
    }
}