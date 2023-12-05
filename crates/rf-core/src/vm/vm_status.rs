use crate::path::Path;
use crate::slot::Slot;
use std::collections::LinkedList;

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
    stack: LinkedList<(Path, i32, Option<i32>)>,
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
            stack: LinkedList::new(),
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
    pub fn stack(&self) -> &LinkedList<(Path, i32, Option<i32>)> {
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
    ///
    /// # Returns
    ///
    /// A new VMStatus with the given neighbour.
    pub fn fold_into(&self, neighbour: Option<i32>) -> Self {
        Self {
            path: self.path.clone(),
            index: self.index.clone(),
            neighbour,
            stack: self.stack.clone(),
        }
    }

    /// Fold out of the current slot.
    ///
    /// # Returns
    ///
    /// A new VMStatus with no neighbour.
    pub fn fold_out(&self) -> Self {
        Self {
            path: self.path.clone(),
            index: self.index.clone(),
            neighbour: None,
            stack: self.stack.clone(),
        }
    }

    /// Push the current status on the stack.
    ///
    /// # Returns
    ///
    /// A new VMStatus with the current status pushed on the stack.
    pub fn push(&self) -> Self {
        let mut new_stack = self.stack.clone();
        new_stack.push_front((
            self.path.clone(),
            self.index.clone(),
            self.neighbour.clone(),
        ));
        Self {
            path: self.path.clone(),
            index: self.index.clone(),
            neighbour: self.neighbour.clone(),
            stack: new_stack,
        }
    }

    /// Pop the current status from the stack.
    ///
    /// # Returns
    ///
    /// A new VMStatus with the current status popped from the stack.
    pub fn pop(&self) -> Self {
        let mut new_stack = self.stack.clone();
        let front = new_stack.pop_front();
        match front {
            Some((p, i, n)) => Self {
                path: p.clone(),
                index: i.clone(),
                neighbour: n.clone(),
                stack: new_stack,
            },
            _ => panic!(),
        }
    }

    /// Nest the given slot.
    ///
    /// # Arguments
    ///
    /// * `slot` the slot to nest.
    ///
    /// # Returns
    ///
    /// A new VMStatus with the given slot nested.
    pub fn nest(&self, slot: Slot) -> Self {
        Self {
            path: self.path.push(slot),
            index: 0,
            neighbour: self.neighbour.clone(),
            stack: self.stack.clone(),
        }
    }

    /// Increment the index of the current slot.
    ///
    /// # Returns
    ///
    /// A new VMStatus with the index incremented.
    pub fn inc_index(&self) -> Self {
        Self {
            path: self.path.clone(),
            index: self.index + 1,
            neighbour: self.neighbour.clone(),
            stack: self.stack.clone(),
        }
    }
}

impl From<Path> for VMStatus {
    fn from(path: Path) -> Self {
        Self {
            path,
            index: 0,
            neighbour: None,
            stack: LinkedList::new(),
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
        let status = VMStatus::new();
        assert_eq!(status.neighbour, None);
        let s1 = status.fold_into(Some(7));
        let s2 = status.fold_into(Some(8));
        assert_eq!(status.neighbour, None);
        assert!(!status.is_folding());
        assert_eq!(s1.neighbour, Some(7));
        assert!(s1.is_folding());
        assert_eq!(s2.neighbour, Some(8));
        assert!(s2.is_folding())
    }

    #[test]
    #[should_panic]
    fn test_as_stack_panic() {
        let status = VMStatus::new();
        status
            .push()
            .fold_into(Some(7))
            .nest(Nbr(2))
            .push()
            .fold_into(Some(8))
            .nest(Rep(4))
            .inc_index()
            .push()
            .pop()
            .pop()
            .pop()
            .pop();
    }

    #[test]
    fn test_as_stack() {
        let status = VMStatus::new();
        let s1 = status.push();
        let s2 = s1.fold_into(Some(7)).nest(Nbr(2)).push();
        let s3 = s2.fold_into(Some(8)).nest(Rep(4)).inc_index().push();
        let s4 = s3.pop();
        let s5 = s4.pop();
        let s6 = s5.pop();
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
        let status = VMStatus::new();
        assert_eq!(status.index, 0);
        assert_eq!(status.inc_index().index, 1);
        assert_eq!(status.inc_index().inc_index().inc_index().index, 3);
        assert_eq!(
            status
                .inc_index()
                .inc_index()
                .nest(Nbr(0))
                .inc_index()
                .index,
            1
        )
    }
}
