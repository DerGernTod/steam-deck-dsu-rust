use std::{error::Error, fmt};

use sdl2::GameControllerSubsystem;
use sdl2::controller::GameController;
use tokio::net::UdpSocket;
use windows::Devices::Sensors::{Accelerometer, Gyrometer};


#[derive(Debug)]
enum MyError {
    SdlError(String),
    WindowsError(String),
    UdpError(std::io::Error),
    Simple(String)
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MyError::SdlError(msg) => write!(f, "SDL error: {}", msg),
            MyError::WindowsError(msg) => write!(f, "Windows SDK error: {}", msg),
            MyError::UdpError(err) => write!(f, "UDP error: {}", err),
            MyError::Simple(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MyError::UdpError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> MyError {
        MyError::UdpError(err)
    }
}

impl From<windows::core::Error> for MyError {
    fn from(err: windows::core::Error) -> MyError {
        MyError::WindowsError(err.to_string())
    }
}

impl From<String> for MyError {
    fn from(err: String) -> MyError {
        MyError::Simple(err)
    }
}

#[tokio::main]
async fn main() -> Result<(), MyError> {
    // Initialize SDL2 and open the controller
    let sdl = sdl2::init()?;
    // Open controller 0
    // let controller = sdl
    //     .game_controller()?
    //     .open(0)
    //     .map_err(|err| err.to_string())?; 

    let acc = Accelerometer::GetDefault()?;
    let gyro = Gyrometer::GetDefault()?;

    // Main loop
    loop {
        // Read gyro data (simplified)
        // let gyro_data = read_gyro_data(&controller);
        let accelerometer_reading = acc.GetCurrentReading()?;
        println!("Accelerometer: {:.2} {:.2} {:.2}",
            accelerometer_reading.AccelerationX()?,
            accelerometer_reading.AccelerationY()?,
            accelerometer_reading.AccelerationZ()?
        );

        let gyro_reading = gyro.GetCurrentReading()?;
        println!("Gyro: {:.2} {:.2} {:.2}",
            gyro_reading.AngularVelocityX()?.to_degrees(),
            gyro_reading.AngularVelocityY()?.to_degrees(),
            gyro_reading.AngularVelocityZ()?.to_degrees()
        );
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


fn read_gyro_data(game_controller: &GameController) -> (f32, f32, f32) {
    // Assuming you have already initialized SDL2 and opened the controller
    let x = game_controller.axis(sdl2::controller::Axis::LeftX) as f32;
    let y = game_controller.axis(sdl2::controller::Axis::LeftY) as f32;
    let z = game_controller.axis(sdl2::controller::Axis::RightX) as f32;
    (x, y, z) // Return gyro data as a tuple
}

fn to_dsu_packet(gyro_data: (f32, f32, f32)) -> Vec<u8> {
    // Convert gyro data (f32) to a format expected by the DSU protocol (e.g., 16-bit integers or floats)
    let x = gyro_data.0.to_bits() as u16; // convert to raw bits or another format
    let y = gyro_data.1.to_bits() as u16;
    let z = gyro_data.2.to_bits() as u16;
    
    // Create DSU packet with header and data (simplified version)
    let mut packet = Vec::new();
    packet.push(0x01);  // Example header byte for DSU packet
    packet.push(0x02);  // Another example byte, might be used for packet type, etc.
    packet.push(x as u8);  // Add gyro data to packet
    packet.push(y as u8);
    packet.push(z as u8);
    packet
}
