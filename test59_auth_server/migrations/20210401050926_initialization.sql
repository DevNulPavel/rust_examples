-- 
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users(
    id uuid DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_login VARCHAR NOT NULL UNIQUE,
    email VARCHAR NOT NULL UNIQUE,
    password_hash VARCHAR NOT NULL,
    password_salt VARCHAR NOT NULL,
    full_name VARCHAR NULL, 
    bio VARCHAR NULL, 
    user_image VARCHAR NULL,
    create_at TIMESTAMP NOT NULL DEFAULT current_timestamp,
    updated_at TIMESTAMP NOT NULL DEFAULT current_timestamp
    -- TODO: Email verified
    -- TODO: Disabled or not
);