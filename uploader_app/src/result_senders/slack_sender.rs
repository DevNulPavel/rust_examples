use std::{
    error::{
        Error
    },
    path::{
        PathBuf
    }
};
use reqwest::{
    Client
};
use tokio::{
    task::{
        JoinHandle,
        // spawn_local
    },
    spawn,
};
use futures::{
    future::{
        join_all,
        select,
        Either,
        Future,
        FutureExt
    },
    // select
};
use async_trait::{
    async_trait
};
use slack_client_lib::{
    SlackClient,
    SlackError,
    SlackMessageTarget,
    UsersCache,
    UsersJsonCache
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
    text_prefix: Option<String>,
    channel: Option<String>,
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
            let client_ref = &client;

            let email_future = params
                .user_email
                .as_ref()
                .map(|email|{
                    client_ref.find_user_id_by_email(&email)
                })
                .map(|fut|{
                    fut.boxed()
                });

            let name_future = params
                .user_name
                .as_ref()
                .map(|name| async move {
                    let cache_file_path = PathBuf::new()
                        .join(dirs::home_dir().unwrap())
                        .join(".cache/uploader_app/users_cache.json");

                    let mut cache = UsersJsonCache::new(cache_file_path).await;

                    client_ref.find_user_id_by_name(&name, Some(&mut cache)).await
                })
                .map(|fut|{
                    fut.boxed()
                });

            let user_id: Option<String> = match (email_future, name_future){
                (Some(email_future), Some(name_future)) => {
                    let id: Option<String> = match select(name_future, email_future).await {
                        Either::Left((id, _)) => id,
                        Either::Right((id, _)) => id,
                    };
                    id
                },
                (None, Some(name_future)) => {
                    name_future.await
                },
                (Some(email_future), None) => {
                    email_future.await
                },
                (None, None) => {
                    None
                }
            };

            SenderResolved{
                client,
                text_prefix: params.text_prefix,
                channel: params.channel,
                user_id
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
        let sender = self.resolve_sender().await;

        let mut futures_vec = Vec::new();

        if let Some(message) = &result.message{
            if let Some(channel) = &sender.channel{
                let target = SlackMessageTarget::to_channel(&channel);
                let fut = sender.client.send_message(&message, target);
                futures_vec.push(fut);
            }
            
            if let Some(user_id) = &sender.user_id {
                let target = SlackMessageTarget::to_user_direct(&user_id);
                let fut = sender.client.send_message(&message, target);
                futures_vec.push(fut);
            }
        }

        join_all(futures_vec).await;
    }
    async fn send_error(&mut self, err: &dyn Error){
        //error!("Uploading task error: {}", err);
    }
}