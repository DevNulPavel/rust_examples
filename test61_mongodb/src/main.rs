use eyre::WrapErr;
use tracing::{debug, error};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

fn initialize_logs() -> Result<(), eyre::Error> {
    use tracing_subscriber::prelude::*;

    /*let level = match arguments.verbose {
        0 => tracing::Level::ERROR,
        1 => tracing::Level::WARN,
        2 => tracing::Level::INFO,
        3 => tracing::Level::DEBUG,
        4 => tracing::Level::TRACE,
        _ => {
            panic!("Verbose level must be in [0, 4] range");
        }
    };
    // Фильтрация на основе настроек
    let filter = tracing_subscriber::filter::LevelFilter::from_level(level);*/

    // Фильтрация на основе окружения
    let filter = tracing_subscriber::filter::EnvFilter::from_default_env();

    // Логи в stdout
    let stdoud_sub = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    // Error layer для формирования слоя ошибки по запросу
    let error_layer = tracing_error::ErrorLayer::default();

    // Специальный слой для отладочной консоли tokio
    // Используем стандартные настройки для фильтрации из переменной RUST_LOG
    // let console_layer = console_subscriber::ConsoleLayer::builder().with_default_env().spawn();

    // Суммарный обработчик
    let full_subscriber = tracing_subscriber::registry()
        // .with(console_layer)
        .with(filter)
        .with(error_layer)
        .with(stdoud_sub);

    // Враппер для библиотеки log
    tracing_log::LogTracer::init().wrap_err("Log wrapper create failed")?;

    // Установка по-умолчанию
    tracing::subscriber::set_global_default(full_subscriber).wrap_err("Global subscriber set failed")?;

    Ok(())
}

async fn run_app() -> Result<(), eyre::Error> {
    Ok(())
}

fn main() {
    // Бектрейсы в ошибках
    color_eyre::install().expect("Color eyre initialize failed");

    // Логи
    initialize_logs().expect("Logs init");

    // Создаем рантайм для работы
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Tokio runtime build");

    // Стартуем сервер
    runtime.block_on(run_app()).expect("Server running fail");
}
