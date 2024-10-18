use crate::state::command::EntryCommand;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use iggy::bytes_serializable::BytesSerializable;
use iggy::error::IggyError;
use iggy::utils::checksum;
use iggy::utils::timestamp::IggyTimestamp;
use std::fmt::{Display, Formatter};

/// State entry in the log
/// - `index` - Index (operation number) of the entry in the log
/// - `term` - Election term (view number) for replication
/// - `leader_id` - Leader ID for replication
/// - `version` - Server version based on semver as number e.g. 1.234.567 -> 1234567
/// - `flags` - Reserved for future use
/// - `timestamp` - Timestamp when the command was issued
/// - `user_id` - User ID of the user who issued the command
/// - `checksum` - Checksum of the entry
/// - `code` - Command code
/// - `command` - Payload of the command
/// - `context` - Optional context e.g. used to enrich the payload with additional data
#[derive(Debug)]
pub struct StateEntry {
    pub index: u64,
    pub term: u64,
    pub leader_id: u32,
    pub version: u32,
    pub flags: u64,
    pub timestamp: IggyTimestamp,
    pub user_id: u32,
    pub checksum: u32,
    pub context: Bytes,
    pub command: Bytes,
}

impl StateEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        index: u64,
        term: u64,
        leader_id: u32,
        version: u32,
        flags: u64,
        timestamp: IggyTimestamp,
        user_id: u32,
        checksum: u32,
        context: Bytes,
        command: Bytes,
    ) -> Self {
        Self {
            index,
            term,
            leader_id,
            version,
            flags,
            timestamp,
            user_id,
            checksum,
            context,
            command,
        }
    }

    pub fn command(&self) -> Result<EntryCommand, IggyError> {
        EntryCommand::from_bytes(self.command.clone())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn calculate_checksum(
        index: u64,
        term: u64,
        leader_id: u32,
        version: u32,
        flags: u64,
        timestamp: IggyTimestamp,
        user_id: u32,
        context: &Bytes,
        command: &Bytes,
    ) -> u32 {
        let mut bytes =
            BytesMut::with_capacity(8 + 8 + 4 + 4 + 8 + 8 + 4 + 4 + context.len() + command.len());
        bytes.put_u64_le(index);
        bytes.put_u64_le(term);
        bytes.put_u32_le(leader_id);
        bytes.put_u32_le(version);
        bytes.put_u64_le(flags);
        bytes.put_u64_le(timestamp.into());
        bytes.put_u32_le(user_id);
        bytes.put_u32_le(context.len() as u32);
        bytes.put_slice(context);
        bytes.extend(command);
        checksum::calculate(&bytes.freeze())
    }
}

impl Display for StateEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StateEntry {{ index: {}, term: {}, leader ID: {}, version: {}, flags: {}, timestamp: {}, user ID: {}, checksum: {} }}",
            self.index,
            self.term,
            self.leader_id,
            self.version,
            self.flags,
            self.timestamp,
            self.user_id,
            self.checksum,
        )
    }
}

impl BytesSerializable for StateEntry {
    fn to_bytes(&self) -> Bytes {
        let mut bytes = BytesMut::with_capacity(
            8 + 8 + 4 + 4 + 8 + 8 + 4 + 4 + 4 + self.context.len() + self.command.len(),
        );
        bytes.put_u64_le(self.index);
        bytes.put_u64_le(self.term);
        bytes.put_u32_le(self.leader_id);
        bytes.put_u32_le(self.version);
        bytes.put_u64_le(self.flags);
        bytes.put_u64_le(self.timestamp.into());
        bytes.put_u32_le(self.user_id);
        bytes.put_u32_le(self.checksum);
        bytes.put_u32_le(self.context.len() as u32);
        bytes.put_slice(&self.context);
        bytes.extend(&self.command);
        bytes.freeze()
    }

    fn from_bytes(bytes: Bytes) -> Result<Self, IggyError>
    where
        Self: Sized,
    {
        let index = bytes.slice(0..8).get_u64_le();
        let term = bytes.slice(8..16).get_u64_le();
        let leader_id = bytes.slice(16..20).get_u32_le();
        let version = bytes.slice(20..24).get_u32_le();
        let flags = bytes.slice(24..32).get_u64_le();
        let timestamp = IggyTimestamp::from(bytes.slice(32..40).get_u64_le());
        let user_id = bytes.slice(40..44).get_u32_le();
        let checksum = bytes.slice(44..48).get_u32_le();
        let context_length = bytes.slice(48..52).get_u32_le() as usize;
        let context = bytes.slice(52..52 + context_length);
        let command = bytes.slice(52 + context_length..);

        Ok(StateEntry {
            index,
            term,
            leader_id,
            version,
            flags,
            timestamp,
            user_id,
            checksum,
            context,
            command,
        })
    }
}
