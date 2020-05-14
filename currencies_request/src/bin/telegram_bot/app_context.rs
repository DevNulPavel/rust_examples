use reqwest::{
    Client
};
use tokio::{
    time::Interval
};
use sqlx::{
    sqlite::{
        SqliteConnection
    }
};
use crate::{
    currency::{
        CurrencyUsersStorrage      
    }
};

pub struct AppContext{
    pub(super) token: String,
    pub(super) proxy_check_timer: Interval, // Ограничение видимости толкьо родителем
    pub(super) check_updates_timer: Interval,
    pub client: Client,
    pub db_conn: SqliteConnection,
    pub users_for_push: CurrencyUsersStorrage
}