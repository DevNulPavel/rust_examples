// use log::{
    // error
// };
use actix_web::{
    client::{
        Client
    }
};
use serde_json::{
    Value
};
use super::{
    error::{
        SlackViewError
    },
    view_open_response::{
        ViewOpenResponse,
        // ViewUpdateResponse,
        // ViewInfo
    },
    view::{
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

    pub async fn open_view<'a>(&'a self, window_json: Value) -> Result<View, SlackViewError>{
        let response = self.client
            .post("https://slack.com/api/views.open")
            .send_body(serde_json::to_string(&window_json).unwrap())
            .await?
            .json::<ViewOpenResponse>()
            .await?;

        match response {
            ViewOpenResponse::Ok{view} => {
                Ok(View::new(self.client.clone(), view))
            },
            ViewOpenResponse::Error(err) => {
                Err(SlackViewError::OpenError(err))
            }
        }
    }
}