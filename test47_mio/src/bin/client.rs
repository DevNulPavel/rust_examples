use std::error::Error;
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
const CLIENT: Token = Token(1);

fn main() -> Result<(), Box<dyn Error>> {
    // Создаем пулинг
    let mut poll = Poll::new()?;
    // Создаем хранилище для ивентов
    let mut events = Events::with_capacity(128);

    // Адрес серверного сокета
    let addr = "127.0.0.1:13265".parse()?;
    // Создаем листнер для сокета
    let mut server = TcpListener::bind(addr)?;
    // Регистрируем наш серверный сокет с токеном на события чтения
    poll.registry()
        .register(&mut server, SERVER, Interest::READABLE)?;

    // Создаем клиентский сокет
    let mut client = TcpStream::connect(addr)?;
    // Регистрируем сокет на события возможности чтения и записи
    poll.registry()
        .register(&mut client, CLIENT, Interest::READABLE | Interest::WRITABLE)?;

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
                    let connection = server.accept();
                    println!("Connection accepted");
                    drop(connection);
                }
                CLIENT => {
                    if event.is_writable() {
                        println!("Socket writable");
                        // Мы скорее всего можем записать в сокет без блокировки
                    }

                    if event.is_readable() {
                        println!("Socket readable");
                        // Мы скорее всего можем читать из сокета без блокировки
                    }

                    // Мы просто выходим из цикла после работы
                    return Ok(());
                }
                _ => {
                    // Не ожидаем никаких больше событий от других токенов
                    unreachable!()
                },
            }
        }
    }
}