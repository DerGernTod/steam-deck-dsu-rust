use super::dsu_error::DsuError;

pub trait DsuProvider {
    fn accelerometer_reading(&self) -> Result<(f64, f64, f64), DsuError>;
    fn gyro_reading(&self) -> Result<(f64, f64, f64), DsuError>;
    fn poll_events(&mut self) -> Result<(), DsuError> {
        Ok(())
    }
}
