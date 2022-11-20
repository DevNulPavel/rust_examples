//! `tiny-lsm` is a dead-simple in-memory LSM for managing
//! fixed-size metadata in more complex systems.
//!
//! Uses crc32fast to checksum all key-value pairs in the log and
//! sstables. Uses zstd to compress all sstables. Performs sstable
//! compaction in the background.
//!
//! Because the data is in-memory, there is no need to put bloom
//! filters on the sstables, and read operations cannot fail due
//! to IO issues.
//!
//! `Lsm` implements `Deref<Target=BTreeMap<[u8; K], [u8; V]>>`
//! to immutably access the data directly without any IO or
//! blocking.
//!
//! `Lsm::insert` writes all data into a 32-kb `BufWriter`
//! in front of a log file, so it will block for very
//! short periods of time here and there. SST compaction
//! is handled completely in the background.
//!
//! This is a bad choice for large data sets if you
//! require quick recovery time because it needs to read all of
//! the sstables and the write ahead log when starting up.
//!
//! The benefit to using tiered sstables at all, despite being
//! in-memory, is that they act as an effective log-deduplication
//! mechanism, keeping space amplification very low.
//!
//! Maximum throughput is not the goal of this project. Low space
//! amplification and very simple code is the goal, because this
//! is intended to maintain metadata in more complex systems.
//!
//! There is currently no compaction throttling. You can play
//! with the `Config` options around compaction to change compaction
//! characteristics.
//!
//! Never change the constant size of keys or values for an existing
//! database.
//!
//! # Examples
//!
//! ```
//! // open up the LSM
//! let mut lsm = tiny_lsm::Lsm::recover("path/to/base/dir").expect("recover lsm");
//!
//! // store some things
//! let key: [u8; 8] = 8_u64.to_le_bytes();
//! let value: [u8; 1] = 255_u8.to_le_bytes();
//! lsm.insert(key, value);
//!
//! assert_eq!(lsm.get(&key), Some(&value));
//!
//! ```
#![cfg_attr(test, feature(no_coverage))]

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, prelude::*, BufReader, BufWriter, Result};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc, Arc,
};

//////////////////////////////////////////////////////////////////////////////////////////////////////

const SSTABLE_DIR: &str = "sstables";
const U64_SZ: usize = std::mem::size_of::<u64>();

//////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    test,
    derive(serde::Serialize, serde::Deserialize, fuzzcheck::DefaultMutator)
)]
pub struct Config {
    /// Если расжатые на диске sstable превышают использование оперативки на данную пропорцию,
    /// то полная упаковка всех sstable будет выполнена.
    /// Это, скорее всего, произойдет в ситуациях, когда множество версий ключей существуют во множестве sstable.
    /// Но не должно происходить когда лишь новые ключи записываются в базу.
    pub max_space_amp: u8,
    /// Когда лог-файлик превышает данный размер, то новый сжатый и упакованный sstable будет сброшен на диск и
    /// лог-файлик будет сброшен.
    pub max_log_length: usize,
    /// Когда фоновый поток упаковщика смотрит на непрерывный диапазон sstable для мержа, то это потребует,
    /// чтобы все sstables были как минимум 1/`merge_ratio` * размер первой sstable в
    /// прилегающем окне рассмотрения.
    pub merge_ratio: u8,
    /// Когда фоновый упаковщик смотрит на диапазон sstable для мержа, то
    /// это потребует диапазоны длиной как-минимум данного значения.
    pub merge_window: u8,
    /// Все добавления напрямую в `BufWriter`, оборачивающий log-файлик. Данная опция определяет
    /// на сколько большим будет данный буффер.
    pub log_bufwriter_size: u32,
    /// Размер компрессии для zstd и sstables.
    pub zstd_sstable_compression_level: u8,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            max_space_amp: 2,
            max_log_length: 32 * 1024 * 1024,
            merge_ratio: 3,
            merge_window: 10,
            log_bufwriter_size: 32 * 1024,
            zstd_sstable_compression_level: 3,
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

struct WorkerStats {
    read_bytes: AtomicU64,
    written_bytes: AtomicU64,
}

#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub resident_bytes: u64,
    pub on_disk_bytes: u64,
    pub logged_bytes: u64,
    pub written_bytes: u64,
    pub read_bytes: u64,
    pub space_amp: f64,
    pub write_amp: f64,
}

/// Подсчет хеша от ключа и значения.
/// В качестве константных значений размера массива используем шаблонные параметры.
fn hash<const K: usize, const V: usize>(k: &[u8; K], v: &Option<[u8; V]>) -> u32 {
    // Создаем хешер
    let mut hasher = crc32fast::Hasher::new();
    // Пишем флаг наличия значения
    hasher.update(&[v.is_some() as u8]);
    // Пишем ключ
    hasher.update(k);

    // Пишем само значение если оно есть, либо пишем просто нули размерностью ключа
    if let Some(v) = v {
        hasher.update(v);
    } else {
        hasher.update(&[0; V]);
    }

    // we XOR the hash to make sure it's something other than 0 when empty,
    // because 0 is an easy value to create accidentally or via corruption.

    // Ксорим значение, чтобы убедиться, что это как-то отличается от 0 когда пусто.
    // Так как 0 - это значение можно легко случайно получить при повреждении базы и тд.
    // TODO: Смысл не очень понятен, 0 все еще можно получить с той же вероятность (вроде бы)
    hasher.finalize() ^ 0xFF
}

/// Тоже хеширование, но просто длины
#[inline]
fn hash_batch_len(len: usize) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&(len as u64).to_le_bytes());

    hasher.finalize() ^ 0xFF
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

/// Сообщение воркера
enum WorkerMessage {
    /// Создаем новый sorted string table
    NewSST { id: u64, sst_sz: u64, db_sz: u64 },

    /// Останавливаем воркер
    Stop(mpsc::Sender<()>),

    /// Пингуем воркер
    Heartbeat(mpsc::Sender<()>),
}

/// Структура с информацией о воркере
struct Worker<const K: usize, const V: usize> {
    sstable_directory: BTreeMap<u64, u64>,
    /// Канал для коммуникации с внешним миром
    inbox: mpsc::Receiver<WorkerMessage>,
    /// Размер базы
    db_sz: u64,
    /// Путь к базе
    path: PathBuf,
    // Конфиг непосредственно базы
    config: Config,
    /// Статистика работы
    stats: Arc<WorkerStats>,
}

impl<const K: usize, const V: usize> Worker<K, V> {
    #[cfg(not(test))]
    fn run(mut self) {
        while self.tick() {}
        log::info!("tiny-lsm compaction worker quitting");
    }

    fn tick(&mut self) -> bool {
        // Получаем сообщение из внешнего канала
        match self.inbox.recv() {
            Ok(message) => {
                if !self.handle_message(message) {
                    return false;
                }
            }
            Err(mpsc::RecvError) => {
                return false;
            }
        }

        // Сжимаем лишь один раз перед проверкой новых сообщений
        if let Err(e) = self.sstable_maintenance() {
            log::error!(
                "error while compacting sstables \
                in the background: {:?}",
                e
            );
        }

        true
    }

    /// Обработка сообщения входящего
    fn handle_message(&mut self, message: WorkerMessage) -> bool {
        match message {
            WorkerMessage::NewSST { id, sst_sz, db_sz } => {
                // Записываем новый размер базы,
                self.db_sz = db_sz;
                // Пишем в дереве в оперативке id и размер
                self.sstable_directory.insert(id, sst_sz);
                true
            }
            WorkerMessage::Stop(dropper) => {
                // Уничтожаем канал, тем самым сбрасывая блокировку ожидания снаружи
                drop(dropper);
                false
            }
            WorkerMessage::Heartbeat(dropper) => {
                drop(dropper);
                true
            }
        }
    }

    // Обработка всех sstable
    fn sstable_maintenance(&mut self) -> Result<()> {
        // Суммируем значения размеров всех sstable, получая размер данных на диске
        let on_disk_size: u64 = self.sstable_directory.values().sum();

        // Размер данных на диске
        log::debug!("disk size: {} mem size: {}", on_disk_size, self.db_sz);

        // Если у нас больше одной sstable + размер на диске, деленный на размер базы? больше максимального размера
        if self.sstable_directory.len() > 1
            && on_disk_size / (self.db_sz + 1) > self.config.max_space_amp as u64
        {
            // Тогда выполняем полную упаковку, разжатый размер на диске будет увеличен на определенное количество байт
            log::debug!(
                "performing full compaction, decompressed on-disk \
                database size has grown beyond {}x the in-memory size",
                self.config.max_space_amp
            );

            // Собираем ключи из дерева в кучу
            let run_to_compact: Vec<u64> = self.sstable_directory.keys().copied().collect();

            // Запускаем упаковку
            self.compact_sstable_run(&run_to_compact)?;

            return Ok(());
        }

        // Если размер упаковки пока не очень большой, то все норм
        if self.sstable_directory.len() < self.config.merge_window.max(2) as usize {
            return Ok(());
        }

        // Для индекса делаем обход, собирая все в вектор
        for window in self
            .sstable_directory
            .iter()
            .collect::<Vec<_>>()
            .windows(self.config.merge_window.max(2) as usize)
        {
            if window
                .iter()
                .skip(1)
                .all(|w| *w.1 * self.config.merge_ratio as u64 > *window[0].1)
            {
                let run_to_compact: Vec<u64> = window.into_iter().map(|(id, _sum)| **id).collect();

                self.compact_sstable_run(&run_to_compact)?;
                return Ok(());
            }
        }

        Ok(())
    }

    /// Данная функция может падать в любом месте без оставления системы в невосстанавливаемом состоянии.
    /// Данные так же не теряются. Данная функция должна быть независимая от внешнего API в перспективе.
    fn compact_sstable_run(&mut self, sstable_ids: &[u64]) -> Result<()> {
        // Выводим в лог попытку упаковать id
        log::debug!(
            "trying to compact sstable_ids {:?}",
            sstable_ids
                .iter()
                .map(|id| id_format(*id))
                .collect::<Vec<_>>()
        );

        // Создаем временную мапу для значений
        let mut map = BTreeMap::new();

        // Сколько прочитали пар
        let mut read_pairs = 0;

        // Читаем id sstable
        for sstable_id in sstable_ids {
            // Читаем файлик sstable с диска
            for (k, v) in read_sstable::<K, V>(&self.path, *sstable_id)? {
                // Пишем в мапу в оперативке + увеличиваем счетчик
                map.insert(k, v);
                read_pairs += 1;
            }
        }

        // Записываем счетчик прочитанного размера байт
        self.stats
            .read_bytes
            .fetch_add(read_pairs * (4 + 1 + K + V) as u64, Ordering::Relaxed);

        // Находим максимальный id в sstable
        let sst_id = sstable_ids
            .iter()
            .max()
            .expect("compact_sstable_run called with empty set of sst ids");

        // Пишем новую sstable из мапы на диск
        write_sstable(&self.path, *sst_id, &map, true, &self.config)?;

        // Пишем статистику в метрики
        self.stats
            .written_bytes
            .fetch_add(map.len() as u64 * (4 + 1 + K + V) as u64, Ordering::Relaxed);

        // Теперть это общая компактная sstable, можем указать в мапе лишь ее
        let sst_sz = map.len() as u64 * (4 + K + V) as u64;
        self.sstable_directory.insert(*sst_id, sst_sz);

        log::debug!("compacted range into sstable {}", id_format(*sst_id));

        // Теперь идем по всем id
        for sstable_id in sstable_ids {
            // Если это новая таблица - пропускаем
            if sstable_id == sst_id {
                continue;
            }
            // Удаляем старые файлики таблиц
            fs::remove_file(self.path.join(SSTABLE_DIR).join(id_format(*sstable_id)))?;
            // Аналогично - удаляем из индекс-дерева id
            self.sstable_directory
                .remove(sstable_id)
                .expect("compacted sst not present in sstable_directory");
        }

        // Открываем директорию как файл и делаем fsync
        fs::File::open(self.path.join(SSTABLE_DIR))?.sync_all()?;

        Ok(())
    }
}

/// Форматируем id как строку
fn id_format(id: u64) -> String {
    format!("{:016x}", id)
}

/// Получаем список sstable в директории
fn list_sstables(path: &Path, remove_tmp: bool) -> Result<BTreeMap<u64, u64>> {
    // Создаем мапу
    let mut sstable_map = BTreeMap::new();

    // Обходим файлики в директории
    for dir_entry_res in fs::read_dir(path.join(SSTABLE_DIR))? {
        let dir_entry = dir_entry_res?;
        let file_name = if let Ok(f) = dir_entry.file_name().into_string() {
            f
        } else {
            continue;
        };

        // Конвертируем строковое представление числа в u64 в 16тиричной системе
        if let Ok(id) = u64::from_str_radix(&file_name, 16) {
            let metadata = dir_entry.metadata()?;

            sstable_map.insert(id, metadata.len());
        } else {
            // Если это какой-то временный файлик, тогда удаляем
            if remove_tmp && file_name.ends_with("-tmp") {
                log::warn!("removing incomplete sstable rewrite {}", file_name);
                fs::remove_file(path.join(SSTABLE_DIR).join(file_name))?;
            }
        }
    }

    Ok(sstable_map)
}

/// Пишем sstable на диск
fn write_sstable<const K: usize, const V: usize>(
    path: &Path,
    id: u64,
    items: &BTreeMap<[u8; K], Option<[u8; V]>>,
    tmp_mv: bool,
    config: &Config,
) -> Result<()> {
    let sst_dir_path = path.join(SSTABLE_DIR);

    // Временный ли файлик?
    let sst_path = if tmp_mv {
        sst_dir_path.join(format!("{:x}-tmp", id))
    } else {
        sst_dir_path.join(id_format(id))
    };

    // Открываем файлик на запись
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&sst_path)?;

    // Уровень сжатия
    let max_zstd_level = zstd::compression_level_range();
    let zstd_level = config
        .zstd_sstable_compression_level
        .min(*max_zstd_level.end() as u8);

    // Создаем буфферизированного писателя с поддержкой сжатия для записи в файлик
    let mut bw =
        BufWriter::new(zstd::Encoder::new(file, zstd_level as _).expect("zstd encoder failure"));

    // Пишем благополучно количество элементов на диск с помощью этого вызова
    bw.write_all(&(items.len() as u64).to_le_bytes())?;

    // Теперь пишем каждый элемент
    for (k, v) in items {
        // Делаем хеш
        let crc: u32 = hash(k, v);
        // Пишем хеш
        bw.write_all(&crc.to_le_bytes())?;
        // Пишем флаг значения
        bw.write_all(&[v.is_some() as u8])?;
        // Пишем ключ
        bw.write_all(k)?;

        // Пишем само значение или нули
        if let Some(v) = v {
            bw.write_all(v)?;
        } else {
            bw.write_all(&[0; V])?;
        }
    }

    // Скидываем все на диск
    bw.flush()?;

    // Делаем fsync
    bw.get_mut().get_mut().sync_all()?;

    // Делаем синхронизацию для директории
    fs::File::open(path.join(SSTABLE_DIR))?.sync_all()?;

    // Если у нас это был временный файлик, перемещаем в постоянный
    if tmp_mv {
        let new_path = sst_dir_path.join(id_format(id));
        fs::rename(sst_path, new_path)?;
    }

    Ok(())
}

/// Читаем SSTable из файлика
fn read_sstable<const K: usize, const V: usize>(
    path: &Path,
    id: u64,
) -> Result<Vec<([u8; K], Option<[u8; V]>)>> {
    // Открываем файлик на чтение
    let file = fs::OpenOptions::new()
        .read(true)
        .open(path.join(SSTABLE_DIR).join(id_format(id)))?;

    // Создаем ридер на чтение сжатых данных, внутренний буффер - 16 мегабайт?
    let mut reader = zstd::Decoder::new(BufReader::with_capacity(16 * 1024 * 1024, file)).unwrap();

    // Читаем значение количества элементов
    let len_buf = &mut [0; 8];
    reader.read_exact(len_buf)?;
    let expected_len: u64 = u64::from_le_bytes(*len_buf);

    // Создаем вектор для чтения нужного количества элементов sstable
    let mut sstable = Vec::with_capacity(expected_len as usize);

    // Создаем локальный буффер для чтения значения
    // crc + tombstone discriminant + key + value
    let mut buf = vec![0; 4 + 1 + K + V];

    // Читаем каждое значеение отдельно в буффер из файлика
    while let Ok(()) = reader.read_exact(&mut buf) {
        // Вычитываем значение контрольной суммы
        let crc_expected: u32 = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        // Получаем флаг удаления значения
        let d: bool = match buf[4] {
            0 => false,
            1 => true,
            _ => {
                log::warn!("detected torn-write while reading sstable {:016x}", id);
                break;
            }
        };
        // Читаем ключ
        let k: [u8; K] = buf[5..K + 5].try_into().unwrap();
        // Читаем сами данные если они есть
        let v: Option<[u8; V]> = if d {
            Some(buf[K + 5..5 + K + V].try_into().unwrap())
        } else {
            None
        };

        // Пеересчитываем значения хеш-суммы
        let crc_actual: u32 = hash(&k, &v);

        // Делаем сверку
        if crc_expected != crc_actual {
            log::warn!("detected torn-write while reading sstable {:016x}", id);
            break;
        }

        // Сохраняем результат
        sstable.push((k, v));
    }

    // Проверим количество элементов в списке
    if sstable.len() as u64 != expected_len {
        log::warn!(
            "sstable {:016x} tear detected - process probably crashed \
            before full sstable could be written out",
            id
        );
    }

    Ok(sstable)
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Lsm<const K: usize, const V: usize> {
    // `BufWriter` flushes on drop
    // TODO: Непосредственно само дерево значений?
    // Судя по коду - это отдельное дерево с лог-файликом для непосредственной работы в реальном времени
    memtable: BTreeMap<[u8; K], Option<[u8; V]>>,
    // TODO: ???
    // Вроде как похоже на ту же самую базу в оперативной памяти, которая восстановлена с диска
    db: BTreeMap<[u8; K], [u8; V]>,
    /// Канал для отправки значений воркеру
    worker_outbox: mpsc::Sender<WorkerMessage>,
    // Следующее значение id для следующей таблицы, значения просто увеличиваются постоянно
    next_sstable_id: u64,
    // TODO: ???
    dirty_bytes: usize,

    // TODO:
    /// Сам воркер
    #[cfg(test)]
    worker: Worker<K, V>,

    // TODO: Лог-файлик?
    #[cfg(test)]
    pub log: tearable::Tearable<fs::File>,

    /// Обычный лог файлик
    #[cfg(not(test))]
    log: BufWriter<fs::File>,

    path: PathBuf,
    config: Config,
    stats: Stats,
    worker_stats: Arc<WorkerStats>,
}

impl<const K: usize, const V: usize> Drop for Lsm<K, V> {
    fn drop(&mut self) {
        // Канал ожидания завершения работы воркера
        let (tx, rx) = mpsc::channel();

        // Отсылаем событие завершения работы воркера
        if self.worker_outbox.send(WorkerMessage::Stop(tx)).is_err() {
            log::error!("failed to shut down compaction worker on Lsm drop");
            return;
        }

        // Ассерт, который срабатывает лишь при тестах
        // Воркер уже должен быть завершен
        #[cfg(test)]
        assert!(!self.worker.tick());

        // Ждем завершения воркера
        for _ in rx {}
    }
}

impl<const K: usize, const V: usize> std::ops::Deref for Lsm<K, V> {
    type Target = BTreeMap<[u8; K], [u8; V]>;

    /// Возаращаем фактическое дерево значений для LSM на чтение
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl<const K: usize, const V: usize> Lsm<K, V> {
    /// Восстанавливаем LSM дерево с диска. Лучше удостовериться сначала, что мы не восстанавливаем данные
    /// с диска с другими параметрами, иначе возможна потеря данных.
    ///
    /// Это действие происходит за O(N) и включает в себя чтение всех предыдущих записанных sstable и лог-файлика
    /// для восстановления всех данных в in-memory BTreeMap.
    pub fn recover<P: AsRef<Path>>(p: P) -> Result<Lsm<K, V>> {
        Lsm::recover_with_config(p, Config::default())
    }

    /// Восстановим LSM дерево с кастомным конфигов. Значения конфига можно менять все, кроме размера ключа и значения.
    pub fn recover_with_config<P: AsRef<Path>>(p: P, config: Config) -> Result<Lsm<K, V>> {
        let path = p.as_ref();
        if !path.exists() {
            // Создаем директории все нужные
            fs::create_dir_all(path)?;
            fs::create_dir(path.join(SSTABLE_DIR))?;
            fs::File::open(path.join(SSTABLE_DIR))?.sync_all()?;
            fs::File::open(path)?.sync_all()?;

            let mut parent_opt = path.parent();

            // Нам нужно рекурсивно сделать fsync для родителей всех данной директории из-за того
            // что мы используем create_dir_all
            while let Some(parent) = parent_opt {
                if parent.file_name().is_none() {
                    break;
                }
                if fs::File::open(parent).and_then(|f| f.sync_all()).is_err() {
                    // we made a reasonable attempt, but permissions
                    // can sometimes get in the way, and at this point it's
                    // becoming pedantic.
                    break;
                }
                parent_opt = parent.parent();
            }
        }

        // Получаем список id LSM файликов в директории
        let sstable_directory = list_sstables(path, true)?;

        // Создаем дерево базы
        let mut db = BTreeMap::new();

        // Читаем каждый файлик, расширяя базу
        for sstable_id in sstable_directory.keys() {
            let table = read_sstable::<K, V>(path, *sstable_id);
            for (k, v) in table? {
                if let Some(v) = v {
                    db.insert(k, v);
                } else {
                    db.remove(&k);
                }
            }
        }

        // Получаем максимальный id
        let max_sstable_id = sstable_directory.keys().next_back().copied();

        // Создаем log файлик, либо открываем имеющийся уже файлик
        let log = fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path.join("log"))?;

        // Читаем этот самый лог-файлик
        let mut reader = BufReader::new(log);

        // Буфферы для чтения значений из этого самого файлика
        let tuple_sz = U64_SZ.max(K + V);
        let header_sz = 5;
        let header_tuple_sz = header_sz + tuple_sz;
        let mut buf = vec![0; header_tuple_sz];

        // Создаем другое дерево из LSM лога
        let mut memtable = BTreeMap::new();
        let mut recovered = 0;

        // write_batch is the pending memtable updates, the number
        // of remaining items in the write batch, and the number of
        // bytes that have been recovered in the write batch.

        // write_batch - это ожидающие обновления memtable, количество оставшихся элементов в
        // группе записи.
        // Количество байт, которые были восстановлены.
        let mut write_batch: Option<(_, usize, u64)> = None;

        // Читаем буффер из лог-файлика
        while let Ok(()) = reader.read_exact(&mut buf) {
            // Считаем хеш
            let crc_expected: u32 = u32::from_le_bytes(buf[0..4].try_into().unwrap());
            // Удаленное ли это значение или групповое значение
            let d: bool = match buf[4] {
                0 => false,
                1 => true,
                2 if write_batch.is_none() => {
                    // begin batch
                    let batch_sz_buf: [u8; 8] = buf[5..5 + 8].try_into().unwrap();
                    let batch_sz: u64 = u64::from_le_bytes(batch_sz_buf);
                    log::debug!("processing batch of len {}", batch_sz);

                    let crc_actual = hash_batch_len(usize::try_from(batch_sz).unwrap());
                    if crc_expected != crc_actual {
                        log::warn!("crc mismatch for batch size marker");
                        break;
                    }

                    if !buf[5 + U64_SZ..].iter().all(|e| *e == 0) {
                        log::warn!(
                            "expected all pad bytes after logged \
                            batch manifests to be zero, but some \
                            corruption was detected"
                        );
                        break;
                    }

                    if batch_sz > usize::MAX as u64 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "recovering a batch size over usize::MAX is not supported",
                        ));
                    }

                    let wb_remaining = batch_sz as usize;
                    let wb_recovered = buf.len() as u64;

                    if wb_remaining > 0 {
                        write_batch = Some((
                            Vec::with_capacity(batch_sz as usize),
                            wb_remaining,
                            wb_recovered,
                        ));
                    } else {
                        recovered += buf.len() as u64;
                    }

                    continue;
                }
                _ => {
                    log::warn!("invalid log message discriminant detected: {}", buf[4]);
                    break;
                }
            };

            // Читаем ключ
            let k: [u8; K] = buf[5..5 + K].try_into().unwrap();

            // Читаем значение, если оно еще есть
            let v: Option<[u8; V]> = if d {
                Some(buf[5 + K..5 + K + V].try_into().unwrap())
            } else {
                None
            };

            let crc_actual: u32 = hash(&k, &v);

            if crc_expected != crc_actual {
                log::warn!(
                    "crc mismatch for kv pair {:?}-{:?}: expected {} actual {}, torn log detected",
                    k,
                    v,
                    crc_expected,
                    crc_actual
                );
                break;
            }

            let pad_start = if v.is_some() { 5 + K + V } else { 5 + K };

            if !buf[pad_start..].iter().all(|e| *e == 0) {
                log::warn!(
                    "expected all pad bytes for logged kv entries \
                    to be zero, but some corruption was detected"
                );
                break;
            }

            if let Some((mut wb, mut wb_remaining, mut wb_recovered)) = write_batch.take() {
                wb.push((k, v));
                wb_remaining = wb_remaining.checked_sub(1).unwrap();
                wb_recovered = wb_recovered.checked_add(buf.len() as u64).unwrap();

                // apply the write batch all at once
                // or never at all
                if wb_remaining == 0 {
                    for (k, v) in wb {
                        memtable.insert(k, v);

                        if let Some(v) = v {
                            db.insert(k, v);
                        } else {
                            db.remove(&k);
                        }
                    }
                    recovered += wb_recovered;
                } else {
                    write_batch = Some((wb, wb_remaining, wb_recovered));
                }
            } else {
                memtable.insert(k, v);

                if let Some(v) = v {
                    db.insert(k, v);
                } else {
                    db.remove(&k);
                }

                recovered += buf.len() as u64;
            }
        }

        // need to back up a few bytes to chop off the torn log
        log::debug!("recovered {} kv pairs", db.len());
        log::debug!("rewinding log down to length {}", recovered);
        let log_file = reader.get_mut();
        log_file.seek(io::SeekFrom::Start(recovered))?;
        log_file.set_len(recovered)?;
        log_file.sync_all()?;
        fs::File::open(path.join(SSTABLE_DIR))?.sync_all()?;

        let (tx, rx) = mpsc::channel();

        let worker_stats = Arc::new(WorkerStats {
            read_bytes: 0.into(),
            written_bytes: 0.into(),
        });

        // Создаем воркер
        let worker: Worker<K, V> = Worker {
            path: path.to_owned(),
            sstable_directory,
            inbox: rx,
            db_sz: db.len() as u64 * (K + V) as u64,
            config,
            stats: worker_stats.clone(),
        };

        // Воркера запускаем в работу
        #[cfg(not(test))]
        std::thread::spawn(move || worker.run());

        let (hb_tx, hb_rx) = mpsc::channel();
        tx.send(WorkerMessage::Heartbeat(hb_tx)).unwrap();

        #[cfg(test)]
        let mut worker = worker;

        #[cfg(test)]
        assert!(worker.tick());

        for _ in hb_rx {}

        let lsm = Lsm {
            #[cfg(not(test))]
            log: BufWriter::with_capacity(config.log_bufwriter_size as usize, reader.into_inner()),
            #[cfg(test)]
            log: tearable::Tearable::new(reader.into_inner()),
            #[cfg(test)]
            worker,
            path: path.into(),
            next_sstable_id: max_sstable_id.unwrap_or(0) + 1,
            dirty_bytes: recovered as usize,
            worker_outbox: tx,
            config,
            stats: Stats {
                logged_bytes: recovered,
                on_disk_bytes: 0,
                read_bytes: 0,
                written_bytes: 0,
                resident_bytes: db.len() as u64 * (K + V) as u64,
                space_amp: 0.,
                write_amp: 0.,
            },
            worker_stats,
            db,
            memtable,
        };

        Ok(lsm)
    }

    /// Пишем KV в LSM, возвращая предыдущее значение если оно было.
    /// Данная операция может быть заблокирована на доволньо короткий момент времени, так
    /// как 32х килобайтный буффер из `BufWriter` может сбрасывать данные на диск.
    ///
    /// Если требуется блокировка до тех пор пока все данные не будут записаны, можно использовать
    /// вызов `Lsm::flush`.
    pub fn insert(&mut self, k: [u8; K], v: [u8; V]) -> Result<Option<[u8; V]>> {
        // Дописываем в лог файлик
        self.log_mutation(k, Some(v))?;

        // Сбрасываем на диск при достижении размера лога
        if self.dirty_bytes > self.config.max_log_length {
            self.flush()?;
        }

        // Пишем в основную базу тоже
        Ok(self.db.insert(k, v))
    }

    /// Removes a KV pair from the `Lsm`, returning the
    /// previous value if it existed. This operation might
    /// involve blocking for a very brief moment as a 32kb
    /// `BufWriter` wrapping the log file is flushed.
    ///
    /// If you require blocking until all written data is
    /// durable, use the `Lsm::flush` method below.
    pub fn remove(&mut self, k: &[u8; K]) -> Result<Option<[u8; V]>> {
        self.log_mutation(*k, None)?;

        if self.dirty_bytes > self.config.max_log_length {
            self.flush()?;
        }

        Ok(self.db.remove(k))
    }

    /// Apply a set of updates to the `Lsm` and
    /// log them to disk in a way that will
    /// be recovered only if every update is
    /// present.
    pub fn write_batch(&mut self, write_batch: &[([u8; K], Option<[u8; V]>)]) -> Result<()> {
        let batch_len: [u8; 8] = (write_batch.len() as u64).to_le_bytes();
        let crc = hash_batch_len(write_batch.len());

        self.log.write_all(&crc.to_le_bytes())?;
        self.log.write_all(&[2_u8])?;
        self.log.write_all(&batch_len)?;

        // the zero pad is necessary because every log
        // entry must have the same length, whether
        // it's a batch size or actual kv tuple.
        let tuple_sz = U64_SZ.max(K + V);
        let pad_sz = tuple_sz - U64_SZ;
        let pad = [0; U64_SZ];
        self.log.write_all(&pad[..pad_sz])?;

        for (k, v_opt) in write_batch {
            if let Some(v) = v_opt {
                self.db.insert(*k, *v);
            } else {
                self.db.remove(k);
            }

            self.log_mutation(*k, *v_opt)?;
            self.memtable.insert(*k, *v_opt);
        }

        if self.dirty_bytes > self.config.max_log_length {
            self.flush()?;
        }

        Ok(())
    }

    /// Модификация лог-файлика, записываем новые значения в файлик + обновляем дерево в оперативке на новое значение.
    fn log_mutation(&mut self, k: [u8; K], v: Option<[u8; V]>) -> Result<()> {
        let crc: u32 = hash(&k, &v);
        self.log.write_all(&crc.to_le_bytes())?;
        self.log.write_all(&[v.is_some() as u8])?;
        self.log.write_all(&k)?;

        if let Some(v) = v {
            self.log.write_all(&v)?;
        } else {
            self.log.write_all(&[0; V])?;
        };

        // the zero pad is necessary because every log
        // entry must have the same length, whether
        // it's a batch size or actual kv tuple.
        let min_tuple_sz = U64_SZ.max(K + V);
        let pad_sz = min_tuple_sz - (K + V);
        let pad = [0; U64_SZ];
        self.log.write_all(&pad[..pad_sz])?;

        let logged_bytes = 4 + 1 + min_tuple_sz;

        self.memtable.insert(k, v);

        self.dirty_bytes += logged_bytes;
        self.stats.logged_bytes += logged_bytes as u64;
        self.stats.written_bytes += logged_bytes as u64;

        Ok(())
    }

    /// Блокируемся, пока все данные из лог-файлика не будут записаны на диск и синхронизованы.
    /// Если лог-файлик вырос до определенного лимита, он будет сжат в новую sstable, а лог файлик будет урезан
    /// после записи таблицы новой
    pub fn flush(&mut self) -> Result<()> {
        #[cfg(test)]
        {
            if self.log.tearing {
                return Ok(());
            }
        }

        // Сброс самого файлика на диск
        self.log.flush()?;
        self.log.get_mut().sync_all()?;

        // Превышен ли лимит байтов?
        if self.dirty_bytes > self.config.max_log_length {
            log::debug!("compacting log to sstable");

            // Обнуляем дерево данного класса
            let memtable = std::mem::take(&mut self.memtable);

            // Пишем на диск дерево из оперативки с новым id
            let sst_id = self.next_sstable_id;
            if let Err(e) = write_sstable(&self.path, sst_id, &memtable, false, &self.config) {
                // При ошибке - возвращаем назад дерево в оперативку
                self.memtable = memtable;
                log::error!("failed to flush lsm log to sstable: {:?}", e);
                return Err(e);
            }

            let sst_sz = 8 + (memtable.len() as u64 * (4 + K + V) as u64);
            let db_sz = self.db.len() as u64 * (K + V) as u64;

            // Отсылаем воркеру сообщение о создании новой таблицы со строками
            // В фоне будут мержиться данные таблицы в общую кучу
            if let Err(e) = self.worker_outbox.send(WorkerMessage::NewSST {
                id: sst_id,
                sst_sz,
                db_sz,
            }) {
                log::error!("failed to send message to worker: {:?}", e);
                log::logger().flush();
                panic!("failed to send message to worker: {:?}", e);
            }

            #[cfg(test)]
            assert!(self.worker.tick());

            // Увеличиваем значение id следующей таблицы
            self.next_sstable_id += 1;

            // Лог-файлик обрезаем до нулевого размера
            let log_file: &mut fs::File = self.log.get_mut();
            log_file.seek(io::SeekFrom::Start(0))?;
            log_file.set_len(0)?;
            log_file.sync_all()?;
            fs::File::open(self.path.join(SSTABLE_DIR))?.sync_all()?;

            self.dirty_bytes = 0;
        }

        Ok(())
    }

    pub fn stats(&mut self) -> Result<Stats> {
        self.stats.written_bytes += self.worker_stats.written_bytes.swap(0, Ordering::Relaxed);
        self.stats.read_bytes += self.worker_stats.read_bytes.swap(0, Ordering::Relaxed);
        self.stats.resident_bytes = self.db.len() as u64 * (K + V) as u64;

        let mut on_disk_bytes: u64 = std::fs::metadata(self.path.join("log"))?.len();

        on_disk_bytes += list_sstables(&self.path, false)?
            .into_iter()
            .map(|(_, len)| len)
            .sum::<u64>();

        self.stats.on_disk_bytes = on_disk_bytes;

        self.stats.write_amp =
            self.stats.written_bytes as f64 / self.stats.on_disk_bytes.max(1) as f64;
        self.stats.space_amp =
            self.stats.on_disk_bytes as f64 / self.stats.resident_bytes.max(1) as f64;
        Ok(self.stats)
    }
}

#[cfg(test)]
mod tearable;

#[cfg(test)]
mod fuzz;
