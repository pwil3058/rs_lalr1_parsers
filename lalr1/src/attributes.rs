// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use std::fmt::{Debug, Display};

use crate::production::ProductionTail;
use crate::symbol::non_terminal::NonTerminal;
use crate::symbol::tag::TagOrToken;
use crate::symbol::{Associativity, Symbol};

use crate::OrderedSet;

#[allow(unused)]
#[derive(Debug, Default, Clone)]
pub enum AttributeData<A: Copy + Ord + Debug + Display> {
    Token(lexan::Token<A>),
    SyntaxError(lexan::Token<A>, OrderedSet<A>),
    LexicalError(lexan::Error<A>, OrderedSet<A>),
    Number(u32),
    Symbol(Symbol),
    SymbolList(Vec<Symbol>),
    LeftHandSide(NonTerminal),
    TagOrToken(TagOrToken),
    TagOrTokenList(Vec<TagOrToken>),
    ProductionTail(ProductionTail),
    ProductionTailList(Vec<ProductionTail>),
    Action(String),
    #[cfg(feature = "augmented")]
    Predicate(String),
    AssociativityAndPrecedence(Associativity, u16),
    #[default]
    Default,
}

impl<A: Copy + Ord + Debug + Display> AttributeData<A> {
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

    #[cfg(feature = "augmented")]
    pub fn predicate(&self) -> &str {
        match self {
            AttributeData::Predicate(predicate) => predicate,
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

impl<A: Copy + Ord + Debug + Display> From<lexan::Token<A>> for AttributeData<A> {
    fn from(token: lexan::Token<A>) -> Self {
        AttributeData::Token(token)
    }
}

impl<A: Copy + Ord + Debug + Display> From<crate::parser::Error<A>> for AttributeData<A> {
    fn from(error: crate::parser::Error<A>) -> Self {
        match error {
            crate::parser::Error::LexicalError(error, expected) => {
                AttributeData::LexicalError(error, expected)
            }
            crate::parser::Error::SyntaxError(token, expected) => {
                AttributeData::SyntaxError(token, expected)
            }
        }
    }
}
