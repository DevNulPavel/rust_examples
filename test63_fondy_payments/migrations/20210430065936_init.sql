-- Схема базы данных
-- https://dbdiagram.io/d/608babe3b29a09603d12cda0

TODO:!!!!
ENUM

CREATE TABLE products (
    product_id SERIAL PRIMARY KEY,
    product_name VARCHAR(64) UNIQUE NOT NULL,
    price price NOT NULL
);

CREATE TABLE purchases (
    purchase_id SERIAL PRIMARY KEY,
    product_id INTEGER NOT NULL,
    purchase_time TIMESTAMP NOT NULL 
        DEFAULT(now()),
    purchase_status VARCHAR(30) NOT NULL 
        DEFAULT('not_consumed'),

    CONSTRAINT product_id_ref 
        FOREIGN KEY (product_id) 
        REFERENCES products(product_id),

    CONSTRAINT purchase_status_check 
        CHECK (purchase_status IN ('not_consumed', 'consumed'))
);

-- CREATE INDEX products_idx ON products (product_id);
-- CREATE INDEX purchases_idx ON purchases (purchase_id);
CREATE INDEX purchases_products_idx ON purchases (product_id);
