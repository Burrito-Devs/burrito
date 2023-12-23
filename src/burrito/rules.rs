use serde_derive::{Deserialize, Serialize};

pub trait Comparable: Eq + Ord + PartialEq + PartialOrd {}
impl<T: Eq + Ord + PartialEq + PartialOrd> Comparable for T {} 

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EventRule {
    rule: ComparisonRule,
    compare: Box<dyn Comparable>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ComparisonRule {
    LessThan,
    LessThanOrEqualTo,
    NotEqualTo,
    EqualTo,
    GreaterThanOrEqualTo,
    GreaterThan,
}
