use crate::export::{Export, Result};
use crate::path::Path;
use crate::sensor_id::SensorId;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

/// # Context implementation
///
/// * `selfId` The ID of the device that this context is for.
///
/// * `local_sensor` The values perceived by the local sensors of the device.
///
/// * `nbr_sensor` The values perceived by the sensors for each neighbor of the device.
///
/// * `exports` All the export that are available to the device.
#[derive(Debug, Clone)]
pub struct Context {
    self_id: i32,
    local_sensor: HashMap<SensorId, Rc<Box<dyn Any>>>,
    nbr_sensor: HashMap<SensorId, HashMap<i32, Rc<Box<dyn Any>>>>,
    exports: HashMap<i32, Export>,
}

impl Context {
    /// Create new Context of a device from the given parameters.
    ///
    /// # Arguments
    ///
    /// * `self_id` - the ID of the device
    ///
    /// * `local_sensor` - The values perceived by the local sensors of the device.
    ///
    /// * `nbr_sensor` - The values perceived by the sensors for each neighbor of the device.
    ///
    /// * `exports` - All the export that are available to the device.
    ///
    /// # Returns
    ///
    /// The new Context.
    pub fn new(
        self_id: i32,
        local_sensor: HashMap<SensorId, Rc<Box<dyn Any>>>,
        nbr_sensor: HashMap<SensorId, HashMap<i32, Rc<Box<dyn Any>>>>,
        exports: HashMap<i32, Export>,
    ) -> Self {
        Self {
            self_id,
            local_sensor,
            nbr_sensor,
            exports,
        }
    }

    pub fn self_id(&self) -> &i32 {
        &self.self_id
    }

    pub fn exports(&self) -> &HashMap<i32, Export> {
        &self.exports
    }

    /// Add an export of a device to the context.
    ///
    /// # Arguments
    ///
    /// * `id`  the ID of the device
    /// * `data` the export of the device
    pub fn put_export(&mut self, id: i32, data: Export) {
        self.exports.insert(id, data);
    }

    /// Read the value corresponding to the given path from the export of a device.
    ///
    /// # Arguments
    ///
    /// * `id` the ID of the device
    /// * `path` the path to the value
    ///
    /// # Generic Parameters
    ///
    /// * `A` the type of the value to return. It must have a `'static` lifetime.
    ///
    /// # Returns
    ///
    /// An `Option` of the value if it exists
    pub fn read_export_value<A: 'static + FromStr + Clone>(
        &self,
        id: &i32,
        path: &Path,
    ) -> Result<A> {
        self.exports
            .get(id)
            .ok_or("Export not found".into())
            .and_then(|export| export.get(path))
    }

    pub fn local_sensors(&self) -> &HashMap<SensorId, Rc<Box<dyn Any>>> {
        &self.local_sensor
    }

    /// Get the value of the given sensor.
    ///
    /// # Arguments
    ///
    /// * `name` the name of the sensor
    ///
    /// # Generic Parameters
    /// * `A` the type of the value to return. It must have a `'static` lifetime.
    ///
    /// # Returns
    ///
    /// An `Option` of the value if it exists
    pub fn local_sense<A: 'static>(&self, local_sensor_id: &SensorId) -> Option<&A> {
        self.local_sensor
            .get(local_sensor_id)
            .and_then(|value| value.downcast_ref::<A>())
    }

    pub fn nbr_sensors(&self) -> &HashMap<SensorId, HashMap<i32, Rc<Box<dyn Any>>>> {
        &self.nbr_sensor
    }

    /// Get the value of the given sensor for the given neighbor.
    ///
    /// # Arguments
    ///
    /// * `sensor_id` the neighbor sensor id
    /// * `nbr_id` the neighbor id
    ///
    /// # Generic Parameters
    ///
    /// * `A` the type of the value to return. It must have a `'static` lifetime.
    ///
    /// # Returns
    ///
    /// An `Option` of the value if it exists
    pub fn nbr_sense<A: 'static>(&self, sensor_id: &SensorId, nbr_id: &i32) -> Option<&A> {
        self.nbr_sensor
            .get(sensor_id)
            .and_then(|value| value.get(nbr_id))
            .and_then(|value| value.downcast_ref::<A>())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::path::Path;
    use crate::sensor_id::{sensor, SensorId};
    use crate::slot::Slot::{Branch, Nbr, Rep};
    use crate::{export, path};
    use std::any::Any;
    use std::collections::HashMap;
    use std::rc::Rc;

    fn context_builder() -> Context {
        let local_sensor = HashMap::from([(sensor("test"), Rc::new(Box::new(10) as Box<dyn Any>))]);
        let nbr_sensor = HashMap::from([(
            sensor("test"),
            HashMap::from([(0, Rc::new(Box::new(10) as Box<dyn Any>))]),
        )]);
        let export = HashMap::from([(0, export!((path!(Rep(0), Nbr(0)), 10)))]);
        Context::new(7, local_sensor, nbr_sensor, export)
    }

    #[test]
    fn assert_on_fields() {
        let context = context_builder();
        assert_eq!(context.self_id, 7);
        assert_eq!(context.exports.len(), 1);
        assert_eq!(context.local_sensor.len(), 1);
        assert_eq!(context.nbr_sensor.len(), 1);
    }

    #[test]
    fn test_put_export() {
        let mut context = context_builder();
        assert_eq!(context.exports.len(), 1);
        let add_export = export!((path!(Branch(0), Nbr(0)), 5));
        context.put_export(1, add_export);
        assert_eq!(context.exports.len(), 2)
    }

    #[test]
    fn test_read_export_value() {
        let context = context_builder();
        assert_eq!(
            context
                .read_export_value::<i32>(&0, &path!(Rep(0), Nbr(0)))
                .unwrap(),
            10
        );
        assert!(context.read_export_value::<i32>(&1, &Path::new()).is_err());
        assert!(context.read_export_value::<i32>(&0, &Path::new()).is_err());
    }

    #[test]
    fn test_local_sense() {
        let context = context_builder();
        assert_eq!(
            context
                .local_sense::<i32>(&SensorId::new("test".to_string()))
                .unwrap(),
            &10
        );
    }

    #[test]
    fn test_nbr_sense() {
        let context = context_builder();
        assert_eq!(
            context
                .nbr_sense::<i32>(&SensorId::new("test".to_string()), &0)
                .unwrap(),
            &10
        );
    }
}
