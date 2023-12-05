use crate::context::Context;
use crate::export::{Export, Result};
use crate::path::Path;
use crate::sensor_id::SensorId;
use crate::slot::Slot;
use crate::vm::vm_status::VMStatus;
use std::str::FromStr;

/// A Round correspond to a local computation in a device. Create the context, evaluate the aggregate program and share the exports to the neighborhood.
///
/// * `context` - The context of the current round.
///
/// * `status` - The status of the current round.
///
/// * `export_stack` - The stack of exports of the current round.
#[derive(Debug, Clone)]
pub struct RoundVM {
    context: Context,
    status: VMStatus,
    export_stack: Vec<Export>,
    isolated: bool,
}

impl RoundVM {
    /// Create a new RoundVM
    ///
    /// ### Arguments
    ///
    /// * `context` - The context of the current round.
    ///
    /// # Returns
    ///
    /// A `RoundVM` instance.
    pub fn new(context: Context) -> Self {
        Self {
            context,
            status: VMStatus::new(),
            export_stack: vec![],
            isolated: false,
        }
    }

    /// Get the first export of the stack.
    ///
    /// # Returns
    ///
    /// The first export of the stack, of type `&mut Export`.
    pub fn export_data(&mut self) -> &mut Export {
        self.export_stack.first_mut().unwrap()
    }

    /// # Returns
    ///
    /// The id of the device, of type `i32`.
    pub fn self_id(&self) -> &i32 {
        self.context.self_id()
    }

    /// Register the given value for the root path.
    ///
    /// # Arguments
    ///
    /// * `v` - The value to register.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of value. It must implement the `Copy` trait
    ///         and have a `'static` lifetime.
    pub fn register_root<A: 'static + Clone>(&mut self, v: A) {
        self.export_data().put(Path::new(), v.clone());
    }

    /// If the computation is folding on a neighbor, return the id of the neighbor
    ///
    /// # Returns
    ///
    /// An `&Option<i32>` containing the id of the neighbor, if present
    pub fn neighbor(&self) -> &Option<i32> {
        self.status.neighbour()
    }

    /// # Returns
    ///
    ///  The index of the current computation.
    pub fn index(&self) -> i32 {
        self.status.index()
    }

    /// Obtain the value of the previous round for the current device and the current path.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of value. It must implement the `Clone` trait
    ///         and have a `'static` lifetime.
    ///
    /// # Returns
    ///
    /// An `Option` containing the value of the current path for the current device, if present.
    pub fn previous_round_val<A: 'static + Clone + FromStr>(&self) -> Result<A> {
        self.context
            .read_export_value::<A>(&self.self_id(), self.status.path())
    }

    /// Obtain the value of the current path for the current neighbor
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of value. It must implement the `Clone` trait
    ///         and have a `'static` lifetime.
    ///
    /// # Returns
    ///
    ///  An `Option` containing the value of the current path for the current neighbor, if present.
    pub fn neighbor_val<A: 'static + Clone + FromStr>(&self) -> Result<A> {
        let n: Result<i32> = self.neighbor().ok_or("Isolated".into());
        self.context.read_export_value::<A>(&n?, self.status.path())
    }

    /// Obtain the local value of a given sensor.
    ///
    /// # Arguments
    ///
    /// * - `sensor_id` - The id of the sensor.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of value returned by the sensor. It must have a `'static` lifetime.
    ///
    /// # Returns
    ///
    /// An `Option` containing the local value of the given sensor, if present.
    pub fn local_sense<A: 'static>(&self, sensor_id: &SensorId) -> Option<&A> {
        self.context.local_sense::<A>(sensor_id)
    }

    /// Obtain the value of a given sensor for the current neighbor.
    ///
    /// # Arguments
    ///
    /// * `sensor_id` - The id of the sensor.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of value returned by the sensor. It must have a `'static` lifetime.
    ///
    /// # Returns
    ///
    /// An `Option` containing the value of the given sensor for the current neighbor, if present.
    pub fn nbr_sense<A: 'static>(&self, sensor_id: &SensorId) -> Option<&A> {
        self.neighbor()
            .map(|id| self.context.nbr_sense::<A>(sensor_id, &id))
            .flatten()
    }

    /// Evaluates the given expression locally and return the result.
    ///
    /// # Arguments
    ///
    /// * `expr` The expression to evaluate, which takes a [RoundVM] as argument and returns a tuple of `RoundVM` and `A`.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of value returned by the expression.
    /// * `F` - The type of the closure, which must be a mutable closure that takes a [RoundVM] as argument and returns a tuple of `RoundVM` and `A`.
    ///
    /// # Returns
    ///
    /// The result of the closure `expr`.
    pub fn locally<A: Clone + 'static + FromStr, F>(&mut self, expr: F) -> (RoundVM, A)
    where
        F: Fn(RoundVM) -> (RoundVM, A),
    {
        let mut proxy = self.clone();
        let current_neighbour = proxy.neighbor().map(|id| id.clone());
        proxy.status = proxy.status.fold_out();
        let (mut vm_, result) = expr(proxy.clone());
        vm_.status = vm_.status.fold_into(current_neighbour);
        (vm_, result)
    }

    /// Perform a folded evaluation of the given expression in the given neighbor and return the result.
    ///
    /// # Arguments
    ///
    /// * `expr` - The expression to evaluate, which takes a [RoundVM] as argument and returns a tuple of `RoundVM` and `A`.
    /// * `id` - The id of the neighbor. It is of type `i32`.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of value returned by the expression.
    /// * `F` - The type of the expression, which must be a closure that takes a [RoundVM] as argument and returns a tuple of `RoundVM` and `A`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the result of the expression.
    pub fn folded_eval<A: Clone + 'static, F>(&mut self, expr: F, id: i32) -> (RoundVM, Option<A>)
    where
        F: Fn(RoundVM) -> (RoundVM, A),
    {
        let mut proxy = self.clone();
        proxy.status = proxy.status.push();
        proxy.status = proxy.status.fold_into(Some(id));
        let (mut vm_, result) = expr(proxy.clone());
        vm_.status = vm_.status.pop();
        (vm_, Some(result))
    }

    /// Evaluate the given expression while also writing on the [Export] stack.
    ///
    /// # Arguments
    ///
    /// * `slot` - The slot to write in the current [Path].
    /// * `write` - A boolean indicating whether to write the result of the expression on the `Export` stack.
    /// * `inc` - A boolean indicating whether to increment the index of the current [VMStatus].
    /// * `expr` - The expression to evaluate, which takes a [RoundVM] as argument and returns a tuple of `RoundVM` and `A`.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of value returned by the expression.
    /// * `F` - The type of the expression, which must be a closure that takes a [RoundVM] as argument and returns a tuple of `RoundVM` and `A`.
    ///
    /// # Returns
    ///
    /// A tuple of `RoundVM` and `A`.
    pub fn nest<A: Clone + 'static + FromStr, F>(
        &mut self,
        slot: Slot,
        write: bool,
        inc: bool,
        expr: F,
    ) -> (RoundVM, A)
    where
        F: Fn(RoundVM) -> (RoundVM, A),
    {
        let mut proxy = self.clone();
        proxy.status = proxy.status.push().nest(slot);
        let (mut vm, val) = expr(proxy);
        let res = if write {
            let cloned_path = vm.status.path().clone();
            vm.export_data()
                .get::<A>(&cloned_path)
                .unwrap_or(
                    vm.export_data()
                        .put_lazy_and_return(cloned_path, || val.clone()),
                )
                .clone()
        } else {
            val
        };
        vm.status = match inc {
            true => vm.status.pop().inc_index(),
            false => vm.status.pop(),
        };
        (vm, res)
    }

    /// Get a vector of aligned neighbor identifiers.
    ///
    /// # Returns
    ///
    /// A vector of aligned neighbor identifiers.
    pub fn aligned_neighbours<A: 'static + FromStr + Clone>(&self) -> Vec<i32> {
        let mut tmp: Vec<i32> = Vec::new();
        if !self.isolated {
            tmp = self
                .context
                .exports()
                .clone()
                .into_iter()
                .filter(|(id, _)| id != self.self_id())
                .filter(|(_, export)| {
                    self.status.path().is_root() || export.get::<A>(self.status.path()).is_ok()
                })
                .map(|(id, _)| id.clone())
                .collect();
            tmp.insert(0, self.self_id().clone());
        }
        tmp
    }

    /// Isolate the current device and evaluate the given expression
    ///
    /// # Arguments
    ///
    /// * `expr` - The closure to execute, which takes a [RoundVM] as argument and returns a tuple of `RoundVM` and `A`.
    ///
    /// # Generic Parameters
    ///
    /// * `A` - The type of value returned by the closure.
    /// * `F` - The type of the closure, which must be a mutable closure takes a [RoundVM] as argument and returns a tuple of `RoundVM` and `A`.
    ///
    /// # Returns
    ///
    /// The result of the closure `expr`.
    pub fn isolate<A, F>(&mut self, expr: F) -> (RoundVM, A)
    where
        F: Fn(RoundVM) -> (RoundVM, A),
    {
        let mut proxy = self.clone();
        let was_isolated = proxy.isolated;
        proxy.isolated = true;
        let (mut vm_, result) = expr(proxy.clone());
        vm_.isolated = was_isolated;
        (vm_, result)
    }

    /// Check if folding is not being performed on neighbor.
    ///
    /// # Returns
    ///
    /// - `true` if folding is being performed on self.
    /// - `false` if folding is being performed on neighbor.
    pub fn unless_folding_on_others(&self) -> bool {
        match self.neighbor() {
            Some(neighbor) => neighbor == self.self_id(),
            None => true,
        }
    }

    /// Check if folding is being performed on self.
    ///
    /// # Returns
    ///
    /// - `true` if folding is being performed on self.
    /// - `false` otherwise.
    pub fn only_when_folding_on_self(&self) -> bool {
        match self.neighbor() {
            Some(neighbor) => neighbor == self.self_id(),
            _ => false,
        }
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Create a new export stack with an empty [Export]. This function needs to be called when a new
    /// [RoundVM] is created.
    pub fn new_export_stack(&mut self) {
        self.export_stack.push(Export::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::context::Context;
    use crate::export;
    use crate::export::Export;
    use crate::path;
    use crate::path::Path;
    use crate::sensor_id::{sensor, SensorId};
    use crate::slot::Slot::{Nbr, Rep};
    use crate::vm::round_vm::RoundVM;
    use crate::vm::vm_status::VMStatus;
    use std::any::Any;
    use std::collections::HashMap;
    use std::rc::Rc;
    fn round_vm_builder() -> RoundVM {
        let local_sensor =
            HashMap::from([(sensor("sensor1"), Rc::new(Box::new(10) as Box<dyn Any>))]);
        let nbr_sensor = HashMap::from([(
            sensor("sensor1"),
            HashMap::from([(0, Rc::new(Box::new(4) as Box<dyn Any>))]),
        )]);
        let exports = HashMap::from([
            (
                7,
                /*Export::from(HashMap::from([(
                    Path::from(vec![Rep(0), Nbr(0)]),
                    Box::new(10) as Box<dyn Any>,
                )]))*/
                export!((path!(Nbr(0), Rep(0)), 10)),
            ),
            (
                0,
                /*Export::from(HashMap::from([(
                    Path::from(vec![Rep(0), Nbr(0)]),
                    Box::new(2) as Box<dyn Any>,
                )]))*/
                export!((path!(Nbr(0), Rep(0)), 2)),
            ),
        ]);

        let context = Context::new(7, local_sensor, nbr_sensor, exports);
        let mut vm = RoundVM::new(context);
        vm.export_stack.push(export!((Path::new(), 0)));
        let status = VMStatus::new();
        vm.status = status.fold_into(Some(0));
        vm
    }

    fn expr(vm: RoundVM) -> (RoundVM, i32) {
        (vm, 5 * 3)
    }

    #[test]
    fn test_export_data() {
        let mut vm = round_vm_builder();
        assert_eq!(vm.export_data().root::<i32>(), 0)
    }

    #[test]
    fn test_register_root() {
        let mut vm = round_vm_builder();
        vm.register_root(5 * 3);
        assert_eq!(vm.export_data().root::<i32>(), 15)
    }

    #[test]
    fn test_folded_eval() {
        let mut vm = round_vm_builder();
        let result = vm.folded_eval(expr, 7);
        assert_eq!(round_vm_builder().status, vm.status);
        assert_eq!(result.1.unwrap(), 15)
    }

    #[test]
    fn test_previous_round_val() {
        let mut vm = round_vm_builder();
        vm.status = vm.status.nest(Rep(0)).nest(Nbr(0));
        assert_eq!(vm.previous_round_val::<i32>().unwrap(), 10)
    }

    #[test]
    fn test_neighbor_val() {
        let mut vm = round_vm_builder();
        vm.status = vm.status.nest(Rep(0)).nest(Nbr(0));
        assert_eq!(vm.neighbor_val::<i32>().unwrap(), 2)
    }

    #[test]
    fn test_local_sense() {
        let vm = round_vm_builder();
        assert_eq!(
            vm.local_sense::<i32>(&SensorId::new("sensor1".to_string()))
                .unwrap(),
            &10
        )
    }

    #[test]
    fn test_nbr_sense() {
        let vm = round_vm_builder();
        assert_eq!(
            vm.nbr_sense::<i32>(&SensorId::new("sensor1".to_string()))
                .unwrap(),
            &4
        )
    }

    #[test]
    fn test_aligned_neighbours() {
        let vm = round_vm_builder();
        assert_eq!(vm.aligned_neighbours::<i32>(), vec![7, 0])
    }

    #[test]
    fn test_isolate() {
        let mut vm = round_vm_builder();
        let was_isolated = vm.isolated.clone();
        let result = vm.isolate(|vm| (vm, 5 * 3));
        assert_eq!(vm.isolated, was_isolated);
        assert_eq!(result.1, 15)
    }

    #[test]
    fn test_unless_folding_on_others() {
        let mut vm = round_vm_builder();
        assert!(!vm.unless_folding_on_others());
        vm.status = vm.status.fold_into(None);
        assert!(vm.unless_folding_on_others());
        vm.status = vm.status.fold_into(Some(7));
        assert!(vm.unless_folding_on_others());
    }

    #[test]
    fn test_only_when_folding_on_self() {
        let mut vm = round_vm_builder();
        assert!(!vm.only_when_folding_on_self());
        vm.status = vm.status.fold_into(None);
        assert!(!vm.only_when_folding_on_self());
        vm.status = vm.status.fold_into(Some(7));
        assert!(vm.only_when_folding_on_self());
    }
}
