use rand::{
    prelude::{
        *
    }
};
// Использование общих трейтов и классов через crate от корня
use crate::{
    traits::{
        Normalizable,
        Dotable,
        Clamp,
        Zero
        // PixelColor
    },
    structs::{
        Vector3,
        Color
    },
    material::{
        RefractionInfo,
        MaterialModificator
    },
    figures::{
        FiguresContainer,
        Figure
    },
    render::{
        Ray
    },
    light::{
        Light,
        LightDistance,
        LightsContainer
    }
};
// Использование соседних файликов через super
use super::{
    intersection::{
        Intersection
    },
    intersection_full::{
        IntersectionFull
    }
};

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub fov: f32,
    pub ambient_light_intensivity: f32,
    pub bias: f32,
    pub max_recursive_level: u32,
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
                i1.get_distance()
                    .partial_cmp(&i2.get_distance())
                    .unwrap()
            })
    }

    /// Для найденного пересечения расчитываем цвет пикселя
    pub fn calculate_intersection_color(&self, ray: &Ray, intersection: &IntersectionFull) -> Color{
        self.calculate_intersection_color_with_level(ray, intersection, 0)
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

    /// Получаем степень освещенности для конкретного пересечения и нормали
    fn calculate_light_intensivity(&self, intersection: &IntersectionFull) -> f32{
        // Идем по всем источникам света и получаем суммарный свет
        let light_intensivity: f32 = self.lights
            .iter()
            .map(|light: &dyn Light|{
                // Направление к свету
                let direction_to_light = light.direction_to_light(&intersection.get_hit_point());

                // В из найденной точки пересечения снова пускем луч к источнику света
                let shadow_ray = {
                    // Делаем небольшое смещение от точки пересечения, чтобы не было z-fight
                    let shadow_ray_offset = *intersection.get_normal() * self.bias;
                    Ray {
                        origin: *intersection.get_hit_point() + shadow_ray_offset,
                        direction: direction_to_light,
                    }
                };

                // Нашли пересечение или нет с чем-то для определения тени
                let in_light: bool = match self.trace_first_intersection(&shadow_ray){
                    Some(shadow_intersection) => {
                        // Если расстояние до источника света меньше, 
                        // чем до ближайшего пересечения луча тени c объектом - значит все ок
                        let from_shadow_hit_to_light_dist = light.distance_to_point(intersection.get_hit_point());
                        match from_shadow_hit_to_light_dist {
                            LightDistance::Some(from_light_to_hit_dist) => {
                                shadow_intersection.get_distance() > from_light_to_hit_dist
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
                    light.intensivity_for_point(&intersection.get_hit_point()) 
                } else { 
                    return 0.0_f32;
                };

                // Вычисляем свет как скалярное произведение (косинус угла между векторами),
                // чем сонаправленнее, тем сильнее
                let light_power = (intersection.get_normal().dot(&direction_to_light) as f32) * light_intensity;

                light_power
            })
            .sum::<f32>()
            .min(1.0_f32);
        
        light_intensivity
    }

    /// Получаем степень влияния окружающих объектов на текущее пересечение
    fn calculate_ambient_color(&self, intersection: &IntersectionFull, cur_level: u32) -> Color {
        // TODO: Долго ли занимает инициализация данного объекта? Может вынести выше?
        let mut rng = rand::thread_rng();

        let normal = intersection.get_normal();

        // TODO: Работает как-то неправильно
        // Но данный эффект надо делать с помощью кидания кучи лучей вокруг
        // Эффект Ambient Occlusion?
        const AMBIEN_ITERATIONS_COUNT: usize = 5;
        let ambient_color = (0..AMBIEN_ITERATIONS_COUNT)
            .fold(Color::zero(), |acc, _|{
                let random_offset = Vector3{
                    x: rng.gen(),
                    y: rng.gen(),
                    z: rng.gen(),
                }.normalize() * 0.5;
                let random_direction = (*normal + random_offset).normalize();

                // Создаем луч по направлению нормали для получения внешних цветов
                let ambient_ray = Ray{
                    origin: *intersection.get_hit_point() + *normal * self.bias,
                    direction: random_direction
                };

                // Находим пересечение с объектом для луча отражения
                let ambient_intersection = self.trace_nearest_intersection(&ambient_ray);

                let tmp_color = match ambient_intersection {
                    Some(ambient_intersection) => {
                        let full_ambient_intersection: IntersectionFull = ambient_intersection.into();
                        let ambient_color = self.calculate_intersection_color_with_level(&ambient_ray, &full_ambient_intersection, cur_level + 1);
                        ambient_color * (1.0 / (1.0 + full_ambient_intersection.get_distance()))
                    },
                    None => {
                        Color::zero()
                    }
                };

                acc + tmp_color / (AMBIEN_ITERATIONS_COUNT as f32)
            });
        
        ambient_color
    }

    // Расчитываем цвет с учетом отражения если есть
    fn calculate_reflection_color(&self, 
                                  reflection_level: f32,
                                  ray: &Ray, 
                                  intersection: &IntersectionFull, 
                                  diffuse_color: &Color, 
                                  light_intensivity: f32,
                                  cur_level: u32) -> Color{
        // Создаем луч отражения из данной точки
        let reflection_ray = Ray::create_reflection(*intersection.get_hit_point(), 
                                                    *intersection.get_normal(),
                                                    ray.direction,
                                                    self.bias);
        
        // Находим пересечение с объектом для луча отражения
        let reflection_intersection = self.trace_nearest_intersection(&reflection_ray);

        let reflection_color = match reflection_intersection {
            Some(reflection_intersection) => {
                let full_reflection_intersection: IntersectionFull = reflection_intersection.into();
                self.calculate_intersection_color_with_level(&reflection_ray, &full_reflection_intersection, cur_level + 1)
            },
            None => {
                Color::zero()
            }
        };

        let reverse_color = *diffuse_color * (1.0 - reflection_level);
        
        return reverse_color * light_intensivity + reflection_color;
    }

    // Расчитываем цвет с учетом отражения если есть
    fn calculate_refraction_color(&self, 
                                  info: &RefractionInfo,
                                  ray: &Ray, 
                                  intersection: &IntersectionFull, 
                                  diffuse_color: &Color, 
                                  light_intensivity: f32,
                                  cur_level: u32) -> Color {
        
        let RefractionInfo{index, transparense_level} = info;

        // Создаем луч преломления из данной точки
        let refraction_ray = Ray::create_refraction(*intersection.get_hit_point(), 
                                                    *intersection.get_normal(),
                                                    ray.direction, 
                                                    *index,
                                                    self.bias);

        // Находим обратный цвет
        let reverse_color = *diffuse_color * (1.0 - transparense_level);
        
        match refraction_ray{
            Some(refraction_ray) =>{
                // Находим пересечение с объектом для луча преломления
                let refraction_intersection = self.trace_nearest_intersection(&refraction_ray);

                // Находим цвет этого пересечения
                let refraction_color = match refraction_intersection {
                    Some(refraction_intersection) => {
                        let full_refraction_intersection: IntersectionFull = refraction_intersection.into();
                        self.calculate_intersection_color_with_level(&refraction_ray, &full_refraction_intersection, cur_level + 1)
                    },
                    None => {
                        Color::zero()
                    }
                };

                // Выдаем результат
                reverse_color * light_intensivity + refraction_color
            },
            None => {
                reverse_color * light_intensivity
            }
        }
    }

    // Для найденного пересечения расчитываем цвет пикселя
    // TODO: Use option
    fn calculate_intersection_color_with_level(&self, ray: &Ray, intersection: &IntersectionFull, cur_level: u32) -> Color{
        // TODO: Учитывать затенения в отражениях
        // TODO: Убрать рекурсию, либо по-максимуму убрать временные переменные для снижения потребления памяти

        // Если мы дошли до максимума - выходим
        if cur_level >= self.max_recursive_level {
            return Color::zero();
        }

        // https://bheisler.github.io/post/writing-raytracer-in-rust-part-2/
        
        // Получаем степень освещенности для конкретного пересечения и нормали
        let light_intensivity = self.calculate_light_intensivity(intersection);

        // Стандартный цвет объекта без учета освещения
        let diffuse_color = intersection.get_color();

        // TODO: Работает как-то неправильно
        // Но данный эффект надо делать с помощью кидания кучи лучей вокруг? Эффект Ambient Occlusion?
        let ambient_color = self.calculate_ambient_color(intersection, cur_level);

        // Финальный цвет в зависимости от модификатора материала
        let result_color: Color = match intersection.get_object().get_material().get_modificator() {
            // Отражение
            MaterialModificator::Reflection(reflection_level) => {
                let reflection_color = self.calculate_reflection_color(*reflection_level, ray, intersection, diffuse_color, light_intensivity, cur_level);
                reflection_color + ambient_color
            },
            // Преломление
            MaterialModificator::Refraction(refraction_info) => {
                let refraction_color = self.calculate_refraction_color(refraction_info, ray, intersection, diffuse_color, light_intensivity, cur_level);
                refraction_color + ambient_color
            },
            // Просто цвет
            MaterialModificator::None => {
                *diffuse_color * light_intensivity + ambient_color
            }
        };

        // Ограничим значениями 0 - 1
        result_color.clamp(0.0_f32, 1.0_f32)
    }
}
