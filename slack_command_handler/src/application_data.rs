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
        View
    }
};

//#[derive(Clone)]
pub struct ApplicationData{
    pub slack_client: SlackClient,
    pub jenkins_client: JenkinsClient,
    pub active_views: Arc<Mutex<HashMap<String, View>>> // TODO: Async Mutex??
}

impl ApplicationData{
    pub fn save_view(&self, view: View){
        if let Ok(guard) = self.active_views.lock(){
            guard.insert(view.get_id().to_owned(), view);
        }
    }
}