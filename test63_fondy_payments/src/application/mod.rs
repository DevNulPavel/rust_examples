use handlebars::{
    Handlebars
};
use crate::{
    database::{
        Database
    }
};



#[derive(Debug)]
pub struct Application{
    pub db: Database,
    pub templates: Handlebars<'static>
}