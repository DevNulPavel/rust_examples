mod http;
mod quic;
mod tcp;

use assert_cmd::prelude::CommandCargoExt;
use iggy::utils::byte_size::IggyByteSize;
use integration::test_server::{Transport, TEST_VERBOSITY_ENV_VAR};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    process::{Command, Stdio},
    thread::panicking,
};
use uuid::Uuid;

const BENCH_FILES_PREFIX: &str = "bench_";
const MESSAGE_BATCHES: u64 = 100;
const MESSAGES_PER_BATCH: u64 = 100;
const DEFAULT_NUMBER_OF_STREAMS: u64 = 10;

pub fn run_bench_and_wait_for_finish(
    server_addr: &str,
    transport: Transport,
    bench: &str,
    amount_of_data_to_process: IggyByteSize,
) {
    let mut command = Command::cargo_bin("iggy-bench").unwrap();

    let mut stderr_file_path = None;
    let mut stdout_file_path = None;

    if std::env::var(TEST_VERBOSITY_ENV_VAR).is_err() {
        let stderr_file = get_random_path();
        let stdout_file = get_random_path();
        stderr_file_path = Some(stderr_file);
        stdout_file_path = Some(stdout_file);
    }

    // Calculate message size based on input
    let total_bytes_to_process_per_stream =
        amount_of_data_to_process.as_bytes_u64() / DEFAULT_NUMBER_OF_STREAMS;
    let messages_total = MESSAGES_PER_BATCH * MESSAGE_BATCHES;
    let message_size = total_bytes_to_process_per_stream / messages_total;

    command.args([
        bench,
        "--messages-per-batch",
        &MESSAGES_PER_BATCH.to_string(),
        "--message-batches",
        &MESSAGE_BATCHES.to_string(),
        "--message-size",
        &message_size.to_string(),
        &format!("{}", transport),
        "--server-address",
        server_addr,
    ]);

    // By default, all iggy-bench logs are redirected to files,
    // and dumped to stderr when test fails. With IGGY_TEST_VERBOSE=1
    // logs are dumped to stdout during test execution.
    if std::env::var(TEST_VERBOSITY_ENV_VAR).is_ok() {
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
    } else {
        command.stdout(File::create(stdout_file_path.as_ref().unwrap()).unwrap());
        stdout_file_path = Some(
            fs::canonicalize(stdout_file_path.unwrap())
                .unwrap()
                .display()
                .to_string(),
        );

        command.stderr(File::create(stderr_file_path.as_ref().unwrap()).unwrap());
        stderr_file_path = Some(
            fs::canonicalize(stderr_file_path.unwrap())
                .unwrap()
                .display()
                .to_string(),
        );
    }

    let mut child = command.spawn().unwrap();
    let result = child.wait().unwrap();

    // Cleanup
    if let Ok(output) = child.wait_with_output() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(stderr_file_path) = &stderr_file_path {
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(stderr_file_path)
                .unwrap()
                .write_all(stderr.as_bytes())
                .unwrap();
        }

        if let Some(stdout_file_path) = &stdout_file_path {
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(stdout_file_path)
                .unwrap()
                .write_all(stdout.as_bytes())
                .unwrap();
        }
    } else {
        panic!("Failed to get output from iggy-bench");
    }

    // TODO: fix this, it needs to be called in Drop
    if panicking() {
        if let Some(stdout_file_path) = &stdout_file_path {
            eprintln!(
                "Iggy bench stdout:\n{}",
                fs::read_to_string(stdout_file_path).unwrap()
            );
        }

        if let Some(stderr_file_path) = &stderr_file_path {
            eprintln!(
                "Iggy bench stderr:\n{}",
                fs::read_to_string(stderr_file_path).unwrap()
            );
        }
    }

    if let Some(stdout_file_path) = &stdout_file_path {
        fs::remove_file(stdout_file_path).unwrap();
    }
    if let Some(stderr_file_path) = &stderr_file_path {
        fs::remove_file(stderr_file_path).unwrap();
    }

    assert!(result.success());
}

pub fn get_random_path() -> String {
    format!("{}{}", BENCH_FILES_PREFIX, Uuid::now_v7().to_u128_le())
}
