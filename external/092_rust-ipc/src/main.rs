use clap::Parser;
use ipc::{
    cpu_warmup, iceoryx::IceoryxRunner, mmap::MmapRunner, pipes::PipeRunner, shmem::ShmemRunner,
    tcp::TcpRunner, udp::UdpRunner, unix_datagram::UnixDatagramRunner,
    unix_stream::UnixStreamRunner, KB,
};

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Copy, Clone, clap::ValueEnum)]
enum Method {
    #[default]
    Stdout,
    Shmem,
    Tcp,
    Udp,
    Iceoryx,
    Mmap,
    Unixstream,
    Unixdatagram,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long)]
    number: usize,

    #[clap(short, long, default_value_t, value_enum)]
    method: Method,

    #[arg(short, long, action, default_value_t = true)]
    start_child: bool,

    /// количество килобайт в виде степени двойки.
    /// Пример: (2 ^ <val>) * 1024
    #[arg(short, long, action, default_value_t = 10)]
    kb_pow_max: u32,
}

////////////////////////////////////////////////////////////////////////////////

fn main() {
    // Парсим параметры
    let args = Cli::parse();

    // Выбираем нужный метод тестирования
    match args.method {
        // Просто в stdout
        Method::Stdout => {
            stdout(&args);
        }
        // Общая память какая-то
        Method::Shmem => {
            shared_memory(&args);
        }
        // TCP
        Method::Tcp => {
            tcp(&args);
        }
        // UDP
        Method::Udp => {
            udp(&args);
        }
        Method::Iceoryx => {
            for data_size in 1..=args.kb_pow_max {
                let data_size = 2u64.pow(data_size as u32) as usize * KB;
                let mut runner = IceoryxRunner::new(true, data_size);

                core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
                cpu_warmup();

                runner.run(args.number, true);
            }
        }
        Method::Mmap => {
            for data_size in 1..=args.kb_pow_max {
                let data_size = 2u64.pow(data_size as u32) as usize * KB;
                let mut runner = MmapRunner::new(true, data_size);

                core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
                cpu_warmup();

                runner.run(args.number, true);
            }
        }
        Method::Unixstream => {
            for data_size in 1..=args.kb_pow_max {
                let data_size = 2u64.pow(data_size as u32) as usize * KB;
                let mut runner = UnixStreamRunner::new(true, data_size);

                core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
                cpu_warmup();

                runner.run(args.number, true);
            }
        }
        Method::Unixdatagram => {
            for data_size in 1..=args.kb_pow_max {
                let data_size = 2u64.pow(data_size as u32) as usize * KB;
                let mut runner = UnixDatagramRunner::new(true, data_size);

                core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
                cpu_warmup();

                runner.run(args.number, true);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

fn stdout(args: &Cli) {
    // Итерируемся в диапазоне от 1 до максимума килобайт
    for data_size in 1_u32..=args.kb_pow_max {
        // Возводим в степень для получения количества байт
        let data_size = 2_usize.pow(data_size) * KB;

        // Запускаем работу с пайпом для нужного размера данных
        let mut pr = PipeRunner::new(data_size);

        // Дополнительно прицепимся к конкретному ядру в системе
        core_affinity::set_for_current(core_affinity::CoreId { id: 1 });

        // Делаем прогрес на всякий случай для того, чтобы у нас процессор и кеши прогрелись
        // и перешли в рабочий и производительный режимы
        cpu_warmup();

        // Запускаем
        pr.run(args.number, true);
    }
}

////////////////////////////////////////////////////////////////////////////////

fn shared_memory(args: &Cli) {
    for data_size in 1..=args.kb_pow_max {
        let data_size = 2u64.pow(data_size as u32) as usize * KB;
        let mut runner = ShmemRunner::new(args.start_child, data_size);

        core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
        cpu_warmup();

        runner.run(args.number, true);
    }
}

////////////////////////////////////////////////////////////////////////////////

fn tcp(args: &Cli) {
    for data_size in 1..=args.kb_pow_max {
        let data_size = 2u64.pow(data_size as u32) as usize * KB;
        let mut runner = TcpRunner::new(args.start_child, true, data_size);

        core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
        cpu_warmup();

        runner.run(args.number, true);
    }
}

////////////////////////////////////////////////////////////////////////////////

fn udp(args: &Cli) {
    for data_size in 1..=args.kb_pow_max {
        let data_size = 2u64.pow(data_size as u32) as usize * KB;
        let mut runner = UdpRunner::new(true, data_size);

        core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
        cpu_warmup();

        runner.run(args.number, true);
        drop(runner);
    }
}
