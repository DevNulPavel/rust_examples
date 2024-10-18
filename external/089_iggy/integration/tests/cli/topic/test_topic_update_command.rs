use crate::cli::common::{
    IggyCmdCommand, IggyCmdTest, IggyCmdTestCase, TestHelpCmd, TestStreamId, TestTopicId,
    CLAP_INDENT, USAGE_PREFIX,
};
use assert_cmd::assert::Assert;
use async_trait::async_trait;
use humantime::Duration as HumanDuration;
use iggy::client::Client;
use iggy::compression::compression_algorithm::CompressionAlgorithm;
use iggy::utils::expiry::IggyExpiry;
use iggy::utils::topic_size::MaxTopicSize;
use predicates::str::diff;
use serial_test::parallel;
use std::time::Duration;

struct TestTopicUpdateCmd {
    stream_id: u32,
    stream_name: String,
    topic_id: u32,
    topic_name: String,
    compression_algorithm: CompressionAlgorithm,
    message_expiry: Option<Vec<String>>,
    max_topic_size: MaxTopicSize,
    replication_factor: u8,
    topic_new_name: String,
    topic_new_compression_algorithm: CompressionAlgorithm,
    topic_new_message_expiry: Option<Vec<String>>,
    topic_new_max_size: MaxTopicSize,
    topic_new_replication_factor: u8,
    using_stream_id: TestStreamId,
    using_topic_id: TestTopicId,
}

impl TestTopicUpdateCmd {
    #[allow(clippy::too_many_arguments)]
    fn new(
        stream_id: u32,
        stream_name: String,
        topic_id: u32,
        topic_name: String,
        compression_algorithm: CompressionAlgorithm,
        message_expiry: Option<Vec<String>>,
        max_topic_size: MaxTopicSize,
        replication_factor: u8,
        topic_new_name: String,
        topic_new_compression_algorithm: CompressionAlgorithm,
        topic_new_message_expiry: Option<Vec<String>>,
        topic_new_max_size: MaxTopicSize,
        topic_new_replication_factor: u8,
        using_stream_id: TestStreamId,
        using_topic_id: TestTopicId,
    ) -> Self {
        Self {
            stream_id,
            stream_name,
            topic_id,
            topic_name,
            compression_algorithm,
            message_expiry,
            max_topic_size,
            replication_factor,
            topic_new_name,
            topic_new_compression_algorithm,
            topic_new_message_expiry,
            topic_new_max_size,
            topic_new_replication_factor,
            using_stream_id,
            using_topic_id,
        }
    }

    fn to_args(&self) -> Vec<String> {
        let mut command = match self.using_stream_id {
            TestStreamId::Numeric => vec![format!("{}", self.stream_id)],
            TestStreamId::Named => vec![self.stream_name.clone()],
        };

        command.push(match self.using_topic_id {
            TestTopicId::Numeric => format!("{}", self.topic_id),
            TestTopicId::Named => self.topic_name.clone(),
        });

        command.push(self.topic_new_name.clone());
        command.push(self.topic_new_compression_algorithm.to_string());

        if let MaxTopicSize::Custom(max_size) = &self.topic_new_max_size {
            command.push(format!("--max-topic-bytes={}", max_size));
        }

        if self.topic_new_replication_factor != 1 {
            command.push(format!(
                "--replication-factor={}",
                self.topic_new_replication_factor
            ));
        }

        if let Some(message_expiry) = &self.topic_new_message_expiry {
            command.extend(message_expiry.clone());
        }

        command
    }
}

#[async_trait]
impl IggyCmdTestCase for TestTopicUpdateCmd {
    async fn prepare_server_state(&mut self, client: &dyn Client) {
        let stream = client
            .create_stream(&self.stream_name, Some(self.stream_id))
            .await;
        assert!(stream.is_ok());

        let message_expiry = match &self.message_expiry {
            None => IggyExpiry::NeverExpire,
            Some(message_expiry) => {
                let duration: Duration =
                    *message_expiry.join(" ").parse::<HumanDuration>().unwrap();

                IggyExpiry::ExpireDuration(duration.into())
            }
        };

        let topic = client
            .create_topic(
                &self.stream_id.try_into().unwrap(),
                &self.topic_name,
                1,
                self.compression_algorithm,
                Some(self.replication_factor),
                Some(self.topic_id),
                message_expiry,
                self.max_topic_size,
            )
            .await;
        assert!(topic.is_ok());
    }

    fn get_command(&self) -> IggyCmdCommand {
        IggyCmdCommand::new()
            .arg("topic")
            .arg("update")
            .args(self.to_args())
            .with_env_credentials()
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

        let compression_algorithm = &self.topic_new_compression_algorithm;

        let message_expiry = (match &self.topic_new_message_expiry {
            Some(value) => value.join(" "),
            None => IggyExpiry::NeverExpire.to_string(),
        })
        .to_string();

        let max_topic_size = self.max_topic_size.to_string();

        let replication_factor = self.replication_factor;
        let new_topic_name = &self.topic_new_name;

        let expected_message = format!("Executing update topic with ID: {topic_id}, name: {new_topic_name}, \
                                message expiry: {message_expiry}, compression algorithm: {compression_algorithm}, max topic size: {max_topic_size}, \
                                replication factor: {replication_factor}, in stream with ID: {stream_id}\n\
                                Topic with ID: {topic_id} updated name: {new_topic_name}, updated message expiry: {message_expiry}, \
                                updated compression algorithm: {compression_algorithm} in stream with ID: {stream_id}\n");

        command_state.success().stdout(diff(expected_message));
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
        assert_eq!(topic_details.name, self.topic_new_name);
        assert_eq!(topic_details.id, self.topic_id);
        assert_eq!(topic_details.messages_count, 0);

        if self.topic_new_message_expiry.is_some() {
            let duration: Duration = *self
                .topic_new_message_expiry
                .clone()
                .unwrap()
                .join(" ")
                .parse::<HumanDuration>()
                .unwrap();
            assert_eq!(
                topic_details.message_expiry,
                IggyExpiry::ExpireDuration(duration.into())
            );
        }

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
    iggy_cmd_test.setup().await;
    iggy_cmd_test
        .execute_test(TestTopicUpdateCmd::new(
            1,
            String::from("main"),
            1,
            String::from("sync"),
            Default::default(),
            None,
            MaxTopicSize::ServerDefault,
            1,
            String::from("new_name"),
            CompressionAlgorithm::Gzip,
            None,
            MaxTopicSize::ServerDefault,
            1,
            TestStreamId::Numeric,
            TestTopicId::Numeric,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestTopicUpdateCmd::new(
            2,
            String::from("production"),
            3,
            String::from("topic"),
            Default::default(),
            None,
            MaxTopicSize::ServerDefault,
            1,
            String::from("testing"),
            CompressionAlgorithm::Gzip,
            None,
            MaxTopicSize::ServerDefault,
            1,
            TestStreamId::Named,
            TestTopicId::Numeric,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestTopicUpdateCmd::new(
            3,
            String::from("testing"),
            5,
            String::from("development"),
            Default::default(),
            None,
            MaxTopicSize::ServerDefault,
            1,
            String::from("development"),
            CompressionAlgorithm::Gzip,
            None,
            MaxTopicSize::ServerDefault,
            1,
            TestStreamId::Numeric,
            TestTopicId::Named,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestTopicUpdateCmd::new(
            2,
            String::from("other"),
            2,
            String::from("probe"),
            Default::default(),
            None,
            MaxTopicSize::ServerDefault,
            1,
            String::from("development"),
            CompressionAlgorithm::Gzip,
            Some(vec![
                String::from("1day"),
                String::from("1h"),
                String::from("1m"),
                String::from("1s"),
            ]),
            MaxTopicSize::ServerDefault,
            1,
            TestStreamId::Numeric,
            TestTopicId::Numeric,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestTopicUpdateCmd::new(
            3,
            String::from("stream"),
            1,
            String::from("testing"),
            Default::default(),
            Some(vec![String::from("1s")]),
            MaxTopicSize::ServerDefault,
            1,
            String::from("testing"),
            CompressionAlgorithm::Gzip,
            Some(vec![String::from("1m 6s")]),
            MaxTopicSize::ServerDefault,
            1,
            TestStreamId::Numeric,
            TestTopicId::Numeric,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestTopicUpdateCmd::new(
            4,
            String::from("testing"),
            2,
            String::from("testing"),
            Default::default(),
            Some(vec![
                String::from("1s"),
                String::from("1m"),
                String::from("1h"),
            ]),
            MaxTopicSize::ServerDefault,
            1,
            String::from("testing"),
            CompressionAlgorithm::Gzip,
            None,
            MaxTopicSize::ServerDefault,
            1,
            TestStreamId::Numeric,
            TestTopicId::Named,
        ))
        .await;
}

#[tokio::test]
#[parallel]
pub async fn should_help_match() {
    let mut iggy_cmd_test = IggyCmdTest::help_message();

    iggy_cmd_test
        .execute_test_for_help_command(TestHelpCmd::new(
            vec!["topic", "update", "--help"],
            format!(
                r#"Update topic name, compression algorithm and message expiry time for given topic ID in given stream ID

Stream ID can be specified as a stream name or ID
Topic ID can be specified as a topic name or ID

Examples
 iggy update 1 1 sensor3 none
 iggy update prod sensor3 old-sensor none
 iggy update test debugs ready gzip 15days
 iggy update 1 1 new-name gzip
 iggy update 1 2 new-name none 1day 1hour 1min 1sec

{USAGE_PREFIX} topic update [OPTIONS] <STREAM_ID> <TOPIC_ID> <NAME> <COMPRESSION_ALGORITHM> [MESSAGE_EXPIRY]...

Arguments:
  <STREAM_ID>
          Stream ID to update topic
{CLAP_INDENT}
          Stream ID can be specified as a stream name or ID

  <TOPIC_ID>
          Topic ID to update
{CLAP_INDENT}
          Topic ID can be specified as a topic name or ID

  <NAME>
          New name for the topic

  <COMPRESSION_ALGORITHM>
          Compression algorithm for the topic, set to "none" for no compression

  [MESSAGE_EXPIRY]...
          New message expiry time in human-readable format like 15days 2min 2s
{CLAP_INDENT}
          ("unlimited" or skipping parameter causes removal of expiry parameter in topic)

Options:
  -m, --max-topic-size <MAX_TOPIC_SIZE>
          New max topic size
{CLAP_INDENT}
          ("unlimited" or skipping parameter causes removal of max topic size parameter in topic)
          Can't be lower than segment size in the config.
{CLAP_INDENT}
          [default: unlimited]

  -r, --replication-factor <REPLICATION_FACTOR>
          New replication factor for the topic
{CLAP_INDENT}
          [default: 1]

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
            vec!["topic", "update", "-h"],
            format!(
                r#"Update topic name, compression algorithm and message expiry time for given topic ID in given stream ID

{USAGE_PREFIX} topic update [OPTIONS] <STREAM_ID> <TOPIC_ID> <NAME> <COMPRESSION_ALGORITHM> [MESSAGE_EXPIRY]...

Arguments:
  <STREAM_ID>              Stream ID to update topic
  <TOPIC_ID>               Topic ID to update
  <NAME>                   New name for the topic
  <COMPRESSION_ALGORITHM>  Compression algorithm for the topic, set to "none" for no compression
  [MESSAGE_EXPIRY]...      New message expiry time in human-readable format like 15days 2min 2s

Options:
  -m, --max-topic-size <MAX_TOPIC_SIZE>          New max topic size [default: unlimited]
  -r, --replication-factor <REPLICATION_FACTOR>  New replication factor for the topic [default: 1]
  -h, --help                                     Print help (see more with '--help')
"#,
            ),
        ))
        .await;
}
