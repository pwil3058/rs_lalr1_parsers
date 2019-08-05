use std::fmt::Debug;

use lexan;

use crate::symbols::*;

#[derive(Debug, Clone)]
pub enum AttributeData<T>
where
    T: Debug + Copy + Eq + Ord,
{
    Token(lexan::Token<T>),
    SyntaxError(lexan::Token<T>, Vec<T>),
    LexicalError(lexan::Error<T>),
    SymbolList(Vec<Symbol>),
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
    pub fn matched_text<'a>(&'a self) -> &'a str {
        match self {
            AttributeData::Token(token) => token.lexeme(),
            AttributeData::SyntaxError(token, _) => token.lexeme(),
            AttributeData::LexicalError(error) => match error {
                lexan::Error::UnexpectedText(text, _) => text,
                lexan::Error::AmbiguousMatches(_, text, _) => text,
                lexan::Error::AdvancedWhenEmpty(_) => "",
            },
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn location<'a>(&'a self) -> &'a lexan::Location {
        match self {
            AttributeData::Token(token) => token.location(),
            AttributeData::SyntaxError(token, _) => token.location(),
            AttributeData::LexicalError(error) => match error {
                lexan::Error::UnexpectedText(_, location) => location,
                lexan::Error::AmbiguousMatches(_, _, location) => location,
                lexan::Error::AdvancedWhenEmpty(location) => location,
            },
            _ => panic!("Wrong attribute variant."),
        }
    }

    pub fn symbol_list<'a>(&'a self) -> &'a Vec<Symbol> {
        match self {
            AttributeData::SymbolList(list) => list,
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
