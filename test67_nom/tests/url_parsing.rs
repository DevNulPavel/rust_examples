use test67_nom::url_parse::parse_url;

use std::process::exit;

fn execute_app() -> Result<(), eyre::Error> {
    // hex_color_example::test_parse_hex_color()?;

    Ok(())
}

fn main() {
    // Делаем так, чтобы ошибки захватывали backtrace
    color_eyre::install().expect("Backtraces error backtraces setup disabled");

    // Настройка логирования
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    // Запуск приложения c обработкой ошибки
    if let Err(err) = execute_app() {
        eprintln!("{:?}", err);
        exit(1);
    }
}
