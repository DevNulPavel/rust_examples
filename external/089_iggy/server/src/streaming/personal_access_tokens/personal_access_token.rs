use crate::streaming::utils::hash;
use iggy::models::user_info::UserId;
use iggy::utils::expiry::IggyExpiry;
use iggy::utils::text::as_base64;
use iggy::utils::timestamp::IggyTimestamp;
use ring::rand::SecureRandom;

const SIZE: usize = 50;

#[derive(Debug)]
pub struct PersonalAccessToken {
    pub user_id: UserId,
    pub name: String,
    pub token: String,
    pub expiry_at: Option<IggyTimestamp>,
}

impl PersonalAccessToken {
    // Raw token is generated and returned only once
    pub fn new(
        user_id: UserId,
        name: &str,
        now: IggyTimestamp,
        expiry: IggyExpiry,
    ) -> (Self, String) {
        let mut buffer: [u8; SIZE] = [0; SIZE];
        let system_random = ring::rand::SystemRandom::new();
        system_random.fill(&mut buffer).unwrap();
        let token = as_base64(&buffer);
        let token_hash = Self::hash_token(&token);
        (
            Self {
                user_id,
                name: name.to_string(),
                token: token_hash,
                expiry_at: Self::calculate_expiry_at(now, expiry),
            },
            token,
        )
    }

    pub fn raw(
        user_id: UserId,
        name: &str,
        token_hash: &str,
        expiry_at: Option<IggyTimestamp>,
    ) -> Self {
        Self {
            user_id,
            name: name.into(),
            token: token_hash.into(),
            expiry_at,
        }
    }

    pub fn is_expired(&self, now: IggyTimestamp) -> bool {
        match self.expiry_at {
            None => false,
            Some(expiry_at) => expiry_at.as_micros() <= now.as_micros(),
        }
    }

    pub fn hash_token(token: &str) -> String {
        hash::calculate_256(token.as_bytes())
    }

    pub fn calculate_expiry_at(now: IggyTimestamp, expiry: IggyExpiry) -> Option<IggyTimestamp> {
        match expiry {
            IggyExpiry::ExpireDuration(expiry) => {
                Some(IggyTimestamp::from(now.as_micros() + expiry.as_micros()))
            }
            IggyExpiry::NeverExpire => None,
            IggyExpiry::ServerDefault => None, // This will
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iggy::utils::duration::IggyDuration;
    use iggy::utils::timestamp::IggyTimestamp;

    #[test]
    fn personal_access_token_should_be_created_with_random_secure_value_and_hashed_successfully() {
        let user_id = 1;
        let now = IggyTimestamp::now();
        let name = "test_token";
        let (personal_access_token, raw_token) =
            PersonalAccessToken::new(user_id, name, now, IggyExpiry::NeverExpire);
        assert_eq!(personal_access_token.name, name);
        assert!(!personal_access_token.token.is_empty());
        assert!(!raw_token.is_empty());
        assert_ne!(personal_access_token.token, raw_token);
        assert_eq!(
            personal_access_token.token,
            PersonalAccessToken::hash_token(&raw_token)
        );
    }

    #[test]
    fn personal_access_token_should_be_expired_given_passed_expiry() {
        let user_id = 1;
        let now = IggyTimestamp::now();
        let expiry_ms = 10;
        let expiry = IggyExpiry::ExpireDuration(IggyDuration::from(expiry_ms));
        let name = "test_token";
        let (personal_access_token, _) = PersonalAccessToken::new(user_id, name, now, expiry);
        let later = IggyTimestamp::from(now.as_micros() + expiry_ms + 1);
        assert!(personal_access_token.is_expired(later));
    }
}
