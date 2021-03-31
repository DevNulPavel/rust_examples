use std::{
    fs::{
        File
    }
};
use serde::{
    Deserialize
};
use serde_with::{
    serde_as,
    DisplayFromStr
};


#[serde_as]
#[derive(Debug, Deserialize)]
pub struct GoogleEnvParams{
    pub client_id: String,
    pub client_secret: String,
    pub project_id: String,
    pub redirect_uris: Vec<String>,

    #[serde_as(as = "DisplayFromStr")] // TODO: ???
    pub auth_uri: url::Url,

    #[serde_as(as = "DisplayFromStr")] // TODO: ???
    pub token_uri: url::Url,

    #[serde_as(as = "DisplayFromStr")] // TODO: ???
    pub auth_provider_x509_cert_url: url::Url,
}

impl GoogleEnvParams {
    pub fn get_from_env() -> GoogleEnvParams {
        let auth_file_path = std::env::var("GOOGLE_OAUTH_CREDENTIAL_FILE")
            .expect("GOOGLE_OAUTH_CREDENTIAL_FILE environment variable is missing");

        let file = File::open(auth_file_path)
            .expect("Google auth file open failed");

        #[derive(Debug, Deserialize)]
        pub struct Data{
            web: GoogleEnvParams
        }
        
        let res: Data = serde_json::from_reader(file)
            .expect("Google auth data parsing failed");

        res.web
    }
}