mod traits;
mod solid_color_material;
mod texture_material;
mod tex_coord_delegate;
mod refraction_info;
mod material_modificator;
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
    },
    refraction_info::{
        RefractionInfo
    },
    material_modificator::{
        MaterialModificator
    }
};