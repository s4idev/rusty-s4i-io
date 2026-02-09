//! Modbus protocol example

#[cfg(feature = "modbus")]
use rusty_s4i_io::protocol::{
    modbus::{ModbusConfig, ModbusHandler, ModbusProtocol},
    ProtocolHandler,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "modbus")]
    {
        // Initialize logging
        env_logger::init();

        // Create Modbus configuration
        let config = ModbusConfig {
            protocol: ModbusProtocol::Tcp,
            address: "127.0.0.1:502".to_string(),
            slave_id: 1,
        };

        // Create Modbus handler
        let mut handler = ModbusHandler::new(config);

        // Connect to Modbus server
        handler.connect().await?;
        println!("Connected to Modbus server");

        // Read holding registers
        match handler.read_holding_registers(100, 10).await {
            Ok(values) => println!("Read {} registers", values.len()),
            Err(e) => println!("Error reading registers: {}", e),
        }

        // Write single register
        match handler.write_single_register(100, 42).await {
            Ok(_) => println!("Register written successfully"),
            Err(e) => println!("Error writing register: {}", e),
        }

        // Disconnect
        handler.disconnect().await?;
    }

    #[cfg(not(feature = "modbus"))]
    {
        println!("Modbus feature not enabled. Build with --features modbus");
    }

    Ok(())
}
