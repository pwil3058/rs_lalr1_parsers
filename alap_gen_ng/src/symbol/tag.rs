// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::symbol::terminal::Token;
use crate::symbol::Associativity;

#[derive(Debug, Default)]
pub struct TagData {
    name: String,
    defined_at: lexan::Location,
    used_at: RefCell<Vec<lexan::Location>>,
    associativity: Cell<Associativity>,
    precedence: Cell<u16>,
}

impl TagData {
    pub fn new(name: &str, defined_at: &lexan::Location) -> Self {
        let mut tag_data = TagData::default();
        tag_data.name = name.to_string();
        tag_data.defined_at = defined_at.clone();
        tag_data
    }
}

#[derive(Debug, Clone)]
pub struct Tag(Rc<TagData>);

impl Tag {
    pub fn new(name: &str, defined_at: &lexan::Location) -> Self {
        Self(Rc::new(TagData::new(name, defined_at)))
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn defined_at(&self) -> &lexan::Location {
        &self.0.defined_at
    }

    pub fn associativity(&self) -> Associativity {
        self.0.associativity.get()
    }

    pub fn precedence(&self) -> u16 {
        self.0.precedence.get()
    }

    pub fn add_used_at(&self, used_at: &lexan::Location) {
        self.0.used_at.borrow_mut().push(used_at.clone())
    }

    pub fn set_associativity(&self, associativity: Associativity) {
        self.0.associativity.set(associativity)
    }

    pub fn set_precedence(&self, precedence: u16) {
        self.0.precedence.set(precedence)
    }
}

#[derive(Debug, Clone)]
pub enum TagOrToken {
    Tag(Tag),
    Token(Token),
    Invalid,
}

impl From<&Tag> for TagOrToken {
    fn from(tag: &Tag) -> Self {
        TagOrToken::Tag(tag.clone())
    }
}

impl From<&Token> for TagOrToken {
    fn from(token: &Token) -> Self {
        TagOrToken::Token(token.clone())
    }
}
