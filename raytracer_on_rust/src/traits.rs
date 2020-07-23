pub trait Zero {
    fn zero() -> Self;
}

pub trait Normalize {
    fn normalize(&self) -> Self;
}