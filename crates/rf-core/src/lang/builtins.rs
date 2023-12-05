use crate::lang::{foldhood, mid, nbr};
use crate::vm::round_vm::RoundVM;
use std::str::FromStr;

/// Evaluates the given expressions and returns the result based on the given condition.
/// N.B both th and el will be evaluated, thus they will both affect the [Path], but only the result of one of them will be returned.
///
/// # Arguments
/// * `vm` - The current VM.
/// * `cond` - The condition to evaluate, which should return a boolean.
/// * `th` - The then-expression to evaluate.
/// * `el` - The else-expression to evaluate.
///
/// # Returns
/// The result of the evaluation of the then-expression if the condition is true, else the result of the evaluation of the else-expression alongside the RoundVM.
pub fn mux<A, C, TH, EL>(vm: RoundVM, cond: C, th: TH, el: EL) -> (RoundVM, A)
where
    C: Fn(RoundVM) -> (RoundVM, bool),
    TH: Fn(RoundVM) -> (RoundVM, A),
    EL: Fn(RoundVM) -> (RoundVM, A),
{
    let (vm_, flag) = cond(vm);
    let (th_vm, th_val) = th(vm_);
    let (el_vm, el_val) = el(th_vm);
    if flag {
        (el_vm, th_val)
    } else {
        (el_vm, el_val)
    }
}

/// Performs a foldhood on the given expression, excluding self from the aligned neighbors.
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
/// * `F` - The type of init, which must be a closure that takes no arguments and returns a value of type `A`.
/// * `G` - The type of aggr, which must be a closure that takes a tuple `(A, A)` and returns a value of type `A`.
/// * `H` - The type of expr, which must be a closure that takes a `RoundVM` as argument and returns a tuple `(RoundVM, A)`.
///
/// # Returns
///
/// the aggregated value
pub fn foldhood_plus<A: Copy + 'static + FromStr, F, G, H>(
    vm: RoundVM,
    init: F,
    aggr: G,
    expr: H,
) -> (RoundVM, A)
where
    F: Fn(RoundVM) -> (RoundVM, A) + Copy,
    G: Fn(A, A) -> A,
    H: Fn(RoundVM) -> (RoundVM, A) + Copy,
{
    foldhood(vm, init, aggr, |vm1| {
        let (vm_, self_id) = mid(vm1);
        let (vm__, nbr_id) = nbr(vm_, |vm2| mid(vm2));
        mux(vm__, |vm3| (vm3, self_id == nbr_id), init, expr)
    })
}
