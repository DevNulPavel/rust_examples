mod env_parameters;

use tokio::{
    runtime::{
        Builder
    }
};

async fn async_main() {
}

fn setup_logs(){
    // Активируем логирование и настраиваем уровни вывода
    // https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html
    #[cfg(debug_assertions)]
    {
        if !std::env::var("RUST_LOG").is_ok() {
            std::env::set_var("RUST_LOG", "uploader_app=trace");
        }
    }

    env_logger::init();
}

fn main() {
    // Активируем логирование и настраиваем уровни вывода
    setup_logs();

    // Запускаем асинхронный рантайм
    let mut runtime = Builder::default()
        .enable_io()
        .basic_scheduler()
        //.threaded_scheduler()
        //.core_threads(1)
        //.max_threads(2)
        .build()
        .expect("Tokio runtime create failed");

    runtime.block_on(async_main());

    // Dump the report to disk
    #[cfg(feature = "flame_it")]
    flame::dump_html(&mut std::fs::File::create("flame-graph.html").unwrap()).unwrap();
}
