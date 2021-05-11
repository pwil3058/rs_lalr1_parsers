// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::{btree_set, BTreeSet};
use std::fmt;
use std::iter::FromIterator;
use std::ops::{BitOr, BitOrAssign};

use crate::symbol::{Associativity, Symbol};
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct TokenData {
    pub name: String,
    pub text: String,
    defined_at: lexan::Location,
    used_at: RefCell<Vec<lexan::Location>>,
    associativity: Cell<Associativity>,
    precedence: Cell<u16>,
}

impl TokenData {
    pub fn new(name: &str, text: &str, defined_at: &lexan::Location) -> Self {
        let mut token_data = TokenData::default();
        token_data.name = name.to_string();
        token_data.text = text.to_string();
        token_data.defined_at = defined_at.clone();
        token_data
    }
}

impl PartialEq for TokenData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for TokenData {}

impl PartialOrd for TokenData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for TokenData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Token {
    Literal(Rc<TokenData>),
    Regex(Rc<TokenData>),
    EndToken,
}

impl Token {
    pub fn new_literal_token(name: &str, text: &str, defined_at: &lexan::Location) -> Self {
        Token::Literal(Rc::new(TokenData::new(name, text, defined_at)))
    }

    pub fn new_regex_token(name: &str, text: &str, defined_at: &lexan::Location) -> Self {
        Token::Regex(Rc::new(TokenData::new(name, text, defined_at)))
    }

    pub fn name(&self) -> &str {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => &token_data.name,
            Token::EndToken => panic!("should not be asking end token's name"),
        }
    }

    pub fn text(&self) -> &str {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => &token_data.text,
            Token::EndToken => panic!("should not be asking end token's name"),
        }
    }

    pub fn defined_at(&self) -> &lexan::Location {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => &token_data.defined_at,
            Token::EndToken => panic!("should not be asking end token's definition location"),
        }
    }

    pub fn associativity(&self) -> Associativity {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => token_data.associativity.get(),
            Token::EndToken => Associativity::default(),
        }
    }

    pub fn precedence(&self) -> u16 {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => token_data.precedence.get(),
            Token::EndToken => 0,
        }
    }

    pub fn is_unused(&self) -> bool {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => {
                token_data.used_at.borrow().is_empty()
            }
            Token::EndToken => false,
        }
    }

    pub fn associativity_and_precedence(&self) -> (Associativity, u16) {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => {
                (token_data.associativity.get(), token_data.precedence.get())
            }
            Token::EndToken => (Associativity::default(), 0),
        }
    }

    pub fn add_used_at(&self, used_at: &lexan::Location) {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => {
                token_data.used_at.borrow_mut().push(used_at.clone())
            }
            Token::EndToken => panic!("should not be trying to modify end token's usage locations"),
        }
    }

    pub fn set_associativity(&self, associativity: Associativity) {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => {
                token_data.associativity.set(associativity)
            }
            Token::EndToken => panic!("should not be trying to set end token's associativity"),
        }
    }

    pub fn set_precedence(&self, precedence: u16) {
        debug_assert!(precedence > 0);
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => {
                token_data.precedence.set(precedence)
            }
            Token::EndToken => panic!("should not be trying to set end token's precedence"),
        }
    }

    pub fn precedence_has_been_set(&self) -> bool {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => {
                token_data.precedence.get() > 0
            }
            Token::EndToken => false,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TokenSet(BTreeSet<Token>);

impl TokenSet {
    pub fn new() -> Self {
        TokenSet::default()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first_all_caps(symbol_string: &[Symbol], token: &Token) -> Self {
        let mut token_set = TokenSet::new();
        for symbol in symbol_string.iter() {
            match symbol {
                Symbol::NonTerminal(non_terminal) => {
                    let firsts_data = non_terminal.firsts_data();
                    token_set |= &firsts_data.token_set;
                    if !firsts_data.transparent {
                        return token_set;
                    }
                }
                Symbol::Terminal(token) => {
                    token_set.insert(token);
                    return token_set;
                }
            }
        }
        token_set.insert(token);
        token_set
    }

    pub fn contains(&self, token: &Token) -> bool {
        self.0.contains(token)
    }

    pub fn insert(&mut self, token: &Token) -> bool {
        self.0.insert(token.clone())
    }

    pub fn remove(&mut self, token: &Token) -> bool {
        self.0.remove(token)
    }

    pub fn difference<'a>(&'a self, other: &'a Self) -> btree_set::Difference<'a, Token> {
        self.0.difference(&other.0)
    }

    pub fn intersection<'a>(&'a self, other: &'a Self) -> btree_set::Intersection<'a, Token> {
        self.0.intersection(&other.0)
    }

    pub fn union<'a>(&'a self, other: &'a Self) -> btree_set::Union<'a, Token> {
        self.0.union(&other.0)
    }

    pub fn iter(&self) -> btree_set::Iter<Token> {
        self.0.iter()
    }

    pub fn formated_as_macro_call(&self) -> String {
        let mut string = "btree_set![".to_string();
        for (index, token) in self.0.iter().enumerate() {
            if index == 0 {
                string += &format!("{}", token.name());
            } else {
                string += &format!(", {}", token.name());
            }
        }
        string += "]";
        string
    }

    pub fn formated_as_or_list(&self) -> String {
        let mut string = "".to_string();
        for (index, token) in self.0.iter().enumerate() {
            if index == 0 {
                string += &format!("{}", token.name());
            } else {
                string += &format!(" | {}", token.name());
            }
        }
        string
    }
}

impl BitOrAssign<&Self> for TokenSet {
    fn bitor_assign(&mut self, rhs: &Self) {
        self.0 = self.0.bitor(&rhs.0)
    }
}

impl FromIterator<Token> for TokenSet {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Token>,
    {
        Self(BTreeSet::<Token>::from_iter(iter))
    }
}

impl fmt::Display for TokenSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut set_string = "TokenSet{".to_string();
        for (index, item) in self.iter().enumerate() {
            if index == 0 {
                set_string += &format!("{}", item.name());
            } else {
                set_string += &format!(", {}", item.name());
            }
        }
        set_string += "}";
        write!(f, "{}", set_string)
    }
}
