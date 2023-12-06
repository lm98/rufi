use rf_core::lang::builtins::{foldhood_plus, mux};
use rf_core::lang::{nbr, rep};
use rf_core::sensor_id::sensor;
use rf_core::vm::round_vm::RoundVM;
use rf_core::{foldhood_plus, lift};

/// Compute the gradient of a source.
/// N.B. The source must be present in the local [Context] by setting the "source" [Sensor] to true.
/// # Arguments:
/// * `vm` - The RoundVM to compute the gradient on.
/// # Returns:
/// * `(RoundVM, f64)` - A tuple with the RoundVM after the gradient has been computed and the distance from the source.
pub fn gradient(vm: RoundVM) -> (RoundVM, f64) {
    fn is_source(vm: RoundVM) -> (RoundVM, bool) {
        let val = vm.local_sense::<bool>(&sensor("source")).unwrap().clone();
        (vm, val)
    }

    rep(vm, lift!(f64::INFINITY), |vm1, d| {
        mux(
            vm1,
            is_source,
            lift!(0.0),
            foldhood_plus!(lift!(f64::INFINITY), |a, b| a.min(b), |vm2| {
                let (vm_, val) = nbr(vm2, lift!(d));
                (vm_, val + 1.0)
            }),
        )
    })
}
