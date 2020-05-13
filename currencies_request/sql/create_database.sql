create table monitoring_users(
    user_id integer PRIMARY KEY
);

create table currency_minimum(
    id integer PRIMARY KEY AUTOINCREMENT,
    user_id integer,
    bank_name varchar(16),
    value integer,
    cur_type varchar(8),
    update_time varchar(32),

    FOREIGN KEY(user_id) REFERENCES monitoring_users(user_id) ON DELETE CASCADE
);
