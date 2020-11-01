use std::{
    sync::{
        atomic::{
            AtomicU8, 
            AtomicU16, 
            AtomicU32
        }
    }
};
use crate::{
    cache_line::{
        CacheAligned
    },
    utp::{
        connection_id::{
            ConnectionId
        },
        state::{
            State as UtpState,
        },
        sequence_number::{
            SequenceNumber
        },
        constants::{
            INIT_CWND,
            MSS
        }
    }
};

#[derive(Debug)]
pub struct AtomicState {
    pub utp_state: CacheAligned<AtomicU8>,
    pub recv_id: CacheAligned<AtomicU16>,
    pub send_id: CacheAligned<AtomicU16>,
    pub ack_number: CacheAligned<AtomicU16>,
    pub seq_number: CacheAligned<AtomicU16>,
    pub remote_window: CacheAligned<AtomicU32>,
    pub cwnd: CacheAligned<AtomicU32>,
    pub in_flight: CacheAligned<AtomicU32>,
}

impl Default for AtomicState {
    fn default() -> AtomicState {
        let (recv_id, send_id) = ConnectionId::make_ids();

        AtomicState {
            utp_state: CacheAligned::new(AtomicU8::new(UtpState::None.into())),
            recv_id: CacheAligned::new(AtomicU16::new(recv_id.into())),
            send_id: CacheAligned::new(AtomicU16::new(send_id.into())),
            ack_number: CacheAligned::new(AtomicU16::new(SequenceNumber::zero().into())),
            seq_number: CacheAligned::new(AtomicU16::new(SequenceNumber::random().into())),
            remote_window: CacheAligned::new(AtomicU32::new(INIT_CWND * MSS)),
            cwnd: CacheAligned::new(AtomicU32::new(INIT_CWND * MSS)),
            in_flight: CacheAligned::new(AtomicU32::new(0))
        }
    }
}
