// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    collections::{btree_set, BTreeSet},
    fmt,
    iter::FromIterator,
    ops::{BitOr, BitOrAssign},
    rc::Rc,
};

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

    pub fn set_associativity(&self, associativity: Associativity) {
        self.associativity.set(associativity)
    }

    pub fn set_precedence(&self, precedence: u16) {
        self.precedence.set(precedence)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Literal(Rc<TokenData>),
    Regex(Rc<TokenData>),
}

impl Token {
    pub fn new_literal_token(name: &str, text: &str, defined_at: lexan::Location) -> Self {
        let token_data = TokenData::new(name, text, defined_at);
        Token::Literal(Rc::new(token_data))
    }

    pub fn new_regex_token(name: &str, text: &str, defined_at: lexan::Location) -> Self {
        let token_data = TokenData::new(name, text, defined_at);
        Token::Regex(Rc::new(token_data))
    }

    pub fn name(&self) -> &str {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => token_data.name(),
        }
    }

    pub fn add_used_at(&self, used_at: lexan::Location) {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => {
                token_data.add_used_at(used_at)
            }
        }
    }

    pub fn set_associativity(&self, associativity: Associativity) {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => {
                token_data.set_associativity(associativity)
            }
        }
    }

    pub fn set_precedence(&self, precedence: u16) {
        match self {
            Token::Literal(token_data) | Token::Regex(token_data) => {
                token_data.set_precedence(precedence)
            }
        }
    }
}

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
