use crate::{
    traits::{
        Dotable,
        Normalizable
    },
    structs::{
        Vector2,
        Vector3,
        Color
    },
    render::{
        Ray
    },
    material::{
        Material,
        MaterialsContainer,
        TexCoordDelegate
    }
};
use super::{
    traits::{
        Intersectable,
        Figure,
        Normalable,
        Texturable,
        Materiable,
        Colorable
    }
};

pub struct Sphere {
    pub center: Vector3,
    pub radius: f32,
    pub material: MaterialsContainer
}

impl Texturable for Sphere {
    fn tex_coords_at(&self, hit_point: &Vector3) -> Vector2 {
        // Получаем вектор из центра к точке соприкосновения
        let hit_vec = *hit_point - self.center;
        // TODO: Расчет
        // https://bheisler.github.io/post/writing-raytracer-in-rust-part-3/
        const PI: f32 = std::f32::consts::PI;
        let x = (1.0_f32 + (hit_vec.z.atan2(hit_vec.x)) / PI) * 0.5_f32;
        let y = (hit_vec.y / self.radius).acos() / PI;
        Vector2 {
            x,
            y,
        }
    }
}

impl Colorable for Sphere{
    fn color_at(&self, hit_point: &Vector3) -> Color {
        let tex_coord_delegate = TexCoordDelegate{
            target: self,
            hit_point: hit_point
        };
        let color = self.material.get_material().get_color_at_tex_coord(tex_coord_delegate);
        color
    }
}

// Реализация проверки пересечения с лучем
impl Intersectable for Sphere {
    /*fn intersect(&self, ray: &Ray) -> bool {
        // https://bheisler.github.io/post/writing-raytracer-in-rust-part-1/
        // https://bheisler.github.io/static/sphere-intersection-test.png

        // Создаем вектор между начальной точкой луча и центром сферы
        let ray_origin_to_center: Vector3 = self.center - ray.origin;

        // Используем векторное произведение и луч как гипотенузу для нахождения перпендикуляра, 
        // который является вектором от центра к лучу рейтрейсинга
        let adj2 = ray_origin_to_center.dot(&ray.direction);
        
        // Находим квадрат длины этого вектора? (Find the length-squared of the opposite side)
        // Это эквавалентно, но быстрее чем (l.length() * l.length()) - (adj2 * adj2)
        let d2 = ray_origin_to_center.dot(&ray_origin_to_center) - (adj2 * adj2);

        // Если квадрат длины длина меньше, чем квадрат радиуса - значит есть пересечение
        d2 < (self.radius * self.radius)
    }*/

    // Возвращает расстояние от начала луча до точки пересечения со сферой
    /*fn intersect(&self, ray: &Ray) -> Option<f32> {
        // https://bheisler.github.io/post/writing-raytracer-in-rust-part-2/
        // https://bheisler.github.io/static/intersection-distance.png

        // Создаем вектор между начальной точкой луча и центром сферы
        let ray_origin_to_center: Vector3 = self.center - ray.origin;
        
        // Используем векторное произведение и луч как гипотенузу для нахождения перпендикуляра, 
        // который является вектором от центра к лучу рейтрейсинга
        let adj = ray_origin_to_center.dot(&ray.direction);
        
        // Находим квадрат длины этого вектора? (Find the length-squared of the opposite side)
        // Это эквавалентно, но быстрее чем (l.length() * l.length()) - (adj2 * adj2)
        let d2 = ray_origin_to_center.dot(&ray_origin_to_center) - (adj * adj);
        
        // Сначала проверяем квадрат радиуса - если меньше, значит вообще нет
        let radius2 = self.radius * self.radius;
        if d2 > radius2 {
            return None;
        }

        // Вычилсляем ближайшее расстояние от начала луча до точки пересечения со сферой
        let thc = (radius2 - d2).sqrt();
        let t0 = adj - thc;
        let t1 = adj + thc;
 
        if t0 < 0.0 && t1 < 0.0 {
            return None;
        }
 
        let distance = if t0 < t1 { 
            t0 
        } else { 
            t1 
        };

        Some(distance)
    }*/

    // TODO: Исправленный вариант, нужны комменты
    // https://bheisler.github.io/post/writing-raytracer-in-rust-part-3/
    fn intersect(&self, ray: &Ray) -> Option<f32> {
        // https://bheisler.github.io/post/writing-raytracer-in-rust-part-2/
        // https://bheisler.github.io/static/intersection-distance.png

        // Создаем вектор между начальной точкой луча и центром сферы
        let ray_origin_to_center: Vector3 = self.center - ray.origin;

        // Используем векторное произведение и луч как гипотенузу для нахождения перпендикуляра, 
        // который является вектором от центра к лучу рейтрейсинга
        let adj = ray_origin_to_center.dot(&ray.direction);

        // Находим квадрат длины этого вектора? (Find the length-squared of the opposite side)
        // Это эквавалентно, но быстрее чем (l.length() * l.length()) - (adj2 * adj2)
        let d2 = ray_origin_to_center.dot(&ray_origin_to_center) - (adj * adj);

        // Сначала проверяем квадрат радиуса - если меньше, значит вообще нет
        let radius2 = self.radius * self.radius;
        if d2 > radius2 {
            return None;
        }

        // Вычилсляем ближайшее расстояние от начала луча до точки пересечения со сферой
        let thc = (radius2 - d2).sqrt();
        let t0 = adj - thc;
        let t1 = adj + thc;

        if t0 < 0.0 && t1 < 0.0 {
            None
        } else if t0 < 0.0 {
            Some(t1)
        } else if t1 < 0.0 {
            Some(t0)
        } else {
            let distance = if t0 < t1 { t0 } else { t1 };
            Some(distance)
        }
    }
}

impl Normalable for Sphere {
    fn normal_at(&self, hit_point: &Vector3) -> Vector3{
        (hit_point.clone() - self.center).normalize()
    }
}

impl Materiable for Sphere{
    fn get_material<'a>(&'a self) -> &'a dyn Material{
        self.material.get_material()
    }
}

// Пустая реализация просто чтобы пометить тип
impl Figure for Sphere{
}