use eyre::WrapErr;
use log::{debug, LevelFilter};
use bytesize::ByteSize;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging() -> Result<(), eyre::Error> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(LevelFilter::Trace)
        .try_init()?;
    Ok(())
}

/// В отличие от классического Bloom фильтра, данный фильтр позволяет так же удалять из общего set различные итемы.
fn test_cuckoo_filter() -> Result<(), eyre::Error> {
    let value: &str = "hello world";

    // Создаем cuckoo фильтр со стандартной емкостью в 1000000 итемов
    let mut cf = cuckoofilter::CuckooFilter::new();

    // Добавляем в фильтр
    cf.add(value).wrap_err("Add failed")?;

    // Делаем проверку, то элемент у нас уже есть в фильтре
    eyre::ensure!(cf.contains(value), "Must contains value");

    // Проверяем наличие элемента, если элемента такого нету, то добавляем его
    eyre::ensure!(!cf.test_and_add(value).wrap_err("Test and add failed")?, "Data must be exist");

    // Удаляем данные из нашего фильтра
    eyre::ensure!(cf.delete(value), "Data does not exist");

    debug!("Chuckoo filter memory usage: {}", ByteSize(cf.memory_usage() as u64));

    Ok(())
}

fn execute_app() -> Result<(), eyre::Error> {
    // Настройка логирования на основании количества флагов verbose
    setup_logging().wrap_err("Logging setup")?;

    test_cuckoo_filter().wrap_err("Chuckoo filter")?;

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
