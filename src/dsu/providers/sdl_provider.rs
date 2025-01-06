use sdl2::event::Event;
use sdl2::joystick::{self, Joystick};
use sdl2::{EventPump, GameControllerSubsystem, JoystickSubsystem};
use sdl2::controller::GameController;
use crate::dsu::{dsu_error::DsuError, dsu_provider::DsuProvider};

pub struct SdlProvider {
    joystick: Option<Joystick>,
    event_pump: EventPump,
    joystick_subsystem: JoystickSubsystem
}


impl SdlProvider {
    pub fn new() -> Result<SdlProvider, DsuError> {
        println!("initialized sdl provider");
        let sdl = sdl2::init()?;
        let event_pump = sdl.event_pump()?;
        let joystick_subsystem = sdl.joystick()?;
        let joystick = joystick_subsystem.open(0).ok();
        
        println!("Connected joysticks: {}", joystick_subsystem.num_joysticks()?);
        println!("Connected controllers: {}", sdl.game_controller()?.num_joysticks()?);
        Ok(SdlProvider {
            joystick,
            event_pump,
            joystick_subsystem
        })
    }
}

impl DsuProvider for SdlProvider {
    fn accelerometer_reading(&self) -> Result<(f64, f64, f64), DsuError> {
        if let Some(joystick) = &self.joystick {
            let x = joystick.axis(0)? as f64;
            let y = joystick.axis(1)? as f64;
            let z = joystick.axis(2)? as f64;
            Ok((x, y, z))
        } else {
            Ok((0.0, 0.0, 0.0))
        }
    }
    fn gyro_reading(&self) -> Result<(f64, f64, f64), DsuError> {
        if let Some(controller) = &self.joystick {
            let x = controller.axis(3)? as f64;
            let y = controller.axis(4)? as f64;
            let z = controller.axis(5)? as f64;
            Ok((x, y, z))
        } else {
            Ok((0.0, 0.0, 0.0))
        }
    }

    fn poll_events(&mut self) -> Result<(), DsuError> {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::JoyDeviceAdded { which: 0, .. } => {
                    println!("connected joystick 0");
                    self.joystick = self.joystick_subsystem.open(0).ok();
                }
                Event::JoyDeviceRemoved { which: 0, .. } => {
                    println!("disconnected joystick 0");
                    self.joystick = None;
                }
                Event::JoyDeviceAdded { which, .. } => {
                    println!("connected joystick {}", which);
                }
                Event::JoyDeviceRemoved { which, .. } => {
                    println!("disconnected joystick {}", which);
                }
                _ => {}
            }
        }
        Ok(())
    }
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
