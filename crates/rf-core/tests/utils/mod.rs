use rf_core::context::Context;
use rf_core::export::Export;
use rf_core::lang::execution::round;
use rf_core::path::Path;
use rf_core::sensor_id::SensorId;
use rf_core::vm::round_vm::RoundVM;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::str::FromStr;

pub fn init_vm() -> RoundVM {
    let context = Context::new(
        1,
        Default::default(),
        Default::default(),
        Default::default(),
    );
    init_with_ctx(context)
}

pub fn init_with_ctx(ctx: Context) -> RoundVM {
    let mut vm = RoundVM::new(ctx);
    vm.new_export_stack();
    vm
}

pub fn push_to_ctx<A: Copy + 'static>(mut ctx: Context, path: Path, val: A) -> Context {
    let mut export = Export::new();
    export.put(path, val);
    ctx.put_export(ctx.self_id().clone(), export);
    ctx
}

pub fn vm(
    self_id: i32,
    local_sensor: HashMap<SensorId, Rc<Box<dyn Any>>>,
    nbr_sensor: HashMap<SensorId, HashMap<i32, Rc<Box<dyn Any>>>>,
    exports: HashMap<i32, Export>,
) -> RoundVM {
    let context = Context::new(self_id, local_sensor, nbr_sensor, exports);
    init_with_ctx(context)
}
pub fn combine<A, F, G, H>(expr1: F, expr2: G, comb: H) -> impl Fn(RoundVM) -> (RoundVM, A)
where
    F: Fn(RoundVM) -> (RoundVM, A),
    G: Fn(RoundVM) -> (RoundVM, A),
    H: Fn(A, A) -> A,
{
    move |vm| {
        let (vm_, res1) = expr1(vm);
        let (vm__, res2) = expr2(vm_);
        (vm__, comb(res1, res2))
    }
}

pub fn assert_equivalence<A, F, G>(
    exec_order: Vec<i32>,
    nbrs: HashMap<i32, Vec<i32>>,
    program_1: F,
    program_2: G,
) -> bool
where
    F: Fn(RoundVM) -> (RoundVM, A) + Copy,
    G: Fn(RoundVM) -> (RoundVM, A) + Copy,
    A: Eq + Clone + 'static + Debug + FromStr,
{
    let states: HashMap<i32, (RoundVM, RoundVM)> = nbrs
        .iter()
        .map(|(curr, neighbors)| {
            let ex_1: HashMap<i32, Export> = neighbors
                .iter()
                .map(|nbr| (nbr.clone(), Export::new()))
                .collect();
            let ex_2: HashMap<i32, Export> = neighbors
                .iter()
                .map(|nbr| (nbr.clone(), Export::new()))
                .collect();
            (
                curr.clone(),
                (
                    vm(curr.clone(), Default::default(), Default::default(), ex_1),
                    vm(curr.clone(), Default::default(), Default::default(), ex_2),
                ),
            )
        })
        .collect();
    assert_equivalence_rec(exec_order, states, program_1, program_2)
}

fn assert_equivalence_rec<A, F, G>(
    mut exec_order: Vec<i32>,
    states: HashMap<i32, (RoundVM, RoundVM)>,
    program_1: F,
    program_2: G,
) -> bool
where
    F: Fn(RoundVM) -> (RoundVM, A) + Copy,
    G: Fn(RoundVM) -> (RoundVM, A) + Copy,
    A: Eq + Clone + 'static + Debug + FromStr,
{
    if exec_order.is_empty() {
        return true;
    }

    let curr = exec_order.pop().unwrap();

    let new_states: HashMap<i32, (RoundVM, RoundVM)> = states
        .into_iter()
        .map(|(id, (vm_1, vm_2))| {
            if id == curr {
                let (vm_1_, res_1) = round(vm_1, program_1);
                let (vm_2_, res_2) = round(vm_2, program_2);
                if res_1 != res_2 {
                    panic!("Programs are not equivalent: {:?} != {:?}", res_1, res_2);
                }
                (id, (vm_1_, vm_2_))
            } else {
                (id, (vm_1, vm_2))
            }
        })
        .collect();
    assert_equivalence_rec(exec_order, new_states, program_1, program_2)
}

pub fn fully_connected_topology_map(elems: Vec<i32>) -> HashMap<i32, Vec<i32>> {
    let new_elems = elems.clone();
    elems
        .into_iter()
        .map(|elem| (elem, new_elems.clone()))
        .collect()
}

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
