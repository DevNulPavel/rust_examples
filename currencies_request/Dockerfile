# FROM alpine:3.7

FROM busybox
ENV TELEGRAM_TOKEN=""
# Копия файлика
COPY ./target/arm-unknown-linux-gnueabi/release/telegram_bot /telegram_bot
COPY ./sql/create_database.sql /sql/create_database.sql
# Порт наружу
# EXPOSE 5000
# Запуск
CMD ["/telegram_bot"]
# RUN apk add --no-cache mysql-client
# ENTRYPOINT ["mysql"]
