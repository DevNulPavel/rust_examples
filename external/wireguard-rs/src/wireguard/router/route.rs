use super::ip::*;

// TODO: no_std alternatives
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use spin::RwLock;
use treebitmap::address::Address;
use treebitmap::IpLookupTable;
use zerocopy::LayoutVerified;

/* Functions for obtaining and validating "cryptokey" routes */

pub struct RoutingTable<T: Eq + Clone> {
    ipv4: RwLock<IpLookupTable<Ipv4Addr, T>>,
    ipv6: RwLock<IpLookupTable<Ipv6Addr, T>>,
}

impl<T: Eq + Clone> RoutingTable<T> {
    /// Создаем таблицу маршрутизации
    pub fn new() -> Self {
        RoutingTable {
            ipv4: RwLock::new(IpLookupTable::new()),
            ipv6: RwLock::new(IpLookupTable::new()),
        }
    }

    /// Собираем значения из таблицы, где значения совпадают с определенным значением
    fn collect<A>(table: &IpLookupTable<A, T>, value: &T) -> Vec<(A, u32)>
    where
        A: Address,
    {
        let mut res = Vec::new();
        for (ip, cidr, v) in table.iter() {
            if v == value {
                res.push((ip, cidr))
            }
        }
        res
    }

    /// Добавляем для определенного адреса с маской значение
    pub fn insert(&self, ip: IpAddr, cidr: u32, value: T) {
        match ip {
            IpAddr::V4(v4) => self.ipv4.write().insert(v4.mask(cidr), cidr, value),
            IpAddr::V6(v6) => self.ipv6.write().insert(v6.mask(cidr), cidr, value),
        };
    }

    /// Получаем все адреса для определенного значения
    pub fn list(&self, value: &T) -> Vec<(IpAddr, u32)> {
        let mut res = vec![];
        res.extend(
            Self::collect(&*self.ipv4.read(), value)
                .into_iter()
                .map(|(ip, cidr)| (IpAddr::V4(ip), cidr)),
        );
        res.extend(
            Self::collect(&*self.ipv6.read(), value)
                .into_iter()
                .map(|(ip, cidr)| (IpAddr::V6(ip), cidr)),
        );
        res
    }

    pub fn remove(&self, value: &T) {
        let mut v4 = self.ipv4.write();
        for (ip, cidr) in Self::collect(&*v4, value) {
            v4.remove(ip, cidr);
        }

        let mut v6 = self.ipv6.write();
        for (ip, cidr) in Self::collect(&*v6, value) {
            v6.remove(ip, cidr);
        }
    }

    /// Получаем маршрут для определенного адреса
    #[inline(always)]
    pub fn get_route(&self, packet: &[u8]) -> Option<T> {
        // Определение версии протокола по первому байту данных
        match packet.get(0)? >> 4 {
            VERSION_IP4 => {
                // Проверяем длину и кастим в IPV4 заголовок
                let (header, _): (LayoutVerified<&[u8], IPv4Header>, _) =
                    LayoutVerified::new_from_prefix(packet)?;

                log::trace!(
                    "router, get route for IPv4 destination: {:?}",
                    Ipv4Addr::from(header.f_destination)
                );

                // Находим лучшее соответствие для заголовка
                self.ipv4
                    .read()
                    .longest_match(Ipv4Addr::from(header.f_destination))
                    .map(|(_, _, p)| p.clone())
            }
            VERSION_IP6 => {
                // check length and cast to IPv6 header
                let (header, _): (LayoutVerified<&[u8], IPv6Header>, _) =
                    LayoutVerified::new_from_prefix(packet)?;

                log::trace!(
                    "router, get route for IPv6 destination: {:?}",
                    Ipv6Addr::from(header.f_destination)
                );

                // check IPv6 source address
                self.ipv6
                    .read()
                    .longest_match(Ipv6Addr::from(header.f_destination))
                    .map(|(_, _, p)| p.clone())
            }
            v => {
                log::trace!("router, invalid IP version {}", v);
                None
            }
        }
    }

    #[inline(always)]
    pub fn check_route(&self, peer: &T, packet: &[u8]) -> bool {
        // Читаем версию IP из первого байта данных
        match packet.get(0).map(|v| v >> 4) {
            Some(VERSION_IP4) => {
                LayoutVerified::new_from_prefix(packet)
                    .and_then(|(header, _): (LayoutVerified<&[u8], IPv4Header>, _)| {
                        self.ipv4
                            .read()
                            .longest_match(Ipv4Addr::from(header.f_source))
                            .map(|(_, _, p)| p == peer)
                    })
                    .is_some()
            },
            Some(VERSION_IP6) => LayoutVerified::new_from_prefix(packet)
                .and_then(|(header, _): (LayoutVerified<&[u8], IPv6Header>, _)| {
                    self.ipv6
                        .read()
                        .longest_match(Ipv6Addr::from(header.f_source))
                        .map(|(_, _, p)| p == peer)
                })
                .is_some(),
            _ => false,
        }
    }
}
