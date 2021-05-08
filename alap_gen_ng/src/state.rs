// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::production::{GrammarItemKey, GrammarItemSet};
use crate::symbol::non_terminal::NonTerminal;
use crate::symbol::terminal::{Token, TokenSet};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::BTreeMap;
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
}
