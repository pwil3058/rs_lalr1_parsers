// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

pub mod grammar;

mod attributes;
mod parser;
mod production;
mod state;
mod symbol;

use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use thiserror::Error;

use lalr1;

use crate::parser::AATerminal;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Specification error {0}")]
    SpecificationError(#[from] lalr1::Error<AATerminal>),
    #[error("Grammar error {0}")]
    GrammarError(#[from] grammar::Error),
    #[error("I/O error {0}")]
    IoError(#[from] io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct ParserGenerator(grammar::Grammar);

impl ParserGenerator {
    pub fn new(text: &str, label: &str) -> Result<ParserGenerator> {
        let specification = grammar::Specification::new(text, label)?;
        let grammar = grammar::Grammar::try_from((specification, false, false))?;
        Ok(Self(grammar))
    }

    pub fn write_parser_code_to_file(&self, output_path: impl AsRef<Path>) -> io::Result<()> {
        let output_path = output_path.as_ref();
        self.0.write_parser_code_to_file(output_path)
    }
}

impl TryFrom<PathBuf> for ParserGenerator {
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<ParserGenerator> {
        let mut file = std::fs::File::open(&path)?;
        let mut specification_text = String::new();
        file.read_to_string(&mut specification_text)?;
        ParserGenerator::new(specification_text.trim(), path.to_str().unwrap())
    }
}
