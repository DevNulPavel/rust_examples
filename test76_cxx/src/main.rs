use log::{debug, LevelFilter};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Настойка уровня логирования
fn setup_logging() -> Result<(), eyre::Error> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(LevelFilter::Trace)
        .try_init()?;
    Ok(())
}

#[cxx::bridge(namespace = "blobstore")]
mod ffi {
    // Общие структуры данных для C++ / Rust
    #[derive(Debug)]
    struct BlobMetadata {
        size: usize,
        tags: Vec<String>,
    }

    // Объявления, которые будут видны из C++ кода
    extern "Rust" {
        type MultiBuf;
        fn next_chunk(&mut self) -> &[u8];
    }

    // extern "C++" означает объявления, которые видны из Rust кода и могут быть там использованы
    unsafe extern "C++" {
        // Добавляем необходимые .h файлики
        // Важно! Путь указывается полный, включая имя нашего проекта в виде корня
        include!("test76_cxx/libs/cpp_test_lib/include/blobstore.h");

        // Тип с методами
        type BlobstoreClient;
        fn put(&self, parts: &mut MultiBuf) -> u64;
        fn tag(&self, blobid: u64, tag: &str);
        fn metadata(&self, blob_id: u64) -> BlobMetadata;

        // Билдер UniquePtr
        fn new_blobstore_client() -> UniquePtr<BlobstoreClient>;
    }
}

/////////////////////////////////////////////////////////////////////////////////////

// Rust класс, который будет доступен из C++
pub struct MultiBuf {
    chunks: Vec<Vec<u8>>,
    pos: usize,
}

impl MultiBuf {
    pub fn next_chunk(&mut self) -> &[u8] {
        let next = self.chunks.get(self.pos);
        self.pos += 1;
        next.map_or(&[], Vec::as_slice)
    }   
}

/////////////////////////////////////////////////////////////////////////////////////

fn execute_app() -> Result<(), eyre::Error> {
    let client = ffi::new_blobstore_client();

    // Upload a blob.
    let chunks = vec![b"fearless".to_vec(), b"concurrency".to_vec()];
    let mut buf = MultiBuf { chunks, pos: 0 };
    let blob_id = client.put(&mut buf);
    debug!("blobid = {}", blob_id);

    client.tag(blob_id, "test_blob_tag"); // TODO: Возвращать Result

    let meta = client.metadata(blob_id); // TODO: Возвращать Option
    debug!("Metadata: {:?}", meta);

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
