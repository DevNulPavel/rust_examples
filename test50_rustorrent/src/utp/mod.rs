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
mod header;
mod connection_id;
mod payload;
mod packet;
mod constants;

use std::{
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
    error::{
        UtpError
    },
    packet_type::{
        PacketType
    },
    header::{
        HEADER_SIZE
    },
    connection_id::{
        ConnectionId
    }
};

//////////////////////////////////////////////////////////////////////////////////////////

type Result<T> = std::result::Result<T, UtpError>;

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
    pub fn new(packet: &'a packet::Packet) -> ExtensionIterator<'a> {
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

