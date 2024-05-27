use std::{
    collections::HashMap
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
#[allow(unused_imports)]
use sqlx::{
    prelude::*,
    // Transaction,
    sqlite::{
        SqliteConnection
    }
};
use crate::{
    app_context::{
        AppContext
    },
    bot_context::{
        BotContext
    },
    error::{
        TelegramBotError
    }
};

async fn build_minimum_value(conn: &mut SqliteConnection,
                             user_id: &UserId,
                             bank_name: &str,
                             update_time: Option<DateTime<Utc>>,
                             received_value: &CurrencyValue) ->  Option<CurrencyMinimum>{
    
    let timestamp = match update_time{
        Some(time) => {
            time.timestamp()
        },
        None => {
            -1
        }
    };

    let user_id_int: i64 = (*user_id).into();
    let type_str: &'static str = received_value.cur_type.into();
    let q = sqlx::query("BEGIN; \
                            DELETE FROM currency_minimum WHERE user_id = ? AND cur_type = ?; \
                            INSERT INTO currency_minimum(user_id, bank_name, value, cur_type, update_time) VALUES (?, ?, ?, ?, ?); \
                         COMMIT;")
        .bind(user_id_int)
        .bind(type_str)
        .bind(user_id_int)
        .bind(bank_name)
        .bind(&received_value.buy)
        .bind(type_str)
        .bind(timestamp);
                
    let query_result = q.execute(&mut (*conn)).await;
    match query_result{
        Ok(_)=>{
            // Обновляем значение
            let minimum = CurrencyMinimum{
                bank_name: bank_name.to_string(),
                value: received_value.buy,
                cur_type: received_value.cur_type,
                update_time: update_time
            };

            return Some(minimum);
        },
        Err(e) => {
            error!("Insert new minimum error: {}", e);
        }
    }
    None 
}

async fn process_currency_value(conn: &mut SqliteConnection,
                                user_id: &UserId, 
                                previous_value: &Option<CurrencyMinimum>, 
                                bank_name: &str,
                                update_time: Option<DateTime<Utc>>,
                                received_value: &CurrencyValue) ->  Option<CurrencyMinimum> {
    // Если есть предыдущее значение
    if let Some(previous_value) = previous_value {
        // Проверяем минимум
        let is_lower = previous_value.value > received_value.buy;

        if is_lower {
            info!("New minimum update");

            return build_minimum_value(conn, user_id, bank_name, update_time, received_value).await;
        }
    }else{
        info!("New minimum");

        return build_minimum_value(conn, user_id, bank_name, update_time, received_value).await;
    }

    None
}

async fn check_minimum_for_value(user_id: &UserId,
                                 conn: &mut SqliteConnection,
                                 cur_type: CurrencyType,
                                 received_minimum: &CurrencyResult, 
                                 previous_minimum_values: &mut Vec<CurrencyMinimum>) -> Option<String> {
    // Ищем предыдущее значение для банка у юзера
    let previous_minimum: Option<CurrencyMinimum> = previous_minimum_values
        .iter()
        .find(|value|{
            value.cur_type == cur_type
        })
        .map(|ref_val|{
            ref_val.clone()
        });

    let received_minimum_val = match cur_type{
        CurrencyType::USD => &received_minimum.usd,
        CurrencyType::EUR => &received_minimum.eur,
    };

    // Если есть предыдущее значение
    let new_minimum = process_currency_value(conn,
                                            user_id,
                                            &previous_minimum,  
                                            &received_minimum.bank_name,
                                            received_minimum.update_time,
                                            received_minimum_val).await;

    if let Some(new_minimum) = new_minimum{
        // Форматируем изменение
        let result = match previous_minimum{
            Some(ref prev) => {
                markdown_format_minimum(&new_minimum, prev)
            },
            None => {
                markdown_format_minimum_for_status(&new_minimum)
            }
        };

        let old = previous_minimum_values
            .iter_mut()
            .find(|val|{
                val.cur_type == new_minimum.cur_type 
            });
        if let Some(old) = old {
            *old = new_minimum;
        }else{
            previous_minimum_values.push(new_minimum);
        }

        return Some(result);
    }    
      
    None
}

fn comparator(v1: f32, v2: f32) -> std::cmp::Ordering{
    if v1 < v2{
        std::cmp::Ordering::Less
    }else if v1 > v2 {
        std::cmp::Ordering::Greater
    }else{
        std::cmp::Ordering::Equal
    }
}

pub async fn check_currencies_update(bot_context: &mut BotContext) {
    // Если некому слать - выходим
    if bot_context.app_context.users_for_push.is_empty(){
        return;
    }

    // Получаем новые значения и фильтруем только валидные
    let received_bank_currencies: Vec<CurrencyResult> = get_all_currencies(&bot_context.app_context.client)
        .await
        .into_iter()
        .filter_map(|result|{
            result.ok()
        })
        .collect();

    // Если пусто - выходим
    if received_bank_currencies.is_empty(){
        return;
    }

    // Кому и какие обновления рассылаем
    let mut updates: HashMap<UserId, Vec<String>> = HashMap::new();

    // TODO: Переделать на Reactive
    // Идем по всем пользователям, у которых включено оповещение о снижении
    let AppContext {
        users_for_push: ref mut users_container,
        db_conn: ref mut conn,
        ..
    } = bot_context.app_context;

    // Долларовый минимум
    let usd_received_minimum = received_bank_currencies
        .iter()
        .min_by(|val1, val2|{
            let v1 = val1.usd.buy;
            let v2 = val2.usd.buy;
            comparator(v1, v2)
        });

    info!("Minimum USD: {:?}\n", usd_received_minimum);

    // Евро минимум
    let eur_received_minimum = received_bank_currencies
        .iter()
        .min_by(|val1, val2|{
            let v1 = val1.eur.buy;
            let v2 = val2.eur.buy;
            comparator(v1, v2)
        });

    info!("Minimum EUR: {:?}\n", usd_received_minimum);

    // Идем по всем пользователям
    for (user_id, user_subscribe_info) in &mut users_container.users_for_push {
        // Список апдейтов для пользователя
        let mut updates_for_user: Vec<String> = vec![];

        // Если у юзера есть предыдущие значения
        let previous_minimum_values: &mut Vec<CurrencyMinimum> = &mut user_subscribe_info.minimum_values;
        
        // Получаем апдейт для долларов
        if let Some(usd_received_minimum) = usd_received_minimum {
            let update = check_minimum_for_value(user_id, 
                                                 conn, 
                                                 CurrencyType::USD, 
                                                 usd_received_minimum, 
                                                 previous_minimum_values).await;
            if let Some(update) = update {
                updates_for_user.push(update);
            }
        }

        // Получаем апдейт для евро
        if let Some(eur_received_minimum) = eur_received_minimum {
            let update = check_minimum_for_value(user_id, 
                                                 conn, 
                                                 CurrencyType::EUR, 
                                                 eur_received_minimum, 
                                                 previous_minimum_values).await;
            if let Some(update) = update {
                updates_for_user.push(update);
            }
        }

        // Если есть что отправлять - отправляем
        if !updates_for_user.is_empty(){
            updates.insert(user_id.clone(), updates_for_user);
        }
    }

    if updates.is_empty() {
        return;
    }

    let futures_iter = updates
        .into_iter()
        .map(|(user_id, updates_for_user)|{
            let text: String = updates_for_user
                .into_iter()
                .collect();
            let mut req = SendMessage::new(user_id, text);
            req.parse_mode(ParseMode::Markdown);
            bot_context.api.send(req)
        });

    futures::future::join_all(futures_iter).await;
}

fn markdown_format_currency_result(info: &CurrencyResult) -> String{
    let time_str: String = match info.update_time {
        Some(time) => {
            let local_time = time.with_timezone(&chrono::offset::Local{});
            local_time.format("%H:%M %d-%m-%Y").to_string()
        },
        None => "No time".into()
    };

    let bank_text = format!(   "*{} ({})*:\n\
                                ```\n$: buy = {:.2} {}, sell = {:.2} {}\n\
                                   €: buy = {:.2} {}, sell = {:.2} {}```\n\n",
            info.bank_name,
            time_str,
            info.usd.buy,
            info.usd.buy_change,
            info.usd.sell,
            info.usd.sell_change,
            info.eur.buy,
            info.eur.buy_change,
            info.eur.sell,
            info.eur.sell_change);

    bank_text
}

fn markdown_format_minimum(new_minimum: &CurrencyMinimum, previous_value: &CurrencyMinimum) -> String{
    let time_str: String = match new_minimum.update_time {
        Some(time) => {
            let local_time = time.with_timezone(&chrono::offset::Local{});
            local_time.format("%H:%M %d-%m-%Y").to_string()
        },
        None => "No time".into()
    };

    let bank_text = format!(   "New buy minimum\n\
                                *{} ({})*:\n\
                                ```\n\
                                {}: new = {:.2}, old = {:.2}```\n\n",
            new_minimum.bank_name,
            time_str,
            new_minimum.cur_type,
            new_minimum.value,
            previous_value.value);

    bank_text
}

fn markdown_format_minimum_for_status(new_minimum: &CurrencyMinimum) -> String{
    let time_str: String = match new_minimum.update_time {
        Some(time) => {
            let local_time = time.with_timezone(&chrono::offset::Local{});
            local_time.format("%H:%M %d-%m-%Y").to_string()
        },
        None => "No time".into()
    };

    let bank_text = format!(   "Buy minimum\n\
                                *{} ({})*:\n\
                                ```\n\
                                {}: {:.2}```\n",
            new_minimum.bank_name,
            time_str,
            new_minimum.cur_type,
            new_minimum.value);

    bank_text
}

pub async fn process_currencies_command(bot_context: &BotContext, message: &Message) -> Result<(), TelegramBotError> {
    let mut text = String::new();

    // Выводим текст, используем into_iter для потребляющего итератора
    for info in get_all_currencies(&bot_context.app_context.client).await {
        if let Ok(info) = info {
            let info: CurrencyResult = info;
            let bank_text = markdown_format_currency_result(&info);
            text.push_str(bank_text.as_str());    
        }
    }

    let mut private_messaage = message.from.text(text);
    bot_context.api.send(private_messaage.parse_mode(ParseMode::Markdown)).await?;

    Ok(())
}

pub async fn process_currencies_status(bot_context: &BotContext, message: &Message) -> Result<(), TelegramBotError> {
    let user_status = bot_context.app_context.users_for_push.try_get_user(&message.from.id);

    let text: String = match user_status{
        Some(status) => {
            if status.minimum_values.is_empty() {
                "Currency check enabled, buy there is no currency minimums".into()
            }else{
                status.minimum_values
                    .iter()
                    .map(|min| {
                        markdown_format_minimum_for_status(min)
                    })
                    .collect()
            }
        },
        None => {
            "Currency check disabled".into()
        }
    };

    let mut private_messaage = message.from.text(text);
    bot_context.api.send(private_messaage.parse_mode(ParseMode::Markdown)).await?;

    Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;
    use currency_lib::{
        CurrencyChange
    };
    use crate::{
        database::{
            build_sqlite_connection
        },
    };

    fn get_usd_val(buy: f32, sell: f32) -> CurrencyValue {
        let usd_val = CurrencyValue{
            cur_type: CurrencyType::USD,
            buy: buy,
            sell: sell,
            buy_change: CurrencyChange::Decrease,
            sell_change: CurrencyChange::Increase,
        };
        usd_val
    }

    fn get_eur_val(buy: f32, sell: f32) -> CurrencyValue {
        let eur_val = CurrencyValue{
            cur_type: CurrencyType::EUR,
            buy: buy,
            sell: sell,
            buy_change: CurrencyChange::Increase,
            sell_change: CurrencyChange::Decrease,
        };
        eur_val
    }

    #[tokio::test]
    async fn test(){
        let time_now = chrono::Utc::now();

        let user_int: i64 = 12345;
        let user = UserId::new(user_int);

        let mut conn = {
            const DB_FILE: &str = "test.db";
            const SQL_FILE: &str = "sqlite:test.db";
            scopeguard::defer!(
                std::fs::remove_file(std::path::Path::new(DB_FILE)).ok();
            );
            let conn = build_sqlite_connection(SQL_FILE).await;
            let mut conn = scopeguard::guard(conn, |conn|{
                conn.close();
            });
            let q = sqlx::query("INSERT INTO monitoring_users(user_id) VALUES (?)")
                .bind(user_int);
            conn.execute(q)
                .await
                .expect("Insert test user error");
            
            conn
        };

        let received_minimum_alpha = CurrencyResult{
            bank_name: "Alpha".to_string(),
            usd: get_usd_val(74.20, 73.1),
            eur: get_eur_val(83.60, 80.50),
            update_time: Some(time_now)
        };
        let mut previous_minimums = vec![
        ];

        {
            let cur_type = CurrencyType::USD;

            let usd_result: Option<String> = check_minimum_for_value(&user, 
                                                                    &mut conn, 
                                                                    cur_type, 
                                                                    &received_minimum_alpha, 
                                                                    &mut previous_minimums).await;

            assert_eq!(previous_minimums.len(), 1);
            assert_eq!(usd_result.is_some(), true);

            let saved_minimum: &CurrencyMinimum = previous_minimums.get(0).expect("First elemement doesn't exist");
            assert_eq!(saved_minimum.cur_type, cur_type);
            assert_eq!(saved_minimum.value, received_minimum_alpha.usd.buy);
            assert_eq!(saved_minimum.bank_name, received_minimum_alpha.bank_name);
        }

        {
            let cur_type = CurrencyType::EUR;

            let eur_result: Option<String> = check_minimum_for_value(&user, 
                                                                    &mut conn, 
                                                                    cur_type, 
                                                                    &received_minimum_alpha, 
                                                                    &mut previous_minimums).await;

            assert_eq!(previous_minimums.len(), 2);
            assert_eq!(eur_result.is_some(), true);

            let saved_minimum: &CurrencyMinimum = previous_minimums.get(1).expect("Second elemement doesn't exist");
            assert_eq!(saved_minimum.cur_type, cur_type);
            assert_eq!(saved_minimum.value, received_minimum_alpha.eur.buy);
            assert_eq!(saved_minimum.bank_name, received_minimum_alpha.bank_name); 
        }
    }
}