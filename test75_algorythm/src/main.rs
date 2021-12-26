use eyre::{ContextCompat, WrapErr};
use log::{debug, info, warn, LevelFilter};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use smallstr::SmallString;
use std::time::Instant;

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
    // Для тестовости делаем размерность возможных элементов не очень большой
    let mut bloom = bloomfilter::Bloom::new(16, 32);

    for i in 0..10_000_000_u64 {
        // Используем small string чтобы не аллоцировать временную переменную в куче
        type StringContainer = SmallString<[u8; 128]>;
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

    debug!("Chuckoo filter memory usage: {}", bytesize::ByteSize(cf.memory_usage() as u64));

    Ok(())
}

/// Алгоритм HyperLogLog позволяет найти количество уникальных элементов за счет расчета хешей
fn test_hyper_log_log() -> Result<(), eyre::Error> {
    use hyperloglogplus::{HyperLogLog, HyperLogLogPlus};
    use std::collections::hash_map::RandomState;

    // Реализация HyperLogLog plus
    let mut hllp = HyperLogLogPlus::<&str, RandomState>::new(16, RandomState::new())
        .map_err(|err| eyre::Error::msg(format!("Hyper log log create err: {}", err.to_string())))?;

    let begin_time = Instant::now();

    const ITER_COUNT: u64 = 10_000_000_u64;

    for _ in 0..ITER_COUNT {
        // Используем small string чтобы не аллоцировать временную переменную в куче
        type StringContainer = SmallString<[u8; 128]>;
        let rand_string: StringContainer = thread_rng().sample_iter(&Alphanumeric).take(64).map(char::from).collect();

        hllp.insert(rand_string.as_str());
    }

    let duration = Instant::now().saturating_duration_since(begin_time);

    debug!(
        "Hyper log log unique elements: {}/{}, calculate duration: {}mSec",
        hllp.count(),
        ITER_COUNT,
        duration.as_millis()
    );

    Ok(())
}

/// Алгоритм Aho-Corasick позволяет быстро искать подстроку в большом тексте
fn test_aho_corasick() -> Result<(), eyre::Error> {
    use aho_corasick::AhoCorasick;

    // Паттерны, которые мы ищем в тексте
    let patterns = &["apple", "maple", "Snapple"];

    // Текст в котором будем искать
    let test_text = "Nobody likes maple in their apple flavored Snapple.";

    // Создаем конечный автомат для поиска из списка паттернов
    let ac = AhoCorasick::new(patterns);

    // Находим все вхождения паттернов в тексте
    ac.find_iter(test_text).try_for_each(|mat| -> Result<(), eyre::Error> {
        let pat_indx = mat.pattern();
        let pattern = patterns.get(pat_indx).wrap_err(format!("Wrong pattern index {}", pat_indx))?;
        let begin = mat.start();
        let end = mat.end();
        debug!("Aho-Corasick found {}: [{}, {}]", pattern, begin, end);
        Ok(())
    })?;

    debug!("Aho-Corasick memory usage: {}", bytesize::ByteSize(ac.heap_bytes() as u64));

    Ok(())
}

fn execute_app() -> Result<(), eyre::Error> {
    // Настройка логирования на основании количества флагов verbose
    setup_logging().wrap_err("Logging setup")?;

    // TODO: MinHash

    // Фильтр блума
    test_bloom_filter().wrap_err("Bloom filter")?;

    // Аналог фильтраблума
    test_cuckoo_filter().wrap_err("Chuckoo filter")?;

    // Hyper Log Log для поиска количества уникальных элементов
    test_hyper_log_log().wrap_err("Hyper log log")?;

    // Aho-Corasick
    test_aho_corasick().wrap_err("Aho-Corasick")?;

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
