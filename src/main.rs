mod dsu;
use dsu::{dsu_error::DsuError, dsu_provider::DsuProvider, providers::{
    // sdl_provider::SdlProvider,
    steam_controller_provider::SteamControllerProvider
}};
use tokio::net::UdpSocket;


#[tokio::main]
async fn main() -> Result<(), DsuError> {
    // Initialize SDL2 and open the controller

    let mut provider = get_provider()?;
    // Main loop
    loop {
        provider.poll_events()?;
        let accelerometer_reading = provider.accelerometer_reading()?;
        let gyro_reading = provider.gyro_reading()?;
        println!("Accelerometer: {:?}\nGyro: {:?}", accelerometer_reading, gyro_reading);
        // Convert to DSU packet
        let dsu_packet = vec![];//to_dsu_packet(gyro_data);

        // Send the DSU packet over UDP
        let target_address = "127.0.0.1:8080"; // Example address
        send_dsu_packet(dsu_packet, target_address).await.map_err(|err| err.to_string())?;
        std::thread::sleep(std::time::Duration::from_millis(500)); // Sleep for a while
    }
}

async fn send_dsu_packet(packet: Vec<u8>, address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;  // Bind to a random port
    socket.send_to(&packet, address).await?;
    Ok(())
}

fn get_provider() -> Result<Box<dyn DsuProvider>, DsuError> {
     fn box_provider<P: DsuProvider + 'static>(provider: P) -> Box<dyn DsuProvider> {
        Box::new(provider)
    }

    SteamControllerProvider::new().map(box_provider)
    
}