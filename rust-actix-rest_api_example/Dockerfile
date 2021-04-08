# Аргументы сборки в докере можно указывать следующим образом, так же можно указать дефолтное значение
# ARG BASE_IMAGE=ekidd/rust-musl-builder:latest
# ARG BASE_IMAGE=rust:1.51.0

# Использовать базовый образ будем в качестве сборщика
FROM ${BASE_IMAGE} AS builder
WORKDIR /usr/src/app
# Добавляем все исходники в наш контейнер
COPY . ./
# Собираем наше приложение
RUN cargo build --release

# Сейчас нам нужно собрать уже рабочий контейнер, копируя туда все наши ресурсы из сборщика
FROM debian:latest
# Добавляем зависимости
# RUN apk --no-cache add ca-certificates
# Копируем наш собранный бинарник
COPY --from=builder \
    /usr/src/app/target/release/rust-actix-rest_api_example \
    /usr/local/bin/
# Запуск
CMD /usr/local/bin/rust-actix-rest_api_example