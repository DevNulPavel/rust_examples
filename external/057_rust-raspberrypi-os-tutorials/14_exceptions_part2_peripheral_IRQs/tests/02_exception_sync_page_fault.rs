// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2019-2020 Andre Richter <andre.o.richter@gmail.com>

//! Page faults must result in synchronous exceptions.

#![feature(format_args_nl)]
#![no_main]
#![no_std]

/// Overwrites libkernel's `panic_wait::_panic_exit()` with the QEMU-exit version.
///
/// Reaching this code is a success, because it is called from the synchronous exception handler,
/// which is what this test wants to achieve.
///
/// It also means that this integration test can not use any other code that calls panic!() directly
/// or indirectly.
mod panic_exit_success;

use libkernel::{bsp, cpu, exception, memory, println};

#[no_mangle]
unsafe fn kernel_init() -> ! {
    use memory::mmu::interface::MMU;

    bsp::console::qemu_bring_up_console();

    println!("Testing synchronous exception handling by causing a page fault");
    println!("-------------------------------------------------------------------\n");

    exception::handling_init();

    if let Err(string) = memory::mmu::mmu().init() {
        println!("MMU: {}", string);
        cpu::qemu_exit_failure()
    }

    println!("Writing beyond mapped area to address 9 GiB...");
    let big_addr: u64 = 9 * 1024 * 1024 * 1024;
    core::ptr::read_volatile(big_addr as *mut u64);

    // If execution reaches here, the memory access above did not cause a page fault exception.
    cpu::qemu_exit_failure()
}
