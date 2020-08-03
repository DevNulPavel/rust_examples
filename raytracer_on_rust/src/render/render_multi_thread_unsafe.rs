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

#[cfg(all(feature = "multi_threaded", feature = "allow_unsafe"))]
pub fn render(scene: &Scene) -> DynamicImage {
    // Обходим все строки и столбцы картинки
    // Создаем базовый цвет
    let black = Rgba::from_channels(0, 0, 0, 0);

    let mut image = DynamicImage::new_rgb8(scene.width, scene.height);

    let image_ref_iter = rayon::iter::repeat(&image);
    let par_image_iter = image_ref_iter.into_par_iter();
    // let image_ref_iter = rayon::iter::repeat(10).p;

    unsafe{
    (0..scene.width)
        .into_par_iter()
        .zip(par_image_iter)
        .for_each(|(x, image)|{
            for y in 0..scene.height {
                let ray = Ray::create_prime(x, y, scene.width, scene.height, scene.fov);

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
        });
    }

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
