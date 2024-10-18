use crate::bytes_serializable::BytesSerializable;
use crate::command::{Command, GET_TOPICS_CODE};
use crate::error::IggyError;
use crate::identifier::Identifier;
use crate::validatable::Validatable;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// `GetTopics` command is used to retrieve the collection of topics from a stream.
/// It has additional payload:
/// - `stream_id` - unique stream ID (numeric or name).
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct GetTopics {
    /// Unique stream ID (numeric or name).
    #[serde(skip)]
    pub stream_id: Identifier,
}

impl Command for GetTopics {
    fn code(&self) -> u32 {
        GET_TOPICS_CODE
    }
}

impl Validatable<IggyError> for GetTopics {
    fn validate(&self) -> Result<(), IggyError> {
        Ok(())
    }
}

impl BytesSerializable for GetTopics {
    fn to_bytes(&self) -> Bytes {
        self.stream_id.to_bytes()
    }

    fn from_bytes(bytes: Bytes) -> std::result::Result<GetTopics, IggyError> {
        if bytes.len() < 3 {
            return Err(IggyError::InvalidCommand);
        }

        let stream_id = Identifier::from_bytes(bytes)?;
        let command = GetTopics { stream_id };
        Ok(command)
    }
}

impl Display for GetTopics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.stream_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_serialized_as_bytes() {
        let command = GetTopics {
            stream_id: Identifier::numeric(1).unwrap(),
        };

        let bytes = command.to_bytes();
        let stream_id = Identifier::from_bytes(bytes.clone()).unwrap();

        assert!(!bytes.is_empty());
        assert_eq!(stream_id, command.stream_id);
    }

    #[test]
    fn should_be_deserialized_from_bytes() {
        let stream_id = Identifier::numeric(1).unwrap();
        let bytes = stream_id.to_bytes();
        let command = GetTopics::from_bytes(bytes);
        assert!(command.is_ok());

        let command = command.unwrap();
        assert_eq!(command.stream_id, stream_id);
    }
}
