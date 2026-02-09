//! Integration tests for rusty-s4i-io

use rusty_s4i_io::{TransportConfig, TransportManager, TransportId};
use bytes::Bytes;

#[tokio::test]
async fn test_transport_manager_lifecycle() {
    let mut manager = TransportManager::new();
    
    // Add TCP transport
    let tcp_config = TransportConfig::tcp_server("127.0.0.1:0").unwrap();
    let tcp_id = manager.add_transport(tcp_config).await.unwrap();
    
    // Connect
    manager.connect(&tcp_id).await.unwrap();
    
    // Disconnect
    manager.disconnect(&tcp_id).await.unwrap();
    
    // Remove
    manager.remove_transport(&tcp_id).await.unwrap();
}

#[tokio::test]
async fn test_multiple_transports() {
    let mut manager = TransportManager::new();
    
    // Add multiple transports
    let tcp_config = TransportConfig::tcp_server("127.0.0.1:0").unwrap();
    let tcp_id = manager.add_transport(tcp_config).await.unwrap();
    
    let udp_config = TransportConfig {
        transport_type: rusty_s4i_io::transport::TransportType::Udp,
        address: "127.0.0.1:0".to_string(),
        is_server: true,
        options: Default::default(),
    };
    let udp_id = manager.add_transport(udp_config).await.unwrap();
    
    // Connect both
    manager.connect(&tcp_id).await.unwrap();
    manager.connect(&udp_id).await.unwrap();
    
    // Poll events from all transports
    let events = manager.poll_events().await.unwrap();
    // Should have at least 2 connected events
    assert!(events.len() >= 2);
}

#[tokio::test]
async fn test_url_parsing() {
    // Test various URL formats
    let tcp_config = TransportConfig::from_url("tcp://localhost:8080").unwrap();
    assert_eq!(tcp_config.transport_type, rusty_s4i_io::transport::TransportType::Tcp);
    
    let udp_config = TransportConfig::from_url("udp://192.168.1.1:9000").unwrap();
    assert_eq!(udp_config.transport_type, rusty_s4i_io::transport::TransportType::Udp);
    
    let http_config = TransportConfig::from_url("http://example.com").unwrap();
    assert_eq!(http_config.transport_type, rusty_s4i_io::transport::TransportType::Http);
}

#[tokio::test]
async fn test_error_handling() {
    let manager = TransportManager::new();
    
    // Try to connect to non-existent transport
    let result = manager.connect("non_existent").await;
    assert!(result.is_err());
    
    // Try to send to non-existent transport
    let result = manager.send("non_existent", &TransportId::Connection(0), Bytes::new()).await;
    assert!(result.is_err());
}
