
/*use std::{
    path::{
        Path,
    },
    fs::{
        File,
        create_dir,
    },
    env
};
use log::{
    info,
    // warn,
    // debug
};
use sqlx::{
    sqlite::{
        SqliteConnection,
        SqliteRow
    },
    Connection,
};
/*use sqlx::{
    Connect,
    Cursor,
    sqlite::{
        SqliteConnection,
        SqliteCursor,
        SqliteRow
    }
};*/
use super::{
    error::{
        DatabaseError
    }
};



/*async fn build_tables(conn: &mut SqliteConnection){
    let sql_path = Path::new("sql/create_database.sql");
    let sql_text: String = read_to_string(sql_path)
        .await
        .expect("File сreate_database.sql does not exist");

    sqlx::query(&sql_text)
        .execute(conn)
        .await
        .expect("Create database failed");
}

async fn check_tables_exists(conn: &mut SqliteConnection) -> bool {
    const SQL: &str = "SELECT name \
                       FROM sqlite_master \
                       WHERE type='table' AND name='monitoring_users'";
    let mut cursor: SqliteCursor = sqlx::query(SQL)
        .fetch(conn);
    
    let value: Option<SqliteRow> = cursor.next().await.expect("Cursor error");

    value.is_some()
}

pub async fn build_sqlite_connection(connection_param: &str) -> SqliteConnection {
    let mut db_conn = SqliteConnection::connect(connection_param)
        .await
        .expect("Sqlite connection create failed");

    let tables_exist = check_tables_exists(&mut db_conn).await;
    if !tables_exist {
        info!("Database doesn't exist, need to create it");
        build_tables(&mut db_conn).await;
    }else{
        info!("Database already exists");
    }

    db_conn
}*/

struct DatabaseClient{
    connection: SqliteConnection
}

impl DatabaseClient{
    pub fn new(path: &Path) -> Result<DatabaseClient, DatabaseError>{
        // Проверяем, есть ли уже файлик
        // TODO: Асихронные действия?
        if path.exists() == false {
            if let Some(folder) = path.parent(){
                create_dir(folder).
                    map_err(|err|{
                        DatabaseError::DatabaseFileOpenErr(err, "Database folder create failed".into());
                    })?;
            }

            File::create(path)
                .expect("Database file create failed");
        }

        // База данных
        let sqlite_connection_path: String = format!("sqlite:{}", path
            .to_str()
            .expect("Database path to string failed"));

        let mut db_conn = SqliteConnection::connect(connection_param)
            .await
            .expect("Sqlite connection create failed");
    
        let tables_exist = check_tables_exists(&mut db_conn).await;
        if !tables_exist {
            info!("Database doesn't exist, need to create it");
            build_tables(&mut db_conn).await;
        }else{
            info!("Database already exists");
        }
    
        db_conn
    }
}
*/