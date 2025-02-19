use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, process};

use byteorder::{WriteBytesExt, LE};
use ixy::memory::Packet;
use ixy::*;
use simple_logger::SimpleLogger;

const BATCH_SIZE: usize = 32;

pub fn main() -> Result<(), io::Error> {
    // Создаем тестовый логгер
    SimpleLogger::new().init().unwrap();

    // Перем параметры с которыми было запущего приложение + пропустим имя самого приложения
    let mut args = env::args().skip(1);

    // Первым параметром здесь принимаем непосредственно адрес устройства
    // Вторым параметром принимаем путь к выходному файкику
    let (pci_addr, output_file) = match (args.next(), args.next()) {
        (Some(pci_addr), Some(output_file)) => (pci_addr, output_file),
        _ => {
            eprintln!("Usage: cargo run --example pcap <pci bus id> <output file> [n packets]");
            process::exit(1);
        }
    };

    // Следующим параметром принимаем количество пакетов, которые мы хотим сохранить.
    // Данного значения у нас может и не быть, тогда мы спококойно пропускаем.
    let mut n_packets: Option<usize> = args
        .next()
        .map(|n| n.parse().expect("failed to parse n packets"));

    // Распечатаем в лог
    if let Some(n) = n_packets {
        println!("Capturing {} packets...", n);
    } else {
        println!("Capturing packets...");
    }

    // Создаем файлик, а так же создаем буфферизатор для снижения системных вызовов
    let mut pcap = BufWriter::new(File::create(output_file)?);

    // Записываем в файлик теперь непосредственно заголовок .pcap файлика
    pcap.write_u32::<LE>(0xa1b2_c3d4)?; // magic_number
    pcap.write_u16::<LE>(2)?; // version_major
    pcap.write_u16::<LE>(4)?; // version_minor
    pcap.write_i32::<LE>(0)?; // thiszone
    pcap.write_u32::<LE>(0)?; // sigfigs
    pcap.write_u32::<LE>(65535)?; // snaplen
    pcap.write_u32::<LE>(1)?; // network: Ethernet

    // Сбросим заголовок на диск
    pcap.flush()?;

    // На основе входного адреса создадим девайс
    let mut dev = ixy_init(&pci_addr, 1, 1, 0).unwrap();

    // Создаем буфер для пакетов сразу нужного размера
    let mut buffer: VecDeque<Packet> = VecDeque::with_capacity(BATCH_SIZE);

    // Читаем и записываем лишь нужное количество пакетов.
    // Либо вычитываем бесконечно, если лимита пакетов нету.
    while n_packets != Some(0) || n_packets.is_none() {
        // Делаем вычитывание пакетов
        let _read_count = dev.rx_batch(0, &mut buffer, BATCH_SIZE);

        // Timestamp время
        let time = {
            let time = SystemTime::now();
            time.duration_since(UNIX_EPOCH).unwrap()
        };

        // Разгребаем теперь
        for packet in buffer.drain(std::ops::RangeFull) {
            // Записываем в файлик pcap заголовок
            pcap.write_u32::<LE>(time.as_secs() as u32)?; // ts_sec
            pcap.write_u32::<LE>(time.subsec_millis())?; // ts_usec
            pcap.write_u32::<LE>(packet.len() as u32)?; // incl_len
            pcap.write_u32::<LE>(packet.len() as u32)?; // orig_len

            // Запишем теперь сам пакет туда
            pcap.write_all(packet.as_ref())?;

            // Отнимаем количество пакетов, если есть такое ограничение у нас
            n_packets = n_packets.map(|n| n - 1);

            // Если мы во внутреннем цикле дошли до нуля - прерыавем его тоже
            if n_packets == Some(0) {
                break;
            }
        }

        // Сбросим дополнительно на диск прилетевшие пакеты
        pcap.flush()?;
    }

    // Сбросим заголовок на данные остаточные
    pcap.flush()?;

    Ok(())
}
