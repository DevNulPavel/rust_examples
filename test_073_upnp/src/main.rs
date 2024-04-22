mod app_arguments;

use crate::app_arguments::AppArguments;
use eyre::{ContextCompat, WrapErr};
use igd::{search_gateway, SearchOptions};
use local_ip_address::list_afinet_netifas;
use log::{debug, LevelFilter};
use owo_colors::OwoColorize;
use std::{io::{stdin, stdout, Read, Write}, net::{IpAddr, SocketAddrV4}};
use smallvec::SmallVec;
use structopt::StructOpt;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging(arguments: &AppArguments) -> Result<(), eyre::Error> {
    // Настройка логирования на основании количества флагов verbose
    let level = match arguments.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        4 => LevelFilter::Trace,
        _ => {
            return Err(eyre::eyre!("Verbose level must be in [0, 4] range"));
        }
    };
    pretty_env_logger::formatted_timed_builder().filter_level(level).try_init()?;
    Ok(())
}

fn read_line_with_length_limit(max_text_length: usize) -> Result<String, eyre::Error> {
    let mut text = String::new();

    let stdin = stdin();
    let mut stdin_lock = stdin.lock();

    let mut buf = [0_u8; 8];
    loop {
        let count = stdin_lock.read(&mut buf)?;
        if count > 0 {
            let mut complete = false;
            let mut end = 0;

            for char in buf {
                // У нас символ переноса строки или максимальный лимит?
                if (char != b'\n') && ((text.len() + end) < max_text_length) {
                    end += 1;
                } else {
                    complete = true;
                    break;
                }
            }
            let buf_slice = buf.get(0..end).wrap_err("Invalid slice index")?;

            let substr = std::str::from_utf8(buf_slice).wrap_err("Not utf-8 chars")?;

            text.push_str(substr);

            if complete {
                break;
            }
        } else {
            break;
        }
    }

    Ok(text)
}

fn execute_app() -> Result<(), eyre::Error> {
    // Аргументы коммандной строки
    let arguments = app_arguments::AppArguments::from_args_safe().wrap_err("Arguments parsing")?;

    // Настройка логирования на основании количества флагов verbose
    setup_logging(&arguments).wrap_err("Logging setup")?;

    // Ищем шлюз в нашей вети
    let gateway = {
        let options = SearchOptions { ..Default::default() };
        search_gateway(options).wrap_err("Gateway search err")?
    };
    let external_ip = gateway.get_external_ip().wrap_err("External ip get err")?;
    println!("Found gateway: {}\nExternal ip: {}", gateway, external_ip);

    // Получаем список адресов V4
    // Количество элементов у нас мелкое, поэтому размещать будем в стековом контейнере
    let ipv4_interfaces: SmallVec<[_; 4]> = list_afinet_netifas()
        .wrap_err("Network interfaces receive error")?
        .into_iter()
        .filter_map(|(interface, ip_addr)| match ip_addr {
            IpAddr::V4(v4) => Some((interface, v4)),
            _ => None,
        })
        .collect();

    let local_ip = match ipv4_interfaces.len() {
        0 => {
            return Err(eyre::eyre!("Empty network ip V4 interfaces list"));
        }
        1 => ipv4_interfaces.first().wrap_err("Invalid interface count")?.1,
        _ => {
            // Выводим описание интерфейсов
            {
                let stdout = stdout();
                let mut stdout_lock = stdout.lock();
                writeln!(&mut stdout_lock, "Select interface")?;
                ipv4_interfaces.iter().enumerate().try_for_each(|(index, (interface, ip_addr))| {
                    writeln!(&mut stdout_lock, "{}: {} -> {}", index.green(), interface, ip_addr)
                })?;
            };

            loop {
                // Ограничиваем размер входного буффера??? Чтобы не перегрузить размер входного буфера
                // Парсим полученное значение
                let ip_addr = read_line_with_length_limit(16)?
                    .parse::<usize>()
                    .ok()
                    .and_then(|val| ipv4_interfaces.get(val))
                    .map(|val| val.1);

                match ip_addr {
                    Some(ip_addr) => {
                        break ip_addr;
                    }
                    _ => {
                        println!("Input value must be number from 0 to {}", ipv4_interfaces.len());
                    }
                }
            }
        }
    };
    debug!("Selected local ip: {}", local_ip);

    // Создаем адрес полноценный
    let protocol_type = igd::PortMappingProtocol::TCP;
    let external_port = 9999;
    let local_addr = SocketAddrV4::new(local_ip, 9999_u16);
    let duration_sec = 30;
    let descritption = "Add port example";

    // Добавляем на роутер проброс с порта
    match gateway.add_port(protocol_type, external_port, local_addr, duration_sec, descritption) {
        Err(err) => {
            println!("There was an error! {}", err);
        }
        Ok(()) => {
            println!("It worked");
        }
    }

    Ok(())
}

fn main() {
    // Настройка color eyre для ошибок
    color_eyre::install().expect("Error setup failed");

    // Запуск приложения
    if let Err(err) = execute_app() {
        // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
        eprint!("Error! Failed with: {:?}", err);
        std::process::exit(1);
    }
}
