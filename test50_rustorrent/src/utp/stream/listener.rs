use async_std::{
    sync::{
        Arc, 
        RwLock,
        channel, 
        Sender
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
use shared_arena::{
    SharedArena
};
use hashbrown::{
    HashMap
};
use crate::{
    utp::{
        header::{
            HEADER_SIZE
        },
        packet::{
            Packet,
            PACKET_MAX_SIZE
        },
        timestamp::{
            Timestamp
        },
        state::{
            State as UtpState
        },
        tick::{
            Tick
        },
        packet_type::{
            PacketType
        },
        Result
    },
};
use super::{
    event::{
        UtpEvent
    },
    stream::{
        UtpStream
    },
    manager::{
        UtpManager
    },
    state::{
        State
    }
};

const BUFFER_CAPACITY: usize = 1500;

pub struct UtpListener {
    v4: Arc<UdpSocket>,
    v6: Arc<UdpSocket>,
    /// The hashmap might be modified by different tasks so we wrap it in a RwLock
    streams: Arc<RwLock<HashMap<SocketAddr, Sender<UtpEvent>>>>,
    packet_arena: Arc<SharedArena<Packet>>
}

enum IncomingEvent {
    V4((usize, SocketAddr)),
    V6((usize, SocketAddr)),
}

impl UtpListener {
    pub async fn new(port: u16) -> Result<Arc<UtpListener>> {
        use async_std::prelude::*;

        let v4 = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port));
        let v6 = UdpSocket::bind(SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), port, 0, 0));

        let (v4, v6) = v4.join(v6).await;
        let (v4, v6) = (v4?, v6?);

        let listener = Arc::new(UtpListener {
            v4: Arc::new(v4),
            v6: Arc::new(v6),
            streams: Default::default(),
            packet_arena: Default::default()
        });

        listener.clone().start();

        Ok(listener)
    }

    fn get_matching_socket(&self, sockaddr: &SocketAddr) -> Arc<UdpSocket> {
        if sockaddr.is_ipv4() {
            Arc::clone(&self.v4)
        } else {
            Arc::clone(&self.v6)
        }
    }

    pub async fn connect(&self, sockaddr: SocketAddr) -> Result<UtpStream> {
        let socket = self.get_matching_socket(&sockaddr);

        let (on_connected, is_connected) = channel(1);

        let state = State::with_utp_state(UtpState::MustConnect);
        let state = Arc::new(state);

        let (sender, receiver) = channel(10);
        let mut manager = UtpManager::new_with_state(socket, sockaddr, receiver, state, self.packet_arena.clone());
        manager.set_on_connected(on_connected);

        {
            let mut streams = self.streams.write().await;
            streams.insert(sockaddr, sender.clone());
        }

        let stream = manager.get_stream();

        task::spawn(async move { manager.start().await });

        if let Ok(true) = is_connected.recv().await {
            Ok(stream)
        } else {
            Err(Error::new(ErrorKind::TimedOut, "utp connect timed out").into())
        }
    }

    async fn new_connection(&self, sockaddr: SocketAddr) -> Sender<UtpEvent> {
        //println!("NEW CONNECTION {:?}", sockaddr);
        let socket = if sockaddr.is_ipv4() {
            Arc::clone(&self.v4)
        } else {
            Arc::clone(&self.v6)
        };

        let (sender, receiver) = channel(10);
        let manager = UtpManager::new(socket, sockaddr, receiver, self.packet_arena.clone());

        {
            let mut streams = self.streams.write().await;
            streams.insert(sockaddr, sender.clone());
        }

        task::spawn(async move {
            manager.start().await
        });

        sender
    }

    pub fn start(self: Arc<Self>) {
        task::spawn(async move {
            Tick::new(self.streams.clone()).start();
            self.process_incoming().await.unwrap()
        });
    }

    async fn poll(&self, buffer_v4: &mut [u8], buffer_v6: &mut [u8]) -> Result<IncomingEvent> {
        let v4 = self.v4.recv_from(buffer_v4);
        let v6 = self.v6.recv_from(buffer_v6);
        pin_mut!(v4); // Pin on the stack
        pin_mut!(v6); // Pin on the stack

        let fun = |cx: &mut Context<'_>| {
            match FutureExt::poll_unpin(&mut v4, cx).map_ok(IncomingEvent::V4) {
                v @ Poll::Ready(_) => return v,
                _ => {}
            }

            match FutureExt::poll_unpin(&mut v6, cx).map_ok(IncomingEvent::V6) {
                v @ Poll::Ready(_) => v,
                _ => Poll::Pending
            }
        };

        future::poll_fn(fun)
            .await
            .map_err(Into::into)
    }

    async fn poll_event(&self, buffer_v4: &mut [u8], buffer_v6: &mut [u8]) -> Result<IncomingEvent> {
        loop {
            match self.poll(buffer_v4, buffer_v6).await {
                Err(ref e) if e.should_continue() => {
                    // WouldBlock or TimedOut
                    continue
                }
                x => return x
            }
        }
    }

    async fn process_incoming(&self) -> Result<()> {
        use IncomingEvent::*;

        let mut buffer_v4 = [0; BUFFER_CAPACITY];
        let mut buffer_v6 = [0; BUFFER_CAPACITY];

        loop {
            let (buffer, addr) = match self.poll_event(&mut buffer_v4[..], &mut buffer_v6[..]).await? {
                V4((size, addr)) => {
                    (&buffer_v4[..size], addr)
                },
                V6((size, addr)) => {
                    (&buffer_v6[..size], addr)
                },
            };

            if buffer.len() < HEADER_SIZE || buffer.len() > PACKET_MAX_SIZE {
                continue;
            }

            let timestamp = Timestamp::now();

            let packet = self.packet_arena.alloc_with(|packet_uninit| {
                Packet::from_incoming_in_place(packet_uninit, buffer, timestamp)
            });

            {
                if let Some(addr) = self.streams.read().await.get(&addr) {
                    let incoming = UtpEvent::IncomingPacket { packet };

                    // self.streams is still borrowed at this point
                    // can add.send() blocks and so self.streams be deadlock ?
                    // A solution is to clone the addr, but it involves its drop overhead.
                    // Or use try_send when available and clone only if error
                    addr.send(incoming).await;
                    continue;
                }
            }

            if let Ok(PacketType::Syn) = packet.get_type() {
                let incoming = UtpEvent::IncomingPacket { packet };
                self.new_connection(addr)
                    .await
                    .send(incoming)
                    .await;
            }
        }
    }
}
