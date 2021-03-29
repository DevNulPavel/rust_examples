use std::{
    env::{
        self
    }
};
use log::{
    debug
};
use sqlx::{
    // prelude::{
    //     *
    // },
    sqlite::{
        SqlitePool
    }
};
use crate::{
    error::{
        AppError
    }
};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_uuid: String,
    pub facebook_uid: Option<String>,
    pub google_uid: Option<String>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Database{
    db: SqlitePool
}
impl Database{
    pub async fn open() -> Database {
        let sqlite_conn = SqlitePool::connect(&env::var("DATABASE_URL")
                                                .expect("DATABASE_URL env variable is missing"))
            .await
            .expect("Database connection failed");

        // Включаем все миграции базы данных сразу в наш бинарник, выполняем при старте
        sqlx::migrate!("./migrations")
            .run(&sqlite_conn)
            .await
            .expect("database migration failed");

        Database{
            db: sqlite_conn
        }
    }

    /// Пытаемся найти нового пользователя для FB ID 
    pub async fn try_to_find_user_uuid_with_fb_id(&self, id: &str) -> Result<Option<String>, AppError>{
        struct Res{
            user_uuid: String
        }
        let res = sqlx::query_as!(Res,
                        r#"   
                            SELECT app_users.user_uuid
                            FROM app_users 
                            INNER JOIN facebook_users 
                                    ON facebook_users.app_user_id = app_users.id
                            WHERE facebook_users.facebook_uid = ?
                        "#, id)
            .fetch_optional(&self.db)
            .await
            .map_err(AppError::from)?
            .map(|val|{
                val.user_uuid
            });
        Ok(res)
    }

    pub async fn insert_uuid_for_facebook_user(&self, uuid: &str, fb_uid: &str) -> Result<(), AppError>{
        // Стартуем транзакцию, если будет ошибка, то вызовется rollback автоматически в drop
        // если все хорошо, то руками вызываем commit
        let transaction = self.db.begin().await?;

        // TODO: ???
        // Если таблица иммет поле INTEGER PRIMARY KEY тогда last_insert_rowid - это алиас
        // Но вроде бы наиболее надежный способ - это сделать подзапрос
        let new_row_id = sqlx::query!(r#"
                        INSERT INTO app_users(user_uuid)
                            VALUES (?);
                        INSERT INTO facebook_users(facebook_uid, app_user_id)
                            VALUES (?, (SELECT id FROM app_users WHERE user_uuid = ?));
                    "#, uuid, fb_uid, uuid)
            .execute(&self.db)
            .await?
            .last_insert_rowid();

        transaction.commit().await?;

        debug!("New facebook user included: row_id = {}", new_row_id);

        Ok(())
    }

    /// Пытаемся найти нового пользователя для FB ID 
    pub async fn try_find_full_user_info_for_uuid(&self, uuid: &str) -> Result<Option<UserInfo>, AppError>{
        // Специальным образом описываем, что поле действительно может быть нулевым с 
        // помощью вопросика в переименовании - as 'facebook_uid?'
        // так же можно описать, что поле точно ненулевое, чтобы не использовать Option
        // as 'facebook_uid!'
        // https://docs.rs/sqlx/0.4.0-beta.1/sqlx/macro.query.html#overrides-cheatsheet
        sqlx::query_as!(UserInfo,
                        r#"   
                            SELECT 
                                app_users.user_uuid, 
                                facebook_users.facebook_uid as 'facebook_uid?',
                                google_users.google_uid as 'google_uid?'
                            FROM app_users
                            LEFT JOIN facebook_users
                                ON facebook_users.app_user_id = app_users.id
                            LEFT JOIN google_users
                                ON google_users.app_user_id = app_users.id                            
                            WHERE app_users.user_uuid = ?
                        "#, uuid)
            .fetch_optional(&self.db)
            .await
            .map_err(AppError::from)
    }

    /// Пытаемся найти нового пользователя для FB ID 
    pub async fn does_user_uuid_exist(&self, uuid: &str) -> Result<bool, AppError>{
        // TODO: Более оптимальный вариант
        let res = sqlx::query!(r#"   
                                    SELECT user_uuid 
                                    FROM app_users 
                                    WHERE app_users.user_uuid = ?
                                "#, uuid)
            .fetch_optional(&self.db)
            .await
            .map_err(AppError::from)?;
        
        Ok(res.is_some())
    }
}