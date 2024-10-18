use crate::bytes_serializable::BytesSerializable;
use crate::command::{Command, PURGE_STREAM_CODE};
use crate::error::IggyError;
use crate::identifier::Identifier;
use crate::validatable::Validatable;
use bytes::{BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// `PurgeStream` command is used to purge stream data (all the messages from its topics).
/// It has additional payload:
/// - `stream_id` - unique stream ID (numeric or name).
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct PurgeStream {
    /// Unique stream ID (numeric or name).
    #[serde(skip)]
    pub stream_id: Identifier,
}

impl Command for PurgeStream {
    fn code(&self) -> u32 {
        PURGE_STREAM_CODE
    }
}

impl Validatable<IggyError> for PurgeStream {
    fn validate(&self) -> Result<(), IggyError> {
        Ok(())
    }
}

impl BytesSerializable for PurgeStream {
    fn to_bytes(&self) -> Bytes {
        let stream_id_bytes = self.stream_id.to_bytes();
        let mut bytes = BytesMut::with_capacity(stream_id_bytes.len());
        bytes.put_slice(&stream_id_bytes);
        bytes.freeze()
    }

    fn from_bytes(bytes: Bytes) -> Result<PurgeStream, IggyError> {
        if bytes.len() < 5 {
            return Err(IggyError::InvalidCommand);
        }

        let stream_id = Identifier::from_bytes(bytes)?;
        let command = PurgeStream { stream_id };
        Ok(command)
    }
}

impl Display for PurgeStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.stream_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_serialized_as_bytes() {
        let command = PurgeStream {
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
        let command = PurgeStream::from_bytes(bytes);
        assert!(command.is_ok());

        let command = command.unwrap();
        assert_eq!(command.stream_id, stream_id);
    }
}
