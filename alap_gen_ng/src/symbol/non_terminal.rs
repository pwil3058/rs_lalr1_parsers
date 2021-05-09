// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    fmt,
    rc::Rc,
};

use crate::symbol::{terminal::TokenSet, Associativity};

#[derive(Debug, Clone, Default)]
pub struct FirstsData {
    pub token_set: TokenSet,
    pub transparent: bool,
}

impl fmt::Display for FirstsData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:({})", self.token_set, self.transparent)
    }
}

#[derive(Debug, Default)]
pub struct NonTerminalData {
    name: String,
    defined_at: RefCell<Vec<lexan::Location>>,
    used_at: RefCell<Vec<lexan::Location>>,
    firsts_data: RefCell<Option<FirstsData>>,
    associativity: Cell<Associativity>,
    precedence: Cell<u16>,
}

impl PartialEq for NonTerminalData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for NonTerminalData {}

impl PartialOrd for NonTerminalData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for NonTerminalData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NonTerminal(Rc<NonTerminalData>);

impl NonTerminal {
    pub fn new(name: &str) -> Self {
        let mut token_data = NonTerminalData::default();
        token_data.name = name.to_string();
        Self(Rc::new(token_data))
    }

    pub fn new_defined(name: &str, defined_at: &lexan::Location) -> Self {
        let mut token_data = NonTerminalData::default();
        token_data.name = name.to_string();
        token_data.defined_at.borrow_mut().push(defined_at.clone());
        Self(Rc::new(token_data))
    }

    pub fn new_used(name: &str, used_at: &lexan::Location) -> Self {
        let mut token_data = NonTerminalData::default();
        token_data.name = name.to_string();
        token_data.used_at.borrow_mut().push(used_at.clone());
        Self(Rc::new(token_data))
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn first_definition(&self) -> Option<lexan::Location> {
        Some(self.0.defined_at.borrow().first()?.clone())
    }

    pub fn add_defined_at(&self, defined_at: &lexan::Location) {
        self.0.defined_at.borrow_mut().push(defined_at.clone())
    }

    pub fn add_used_at(&self, defined_at: &lexan::Location) {
        self.0.used_at.borrow_mut().push(defined_at.clone())
    }
}
