// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::alap_gen_ng::{AANonTerminal, AATerminal};
use crate::production::{GrammarItemKey, GrammarItemSet, Production, ProductionTail};
use crate::state::ParserState;
use crate::symbol::non_terminal::NonTerminal;
use crate::symbol::terminal::{Token, TokenSet};
use crate::symbol::{Symbol, SymbolTable};
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;
use std::io::{self, stderr, Write};

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
            let location = left_hand_side
                .first_definition()
                .expect("should be defined");
            left_hand_side.add_used_at(&location);
            let start_symbol = self.symbol_table.start_non_terminal_used_at(&location);
            let start_tail = ProductionTail::new(&[left_hand_side.into()], None, None, None);
            let start_production = Production::new(0, start_symbol, start_tail);
            self.productions.push(start_production);
        }
        let ident = self.productions.len() as u32;
        self.productions
            .push(Production::new(ident, left_hand_side.clone(), tail.clone()));
    }

    fn closure(&self, mut closure_set: GrammarItemSet) -> GrammarItemSet {
        let mut additions_made = true;
        while additions_made {
            additions_made = false;
            // Closables extraction as a new separate map necessary to avoid borrow conflict
            for (item_key, look_ahead_set) in closure_set.iter() {
                if let Some(symbol) = item_key.next_symbol() {
                    match symbol {
                        Symbol::Terminal(_) => (),
                        Symbol::NonTerminal(prospective_lhs) => {
                            for look_ahead_symbol in look_ahead_set.iter() {
                                let firsts = TokenSet::first_all_caps(
                                    item_key.rhs_tail(),
                                    look_ahead_symbol,
                                );
                                for production in self
                                    .productions
                                    .iter()
                                    .filter(|x| x.left_hand_side() == prospective_lhs)
                                {
                                    let prospective_key = GrammarItemKey::from(production);
                                    if let Some(set) = closure_set.get_mut(&prospective_key) {
                                        let len = set.len();
                                        *set |= &firsts;
                                        additions_made = additions_made || set.len() > len;
                                    } else {
                                        closure_set.insert(prospective_key, firsts.clone());
                                        additions_made = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        closure_set
    }
}

pub struct Grammar {
    specification: Specification,
    parser_states: Vec<ParserState>,
    unresolved_sr_conflicts: usize,
    unresolved_rr_conflicts: usize,
}

#[derive(Debug)]
pub enum Error {
    TooManyErrors(u32),
    UndefinedSymbols(u32),
}

impl TryFrom<Specification> for Grammar {
    type Error = Error;

    fn try_from(mut specification: Specification) -> Result<Self, Error> {
        for token in specification
            .symbol_table
            .tokens()
            .filter(|t| t.is_unused())
        {
            report_warning(
                token.defined_at(),
                &format!("Token \"{}\" is not used", token.name()),
            )
        }

        for tag in specification.symbol_table.tags().filter(|t| t.is_unused()) {
            report_warning(
                tag.defined_at(),
                &format!("Tag \"{}\" is not used", tag.name()),
            )
        }

        for non_terminal in specification
            .symbol_table
            .non_terminals()
            .filter(|t| t.is_unused())
        {
            report_warning(
                &non_terminal
                    .first_definition()
                    .expect("can't be both unused and undefined"),
                &format!("Non terminal \"{}\" is not used", non_terminal.name()),
            )
        }

        let mut undefined_symbols = 0;
        for non_terminal in specification
            .symbol_table
            .non_terminals()
            .filter(|t| t.is_undefined())
        {
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
            let start_item_key = GrammarItemKey::from(&specification.productions[0]);
            let mut start_look_ahead_set = TokenSet::new();
            start_look_ahead_set.insert(&Token::EndToken);
            let mut map = BTreeMap::<GrammarItemKey, TokenSet>::new();
            map.insert(start_item_key, start_look_ahead_set);
            let start_kernel = specification.closure(GrammarItemSet::from(map));
            let mut grammar = Self {
                specification,
                parser_states: vec![],
                unresolved_rr_conflicts: 0,
                unresolved_sr_conflicts: 0,
            };
            grammar.new_parser_state(start_kernel);
            while let Some(unprocessed_state) =
                grammar.parser_states.iter().find(|x| !x.is_processed())
            {
                let first_time = unprocessed_state.is_unprocessed();
                unprocessed_state.mark_as_processed();
                let mut already_done = BTreeSet::<Symbol>::new();
                for item_key in unprocessed_state.non_kernel_keys().iter() {
                    let symbol_x = item_key.next_symbol().expect("not reducible");
                    if !already_done.insert(symbol_x.clone()) {
                        continue;
                    };
                    let kernel_x = unprocessed_state.generate_goto_kernel(&symbol_x);
                    let item_set_x = grammar.specification.closure(kernel_x);
                    let goto_state =
                        if let Some(equivalent_state) = grammar.equivalent_state(&item_set_x) {
                            equivalent_state.merge_lookahead_sets(&item_set_x);
                            equivalent_state.clone()
                        } else {
                            grammar.new_parser_state(item_set_x)
                        };
                    if first_time {
                        match symbol_x {
                            Symbol::Terminal(token) => {
                                unprocessed_state.add_shift_action(token, goto_state)
                            }
                            Symbol::NonTerminal(non_terminal) => {
                                if non_terminal.is_error_non_terminal() {
                                    unprocessed_state.set_error_recovery_state(&goto_state);
                                }
                                unprocessed_state.add_goto(non_terminal, goto_state);
                            }
                        }
                    }
                }
            }
            grammar.resolve_conflicts();
            Ok(grammar)
        }
    }
}

impl Grammar {
    fn first_unprocessed_state(&self) -> Option<ParserState> {
        match self.parser_states.iter().find(|x| !x.is_processed()) {
            Some(unprocessed_state) => Some(unprocessed_state.clone()),
            None => None,
        }
    }

    fn new_parser_state(&mut self, grammar_items: GrammarItemSet) -> ParserState {
        let ident = self.parser_states.len() as u32;
        let parser_state = ParserState::new(ident, grammar_items);
        self.parser_states.push(parser_state.clone());
        parser_state
    }

    fn equivalent_state(&self, item_set: &GrammarItemSet) -> Option<&ParserState> {
        let target_keys = item_set.kernel_keys();
        if target_keys.len() > 0 {
            for parser_state in self.parser_states.iter() {
                if target_keys == parser_state.kernel_keys() {
                    return Some(parser_state);
                }
            }
        };
        None
    }
}
