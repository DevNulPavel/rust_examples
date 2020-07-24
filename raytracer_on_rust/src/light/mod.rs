mod traits;
mod directional_light;
mod spherical_light;
mod light_distance;

pub use self::{
    traits::{
        Light
    },
    light_distance::{
        LightDistance
    },
    directional_light::{
        DirectionalLight
    },
    spherical_light::{
        SphericalLight
    }
};