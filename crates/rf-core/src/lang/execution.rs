use crate::vm::round_vm::RoundVM;
use std::str::FromStr;

pub fn round<A: Clone + 'static + FromStr>(
    vm: RoundVM,
    program: impl Fn(RoundVM) -> (RoundVM, A),
) -> (RoundVM, A) {
    let (mut vm_, res) = program(vm);
    vm_.register_root(res);
    let res = vm_.export_data().root::<A>().clone();
    (vm_, res)
}
