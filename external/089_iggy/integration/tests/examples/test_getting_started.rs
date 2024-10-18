use super::verify_stdout_contains_expected_logs;
use crate::examples::{IggyExampleTest, IggyExampleTestCase};
use serial_test::parallel;

static EXPECTED_CONSUMER_OUTPUT: [&str; 6] = [
    "Messages will be consumed from stream: 1, topic: 1, partition: 1 with interval 500ms.",
    "Handling message at offset: 0, payload: message-1...",
    "Handling message at offset: 1, payload: message-2...",
    "Handling message at offset: 2, payload: message-3...",
    "Handling message at offset: 3, payload: message-4...",
    "Handling message at offset: 4, payload: message-5...",
];

struct TestGettingStarted<'a> {
    expected_producer_output: Vec<&'a str>,
    expected_consumer_output: Vec<&'a str>,
}

impl<'a> IggyExampleTestCase for TestGettingStarted<'a> {
    fn verify_log_output(&self, producer_stdout: &str, consumer_stdout: &str) {
        verify_stdout_contains_expected_logs(
            producer_stdout,
            consumer_stdout,
            &self.expected_producer_output,
            &self.expected_consumer_output,
        );
    }

    fn verify_message_output(&self, _producer_stdout: &str, _consumer_stdout: &str) {}
}

#[tokio::test]
#[parallel]
async fn should_succeed_with_no_existing_stream_or_topic() {
    let mut iggy_example_test = IggyExampleTest::new("getting-started");
    iggy_example_test.setup(false).await;

    iggy_example_test
        .execute_test(TestGettingStarted {
            expected_producer_output: vec![
                "Stream was created.",
                "Topic was created.",
                "Messages will be sent to stream: 1, topic: 1, partition: 1 with interval 500ms.",
                "Sent 10 message(s).",
            ],
            expected_consumer_output: EXPECTED_CONSUMER_OUTPUT.to_vec(),
        })
        .await;
}

#[tokio::test]
#[parallel]
async fn should_succeed_with_preexisting_stream_and_topic() {
    let mut iggy_example_test = IggyExampleTest::new("getting-started");
    iggy_example_test.setup(true).await;

    iggy_example_test
        .execute_test(TestGettingStarted {
            expected_producer_output: vec![
                "Stream already exists and will not be created again.",
                "Topic already exists and will not be created again.",
                "Sent 10 message(s).",
            ],
            expected_consumer_output: EXPECTED_CONSUMER_OUTPUT.to_vec(),
        })
        .await;
}
