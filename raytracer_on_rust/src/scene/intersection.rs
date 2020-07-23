use crate::{
    traits::{
        Figure
    }
};

pub struct Intersection<'a> {
    pub distance: f32,
    pub object: &'a dyn Figure,
}

impl<'a> Intersection<'a> {
    pub fn new(distance: f32, object: &'a dyn Figure) -> Intersection<'a> {
        Intersection{
            distance,
            object
        }
    }
}