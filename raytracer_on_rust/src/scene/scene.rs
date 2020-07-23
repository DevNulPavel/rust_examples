// Использование общих трейтов и классов через crate от корня
use crate::{
    traits::{
        Figure,
        Normalizable,
        Dotable,
        Clamp
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
    pub fn trace_intersection<'a>(&'a self, ray: &Ray) -> Option<Intersection<'a>> {
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
                // Место, где у нас нашлось пересечение
                let hit_point = ray.origin + (ray.direction * d);

                // Объект
                let figure: &'a dyn Figure = s.as_ref();

                Intersection::new(d, hit_point, figure)
            }))
            // Находим среди всех минимум
            .min_by(|i1, i2| {
                // Можно спокойно вызывать unwrap(), так как Nan был отфильтрован выше
                i1.distance
                    .partial_cmp(&i2.distance)
                    .unwrap()
            })
    }

    // Для найденного пересечения расчитываем цвет пикселя
    pub fn calculate_intersection_color(&self, intersection: &Intersection) -> Color{
        // Нормаль в точке пересечения
        let surface_normal = intersection.get_normal();
        
        // Направление к свету
        let direction_to_light = -self.light.direction;
        
        // Вычисляем свет как скалярное произведение (косинус угла между векторами),
        // чем сонаправленнее, тем сильнее
        let light_power = (surface_normal.dot(&direction_to_light) as f32) * self.light.intensity;

        // Стандартный цвет объекта
        let diffuse_color = intersection.object.get_diffuse_color().clone();

        // Финальный цвет
        let result_color: Color = diffuse_color * light_power;
        
        result_color.clamp(0.0_f32, 1.0_f32)
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
            radius: 1.5,
            diffuse_color: Color {
                red: 0.4,
                green: 1.0,
                blue: 0.4,
            },
            albedo_color: Color {
                red: 0.4,
                green: 1.0,
                blue: 0.4,
            }
        }),
        // 2
        Box::new(Sphere {
            center: Vector3 {
                x: 1.0,
                y: 1.0,
                z: -4.0,
            },
            radius: 1.3,
            diffuse_color: Color {
                red: 0.0,
                green: 0.0,
                blue: 0.9,
            },
            albedo_color: Color {
                red: 0.0,
                green: 0.0,
                blue: 0.9,
            }
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
            diffuse_color: Color {
                red: 1.0,
                green: 0.0,
                blue: 0.2,
            },
            albedo_color: Color {
                red: 1.0,
                green: 0.0,
                blue: 0.2,
            },
        }),
    ];


    let light = { 
        let direction = Vector3{
            x: -1.0,
            y: -2.0,
            z: -1.0
        };
        Light{
            direction: direction.normalize(),
            color: Color{
                red: 1.0,
                green: 1.0,
                blue: 1.0
            },
            intensity: 0.9
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