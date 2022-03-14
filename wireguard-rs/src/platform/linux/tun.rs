use super::super::tun::*;

use std::error::Error;
use std::fmt;
use std::mem;
use std::os::raw::c_short;
use std::os::unix::io::RawFd;

const TUNSETIFF: u64 = 0x4004_54ca;
const CLONE_DEVICE_PATH: &[u8] = b"/dev/net/tun\0";

/// Сишная структурка для передачи в системный вызов
#[repr(C)]
struct Ifreq {
    name: [u8; libc::IFNAMSIZ],
    flags: c_short,
    _pad: [u8; 64],
}

// man 7 rtnetlink
// Layout from: https://elixir.bootlin.com/linux/latest/source/include/uapi/linux/rtnetlink.h#L516
#[repr(C)]
struct IfInfomsg {
    ifi_family: libc::c_uchar,
    __ifi_pad: libc::c_uchar,
    ifi_type: libc::c_ushort,
    ifi_index: libc::c_int,
    ifi_flags: libc::c_uint,
    ifi_change: libc::c_uint,
}

pub struct LinuxTun {}

/////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LinuxTunError {
    InvalidTunDeviceName,
    FailedToOpenCloneDevice,
    SetIFFIoctlFailed,
    GetMTUIoctlFailed,
    NetlinkFailure,
    Closed, // TODO
}

impl fmt::Display for LinuxTunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinuxTunError::InvalidTunDeviceName => write!(f, "Invalid name (too long)"),
            LinuxTunError::FailedToOpenCloneDevice => {
                write!(f, "Failed to obtain fd for clone device")
            }
            LinuxTunError::SetIFFIoctlFailed => {
                write!(f, "set_iff ioctl failed (insufficient permissions?)")
            }
            LinuxTunError::Closed => write!(f, "The tunnel has been closed"),
            LinuxTunError::GetMTUIoctlFailed => write!(f, "ifmtu ioctl failed"),
            LinuxTunError::NetlinkFailure => write!(f, "Netlink listener error"),
        }
    }
}

impl Error for LinuxTunError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        unimplemented!()
    }

    fn description(&self) -> &str {
        unimplemented!()
    }
}

/////////////////////////////////////////////////////////////////////////////////////

pub struct LinuxTunReader {
    fd: RawFd,
}

impl Reader for LinuxTunReader {
    type Error = LinuxTunError;

    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, Self::Error> {
        /*
        debug_assert!(
            offset < buf.len(),
            "There is no space for the body of the read"
        );
        */
        let n: isize =
            unsafe { libc::read(self.fd, buf[offset..].as_mut_ptr() as _, buf.len() - offset) };
        if n < 0 {
            Err(LinuxTunError::Closed)
        } else {
            // conversion is safe
            Ok(n as usize)
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////

pub struct LinuxTunWriter {
    fd: RawFd,
}

impl Writer for LinuxTunWriter {
    type Error = LinuxTunError;

    fn write(&self, src: &[u8]) -> Result<(), Self::Error> {
        match unsafe { libc::write(self.fd, src.as_ptr() as _, src.len() as _) } {
            -1 => Err(LinuxTunError::Closed),
            _ => Ok(()),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////

/// Выдает значение индекса сетевого интерфейса по его имени
fn get_ifindex(name: &[u8; libc::IFNAMSIZ]) -> i32 {
    debug_assert_eq!(
        name[libc::IFNAMSIZ - 1],
        0,
        "name buffer not null-terminated"
    );

    let name = *name;
    let idx = unsafe {
        let ptr: *const libc::c_char = mem::transmute(&name);
        libc::if_nametoindex(ptr)
    };
    idx as i32
}

fn get_mtu(name: &[u8; libc::IFNAMSIZ]) -> Result<usize, LinuxTunError> {
    #[repr(C)]
    struct arg {
        name: [u8; libc::IFNAMSIZ],
        mtu: u32,
    }

    debug_assert_eq!(
        name[libc::IFNAMSIZ - 1],
        0,
        "name buffer not null-terminated"
    );

    // create socket
    let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if fd < 0 {
        return Err(LinuxTunError::GetMTUIoctlFailed);
    }

    // do SIOCGIFMTU ioctl
    let buf = arg {
        name: *name,
        mtu: 0,
    };
    let err = unsafe {
        let ptr: &libc::c_void = &*(&buf as *const _ as *const libc::c_void);
        libc::ioctl(fd, libc::SIOCGIFMTU, ptr)
    };

    // close socket
    unsafe { libc::close(fd) };

    // handle error from ioctl
    if err != 0 {
        return Err(LinuxTunError::GetMTUIoctlFailed);
    }

    // upcast to usize
    Ok(buf.mtu as usize)
}

pub struct LinuxTunStatus {
    events: Vec<TunEvent>,
    index: i32,
    name: [u8; libc::IFNAMSIZ],
    fd: RawFd,
}

/////////////////////////////////////////////////////////////////////////////////////

// Реализация получения статуса
impl LinuxTunStatus {
    // Удобно, что константы можно объявлять в пределах зоны видимости реализации
    // RTMGRP_LINK — эта группа получает уведомления об изменениях в сетевых интерфейсах (интерфейс удалился, добавился, опустился, поднялся)
    // RTMGRP_IPV4_IFADDR — эта группа получает уведомления об изменениях в IPv4 адресах интерфейсов (адрес был добавлен или удален)
    // RTMGRP_IPV6_IFADDR — эта группа получает уведомления об изменениях в IPv6 адресах интерфейсов (адрес был добавлен или удален)
    // RTMGRP_IPV4_ROUTE — эта группа получает уведомления об изменениях в таблице маршрутизации для IPv4 адресов
    // RTMGRP_IPV6_ROUTE — эта группа получает уведомления об изменениях в таблице маршрутизации для IPv6 адресов
    const RTNLGRP_LINK: libc::c_uint = 1;
    const RTNLGRP_IPV4_IFADDR: libc::c_uint = 5;
    const RTNLGRP_IPV6_IFADDR: libc::c_uint = 9;

    fn new(name: [u8; libc::IFNAMSIZ]) -> Result<LinuxTunStatus, LinuxTunError> {
        // Создаем сокет типа NETLINK для контроля маршрутов в системе Linux + оповещения об изменениях
        let fd = unsafe { libc::socket(libc::AF_NETLINK, libc::SOCK_RAW, libc::NETLINK_ROUTE) };
        if fd < 0 {
            return Err(LinuxTunError::Closed);
        }

        // Подготовка описания адреса
        // struct sockaddr_nl
        // {
        //     sa_family_t nl_family; 	// семейство протоколов - всегда AF_NETLINK
        //     unsigned short nl_pad; 	// поле всегда заполнено нулями
        //     pid_t nl_pid; 			// идентификатор процесса
        //     __u32 nl_groups; 		// маска групп получателей/отправителей
        // };
        let groups = (1 << (Self::RTNLGRP_LINK - 1))    // Уведомления об изменениях сетевых интерфейсов
            | (1 << (Self::RTNLGRP_IPV4_IFADDR - 1))    // Уведомления об изменении IPv4 адресов
            | (1 << (Self::RTNLGRP_IPV6_IFADDR - 1));   // Уведомления об изменении IPv4 адресов
        // Заполняем структуру сокета для получения оповещений об изменениях
        let mut sockaddr: libc::sockaddr_nl = unsafe { mem::zeroed() };
        sockaddr.nl_family = libc::AF_NETLINK as u16;
        sockaddr.nl_groups = groups;
        sockaddr.nl_pid = 0; // Это системный сокет, у нас всегда здесь ноль

        // Пробуем забиндить сокет
        let res = unsafe {
            libc::bind(
                fd,
                mem::transmute(&mut sockaddr),
                mem::size_of::<libc::sockaddr_nl>() as u32,
            )
        };

        if res != 0 {
            Err(LinuxTunError::Closed)
        } else {
            // Создаем результат
            Ok(LinuxTunStatus {
                // Первое событие в очереди - это событие запуска
                events: vec![
                    #[cfg(feature = "start_up")]
                    TunEvent::Up(1500),
                ],
                // Получаем индекс данного сетевого интерфейса с событиями
                index: get_ifindex(&name),
                // Дескриптор сокета
                fd,
                // Его имя
                name,
            })
        }
    }
}

impl Status for LinuxTunStatus {
    type Error = LinuxTunError;

    fn event(&mut self) -> Result<TunEvent, Self::Error> {
        const DONE: u16 = libc::NLMSG_DONE as u16;
        const ERROR: u16 = libc::NLMSG_ERROR as u16;
        const INFO_SIZE: usize = mem::size_of::<IfInfomsg>();
        const HDR_SIZE: usize = mem::size_of::<libc::nlmsghdr>();

        // Буффер для событий
        let mut buf = [0u8; 1 << 12];
        log::debug!("netlink, fetch event (fd = {})", self.fd);
        loop {
            // Может быть у нас есть уже какие-то забуфферизированные события?
            // Тогда возвращаем их
            if let Some(event) = self.events.pop() {
                return Ok(event);
            }

            // Читаем сообщение из сокета в буфер
            let size: libc::ssize_t =
                unsafe { libc::recv(self.fd, mem::transmute(&mut buf), buf.len(), 0) };
            if size < 0 {
                break Err(LinuxTunError::NetlinkFailure);
            }

            // Обрезаем буфер до нужного размера
            let size: usize = size as usize;
            let mut remain = &buf[..size];
            log::debug!("netlink, received message ({} bytes)", size);

            // Обрабатываем прилетевшие сообщениия из буфера
            while remain.len() >= HDR_SIZE {
                // Извлекаем заголовок сообщения в C стиле
                // struct nlmsghdr{
                //     __u32 nlmsg_len;	    // размер сообщения, с учетом заголовка
                //     __u16 nlmsg_type; 	// тип содержимого сообщения (об этом ниже)
                //     __u16 nlmsg_flags;	// различные флаги сообщения
                //     __u32 nlmsg_seq; 	// порядковый номер сообщения
                //     __u32 nlmsg_pid; 	// идентификатор процесса (PID), отославшего сообщение
                // };
                assert!(remain.len() > HDR_SIZE); // Еще раз проверим, что размер валидный
                let hdr: libc::nlmsghdr = unsafe {
                    // Перегоняем массив байтов в структуру libc::nlmsghdr,
                    // Может быть можно было бы обойтись без копирования, а просто скастить одно значение к другому?
                    let mut hdr = [0u8; HDR_SIZE];
                    hdr.copy_from_slice(&remain[..HDR_SIZE]);
                    mem::transmute(hdr)
                };

                // Все остальное считаем как body
                let body: &[u8] = &remain[HDR_SIZE..];

                // Проверяем, что размер всего сообщения из заголовка и размер оставшихся данных
                // меньше оставшихся данных
                let msg_len: usize = hdr.nlmsg_len as usize;
                assert!(msg_len <= remain.len(), "malformed netlink message");

                // Обрабатываем тело запроса в зависимости от типа сообщения
                match hdr.nlmsg_type {
                    DONE => break,
                    ERROR => break,
                    libc::RTM_NEWLINK => {
                        // Проверяем размер валидный ли?
                        if body.len() < INFO_SIZE {
                            return Err(LinuxTunError::NetlinkFailure);
                        }

                        // Кастим данные в структуру сообщения
                        // struct ifinfomsg{
                        //     unsigned char  ifi_family;  // семейство (AF_UNSPEC)
                        //     unsigned short ifi_type;    // тип устройства
                        //     int            ifi_index;   // индекс интерфейса
                        //     unsigned int   ifi_flags;   // флаги устройства
                        //     unsigned int   ifi_change;  // маска смены, зарезервировано для использования в будущем и всегда должно быть равно 0xFFFFFFFF
                        // };
                        let info: IfInfomsg = unsafe {
                            let mut info = [0u8; INFO_SIZE];
                            info.copy_from_slice(&body[..INFO_SIZE]);
                            mem::transmute(info)
                        };

                        // Выводим информацию о сообщении
                        log::trace!(
                            "netlink, IfInfomsg{{ family = {}, type = {}, index = {}, flags = {}, change = {}}}",
                            info.ifi_family,
                            info.ifi_type,
                            info.ifi_index,
                            info.ifi_flags,
                            info.ifi_change,
                        );
                        debug_assert_eq!(info.__ifi_pad, 0);

                        // Если в сообщении индекс устройства совпадает с индексом
                        if info.ifi_index == self.index {
                            // Обработка наличия и отсутствия флага включения или отключения интерфейса
                            if info.ifi_flags & (libc::IFF_UP as u32) != 0 {
                                let mtu = get_mtu(&self.name)?;
                                log::trace!("netlink, up event, mtu = {}", mtu);
                                self.events.push(TunEvent::Up(mtu));
                            } else {
                                log::trace!("netlink, down event");
                                self.events.push(TunEvent::Down);
                            }
                        }
                    }
                    _ => (),
                };

                // Идем дальше к следующему сообщению
                remain = &remain[msg_len..];
            }
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////

// Реализация туннеля для Linux
impl Tun for LinuxTun {
    type Writer = LinuxTunWriter;
    type Reader = LinuxTunReader;
    type Error = LinuxTunError;
}

impl PlatformTun for LinuxTun {
    type Status = LinuxTunStatus;

    // Создаем туннель
    #[allow(clippy::type_complexity)]
    fn create(name: &str) -> Result<(Vec<Self::Reader>, Self::Writer, Self::Status), Self::Error> {
        // Создаем структуру запроса
        let mut req = Ifreq {
            name: [0u8; libc::IFNAMSIZ],
            flags: (libc::IFF_TUN | libc::IFF_NO_PI) as c_short,
            _pad: [0u8; 64],
        };

        // Делаем проверку длины имени девайса перед записью
        let bs = name.as_bytes();
        if bs.len() > libc::IFNAMSIZ - 1 {
            return Err(LinuxTunError::InvalidTunDeviceName);
        }
        // Пишем размер
        req.name[..bs.len()].copy_from_slice(bs);

        // Рткрываем файлик туннеля
        let fd: RawFd = match unsafe { libc::open(CLONE_DEVICE_PATH.as_ptr() as _, libc::O_RDWR) } {
            -1 => return Err(LinuxTunError::FailedToOpenCloneDevice),
            fd => fd,
        };
        assert!(fd >= 0);

        // Создаем девайс туннеля
        if unsafe { libc::ioctl(fd, TUNSETIFF as _, &req) } < 0 {
            return Err(LinuxTunError::SetIFFIoctlFailed);
        }

        // create PlatformTunMTU instance
        Ok((
            vec![LinuxTunReader { fd }], // TODO: use multi-queue for Linux
            LinuxTunWriter { fd },
            LinuxTunStatus::new(req.name)?,
        ))
    }
}
