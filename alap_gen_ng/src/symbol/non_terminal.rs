// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    fmt,
    rc::Rc,
};

use crate::alap_gen_ng::AANonTerminal;
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
pub enum NonTerminal {
    UserDefined(Rc<NonTerminalData>),
    Error(Rc<NonTerminalData>),
    Start(Rc<NonTerminalData>),
}

impl NonTerminal {
    pub fn new(name: &str) -> Self {
        let mut non_terminal_data = NonTerminalData::default();
        non_terminal_data.name = name.to_string();
        NonTerminal::UserDefined(Rc::new(non_terminal_data))
    }

    pub fn new_defined(name: &str, defined_at: &lexan::Location) -> Self {
        let mut non_terminal_data = NonTerminalData::default();
        non_terminal_data.name = name.to_string();
        non_terminal_data
            .defined_at
            .borrow_mut()
            .push(defined_at.clone());
        NonTerminal::UserDefined(Rc::new(non_terminal_data))
    }

    pub fn new_used(name: &str, used_at: &lexan::Location) -> Self {
        let mut non_terminal_data = NonTerminalData::default();
        non_terminal_data.name = name.to_string();
        non_terminal_data.used_at.borrow_mut().push(used_at.clone());
        NonTerminal::UserDefined(Rc::new(non_terminal_data))
    }

    pub fn new_error(name: &str) -> Self {
        let mut non_terminal_data = NonTerminalData::default();
        non_terminal_data.name = name.to_string();
        NonTerminal::Error(Rc::new(non_terminal_data))
    }

    pub fn name(&self) -> &str {
        match self {
            NonTerminal::UserDefined(non_terminal) => &non_terminal.name,
            _ => panic!("should not be asking the name of special symbols"),
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            NonTerminal::Error(_) => true,
            _ => false,
        }
    }

    pub fn is_start(&self) -> bool {
        match self {
            NonTerminal::Start(_) => true,
            _ => false,
        }
    }

    pub fn is_unused(&self) -> bool {
        match self {
            NonTerminal::UserDefined(non_terminal_data)
            | NonTerminal::Error(non_terminal_data)
            | NonTerminal::Start(non_terminal_data) => {
                non_terminal_data.used_at.borrow().is_empty()
            }
        }
    }

    pub fn is_undefined(&self) -> bool {
        match self {
            NonTerminal::UserDefined(non_terminal) => non_terminal.defined_at.borrow().is_empty(),
            NonTerminal::Error(_) | NonTerminal::Start(_) => false,
        }
    }

    pub fn first_definition(&self) -> Option<lexan::Location> {
        match self {
            NonTerminal::UserDefined(non_terminal) => {
                Some(non_terminal.defined_at.borrow().first()?.clone())
            }
            _ => None,
        }
    }

    pub fn used_at(&self) -> Vec<lexan::Location> {
        match self {
            NonTerminal::UserDefined(non_terminal_data)
            | NonTerminal::Error(non_terminal_data)
            | NonTerminal::Start(non_terminal_data) => {
                non_terminal_data.used_at.borrow().iter().cloned().collect()
            }
        }
    }

    pub fn add_defined_at(&self, defined_at: &lexan::Location) {
        match self {
            NonTerminal::UserDefined(non_terminal_data) => non_terminal_data
                .defined_at
                .borrow_mut()
                .push(defined_at.clone()),
            _ => panic!("should not be adding definitions to special symbols"),
        }
    }

    pub fn add_used_at(&self, used_at: &lexan::Location) {
        match self {
            NonTerminal::UserDefined(non_terminal_data)
            | NonTerminal::Error(non_terminal_data)
            | NonTerminal::Start(non_terminal_data) => {
                non_terminal_data.used_at.borrow_mut().push(used_at.clone())
            }
        }
    }

    pub fn firsts_data(&self) -> FirstsData {
        let msg = format!("{} :should be set", self.name());
        match self {
            NonTerminal::UserDefined(non_terminal_data)
            | NonTerminal::Error(non_terminal_data)
            | NonTerminal::Start(non_terminal_data) => {
                non_terminal_data.firsts_data.borrow().clone().expect(&msg)
            }
        }
    }
}
