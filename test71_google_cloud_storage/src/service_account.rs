use serde::Deserialize;
use std::{path::Path, io::BufReader, fs::File};
use serde_json::from_reader;
use eyre::WrapErr;

#[derive(Debug, Deserialize)]
pub struct ServiceAccountData{
    #[serde(rename = "type")]
    pub acc_type: String,   // TODO: Enum
    pub project_id: String,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub auth_provider_x509_cert_url: String,
    pub client_x509_cert_url: String
}

impl ServiceAccountData{
    pub fn new_from_file(path: &Path) -> Result<ServiceAccountData, eyre::Error>{
        let file = File::open(path).wrap_err("Service account file open err")?;
        let reader = BufReader::new(file);
        let data: ServiceAccountData = from_reader(reader).wrap_err("Service accound data parsing failed")?;
        Ok(data)
    }
}