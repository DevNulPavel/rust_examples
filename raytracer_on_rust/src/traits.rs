
// TODO: оставить тут только общие трейты, остальные разнести

pub trait Zero {
    fn zero() -> Self;
}

pub trait Normalizable {
    fn normalize(&self) -> Self;
}

pub trait Length {
    fn length(&self) -> f32;
}

pub trait Clamp<T> {
    fn clamp(self, min: T, max: T) -> Self;
}

pub trait Dotable {
    type Operand;
    fn dot(&self, other: &Self::Operand) -> f32;
}

/*pub trait Iterable{
    type Item;
    type Out: Iterator<Item=&'a dyn Self::Item>;
    fn iter<'a>(&'a self) -> Self::Out;
}*/