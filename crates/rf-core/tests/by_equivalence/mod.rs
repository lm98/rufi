use crate::utils::{assert_equivalence, fully_connected_topology_map};
use rand::Rng;
use rf_core::lang::{foldhood, mid, nbr, rep};
use rf_core::vm::round_vm::RoundVM;
use std::collections::HashMap;

struct Fixture {
    exec_order: Vec<i32>,
    nbrs: HashMap<i32, Vec<i32>>,
}

impl Fixture {
    fn new() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            exec_order: std::iter::repeat_with(|| rng.gen_range(0..3))
                .take(100)
                .collect(),
            nbrs: fully_connected_topology_map(vec![0, 1, 2]),
        }
    }
}

#[test]
fn foldhood_multiple_nbrs() {
    let fixture = Fixture::new();
    let program_1 = |vm: &mut RoundVM| {
        foldhood(
            vm,
            |vm| 0,
            |a, b| a + b,
            |vm| {
                let nbr_1 = nbr(vm, |_vm| 1);
                let nbr_2 = nbr(vm, |_vm| 2);
                let nbr_3 = nbr(vm, mid);
                nbr_1 + nbr_2 + nbr_3
            },
        )
    };

    let program_2 = |vm: &mut RoundVM| {
        foldhood(
            vm,
            |_vm| 0,
            |a, b| a + b,
            |vm| nbr(vm, |vm| 1 + 2 + mid(vm)),
        )
    };

    assert_equivalence(fixture.exec_order, fixture.nbrs, program_1, program_2);
}

#[test]
fn nbr_nbr_ignored() {
    let fixture = Fixture::new();
    let program_1 = |vm: &mut RoundVM| {
        foldhood(
            vm,
            |_vm| 0,
            |a, b| a + b,
            |vm| {
                nbr(vm, |vm| {
                    let mid_1 = mid(vm);
                    let nbr_1 = nbr(vm, mid);
                    mid_1 + nbr_1
                })
            },
        )
    };

    let program_2 = |vm: &mut RoundVM| 2 * foldhood(vm, |_vm| 0, |a, b| a + b, |vm| nbr(vm, mid));

    assert_equivalence(fixture.exec_order, fixture.nbrs, program_1, program_2);
}

#[test]
fn rep_nbr_ignored_first_argument() {
    let fixture = Fixture::new();
    let program_1 = |vm: &mut RoundVM| {
        foldhood(
            vm,
            |_vm| 0,
            |a, b| a + b,
            |vm| rep(vm, |vm| nbr(vm, mid), |_vm, a| a),
        )
    };

    let program_2 =
        |vm: &mut RoundVM| foldhood(vm, |_vm| 0, |a, b| a + b, |vm| rep(vm, mid, |_vm, a| a));

    assert_equivalence(fixture.exec_order, fixture.nbrs, program_1, program_2);
}

#[test]
fn rep_nbr_ignored_overall() {
    let fixture = Fixture::new();
    let program_1 = |vm: &mut RoundVM| {
        foldhood(
            vm,
            |vm| 0,
            |a, b| a + b,
            |vm| {
                rep(
                    vm,
                    |vm| nbr(vm, |vm_| mid(vm_)),
                    |vm, a| {
                        let nbr_1 = nbr(vm, |_vm| a);
                        let nbr_2 = nbr(vm, mid);
                        a + nbr_1 + nbr_2
                    },
                )
            },
        )
    };

    let program_2 = |vm: &mut RoundVM| {
        foldhood(
            vm,
            |_vm| 0,
            |a, b| a + b,
            |vm| {
                rep(
                    vm,
                    |vm| mid(vm),
                    |vm, a| {
                        let nbr_1 = nbr(vm, mid);
                        a * 2 + nbr_1
                    },
                )
            },
        )
    };

    assert_equivalence(fixture.exec_order, fixture.nbrs, program_1, program_2);
}

#[test]
fn fold_init_nbr_ignored() {
    let fixture = Fixture::new();
    let program_1 = |vm: &mut RoundVM| {
        foldhood(
            vm,
            |_vm| 0,
            |a, b| a + b,
            |vm| foldhood(vm, |vm| nbr(vm, mid), |a, b| a + b, |_vm| 1),
        )
    };

    let program_2 = |vm: &mut RoundVM| {
        let res_1 = foldhood(vm, |_vm| 0, |a, b| a + b, |_vm| 1);
        let res_2 = foldhood(vm, mid, |a, b| a + b, |_vm| 1);
        res_1 * res_2
    };

    assert_equivalence(fixture.exec_order, fixture.nbrs, program_1, program_2);
}

#[test]
fn fold_fold_work() {
    let fixture = Fixture::new();
    let program_1 = |vm: &mut RoundVM| {
        foldhood(
            vm,
            |_vm| 0,
            |a, b| a + b,
            |vm| foldhood(vm, |_vm| 0, |a, b| a + b, |_vm| 1),
        )
    };

    let program_2 = |vm: &mut RoundVM| {
        let res = foldhood(
            vm,
            |_vm| 0,
            |a, b| a + b,
            //for some reason rust compiler infers the 1 to be i8 here
            |_vm| 1i32,
        );
        res.pow(2)
    };

    assert_equivalence(fixture.exec_order, fixture.nbrs, program_1, program_2);
}
