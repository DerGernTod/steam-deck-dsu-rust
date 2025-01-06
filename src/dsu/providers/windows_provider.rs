use windows::Devices::Sensors::{Accelerometer, Gyrometer};

use crate::dsu::{dsu_error::DsuError, dsu_provider::DsuProvider};

pub struct WindowsProvider {
    accelerometer: Accelerometer,
    gyro: Gyrometer
}


impl WindowsProvider {
    pub fn new() -> Result<WindowsProvider, DsuError> {
        let accelerometer = Accelerometer::GetDefault()
            .map_err(|err| DsuError::from(format!("Accelerometer not found: {}", err.to_string())))?;
        let gyro = Gyrometer::GetDefault()
        .map_err(|err| DsuError::from(format!("Gyrometer not found: {}", err.to_string())))?;
        Ok(WindowsProvider {
            accelerometer,
            gyro
        })
    }
}

impl DsuProvider for WindowsProvider {
    fn accelerometer_reading(&self) -> Result<(f64, f64, f64), DsuError> {
        let reading = self.accelerometer.GetCurrentReading()?;
        Ok((
            reading.AccelerationX()?,
            reading.AccelerationY()?,
            reading.AccelerationZ()?
        ))
    }
    fn gyro_reading(&self) -> Result<(f64, f64, f64), DsuError> {
        let reading = self.gyro.GetCurrentReading()?;
        Ok((
            reading.AngularVelocityX()?.to_degrees(),
            reading.AngularVelocityY()?.to_degrees(),
            reading.AngularVelocityZ()?.to_degrees()
        ))
    }
}