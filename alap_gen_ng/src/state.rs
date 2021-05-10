// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::production::{GrammarItemKey, GrammarItemSet};
use crate::symbol::non_terminal::NonTerminal;
use crate::symbol::terminal::{Token, TokenSet};
use crate::symbol::{Associativity, Symbol};
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
        match self.0.processed_state.get() {
            ProcessedState::Processed => true,
            _ => false,
        }
    }

    pub fn is_unprocessed(&self) -> bool {
        match self.0.processed_state.get() {
            ProcessedState::Unprocessed => true,
            _ => false,
        }
    }

    pub fn needs_reprocessing(&self) -> bool {
        match self.0.processed_state.get() {
            ProcessedState::NeedsReprocessing => true,
            _ => false,
        }
    }

    pub fn mark_as_processed(&self) {
        self.0.processed_state.set(ProcessedState::Processed)
    }

    pub fn merge_lookahead_sets(&self, item_set: &GrammarItemSet) {
        let mut additions = 0;
        for (key, other_look_ahead_set) in item_set.iter().filter(|(k, _)| k.is_kernel_item()) {
            if let Some(look_ahead_set) = self.0.grammar_items.borrow_mut().get_mut(key) {
                let current_len = look_ahead_set.len();
                *look_ahead_set |= other_look_ahead_set;
                additions += look_ahead_set.len() - current_len;
            } else {
                panic!("key sets should be identical to get here")
            }
        }
        if additions > 0 && self.is_processed() {
            self.0
                .processed_state
                .set(ProcessedState::NeedsReprocessing);
        }
    }

    pub fn add_shift_action(&self, token: Token, state: ParserState) {
        self.0.shift_list.borrow_mut().insert(token, state);
    }

    pub fn add_goto(&self, non_terminal: NonTerminal, state: ParserState) {
        self.0.goto_table.borrow_mut().insert(non_terminal, state);
    }

    pub fn set_error_recovery_state(&self, state: &ParserState) {
        *self.0.error_recovery_state.borrow_mut() = Some(state.clone());
    }

    pub fn kernel_key_set(&self) -> BTreeSet<GrammarItemKey> {
        self.0.grammar_items.borrow().kernel_key_set()
    }

    pub fn non_kernel_key_set(&self) -> BTreeSet<GrammarItemKey> {
        self.0.grammar_items.borrow().irreducible_key_set()
    }

    pub fn generate_goto_kernel(&self, symbol: &Symbol) -> GrammarItemSet {
        self.0.grammar_items.borrow().generate_goto_kernel(symbol)
    }

    pub fn resolve_shift_reduce_conflicts(&self) -> usize {
        // do this in two stages to avoid borrow/access conflicts
        let mut conflicts = vec![];
        for (shift_symbol, goto_state) in self.0.shift_list.borrow().iter() {
            for (item, look_ahead_set) in self
                .0
                .grammar_items
                .borrow()
                .iter()
                .filter(|x| x.0.is_reducible())
            {
                if look_ahead_set.contains(shift_symbol) {
                    conflicts.push((
                        shift_symbol.clone(),
                        goto_state.clone(),
                        item.clone(),
                        look_ahead_set.clone(),
                    ))
                }
            }
        }
        let mut shift_reduce_conflicts = self.0.shift_reduce_conflicts.borrow_mut();
        let mut shift_list = self.0.shift_list.borrow_mut();
        let mut grammar_items = self.0.grammar_items.borrow_mut();
        for (shift_symbol, goto_state, reducible_item, look_ahead_set) in conflicts.iter() {
            if shift_symbol.precedence() < reducible_item.precedence() {
                shift_list.remove(shift_symbol);
            } else if shift_symbol.precedence() > reducible_item.precedence() {
                grammar_items
                    .get_mut(&reducible_item)
                    .unwrap()
                    .remove(shift_symbol);
            } else if reducible_item.associativity() == Associativity::Left {
                shift_list.remove(shift_symbol);
            } else if reducible_item.has_error_recovery_tail() {
                grammar_items
                    .get_mut(&reducible_item)
                    .unwrap()
                    .remove(shift_symbol);
            } else {
                // Default: resolve in favour of shift but mark as unresolved
                // to give the user the option of accepting this resolution
                grammar_items
                    .get_mut(&reducible_item)
                    .unwrap()
                    .remove(shift_symbol);
                shift_reduce_conflicts.push((
                    shift_symbol.clone(),
                    goto_state.clone(),
                    reducible_item.clone(),
                    look_ahead_set.clone(),
                ))
            }
        }
        shift_reduce_conflicts.len()
    }

    pub fn resolve_reduce_reduce_conflicts(&self) -> usize {
        // TODO: think about moving reduce/reduce conflict resolution inside GrammarItemSet
        let reducible_key_set = self.0.grammar_items.borrow().reducible_key_set();
        if reducible_key_set.len() < 2 {
            return 0;
        }

        let mut reduce_reduce_conflicts = self.0.reduce_reduce_conflicts.borrow_mut();
        let reducible_key_set_2 = reducible_key_set.clone();
        for key_1 in reducible_key_set.iter() {
            for key_2 in reducible_key_set_2.iter() {
                if key_2 > key_1 {
                    let intersection = self
                        .0
                        .grammar_items
                        .borrow()
                        .look_ahead_intersection(key_1, key_2);
                    if intersection.len() > 0 && key_1.has_no_predicate() {
                        if key_1.has_error_recovery_tail() {
                            self.0
                                .grammar_items
                                .borrow_mut()
                                .remove_look_ahead_symbols(key_1, &intersection);
                        } else if key_2.has_error_recovery_tail() {
                            self.0
                                .grammar_items
                                .borrow_mut()
                                .remove_look_ahead_symbols(key_2, &intersection);
                        } else {
                            // Default: resolve in favour of first declared production
                            // but mark unresolved to give the user some options
                            self.0
                                .grammar_items
                                .borrow_mut()
                                .remove_look_ahead_symbols(key_2, &intersection);
                            reduce_reduce_conflicts
                                .push(((key_1.clone(), key_2.clone()), intersection))
                        }
                    }
                }
            }
        }
        reduce_reduce_conflicts.len()
    }
}
