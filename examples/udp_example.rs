//! UDP transport example

use bytes::Bytes;
use rusty_s4i_io::{transport::TransportType, TransportConfig, TransportId, TransportManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Create transport manager
    let mut manager = TransportManager::new();

    // Create and add UDP server
    let server_config = TransportConfig {
        transport_type: TransportType::Udp,
        address: "127.0.0.1:9000".to_string(),
        is_server: true,
        options: Default::default(),
    };
    let server_id = manager.add_transport(server_config).await?;
    manager.connect(&server_id).await?;

    println!("UDP server started on 127.0.0.1:9000");

    // Create UDP client
    let client_config = TransportConfig::udp("127.0.0.1:9000")?;
    let client_id = manager.add_transport(client_config).await?;
    manager.connect(&client_id).await?;

    // Send a test message
    let message = Bytes::from("Hello UDP!");
    manager
        .send(&client_id, &TransportId::Connection(0), message)
        .await?;

    println!("Message sent");

    // Poll for events
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let events = manager.poll_events().await?;
    for (transport_id, event) in events {
        println!("Event from {}: {:?}", transport_id, event);
    }

    Ok(())
}
