use crate::server::scenarios::{
    cleanup, create_client, join_consumer_group, CONSUMER_GROUP_ID, CONSUMER_GROUP_NAME,
    PARTITIONS_COUNT, STREAM_ID, STREAM_NAME, TOPIC_ID, TOPIC_NAME, USERNAME_1, USERNAME_2,
    USERNAME_3,
};
use iggy::client::{ConsumerGroupClient, StreamClient, SystemClient, TopicClient};
use iggy::clients::client::IggyClient;
use iggy::compression::compression_algorithm::CompressionAlgorithm;
use iggy::identifier::Identifier;
use iggy::models::client_info::ClientInfoDetails;
use iggy::models::consumer_group::ConsumerGroupDetails;
use iggy::utils::expiry::IggyExpiry;
use iggy::utils::topic_size::MaxTopicSize;
use integration::test_server::{
    assert_clean_system, create_user, login_root, login_user, ClientFactory,
};

pub async fn run(client_factory: &dyn ClientFactory) {
    let system_client = create_client(client_factory).await;

    let client1 = create_client(client_factory).await;
    let client2 = create_client(client_factory).await;
    let client3 = create_client(client_factory).await;

    login_root(&system_client).await;

    // 1. Create the stream
    system_client
        .create_stream(STREAM_NAME, Some(STREAM_ID))
        .await
        .unwrap();

    // 2. Create the topic
    system_client
        .create_topic(
            &Identifier::numeric(STREAM_ID).unwrap(),
            TOPIC_NAME,
            PARTITIONS_COUNT,
            CompressionAlgorithm::default(),
            None,
            Some(TOPIC_ID),
            IggyExpiry::NeverExpire,
            MaxTopicSize::ServerDefault,
        )
        .await
        .unwrap();

    // 3. Create the consumer group
    system_client
        .create_consumer_group(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            CONSUMER_GROUP_NAME,
            Some(CONSUMER_GROUP_ID),
        )
        .await
        .unwrap();

    // 4. Create the users for all clients
    create_user(&system_client, USERNAME_1).await;
    create_user(&system_client, USERNAME_2).await;
    create_user(&system_client, USERNAME_3).await;

    // 5. Login all the clients
    login_user(&client1, USERNAME_1).await;
    login_user(&client2, USERNAME_2).await;
    login_user(&client3, USERNAME_3).await;

    // 5. Join the consumer group by client 1
    join_consumer_group(&client1).await;

    // 5. Get client1 info and validate that it contains the single consumer group
    let client1_info = get_me_and_validate_consumer_groups(&client1).await;

    // 6. Validate that the consumer group has 1 member and this member has all partitions assigned
    let consumer_group = get_consumer_group_and_validate_members(&system_client, 1).await;
    let member = &consumer_group.members[0];
    assert_eq!(member.id, client1_info.client_id);
    assert_eq!(member.partitions_count, PARTITIONS_COUNT);
    assert_eq!(member.partitions.len() as u32, PARTITIONS_COUNT);

    // 7. Join the consumer group by client 2
    join_consumer_group(&client2).await;

    // 8. Validate that client 2 contains the single consumer group
    get_me_and_validate_consumer_groups(&client2).await;

    // 9. Validate that the consumer group has 2 members and partitions are distributed between them
    let consumer_group = get_consumer_group_and_validate_members(&system_client, 2).await;
    let member1 = &consumer_group.members[0];
    let member2 = &consumer_group.members[1];
    assert!(member1.partitions_count >= 1 && member1.partitions_count < PARTITIONS_COUNT);
    assert!(member2.partitions_count >= 1 && member2.partitions_count < PARTITIONS_COUNT);
    assert_eq!(
        member1.partitions_count + member2.partitions_count,
        PARTITIONS_COUNT
    );

    // 10. Join the consumer group by client 3
    join_consumer_group(&client3).await;

    // 11. Validate that client 3 contains the single consumer group
    get_me_and_validate_consumer_groups(&client3).await;

    // 12. Validate that the consumer group has 3 members and partitions are equally distributed between them
    let consumer_group = get_consumer_group_and_validate_members(&system_client, 3).await;
    let member1 = &consumer_group.members[0];
    let member2 = &consumer_group.members[1];
    let member3 = &consumer_group.members[2];
    assert_eq!(member1.partitions_count, 1);
    assert_eq!(member2.partitions_count, 1);
    assert_eq!(member3.partitions_count, 1);
    assert_ne!(member1.partitions[0], member2.partitions[0]);
    assert_ne!(member1.partitions[0], member3.partitions[0]);
    assert_ne!(member2.partitions[0], member3.partitions[0]);

    cleanup(&system_client, true).await;
    assert_clean_system(&system_client).await;
}

async fn get_me_and_validate_consumer_groups(client: &IggyClient) -> ClientInfoDetails {
    let client_info = client.get_me().await.unwrap();

    assert!(client_info.client_id > 0);
    assert_eq!(client_info.consumer_groups_count, 1);
    assert_eq!(client_info.consumer_groups.len(), 1);

    let consumer_group = &client_info.consumer_groups[0];
    assert_eq!(consumer_group.stream_id, STREAM_ID);
    assert_eq!(consumer_group.topic_id, TOPIC_ID);
    assert_eq!(consumer_group.group_id, CONSUMER_GROUP_ID);

    client_info
}

async fn get_consumer_group_and_validate_members(
    client: &IggyClient,
    members_count: u32,
) -> ConsumerGroupDetails {
    let consumer_group = client
        .get_consumer_group(
            &Identifier::numeric(STREAM_ID).unwrap(),
            &Identifier::numeric(TOPIC_ID).unwrap(),
            &Identifier::numeric(CONSUMER_GROUP_ID).unwrap(),
        )
        .await
        .unwrap()
        .expect("Failed to get consumer group");

    assert_eq!(consumer_group.id, CONSUMER_GROUP_ID);
    assert_eq!(consumer_group.name, CONSUMER_GROUP_NAME);
    assert_eq!(consumer_group.partitions_count, PARTITIONS_COUNT);
    assert_eq!(consumer_group.members_count, members_count);
    assert_eq!(consumer_group.members.len() as u32, members_count);

    consumer_group
}
