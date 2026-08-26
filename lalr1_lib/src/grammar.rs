// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

#[cfg(feature = "bootstrap")]
use crate::bootstrap::AATerminal;
#[cfg(not(feature = "bootstrap"))]
use crate::parser::AATerminal;

use crate::production::{GrammarItemKey, GrammarItemSet, Production, ProductionTail, Productions};
use crate::state::ParserStates;
use crate::symbol::non_terminal::NonTerminal;
use crate::symbol::terminal::{Token, TokenSet};
use crate::symbol::{Symbol, SymbolTable};

use lalr1::{OrderedSet, Parser};

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::io;
use std::io::{Write, stderr};
use std::path::Path;

pub fn report_error(location: &lexan::Location, what: &str) {
    writeln!(stderr(), "{location}: Error: {what}.").expect("what?");
}

pub fn report_warning(location: &lexan::Location, what: &str) {
    writeln!(stderr(), "{location}: Warning: {what}.").expect("what?");
}

#[derive(Debug, Default)]
pub struct Specification {
    pub symbol_table: SymbolTable,
    productions: Productions,
    preamble: String,
    pub attribute_type: String,
    pub target_type: String,
    pub error_count: u32,
    pub warning_count: u32,
    pub expected_rr_conflicts: u32,
    pub expected_sr_conflicts: u32,
}

impl lalr1::ReportError<AATerminal> for Specification {}

impl Specification {
    pub fn new(text: &str, label: &str) -> Result<Self, lalr1::Error<AATerminal>> {
        let mut spec = Specification::default();
        spec.parse_text(text, label)?;
        // Add dummy error production last so that it has lowest precedence during conflict resolution
        let symbol = spec.symbol_table.error_non_terminal.clone();
        let tail = ProductionTail::default();
        if !spec.symbol_table.error_non_terminal().is_unused() {
            spec.productions.push(Production::new(symbol, tail));
        }
        spec.symbol_table
            .start_non_terminal()
            .set_firsts_data(&spec.productions);
        spec.symbol_table
            .error_non_terminal()
            .set_firsts_data(&spec.productions);
        for non_terminal in spec.symbol_table.non_terminals() {
            non_terminal.set_firsts_data(&spec.productions)
        }
        Ok(spec)
    }

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
        if self.productions.is_empty() {
            let location = left_hand_side
                .first_definition()
                .expect("should be defined");
            left_hand_side.add_used_at(&location);
            let start_symbol = self.symbol_table.start_non_terminal_used_at(&location);
            let start_tail = ProductionTail::new(&[left_hand_side.into()], None, None);
            let start_production = Production::new(start_symbol, start_tail);
            self.productions.push(start_production);
        }
        self.productions
            .push(Production::new(left_hand_side.clone(), tail.clone()));
    }

    pub fn write_preamble_text<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write_all(self.preamble.as_bytes())?;
        Ok(())
    }
}

pub struct Grammar {
    specification: Specification,
    parser_states: ParserStates,
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Too many errors: {0}")]
    TooManyErrors(u32),
    #[error("{0} undefined symbols")]
    UndefinedSymbols(u32),
    #[error("Unexpected Shift/Reduce conflicts: {0} {1} {2}")]
    UnexpectedSRConflicts(u32, u32, String),
    #[error("Unexpected Reduce/Reduce conflicts: {0} {1} {2}")]
    UnexpectedRRConflicts(u32, u32, String),
}

impl TryFrom<(Specification, bool, bool)> for Grammar {
    type Error = Error;

    fn try_from(arg: (Specification, bool, bool)) -> Result<Self, Error> {
        let specification = arg.0;
        let ignore_sr_conflicts = arg.1;
        let ignore_rr_conflicts = arg.2;
        for token in specification.symbol_table.unused_tokens() {
            report_warning(
                token.defined_at(),
                &format!("Token \"{}\" is not used", token.name()),
            )
        }

        for tag in specification.symbol_table.unused_tags() {
            report_warning(
                tag.defined_at(),
                &format!("Tag \"{}\" is not used", tag.name()),
            )
        }

        for non_terminal in specification.symbol_table.unused_non_terminals() {
            report_warning(
                &non_terminal
                    .first_definition()
                    .expect("can't be both unused and undefined"),
                &format!("Non terminal \"{}\" is not used", non_terminal.name()),
            )
        }

        let mut undefined_symbols = 0;
        for non_terminal in specification.symbol_table.undefined_non_terminals() {
            for location in non_terminal.used_at() {
                report_error(
                    &location,
                    &format!("Non terminal \"{}\" is not defined", non_terminal.name()),
                );
            }
            undefined_symbols += 1;
        }

        if undefined_symbols > 0 {
            Err(Error::UndefinedSymbols(undefined_symbols))
        } else if specification.error_count > 0 {
            Err(Error::TooManyErrors(specification.error_count))
        } else {
            let start_item_key = GrammarItemKey::from(specification.productions.base());
            let mut start_look_ahead_set = TokenSet::new();
            start_look_ahead_set.insert(&Token::End);
            #[allow(clippy::mutable_key_type)]
            let mut map = BTreeMap::<GrammarItemKey, TokenSet>::new();
            map.insert(start_item_key, start_look_ahead_set);
            let start_kernel = specification.productions.closure(GrammarItemSet::from(map));
            let mut grammar = Self {
                specification,
                parser_states: ParserStates::default(),
            };
            grammar.parser_states.new_parser_state(start_kernel);
            while let Some(unprocessed_state) = grammar.parser_states.first_unprocessed_state() {
                let first_time = !unprocessed_state.needs_reprocessing();
                unprocessed_state.mark_as_processed();
                let mut already_done = OrderedSet::<Symbol>::new();
                for item_key in unprocessed_state.non_kernel_key_set().iter() {
                    let symbol_x = item_key.next_symbol().expect("not reducible");
                    if !already_done.insert(symbol_x.clone()) {
                        continue;
                    };
                    let kernel_x = unprocessed_state.generate_goto_kernel(symbol_x);
                    let item_set_x = grammar.specification.productions.closure(kernel_x);
                    let goto_state = if let Some(equivalent_state) =
                        grammar.parser_states.equivalent_state(&item_set_x)
                    {
                        equivalent_state.merge_lookahead_sets(&item_set_x);
                        equivalent_state.clone()
                    } else {
                        grammar.parser_states.new_parser_state(item_set_x)
                    };
                    if first_time {
                        match symbol_x {
                            Symbol::Terminal(token) => {
                                unprocessed_state.add_shift_action(token.clone(), goto_state)
                            }
                            Symbol::NonTerminal(non_terminal) => {
                                if non_terminal.is_error() {
                                    unprocessed_state.set_error_recovery_state(&goto_state);
                                }
                                unprocessed_state.add_goto(non_terminal.clone(), goto_state);
                            }
                        }
                    }
                }
            }
            let (sr_conflicts, rr_conflicts) = grammar.parser_states.resolve_conflicts();
            if !ignore_sr_conflicts && sr_conflicts != grammar.specification.expected_sr_conflicts {
                Err(Error::UnexpectedSRConflicts(
                    sr_conflicts,
                    grammar.specification.expected_sr_conflicts,
                    grammar.parser_states.describe_sr_conflict_states(),
                ))
            } else if !ignore_rr_conflicts
                && rr_conflicts != grammar.specification.expected_rr_conflicts
            {
                Err(Error::UnexpectedRRConflicts(
                    rr_conflicts,
                    grammar.specification.expected_rr_conflicts,
                    grammar.parser_states.describe_rr_conflict_states(),
                ))
            } else {
                Ok(grammar)
            }
        }
    }
}

impl Grammar {
    fn write_parser_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write_all(b"// generated by lalr1_gen.\n\n")?;

        self.specification.write_preamble_text(wtr)?;
        self.specification
            .symbol_table
            .write_symbol_enum_code(wtr)?;
        self.write_parser_implementation_code(wtr)?;
        Ok(())
    }

    pub fn write_parser_code_to_file(&self, file_path: &Path) -> io::Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        self.write_parser_code(&mut file)?;
        Ok(())
    }

    fn write_parser_implementation_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        let attr = &self.specification.attribute_type;
        let parser = &self.specification.target_type;
        let text =
            format!("impl lalr1::Parser<AATerminal, AANonTerminal, {attr}> for {parser} {{\n");
        wtr.write_all(text.as_bytes())?;
        wtr.write_all(
            b"    fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<AATerminal> {\n",
        )?;
        wtr.write_all(b"        &AALEXAN\n")?;
        wtr.write_all(b"    }\n\n")?;
        self.parser_states
            .write_error_recovery_code(wtr, &self.specification.symbol_table)?;
        self.parser_states.write_look_ahead_set_code(wtr)?;
        self.parser_states
            .write_next_action_code(wtr, &self.specification.attribute_type)?;
        self.specification
            .productions
            .write_production_data_code(wtr)?;
        self.parser_states.write_goto_table_code(wtr)?;
        self.specification
            .productions
            .write_semantic_action_code(wtr, &self.specification.attribute_type)?;
        wtr.write_all(b"}\n")?;
        Ok(())
    }

    pub fn write_description(&self, file_path: &Path) -> io::Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        file.write_all(self.specification.symbol_table.description().as_bytes())?;
        file.write_all(b"\nProductions:\n")?;
        self.specification
            .productions
            .write_description(&mut file)?;
        self.parser_states.write_description(&mut file)?;
        Ok(())
    }
}
