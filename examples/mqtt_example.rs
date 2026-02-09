//! MQTT protocol example

#[cfg(feature = "mqtt")]
use rusty_s4i_io::protocol::{
    mqtt::{MqttConfig, MqttHandler},
    ProtocolHandler,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "mqtt")]
    {
        // Initialize logging
        env_logger::init();

        // Create MQTT configuration
        let config = MqttConfig {
            broker_url: "mqtt://localhost:1883".to_string(),
            client_id: "rusty-s4i-io-example".to_string(),
            username: None,
            password: None,
            clean_session: true,
            keep_alive: 60,
        };

        // Create MQTT handler
        let mut handler = MqttHandler::new(config);

        // Connect to broker
        handler.connect().await?;
        println!("Connected to MQTT broker");

        // Subscribe to a topic
        handler.subscribe("test/topic").await?;
        println!("Subscribed to test/topic");

        println!("Press Ctrl+C to exit");
        tokio::signal::ctrl_c().await?;

        // Disconnect
        handler.disconnect().await?;
    }

    #[cfg(not(feature = "mqtt"))]
    {
        println!("MQTT feature not enabled. Build with --features mqtt");
    }

    Ok(())
}
