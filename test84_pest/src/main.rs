mod csv;
mod ident;
mod simple;
mod ini;
mod json;

use eyre::Context;
// use log::{debug, warn};
use std::env;

fn init_log() -> Result<(), eyre::Error> {
    const LOG_VAR: &str = "RUST_LOG";
    if env::var(LOG_VAR).is_err() {
        env::set_var(LOG_VAR, "debug");
    }
    pretty_env_logger::try_init().wrap_err("Logger setup failed")?;

    Ok(())
}

fn main() -> Result<(), eyre::Error> {
    // Поддержка подробной инфы по ошибкам
    color_eyre::install()?;

    // Инициализируем log
    init_log()?;

    // ident::parse_ident();
    // csv::parse_csv_1();
    // csv::parse_csv_2();
    // simple::parse_simple();
    // ini::parse_ini();
    json::parse_json();

    Ok(())
}
