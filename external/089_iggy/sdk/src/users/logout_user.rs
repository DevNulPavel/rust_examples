use crate::bytes_serializable::BytesSerializable;
use crate::command::{Command, LOGOUT_USER_CODE};
use crate::error::IggyError;
use crate::validatable::Validatable;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// `LogoutUser` command is used to log out the authenticated user.
/// It has no additional payload.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct LogoutUser {}

impl Command for LogoutUser {
    fn code(&self) -> u32 {
        LOGOUT_USER_CODE
    }
}

impl Validatable<IggyError> for LogoutUser {
    fn validate(&self) -> Result<(), IggyError> {
        Ok(())
    }
}

impl BytesSerializable for LogoutUser {
    fn to_bytes(&self) -> Bytes {
        Bytes::new()
    }

    fn from_bytes(bytes: Bytes) -> Result<LogoutUser, IggyError> {
        if !bytes.is_empty() {
            return Err(IggyError::InvalidCommand);
        }

        let command = LogoutUser {};
        Ok(command)
    }
}

impl Display for LogoutUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_serialized_as_empty_bytes() {
        let command = LogoutUser {};
        let bytes = command.to_bytes();
        assert!(bytes.is_empty());
    }

    #[test]
    fn should_be_deserialized_from_empty_bytes() {
        let command = LogoutUser::from_bytes(Bytes::new());
        assert!(command.is_ok());
    }

    #[test]
    fn should_not_be_deserialized_from_empty_bytes() {
        let command = LogoutUser::from_bytes(Bytes::from_static(&[0]));
        assert!(command.is_err());
    }
}
