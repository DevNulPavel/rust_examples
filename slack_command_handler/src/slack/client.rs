// use log::{
    // error
// };
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
    client: Client
}

impl SlackClient {
    pub fn new(token: &str) -> SlackClient {
        let client = Client::builder()
            .bearer_auth(token)
            .header("Content-type", "application/json")
            .finish();

        SlackClient{
            client
        }
    }

    pub async fn open_view<'a>(&'a self, window_json: Value) -> Result<View<'a>, SlackViewError>{
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
                Ok(View::new(&self.client, view))
            },
            ViewOpenResponse::Error(err) => {
                Err(SlackViewError::OpenError(err))
            }
        }
    }
}