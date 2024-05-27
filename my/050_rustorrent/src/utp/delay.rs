use std::{
    ops::{
        Sub, 
    },
    cmp::{
        PartialOrd, 
        Ord
    }
};
use super::{
    timestamp::{
        Timestamp
    },
    relative_delay::{
        RelativeDelay
    }
};

#[derive(Debug, Copy, Clone, Default, Ord, PartialEq, Eq, PartialOrd)]
pub struct Delay(u32);

impl Delay {
    pub fn since(timestamp: Timestamp) -> Delay {
        let now = Timestamp::now();
        now.delay(timestamp)
    }

    pub fn infinity() -> Delay {
        Delay(u32::max_value())
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl Sub for Delay {
    type Output = RelativeDelay;

    fn sub(self, other: Delay) -> RelativeDelay {
        RelativeDelay(self.0.saturating_sub(other.0))
    }
}

impl From<u32> for Delay {
    fn from(n: u32) -> Delay {
        Delay(n)
    }
}

impl Into<u32> for Delay {
    fn into(self) -> u32 {
        self.0 as u32
    }
}