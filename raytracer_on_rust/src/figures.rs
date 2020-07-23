use crate::{
    traits::{
        Intersectable,
        Dot
    },
    structs::{
        Vector3,
        Color
    },
    render::{
        Ray
    }
};

pub struct Sphere {
    pub center: Vector3,
    pub radius: f32,
    pub color: Color,
}

// Реализация проверки пересечения с лучем
impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> bool {
        // https://bheisler.github.io/post/writing-raytracer-in-rust-part-1/
        // https://bheisler.github.io/static/sphere-intersection-test.png

        // Создаем вектор между начальной точкой луча и центром сфера
        let ray_origin_to_center: Vector3 = self.center - ray.origin;

        // Используем векторное произведение и луч как гипотенузу для нахождения перпендикуляра, 
        // который является вектором от центра к лучу рейтрейсинга
        let adj2 = ray_origin_to_center.dot(&ray.direction);
        
        // Находим квадрат длины этого вектора? (Find the length-squared of the opposite side)
        //Это эквавалентно, но быстрее чем (l.length() * l.length()) - (adj2 * adj2)
        let d2 = ray_origin_to_center.dot(&ray_origin_to_center) - (adj2 * adj2);

        // Если квадрат длины длина меньше, чем квадрат радиуса - значит есть пересечение
        d2 < (self.radius * self.radius)
    }
}