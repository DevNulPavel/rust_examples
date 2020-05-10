
use std::{
    // collections::HashSet,
    collections::HashMap
    //pin::Pin,
    //future::Future
};
// use chrono::{
    // Utc,
    // DateTime
// };
use telegram_bot::{
    // Api,
    Error,
    Message,
    CanSendMessage,
    ParseMode,
    UserId
};
// use reqwest::{
    //Client,
// };
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
    users_for_push: HashMap<UserId, CurrencyCheckStatus>
}

impl CurrencyUsersStorrage{
    pub fn new() -> Self{
        CurrencyUsersStorrage{
            users_for_push: HashMap::new()
        }
    }

    pub fn add_user(&mut self, user: &UserId){
        let check = CurrencyCheckStatus::new(user);
        self.users_for_push.insert(user.clone(), check);
    }

    pub fn remove_user(&mut self, user: &UserId){
        self.users_for_push.remove(user);
    }

    // TODO: Mut iter
    /*fn iter_users(&self) -> std::collections::hash_map::Iter<UserId, CurrencyCheckStatus> {
        self.users_for_push.iter()
    }*/

    pub fn is_empty(&self) -> bool{
        self.users_for_push.is_empty()
    }
}

///////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct CurrencyCheckStatus {
    user: UserId,
    minimum_values: Option<Vec<CurrencyResult<'static>>>,
    //minimum_time: DateTime<Utc>,        // TODO:
    //last_check_time: DateTime<Utc>,     // TODO:
}

impl CurrencyCheckStatus{
    pub fn new(user: &UserId) -> Self{
        CurrencyCheckStatus{
            user: user.clone(),
            minimum_values: None
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

///////////////////////////////////////////////////////////////////////////////////////


pub async fn check_currencies_update(bot_context: &mut BotContext) {
    // Если некому слать - выходим
    if bot_context.app_context.users_for_push.is_empty(){
        return;
    }

    // Получаем новые значения
    let received_bank_currencies: Vec<CurrencyResult<'static>> = 
        get_all_currencies(&bot_context.client).await
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
    for (user_id, user_subscribe_info) in &mut bot_context.app_context.users_for_push.users_for_push {
        let mut updates_for_user: Vec<String> = vec![];

        // Если у юзера есть предыдущие значения
        if let Some(previous_minimum_values) = &mut user_subscribe_info.minimum_values {
            // Перебираем информацию о каждом банке, которую получили
            for received_bank_info in &received_bank_currencies {
                // Ищем предыдущее значение для банка у юзера
                let previous_value_for_bank = previous_minimum_values
                    .iter_mut()
                    .find(|value|{
                        value.bank_name.eq(received_bank_info.bank_name)
                    });
                
                // Если есть предыдущее значение
                if let Some(previous_value_for_bank) = previous_value_for_bank{
                    // Проверяем минимум
                    let usd_lower = previous_value_for_bank.usd.buy > received_bank_info.usd.buy;
                    let eur_lower = previous_value_for_bank.eur.buy > received_bank_info.eur.buy;

                    // TODO: Test
                    if usd_lower || eur_lower{
                        // Обновляем значение
                        *previous_value_for_bank = received_bank_info.clone();

                        updates_for_user.push(markdown_format_currency_result(previous_value_for_bank));
                    }
                }
            }
        }else{
            // Иначе просто сохраняем значения для банков, которые мы получили
            updates_for_user.reserve(received_bank_currencies.len());
            
            let str_iter = received_bank_currencies
                .iter()
                .map(|info|{
                    markdown_format_currency_result(info)
                });

            for str_val in str_iter{
                updates_for_user.push(str_val);
            };

            user_subscribe_info.minimum_values = Some(received_bank_currencies.clone());
        }

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
            let mut req = telegram_bot::types::requests::SendMessage::new(user_id, text);
            req.parse_mode(ParseMode::Markdown);
            bot_context.api.send(req)
        });

    futures::future::join_all(futures_iter).await;
}

fn markdown_format_currency_result(info: &CurrencyResult) -> String{
    let time_str: String = match info.update_time {
        Some(time) => time.format("%H:%M %d-%m-%Y").to_string(),
        None => "No time".into()
    };

    let bank_text = format!(   "*{} ({})*:\n\
                                ```\n$: buy = {:.2} {}, sell = {:.2} {}\n\
                                   €: buy = {:.2} {}, sell = {:.2} {}```\n",
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

pub async fn process_currencies_command(bot_context: &BotContext, message: &Message) -> Result<(), Error> {
    let mut text = String::new();

    // Выводим текст, используем into_iter для потребляющего итератора
    for info in get_all_currencies(&bot_context.client).await {
        let info: Result<CurrencyResult, CurrencyError> = info;
        match info {
            Ok(info) =>{
                let info: CurrencyResult = info;

                let bank_text = markdown_format_currency_result(&info);
                
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

    let mut private_messaage = message.from.text(text);
    bot_context.api.send(private_messaage.parse_mode(ParseMode::Markdown)).await?;

    Ok(())
}