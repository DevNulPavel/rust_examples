use std::{
    sync::{
        Arc
    }
};
use reqwest::{
    Client,
    RequestBuilder
};

struct RequestBuilderInternal{
    client: Client,
    token: String
}

pub struct SlackRequestBuilder{
    internal: Arc<RequestBuilderInternal>
}

impl Clone for SlackRequestBuilder {
    fn clone(&self) -> Self {
        SlackRequestBuilder{
            internal: self.internal.clone()
        }
    }
}

impl SlackRequestBuilder {
    pub fn new(client: Client, token: String) -> SlackRequestBuilder {
        let internal = Arc::new(RequestBuilderInternal{
            client,
            token
        });

        SlackRequestBuilder{
            internal
        }
    }


    pub fn build_get_request(&self, url: &str) -> RequestBuilder {
        let RequestBuilderInternal{
            client: client_alt_name,
            token
        } = self.internal.as_ref();

        client_alt_name
            .get(url)
            .bearer_auth(token)
    }

    pub fn build_post_request(&self, url: &str) -> RequestBuilder {
        let RequestBuilderInternal{
            client,
            token
        } = self.internal.as_ref();

        client
            .post(url)
            .bearer_auth(token)
    }
}