

#[derive(Debug, Copy, Clone, Default, Ord, PartialEq, Eq, PartialOrd)]
pub struct RelativeDelay(pub u32);

impl RelativeDelay {
    pub(super) fn infinity() -> RelativeDelay {
        RelativeDelay(u32::max_value())
    }

    pub fn as_i64(self) -> i64 {
        self.0 as i64
    }
}