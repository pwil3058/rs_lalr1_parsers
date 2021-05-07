// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    collections::{btree_set, BTreeMap, BTreeSet},
    fmt,
    iter::FromIterator,
    ops::{BitOr, BitOrAssign},
    rc::Rc,
};

use crate::symbol::non_terminal::NonTerminal;
use crate::symbol::terminal::Token;

mod non_terminal;
mod terminal;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    Terminal(Token),
    NonTerminal(NonTerminal),
}

#[derive(Debug)]
pub enum Error {
    DuplicateToken(Token),
    DuplicateTokenDefinition(Token),
    ConflictsWithToken(Token),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DuplicateToken(token) => {
                write!(
                    f,
                    "Token \"{}\" already defined at {}",
                    token.name(),
                    token.defined_at(),
                )
            }
            Error::DuplicateTokenDefinition(token) => {
                write!(
                    f,
                    "Token \"{}\" defined at {} has same definition",
                    token.name(),
                    token.defined_at(),
                )
            }
            Error::ConflictsWithToken(token) => {
                write!(
                    f,
                    "NonTerminal \"{}\" conflicts with token defined at {}.",
                    token.name(),
                    token.defined_at(),
                )
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    tokens: BTreeMap<String, Token>,
    literal_tokens: BTreeMap<String, Token>,
    regex_tokens: BTreeMap<String, Token>,
    non_terminals: BTreeMap<String, NonTerminal>,
}

impl SymbolTable {
    pub fn new_literal_token(
        &mut self,
        name: &str,
        text: &str,
        defined_at: &lexan::Location,
    ) -> Result<Token, Error> {
        let token = Token::new_literal_token(name, text, defined_at);
        if let Some(other) = self.tokens.insert(name.to_string(), token.clone()) {
            Err(Error::DuplicateToken(other))
        } else if let Some(other) = self.literal_tokens.insert(text.to_string(), token.clone()) {
            Err(Error::DuplicateTokenDefinition(other))
        } else {
            Ok(token)
        }
    }

    pub fn new_regex_token(
        &mut self,
        name: &str,
        text: &str,
        defined_at: &lexan::Location,
    ) -> Result<Token, Error> {
        let token = Token::new_regex_token(name, text, defined_at);
        if let Some(other) = self.tokens.insert(name.to_string(), token.clone()) {
            Err(Error::DuplicateToken(other))
        } else if let Some(other) = self.regex_tokens.insert(text.to_string(), token.clone()) {
            Err(Error::DuplicateTokenDefinition(other))
        } else {
            Ok(token)
        }
    }

    pub fn non_terminal_defined_at(
        &mut self,
        name: &str,
        defined_at: &lexan::Location,
    ) -> Result<NonTerminal, Error> {
        if let Some(non_terminal) = self.non_terminals.get(name) {
            non_terminal.add_defined_at(defined_at);
            Ok(non_terminal.clone())
        } else if let Some(token) = self.tokens.get(name) {
            Err(Error::ConflictsWithToken(token.clone()))
        } else {
            let non_terminal = NonTerminal::new_defined(name, defined_at);
            self.non_terminals
                .insert(name.to_string(), non_terminal.clone());
            Ok(non_terminal)
        }
    }

    pub fn symbol_used_at(&mut self, name: &str, used_at: &lexan::Location) -> Symbol {
        if let Some(token) = self.tokens.get(name) {
            token.add_used_at(used_at);
            Symbol::Terminal(token.clone())
        } else if let Some(non_terminal) = self.non_terminals.get(name) {
            non_terminal.add_used_at(used_at);
            Symbol::NonTerminal(non_terminal.clone())
        } else {
            let non_terminal = NonTerminal::new_used(name, used_at);
            self.non_terminals
                .insert(name.to_string(), non_terminal.clone());
            Symbol::NonTerminal(non_terminal)
        }
    }
}
