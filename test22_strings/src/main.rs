#![allow(unused_imports, dead_code, unused_variables)]

use std::slice;
use std::str;

fn test_strings(){
    {
        // &str - это толстый указатель, представляющий из себя сырой указатель + длину
        let story: &str = "Once upon a time...";

        // Можем получить указатель на сырые данные, но эта строка не заканчивается нулем
        let ptr: *const u8 = story.as_ptr();
        // Можем получить длину
        let len = story.len();

        // В блоке unsafe мы можем пересоздать нашу строку из сырого указателя и длины
        let s = unsafe {
            // Проверяем указатель
            assert_eq!(ptr.is_null(), false);
            // Проверяем длину
            assert_eq!(19, len);

            // Сначала заново создаем слайс &[u8]
            let slice = slice::from_raw_parts(ptr, len);

            // Затем слайс мы превращаем снова в строку
            str::from_utf8(slice)
        };

        assert_eq!(s, Ok(story));
    }

    {
        // Можно получить длину
        let len = "foo".len();
        assert_eq!(3, len);

        let len = "ƒoo".len(); // fancy f! - специальный символ f
        assert_eq!(4, len);
    }

    {
        // Можно конвертировать в байты
        let bytes = "bors".as_bytes();
        assert_eq!(b"bors", bytes);
    }

    {
        // Можно с помощью метода get как получать конкретный элемент, так и подслайс
        let v = String::from("🗻∈🌏");

        assert_eq!(Some("🗻"), v.get(0..4));

        // Индексы находятся вне границ UTF-8
        assert!(v.get(1..=40).is_none()); // 1..=40 - это значит, что мы указываем диапазон включая конкретный элемент
        assert!(v.get(..8).is_none());

        // out of bounds
        assert!(v.get(..42).is_none());
    }
}

fn main() {
    test_strings();
}

#[cfg(test)]
mod test {
    #[test]
    fn name() {
        crate::test_strings();
    }
}