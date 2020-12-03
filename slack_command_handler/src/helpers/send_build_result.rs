use actix_web::{
    web::{
        Data,
    },
    rt::{
        spawn
    }
};
use log::{
    debug,
    info,
    error
};
use crate::{
    qr::{
        create_qr_data
    },
    slack::{
        SlackMessageTaget,
        SlackImageTarget,
        Message
    },
    jenkins::{
        JenkinsJob
    },
    handlers::{
        jenkins_handlers::{
            BuildFinishedParameters,
            BuildResultUserInfo
        },
        slack_handlers::{
            AppMentionMessageInfo
        }
    },
    ApplicationData,
};



pub fn send_message_with_build_result_direct_message(params: BuildFinishedParameters,
                                                     app_data: Data<ApplicationData>) {
    spawn(async move {
        let user_info = match params.user_info{
            Some(info) => info,
            None => return
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

        // Если есть файл, значит грузим на него QR код
        if let Some((image_data, commentary)) = file_info {
            let commentary = format!("Build finished: {}", commentary);
            let result = app_data
                .slack_client
                .send_image(image_data, commentary.clone(), SlackImageTarget::to_user_direct(&user_info.build_user_id))
                .await;
            
            if let Err(err) = result {
                error!("Image upload error: {:?}", err);
                let result = app_data
                    .slack_client
                    .send_message(&commentary,
                                  SlackMessageTaget::to_user_direct(&user_info.build_user_id))
                    .await;
                if let Err(err) = result {
                    error!("Message send error: {:?}", err);
                }
            }
        } else{
            error!("Missing file link information");
        }
    });
}


pub fn send_message_with_build_result_into_thread(_: JenkinsJob, 
                                                 root_message: AppMentionMessageInfo,
                                                 _: Message,
                                                 params: BuildFinishedParameters,
                                                 app_data: Data<ApplicationData>) {
    spawn(async move {
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

        // Если есть файл, значит грузим на него QR код
        if let Some((image_data, commentary)) = file_info {
            let commentary = format!("<@{}> Build finished: {}", root_message.user, commentary);
            let result = app_data
                .slack_client
                .send_image(image_data, commentary.clone(), SlackImageTarget::to_thread(&root_message.channel, &root_message.ts))
                .await;
            
            if let Err(err) = result {
                error!("Image upload error: {:?}", err);
                let result = app_data
                    .slack_client
                    .send_message(&commentary,
                                  SlackMessageTaget::to_thread(&root_message.channel, &root_message.ts))
                    .await;
                if let Err(err) = result {
                    error!("Message send error: {:?}", err);
                }
            }
        }else{
            error!("Missing file link information");
        }
    });
}
