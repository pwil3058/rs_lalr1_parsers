use std::collections::HashMap;
use std::collections::HashSet;
use std::cmp::Eq;
use std::hash::Hash;
use std::fmt::Debug;

#[derive(Clone)]
#[derive(Copy)]
pub struct MatchData<H: Eq + Hash + Debug + Copy> {
    handle: H,
    length: usize
}

struct LiteralMatcherNode<H: Eq + Hash + Debug + Copy> {
    option: Option<MatchData<H>>,
    tails: HashMap<u8, LiteralMatcherNode<H>>
}

impl <H: Eq + Hash + Debug + Copy> LiteralMatcherNode<H> {
    pub fn new(handle: H, string: &str, s_index: usize) -> LiteralMatcherNode<H> {
        assert!(string.len() > 0);
        let mut t = HashMap::<u8, LiteralMatcherNode<H>>::new();
        if string.len() == s_index {
            let md = MatchData{handle: handle, length: string.len()};
            LiteralMatcherNode{option: Some(md), tails: t}
        } else {
            let key = string.as_bytes()[s_index];
            t.insert(key, LiteralMatcherNode::<H>::new(handle, string, s_index + 1));
            LiteralMatcherNode{option: None, tails: t}
        }
    }

    pub fn add(&mut self, handle: H, string: &str, s_index: usize) {
        assert!(string.len() > 0);
        if string.len() == s_index {
            match self.option {
                Some(ref x) => panic!("Duplicate string: {}: {:?} and {:?}", string, x.handle, handle),
                None => (),
            }
            self.option = Some(MatchData{handle: handle, length: string.len()})
        } else {
            let key = string.as_bytes()[s_index];
            if self.tails.contains_key(&key) {
                match self.tails.get_mut(&key) {
                    Some(node) => node.add(handle, string, s_index + 1),
                    None => panic!("Should NOT happen!!"),
                }
            } else {
                self.tails.insert(key, LiteralMatcherNode::<H>::new(handle, string, s_index + 1));
            }
        }
    }
}

pub struct LiteralMatcher<H: Eq + Hash + Debug + Copy> {
    leximes: HashMap<u8, LiteralMatcherNode<H>>,
}

impl <H: Eq + Hash + Debug + Copy> LiteralMatcher<H> {
    pub fn new(leximes: &[(H, &'static str)]) -> LiteralMatcher<H> {
        let mut handles = HashSet::<H>::new();
        let mut lexes = HashMap::<u8, LiteralMatcherNode<H>>::new();
        for &(handle, pattern) in leximes.iter() {
            // make sure that handles are unique and strings are not empty
            if pattern.len() == 0 {
                panic!("Empty pattern for handle: {:?}", handle)
            }
            if handles.contains(&handle) {
                panic!("Duplicate handle: {:?}", handle)
            } else {
                handles.insert(handle);
            }
            println!("lexime {:?}:{}", handle, pattern);

            let key = pattern.as_bytes()[0];
            if lexes.contains_key(&key) {
                match lexes.get_mut(&key) {
                    Some(node) => node.add(handle, pattern, 1),
                    None => panic!("Should NOT happen!!"),
                }
            } else {
                lexes.insert(key, LiteralMatcherNode::<H>::new(handle, pattern, 1));
            }
        }
        LiteralMatcher{leximes: lexes}
    }

    pub fn get_longest_match(&self, string: &str) -> Option<MatchData<H>> {
        let mut rval: Option<MatchData<H>> = None;
        let mut leximes = &self.leximes;
        for key in string.as_bytes().iter() {
            match leximes.get(&key) {
                None => break,
                Some(node) => {
                    match node.option {
                        Some(md) => rval = Some(md),
                        None => (),
                    }
                    leximes = &node.tails;
                }
            }
        };
        rval
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let lm = ::LiteralMatcher::new(&[(0, "test"), (1, "whatever"), (2, "anything"), (3, "anything at all")]);
        let string = "something";
        match lm.get_longest_match(string) {
            Some(find) => println!("{:?} found", find.handle),
            None => println!("{:?} NOT found", string)
        }
        let string = "anything at all something";
        match lm.get_longest_match(string) {
            Some(find) => println!("{:?} found", find.handle),
            None => println!("{:?} NOT found", string)
        }
    }
}
