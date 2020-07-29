mod traits;
mod solid_color_material;
mod texture_material;
mod tex_coord_delegate;
mod materials_container;

pub use self::{
    traits::{
        Material
    },
    materials_container::{
        MaterialsContainer
    },
    tex_coord_delegate::{
        TexCoordDelegate
    },
    solid_color_material::{
        SolidColorMaterial
    },
    texture_material::{
        TextureMaterial
    }
};