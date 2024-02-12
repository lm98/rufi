use rf_core::lang::builtins::{foldhood_plus, mux};
use rf_core::lang::{nbr, rep};
use rf_core::sensor_id::sensor;
use rf_core::vm::round_vm::RoundVM;

/// Compute the gradient of a source.
/// N.B. The source must be present in the local [Context] by setting the "source" [Sensor] to true.
/// # Arguments:
/// * `vm` - The RoundVM to compute the gradient on.
/// # Returns:
/// * `(RoundVM, f64)` - A tuple with the RoundVM after the gradient has been computed and the distance from the source.
pub fn gradient(vm: &mut RoundVM) -> f64 {
    fn is_source(vm: &mut RoundVM) -> bool {
        vm.local_sense::<bool>(&sensor("source")).unwrap().clone()
    }

    rep(
        vm,
        |_| 0.0,
        |vm1, d| {
            mux(
                vm1,
                is_source,
                |_vm| 0.0,
                |vm2| {
                    foldhood_plus(
                        vm2,
                        |_vm| f64::INFINITY,
                        |a, b| a.min(b),
                        |vm3| nbr(vm3, |_vm| d) + 1.0,
                    )
                },
            )
        },
    )
}
