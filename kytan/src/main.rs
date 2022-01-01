// Copyright 2016-2017 Chang Lan
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod device;
mod utils;
mod network;
mod packet;
mod cli;

use std::sync::atomic::Ordering;
use env_logger;
use libc;

extern "C" fn handle_signal(_: libc::c_int) {
    // Выставляем флаг прерывании работы сети
    // Relaxed значит, что мы никак кеши не сбрасываем принудительно, здесь нам не важно
    network::INTERRUPTED.store(true, Ordering::Relaxed);
}

fn main() {
    // Настраиваем логирование
    env_logger::init();

    // Запускаем только под root
    if !utils::is_root() {
        panic!("Please run as root");
    }

    // Установка обработки сигналов
    unsafe {
        libc::signal(libc::SIGINT, handle_signal as libc::sighandler_t);
        libc::signal(libc::SIGTERM, handle_signal as libc::sighandler_t);
    }

    // Парсим параметры
    match cli::get_args().unwrap() {
        cli::Args::Client(client) => network::connect(&client.remote_addr, client.port, client.default_route, &client.key),
        cli::Args::Server(server) => network::serve(server.port, &server.key, server.dns),
    }

    println!("SIGINT/SIGTERM captured. Exit.");
}
