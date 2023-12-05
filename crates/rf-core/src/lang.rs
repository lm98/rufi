use crate::slot::Slot::{Branch, FoldHood, Nbr, Rep};
use crate::vm::round_vm::RoundVM;
use std::str::FromStr;

pub mod builtins;
pub mod execution;
pub mod macros;

/// Observes the value of an expression across neighbors, producing a “field of fields”.
///
/// # Arguments
///
/// * `vm` the current VM
/// * `expr` the expression to evaluate
///
/// # Generic Parameters
///
/// * `A` The type of value returned by the expression.
/// * `F` - The type of the closure, which must be a closure that takes a `RoundVM` as argument and returns a tuple `(RoundVM, A)`.
///
/// # Returns
///
/// the value of the expression
pub fn nbr<A: Clone + 'static + FromStr, F>(mut vm: RoundVM, expr: F) -> (RoundVM, A)
where
    F: Fn(RoundVM) -> (RoundVM, A),
{
    vm.nest(
        Nbr(vm.index().clone()),
        vm.unless_folding_on_others(),
        true,
        |vm| match vm.neighbor() {
            Some(nbr) if nbr != vm.self_id() => match vm.neighbor_val::<A>() {
                Ok(val) => (vm.clone(), val.clone()),
                _ => expr(vm.clone()),
            },
            _ => expr(vm),
        },
    )
}

/// Iteratively updates the value of the input expression at each device using the last computed value.
///
/// # Arguments
///
/// * `vm` the current VM
/// * `init` the initial value
/// * `fun` the function to apply to the value
///
/// # Generic Parameters
///
/// * `A` The type of value returned by the expression.
/// * `F` - The type of the closure, which must be a closure that takes no arguments and returns a value of type `A`.
/// * `G` - The type of the closure, which must be a closure that takes a tuple `(RoundVM, A)` and returns a tuple `(RoundVM, A)`.
///
/// # Returns
///
/// the updated value
pub fn rep<A: Clone + 'static + FromStr, F, G>(mut vm: RoundVM, init: F, fun: G) -> (RoundVM, A)
where
    F: Fn(RoundVM) -> (RoundVM, A),
    G: Fn(RoundVM, A) -> (RoundVM, A),
{

    vm.nest(
        Rep(vm.index().clone()),
        vm.unless_folding_on_others(),
        true,
        |vm| {
            if vm.previous_round_val::<A>().is_ok() {
                let prev = vm.previous_round_val::<A>().unwrap().clone();
                fun(vm, prev)
            } else {
                let init_args = init(vm);
                fun(init_args.0, init_args.1)
            }
        },
    )
}

/// Aggregates the results of the neighbor computation.
///
/// # Arguments
///
/// * `vm` the current VM
/// * `init` the initial value
/// * `aggr` the function to apply to the value
/// * `expr` the expression to evaluate
///
/// # Generic Parameters
///
/// * `A` The type of value returned by the expression.
/// * `F` - The type of inti, which must be a closure that takes no arguments and returns a value of type `A`.
/// * `G` - The type of aggr, which must be a closure that takes a tuple `(A, A)` and returns a value of type `A`.
/// * `H` - The type of expr, which must be a closure that takes a `RoundVM` as argument and returns a tuple `(RoundVM, A)`.
///
/// # Returns
///
/// the aggregated value
pub fn foldhood<A: Clone + 'static + FromStr, F, G, H>(
    mut vm: RoundVM,
    init: F,
    aggr: G,
    expr: H,
) -> (RoundVM, A)
where
    F: Fn(RoundVM) -> (RoundVM, A),
    G: Fn(A, A) -> A,
    H: Fn(RoundVM) -> (RoundVM, A) + Copy,
{
    vm.nest(FoldHood(vm.index().clone()), true, true, |mut vm| {
        let (vm_, local_init) = vm.locally(|vm_| init(vm_));
        let mut proxy_vm = vm_.clone();
        let mut nbr_field: Vec<A> = Vec::new();

        //fill the nbr_field with the values from neighbours
        vm_.aligned_neighbours::<A>().iter().for_each(|id| {
            let (vm__, opt) = proxy_vm.folded_eval(expr, *id);
            proxy_vm = vm__;
            nbr_field.push(opt.unwrap_or(local_init.clone()));
        });

        //fold the nbr_field with the provided aggregation function
        proxy_vm.isolate(|vm_| {
            let res = nbr_field
                .iter()
                .fold(local_init.clone(), |x, y| aggr(x, y.clone()));
            (vm_, res)
        })
    })
}

/// Partitions the domain into two subspaces that do not interact with each other.
///
/// # Arguments
///
/// * `vm` the current VM
/// * `cond` the condition to evaluate
/// * `thn` the expression to evaluate if the condition is true
/// * `els` the expression to evaluate if the condition is false
///
/// # Generic Parameters
///
/// * `A` The type of value returned by the expression.
/// * `B` - The type of cond, which must be a closure that takes no arguments and returns a value of type `bool`.
/// * `F` - The type of thn and els, which must be a closure that takes a `RoundVM` as argument and returns a tuple `(RoundVM, A)`.
///
/// # Returns
///
/// the value of the expression
pub fn branch<A: Clone + 'static + FromStr, B, TH, EL>(
    mut vm: RoundVM,
    cond: B,
    thn: TH,
    els: EL,
) -> (RoundVM, A)
where
    B: Fn() -> bool,
    TH: Fn(RoundVM) -> (RoundVM, A),
    EL: Fn(RoundVM) -> (RoundVM, A),
{

    vm.nest(
        Branch(vm.index().clone()),
        vm.unless_folding_on_others(),
        true,
        |mut vm| {
            let (mut vm_, tag) = vm.locally(|_vm1| (_vm1, cond()));
            let (vm__, val): (RoundVM, A) = match vm_.neighbor() {
                Some(nbr) if nbr != vm_.self_id() => {
                    let val_clone = vm_.neighbor_val::<A>().unwrap().clone();
                    (vm_, val_clone)
                }
                _ => {
                    if tag {
                        vm_.locally(|vm| thn(vm))
                    } else {
                        vm_.locally(|vm| els(vm))
                    }
                }
            };
            (vm__, val)
        },
    )

    /*
    vm.nest_in(Branch(vm.index().clone()));
    let (mut vm, tag) = vm.locally(|_vm1| (_vm1, cond()));
    let (mut vm_, val): (RoundVM, A) = match vm.neighbor() {
        Some(nbr) if nbr != vm.self_id() => {
            let val_clone = vm.neighbor_val::<A>().unwrap().clone();
            (vm, val_clone)
        }
        _ => {
            if tag {
                //locally(vm, thn);
                vm.locally(thn)
            } else {
                //locally(vm, els)
                vm.locally(els)
            }
        }
    };
    let res = vm_.nest_write(vm_.unless_folding_on_others(), val);
    vm_.nest_out(tag);
    (vm_, res)
        */
}

/// Returns the id of the current device.
///
/// # Arguments
///
/// * `vm` the current VM
///
/// # Returns
///
/// the id of the current device
pub fn mid(vm: RoundVM) -> (RoundVM, i32) {
    let mid = vm.self_id().clone();
    (vm, mid)
}
