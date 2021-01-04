use std::{
    path::{
        Path
    }
};
use log::{
    debug
};
use amazon_client::{
    AmazonClient,
    AmazonUploadTask,
    request_token
};
use crate::{
    app_parameters::{
        AmazonParams
    },
    env_parameters::{
        AmazonEnvironment
    },
    uploaders::{
        UploadResult,
        UploadResultData
    }
};


pub async fn upload_in_amazon(http_client: reqwest::Client, 
                              env_params: AmazonEnvironment, 
                              app_params: AmazonParams) -> UploadResult {

    let token = request_token(&http_client, &env_params.client_id, &env_params.client_secret)
        .await?;

    let token_str = token
        .as_str_checked()
        .expect("Token string get failed");

    debug!("Amazon token: {:#?}", token_str);

    let file_path = Path::new(&app_params.file_path);

    // Грузим
    let client = AmazonClient::new(http_client, token);
    let task = AmazonUploadTask{
        application_id: &env_params.app_id,
        file_path: file_path
    };
    client
        .upload(task)
        .await?;

    // Имя файла
    let file_name = file_path
        .file_name()
        .ok_or("Amazon: invalid file name")?
        .to_str()
        .ok_or("Amazon: Invalid file name")?;

    // Финальное сообщение
    let message = format!("Amazon uploading finished:\n- {}", file_name);

    Ok(UploadResultData{
        target: "Amazon",
        message: Some(message),
        install_url: None
    })  
}