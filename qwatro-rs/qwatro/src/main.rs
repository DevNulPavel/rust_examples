mod cli;

use crate::cli::port_scan::PortScanArgs;
use crate::cli::proxy::ProxyArgs;
use crate::cli::{Cli, Commands};
use clap::Parser;
use futures::StreamExt;
use qwatro_port_scanner::builder::PortScannerBuilder;
use qwatro_proxy::run_proxy;
use qwatro_proxy::tcp::TcpProxy;
use std::env;
use std::time::Duration;
use tokio::signal;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "debug");
    }

    env_logger::init();

    let cli = Cli::parse();

    // Глобальный `CancellationToken`, который будет передаваться в компоненты приложения
    let ct = CancellationToken::new();
    tokio::spawn(shutdown(ct.clone()));

    match cli.command {
        Commands::PS(args) => scan(ct, args).await,
        Commands::Proxy(args) => proxy(ct, args).await,
    };
}

/// Запуск сканирования портов
async fn scan(ct: CancellationToken, args: PortScanArgs) {
    let mut builder = PortScannerBuilder::new()
        .ip(args.ip)
        .port_range(args.port_range)
        .max_tasks(args.max_tasks);

    if args.tcp {
        builder = builder.tcp(Some(Duration::from_millis(args.tcp_resp_timeout)));
    }

    let scanner = builder.build();

    // Запускаем сканер
    let mut stream = scanner.run(ct);

    // Выводим элементы потока результата сканирования в stdout
    while let Some(res) = stream.next().await {
        log::info!("{}/{:#?}", res.addr, res.ty);
    }
}

/// Запуск проксирования
async fn proxy(ct: CancellationToken, args: ProxyArgs) {
    match args {
        ProxyArgs::TCP(args) => {
            run_proxy(
                ct.clone(),
                TcpProxy,
                args.host_to_server.into_iter().collect(),
            )
            .await
            .unwrap();
        }
    };

    ct.cancelled().await;
}

/// Future, которая будет ожидать сигнала завершения приложения, после чего завершать `CancellationToken`
async fn shutdown(ct: CancellationToken) {
    signal::ctrl_c().await.unwrap();
    log::info!("got shutdown signal");
    ct.cancel();
}
