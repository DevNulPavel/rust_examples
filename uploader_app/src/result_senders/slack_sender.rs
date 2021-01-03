use std::{
    error::{
        Error
    },
    path::{
        PathBuf
    },
    borrow::{
        Cow
    },
    pin::{
        Pin
    }
};
use log::{
    error
};
use reqwest::{
    Client
};
use tokio::{
    task::{
        JoinHandle,
        spawn_blocking
        // spawn_local
    },
    join,
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
    SlackChannelMessageTarget,
    SlackUserMessageTarget,
    SlackThreadImageTarget,
    SlackUserImageTarget,
    SlackChannelImageTarget,
    // UsersCache,
    // UsersJsonCache,
    UsersSqliteCache
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
    ResultSender, 
    qr::{
        create_qr_data
    }
};

fn qr_future_for_result(install_url: Option<String>) -> Pin<Box<dyn Future<Output=Option<QRInfo>> + Send>>{
    let qr_data_future = match install_url{
        Some(url) => {
            let fut = async move {
                let qr_data = url.clone();
                let res: Option<QRInfo> = spawn_blocking(move || { create_qr_data(&qr_data).ok() })
                    .await
                    .expect("QR code create spawn failed")
                    .map(|qr_data|{
                        /*let inner = Arc::new(QRInfoInner{
                            url: url,
                            qr_data
                        });
                        QRInfo{
                            inner
                        }*/
                        QRInfo{
                            url: url,
                            qr_data
                        }
                    });
                res
            };
            fut.boxed()
        }
        None => {
            futures::future::ready(Option::None).boxed()
        }
    };
    qr_data_future
}

macro_rules! message_target_impl {
    ($fn_name:ident, $target_type:ident$(<$life:lifetime>)?) => {
        async fn $fn_name <'a, Q: Future<Output=Option<QRInfo>>>(sender: &SenderResolved, 
                                                              qr_data_future: Q, 
                                                              target: $target_type$(<$life>)?, 
                                                              text: &str) {
        
            let (message_result, qr) = join!(
                async{
                    sender
                        .client
                        .send_message(text, target)
                        .await
                        .ok()
                        .flatten()
                },
                qr_data_future
            );
        
            match (message_result, qr) {
                (Some(message), Some(qr)) => {
                    let target = SlackThreadImageTarget::new(
                        message.get_channel_id(),
                        message.get_thread_id()
                    );
                    let image_res = sender
                        .client
                        .send_image(
                            qr.qr_data, 
                            None, 
                            target
                        )
                        .await;
            
                    if let Err(err) = image_res {
                        error!("Slack image uploading failed with err: {}", err);    
                    }
                },
                _ => {
                    error!("There is no slack message created or QR create error");
                }
            }
        }  
    };
}
message_target_impl!(message_to_channel_target, SlackChannelMessageTarget<'a>);
message_target_impl!(message_to_user_target, SlackUserMessageTarget<'a>);



// QR код
/*struct QRInfoInner{
    url: String,
    qr_data: Vec<u8>
}
#[derive(Clone)]
struct QRInfo{
    inner: Arc<QRInfoInner>,
}*/
#[derive(Clone)]
struct QRInfo{
    url: String,
    qr_data: Vec<u8>
}

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
                    // Json cache
                    /*let cache_file_path = PathBuf::new()
                        .join(dirs::home_dir().unwrap())
                        .join(".cache/uploader_app/users_cache.json");
                    let cache = UsersJsonCache::new(cache_file_path).await;*/

                    // Sqlite cache
                    let cache_file_path = PathBuf::new()
                        .join(dirs::home_dir().unwrap())
                        .join(".cache/uploader_app/users_cache.sqlite");
                    let cache: Option<UsersSqliteCache> = UsersSqliteCache::new(cache_file_path)
                        .await
                        .ok();

                    // TODO: Как-то сконвертировать в тип сразу?
                    match cache {
                        Some(cache) => {
                            client_ref.find_user_id_by_name(&name, Some(&cache)).await
                        },
                        None => {
                            client_ref.find_user_id_by_name(&name, None).await
                        }
                    }
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

        // Собираем текст в кучу
        let text = {
            let mut strings = Vec::new();
            if let Some(prefix) = &sender.text_prefix {
                strings.push(Cow::from(prefix));
            }
            if let Some(message) = &result.message {
                let text = format!("```{}```", message);
                strings.push(Cow::from(text));
            }
            if strings.len() > 0 {
                Some(strings.join("\n"))
            }else{
                None
            }
        };

        // Создаем футуру с результатом QR
        let qr_data_future = qr_future_for_result(result.install_url.clone());

        // Сообщение
        if let Some(message) = &text {
            // Массив наших тасков
            let mut futures_vec = Vec::new();

            let qr_data_future = qr_data_future.shared();

            // В канал
            if let Some(channel) = &sender.channel {
                let qr_data_future = qr_data_future.clone();
                let fut = async move {
                    let target = SlackChannelMessageTarget::new(&channel);
                    message_to_channel_target(sender, qr_data_future, target, message)
                        .await;
                };

                futures_vec.push(fut.boxed());
            }
            
            // Юзеру
            if let Some(user_id) = &sender.user_id {
                let fut = async move {
                    let target = SlackUserMessageTarget::new(&user_id);
                    message_to_user_target(sender, qr_data_future, target, message)
                        .await;
                };

                futures_vec.push(fut.boxed());
            }

            join_all(futures_vec).await;
        }else {
            // Массив наших тасков
            let mut futures_vec = Vec::new();

            // Просто засылаем QR код если нету сообщения
            if let Some(qr_info) = qr_data_future.await{
                // let QRInfo { url, qr_data } = qr_info.deref();

                // В канал
                if let Some(channel) = &sender.channel {
                    let target = SlackChannelImageTarget::new(&channel);
                    let fut = sender.client.send_image(qr_info.qr_data.clone(), None, target);
                    futures_vec.push(fut.boxed());
                }
                
                // Юзеру
                if let Some(user_id) = &sender.user_id {
                    let target = SlackUserImageTarget::new(&user_id);
                    let fut = sender.client.send_image(qr_info.qr_data, None, target);
                    futures_vec.push(fut.boxed());
                }

                join_all(futures_vec).await;
            }
        }

    }

    async fn send_error(&mut self, err: &dyn Error){
        let sender = self.resolve_sender().await;

        let message = format!("Uploading error:```{}```", err);

        let futures_vec = {
            let mut futures_vec = Vec::new();

            // Пишем в канал
            if let Some(channel) = &sender.channel{
                let target = SlackChannelMessageTarget::new(&channel);
                let fut = sender.client.send_message(&message, target).boxed();
                futures_vec.push(fut);
            }
            
            // Пишем пользователю
            if let Some(user_id) = &sender.user_id {
                let target = SlackUserMessageTarget::new(&user_id);
                let fut = sender.client.send_message(&message, target).boxed();
                futures_vec.push(fut);
            }

            futures_vec
        };
        
        join_all(futures_vec).await;
    }
}