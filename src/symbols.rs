use std::{collections::HashMap, fmt};

use lexan;

pub enum Error {
    AlreadyDefined(String, lexan::Location),
}

impl fmt::Display for Error {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::AlreadyDefined(name, location) => {
                write!(dest, "\"{}\" already defined at {}", name, location)
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

#[derive(Debug, Clone, Copy)]
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
    associative_precedence: AssociativePrecedence,
    defined_at: lexan::Location,
    used_at: Vec<lexan::Location>,
}

#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    tokens: HashMap<String, (String, lexan::Location)>,
    skip_rules: Vec<String>,
    next_precedence: u32,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            skip_rules: Vec::new(),
            next_precedence: u32::max_value(),
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
        if let Some((_, location)) = self
            .tokens
            .insert(name.to_string(), (pattern.to_string(), location.clone()))
        {
            Err(Error::AlreadyDefined(name.to_string(), location.clone()))
        } else {
            Ok(())
        }
    }

    pub fn add_skip_rule(&mut self, rule: &str) {
        self.skip_rules.push(rule.to_string());
    }

    pub fn set_precedences(&mut self, associativity: Associativity, tags: &mut Vec<Symbol>) {
        let precedence = self.next_precedence;
        for symbol in tags.iter_mut() {
            symbol.associative_precedence = AssociativePrecedence{ associativity, precedence };
        }
        self.next_precedence -= 1;
    }
}
