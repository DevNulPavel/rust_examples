use super::{directional_light::DirectionalLight, spherical_light::SphericalLight, traits::Light};
use std::iter::Iterator;

/// Позволяет избавиться от конструкции вида Vec<Box<dyn Light>>
// тем самым при итерировании у нас объекты будут располагаться рядом
pub struct LightsContainer {
    pub directional: Vec<DirectionalLight>,
    pub spherical: Vec<SphericalLight>,
}

impl LightsContainer {
    // TODO: Может есть трейт какой-то???
    // fn get_iter<'a, T>(&'a self) -> T where T: Iterator<Item=&'a dyn Light> {
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a dyn Light> {
        let directional_iter = self.directional.iter().map(|light| {
            let light: &dyn Light = light;
            light
        });
        let spherical_iter = self.spherical.iter().map(|light| {
            let light: &dyn Light = light;
            light
        });

        let final_iter = directional_iter.chain(spherical_iter);

        final_iter
    }
}
