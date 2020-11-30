use actix_web::{ 
    web::{
        Data
    }
};
use crate::{
    application_data::{
        ApplicationData
    }
};


// #[derive(Debug)]
pub struct BaseSession{
    pub app_data: Data<ApplicationData>, 
    pub user_id: String,
    pub user_name: String,
    pub trigger_id: String
}

impl BaseSession {
    pub fn new(app_data: Data<ApplicationData>,
               user_id: String,
               user_name: String,
               trigger_id: String) -> BaseSession{
    BaseSession{
            app_data, 
            user_id,
            user_name,
            trigger_id, 
        }
    }
}