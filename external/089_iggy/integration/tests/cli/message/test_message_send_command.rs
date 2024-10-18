use crate::cli::common::{
    IggyCmdCommand, IggyCmdTest, IggyCmdTestCase, TestHelpCmd, TestStreamId, TestTopicId,
    CLAP_INDENT, USAGE_PREFIX,
};
use assert_cmd::assert::Assert;
use async_trait::async_trait;
use iggy::client::Client;
use iggy::consumer::Consumer;
use iggy::messages::poll_messages::PollingStrategy;
use iggy::models::header::{HeaderKey, HeaderValue};
use iggy::utils::expiry::IggyExpiry;
use iggy::utils::topic_size::MaxTopicSize;
use predicates::str::diff;
use serial_test::parallel;
use std::collections::HashMap;
use std::str::from_utf8;
use xxhash_rust::xxh32::xxh32;

#[derive(Debug)]
enum PartitionSelection {
    Balanced,
    Id(u32),
    Key(String),
}

impl PartitionSelection {
    fn to_args(&self) -> Vec<String> {
        match self {
            PartitionSelection::Balanced => vec![],
            PartitionSelection::Id(id) => vec!["-p".into(), format!("{}", id)],
            PartitionSelection::Key(key) => vec!["-m".into(), key.clone()],
        }
    }
}

#[derive(Debug)]
enum ProvideMessages {
    AsArgs,
    ViaStdin,
}

struct TestMessageSendCmd {
    stream_id: u32,
    stream_name: String,
    topic_id: u32,
    topic_name: String,
    partitions_count: u32,
    messages: Vec<String>,
    message_input: ProvideMessages,
    using_stream_id: TestStreamId,
    using_topic_id: TestTopicId,
    partitioning: PartitionSelection,
    header: Option<HashMap<HeaderKey, HeaderValue>>,
}

impl TestMessageSendCmd {
    #[allow(clippy::too_many_arguments)]
    fn new(
        stream_id: u32,
        stream_name: String,
        topic_id: u32,
        topic_name: String,
        partitions_count: u32,
        messages: Vec<String>,
        message_input: ProvideMessages,
        using_stream_id: TestStreamId,
        using_topic_id: TestTopicId,
        partitioning: PartitionSelection,
        header: Option<HashMap<HeaderKey, HeaderValue>>,
    ) -> Self {
        Self {
            stream_id,
            stream_name,
            topic_id,
            topic_name,
            partitions_count,
            messages,
            message_input,
            using_stream_id,
            using_topic_id,
            partitioning,
            header,
        }
    }

    fn to_args(&self) -> Vec<String> {
        let mut command = self.partitioning.to_args();

        command.extend(match self.using_stream_id {
            TestStreamId::Numeric => vec![format!("{}", self.stream_id)],
            TestStreamId::Named => vec![self.stream_name.clone()],
        });

        command.push(match self.using_topic_id {
            TestTopicId::Numeric => format!("{}", self.topic_id),
            TestTopicId::Named => self.topic_name.clone(),
        });

        if let Some(header) = &self.header {
            command.push("--headers".to_string());
            command.push(
                header
                    .iter()
                    .map(|(k, v)| format!("{k}:{}:{}", v.kind, v.value_only_to_string()))
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }

        match &self.message_input {
            ProvideMessages::AsArgs => {
                command.extend(self.messages.clone());
            }
            ProvideMessages::ViaStdin => {}
        }

        command
    }

    fn calculate_partition_id_from_messages_key(&self, messages_key: &[u8]) -> u32 {
        let messages_key_hash = xxh32(messages_key, 0);
        let mut partition_id = messages_key_hash % self.partitions_count;
        if partition_id == 0 {
            partition_id = self.partitions_count;
        }

        partition_id
    }

    fn get_partition_id(&self) -> u32 {
        match &self.partitioning {
            PartitionSelection::Balanced => 1,
            PartitionSelection::Id(id) => *id,
            PartitionSelection::Key(key) => {
                self.calculate_partition_id_from_messages_key(key.clone().as_bytes())
            }
        }
    }
}

#[async_trait]
impl IggyCmdTestCase for TestMessageSendCmd {
    async fn prepare_server_state(&mut self, client: &dyn Client) {
        let stream = client
            .create_stream(&self.stream_name, self.stream_id.into())
            .await;
        assert!(stream.is_ok());

        let topic = client
            .create_topic(
                &self.stream_id.try_into().unwrap(),
                &self.topic_name,
                self.partitions_count,
                Default::default(),
                None,
                Some(self.topic_id),
                IggyExpiry::NeverExpire,
                MaxTopicSize::ServerDefault,
            )
            .await;
        assert!(topic.is_ok());
    }

    fn get_command(&self) -> IggyCmdCommand {
        IggyCmdCommand::new()
            .arg("message")
            .arg("send")
            .args(self.to_args())
            .with_env_credentials()
    }

    fn provide_stdin_input(&self) -> Option<Vec<String>> {
        match &self.message_input {
            ProvideMessages::ViaStdin => Some(self.messages.clone()),
            ProvideMessages::AsArgs => None,
        }
    }

    fn verify_command(&self, command_state: Assert) {
        let stream_id = match self.using_stream_id {
            TestStreamId::Numeric => format!("{}", self.stream_id),
            TestStreamId::Named => self.stream_name.clone(),
        };

        let topic_id = match self.using_topic_id {
            TestTopicId::Numeric => format!("{}", self.topic_id),
            TestTopicId::Named => self.topic_name.clone(),
        };

        let message = format!("Executing send messages to topic with ID: {} and stream with ID: {}\nSent messages to topic with ID: {} and stream with ID: {}\n",
            topic_id, stream_id, topic_id, stream_id);

        command_state.success().stdout(diff(message));
    }

    async fn verify_server_state(&self, client: &dyn Client) {
        let topic = client
            .get_topic(
                &self.stream_id.try_into().unwrap(),
                &self.topic_id.try_into().unwrap(),
            )
            .await;
        assert!(topic.is_ok());
        let topic_details = topic.unwrap().expect("Failed to get topic");
        assert_eq!(topic_details.messages_count, self.messages.len() as u64);

        let polled_messages = client
            .poll_messages(
                &self.stream_id.try_into().unwrap(),
                &self.topic_id.try_into().unwrap(),
                Some(self.get_partition_id()),
                &Consumer::default(),
                &PollingStrategy::offset(0),
                self.messages.len() as u32,
                false,
            )
            .await;

        assert!(polled_messages.is_ok());
        let polled_messages = polled_messages.unwrap();
        assert_eq!(polled_messages.messages.len(), self.messages.len());
        assert_eq!(
            polled_messages
                .messages
                .iter()
                .map(|m| from_utf8(&m.payload.clone()).unwrap().to_string())
                .collect::<Vec<_>>(),
            self.messages
        );

        if let Some(expected_header) = &self.header {
            polled_messages.messages.iter().for_each(|m| {
                assert!(m.headers.is_some());
                assert_eq!(expected_header, m.headers.as_ref().unwrap());
            })
        };

        let topic = client
            .delete_topic(
                &self.stream_id.try_into().unwrap(),
                &self.topic_id.try_into().unwrap(),
            )
            .await;
        assert!(topic.is_ok());

        let stream = client
            .delete_stream(&self.stream_id.try_into().unwrap())
            .await;
        assert!(stream.is_ok());
    }
}

#[tokio::test]
#[parallel]
pub async fn should_be_successful() {
    let mut iggy_cmd_test = IggyCmdTest::default();

    let test_parameters = vec![
        (
            ProvideMessages::AsArgs,
            TestStreamId::Numeric,
            TestTopicId::Numeric,
            PartitionSelection::Balanced,
        ),
        (
            ProvideMessages::ViaStdin,
            TestStreamId::Numeric,
            TestTopicId::Named,
            PartitionSelection::Balanced,
        ),
        (
            ProvideMessages::AsArgs,
            TestStreamId::Named,
            TestTopicId::Numeric,
            PartitionSelection::Balanced,
        ),
        (
            ProvideMessages::ViaStdin,
            TestStreamId::Named,
            TestTopicId::Named,
            PartitionSelection::Balanced,
        ),
        (
            ProvideMessages::ViaStdin,
            TestStreamId::Numeric,
            TestTopicId::Numeric,
            PartitionSelection::Id(1),
        ),
        (
            ProvideMessages::AsArgs,
            TestStreamId::Named,
            TestTopicId::Numeric,
            PartitionSelection::Id(2),
        ),
        (
            ProvideMessages::AsArgs,
            TestStreamId::Numeric,
            TestTopicId::Named,
            PartitionSelection::Id(3),
        ),
        (
            ProvideMessages::ViaStdin,
            TestStreamId::Named,
            TestTopicId::Named,
            PartitionSelection::Id(4),
        ),
        (
            ProvideMessages::AsArgs,
            TestStreamId::Numeric,
            TestTopicId::Numeric,
            PartitionSelection::Key(String::from("some-complex-key")),
        ),
        (
            ProvideMessages::ViaStdin,
            TestStreamId::Named,
            TestTopicId::Numeric,
            PartitionSelection::Key(String::from("another random value")),
        ),
        (
            ProvideMessages::ViaStdin,
            TestStreamId::Numeric,
            TestTopicId::Named,
            PartitionSelection::Key(String::from("some-random-key")),
        ),
        (
            ProvideMessages::AsArgs,
            TestStreamId::Named,
            TestTopicId::Named,
            PartitionSelection::Key(String::from("just a key")),
        ),
    ];

    iggy_cmd_test.setup().await;
    for (message_input, using_stream_id, using_topic_id, using_partitioning) in test_parameters {
        iggy_cmd_test
            .execute_test(TestMessageSendCmd::new(
                1,
                String::from("stream"),
                2,
                String::from("topic"),
                4,
                vec![
                    String::from("test message1"),
                    String::from("test message2"),
                    String::from("test message3"),
                ],
                message_input,
                using_stream_id,
                using_topic_id,
                using_partitioning,
                Some(HashMap::from([
                    (
                        HeaderKey::new("key1").unwrap(),
                        HeaderValue::from_kind_str_and_value_str("string", "value1").unwrap(),
                    ),
                    (
                        HeaderKey::new("key2").unwrap(),
                        HeaderValue::from_kind_str_and_value_str("int32", "42").unwrap(),
                    ),
                ])),
            ))
            .await;
    }
}

#[tokio::test]
#[parallel]
pub async fn should_help_match() {
    let mut iggy_cmd_test = IggyCmdTest::help_message();

    iggy_cmd_test
        .execute_test_for_help_command(TestHelpCmd::new(
            vec!["message", "send", "--help"],
            format!(
                r#"Send messages to given topic ID and given stream ID

Stream ID can be specified as a stream name or ID
Topic ID can be specified as a topic name or ID

Examples
 iggy message send 1 2 message
 iggy message send stream 2 "long message"
 iggy message send 1 topic message1 message2 message3
 iggy message send stream topic "long message with spaces"

{USAGE_PREFIX} message send [OPTIONS] <STREAM_ID> <TOPIC_ID> [MESSAGES]...

Arguments:
  <STREAM_ID>
          ID of the stream to which the message will be sent
{CLAP_INDENT}
          Stream ID can be specified as a stream name or ID

  <TOPIC_ID>
          ID of the topic to which the message will be sent
{CLAP_INDENT}
          Topic ID can be specified as a topic name or ID

  [MESSAGES]...
          Messages to be sent
{CLAP_INDENT}
          If no messages are provided, the command will read the messages from the
          standard input and each line will be sent as a separate message.
          If messages are provided, they will be sent as is. If message contains
          spaces, it should be enclosed in quotes. Limit of the messages and size
          of each message is defined by the used shell.

Options:
  -p, --partition-id <PARTITION_ID>
          ID of the partition to which the message will be sent

  -m, --message-key <MESSAGE_KEY>
          Messages key which will be used to partition the messages
{CLAP_INDENT}
          Value of the key will be used by the server to calculate the partition ID

  -H, --headers <HEADERS>
          Comma separated list of key:kind:value, sent as header with the message
{CLAP_INDENT}
          Headers are comma seperated key-value pairs that can be sent with the message.
          Kind can be one of the following: raw, string, bool, int8, int16, int32, int64,
          int128, uint8, uint16, uint32, uint64, uint128, float32, float64

      --input-file <INPUT_FILE>
          Input file with messages to be sent
{CLAP_INDENT}
          File should contain messages stored in binary format. If the file does
          not exist, the command will fail. If the file is not specified, the command
          will read the messages from the standard input and each line will
          be sent as a separate message. If the file is specified, the messages
          will be read from the file and sent as is. Option cannot be used
          with the messages option (messages given as command line arguments).

  -h, --help
          Print help (see a summary with '-h')
"#,
            ),
        ))
        .await;
}

#[tokio::test]
#[parallel]
pub async fn should_short_help_match() {
    let mut iggy_cmd_test = IggyCmdTest::default();

    iggy_cmd_test
        .execute_test_for_help_command(TestHelpCmd::new(
            vec!["message", "send", "-h"],
            format!(
                r#"Send messages to given topic ID and given stream ID

{USAGE_PREFIX} message send [OPTIONS] <STREAM_ID> <TOPIC_ID> [MESSAGES]...

Arguments:
  <STREAM_ID>    ID of the stream to which the message will be sent
  <TOPIC_ID>     ID of the topic to which the message will be sent
  [MESSAGES]...  Messages to be sent

Options:
  -p, --partition-id <PARTITION_ID>  ID of the partition to which the message will be sent
  -m, --message-key <MESSAGE_KEY>    Messages key which will be used to partition the messages
  -H, --headers <HEADERS>            Comma separated list of key:kind:value, sent as header with the message
      --input-file <INPUT_FILE>      Input file with messages to be sent
  -h, --help                         Print help (see more with '--help')
"#,
            ),
        ))
        .await;
}
