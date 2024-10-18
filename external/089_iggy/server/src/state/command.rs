use crate::state::models::CreatePersonalAccessTokenWithHash;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use iggy::bytes_serializable::BytesSerializable;
use iggy::command::{
    Command, CHANGE_PASSWORD_CODE, CREATE_CONSUMER_GROUP_CODE, CREATE_PARTITIONS_CODE,
    CREATE_PERSONAL_ACCESS_TOKEN_CODE, CREATE_STREAM_CODE, CREATE_TOPIC_CODE, CREATE_USER_CODE,
    DELETE_CONSUMER_GROUP_CODE, DELETE_PARTITIONS_CODE, DELETE_PERSONAL_ACCESS_TOKEN_CODE,
    DELETE_STREAM_CODE, DELETE_TOPIC_CODE, DELETE_USER_CODE, PURGE_STREAM_CODE, PURGE_TOPIC_CODE,
    UPDATE_PERMISSIONS_CODE, UPDATE_STREAM_CODE, UPDATE_TOPIC_CODE, UPDATE_USER_CODE,
};
use iggy::consumer_groups::create_consumer_group::CreateConsumerGroup;
use iggy::consumer_groups::delete_consumer_group::DeleteConsumerGroup;
use iggy::error::IggyError;
use iggy::partitions::create_partitions::CreatePartitions;
use iggy::partitions::delete_partitions::DeletePartitions;
use iggy::personal_access_tokens::delete_personal_access_token::DeletePersonalAccessToken;
use iggy::streams::create_stream::CreateStream;
use iggy::streams::delete_stream::DeleteStream;
use iggy::streams::purge_stream::PurgeStream;
use iggy::streams::update_stream::UpdateStream;
use iggy::topics::create_topic::CreateTopic;
use iggy::topics::delete_topic::DeleteTopic;
use iggy::topics::purge_topic::PurgeTopic;
use iggy::topics::update_topic::UpdateTopic;
use iggy::users::change_password::ChangePassword;
use iggy::users::create_user::CreateUser;
use iggy::users::delete_user::DeleteUser;
use iggy::users::update_permissions::UpdatePermissions;
use iggy::users::update_user::UpdateUser;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum EntryCommand {
    CreateStream(CreateStream),
    UpdateStream(UpdateStream),
    DeleteStream(DeleteStream),
    PurgeStream(PurgeStream),
    CreateTopic(CreateTopic),
    UpdateTopic(UpdateTopic),
    DeleteTopic(DeleteTopic),
    PurgeTopic(PurgeTopic),
    CreatePartitions(CreatePartitions),
    DeletePartitions(DeletePartitions),
    CreateConsumerGroup(CreateConsumerGroup),
    DeleteConsumerGroup(DeleteConsumerGroup),
    CreateUser(CreateUser),
    UpdateUser(UpdateUser),
    DeleteUser(DeleteUser),
    ChangePassword(ChangePassword),
    UpdatePermissions(UpdatePermissions),
    CreatePersonalAccessToken(CreatePersonalAccessTokenWithHash),
    DeletePersonalAccessToken(DeletePersonalAccessToken),
}

impl BytesSerializable for EntryCommand {
    fn to_bytes(&self) -> Bytes {
        let (code, command) = match self {
            EntryCommand::CreateStream(command) => (command.code(), command.to_bytes()),
            EntryCommand::UpdateStream(command) => (command.code(), command.to_bytes()),
            EntryCommand::DeleteStream(command) => (command.code(), command.to_bytes()),
            EntryCommand::PurgeStream(command) => (command.code(), command.to_bytes()),
            EntryCommand::CreateTopic(command) => (command.code(), command.to_bytes()),
            EntryCommand::UpdateTopic(command) => (command.code(), command.to_bytes()),
            EntryCommand::DeleteTopic(command) => (command.code(), command.to_bytes()),
            EntryCommand::PurgeTopic(command) => (command.code(), command.to_bytes()),
            EntryCommand::CreatePartitions(command) => (command.code(), command.to_bytes()),
            EntryCommand::DeletePartitions(command) => (command.code(), command.to_bytes()),
            EntryCommand::CreateConsumerGroup(command) => (command.code(), command.to_bytes()),
            EntryCommand::DeleteConsumerGroup(command) => (command.code(), command.to_bytes()),
            EntryCommand::CreateUser(command) => (command.code(), command.to_bytes()),
            EntryCommand::UpdateUser(command) => (command.code(), command.to_bytes()),
            EntryCommand::DeleteUser(command) => (command.code(), command.to_bytes()),
            EntryCommand::ChangePassword(command) => (command.code(), command.to_bytes()),
            EntryCommand::UpdatePermissions(command) => (command.code(), command.to_bytes()),
            EntryCommand::CreatePersonalAccessToken(command) => {
                (command.code(), command.to_bytes())
            }
            EntryCommand::DeletePersonalAccessToken(command) => {
                (command.code(), command.to_bytes())
            }
        };

        let mut bytes = BytesMut::with_capacity(4 + 4 + command.len());
        bytes.put_u32_le(code);
        bytes.put_u32_le(command.len() as u32);
        bytes.extend(command);
        bytes.freeze()
    }

    fn from_bytes(bytes: Bytes) -> Result<Self, IggyError>
    where
        Self: Sized,
    {
        let code = bytes.slice(0..4).get_u32_le();
        let length = bytes.slice(4..8).get_u32_le();
        let payload = bytes.slice(8..8 + length as usize);
        match code {
            CREATE_STREAM_CODE => Ok(EntryCommand::CreateStream(CreateStream::from_bytes(
                payload,
            )?)),
            UPDATE_STREAM_CODE => Ok(EntryCommand::UpdateStream(UpdateStream::from_bytes(
                payload,
            )?)),
            DELETE_STREAM_CODE => Ok(EntryCommand::DeleteStream(DeleteStream::from_bytes(
                payload,
            )?)),
            PURGE_STREAM_CODE => Ok(EntryCommand::PurgeStream(PurgeStream::from_bytes(payload)?)),
            CREATE_TOPIC_CODE => Ok(EntryCommand::CreateTopic(CreateTopic::from_bytes(payload)?)),
            UPDATE_TOPIC_CODE => Ok(EntryCommand::UpdateTopic(UpdateTopic::from_bytes(payload)?)),
            DELETE_TOPIC_CODE => Ok(EntryCommand::DeleteTopic(DeleteTopic::from_bytes(payload)?)),
            PURGE_TOPIC_CODE => Ok(EntryCommand::PurgeTopic(PurgeTopic::from_bytes(payload)?)),
            CREATE_PARTITIONS_CODE => Ok(EntryCommand::CreatePartitions(
                CreatePartitions::from_bytes(payload)?,
            )),
            DELETE_PARTITIONS_CODE => Ok(EntryCommand::DeletePartitions(
                DeletePartitions::from_bytes(payload)?,
            )),
            CREATE_CONSUMER_GROUP_CODE => Ok(EntryCommand::CreateConsumerGroup(
                CreateConsumerGroup::from_bytes(payload)?,
            )),
            DELETE_CONSUMER_GROUP_CODE => Ok(EntryCommand::DeleteConsumerGroup(
                DeleteConsumerGroup::from_bytes(payload)?,
            )),
            CREATE_USER_CODE => Ok(EntryCommand::CreateUser(CreateUser::from_bytes(payload)?)),
            UPDATE_USER_CODE => Ok(EntryCommand::UpdateUser(UpdateUser::from_bytes(payload)?)),
            DELETE_USER_CODE => Ok(EntryCommand::DeleteUser(DeleteUser::from_bytes(payload)?)),
            CHANGE_PASSWORD_CODE => Ok(EntryCommand::ChangePassword(ChangePassword::from_bytes(
                payload,
            )?)),
            UPDATE_PERMISSIONS_CODE => Ok(EntryCommand::UpdatePermissions(
                UpdatePermissions::from_bytes(payload)?,
            )),
            CREATE_PERSONAL_ACCESS_TOKEN_CODE => Ok(EntryCommand::CreatePersonalAccessToken(
                CreatePersonalAccessTokenWithHash::from_bytes(payload)?,
            )),
            DELETE_PERSONAL_ACCESS_TOKEN_CODE => Ok(EntryCommand::DeletePersonalAccessToken(
                DeletePersonalAccessToken::from_bytes(payload)?,
            )),
            _ => Err(IggyError::InvalidCommand),
        }
    }
}

impl Display for EntryCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryCommand::CreateStream(command) => write!(f, "CreateStream({})", command),
            EntryCommand::UpdateStream(command) => write!(f, "UpdateStream({})", command),
            EntryCommand::DeleteStream(command) => write!(f, "DeleteStream({})", command),
            EntryCommand::PurgeStream(command) => write!(f, "PurgeStream({})", command),
            EntryCommand::CreateTopic(command) => write!(f, "CreateTopic({})", command),
            EntryCommand::UpdateTopic(command) => write!(f, "UpdateTopic({})", command),
            EntryCommand::DeleteTopic(command) => write!(f, "DeleteTopic({})", command),
            EntryCommand::PurgeTopic(command) => write!(f, "PurgeTopic({})", command),
            EntryCommand::CreatePartitions(command) => write!(f, "CreatePartitions({})", command),
            EntryCommand::DeletePartitions(command) => write!(f, "DeletePartitions({})", command),
            EntryCommand::CreateConsumerGroup(command) => {
                write!(f, "CreateConsumerGroup({})", command)
            }
            EntryCommand::DeleteConsumerGroup(command) => {
                write!(f, "DeleteConsumerGroup({})", command)
            }
            EntryCommand::CreateUser(command) => write!(f, "CreateUser({})", command),
            EntryCommand::UpdateUser(command) => write!(f, "UpdateUser({})", command),
            EntryCommand::DeleteUser(command) => write!(f, "DeleteUser({})", command),
            EntryCommand::ChangePassword(command) => write!(f, "ChangePassword({})", command),
            EntryCommand::UpdatePermissions(command) => write!(f, "UpdatePermissions({})", command),
            EntryCommand::CreatePersonalAccessToken(command) => {
                write!(f, "CreatePersonalAccessToken({})", command)
            }
            EntryCommand::DeletePersonalAccessToken(command) => {
                write!(f, "DeletePersonalAccessToken({})", command)
            }
        }
    }
}
