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

#[derive(Debug)]
struct LiteralMatcherNode<H: PartialEq + Hash + Debug + Copy> {
    option: Option<MatchData<H>>,
    tails: HashMap<u8, LiteralMatcherNode<H>>,
}

impl<H: PartialEq + Hash + Debug + Copy> LiteralMatcherNode<H> {
    fn new(handle: H, string: &str, s_index: usize) -> LiteralMatcherNode<H> {
        debug_assert!(string.len() > 0);
        let mut t = HashMap::<u8, LiteralMatcherNode<H>>::new();
        if string.len() == s_index {
            let md = MatchData {
                handle: handle,
                length: string.len(),
            };
            LiteralMatcherNode {
                option: Some(md),
                tails: t,
            }
        } else {
            let key = string.as_bytes()[s_index];
            t.insert(
                key,
                LiteralMatcherNode::<H>::new(handle, string, s_index + 1),
            );
            LiteralMatcherNode {
                option: None,
                tails: t,
            }
        }
    }

    fn add(&mut self, handle: H, string: &str, s_index: usize) -> Result<(), String> {
        debug_assert!(string.len() > 0);
        if string.len() == s_index {
            if self.option.is_some() {
                return Err(format!(
                    "Duplicate string: \"{}\": handles {:?} and {:?}",
                    string,
                    self.option.unwrap().handle,
                    handle
                ));
            }
            self.option = Some(MatchData {
                handle: handle,
                length: string.len(),
            })
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

#[derive(Debug)]
pub struct LiteralMatcher<H: PartialEq + Hash + Debug + Copy> {
    leximes: HashMap<u8, LiteralMatcherNode<H>>,
}

impl<H: Eq + Hash + Debug + Copy> LiteralMatcher<H> {
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

    pub fn get_longest_match(&self, string: &str) -> Option<MatchData<H>> {
        let mut rval: Option<MatchData<H>> = None;
        let mut leximes = &self.leximes;
        for key in string.as_bytes().iter() {
            match leximes.get(&key) {
                None => break,
                Some(node) => {
                    if node.option.is_some() {
                        rval = node.option;
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
        assert!(lm.get_longest_match("something").is_none());
        assert_eq!(
            lm.get_longest_match("anything at all something"),
            Some(MatchData {
                handle: 3,
                length: 15
            })
        );
        assert_eq!(
            lm.get_longest_match(&"anything at all whatever something"[16..]),
            Some(MatchData {
                handle: 1,
                length: 8
            })
        );
    }
}
