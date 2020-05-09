
use std::{
    collections::HashSet
    //collections::HashMap
    //pin::Pin,
    //future::Future
};
use chrono::{
    // Utc,
    // DateTime
};
use telegram_bot::{
    // Api,
    Error,
    Message,
    CanSendMessage,
};
use reqwest::{
    Client,
};
use currencies_request::{
    CurrencyError,
    CurrencyResult,
    //CurrencyChange,
    get_all_currencies
};
use crate::{
    bot_context::{
        BotContext
    }
};


pub struct CurrencyUsersStorrage{
    users_for_push: HashSet<telegram_bot::UserId>
}

impl CurrencyUsersStorrage{
    pub fn new() -> Self{
        CurrencyUsersStorrage{
            users_for_push: HashSet::new()
        }
    }

    pub fn add_user(&mut self, user: &telegram_bot::UserId){
        self.users_for_push.insert(user.clone());
    }

    pub fn remove_user(&mut self, user: &telegram_bot::UserId){
        self.users_for_push.remove(user);
    }

    fn iter_users(&self) -> std::collections::hash_set::Iter<telegram_bot::UserId> {
        self.users_for_push.iter()
    }

    pub fn is_empty(&self) -> bool{
        self.users_for_push.is_empty()
    }
}

///////////////////////////////////////////////////////////////////////////////////////

pub struct CurrencyCheckStatus {
    minimum_values: Vec<CurrencyResult<'static>>,
    //minimum_time: DateTime<Utc>,        // TODO:
    //last_check_time: DateTime<Utc>,     // TODO:
}

impl CurrencyCheckStatus{
}

///////////////////////////////////////////////////////////////////////////////////////


pub async fn check_currencies_update(bot_context: &mut BotContext) {
    if bot_context.app_context.users_for_push.is_empty(){
        return;
    }

    let received: Vec<CurrencyResult<'static>> = 
        get_all_currencies(&bot_context.client).await
        .into_iter()
        .filter_map(|result|{
            result.ok()
        })
        .collect();

    if received.is_empty(){
        return;
    }

    let mut updates: Vec<String> = vec![];

    if let Some(ref mut status) = bot_context.currency_check_status {
        for received_info in received {
            let previous = status
                .minimum_values
                .iter_mut()
                .find(|value|{
                    value.bank_name.eq(received_info.bank_name)
                });
            if let Some(prev) = previous{
                //let prev: & mut CurrencyResult = prev;

                let usd_lower = prev.usd.buy > received_info.usd.buy;
                let eur_lower = prev.eur.buy > received_info.eur.buy;

                // TODO: Test
                if usd_lower || eur_lower{
                    // Обновляем значение
                    //prev.clone_from(&received_info);
                    *prev = received_info;

                    // Обновилось - можно сообщить
                    updates.push(format!("{:?}\n", prev));
                }
            }
        }
    }else{
        for info in received.iter() {
            updates.push(format!("{:?}\n", info));
        }
        bot_context.currency_check_status = Some(CurrencyCheckStatus{
            minimum_values: received,
        });
        //bot_context.currency_check_status.unwrap().minimum_values;
            
        // let iter = bot_context
        //     .currency_check_status
        //     .unwrap()
        //     .minimum_values;
        // let iter = bot_context
        //     .currency_check_status
        //     .unwrap()
        //     .minimum_values
        //     .iter();
        // for info in iter {
        //     updates.push(info);
        // }
    }

    //println!("{:?}", api.send(telegram_bot::types::requests::GetMe).await);
    //println!("{:?}", api.send(telegram_bot::types::requests::GetChat::new()).await);
    //println!("{:?}", api.send(telegram_bot::types::requests::GetChatMember::n).await);
    //println!("{:?}", api.send(telegram_bot::types::requests::Get).await);

    if updates.is_empty() {
        return;
    }

    // Создает переменную и ссылку на нее с помощью ref
    let ref text: String = updates
        .into_iter()
        .collect();

    for user in bot_context.app_context.users_for_push.iter_users() {
        //let chat = telegram_bot::MessageChat::Private(message.from.clone());
        //let user = telegram_bot::UserId::new(871805190);
        bot_context.api.send(telegram_bot::types::requests::SendMessage::new(user, text))
            .await
            .ok();
    }
}

pub async fn process_currencies_command(bot_context: &BotContext, message: &Message) -> Result<(), Error> {
    let mut text = String::new();

    // Выводим текст, используем into_iter для потребляющего итератора
    for info in get_all_currencies(&bot_context.client).await {
        let info: Result<CurrencyResult, CurrencyError> = info;
        match info {
            Ok(info) =>{
                let info: CurrencyResult = info;

                let time_str: String = match info.update_time {
                    Some(time) => time.format("%H:%M:%S %Y-%m-%d").to_string(),
                    None => "No time".into()
                };

                let bank_text = format!("{} ({}):\nUSD: buy = {}, sell = {}\nEUR: buy = {}, sell = {}\n\n",
                        info.bank_name,
                        time_str,
                        info.usd.buy,
                        info.usd.sell,
                        info.eur.buy,
                        info.eur.sell
                    );
                
                text.push_str(bank_text.as_str())
            },
            Err(_e) => {
                // TODO: Вывод ошибок
                /*let row = Row::new(vec![
                    Cell::new(format!("{:?}", e).as_str()),
                ]);
                table.add_row(row);*/
                println!("{:?}", _e);
            }
        }
    }

    let private_messaage = message.from.text(text);
    bot_context.api.send(private_messaage).await?;

    Ok(())
}