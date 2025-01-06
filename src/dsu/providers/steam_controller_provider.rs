use hidapi::{HidApi, HidDevice, MAX_REPORT_DESCRIPTOR_SIZE};

use crate::dsu::{dsu_error::DsuError, dsu_provider::DsuProvider};

pub struct SteamControllerProvider {
    hid_api: HidApi,
    device: Option<hidapi::HidDevice>
}

const STEAM_CONTROLLER_PROD_ID: u16 = 0x1142;
const VALVE_VENDOR_ID: u16 = 0x28de;
// Define constants based on the Python code
const SCPacketType_CONFIGURE: u8 = 0x87;
const SCPacketLength_CONFIGURE: u8 = 0x15;
const SCConfigType_CONFIGURE: u8 = 0x32;

impl SteamControllerProvider {
    pub fn new() -> Result<SteamControllerProvider, DsuError> {
        let api = HidApi::new()?;
         // Vendor ID and Product ID for Steam Controller
        let mut controller = SteamControllerProvider { device: None, hid_api: api };
        controller.detect_controller();
        controller.enable_gyro()?;
        Ok(controller)
    }

    fn enable_gyro(&self) -> Result<(), DsuError> {
        // Placeholder values for unknown1 and unknown2
        let unknown1: [u8; 13] = [0x18, 0x00, 0x00, 0x31, 0x02, 0x00, 0x08, 0x07, 0x00, 0x07, 0x07, 0x00, 0x30];
        let unknown2: [u8; 2] = [0x00, 0x2e];

        // Enable gyroscope
        let enable_gyros: u8 = 0x14;
            // Idle timeout (example 600 seconds)
        let idle_timeout: u16 = 600;
        let timeout1: u8 = (idle_timeout & 0x00FF) as u8;
        let timeout2: u8 = ((idle_timeout & 0xFF00) >> 8) as u8;

        // Create the buffer to send
        let mut buffer: Vec<u8> = Vec::with_capacity(64);
        buffer.push(SCPacketType_CONFIGURE);
        buffer.push(SCPacketLength_CONFIGURE);
        buffer.push(SCConfigType_CONFIGURE);
        buffer.push(timeout1);
        buffer.push(timeout2);
        buffer.extend_from_slice(&unknown1);
        buffer.push(enable_gyros);
        buffer.extend_from_slice(&unknown2);
        // Ensure the buffer is 64 bytes long
        buffer.resize(64, 0); 

        // Write the buffer to the device
        self.device
            .as_ref()
            .ok_or(DsuError::from(String::from("Device not found")))
            .and_then(|device| {
                let bytes_written = device.write(&buffer)?;
                println!("Wrote {} bytes", bytes_written);
                Ok(())
            })
    }

    fn detect_controller(&mut self) {
        let steam_controller = self.hid_api
            .device_list()
            .filter(|device| device.product_id() == STEAM_CONTROLLER_PROD_ID
                && device.vendor_id() == VALVE_VENDOR_ID)
            .skip(1)
            .take(1)
            .map(|device| device.open_device(&self.hid_api))
            .last();
        
        self.device = steam_controller.and_then(|device| device.ok());

        if let Some(device) = &self.device {
            let mut buf = [0u8; MAX_REPORT_DESCRIPTOR_SIZE];
            if let Ok(_) = device.get_report_descriptor(&mut buf) {
                println!("Detected controller");
                for (i, byte) in buf.iter().enumerate() {
                    if i % 8 == 0 {
                        println!();
                    }
                    print!("{:02x},", byte);
                }
            }
        }
    }

    fn read_motion_data(&self) -> Result<(f64, f64, f64, f64, f64, f64), DsuError> {
        self.device
            .as_ref()
            .ok_or(DsuError::from(String::from("Device not found")))
            .and_then(|device| {
                println!("trying to read into buffer");
                let mut buf = [0u8; 128];
                let bytes_read = device.read(&mut buf)?;
                println!("successfully read {} bytes into buffer", bytes_read);
                for (i, byte) in buf.iter().enumerate() {
                    if i % 8 == 0 {
                        println!();
                    }
                    print!("{:02x},", byte);
                }
                let ax = f64::from(buf[0]);
                let ay = f64::from(buf[1]);
                let az = f64::from(buf[2]);
                let gx = f64::from(buf[3]);
                let gy = f64::from(buf[4]);
                let gz = f64::from(buf[5]);
                Ok((ax, ay, az, gx, gy, gz))
            })
    }
}

impl DsuProvider for SteamControllerProvider {
    fn accelerometer_reading(&self) -> Result<(f64, f64, f64), DsuError> {
        let (ax, ay, az, _, _, _) = self.read_motion_data()?;
        Ok((ax, ay, az))
    }

    fn gyro_reading(&self) -> Result<(f64, f64, f64), DsuError> {
        let (_, _, _, gx, gy, gz) = self.read_motion_data()?;
        Ok((gx, gy, gz))
    }
}