// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>



#[cfg(feature = "bootstrap")]
use crate::bootstrap::AATerminal;
#[cfg(not(feature = "bootstrap"))]
use crate::lap_gen::AATerminal;
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
    AssociativityAndPrecedence(Associativity, u16),
    Default,
}

impl Default for AttributeData {
    fn default() -> Self {
        AttributeData::Default
    }
}

impl AttributeData {
    pub fn matched_text(&self) -> &String {
        match self {
            AttributeData::Token(token) => token.lexeme(),
            AttributeData::SyntaxError(token, _) => token.lexeme(),
            AttributeData::LexicalError(error, _) => match error {
                lexan::Error::UnexpectedText(text, _) => text,
                lexan::Error::AmbiguousMatches(_, text, _) => text,
                lexan::Error::AdvancedWhenEmpty(_) => panic!("Wrong attribute variant."),
            },
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn location(&self) -> &lexan::Location {
        match self {
            AttributeData::Token(token) => token.location(),
            AttributeData::SyntaxError(token, _) => token.location(),
            AttributeData::LexicalError(error, _) => match error {
                lexan::Error::UnexpectedText(_, location) => location,
                lexan::Error::AmbiguousMatches(_, _, location) => location,
                lexan::Error::AdvancedWhenEmpty(location) => location,
            },
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn text_and_location(&self) -> (&String, &lexan::Location) {
        match self {
            AttributeData::Token(token) => (token.lexeme(), token.location()),
            AttributeData::SyntaxError(token, _) => (token.lexeme(), token.location()),
            AttributeData::LexicalError(error, _) => match error {
                lexan::Error::UnexpectedText(text, location) => (text, location),
                lexan::Error::AmbiguousMatches(_, text, location) => (text, location),
                lexan::Error::AdvancedWhenEmpty(_) => panic!("Wrong attribute variant."),
            },
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn number(&self) -> u32 {
        match self {
            AttributeData::Number(number) => *number,
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn symbol(&self) -> &Symbol {
        match self {
            AttributeData::Symbol(symbol) => symbol,
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn symbol_list(&self) -> &Vec<Symbol> {
        match self {
            AttributeData::SymbolList(list) => list,
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn symbol_list_mut(&mut self) -> &mut Vec<Symbol> {
        match self {
            AttributeData::SymbolList(list) => list,
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn left_hand_side(&self) -> &NonTerminal {
        match self {
            AttributeData::LeftHandSide(lhs) => lhs,
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn tag_or_token(&self) -> &TagOrToken {
        match self {
            AttributeData::TagOrToken(tag_or_token) => tag_or_token,
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn tag_or_token_list(&self) -> &Vec<TagOrToken> {
        match self {
            AttributeData::TagOrTokenList(list) => list,
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn tag_or_token_list_mut(&mut self) -> &mut Vec<TagOrToken> {
        match self {
            AttributeData::TagOrTokenList(list) => list,
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn production_tail(&self) -> &ProductionTail {
        match self {
            AttributeData::ProductionTail(production_tail) => production_tail,
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn production_tail_list(&self) -> &Vec<ProductionTail> {
        match self {
            AttributeData::ProductionTailList(list) => list,
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn production_tail_list_mut(&mut self) -> &mut Vec<ProductionTail> {
        match self {
            AttributeData::ProductionTailList(list) => list,
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn action(&self) -> &str {
        match self {
            AttributeData::Action(action) => action,
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }

    pub fn associativity_and_precedence(&self) -> (Associativity, u16) {
        match self {
            AttributeData::AssociativityAndPrecedence(associativity, precedence) => {
                (*associativity, *precedence)
            }
            _ => panic!("{self:?}: Wrong attribute variant."),
        }
    }
}

impl From<lexan::Token<AATerminal>> for AttributeData {
    fn from(token: lexan::Token<AATerminal>) -> Self {
        AttributeData::Token(token)
    }
}

impl From<lalr1::Error<AATerminal>> for AttributeData {
    fn from(error: lalr1::Error<AATerminal>) -> Self {
        match error {
            lalr1::Error::LexicalError(error, expected) => {
                AttributeData::LexicalError(error, expected)
            }
            lalr1::Error::SyntaxError(token, expected) => {
                AttributeData::SyntaxError(token, expected)
            }
        }
    }
}
