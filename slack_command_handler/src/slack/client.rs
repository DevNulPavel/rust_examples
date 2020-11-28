// use log::{
    // error
// };
use std::{
    collections::{
        HashMap
    },
    sync::{
        Mutex,
        Arc
    }
};
use actix_web::{
    client::{
        Client
    }
};
use serde::{
    Deserialize
};
use serde_json::{
    Value
};
use super::{
    error::{
        SlackViewError,
        ViewOpenErrorInfo
    },
    // view_open_response::{
        // ViewOpenResponse,
        // ViewUpdateResponse,
        // ViewInfo
    // },
    view::{
        ViewInfo,
        View
    }
};

pub struct SlackClient{
    token: String,
    client: Client
}

impl SlackClient {
    fn new_http_client(token: &str) -> Client {
        Client::builder()
            .bearer_auth(token)
            .header("Content-type", "application/json")
            .finish()
    }

    pub fn new(token: &str) -> SlackClient {
        SlackClient{
            token: token.to_owned(),
            client: SlackClient::new_http_client(token)
        }
    }

    pub async fn open_view(&self, window_json: Value) -> Result<View, SlackViewError>{
        // https://serde.rs/enum-representations.html
        // https://api.slack.com/methods/views.open#response
        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        pub enum ViewOpenResponse{
            Ok{ view: ViewInfo },
            Error(ViewOpenErrorInfo)
        }

        let response = self.client
            .post("https://slack.com/api/views.open")
            .send_body(serde_json::to_string(&window_json).unwrap())
            .await?
            .json::<ViewOpenResponse>()
            .await?;

        match response {
            ViewOpenResponse::Ok{view} => {
                Ok(View::new(&self.token, view))
            },
            ViewOpenResponse::Error(err) => {
                Err(SlackViewError::OpenError(err))
            }
        }
    }
}