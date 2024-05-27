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
    pub(super) fn try_parse_from_data(data: &[u8]) -> Result<TokenData, eyre::Error>{
        let mut data: TokenData = from_slice(data).wrap_err("Token data parsing failed")?;
        // Удалим лишние символы у токена в конце если они есть
        if data.access_token.ends_with("..."){
            let res = data.access_token.trim_end_matches('.').to_owned();
            data.access_token = res;
        }
        Ok(data)
    }
}