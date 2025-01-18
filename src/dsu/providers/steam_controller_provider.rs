use hidapi::{HidApi, HidDevice, MAX_REPORT_DESCRIPTOR_SIZE};

use crate::dsu::{dsu_error::DsuError, dsu_provider::DsuProvider};

pub struct SteamControllerProvider {
    hid_api: HidApi,
    device: Option<hidapi::HidDevice>,
    accel: (f32, f32, f32),
    gyro: (f32, f32, f32)
}
const ACC_1G: f32 = 0x4000 as f32;
const GYRO_1DEGPERSEC: f32 = 16.0;
const ACCEL_SMOOTH: i16 = 0x1FF;
const GYRO_DEADZONE: f32 = 8.0;
const STEAM_DECK_PROD_ID: u16 = 0x1205;
const STEAM_CONTROLLER_PROD_ID: u16 = 0x1142;
const VALVE_VENDOR_ID: u16 = 0x28de;
// Define constants based on the Python code
const SCPacketType_CONFIGURE: u8 = 0x87;
const SCPacketLength_CONFIGURE: u8 = 0x15;
const SCConfigType_CONFIGURE: u8 = 0x32;


/* Accelerometer has 16 bit resolution and a range of +/- 2g */
const STEAM_DECK_ACCEL_RES_PER_G: i32 = 16384;
const STEAM_DECK_ACCEL_RANGE: i32 = 32768;
const STEAM_DECK_ACCEL_FUZZ: i32 = 32;
/* Gyroscope has 16 bit resolution and a range of +/- 2000 dps */
const STEAM_DECK_GYRO_RES_PER_DPS: i32 = 16;
const STEAM_DECK_GYRO_RANGE: i32 = 32768;
const STEAM_DECK_GYRO_FUZZ: i32 = 1;
/* Input device props */
const INPUT_PROP_ACCELEROMETER: usize = 0x06;
const EV_MSC: usize = 0x04;
const MSC_TIMESTAMP: usize = 0x05;


/*
command to be sent in feature report 
	ID_CHECK_GYRO_FW_LOAD		= 0xC2,
    ID_CLEAR_DIGITAL_MAPPINGS	= 0x81, // lizard 

    /* Values for GYRO_MODE (bitmask) */
enum {
	SETTING_GYRO_MODE_OFF			= 0,
	SETTING_GYRO_MODE_STEERING		= BIT(0),
	SETTING_GYRO_MODE_TILT			= BIT(1),
	SETTING_GYRO_MODE_SEND_ORIENTATION	= BIT(2),
	SETTING_GYRO_MODE_SEND_RAW_ACCEL	= BIT(3),
	SETTING_GYRO_MODE_SEND_RAW_GYRO		= BIT(4),
};
*/

struct InputDevice {
    property_bits: usize,
    event_bits: usize,
    misc_bits: usize,
}

struct SteamDevice {
    hid_device: HidDevice,
    sensors: Option<InputDevice>,
    sensor_timestamp_us: usize
}

fn steam_sensors_register(steam: SteamDevice) -> i32 {
    let hdev = steam.hid_device;
    if let Some(_) = steam.sensors {
        println!("Sensors already registered!");
        return 0;
    }

    // let sensors = input_allocate_device();
    // input_set_drvdata(sensors, steam);
    
}

impl SteamControllerProvider {
    pub fn new() -> Result<SteamControllerProvider, DsuError> {
        let api = HidApi::new()?;
         // Vendor ID and Product ID for Steam Controller
        let mut controller = SteamControllerProvider {
            device: None,
            hid_api: api,
            accel: (0.0, 0.0, 0.0),
            gyro: (0.0, 0.0, 0.0)
        };
        controller.detect_controller();
        // controller.enable_gyro()?;
        Ok(controller)
    }

    fn detect_controller(&mut self) {
        let steam_deck_controller = self.hid_api
            .device_list()
            .filter(|device| device.product_id() == STEAM_DECK_PROD_ID
                && device.vendor_id() == VALVE_VENDOR_ID)
            .skip(2)
            .take(1)
            .map(|device| device.open_device(&self.hid_api))
            .last();
        
        self.device = steam_deck_controller.and_then(|device| device.ok());

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
            println!();
        }
    }

    fn poll_events(&mut self) -> Result<(), DsuError> {
        self.device
            .as_mut()
            .ok_or(DsuError::from(String::from("Device not found")))
            .and_then(|device| {
                println!("trying to read into buffer");
                let mut buf = [0u8; 64];
                let bytes_read = device.read(&mut buf)?;
                println!("successfully read {} bytes into buffer", bytes_read);
                for (i, byte) in buf.iter().enumerate() {
                    if i % 8 == 0 {
                        println!();
                    }
                    print!("{:02x},", byte);
                }
                println!();
                let (ax, ay, az) = accel_from_buffer(&buf);
                self.accel = (
                    smooth_accel(self.accel.0, ax),
                    smooth_accel(self.accel.1, ay),
                    smooth_accel(self.accel.2, az)
                );
                self.gyro = gyro_from_buffer(&buf);
                Ok(())
            })
    }

    fn read_motion_data(&self) -> Result<(f64, f64, f64, f64, f64, f64), DsuError> {
        let (ax, ay, az) = self.accel;
        let (gx, gy, gz) = self.gyro;
        Ok((ax as f64, ay as f64, az as f64, gx as f64, gy as f64, gz as f64))
    }
}

fn accel_from_buffer(buf: &[u8; 64]) -> (f32, f32, f32) {
    let ax = f32::from(((buf[25] as u16) << 8) | buf[26] as u16);
    let ay = f32::from(((buf[27] as u16) << 8) | buf[28] as u16);
    let az = f32::from(((buf[29] as u16) << 8) | buf[30] as u16);
    (ax as f32, ay as f32, az as f32)
}

fn gyro_from_buffer(buf: &[u8; 64]) -> (f32, f32, f32) {
    let gx = f32::from(((buf[31] as u16) << 8) | buf[32] as u16);
    let gy = f32::from(((buf[33] as u16) << 8) | buf[34] as u16);
    let gz = f32::from(((buf[35] as u16) << 8) | buf[36] as u16);
    (gx as f32, gy as f32, gz as f32)
}

fn smooth_accel(last: f32, curr: f32) -> f32 {
    let new = if (curr - last).abs() < ACCEL_SMOOTH as f32 {
        last * 0.95 + curr * 0.05
    } else {
        curr
    };
    new / ACC_1G
}

fn convert_gyro_to_dps(gyro_value: f32) -> f32 {
    if gyro_value.abs() < GYRO_DEADZONE {
        0.0
    } else {
        gyro_value / GYRO_1DEGPERSEC
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

#[cfg(test)]
mod test {
    use crate::dsu::providers::steam_controller_provider::smooth_accel;
    use super::*;

    const TEST_INPUT_A: [u8;64] = [
        0x1,0,0x9,0x40,0xc8,0x98,0x0a,0,
        0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,
        0x2d,0,0xdc,1,0xcb,0x3f,8,0,
        2,0,1,0,0x90,0x77,0x56,0,
        0xec,0xfe,0x52,0xd2,0,0,0,0,
        0x90,3,0xac,0xfa,0x16,0xfd,0x7d,0xfe,
        0,0,0,0,0,0,0,0
    ];
    const TEST_INPUT_B: [u8;64] = [
        0x1,0,0x9,0x40,0xc8,0x98,0x0a,0,
        0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,
        0x46,0,0x3e,1,0x8b,0x3f,0xfc,0xff,
        1,0,0xfe,0xff,0x90,0x77,0x56,0,
        0xec,0xfe,0x52,0xd2,0,0,0,0,
        0x90,0x03,0xac,0xfa,0x16,0xfd,0xa8,0xfe,
        0,0,0,0,0,0,0,0
    ];

    #[test]
    fn test_smooth_accel() {
        let prev_acc = (0.0,0.0,0.0);
        let acc = accel_from_buffer(&TEST_INPUT_A);
        assert_eq!(
            (
                smooth_accel(prev_acc.0, acc.0),
                smooth_accel(prev_acc.1, acc.1),
                smooth_accel(prev_acc.2, acc.2)
            ), 
            (0.0006713867, 0.0014007569, 0.9848633));
        let prev_acc = acc;
        let acc = accel_from_buffer(&TEST_INPUT_B);
        assert_eq!(
            (
                smooth_accel(prev_acc.0, acc.0),
                smooth_accel(prev_acc.1, acc.1),
                smooth_accel(prev_acc.2, acc.2)
            ), 
            (0.012945557, 0.027819823, 0.9856079));
    }

    #[test]
    fn test_convert_gyro() {
        let gyro = gyro_from_buffer(&TEST_INPUT_A);
        assert_eq!(
            (
                convert_gyro_to_dps(gyro.0),
                convert_gyro_to_dps(gyro.1),
                convert_gyro_to_dps(gyro.2)
            ), 
            (0.0, 0.0, 9.0));
        let gyro = gyro_from_buffer(&TEST_INPUT_B);
        assert_eq!(
            (
                convert_gyro_to_dps(gyro.0),
                convert_gyro_to_dps(gyro.1),
                convert_gyro_to_dps(gyro.2)
            ), 
            (4080.0625, 15.875, 4089.0));
    }
}