-- Add migration script here

CREATE TABLE IF NOT EXISTS users
(
    uuid CHARACTER(36) PRIMARY KEY NOT NULL,
    facebook_uid TEXT NOT NULL
);