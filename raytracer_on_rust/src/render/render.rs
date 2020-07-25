use rayon::{
    prelude::{
        *
    }
};
use image::{
    Pixel,
    GenericImage,
    DynamicImage,
    Rgba
};
use crate::{
    traits::{
        Zero,
        Normalizable
    },
    scene::{
        Scene,
        Intersection
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
    pub fn create_prime(x: u32, y: u32, scene: &Scene) -> Ray {
        // Расчет угла видимости
        let fov_adjustment = (scene.fov.to_radians() / 2.0).tan();

        // Расчет соотношения сторон
        assert!(scene.width > scene.height);
        let aspect_ratio = (scene.width as f32) / (scene.height as f32);

        // Приводим значения к диапазону от -1.0 к 1.0 как в OpenGL/DirectX/Metal/Vulkan
        // Направление x - слева направо, y - снизу вверх, z  - от нас
        let sensor_x = ((((x as f32 + 0.5) / scene.width as f32) * 2.0 - 1.0) * aspect_ratio * fov_adjustment) as f32;
        let sensor_y = ((1.0 - ((y as f32 + 0.5) / scene.height as f32) * 2.0) * fov_adjustment) as f32;
   
        // Создаем направление луча рендеринга в нормализованном виде
        let dir = Vector3::new(sensor_x, sensor_y, -1.0).normalize();

        // Создаем луч
        Ray {
            origin: Vector3::zero(),
            direction: dir,
        }
    }
}

// TODO: Включать или нет многопоточный вариант с помощью фич
pub fn render(scene: &Scene) -> DynamicImage {
    // Обходим все строки и столбцы картинки
    // Создаем базовый цвет
    let black = Rgba::from_channels(0, 0, 0, 0);

    /*let mut data = Vec::new();
    data.resize((scene.width * scene.height) as usize, black);
    data
        .par_iter_mut()
        .enumerate()
        .for_each(|(index, color)| {
            let x = index as u32 % scene.width;
            let y = index as u32 / scene.width;
            let ray = Ray::create_prime(x, y, scene);

            // Ближайшее пересечение с объектом
            let intersection: Option<Intersection<'_>> = scene.trace_nearest_intersection(&ray);

            // Если нашлось - считаем свет
            if let Some(intersection) = intersection{
                // Расчет цвета в найденном пересечении
                let result_color = scene.calculate_intersection_color(&intersection);

                // Установка пикселя
                *color = result_color.to_rgba();
            }
        });
    
    let mut image = DynamicImage::new_rgb8(scene.width, scene.height);
    data
        .into_iter()
        .enumerate()
        .for_each(|(index, color)|{
            let x = index as u32 % scene.width;
            let y = index as u32 / scene.width;
            image.put_pixel(x, y, color);
        });*/

    // Создание изображения
    let mut image = DynamicImage::new_rgb8(scene.width, scene.height);

    for x in 0..scene.width {
        for y in 0..scene.height {
            let ray = Ray::create_prime(x, y, scene);

            // Ближайшее пересечение с объектом
            let intersection: Option<Intersection<'_>> = scene.trace_nearest_intersection(&ray);

            // Если нашлось - считаем свет
            if let Some(intersection) = intersection{
                // Расчет цвета в найденном пересечении
                let result_color = scene.calculate_intersection_color(&intersection);

                // Установка пикселя
                image.put_pixel(x, y, result_color.to_rgba());
            }else{
                // Установка пикселя
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
