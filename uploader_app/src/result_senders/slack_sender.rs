use std::{
    error::{
        Error
    }
};
use reqwest::{
    Client
};
use tokio::{
    task::{
        JoinHandle
    },
    spawn
};
use futures::{
    future::{
        join_all,
        Future,
        FutureExt
    }
};
use async_trait::{
    async_trait
};
use slack_client_lib::{
    SlackClient,
    SlackError,
    SlackMessageTarget
};
use crate::{
    uploaders::{
        UploadResultData
    },
    env_parameters::{
        ResultSlackEnvironment
    }
};
use super::{
    ResultSender
};

struct SenderResolved{
    client: SlackClient,
    params: ResultSlackEnvironment,
    user_id: Option<String>
}

enum ResultSenderState{
    Pending(JoinHandle<SenderResolved>),
    Resolved(SenderResolved)
}

pub struct SlackResultSender{
    inner: ResultSenderState
}
impl SlackResultSender {
    pub fn new(http_client: Client, params: ResultSlackEnvironment) -> SlackResultSender{
        let join = spawn(async move{
            let client = SlackClient::new(http_client, params.token.clone()); // TODO: Убрать клонирование

            /**/

            let email_future = params
                .user_email
                .as_ref()
                .map(|email|{
                    client.find_user_id_by_email(&email)
                });

            let name_future = params
                .user_name
                .as_ref()
                .map(|name|{
                    let cache_file_path = PathBuf::new()
                        .join(dirs::home_dir().unwrap())
                        .join(".cache/uploader_app/users_cache.json");

                    client.find_user_id_by_name(&email)
                });

            /*if params.user_email.is_some() || params.user_name.is_some(){
                //let user_email = self.params.user_email.unwrap_or_default();
                // let user_name = self.params.user_name.unwrap_or_default();
                self.client.find_user_id(&user_email, &user_name, cache_file_path).await;
            }*/
            SenderResolved{
                client,
                params,
                user_id: None
            }
        });
        SlackResultSender{
            inner: ResultSenderState::Pending(join)
        }
    }
    async fn resolve_sender(&mut self) -> &SenderResolved {
        let sender = loop {
            match self.inner {
                ResultSenderState::Pending(ref mut join) => {
                    let resolved = join.await.expect("Slack sender resolve failed");
                    self.inner = ResultSenderState::Resolved(resolved);
                },
                ResultSenderState::Resolved(ref sender) => {
                    break sender;
                }
            }
        };
        sender
    }
}
#[async_trait(?Send)]
impl ResultSender for SlackResultSender {
    async fn send_result(&mut self, result: &UploadResultData){
        let sender = self.resolve_sender();

        /*let mut futures_vec = Vec::new();

        if let Some(message) = &result.message{
            if let Some(channel) = &self.params.channel{
                let target = SlackMessageTarget::to_channel(&channel);
                let fut = self.client.send_message(&message, target);
                futures_vec.push(fut);
            }
            
            if self.params.user_email.is_some() || self.params.user_name.is_some(){
                //let user_email = self.params.user_email.unwrap_or_default();
                // let user_name = self.params.user_name.unwrap_or_default();
                let user_id = self.client.find_user_id(&user_email, &user_name, cache_file_path).await;

                if let Some(user_id) = user_id {
                    let target = SlackMessageTarget::to_user_direct(&user_id);
                    let fut = self.client.send_message(&message, target);
                    futures_vec.push(fut);
                }
            }
        }*/
    }
    async fn send_error(&mut self, err: &dyn Error){
        //error!("Uploading task error: {}", err);
    }
}