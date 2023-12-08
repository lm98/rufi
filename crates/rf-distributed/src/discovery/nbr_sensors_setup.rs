use rf_core::context::NbrSensors;

/// This trait represent a strategy to setup the neighbouring sensors of the device.
pub trait NbrSensorSetup {
    /// Setup the neighbouring sensors of the device.
    ///
    /// # Arguments
    /// * `nbrs` - The ids of the neighbouring sensors.
    ///
    /// # Returns
    /// A map containing the neighbouring sensors.
    fn nbr_sensor_setup(&self, nbrs: Vec<i32>) -> NbrSensors;
}