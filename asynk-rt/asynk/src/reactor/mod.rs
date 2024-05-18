pub mod io_handle;

use mio::{event::Source, Events, Interest, Poll, Registry, Token};
use sharded_slab::Slab;
use std::{
    io::{self, Error, Result},
    sync::{Arc, OnceLock},
    task::Waker,
    thread::{self},
};

/// Reactor polls events from mio and calls wakers interested
/// by these events
pub struct Reactor {
    wakers: Arc<Slab<Waker>>,
    registry: Registry,
}

static REACTOR: OnceLock<Reactor> = OnceLock::new();

impl Reactor {
    pub fn new() -> Result<Self> {
        let poll = Poll::new()?;
        let registry = poll.registry().try_clone()?;

        let wakers = Arc::new(Slab::new());

        // Spawn poll events thread
        thread::Builder::new().name("reactor".into()).spawn({
            let wakers = Arc::clone(&wakers);
            move || Self::poll_events_routine(wakers, poll)
        })?;

        Ok(Self { registry, wakers })
    }

    pub fn get() -> &'static Reactor {
        REACTOR.get().expect("reactor is not set")
    }

    pub fn set_global(self) {
        REACTOR.set(self).ok();
    }

    /// Register interested events for the given source
    pub fn register<S>(
        &self,
        source: &mut S,
        interests: Interest,
        waker: Waker,
    ) -> io::Result<Token>
    where
        S: Source,
    {
        let token = self
            .wakers
            .insert(waker)
            .ok_or(Error::other("slab queue is full"))?;

        let token = Token(token);

        self.registry.register(source, token, interests)?;

        Ok(token)
    }

    pub fn reregister<S>(
        &self,
        token: Token,
        source: &mut S,
        interests: Interest,
        waker: Waker,
    ) -> io::Result<Token>
    where
        S: Source,
    {
        self.wakers.remove(token.into());

        let new_token = Token(
            self.wakers
                .insert(waker)
                .ok_or(Error::other("slab queue is full"))?,
        );

        self.registry.reregister(source, new_token, interests)?;

        Ok(new_token)
    }

    pub fn poll_events_routine(wakers: Arc<Slab<Waker>>, mut poll: Poll) {
        let mut events = Events::with_capacity(1024);

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in events.into_iter() {
                if let Some(waker) = wakers.get(event.token().into()) {
                    // Call waker interested by this event
                    waker.wake_by_ref();
                }
            }
        }
    }

    /// Remove the interests for the given source
    pub fn deregister<S>(&self, token: Token, source: &mut S) -> io::Result<()>
    where
        S: Source,
    {
        self.registry.deregister(source)?;
        self.wakers.get(token.0);
        Ok(())
    }
}
