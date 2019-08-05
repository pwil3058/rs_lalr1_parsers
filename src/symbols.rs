use std::{
    cell::RefCell,
    collections::HashMap,
    fmt,
    rc::Rc,
};

use lexan;

pub enum Error {
    AlreadyDefined(Rc<Symbol>),
}

impl fmt::Display for Error {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::AlreadyDefined(symbol) => {
                if let Some(location) = symbol.defined_at() {
                    write!(dest, "\"{}\" already defined at {}", symbol.name(), location)
                } else {
                    write!(dest, "\"{}\" already defined", symbol.name())
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Associativity {
    NonAssoc,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct AssociativePrecedence {
    associativity: Associativity,
    precedence: u32,
}

impl Default for AssociativePrecedence {
    fn default() -> Self {
        Self {
            associativity: Associativity::NonAssoc,
            precedence: 0,
        }
    }
}

impl AssociativePrecedence {
    pub fn explicitly_set(&self) -> bool {
        self.precedence != 0
    }
}

#[derive(Debug, Clone)]
struct SymbolMutableData {
    associative_precedence: AssociativePrecedence,
    defined_at: Option<lexan::Location>,
    used_at: Vec<lexan::Location>,
}

#[derive(Debug, Clone)]
pub enum SymbolType {
    Token,
    Tag,
    NonTerminal,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    ident: u32,
    name: String,
    symbol_type: SymbolType,
    mutable_data: RefCell<SymbolMutableData>,
}

impl Symbol {
    pub fn new_token_at(ident: u32, name: &str, location: &lexan::Location) -> Rc<Symbol> {
        let mutable_data = RefCell::new(SymbolMutableData {
            associative_precedence: AssociativePrecedence::default(),
            defined_at: Some(location.clone()),
            used_at: vec![],
        });
        Rc::new(Self {
            ident,
            name: name.to_string(),
            symbol_type: SymbolType::Token,
            mutable_data,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn defined_at(&self) -> Option<lexan::Location> {
        if let Some(location) = &self.mutable_data.borrow().defined_at {
            Some(location.clone())
        } else {
            None
        }
    }

    pub fn set_associative_precedence(&self, associativity: Associativity, precedence: u32) {
        self.mutable_data.borrow_mut().associative_precedence = AssociativePrecedence{associativity, precedence}
    }
}

#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    tokens: HashMap<String, Rc<Symbol>>,
    skip_rules: Vec<String>,
    next_precedence: u32,
    next_ident: u32,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            skip_rules: Vec::new(),
            next_precedence: u32::max_value(),
            next_ident: 0
        }
    }

    pub fn is_known_non_terminal(&self, _name: &str) -> bool {
        false
    }

    pub fn is_known_tag(&self, _name: &str) -> bool {
        false
    }

    pub fn is_known_token(&self, name: &str) -> bool {
        self.tokens.contains_key(name)
    }

    pub fn add_token(
        &mut self,
        name: &str,
        pattern: &str,
        location: &lexan::Location,
    ) -> Result<(), Error> {
        let token = Symbol::new_token_at(self.next_ident, name, location);
        self.next_ident += 1;
        if let Some(token) = self.tokens.insert(name.to_string(), token) {
            Err(Error::AlreadyDefined(Rc::clone(&token)))
        } else {
            Ok(())
        }
    }

    pub fn add_skip_rule(&mut self, rule: &str) {
        self.skip_rules.push(rule.to_string());
    }

    pub fn set_precedences(&mut self, associativity: Associativity, tags: &Vec<Rc<Symbol>>) {
        let precedence = self.next_precedence;
        for symbol in tags.iter() {
            symbol.set_associative_precedence(associativity, precedence);
        }
        self.next_precedence -= 1;
    }
}
