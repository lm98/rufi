/// # A virtual representation of a sensor.
///
/// `name` is the name of the sensor
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct SensorId {
    pub(crate) name: String,
}
pub fn sensor(name: &str) -> SensorId {
    SensorId::new(name.to_string())
}

impl SensorId {
    /// Given a string, creates a new sensor id.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name of the sensor.
    ///
    /// # Returns
    ///
    /// A new sensor id.
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[cfg(test)]
mod tests {
    use crate::sensor_id::SensorId;

    #[test]
    fn test_new() {
        let sensor_id = SensorId::new("foo".to_string());
        assert_eq!(sensor_id.name, "foo".to_string())
    }
}
