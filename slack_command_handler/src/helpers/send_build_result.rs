use std::{
    path::{
        PathBuf
    }
};
use actix_web::{
    web::{
        Data,
    },
    rt::{
        spawn
    }
};
use log::{
    // debug,
    // info,
    error
};
use slack_client_lib::{
    SlackMessageTarget,
    SlackImageTarget
};
use crate::{
    qr::{
        create_qr_data
    },
    handlers::{
        jenkins_handlers::{
            BuildFinishedParameters
        }
    },
    response_awaiter_holder::{
        ResponseAwaiterCallbackData
    },
    ApplicationData,
};

// TODO: Рефакторинг, есть дублирующийся код

pub fn send_message_with_build_result(params: BuildFinishedParameters,
                                      app_data: Data<ApplicationData>) {
    spawn(async move {
        let user_info = match params.user_info{
            Some(info) => info,
            None => {
                error!("Empty user info");
                return;
            }
        };

        let file_info = params
            .file_info
            .and_then(|link|{
                match create_qr_data(&link.build_file_link){
                    Ok(data) => Some((data, link.build_file_commentary)),
                    Err(err) => {
                        error!("Qr code create error: {:?}", err);
                        None
                    }
                }
            });

        // Так как идентификаторы слака и дженкинса могут не совпадать, тогда ищем по email и полному имени
        // Если не нашли, пробуем айдишник, который прислали
        let user_id = {
            // Пути к папке с кешем пользователей
            let cache_file_full_path = PathBuf::new()
                .join(&dirs::home_dir().unwrap())
                .join(".cache/slack_direct_messenger/users_cache.json");

            app_data
                .slack_client
                .find_user_id(&user_info.build_user_email, &user_info.build_user_name, &cache_file_full_path)
                .await
        };
        let user_id = match user_id {
            Some(user_id) => user_id,
            None => {
                error!("User if not found");   
                user_info.build_user_id
            }
        };

        // Если есть файл, значит грузим на него QR код
        if let Some((image_data, commentary)) = file_info {
            let commentary = commentary.replace("\\n", "\n");
            let commentary = format!(":borat:\n{}\n```{}```", params.job_info.build_job_url, commentary);

            // TODO: Optimize

            if let Some(default_channel) = params.default_channel{
                if let Err(err) = app_data
                                    .slack_client
                                    .send_image(image_data.clone(), commentary.clone(), SlackImageTarget::to_channel(&default_channel))
                                    .await {
                        error!("Image upload error: {:?}", err);
                }
            }  
            
            if let Err(err) = app_data
                                .slack_client
                                .send_image(image_data, commentary.clone(), SlackImageTarget::to_user_direct(&user_id))
                                .await {
                error!("Image upload error: {:?}", err);
            }
        } else{
            let commentary = format!(":borat:\n{}", params.job_info.build_job_url,);

            // TODO: Optimize

            if let Some(default_channel) = params.default_channel{
                if let Err(err) = app_data
                                    .slack_client
                                    .send_message(&commentary,
                                                  SlackMessageTarget::to_channel(&default_channel))
                                    .await {
                    error!("Message send error: {:?}", err);
                }
            }

            if let Err(err) = app_data
                                .slack_client
                                .send_message(&commentary,
                                            SlackMessageTarget::to_user_direct(&user_id))
                                .await {
                error!("Message send error: {:?}", err);
            }
        }
    });
}


pub fn send_message_with_build_result_into_thread(mut data: ResponseAwaiterCallbackData) {
    spawn(async move {
        // Обновление сообщения со ссылкой на джобу
        {
            let new_text = format!("```{}```", data.finished_params.job_info.build_job_url);
            data.build_message
                .update_text(&new_text)
                .await
                .ok();
        }

        // Получаем данные для QR кода
        let file_info = data
            .finished_params
            .file_info
            .and_then(|link|{
                match create_qr_data(&link.build_file_link){
                    Ok(data) => Some((data, link.build_file_commentary)),
                    Err(err) => {
                        error!("Qr code create error: {:?}", err);
                        None
                    }
                }
            });

        // Если есть файл, значит грузим на него QR код
        if let Some((image_data, commentary)) = file_info {
            // Qr код со ссылкой
            let commentary = commentary.replace("\\n", "\n");
            let commentary = format!("<@{}>\n:borat:\n```{}```", data.root_trigger_message.user, commentary);
            let result = data
                .app_data
                .slack_client
                .send_image(image_data, 
                            commentary.clone(), 
                            SlackImageTarget::to_thread(&data.root_trigger_message.channel, &data.root_trigger_message.ts))
                .await;
            
            if let Err(err) = result {
                error!("Image upload error: {:?}", err);
            }
        }else{
            // Просто текст в тред
            let commentary = format!("<@{}>\n:borat:", data.root_trigger_message.user);
            let result = data
                .app_data
                .slack_client
                .send_message(&commentary,
                              SlackMessageTarget::to_thread(&data.root_trigger_message.channel, &data.root_trigger_message.ts))
                .await;
            if let Err(err) = result {
                error!("Message send error: {:?}", err);
            }
        }
    });
}
