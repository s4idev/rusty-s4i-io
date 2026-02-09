//! BACNet/IP protocol example

#[cfg(feature = "bacnet")]
use rusty_s4i_io::protocol::{
    bacnet::{BacnetConfig, BacnetHandler, BacnetObjectType, BacnetProperty},
    ProtocolHandler
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "bacnet")]
    {
        // Initialize logging
        env_logger::init();

        // Create BACNet configuration
        let config = BacnetConfig {
            address: "127.0.0.1".to_string(),
            port: 47808,
            device_id: 1234,
            network_number: 0,
        };

        // Create BACNet handler
        let mut handler = BacnetHandler::new(config);

        // Connect to BACNet network
        handler.connect().await?;
        println!("Connected to BACNet network");

        // Perform Who-Is discovery
        match handler.who_is().await {
            Ok(devices) => println!("Found {} devices", devices.len()),
            Err(e) => println!("Error during Who-Is: {}", e),
        }

        // Read a property
        match handler.read_property(
            1234,
            BacnetObjectType::AnalogInput,
            0,
            BacnetProperty::PresentValue
        ).await {
            Ok(value) => println!("Read property value: {} bytes", value.len()),
            Err(e) => println!("Error reading property: {}", e),
        }

        // Disconnect
        handler.disconnect().await?;
    }

    #[cfg(not(feature = "bacnet"))]
    {
        println!("BACNet feature not enabled. Build with --features bacnet");
    }

    Ok(())
}
