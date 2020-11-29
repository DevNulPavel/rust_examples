// use log::{
    // error
// };
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
    
    token: String
}

impl SlackClient {
    pub fn new(token: &str) -> SlackClient {
        SlackClient{
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

        let client = Client::builder()
            .bearer_auth(&self.token)
            .header("Content-type", "application/json")
            .finish();
    
        let response = client
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