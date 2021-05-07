// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::{btree_set, BTreeSet};
use std::fmt;
use std::iter::FromIterator;
use std::ops::{BitOr, BitOrAssign};

use crate::symbol::Associativity;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct TokenData {
    name: String,
    text: String,
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

#[derive(Debug, Clone)]
pub enum Token {
    Literal(Rc<TokenData>),
    Regex(Rc<TokenData>),
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
        }
    }

    pub fn defined_at(&self) -> &lexan::Location {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => &token_data.defined_at,
        }
    }

    pub fn add_used_at(&self, used_at: &lexan::Location) {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => {
                token_data.used_at.borrow_mut().push(used_at.clone())
            }
        }
    }

    pub fn set_associativity(&self, associativity: Associativity) {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => {
                token_data.associativity.set(associativity)
            }
        }
    }

    pub fn set_precedence(&self, precedence: u16) {
        debug_assert!(precedence > 0);
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => {
                token_data.precedence.set(precedence)
            }
        }
    }

    pub fn precedence_has_been_set(&self) -> bool {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => {
                token_data.precedence.get() > 0
            }
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for Token {}

impl PartialOrd for Token {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name().partial_cmp(other.name())
    }
}

impl Ord for Token {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
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

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
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
