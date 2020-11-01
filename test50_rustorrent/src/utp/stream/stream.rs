
use std::{
    sync::{
        atomic::{
            Ordering
        }
    },
    time::{
        Instant, 
        Duration
    }
};
use async_std::{
    sync::{
        Arc, 
        RwLock,
        channel, 
        Sender, 
        Receiver
    },
    io::{
        ErrorKind, 
        Error
    },
    net::{
        SocketAddr, 
        UdpSocket, 
        SocketAddrV4, 
        SocketAddrV6, 
        Ipv4Addr, 
        Ipv6Addr
    },
    task
};
use futures::{
    future::{
        FutureExt
    },
    task::{
        Context, 
        Poll
    },
    future, 
    pin_mut
};
use hashbrown::{
    HashMap
};
use fixed::{
    types::{
        I48F16
    }
};
use shared_arena::{
    ArenaBox, 
    SharedArena
};
use crate::{
    utils::{
        Map
    },
    utils::{
        FromSlice
    }
};
use crate::{
    cache_line::{
        CacheAligned
    },
    utp::{
        delay_history::{
            DelayHistory,
        },
        sequence_number::{
            SequenceNumber
        },
        packet::{
            Packet,
            PACKET_MAX_SIZE
        },
        header::{
            HEADER_SIZE
        },
        tick::{
            Tick
        },
        constants::{
            UDP_IPV4_MTU, 
            UDP_IPV6_MTU,
            MSS,
            TARGET, 
            MIN_CWND
        },
        UtpError, 
        Result, 
        Timestamp, 
        PacketType, 
        ConnectionId, 
        SelectiveAckBit, 
        State as UtpState   
    }
};
use super::{
    state::{
        State
    },
    writer::{
        WriterUserCommand
    }
};

#[derive(Debug)]
pub struct UtpStream {
    // reader_command: Sender<ReaderCommand>,
    // reader_result: Receiver<ReaderResult>,
    pub writer_user_command: Sender<WriterUserCommand>,
}

impl UtpStream {
    pub async fn read(&self, _data: &mut [u8]) {
        // self.reader_command.send(ReaderCommand {
        //     length: data.len()
        // }).await;

        // self.reader_result.recv().await;
    }

    pub async fn write(&self, data: &[u8]) {
        let data = Vec::from_slice(data);
        self.writer_user_command.send(WriterUserCommand { data }).await;
    }
}