create table monitoring_users(
    user_id integer PRIMARY KEY
);

create table currency_type(
    name varchar(16) PRIMARY KEY
);

create table currency_value(
    id integer PRIMARY KEY AUTOINCREMENT,
    cur_type REFERENCES currency_type(name),
    buy float,
    sell float,
    buy_change char,
    sell_change char
);

create table currency_result(
    id integer PRIMARY KEY AUTOINCREMENT,
    bank_name varchar(16),
    usd integer,
    eur integer,
    update_time varchar(32),
    FOREIGN KEY(usd) REFERENCES currency_value(id),
    FOREIGN KEY(usd) REFERENCES currency_value(id)
);

create table currency_minimum(
    id integer PRIMARY KEY,
    user_id integer,
    minimum_value integer,
    FOREIGN KEY(user_id) REFERENCES monitoring_users(user_id),
    FOREIGN KEY(minimum_value) REFERENCES currency_result(id)
);

insert into 
    currency_type (name) 
values ("EUR"),
       ("USD");