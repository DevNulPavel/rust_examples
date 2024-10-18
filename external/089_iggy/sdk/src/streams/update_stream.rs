use crate::bytes_serializable::BytesSerializable;
use crate::command::{Command, UPDATE_STREAM_CODE};
use crate::error::IggyError;
use crate::identifier::Identifier;
use crate::streams::MAX_NAME_LENGTH;
use crate::utils::text;
use crate::validatable::Validatable;
use bytes::{BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::from_utf8;

/// `UpdateStream` command is used to update an existing stream.
/// It has additional payload:
/// - `stream_id` - unique stream ID (numeric or name).
/// - `name` - unique stream name (string), max length is 255 characters.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct UpdateStream {
    /// Unique stream ID (numeric or name).
    #[serde(skip)]
    pub stream_id: Identifier,
    /// Unique stream name (string), max length is 255 characters.
    pub name: String,
}

impl Command for UpdateStream {
    fn code(&self) -> u32 {
        UPDATE_STREAM_CODE
    }
}

impl Default for UpdateStream {
    fn default() -> Self {
        UpdateStream {
            stream_id: Identifier::default(),
            name: "stream".to_string(),
        }
    }
}

impl Validatable<IggyError> for UpdateStream {
    fn validate(&self) -> Result<(), IggyError> {
        if self.name.is_empty() || self.name.len() > MAX_NAME_LENGTH {
            return Err(IggyError::InvalidStreamName);
        }

        if !text::is_resource_name_valid(&self.name) {
            return Err(IggyError::InvalidStreamName);
        }

        Ok(())
    }
}

impl BytesSerializable for UpdateStream {
    fn to_bytes(&self) -> Bytes {
        let stream_id_bytes = self.stream_id.to_bytes();
        let mut bytes = BytesMut::with_capacity(1 + stream_id_bytes.len() + self.name.len());
        bytes.put_slice(&stream_id_bytes);
        #[allow(clippy::cast_possible_truncation)]
        bytes.put_u8(self.name.len() as u8);
        bytes.put_slice(self.name.as_bytes());
        bytes.freeze()
    }

    fn from_bytes(bytes: Bytes) -> std::result::Result<UpdateStream, IggyError> {
        if bytes.len() < 5 {
            return Err(IggyError::InvalidCommand);
        }

        let mut position = 0;
        let stream_id = Identifier::from_bytes(bytes.clone())?;
        position += stream_id.get_size_bytes() as usize;
        let name_length = bytes[position];
        let name =
            from_utf8(&bytes[position + 1..position + 1 + name_length as usize])?.to_string();
        if name.len() != name_length as usize {
            return Err(IggyError::InvalidCommand);
        }

        let command = UpdateStream { stream_id, name };
        Ok(command)
    }
}

impl Display for UpdateStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}", self.stream_id, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_serialized_as_bytes() {
        let command = UpdateStream {
            stream_id: Identifier::numeric(1).unwrap(),
            name: "test".to_string(),
        };

        let bytes = command.to_bytes();
        let mut position = 0;
        let stream_id = Identifier::from_bytes(bytes.clone()).unwrap();
        position += stream_id.get_size_bytes() as usize;
        let name_length = bytes[position];
        let name = from_utf8(&bytes[position + 1..position + 1 + name_length as usize])
            .unwrap()
            .to_string();

        assert!(!bytes.is_empty());
        assert_eq!(stream_id, command.stream_id);
        assert_eq!(name, command.name);
    }

    #[test]
    fn should_be_deserialized_from_bytes() {
        let stream_id = Identifier::numeric(1).unwrap();
        let name = "test".to_string();

        let stream_id_bytes = stream_id.to_bytes();
        let mut bytes = BytesMut::with_capacity(1 + stream_id_bytes.len() + name.len());
        bytes.put_slice(&stream_id_bytes);
        #[allow(clippy::cast_possible_truncation)]
        bytes.put_u8(name.len() as u8);
        bytes.put_slice(name.as_bytes());
        let command = UpdateStream::from_bytes(bytes.freeze());
        assert!(command.is_ok());

        let command = command.unwrap();
        assert_eq!(command.stream_id, stream_id);
        assert_eq!(command.name, name);
    }
}
