use crate::slot::Slot;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// A Path is a collection of Slots that behave like an immutable stack
#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub struct Path {
    slots: Vec<Slot>,
}

#[macro_export]
macro_rules! path {
        ($($x:expr),*) => {{
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
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
        Self { slots: vec![] }
    }

    /// Push a Slot into the Path
    ///
    /// # Arguments
    ///
    /// * `slot` - The Slot to push
    ///
    /// # Returns
    ///
    /// A new Path with the Slot pushed
    pub fn push(&self, slot: Slot) -> Self {
        Self {
            slots: [&[slot], &self.slots[..]].concat(),
        }
    }

    /// Remove the first Slot from the Path
    ///
    /// # Returns
    ///
    /// A new Path without the first Slot
    pub fn pull(&self) -> Self {
        let mut new_slots = self.slots.clone();
        new_slots.drain(..1);
        Self { slots: new_slots }
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

    /// Return a String representation of the Path
    ///
    /// # Returns
    ///
    /// A String representation of the Path
    pub fn to_str(&self) -> String {
        let slots = &self.slots;
        let path = String::from("P://");
        path + &slots
            .into_iter()
            .map(|slot| slot.to_str())
            .collect::<Vec<String>>()
            .join("/")
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
    pub fn head(&self) -> &Slot {
        self.slots.first().unwrap()
    }
}

impl From<Vec<Slot>> for Path {
    fn from(slots: Vec<Slot>) -> Self {
        let mut reversed_slots = slots;
        reversed_slots.reverse();
        Self {
            slots: reversed_slots,
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.to_str().fmt(f)
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
        assert_eq!(path.head(), &Branch(0))
    }

    #[test]
    #[should_panic]
    fn test_empty_head() {
        let path = Path::new();
        assert_eq!(path.head(), &Rep(0))
    }

    #[test]
    fn test_push() {
        let path = Path::from(vec![Rep(0), Nbr(0), Nbr(1)]).push(Branch(0));
        assert_eq!(path.slots, vec![Branch(0), Nbr(1), Nbr(0), Rep(0)])
    }

    #[test]
    fn test_not_empty_pull() {
        let path = Path::from(vec![Rep(0), Nbr(0), Nbr(1), Branch(0)]);
        assert_eq!(path.pull().slots, vec![Nbr(1), Nbr(0), Rep(0)])
    }

    #[test]
    #[should_panic]
    fn test_empty_pull() {
        let path = Path::new();
        assert_eq!(path.pull(), Path::new())
    }

    #[test]
    fn test_to_str() {
        let path = Path::from(vec![Rep(0), Nbr(0), Nbr(1), Branch(0)]);
        assert_eq!(path.to_str(), "P://Branch(0)/Nbr(1)/Nbr(0)/Rep(0)")
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
