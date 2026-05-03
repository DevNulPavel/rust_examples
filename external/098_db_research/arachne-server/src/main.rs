mod args;
mod paged_array;
mod stripped_lock;
mod timer_wheel;

////////////////////////////////////////////////////////////////////////////////

use arachne_parser::ast::Expr;
use autometrics::autometrics;
use chrono::Utc;
use clap::Parser;
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use fjall::{
    CompressionType, Database, KeyspaceCreateOptions,
    config::{BlockSizePolicy, CompressionPolicy},
};
use futures::{future, prelude::*, stream::FuturesUnordered};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
    },
    time::Instant,
};
use tarpc::{
    server::{Channel, Config},
    tokio_util::{codec::LengthDelimitedCodec, sync::CancellationToken},
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    args::Args, paged_array::PagedArray, stripped_lock::StripedLock, timer_wheel::HierarchicalWheel,
};
use arachne_api::{
    BitcodeCodec, PlatformCreateStatus, PlatformDeleteStatus, Rpc, SelectorCreateStatus,
    SelectorDeleteStatus, SelectorSize, UserAddStatus, UserDeleteStatus, UserId, UserPayload,
    UserReleaseStatus, UserUpdateStatus, rpc_dto,
};

////////////////////////////////////////////////////////////////////////////////

const CONCURRENCY_COUNT: usize = 10_000;

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct RamState {
    /// Платформы
    pub platforms: DashMap<u8, Arc<PlatformState>>,

    /// Задачи для full-scan для селектора
    pub backfill_queue: SegQueue<(u8, u8, u32, u32)>, // (platform_id, selector_id, start_id, end_id)

    /// Если сканирование не идет, уведомляем через notfiy о начале сканирования
    pub backfill_notify: tokio::sync::Notify,
}

impl RamState {
    /// Снапшот на диск состояния
    pub fn snapshot(&self, db_path: &str) -> anyhow::Result<()> {
        std::fs::create_dir_all(db_path)?;

        let instant = Instant::now();

        // Используем потоки для параллельного сохранения платформ на диск
        std::thread::scope(|s| {
            for p_entry in self.platforms.iter() {
                s.spawn(move || {
                    // Получаем идентификатор платформы
                    let platform_id = *p_entry.key();

                    // Сам Arc на платформу
                    let platform = p_entry.value();

                    // TODO: Учитывая, что у нас .unwrap() здесь,
                    // то при нехватке памяти или невозможности переименования
                    // у нас просто упадет хранилище и все
                    //
                    // Локальная функция для атомарной записи.
                    let write_atomic = |file_path: String, data: &[u8]| {
                        let tmp_path = format!("{}.tmp", file_path);
                        std::fs::write(&tmp_path, data).unwrap();
                        std::fs::rename(&tmp_path, &file_path).unwrap();
                    };

                    // TODO: Запись в несколько файликов в пределах одной платформы уже не является надежным
                    // способом и атомарным, тем более, что у нас еще и паника может быть
                    // при записи файлика в одном из потоков

                    // TODO: Получаем идентификатор какой-то максимальный
                    let max_id = platform.next_internal_id.load(Ordering::Relaxed);

                    // Дамп свободных ID
                    {
                        // TODO: ???
                        // Дамп свободных ID
                        let approx_len = platform.free_internal_ids.len();

                        // TODO: ???
                        let mut free_ids = Vec::with_capacity(approx_len);
                        for _ in 0..approx_len {
                            if let Some(id) = platform.free_internal_ids.pop() {
                                free_ids.push(id);
                            }
                        }

                        let free_bytes = unsafe {
                            std::slice::from_raw_parts(
                                free_ids.as_ptr() as *const u8,
                                free_ids.len() * 4,
                            )
                        };

                        write_atomic(format!("{db_path}/p_{platform_id}_free.bin"), free_bytes);
                    }

                    // 1. Дамп версий (Zero-copy memcpy)
                    {
                        // 1. Дамп версий (Zero-copy memcpy)
                        let versions_dump = platform.versions.dump(max_id);
                        let bytes = unsafe {
                            std::slice::from_raw_parts(
                                versions_dump.as_ptr() as *const u8,
                                versions_dump.len() * 4,
                            )
                        };

                        write_atomic(format!("{db_path}/p_{platform_id}_versions.bin"), bytes);
                        write_atomic(
                            format!("{db_path}/p_{platform_id}_max_id.bin"),
                            &max_id.to_be_bytes(),
                        );
                    }

                    // 2. Дамп очередей селекторов
                    for s_entry in platform.selectors.iter() {
                        // Получаем идентификатор селектора и само его значение
                        let selector_id = *s_entry.key();
                        let selector = s_entry.value();

                        // TODO: Дамп не потоковый, а в оперативку, что не очень
                        //
                        // Массив queued_versions уже хранит точную информацию о том,
                        // кто находится в очереди.
                        let q_versions = selector.queued_versions.dump(max_id);

                        let approx_size = selector.exact_size.load(Ordering::Relaxed);

                        // Используем [u32; 2] вместо (u32, u32).
                        // Массивы имеют жестко гарантированный layout в памяти, в отличие от кортежей.
                        let mut pairs: Vec<[u32; 2]> = Vec::with_capacity(approx_size);

                        // Фильтруем тех, чья версия в очереди > 0
                        for (task_id, &q_version) in q_versions.iter().enumerate() {
                            if q_version > 0 {
                                pairs.push([task_id as u32, q_version]);
                            }
                        }

                        // Сохраняем массив [u32; 2] в файл за ОДИН сисколл (Zero-copy запись)
                        let pairs_bytes = unsafe {
                            std::slice::from_raw_parts(
                                pairs.as_ptr() as *const u8,
                                pairs.len() * 8, // 8 байт на пару [id, version]
                            )
                        };

                        write_atomic(
                            format!("{db_path}/p_{platform_id}_s_{selector_id}_q.bin"),
                            pairs_bytes,
                        );

                        // 3. Дамп таймеров
                        let timers = selector.timer_wheel.drain();
                        let timers_bytes = rmp_serde::to_vec(&timers).unwrap();
                        write_atomic(
                            format!("{db_path}/p_{platform_id}_s_{selector_id}_timers.bin"),
                            &timers_bytes,
                        );

                        let is_bf = selector.is_backfilling.load(Ordering::Relaxed);
                        write_atomic(
                            format!("{db_path}/p_{platform_id}_s_{selector_id}_bf.bin"),
                            &[is_bf as u8],
                        );
                    }
                });
            }
        });

        tracing::info!("Dump completed at {:?}", instant.elapsed());

        Ok(())
    }

    /// Восстанавливает ОЗУ из дампа
    pub fn recover(db: &Database, db_path: String) -> anyhow::Result<Self> {
        let instant = Instant::now();

        let state = Self::default();
        let now = current_ts();

        // Получаем из базы список платформ
        let platforms = db.keyspace("platforms", db_options)?;

        // Перебираем теперь платформы
        for kv_res in platforms.iter() {
            let (k, _) = kv_res.into_inner()?;
            let platform_id = k[0];

            let id_db = db.keyspace(&format!("p_{platform_id}_ids"), db_options)?;
            let payload_db = db.keyspace(&format!("p_{platform_id}_payloads"), db_options)?;

            // Чтение пула свободных ID
            let free_internal_ids = SegQueue::new();
            if let Ok(free_bytes) = std::fs::read(format!("{db_path}/p_{platform_id}_free.bin")) {
                for chunk in free_bytes.chunks_exact(4) {
                    free_internal_ids.push(u32::from_ne_bytes(chunk.try_into().unwrap()));
                }
            }

            let mut p_state = PlatformState {
                versions: PagedArray::default(),
                next_internal_id: AtomicU32::new(0),
                selectors: DashMap::new(),
                id_db,
                payload_db,
                free_internal_ids,
                mutation_lock: StripedLock::new(CONCURRENCY_COUNT),
            };

            // Чтение версий и max_id
            let max_id = if let Ok(max_id_bytes) =
                std::fs::read(format!("{db_path}/p_{platform_id}_max_id.bin"))
            {
                let m = u32::from_be_bytes(max_id_bytes.try_into().unwrap());
                p_state.next_internal_id.store(m, Ordering::SeqCst);
                m
            } else {
                0
            };

            if let Ok(versions_bytes) =
                std::fs::read(format!("{db_path}/p_{platform_id}_versions.bin"))
            {
                // Итерация по `chunks_exact` векторизуется компилятором и работает без оверхеда.
                let mut versions_dump = Vec::with_capacity(versions_bytes.len() / 4);
                for chunk in versions_bytes.chunks_exact(4) {
                    versions_dump.push(u32::from_ne_bytes(chunk.try_into().unwrap()));
                }

                p_state.versions = PagedArray::restore(&versions_dump);
            }

            let selectors_ks = db.keyspace(&format!("p_{platform_id}_selectors"), db_options)?;

            // Запускаем пулл стандартных потоков для параллельной обработки очередей
            let mut join_handles = Vec::new();

            for sel_res in selectors_ks.iter() {
                let (sel_k, sel_v) = sel_res.into_inner()?;
                let selector_id = sel_k[0];

                let (query, lock_duration_sec): (String, u64) = rmp_serde::from_slice(&sel_v)?;
                let ast = arachne_parser::parse_query(&query).unwrap();

                let db_path = db_path.clone();
                // Парралельное восстановление селекторов
                let handle = std::thread::spawn(move || {
                    let mut selector = SelectorState {
                        ready_ids: SegQueue::new(),
                        queued_versions: PagedArray::default(),
                        exact_size: std::sync::atomic::AtomicUsize::new(0),
                        prefetch_queue: SegQueue::new(),
                        timer_wheel: Arc::new(Default::default()),
                        ast: Arc::new(*ast),
                        lock_duration_sec,
                        is_backfilling: AtomicBool::new(true),
                    };

                    // Мгновенная преаллокация теневого массива (исключает сотни миллионов проверок is_null)
                    selector.queued_versions.preallocate_up_to(max_id);

                    // Чтение очереди
                    let q_file_path = format!("{db_path}/p_{platform_id}_s_{selector_id}_q.bin");
                    if let Ok(bytes) = std::fs::read(&q_file_path) {
                        // Безопасное чтение старого формата (8 байт на элемент: 4 байта task_id, 4 байта q_version).
                        let chunks = bytes.chunks_exact(8);
                        selector.exact_size.store(chunks.len(), Ordering::SeqCst);

                        for chunk in chunks {
                            let task_id = u32::from_ne_bytes(chunk[0..4].try_into().unwrap());
                            let q_version = u32::from_ne_bytes(chunk[4..8].try_into().unwrap());

                            selector.ready_ids.push(task_id);
                            // Из-за preallocate_up_to get() работает без аллокаций
                            selector
                                .queued_versions
                                .get(task_id)
                                .store(q_version, Ordering::Relaxed);
                        }
                    }

                    // Восстанавливаем таймеры
                    if let Ok(timers_bytes) = std::fs::read(format!(
                        "{db_path}/p_{platform_id}_s_{selector_id}_timers.bin"
                    )) && let Ok(timers) = rmp_serde::from_slice(&timers_bytes)
                    {
                        selector.timer_wheel = Arc::new(HierarchicalWheel::restore(timers, now));
                    }

                    let bf_file = format!("{db_path}/p_{platform_id}_s_{selector_id}_bf.bin");
                    // Если файла нет (старый дамп или краш до записи), считаем что сканирование НЕ было завершено (true). Это безопасно.
                    let was_backfilling = std::fs::read(&bf_file)
                        .map(|b| b.first() == Some(&1))
                        .unwrap_or(true);

                    (selector_id, selector, was_backfilling)
                });

                join_handles.push(handle);
            }

            // Узнаем старый max_id из дампа памяти
            let snapshot_max_id = if let Ok(max_id_bytes) =
                std::fs::read(format!("{db_path}/p_{platform_id}_max_id.bin"))
            {
                u32::from_be_bytes(max_id_bytes.try_into().unwrap())
            } else {
                0
            };

            // Узнаем max_id из бд
            let payload_db = db.keyspace(&format!("p_{platform_id}_payloads"), db_options)?;
            let mut actual_max_id = snapshot_max_id;
            if let Some(Ok((k, _))) = payload_db.iter().next_back().map(|v| v.into_inner()) {
                let db_max = u32::from_be_bytes(k[..4].try_into().unwrap());
                actual_max_id = std::cmp::max(snapshot_max_id, db_max + 1);
            }
            p_state
                .next_internal_id
                .store(actual_max_id, Ordering::SeqCst);

            // Дожидаемся загрузки всех селекторов платформы
            for handle in join_handles {
                let (selector_id, selector, was_backfilling) = handle.join().unwrap();

                if was_backfilling {
                    // Селектор не успел досканировать данные до дампа!
                    // Делаем полное сканирование от 0 до актуального конца базы.
                    state
                        .backfill_queue
                        .push((platform_id, selector_id, 0, actual_max_id));
                    selector.is_backfilling.store(true, Ordering::Release);
                } else if actual_max_id > snapshot_max_id {
                    // Селектор давно готов, но в БД могли добавиться новые юзеры
                    // Сканируем только разницу
                    state.backfill_queue.push((
                        platform_id,
                        selector_id,
                        snapshot_max_id,
                        actual_max_id,
                    ));
                    selector.is_backfilling.store(true, Ordering::Release);
                } else {
                    // Селектор готов, новых данных нет
                    selector.is_backfilling.store(false, Ordering::Release);
                }

                p_state.selectors.insert(selector_id, Arc::new(selector));
            }

            state.platforms.insert(platform_id, Arc::new(p_state));
        }

        tracing::info!("Recovered from dump at {:?}", instant.elapsed());
        Ok(state)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct PlatformState {
    /// Версии (эпохи) пользователей
    pub versions: PagedArray,
    /// Следующий внутренний id для присвоения
    pub next_internal_id: AtomicU32,
    /// Свободные внутренние id
    pub free_internal_ids: SegQueue<u32>,
    /// Список селекторов платформы
    pub selectors: DashMap<u8, Arc<SelectorState>>,
    /// Шардированная блокировка для пользователей
    pub mutation_lock: StripedLock,

    pub id_db: fjall::Keyspace,
    pub payload_db: fjall::Keyspace,
}

impl PlatformState {
    #[inline]
    pub fn allocate_internal_id(&self) -> u32 {
        while let Some(id) = self.free_internal_ids.pop() {
            // сверяемся с БД
            if !self
                .payload_db
                .contains_key(id.to_be_bytes())
                .unwrap_or(true)
            {
                return id;
            }
        }
        // Если очередь пуста, берем новый
        self.next_internal_id.fetch_add(1, Ordering::SeqCst)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct SelectorState {
    /// Хранит id, готовые к выдаче в селекторе (может содержать инвалидированные id)
    pub ready_ids: SegQueue<u32>,
    /// Теневой массив: хранит версию пользователя, с которой он лежит в очереди
    pub queued_versions: PagedArray,
    /// Точный размер очереди селектора
    pub exact_size: AtomicUsize,
    /// Предзагруженные данные, готовые к выдаче
    pub prefetch_queue: SegQueue<(u32, u32, UserPayload)>,
    /// AST дерево запроса селектора
    pub ast: Arc<Expr>,
    /// Колесо времени
    pub timer_wheel: Arc<HierarchicalWheel>,
    /// Время блокировки юзера в селекторе
    pub lock_duration_sec: u64,
    /// Идет ли сейчас сканирование базы для этого селектора?
    pub is_backfilling: AtomicBool,
}

////////////////////////////////////////////////////////////////////////////////

#[inline]
fn dispatch_user_to_selector(
    selector: &SelectorState,
    internal_id: u32,
    version: u32,
    payload: &UserPayload,
    now: u64,
) {
    // Выполняем проверку соответствия указанного пользователя селектору
    let eval = arachne_parser::evaluator::Evaluator::evaluate(&selector.ast, &payload.meta, now);

    // TODO: А проверка is_match тут нужна?
    // Если селектор подходит и время в будущем, добавляем в колесо времени
    if let Some(wake_ts) = eval.wake_up_at
        && wake_ts > now
    {
        selector.timer_wheel.insert(
            now,
            crate::timer_wheel::TimerEntry {
                user_id: internal_id,
                version,
                trigger_ts: wake_ts,
                is_lock_timeout: false,
            },
        );
    }

    // Очередь
    let q_version_ptr = selector.queued_versions.get(internal_id);

    // Если юзер подходит
    if eval.is_match {
        // Получим текущую версию эпохи
        let mut current = q_version_ptr.load(Ordering::Acquire);

        loop {
            // Если в очереди уже лежит более новая или актуальная версия - не затираем её
            if current >= version {
                break;
            }

            // Если нет, кладем в очередь
            match q_version_ptr.compare_exchange_weak(
                current,
                version,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    if current == 0 {
                        selector.exact_size.fetch_add(1, Ordering::Relaxed);
                        selector.ready_ids.push(internal_id);
                    }
                    break;
                }
                Err(x) => current = x,
            }
        }
    } else {
        let mut current = q_version_ptr.load(Ordering::Acquire);
        loop {
            // Если очередь пуста или там лежит более свежая версия (которая, возможно, уже true) - не трогаем
            if current == 0 || current > version {
                break;
            }

            match q_version_ptr.compare_exchange_weak(
                current,
                0,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    selector.exact_size.fetch_sub(1, Ordering::Relaxed);
                    break;
                }
                Err(x) => current = x,
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
struct Server {
    /// База данных (lsm tree)
    db: Database,
    /// Состояние
    state: Arc<RamState>,
    /// Путь к бд
    db_path: Arc<String>,
}

impl Rpc for Server {
    #[autometrics]
    async fn platform_create(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
    ) -> Result<PlatformCreateStatus, String> {
        let platforms = self.db.keyspace("platforms", db_options).unwrap();
        if platforms
            .get([platform_id])
            .map_err(|e| e.to_string())?
            .is_some()
        {
            Ok(PlatformCreateStatus::Exists)
        } else {
            let id_db = self
                .db
                .keyspace(&format!("p_{platform_id}_ids"), db_options)
                .unwrap();
            let payload_db = self
                .db
                .keyspace(&format!("p_{platform_id}_payloads"), db_options)
                .unwrap();
            platforms
                .insert([platform_id], vec![])
                .map_err(|e| e.to_string())?;
            self.state.platforms.insert(
                platform_id,
                Arc::new(PlatformState {
                    versions: PagedArray::default(),
                    next_internal_id: AtomicU32::new(0),
                    selectors: DashMap::new(),
                    id_db,
                    payload_db,
                    free_internal_ids: SegQueue::new(),
                    mutation_lock: StripedLock::new(CONCURRENCY_COUNT),
                }),
            );
            Ok(PlatformCreateStatus::Created)
        }
    }

    #[autometrics]
    async fn platform_delete(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
    ) -> Result<PlatformDeleteStatus, String> {
        // Достаем платформу из RAM и сразу освобождаем DashMap
        if let Some((_, platform)) = self.state.platforms.remove(&platform_id) {
            let db = self.db.clone();

            // Выносим тяжелые операции в spawn_blocking
            tokio::task::spawn_blocking(move || -> Result<(), String> {
                let platforms = db
                    .keyspace("platforms", db_options)
                    .map_err(|e| e.to_string())?;
                let _ = platforms.remove([platform_id]);

                // Полная очистка БД
                if let Ok(id_db) = db.keyspace(&format!("p_{platform_id}_ids"), db_options) {
                    let _ = id_db.clear();
                }
                if let Ok(payload_db) =
                    db.keyspace(&format!("p_{platform_id}_payloads"), db_options)
                {
                    let _ = payload_db.clear();
                }
                if let Ok(selectors_ks) =
                    db.keyspace(&format!("p_{platform_id}_selectors"), db_options)
                {
                    let _ = selectors_ks.clear();
                }

                // Полная зачистка бинарных дампов на файловой системе
                let _ =
                    std::fs::remove_file(format!("{}/p_{platform_id}_versions.bin", self.db_path));
                let _ =
                    std::fs::remove_file(format!("{}/p_{platform_id}_max_id.bin", self.db_path));

                for s_entry in platform.selectors.iter() {
                    let selector_id = *s_entry.key();
                    let _ = std::fs::remove_file(format!(
                        "{}/p_{platform_id}_s_{selector_id}_q.bin",
                        self.db_path
                    ));
                    let _ = std::fs::remove_file(format!(
                        "{}/p_{platform_id}_s_{selector_id}_timers.bin",
                        self.db_path
                    ));
                }

                Ok(())
            })
            .await
            .map_err(|e| e.to_string())??;

            Ok(PlatformDeleteStatus::Deleted)
        } else {
            Ok(PlatformDeleteStatus::NotFound)
        }
    }

    #[autometrics]
    async fn platforms_list(self, _: tarpc::context::Context) -> Vec<u8> {
        self.state.platforms.iter().map(|v| *v.key()).collect()
    }

    #[autometrics]
    async fn selector_create(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
        selector_id: u8,
        request: String,
        lock_duration_sec: u64,
    ) -> SelectorCreateStatus {
        let Some(platform) = self.state.platforms.get(&platform_id) else {
            return SelectorCreateStatus::PlatformDoesNotExist;
        };

        let ast = match arachne_parser::parse_query(&request) {
            Ok(parsed) => Arc::new(*parsed),
            Err(e) => return SelectorCreateStatus::ParseError(e.to_string()),
        };

        let status = match platform.selectors.get(&selector_id) {
            Some(v) => {
                if v.ast == ast {
                    SelectorCreateStatus::Exists
                } else {
                    SelectorCreateStatus::Modified
                }
            }
            None => SelectorCreateStatus::Created,
        };

        // Если селектор изменен или создан, то добавляем его в список (удаляя старый)
        if status != SelectorCreateStatus::Exists {
            platform.selectors.insert(
                selector_id,
                Arc::new(SelectorState {
                    ready_ids: SegQueue::new(),
                    prefetch_queue: SegQueue::new(),
                    timer_wheel: Arc::new(Default::default()),
                    ast,
                    lock_duration_sec,
                    queued_versions: Default::default(),
                    exact_size: AtomicUsize::new(0),
                    is_backfilling: AtomicBool::new(true),
                }),
            );

            let config_data = (request.clone(), lock_duration_sec);
            let config_bytes = match rmp_serde::to_vec_named(&config_data) {
                Ok(b) => b,
                Err(e) => return SelectorCreateStatus::ParseError(format!("Serialize error: {e}")),
            };
            if let Ok(selectors_ks) = self
                .db
                .keyspace(&format!("p_{platform_id}_selectors"), db_options)
                && let Err(e) = selectors_ks.insert([selector_id], &config_bytes)
            {
                tracing::error!("Failed to save selector config to db: {e}");
            }

            // Запускаем заполнение очереди
            let max_id = platform.next_internal_id.load(Ordering::Relaxed);
            self.state
                .backfill_queue
                .push((platform_id, selector_id, 0, max_id));
            self.state.backfill_notify.notify_one();
        }
        status
    }

    #[autometrics]
    async fn selector_delete(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
        selector_id: u8,
    ) -> Result<SelectorDeleteStatus, String> {
        // TODO: При удалении селектора удаляются пользователи что ли все тоже?

        let Some(platform) = self.state.platforms.get(&platform_id) else {
            return Ok(SelectorDeleteStatus::PlatformDoesNotExist);
        };

        if platform.selectors.remove(&selector_id).is_some() {
            let selectors_ks = self
                .db
                .keyspace(&format!("p_{platform_id}_selectors"), db_options)
                .unwrap();
            selectors_ks.remove([selector_id]).unwrap();

            Ok(SelectorDeleteStatus::Deleted)
        } else {
            Ok(SelectorDeleteStatus::NotFound)
        }
    }

    #[autometrics]
    async fn selector_queue_size(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
        selector_id: u8,
    ) -> SelectorSize {
        let Some(platform) = self.state.platforms.get(&platform_id) else {
            return SelectorSize::PlatformDoesNotExist;
        };
        let Some(selector) = platform.selectors.get(&selector_id) else {
            return SelectorSize::SelectorDoesNotExist;
        };
        SelectorSize::Size(selector.exact_size.load(Ordering::Relaxed) as u64)
    }

    #[autometrics]
    async fn selector_pending_size(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
        selector_id: u8,
    ) -> SelectorSize {
        let Some(platform) = self.state.platforms.get(&platform_id) else {
            return SelectorSize::PlatformDoesNotExist;
        };
        let Some(selector) = platform.selectors.get(&selector_id) else {
            return SelectorSize::SelectorDoesNotExist;
        };
        SelectorSize::Size(selector.timer_wheel.waiting_len() as u64)
    }

    #[autometrics]
    async fn selector_locked_size(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
        selector_id: u8,
    ) -> SelectorSize {
        let Some(platform) = self.state.platforms.get(&platform_id) else {
            return SelectorSize::PlatformDoesNotExist;
        };
        let Some(selector) = platform.selectors.get(&selector_id) else {
            return SelectorSize::SelectorDoesNotExist;
        };
        SelectorSize::Size(selector.timer_wheel.locked_len() as u64)
    }

    #[autometrics]
    async fn select_user(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
        selector_id: u8,
    ) -> Result<rpc_dto::RpcSelectUserResponse, String> {
        let Some(platform) = self.state.platforms.get(&platform_id) else {
            return Ok(rpc_dto::RpcSelectUserResponse::PlatformDoesNotExist);
        };
        let Some(selector) = platform.selectors.get(&selector_id) else {
            return Ok(rpc_dto::RpcSelectUserResponse::SelectorDoesNotExist);
        };

        let now = current_ts();

        // Выдача задачи сначала из кеша
        while let Some((internal_id, queued_version, payload)) = selector.prefetch_queue.pop() {
            // Пытаемся вычеркнуть из очереди. Если успешно - юзер наш.
            if selector
                .queued_versions
                .get(internal_id)
                .compare_exchange(queued_version, 0, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                selector.exact_size.fetch_sub(1, Ordering::Relaxed);

                let version_ptr = platform.versions.get(internal_id);

                // Пытаемся заблокировать его на уровне всей платформы (поднимаем глобальную версию)
                if version_ptr
                    .compare_exchange(
                        queued_version,
                        queued_version + 1,
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    let new_version = queued_version + 1;
                    let unlock_ts = now + std::cmp::max(1, selector.lock_duration_sec);

                    selector.timer_wheel.insert(
                        now,
                        crate::timer_wheel::TimerEntry {
                            user_id: internal_id,
                            version: new_version,
                            trigger_ts: unlock_ts,
                            is_lock_timeout: true,
                        },
                    );

                    return Ok(rpc_dto::RpcSelectUserResponse::Found(payload.into()));
                } else {
                    // Версия уже изменилась (сработал user_update или другой селектор забрал).
                    // Наш селектор просто отпускает его, так как user_update всё равно вернёт его в ready_ids если надо.
                    continue;
                }
            } else {
                // Вычеркнуть не удалось - версия в очереди изменилась, или он был удален таймером
                let current_q_version = selector
                    .queued_versions
                    .get(internal_id)
                    .load(Ordering::Acquire);
                // Если юзера удалил таймер, current_q_version будет 0 (он протух)
                if current_q_version > 0 {
                    // Юзер все еще должен быть в очереди, возвращаем его в сырые ID для скачивания свежей копии
                    selector.ready_ids.push(internal_id);
                }
            }
        }

        // Если мы дошли сюда, значит готового пользователя прямо сейчас нет.
        // Проверяем, закончен ли селектор полностью:
        let is_backfilling = selector.is_backfilling.load(Ordering::Acquire);
        let exact_queue_size = selector.exact_size.load(Ordering::Acquire);
        let timers_count = selector.timer_wheel.len();

        // Если идет сканирование, либо есть юзеры в очереди (но они еще не догрузились),
        // либо есть заблокированные/спящие юзеры в колесе времени -> возвращаем Empty.
        if is_backfilling || exact_queue_size > 0 || timers_count > 0 {
            Ok(rpc_dto::RpcSelectUserResponse::Empty)
        } else {
            // В противном случае селектор полностью истощен.
            Ok(rpc_dto::RpcSelectUserResponse::Finished)
        }
    }

    #[autometrics]
    async fn users_add_batch(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
        payloads: Vec<rpc_dto::RpcUserPayload>,
    ) -> Result<UserAddStatus, String> {
        let payloads: Vec<UserPayload> = payloads.into_iter().map(Into::into).collect();

        let Some(platform) = self.state.platforms.get(&platform_id) else {
            return Ok(UserAddStatus::PlatformDoesNotExist);
        };

        let db_clone = self.db.clone();
        let mut id_assigns = Vec::with_capacity(payloads.len());

        // Выдаем ID в RAM
        for payload in payloads {
            // В идеале здесь тоже нужно проверить существование, но для скорости импорта пропустим
            let internal_id = platform.allocate_internal_id();
            let version = platform
                .versions
                .get(internal_id)
                .fetch_add(1, Ordering::SeqCst)
                + 1;
            id_assigns.push((internal_id, version, payload));
        }

        // Пишем в бд одним огромным коммитом
        let id_assigns_for_db = id_assigns.clone();
        let platform_clone = platform.clone();
        tokio::task::spawn_blocking(move || {
            let mut batch = db_clone.batch();
            for (internal_id, _, payload) in id_assigns_for_db {
                let id_bytes = rmp_serde::to_vec_named(&payload.id).unwrap();
                let payload_bytes = rmp_serde::to_vec_named(&payload).unwrap();
                batch.insert(&platform_clone.id_db, id_bytes, internal_id.to_be_bytes());
                batch.insert(
                    &platform_clone.payload_db,
                    internal_id.to_be_bytes(),
                    payload_bytes,
                );
            }
            batch
                .commit()
                .map_err(|e| format!("Batch commit failed: {:?}", e))
        })
        .await
        .map_err(|e| e.to_string())??;

        // Раскидываем по селекторам
        let now = current_ts();
        let selectors: Vec<_> = platform
            .selectors
            .iter()
            .map(|s| s.value().clone())
            .collect();

        for (internal_id, version, payload) in id_assigns {
            for selector_entry in &selectors {
                dispatch_user_to_selector(selector_entry, internal_id, version, &payload, now);
            }
        }

        Ok(UserAddStatus::Success)
    }

    #[autometrics]
    async fn user_add(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
        payload: rpc_dto::RpcUserPayload,
    ) -> Result<UserAddStatus, String> {
        let payload: UserPayload = payload.into();

        let Some(platform) = self.state.platforms.get(&platform_id) else {
            return Ok(UserAddStatus::PlatformDoesNotExist);
        };

        let id_bytes = rmp_serde::to_vec_named(&payload.id).map_err(|e| e.to_string())?;

        // Блокируем мутации конкретно для этого пользователя
        let _guard = platform.mutation_lock.lock(&id_bytes).await;

        if platform.id_db.get(&id_bytes).unwrap().is_some() {
            return Ok(UserAddStatus::Exists);
        }

        let internal_id = platform.allocate_internal_id();
        let payload_bytes = rmp_serde::to_vec_named(&payload).map_err(|e| e.to_string())?;

        let version = platform
            .versions
            .get(internal_id)
            .fetch_add(1, Ordering::SeqCst)
            + 1;

        // Добавляем батчем (потенциально увеличивает скорость)
        tokio::task::block_in_place(|| {
            let mut batch = self.db.batch();
            batch.insert(&platform.id_db, id_bytes, internal_id.to_be_bytes());
            batch.insert(
                &platform.payload_db,
                internal_id.to_be_bytes(),
                payload_bytes,
            );
            batch
                .commit()
                .map_err(|e| format!("Commit failed: {:?}", e))
        })?;

        let now = current_ts();

        // Направляем юзера в селектор
        let selectors: Vec<_> = platform
            .selectors
            .iter()
            .map(|s| s.value().clone())
            .collect();
        for selector_entry in selectors {
            dispatch_user_to_selector(&selector_entry, internal_id, version, &payload, now);
        }

        Ok(UserAddStatus::Success)
    }

    #[autometrics]
    async fn user_delete(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
        user_id: rpc_dto::RpcUserId,
    ) -> Result<UserDeleteStatus, String> {
        let user_id: UserId = user_id.into();

        let Some(platform) = self.state.platforms.get(&platform_id) else {
            return Ok(UserDeleteStatus::PlatformDoesNotExist);
        };

        let id_bytes = rmp_serde::to_vec_named(&user_id).map_err(|e| e.to_string())?;

        // Блокируем мутации конкретно для этого пользователя
        let _guard = platform.mutation_lock.lock(&id_bytes).await;

        let Some(internal_id_bytes) = platform.id_db.get(&id_bytes).unwrap() else {
            return Ok(UserDeleteStatus::NotFound);
        };
        let internal_id_slice: [u8; 4] = internal_id_bytes
            .as_ref()
            .try_into()
            .map_err(|_| "Corrupted db".to_string())?;
        let internal_id = u32::from_be_bytes(internal_id_slice);

        // Инвалидируем. Старые задачи сбросятся воркерами
        let new_version = platform
            .versions
            .get(internal_id)
            .fetch_add(1, Ordering::SeqCst)
            + 1;

        // Удаляем из базы
        tokio::task::block_in_place(|| {
            let mut batch = self.db.batch();
            batch.remove(&platform.id_db, &id_bytes);
            batch.remove(&platform.payload_db, internal_id.to_be_bytes());
            batch
                .commit()
                .map_err(|e| format!("Commit failed: {:?}", e))
        })?;

        // Ленивое вычеркивание из очередей селекторов, чтобы exact_size обновился корректно
        let selectors: Vec<_> = platform
            .selectors
            .iter()
            .map(|s| s.value().clone())
            .collect();
        for selector in selectors {
            let q_version_ptr = selector.queued_versions.get(internal_id);
            let mut current = q_version_ptr.load(Ordering::Acquire);
            loop {
                if current == 0 || current >= new_version {
                    break;
                }
                match q_version_ptr.compare_exchange_weak(
                    current,
                    0,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        selector.exact_size.fetch_sub(1, Ordering::Relaxed);
                        break;
                    }
                    Err(x) => current = x,
                }
            }
        }

        // Возращаем в пул для переиспользования
        platform.free_internal_ids.push(internal_id);

        Ok(UserDeleteStatus::Success)
    }

    #[autometrics]
    async fn user_update(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
        payload: rpc_dto::RpcUserPayload,
    ) -> Result<UserUpdateStatus, String> {
        let payload: UserPayload = payload.into();
        let Some(platform) = self.state.platforms.get(&platform_id) else {
            return Ok(UserUpdateStatus::PlatformDoesNotExist);
        };

        let id_bytes = rmp_serde::to_vec_named(&payload.id).unwrap();

        // Блокируем мутации конкретно для этого пользователя
        let _guard = platform.mutation_lock.lock(&id_bytes).await;

        let internal_id_bytes = platform.id_db.get(&id_bytes).unwrap();
        if internal_id_bytes.is_none() {
            return Ok(UserUpdateStatus::NotFound);
        }
        let internal_id =
            u32::from_be_bytes(internal_id_bytes.unwrap().as_ref().try_into().unwrap());
        let payload_bytes = rmp_serde::to_vec_named(&payload).map_err(|e| e.to_string())?;

        let version = platform
            .versions
            .get(internal_id)
            .fetch_add(1, Ordering::SeqCst)
            + 1;

        tokio::task::block_in_place(|| {
            let mut batch = self.db.batch();
            batch.insert(&platform.id_db, id_bytes, internal_id.to_be_bytes());
            batch.insert(
                &platform.payload_db,
                internal_id.to_be_bytes(),
                payload_bytes,
            );
            batch
                .commit()
                .map_err(|e| format!("Commit failed: {:?}", e))
        })?;

        let now = current_ts();

        // Направляем юзера в селектор
        let selectors: Vec<_> = platform
            .selectors
            .iter()
            .map(|s| s.value().clone())
            .collect();
        for selector_entry in selectors {
            dispatch_user_to_selector(&selector_entry, internal_id, version, &payload, now);
        }

        Ok(UserUpdateStatus::Success)
    }

    #[autometrics]
    async fn user_release(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
        user_id: rpc_dto::RpcUserId,
    ) -> Result<UserReleaseStatus, String> {
        let user_id: UserId = user_id.into();
        let Some(platform) = self.state.platforms.get(&platform_id) else {
            return Ok(UserReleaseStatus::PlatformDoesNotExist);
        };

        let id_bytes = rmp_serde::to_vec_named(&user_id).map_err(|e| e.to_string())?;

        let Some(internal_id_bytes) = platform.id_db.get(&id_bytes).unwrap() else {
            return Ok(UserReleaseStatus::NotFound);
        };

        let internal_id_slice: [u8; 4] = internal_id_bytes
            .as_ref()
            .try_into()
            .map_err(|_| "Corrupted db: ID slice is not 4 bytes".to_string())?;
        let internal_id = u32::from_be_bytes(internal_id_slice);

        let version = platform
            .versions
            .get(internal_id)
            .fetch_add(1, Ordering::SeqCst)
            + 1;

        let payload_bytes = platform
            .payload_db
            .get(internal_id.to_be_bytes())
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "User payload missing for existing ID".to_string())?;
        let payload: UserPayload =
            rmp_serde::from_slice(&payload_bytes).map_err(|e| e.to_string())?;

        let now = current_ts();

        let selectors: Vec<_> = platform
            .selectors
            .iter()
            .map(|s| s.value().clone())
            .collect();
        for selector_entry in selectors {
            dispatch_user_to_selector(&selector_entry, internal_id, version, &payload, now);
        }

        Ok(UserReleaseStatus::Released)
    }

    #[autometrics]
    async fn user_search(
        self,
        _: ::tarpc::context::Context,
        platform_id: u8,
        user_id: rpc_dto::RpcUserId,
    ) -> Result<Option<rpc_dto::RpcUserPayload>, String> {
        let user_id: UserId = user_id.into();

        let Some(platform) = self.state.platforms.get(&platform_id) else {
            return Ok(None);
        };

        let id_bytes = rmp_serde::to_vec_named(&user_id).map_err(|e| e.to_string())?;

        let Some(internal_id_bytes) = platform.id_db.get(&id_bytes).unwrap() else {
            return Ok(None);
        };
        let internal_id_slice: [u8; 4] = internal_id_bytes
            .as_ref()
            .try_into()
            .map_err(|_| "Corrupted db: ID slice is not 4 bytes".to_string())?;
        let internal_id = u32::from_be_bytes(internal_id_slice);

        let payload_bytes = platform
            .payload_db
            .get(internal_id.to_be_bytes())
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "User payload missing for existing ID".to_string())?;
        let payload: UserPayload =
            rmp_serde::from_slice(&payload_bytes).map_err(|e| e.to_string())?;

        Ok(Some(payload.into()))
    }

    #[autometrics]
    async fn total_users_count(self, _: tarpc::context::Context) -> Result<u64, String> {
        let mut total = 0;
        for platform in self.state.platforms.iter() {
            total += platform.value().next_internal_id.load(Ordering::Relaxed) as usize
                - platform.free_internal_ids.len();
        }
        Ok(total as u64)
    }

    #[autometrics]
    async fn total_platformed_users_count(
        self,
        _: tarpc::context::Context,
        platform_id: u8,
    ) -> Result<u64, String> {
        let Some(platform) = self.state.platforms.get(&platform_id) else {
            return Ok(0);
        };
        Ok(platform.value().next_internal_id.load(Ordering::Relaxed) as u64)
    }
}

////////////////////////////////////////////////////////////////////////////////

fn current_ts() -> u64 {
    Utc::now().timestamp() as u64
}

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}

////////////////////////////////////////////////////////////////////////////////

async fn start_backfill_workers(state: Arc<RamState>, token: CancellationToken) {
    loop {
        // Достаем все накопившиеся задачи из очереди
        while let Some((platform_id, selector_id, scan_start, scan_end)) =
            state.backfill_queue.pop()
        {
            let state_clone = state.clone();
            let token_clone = token.clone();

            // Спавним независимую задачу (Task).
            // Это гарантирует, что каждый новый селектор сканирует БД в своем потоке
            // и не ждет завершения сканирования других селекторов.
            tokio::spawn(async move {
                // CHUNK_SIZE: Читаем БД порциями по 100k записей, чтобы не загрузить все данные в оперативную память
                const CHUNK_SIZE: u32 = 100_000;
                // CONCURRENT_CHUNKS: Ограничиваем параллелизм (максимум 2 чанка одновременно),
                // чтобы не исчерпать пул потоков `spawn_blocking` и не "задушить" диск
                const CONCURRENT_CHUNKS_PER_WORKER: usize = 2;

                let platform = {
                    let Some(p) = state_clone.platforms.get(&platform_id) else {
                        return;
                    };
                    p.clone()
                };

                let mut chunk_futures = FuturesUnordered::new();

                for chunk_start in (scan_start..scan_end).step_by(CHUNK_SIZE as usize) {
                    let chunk_end = std::cmp::min(chunk_start + CHUNK_SIZE, scan_end);
                    let db_inner = platform.payload_db.clone();
                    let platform_inner = platform.clone();
                    let token_inner = token_clone.clone();

                    let handle = tokio::task::spawn_blocking(move || {
                        let selector = {
                            let Some(s) = platform_inner.selectors.get(&selector_id) else {
                                return;
                            };
                            s.clone()
                        };

                        let start_key = chunk_start.to_be_bytes();
                        let end_key = chunk_end.to_be_bytes();
                        // Итератор Fjall делает Range Scan — это последовательное (Sequential) чтение,
                        // которое работает быстрее случайного доступа (get).
                        let iter = db_inner.range(start_key..end_key);

                        let mut count = 0;
                        for item in iter {
                            count += 1;
                            // Проверяем токен отмены каждые 1000 записей.
                            // Это позволяет быстро остановить поток при выключении сервера (Ctrl+C).
                            if count % 1000 == 0 && token_inner.is_cancelled() {
                                break;
                            }

                            if let Ok((k, v)) = item.into_inner() {
                                let internal_id = u32::from_be_bytes(k[..4].try_into().unwrap());
                                let current_version = platform_inner
                                    .versions
                                    .get(internal_id)
                                    .load(Ordering::Acquire);
                                if current_version == 0 {
                                    continue;
                                }

                                if let Ok(payload) = rmp_serde::from_slice::<UserPayload>(&v) {
                                    let now = current_ts();
                                    dispatch_user_to_selector(
                                        &selector,
                                        internal_id,
                                        current_version,
                                        &payload,
                                        now,
                                    );
                                }
                            }
                        }
                    });

                    chunk_futures.push(handle);
                    // Если запущено уже 2 чанка, ждем завершения хотя бы одного, прежде чем запускать третий.
                    while chunk_futures.len() >= CONCURRENT_CHUNKS_PER_WORKER {
                        tokio::select! {
                            _ = chunk_futures.next() => {},
                            _ = token_clone.cancelled() => break,
                        }
                    }
                }
                while chunk_futures.next().await.is_some() {
                    if token_clone.is_cancelled() {
                        return;
                    }
                }
                // Когда все чанки базы обработаны, снимаем флаг backfilling
                if let Some(p) = state_clone.platforms.get(&platform_id)
                    && let Some(s) = p.selectors.get(&selector_id)
                {
                    s.is_backfilling.store(false, Ordering::Release);
                }
            });
        }

        // Очередь пуста — спим до сигнала Notify (от нового селектора)
        tokio::select! {
            _ = state.backfill_notify.notified() => {},
            _ = token.cancelled() => return,
        }
    }
}

async fn start_time_ticker(state: Arc<RamState>, db: Database, token: CancellationToken) {
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
    let mut last_ts = current_ts() - 1;

    loop {
        tokio::select! {
            _ = token.cancelled() => return,
            _ = interval.tick() => {}
        }
        let current = current_ts();

        // Защита от "пустых" вызовов в рамках одной и той же секунды
        if current <= last_ts {
            continue;
        }

        // Если сервер сильно загрузили и интервал сработал с задержкой (лагает),
        // мы прогоняем все пропущенные секунды
        for now in (last_ts + 1)..=current {
            // Не блокируем dashmap lock
            let platforms: Vec<_> = state
                .platforms
                .iter()
                .map(|p| (*p.key(), p.value().clone()))
                .collect();
            for (platform_id, platform) in platforms {
                let mut platform_timers = Vec::new();
                // Не блокируем dashmap lock
                let selectors: Vec<_> = platform
                    .selectors
                    .iter()
                    .map(|s| (*s.key(), s.value().clone()))
                    .collect();
                for (selector_id, selector) in selectors {
                    let ready_timers = selector.timer_wheel.tick(now);

                    if !ready_timers.is_empty() {
                        platform_timers.push((selector_id, ready_timers));
                    }
                }

                if platform_timers.is_empty() {
                    continue;
                }

                let payload_db = match db.keyspace(&format!("p_{platform_id}_payloads"), db_options)
                {
                    Ok(ks) => ks,
                    Err(_) => continue,
                };

                for (origin_selector_id, timers) in platform_timers {
                    let mut processed = 0;

                    for timer in timers {
                        let current_version =
                            platform.versions.get(timer.user_id).load(Ordering::Acquire);

                        // Если юзера обновили (user_update) пока он лежал в колесе времени,
                        // его текущая версия будет больше. Игнорируем этот просроченный таймер.
                        if current_version != timer.version {
                            continue;
                        }

                        let mut version_to_dispatch = timer.version;
                        // Принудительно повышаем версию (инвалидируем его во всех горячих очередях)
                        if timer.is_lock_timeout {
                            version_to_dispatch = timer.version + 1;

                            // Если user_update успел вклиниться, то CAS вернет Err.
                            if platform
                                .versions
                                .get(timer.user_id)
                                .compare_exchange(
                                    timer.version,
                                    version_to_dispatch,
                                    Ordering::AcqRel,
                                    Ordering::Acquire,
                                )
                                .is_err()
                            {
                                // Проиграли гонку с user_update. Этот таймер больше не актуален.
                                continue;
                            }
                        }

                        if let Ok(Some(bytes)) = payload_db.get(timer.user_id.to_be_bytes())
                            && let Ok(payload) = rmp_serde::from_slice::<UserPayload>(&bytes)
                        {
                            if timer.is_lock_timeout {
                                // Мы уже подняли версию в CAS выше, просто рассылаем
                                for target_selector_entry in platform.selectors.iter() {
                                    dispatch_user_to_selector(
                                        target_selector_entry.value(),
                                        timer.user_id,
                                        version_to_dispatch,
                                        &payload,
                                        now,
                                    );
                                }
                            } else {
                                // Наступило время из будущего (версию не поднимали)
                                if let Some(origin_selector_entry) =
                                    platform.selectors.get(&origin_selector_id)
                                {
                                    dispatch_user_to_selector(
                                        origin_selector_entry.value(),
                                        timer.user_id,
                                        version_to_dispatch, // Равна timer.version
                                        &payload,
                                        now,
                                    );
                                }
                            }
                        }

                        processed += 1;
                        if processed % 1000 == 0 {
                            tokio::select! {
                            _ = tokio::task::yield_now() => {},
                                _ = token.cancelled() => { return }
                            }
                        }
                    }
                }
            }
        }
        last_ts = current;
    }
}

async fn start_prefetch_workers(
    state: Arc<RamState>,
    db: Database,
    limit: usize,
    token: CancellationToken,
) {
    loop {
        let mut worked = false;

        // Не блокируем dashmap lock
        let platforms: Vec<_> = state
            .platforms
            .iter()
            .map(|p| (*p.key(), p.value().clone()))
            .collect();
        for (platform_id, platform) in platforms {
            let payload_db = match db.keyspace(&format!("p_{platform_id}_payloads"), db_options) {
                Ok(ks) => ks,
                Err(_) => continue,
            };

            // Не блокируем dashmap lock
            let selectors: Vec<_> = platform
                .selectors
                .iter()
                .map(|s| s.value().clone())
                .collect();
            for selector in selectors {
                let mut batch = Vec::new();

                // Собираем батч без обращения к диску
                while selector.prefetch_queue.len() + batch.len() < limit && batch.len() < 500 {
                    let Some(internal_id) = selector.ready_ids.pop() else {
                        break;
                    };
                    let q_version = selector
                        .queued_versions
                        .get(internal_id)
                        .load(Ordering::Acquire);
                    if q_version > 0 {
                        batch.push((internal_id, q_version));
                    }
                }

                if !batch.is_empty() {
                    worked = true;
                    let db_clone = payload_db.clone();

                    let fetched = tokio::task::spawn_blocking(move || {
                        let mut results = Vec::with_capacity(batch.len());
                        for (id, q_v) in batch {
                            let res = db_clone.get(id.to_be_bytes());
                            results.push((id, q_v, res));
                        }
                        results
                    })
                    .await
                    .unwrap();

                    // Раскладываем в RAM
                    for (internal_id, q_version, res) in fetched {
                        if let Ok(Some(bytes)) = res {
                            if let Ok(payload) = rmp_serde::from_slice::<UserPayload>(&bytes) {
                                selector
                                    .prefetch_queue
                                    .push((internal_id, q_version, payload));
                            } else {
                                if selector
                                    .queued_versions
                                    .get(internal_id)
                                    .compare_exchange(
                                        q_version,
                                        0,
                                        Ordering::AcqRel,
                                        Ordering::Relaxed,
                                    )
                                    .is_ok()
                                {
                                    selector.exact_size.fetch_sub(1, Ordering::Relaxed);
                                }
                            }
                        } else {
                            if selector
                                .queued_versions
                                .get(internal_id)
                                .compare_exchange(q_version, 0, Ordering::AcqRel, Ordering::Relaxed)
                                .is_ok()
                            {
                                selector.exact_size.fetch_sub(1, Ordering::Relaxed);
                            }
                        }
                    }

                    // Переключение контекста
                    tokio::time::sleep(std::time::Duration::from_millis(2)).await;
                }
            }
        }

        // Если работы не было, спим 5мс, чтобы не грузить CPU
        if !worked {
            tokio::select! {
                _ =  tokio::time::sleep(std::time::Duration::from_millis(5)) => {},
                _ = token.cancelled() => { return }
            }
        }
    }
}

fn db_options() -> KeyspaceCreateOptions {
    KeyspaceCreateOptions::default()
        // Размер буфера в памяти перед сбросом на диск
        .max_memtable_size(1024 * 1024 * 256)
        .data_block_size_policy(BlockSizePolicy::all(48 * 1024))
        .data_block_compression_policy(CompressionPolicy::all(CompressionType::Lz4))
}

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        //.with(filter)
        .try_init()
        .unwrap();

    //Инициализируем сборщик метрик
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener(args.metrics_addr.parse::<std::net::SocketAddr>().unwrap());
    builder.install().expect("failed to install TCP recorder");

    // Метрики для токио
    tokio::task::spawn(
        tokio_metrics::RuntimeMetricsReporterBuilder::default()
            .with_interval(std::time::Duration::from_secs(15))
            .describe_and_run(),
    );

    let db_path = Arc::new(args.db_path);

    let db = Database::builder(db_path.as_str())
        .cache_size(args.cache_mb.as_u64())
        .max_journaling_size(args.max_journaling_size.as_u64())
        .journal_compression(CompressionType::Lz4)
        .open()?;
    let state = Arc::new(RamState::recover(&db, db_path.to_string()).unwrap());
    let token = CancellationToken::new();

    let mut workers = tokio::task::JoinSet::<()>::new();
    workers.spawn(start_time_ticker(state.clone(), db.clone(), token.clone()));
    workers.spawn(start_backfill_workers(state.clone(), token.clone()));
    workers.spawn(start_prefetch_workers(
        state.clone(),
        db.clone(),
        args.prefetch_limit,
        token.clone(),
    ));

    // Создаем TCP Listener руками
    let tcp_listener = tokio::net::TcpListener::bind(args.bind).await?;

    // Настраиваем парсер фреймов
    let mut codec_builder = LengthDelimitedCodec::builder();
    codec_builder.max_frame_length(usize::MAX);

    // Создаем поток кастомных Транспортов
    let listener = tokio_stream::wrappers::TcpListenerStream::new(tcp_listener)
        .filter_map(|sock_res| future::ready(sock_res.ok()))
        .map(move |sock| {
            // Отключаем буферизацию для моментальных ответов
            let _ = sock.set_nodelay(true);

            // Оборачиваем голый сокет во фреймер
            let framed = codec_builder.new_framed(sock);

            // Превращаем в Tarpc Transport с форматом Json
            tarpc::serde_transport::new(framed, BitcodeCodec)
        });

    let snapshot_state = state.clone();
    let snapshot_token = token.clone();
    let path = db_path.clone();

    // Signal handling (Ctrl+C, SIGTERM)
    let task = tokio::spawn(async move {
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = sigterm.recv() => {},
        }
        snapshot_token.cancel();
        workers.join_all().await;
        tokio::task::spawn_blocking(move || snapshot_state.snapshot(&path)).await
    });

    listener
        .take_until(token.cancelled())
        .map(|t| {
            Config {
                pending_response_buffer: CONCURRENCY_COUNT,
            }
            .channel(t)
        })
        .map(|channel| {
            let server = Server {
                db: db.clone(),
                state: state.clone(),
                db_path: db_path.clone(),
            };

            // Разрешаем держать открытыми до CONCURRENCY_COUNT клиентских подключений одновременно
            channel
                .max_concurrent_requests(CONCURRENCY_COUNT)
                .execute(server.serve())
                .for_each(spawn)
        })
        // Разрешаем держать открытыми до CONCURRENCY_COUNT клиентских подключений одновременно
        .buffer_unordered(CONCURRENCY_COUNT)
        .for_each(|_| async {})
        .await;

    task.await???;
    Ok(())
}
