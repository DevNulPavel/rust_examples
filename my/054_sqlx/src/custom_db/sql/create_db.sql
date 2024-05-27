create table monitoring_users(
    user_id INTEGER PRIMARY KEY NOT NULL
);

create table currency_minimum(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    bank_name VARCHAR(16) NOT NULL,
    min_value MONEY NOT NULL,
    cur_type VARCHAR(8) NOT NULL,
    update_time INTEGER NOT NULL,
    
    -- PRIMARY KEY (id),

    UNIQUE (id),

    FOREIGN KEY (user_id) 
        REFERENCES monitoring_users(user_id) 
        ON DELETE CASCADE,
);

-- https://youtu.be/dGwkG2VyDTY?t=3275

CREATE TABLE films (
    -- Ограничение с именем firstkey - первичный ключ
    -- Ограничение по имени нужно для удаления ограничений в будущем
    code char(5) CONSTRAINT firstkey PRIMARY KEY,
    -- Имя не может быть NULL
    title varchar(5) NOT NULL,
    -- Дата по-умолчанию нулевая
    date_prod date,
    -- Тип по-умолчанию нулевой
    kind varchar(10),
    -- Длительность - интервал времени от часов до минут
    len interval hour to munite,
    -- Рейтинг - явно указываем, что он NULL
    imdb varchar(20) NULL,
    
    -- NULL никогда не равен NULL,
    -- Может быть любое количество фильмов, у которых пустой рейтинг
    -- Но добавить два фильма с одинаковым рейтингом не получится
    CONSTRAINT uniq_imdb UNIQUE(imdb)
);

CREATE TABLE distributors(
    -- Значение по-умолчанию будет serial
    id integer PRIMARY KEY DEFAULT nextval('serial'),
    -- Проверяем, что не NULL + проверяем, что строка не пустая
    name varchar(40) NOT NULL CHECK (name <> '')
);