#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{
    path::{
        Path
    },
    ffi::{
        CString,
        CStr
    }
    // os::{
    //     raw::{
    //         c_uint
    //     }
    // }
};
use libc::{
    c_uint,
    c_int,
    c_void,
    c_char
};

// https://github.com/nlfiedler/magick-rust/

// https://imagemagick.org/api/resize.php
// https://imagemagick.org/script/develop.php
// https://imagemagick.org/script/magick-wand.php
// https://imagemagick.org/script/magick-core.php

// /opt/homebrew/Cellar/imagemagick/7.0.10-58/include/ImageMagick-7/MagickCore/MagickCore.h
// /opt/homebrew/Cellar/imagemagick/7.0.10-58/include/ImageMagick-7/MagickWand/MagickWand.h
// /opt/homebrew/Cellar/imagemagick/7.0.10-58/include/ImageMagick-7/MagickWand/magick-image.h
// /opt/homebrew/Cellar/imagemagick/7.0.10-58/include/ImageMagick-7/MagickCore/magick-type.h

type MagickBooleanType = ::std::os::raw::c_uint;
const MagickFalse: MagickBooleanType = 0;
const MagickTrue: MagickBooleanType = 1;

// Указываем у какой библиотеки будут наши функции
#[link(name = "MagickWand-7.Q16HDRI")]
extern "C" {
    fn MagickWandGenesis();
    fn NewMagickWand() -> *const c_void;
    fn DestroyMagickWand(wand: *const c_void);
    fn MagickReadImage(wand: *const c_void, path: *const c_char) -> MagickBooleanType;
    fn MagickAdaptiveResizeImage(wand: *const c_void, width: usize, height: usize) -> MagickBooleanType;
}

// /// Read the image data from the named file.
// pub fn read_image(&self, path: &str) -> Result<(), &'static str> {
//     let c_name = CString::new(path).unwrap();
//     let result = unsafe { bindings::MagickReadImage(self.wand, c_name.as_ptr()) };
//     match result {
//         bindings::MagickBooleanType_MagickTrue => Ok(()),
//         _ => Err("failed to read image"),
//     }
// }

// /// Adaptively resize the currently selected image.
// pub fn adaptive_resize_image(&self, width: usize, height: usize) -> Result<(), &'static str> {
//     match unsafe { bindings::MagickAdaptiveResizeImage(self.wand, width, height) } {
//         bindings::MagickBooleanType_MagickTrue => Ok(()),
//         _ => Err("failed to adaptive-resize image"),
//     }
// }

/// Resize the image to fit within the given dimensions, maintaining
// /// the current aspect ratio.
// pub fn fit(&self, width: size_t, height: size_t) {
//     let mut width_ratio = width as f64;
//     width_ratio /= self.get_image_width() as f64;
//     let mut height_ratio = height as f64;
//     height_ratio /= self.get_image_height() as f64;
//     let (new_width, new_height) = if width_ratio < height_ratio {
//         (
//             width,
//             (self.get_image_height() as f64 * width_ratio) as size_t,
//         )
//     } else {
//         (
//             (self.get_image_width() as f64 * height_ratio) as size_t,
//             height,
//         )
//     };
//     unsafe {
//         bindings::MagickResetIterator(self.wand);
//         while bindings::MagickNextImage(self.wand) != bindings::MagickBooleanType_MagickFalse {
//             bindings::MagickResizeImage(
//                 self.wand,
//                 new_width,
//                 new_height,
//                 bindings::FilterType_LanczosFilter,
//             );
//         }
//     }
// }

//     /// Retrieve the width of the image.
// pub fn get_image_width(&self) -> usize {
//     unsafe { bindings::MagickGetImageWidth(self.wand) as usize }
// }

/// Retrieve the height of the image.
// pub fn get_image_height(&self) -> usize {
//     unsafe { bindings::MagickGetImageHeight(self.wand) as usize }
// }

pub fn resize_image(path: &Path, max_width: usize, max_height: usize) -> Result<Vec<u8>, ()> {

    if !path.exists() {
        return Err(());
    }
    let path_str = path.to_str().ok_or(())?;
    
    // Строка с нулем в конце
    let c_path = CString::new(path_str).unwrap();

    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(||{
        unsafe{
            MagickWandGenesis();
        }
    });

    let mut width_ratio = width as f64;
    width_ratio /= self.get_image_width() as f64;
    let mut height_ratio = height as f64;
    height_ratio /= self.get_image_height() as f64;
    let (new_width, new_height) = if width_ratio < height_ratio {
        (width, (self.get_image_height() as f64 * width_ratio) as size_t)
    } else {
        ((self.get_image_width() as f64 * height_ratio) as size_t, height)
    };

    let wand = unsafe { NewMagickWand() };

    scopeguard::defer!( unsafe { DestroyMagickWand(wand) } );

    let result = unsafe { MagickReadImage(wand, c_path.as_ptr()) };
    if result == MagickFalse{
        return Err(());
    }

    let result = unsafe { MagickAdaptiveResizeImage(wand, max_width, max_height) };
    if result == MagickFalse{
        return Err(());
    }

    Ok(vec![])
}