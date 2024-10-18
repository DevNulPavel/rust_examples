mod figures_container;
mod plane;
mod sphere;
mod traits;

pub(crate) use self::{
    figures_container::FiguresContainer,
    plane::Plane,
    sphere::Sphere,
    traits::{Figure, Texturable},
};
