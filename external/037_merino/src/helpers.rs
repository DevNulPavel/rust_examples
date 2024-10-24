use crate::{
    addr_type::AddrType,
    error::{MerinoError, ResponseCode},
    sock_req::SOCKSReq,
    sock_command::SockCommand,
};
use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs}
};

/// Конвертируем адрес и тип адреса в массив сокетов
pub async fn addr_to_socket(addr_type: &AddrType, addr: &[u8], port: u16) -> io::Result<Vec<SocketAddr>> {
    // Тип адреса
    match addr_type {
        AddrType::V6 => {
            let new_addr = (0..8)
                .map(|x| {
                    trace!("{} and {}", x * 2, (x * 2) + 1);
                    (u16::from(addr[(x * 2)]) << 8) | u16::from(addr[(x * 2) + 1])
                })
                .collect::<Vec<u16>>();

            Ok(vec![SocketAddr::from(SocketAddrV6::new(
                Ipv6Addr::new(
                    new_addr[0],
                    new_addr[1],
                    new_addr[2],
                    new_addr[3],
                    new_addr[4],
                    new_addr[5],
                    new_addr[6],
                    new_addr[7],
                ),
                port,
                0,
                0,
            ))])
        }
        AddrType::V4 => Ok(vec![SocketAddr::from(SocketAddrV4::new(
            Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]),
            port,
        ))]),
        AddrType::Domain => {
            let mut domain = String::from_utf8_lossy(addr).to_string();
            domain.push(':');
            domain.push_str(&port.to_string());

            // TODO: Переделать резолв на асинхронный
            Ok(tokio::net::lookup_host(domain).await?.collect())
        }
    }
}

/// Convert an AddrType and address to String
pub fn pretty_print_addr(addr_type: &AddrType, addr: &[u8]) -> String {
    match addr_type {
        AddrType::Domain => String::from_utf8_lossy(addr).to_string(),
        AddrType::V4 => addr
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>()
            .join("."),
        AddrType::V6 => {
            let addr_16 = (0..8)
                .map(|x| (u16::from(addr[(x * 2)]) << 8) | u16::from(addr[(x * 2) + 1]))
                .collect::<Vec<u16>>();

            addr_16
                .iter()
                .map(|x| format!("{:x}", x))
                .collect::<Vec<String>>()
                .join(":")
        }
    }
}
