#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{
    path::{
        Path
    },
    ffi::{
        OsString,
        OsStr,
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
    c_char,
    c_uchar,
    size_t
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
// /opt/homebrew/Cellar/imagemagick/7.0.10-58/include/ImageMagick-7/MagickCore/resample.h

type MagickBooleanType = ::std::os::raw::c_uint;
const MagickFalse: MagickBooleanType = 0;
const MagickTrue: MagickBooleanType = 1;

type MagicFilterType = ::std::os::raw::c_uint;
const UndefinedFilter: MagicFilterType = 0;
const PointFilter: MagicFilterType = 1;
const BoxFilter: MagicFilterType = 2;
const TriangleFilter: MagicFilterType = 3;
const HermiteFilter: MagicFilterType = 4;
const HannFilter: MagicFilterType = 5;
const HammingFilter: MagicFilterType = 6;
const BlackmanFilter: MagicFilterType = 7;
const GaussianFilter: MagicFilterType = 8;
const QuadraticFilter: MagicFilterType = 9;
const CubicFilter: MagicFilterType = 10;
const CatromFilter: MagicFilterType = 11;
const MitchellFilter: MagicFilterType = 12;
const JincFilter: MagicFilterType = 13;
const SincFilter: MagicFilterType = 14;
const SincFastFilter: MagicFilterType = 15;
const KaiserFilter: MagicFilterType = 16;
const WelchFilter: MagicFilterType = 17;
const ParzenFilter: MagicFilterType = 18;
const BohmanFilter: MagicFilterType = 19;
const BartlettFilter: MagicFilterType = 20;
const LagrangeFilter: MagicFilterType = 21;
const LanczosFilter: MagicFilterType = 22;
const LanczosSharpFilter: MagicFilterType = 23;
const Lanczos2Filter: MagicFilterType = 24;
const Lanczos2SharpFilter: MagicFilterType = 25;
const RobidouxFilter: MagicFilterType = 26;
const RobidouxSharpFilter: MagicFilterType = 27;
const CosineFilter: MagicFilterType = 28;
const SplineFilter: MagicFilterType = 29;
const LanczosRadiusFilter: MagicFilterType = 30;
const CubicSplineFilter: MagicFilterType = 31;
const SentinelFilter: MagicFilterType = 32;

// Указываем у какой библиотеки будут наши функции
#[link(name = "MagickWand-7.Q16HDRI")]
extern "C" {
    fn MagickWandGenesis();
    fn NewMagickWand() -> *const c_void;
    fn DestroyMagickWand(wand: *const c_void);
    fn MagickReadImage(wand: *const c_void, path: *const c_char) -> MagickBooleanType;
    fn MagickReadImageBlob(wand: *const c_void, data: *const c_void, size: size_t) -> MagickBooleanType;
    fn MagickGetImageWidth(wand: *const c_void) -> size_t;
    fn MagickGetImageHeight(wand: *const c_void) -> size_t;
    fn MagickAdaptiveResizeImage(wand: *const c_void, width: size_t, height: size_t) -> MagickBooleanType;
    fn MagickResizeImage(wand: *const c_void, width: size_t, height: size_t, filter: MagicFilterType) -> MagickBooleanType;
    fn MagickGetImageBlob(wand: *const c_void, length: *mut size_t) -> *const c_uchar;
    fn MagickRelinquishMemory(wand: *const c_void) -> *const c_void;
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
//                 bindings::MagicFilterType_LanczosFilter,
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

// pub fn write_image_blob(&self, format: &str) -> Result<Vec<u8>, &'static str> {
//     let c_format = CString::new(format).unwrap();
//     let mut length: size_t = 0;
//     let blob = unsafe {
//                 bindings::MagickResetIterator(self.wand);
//                 bindings::MagickSetImageFormat(self.wand, c_format.as_ptr());
//                 bindings::MagickGetImageBlob(self.wand, &mut length)
//             };
//     let mut bytes = Vec::with_capacity(length as usize);
//     unsafe {
//                 bytes.set_len(length as usize);
//                 ptr::copy_nonoverlapping(blob, bytes.as_mut_ptr(), length as usize);
//                 bindings::MagickRelinquishMemory(blob as *mut c_void);
//             };
//     Ok(bytes)
//         }

// TODO: Ошибка

pub fn fit_image(data: Vec<u8>, max_width: usize, max_height: usize) -> Result<Vec<u8>, ()> {

    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(||{
        unsafe{
            MagickWandGenesis();
        }
    });

    let wand = unsafe { NewMagickWand() };
    scopeguard::defer!( unsafe { DestroyMagickWand(wand) } );

    
    let result = unsafe { MagickReadImageBlob(wand, data.as_ptr() as *const c_void, data.len()) };
    if result == MagickFalse{
        return Err(());
    }
    return Ok(vec![]);

    let (new_width, new_height) = {
        let source_width = unsafe{ MagickGetImageWidth(wand) };
        let source_height = unsafe{ MagickGetImageWidth(wand) };   

        let mut width_ratio = max_width as f64;
        width_ratio /= source_width as f64;
        
        let mut height_ratio = max_height as f64;
        height_ratio /= source_height as f64;

        // Если у нас и так все влезает - не конвертируем ничего
        if (width_ratio < 1.0) && (height_ratio < 1.0) {
            return Ok(data);
        }

        if width_ratio < height_ratio {
            (source_width, (source_height as f64 * width_ratio) as size_t)
        } else {
            ((source_width as f64 * height_ratio) as size_t, source_height)
        }
    };

    // bindings::MagickResetIterator(self.wand);
    // while bindings::MagickNextImage(self.wand) != bindings::MagickBooleanType_MagickFalse {

    let result = unsafe { MagickResizeImage(wand, new_width, new_height, LanczosFilter) };
    if result == MagickFalse{
        return Err(());
    }

    let mut result_data_size: size_t = 0;
    let blob = unsafe{ MagickGetImageBlob(wand, &mut result_data_size) };
    if blob.is_null() {
        return Err(());
    }

    let mut buffer: Vec<u8> = Vec::with_capacity(result_data_size);
    unsafe {
        buffer.set_len(result_data_size);
        // Неперекрывающееся копирование
        std::ptr::copy_nonoverlapping(blob, buffer.as_mut_ptr(), result_data_size);
        MagickRelinquishMemory(blob as *const c_void); // TODO: Возвращемое значение?
    }

    Ok(buffer)
}

#[cfg(test)]
mod tests{
    use super::{
        *
    };
    use std::{
        fs::{
            read
        }
    };

    #[test]
    fn test_imagemagic_ffi_png(){
        let data = read("test_images/airplane.png").expect("File read failed");
        let result = fit_image(data, 100, 100).expect("Fit failed");
    }
}