mod traits;
mod structs;
mod render;
mod scene;

use std::{
    path::{
        Path
    }
};
use image::{
    DynamicImage
};
use crate::{
    render::{
        render
    },
    scene::{
        Scene,
        build_scene,   
    }
};

fn main(){
    let scene: Scene = build_scene();

    let img: DynamicImage = render(&scene);
    img.save(Path::new("test.png")).unwrap();
}