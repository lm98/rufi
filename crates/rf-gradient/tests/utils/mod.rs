use rufi::core::export::Export;
use rufi::core::sensor_id::SensorId;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct DeviceState {
    pub self_id: i32,
    pub exports: HashMap<i32, Export>,
    pub local_sensor: HashMap<SensorId, Rc<Box<dyn Any>>>,
    pub nbr_sensor: HashMap<SensorId, HashMap<i32, Rc<Box<dyn Any>>>>,
}

impl DeviceState {
    pub fn update_exports(&mut self, nbr: i32, export: Export) {
        self.exports.insert(nbr, export);
    }
}

#[derive(Debug, Clone)]
pub struct Topology {
    pub devices: Vec<i32>,
    pub states: HashMap<i32, DeviceState>,
}

impl Topology {
    pub fn new(devices: Vec<i32>, states: HashMap<i32, DeviceState>) -> Self {
        Topology { devices, states }
    }
}
