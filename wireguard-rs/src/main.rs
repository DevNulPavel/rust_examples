#![cfg_attr(feature = "unstable", feature(test))]

extern crate alloc;

#[cfg(feature = "profiler")]
extern crate cpuprofiler;

#[cfg(feature = "profiler")]
use cpuprofiler::PROFILER;

mod configuration;
mod platform;
mod wireguard;

mod util;

use std::env;
use std::process::exit;
use std::thread;

use configuration::Configuration;

use platform::tun::{PlatformTun, Status};
use platform::uapi::{BindUAPI, PlatformUAPI};
use platform::*;

use wireguard::WireGuard;

#[cfg(feature = "profiler")]
fn profiler_stop() {
    println!("Stopping profiler");
    PROFILER.lock().unwrap().stop().unwrap();
}

#[cfg(not(feature = "profiler"))]
fn profiler_stop() {}

#[cfg(feature = "profiler")]
fn profiler_start(name: &str) {
    use std::path::Path;

    // Пытаемся в цикле создать объект профайлера для нужного файлика, которого еще нету
    let mut n = 0;
    loop {
        let path = format!("./{}-{}.profile", name, n);
        if !Path::new(path.as_str()).exists() {
            println!("Starting profiler: {}", path);
            PROFILER.lock().unwrap().start(path).unwrap();
            break;
        };
        n += 1;
    }
}

fn main() {
    // Парсим аргументы командной строки
    let mut name = None;
    let mut drop_privileges = true;
    let mut foreground = false;
    let mut args = env::args();

    // Пропускаем имя текущего приложения и обходим все элементы выставляя флаги нужные
    args.next();
    for arg in args {
        match arg.as_str() {
            "--foreground" | "-f" => {
                foreground = true;
            }
            "--disable-drop-privileges" => {
                drop_privileges = false;
            }
            dev => name = Some(dev.to_owned()),
        }
    }

    // Имя девайса у нас обязательно
    let name = match name {
        None => {
            eprintln!("No device name supplied");
            exit(-1);
        }
        Some(name) => name,
    };

    // Создаем сокет для работы с тунелем на данной платформе
    let uapi = plt::UAPI::bind(name.as_str()).unwrap_or_else(|e| {
        eprintln!("Failed to create UAPI listener: {}", e);
        exit(-2);
    });

    // Создаем устройство туннеля для текущей платформы локальной
    let (mut readers, writer, status) = plt::Tun::create(name.as_str()).unwrap_or_else(|e| {
        eprintln!("Failed to create TUN device: {}", e);
        exit(-3);
    });

    // Смена привилегий пользователя на nobody
    if drop_privileges {
        match util::drop_privileges() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to drop privileges: {}", e);
                exit(-4);
            }
        }
    }

    // Включаем режим демона
    if !foreground {
        match util::daemonize() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to daemonize: {}", e);
                exit(-5);
            }
        }
    }

    // Запускаем систему логирования
    env_logger::builder()
        .try_init()
        .expect("Failed to initialize event logger");

    log::info!("Starting {} WireGuard device.", name);

    // Запускаем профайлер если нужно
    #[cfg(feature = "profiler")]
    profiler_start(name.as_str());

    // Создаем Wireguard девайс с писателем
    let wg: WireGuard<plt::Tun, plt::UDP> = WireGuard::new(writer);

    // add all Tun readers
    while let Some(reader) = readers.pop() {
        wg.add_tun_reader(reader);
    }

    // wrap in configuration interface
    let cfg = configuration::WireGuardConfig::new(wg.clone());

    // start Tun event thread
    {
        let cfg = cfg.clone();
        let mut status = status;
        thread::spawn(move || loop {
            match status.event() {
                Err(e) => {
                    log::info!("Tun device error {}", e);
                    profiler_stop();
                    exit(0);
                }
                Ok(tun::TunEvent::Up(mtu)) => {
                    log::info!("Tun up (mtu = {})", mtu);
                    let _ = cfg.up(mtu); // TODO: handle
                }
                Ok(tun::TunEvent::Down) => {
                    log::info!("Tun down");
                    cfg.down();
                }
            }
        });
    }

    // start UAPI server
    thread::spawn(move || loop {
        // accept and handle UAPI config connections
        match uapi.connect() {
            Ok(mut stream) => {
                let cfg = cfg.clone();
                thread::spawn(move || {
                    configuration::uapi::handle(&mut stream, &cfg);
                });
            }
            Err(err) => {
                log::info!("UAPI connection error: {}", err);
                profiler_stop();
                exit(-1);
            }
        }
    });

    // block until all tun readers closed
    wg.wait();
    profiler_stop();
}
