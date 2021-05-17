use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Update {
    pub html_url: String,
    pub name: String,
}

// Урл для получения последних версий
const UPDATE_URL: &str = "https://api.github.com/repos/agrinman/tunnelto/releases/latest";
// Из окружения при сборке вытягиваем версию текущего приложения
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Проверяем, не доступно ли нам обновление версии?
pub async fn check() {
    match check_inner().await {
        Ok(val) => {
            match val {
                Some(new) => {
                    eprintln!(
                        "{} {} => {} ({})\n",
                        "New version available:".yellow().italic(),
                        CURRENT_VERSION.bright_blue(),
                        new.name.as_str().green(),
                        new.html_url
                    );
                },
                None => {
                    log::debug!("Using latest version.")
                },
            }
        }
        Err(error) => {
            log::error!("Failed to check version: {:?}", error)
        },
    }
}

/// checks for a new release on github
async fn check_inner() -> Result<Option<Update>, Box<dyn std::error::Error>> {
    // Делаем запрос
    let update: Update = reqwest::Client::new()
        .get(UPDATE_URL)
        .header("User-Agent", "tunnelto-client")
        .header("Accept", "application/vnd.github.v3+json") // Странный формат принимаемых данных
        .send()
        .await?
        .json()
        .await?;

    if update.name.as_str() != CURRENT_VERSION {
        Ok(Some(update))
    } else {
        Ok(None)
    }
}
