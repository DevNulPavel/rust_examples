use std::{
    collections::{
        HashMap
    }
};
use actix_web::{
    web::{
        Data
    }
};
use crate::{
    slack::{
        Message
    },
    jenkins::{
        JobUrl,
        JenkinsJob
    },
    application_data::{
        ApplicationData
    },
    handlers::{
        jenkins_handlers::{
            BuildFinishedParameters
        }
    }
};

type ResponseAwaiterCallback = dyn FnOnce(JenkinsJob, (String, String), Message, BuildFinishedParameters, Data<ApplicationData>) + Send;

struct ResponseAwaiter{
    job: Option<JenkinsJob>,
    root_message: Option<(String, String)>,
    message: Option<Message>,
    params: Option<BuildFinishedParameters>,
    complete: Box<ResponseAwaiterCallback>
}

impl ResponseAwaiter{
    fn new(complete: Box<ResponseAwaiterCallback>) -> ResponseAwaiter {
        ResponseAwaiter{
            job: None,
            root_message: None,
            message: None,
            params: None,
            complete
        }
    }
    fn is_complete(&self) -> bool{
        if self.job.is_some() && 
            self.message.is_some() && 
            self.params.is_some() &&
            self.root_message.is_some() {
            true
        }else{
            false
        }
    }
}

#[derive(Default)]
pub struct ResponseAwaiterHolder{
    awaiters: HashMap<JobUrl, ResponseAwaiter>,
}

// TODO: Fix box
impl ResponseAwaiterHolder {
    pub fn provide_build_complete_params(&mut self, url: &JobUrl, params: BuildFinishedParameters, app_data: Data<ApplicationData>, complete: Box<ResponseAwaiterCallback>) {
        let entry = self.awaiters
            .entry(url.to_owned());

        let awaiter = entry.or_insert_with(||{
                ResponseAwaiter::new(Box::new(complete))
            });

        awaiter.params = Some(params);

        if awaiter.is_complete() {
            if let Some(obj) = self.awaiters.remove(url){
                let ResponseAwaiter{complete, job, root_message, message, params}= obj;
                complete(
                    job.expect("Job unwrap failed"),
                    root_message.expect("Message unwrap failed"),
                    message.expect("Message unwrap failed"),
                    params.expect("Params unwrap failed"),
                    app_data
                );
            }
        }
    }

    pub fn provide_job(&mut self, url: &JobUrl, job: JenkinsJob, root_message: (String, String), message: Message, app_data: Data<ApplicationData>, complete: Box<ResponseAwaiterCallback>) {
        let entry = self.awaiters
        .entry(url.to_owned());

        let awaiter = entry.or_insert_with(||{
                ResponseAwaiter::new(Box::new(complete))
            });

        awaiter.job = Some(job);
        awaiter.message = Some(message);
        awaiter.root_message = Some(root_message);

        if awaiter.is_complete() {
            if let Some(obj) = self.awaiters.remove(url){
                let ResponseAwaiter{complete, job, root_message, message, params}= obj;
                complete(
                    job.expect("Job unwrap failed"),
                    root_message.expect("Message unwrap failed"),
                    message.expect("Message unwrap failed"),
                    params.expect("Params unwrap failed"),
                    app_data
                );
            }
        }
    }
}