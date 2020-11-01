
pub const BASE_HISTORY: usize = 10;
pub const INIT_CWND: u32 = 2;
pub const MIN_CWND: u32 = 2;
/// Sender's Maximum Segment Size
/// Set to Ethernet MTU
pub const MSS: u32 = 1400;
pub const TARGET: i64 = 100_000; //100;
pub const GAIN: u32 = 1;
pub const ALLOWED_INCREASE: u32 = 1;


pub const ETHERNET_MTU: usize = 1500;
pub const IPV4_HEADER_SIZE: usize = 20;
pub const IPV6_HEADER_SIZE: usize = 40;
pub const UDP_HEADER_SIZE: usize = 8;
pub const GRE_HEADER_SIZE: usize = 24;
pub const PPPOE_HEADER_SIZE: usize = 8;
pub const MPPE_HEADER_SIZE: usize = 2;
// packets have been observed in the wild that were fragmented
// with a payload of 1416 for the first fragment
// There are reports of routers that have MTU sizes as small as 1392
pub const FUDGE_HEADER_SIZE: usize = 36;
pub const TEREDO_MTU: usize = 1280;

pub const UDP_IPV4_OVERHEAD: usize = IPV4_HEADER_SIZE + UDP_HEADER_SIZE;
pub const UDP_IPV6_OVERHEAD: usize = IPV6_HEADER_SIZE + UDP_HEADER_SIZE;
pub const UDP_TEREDO_OVERHEAD: usize = UDP_IPV4_OVERHEAD + UDP_IPV6_OVERHEAD;

pub const UDP_IPV4_MTU: usize =
    ETHERNET_MTU - IPV4_HEADER_SIZE - UDP_HEADER_SIZE - GRE_HEADER_SIZE
     - PPPOE_HEADER_SIZE - MPPE_HEADER_SIZE - FUDGE_HEADER_SIZE;

pub const UDP_IPV6_MTU: usize =
    ETHERNET_MTU - IPV6_HEADER_SIZE - UDP_HEADER_SIZE - GRE_HEADER_SIZE
     - PPPOE_HEADER_SIZE - MPPE_HEADER_SIZE - FUDGE_HEADER_SIZE;

pub const UDP_TEREDO_MTU: usize = TEREDO_MTU - IPV6_HEADER_SIZE - UDP_HEADER_SIZE;