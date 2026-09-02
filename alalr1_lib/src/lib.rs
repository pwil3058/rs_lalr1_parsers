// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

pub mod grammar;

#[cfg(feature = "bootstrap")]
mod bootstrap;
#[cfg(not(feature = "bootstrap"))]
mod parser;

use std::io;
use std::io::Write;
use std::path::Path;

use thiserror::Error;

#[cfg(not(feature = "bootstrap"))]
use crate::parser::AATerminal;
#[cfg(feature = "bootstrap")]
use bootstrap::AATerminal;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Specification error {0}")]
    SpecificationError(#[from] lalr1::Error<AATerminal>),
    #[error("Grammar error {0}")]
    GrammarError(#[from] lalr1_common::GrammarError),
    #[error("I/O error {0}")]
    IoError(#[from] io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct ParserGenerator(grammar::Grammar);

impl ParserGenerator {
    pub fn new(path: impl AsRef<Path>) -> Result<ParserGenerator> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)?;
        let specification = grammar::Specification::new(&text, &path.to_string_lossy())?;
        let grammar = grammar::Grammar::try_from((specification, false, false))?;
        Ok(Self(grammar))
    }

    pub fn write_parser_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        self.0.write_parser_code(wtr)
    }

    pub fn write_parser_code_to_file(&self, output_path: impl AsRef<Path>) -> io::Result<()> {
        self.0.write_parser_code_to_file(output_path.as_ref())
    }

    pub fn write_description<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        self.0.write_description(wtr)
    }

    pub fn write_description_to_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
        self.0.write_description_to_file(path.as_ref())
    }
}

impl TryFrom<&str> for ParserGenerator {
    type Error = Error;

    fn try_from(text: &str) -> Result<ParserGenerator> {
        let specification = grammar::Specification::new(text, "text")?;
        let grammar = grammar::Grammar::try_from((specification, false, false))?;
        Ok(Self(grammar))
    }
}
