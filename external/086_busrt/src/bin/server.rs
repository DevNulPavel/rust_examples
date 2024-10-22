#[cfg(feature = "rpc")]
use busrt::broker::BrokerEvent;
use busrt::broker::{Broker, Options, ServerConfig};
use chrono::prelude::*;
use clap::Parser;
use colored::Colorize;
use log::{error, info, trace, Level, LevelFilter};
use std::{sync::atomic, time::Duration};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::Mutex,
    time::sleep,
};

#[macro_use]
extern crate lazy_static;

////////////////////////////////////////////////////////////////////////////////

/// Включаем какой-то кастомный аллокатор если требуется
#[cfg(not(feature = "std-alloc"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

////////////////////////////////////////////////////////////////////////////////

static SERVER_ACTIVE: atomic::AtomicBool = atomic::AtomicBool::new(true);

lazy_static! {
    /// Файлик с идентификатором процесса
    static ref PID_FILE: Mutex<Option<String>> = Mutex::new(None);

    /// Файлики сокетов
    static ref SOCK_FILES: Mutex<Vec<String>> = Mutex::new(Vec::new());

    /// Брокер какой-то
    static ref BROKER: Mutex<Option<Broker>> = Mutex::new(None);
}

////////////////////////////////////////////////////////////////////////////////

#[allow(clippy::struct_excessive_bools)]
#[derive(Parser)]
struct Opts {
    #[clap(
        short = 'B',
        long = "bind",
        required = true,
        help = "Unix socket path, IP:PORT or fifo:path, can be specified multiple times"
    )]
    path: Vec<String>,

    #[clap(short = 'P', long = "pid-file")]
    pid_file: Option<String>,

    #[clap(long = "verbose", help = "Verbose logging")]
    verbose: bool,

    #[clap(short = 'D')]
    daemonize: bool,

    #[clap(long = "log-syslog", help = "Force log to syslog")]
    log_syslog: bool,

    #[clap(
        long = "force-register",
        help = "Force register new clients with duplicate names"
    )]
    force_register: bool,

    #[clap(short = 'w', default_value = "4")]
    workers: usize,

    #[clap(short = 't', default_value = "5", help = "timeout (seconds)")]
    timeout: f64,

    #[clap(
        long = "buf-size",
        default_value = "16384",
        help = "I/O buffer size, per client"
    )]
    buf_size: usize,

    #[clap(
        long = "buf-ttl",
        default_value = "10",
        help = "Write buffer TTL (microseconds)"
    )]
    buf_ttl: u64,

    #[clap(
        long = "queue-size",
        default_value = "8192",
        help = "frame queue size, per client"
    )]
    queue_size: usize,
}

////////////////////////////////////////////////////////////////////////////////

/// Статически созданный логгер
static LOGGER: SimpleLogger = SimpleLogger;

////////////////////////////////////////////////////////////////////////////////

// Какой-то логгер наш кастомный простой
struct SimpleLogger;

/// Реализация системы логировани для логгера
impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            // Создаем аргументы для форматирования строки логгера
            let s = format_args!(
                "{}  {}",
                Local::now().to_rfc3339_opts(SecondsFormat::Secs, false),
                record.args()
            );

            // Делаем вывод непосредственно теперь
            println!(
                "{}",
                match record.level() {
                    Level::Trace => s.black().dimmed(),
                    Level::Debug => s.dimmed(),
                    Level::Warn => s.yellow().bold(),
                    Level::Error => s.red(),
                    Level::Info => s.normal(),
                }
            );
        }
    }

    fn flush(&self) {}
}

////////////////////////////////////////////////////////////////////////////////

/// Настройки теперь уровня подробности логирования
fn set_verbose_logger(filter: LevelFilter) {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(filter))
        .unwrap();
}

////////////////////////////////////////////////////////////////////////////////

/// Обработка завершения работы, выполнение всяких служебных дел по завершению
async fn terminate(allow_log: bool) {
    // Смотрим на наличие PID-файлика
    if let Some(f) = PID_FILE.lock().await.as_ref() {
        // Надо ли логировать здесь?
        if allow_log {
            trace!("removing pid file {}", f);
        }
        // Удаляем этот самый файлик
        let _r = std::fs::remove_file(f);
    }

    // Перебираем теперь файлики сокетов для удаления
    for f in SOCK_FILES.lock().await.iter() {
        // Если нам надо, то запишем логи
        if allow_log {
            trace!("removing sock file {}", f);
        }
        // Удаляем файлики сокетов если такие есть у нас
        let _r = std::fs::remove_file(f);
    }

    // Сообщение в лог про завершение работы
    if allow_log {
        info!("terminating");
    }

    // Если есть брокер, то его завершаем тоже
    #[cfg(feature = "rpc")]
    if let Some(broker) = BROKER.lock().await.as_ref() {
        if let Err(e) = broker.announce(BrokerEvent::shutdown()).await {
            error!("{}", e);
        }
    }

    // Сохраняем флаг завершения работы
    SERVER_ACTIVE.store(false, atomic::Ordering::Relaxed);

    // Чуть подождем, если у нас rpc режим + брокер есть какой-то
    #[cfg(feature = "rpc")]
    sleep(Duration::from_secs(1)).await;
}

////////////////////////////////////////////////////////////////////////////////

/// Отдельный макрос для обработки сигнала
macro_rules! handle_term_signal {
    ($kind: expr, $allow_log: expr) => {
        // TODO: Взаимоблокировка сигналов разных, если какой-то
        // один уже начал работать

        // Запускаем футуру отдельную
        tokio::spawn(async move {
            trace!("starting handler for {:?}", $kind);

            // Запускаем обработку в цикле
            loop {
                // Ждем сигнал какой-то определенного типа в этой футуре
                match signal($kind) {
                    Ok(mut v) => {
                        v.recv().await;
                    }
                    Err(e) => {
                        error!("Unable to bind to signal {:?}: {}", $kind, e);
                        break;
                    }
                }

                // do not log anything on C-c
                if $allow_log {
                    trace!("got termination signal");
                }

                // Вызываем завершение работы теперь
                terminate($allow_log).await
            }
        });
    };
}

////////////////////////////////////////////////////////////////////////////////

#[allow(clippy::too_many_lines)]
fn main() {
    #[cfg(feature = "tracing")]
    console_subscriber::init();

    // Парсим параметры приложения
    let opts: Opts = Opts::parse();

    // Подробный вывод логов?
    if opts.verbose {
        set_verbose_logger(LevelFilter::Trace);
    }
    // Не фоновый запуск + не отключен syslog
    else if (!opts.daemonize || std::env::var("DISABLE_SYSLOG").as_deref() == Ok("1"))
        && !opts.log_syslog
    {
        set_verbose_logger(LevelFilter::Info);
    }
    // Остальное все по логированию
    else {
        // Создаем форматирование для syslog
        let formatter = syslog::Formatter3164 {
            facility: syslog::Facility::LOG_USER,
            hostname: None,
            process: "busrtd".into(),
            pid: 0,
        };

        // Запускаем syslog
        match syslog::unix(formatter) {
            // Создалось успешно
            Ok(logger) => {
                // Устанавливаем теперь успешно созданный логгер
                // в крейт log
                log::set_boxed_logger(Box::new(syslog::BasicLogger::new(logger)))
                    .map(|()| log::set_max_level(LevelFilter::Info))
                    .unwrap();
            }
            // Ошибка
            Err(_) => {
                // TODO: Здесь лучше бы вывести что-то
                //
                set_verbose_logger(LevelFilter::Info);
            }
        }
    }

    // Параметр таймаута в секундах
    let timeout = Duration::from_secs_f64(opts.timeout);

    // TTL в микросекундах
    let buf_ttl = Duration::from_micros(opts.buf_ttl);

    // Подробная информация теперь
    info!("starting BUS/RT server");
    info!("workers: {}", opts.workers);
    info!("buf size: {}", opts.buf_size);
    info!("buf ttl: {:?}", buf_ttl);
    info!("queue size: {}", opts.queue_size);
    info!("timeout: {:?}", timeout);

    // Надо ли нам делать процесс фоновым через форк?
    if opts.daemonize {
        // Запускаем переход в форк,
        if let Ok(fork::Fork::Child) = fork::daemon(true, false) {
            // Вроде как получается, если мы в родителе, а запустился чилд
            // у нас, то можем вайти теперь из текущего процесса
            std::process::exit(0);
        }
    }

    // Запускаем теперь рантайм tokio
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(opts.workers)
        .enable_all()
        .build()
        .unwrap();

    // В рантайме запускаем асинхронную уже задачу основную
    rt.block_on(async move {
        // Надо ли нам как-то сохранять pid процесса в файлик?
        if let Some(pid_file) = opts.pid_file {
            // Получаем идентификатор текущего процесса
            let pid = std::process::id().to_string();

            // Делаем для чего-то именно асинхронную запись в файлик
            tokio::fs::write(&pid_file, pid)
                .await
                .expect("Unable to write pid file");

            info!("created pid file {}", pid_file);

            // Дополнительно сохраняем для удаления будущего путь этого самого
            // файлика для удаления
            PID_FILE.lock().await.replace(pid_file);
        }

        // Назначаем обработку сигналов для завершения работы приложения
        handle_term_signal!(SignalKind::interrupt(), false);
        handle_term_signal!(SignalKind::terminate(), true);

        let mut broker = Broker::create(&Options::default().force_register(opts.force_register));

        #[cfg(feature = "rpc")]
        broker.init_default_core_rpc().await.unwrap();
        broker.set_queue_size(opts.queue_size);
        let mut sock_files = SOCK_FILES.lock().await;
        for path in opts.path {
            info!("binding at {}", path);
            #[allow(clippy::case_sensitive_file_extension_comparisons)]
            if let Some(_fifo) = path.strip_prefix("fifo:") {
                #[cfg(feature = "rpc")]
                {
                    broker
                        .spawn_fifo(_fifo, opts.buf_size)
                        .await
                        .expect("unable to start fifo server");
                    sock_files.push(_fifo.to_owned());
                }
            } else {
                let server_config = ServerConfig::new()
                    .buf_size(opts.buf_size)
                    .buf_ttl(buf_ttl)
                    .timeout(timeout);
                if path.ends_with(".sock")
                    || path.ends_with(".socket")
                    || path.ends_with(".ipc")
                    || path.starts_with('/')
                {
                    broker
                        .spawn_unix_server(&path, server_config)
                        .await
                        .expect("Unable to start unix server");
                    sock_files.push(path);
                } else {
                    broker
                        .spawn_tcp_server(&path, server_config)
                        .await
                        .expect("Unable to start tcp server");
                }
            }
        }
        drop(sock_files);
        BROKER.lock().await.replace(broker);
        info!("BUS/RT broker started");
        let sleep_step = Duration::from_millis(100);
        loop {
            if !SERVER_ACTIVE.load(atomic::Ordering::Relaxed) {
                break;
            }
            sleep(sleep_step).await;
        }
    });
}
