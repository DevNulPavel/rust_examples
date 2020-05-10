use telegram_bot::{
    Api,
};
// use reqwest::{
    //Client
// };
use crate::{
    // currency::{
    //     CurrencyCheckStatus
    // },
    app_context::{
        AppContext
    },
};

pub struct BotContext{
    pub app_context: AppContext,
    pub api: Api,
    //pub currency_check_status: Option<CurrencyCheckStatus>
}

impl BotContext {
    pub fn new(app_context: AppContext, api: Api) -> Self{
        BotContext{
            app_context,
            api,
            // currency_check_status: None
        }
    }

    /*pub fn get_client(&self) -> &Client{
        &self.app_context.client
    }
    pub fn get_users(&self) -> &CurrencyUsersStorrage{
        &self.app_context.users_for_push
    }*/
}

impl Into<AppContext> for BotContext{
    fn into(self) -> AppContext {
        self.app_context
    }
}

impl std::ops::Deref for BotContext{
    type Target = AppContext;

    fn deref(&self) -> &Self::Target {
        &self.app_context
    }
}

impl std::ops::DerefMut for BotContext{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.app_context
    }
}