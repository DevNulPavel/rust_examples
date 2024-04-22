use std::{
    ops::{
        Deref,
        DerefMut
    },
    mem::{
        MaybeUninit
    }
};
use super::{
    header::{
        HEADER_SIZE,
        Header
    },
    payload::{
        PAYLOAD_SIZE,
        Payload
    },
    packet_type::{
        PacketType
    },
    sequence_number::{
        SequenceNumber
    },
    timestamp::{
        Timestamp
    },
    ExtensionIterator
};

pub const PACKET_MAX_SIZE: usize = HEADER_SIZE + PAYLOAD_SIZE;

pub struct PacketPool {
    pool: Vec<Packet>
}

#[repr(C, packed)]
pub struct Packet {
    /// Заголовок нашего пакета
    header: Header,

    /// Данные нашего пакета
    pub(super) payload: Payload,

    /// Используется для дальнейшего чтения seq_nr без
    /// необоходимости конвертации из big endian из заголовка
    seq_number: SequenceNumber,

    /// True если данный пакет был переотправлен
    pub(super) resent: bool,
    pub(super) last_sent: Timestamp,
    pub(super) lost: bool,
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
