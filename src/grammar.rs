use std::io::{stderr, Write};

use lexan;

use crate::symbols::{self, Associativity, SymbolTable};

#[derive(Debug, Default, Clone)]
pub struct ParserSpecification {
    symbol_table: SymbolTable,
    coda: String,
    header: String,
    preamble: String,
    error_count: u32,
    warning_count: u32,
}

impl ParserSpecification {
    pub fn is_allowable_name(name: &str) -> bool {
        !(name.starts_with("aa") || name.starts_with("AA"))
    }

    pub fn is_known_tag(&self, name: &str) -> bool {
        self.symbol_table.is_known_tag(name)
    }

    pub fn is_known_token(&self, name: &str) -> bool {
        self.symbol_table.is_known_token(name)
    }

    pub fn is_known_non_terminal(&self, name: &str) -> bool {
        self.symbol_table.is_known_non_terminal(name)
    }

    pub fn error(&mut self, location: &lexan::Location, what: &str) {
        writeln!(stderr(), "{}:Error: {}.", location, what).expect("what?");
        self.error_count += 1;
    }

    pub fn warning(&mut self, location: &lexan::Location, what: &str) {
        writeln!(stderr(), "{}:Warning: {}.", location, what).expect("what?");
        self.warning_count += 1;
    }

    pub fn set_preamble(&mut self, preamble: &str) {
        self.preamble = preamble.to_string();
    }

    pub fn add_token(
        &mut self,
        name: &str,
        pattern: &str,
        location: &lexan::Location,
    ) -> Result<(), symbols::Error> {
        self.symbol_table.add_token(name, pattern, location)
    }

    pub fn add_skip_rule(&mut self, rule: & str) {
        self.symbol_table.add_skip_rule(rule);
    }

    pub fn set_precedence(&mut self, associativity: Associativity, tags: &Vec<String>) {
        self.symbol_table.set_precedence(associativity, tags);
    }
}
