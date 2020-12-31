use std::{
    error::{
        Error
    },
    path::{
        PathBuf
    },
    sync::{
        Arc
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
    sync::{
        Mutex
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
    SlackError,
    SlackMessageTarget,
    SlackChannelMessageTarget,
    SlackUserMessageTarget,
    SlackThreadMessageTarget,
    SlackThreadImageTarget,
    UsersCache,
    UsersJsonCache,
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
    qr::{
        create_qr_data,
        QrCodeError
    },
    ResultSender,
};

fn qr_future_for_result(install_url: Option<String>) -> impl Future<Output=Option<QRInfo>> {
    let qr_data_future = match install_url{
        Some(url) => {
            let fut = async move {
                let res: Option<QRInfo> = spawn_blocking(||{ create_qr_data("This is test text").ok() })
                    .await
                    .expect("QR code create spawn failed")
                    .map(|qr_data|{
                        let inner = Arc::new(QRInfoInner{
                            url: url,
                            qr_data
                        });
                        QRInfo{
                            inner
                        }
                    });
                res
            };
            fut.shared().boxed()
        }
        None => {
            futures::future::ready(Option::None).shared().boxed()
        }
    };
    qr_data_future
}

// QR код
struct QRInfoInner{
    url: String,
    qr_data: Vec<u8>
}
#[derive(Clone)]
struct QRInfo{
    inner: Arc<QRInfoInner>,
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
            if let Some(message) = &result.message{
                strings.push(Cow::from(message));
            }
            if let Some(download_url) = &result.download_url{
                let text = format!("Download url: ```{}```", download_url);
                strings.push(Cow::from(text));
            }
            if strings.len() > 0 {
                Some(strings.join("\n\n"))
            }else{
                None
            }
        };

        // Создаем футуру с результатом QR
        let qr_data_future = qr_future_for_result(result.install_url.clone());

        let message_futures_vec = {
            // Массив наших тасков
            let mut futures_vec = Vec::new();

            // Сообщение
            if let Some(message) = &text {
                if let Some(channel) = &sender.channel {
                    let fut = async move {
                        let target = SlackChannelMessageTarget::new(&channel);
                        let (message_result, qr) = join!(
                            async{
                                sender
                                    .client
                                    .send_message(&message, target)
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
                                        qr.inner.qr_data.clone(), 
                                        qr.inner.url.clone(), 
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
                    };

                    futures_vec.push(fut.boxed());
                }
                
                if let Some(user_id) = &sender.user_id {
                    // let target = SlackMessageTarget::to_user_direct(&user_id);
                    // let fut = sender.client.send_message(&message, target);
                    // futures_vec.push(fut);
                }
            }

            futures_vec
        };

        // TODO: QR + Ссылка

        let messages = join_all(message_futures_vec).await;
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