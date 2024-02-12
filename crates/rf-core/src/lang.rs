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
pub fn nbr<A: Clone + 'static + FromStr, F>(vm: &mut RoundVM, expr: F) -> A
where
    F: Fn(&mut RoundVM) -> A,
{
    vm.nest(
        Nbr(vm.index()),
        vm.unless_folding_on_others(),
        true,
        |vm| match vm.neighbor() {
            Some(nbr) if nbr != vm.self_id() => match vm.neighbor_val::<A>() {
                Ok(val) => val,
                _ => expr(vm),
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
pub fn rep<A: Clone + 'static + FromStr, F, G>(vm: &mut RoundVM, init: F, fun: G) -> A
where
    F: Fn(&mut RoundVM) -> A,
    G: Fn(&mut RoundVM, A) -> A,
{
    vm.nest(Rep(vm.index()), vm.unless_folding_on_others(), true, |vm| {
        if vm.previous_round_val::<A>().is_ok() {
            let prev = vm.previous_round_val::<A>().unwrap().clone();
            fun(vm, prev)
        } else {
            let init_args = init(vm);
            fun(vm, init_args)
        }
    })
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
    vm: &mut RoundVM,
    init: F,
    aggr: G,
    expr: H,
) -> A
where
    F: Fn(&mut RoundVM) -> A + Copy,
    G: Fn(A, A) -> A,
    H: Fn(&mut RoundVM) -> A + Copy,
{
    vm.nest(FoldHood(vm.index()), true, true, |vm| {
        let local_init = vm.locally(init);
        let mut nbr_field: Vec<A> = Vec::new();

        //fill the nbr_field with the values from neighbours
        vm.aligned_neighbours::<A>().iter().for_each(|id| {
            let opt = vm.folded_eval(expr, *id);
            nbr_field.push(opt.unwrap_or(local_init.clone()));
        });

        //fold the nbr_field with the provided aggregation function
        vm.isolate(|_vm| {
            let res = nbr_field
                .iter()
                .fold(local_init.clone(), |x, y| aggr(x, y.clone()));
            res
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
    vm: &mut RoundVM,
    cond: B,
    thn: TH,
    els: EL,
) -> A
where
    B: Fn() -> bool,
    TH: Fn(&mut RoundVM) -> A + Copy,
    EL: Fn(&mut RoundVM) -> A + Copy,
{
    vm.nest(
        Branch(vm.index()),
        vm.unless_folding_on_others(),
        true,
        |vm| {
            let tag = vm.locally(|_vm1| cond());
            let val: A = match vm.neighbor() {
                Some(nbr) if nbr != vm.self_id() => {
                    vm.neighbor_val::<A>().unwrap()
                }
                _ => {
                    if tag {
                        vm.locally(thn)
                    } else {
                        vm.locally(els)
                    }
                }
            };
            val
        },
    )
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
pub fn mid(vm: &mut RoundVM) -> i32 {
    *vm.self_id()
}
