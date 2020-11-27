use actix_web::{
    client::{
        Client
    }
};
use serde_json::{
    Value
};
use super::{
    view_open_response::{
        ViewInfo
    },
    error::{
        SlackViewError
    },
    view_open_response::{
        ViewUpdateResponse
    }
};

pub struct View {
    client: Client,
    info: ViewInfo
}

impl View {
    pub fn new(client: Client, info: ViewInfo) -> View{
        View{
            client,
            info
        }
    }

    pub async fn update_view(self, view_json: Value) -> Result<View, SlackViewError>{
        // TODO: Снизить область видимости
        let window = serde_json::json!({
            "view_id": self.info.id,
            "hash": self.info.hash,
            "view": view_json
        });

        let response = self.client
            .post("https://slack.com/api/views.update")
            .send_body(serde_json::to_string(&window).unwrap())
            .await?
            .json::<ViewUpdateResponse>()
            .await?;

        match response {
            ViewUpdateResponse::Ok{view} => {
                Ok(View::new(self.client, view))
            },
            ViewUpdateResponse::Error(err) => {
                Err(SlackViewError::UpdateError(err))
            }
        }
    }
}