use crate::streaming::personal_access_tokens::personal_access_token::PersonalAccessToken;
use crate::streaming::session::Session;
use crate::streaming::systems::system::System;
use crate::streaming::users::user::User;
use iggy::error::IggyError;
use iggy::utils::expiry::IggyExpiry;
use iggy::utils::text;
use iggy::utils::timestamp::IggyTimestamp;
use tracing::{error, info};

impl System {
    pub async fn get_personal_access_tokens(
        &self,
        session: &Session,
    ) -> Result<Vec<&PersonalAccessToken>, IggyError> {
        self.ensure_authenticated(session)?;
        let user_id = session.get_user_id();
        let user = self.get_user(&user_id.try_into()?)?;
        info!("Loading personal access tokens for user with ID: {user_id}...",);
        let personal_access_tokens: Vec<_> = user.personal_access_tokens.values().collect();
        info!(
            "Loaded {} personal access tokens for user with ID: {user_id}.",
            personal_access_tokens.len(),
        );
        Ok(personal_access_tokens)
    }

    pub async fn create_personal_access_token(
        &mut self,
        session: &Session,
        name: &str,
        expiry: IggyExpiry,
    ) -> Result<String, IggyError> {
        self.ensure_authenticated(session)?;
        let user_id = session.get_user_id();
        let identifier = user_id.try_into()?;
        {
            let user = self.get_user(&identifier)?;
            let max_token_per_user = self.personal_access_token.max_tokens_per_user;
            if user.personal_access_tokens.len() as u32 >= max_token_per_user {
                error!(
                "User with ID: {user_id} has reached the maximum number of personal access tokens: {max_token_per_user}.",
            );
                return Err(IggyError::PersonalAccessTokensLimitReached(
                    user_id,
                    max_token_per_user,
                ));
            }
        }

        let user = self.get_user_mut(&identifier)?;
        let name = text::to_lowercase_non_whitespace(name);
        if user
            .personal_access_tokens
            .values()
            .any(|pat| pat.name == name)
        {
            error!("Personal access token: {name} for user with ID: {user_id} already exists.");
            return Err(IggyError::PersonalAccessTokenAlreadyExists(name, user_id));
        }

        info!("Creating personal access token: {name} for user with ID: {user_id}...");
        let (personal_access_token, token) =
            PersonalAccessToken::new(user_id, &name, IggyTimestamp::now(), expiry);
        user.personal_access_tokens
            .insert(personal_access_token.token.clone(), personal_access_token);
        info!("Created personal access token: {name} for user with ID: {user_id}.");
        Ok(token)
    }

    pub async fn delete_personal_access_token(
        &mut self,
        session: &Session,
        name: &str,
    ) -> Result<(), IggyError> {
        self.ensure_authenticated(session)?;
        let user_id = session.get_user_id();
        let user = self.get_user_mut(&user_id.try_into()?)?;
        let name = text::to_lowercase_non_whitespace(name);
        let token;

        {
            let pat = user
                .personal_access_tokens
                .iter()
                .find(|(_, pat)| pat.name == name);
            if pat.is_none() {
                error!("Personal access token: {name} for user with ID: {user_id} does not exist.",);
                return Err(IggyError::ResourceNotFound(name));
            }

            token = pat.unwrap().1.token.clone();
        }

        info!("Deleting personal access token: {name} for user with ID: {user_id}...");
        user.personal_access_tokens.remove(&token);
        info!("Deleted personal access token: {name} for user with ID: {user_id}.");
        Ok(())
    }

    pub async fn login_with_personal_access_token(
        &self,
        token: &str,
        session: Option<&Session>,
    ) -> Result<&User, IggyError> {
        let token_hash = PersonalAccessToken::hash_token(token);
        let mut personal_access_token = None;
        for user in self.users.values() {
            if let Some(pat) = user.personal_access_tokens.get(&token_hash) {
                personal_access_token = Some(pat);
                break;
            }
        }

        if personal_access_token.is_none() {
            error!("Personal access token: {} does not exist.", token);
            return Err(IggyError::ResourceNotFound(token.to_owned()));
        }

        let personal_access_token = personal_access_token.unwrap();
        if personal_access_token.is_expired(IggyTimestamp::now()) {
            error!(
                "Personal access token: {} for user with ID: {} has expired.",
                personal_access_token.name, personal_access_token.user_id
            );
            return Err(IggyError::PersonalAccessTokenExpired(
                personal_access_token.name.clone(),
                personal_access_token.user_id,
            ));
        }

        let user = self.get_user(&personal_access_token.user_id.try_into()?)?;
        self.login_user_with_credentials(&user.username, None, session)
            .await
    }
}
