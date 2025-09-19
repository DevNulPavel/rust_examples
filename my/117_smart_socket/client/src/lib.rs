// TODO: Здесь можно было бы объединить в группу все импорты, которые
// связанные с std вот так.
// use std::{
//  error::Error,
//  fmt,
//  io::{Read, Write},
//  net::TcpStream,
// };
//
// TODO: Здесь можно было бы сразу импортировать fmt::Display трейт.
//
use std::error::Error;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;

////////////////////////////////////////////////////////////////////////////////

// TODO: Здесь можно для красоты сделать документацию к enum +
// описание к каждому элементу, так как это публичный enum, да и в целом -
// это красиво будет
// 
// TODO: Команда используется на клиенте и на сервере, поэтому оптимальнее было бы сделать
// какую-то общую библиотечку.
pub enum Command {
    Turnoff,
    TurnOn,
    IsEnabled,
    GetPower,
    Unknown,
}

impl From<u8> for Command {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Turnoff,
            1 => Self::TurnOn,
            2 => Self::IsEnabled,
            3 => Self::GetPower,
            _ => Self::Unknown,
        }
    }
}

impl From<Command> for u8 {
    fn from(cmd: Command) -> Self {
        match cmd {
            Command::Turnoff => 0,
            Command::TurnOn => 1,
            Command::IsEnabled => 2,
            Command::GetPower => 3,
            // TODO: Здесь получается не совсем симметричная
            // конвертация между типами, но с другой стороны -
            // для чего именно нужна конвертация в число -
            // точно не ясно
            Command::Unknown => 255,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

// TODO: Здесь можно для красоты сделать документацию к enum +
// описание к каждому элементу, так как это публичный enum, да и в целом -
// это красиво будет.
//
// TODO: Тип ответа используется на клиенте и на сервере, поэтому оптимальнее было бы сделать
// какую-то общую библиотечку.
pub enum Response {
    Ok,
    Enabled,
    Disabled,
    Power(f32),
    Unknown,
}

impl From<[u8; 5]> for Response {
    fn from(bytes: [u8; 5]) -> Self {
        // TODO: Здесь у нас проверяется лишь первый байт в большинстве случаев,
        // возможно, что было бы полезно еще проверять остальные байты
        // на 0. А если там не 0, то кидаем ошибку. Вдруг в целом
        // что-то не то с протоколом.
        match bytes {
            [0, ..] => Self::Ok,
            [1, ..] => Self::Enabled,
            [2, ..] => Self::Disabled,
            [3, ..] => {
                let mut buf = [0u8; 4];
                buf.copy_from_slice(&bytes[1..]);
                Self::Power(f32::from_be_bytes(buf))
            }
            // TODO: ???
            _ => Self::Unknown,
        }
    }
}

impl From<Response> for [u8; 5] {
    fn from(resp: Response) -> Self {
        // Создаем буфер на стеке с нулями всеми
        let mut buffer = [0u8; 5];
        match resp {
            Response::Ok => {}
            Response::Enabled => buffer[0] = 1,
            Response::Disabled => buffer[0] = 2,
            Response::Power(pwr) => {
                buffer[0] = 3;
                buffer[1..].copy_from_slice(&pwr.to_be_bytes())
            }
            // TODO: Здесь получается не совсем симметричная
            // конвертация между типами, но с другой стороны -
            // для чего именно нужна конвертация в число -
            // точно не ясно
            Response::Unknown => buffer[0] = 255,
        };
        buffer
    }
}

// TODO: Здесь можно было бы сразу импортировать fmt::Display трейт.
impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::Ok => write!(f, "Ok"),
            Response::Enabled => write!(f, "Enabled"),
            Response::Disabled => write!(f, "Disabled"),
            // TODO: Последняя версия Rust позволяет уже сразу
            // же переменные подставлять в шаблон
            Response::Power(power) => write!(f, "Power: {power}"),
            // Response:: Power (power) => write! (f, "Power: {}", power),
            Response::Unknown => write!(f, "Unknown"),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

// TODO: Здесь можно для красоты сделать документацию
pub struct SmartSocketClient {
    stream: TcpStream,
}

impl SmartSocketClient {
    // TODO: Для универсальности здесь можно принимать не String,
    // а аналогичный обобщенный тип `A: ToSocketAddrs`.
    // Тогда конструктор будет универсальнее.
    //
    // TODO: Здесь не обязательно лишний раз делать аллокации, можно было бы сразу
    // выдавать `std::io::Error` тип вместо общего `Box<dyn Error>`.
    //
    // TODO: Аналогично коду выше - здесь для полной красоты можно
    // было бы сделать еще и документацию красивую.
    pub fn new(server_address: String) -> Result<Self, Box<dyn Error>> {
        let stream = TcpStream::connect(server_address)?;
        Ok(Self { stream })
    }

    // TODO: Здесь не обязательно лишний раз делать аллокации, можно было бы сразу
    // выдавать `std::io::Error` тип вместо общего `Box<dyn Error>`.
    // Но еще лучше здесь сделать свой тип ошибки, который будет содержать как `std::io::Error`,
    // так и прочие ошибки логики работы.
    // 
    // TODO: Сокет может оказаться закрытым, поэтому здесь лучше бы возвращать, например,
    // `Option<Response>`, который будет отражать состояние закрытия клиента.
    // Сейчас закрытие сокета только как ошибка воспринимается, хотя это вполне штатная ситуация.
    // Но это чисто мысли. Сейчас тоже работает код.
    pub fn run_command(&mut self, command: Command) -> Result<Response, Box<dyn Error>> {
        // TODO: `.into()` здесь работает, но так как он является
        // не совсем явным в плане типа, то здесь для красоты можно было бы
        // создать отдельную переменную с явным типом и ее
        // передавать.
        // 
        // TODO: Можно было бы здесь использовать просто метод `.write()`,
        // так как все равно передаем один байт. А так же можно явно тогда
        self.stream.write_all(&[command.into()])?;

        let mut buffer = [0u8; 5];
        self.stream.read_exact(&mut buffer)?;
        
        Ok(buffer.into())
    }
}
