use std::{
    path::{
        Path
    }
};
// Использование общих трейтов и классов через crate от корня
use crate::{
    traits::{
        Normalizable
    },
    structs::{
        Vector3,
        Color
    },
    material::{
        MaterialsContainer,
        SolidColorMaterial,
        TextureMaterial,
        RefractionInfo,
        MaterialModificator
    },
    figures::{
        FiguresContainer,
        Sphere,
        Plane
    },
    light::{
        DirectionalLight,
        SphericalLight,
        LightsContainer
    }
};
// Использование соседних файликов через super
use super::{
    scene::{
        Scene
    }
};

pub fn build_test_scene() -> Scene {
    // Список сфер
    let figures: FiguresContainer = FiguresContainer{
        // 1
        spheres: vec![
            Sphere {
                center: Vector3 {
                    x: 0.0,
                    y: -0.6,
                    z: -6.0,
                },
                radius: 0.8,
                material: MaterialsContainer::Solid(SolidColorMaterial{
                    diffuse_solid_color: Color {
                        red: 0.4,
                        green: 1.0,
                        blue: 0.4,
                    },
                    modificator: MaterialModificator::Reflection(0.6)
                })
            },
            // 2
            Sphere {
                center: Vector3 {
                    x: 0.2,
                    y: 0.2,
                    z: -2.0,
                },
                radius: 0.5,
                material: MaterialsContainer::Solid(SolidColorMaterial{
                    diffuse_solid_color: Color {
                        red: 1.0,
                        green: 0.1,
                        blue: 0.3,
                    },
                    modificator: MaterialModificator::Refraction(RefractionInfo{
                        index: 0.99,
                        transparense_level: 0.8
                    })
                })
            }
        ],
        // 3
        planes: vec![
            Plane {
                origin: Vector3 {
                    x: 0.0,
                    y: -2.0,
                    z: -4.0,
                },
                normal: Vector3 {
                    x: 0.0,
                    y: -1.0,
                    z: 0.0,
                },
                material: MaterialsContainer::Texture(TextureMaterial{
                    texture: image::open(Path::new("res/grass.jpg")).unwrap(),
                    modificator: MaterialModificator::None
                })
            }
        ],
    };


    let lights: LightsContainer = LightsContainer{
        directional: vec![
            DirectionalLight{
                direction: Vector3{
                    x: 0.0,
                    y: -1.0,
                    z: -1.0
                }.normalize(),
                color: Color{
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0
                },
                intensity: 0.3
            },
            DirectionalLight{
                direction: Vector3{
                    x: 1.0,
                    y: -1.0,
                    z: -1.0
                }.normalize(),
                color: Color{
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0
                },
                intensity: 0.2
            }
        ],
        spherical: vec![
            SphericalLight{
                position: Vector3{
                    x: -1.0,
                    y: 0.8,
                    z: -0.5
                },
                color: Color{
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0
                },
                intensity: 0.9
            }
        ]
    };

    let scene = Scene {
        width: 1024,
        height: 768,
        fov: 90.0,
        ambient_light_intensivity: 0.3,
        bias: 0.000006_f32,
        max_recursive_level: 4,
        lights: lights,
        figures,
    };
    
    scene
}