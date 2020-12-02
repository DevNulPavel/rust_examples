use std::{
    collections::{
        HashMap
    },
    sync::{
        Mutex,
        Arc
    }
};
use crate::{
    jenkins::{
        JenkinsClient
    },
    slack::{
        SlackClient,
        ViewActionHandler
    },
    response_awaiter_holder::{
        ResponseAwaiterHolder
    }
};

pub type ViewsHandlersMap = HashMap<String, Box<dyn ViewActionHandler + Send>>;

pub struct ApplicationData{
    pub slack_client: SlackClient,
    pub jenkins_client: JenkinsClient,
    pub response_awaiter: Arc<Mutex<ResponseAwaiterHolder>>,
    active_views: Arc<Mutex<ViewsHandlersMap>>
    //active_views: Arc<RwLock<HashMap<String, Box<dyn ViewActionHandler> > >> // TODO: Async Mutex??
    //active_views: Vec< Mutex<Arc<dyn ViewActionHandler>> > // TODO: Async Mutex??
}

impl ApplicationData{
    pub fn new(slack_client: SlackClient, 
               jenkins_client: JenkinsClient, 
               response_awaiter: Arc<Mutex<ResponseAwaiterHolder>>,
               active_views: Arc<Mutex<ViewsHandlersMap>>) -> ApplicationData {
                   
        ApplicationData{
            slack_client,
            jenkins_client,
            response_awaiter,
            active_views
        }
    }

    pub fn push_view_handler(&self, view_handler: Box<dyn ViewActionHandler + Send>){
        if let Ok(mut guard) = self.active_views.lock(){
            guard.insert(view_handler.get_view().get_id().to_owned(), view_handler);
        }
    }

    pub fn pop_view_handler(&self, id: &str) -> Option<Box<dyn ViewActionHandler + Send>>{
        if let Ok(mut guard) = self.active_views.lock(){
            guard.remove(id)
        }else{
            None
        }
    }
}