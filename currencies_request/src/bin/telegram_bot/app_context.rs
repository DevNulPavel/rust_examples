use reqwest::{
    Client
};
use tokio::{
    time::Interval
};
use crate::{
    currency::{
        CurrencyUsersStorrage      
    }
};

pub struct AppContext{
    pub(super) token: String,
    pub(super) proxy_check_timer: Interval, // Ограничение видимости толкьо родителем
    pub(super) send_message_timer: Interval,
    pub client: Client,
    pub users_for_push: CurrencyUsersStorrage
}

impl AppContext{

}