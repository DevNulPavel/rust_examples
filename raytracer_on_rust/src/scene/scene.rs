// Использование общих трейтов и классов через crate от корня
use crate::{
    traits::{
        Normalizable,
        Dotable,
        Clamp
        // PixelColor
    },
    structs::{
        Vector3,
        Color
    },
    material::{
        Material
    },
    figures::{
        FiguresContainer,
        Figure,
        Sphere,
        Plane
    },
    render::{
        Ray
    },
    light::{
        Light,
        LightDistance,
        DirectionalLight,
        SphericalLight,
        LightsContainer
    }
};
// Использование соседних файликов через super
use super::{
    intersection::{
        Intersection,
    }
};

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub fov: f32,
    pub ambient_light_intensivity: f32,
    pub lights: LightsContainer,
    pub figures: FiguresContainer,
}

impl Scene {
    /// Находим пересечение с ближайшим объектом
    pub fn trace_nearest_intersection<'a>(&'a self, ray: &Ray) -> Option<Intersection<'a>> {
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
                let hit_point: Vector3 = ray.origin + (ray.direction * d);

                // Объект
                let figure: &'a dyn Figure = s;

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

    /// Находим пересечение с любым объектом, используется для теней
    fn trace_first_intersection<'a>(&'a self, ray: &Ray) -> Option<Intersection<'a>> {
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
                let hit_point: Vector3 = ray.origin + (ray.direction * d);

                // Объект
                let figure: &'a dyn Figure = s;

                Intersection::new(d, hit_point, figure)
            }))
            .take(1)
            .next()
    }

    // Для найденного пересечения расчитываем цвет пикселя
    pub fn calculate_intersection_color(&self, intersection: &Intersection) -> Color{
        // https://bheisler.github.io/post/writing-raytracer-in-rust-part-2/

        // Нормаль в точке пересечения
        let surface_normal = intersection.get_normal();
        
        // Идем по всем источникам света и получаем суммарный свет
        let directional_light_intensivity: f32 = self.lights
            .iter()
            .map(|light: &dyn Light|{
                // Направление к свету
                let direction_to_light = light.direction_to_light(&intersection.hit_point);

                // В из найденной точки пересечения снова пускем луч к источнику света
                let shadow_ray = {
                    // Делаем небольшое смещение от точки пересечения, чтобы не было z-fight
                    let shadow_ray_offset = surface_normal * 0.000004_f32;
                    Ray {
                        origin: intersection.hit_point + shadow_ray_offset,
                        direction: direction_to_light,
                    }
                };

                // Нашли пересечение или нет с чем-то для определения тени
                let in_light: bool = match self.trace_first_intersection(&shadow_ray){
                    Some(shadow_intersection) => {
                        // Если расстояние до источника света меньше, 
                        // чем до ближайшего пересечения луча тени c объектом - значит все ок
                        let from_shadow_hit_to_light_dist = light.distance_to_point(&intersection.hit_point);
                        match from_shadow_hit_to_light_dist {
                            LightDistance::Some(from_light_to_hit_dist) => {
                                shadow_intersection.distance > from_light_to_hit_dist
                            }
                            LightDistance::Infinite => {
                                false
                            }
                        }
                    },
                    None => {
                        true
                    }
                };
                
                // Определяем интенсивность свечения в зависимости от тени
                let light_intensity = if in_light { 
                    light.intensivity_for_point(&intersection.hit_point) 
                } else { 
                    return 0.0_f32;
                };

                // Вычисляем свет как скалярное произведение (косинус угла между векторами),
                // чем сонаправленнее, тем сильнее
                let light_power = (surface_normal.dot(&direction_to_light) as f32) * light_intensity;

                light_power
            })
            .sum::<f32>()
            .min(1.0_f32);

        // Стандартный цвет объекта
        let diffuse_color = intersection.get_color();

        // Финальный цвет
        let result_color: Color = diffuse_color * directional_light_intensivity;
        
        // Ограничим значениями 0 - 1
        result_color.clamp(0.0_f32, 1.0_f32)
    }
}

pub fn build_test_scene() -> Scene {
    // Список сфер
    let figures: FiguresContainer = FiguresContainer{
        // 1
        spheres: vec![
            Sphere {
                center: Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: -5.0,
                },
                radius: 0.8,
                material: Material{
                    diffuse_color: Color {
                        red: 0.4,
                        green: 1.0,
                        blue: 0.4,
                    }    
                }
            },
            // 2
            Sphere {
                center: Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: -3.0,
                },
                radius: 1.0,
                material: Material{
                    diffuse_color: Color {
                        red: 1.0,
                        green: 0.1,
                        blue: 0.3,
                    }    
                }
            }
        ],
        // 3
        planes: vec![
            Plane {
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
                material: Material{
                    diffuse_color: Color {
                        red: 0.3,
                        green: 0.3,
                        blue: 1.0,
                    }    
                }
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
                    y: 1.0,
                    z: 0.0
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
        width: 800,
        height: 600,
        fov: 90.0,
        ambient_light_intensivity: 0.3,
        lights: lights,
        figures,
    };
    
    scene
}