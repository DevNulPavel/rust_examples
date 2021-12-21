use bytesize::ByteSize;
use eyre::WrapErr;
use log::{debug, info, warn, LevelFilter};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging() -> Result<(), eyre::Error> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(LevelFilter::Trace)
        .try_init()?;
    Ok(())
}

/// Фильтр блума
fn test_bloom_filter() -> Result<(), eyre::Error> {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use smallstr::SmallString;

    // Для тестовости делаем размерность возможных элементов не очень большой
    let mut bloom = bloomfilter::Bloom::new(16, 32);

    for i in 0..10_000_000_u64 {
        // Используем small string чтобы не аллоцировать временную переменную в куче
        type StringContainer = SmallString<[u8; 64]>;
        let rand_string: StringContainer = thread_rng().sample_iter(&Alphanumeric).take(64).map(char::from).collect();

        // Устанавливаем рандомное значение строки
        bloom.set(&rand_string);

        // Проверяем наличие элемента, может вернуть false в некоторых случаях, когда элемент там все-таки есть
        // То есть false positive
        if !bloom.check(&rand_string) {
            warn!("False-positive bloom filter at iteration: {}", i)
        }
    }

    info!("Bloom filter complete");

    Ok(())
}

/// В отличие от классического Bloom фильтра, данный фильтр позволяет так же удалять из общего set различные итемы.
/// Но это делается ценой сохранения fingerprint для ключа, позволяя как-то из конечного битового массива отнять значения.
/// Но само собой это досигается ценой более высокого потребления памяти, но не сильно
/// https://crates.io/crates/cuckoofilter
fn test_cuckoo_filter() -> Result<(), eyre::Error> {
    let value: &str = "hello world";

    // Создаем cuckoo фильтр со стандартной емкостью в 1000000 итемов
    let mut cf = cuckoofilter::CuckooFilter::new();

    // Добавляем в фильтр
    // Возвращает ошибку если элемент был добавлен, но тем самым перезатер уже имеющийся старый
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

    // TODO: Aho-Corasic
    // TODO: LogHashHash
    // TODO: MinHash

    // Фильтр блума
    test_bloom_filter().wrap_err("Bloom filter")?;

    // Аналог фильтраблума
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
