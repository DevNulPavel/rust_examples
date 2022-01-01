// Copyright 2016-2020 Chang Lan
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

use std::{fs, process, io};
use libc::*;
use std::os::unix::io::{RawFd, AsRawFd};
use std::io::{Write, Read};

const MTU: &'static str = "1380";

#[cfg(target_os = "linux")]
use std::path;
#[cfg(target_os = "linux")]
const IFNAMSIZ: usize = 16;
#[cfg(target_os = "linux")]
const IFF_TUN: c_short = 0x0001;
#[cfg(target_os = "linux")]
const IFF_NO_PI: c_short = 0x1000;
#[cfg(all(target_os = "linux", target_env = "musl"))]
const TUNSETIFF: c_int = 0x400454ca; // TODO: use _IOW('T', 202, int)
#[cfg(all(target_os = "linux", not(target_env = "musl")))]
const TUNSETIFF: c_ulong = 0x400454ca; // TODO: use _IOW('T', 202, int)

#[cfg(target_os = "macos")]
use std::mem;
#[cfg(target_os = "macos")]
use std::os::unix::io::FromRawFd;
#[cfg(target_os = "macos")]
const AF_SYS_CONTROL: u16 = 2;
#[cfg(target_os = "macos")]
const AF_SYSTEM: u8 = 32;
#[cfg(target_os = "macos")]
const PF_SYSTEM: c_int = AF_SYSTEM as c_int;
#[cfg(target_os = "macos")]
const SYSPROTO_CONTROL: c_int = 2;
#[cfg(target_os = "macos")]
const UTUN_OPT_IFNAME: c_int = 2;
#[cfg(target_os = "macos")]
const CTLIOCGINFO: c_ulong = 0xc0644e03; // TODO: use _IOWR('N', 3, struct ctl_info)
#[cfg(target_os = "macos")]
const UTUN_CONTROL_NAME: &'static str = "com.apple.net.utun_control";

#[cfg(target_os = "linux")]
#[repr(C)]
pub struct ioctl_flags_data {
    pub ifr_name: [u8; IFNAMSIZ],
    pub ifr_flags: c_short,
}

/// Сишная струтурка данных для установки имени для сокета
#[cfg(target_os = "macos")]
#[repr(C)]
pub struct ctl_info {
    pub ctl_id: u32,
    pub ctl_name: [u8; 96],
}

/// Сишная структурка для установки параметров сокета
#[cfg(target_os = "macos")]
#[repr(C)]
pub struct sockaddr_ctl {
    pub sc_len: u8,
    pub sc_family: u8,
    pub ss_sysaddr: u16,
    pub sc_id: u32,
    pub sc_unit: u32,
    pub sc_reserved: [u32; 5],
}

pub struct Tun {
    handle: fs::File,
    if_name: String,
}

impl AsRawFd for Tun {
    fn as_raw_fd(&self) -> RawFd {
        self.handle.as_raw_fd()
    }
}

impl Tun {
    #[cfg(target_os = "linux")]
    pub fn create(name: u8) -> Result<Tun, io::Error> {
        // Получаем путь к Linux файлику девайса
        let path = path::Path::new("/dev/net/tun");
        // Открываем одновременно на чтение и запись с помощью опций
        let file = fs::OpenOptions::new().read(true).write(true).open(&path)?;

        // Описываем C-шную структурку
        let mut req = ioctl_flags_data {
            ifr_name: {
                let mut buffer = [0u8; IFNAMSIZ];
                let full_name = format!("tun{}", name);
                buffer[..full_name.len()].clone_from_slice(full_name.as_bytes());
                buffer
            },
            ifr_flags: IFF_TUN | IFF_NO_PI,
        };

        // Устанавливаем параметры
        let res = unsafe { ioctl(file.as_raw_fd(), TUNSETIFF, &mut req) }; // TUNSETIFF
        if res < 0 {
            return Err(io::Error::last_os_error());
        }

        // Создаем дескриптор туннеля
        let tun = Tun {
            handle: file,
            if_name: std::ffi::CStr::from_bytes_with_nul(&req.ifr_name)
                .expect("Socket name: CStr error")
                .to_str()
                .expect("Socket name: CStr to CString")
                .to_owned()
                .to_string(),
        };
        Ok(tun)
    }

    #[cfg(target_os = "macos")]
    pub fn create(name: u8) -> Result<Tun, io::Error> {
        // Создаем UDP сокет и оборачиваем в файлик
        let handle = {
            // Создаем UDP сокет с помощью C-шной функции
            let fd = unsafe { socket(PF_SYSTEM, SOCK_DGRAM, SYSPROTO_CONTROL) };
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            // Оборачиваем в Rust файлик
            unsafe { fs::File::from_raw_fd(fd) }
        };

        // Создаем структурку управления нашим сокетом для установки имени
        let mut info = ctl_info {
            // ID
            ctl_id: 0,
            // Имя в виде массива, используем функциональный блок для создания массива в котором у нас
            // хранится имя, но в сишном варианте, заканчивается нулем
            ctl_name: {
                let mut buffer = [0u8; 96];
                buffer[..UTUN_CONTROL_NAME.len()].clone_from_slice(UTUN_CONTROL_NAME.as_bytes());
                buffer
            },
        };

        // Устанавливаем имя и ID для сокета
        let res = unsafe { ioctl(handle.as_raw_fd(), CTLIOCGINFO, &mut info) };
        if res != 0 {
            // Files are automatically closed when they go out of scope.
            return Err(io::Error::last_os_error());
        }

        // Сишная структурка для установки адреса работы сокета
        let addr = sockaddr_ctl {
            sc_id: info.ctl_id,
            sc_len: mem::size_of::<sockaddr_ctl>() as u8,
            sc_family: AF_SYSTEM,
            ss_sysaddr: AF_SYS_CONTROL,
            // Передаем ID в структурку
            sc_unit: name as u32 + 1,
            sc_reserved: [0; 5],
        };

        // If connect() is successful, a tun%d device will be created, where "%d"
        // is our sc_unit-1
        // Если соединение было успешно, tun%d создан успешно, где "%d" - это sc_unit-1
        let res = unsafe {
            // Кастим указатель нашей структурки к сишной структурке
            let addr_ptr = &addr as *const sockaddr_ctl;
            // Устанавливаем параметры в сокет
            connect(handle.as_raw_fd(),
                    addr_ptr as *const sockaddr,
                    mem::size_of_val(&addr) as socklen_t)
        };
        if res != 0 {
            return Err(io::Error::last_os_error());
        }

        // TODO: Получаем имя сокета??
        let mut name_buf = [0u8; 64];
        let mut name_length: socklen_t = 64;
        let res = unsafe {
            getsockopt(handle.as_raw_fd(),
                       SYSPROTO_CONTROL,
                       UTUN_OPT_IFNAME,
                       &mut name_buf as *mut _ as *mut c_void,
                       &mut name_length as *mut socklen_t)
        };
        if res != 0 {
            return Err(io::Error::last_os_error());
        }

        // Делаем сокет неблокирующим
        let res = unsafe { fcntl(handle.as_raw_fd(), F_SETFL, O_NONBLOCK) };
        if res == -1 {
            return Err(io::Error::last_os_error());
        }

        // При любом успешном вызове exec данный сокет будет закрываться
        let res = unsafe { fcntl(handle.as_raw_fd(), F_SETFD, FD_CLOEXEC) };
        if res == -1 {
            return Err(io::Error::last_os_error());
        }

        // Возвращаем настроенный сокет
        let tun = Tun {
            handle: handle,
            // Имя берем из полученного буффера выше
            if_name: {
                std::ffi::CStr::from_bytes_with_nul(&name_buf)
                    .expect("Socket name: CStr error")
                    .to_str()
                    .expect("Socket name: CStr to CString")
                    .to_owned()
                    .to_string()
                // let len = name_buf.iter().position(|&r| r == 0).unwrap();
                // String::from_utf8(name_buf[..len].to_vec()).unwrap()
            },
        };
        Ok(tun)
    }

    pub fn name(&self) -> &str {
        &self.if_name
    }

    pub fn up(&self, self_id: u8) {
        // Получаем статус открытого сокета с помощью утилиты ifconfig
        let mut status = if cfg!(target_os = "linux") {
            process::Command::new("ifconfig")
                .arg(self.if_name.clone())
                .arg(format!("10.10.10.{}/24", self_id))
                .status()
                .unwrap()
        } else if cfg!(target_os = "macos") {
            process::Command::new("ifconfig")
                .arg(self.if_name.clone())
                .arg(format!("10.10.10.{}", self_id))
                .arg("10.10.10.1")
                .status()
                .unwrap()
        } else {
            unimplemented!()
        };

        // Проверяем, что все инициализированно
        assert!(status.success());

        // Стартуем
        status = if cfg!(target_os = "linux") {
            process::Command::new("ifconfig")
                .arg(self.if_name.clone())
                .arg("mtu")
                .arg(MTU)
                .arg("up")
                .status()
                .unwrap()
        } else if cfg!(target_os = "macos") {
            process::Command::new("ifconfig")
                .arg(self.if_name.clone())
                .arg("mtu")
                .arg(MTU)
                .arg("up")
                .status()
                .unwrap()
        } else {
            unimplemented!()
        };

        assert!(status.success());
    }
}

impl Read for Tun {
    #[cfg(target_os = "linux")]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.handle.read(buf)
    }

    #[cfg(target_os = "macos")]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // TODO: На OSX вычитывать можно максимально 1600 байт?
        let mut data = [0u8; 1600];
        let result = self.handle.read(&mut data);
        match result {
            Ok(len) => {
                // В буффер записываем все, кроме первых 4х байтов
                let real_data = data.get(4..len).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::BrokenPipe, "Read less than 4 bytes")
                })?;
                buf[..len - 4].clone_from_slice(real_data);
                Ok(if len > 4 { len - 4 } else { 0 })
            }
            Err(e) => Err(e),
        }
    }
}

impl Write for Tun {
    #[cfg(target_os = "linux")]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.handle.write(buf)
    }

    #[cfg(target_os = "macos")]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let ip_v = buf[0] & 0xf;
        let mut data: Vec<u8> = if ip_v == 6 {
            vec![0, 0, 0, 10]
        } else {
            vec![0, 0, 0, 2]
        };
        data.write_all(buf).unwrap();
        match self.handle.write(&data) {
            Ok(len) => Ok(if len > 4 { len - 4 } else { 0 }),
            Err(e) => Err(e),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.handle.flush()
    }
}

#[cfg(test)]
mod tests {
    use std::process;
    use crate::utils;
    use crate::device::*;

    #[test]
    fn create_tun_test() {
        assert!(utils::is_root());

        let tun = Tun::create(10).unwrap();
        let name = tun.name();

        let output = process::Command::new("ifconfig")
            .arg(name)
            .output()
            .expect("failed to create tun device");
        assert!(output.status.success());

        tun.up(1);
    }
}
