extern crate bus;
extern crate crossbeam_channel;

use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::time::Instant;

use args::Args;
use config::*;
use ignore::*;
use intermediate_config::IntermediateConfig;
use sync::{PullMode};
use time::*;

mod args;
mod config;
mod intermediate_config;
mod ignore;
mod remote_command;
mod sync;
mod time;

// TODO use Reactive Streams instead of Channels.

fn main() {
    // Время старта
    let total_start = Instant::now();

    println!(":: Mainframer v{}\n", env!("CARGO_PKG_VERSION"));

    // Аргументы коммандной строки без имени утилиты
    let raw_args: Vec<String> = env::args().skip(1).collect();

    // Парсим аргументы с помощью модуля утилит
    let args = match Args::parse(raw_args.as_ref()) {
        Err(message) => exit_with_error(&message, 1),
        Ok(value) => value,
    };

    // Абсолютный путь к текущей папке
    let local_dir_absolute_path = match env::current_dir() {
        Err(_) => exit_with_error("Could not resolve working directory, make sure it exists and user has enough permissions to work with it.", 1),
        Ok(value) => fs::canonicalize(value).unwrap()
    };

    // Путь к файлу конфига в директории
    let mut config_file = local_dir_absolute_path.clone();
    config_file.push(".mainframer/config.yml");

    // Мержим конфиг приложения по-умолчанию с тем, что получили из файлика
    let config = match merge_configs(&config_file) {
        Err(error) => exit_with_error(&error, 1),
        Ok(value) => value
    };

    // TODO: Что надо игнорировать?
    let ignore = Ignore::from_working_dir(&local_dir_absolute_path.clone());

    println!("Pushing...");

    // Пушим изменения на сервер
    match sync::push(&local_dir_absolute_path, &config, &ignore) {
        Err(err) => exit_with_error(&format!("Push failed: {}, took {}", err.message, format_duration(err.duration)), 1),
        Ok(ok) => println!("Push done: took {}.\n", format_duration(ok.duration)),
    }

    // В зависимости от метода пулинга - выводим сообщение
    match config.pull.mode {
        PullMode::Serial => println!("Executing command on remote machine...\n"),
        PullMode::Parallel(_) => println!("Executing command on remote machine (pulling in parallel)...\n")
    }

    // Выполняем удаленную команду
    let mut remote_command_readers = remote_command::execute_remote_command(
        args.command.clone(),
        config.clone(),
        sync::project_dir_on_remote_machine(&local_dir_absolute_path.clone()),
        2
    );

    // Получаем канал-получатель
    let pull_finished_rx = sync::pull(&local_dir_absolute_path, config.clone(), ignore, &config.pull.mode, remote_command_readers.pop().unwrap());

    // Получаем результат выполнения команды
    let remote_command_result = remote_command_readers
        .pop()
        .unwrap()
        .recv()
        .unwrap();

    match remote_command_result {
        Err(ref err) => eprintln!("\nExecution failed: took {}.\nPulling...", format_duration(err.duration)),
        Ok(ref ok) => println!("\nExecution done: took {}.\nPulling...", format_duration(ok.duration))
    }

    // Получаем из канала результат пулинга данных с сервера
    let pull_result = pull_finished_rx
        .recv()
        .expect("Could not receive remote_to_local_sync_result");

    // Сколько это всего заняло времени?
    let total_duration = total_start.elapsed();

    // Результат пулинга
    match pull_result {
        Err(ref err) => eprintln!("Pull failed: {}, took {}.", err.message, format_duration(err.duration)),
        Ok(ref ok) => println!("Pull done: took {}", format_duration(ok.duration)),
    }

    // Результат исполнения команды на сервере
    if remote_command_result.is_err() || pull_result.is_err() {
        exit_with_error(&format!("\nFailure: took {}.", format_duration(total_duration)), 1);
    } else {
        println!("\nSuccess: took {}.", format_duration(total_duration));
    }
}

fn exit_with_error(message: &str, code: i32) -> ! {
    if !message.is_empty() {
        eprintln!("{}", message);
    }
    process::exit(code);
}

fn merge_configs(project_config_file: &Path) -> Result<Config, String> {
    let default_push_compression = 3;
    let default_pull_compression = 1;
    let default_pull_mode = PullMode::Serial; // TODO: consider making Parallel the default mode.

    Ok(match IntermediateConfig::from_file(project_config_file) {
        Err(message) => return Err(message),
        Ok(intermediate_config) => {
            let remote = match intermediate_config.remote {
                None => return Err(String::from("Configuration must specify 'remote'")),
                Some(value) => value
            };

            Config {
                remote: Remote {
                    host: match remote.host {
                        None => return Err(String::from("Configuration must specify 'remote.host'")),
                        Some(value) => value
                    },
                },
                push: match intermediate_config.push {
                    None => Push {
                        compression: default_push_compression,
                    },
                    Some(push) => Push {
                        compression: push.compression.unwrap_or(default_push_compression),
                    }
                },
                pull: match intermediate_config.pull {
                    None => Pull {
                        compression: default_pull_compression,
                        mode: default_pull_mode,
                    },
                    Some(pull) => Pull {
                        compression: pull.compression.unwrap_or(default_pull_compression),
                        mode: pull.mode.unwrap_or(default_pull_mode),
                    }
                },
            }
        }
    })
}

