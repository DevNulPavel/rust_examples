use std::{
    path::{
        Path
    }
};
use libc::{
    c_int,
    c_void
};

// Примеры
// https://funvision.blogspot.com/2015/12/basic-opencv-mat-tutorial-part-1.html
// http://robocraft.ru/blog/computervision/287.html

// Заголовки:
// /opt/homebrew/Cellar/opencv/4.5.1_2/include/opencv4/opencv2/imgproc/imgproc_c.h
// /opt/homebrew/Cellar/opencv/4.5.1_2/include/opencv4/opencv2/imgproc/types_c.h

// Указываем у какой библиотеки будут наши функции
#[link(name = "opencv_imgproc")]
extern "C" {
    fn cvResize(src: *const c_void, dst: *mut c_void, interpolation: c_int);
}

// Minimal implementation of single precision complex numbers
// #[repr(C)]
// #[derive(Clone, Copy)]
// struct Complex {
//     re: f32,
//     im: f32,
// }

// Since calling foreign functions is considered unsafe,
// it's common to write safe wrappers around them.
// fn cos(z: Complex) -> Complex {
//     // unsafe { ccosf(z) }
// }

pub fn resize_image(path: &Path, max_width: u32, max_height: u32) -> Vec<u8> {
    vec![]
}