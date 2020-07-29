mod sphere;
mod plane;
mod traits;
mod figures_container;

pub(crate) use self::{
    traits::{
        Figure,
        Texturable
    },
    figures_container::{
        FiguresContainer
    },
    sphere::{
        Sphere
    },
    plane::{
        Plane
    }
};