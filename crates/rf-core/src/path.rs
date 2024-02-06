use std::collections::VecDeque;
use crate::slot::Slot;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// A Path is a collection of Slots that behave like an immutable stack
#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Path {
    slots: VecDeque<Slot>,
}

#[macro_export]
macro_rules! path {
        ($($x:expr),*) => {{
            let mut temp_vec = vec![$($x),*];
            temp_vec.reverse();
            Path::from(temp_vec)
        }};
    }

impl Path {
    /// Factory method to create a new Path
    ///
    /// # Returns
    ///
    /// A new Path
    pub fn new() -> Self {
        Self { slots: vec![].into() }
    }

    /// Push a Slot into the Path
    ///
    /// # Arguments
    ///
    /// * `slot` - The Slot to push
    pub fn push(&mut self, slot: Slot) {
        self.slots.push_front(slot);
    }

    /// Remove the first Slot from the Path
    ///
    /// # Panics
    ///
    /// If the Path is empty
    pub fn pull(&mut self) -> Option<Slot> {
        self.slots.pop_front()
    }

    /// Check if the Path is empty
    ///
    /// # Returns
    ///
    /// `true` if the Path is empty
    /// `false` otherwise
    pub fn is_root(&self) -> bool {
        self.slots.is_empty()
    }

    /// Check if the Path matches another Path
    ///
    /// # Arguments
    ///
    /// * `p` - The Path to check
    ///
    /// # Returns
    ///
    /// `true` if the Path matches
    /// `false` otherwise
    pub fn matches(&self, p: &Path) -> bool {
        self == p
    }

    /// Obtain the first Slot of the Path
    ///
    /// # Return
    ///
    /// The Slot at the head of the Path
    pub fn head(&self) -> Option<&Slot> {
        self.slots.front()
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<Slot>> for Path {
    fn from(slots: Vec<Slot>) -> Self {
        let mut reversed_slots = slots;
        reversed_slots.reverse();
        Self {
            slots: reversed_slots.into(),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "P://")?;
        for (i, slot) in self.slots.iter().enumerate() {
            if i != 0 {
                write!(f, "/")?;
            }
            write!(f, "{}", slot)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::Slot::*;

    #[test]
    fn test_is_root() {
        let empty_path = Path::from(Vec::new());
        let not_empty_path = Path::from(vec![Rep(0), Nbr(0), Nbr(1), Branch(0)]);
        assert!(empty_path.is_root());
        assert!(!not_empty_path.is_root())
    }

    #[test]
    fn test_not_empty_head() {
        let path = Path::from(vec![Rep(0), Nbr(0), Nbr(1), Branch(0)]);
        assert_eq!(path.head().unwrap(), &Branch(0))
    }

    #[test]
    fn test_empty_head() {
        let path = Path::new();
        assert!(path.head().is_none())
    }

    #[test]
    fn test_push() {
        let mut path = path![Nbr(1), Nbr(0), Rep(0)];
        path.push(Branch(0));
        assert_eq!(path, path![Branch(0), Nbr(1), Nbr(0), Rep(0)])
    }

    #[test]
    fn test_not_empty_pull() {
        let mut path = Path::from(vec![Rep(0), Nbr(0), Nbr(1), Branch(0)]);
        path.pull();
        assert_eq!(path.slots, vec![Nbr(1), Nbr(0), Rep(0)])
    }

    #[test]
    fn test_empty_pull() {
        let mut path = Path::new();
        assert!(path.pull().is_none());
    }

    #[test]
    fn test_to_str() {
        let path = path![Branch(0), Nbr(1), Nbr(0), Rep(0)];
        assert_eq!(path.to_string(), "P://Branch(0)/Nbr(1)/Nbr(0)/Rep(0)")
    }

    #[test]
    fn test_matches() {
        let path = Path::from(vec![Rep(0), Nbr(0), Nbr(1), Branch(0)]);
        assert!(path.matches(&Path::from(vec![Rep(0), Nbr(0), Nbr(1), Branch(0)])));
        assert!(!path.matches(&Path::from(vec![Nbr(0), Nbr(1), Branch(0)])))
    }

    #[test]
    fn test_serialize_and_deserialize() {
        let path = path!(Rep(0), FoldHood(0), Nbr(0), Nbr(1));
        let path_str = serde_json::to_string(&path).unwrap();
        let path_des = serde_json::from_str(&path_str).unwrap();
        assert_eq!(path, path_des);
    }
}
