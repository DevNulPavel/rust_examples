use std::{
    time::{
        Duration
    },
    path::{
        Path,
        PathBuf
    }
};
use tokio::{
    time::{
        delay_for
    },
    fs::{
        File
    },
    io::{
        AsyncRead,
        AsyncReadExt
    }
};
use tokio_util::{
    codec::{
        FramedRead,
        BytesCodec
    }
};
use log::{
    info,
    //debug,
    error
};
use reqwest::{
    Body
};
use google_drive::{
    GoogleDrive,
    FileUploadContent
};
use yup_oauth2::{
    parse_application_secret,
    read_service_account_key, 
    ServiceAccountAuthenticator,
    ServiceAccountKey
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

pub async fn upload_in_google_drive(env_params: GoogleDriveEnvironment, app_params: GoogleDriveParams) -> UploadResult {
    info!("Start google drive uploading");

    // Содержимое Json файлика ключа 
    /*let key = ServiceAccountKey{
        client_email: env_params.email,
        private_key: env_params.key,
        private_key_id: Some(env_params.key_id),
        auth_provider_x509_cert_url: None,
        auth_uri: Some("https://accounts.google.com/o/oauth2/auth".to_owned()),
        client_id: Some("103678887665860884939".to_owned()),
        client_x509_cert_url: Some("https://www.googleapis.com/oauth2/v1/certs".to_owned()),
        key_type: Some("service_account".to_owned()),
        project_id: Some("testapp-258813".to_owned()),
        token_uri: "https://oauth2.googleapis.com/token".to_owned()
    };*/
    let key = read_service_account_key(env_params.auth_file)
        .await
        .expect("Google drive auth file parsing failed");

    let auth = ServiceAccountAuthenticator::builder(key)
          .subject("")
          .build()
          .await
          .expect("Failed to create google drive authenticator");
 
    // Add the scopes to the secret and get the token.
    let token = auth
        .token(&["https://www.googleapis.com/auth/drive"])
        .await
        .expect("Failed to get google drive token");
 
    if token.as_str().is_empty() {
        panic!("Empty google drive token is not valid");
    }

    // Drive client
    let client = GoogleDrive::new(token);

    // 


    for file_path_str in app_params.files{
        // https://www.reddit.com/r/rust/comments/ilpqvy/faster_way_to_read_a_file_in_chunks/

        let path = Path::new(&file_path_str);
        let file_name = path
            .file_name()
            .expect("Google drive Invalid file path")
            .to_str()
            .expect("Path convert failed");
        let file = File::open(path).await?;
        let file_length = file.metadata().await?.len();
        let reader = FramedRead::new(file, BytesCodec::new());
        let stream = Body::wrap_stream(reader);

        let contents = FileUploadContent::new(file_length, stream);
        client
            .create_or_upload_file(&app_params.target_drive_id, &app_params.target_folder_id, file_name, "application/octet-stream", contents)
            .await?;
    }

    Ok(UploadResultData{
        target: "Google drive",
        message: None,
        download_url: None,
        install_url: None
    })
}