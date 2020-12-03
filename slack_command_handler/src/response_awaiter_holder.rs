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

type ResponseAwaiterCallback = dyn FnOnce(JobUrl,
                                          JenkinsJob, 
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
        // Время жизни объекта - 45 минут
        ResponseAwaiter{
            destroy_time: Instant::now().checked_add(Duration::from_secs(60 * 45)).unwrap(),
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
    /// Создаем потокобезопасный контейнер с запущенной периодической чисткой
    pub fn new() -> Data<Self>{
        let response_awaiter = Data::new(ResponseAwaiterHolder::default());
        ResponseAwaiterHolder::start_periodical_cleanup(response_awaiter.clone());
        response_awaiter
    }

    fn start_periodical_cleanup(self_obj: Data<Self>){
        spawn(async move {
            use actix_web::rt::time::delay_for;

            // Чистку будем делать каждые 2 минуты
            loop {
                delay_for(Duration::from_secs(60 * 2)).await;

                let now = Instant::now();
                if let Ok(mut holder) = self_obj.awaiters.lock(){
                    // Оставляем только те объекты, которые еще должны жить
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

        // Блокируемя на объекте
        if let Ok(mut awaiter) = self.awaiters.lock(){
            // Получаем хранитель из мапы
            let entry = awaiter
                .entry(url.to_owned());

            // Создаем объект
            let awaiter_obj = entry
                .or_insert_with(||{
                    ResponseAwaiter::new(Box::new(complete))
                });

            // Исполняем внешнее обновление
            update(awaiter_obj);

            // Если наш объект заполнен
            if awaiter_obj.is_complete() {
                // TODO: Оптимизировать

                // Извлекаем объект
                if let Some(obj) = awaiter.remove(&url){

                    // Разворачиваем объект на содержимое
                    let ResponseAwaiter{complete, job, root_message, message, params, ..}= obj;

                    // Вызываем коллбек с данными
                    complete(
                        url,
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

    /// Предоставляем данные от запроса-коллбека
    pub fn provide_build_complete_params(&self, 
                                         url: JobUrl, 
                                         params: BuildFinishedParameters, 
                                         app_data: Data<ApplicationData>, 
                                         complete: Box<ResponseAwaiterCallback>) {
        self.try_to_update_entry_with_complete(url, app_data, complete, |entry: &mut ResponseAwaiter|{
            entry.params = Some(params);
        });
    }

    /// Предоставляем данные после получения урла джобы
    pub fn provide_job(&self, 
                       url: JobUrl, 
                       job: JenkinsJob, 
                       root_message: AppMentionMessageInfo, 
                       message: Message, 
                       app_data: Data<ApplicationData>, 
                       complete: Box<ResponseAwaiterCallback>) {
        self.try_to_update_entry_with_complete(url, app_data, complete, |entry: &mut ResponseAwaiter|{
            entry.job = Some(job);
            entry.message = Some(message);
            entry.root_message = Some(root_message);    
        });
    }
}