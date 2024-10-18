use crate::cli::common::{
    IggyCmdCommand, IggyCmdTest, IggyCmdTestCase, TestHelpCmd, TestStreamId, CLAP_INDENT,
    USAGE_PREFIX,
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

struct TestTopicCreateCmd {
    stream_id: u32,
    stream_name: String,
    topic_id: Option<u32>,
    topic_name: String,
    partitions_count: u32,
    compression_algorithm: CompressionAlgorithm,
    message_expiry: Option<Vec<String>>,
    max_topic_size: MaxTopicSize,
    replication_factor: u8,
    using_identifier: TestStreamId,
}

impl TestTopicCreateCmd {
    #[allow(clippy::too_many_arguments)]
    fn new(
        stream_id: u32,
        stream_name: String,
        topic_id: Option<u32>,
        topic_name: String,
        partitions_count: u32,
        compression_algorithm: CompressionAlgorithm,
        message_expiry: Option<Vec<String>>,
        max_topic_size: MaxTopicSize,
        replication_factor: u8,
        using_identifier: TestStreamId,
    ) -> Self {
        Self {
            stream_id,
            stream_name,
            topic_id,
            topic_name,
            partitions_count,
            compression_algorithm,
            message_expiry,
            max_topic_size,
            replication_factor,
            using_identifier,
        }
    }

    fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        if let Some(topic_id) = self.topic_id {
            args.push("-t".to_string());
            args.push(format!("{}", topic_id));
        };

        match self.using_identifier {
            TestStreamId::Numeric => args.extend(vec![format!("{}", self.stream_id)]),
            TestStreamId::Named => args.extend(vec![self.stream_name.clone()]),
        };

        args.push(self.topic_name.clone());
        args.push(format!("{}", self.partitions_count));
        args.push(format!("{}", self.compression_algorithm));
        args.extend(self.message_expiry.clone().unwrap_or_default());

        args
    }
}

#[async_trait]
impl IggyCmdTestCase for TestTopicCreateCmd {
    async fn prepare_server_state(&mut self, client: &dyn Client) {
        let stream = client
            .create_stream(&self.stream_name, Some(self.stream_id))
            .await;
        assert!(stream.is_ok());
    }

    fn get_command(&self) -> IggyCmdCommand {
        IggyCmdCommand::new()
            .arg("topic")
            .arg("create")
            .args(self.to_args())
            .with_env_credentials()
    }

    fn verify_command(&self, command_state: Assert) {
        let stream_id = match self.using_identifier {
            TestStreamId::Numeric => format!("{}", self.stream_id),
            TestStreamId::Named => self.stream_name.clone(),
        };
        let partitions_count = self.partitions_count;
        let topic_id = match self.topic_id {
            Some(topic_id) => format!("ID: {}", topic_id),
            None => "ID auto incremented".to_string(),
        };
        let topic_name = &self.topic_name;
        let compression_algorithm = &self.compression_algorithm;
        let message_expiry = (match &self.message_expiry {
            Some(value) => value.join(" "),
            None => IggyExpiry::NeverExpire.to_string(),
        })
        .to_string();

        let max_topic_size = self.max_topic_size.to_string();

        let replication_factor = self.replication_factor;

        let message = format!(
            "Executing create topic with name: {topic_name}, {topic_id}, message expiry: {message_expiry}, compression algorithm: {compression_algorithm}, \
            max topic size: {max_topic_size}, replication factor: {replication_factor} in stream with ID: {stream_id}\n\
            Topic with name: {topic_name}, {topic_id}, partitions count: {partitions_count}, compression algorithm: {compression_algorithm}, message expiry: {message_expiry}, \
            max topic size: {max_topic_size}, replication factor: {replication_factor} created in stream with ID: {stream_id}\n",
        );

        command_state.success().stdout(diff(message));
    }

    async fn verify_server_state(&self, client: &dyn Client) {
        let topic = client
            .get_topic(
                &self.stream_id.try_into().unwrap(),
                &self.topic_name.clone().try_into().unwrap(),
            )
            .await;
        assert!(topic.is_ok());
        let topic_details = topic.unwrap().expect("Topic not found");
        assert_eq!(topic_details.name, self.topic_name);
        assert_eq!(topic_details.partitions_count, self.partitions_count);
        assert_eq!(topic_details.messages_count, 0);
        if let Some(topic_id) = self.topic_id {
            assert_eq!(topic_details.id, topic_id);
        }

        if self.message_expiry.is_some() {
            let duration: Duration = *self
                .message_expiry
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

        let delete_topic = client
            .delete_topic(
                &self.stream_id.try_into().unwrap(),
                &self.topic_name.clone().try_into().unwrap(),
            )
            .await;
        assert!(delete_topic.is_ok());

        let delete_stream = client
            .delete_stream(&self.stream_id.try_into().unwrap())
            .await;
        assert!(delete_stream.is_ok());
    }
}

#[tokio::test]
#[parallel]
pub async fn should_be_successful() {
    let mut iggy_cmd_test = IggyCmdTest::default();

    iggy_cmd_test.setup().await;
    iggy_cmd_test
        .execute_test(TestTopicCreateCmd::new(
            1,
            String::from("main"),
            None,
            String::from("sync"),
            1,
            Default::default(),
            None,
            MaxTopicSize::ServerDefault,
            1,
            TestStreamId::Numeric,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestTopicCreateCmd::new(
            2,
            String::from("testing"),
            Some(2),
            String::from("topic"),
            5,
            Default::default(),
            None,
            MaxTopicSize::ServerDefault,
            1,
            TestStreamId::Named,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestTopicCreateCmd::new(
            3,
            String::from("prod"),
            None,
            String::from("named"),
            1,
            Default::default(),
            Some(vec![String::from("3days"), String::from("5s")]),
            MaxTopicSize::ServerDefault,
            1,
            TestStreamId::Named,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestTopicCreateCmd::new(
            4,
            String::from("big"),
            Some(1),
            String::from("probe"),
            2,
            Default::default(),
            Some(vec![
                String::from("1day"),
                String::from("1h"),
                String::from("1m"),
                String::from("1s"),
            ]),
            MaxTopicSize::ServerDefault,
            1,
            TestStreamId::Numeric,
        ))
        .await;
}

#[tokio::test]
#[parallel]
pub async fn should_help_match() {
    let mut iggy_cmd_test = IggyCmdTest::help_message();

    iggy_cmd_test
        .execute_test_for_help_command(TestHelpCmd::new(
            vec!["topic", "create", "--help"],
            format!(
                r#"Create topic with given name, number of partitions, compression algorithm and expiry time for given stream ID

Stream ID can be specified as a stream name or ID
If topic ID is not provided then the server will automatically assign it

Examples
 iggy topic create 1 sensor1 2 gzip 15days
 iggy topic create prod sensor2 2 none
 iggy topic create test debugs 2 gzip 1day 1hour 1min 1sec
 iggy topic create -t 3 1 sensor3 2 none unlimited

{USAGE_PREFIX} topic create [OPTIONS] <STREAM_ID> <NAME> <PARTITIONS_COUNT> <COMPRESSION_ALGORITHM> [MESSAGE_EXPIRY]...

Arguments:
  <STREAM_ID>
          Stream ID to create topic
{CLAP_INDENT}
          Stream ID can be specified as a stream name or ID

  <NAME>
          Name of the topic

  <PARTITIONS_COUNT>
          Number of partitions inside the topic

  <COMPRESSION_ALGORITHM>
          Compression algorithm for the topic, set to "none" for no compression

  [MESSAGE_EXPIRY]...
          Message expiry time in human-readable format like 15days 2min 2s
{CLAP_INDENT}
          ("unlimited" or skipping parameter disables message expiry functionality in topic)

Options:
  -t, --topic-id <TOPIC_ID>
          Topic ID to create

  -m, --max-topic-size <MAX_TOPIC_SIZE>
          Max topic size
{CLAP_INDENT}
          ("unlimited" or skipping parameter disables max topic size functionality in topic)
          Can't be lower than segment size in the config.
{CLAP_INDENT}
          [default: unlimited]

  -r, --replication-factor <REPLICATION_FACTOR>
          Replication factor for the topic
{CLAP_INDENT}
          [default: 1]

  -h, --help
          Print help (see a summary with '-h')
"#,
        )))
        .await;
}

#[tokio::test]
#[parallel]
pub async fn should_short_help_match() {
    let mut iggy_cmd_test = IggyCmdTest::default();

    iggy_cmd_test
        .execute_test_for_help_command(TestHelpCmd::new(
            vec!["topic", "create", "-h"],
            format!(
                r#"Create topic with given name, number of partitions, compression algorithm and expiry time for given stream ID

{USAGE_PREFIX} topic create [OPTIONS] <STREAM_ID> <NAME> <PARTITIONS_COUNT> <COMPRESSION_ALGORITHM> [MESSAGE_EXPIRY]...

Arguments:
  <STREAM_ID>              Stream ID to create topic
  <NAME>                   Name of the topic
  <PARTITIONS_COUNT>       Number of partitions inside the topic
  <COMPRESSION_ALGORITHM>  Compression algorithm for the topic, set to "none" for no compression
  [MESSAGE_EXPIRY]...      Message expiry time in human-readable format like 15days 2min 2s

Options:
  -t, --topic-id <TOPIC_ID>                      Topic ID to create
  -m, --max-topic-size <MAX_TOPIC_SIZE>          Max topic size [default: unlimited]
  -r, --replication-factor <REPLICATION_FACTOR>  Replication factor for the topic [default: 1]
  -h, --help                                     Print help (see more with '--help')
"#,
            ),
        ))
        .await;
}
