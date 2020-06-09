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
    currency_check_status::{
        CurrencyCheckStatus
    },
    error::{
        TelegramBotError,
        TelegramBotResult,
        DatabaseErrKind
    }
};

pub struct CurrencyUsersStorrage{
    pub users_for_push: HashMap<UserId, CurrencyCheckStatus>
}

impl CurrencyUsersStorrage{
    pub async fn new(conn: &mut SqliteConnection) -> Self{
        // TODO: Заменить на ошибку вместо падения?
        let result_ids: Vec<i64> = sqlx::query("SELECT user_id FROM monitoring_users")
            .map(|row: SqliteRow| {
                let user_id: i64 = row.get(0);
                user_id
            })
            .fetch_all(&mut *conn)
            .await
            .expect("Failed select users form DB");
        
        // Полученные id пользователей перегоняем в CurrencyCheckStatus, загружая информацию о минимумах внутри
        let results: Vec<CurrencyCheckStatus> = {
            let mut results = Vec::with_capacity(result_ids.len());
            for id_val in result_ids{
                let status = CurrencyCheckStatus::load(UserId::new(id_val), &mut (*conn)).await;
                results.push(status);
            }
            results
        };
        
        // Просто перегоняем в HashMap
        let map: HashMap<UserId, CurrencyCheckStatus> = results
            .into_iter()
            .map(|val|{
                (val.user.clone(), val)
            })
            .collect();

        // Создаем хранилище с результатами
        CurrencyUsersStorrage{
            users_for_push: map
        }
    }

    pub async fn add_user(&mut self, user: &UserId, conn: &mut SqliteConnection) -> TelegramBotResult {
        if self.users_for_push.contains_key(&*user) {
            return Err(TelegramBotError::CustomError("User monitoring already enabled".into()));
        }

        let check = CurrencyCheckStatus::new(user.clone());
        
        // TODO: Именованные параметры в SQL, убрать дупликаты
        // TODO: Валидация параметров!!
        let id_num: i64 = (*user).into();
        const SQL: &str =   "BEGIN; \
                                DELETE FROM currency_minimum WHERE user_id = ?; \
                                INSERT INTO monitoring_users(user_id) VALUES (?); \
                            COMMIT;";
        let insert_result = sqlx::query(SQL)
            .bind(id_num)
            .bind(id_num)
            .execute(conn)
            .await;                         

        /*const SQL: &str =   "DELETE FROM currency_minimum WHERE user_id = ?; \
                             INSERT INTO monitoring_users(user_id) VALUES (?);";
        let mut transaction = conn.begin().await?;
        let insert_result = sqlx::query(SQL)
            .bind(id_num)
            .execute(&mut transaction)
            .await;
        transaction.commit().await?;*/

        match insert_result{
            Ok(rows_updated) if rows_updated > 0 => {
                self.users_for_push.insert(user.clone(), check);
                Ok(())
            },
            Ok(_) => {
                Err(TelegramBotError::CustomError("User insert failed, 0 rows included".into()))
            },
            Err(e) => {
                Err(TelegramBotError::DatabaseErr{
                    err: e,
                    context: DatabaseErrKind::InsertUser
                })
            }
        }
    }

    pub async fn remove_user(&mut self, user: &UserId, conn: &mut SqliteConnection) -> TelegramBotResult {
        if self.users_for_push.contains_key(&*user) == false {
            return Err(TelegramBotError::CustomError("User monitoring doesn't enabled".into()));
        }

        // TODO: Нужна ли транзакция? Можно ли как-то удалить все, что относится к user
        // TODO: Валидация параметров!!
        const SQL: &str =   "BEGIN; \
                                DELETE FROM currency_minimum WHERE user_id = ?; \
                                DELETE FROM monitoring_users WHERE user_id = ?; \
                            COMMIT;";
        let id_num: i64 = (*user).into();
        let remove_result = sqlx::query(SQL)
            .bind(id_num)
            .bind(id_num)
            .execute(conn)
            .await;
        
        match remove_result{
            Ok(users_updated) if users_updated > 0 =>{
                self.users_for_push.remove(user);
                Ok(())    
            },
            Ok(_) => {
                Err(TelegramBotError::CustomError("User remove failed, 0 rows removed".into()))
            },
            Err(e)=>{
                Err(TelegramBotError::DatabaseErr{
                    err: e,
                    context: DatabaseErrKind::RemoveUser
                })
            }
        }
    }

    pub fn try_get_user(&self, user: &UserId) -> Option<&CurrencyCheckStatus> {
        self.users_for_push.get(user)
    }

    pub fn is_empty(&self) -> bool{
        self.users_for_push.is_empty()
    }
}
