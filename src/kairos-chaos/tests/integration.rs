use crate::*;

#[cfg(test)]
mod integration {
    use super::*;

    #[tokio::test]
    async fn test_score_starts_at_100() {
        let engine = ChaosEngine::new(ChaosConfig::default());
        assert_eq!(engine.get_score().await, 100);
    }

    #[tokio::test]
    async fn test_execute_rotate_logs() {
        let engine = ChaosEngine::new(ChaosConfig::default());
        let result = engine.execute(ChaosAction::RotateLogs, 0).await;
        assert!(result.is_ok());
        let event = result.unwrap();
        assert!(event.id.starts_with("chaos-"));
        assert_eq!(event.status, ChaosStatus::Succeeded);
    }

    #[tokio::test]
    async fn test_execute_memory_pressure() {
        let engine = ChaosEngine::new(ChaosConfig::default());
        let result = engine.execute(ChaosAction::MemoryPressure(50), 1).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_events() {
        let engine = ChaosEngine::new(ChaosConfig::default());
        for _ in 0..3 {
            let _ = engine.execute(ChaosAction::RotateLogs, 0).await;
        }
        let events = engine.get_events().await;
        assert_eq!(events.len(), 3);
    }

    #[tokio::test]
    async fn test_score_impact() {
        let engine = ChaosEngine::new(ChaosConfig::default());
        let event = engine.execute(ChaosAction::RotateLogs, 0).await.unwrap();
        assert!(event.score_impact.is_some());
        assert!(*engine.score.read().await < 100);
    }
}
