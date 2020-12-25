create table monitoring_users(
    user_id integer PRIMARY KEY
);

create table currency_minimum(
    id integer PRIMARY KEY AUTOINCREMENT,
    user_id integer NOT NULL,
    bank_name varchar(16) NOT NULL,
    value money NOT NULL,
    cur_type varchar(8) NOT NULL,
    update_time integer NOT NULL,
    
    FOREIGN KEY(user_id) 
        REFERENCES monitoring_users(user_id) 
        ON DELETE CASCADE
);
