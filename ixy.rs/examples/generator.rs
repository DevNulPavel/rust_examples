use byteorder::{ByteOrder, LittleEndian};
use ixy::{
    memory::{alloc_pkt_batch, Mempool, Packet},
    *,
};
use simple_logger::SimpleLogger;
use std::{collections::VecDeque, env, process, time::Instant};

//////////////////////////////////////////////////////////////////////////////////////////////////

// Количество пакетов, отправляемых параллельно драйвером
const BATCH_SIZE: usize = 32;

// Количество пакетов в пуле памяти
const NUM_PACKETS: usize = 2048;

// Размер наших пакетов
const PACKET_SIZE: usize = 60;

//////////////////////////////////////////////////////////////////////////////////////////////////

pub fn main() {
    // Создаем тестовый логгер
    SimpleLogger::new().init().unwrap();

    // Перем параметры с которыми было запущего приложение
    let mut args = env::args();

    // Пропустим имя самого приложения
    args.next();

    // Получаем адрес устройства
    let pci_addr = match args.next() {
        Some(arg) => arg,
        None => {
            eprintln!("Usage: cargo run --example generator <pci bus id>");
            process::exit(1);
        }
    };

    // На основе этих самых адресов создаем девайс
    let mut dev = ixy_init(&pci_addr, 1, 1, 0).unwrap();

    // Массив байт пакета
    // https://www.eit.lth.se/ppplab/IPHeader.htm
    // https://habr.com/ru/articles/413851/
    #[rustfmt::skip]
    let mut pkt_data = [
        // Часть Ethernet

        // Целевой MAC
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
        // Исходный MAC
        0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
        // Тип пакета: IPv4
        0x08, 0x00,

        // Часть IP

        // Version, размер заголовка, type of service для обнаружения заторов и отбросов
        0x45, 0x00,
        // Полная длина пакета без etherner заголовка, первый верхний байт
        ((PACKET_SIZE - 14) >> 8) as u8,
        // Полная длина пакета без etherner заголовка, второй нижний байт
        ((PACKET_SIZE - 14) & 0xFF) as u8,
        // Идентификатор, флаги, смещение фрагмента
        0x00, 0x00, 0x00, 0x00,
        // Значение TTL (64), протокол (UDP), контрольная сумма пакета
        0x40, 0x11, 0x00, 0x00,
        // Исходный адрес пакета ip (10.0.0.1)
        0x0A, 0x00, 0x00, 0x01,
        // Целевой адрес ip пакета (10.0.0.2)
        0x0A, 0x00, 0x00, 0x02,
        // Исходный и конечные порты у пакета (42 -> 1337)
        0x00, 0x2A, 0x05, 0x39,

        // UDP часть

        // Длина UDP, исключая размеры IP & ethernet, первый верхний байт
        ((PACKET_SIZE - 20 - 14) >> 8) as u8,
        // Длина UDP, исключая размеры IP & ethernet, второй младший байт
        ((PACKET_SIZE - 20 - 14) & 0xFF) as u8,
        // Контрольная сумма UDP, опциональный один байт
        0x00, 0x00,                                 
        // Какие-то данные в этом UDP пакете
        b'i', b'x', b'y'    
                                
        // Остаток данных является незаполненным, так как пулы памяти гарантируют пустые буфферы
    ];

    // Получаем ссылку на слайс исходного MAC адреса и записываем туда реальный MAC адрес
    // TODO: spoof check of PF
    {
        // Мутабельный слайс на MAC адрес
        let mac_slice_mut = unsafe { pkt_data.get_unchecked_mut(6..12) };
        // Получаем реальный MAC адрес
        let dev_mac_addr = dev.get_mac_addr();
        // Клонируем его в девайс
        mac_slice_mut.clone_from_slice(&dev_mac_addr);
    }

    // Создаем пул для пакетов нужного размера
    let pool = Mempool::allocate(NUM_PACKETS, 0).unwrap();

    // Предзаполняем буферы всех пакетов в пуле данными и возвращаем их в пулы
    {
        // Создаем буфер для пакетов пулы
        let mut buffer: VecDeque<Packet> = VecDeque::with_capacity(NUM_PACKETS);

        // TODO: ???
        // Аллоцируем сразу же большую кучу пакетов в буффере с использованием пула памяти
        alloc_pkt_batch(&pool, &mut buffer, NUM_PACKETS, PACKET_SIZE);

        // Для каждого пакеты в буффере
        for p in buffer.iter_mut() {
            // Перебираем байты исходного референсного пакета
            for (i, data) in pkt_data.iter().enumerate() {
                // Получаем мутабельную ссылку на байт пакета
                let target_byte = p.get_mut(i).unwrap();

                // Теперь записываем в целевой пакет нужный нам байт
                *target_byte = *data;
            }

            // Байты пакета от которых надо полчитать контрольную сумму
            let checksum_bytes = {
                let begin = 14;
                let end = begin + 20;
                p.get(begin..end).unwrap()
            };

            // Делаем расчет теперь котрольной суммы + конвертируем сразу в BigEndian формат
            let checksum = calc_ipv4_checksum(checksum_bytes).to_be_bytes();

            // Записываем контрольную сумму в пакет
            p.get_mut(24..=25).unwrap().copy_from_slice(&checksum);
        }
    }

    // Сбрасываем статистику на девайсе
    dev.reset_stats();

    // Создаем буферы для статистики
    let mut dev_stats = Default::default();
    let mut dev_stats_old = Default::default();

    // Сразу же вычитываем новую статистику
    dev.read_stats(&mut dev_stats);
    dev.read_stats(&mut dev_stats_old);

    // Создаем буферы и переменные
    let mut buffer: VecDeque<Packet> = VecDeque::with_capacity(BATCH_SIZE);
    let mut time = Instant::now();
    let mut seq_num = 0;
    let mut counter = 0;

    // Главный цикл
    loop {
        // Аллоцируем небольшую кучу пакетов в буффере с использованием пула памяти.
        // Пакетов берем из пула немного - 32 всего
        alloc_pkt_batch(&pool, &mut buffer, BATCH_SIZE, PACKET_SIZE);

        // Обновляем номер сообщения всех пакетов, которые аллоцировали пакетно,
        // а так же их контрольные суммы тоже.
        for p in buffer.iter_mut() {
            // TODO: Const
            let begin = PACKET_SIZE - 4;
            let end = PACKET_SIZE;

            // Мутабельный слайс
            let packet_bytes = p.get_mut(begin..end).unwrap();

            // Записываем туда номер
            LittleEndian::write_u32(packet_bytes, seq_num);

            // Увеличиваем номер
            seq_num = seq_num.wrapping_add(1);
        }

        // Дожидаемся возможности очередной записи данных и пишем пачку пакетов в девайс
        dev.tx_batch_busy_wait(0, &mut buffer);

        // Если количество итераций кратно 0xfff
        if counter & 0xfff == 0 {
            // Делаем замер времени
            let elapsed = time.elapsed();
            // Считаем количество наносекунд
            let nanos = elapsed.as_secs() * 1_000_000_000 + u64::from(elapsed.subsec_nanos());

            // Если прошедшее время больше 1-й секунды, тогда
            if nanos > 1_000_000_000 {
                // Читаем статы
                dev.read_stats(&mut dev_stats);

                // Выводим разницу в статистике по сравнению с прошлым разом
                dev_stats.print_stats_diff(&*dev, &dev_stats_old, nanos);

                // Обновляем переменную со статами на новые статы
                dev_stats_old = dev_stats;

                // Начинаем снова замеры
                time = Instant::now();
            }
        }

        counter += 1;
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// Calculates IPv4 header checksum
fn calc_ipv4_checksum(ipv4_header: &[u8]) -> u16 {
    assert_eq!(ipv4_header.len() % 2, 0);
    let mut checksum = 0;
    for i in 0..ipv4_header.len() / 2 {
        if i == 5 {
            // Assume checksum field is set to 0
            continue;
        }
        checksum += (u32::from(ipv4_header[i * 2]) << 8) + u32::from(ipv4_header[i * 2 + 1]);
        if checksum > 0xffff {
            checksum = (checksum & 0xffff) + 1;
        }
    }
    !(checksum as u16)
}

//////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_checksum() {
        // Test case from the Wikipedia article "IPv4 header checksum"
        assert_eq!(
            calc_ipv4_checksum(
                b"\x45\x00\x00\x73\x00\x00\x40\x00\x40\x11\xb8\x61\xc0\xa8\x00\x01\xc0\xa8\x00\xc7"
            ),
            0xb861
        );
    }
}
