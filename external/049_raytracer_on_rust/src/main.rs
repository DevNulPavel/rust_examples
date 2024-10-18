mod figures;
mod light;
mod material;
mod render;
mod scene;
mod structs;
mod traits;

use crate::{
    render::render,
    scene::{build_test_scene, Scene},
};
use image::DynamicImage;
use std::path::Path;

fn main() {
    let scene: Scene = build_test_scene();

    let img: DynamicImage = render(&scene);
    img.save(Path::new("test.png")).unwrap();
}
