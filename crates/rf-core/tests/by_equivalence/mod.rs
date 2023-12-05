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

#[cfg(test)]
mod macros {
    use crate::by_equivalence::Fixture;
    use crate::utils::assert_equivalence;
    use rf_core::lang::{mid, nbr};
    use rf_core::{mid, nbr};

    #[test]
    fn macros() {
        let fixture = Fixture::new();

        let program_1 = nbr!(mid!());
        let program_2 = |vm| nbr(vm, |vm| mid(vm));

        assert_equivalence(fixture.exec_order, fixture.nbrs, program_1, program_2);
    }
}

#[test]
fn foldhood_multiple_nbrs() {
    let fixture = Fixture::new();
    let program_1 = |vm: RoundVM| {
        foldhood(
            vm,
            |vm| (vm, 0),
            |a, b| a + b,
            |vm| {
                let (vm_, nbr_1) = nbr(vm, |vm| (vm, 1));
                let (vm__, nbr_2) = nbr(vm_, |vm| (vm, 2));
                let (vm___, nbr_3) = nbr(vm__, |vm| mid(vm));
                (vm___, nbr_1 + nbr_2 + nbr_3)
            },
        )
    };

    let program_2 = |vm: RoundVM| {
        foldhood(
            vm,
            |vm| (vm, 0),
            |a, b| a + b,
            |vm| {
                nbr(vm, |vm| {
                    let (vm_, i) = mid(vm);
                    (vm_, 1 + 2 + i)
                })
            },
        )
    };

    assert_equivalence(fixture.exec_order, fixture.nbrs, program_1, program_2);
}

#[test]
fn nbr_nbr_ignored() {
    let fixture = Fixture::new();
    let program_1 = |vm: RoundVM| {
        foldhood(
            vm,
            |vm| (vm, 0),
            |a, b| a + b,
            |vm| {
                nbr(vm, |vm| {
                    let (vm_, mid_1) = mid(vm);
                    let (vm__, nbr_1) = nbr(vm_, |vm| mid(vm));
                    (vm__, mid_1 + nbr_1)
                })
            },
        )
    };

    let program_2 = |vm: RoundVM| {
        let (vm_, res) = foldhood(vm, |vm| (vm, 0), |a, b| a + b, |vm| nbr(vm, |vm| mid(vm)));
        (vm_, 2 * res)
    };

    assert_equivalence(fixture.exec_order, fixture.nbrs, program_1, program_2);
}

#[test]
fn rep_nbr_ignored_first_argument() {
    let fixture = Fixture::new();
    let program_1 = |vm: RoundVM| {
        foldhood(
            vm,
            |vm| (vm, 0),
            |a, b| a + b,
            |vm| rep(vm, |vm| nbr(vm, |vm_| mid(vm_)), |vm, a| (vm, a)),
        )
    };

    let program_2 = |vm: RoundVM| {
        foldhood(
            vm,
            |vm| (vm, 0),
            |a, b| a + b,
            |vm| rep(vm, |vm| mid(vm), |vm, a| (vm, a)),
        )
    };

    assert_equivalence(fixture.exec_order, fixture.nbrs, program_1, program_2);
}

#[test]
fn rep_nbr_ignored_overall() {
    let fixture = Fixture::new();
    let program_1 = |vm: RoundVM| {
        foldhood(
            vm,
            |vm| (vm, 0),
            |a, b| a + b,
            |vm| {
                rep(
                    vm,
                    |vm| nbr(vm, |vm_| mid(vm_)),
                    |vm, a| {
                        let (vm_1, nbr_1) = nbr(vm, |vm| (vm, a.clone()));
                        let (vm_2, nbr_2) = nbr(vm_1, |vm| mid(vm));
                        (vm_2, a + nbr_1 + nbr_2)
                    },
                )
            },
        )
    };

    let program_2 = |vm: RoundVM| {
        foldhood(
            vm,
            |vm| (vm, 0),
            |a, b| a + b,
            |vm| {
                rep(
                    vm,
                    |vm| mid(vm),
                    |vm, a| {
                        let (vm_1, nbr_1) = nbr(vm, |vm| mid(vm));
                        (vm_1, a * 2 + nbr_1)
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
    let program_1 = |vm: RoundVM| {
        foldhood(
            vm,
            |vm| (vm, 0),
            |a, b| a + b,
            |vm| foldhood(vm, |vm| nbr(vm, |vm| mid(vm)), |a, b| a + b, |vm| (vm, 1)),
        )
    };

    let program_2 = |vm: RoundVM| {
        let (vm_, res_1) = foldhood(vm, |vm| (vm, 0), |a, b| a + b, |vm| (vm, 1));
        let (vm__, res_2) = foldhood(vm_, |vm| mid(vm), |a, b| a + b, |vm| (vm, 1));
        (vm__, res_1 * res_2)
    };

    assert_equivalence(fixture.exec_order, fixture.nbrs, program_1, program_2);
}

#[test]
fn fold_fold_work() {
    let fixture = Fixture::new();
    let program_1 = |vm: RoundVM| {
        foldhood(
            vm,
            |vm| (vm, 0),
            |a, b| a + b,
            |vm| foldhood(vm, |vm| (vm, 0), |a, b| a + b, |vm| (vm, 1)),
        )
    };

    let program_2 = |vm: RoundVM| {
        let (vm_, res) = foldhood(
            vm,
            |vm| (vm, 0),
            |a, b| a + b,
            //for some reason rust compiler infers the 1 to be i8 here
            |vm| (vm, 1i32),
        );
        (vm_, res.pow(2))
    };

    assert_equivalence(fixture.exec_order, fixture.nbrs, program_1, program_2);
}
