use serde::Deserialize;
use serde_json::from_slice;
use eyre::WrapErr;

#[derive(Debug, Deserialize)]
pub struct TokenData{
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String
}

impl TokenData{
    pub fn try_parse_from_data(data: &[u8]) -> Result<TokenData, eyre::Error>{
        let data: TokenData = from_slice(data).wrap_err("Token data parsing failed")?;
        Ok(data)
    }
}