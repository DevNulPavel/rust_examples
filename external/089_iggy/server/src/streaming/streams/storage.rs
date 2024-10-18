use crate::state::system::StreamState;
use crate::streaming::storage::StreamStorage;
use crate::streaming::streams::stream::Stream;
use crate::streaming::topics::topic::Topic;
use async_trait::async_trait;
use futures::future::join_all;
use iggy::error::IggyError;
use iggy::utils::timestamp::IggyTimestamp;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::fs::create_dir;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

#[derive(Debug)]
pub struct FileStreamStorage;

unsafe impl Send for FileStreamStorage {}
unsafe impl Sync for FileStreamStorage {}

#[derive(Debug, Serialize, Deserialize)]
struct StreamData {
    name: String,
    created_at: IggyTimestamp,
}

#[async_trait]
impl StreamStorage for FileStreamStorage {
    async fn load(&self, stream: &mut Stream, mut state: StreamState) -> Result<(), IggyError> {
        info!("Loading stream with ID: {} from disk...", stream.stream_id);
        if !Path::new(&stream.path).exists() {
            return Err(IggyError::StreamIdNotFound(stream.stream_id));
        }

        let mut unloaded_topics = Vec::new();
        let dir_entries = fs::read_dir(&stream.topics_path).await;
        if dir_entries.is_err() {
            return Err(IggyError::CannotReadTopics(stream.stream_id));
        }

        let mut dir_entries = dir_entries.unwrap();
        while let Some(dir_entry) = dir_entries.next_entry().await.unwrap_or(None) {
            let name = dir_entry.file_name().into_string().unwrap();
            let topic_id = name.parse::<u32>();
            if topic_id.is_err() {
                error!("Invalid topic ID file with name: '{}'.", name);
                continue;
            }

            let topic_id = topic_id.unwrap();
            let topic_state = state.topics.get(&topic_id);
            if topic_state.is_none() {
                let stream_id = stream.stream_id;
                error!("Topic with ID: '{topic_id}' for stream with ID: '{stream_id}' was not found in state, but exists on disk and will be removed.");
                if let Err(error) = fs::remove_dir_all(&dir_entry.path()).await {
                    error!("Cannot remove topic directory: {error}");
                } else {
                    warn!("Topic with ID: '{topic_id}' for stream with ID: '{stream_id}' was removed.");
                }
                continue;
            }

            let topic_state = topic_state.unwrap();
            let topic = Topic::empty(
                stream.stream_id,
                topic_id,
                &topic_state.name,
                stream.size_bytes.clone(),
                stream.messages_count.clone(),
                stream.segments_count.clone(),
                stream.config.clone(),
                stream.storage.clone(),
            );
            unloaded_topics.push(topic);
        }

        let state_topic_ids = state.topics.keys().copied().collect::<HashSet<u32>>();
        let unloaded_topic_ids = unloaded_topics
            .iter()
            .map(|topic| topic.topic_id)
            .collect::<HashSet<u32>>();
        let missing_ids = state_topic_ids
            .difference(&unloaded_topic_ids)
            .copied()
            .collect::<HashSet<u32>>();
        if missing_ids.is_empty() {
            info!(
                "All topics for stream with ID: '{}' found on disk were found in state.",
                stream.stream_id
            );
        } else {
            error!("Topics with IDs: '{missing_ids:?}' for stream with ID: '{}' were not found on disk.", stream.stream_id);
            if !stream.config.recovery.recreate_missing_state {
                warn!("Recreating missing state in recovery config is disabled, missing topics will not be created for stream with ID: '{}'.", stream.stream_id);
                return Err(IggyError::MissingTopics(stream.stream_id));
            }

            info!(
                "Recreating missing state in recovery config is enabled, missing topics will be created for stream with ID: '{}'.",
                stream.stream_id
            );
            for topic_id in missing_ids {
                let topic_state = state.topics.get(&topic_id).unwrap();
                let topic = Topic::empty(
                    stream.stream_id,
                    topic_id,
                    &topic_state.name,
                    stream.size_bytes.clone(),
                    stream.messages_count.clone(),
                    stream.segments_count.clone(),
                    stream.config.clone(),
                    stream.storage.clone(),
                );
                topic.persist().await?;
                unloaded_topics.push(topic);
                info!(
                    "Created missing topic with ID: '{}', name: {}, for stream with ID: '{}'.",
                    topic_id, &topic_state.name, stream.stream_id
                );
            }
        }

        let loaded_topics = Arc::new(Mutex::new(Vec::new()));
        let mut load_topics = Vec::new();
        for mut topic in unloaded_topics {
            let loaded_topics = loaded_topics.clone();
            let topic_state = state.topics.remove(&topic.topic_id).unwrap();
            let load_topic = tokio::spawn(async move {
                match topic.load(topic_state).await {
                    Ok(_) => loaded_topics.lock().await.push(topic),
                    Err(error) => error!(
                        "Failed to load topic with ID: {} for stream with ID: {}. Error: {}",
                        topic.topic_id, topic.stream_id, error
                    ),
                }
            });
            load_topics.push(load_topic);
        }

        join_all(load_topics).await;
        for topic in loaded_topics.lock().await.drain(..) {
            if stream.topics.contains_key(&topic.topic_id) {
                error!(
                    "Topic with ID: '{}' already exists for stream with ID: {}.",
                    &topic.topic_id, &stream.stream_id
                );
                continue;
            }

            if stream.topics_ids.contains_key(&topic.name) {
                error!(
                    "Topic with name: '{}' already exists for stream with ID: {}.",
                    &topic.name, &stream.stream_id
                );
                continue;
            }

            stream.topics_ids.insert(topic.name.clone(), topic.topic_id);
            stream.topics.insert(topic.topic_id, topic);
        }

        info!(
            "Loaded stream: '{}' with ID: {} from disk.",
            &stream.name, &stream.stream_id
        );

        Ok(())
    }

    async fn save(&self, stream: &Stream) -> Result<(), IggyError> {
        if !Path::new(&stream.path).exists() && create_dir(&stream.path).await.is_err() {
            return Err(IggyError::CannotCreateStreamDirectory(
                stream.stream_id,
                stream.path.clone(),
            ));
        }

        if !Path::new(&stream.topics_path).exists()
            && create_dir(&stream.topics_path).await.is_err()
        {
            return Err(IggyError::CannotCreateTopicsDirectory(
                stream.stream_id,
                stream.topics_path.clone(),
            ));
        }

        info!("Saved stream with ID: {}.", stream.stream_id);

        Ok(())
    }

    async fn delete(&self, stream: &Stream) -> Result<(), IggyError> {
        info!("Deleting stream with ID: {}...", stream.stream_id);
        if fs::remove_dir_all(&stream.path).await.is_err() {
            return Err(IggyError::CannotDeleteStreamDirectory(stream.stream_id));
        }
        info!("Deleted stream with ID: {}.", stream.stream_id);
        Ok(())
    }
}
