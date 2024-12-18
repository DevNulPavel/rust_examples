// SPDX-License-Identifier: Apache-2.0
#![cfg(not(miri))]
#![cfg(not(feature = "gdb"))]

use std::io::Read;
use std::mem::{size_of, MaybeUninit};
use std::path::Path;
use std::slice::from_raw_parts_mut;

use process_control::Output;
use serial_test::serial;

mod common;
use common::{run_test, CRATE, OUT_DIR, TEST_BINS_OUT};

/// Returns a handle to a child process through which output (stdout, stderr) can
/// be accessed.
pub fn run_c_test<'a>(
    bin: &str,
    status: i32,
    input: impl Into<Option<&'a [u8]>>,
    expected_stdout: impl Into<Option<&'a [u8]>>,
    expected_stderr: impl Into<Option<&'a [u8]>>,
) -> Output {
    let bin = Path::new(CRATE).join(OUT_DIR).join(TEST_BINS_OUT).join(bin);
    run_test(bin, status, input, expected_stdout, expected_stderr)
}

fn read_item<T: Copy>(mut rdr: impl Read) -> std::io::Result<T> {
    let mut item = MaybeUninit::uninit();
    let ptr = item.as_mut_ptr() as *mut u8;
    let buf = unsafe { from_raw_parts_mut(ptr, size_of::<T>()) };
    rdr.read_exact(buf)?;
    Ok(unsafe { item.assume_init() })
}

#[test]
#[serial]
fn exit_zero() {
    run_c_test("exit_zero", 0, None, None, None);
}

#[test]
#[serial]
fn exit_one() {
    run_c_test("exit_one", 1, None, None, None);
}

#[test]
#[serial]
fn clock_gettime() {
    use libc::{clock_gettime, CLOCK_MONOTONIC};

    // Get the time from inside the keep.
    let stdout = run_c_test("clock_gettime", 0, None, None, None).stdout;
    let theirs: libc::timespec = read_item(stdout.as_slice()).unwrap();

    // Get the time from outside the keep.
    let ours = unsafe {
        let mut ts = MaybeUninit::uninit();
        assert_eq!(0, clock_gettime(CLOCK_MONOTONIC, ts.as_mut_ptr()));
        ts.assume_init()
    };

    // Validate that the difference in time is minor...
    const NSEC_PER_SEC: libc::c_long = 1_000_000_000;
    const MAX_SEC: libc::c_long = 60;

    let sec = ours.tv_sec - theirs.tv_sec;
    assert!(sec >= 0);
    assert!(sec < MAX_SEC);

    let nsec = sec * NSEC_PER_SEC + ours.tv_nsec - theirs.tv_nsec;
    assert!(nsec >= 0);
    assert!(nsec < MAX_SEC * NSEC_PER_SEC);
}

#[test]
#[serial]
fn close() {
    run_c_test("close", 0, None, None, None);
}

#[test]
#[serial]
fn write_stdout() {
    run_c_test("write_stdout", 0, None, &b"hi\n"[..], None);
}

#[cfg(not(feature = "dbg"))]
#[test]
#[serial]
// v0.1.0 KEEP-CONFIG HACK: logging is hardcoded to send output to stderr,
// which clobbers the output here. Skip this test until we have a way to
// disable log output and/or send it somewhere other than stderr.
#[ignore]
fn write_stderr() {
    run_c_test("write_stderr", 0, None, None, &b"hi\n"[..]);
}

#[test]
#[serial]
// FIXME this should not be ignored, this was applied as part
// of a commit that must be reverted and implemented properly.
#[ignore]
fn write_emsgsize() {
    run_c_test("write_emsgsize", 0, None, None, None);
}

#[test]
#[serial]
fn read() {
    const INPUT: &[u8; 12] = b"hello world\n";
    run_c_test("read", 0, INPUT.as_slice(), INPUT.as_slice(), None);
}

#[test]
#[serial]
fn readv() {
    const INPUT: &[u8; 36] = b"hello, worldhello, worldhello, world";
    run_c_test("readv", 0, INPUT.as_slice(), INPUT.as_slice(), None);
}

#[test]
#[serial]
fn uname() {
    run_c_test("uname", 0, None, None, None);
}

#[test]
#[serial]
fn read_udp() {
    // The maximum UDP message size is 65507, as determined by the following formula:
    // 0xffff - (sizeof(minimal IP Header) + sizeof(UDP Header)) = 65535-(20+8) = 65507
    const MAX_UDP_PACKET_SIZE: usize = 65507;

    let mut input: Vec<u8> = Vec::with_capacity(MAX_UDP_PACKET_SIZE);
    for i in 0..input.capacity() {
        input.push(i as _);
    }
    let input = input.as_slice();

    run_c_test("read_udp", 0, input, input, None);
}

#[cfg_attr(not(host_can_test_attestation), ignore)]
#[test]
#[serial]
fn get_att() {
    run_c_test("get_att", 0, None, None, None);
}

#[cfg(feature = "backend-sgx")]
#[cfg_attr(any(not(host_can_test_sgx), not(host_can_test_attestation)), ignore)]
#[test]
#[serial]
fn sgx_get_att_quote() {
    run_c_test("sgx_get_att_quote", 0, None, None, None);
}

#[cfg(feature = "backend-sgx")]
#[cfg_attr(any(not(host_can_test_sgx), not(host_can_test_attestation)), ignore)]
#[test]
#[serial]
fn sgx_get_att_quote_size() {
    run_c_test("sgx_get_att_quote_size", 0, None, None, None);
}

#[test]
#[serial]
fn getuid() {
    run_c_test("getuid", 0, None, None, None);
}

#[test]
#[serial]
fn geteuid() {
    run_c_test("geteuid", 0, None, None, None);
}

#[test]
#[serial]
fn getgid() {
    run_c_test("getgid", 0, None, None, None);
}

#[test]
#[serial]
fn getegid() {
    run_c_test("getegid", 0, None, None, None);
}

#[test]
#[serial]
fn socket() {
    run_c_test("socket", 0, None, None, None);
}

#[test]
#[serial]
fn bind() {
    run_c_test("bind", 0, None, None, None);
}

#[test]
#[serial]
fn listen() {
    run_c_test("listen", 0, None, None, None);
}
