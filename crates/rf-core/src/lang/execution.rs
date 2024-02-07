use crate::vm::round_vm::RoundVM;
use std::str::FromStr;

pub fn round<A: Clone + 'static + FromStr>(
    vm: &mut RoundVM,
    program: impl Fn(&mut RoundVM) -> A,
) -> A {
    let res = program(vm);
    vm.register_root(res);
    vm.export_data().root::<A>()
}
