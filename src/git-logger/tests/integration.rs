use git_logger::timeline::ImmutableTimeline;
use tempfile::TempDir;

#[tokio::test]
async fn test_timeline_init_and_snapshot() {
    let tmp = TempDir::new().unwrap();
    let timeline = ImmutableTimeline::new(tmp.path());
    timeline.init_repo().await.unwrap();
    assert!(tmp.path().join(".git").exists());
}

#[tokio::test]
async fn test_timeline_multiple_generations() {
    let tmp = TempDir::new().unwrap();
    let timeline = ImmutableTimeline::new(tmp.path());
    timeline.init_repo().await.unwrap();

    for i in 1..=3 {
        std::fs::write(tmp.path().join(format!("f{}.txt", i)), b"data").unwrap();
        let entry = timeline.snapshot_repo().await.unwrap();
        assert_eq!(entry.snapshot.generation, i as u64);
        assert!(!entry.snapshot.root_hash.is_empty());
    }
    assert_eq!(timeline.current_generation().await, 3);
}

#[tokio::test]
async fn test_timeline_is_repo_clean() {
    let tmp = TempDir::new().unwrap();
    let timeline = ImmutableTimeline::new(tmp.path());
    timeline.init_repo().await.unwrap();
    assert!(timeline.is_repo_clean().await);
}
