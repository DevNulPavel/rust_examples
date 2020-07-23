use image::{
    Pixel,
    GenericImage,
    DynamicImage,
    Rgba
};
use crate::{
    traits::{
        Zero,
        Normalize,
    },
    scene::{
        Scene,
        Intersection
    },
    structs::{
        Vector3,
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
    pub fn create_prime(x: u32, y: u32, scene: &Scene) -> Ray {
        // Расчет угла видимости
        let fov_adjustment = (scene.fov.to_radians() / 2.0).tan();

        // Расчет соотношения сторон
        assert!(scene.width > scene.height);
        let aspect_ratio = (scene.width as f64) / (scene.height as f64);

        // Приводим значения к диапазону от -1.0 к 1.0 как в OpenGL/DirectX/Metal/Vulkan
        // Направление x - слева направо, y - снизу вверх, z  - от нас
        let sensor_x = ((((x as f64 + 0.5) / scene.width as f64) * 2.0 - 1.0) * aspect_ratio * fov_adjustment) as f32;
        let sensor_y = ((1.0 - ((y as f64 + 0.5) / scene.height as f64) * 2.0) * fov_adjustment) as f32;
   
        // Создаем направление луча рендеринга в нормализованном виде
        let dir = Vector3::new(sensor_x, sensor_y, -1.0).normalize();

        // Создаем луч
        Ray {
            origin: Vector3::zero(),
            direction: dir,
        }
    }
}

pub fn render(scene: &Scene) -> DynamicImage {
    // Создание изображения
    let mut image = DynamicImage::new_rgb8(scene.width, scene.height);
    
    // Создаем базовый цвет
    let black = Rgba::from_channels(0, 0, 0, 0);
    
    // Обходим все строки и столбцы картинки
    // TODO: Rayon и многопоточность
    for x in 0..scene.width {
        for y in 0..scene.height {
            let ray = Ray::create_prime(x, y, scene);

            let intersection: Option<Intersection<'_>> = scene.trace(&ray);

            if let Some(intersection) = intersection{
                image.put_pixel(x, y, intersection.object.get_pixel_color().to_rgba());
            }else{
                image.put_pixel(x, y, black);
            }
        }
    }
    image
}

/*#[cfg(tests)]
mod test{
    #[test]
    fn test_can_render_scene() {
        let scene = build_scene();

        let img: DynamicImage = render(&scene);
        //assert_eq!(scene.width, img.width());
        //assert_eq!(scene.height, img.height());
    }
}*/
