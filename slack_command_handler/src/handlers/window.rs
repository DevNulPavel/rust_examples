use std::{
    collections::{
        HashMap
    },
    // fmt::{
    //     self
    // }
};
use serde::{
    Deserialize
};
use serde_json::{
    Value
};
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
use crate::{
    application_data::{
        ApplicationData
    },
    slack::{
        ViewInfo
    }
};


// https://serde.rs/enum-representations.html
// https://api.slack.com/reference/interaction-payloads
// https://api.slack.com/interactivity/handling#payloads
#[derive(Deserialize)]
#[serde(tag = "type")]          // Вариант enum будет выбираться по полю type, значения переименовываются
pub enum WindowParametersPayload{
    /// Типа данных при нажатии Sumbit у окошка
    /// https://api.slack.com/reference/interaction-payloads/views#view_submission
    #[serde(rename = "view_submission")]
    Submit{
        trigger_id: String,
        user: String,
        view: ViewInfo,
        //response_url: Option<String>,
    },
    
    /// Типа данных для действий при работе непосредственно с окном
    // https://api.slack.com/reference/interaction-payloads/block-actions
    #[serde(rename = "block_actions")]
    Update{
        trigger_id: String,
        view: ViewInfo,
        actions: Vec<Value>,
        //response_url: Option<String>,
    },

    /// Типа данных для действий при работе непосредственно с окном
    // https://api.slack.com/reference/interaction-payloads/views#view_closed
    #[serde(rename = "view_closed")]
    Close{
        trigger_id: String,
        view: ViewInfo,
        actions: Vec<Value>,
        //response_url: Option<String>,
    },

    /// Типа данных для действий при работе непосредственно с окном
    // https://api.slack.com/reference/interaction-payloads/shortcuts#message_actions
    #[serde(rename = "message_actions")]
    MessageAction{
        trigger_id: String,
        view: ViewInfo,
        actions: Vec<Value>,
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
pub async fn window_handler(parameters: Form<WindowHandlerParameters>, app_data: Data<ApplicationData>) -> HttpResponse {
    debug!("Jenkins window parameters: {:?}", parameters);

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
                    WindowParametersPayload::Submit{view, trigger_id, ..} => {
                        debug!("Submit button processing with trigger_id: {}", trigger_id);
            
                        // TODO: Найти вьюшку здесь и вызвать обработчик вьюшки

                        // Открываем окно с параметрами сборки
                        //open_build_properties_window_by_reponse(trigger_id, view, app_data).await;
                    },
            
                    // Вызывается на нажатие разных кнопок в самом меню
                    // TODO: Можно делать валидацию ветки здесь
                    WindowParametersPayload::Action{..} => {
                        debug!("Action processing");
            
                        // TODO: Найти вьюшку здесь и вызвать обработчик экшенов

                        //update_main_window(view, app_data).await;
                        // push_new_window
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