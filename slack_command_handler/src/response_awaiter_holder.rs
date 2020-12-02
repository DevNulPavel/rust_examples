use std::{
    collections::{
        HashMap
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
    handlers::{
        jenkins_handlers::{
            BuildFinishedParameters
        }
    }
};

type ResponseAwaiterCallback = dyn FnOnce(JenkinsJob, Message, BuildFinishedParameters) + Send;

struct ResponseAwaiter{
    job: Option<JenkinsJob>,
    message: Option<Message>,
    params: Option<BuildFinishedParameters>,
    complete: Box<ResponseAwaiterCallback>
}

impl ResponseAwaiter{
    fn new(complete: Box<ResponseAwaiterCallback>) -> ResponseAwaiter {
        ResponseAwaiter{
            job: None,
            message: None,
            params: None,
            complete
        }
    }
    fn is_complete(&self) -> bool{
        if self.job.is_some() && self.message.is_some() && self.params.is_some(){
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
    pub fn provide_build_complete_params(&mut self, url: &JobUrl, params: BuildFinishedParameters, complete: Box<ResponseAwaiterCallback>) {
        let entry = self.awaiters
            .entry(url.to_owned());

        let awaiter = entry.or_insert_with(||{
                ResponseAwaiter::new(Box::new(complete))
            });

        awaiter.params = Some(params);

        if awaiter.is_complete() {
            if let Some(obj) = self.awaiters.remove(url){
                let ResponseAwaiter{complete, job, message, params}= obj;
                complete(
                    job.expect("Job unwrap failed"),
                    message.expect("Message unwrap failed"),
                    params.expect("Params unwrap failed")
                );
            }
        }
    }

    pub fn provide_job(&mut self, url: &JobUrl, job: JenkinsJob, message: Message, complete: Box<ResponseAwaiterCallback>) {
        let entry = self.awaiters
        .entry(url.to_owned());

        let awaiter = entry.or_insert_with(||{
                ResponseAwaiter::new(Box::new(complete))
            });

        awaiter.job = Some(job);
        awaiter.message = Some(message);

        if awaiter.is_complete() {
            if let Some(obj) = self.awaiters.remove(url){
                let ResponseAwaiter{complete, job, message, params}= obj;
                complete(
                    job.expect("Job unwrap failed"),
                    message.expect("Message unwrap failed"),
                    params.expect("Params unwrap failed")
                );
            }
        }
    }
}