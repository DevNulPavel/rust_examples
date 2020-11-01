pub mod stream;
pub mod tick;
mod sequence_number;
mod state;
mod timestamp;
mod delay;
mod relative_delay;
mod delay_history;
mod error;
mod packet_type;

use std::{
    convert::{
        TryInto
    },
    ops::{
        Deref,
        DerefMut, 
        Add
    },
    mem::{
        MaybeUninit
    }
};
use stream::{
    UtpEvent
};
use self::{
    sequence_number::{
        SequenceNumber
    },
    state::{
        State
    },
    timestamp::{
        Timestamp
    },
    delay::{
        Delay
    },
    error::{
        UtpError
    },
    packet_type::{
        PacketType
    }
};

//////////////////////////////////////////////////////////////////////////////////////////

pub const BASE_HISTORY: usize = 10;
pub const INIT_CWND: u32 = 2;
pub const MIN_CWND: u32 = 2;
/// Sender's Maximum Segment Size
/// Set to Ethernet MTU
pub const MSS: u32 = 1400;
pub const TARGET: i64 = 100_000; //100;
pub const GAIN: u32 = 1;
pub const ALLOWED_INCREASE: u32 = 1;

//////////////////////////////////////////////////////////////////////////////////////////

type Result<T> = std::result::Result<T, UtpError>;


pub const HEADER_SIZE: usize = std::mem::size_of::<Header>();

/// 0       4       8               16              24              32
/// +-------+-------+---------------+---------------+---------------+
/// | type  | ver   | extension     | connection_id                 |
/// +-------+-------+---------------+---------------+---------------+
/// | timestamp_microseconds                                        |
/// +---------------+---------------+---------------+---------------+
/// | timestamp_difference_microseconds                             |
/// +---------------+---------------+---------------+---------------+
/// | wnd_size                                                      |
/// +---------------+---------------+---------------+---------------+
/// | seq_nr                        | ack_nr                        |
/// +---------------+---------------+---------------+---------------+

#[repr(C, packed)]
pub struct Header {
    /// Protocol version and packet type.
    /// type = type_version >> 4
    /// version = type_version & 0x0F
    /// The current version is 1.
    /// The type field describes the type of packet.
    type_version: u8,
    /// The type of the first extension in a linked list of extension headers.
    /// 0 means no extension.
    extension: u8,
    /// This is a random, unique, number identifying all the packets
    /// that belong to the same connection. Each socket has one
    /// connection ID for sending packets and a different connection
    /// ID for receiving packets. The endpoint initiating the connection
    /// decides which ID to use, and the return path has the same ID + 1.
    connection_id: u16,
    /// This is the 'microseconds' parts of the timestamp of when this packet
    /// was sent. This is set using gettimeofday() on posix and
    /// QueryPerformanceTimer() on windows. The higher resolution this timestamp
    /// has, the better. The closer to the actual transmit time it is set, the better.
    timestamp_micro: u32,
    /// This is the difference between the local time and the timestamp in the last
    /// received packet, at the time the last packet was received. This is the
    /// latest one-way delay measurement of the link from the remote peer to the local
    /// machine. When a socket is newly opened and doesn't have any delay
    /// samples yet, this must be set to 0.
    timestamp_difference_micro: u32,
    /// Advertised receive window. This is 32 bits wide and specified in bytes.
    /// The window size is the number of bytes currently in-flight, i.e. sent but
    /// not acked. The advertised receive window lets the other end cap the
    /// window size if it cannot receive any faster, if its receive buffer is
    /// filling up.
    /// When sending packets, this should be set to the number of bytes left in
    /// the socket's receive buffer.
    window_size: u32,
    /// This is the sequence number of this packet. As opposed to TCP, uTP
    /// sequence numbers are not referring to bytes, but packets. The sequence
    /// number tells the other end in which order packets should be served back
    /// to the application layer.
    seq_nr: u16,
    /// This is the sequence number the sender of the packet last received in the
    /// other direction.
    ack_nr: u16,
}

impl Header {
    fn get_type(&self) -> Result<PacketType> {
        self.check_version()?;
        self.type_version.try_into()
    }

    fn check_version(&self) -> Result<()> {
        match self.type_version & 0x0F {
            1 => Ok(()),
            _ => Err(UtpError::WrongVersion)
        }
    }

    // Getters
    fn get_connection_id(&self) -> ConnectionId {
        u16::from_be(self.connection_id).into()
    }
    fn get_version(&self) -> u8 {
        self.type_version & 0xF
    }
    fn get_timestamp(&self) -> Timestamp {
        u32::from_be(self.timestamp_micro).into()
    }
    fn get_timestamp_diff(&self) -> Delay {
        u32::from_be(self.timestamp_difference_micro).into()
    }
    fn get_window_size(&self) -> u32 {
        u32::from_be(self.window_size)
    }
    fn get_seq_number(&self) -> SequenceNumber {
        SequenceNumber::from_be(self.seq_nr)
    }
    fn get_ack_number(&self) -> SequenceNumber {
        SequenceNumber::from_be(self.ack_nr)
    }
    fn get_extension_type(&self) -> ExtensionType {
        ExtensionType::from(self.extension)
    }
    fn has_extension(&self) -> bool {
        self.extension != 0
    }

    // Setters
    fn set_connection_id(&mut self, id: ConnectionId) {
        self.connection_id = u16::to_be(id.into());
    }
    fn set_timestamp(&mut self, timestamp: Timestamp) {
        self.timestamp_micro = u32::to_be(timestamp.into());
    }
    fn set_timestamp_diff(&mut self, delay: Delay) {
        self.timestamp_difference_micro = u32::to_be(delay.into());
    }
    fn set_window_size(&mut self, window_size: u32) {
        self.window_size = u32::to_be(window_size);
    }
    fn set_seq_number(&mut self, seq_number: SequenceNumber) {
        self.seq_nr = seq_number.to_be();
    }
    fn set_ack_number(&mut self, ack_number: SequenceNumber) {
        self.ack_nr = ack_number.to_be();
    }

    // fn update_timestamp(&mut self) {
    //     self.set_timestamp(Timestamp::now());
    // }

    pub fn new(packet_type: PacketType) -> Header {
        let packet_type: u8 = packet_type.into();
        Header {
            type_version: packet_type << 4 | 1,
            ..Default::default()
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { &*(self as *const Header as *const [u8; std::mem::size_of::<Header>()]) }
    }
}

impl Default for Header {
    fn default() -> Header {
        let packet_type: u8 = PacketType::Data.into();
        Header {
            type_version: packet_type << 4 | 1,
            extension: 0,
            connection_id: 0,
            timestamp_micro: 0,
            timestamp_difference_micro: 0,
            window_size: 0,
            seq_nr: 0,
            ack_nr: 0,
        }
    }
}

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Header")
         .field("type", &self.get_type())
         .field("version", &self.get_version())
         // .field("extension", &self.get_extension_type())
         .field("connection_id", &self.get_connection_id())
         .field("timestamp", &self.get_timestamp())
         .field("timestamp_difference", &self.get_timestamp_diff())
         .field("window_size", &self.get_window_size())
         .field("seq_number", &self.get_seq_number())
         .field("ack_number", &self.get_ack_number())
         .finish()
    }
}

#[derive(Copy, Clone, Debug)]
struct ConnectionId(u16);

impl From<u16> for ConnectionId {
    fn from(byte: u16) -> ConnectionId {
        ConnectionId(byte)
    }
}

impl Into<u16> for ConnectionId {
    fn into(self) -> u16 {
        self.0
    }
}

impl Add<u16> for ConnectionId {
    type Output = Self;

    fn add(self, o: u16) -> ConnectionId {
        ConnectionId(self.0 + o)
    }
}

use rand::Rng;

impl ConnectionId {
    pub fn make_ids() -> (ConnectionId, ConnectionId) {
        let id = rand::thread_rng().gen::<u16>();
        if id == 0 {
            (id.into(), (id + 1).into())
        } else {
            ((id - 1).into(), id.into())
        }
    }
}

const PAYLOAD_SIZE: usize = 1500;

#[repr(C, packed)]
struct Payload {
    data: [u8; PAYLOAD_SIZE],
    len: usize
}

impl Payload {
    fn new_in_place(place: &mut Payload, data: &[u8]) {
        let data_len = data.len();
        place.data[..data_len].copy_from_slice(data);
        place.len = data_len;
    }

    fn new(data: &[u8]) -> Payload {
        let data_len = data.len();
        let mut payload = [0; PAYLOAD_SIZE];
        payload[..data_len].copy_from_slice(data);
        Payload {
            data: payload,
            len: data_len
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

pub struct PacketPool {
    pool: Vec<Packet>
}

const PACKET_MAX_SIZE: usize = HEADER_SIZE + PAYLOAD_SIZE;

#[repr(C, packed)]
pub struct Packet {
    header: Header,
    payload: Payload,
    /// Used to read the seq_nr later, without the need to convert from
    /// big endian from the header
    seq_number: SequenceNumber,
    /// True if this packet was resent
    resent: bool,
    last_sent: Timestamp,
    lost: bool,
    received_at: Option<Timestamp>,
}

impl std::fmt::Debug for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let seq_number = self.seq_number;
        f.debug_struct("Packet")
         .field("seq_nr", &seq_number)
         .field("header", &self.header)
         .finish()
    }
}

impl Deref for Packet {
    type Target = Header;

    fn deref(&self) -> &Header {
        &self.header
    }
}

impl DerefMut for Packet {
    //type Target = Header;

    fn deref_mut(&mut self) -> &mut Header {
        &mut self.header
    }
}

impl Packet {
    pub fn new_in_place<'a>(place: &'a mut MaybeUninit<Packet>, data: &[u8]) -> &'a Packet {
        let place = unsafe { &mut *place.as_mut_ptr() };

        place.header = Header::default();
        Payload::new_in_place(&mut place.payload,  data);
        // Fill rest of Packet with non-uninitialized data
        // Ensure that we don't invoke any Drop here
        place.seq_number = SequenceNumber::zero();
        place.resent = false;
        place.last_sent = Timestamp::zero();
        place.lost = false;
        place.received_at = None;

        place
    }

//    pub fn from_incoming_in_place(place: &mut Packet, data: &[u8], timestamp: Timestamp) {
    pub fn from_incoming_in_place<'a>(place: &'a mut MaybeUninit<Packet>, data: &[u8], timestamp: Timestamp) -> &'a Packet {
        //let slice = unsafe { &mut *(place as *mut Packet as *mut [u8; PACKET_MAX_SIZE]) };
        let slice = unsafe { &mut *(place.as_mut_ptr() as *mut [u8; PACKET_MAX_SIZE]) };
        let data_len = data.len();

        assert!(data_len >= HEADER_SIZE && data_len < PACKET_MAX_SIZE);

        slice[..data_len].copy_from_slice(data);

        let place = unsafe { &mut *place.as_mut_ptr() };

        // Fill rest of Packet with non-uninitialized data
        // Ensure that we don't invoke any Drop here
        place.payload.len = data_len - HEADER_SIZE;
        place.seq_number = place.get_seq_number();
        place.resent = false;
        place.last_sent = Timestamp::zero();
        place.lost = false;
        place.received_at = Some(timestamp);

        place
    }

    pub fn new(data: &[u8]) -> Packet {
        Packet {
            header: Header::default(),
            payload: Payload::new(data),
            seq_number: SequenceNumber::zero(),
            resent: false,
            last_sent: Timestamp::zero(),
            lost: false,
            received_at: None,
        }
    }

    pub fn syn() -> Packet {
        Packet {
            header: Header::new(PacketType::Syn),
            payload: Payload::new(&[]),
            seq_number: SequenceNumber::zero(),
            resent: false,
            last_sent: Timestamp::zero(),
            lost: false,
            received_at: None,
        }
    }

    pub fn new_type(ty: PacketType) -> Packet {
        Packet {
            header: Header::new(ty),
            payload: Payload::new(&[]),
            seq_number: SequenceNumber::zero(),
            resent: false,
            last_sent: Timestamp::zero(),
            lost: false,
            received_at: None,
        }
    }

    pub fn received_at(&self) -> Timestamp {
        self.received_at.expect("Packet wasn't received")
    }

    pub fn payload_len(&self) -> usize {
        self.payload.len()
    }

    pub fn iter_sacks(&self) -> ExtensionIterator {
        ExtensionIterator::new(self)
    }

    pub fn update_timestamp(&mut self) {
        let timestamp = Timestamp::now();
        self.set_timestamp(timestamp);
        self.last_sent = timestamp;
    }

    pub fn millis_since_sent(&self, now: Timestamp) -> u32 {
        self.last_sent.elapsed_millis(now)
    }

    pub fn set_packet_seq_number(&mut self, n: SequenceNumber) {
        self.header.set_seq_number(n);
        self.seq_number = n;
    }

    pub fn get_packet_seq_number(&self) -> SequenceNumber {
        self.seq_number
    }

    pub fn is_seq_less_equal(&self, n: SequenceNumber) -> bool {
        self.seq_number.cmp_less_equal(n)
    }

    pub fn size(&self) -> usize {
        self.payload.len() + HEADER_SIZE
    }

    pub fn as_bytes(&self) -> &[u8] {
        let slice = unsafe { &*(self as *const Packet as *const [u8; std::mem::size_of::<Packet>()]) };
        &slice[..std::mem::size_of::<Header>() + self.payload.len]
    }

    // pub fn iter_extensions(&self) -> ExtensionIterator {
    //     ExtensionIterator::new(self)
    // }
}

pub enum ExtensionType {
    SelectiveAck,
    None,
    Unknown
}

impl From<u8> for ExtensionType {
    fn from(byte: u8) -> ExtensionType {
        match byte {
            0 => ExtensionType::None,
            1 => ExtensionType::SelectiveAck,
            _ => ExtensionType::Unknown
        }
    }
}

pub struct SelectiveAck<'a> {
    bitfield: &'a [u8],
    byte_index: usize,
    bit_index: u8,
    ack_number: SequenceNumber,
    first: bool,
}

impl SelectiveAck<'_> {
    pub fn has_missing_ack(&self) -> bool {
        self.bitfield.iter().any(|b| b.count_zeros() > 0)
    }

    pub fn nackeds(&self) -> u32 {
        self.bitfield.iter().map(|b| b.count_ones()).sum()
    }
}

pub enum SelectiveAckBit {
    Acked(SequenceNumber),
    Missing(SequenceNumber)
}

impl Iterator for SelectiveAck<'_> {
    type Item = SelectiveAckBit;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            // for byte in self.bitfield {
            //     println!("BITFIELD {:08b}", byte);
            // }
            self.first = false;
            return Some(SelectiveAckBit::Missing(self.ack_number + 1));
        }

        let byte = *self.bitfield.get(self.byte_index)?;
        let bit = byte & (1 << self.bit_index);

        let ack_number = self.ack_number
            + self.byte_index as u16 * 8
            + self.bit_index as u16
            + 2;

        if self.bit_index == 7 {
            self.byte_index += 1;
            self.bit_index = 0;
        } else {
            self.bit_index += 1;
        }

        if bit == 0 {
            Some(SelectiveAckBit::Missing(ack_number))
        } else {
            Some(SelectiveAckBit::Acked(ack_number))
        }
    }
}

pub struct ExtensionIterator<'a> {
    current_type: ExtensionType,
    slice: &'a [u8],
    ack_number: SequenceNumber,
}

impl<'a> ExtensionIterator<'a> {
    pub fn new(packet: &'a Packet) -> ExtensionIterator<'a> {
        let current_type = packet.get_extension_type();
        let slice = &packet.payload.data[..packet.size() - HEADER_SIZE];
        let ack_number = packet.get_ack_number();

        // for byte in &packet.packet_ref.payload.data[..packet.len - HEADER_SIZE] {
        //     //println!("BYTE {:x}", byte);
        // }
        ExtensionIterator { current_type, slice, ack_number }
    }
}

impl<'a> Iterator for ExtensionIterator<'a> {
    type Item = SelectiveAck<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current_type {
                ExtensionType::None => {
                    return None;
                }
                ExtensionType::SelectiveAck => {
                    let len = self.slice.get(1).copied()? as usize;
                    let bitfield = &self.slice.get(2..2 + len)?;

                    self.current_type = self.slice.get(0).copied()?.into();
                    self.slice = &self.slice.get(2 + len..)?;

                    return Some(SelectiveAck {
                        bitfield,
                        byte_index: 0,
                        bit_index: 0,
                        ack_number: self.ack_number,
                        first: true
                    });
                }
                _ => {
                    self.current_type = self.slice.get(0).copied()?.into();
                    let len = self.slice.get(1).copied()? as usize;
                    self.slice = &self.slice.get(len..)?;
                }
            }
        }
    }
}

// Following constants found in libutp

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
