use eyre::{Context, ContextCompat};
use log::{debug, info, LevelFilter};
use pnet::{
    
    datalink::Channel::Ethernet,
    datalink::{self, NetworkInterface},
    packet::ethernet::{EthernetPacket, MutableEthernetPacket},
    packet::{MutablePacket, Packet},
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging() -> Result<(), eyre::Error> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(LevelFilter::Trace)
        .try_init()?;
    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////

fn execute_app() -> Result<(), eyre::Error> {
    // Ищем конкретный интерфейс с конкретным именем
    const INTERFACE_NAME: &str = "en0";
    let interface = datalink::interfaces()
        .into_iter()
        .find(|iface: &NetworkInterface| iface.name == INTERFACE_NAME)
        .wrap_err_with(|| format!("There is no interface: {}", INTERFACE_NAME))?;
    info!("Interface found: {:?}", interface);

    // Создаем новый канал уровня канала для отслеживания пакетов
    let datalink_config = datalink::Config { ..Default::default() };
    let (mut tx, mut rx) = match datalink::channel(&interface, datalink_config)
        .wrap_err("An error occurred when creating the datalink channel")?
    {
        Ethernet(tx, rx) => (tx, rx),
        _ => eyre::bail!("Unhandled channel type"),
    };

    loop {
        // Новые прилетевшие данные на интерфейсе
        let packet_data = rx.next().wrap_err("Invalid packet")?;
        // Парсим данные в виде пакета
        let packet = EthernetPacket::new(packet_data).wrap_err("Is not ethernet packet")?;
        info!("Parsed packet: {:?}", packet);
    }

    Ok(())
}

fn main() {
    // Настройка color eyre для ошибок
    color_eyre::install().expect("Error setup failed");

    // Настройка логирования на основании количества флагов verbose
    setup_logging().expect("Logging setup");

    // Запуск приложения
    if let Err(err) = execute_app() {
        // При ошибке не паникуем, а спокойно выводим сообщение и завершаем приложение с кодом ошибки
        eprint!("Error! Failed with: {:?}", err);
        std::process::exit(1);
    }
}
