use ixy::{memory::Packet, *};
use simple_logger::SimpleLogger;
use std::{collections::VecDeque, env, ops::DerefMut, process, time::Instant};

//////////////////////////////////////////////////////////////////////////////////////////////////

const BATCH_SIZE: usize = 32;

//////////////////////////////////////////////////////////////////////////////////////////////////

pub fn main() {
    // Создаем тестовый логгер
    SimpleLogger::new().init().unwrap();

    // Перем параметры с которыми было запущего приложение
    let mut args = env::args();

    // Пропустим имя самого приложения
    args.next();

    // Получаем адрес первого устройства
    let pci_addr_1 = match args.next() {
        Some(arg) => arg,
        None => {
            eprintln!("Usage: cargo run --example echoer <pci bus id1> <pci bus id2>");
            process::exit(1);
        }
    };

    // Получаем адрес второго устройства
    let pci_addr_2 = match args.next() {
        Some(arg) => arg,
        None => {
            eprintln!("Usage: cargo run --example echoer <pci bus id1> <pci bus id2>");
            process::exit(1);
        }
    };

    // На основе этих самых адресов создаем девайсы
    let mut dev1 = ixy_init(&pci_addr_1, 1, 1, 0).unwrap();
    let mut dev2 = ixy_init(&pci_addr_2, 1, 1, 0).unwrap();

    // На девайсах делаем сброс статистиики
    dev1.reset_stats();
    dev2.reset_stats();

    // Создаем буферы для записи статистики
    let mut dev1_stats = Default::default();
    let mut dev1_stats_old = Default::default();
    let mut dev2_stats = Default::default();
    let mut dev2_stats_old = Default::default();

    // Вычитываем статистику
    dev1.read_stats(&mut dev1_stats);
    dev1.read_stats(&mut dev1_stats_old);
    dev2.read_stats(&mut dev2_stats);
    dev2.read_stats(&mut dev2_stats_old);

    // Создаем кольцевой буфер сразу с желаемой емкостью
    let mut buffer: VecDeque<Packet> = VecDeque::with_capacity(BATCH_SIZE);

    // Счетчик
    let mut counter = 0;

    // Время запуска
    let mut time = Instant::now();

    // Запусаем работу в цикле
    loop {
        // Делаем эхо входящих и исходящих пакетов для первого девайса
        echo(&mut buffer, dev1.deref_mut(), 0, 0);

        // Делаем эхо входящих и исходящих пакетов для второго девайса
        echo(&mut buffer, dev2.deref_mut(), 0, 0);

        // Если количество итераций кратно 0xfff
        if counter & 0xfff == 0 {
            // Делаем замер времени
            let elapsed = time.elapsed();

            // Считаем количество наносекунд
            let nanos: u64 = elapsed.as_secs() * 1_000_000_000 + u64::from(elapsed.subsec_nanos());

            // Если прошедшее время больше 1-й секунды, тогда
            if nanos > 1_000_000_000 {
                {
                    // Читаем статы по первому устройству
                    dev1.read_stats(&mut dev1_stats);
                    // Выводим разницу в статистике по сравнению с прошлым разом
                    dev1_stats.print_stats_diff(&dev1, &dev1_stats_old, nanos);
                    // Обновляем переменную со статами на новые статы
                    dev1_stats_old = dev1_stats;
                }

                {
                    // Читаем статы по второму устройству
                    dev2.read_stats(&mut dev2_stats);
                    // Выводим эти статы
                    dev2_stats.print_stats_diff(&dev2, &dev2_stats_old, nanos);
                    // Обновляем переменную со статами на новые статы
                    dev2_stats_old = dev2_stats;
                }

                // Обновляем время для повторной статистики
                time = Instant::now();
            }
        }

        // +1 к итерациям
        counter += 1;
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////

/// Специальная эхо-фунция, которая берет прилетевшие данные и отправляет
/// эти самые данные назад, лишь слегка модифицируя 48й байт пакетов
fn echo(buffer: &mut VecDeque<Packet>, dev: &mut dyn IxyDevice, rx_queue: u16, tx_queue: u16) {
    // Получаем какое-то количество пакетов
    // в кольцевой буфер из очереди сетевой карты
    let num_rx = dev.rx_batch(rx_queue, buffer, BATCH_SIZE);

    // Если у нас количество пакетов больше, чем 0
    if num_rx > 0 {
        // Для приличия сделаем модификацию прилетевших данных
        for p in buffer.iter_mut() {
            // Модифицируем 48 байт в пакете данных
            *p.get_mut(48).unwrap() += 1;
        }

        // Теперь мы можем те же самые данные в буфере отправить клиенту назад в тот же интерфейс
        dev.tx_batch(tx_queue, buffer);

        // Очищаем пакеты в буффере, если там какие-то остались еще неотправленные
        buffer.clear();

        // RangeFull
        // let range = ..;
        // buffer.drain(range);
    }
}
