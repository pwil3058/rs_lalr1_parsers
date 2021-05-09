// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::production::{GrammarItemKey, GrammarItemSet};
use crate::symbol::non_terminal::NonTerminal;
use crate::symbol::terminal::{Token, TokenSet};
use crate::symbol::Symbol;
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub enum ProcessedState {
    Unprocessed,
    NeedsReprocessing,
    Processed,
}

impl Default for ProcessedState {
    fn default() -> Self {
        ProcessedState::Unprocessed
    }
}

#[derive(Debug, Default)]
pub struct ParserStateData {
    pub ident: u32,
    grammar_items: RefCell<GrammarItemSet>,
    shift_list: RefCell<BTreeMap<Token, ParserState>>,
    goto_table: RefCell<BTreeMap<NonTerminal, ParserState>>,
    error_recovery_state: RefCell<Option<ParserState>>,
    processed_state: Cell<ProcessedState>,
    shift_reduce_conflicts: RefCell<Vec<(Token, ParserState, GrammarItemKey, TokenSet)>>,
    reduce_reduce_conflicts: RefCell<Vec<((GrammarItemKey, GrammarItemKey), TokenSet)>>,
}

#[derive(Debug, Clone)]
pub struct ParserState(Rc<ParserStateData>);

impl PartialEq for ParserState {
    fn eq(&self, other: &Self) -> bool {
        self.0.ident == other.0.ident
    }
}

impl Eq for ParserState {}

impl PartialOrd for ParserState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.ident.partial_cmp(&other.0.ident)
    }
}

impl Ord for ParserState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl ParserState {
    pub fn new(ident: u32, grammar_items: GrammarItemSet) -> Self {
        let mut data = ParserStateData::default();
        *data.grammar_items.borrow_mut() = grammar_items;
        Self(Rc::new(data))
    }

    pub fn is_processed(&self) -> bool {
        match self.processed_state.get() {
            ProcessedState::Processed => true,
            _ => false,
        }
    }

    pub fn is_unprocessed(&self) -> bool {
        match self.processed_state.get() {
            ProcessedState::Unprocessed => true,
            _ => false,
        }
    }

    pub fn mark_as_processed(&self) {
        self.processed_state.set(ProcessedState::Processed)
    }

    pub fn merge_lookahead_sets(&self, item_set: &GrammarItemSet) {
        let mut additions = 0;
        for (key, other_look_ahead_set) in item_set.iter().filter(|(k, _)| k.is_kernel_item()) {
            if let Some(look_ahead_set) = self.grammar_items.borrow_mut().get_mut(key) {
                let current_len = look_ahead_set.len();
                *look_ahead_set |= other_look_ahead_set;
                additions += look_ahead_set.len() - current_len;
            } else {
                panic!("key sets should be identical to get here")
            }
        }
        if additions > 0 && self.is_processed() {
            self.processed_state.set(ProcessedState::NeedsReprocessing);
        }
    }

    pub fn non_kernel_keys(&self) -> BTreeSet<GrammarItemKey> {
        self.grammar_items.borrow().irreducible_keys()
    }

    pub fn generate_goto_kernel(&self, symbol: &Symbol) -> GrammarItemSet {
        self.grammar_items.borrow().generate_goto_kernel(symbol)
    }
}
