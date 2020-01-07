#![allow(dead_code)]

extern crate libc;

use std::ops::Deref;
use libc::size_t;
use libc::c_int;


#[link(name = "snappy", kind = "static")]
extern "C" {
    // В этом блоке мы описываем наши функции, лежащие в C библиотеке
    fn snappy_max_compressed_length(source_length: size_t) -> size_t;
    fn snappy_compress(input: *const u8, input_length: size_t, compressed: *mut u8, compressed_length: *mut size_t) -> c_int;
    fn snappy_uncompress(compressed: *const u8, compressed_length: size_t, uncompressed: *mut u8,uncompressed_length: *mut size_t) -> c_int;
    fn snappy_uncompressed_length(compressed: *const u8, compressed_length: size_t, result: *mut size_t) -> c_int;
    fn snappy_validate_compressed_buffer(compressed: *const u8, compressed_length: size_t) -> c_int;
}

pub fn validate_compressed_buffer(src: &[u8]) -> bool {
    // Весь код мы будем использовать в блоке unsafe
    unsafe {
        // Мы можем у массива вызвать метод as_ptr, чтобы получить сырой указатель
        // А размер массива можно скастить к size_t из libc
        snappy_validate_compressed_buffer(src.as_ptr(), src.len() as size_t) == 0
    }
}

pub fn compress(src: &[u8]) -> Option<Vec<u8>> {
    unsafe {
        // Получаем исходный размер и указатель на данные
        let srclen = src.len() as size_t;
        let psrc = src.as_ptr();

        // Получаем размер максимальный размер выходного буффера
        let mut dstlen = snappy_max_compressed_length(srclen);
        // Создаем буффер нужного размера в виде вектора с определенной емкостью
        let mut dst: Vec<u8> = Vec::with_capacity(dstlen as usize);
        // Получам указатель на этот вектор
        let pdst = dst.as_mut_ptr();

        // Выполняем сжатие
        if snappy_compress(psrc, srclen, pdst, &mut dstlen) == 0{
            // Выставляем получившийся размер буффера для ограничения
            dst.set_len(dstlen as usize);

            return Some(dst);
        }

        None
    }
}

pub fn uncompress(src: &[u8]) -> Option<Vec<u8>> {
    unsafe {
        // Получаем исходный размер и указатель на данные
        let srclen = src.len() as size_t;
        let psrc = src.as_ptr();

        // Получаем размер распакованных данных
        let mut dstlen: size_t = 0;
        snappy_uncompressed_length(psrc, srclen, &mut dstlen);

        // Создаем буффер нужного размера и получаем на него указатель
        let mut dst = Vec::with_capacity(dstlen as usize);
        let pdst = dst.as_mut_ptr();

        // Вызываем распаковку
        if snappy_uncompress(psrc, srclen, pdst, &mut dstlen) == 0 {
            // Устанавливаем размер и возвращаем результат
            dst.set_len(dstlen as usize);

            Some(dst)
        } else {
            None // SNAPPY_INVALID_INPUT
        }
    }
}

pub fn test_snappy() {
    let x = unsafe { 
        snappy_max_compressed_length(100) 
    };
    println!("максимальный размер сжатого буфера длиной 100 байт: {}", x);

    let data = [10, 20, 30, 40];

    if let Some(compressed) = compress(&data) {
        let valid = validate_compressed_buffer(&compressed);
        match uncompress(&compressed){
            Some(uncompressed) if valid => {
                assert_eq!(data, uncompressed.deref());
                println!("Snappy test success");    
            },
            _ => {
                println!("Snappy test failed");    
            }
        }
    }
}
