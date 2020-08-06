use crate::{
    traits::{
        Zero,
        Normalizable,
        Dotable,
    },
    structs::{
        Vector3
    }
};

pub struct Ray {
    // Откуда
    pub origin: Vector3,
    // Куда пускаем луч
    pub direction: Vector3,
}

impl Ray {
    // Пускаем луч из координат экрана для сцены
    pub fn create_prime(x: u32, y: u32, width: u32, height: u32, fov: f32) -> Ray {
        // Расчет угла видимости
        let fov_adjustment = (fov.to_radians() / 2.0).tan();

        // Расчет соотношения сторон
        assert!(width > height);
        let aspect_ratio = (width as f32) / (height as f32);

        // Приводим значения к диапазону от -1.0 к 1.0 как в OpenGL/DirectX/Metal/Vulkan
        // Направление x - слева направо, y - снизу вверх, z  - от нас
        let sensor_x = ((((x as f32 + 0.5) / width as f32) * 2.0 - 1.0) * aspect_ratio * fov_adjustment) as f32;
        let sensor_y = ((1.0 - ((y as f32 + 0.5) / height as f32) * 2.0) * fov_adjustment) as f32;
   
        // Создаем направление луча рендеринга в нормализованном виде
        let dir = Vector3::new(sensor_x, sensor_y, -1.0).normalize();

        // Создаем луч
        Ray {
            origin: Vector3::zero(),
            direction: dir,
        }
    }

    pub fn create_reflection(origin: Vector3, 
                             normal: Vector3, 
                             direction_to_origin: Vector3, 
                             bias: f32) -> Ray {
        Ray {
            origin: origin + (normal * bias),
            direction: direction_to_origin - (normal * 2.0 * direction_to_origin.dot(&normal)),
        }
    }

    // TODO: TEST + Remove mut
    pub fn create_refraction(origin: Vector3,
                             normal: Vector3,
                             direction_to_origin: Vector3,
                             index: f32,
                             bias: f32) -> Option<Ray> {
        let mut ref_n = normal;
        let mut eta_t = index;
        let mut eta_i = 1.0;
        let mut i_dot_n = direction_to_origin.dot(&normal);
        if i_dot_n < 0.0 {
            //Outside the surface
            i_dot_n = -i_dot_n;
        } else {
            //Inside the surface; invert the normal and swap the indices of refraction
            ref_n = -normal;
            eta_t = 1.0;
            eta_i = index;
        }

        let eta = eta_i / eta_t;
        let k = 1.0 - (eta * eta) * (1.0 - i_dot_n * i_dot_n);
        if k < 0.0 {
            None
        } else {
            Some(Ray {
                origin: origin + (ref_n * -bias),
                direction: (direction_to_origin + ref_n * i_dot_n) * eta - ref_n * k.sqrt(),
            })
        }
    }

    /*pub fn create_transmission_TEST(normal: Vector3,
                                    incident: Vector3,
                                    intersection: Vector3,
                                    bias: f32,
                                    index: f32)
                            -> Option<Ray> {
        let mut ref_n = normal;
        let mut eta_t = index;
        let mut eta_i = 1.0;
        let mut i_dot_n = incident.dot(&normal);
        if i_dot_n < 0.0 {
            //Outside the surface
            i_dot_n = -i_dot_n;
        } else {
            //Inside the surface; invert the normal and swap the indices of refraction
            ref_n = -normal;
            eta_i = eta_t;
            eta_t = 1.0;
        }

        let eta = eta_i / eta_t;
        let k = 1.0 - (eta * eta) * (1.0 - i_dot_n * i_dot_n);
        if k < 0.0 {
            None
        } else {
            Some(Ray {
                origin: intersection + (ref_n * -bias),
                direction: (incident + ref_n * i_dot_n) * eta - ref_n * k.sqrt(),
            })
        }
    }*/
}

#[cfg(test)]
mod test{
    use super::*;

    // TODO: Tests

    #[test]
    fn test_refraction_1(){
        let origin = Vector3::new(0.0, 0.0, 0.0);
        let normal = Vector3::new(0.0, 1.0, 0.0).normalize();
        let direction = Vector3::new(1.0, 1.0, 0.0).normalize();
        let index = 0.001_f32;
        let bias = 0.001_f32;
        let ray = Ray::create_refraction(origin, normal, direction, index, bias).unwrap();
        assert_eq!(ray.origin, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(ray.direction, Vector3::new(0.0, 0.0, 0.0));
    }

    /*#[test]
    fn test_refraction_2(){
        let origin = Vector3::new(0.0, 0.0, 0.0);
        let normal = Vector3::new(0.0, 1.0, 0.0).normalize();
        let direction = Vector3::new(1.0, 1.0, 0.0).normalize();
        // assert_eq!(direction, Vector3::new(0.0, 0.0, 0.0));
        let index = 0.999_f32;
        let bias = 0.01_f32;
        let ray = Ray::create_transmission_TEST(normal, direction, origin, bias, index);
        assert!(ray.is_some());
        let ray = ray.unwrap();
        //assert_eq!(ray.origin, Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(ray.direction, Vector3::new(0.0, 0.0, 0.0));
    }*/
}