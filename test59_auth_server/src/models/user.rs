use uuid::{
    Uuid
};
use chrono::{
    NaiveDateTime
};
use serde::{
    Serialize,
    Deserialize
};
use validator_derive::{
    Validate
};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewUser {
    #[validate(length(min = 3))]
    pub user_name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3))]
    pub password: String
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateProfile {
    pub full_name: Option<String>,
    pub bio: Option<String>,
    #[validate(url)]
    pub image: Option<String>
}