use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::TryRecvError::*;
use std::thread;
use std::time::{Duration, Instant};

use bus::BusReader;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use crossbeam_channel::unbounded;

use config::Config;
use ignore::Ignore;
use remote_command::{RemoteCommandOk, RemoteCommandErr};

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Clone)]
pub struct PushOk {
    pub duration: Duration,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PushErr {
    pub duration: Duration,
    pub message: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Clone)]
pub enum PullMode {
    /// Serial, after remote command execution.
    Serial,

    /// Parallel to remote command execution.
    /// First parameter is pause between pulls.
    Parallel(Duration),
}

#[derive(Debug, PartialEq, Clone)]
pub struct PullOk {
    pub duration: Duration,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PullErr {
    pub duration: Duration,
    pub message: String,
}

////////////////////////////////////////////////////////////////////////////////////////////////

// Пушим на сервер данные
pub fn push(local_dir_absolute_path: &Path, config: &Config, ignore: &Ignore) -> Result<PushOk, PushErr> {
    // Время старта
    let start_time = Instant::now();

    // Команда rsync
    let mut command = Command::new("rsync");

    // Параметры для rsync
    command
        .arg("--archive")
        .arg("--delete")
        // Create (if not exists) project dir on remote machine.
        .arg(format!("--rsync-path=mkdir -p {} && rsync", project_dir_on_remote_machine(local_dir_absolute_path)))
        .arg(format!("--compress-level={}", config.push.compression));

    apply_exclude_from(&mut command, &ignore.common_ignore_file);
    apply_exclude_from(&mut command, &ignore.local_ignore_file);

    command
        .arg("--rsh=ssh")
        .arg("./");

    // Параметры подключения
    command.arg(format!(
        "{remote_machine_name}:{project_dir_on_remote_machine}",
        remote_machine_name = config.remote.host,
        project_dir_on_remote_machine = project_dir_on_remote_machine(local_dir_absolute_path))
    );

    // Выполняем rsync команду
    match execute_rsync(&mut command) {
        Ok(_) => Ok(PushOk {
            duration: start_time.elapsed()
        }),        
        Err(reason) => Err(PushErr {
            duration: start_time.elapsed(),
            message: reason,
        }),
    }
}

/// Пуллим результат работы с сервера
pub fn pull(local_dir_absolute_path: &Path, 
            config: Config, 
            ignore: Ignore, 
            pull_mode: &PullMode, 
            remote_command_finished_signal: BusReader<Result<RemoteCommandOk, RemoteCommandErr>>) -> Receiver<Result<PullOk, PullErr>> {

    match pull_mode {
        PullMode::Serial => pull_serial(local_dir_absolute_path.to_path_buf(), 
                                        config, 
                                        ignore, 
                                        remote_command_finished_signal),
        PullMode::Parallel(pause_between_pulls) => pull_parallel(local_dir_absolute_path.to_path_buf(), 
                                                                 config, 
                                                                 ignore, 
                                                                 *pause_between_pulls, 
                                                                 remote_command_finished_signal)
    }
}

fn pull_serial(local_dir_absolute_path: PathBuf, 
               config: Config, 
               ignore: Ignore, 
               mut remote_command_finished_rx: BusReader<Result<RemoteCommandOk, RemoteCommandErr>>) -> Receiver<Result<PullOk, PullErr>> {

    // Создаем канал получения данных
    let (pull_finished_tx, pull_finished_rx): (Sender<Result<PullOk, PullErr>>, Receiver<Result<PullOk, PullErr>>) = unbounded();

    #[allow(unused_must_use)] // We don't handle remote_command_result, in any case we need to pull after it.
    thread::spawn(move || {
        // Получаем результат выполнения команды
        remote_command_finished_rx
            .recv()
            .expect("Could not receive remote_command_finished_rx");

        // Пулим результат работы с сервера
        let pull_result = _pull(local_dir_absolute_path.as_path(), &config, &ignore);

        // Отправляем наружу
        pull_finished_tx
            .send(pull_result)
            .expect("Could not send pull_finished signal");
    });

    pull_finished_rx
}

fn pull_parallel(local_dir_absolute_path: PathBuf, 
                 config: Config, 
                 ignore: Ignore, 
                 pause_between_pulls: Duration, 
                 mut remote_command_finished_signal: BusReader<Result<RemoteCommandOk, RemoteCommandErr>>) -> Receiver<Result<PullOk, PullErr>> {

    // Создаем канал данных
    let (pull_finished_tx, pull_finished_rx): (Sender<Result<PullOk, PullErr>>, Receiver<Result<PullOk, PullErr>>) = unbounded();
    let start_time = Instant::now();

    thread::spawn(move || {
        loop {
            // Пулим результат
            if let Err(pull_err) = _pull(local_dir_absolute_path.as_path(), &config, &ignore) {
                // Если вылезла ошибка - отправляем назад
                pull_finished_tx
                    .send(Err(pull_err)) // TODO handle code 24.
                    .expect("Could not send pull_finished signal");
                break;
            }

            // Ждем завершения команды
            match remote_command_finished_signal.try_recv() {
                // Получили
                Ok(remote_command_result) => {
                    // Успешно или нет выполнилась команда?
                    let remote_command_duration = match remote_command_result {
                        Err(err) => err.duration,
                        Ok(ok) => ok.duration
                    };
                    
                    // Пулим последний раз, чтобы удостовериться, что все прилетело точно
                    let duration = calculate_perceived_pull_duration(start_time.elapsed(), remote_command_duration);
                    match _pull(local_dir_absolute_path.as_path(), &config, &ignore) {
                        Ok(_) => { 
                            pull_finished_tx
                                .send(Ok(PullOk {
                                    duration
                                }))
                                .expect("Could not send pull finished signal (last iteration)")
                        },
                        Err(err) => {
                            pull_finished_tx
                                .send(Err(PullErr {
                                    duration,
                                    message: err.message
                                }))
                                .expect("Could not send pull finished signal (last iteration)")
                        }
                    }
                    
                    break;
                }
                // Не получили пока что
                Err(reason) => match reason {
                    Empty => {
                        // Так как там пусто - ждем еще рещультатов
                        thread::sleep(pause_between_pulls)
                    }
                    Disconnected => break,
                },
            }
        }
    });

    pull_finished_rx
}

/// Вызов фактического пулинга результата с помощью rsync
fn _pull(local_dir_absolute_path: &Path, config: &Config, ignore: &Ignore) -> Result<PullOk, PullErr> {
    let start_time = Instant::now();

    let mut command = Command::new("rsync");

    command
        .arg("--archive")
        .arg("--delete")
        .arg(format!("--compress-level={}", config.pull.compression));

    apply_exclude_from(&mut command, &ignore.common_ignore_file);
    apply_exclude_from(&mut command, &ignore.remote_ignore_file);

    command
        .arg("--rsh=ssh")
        .arg(format!("{remote_machine_name}:{project_dir_on_remote_machine}/",
                     remote_machine_name = config.remote.host,
                     project_dir_on_remote_machine = project_dir_on_remote_machine(local_dir_absolute_path)))
        .arg("./");

    match execute_rsync(&mut command) {
        Err(reason) => Err(PullErr {
            duration: start_time.elapsed(),
            message: reason
        }),
        Ok(_) => Ok(PullOk {
            duration: start_time.elapsed(),
        })
    }
}

pub fn project_dir_on_remote_machine(local_dir_absolute_path: &Path) -> String {
    format!("~/mainframer{}", local_dir_absolute_path.to_string_lossy())
}

fn apply_exclude_from(rsync_command: &mut Command, exclude_file: &Option<PathBuf>) {
    match exclude_file {
        Some(ref value) => {
            rsync_command.arg(format!("--exclude-from={}", value.to_string_lossy()));
        }
        None => ()
    };
}

/// Вызов rsync
fn execute_rsync(rsync: &mut Command) -> Result<(), String> {
    // Выполняем команду и получаем вывод
    let result = rsync.output();

    // Смотрим на полученный вывод
    match result {
        Ok(output) => match output.status.code() {
            // Смотрим код ошибки
            Some(status_code) => match status_code {
                0 => Ok(()),
                _ => {
                    let text = format!("rsync exit code '{exit_code}',\n\
                                        rsync stdout '{stdout}',\n\
                                        rsync stderr '{stderr}'.",
                                        exit_code = status_code,
                                        stdout = String::from_utf8_lossy(&output.stdout),
                                        stderr = String::from_utf8_lossy(&output.stderr));                 
                    Err(text)
                }
            },
            None => Err(String::from("rsync was terminated.")),
        },
        Err(_) => Err(String::from("Generic rsync error.")), // Rust doc doesn't really say when can an error occur.
    }
}

fn calculate_perceived_pull_duration(total_pull_duration: Duration, remote_command_duration: Duration) -> Duration {
    match total_pull_duration.checked_sub(remote_command_duration) {
        None => Duration::from_millis(0),
        Some(duration) => duration,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_perceived_pull_duration_equals() {
        assert_eq!(
            calculate_perceived_pull_duration(Duration::from_millis(10), Duration::from_millis(10)),
            Duration::from_millis(0)
        );
    }

    #[test]
    fn calculate_perceived_pull_duration_pull_longer_than_execution() {
        assert_eq!(
            calculate_perceived_pull_duration(Duration::from_secs(10), Duration::from_secs(8)),
            Duration::from_secs(2)
        );
    }

    #[test]
    fn calculate_perceived_pull_duration_pull_less_than_execution() {
        assert_eq!(
            calculate_perceived_pull_duration(Duration::from_secs(7), Duration::from_secs(9)),
            Duration::from_secs(0)
        );
    }
}
