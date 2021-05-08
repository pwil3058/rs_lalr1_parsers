// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use lexan;

use crate::alap_gen_ng::AATerminal;
use crate::production::ProductionTail;
use crate::symbol::non_terminal::NonTerminal;
use crate::symbol::tag::TagOrToken;

#[derive(Debug, Clone)]
pub enum AttributeData {
    Token(lexan::Token<AATerminal>),
    TagOrToken(TagOrToken),
    TagOrTokenList(Vec<TagOrToken>),
    NonTerminal(NonTerminal),
    ProductionTail(ProductionTail),
    ProductionTailList(Vec<ProductionTail>),
    Default,
}

impl Default for AttributeData {
    fn default() -> Self {
        AttributeData::Default
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

impl AttributeData {
    pub fn matched_text<'a>(&'a self) -> &'a String {
        match self {
            AttributeData::Token(token) => token.lexeme(),
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn text_and_location<'a>(&'a self) -> (&'a String, &'a lexan::Location) {
        match self {
            AttributeData::Token(token) => (token.lexeme(), token.location()),
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn tag_or_token<'a>(&'a self) -> &'a TagOrToken {
        match self {
            AttributeData::TagOrToken(tag_or_token) => tag_or_token,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn tag_or_token_list<'a>(&'a self) -> &'a Vec<TagOrToken> {
        match self {
            AttributeData::TagOrTokenList(list) => list,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn tag_or_token_list_mut<'a>(&'a mut self) -> &'a mut Vec<TagOrToken> {
        match self {
            AttributeData::TagOrTokenList(list) => list,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn non_terminal<'a>(&'a self) -> &'a NonTerminal {
        match self {
            AttributeData::NonTerminal(non_terminal) => non_terminal,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn production_tail<'a>(&'a self) -> &'a ProductionTail {
        match self {
            AttributeData::ProductionTail(production_tail) => production_tail,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn production_tail_list<'a>(&'a self) -> &'a Vec<ProductionTail> {
        match self {
            AttributeData::ProductionTailList(list) => list,
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn production_tail_list_mut<'a>(&'a mut self) -> &'a mut Vec<ProductionTail> {
        match self {
            AttributeData::ProductionTailList(list) => list,
            _ => panic!("Wrong attribute variant."),
        }
    }
}
