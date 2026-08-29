// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use std::fmt::{Debug, Display};
use std::sync::Arc;

use thiserror::Error;

mod analyzer;
mod lexicon;
mod matcher;

pub use analyzer::{Location, Token, TokenStream};
use lexicon::Lexicon;

#[derive(Error, Debug, PartialEq)]
pub enum Error<T: Display + Copy + Debug + Eq> {
    #[error(transparent)]
    MatcherError(#[from] matcher::Error<T>),
    #[error(transparent)]
    AnalyzerError(#[from] analyzer::Error<T>),
}

pub struct LexicalAnalyzer<T>
where
    T: Ord + Copy + PartialEq + Debug + Display,
{
    lexicon: Arc<Lexicon<T>>,
}

impl<T> LexicalAnalyzer<T>
where
    T: Ord + Copy + PartialEq + Debug + Display,
{
    pub fn new(
        literal_lexemes: &[(T, &str)],
        regex_lexemes: &[(T, &str)],
        skip_regexes: &[&str],
        end_marker: T,
    ) -> Result<Self, Error<T>> {
        Ok(Self {
            lexicon: Arc::new(Lexicon::new(
                literal_lexemes,
                regex_lexemes,
                skip_regexes,
                end_marker,
            )?),
        })
    }

    pub fn token_stream(&self, text: &str, label: &str) -> TokenStream<T> {
        TokenStream::new(&self.lexicon, text, label)
    }
}

#[cfg(test)]
mod tests;
