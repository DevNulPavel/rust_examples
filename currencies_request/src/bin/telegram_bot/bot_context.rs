use telegram_bot::{
    Api,
};
use crate::{
    app_context::{
        AppContext
    },
};

pub struct BotContext{
    pub app_context: AppContext,
    pub api: Api,
}

impl BotContext {
    pub fn new(app_context: AppContext, api: Api) -> Self{
        BotContext{
            app_context,
            api
        }
    }
}

impl Into<AppContext> for BotContext {
    fn into(self) -> AppContext {
        self.app_context
    }
}