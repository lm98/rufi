use std::collections::HashMap;
use rf_core::export::Export;

/// This type alias represent the states of the device inside an aggregate computation.
pub type States = HashMap<i32, Export>;