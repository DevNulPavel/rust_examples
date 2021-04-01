use uuid::{
    Uuid
};
use chrono::{
    NaiveDateTime
};

pub struct User{
    pub uuid: Uuid,
    pub user_name: String,
    pub email: String,
    pub password_hash: String,
    pub password_salt: String,
    pub full_name: Option<String>,
    pub bio: Option<String>,
    pub user_image: Option<String>,
    pub create_at: NaiveDateTime,
    pub updated_at: NaiveDateTime
}