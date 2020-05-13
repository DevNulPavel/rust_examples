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

/*impl std::ops::Deref for BotContext{
    type Target = AppContext;

    fn deref(&self) -> &Self::Target {
        &self.app_context
    }
}

impl std::ops::DerefMut for BotContext{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.app_context
    }
}*/