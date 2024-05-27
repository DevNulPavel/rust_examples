

use std::{
    time::{
        Instant, 
        Duration
    }
};
use super::{
    delay::{
        Delay
    },
    relative_delay::{
        RelativeDelay
    }
};

#[derive(Debug)]
pub(super) struct DelayHistory {
    /// Array of delays, 1 per minute
    /// history[x] is the lowest delay in the minute x
    history: [Delay; 20],
    /// Index in `history` array
    index: u8,
    /// Number of delays in the current minute
    ndelays: u16,
    /// Lowest delay in the last 20 mins
    lowest: Delay,
    next_index_time: Instant,
    /// 3 lowest relative delays
    last_relatives: [RelativeDelay; 3],
    /// Index in `last_relatives`
    index_relative: u8,
}

impl DelayHistory {
    pub(super) fn new() -> DelayHistory {
        DelayHistory {
            history: [Delay::infinity(); 20],
            index: 0,
            ndelays: 0,
            lowest: Delay::infinity(),
            next_index_time: Instant::now() + Duration::from_secs(1),
            last_relatives: [RelativeDelay::infinity(); 3],
            index_relative: 0,
        }
    }

    pub fn get_lowest(&self) -> Delay {
        self.lowest
    }

    pub fn add_delay(&mut self, delay: Delay) {
        self.ndelays = self.ndelays.saturating_add(1);

        let index = self.index as usize;
        if delay < self.lowest {
            self.lowest = delay;
            self.history[index] = delay;
        } else if delay < self.history[index] {
            self.history[index] = delay;
        }

        let value = delay - self.lowest;
        self.save_relative(value);

        if self.ndelays > 120 &&
            self.next_index_time
                .checked_duration_since(Instant::now())
                .is_some()
        {
            self.next_index_time = Instant::now() + Duration::from_secs(1);
            self.index = (self.index + 1) % self.history.len() as u8;
            self.ndelays = 0;
            self.history[self.index as usize] = delay;
            self.lowest = self.history.iter().min().copied().unwrap();
        }

        //println!("HISTORY {:?}", self);
        //println!("VALUE {:?} FROM {:?} AND {:?}", value, delay, self.lowest);
    }

    fn save_relative(&mut self, relative: RelativeDelay) {
        let index = self.index_relative as usize;
        self.last_relatives[index] = relative;
        self.index_relative = ((index + 1) % self.last_relatives.len()) as u8;
    }

    pub(super) fn lowest_relative(&self) -> RelativeDelay {
        self.last_relatives.iter().min().copied().unwrap()
    }

    // fn lowest_in_history(&self) -> Delay {
    //     let mut lowest = self.history[0];
    //     for delay in &self.history {
    //         if delay.cmp_less(lowest) {
    //             lowest = *delay;
    //         }
    //     }
    //     lowest
    // }
}