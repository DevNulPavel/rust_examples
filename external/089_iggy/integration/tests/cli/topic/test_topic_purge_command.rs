use crate::cli::common::{
    IggyCmdCommand, IggyCmdTest, IggyCmdTestCase, TestHelpCmd, TestStreamId, TestTopicId,
    CLAP_INDENT, USAGE_PREFIX,
};
use assert_cmd::assert::Assert;
use async_trait::async_trait;
use iggy::client::Client;
use iggy::messages::send_messages::{Message, Partitioning};
use iggy::utils::expiry::IggyExpiry;
use iggy::utils::topic_size::MaxTopicSize;
use predicates::str::diff;
use serial_test::parallel;
use std::str::FromStr;

struct TestTopicPurgeCmd {
    stream_id: u32,
    stream_name: String,
    topic_id: u32,
    topic_name: String,
    using_stream_id: TestStreamId,
    using_topic_id: TestTopicId,
}

impl TestTopicPurgeCmd {
    fn new(
        stream_id: u32,
        stream_name: String,
        topic_id: u32,
        topic_name: String,
        using_stream_id: TestStreamId,
        using_topic_id: TestTopicId,
    ) -> Self {
        Self {
            stream_id,
            stream_name,
            topic_id,
            topic_name,
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

        command
    }
}

#[async_trait]
impl IggyCmdTestCase for TestTopicPurgeCmd {
    async fn prepare_server_state(&mut self, client: &dyn Client) {
        let stream = client
            .create_stream(&self.stream_name, Some(self.stream_id))
            .await;
        assert!(stream.is_ok());

        let topic = client
            .create_topic(
                &self.stream_id.try_into().unwrap(),
                &self.topic_name,
                10,
                Default::default(),
                None,
                Some(self.topic_id),
                IggyExpiry::NeverExpire,
                MaxTopicSize::ServerDefault,
            )
            .await;
        assert!(topic.is_ok());

        let mut messages = (1..100)
            .map(|n| format!("message {}", n))
            .filter_map(|s| Message::from_str(s.as_str()).ok())
            .collect::<Vec<_>>();

        let send_status = client
            .send_messages(
                &self.stream_id.try_into().unwrap(),
                &self.topic_id.try_into().unwrap(),
                &Partitioning::default(),
                &mut messages,
            )
            .await;
        assert!(send_status.is_ok());

        let topic_state = client
            .get_topic(
                &self.stream_id.try_into().unwrap(),
                &self.topic_id.try_into().unwrap(),
            )
            .await;
        assert!(topic_state.is_ok());
        let topic_state = topic_state.unwrap().expect("Topic not found");
        assert!(topic_state.size > 0);
    }

    fn get_command(&self) -> IggyCmdCommand {
        IggyCmdCommand::new()
            .arg("topic")
            .arg("purge")
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

        let message = format!("Executing purge topic with ID: {} in stream with ID: {}\nTopic with ID: {} in stream with ID: {} purged\n",
                              topic_id, stream_id, topic_id,  stream_id);

        command_state.success().stdout(diff(message));
    }

    async fn verify_server_state(&self, client: &dyn Client) {
        let topic_state = client
            .get_topic(
                &self.stream_id.try_into().unwrap(),
                &self.topic_id.try_into().unwrap(),
            )
            .await;
        assert!(topic_state.is_ok());
        let topic_state = topic_state.unwrap().expect("Topic not found");
        assert_eq!(topic_state.size, 0);

        let topic_delete = client
            .delete_topic(
                &self.stream_id.try_into().unwrap(),
                &self.topic_id.try_into().unwrap(),
            )
            .await;
        assert!(topic_delete.is_ok());

        let stream_delete = client
            .delete_stream(&self.stream_id.try_into().unwrap())
            .await;
        assert!(stream_delete.is_ok());
    }
}

#[tokio::test]
#[parallel]
pub async fn should_be_successful() {
    let mut iggy_cmd_test = IggyCmdTest::default();

    iggy_cmd_test.setup().await;
    iggy_cmd_test
        .execute_test(TestTopicPurgeCmd::new(
            1,
            String::from("main"),
            1,
            String::from("sync"),
            TestStreamId::Numeric,
            TestTopicId::Numeric,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestTopicPurgeCmd::new(
            2,
            String::from("testing"),
            2,
            String::from("topic"),
            TestStreamId::Named,
            TestTopicId::Named,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestTopicPurgeCmd::new(
            3,
            String::from("prod"),
            1,
            String::from("named"),
            TestStreamId::Named,
            TestTopicId::Numeric,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestTopicPurgeCmd::new(
            4,
            String::from("big"),
            1,
            String::from("probe"),
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
            vec!["topic", "purge", "--help"],
            format!(
                r"Purge topic with given ID in given stream ID

Command removes all messages from given topic
Stream ID can be specified as a stream name or ID
Topic ID can be specified as a topic name or ID

Examples
 iggy topic purge 1 1
 iggy topic purge prod 2
 iggy topic purge test debugs
 iggy topic purge 2 debugs

{USAGE_PREFIX} topic purge <STREAM_ID> <TOPIC_ID>

Arguments:
  <STREAM_ID>
          Stream ID to purge topic
{CLAP_INDENT}
          Stream ID can be specified as a stream name or ID

  <TOPIC_ID>
          Topic ID to purge
{CLAP_INDENT}
          Topic ID can be specified as a topic name or ID

Options:
  -h, --help
          Print help (see a summary with '-h')
",
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
            vec!["topic", "purge", "-h"],
            format!(
                r#"Purge topic with given ID in given stream ID

{USAGE_PREFIX} topic purge <STREAM_ID> <TOPIC_ID>

Arguments:
  <STREAM_ID>  Stream ID to purge topic
  <TOPIC_ID>   Topic ID to purge

Options:
  -h, --help  Print help (see more with '--help')
"#,
            ),
        ))
        .await;
}
