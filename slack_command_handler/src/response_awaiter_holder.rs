use std::{
    collections::{
        HashMap
    },
    sync::{
        Mutex
    },
    time::{
        Instant,
        Duration
    }
};
use log::{
    error
};
use actix_web::{
    web::{
        Data
    },
    rt::{
        spawn
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
        slack_handlers::{
            AppMentionMessageInfo
        },
        jenkins_handlers::{
            BuildFinishedParameters
        }
    }
};

type ResponseAwaiterCallback = dyn FnOnce(JenkinsJob, 
                                          AppMentionMessageInfo, 
                                          Message, 
                                          BuildFinishedParameters, 
                                          Data<ApplicationData>) + Send;

struct ResponseAwaiter{
    destroy_time: Instant,
    job: Option<JenkinsJob>,
    root_message: Option<AppMentionMessageInfo>,
    message: Option<Message>,
    params: Option<BuildFinishedParameters>,
    complete: Box<ResponseAwaiterCallback>
}

impl ResponseAwaiter{
    fn new(complete: Box<ResponseAwaiterCallback>) -> ResponseAwaiter {
        // Время жизни объекта - 30 минут
        ResponseAwaiter{
            destroy_time: Instant::now().checked_add(Duration::from_secs(60 * 30)).unwrap(),
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
    awaiters: Mutex<HashMap<JobUrl, ResponseAwaiter>>,
}

// TODO: Fix box
impl ResponseAwaiterHolder {
    pub fn new() -> Data<Self>{
        let response_awaiter = Data::new(ResponseAwaiterHolder::default());
        ResponseAwaiterHolder::start_periodical_cleanup(response_awaiter.clone());
        response_awaiter
    }

    fn start_periodical_cleanup(self_obj: Data<Self>){
        spawn(async move {
            use actix_web::rt::time::delay_for;

            loop {
                delay_for(Duration::from_secs(30)).await;

                let now = Instant::now();
                if let Ok(mut holder) = self_obj.awaiters.lock(){
                    holder.retain(|_, val|{
                        val.destroy_time > now
                    });
                }else{
                    error!("Response awaiter lock failed");
                    break;
                }
            }
        });
    }

    fn try_to_update_entry_with_complete<U>(&self, 
                                            url: JobUrl, 
                                            app_data: Data<ApplicationData>, 
                                            complete: Box<ResponseAwaiterCallback>, 
                                            update: U)
    where U: FnOnce(&mut ResponseAwaiter) {

        if let Ok(mut awaiter) = self.awaiters.lock(){
            let entry = awaiter
                .entry(url.to_owned());

            let awaiter_entry = entry
                .or_insert_with(||{
                    ResponseAwaiter::new(Box::new(complete))
                });

            update(awaiter_entry);

            if awaiter_entry.is_complete() {
                // TODO: Оптимизировать
                if let Some(obj) = awaiter.remove(&url){
                    let ResponseAwaiter{complete, job, root_message, message, params, ..}= obj;
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

    pub fn provide_build_complete_params(&self, url: JobUrl, params: BuildFinishedParameters, app_data: Data<ApplicationData>, complete: Box<ResponseAwaiterCallback>) {
        self.try_to_update_entry_with_complete(url, app_data, complete, |entry: &mut ResponseAwaiter|{
            entry.params = Some(params);
        });
    }

    pub fn provide_job(&self, url: JobUrl, job: JenkinsJob, root_message: AppMentionMessageInfo, message: Message, app_data: Data<ApplicationData>, complete: Box<ResponseAwaiterCallback>) {
        self.try_to_update_entry_with_complete(url, app_data, complete, |entry: &mut ResponseAwaiter|{
            entry.job = Some(job);
            entry.message = Some(message);
            entry.root_message = Some(root_message);    
        });
    }
}