use std::{
    ops::{
        Add, 
        Sub, 
        AddAssign, 
        SubAssign
    }
};

/// Безопасный тип на основе u16 с безопасными математическими операциями
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct SequenceNumber(u16);

impl SequenceNumber {
    pub fn random() -> SequenceNumber {
        use rand::{
            prelude::{
                *
            }
        };
        SequenceNumber(rand::thread_rng().gen())
    }

    pub fn zero() -> SequenceNumber {
        SequenceNumber(0)
    }

    /// Доступно только текущему и родительскому модулю, конвертация в BigEndian
    pub(super) fn to_be(self) -> u16 {
        u16::to_be(self.0)
    }

    /// Доступно только текущему родительскому модулю, конвертация из BigEndian
    pub(super) fn from_be(n: u16) -> SequenceNumber {
        SequenceNumber(u16::from_be(n))
    }

    /// Сравниваем себя с другим значением. Нельзя реализовать 
    /// PartialOrd так как он не удовлетворяет антисимметрии
    pub fn cmp_less(self, other: SequenceNumber) -> bool {
	    let dist_down = self - other;
	    let dist_up = other - self;

	    dist_up.0 < dist_down.0
    }

    /// Сравниваем себя с другим значением. Нельзя реализовать 
    /// PartialOrd так как он не удовлетворяет антисимметрии
    pub fn cmp_less_equal(self, other: SequenceNumber) -> bool {
	    let dist_down = self - other;
	    let dist_up = other - self;

	    dist_up.0 <= dist_down.0
    }
}

impl From<u16> for SequenceNumber {
    fn from(n: u16) -> SequenceNumber {
        SequenceNumber(n)
    }
}

impl From<SequenceNumber> for u16 {
    fn from(s: SequenceNumber) -> u16 {
        s.0
    }
}

impl Add<u16> for SequenceNumber {
    type Output = Self;

    fn add(self, n: u16) -> Self {
        Self(self.0.wrapping_add(n))
    }
}

impl AddAssign<u16> for SequenceNumber {
    fn add_assign(&mut self, other: u16) {
        // Use Add impl, with wrapping
        *self = *self + other;
    }
}

impl Sub<u16> for SequenceNumber {
    type Output = Self;

    fn sub(self, n: u16) -> Self {
        Self(self.0.wrapping_sub(n))
    }
}

impl Sub<SequenceNumber> for SequenceNumber {
    type Output = Self;

    fn sub(self, n: SequenceNumber) -> Self {
        Self(self.0.wrapping_sub(n.0))
    }
}

impl SubAssign<u16> for SequenceNumber {
    fn sub_assign(&mut self, other: u16) {
        // Use Sub impl, with wrapping
        *self = *self - other;
    }
}
