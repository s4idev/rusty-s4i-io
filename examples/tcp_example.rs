use rusty_s4i_io::{TransportConfig, TransportManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Create transport manager
    let mut manager = TransportManager::new();

    // Create and add TCP server
    let server_config = TransportConfig::tcp_server("127.0.0.1:8080")?;
    let server_id = manager.add_transport(server_config).await?;
    manager.connect(&server_id).await?;

    println!("TCP server started on 127.0.0.1:8080");
    println!("Press Ctrl+C to exit");

    // Keep running
    tokio::signal::ctrl_c().await?;

    // Disconnect
    manager.disconnect(&server_id).await?;

    Ok(())
}
