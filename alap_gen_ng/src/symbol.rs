use std::cell::{Cell, RefCell};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    NonAssoc,
    Left,
    Right,
}

impl Default for Associativity {
    fn default() -> Self {
        Associativity::NonAssoc
    }
}

impl std::fmt::Display for Associativity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Associativity::*;
        match self {
            NonAssoc => write!(f, "NonAssoc"),
            Left => write!(f, "Left"),
            Right => write!(f, "Right"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct TokenData {
    name: String,
    text: String,
    defined_at: lexan::Location,
    used_at: RefCell<Vec<lexan::Location>>,
    associativity: Cell<Associativity>,
    precedence: Cell<u16>,
}

impl TokenData {
    pub fn new(name: &str, text: &str, defined_at: lexan::Location) -> Self {
        let mut token_data = TokenData::default();
        token_data.name = name.to_string();
        token_data.text = text.to_string();
        token_data.defined_at = defined_at;
        token_data
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn defined_at(&self) -> &lexan::Location {
        &self.defined_at
    }

    pub fn used_at(&self) -> Vec<lexan::Location> {
        self.used_at.borrow().clone()
    }

    pub fn associativity(&self) -> Associativity {
        self.associativity.get()
    }

    pub fn precedence(&self) -> u16 {
        self.precedence.get()
    }

    pub fn add_used_at(&self, used_at: lexan::Location) {
        self.used_at.borrow_mut().push(used_at)
    }

    pub fn set_associativity(self, associativity: Associativity) {
        self.associativity.set(associativity)
    }

    pub fn set_precedence(self, precedence: u16) {
        self.precedence.set(precedence)
    }
}
