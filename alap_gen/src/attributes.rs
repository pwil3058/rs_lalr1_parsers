use std::{collections::BTreeSet, rc::Rc};

use lexan;

#[cfg(not(feature = "bootstrap"))]
use crate::alapgen::AATerminal;
#[cfg(feature = "bootstrap")]
use crate::bootstrap::AATerminal;
use crate::state::ProductionTail;
use crate::symbols::*;

#[derive(Debug, Clone)]
pub enum AttributeData {
    Token(lexan::Token<AATerminal>),
    SyntaxError(lexan::Token<AATerminal>, BTreeSet<AATerminal>),
    LexicalError(lexan::Error<AATerminal>, BTreeSet<AATerminal>),
    SymbolList(Vec<Rc<Symbol>>),
    Symbol(Rc<Symbol>),
    LeftHandSide(Rc<Symbol>),
    ProductionTail(ProductionTail),
    ProductionTailList(Vec<ProductionTail>),
    Action(String),
    Predicate(String),
    AssociativePrecedence(AssociativePrecedence),
    Default,
}

impl Default for AttributeData {
    fn default() -> Self {
        AttributeData::Default
    }
}

impl AttributeData {
    pub fn matched_text<'a>(&'a self) -> &'a String {
        match self {
            AttributeData::Token(token) => token.lexeme(),
            AttributeData::SyntaxError(token, _) => token.lexeme(),
            AttributeData::LexicalError(error, _) => match error {
                lexan::Error::UnexpectedText(text, _) => text,
                lexan::Error::AmbiguousMatches(_, text, _) => text,
                lexan::Error::AdvancedWhenEmpty(_) => panic!("Wrong attribute variant."),
            },
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn location<'a>(&'a self) -> &'a lexan::Location {
        match self {
            AttributeData::Token(token) => token.location(),
            AttributeData::SyntaxError(token, _) => token.location(),
            AttributeData::LexicalError(error, _) => match error {
                lexan::Error::UnexpectedText(_, location) => location,
                lexan::Error::AmbiguousMatches(_, _, location) => location,
                lexan::Error::AdvancedWhenEmpty(location) => location,
            },
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn text_and_location<'a>(&'a self) -> (&'a String, &'a lexan::Location) {
        match self {
            AttributeData::Token(token) => (token.lexeme(), token.location()),
            AttributeData::SyntaxError(token, _) => (token.lexeme(), token.location()),
            AttributeData::LexicalError(error, _) => match error {
                lexan::Error::UnexpectedText(text, location) => (text, location),
                lexan::Error::AmbiguousMatches(_, text, location) => (text, location),
                lexan::Error::AdvancedWhenEmpty(_) => panic!("Wrong attribute variant."),
            },
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn symbol<'a>(&'a self) -> &'a Rc<Symbol> {
        match self {
            AttributeData::Symbol(symbol) => symbol,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn symbol_list<'a>(&'a self) -> &'a Vec<Rc<Symbol>> {
        match self {
            AttributeData::SymbolList(list) => list,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn symbol_list_mut<'a>(&'a mut self) -> &'a mut Vec<Rc<Symbol>> {
        match self {
            AttributeData::SymbolList(list) => list,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn left_hand_side<'a>(&'a self) -> &'a Rc<Symbol> {
        match self {
            AttributeData::LeftHandSide(lhs) => lhs,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn production_tail<'a>(&'a self) -> &'a ProductionTail {
        match self {
            AttributeData::ProductionTail(tail) => tail,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn production_tail_list<'a>(&'a self) -> &'a Vec<ProductionTail> {
        match self {
            AttributeData::ProductionTailList(list) => list,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn action<'a>(&'a self) -> &'a str {
        match self {
            AttributeData::Action(action) => action,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn predicate<'a>(&'a self) -> &'a str {
        match self {
            AttributeData::Predicate(predicate) => predicate,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn associative_precedence<'a>(&'a self) -> &'a AssociativePrecedence {
        match self {
            AttributeData::AssociativePrecedence(associative_precedence) => associative_precedence,
            _ => panic!("Wrong attribute variant."),
        }
    }
}

impl From<lexan::Token<AATerminal>> for AttributeData {
    fn from(token: lexan::Token<AATerminal>) -> Self {
        AttributeData::Token(token)
    }
}

impl From<lalr1_plus::Error<AATerminal>> for AttributeData {
    fn from(error: lalr1_plus::Error<AATerminal>) -> Self {
        match error {
            lalr1_plus::Error::LexicalError(error, expected) => {
                AttributeData::LexicalError(error, expected)
            }
            lalr1_plus::Error::SyntaxError(token, expected) => {
                AttributeData::SyntaxError(token, expected)
            }
        }
    }
}
