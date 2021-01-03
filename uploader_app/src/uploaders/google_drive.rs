use std::{
    path::{
        PathBuf
    }
};
use log::{
    info,
    debug,
    //error
};
use yup_oauth2::{
    read_service_account_key, 
    ServiceAccountAuthenticator
};
use google_drive_client::{
    GoogleDriveClient,
    GoogleDriveUploadTask
};
use crate::{
    app_parameters::{
        GoogleDriveParams
    },
    env_parameters::{
        GoogleDriveEnvironment
    }
};
use super::{
    upload_result::{
        UploadResult,
        UploadResultData
    }
};

pub async fn upload_in_google_drive(client: reqwest::Client, env_params: GoogleDriveEnvironment, app_params: GoogleDriveParams) -> UploadResult {
    info!("Start google drive uploading");

    // Содержимое Json файлика ключа 
    let key = read_service_account_key(env_params.auth_file)
        .await
        .expect("Google drive auth file parsing failed");

    // Аутентификация на основе прочитанного файлика
    let auth = ServiceAccountAuthenticator::builder(key)
          .build()
          .await
          .expect("Failed to create google drive authenticator");
 
    // Add the scopes to the secret and get the token.
    let token = auth
        .token(&["https://www.googleapis.com/auth/drive"])
        .await
        .expect("Failed to get google drive token");

    // Проверяем получившийся токен
    if token.as_str().is_empty() {
        panic!("Empty google drive token is not valid");
    }

    // Клиент
    let client = GoogleDriveClient::new(client, token);

    // Целевая папка
    let folder = {
        let folder = client
            .get_folder_for_id(&app_params.target_folder_id)
            .await?
            .ok_or_else(||{
                "Target google drive folder is not found"
            })?;
        if let Some(sub_folder_name) = app_params.target_subfolder_name{
            folder
                .create_subfolder_if_needed(&sub_folder_name)
                .await?
        }else{
            folder
        }
    };

    // Грузим файлы
    let mut results = Vec::with_capacity(app_params.files.len());
    for file_path_str in app_params.files {
        let task = GoogleDriveUploadTask{
            file_path: PathBuf::from(file_path_str),
            owner_domain: app_params.target_owner_email.as_ref(),
            owner_email: app_params.target_owner_email.as_ref(),
            parent_folder: &folder
        };
        let result = client
            .upload(task)
            .await?;
        
        debug!("Google drive uploading result: {:#?}", result);
        results.push(result);
    }

    // Финальное сообщение
    let message_begin = format!("Google drive folder:\n- \"{}\"\n  {}\n\nFiles:", 
                                    folder.get_info().name, 
                                    folder.get_info().web_view_link);
    let message = results
        .into_iter()
        .fold(message_begin, |prev, res|{
            format!("{}\n- {}\n  {}", prev, res.file_name, res.web_view_link)
        });

    Ok(UploadResultData{
        target: "Google drive",
        message: Some(message),
        install_url: None
    })
}