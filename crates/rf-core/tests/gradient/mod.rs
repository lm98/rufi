use rf_core::lang::builtins::{foldhood_plus, mux};
use rf_core::lang::{nbr, rep};
use rf_core::vm::round_vm::RoundVM;
use rf_core::{foldhood_plus, lift};
use rf_core::context::Context;
use rf_core::export::Export;
use rf_core::lang::execution::round;
use rf_core::path::Path;
use rf_core::slot::Slot::{FoldHood, Nbr, Rep};
use rf_core::sensor_id::{sensor, SensorId};
use rf_core::{export, path};
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::iter;
use std::rc::Rc;
use std::str::FromStr;
use crate::utils::{DeviceState, Topology};

pub fn gradient(vm: RoundVM) -> (RoundVM, f64) {
    fn is_source(vm: RoundVM) -> (RoundVM, bool) {
        let val = vm.local_sense::<bool>(&sensor("source")).unwrap().clone();
        (vm, val)
    }

    rep(vm, lift!(f64::INFINITY), |vm1, d| {
        mux(
            vm1,
            is_source,
            lift!(0.0),
            foldhood_plus!(lift!(f64::INFINITY), |a, b| a.min(b), |vm2| {
                let (vm_, val) = nbr(vm2, lift!(d));
                (vm_, val + 1.0)
            }),
        )
    })
}

fn setup_test_topology(devices: Vec<i32>) -> Topology {
    /* Set up a simple topology that will be used for these tests.
     *  Topology: [1] -- [2] -- [3] -- [4] -- [5].
     */
    let states: HashMap<i32, DeviceState> = devices
        .iter()
        .map(|d| {
            let nbrs: Vec<i32> = vec![d.clone() - 1, d.clone(), d.clone() + 1]
                .into_iter()
                .filter(|n| (n > &0 && n < &6))
                .collect();
            let local_sensor: HashMap<SensorId, Rc<Box<dyn Any>>> =
                vec![(sensor("source"), Rc::new(Box::new(false) as Box<dyn Any>))]
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
    Topology::new(devices, states)
}

fn add_source(topology: &mut Topology, source: i32) {
    // Add a source to the topology.
    let mut source_state = topology.states.get(&source).unwrap().clone();
    source_state
        .local_sensor
        .insert(sensor("source"), Rc::new(Box::new(true) as Box<dyn Any>));
    topology.states.insert(source, source_state);
}

fn run_on_device<A, F: Copy>(program: F, mut topology: Topology, d: i32) -> Topology
    where
        F: Fn(RoundVM) -> (RoundVM, A),
        A: Clone + 'static + FromStr,
{
    // Setup the VM
    let curr = topology.states.get(&d).unwrap().clone();
    let ctx = Context::new(d, curr.local_sensor, curr.nbr_sensor, curr.exports);
    let mut vm = RoundVM::new(ctx);
    vm.new_export_stack();
    // Run the program
    let (mut vm_, _res) = round(vm, program);
    println!("{}: {}", d, vm_.export_data().to_string());
    // Update the topology with the new exports
    let mut to_update = topology.states.get(&d).unwrap().clone();
    to_update.update_exports(d, vm_.export_data().clone());
    // Update the exports of the neighbors, simulating the message passing
    to_update
        .nbr_sensor
        .get(&sensor("nbr_range"))
        .unwrap()
        .keys()
        .for_each(|nbr| {
            let mut nbr_state = topology.states.get(nbr).unwrap().clone();
            nbr_state.update_exports(d, to_update.exports.get(&d).unwrap().clone());
            topology.states.insert(nbr.clone(), nbr_state);
        });
    topology.states.insert(d, to_update);
    topology
}

fn run_on_topology<A, F>(program: F, mut topology: Topology, scheduling: &Vec<i32>) -> Topology
    where
        F: Fn(RoundVM) -> (RoundVM, A) + Copy,
        A: Clone + 'static + FromStr,
{
    // For each device in the provided scheduling, run the program on the device.
    for d in scheduling {
        topology = run_on_device(program, topology, d.clone());
    }
    topology
}

#[test]
fn test_single_source() {
    let devices = vec![1, 2, 3, 4, 5];
    let scheduling: Vec<i32> = iter::repeat(devices.clone()).take(10).flatten().collect();
    let expected_results: HashMap<i32, HashMap<i32, f64>> = HashMap::from([
        (
            1,
            HashMap::from([(1, 0.0), (2, 1.0), (3, 2.0), (4, 3.0), (5, 4.0)]),
        ),
        (
            2,
            HashMap::from([(1, 1.0), (2, 0.0), (3, 1.0), (4, 2.0), (5, 3.0)]),
        ),
        (
            3,
            HashMap::from([(1, 2.0), (2, 1.0), (3, 0.0), (4, 1.0), (5, 2.0)]),
        ),
        (
            4,
            HashMap::from([(1, 3.0), (2, 2.0), (3, 1.0), (4, 0.0), (5, 1.0)]),
        ),
        (
            5,
            HashMap::from([(1, 4.0), (2, 3.0), (3, 2.0), (4, 1.0), (5, 0.0)]),
        ),
    ]);

    for d in devices.clone() {
        let mut topology = setup_test_topology(devices.clone());
        add_source(&mut topology, d);
        let final_topology = run_on_topology(gradient, topology, &scheduling);
        let results: HashMap<i32, f64> = final_topology
            .states
            .iter()
            .map(|(d, s)| {
                let result = s.exports.get(&d).unwrap().root::<f64>().clone();
                (d.clone(), result)
            })
            .collect();
        assert_eq!(results, expected_results.get(&d).unwrap().clone());
    }
}

#[test]
fn test_multiple_sources() {
    let devices = vec![1, 2, 3, 4, 5];
    let scheduling: Vec<i32> = iter::repeat(devices.clone()).take(5).flatten().collect();
    let mut topology = setup_test_topology(devices.clone());
    add_source(&mut topology, 1);
    add_source(&mut topology, 5);
    let final_topology = run_on_topology(gradient, topology, &scheduling);
    let results: HashMap<i32, f64> = final_topology
        .states
        .iter()
        .map(|(d, s)| {
            let result = s.exports.get(&d).unwrap().root::<f64>().clone();
            (d.clone(), result)
        })
        .collect();
    let expected_results: HashMap<i32, f64> =
        HashMap::from([(1, 0.0), (2, 1.0), (3, 2.0), (4, 1.0), (5, 0.0)]);
    assert_eq!(results, expected_results);
}

#[test]
fn test_exports() {
    let devices = vec![1, 2, 3, 4, 5];
    let scheduling: Vec<i32> = iter::repeat(devices.clone()).take(5).flatten().collect();
    let mut topology = setup_test_topology(devices.clone());
    add_source(&mut topology, 2);

    let final_topology = run_on_topology(gradient, topology, &scheduling);

    let actual_exports: HashMap<i32, Export> = final_topology
        .states
        .iter()
        .map(|(d, s)| (d.clone(), s.exports.get(&d).unwrap().clone()))
        .collect();

    let expected_exports: HashMap<i32, Export> = HashMap::from([
        (
            1,
            export!(
                (path!(FoldHood(0), Rep(0)), 1.0),
                (path!(Nbr(0), FoldHood(0), Rep(0)), 1),
                (path!(Nbr(1), FoldHood(0), Rep(0)), 1.0),
                (path!(Rep(0)), 1.0),
                (Path::new(), 1.0)
            ),
        ),
        (
            2,
            export!(
                (path!(FoldHood(0), Rep(0)), 2.0),
                (path!(Nbr(0), FoldHood(0), Rep(0)), 2),
                (path!(Nbr(1), FoldHood(0), Rep(0)), 0.0),
                (path!(Rep(0)), 0.0),
                (Path::new(), 0.0)
            ),
        ),
        (
            3,
            export!(
                (path!(FoldHood(0), Rep(0)), 1.0),
                (path!(Nbr(0), FoldHood(0), Rep(0)), 3),
                (path!(Nbr(1), FoldHood(0), Rep(0)), 1.0),
                (path!(Rep(0)), 1.0),
                (Path::new(), 1.0)
            ),
        ),
        (
            4,
            export!(
                (path!(FoldHood(0), Rep(0)), 2.0),
                (path!(Nbr(0), FoldHood(0), Rep(0)), 4),
                (path!(Nbr(1), FoldHood(0), Rep(0)), 2.0),
                (path!(Rep(0)), 2.0),
                (Path::new(), 2.0)
            ),
        ),
        (
            5,
            export!(
                (path!(FoldHood(0), Rep(0)), 3.0),
                (path!(Nbr(0), FoldHood(0), Rep(0)), 5),
                (path!(Nbr(1), FoldHood(0), Rep(0)), 3.0),
                (path!(Rep(0)), 3.0),
                (Path::new(), 3.0)
            ),
        ),
    ]);

    for (d, e) in actual_exports.iter() {
        let actual_root = e.root::<f64>();
        let actual_paths = e
            .paths()
            .keys()
            .map(|p| p.clone())
            .collect::<HashSet<Path>>();
        let expected_root = expected_exports.get(d).unwrap().root::<f64>();
        let expected_paths = expected_exports
            .get(d)
            .unwrap()
            .paths()
            .keys()
            .map(|p| p.clone())
            .collect::<HashSet<Path>>();
        assert_eq!(actual_root, expected_root);
        assert_eq!(actual_paths, expected_paths);
    }
}
