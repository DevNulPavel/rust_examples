use crate::error::IggyError;
use crate::models::identity_info::IdentityInfo;
use async_trait::async_trait;
use reqwest::{Response, Url};
use serde::Serialize;

#[allow(deprecated)]
pub mod client;
pub mod config;
pub mod consumer_groups;
pub mod consumer_offsets;
pub mod messages;
pub mod partitions;
pub mod personal_access_tokens;
pub mod streams;
pub mod system;
pub mod topics;
pub mod users;

#[async_trait]
pub trait HttpTransport {
    /// Get full URL for the provided path.
    fn get_url(&self, path: &str) -> Result<Url, IggyError>;

    /// Invoke HTTP GET request to the Iggy API.
    async fn get(&self, path: &str) -> Result<Response, IggyError>;

    /// Invoke HTTP GET request to the Iggy API with query parameters.
    async fn get_with_query<T: Serialize + Sync + ?Sized>(
        &self,
        path: &str,
        query: &T,
    ) -> Result<Response, IggyError>;

    /// Invoke HTTP POST request to the Iggy API.
    async fn post<T: Serialize + Sync + ?Sized>(
        &self,
        path: &str,
        payload: &T,
    ) -> Result<Response, IggyError>;

    /// Invoke HTTP PUT request to the Iggy API.
    async fn put<T: Serialize + Sync + ?Sized>(
        &self,
        path: &str,
        payload: &T,
    ) -> Result<Response, IggyError>;

    /// Invoke HTTP DELETE request to the Iggy API.
    async fn delete(&self, path: &str) -> Result<Response, IggyError>;

    /// Invoke HTTP DELETE request to the Iggy API with query parameters.
    async fn delete_with_query<T: Serialize + Sync + ?Sized>(
        &self,
        path: &str,
        query: &T,
    ) -> Result<Response, IggyError>;

    /// Returns true if the client is authenticated.
    async fn is_authenticated(&self) -> bool;

    /// Refresh the access token using the provided refresh token.
    async fn refresh_access_token(&self) -> Result<(), IggyError>;

    /// Set the access token.
    async fn set_access_token(&self, token: Option<String>);

    /// Set the access token and refresh token from the provided identity.
    async fn set_token_from_identity(&self, identity: &IdentityInfo) -> Result<(), IggyError>;
}
