// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::alap_gen_ng::{AANonTerminal, AATerminal};
use crate::production::{Production, ProductionTail};
use crate::symbol::non_terminal::NonTerminal;
use crate::symbol::SymbolTable;
use std::io::stderr;

pub fn report_error(location: &lexan::Location, what: &str) {
    writeln!(stderr(), "{}: Error: {}.", location, what).expect("what?");
}

pub fn report_warning(location: &lexan::Location, what: &str) {
    writeln!(stderr(), "{}: Warning: {}.", location, what).expect("what?");
}

#[derive(Debug, Default)]
pub struct Specification {
    pub symbol_table: SymbolTable,
    productions: Vec<Production>,
    preamble: String,
    pub attribute_type: String,
    pub target_type: String,
    pub error_count: u32,
    pub warning_count: u32,
}

impl lalr1_plus::ReportError<AATerminal> for Specification {}

impl Specification {
    pub fn is_allowable_name(name: &str) -> bool {
        !(name.starts_with("aa") || name.starts_with("AA"))
    }

    pub fn error(&mut self, location: &lexan::Location, what: &str) {
        report_error(location, what);
        self.error_count += 1;
    }

    pub fn warning(&mut self, location: &lexan::Location, what: &str) {
        report_warning(location, what);
        self.warning_count += 1;
    }

    pub fn set_preamble(&mut self, preamble: &str) {
        self.preamble = preamble.to_string();
    }

    pub fn new_production(&mut self, left_hand_side: &NonTerminal, tail: &ProductionTail) {
        if self.productions.len() == 0 {
            let location = left_hand_side.defined_at().expect("should be defined");
            left_hand_side.add_used_at(&location);
            let start_symbol = self
                .symbol_table
                .use_symbol_named(&AANonTerminal::AAStart.to_string(), &location)
                .unwrap();
            let start_tail = ProductionTail::new(vec![left_hand_side.into()], None, None, None);
            let start_production = Production::new(0, start_symbol, start_tail);
            self.productions.push(start_production);
        }
        let ident = self.productions.len() as u32;
        self.productions
            .push(Production::new(ident, left_hand_side.clone(), tail.clone()));
    }
}
