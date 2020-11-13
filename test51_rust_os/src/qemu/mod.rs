use core::{
    convert::{
        Into,
        From
    }
};
use x86_64::{
    instructions::{
        port::{
            Port
        }
    }
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

impl From<QemuExitCode> for u32 {
    fn from(val: QemuExitCode) -> u32 {
        val as u32
    }
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        // Создаем новый порт, который мы указали в конфигурировании в Cargo.toml
        let mut port = Port::new(0xf4);
        // Пишем в порт код
        let code_val: u32 = exit_code.into();
        port.write(code_val);
    }
}