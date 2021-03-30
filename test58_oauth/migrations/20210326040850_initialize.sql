-- Add migration script here

CREATE TABLE IF NOT EXISTS app_users
(
    id INTEGER PRIMARY KEY,
    user_uuid VARCHAR(64) NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS facebook_users
(
    id INTEGER PRIMARY KEY,
    facebook_uid VARCHAR(64) NOT NULL UNIQUE,
    app_user_id INTEGER UNIQUE,

    FOREIGN KEY (app_user_id) REFERENCES app_users(id)
);

CREATE TABLE IF NOT EXISTS google_users
(
    id INTEGER PRIMARY KEY,
    google_uid VARCHAR(64) NOT NULL UNIQUE,
    app_user_id INTEGER UNIQUE,

    FOREIGN KEY (app_user_id) REFERENCES app_users(id)
);

CREATE INDEX IF NOT EXISTS app_search_index ON app_users(id, user_uuid);
CREATE INDEX IF NOT EXISTS facebook_search_index ON facebook_users(facebook_uid);
CREATE INDEX IF NOT EXISTS google_search_index ON google_users(google_uid);