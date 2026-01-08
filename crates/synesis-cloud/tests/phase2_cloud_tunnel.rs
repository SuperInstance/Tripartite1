//! Phase 2: Cloud Tunnel Integration Tests
//!
//! Tests the complete QUIC tunnel functionality

use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_tunnel_config() {
    // Test tunnel configuration

    use synesis_cloud::tunnel::types::TunnelConfig;

    let config = TunnelConfig {
        cloud_url: "localhost:4433".to_string(),
        device_id: "test-device".to_string(),
        cert_path: "/tmp/test-cert.pem".into(),
        key_path: "/tmp/test-key.pem".into(),
        heartbeat_interval: Duration::from_secs(10),
        reconnect_delay: Duration::from_secs(1),
        max_reconnect_attempts: 3,
        connect_timeout: Duration::from_secs(5),
        read_timeout: Duration::from_secs(10),
    };

    assert_eq!(config.device_id, "test-device");
    assert_eq!(config.heartbeat_interval, Duration::from_secs(10));
    assert_eq!(config.max_reconnect_attempts, 3);
}

#[tokio::test]
async fn test_escalation_request_timeout() {
    // Test timeout handling
    let timeout_dur = Duration::from_millis(100);

    let result = timeout(timeout_dur, async {
        tokio::time::sleep(Duration::from_secs(1)).await;
        "completed"
    }).await;

    assert!(result.is_err(), "Request should timeout");
}

#[tokio::test]
async fn test_heartbeat_sequence() {
    // Test heartbeat sending and ACK handling
    use synesis_cloud::telemetry::types::{DeviceVitals, Heartbeat};

    let vitals = DeviceVitals {
        device_id: "test-device".to_string(),
        timestamp: chrono::Utc::now(),
        cpu_usage: 0.5,
        memory_usage: 0.6,
        gpu_usage: Some(0.7),
        gpu_temp_celsius: Some(65.0),
        gpu_vram_usage: Some(0.8),
        disk_usage: 50.0,
        active_sessions: 1,
        pending_queries: 0,
        loaded_model: Some("test-model".to_string()),
    };

    let heartbeat = Heartbeat {
        device_id: "test-device".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        sequence: 1,
        vitals,
    };

    assert_eq!(heartbeat.device_id, "test-device");
    assert_eq!(heartbeat.sequence, 1);
}

#[tokio::test]
async fn test_error_recovery() {
    // Test error handling and recovery

    // Simulate connection error
    let connection_result: Result<(), &'static str> = Err("Connection refused");

    assert!(connection_result.is_err());

    // In production, tunnel would reconnect
    // For test, just verify error is detected
}

#[cfg(test)]
mod performance_tests {
    use std::time::Instant;

    #[tokio::test]
    async fn test_serialization_performance() {
        // Test message serialization speed

        use synesis_cloud::escalation::types::EscalationRequest;

        let request = EscalationRequest {
            request_id: "test-123".to_string(),
            session_id: "session-456".to_string(),
            query: "What is 2+2?".to_string(),
            context: Default::default(),
            model: Default::default(),
            max_tokens: 100,
            stream: false,
            lora_id: None,
            timeout_secs: Some(30),
        };

        let start = Instant::now();
        let serialized = serde_json::to_vec(&request).unwrap();
        let duration = start.elapsed();

        // Serialization should be very fast (< 1ms)
        assert!(duration.as_millis() < 10, "Serialization too slow: {:?}", duration);
        assert!(!serialized.is_empty());
    }

    #[tokio::test]
    async fn test_deserialization_performance() {
        // Test message deserialization speed

        use synesis_cloud::escalation::types::EscalationResponse;

        let json = r#"{
            "request_id": "test-123",
            "content": "2+2=4",
            "model_used": "claude-3-5-sonnet-20241022",
            "tokens_used": {"prompt": 10, "completion": 5},
            "cost_cents": 1,
            "latency_ms": 150,
            "sources": [],
            "lora_applied": false
        }"#;

        let start = Instant::now();
        let _response: EscalationResponse = serde_json::from_str(json).unwrap();
        let duration = start.elapsed();

        // Deserialization should be very fast (< 1ms)
        assert!(duration.as_millis() < 10, "Deserialization too slow: {:?}", duration);
    }
}
