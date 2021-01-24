# Описание
Простенький REST-API сервер:
- принимает multipart/json POST запросы, выдает превьюшки максимальным размером 100x100
- для конвертации через FFI используется ImageMagic
- gracefull shutdown средствами actix-web

# Системные требования
- проверялось только на OSX 11.1 Arm64
- FFI расчитан на ImageMagic 7.0.10-58, установленный через Homebrew

# Установка зависимоcтей
```
brew install imagemagick zlib libxml2 libiconv bzip2 little-cms2
```

# Пример работы с API
Для описания API смотреть файлик [tests.rs](./src/tests.rs)

# Запуск тестов
```
cargo test
```