use lambda_runtime::{handler_fn, run, Context};
use serde_json::{json, Value};

async fn func(event: Value, _: Context) -> Result<Value, lambda_runtime::Error> {
    let first_name = event["firstName"].as_str().unwrap_or("world");

    Ok(json!({ "message": format!("Hello, {}!", first_name) }))
}

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    // Делаем так, чтобы ошибки захватывали backtrace
    // color_eyre::install().expect("Backtraces setup disabled");

    // Настройка логирования
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init()
        .expect("Log system init failed");

    run(handler_fn(func)).await
}
