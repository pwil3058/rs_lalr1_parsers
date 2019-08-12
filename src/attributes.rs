use std::{fmt::Debug, rc::Rc};

use lexan;

use crate::state::ProductionTail;
use crate::symbols::*;

#[derive(Debug, Clone)]
pub enum AttributeData<T>
where
    T: Debug + Copy + Eq + Ord,
{
    Token(lexan::Token<T>),
    SyntaxError(lexan::Token<T>, Vec<T>),
    LexicalError(lexan::Error<T>),
    SymbolList(Vec<Rc<Symbol>>),
    Symbol(Option<Rc<Symbol>>),
    LeftHandSide(Rc<Symbol>),
    ProductionTail(ProductionTail),
    ProductionTailList(Vec<ProductionTail>),
    Action(String),
    Predicate(String),
    AssociativePrecedence(AssociativePrecedence),
    Default,
}

impl<T> Default for AttributeData<T>
where
    T: Debug + Copy + Eq + Ord,
{
    fn default() -> Self {
        AttributeData::Default
    }
}

impl<T> AttributeData<T>
where
    T: Debug + Copy + Eq + Ord,
{
    pub fn matched_text<'a>(&'a self) -> Option<&'a String> {
        match self {
            AttributeData::Token(token) => Some(token.lexeme()),
            AttributeData::SyntaxError(token, _) => Some(token.lexeme()),
            AttributeData::LexicalError(error) => match error {
                lexan::Error::UnexpectedText(text, _) => Some(text),
                lexan::Error::AmbiguousMatches(_, text, _) => Some(text),
                lexan::Error::AdvancedWhenEmpty(_) => None,
            },
            _ => None, //panic!("Wrong attribute variant."),
        }
    }

    pub fn text_and_location<'a>(&'a self) -> Option<(&'a String, &'a lexan::Location)> {
        match self {
            AttributeData::Token(token) => Some((token.lexeme(), token.location())),
            AttributeData::SyntaxError(token, _) => Some((token.lexeme(), token.location())),
            AttributeData::LexicalError(error) => match error {
                lexan::Error::UnexpectedText(text, location) => Some((text, location)),
                lexan::Error::AmbiguousMatches(_, text, location) => Some((text, location)),
                lexan::Error::AdvancedWhenEmpty(_) => None,
            },
            _ => None, // panic!("Wrong attribute variant."),
        }
    }

    pub fn location<'a>(&'a self) -> Option<&'a lexan::Location> {
        match self {
            AttributeData::Token(token) => Some(token.location()),
            AttributeData::SyntaxError(token, _) => Some(token.location()),
            AttributeData::LexicalError(error) => match error {
                lexan::Error::UnexpectedText(_, location) => Some(location),
                lexan::Error::AmbiguousMatches(_, _, location) => Some(location),
                lexan::Error::AdvancedWhenEmpty(location) => Some(location),
            },
            _ => None, //panic!("Wrong attribute variant."),
        }
    }

    pub fn symbol<'a>(&'a self) -> &'a Option<Rc<Symbol>> {
        match self {
            AttributeData::Symbol(symbol) => symbol,
            _ => &None, //panic!("Wrong attribute variant."),
        }
    }

    pub fn symbol_list<'a>(&'a self) -> &'a Vec<Rc<Symbol>> {
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

impl<T> From<lexan::Token<T>> for AttributeData<T>
where
    T: Debug + Copy + Eq + Ord,
{
    fn from(token: lexan::Token<T>) -> Self {
        AttributeData::Token(token)
    }
}

impl<T> From<lalr1plus::Error<T>> for AttributeData<T>
where
    T: Debug + Copy + Eq + Ord,
{
    fn from(error: lalr1plus::Error<T>) -> Self {
        match error {
            lalr1plus::Error::LexicalError(error) => AttributeData::LexicalError(error),
            lalr1plus::Error::SyntaxError(token, expected) => {
                AttributeData::SyntaxError(token, expected)
            }
        }
    }
}
