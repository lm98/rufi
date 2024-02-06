use crate::path::Path;
use crate::slot::Slot;
use std::collections::VecDeque;

/// # Models the status of the virtual machine.
///
/// * `path` - The path of the computation.
/// * `index` - The index of the current slot.
/// * `neighbour` - The id of the current neighbour. If the current slot is not a folding slot, this value is None.
/// * `stack` - Stack that contains the list of the statuses
#[derive(Debug, PartialEq, Clone)]
pub struct VMStatus {
    path: Path,
    index: i32,
    neighbour: Option<i32>,
    stack: VecDeque<(Path, i32, Option<i32>)>,
}

impl VMStatus {
    /// Create new VMStatus.
    ///
    /// # Returns
    ///
    /// The new VMStatus.
    pub fn new() -> Self {
        Self {
            path: Path::new(),
            index: 0,
            neighbour: None,
            stack: VecDeque::new(),
        }
    }

    /// # Returns
    ///
    /// The current [Path].
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// # Returns
    ///
    /// The current index.
    pub fn index(&self) -> i32 {
        self.index
    }

    /// # Returns
    ///
    /// The current neighbour.
    pub fn neighbour(&self) -> &Option<i32> {
        &self.neighbour
    }

    /// # Returns
    ///
    /// The stack of statuses.
    pub fn stack(&self) -> &VecDeque<(Path, i32, Option<i32>)> {
        &self.stack
    }

    /// Whether the VM is folding or not.
    ///
    /// # Returns
    ///
    /// True if the VM is folding, false otherwise.
    pub fn is_folding(&self) -> bool {
        self.neighbour.is_some()
    }

    /// Fold into the given neighbour.
    ///
    /// # Arguments
    /// * `neighbour` he id of the neighbour.
    pub fn fold_into(&mut self, neighbour: Option<i32>) {
        self.neighbour = neighbour;
    }

    /// Fold out of the current slot, removing the neighbour.
    pub fn fold_out(&mut self) {
        self.neighbour = None;
    }

    /// Push the current status on the stack.
    pub fn push(&mut self) {
        self.stack.push_front((self.path.clone(), self.index, self.neighbour));
    }

    /// Pop the current status from the stack.
    pub fn pop(&mut self) {
        if let Some((p, i, n)) = self.stack.pop_front() {
            self.path = p;
            self.index = i;
            self.neighbour = n;
        }
    }

    /// Nest the given slot, pushing it into the path.
    ///
    /// # Arguments
    ///
    /// * `slot` the slot to nest.
    pub fn nest(&mut self, slot: Slot) {
        self.path.push(slot);
        self.index = 0;
    }

    /// Increment the index of the current slot.
    pub fn inc_index(&mut self) {
        self.index += 1;
    }
}

impl From<Path> for VMStatus {
    fn from(path: Path) -> Self {
        Self {
            path,
            index: 0,
            neighbour: None,
            stack: VecDeque::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::path::Path;
    use crate::slot::Slot::{Nbr, Rep};

    #[test]
    fn test_empty() {
        let status = VMStatus::new();
        assert_eq!(status.path, Path::new());
        assert_eq!(status.index, 0);
        assert_eq!(status.neighbour, None)
    }

    #[test]
    fn test_fold_unfold() {
        let mut status_1 = VMStatus::new();
        let mut status_2 = VMStatus::new();
        assert_eq!(status_1.neighbour, None);
        assert!(!status_1.is_folding());
        status_1.fold_into(Some(7));
        status_2.fold_into(Some(8));
        assert_eq!(status_1.neighbour, Some(7));
        assert!(status_1.is_folding());
        assert_eq!(status_2.neighbour, Some(8));
        assert!(status_2.is_folding())
    }

    #[test]
    fn test_as_stack() {
        let mut status = VMStatus::new();
        status.push();
        let mut s2 = status.clone();
        s2.fold_into(Some(7));
        s2.nest(Nbr(2));
        s2.push();
        let mut s3 = s2.clone();
        s3.fold_into(Some(8));
        s3.nest(Rep(4));
        s3.inc_index();
        s3.push();
        let mut s4 = s3.clone();
        s4.pop();
        let mut s5 = s4.clone();
        s5.pop();
        let mut s6 = s5.clone();
        s6.pop();
        assert_eq!(s4.index, 1);
        assert_eq!(s4.neighbour, Some(8));
        assert_eq!(s4.path, Path::from(vec![Nbr(2), Rep(4)]));
        assert_eq!(s5.index, 0);
        assert_eq!(s5.neighbour, Some(7));
        assert_eq!(s5.path, Path::from(vec![Nbr(2)]));
        assert_eq!(s6.index, 0);
        assert_eq!(s6.neighbour, None);
        assert_eq!(s6.path, Path::new());
    }

    #[test]
    fn test_index() {
        let mut status = VMStatus::new();
        assert_eq!(status.index, 0);
        status.inc_index();
        assert_eq!(status.index, 1);
        status.inc_index();
        status.inc_index();
        assert_eq!(status.index, 3);
        status.nest(Nbr(0));
        assert_eq!(status.index, 0);
        status.inc_index();
        assert_eq!(status.index, 1);
    }
}
