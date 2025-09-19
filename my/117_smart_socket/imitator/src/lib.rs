// TODO: Здесь можно было бы объединить в группу все импорты, которые
// связанные с std.
use std::io::{Read, Write};
use std::net::TcpListener;

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

////////////////////////////////////////////////////////////////////////////////

// TODO: Функция main должна располагаться в файлике `main.rs`,
// файлики `lib.rs` предназначены для библиотек, а не для бинарников.
fn main() {
    let mut args = std::env::args();
    args.next().unwrap();

    // TODO: Здесь для дополнительной валидации входного параметра
    // можно было бы парсить сначала строку в тип `std::net::SocketAddrV4`.
    //
    // Строка - это все-таки универсальный тип,
    // на вход может прилететь что угодно нам.
    let server_address = args.next().unwrap_or_else(|| "127.0.0.1:7890".into());

    let listener = TcpListener::bind(server_address).expect("can't bind tcp listener");

    // TODO: В клиенте тип назывался `SmartSocketClient`
    let mut smart_socket = SmartSocket::default();

    while let Some(connection) = listener.incoming().next() {
        let mut stream = match connection {
            Ok(conn) => conn,
            Err(err) => {
                println!("can't receive connection: {err}");
                continue;
            }
        };

        // TODO: Возможно, что здесь не обязательно было
        // бы конвртировать в строку и делать лишние аллокации
        // просто ради вывода строки.
        // Можно было бы с помощью match сделать разные выводы.
        let peer = stream
            .peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "unknown".into());
        println!("Peer '{peer}' connected");

        // TODO: Для поддержки сразу нескольких подключений, можно
        // было бы обернуть в тип розетки в `Arc<Mutex>`, а на каждое новое подключение
        // делать отдельный поток.
        // А если уж совсем делать красиво и поддержку очень большого
        // количества подключений, то нужно переходить на асинхронный код.

        // TODO: Входной буфер лишь для одного байта.
        // Можно было бы даже не делать массив для этих целей, а
        // вычитывать один байт с проверкой размера данных для отслеживания
        // закрытия сокета.
        let mut in_buffer = [0u8];
        while stream.read_exact(&mut in_buffer).is_ok() {
            // TODO: `.into()` здесь работает, но так как он является
            // не совсем явным в плане типа, то здесь для красоты можно было бы
            // создать отдельную переменную с явным типом и ее
            // передавать.
            let response = smart_socket.process_command(in_buffer[0].into());

            let response_buf: [u8; 5] = response.into();
            if stream.write_all(&response_buf).is_err() {
                break;
            }

            println!("Connection with {peer} lost. Waiting for new connections...");
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

// TODO: Здесь можно для красоты сделать документацию к enum +
// описание к каждому элементу, так как это публичный enum, да и в целом -
// это красиво будет.
#[derive(Default)]
struct SmartSocket {
    enabled: bool,
}

impl SmartSocket {
    fn process_command(&mut self, cmd: Command) -> Response {
        match cmd {
            Command::TurnOn => {
                self.enabled = true;
                Response::Ok
            }
            Command::Turnoff => {
                self.enabled = false;
                Response::Ok
            }
            Command::IsEnabled => {
                if self.enabled {
                    Response::Enabled
                } else {
                    Response::Disabled
                }
            }
            Command::GetPower => {
                if self.enabled {
                    Response::Power(220.5)
                } else {
                    Response::Power(0.0)
                }
            }
            Command::Unknown => {
                println!("Unknown command received");
                Response::Unknown
            }
        }
    }
}
