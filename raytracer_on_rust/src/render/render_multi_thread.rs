#[cfg(feature = "multi_threaded")]
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
    scene::{
        Scene,
        Intersection
    },
};
use super::{
    Ray
};

#[cfg(all(feature = "multi_threaded", not(feature = "allow_unsafe")))]
pub fn render(scene: &Scene) -> DynamicImage {
    // Обходим все строки и столбцы картинки
    // Создаем базовый цвет
    let black = Rgba::from_channels(0, 0, 0, 0);

    let mut data = Vec::new();
    data.resize((scene.width * scene.height) as usize, black);
    data
        .par_iter_mut()
        .enumerate()
        .for_each(|(index, color)| {
            let x = index as u32 % scene.width;
            let y = index as u32 / scene.width;
            let ray = Ray::create_prime(x, y, scene.width, scene.height, scene.fov);

            // Ближайшее пересечение с объектом
            let intersection: Option<Intersection<'_>> = scene.trace_nearest_intersection(&ray);

            // Если нашлось - считаем свет
            if let Some(intersection) = intersection{
                // Расчет цвета в найденном пересечении
                let result_color = scene.calculate_intersection_color(&ray, &intersection.into());

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
        });

    return image;
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
