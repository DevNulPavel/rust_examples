// use log::{
    // error
// };
use serde::{
    Deserialize
};
use serde_json::{
    Value
};
use reqwest::{
    Client
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
    client: Client,
    token: String
}

impl SlackClient {
    pub fn new(client: Client, token: &str) -> SlackClient {
        SlackClient{
            client: client,
            token: token.to_owned()
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
            .bearer_auth(&self.token)
            .header("Content-type", "application/json")
            .body(serde_json::to_string(&window_json).unwrap())
            .send()
            .await
            .map_err(|err|{
                SlackViewError::RequestErr(err)
            })?
            .json::<ViewOpenResponse>()
            .await
            .map_err(|err|{
                SlackViewError::JsonParseError(err)
            })?;

        match response {
            ViewOpenResponse::Ok{view} => {
                Ok(View::new(self.client.clone(), &self.token, view))
            },
            ViewOpenResponse::Error(err) => {
                Err(SlackViewError::OpenError(err))
            }
        }
    }
}