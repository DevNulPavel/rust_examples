use std::{
    collections::HashMap,
    convert::TryFrom
};
use log::{
    info,
    error
};
use chrono::prelude::*;
use telegram_bot::{
    Message,
    CanSendMessage,
    ParseMode,
    UserId,
    SendMessage
};
use currency_lib::{
    CurrencyResult,
    CurrencyMinimum,
    CurrencyValue,
    CurrencyType,
    get_all_currencies
};
use sqlx::{
    prelude::*,
    // Transaction,
    sqlite::{
        SqliteConnection,
        SqliteRow
    }
};
use crate::{
    app_context::{
        AppContext
    },
    bot_context::{
        BotContext
    },
    currency_users_storrage::{
        CurrencyUsersStorrage
    },
    error::{
        TelegramBotError,
        TelegramBotResult,
        DatabaseErrKind
    }
};

#[derive(Clone)]
pub struct CurrencyCheckStatus {
    pub user: UserId,
    pub minimum_values: Vec<CurrencyMinimum>,
}

impl CurrencyCheckStatus{
    pub fn new(user: UserId) -> Self{
        CurrencyCheckStatus{
            user: user,
            minimum_values: vec![]
        }
    }

    pub async fn load(user: UserId, conn: &mut SqliteConnection) -> Self{
        const SQL: &str = "SELECT * FROM currency_minimum \
                            WHERE user_id = ?";
        let user_id: i64 = user.into();
        let results: Vec<CurrencyMinimum> = sqlx::query(SQL)
            .bind(user_id)
            .map(|row: SqliteRow| {                
                // Валюта
                let cur_type = {
                    let type_str: String = row.get("cur_type");
                    CurrencyType::try_from(type_str.as_str()).expect("Invalid currency type")
                };

                // Получаем время в виде timestamp в UTC
                let time: Option<chrono::DateTime<chrono::Utc>> = {
                    let timestamp: i64 = row.get("update_time");
                    if timestamp > 0 {
                        let native_time = chrono::NaiveDateTime::from_timestamp(timestamp, 0);
                        let time = chrono::DateTime::<chrono::Utc>::from_utc(native_time, chrono::Utc);
                        Some(time)
                    } else {
                        None
                    }
                };

                let res = CurrencyMinimum{
                    bank_name: row.get("bank_name"),
                    value: row.get("value"),
                    cur_type: cur_type,
                    update_time: time
                };
                info!("Load user's minimum for id = {} from database: {:?}", user_id, res);
                res
            })
            .fetch_all(conn)
            .await
            .expect("Failed select user's minimums form DB");

        CurrencyCheckStatus{
            user: user,
            minimum_values: results
        }
    }
}

// Пустая реализация на базе PartialEq без перегрузок
impl Eq for CurrencyCheckStatus {
}

impl PartialEq for CurrencyCheckStatus{
    fn eq(&self, other: &CurrencyCheckStatus) -> bool{
        self.user.eq(&other.user)
    }
}

impl std::hash::Hash for CurrencyCheckStatus {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.user.hash(state);
    }
}

impl CurrencyCheckStatus{
}