use crate::archiver::DiskArchiverSetup;
use server::archiver::Archiver;
use server::server_error::ServerError;
use server::streaming::utils::file;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn should_init_base_archiver_directory() {
    let setup = DiskArchiverSetup::init().await;
    let archiver = setup.archiver();
    let result = archiver.init().await;
    assert!(result.is_ok());
    let path = Path::new(&setup.archive_path);
    assert!(path.exists());
}

#[tokio::test]
async fn should_archive_file_on_disk_by_making_a_copy_of_original_file() {
    let setup = DiskArchiverSetup::init().await;
    let archiver = setup.archiver();
    let content = "hello world";
    let file_to_archive_path = format!("{}/file_to_archive", setup.base_path);
    create_file(&file_to_archive_path, content).await;
    let files_to_archive = vec![file_to_archive_path.as_ref()];

    let result = archiver.archive(&files_to_archive, None).await;
    assert!(result.is_ok());
    let archived_file_path = format!("{}/{}", setup.archive_path, file_to_archive_path);
    assert_archived_file(&file_to_archive_path, &archived_file_path, content).await;
}

#[tokio::test]
async fn should_archive_file_on_disk_within_additional_base_directory() {
    let setup = DiskArchiverSetup::init().await;
    let archiver = setup.archiver();
    let base_directory = "base";
    let content = "hello world";
    let file_to_archive_path = format!("{}/file_to_archive", setup.base_path);
    create_file(&file_to_archive_path, content).await;
    let files_to_archive = vec![file_to_archive_path.as_ref()];

    let result = archiver
        .archive(&files_to_archive, Some(base_directory.to_string()))
        .await;
    assert!(result.is_ok());
    let archived_file_path = format!(
        "{}/{base_directory}/{}",
        setup.archive_path, file_to_archive_path
    );
    assert_archived_file(&file_to_archive_path, &archived_file_path, content).await;
}

#[tokio::test]
async fn should_return_true_when_file_is_archived() {
    let setup = DiskArchiverSetup::init().await;
    let archiver = setup.archiver();
    let content = "hello world";
    let file_to_archive_path = format!("{}/file_to_archive", setup.base_path);
    create_file(&file_to_archive_path, content).await;
    let files_to_archive = vec![file_to_archive_path.as_ref()];
    archiver.archive(&files_to_archive, None).await.unwrap();

    let is_archived = archiver.is_archived(&file_to_archive_path, None).await;
    assert!(is_archived.is_ok());
    assert!(is_archived.unwrap());
}

#[tokio::test]
async fn should_return_false_when_file_is_not_archived() {
    let setup = DiskArchiverSetup::init().await;
    let archiver = setup.archiver();
    let content = "hello world";
    let file_to_archive_path = format!("{}/file_to_archive", setup.base_path);
    create_file(&file_to_archive_path, content).await;

    let is_archived = archiver.is_archived(&file_to_archive_path, None).await;
    assert!(is_archived.is_ok());
    assert!(!is_archived.unwrap());
}

#[tokio::test]
async fn should_fail_when_file_to_archive_does_not_exist() {
    let setup = DiskArchiverSetup::init().await;
    let archiver = setup.archiver();
    let file_to_archive_path = "invalid_file_to_archive";
    let files_to_archive = vec![file_to_archive_path];
    let result = archiver.archive(&files_to_archive, None).await;

    assert!(result.is_err());
    let error = result.err().unwrap();
    assert!(matches!(error, ServerError::FileToArchiveNotFound(_)));
}

async fn create_file(path: &str, content: &str) {
    let mut file = file::overwrite(path).await.unwrap();
    file.write_all(content.as_bytes()).await.unwrap();
}

async fn assert_archived_file(file_to_archive_path: &str, archived_file_path: &str, content: &str) {
    assert!(Path::new(&file_to_archive_path).exists());
    assert!(Path::new(&archived_file_path).exists());
    let archived_file = file::open(archived_file_path).await;
    assert!(archived_file.is_ok());
    let mut archived_file = archived_file.unwrap();
    let mut archived_file_content = String::new();
    archived_file
        .read_to_string(&mut archived_file_content)
        .await
        .unwrap();
    assert_eq!(content, archived_file_content);
}
