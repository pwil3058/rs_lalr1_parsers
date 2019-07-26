extern crate regex;

use std::fmt::Debug;
use std::rc::Rc;

mod analyzer;
mod error;
mod lexicon;
mod matcher;

pub use analyzer::{Error, InjectableTokenStream, Location, Token, TokenStream};
use lexicon::Lexicon;

pub struct LexicalAnalyzer<H>
where
    H: Ord + Copy + PartialEq + Debug,
{
    lexicon: Rc<Lexicon<H>>,
}

impl<H> LexicalAnalyzer<H>
where
    H: Ord + Copy + PartialEq + Debug,
{
    pub fn new<'a>(
        literal_lexemes: &[(H, &'a str)],
        regex_lexemes: &[(H, &'a str)],
        skip_regex_strs: &[&'a str],
    ) -> Self {
        let lexicon = match Lexicon::new(literal_lexemes, regex_lexemes, skip_regex_strs) {
            Ok(lexicon) => Rc::new(lexicon),
            Err(err) => panic!("Fatal Error: {:?}", err)
        };
        Self { lexicon }
    }

    pub fn token_stream<'a>(&self, text: &'a str, label: &'a str) -> TokenStream<'a, H> {
        TokenStream::new(&self.lexicon, text, label)
    }

    pub fn injectable_token_stream<'a>(
        &self,
        text: &'a str,
        label: &'a str,
    ) -> InjectableTokenStream<'a, H> {
        InjectableTokenStream::new(&self.lexicon, text, label)
    }
}
