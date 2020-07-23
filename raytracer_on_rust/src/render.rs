use image::{
    DynamicImage,
};
use crate::{
    traits::{
        Zero
    },
    scene::{
        Scene,
    },
    structs::{
        Point,
        Vector3
    }
};

pub struct Ray {
    // Откуда
    pub origin: Point,
    // Куда пускаем луч
    pub direction: Vector3,
}

impl Ray {
    // Пускаем луч из координат экрана для сцены
    pub fn create_prime(x: u32, y: u32, scene: &Scene) -> Ray {
        let sensor_x = ((x as f64 + 0.5) / scene.width as f64) * 2.0 - 1.0;
        let sensor_y = 1.0 - ((y as f64 + 0.5) / scene.height as f64) * 2.0;
   
        let vec3 = Vector3::new(sensor_x, sensor_y, -1.0).normalize();

        Ray {
            origin: Point::zero(),
            direction: ,
        }
    }
}

pub fn render(scene: &Scene) -> DynamicImage {
    DynamicImage::new_rgb8(scene.width, scene.height)
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
