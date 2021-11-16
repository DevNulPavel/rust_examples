use crate::{
    oauth2::{get_token_data, ServiceAccountData, TokenData},
    types::HttpClient,
};
use eyre::{Context, ContextCompat};
use std::{
    path::Path,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tracing::{debug, instrument};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ReceivedTokenInfo {
    data: TokenData,
    expire_time: Instant,
}

impl ReceivedTokenInfo {
    fn is_expire_soon(&self) -> Result<bool, eyre::Error> {
        let now = Instant::now();
        let expire_time = self
            .expire_time
            .checked_sub(Duration::from_secs(10))
            .wrap_err("Invalid expire time")?;
        if now > expire_time {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct AuthTokenProvider {
    http_client: HttpClient,
    account_data: ServiceAccountData,
    scopes: &'static str,
    token_info: Mutex<Option<ReceivedTokenInfo>>,
}

impl AuthTokenProvider {
    pub fn new(http_client: HttpClient, service_account_json_path: &Path, scopes: &'static str) -> Result<AuthTokenProvider, eyre::Error> {
        // Прочитаем креденшиалы для гугла
        let service_acc_data = ServiceAccountData::new_from_file(service_account_json_path).wrap_err("Service account file read err")?;
        debug!("Service account data: {:?}", service_acc_data);

        Ok(AuthTokenProvider {
            http_client,
            account_data: service_acc_data,
            scopes,
            token_info: Mutex::new(None),
        })
    }

    #[instrument(level = "error", skip(self))]
    async fn receive_token(&self) -> Result<ReceivedTokenInfo, eyre::Error> {
        let data = get_token_data(&self.http_client, &self.account_data, self.scopes)
            .await
            .wrap_err("Token receive")?;

        let expire_time = Instant::now()
            .checked_add(Duration::from_secs(data.expires_in))
            .wrap_err("Invalid token expire time")?;

        Ok(ReceivedTokenInfo { data, expire_time })
    }

    #[instrument(level = "error", skip(self))]
    pub async fn get_token(&self) -> Result<String, eyre::Error> {
        // Блокируемся, тем самым не даем другим клиентам тоже получать токены
        let mut token_lock = self.token_info.lock().await;

        // Ограничиваемся количеством итераций, вдруг время жизни токена будет кривое приходить
        for _ in 0..5 {
            // Если токен есть и не протух
            if let Some(info) = token_lock.as_ref() {
                if info.is_expire_soon()? == false {
                    return Ok(info.data.access_token.clone());
                }
            }

            // Иначе запрашиваем токен и обновляем значение
            let new_info = self.receive_token().await.wrap_err("Token receive")?;
            token_lock.replace(new_info);
        }

        return Err(eyre::eyre!("Invalid tokens received more than 5 times"));
    }
}
