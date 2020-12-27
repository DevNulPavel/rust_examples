use slack_client_lib::{
    View,
    ViewInfo
};
use crate::{
    session::{
        WindowSession
    }
};


// #[async_trait]
pub trait ViewActionHandler: Send {
    fn update_info(&mut self, new_info: ViewInfo);
    fn get_view(&self) -> &View;
    fn on_submit(self: Box<Self>, session: WindowSession);
    fn on_update(&self);
    fn on_close(self: Box<Self>, session: WindowSession);
}