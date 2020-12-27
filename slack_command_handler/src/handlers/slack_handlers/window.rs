use serde::{
    Deserialize
};
// use serde_json::{
//     Value
// };
use actix_web::{
    web::{
        Data,
        Form
    },
    rt::{
        spawn
    },
    HttpResponse
};
use log::{
    debug,
    // info,
    error
};
use slack_client_lib::{
    ViewInfo
};
use crate::{
    session::{
        WindowSession
    },
    application_data::{
        ApplicationData
    },
    active_views_holder::{
        ViewsHandlersHolder
    }
};

#[derive(Deserialize)]
struct UserInfo{
    id: String,
    name: String
}

// https://serde.rs/enum-representations.html
// https://api.slack.com/reference/interaction-payloads
// https://api.slack.com/interactivity/handling#payloads
#[derive(Deserialize)]
#[serde(tag = "type")]          // Вариант enum будет выбираться по полю type, значения переименовываются
enum WindowParametersPayload{
    /// Типа данных при нажатии Sumbit у окошка
    /// https://api.slack.com/reference/interaction-payloads/views#view_submission
    #[serde(rename = "view_submission")]
    Submit{
        trigger_id: String,
        user: UserInfo,
        view: ViewInfo,
        // response_url: String,
    },
    
    /// Типа данных для действий при работе непосредственно с окном
    // https://api.slack.com/reference/interaction-payloads/block-actions
    #[serde(rename = "block_actions")]
    Update{
        // trigger_id: String,
        view: ViewInfo,
        // actions: Vec<Value>
    },

    /// Типа данных для действий при работе непосредственно с окном
    // https://api.slack.com/reference/interaction-payloads/views#view_closed
    #[serde(rename = "view_closed")]
    Close{
        trigger_id: String,
        user: UserInfo,
        view: ViewInfo,
        // response_url: String,
    },

    /// Типа данных для действий при работе непосредственно с окном
    // https://api.slack.com/reference/interaction-payloads/shortcuts#message_actions
    #[serde(rename = "message_actions")]
    MessageAction{
        //trigger_id: String,
        //message: String,
        //response_url: Option<String>,
    }
}

/*impl fmt::Debug for WindowParametersPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(serde_json::to_string_pretty(self)
            .unwrap()
            .as_str())
    }
}*/

#[derive(Deserialize, Debug)]
pub struct WindowHandlerParameters{
    pub payload: String
}

/// Обработчик открытия окна Jenkins
pub async fn slack_window_handler(parameters: Form<WindowHandlerParameters>, app_data: Data<ApplicationData>, views_holder: Data<ViewsHandlersHolder>) -> HttpResponse {
    debug!("Jenkins window parameters: {:?}", parameters);

    //debug!("Jenkins window parameters: {:?}", parameters.payload);

    // Парсим переданные данные
    match serde_json::from_str::<WindowParametersPayload>(parameters.payload.as_str()){
        // Распарсили без проблем
        Ok(payload) => {
            //info!("Parsed payload: {:?}", payload);

            // Обрабатываем команды окна, запуск происходит асинхронно, 
            // чтобы максимально быстро ответить серверу
            // https://api.slack.com/surfaces/modals/using#interactions
            spawn(async move {
                // TODO: Хрень полная, но больше никак не удостовериться, что сервер слака получил ответ обработчика
                actix_web::rt::time::delay_for(std::time::Duration::from_millis(200)).await;

                match payload {
                    // Вызывается на нажатие кнопки подтверждения
                    WindowParametersPayload::Submit{view, user, trigger_id} => {
                        debug!("Submit button processing with trigger_id: {}", trigger_id);

                        // Найти вьюшку здесь и вызвать обработчик вьюшки
                        if let Some(mut view_obj) = views_holder.pop_view_handler(view.get_id()){
                            // Создание сессии
                            let session = WindowSession::new(app_data, 
                                                             views_holder,
                                                             user.id,
                                                             user.name,
                                                             trigger_id);

                            view_obj.update_info(view);

                            view_obj.on_submit(session);
                        }else{
                            error!("Cannot find view for id: {}", view.get_id());
                        }
                    },
            
                    // Вызывается на нажатие разных кнопок в самом меню
                    // TODO: Можно делать валидацию ветки здесь
                    WindowParametersPayload::Update{view, ..} => {
                        debug!("Action processing");
            
                        // Найти вьюшку здесь и вызвать обработчик вьюшки
                        if let Some(mut view_obj) = views_holder.pop_view_handler(view.get_id()){
                            view_obj.update_info(view);

                            view_obj.on_update();

                            // Сохраняем для дальнейших обновлений
                            views_holder.push_view_handler(view_obj);
                        }else{
                            error!("Cannot find view for id: {}", view.get_id());
                        }
                    },

                    WindowParametersPayload::Close{user, view, trigger_id} => {
                        debug!("Close processing");

                        // Найти вьюшку здесь и вызвать обработчик вьюшки
                        if let Some(mut view_obj) = views_holder.pop_view_handler(view.get_id()){
                            // Создание сессии
                            let session = WindowSession::new(app_data,
                                                             views_holder, 
                                                             user.id,
                                                             user.name,
                                                             trigger_id);

                            view_obj.update_info(view);

                            view_obj.on_close(session);
                        }else{
                            error!("Cannot find view for id: {}", view.get_id());
                        }
                    },

                    WindowParametersPayload::MessageAction{..} => {
                        debug!("Message action processing");
                    }
                }
            });

            HttpResponse::Ok()
                .finish()
        },
        Err(err) => {
            error!("Payload parse error: {:?}", err);
            
            // TODO: Error
            HttpResponse::Ok()
                .body(format!("Payload parse error: {}", err))
        }
    }
}