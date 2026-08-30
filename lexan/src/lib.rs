// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use std::fmt::{Debug, Display};
use std::sync::Arc;

mod matcher;

pub mod lexicon;
pub mod token_stream;

pub use lexicon::Lexicon;
pub use token_stream::{Location, Token, TokenStream};

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
    ) -> Result<Self, lexicon::Error<T>> {
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
