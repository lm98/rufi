use rufi::core::context::Context;
use rufi::core::lang::execution::round;
use rufi::core::sensor_id::{sensor, SensorId};
use rufi::core::vm::round_vm::RoundVM;
use rufi::programs::gradient;
use std::any::Any;
use std::collections::HashMap;
use std::iter;
use std::rc::Rc;
use local_gradient::DeviceState;


fn main() {
    let devices = vec![1, 2, 3, 4, 5];
    /* Set up a simple topology that will be used for these tests.
     *  Topology: [1] -- [2] -- [3] -- [4] -- [5].
     */
    let mut states: HashMap<i32, DeviceState> = devices
        .iter()
        .map(|d| {
            let nbrs: Vec<i32> = vec![d.clone() - 1, d.clone(), d.clone() + 1]
                .into_iter()
                .filter(|n| (n > &0 && n < &6))
                .collect();
            // In this example, we set the source to be device 2.
            let local_sensor: HashMap<SensorId, Rc<Box<dyn Any>>> =
                vec![(sensor("source"), Rc::new(Box::new(*d == 2) as Box<dyn Any>))]
                    .into_iter()
                    .collect();
            let nbr_sensor: HashMap<SensorId, HashMap<i32, Rc<Box<dyn Any>>>> = HashMap::from([(
                sensor("nbr_range"),
                nbrs.iter()
                    .map(|n| {
                        (
                            n.clone(),
                            Rc::new(Box::new(i32::abs(d - n)) as Box<dyn Any>),
                        )
                    })
                    .collect(),
            )]);
            let state = DeviceState {
                self_id: d.clone(),
                exports: HashMap::new(),
                local_sensor,
                nbr_sensor,
            };
            (d.clone(), state)
        })
        .collect();

    let scheduling: Vec<i32> = iter::repeat(devices).take(5).flatten().collect();

    // For each device in the provided scheduling, run the program on the device.
    for d in scheduling {
        let curr = states.get(&d).unwrap().clone();
        let ctx = Context::new(d, curr.local_sensor, curr.nbr_sensor, curr.exports);
        println!("RUN: DEVICE {}\n\tCONTEXT {:?}", d, ctx);
        // Setup the VM
        let mut vm = RoundVM::new(ctx);
        vm.new_export_stack();
        // Run the program
        let (mut vm_, res) = round(vm, gradient);
        let mut to_update = states.get(&d).unwrap().clone();
        to_update.update_exports(d, vm_.export_data().clone());
        // Update the exports of the neighbors, simulating the message passing
        to_update
            .nbr_sensor
            .get(&sensor("nbr_range"))
            .unwrap()
            .keys()
            .for_each(|nbr| {
                let mut nbr_state = states.get(nbr).unwrap().clone();
                nbr_state.update_exports(d, to_update.exports.get(&d).unwrap().clone());
                states.insert(nbr.clone(), nbr_state);
            });
        // Update the topology with the new state
        states.insert(d, to_update);
        println!(
            "\t EXPORT: {:?}\n\t OUTPUT: {:?}\n\t",
            states.get(&d).unwrap().exports.get(&d).unwrap(),
            res
        );
    }
}
