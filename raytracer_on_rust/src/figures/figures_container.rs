use std::{
    iter::{
        Iterator,
    }
};
// use crate::{
//     traits::{
//         Iterable
//     }
// };
use super::{
    traits::{
        Figure
    },
    plane::{
        Plane
    },
    sphere::{
        Sphere
    }
};

/// Позволяет избавиться от конструкции вида Vec<Box<dyn Figure>>
// тем самым при итерировании у нас объекты будут располагаться рядом
pub struct FiguresContainer{
    pub spheres: Vec<Sphere>,
    pub planes: Vec<Plane>,
}

impl FiguresContainer{
    // TODO: Может есть трейт какой-то???
    // fn get_iter<'a, T>(&'a self) -> T where T: Iterator<Item=&'a dyn Light> {
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=&'a dyn Figure> {
        let spheres_iter = self.spheres
            .iter()
            .map(|figure|{
                let light: &dyn Figure = figure;
                light
            });
        let planes_iter = self.planes
            .iter()
            .map(|figure|{
                let light: &dyn Figure = figure;
                light
            });
        
        let final_iter = spheres_iter
            .chain(planes_iter);

        final_iter
    }
}