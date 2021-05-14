// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use lexan;

#[cfg(not(feature = "bootstrap"))]
use crate::alap_gen_ng::AATerminal;
#[cfg(feature = "bootstrap")]
use crate::bootstrap::AATerminal;
use crate::production::ProductionTail;
use crate::symbol::non_terminal::NonTerminal;
use crate::symbol::tag::TagOrToken;
use crate::symbol::{Associativity, Symbol};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub enum AttributeData {
    Token(lexan::Token<AATerminal>),
    SyntaxError(lexan::Token<AATerminal>, BTreeSet<AATerminal>),
    LexicalError(lexan::Error<AATerminal>, BTreeSet<AATerminal>),
    Number(u32),
    Symbol(Symbol),
    SymbolList(Vec<Symbol>),
    LeftHandSide(NonTerminal),
    TagOrToken(TagOrToken),
    TagOrTokenList(Vec<TagOrToken>),
    ProductionTail(ProductionTail),
    ProductionTailList(Vec<ProductionTail>),
    Action(String),
    Predicate(String),
    AssociativityAndPrecedence(Associativity, u16),
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
            _ => panic!("{:?}: Wrong attribute variant.", self),
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
            _ => panic!("{:?}: Wrong attribute variant.", self),
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
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn number(&self) -> u32 {
        match self {
            AttributeData::Number(number) => *number,
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn symbol<'a>(&'a self) -> &'a Symbol {
        match self {
            AttributeData::Symbol(symbol) => symbol,
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn symbol_list<'a>(&'a self) -> &'a Vec<Symbol> {
        match self {
            AttributeData::SymbolList(list) => list,
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn symbol_list_mut<'a>(&'a mut self) -> &'a mut Vec<Symbol> {
        match self {
            AttributeData::SymbolList(list) => list,
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn left_hand_side<'a>(&'a self) -> &'a NonTerminal {
        match self {
            AttributeData::LeftHandSide(lhs) => lhs,
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn tag_or_token<'a>(&'a self) -> &'a TagOrToken {
        match self {
            AttributeData::TagOrToken(tag_or_token) => tag_or_token,
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn tag_or_token_list<'a>(&'a self) -> &'a Vec<TagOrToken> {
        match self {
            AttributeData::TagOrTokenList(list) => list,
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn tag_or_token_list_mut<'a>(&'a mut self) -> &'a mut Vec<TagOrToken> {
        match self {
            AttributeData::TagOrTokenList(list) => list,
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn production_tail<'a>(&'a self) -> &'a ProductionTail {
        match self {
            AttributeData::ProductionTail(production_tail) => production_tail,
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn production_tail_list<'a>(&'a self) -> &'a Vec<ProductionTail> {
        match self {
            AttributeData::ProductionTailList(list) => list,
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn production_tail_list_mut<'a>(&'a mut self) -> &'a mut Vec<ProductionTail> {
        match self {
            AttributeData::ProductionTailList(list) => list,
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn action<'a>(&'a self) -> &'a str {
        match self {
            AttributeData::Action(action) => action,
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn predicate<'a>(&'a self) -> &'a str {
        match self {
            AttributeData::Predicate(predicate) => predicate,
            _ => panic!("{:?}: Wrong attribute variant.", self),
        }
    }

    pub fn associativity_and_precedence(&self) -> (Associativity, u16) {
        match self {
            AttributeData::AssociativityAndPrecedence(associativity, precedence) => {
                (*associativity, *precedence)
            }
            _ => panic!("{:?}: Wrong attribute variant.", self),
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
