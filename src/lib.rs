extern crate regex;

use std::{
    cmp::Eq,
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

pub mod analyzer;
pub mod lexicon;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct MatchData<H: PartialEq + Hash + Debug + Copy> {
    handle: H,
    length: usize,
}

#[derive(Debug, Default)]
struct LiteralMatcherNode<H: PartialEq + Hash + Debug + Copy + Default> {
    handle: Option<H>,
    length: usize,
    tails: HashMap<u8, LiteralMatcherNode<H>>,
}

impl<H: PartialEq + Hash + Debug + Copy + Default> LiteralMatcherNode<H> {
    fn new(handle: H, string: &str, s_index: usize) -> LiteralMatcherNode<H> {
        debug_assert!(string.len() > 0);
        let mut t = HashMap::<u8, LiteralMatcherNode<H>>::new();
        if string.len() == s_index {
            LiteralMatcherNode {
                handle: Some(handle),
                length: string.len(),
                tails: t,
            }
        } else {
            let key = string.as_bytes()[s_index];
            t.insert(
                key,
                LiteralMatcherNode::<H>::new(handle, string, s_index + 1),
            );
            LiteralMatcherNode {
                handle: None,
                length: s_index,
                tails: t,
            }
        }
    }

    fn add(&mut self, handle: H, string: &str, s_index: usize) -> Result<(), String> {
        debug_assert!(string.len() > 0);
        if string.len() == s_index {
            if self.handle.is_some() {
                return Err(format!(
                    "Duplicate string: \"{}\": handles {:?} and {:?}",
                    string,
                    self.handle.unwrap(),
                    handle
                ));
            }
            self.handle = Some(handle);
            self.length = string.len();
        } else {
            let key = string.as_bytes()[s_index];
            // Couldn't do this with match because of ownership issues with "tails"
            if self.tails.contains_key(&key) {
                self.tails
                    .get_mut(&key)
                    .unwrap()
                    .add(handle, string, s_index + 1)?;
            } else {
                self.tails.insert(
                    key,
                    LiteralMatcherNode::<H>::new(handle, string, s_index + 1),
                );
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct LiteralMatcher<H: PartialEq + Hash + Debug + Copy + Default> {
    leximes: HashMap<u8, LiteralMatcherNode<H>>,
}

impl<H: Eq + Hash + Debug + Copy + Default> LiteralMatcher<H> {
    pub fn new(leximes: &[(H, &'static str)]) -> Result<LiteralMatcher<H>, String> {
        let mut handles = HashSet::<H>::new();
        let mut lexes = HashMap::<u8, LiteralMatcherNode<H>>::new();
        for &(handle, pattern) in leximes.iter() {
            // make sure that handles are unique and strings are not empty
            if pattern.len() == 0 {
                return Err(format!("Empty pattern for handle: {:?}", handle));
            }
            if handles.contains(&handle) {
                return Err(format!("Duplicate handle: {:?}", handle));
            } else {
                handles.insert(handle);
            }

            let key = pattern.as_bytes()[0];
            if lexes.contains_key(&key) {
                lexes.get_mut(&key).unwrap().add(handle, pattern, 1)?;
            } else {
                lexes.insert(key, LiteralMatcherNode::<H>::new(handle, pattern, 1));
            }
        }
        Ok(LiteralMatcher { leximes: lexes })
    }

    pub fn longest_match(&self, string: &str) -> Option<(H, usize)> {
        let mut rval: Option<(H, usize)> = None;
        let mut leximes = &self.leximes;
        for key in string.as_bytes().iter() {
            match leximes.get(&key) {
                None => break,
                Some(node) => {
                    if let Some(handle) = node.handle {
                        rval = Some((handle, node.length));
                    }
                    leximes = &node.tails;
                }
            }
        }
        rval
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_matcher() {
        let lm = crate::LiteralMatcher::new(&[
            (0, "test"),
            (1, "whatever"),
            (2, "anything"),
            (3, "anything at all"),
        ])
        .unwrap();
        assert!(lm.longest_match("something").is_none());
        assert_eq!(
            lm.longest_match("anything at all something"),
            Some((3, 15))
        );
        assert_eq!(
            lm.longest_match(&"anything at all whatever something"[16..]),
            Some((1, 8))
        );
    }
}
