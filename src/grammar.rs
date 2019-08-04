use std::io::{stderr, Write};

use lexan;

use crate::symbols::{self, SymbolTable};

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

    pub fn set_header(&mut self, header: &str) {
        self.header = header.to_string();
    }

    pub fn set_coda(&mut self, coda: &str) {
        self.coda = coda.to_string();
    }

    pub fn add_field(
        &mut self,
        name: &str,
        field_type: &str,
        location: &lexan::Location,
    ) -> Result<(), symbols::Error> {
        self.symbol_table.add_field(name, field_type, location)
    }
}
