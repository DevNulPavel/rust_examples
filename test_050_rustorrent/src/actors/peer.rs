use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use futures::future::{Fuse, FutureExt};
use async_std::net::TcpStream;
use async_std::io::BufReader;
use async_std::prelude::*;
use async_std::sync::{channel, Sender};
use coarsetime::Instant;

use std::pin::Pin;
use std::io::{Cursor, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::convert::TryFrom;
use std::net::SocketAddr;
use std::convert::TryInto;

use crate::bitfield::BitFieldUpdate;
use crate::utils::Map;
use crate::pieces::{Pieces, PieceToDownload, PieceBuffer};
use crate::supervisors::torrent::{TorrentNotification, Result};
use crate::errors::TorrentError;
use crate::extensions::{ExtendedMessage, ExtendedHandshake, PEXMessage};

static PEER_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub type PeerId = usize;

#[derive(Debug)]
enum MessagePeer<'a> {
    KeepAlive,
    Choke,
    UnChoke,
    Interested,
    NotInterested,
    Have {
        piece_index: u32
    },
    BitField(&'a [u8]),
    Request {
        index: u32,
        begin: u32,
        length: u32,
    },
    Piece {
        index: u32,
        begin: u32,
        block: &'a [u8],
    },
    Cancel {
        index: u32,
        begin: u32,
        length: u32,
    },
    Port(u16),
    Extension(ExtendedMessage<'a>),
    Unknown {
        id: u8,
        buffer: &'a [u8]
    }
}

impl<'a> TryFrom<&'a [u8]> for MessagePeer<'a> {
    type Error = TorrentError;

    fn try_from(buffer: &'a [u8]) -> Result<MessagePeer> {
        if buffer.is_empty() {
            return Ok(MessagePeer::KeepAlive);
        }
        let id = buffer[0];
        let buffer = &buffer[1..];
        Ok(match id {
            0 => MessagePeer::Choke,
            1 => MessagePeer::UnChoke,
            2 => MessagePeer::Interested,
            3 => MessagePeer::NotInterested,
            4 => {
                let mut cursor = Cursor::new(buffer);
                let piece_index = cursor.read_u32::<BigEndian>()?;

                MessagePeer::Have { piece_index }
            }
            5 => {
                MessagePeer::BitField(buffer)
            }
            6 => {
                let mut cursor = Cursor::new(buffer);
                let index = cursor.read_u32::<BigEndian>()?;
                let begin = cursor.read_u32::<BigEndian>()?;
                let length = cursor.read_u32::<BigEndian>()?;

                MessagePeer::Request { index, begin, length }
            }
            7 => {
                let mut cursor = Cursor::new(buffer);
                let index = cursor.read_u32::<BigEndian>()?;
                let begin = cursor.read_u32::<BigEndian>()?;
                let block = &buffer[8..];

                MessagePeer::Piece { index, begin, block }
            }
            8 => {
                let mut cursor = Cursor::new(buffer);
                let index = cursor.read_u32::<BigEndian>()?;
                let begin = cursor.read_u32::<BigEndian>()?;
                let length = cursor.read_u32::<BigEndian>()?;

                MessagePeer::Cancel { index, begin, length }
            }
            9 => {
                let mut cursor = Cursor::new(buffer);
                let port = cursor.read_u16::<BigEndian>()?;

                MessagePeer::Port(port)
            }
            20 => {
                let mut cursor = Cursor::new(buffer);
                let id = cursor.read_u8()?;

                match id {
                    0 => {
                        let handshake = crate::bencode::de::from_bytes(&buffer[1..])?;
                        MessagePeer::Extension(ExtendedMessage::Handshake(handshake))
                    }
                    _ => {
                        MessagePeer::Extension(ExtendedMessage::Message {
                            id,
                            buffer: &buffer[1..]
                        })
                    }
                }
            }
            id => MessagePeer::Unknown { id, buffer }
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Choke {
    UnChoked,
    Choked
}

use std::collections::VecDeque;

pub type PeerTask = Arc<async_std::sync::RwLock<VecDeque<PieceToDownload>>>;

#[derive(Debug)]
pub enum PeerCommand {
    TasksAvailables,
    Die
}

enum PeerWaitEvent {
    Peer(Result<usize>),
    Supervisor(Option<PeerCommand>),
}

use hashbrown::HashMap;

#[derive(Default)]
struct PeerDetail {
    extension_ids: HashMap<String, i64>,
}

impl PeerDetail {
    fn update_with_extension(&mut self, ext: ExtendedHandshake) {
        if let Some(m) = ext.m {
            self.extension_ids = m;
        };
    }
}

/// Peer extern ID
/// Correspond to peer_id in the protocol and is 20 bytes long
pub struct PeerExternId([u8; 20]);

impl PeerExternId {
    fn new(bytes: &[u8]) -> PeerExternId {
        PeerExternId(bytes.try_into().expect("PeerExternId must be 20 bytes"))
    }

    pub fn generate() -> PeerExternId {
        use rand::Rng;
        use rand::distributions::Alphanumeric;

        // TODO: Improve this

        const VERSION: usize = 1;

        let random = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .collect::<String>();

        let s = format!("-RR{:04}-{}", VERSION, random);

        let id = s.as_bytes()
                  .try_into()
                  .expect("PeerExternId are 20 bytes long");

        PeerExternId(id)
    }
}

use std::ops::Deref;

impl Deref for PeerExternId {
    type Target = [u8; 20];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for PeerExternId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", String::from_utf8_lossy(&self.0))
    }
}

impl PartialEq for PeerExternId {
    fn eq(&self, other: &PeerExternId) -> bool {
        super::sha1::compare_20_bytes(&self.0, &other.0)
    }
}

impl Eq for PeerExternId {}

pub struct Peer {
    id: PeerId,
    addr: SocketAddr,
    supervisor: Sender<TorrentNotification>,
    reader: BufReader<TcpStream>,
    buffer: Vec<u8>,
    /// Are we choked from the peer
    choked: Choke,
    /// List of pieces to download
    tasks: PeerTask,
    _tasks: Option<VecDeque<PieceToDownload>>,

    /// Small buffers where the downloaded blocks are kept.
    /// Once the piece is full, we send it to TorrentSupervisor
    /// and remove it here.
    pieces_buffer: Map<usize, PieceBuffer>,

    pieces_detail: Pieces,

    nblocks: usize, // Downloaded
    start: Option<Instant>, // Downloaded,

    peer_detail: PeerDetail,

    extern_id: Arc<PeerExternId>
}

impl Peer {
    pub async fn new(
        addr: SocketAddr,
        pieces_detail: Pieces,
        supervisor: Sender<TorrentNotification>,
        extern_id: Arc<PeerExternId>
    ) -> Result<Peer> {
        let stream = TcpStream::connect(&addr).await?;

        let id = PEER_COUNTER.fetch_add(1, Ordering::SeqCst);

        Ok(Peer {
            addr,
            supervisor,
            pieces_detail,
            id,
            extern_id,
            tasks: PeerTask::default(),
            reader: BufReader::with_capacity(32 * 1024, stream),
            buffer: Vec::with_capacity(32 * 1024),
            choked: Choke::Choked,
            nblocks: 0,
            start: None,
            _tasks: None,
            pieces_buffer: Map::default(),
            peer_detail: Default::default(),
        })
    }

    async fn send_message(&mut self, msg: MessagePeer<'_>) -> Result<()> {
        self.write_message_in_buffer(msg);

        let writer = self.reader.get_mut();
        writer.write_all(self.buffer.as_slice()).await?;
        writer.flush().await?;

        Ok(())
    }

    fn write_message_in_buffer(&mut self, msg: MessagePeer<'_>) {
        self.buffer.clear();
        let mut cursor = Cursor::new(&mut self.buffer);

        match msg {
            MessagePeer::Choke => {
                cursor.write_u32::<BigEndian>(1).unwrap();
                cursor.write_u8(0).unwrap();
            }
            MessagePeer::UnChoke => {
                cursor.write_u32::<BigEndian>(1).unwrap();
                cursor.write_u8(1).unwrap();
            }
            MessagePeer::Interested => {
                cursor.write_u32::<BigEndian>(1).unwrap();
                cursor.write_u8(2).unwrap();
            }
            MessagePeer::NotInterested => {
                cursor.write_u32::<BigEndian>(1).unwrap();
                cursor.write_u8(3).unwrap();
            }
            MessagePeer::Have { piece_index } => {
                cursor.write_u32::<BigEndian>(5).unwrap();
                cursor.write_u8(4).unwrap();
                cursor.write_u32::<BigEndian>(piece_index).unwrap();
            }
            MessagePeer::BitField (bitfield) => {
                cursor.write_u32::<BigEndian>(1 + bitfield.len() as u32).unwrap();
                cursor.write_u8(5).unwrap();
                cursor.write_all(bitfield).unwrap();
            }
            MessagePeer::Request { index, begin, length } => {
                cursor.write_u32::<BigEndian>(13).unwrap();
                cursor.write_u8(6).unwrap();
                cursor.write_u32::<BigEndian>(index).unwrap();
                cursor.write_u32::<BigEndian>(begin).unwrap();
                cursor.write_u32::<BigEndian>(length).unwrap();
            }
            MessagePeer::Piece { index, begin, block } => {
                cursor.write_u32::<BigEndian>(9 + block.len() as u32).unwrap();
                cursor.write_u8(7).unwrap();
                cursor.write_u32::<BigEndian>(index).unwrap();
                cursor.write_u32::<BigEndian>(begin).unwrap();
                cursor.write_all(block).unwrap();
            }
            MessagePeer::Cancel { index, begin, length } => {
                cursor.write_u32::<BigEndian>(13).unwrap();
                cursor.write_u8(8).unwrap();
                cursor.write_u32::<BigEndian>(index).unwrap();
                cursor.write_u32::<BigEndian>(begin).unwrap();
                cursor.write_u32::<BigEndian>(length).unwrap();
            }
            MessagePeer::Port (port) => {
                cursor.write_u32::<BigEndian>(3).unwrap();
                cursor.write_u8(9).unwrap();
                cursor.write_u16::<BigEndian>(port).unwrap();
            }
            MessagePeer::KeepAlive => {
                cursor.write_u32::<BigEndian>(0).unwrap();
            }
            MessagePeer::Extension(ExtendedMessage::Handshake(handshake)) => {
                let bytes = crate::bencode::ser::to_bytes(&handshake).unwrap();
                cursor.write_u32::<BigEndian>(2 + bytes.len() as u32).unwrap();
                cursor.write_u8(20).unwrap();
                cursor.write_u8(0).unwrap();
                cursor.write_all(&bytes).unwrap();
            }
            MessagePeer::Extension(ExtendedMessage::Message { .. }) => {

            }
            //MessagePeer::Extension { .. } => unreachable!()
            MessagePeer::Unknown { .. } => unreachable!(),
        }

        cursor.flush().unwrap();
    }

    fn writer(&mut self) -> &mut TcpStream {
        self.reader.get_mut()
    }

    async fn wait_event(
        &mut self,
        mut cmds: Pin<&mut Fuse<impl Future<Output = std::result::Result<PeerCommand, async_std::sync::RecvError>>>>
    ) -> PeerWaitEvent {
        // use futures::async_await::*;
        use futures::task::{Context, Poll};
        use futures::{future, pin_mut};

        let msgs = self.read_messages();
        pin_mut!(msgs); // Pin on the stack

        // assert_unpin(&msgs);
        // assert_unpin(&cmds);
        // assert_fused_future();

        let fun = |cx: &mut Context<'_>| {
            match FutureExt::poll_unpin(&mut msgs, cx).map(PeerWaitEvent::Peer) {
                v @ Poll::Ready(_) => return v,
                _ => {}
            }

            match FutureExt::poll_unpin(&mut cmds, cx).map(|v| v.ok()).map(PeerWaitEvent::Supervisor) {
                v @ Poll::Ready(_) => v,
                _ => Poll::Pending
            }
        };

        future::poll_fn(fun).await
    }

    pub async fn start(&mut self) -> Result<()> {

        let (addr, cmds) = channel(1000);

        let extern_id = self.do_handshake().await?;

        self.supervisor.send(TorrentNotification::AddPeer {
            id: self.id,
            queue: self.tasks.clone(),
            addr,
            socket: self.addr,
            extern_id
        }).await;

        let cmds = cmds.recv().fuse();
        let mut cmds = Box::pin(cmds);

        loop {
            use PeerWaitEvent::*;

            match self.wait_event(cmds.as_mut()).await {
                Peer(Ok(_n)) => {
                    self.dispatch().await?;
                }
                Supervisor(command) => {
                    use PeerCommand::*;

                    match command {
                        Some(TasksAvailables) => {
                            self.maybe_send_request().await?;
                        }
                        Some(Die) => {
                            return Ok(());
                        }
                        None => {
                            // Disconnected
                        }
                    }
                }
                Peer(Err(e)) => {
                    eprintln!("[{}] PEER ERROR MSG {:?}", self.id, e);
                    return Err(e);
                }
            }
        }
    }

    async fn take_tasks(&mut self) -> Option<PieceToDownload> {
        if self._tasks.is_none() {
            let t = self.tasks.read().await;
            self._tasks = Some(t.clone());
        }
        self._tasks.as_mut().and_then(|t| t.pop_front())
    }

    async fn maybe_send_request(&mut self) -> Result<()> {
        if !self.am_choked() {

            let task = match self._tasks.as_mut() {
                Some(tasks) => tasks.pop_front(),
                _ => self.take_tasks().await
            };

            if let Some(task) = task {
                // println!("[{}] SENT TASK {:?}", self.id, task);
                self.send_request(task).await?;
            } else {
                //self.pieces_actor.get_pieces_to_downloads().await;
                println!("[{:?}] No More Task ! {} downloaded in {:?}s", self.id, self.nblocks, self.start.map(|s| s.elapsed().as_secs()));
                // Steal others tasks
            }
        } else {
            self.send_message(MessagePeer::Interested).await?;
            println!("[{}] SENT INTERESTED", self.id);
        }
        Ok(())
    }

    async fn send_request(&mut self, task: PieceToDownload) -> Result<()> {
        self.send_message(MessagePeer::Request {
            index: task.piece,
            begin: task.start,
            length: task.size,
        }).await
    }

    fn set_choked(&mut self, choked: bool) {
        self.choked = if choked {
            Choke::Choked
        } else {
            Choke::UnChoked
        };
    }

    fn am_choked(&self) -> bool {
        self.choked == Choke::Choked
    }

    async fn dispatch(&mut self) -> Result<()> {
        use MessagePeer::*;

        let msg = MessagePeer::try_from(self.buffer.as_slice())?;

        match msg {
            Choke => {
                self.set_choked(true);
                println!("[{}] CHOKE", self.id);
            },
            UnChoke => {
                // If the peer has piece we're interested in
                // Send a Request
                self.set_choked(false);
                println!("[{}] UNCHOKE", self.id);

                self.maybe_send_request().await?;
            },
            Interested => {
                // Unshoke this peer
                println!("INTERESTED", );
            },
            NotInterested => {
                // Shoke this peer
                println!("NOT INTERESTED", );
            },
            Have { piece_index } => {
                use TorrentNotification::UpdateBitfield;

                let update = BitFieldUpdate::from(piece_index);

                self.supervisor.send(UpdateBitfield { id: self.id, update }).await;

                println!("[{:?}] HAVE {}", self.id, piece_index);
            },
            BitField (bitfield) => {
                // Send an Interested ?
                use crate::bitfield::BitField;
                use TorrentNotification::UpdateBitfield;

                let bitfield = BitField::from(
                    bitfield,
                    self.pieces_detail.num_pieces
                )?;

                let update = BitFieldUpdate::from(bitfield);

                self.supervisor.send(UpdateBitfield { id: self.id, update }).await;

                println!("[{:?}] BITFIELD", self.id);
            },
            Request { index, begin, length } => {
                // Mark this peer as interested
                // Make sure this peer is not choked or resend a choke
                println!("REQUEST {} {} {}", index, begin, length);
            },
            Piece { index, begin, block } => {
                // If we already have it, send another Request
                // Check the sum and write to disk
                // Send Request
                //println!("[{:?}] PIECE {} {} {}", self.id, index, begin, block.len());

                if self.start.is_none() {
                    self.start.replace(Instant::now());
                }

                self.nblocks += block.len();

                let piece_size = self.pieces_detail
                                     .piece_size(index as usize);
                let mut is_completed = false;

                self.pieces_buffer
                    .entry(index as usize)
                    .and_modify(|p| {
                        p.add_block(begin, block);
                        is_completed = p.is_completed();
                    })
                    .or_insert_with(|| {
                        PieceBuffer::new_with_block(index, piece_size, begin, block)
                    });

                if is_completed {
                    self.send_completed(index).await;
                }

                self.maybe_send_request().await?;
            },
            Cancel { index, begin, length } => {
                // Cancel a Request
                println!("PIECE {} {} {}", index, begin, length);
            },
            Port (port) => {
                println!("PORT {}", port);
            },
            KeepAlive => {
                println!("KEEP ALICE");
            }
            Extension(ExtendedMessage::Handshake(_handshake)) => {
                self.send_extended_handshake().await?;
                //self.maybe_send_request().await;
                //println!("[{}] EXTENDED HANDSHAKE SENT", self.id);
            }
            Extension(ExtendedMessage::Message { id, buffer }) => {
                if id == 1 {
                    if let Ok(addrs) = crate::bencode::de::from_bytes::<PEXMessage>(buffer) {
                        self.supervisor.send(TorrentNotification::PeerDiscovered {
                            addrs: addrs.into()
                        }).await;
                    };
                }
            }
            Unknown { id, buffer } => {
                // Check extension
                // Disconnect
                println!("UNKNOWN {:?} {}", id, String::from_utf8_lossy(buffer));
            }
        }
        Ok(())
    }

    async fn send_extended_handshake(&mut self) -> Result<()> {
        let mut extensions = HashMap::new();
        extensions.insert("ut_pex".to_string(), 1);
        let handshake = ExtendedHandshake {
            m: Some(extensions),
            v: Some(String::from("Rustorrent 0.1")),
            p: Some(6801),
            ..Default::default()
        };
        self.send_message(MessagePeer::Extension(ExtendedMessage::Handshake(handshake))).await
    }

    async fn send_completed(&mut self, index: u32) {
        let piece_buffer = self.pieces_buffer.remove(&(index as usize)).unwrap();

        self.supervisor.send(TorrentNotification::AddPiece(piece_buffer)).await;
        //println!("[{}] PIECE COMPLETED {}", self.id, index);
    }

    async fn read_messages(&mut self) -> Result<usize> {
        self.read_exactly(4).await?;

        let length = {
            let mut cursor = Cursor::new(&self.buffer);
            cursor.read_u32::<BigEndian>()? as usize
        };

        if length == 0 {
            return Ok(0); // Keep Alive
        }

        self.read_exactly(length).await?;

        Ok(length)
    }

    async fn read_exactly(&mut self, n: usize) -> Result<()> {
        let reader = self.reader.by_ref();
        self.buffer.clear();

        if reader.take(n as u64).read_to_end(&mut self.buffer).await? != n {
            return Err(async_std::io::Error::new(
                async_std::io::ErrorKind::UnexpectedEof,
                "Size doesn't match"
            ).into());
        }

        Ok(())
    }

    async fn write(&mut self, data: &[u8]) -> Result<()> {
        let writer = self.writer();
        writer.write_all(data).await?;
        writer.flush().await?;
        Ok(())
    }

    async fn do_handshake(&mut self) -> Result<Arc<PeerExternId>> {
        let mut handshake: [u8; 68] = [0; 68];

        let mut cursor = Cursor::new(&mut handshake[..]);

        let mut reserved: [u8; 8] = [0,0,0,0,0,0,0,0];

        reserved[5] |= 0x10; // Support Extension Protocol

        cursor.write_all(&[19])?;
        cursor.write_all(b"BitTorrent protocol")?;
        cursor.write_all(&reserved[..])?;
        cursor.write_all(self.pieces_detail.info_hash.as_ref())?;
        cursor.write_all(&**self.extern_id)?;

        self.write(&handshake).await?;

        self.read_exactly(1).await?;
        let len = self.buffer[0] as usize;
        self.read_exactly(len + 48).await?;

        // TODO: Check the info hash and send to other TorrentSupervisor if necessary

        println!("[{}] HANDSHAKE DONE", self.id);

        let peer_id = PeerExternId::new(&self.buffer[len + 28..len + 48]);

        Ok(Arc::new(peer_id))
    }
}
