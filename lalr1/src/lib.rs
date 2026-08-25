// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use std::{
    default::Default,
    fmt::{self, Debug, Display},
};

#[cfg(feature = "augmented")]
pub mod alalr1_parser;
#[cfg(not(feature = "augmented"))]
pub mod lalr1_parser;

pub mod attributes;
pub mod grammar;
pub mod parser;
pub mod production;
pub mod state;
pub mod symbol;

// Create a wrapper around BTreeSet so we can implement Display on it
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderedSet<T: Display + Ord + Clone>(pub std::collections::BTreeSet<T>);

impl<T: Display + Clone + Ord> OrderedSet<T> {
    pub fn new() -> Self {
        Self(std::collections::BTreeSet::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, item: &T) -> bool {
        self.0.contains(item)
    }

    pub fn insert(&mut self, item: T) -> bool {
        self.0.insert(item)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
}

impl<T: Ord + Display + Clone> FromIterator<T> for OrderedSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let btree_set = std::collections::BTreeSet::from_iter(iter);
        Self(btree_set)
    }
}
impl<'a, T: Ord + Display + Clone> IntoIterator for &'a OrderedSet<T> {
    type Item = &'a T;
    type IntoIter = <&'a std::collections::BTreeSet<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (self.0).iter()
    }
}

impl<T: Display + Ord + Clone> Display for OrderedSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();
        for (index, item) in self.0.iter().enumerate() {
            if index == 0 {
                string += &item.to_string();
            } else {
                string += ", ";
                string += &item.to_string()
            }
        }
        write!(f, "{}", string)
    }
}

#[cfg(test)]
mod tests;
