// Использование общих трейтов и классов через crate от корня
use crate::{
    traits::{
        Figure,
        Normalize
        // PixelColor
    },
    structs::{
        Vector3,
        Color
    },
    figures::{
        Sphere,
        Plane
    },
    render::{
        Ray
    }
};
// Использование соседних файликов через super
use super::{
    intersection::{
        Intersection,
    },
    light::{
        Light
    }
};

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub fov: f64,
    pub light: Light,
    pub figures: Vec<Box<dyn Figure>>,
}

impl Scene {
    pub fn trace<'a>(&'a self, ray: &Ray) -> Option<Intersection<'a>> {
        // Обходим все сферы
        self.figures
            .iter()
            // Фильтруем только найденные пересечения
            .filter_map(|s| {
                let found_opt = s.intersect(ray);
                // На всякий пожарный фильтруем Nan значения
                match found_opt {
                    Some(val) =>{
                        if !val.is_nan(){
                            Some((val, s))
                        }else{
                            None
                        }
                    },
                    None => {
                        None
                    }
                }
            }
            // Создаем объект пересечения со ссылкой
            .map(|(d, s)| {
                let figure: &'a dyn Figure = s.as_ref();
                Intersection::new(d, figure)
            }))
            // Находим среди всех минимум
            .min_by(|i1, i2| {
                // Можно спокойно вызывать unwrap(), так как Nan был отфильтрован выше
                i1.distance
                    .partial_cmp(&i2.distance)
                    .unwrap()
            })
    }
}

pub fn build_test_scene() -> Scene {
    // Список сфер
    let figures: Vec<Box<dyn Figure>> = vec![
        // 1
        Box::new(Sphere {
            center: Vector3 {
                x: 0.0,
                y: 0.0,
                z: -5.0,
            },
            radius: 1.0,
            color: Color {
                red: 0.4,
                green: 1.0,
                blue: 0.4,
            },
        }),
        // 2
        Box::new(Sphere {
            center: Vector3 {
                x: 1.0,
                y: 1.0,
                z: -4.0,
            },
            radius: 0.5,
            color: Color {
                red: 0.0,
                green: 0.0,
                blue: 0.9,
            },
        }),
        // 3
        Box::new(Plane {
            origin: Vector3 {
                x: 0.0,
                y: -2.0,
                z: -3.0,
            },
            normal: Vector3 {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            color: Color {
                red: 1.0,
                green: 0.0,
                blue: 0.2,
            },
        }),
    ];


    let light = { 
        let direction = Vector3{
            x: -2.0,
            y: 2.0,
            z: -1.0
        };
        Light{
            direction: direction.normalize(),
            color: Color{
                red: 1.0,
                green: 1.0,
                blue: 1.0
            },
            intensity: 0.6
        }
    };

    let scene = Scene {
        width: 800,
        height: 600,
        fov: 90.0,
        light: light,
        figures,
    };
    
    scene
}