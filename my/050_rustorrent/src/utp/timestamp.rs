use super::{
    delay::Delay
};

/// Конкретное время
#[derive(Debug, Copy, Clone)]
pub struct Timestamp(u32);

impl Timestamp {
    pub fn now() -> Timestamp {
        use crate::time;

        let (sec, nano) = time::get_time();
        Timestamp((sec * 1_000_000 + nano / 1000) as u32)
        // let since_epoch = coarsetime::Clock::now_since_epoch();
        // println!("SINCE_EPOCH {:?}", since_epoch);
        // let now = since_epoch.as_secs() * 1_000_000 + (since_epoch.subsec_nanos() / 1000) as u64;
        // Timestamp(now as u32)
    }

    pub(super) fn zero() -> Timestamp {
        Timestamp(0)
    }

    pub fn elapsed_millis(self, now: Timestamp) -> u32 {
        //let now = Timestamp::now().0 / 1000;
        (now.0 / 1000) - (self.0 / 1000)
    }

    // pub fn elapsed_millis(self) -> u32 {
    //     let now = Timestamp::now().0 / 1000;
    //     now - (self.0 / 1000)
    // }

    // return uint64(ts.tv_sec) * 1000000 + uint64(ts.tv_nsec) / 1000;

    pub fn delay(self, o: Timestamp) -> Delay {
        if self.0 > o.0 {
            (self.0 - o.0).into()
        } else {
            (o.0 - self.0).into()
        }
    }
}

impl From<u32> for Timestamp {
    fn from(n: u32) -> Timestamp {
        Timestamp(n)
    }
}

impl Into<u32> for Timestamp {
    fn into(self) -> u32 {
        self.0
    }
}