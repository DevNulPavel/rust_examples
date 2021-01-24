#![allow(non_upper_case_globals)]

use std::{
    fmt::{
        self,
        Formatter,
        Display
    },
    error::{
        self
    },
    ffi::{
        CStr
    }
};
use libc::{
    c_uint,
    c_void,
    c_char,
    c_uchar,
    size_t
};
use log::{
    error,
    debug,
    warn,
    info
};

// Поддерживаемые форматы imagemagic: identify -list format -v

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
// /opt/homebrew/Cellar/imagemagick/7.0.10-58/include/ImageMagick-7/MagickCore/exception.h

////////////////////////////////////////////////////////////////////////////////////////////////

type MagickBooleanType = c_uint;
const MAGIC_FALSE: MagickBooleanType = 0;
// const MagickTrue: MagickBooleanType = 1;

////////////////////////////////////////////////////////////////////////////////////////////////

type MagicFilterType = c_uint;
const LANCHOZ_FILTER: MagicFilterType = 22;

////////////////////////////////////////////////////////////////////////////////////////////////

// Указываем у какой библиотеки будут наши функции
#[link(name = "MagickWand-7.Q16HDRI")]
extern "C" {
    fn MagickWandGenesis();
    //fn MagickWandTerminus();
    //fn IsMagickWandInstantiated() -> MagickBooleanType;
    //fn MagickGetExceptionType(wand: *const c_void) -> c_uint;
    fn MagickGetException(wand: *const c_void, exception_type: *mut c_uint) -> *const c_char;
    fn NewMagickWand() -> *const c_void;
    fn DestroyMagickWand(wand: *const c_void);
    //fn MagickReadImage(wand: *const c_void, path: *const c_char) -> MagickBooleanType;
    fn MagickReadImageBlob(wand: *const c_void, data: *const c_void, size: size_t) -> MagickBooleanType;
    fn MagickGetImageWidth(wand: *const c_void) -> size_t;
    fn MagickGetImageHeight(wand: *const c_void) -> size_t;
    //fn MagickAdaptiveResizeImage(wand: *const c_void, width: size_t, height: size_t) -> MagickBooleanType;
    fn MagickResizeImage(wand: *const c_void, width: size_t, height: size_t, filter: MagicFilterType) -> MagickBooleanType;
    fn MagickGetImageBlob(wand: *const c_void, length: *mut size_t) -> *const c_uchar;
    fn MagickRelinquishMemory(wand: *const c_void) -> *const c_void;
}

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ImageMagicErrorData{
    info: String, 
    code: i32
}

#[derive(Debug)]
pub enum ImageMagicError{
    ExceptionParseFailed(std::str::Utf8Error),
    ReadFailed(ImageMagicErrorData),
    ResizeFailed,
    GetResultFailed
}
impl Display for ImageMagicError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ImageMagicError: {:#?}", self)
    }
}
impl error::Error for ImageMagicError {
}
impl From<std::str::Utf8Error> for ImageMagicError{
    fn from(err: std::str::Utf8Error) -> ImageMagicError {
        ImageMagicError::ExceptionParseFailed(err)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////

pub fn imagemagic_fit_image(data: Vec<u8>, max_width: usize, max_height: usize) -> Result<Vec<u8>, ImageMagicError> {

    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(||{
        unsafe{
            MagickWandGenesis();
        }
    });

    // Инициализация обработки
    let wand = unsafe { NewMagickWand() };
    scopeguard::defer!( unsafe { DestroyMagickWand(wand) } );
    
    // Читаем наши данные
    let result = unsafe { MagickReadImageBlob(wand, data.as_ptr() as *const c_void, data.len()) };
    if result == MAGIC_FALSE{
        let mut exc_type: c_uint = 0;
        let text = unsafe { 
            let exc_text = MagickGetException(wand, &mut exc_type);
            CStr::from_ptr(exc_text).to_str()?.to_owned()
        };
        let info = ImageMagicErrorData{
            info: text,
            code: exc_type as i32
        };
        return Err(ImageMagicError::ReadFailed(info));
    }

    // Вычисляем размер новый, либо сразу возвращаем, если уже меньше
    let (new_width, new_height) = {
        let source_width = unsafe{ MagickGetImageWidth(wand) };
        let source_height = unsafe{ MagickGetImageHeight(wand) };   

        let mut width_ratio = max_width as f64;
        width_ratio /= source_width as f64;
        
        let mut height_ratio = max_height as f64;
        height_ratio /= source_height as f64;

        // Если у нас и так все влезает - не конвертируем ничего
        /*if (width_ratio >= 1.0) && (height_ratio <= 1.0) {
            return Ok(data);
        }*/

        if width_ratio < height_ratio {
            (((source_width as f64) * width_ratio) as size_t, (source_height as f64 * width_ratio) as size_t)
        } else {
            ((source_width as f64 * height_ratio) as size_t, ((source_height as f64)  * height_ratio) as size_t)
        }
    };

    // println!("New size: {} x {}", new_width, new_height);

    // bindings::MagickResetIterator(self.wand);
    // while bindings::MagickNextImage(self.wand) != bindings::MagickBooleanType_MagickFalse {

    // Смена размера
    let result = unsafe { MagickResizeImage(wand, new_width, new_height, LANCHOZ_FILTER) };
    if result == MAGIC_FALSE{
        return Err(ImageMagicError::ResizeFailed);
    }

    // Результат
    let mut result_data_size: size_t = 0;
    let blob = unsafe{ MagickGetImageBlob(wand, &mut result_data_size) };
    if blob.is_null() {
        return Err(ImageMagicError::GetResultFailed);
    }

    // Буффер для результата
    let mut buffer: Vec<u8> = Vec::with_capacity(result_data_size);
    unsafe {
        buffer.set_len(result_data_size);
        // Неперекрывающееся копирование
        std::ptr::copy_nonoverlapping(blob, buffer.as_mut_ptr(), result_data_size);
        MagickRelinquishMemory(blob as *const c_void); // TODO: Возвращемое значение?
    }

    Ok(buffer)
}

///////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests{
    use super::{
        *
    };
    use std::{
        fs::{
            read,
            write,
            create_dir,
        },
        path::{
            Path,
        }
    };

    fn make_result_dir<P: AsRef<Path>> (path: P){
        if !path.as_ref().exists() {
            create_dir(path).expect("Result directory create failed");
        }
    }

    fn is_vectors_eq<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
        let matching = a
            .iter()
            .zip(b.iter())
            .filter(|&(a, b)| a == b)
            .count();
        (matching == a.len()) && (matching == b.len())
    }

    #[test]
    fn test_imagemagic_ffi_png(){
        make_result_dir("test_results");
        let data = read("test_images/airplane.png").expect("File read failed");
        let result = imagemagic_fit_image(data, 100, 100).expect("Fit failed");
        assert!(result.len() > 0);
        
        // TODO: Меняется мета у файлика каждый раз? поэтому визуальная проверка
        write("test_results/small_airplane.png", result).expect("Write failed");
        
        // let reference_result = read("test_results/small_airplane.png").expect("File read failed");
        // assert!(is_vectors_eq(&result, &reference_result));
    }

    #[test]
    fn test_imagemagic_ffi_jpg_1(){
        make_result_dir("test_results");
        let data = read("test_images/building.jpg").expect("File read failed");
        let result = imagemagic_fit_image(data, 100, 100).expect("Fit failed");
        assert!(result.len() > 0);

        let reference_result = read("test_results/small_building.jpg").expect("File read failed");
        assert!(is_vectors_eq(&result, &reference_result));
    }

    #[test]
    fn test_imagemagic_ffi_jpg_2(){
        make_result_dir("test_results");
        let data = read("test_images/logo.jpg").expect("File read failed");
        let result = imagemagic_fit_image(data, 100, 100).expect("Fit failed");
        assert!(result.len() > 0);

        // write("test_results/small_logo.jpg", result).expect("Write failed");

        let reference_result = read("test_results/small_logo.jpg").expect("File read failed");
        assert!(is_vectors_eq(&result, &reference_result));
    }

    #[test]
    fn test_imagemagic_ffi_bmp(){
        make_result_dir("test_results");
        let data = read("test_images/barbara.bmp").expect("File read failed");
        let result = imagemagic_fit_image(data, 100, 100).expect("Fit failed");
        assert!(result.len() > 0);

        let reference_result = read("test_results/small_barbara.bmp").expect("File read failed");
        assert!(is_vectors_eq(&result, &reference_result));
    }

    #[test]
    fn test_imagemagic_ffi_tif(){
        make_result_dir("test_results");
        let data = read("test_images/cameraman.tif").expect("File read failed");
        let result = imagemagic_fit_image(data, 100, 100).expect("Fit failed");
        assert!(result.len() > 0);

        let reference_result = read("test_results/small_cameraman.tif").expect("File read failed");
        assert!(is_vectors_eq(&result, &reference_result));
    }
}