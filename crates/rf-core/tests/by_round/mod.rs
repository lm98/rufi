use crate::utils::{combine, init_vm, init_with_ctx, push_to_ctx};
use rf_core::context::Context;
use rf_core::export::Export;
use rf_core::lang::execution::round;
use rf_core::lang::{branch, foldhood, mid, nbr, rep};
use rf_core::path::Path;
use rf_core::sensor_id::sensor;
use rf_core::slot::Slot::{FoldHood, Nbr, Rep};
use rf_core::vm::round_vm::RoundVM;
use rf_core::{export, path};
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_multiple_rounds() {
    //create the vm
    let mut vm = init_vm();
    //write the aggregate program
    let program = |vm1: &mut RoundVM| {
        rep(
            vm1,
            |_vm| 0,
            |vm2, a| {
                let res = nbr(vm2, |_vm| a);
                res + 1
            },
        )
    };
    //first round
    let res: i32 = round(&mut vm, program);
    assert_eq!(1, res);
    //add to the context the result of the previous round
    let ctx_ = push_to_ctx(vm.context().clone(), Path::from(vec![Rep(0)]), res);
    //second round
    let res_: i32 = round(&mut init_with_ctx(ctx_), program);
    assert_eq!(2, res_);
}

#[test]
fn test_local_value() {
    let context = Context::new(
        0,
        Default::default(),
        Default::default(),
        Default::default(),
    );
    let result = round(&mut init_with_ctx(context), |vm| 10);
    assert_eq!(10, result);
}

#[test]
fn test_alignment() {
    // No neighbor is aligned
    let context = Context::new(
        0,
        Default::default(),
        Default::default(),
        Default::default(),
    );
    // Program: rep(0, foldhood(0)(_ + _)(1))
    let program = |vm1: &mut RoundVM| {
        rep(
            vm1,
            |_vm| 0,
            |vm2, _| foldhood(vm2, |_vm| 0, |a, b| a + b, |_vm3| 1),
        )
    };
    let result = round(&mut init_with_ctx(context), program);
    assert_eq!(1, result);

    // One neighbor is aligned
    // Export: Map(1 -> Export(Rep(0) -> 1, Rep(0) / FoldHood(0) -> 1))
    let export_dev_1 = export!((path!(Rep(0)), 1), (path!(FoldHood(0), Rep(0)), 1));
    let mut exports: HashMap<i32, Export> = HashMap::new();
    exports.insert(1, export_dev_1);
    let context = Context::new(0, Default::default(), Default::default(), exports);
    let mut vm = init_with_ctx(context);
    let result = round(&mut vm, program);
    assert_eq!(2, result);
}

#[test]
// This test differs from the Scala counterpart: in Rust, we can't assert the equality of two Exports, so we assert the equality of the root values instead
fn export_should_compose() {
    fn ctx() -> Context {
        Context::new(
            0,
            HashMap::from([(sensor("sensor"), Rc::new(Box::new(5) as Box<dyn Any>))]),
            Default::default(),
            Default::default(),
        )
    }

    let expr_1 = |_vm: &mut RoundVM| 1;
    let expr_2 = |vm: &mut RoundVM| rep(vm, |_vm1| 7, |_vm2, val| val + 1);
    let expr_3 = |vm: &mut RoundVM| {
        foldhood(
            vm,
            |_vm1| 0,
            |a, b| (a + b),
            |vm2| {
                nbr(vm2, |vm3| {
                    *vm3.local_sense::<i32>(&sensor("sensor")).unwrap()
                })
            },
        )
    };

    let mut vm = init_vm();
    let _ = round(&mut vm, combine(expr_1, expr_1.clone(), |a, b| a + b));
    assert_eq!(2, vm.export_data().root::<i32>().clone());

    let mut vm1 = init_vm();
    let _ = round(&mut vm1, combine(expr_2, expr_2.clone(), |a, b| a + b));
    assert_eq!(16, vm1.export_data().root::<i32>().clone());

    let mut vm2 = init_with_ctx(ctx());
    let _ = round(&mut vm2, combine(expr_3, expr_3.clone(), |a, b| a + b));
    assert_eq!(10, vm2.export_data().root::<i32>().clone());

    let mut vm3 = init_vm();
    let _ = round(&mut vm3, |vm: &mut RoundVM| {
        rep(
            vm,
            |vm1| 0,
            |vm2, _| rep(vm2, |_vm3| 0, |vm4, _| expr_1(vm4)),
        )
    });
    assert_eq!(1, vm3.export_data().root::<i32>().clone());

    let mut vm4 = init_vm();
    let _ = round(&mut vm4, |vm| {
        rep(
            vm,
            |vm1| 0,
            |vm2, _| rep(vm2, |vm3| 0, |vm4, _| expr_2(vm4)),
        )
    });
    assert_eq!(8, vm4.export_data().root::<i32>().clone());

    let mut vm5 = init_with_ctx(ctx());
    let _ = round(&mut vm5, |vm: &mut RoundVM| {
        rep(
            vm,
            |_vm1| 0,
            |vm2, _| rep(vm2, |_vm3| 0, |vm4, _| expr_3(vm4)),
        )
    });
    assert_eq!(5, vm5.export_data().root::<i32>().clone());
}

#[test]
fn test_foldhood_basic() {
    // Export of device 2: Export(/ -> 1, FoldHood(0) -> 1)
    let export_dev_2 = export!((path!(), 1), (path!(FoldHood(0)), 1));
    // Export of device 4: Export(/ -> 3, FoldHood(0) -> 3)
    let export_dev_4 = export!((path!(), 3), (path!(FoldHood(0)), 3));
    // Exports of the context: Map(2 -> Export(/ -> 1, FoldHood(0) -> 1), 4 -> Export(/ -> 3, FoldHood(0) -> 3))
    let mut exports: HashMap<i32, Export> = HashMap::new();
    exports.insert(2, export_dev_2);
    exports.insert(4, export_dev_4);
    let context = Context::new(0, Default::default(), Default::default(), exports);
    // Program: foldhood(1)(_ + _)(2)
    let program = |vm: &mut RoundVM| foldhood(vm, |_vm| 1, |a, b| a + b, |_vm1| 2);
    let result = round(&mut init_with_ctx(context), program);
    assert_eq!(7, result);
}

#[test]
fn test_foldhood_advanced() {
    // Export of device 2: Export(/ -> "1", FoldHood(0) -> "1", FoldHood(0) / Nbr(0) -> 4)
    let export_dev_2 = export!(
        (path!(), 1),
        (path!(FoldHood(0)), 1),
        (path!(Nbr(0), FoldHood(0)), 4)
    );
    // Export of device 4: Export(/ -> "3", FoldHood(0) -> "3")
    let export_dev_4 = export!(
        (path!(), 3),
        (path!(FoldHood(0)), 3),
        (path!(Nbr(0), FoldHood(0)), 19)
    );
    let mut exports: HashMap<i32, Export> = HashMap::new();
    exports.insert(2, export_dev_2);
    exports.insert(4, export_dev_4);
    let context = Context::new(0, Default::default(), Default::default(), exports);
    // Program: foldhood(-5)(_ + _)(nbr(2))
    let program =
        |vm: &mut RoundVM| foldhood(vm, |_vm| -5, |a, b| (a + b), |vm1| nbr(vm1, |_vm2| 2));
    let result = round(&mut init_with_ctx(context), program);
    assert_eq!(20, result);
}

#[test]
fn test_nbr() {
    fn create_exports_nbr_test() -> HashMap<i32, Export> {
        // Create this export: Map(
        //       1 -> Export(/ -> "any", FoldHood(0) -> 1, FoldHood(0) / Nbr(0) -> 1),
        //       2 -> Export(/ -> "any", FoldHood(0) -> 2, FoldHood(0) / Nbr(0) -> 2)
        //     )
        let export_dev_1 = export!(
            (path!(), "any"),
            (path!(FoldHood(0)), 1),
            (path!(Nbr(0), FoldHood(0)), 1)
        );
        let export_dev_2 = export!(
            (path!(), "any"),
            (path!(FoldHood(0)), 2),
            (path!(Nbr(0), FoldHood(0)), 2)
        );
        let mut exports: HashMap<i32, Export> = HashMap::new();
        exports.insert(1, export_dev_1);
        exports.insert(2, export_dev_2);
        exports
    }

    // 1 - NBR needs not to be nested into fold
    let context = Context::new(
        0,
        Default::default(),
        Default::default(),
        Default::default(),
    );
    let result = round(&mut init_with_ctx(context), |vm| nbr(vm, |vm1| 7));
    assert_eq!(7, result);

    // 2 - NBR should support interaction between aligned devices
    let context = Context::new(
        0,
        Default::default(),
        Default::default(),
        create_exports_nbr_test(),
    );
    // Program: foldhood(0)(_ + _)(if (nbr(mid()) == mid()) 0 else 1)
    let program = |vm: &mut RoundVM| {
        foldhood(
            vm,
            |_vm| 0,
            |a, b| a + b,
            |vm1| {
                let res = nbr(vm1, mid);
                if res == vm1.self_id().clone() {
                    0
                } else {
                    1
                }
            },
        )
    };
    let result = round(&mut init_with_ctx(context), program);
    assert_eq!(2, result);
}

#[test]
// Rep should support dynamic evolution of fields
fn test_rep() {
    let context = Context::new(
        0,
        Default::default(),
        Default::default(),
        Default::default(),
    );
    // Program: rep(9)(_ * 2)
    let program = |vm: &mut RoundVM| rep(vm, |vm| 9, |vm1, a| a * 2);
    // Check if rep use the initial value
    let result = round(&mut init_with_ctx(context), program);
    assert_eq!(18, result);

    // Export: Map(0 -> Export(Rep(0) -> 7))
    let export_dev_0 = export!((path!(Rep(0)), 7));
    let mut exports: HashMap<i32, Export> = HashMap::new();
    exports.insert(0, export_dev_0);
    let context = Context::new(0, Default::default(), Default::default(), exports);
    // Rep should build upon previous state.
    let result = round(&mut init_with_ctx(context), program);
    assert_eq!(14, result);
}

#[test]
// Branch should support domain restriction, thus affecting the structure of exports
fn test_branch() {
    // Program: rep(0) { x => branch(x % 2 == 0)(7)(rep(4)(_ => 4)); x + 1 }
    let program = |vm: &mut RoundVM| {
        rep(
            vm,
            |_vm| 0,
            |vm1, x| {
                branch(
                    vm1,
                    || x % 2 == 0,
                    |_vm3| 7,
                    |vm4| rep(vm4, |_vm| 4, |_vm5, _| 4),
                );
                x + 1
            },
        )
    };
    let context = Context::new(
        0,
        Default::default(),
        Default::default(),
        Default::default(),
    );
    let result = round(&mut init_with_ctx(context), program);
    assert_eq!(1, result);

    // Export: Map(0 -> Export(Rep(0) -> 1))
    let export_dev_0 = export!((path!(Rep(0)), 1));
    let mut exports: HashMap<i32, Export> = HashMap::new();
    exports.insert(0, export_dev_0);
    let context = Context::new(0, Default::default(), Default::default(), exports);
    let result = round(&mut init_with_ctx(context), program);
    assert_eq!(2, result);
}

#[test]
fn test_sense() {
    // Sense should simply evaluate to the last value read by sensor
    fn ctx() -> Context {
        Context::new(
            0,
            HashMap::from([
                (sensor("a"), Rc::new(Box::new(7) as Box<dyn Any>)),
                (sensor("b"), Rc::new(Box::new("right") as Box<dyn Any>)),
            ]),
            Default::default(),
            Default::default(),
        )
    }

    let res = round(&mut init_with_ctx(ctx()), |vm| {
        vm.local_sense::<i32>(&sensor("a")).unwrap().clone()
    });
    assert_eq!(7, res);

    let res = round(&mut init_with_ctx(ctx()), |vm| {
        vm.local_sense::<&str>(&sensor("b"))
            .cloned()
            .unwrap()
            .to_string()
    });
    assert_eq!("right", res);
}
