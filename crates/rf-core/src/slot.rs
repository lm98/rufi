use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// # Slot is an enum that represents the different constructs of the language.
///
/// * `Nbr(index)` - The value of an expression across neighbours.
/// * `Rep(index)` - It iteratively updates the value of the input expression at each device using the last computed value.
/// * `Branch(index)` - Partition the domain into two subspaces that do not interact with each other.
/// * `Exchange(index)` - The exchange construct handles neighbour-to-neighbour propagation of partial accumulates.
#[derive(PartialEq, Debug, Clone, Eq, Hash, Serialize, Deserialize)]
pub enum Slot {
    Nbr(i32),
    Rep(i32),
    FoldHood(i32),
    Branch(i32),
    Exchange(i32),
}
impl Slot {
    /// String representation of the Slot.
    ///
    /// # Returns
    ///
    /// A string representing the Slot.
    pub fn to_str(&self) -> String {
        match self {
            Slot::Nbr(index) => "Nbr(".to_owned() + &index.to_string() + ")",
            Slot::Rep(index) => "Rep(".to_owned() + &index.to_string() + ")",
            Slot::FoldHood(index) => "FoldHood(".to_owned() + &index.to_string() + ")",
            Slot::Branch(index) => "Branch(".to_owned() + &index.to_string() + ")",
            Slot::Exchange(index) => "Exchange(".to_owned() + &index.to_string() + ")",
        }
    }
}

impl Display for Slot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.to_str().fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slot_creation() {
        let nbr = Slot::Nbr(0);
        let rep = Slot::Rep(0);
        let foldhood = Slot::FoldHood(0);
        let branch = Slot::Branch(0);
        let exchange = Slot::Exchange(0);
        assert_eq!(nbr, Slot::Nbr(0));
        assert_eq!(rep, Slot::Rep(0));
        assert_eq!(foldhood, Slot::FoldHood(0));
        assert_eq!(branch, Slot::Branch(0));
        assert_eq!(exchange, Slot::Exchange(0));
    }

    #[test]
    fn test_slot_to_string() {
        let nbr = Slot::Nbr(0);
        let rep = Slot::Rep(0);
        let branch = Slot::Branch(0);
        let exchange = Slot::Exchange(0);
        assert_eq!(nbr.to_str(), "Nbr(0)");
        assert_eq!(rep.to_str(), "Rep(0)");
        assert_eq!(branch.to_str(), "Branch(0)");
        assert_eq!(exchange.to_str(), "Exchange(0)");
    }

    #[test]
    fn test_serialize_deserialize() {
        let nbr = Slot::Nbr(0);
        let rep = Slot::Rep(0);
        let foldhood = Slot::FoldHood(0);
        let branch = Slot::Branch(0);
        let exchange = Slot::Exchange(0);
        let nbr_str = serde_json::to_string(&nbr).unwrap();
        let rep_str = serde_json::to_string(&rep).unwrap();
        let foldhood_str = serde_json::to_string(&foldhood).unwrap();
        let branch_str = serde_json::to_string(&branch).unwrap();
        let exchange_str = serde_json::to_string(&exchange).unwrap();
        let nbr_des: Slot = serde_json::from_str(&nbr_str).unwrap();
        let rep_des: Slot = serde_json::from_str(&rep_str).unwrap();
        let foldhood_des: Slot = serde_json::from_str(&foldhood_str).unwrap();
        let branch_des: Slot = serde_json::from_str(&branch_str).unwrap();
        let exchange_des: Slot = serde_json::from_str(&exchange_str).unwrap();
        assert_eq!(nbr, nbr_des);
        assert_eq!(rep, rep_des);
        assert_eq!(foldhood, foldhood_des);
        assert_eq!(branch, branch_des);
        assert_eq!(exchange, exchange_des);
    }
}
