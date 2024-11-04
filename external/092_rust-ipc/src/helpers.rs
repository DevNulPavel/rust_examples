////////////////////////////////////////////////////////////////////////////////

use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::Instant;
use std::{path::PathBuf, time::Duration};

////////////////////////////////////////////////////////////////////////////////

pub(crate) fn generate_random_data(data_size: usize, seed: u64) -> Vec<u8> {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = StdRng::seed_from_u64(seed);

    (0..data_size)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx]
        })
        .collect()
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) fn get_payload(data_size: usize) -> (Vec<u8>, Vec<u8>) {
    let request_data = generate_random_data(data_size, 1);
    let response_data = generate_random_data(data_size, 2);
    (request_data, response_data)
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) fn cpu_warmup() {
    let warmup = Instant::now();
    loop {
        if warmup.elapsed() > Duration::from_millis(1000) {
            break;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) struct ExecutionResult {
    name: String,
    elapsed: Duration,
    cycles: usize,
}

impl ExecutionResult {
    fn new(name: String, elapsed: Duration, cycles: usize) -> ExecutionResult {
        ExecutionResult {
            name,
            elapsed,
            cycles,
        }
    }

    fn print_info(&self) {
        let duration = humantime::Duration::from(self.elapsed);
        let ps = 1_000_000f32 * (self.cycles as f32) / (duration.as_micros() as f32);
        let per_op =
            humantime::Duration::from(Duration::from_nanos((1_000_000_000f32 / ps) as u64));
        println!(
            "IPC method - {}\n\t{} cycles completed in {} \n\t{} per second\n\t{} per operation",
            self.name, self.cycles, duration, ps, per_op
        );
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Получаем путь к исполняемому файлику
/// с нужным именем на текущей системе
pub(crate) fn executable_path(name: &str) -> PathBuf {
    // Получаем имя исполняемого файлика
    #[cfg(target_os = "windows")]
    let exe = name.to_owned() + ".exe";
    #[cfg(target_family = "unix")]
    let exe = name.to_owned();


    // Получаем полный путь к файлику
    #[cfg(debug_assertions)]
    let out = PathBuf::from("./target/debug/").join(exe);
    #[cfg(not(debug_assertions))]
    let out = PathBuf::from("./target/release/").join(exe);

    out
}
