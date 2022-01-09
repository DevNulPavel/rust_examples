use std::net::SocketAddr;
use structopt::clap::AppSettings;
use structopt::StructOpt;

/// tcp connection hijacker, rust rewrite of shijack
#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp], after_help=r#"The original shijack in C was written by spwny and released around 2001.
shijack credited cyclozine for inspiration."#)]
pub struct Args {
    /// Интерфейс, который мы собираемся ломать
    pub interface: String,
    /// Исходный адрес
    pub src: SocketAddr,
    /// Целевой ардес
    pub dst: SocketAddr,
    /// Начальный номер seq
    #[structopt(long)]
    pub seq: Option<u32>,
    /// Начальный номер ack
    #[structopt(long)]
    pub ack: Option<u32>,
    /// Сбрасываем соединение вместо взлома
    #[structopt(short = "r", long)]
    pub reset: bool,
    /// Рассинхронизуем исходное соединение с помощью отправки 1kb нулевых байтов
    #[structopt(short = "0", long)]
    pub send_null: bool,
    /// Отключаем подробный вывод данных
    #[structopt(short, long, parse(from_occurrences))]
    pub quiet: u8,
}
