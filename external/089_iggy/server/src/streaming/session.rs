use iggy::models::user_info::{AtomicUserId, UserId};
use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};

// This might be extended with more fields in the future e.g. custom name, permissions etc.
#[derive(Debug)]
pub struct Session {
    user_id: AtomicUserId,
    active: AtomicBool,
    pub client_id: u32,
    pub ip_address: SocketAddr,
}

impl Session {
    pub fn new(client_id: u32, user_id: UserId, ip_address: SocketAddr) -> Self {
        Self {
            client_id,
            active: AtomicBool::new(true),
            user_id: AtomicUserId::new(user_id),
            ip_address,
        }
    }

    pub fn stateless(user_id: UserId, ip_address: SocketAddr) -> Self {
        Self::new(0, user_id, ip_address)
    }

    pub fn from_client_id(client_id: u32, ip_address: SocketAddr) -> Self {
        Self::new(client_id, 0, ip_address)
    }

    pub fn get_user_id(&self) -> UserId {
        self.user_id.load(Ordering::Acquire)
    }

    pub fn set_user_id(&self, user_id: UserId) {
        self.user_id.store(user_id, Ordering::Release)
    }

    pub fn set_stale(&self) {
        self.active.store(false, Ordering::Release)
    }

    pub fn clear_user_id(&self) {
        self.set_user_id(0)
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub fn is_authenticated(&self) -> bool {
        self.get_user_id() > 0
    }
}

impl Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let user_id = self.get_user_id();
        if user_id > 0 {
            return write!(
                f,
                "client ID: {}, user ID: {}, IP address: {}",
                self.client_id, user_id, self.ip_address
            );
        }

        write!(
            f,
            "client ID: {}, IP address: {}",
            self.client_id, self.ip_address
        )
    }
}
