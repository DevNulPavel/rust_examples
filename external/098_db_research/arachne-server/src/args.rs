use bytesize::ByteSize;
use clap::Parser;

////////////////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Адрес и порт для запуска TCP сервера
    #[arg(short, long, default_value = "127.0.0.1:50051")]
    pub bind: String,

    /// Адрес и порт для запуска TCP сервера метрик
    #[arg(short, long, default_value = "127.0.0.1:9090")]
    pub metrics_addr: String,

    /// Путь к директории базы данных
    #[arg(short, long)]
    pub db_path: String,

    /// Размер кэша базы данных
    #[arg(short, long, value_parser = parse_bytesize, default_value_t = ByteSize::gb(4))]
    pub cache_mb: ByteSize,

    /// Размер журнала lsm дерева
    #[arg(short, long, value_parser = parse_bytesize, default_value_t = ByteSize::gb(1))]
    pub max_journaling_size: ByteSize,

    /// Лимит пользователей в очереди Prefetch на каждый селектор
    #[arg(short, long, default_value_t = 50_000)]
    pub prefetch_limit: usize,
}

////////////////////////////////////////////////////////////////////////////////

fn parse_bytesize(s: &str) -> Result<ByteSize, String> {
    s.parse::<ByteSize>().map_err(|e| e.to_string())
}
