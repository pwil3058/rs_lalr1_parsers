// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::production::{GrammarItemKey, GrammarItemSet};
use crate::symbol::non_terminal::NonTerminal;
use crate::symbol::terminal::{Token, TokenSet};
use crate::symbol::{Associativity, Symbol};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
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

#[derive(Debug)]
pub struct ParserState(Rc<ParserStateData>);

impl Clone for ParserState {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

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
        data.ident = ident;
        *data.grammar_items.borrow_mut() = grammar_items;
        Self(Rc::new(data))
    }

    pub fn ident(&self) -> u32 {
        self.0.ident
    }

    pub fn is_processed(&self) -> bool {
        match self.0.processed_state.get() {
            ProcessedState::Processed => true,
            _ => false,
        }
    }

    pub fn reduce_reduce_conflict_count(&self) -> usize {
        self.0.reduce_reduce_conflicts.borrow().len()
    }

    pub fn shift_reduce_conflict_count(&self) -> usize {
        self.0.shift_reduce_conflicts.borrow().len()
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

    pub fn error_goto_state_ident(&self) -> Option<u32> {
        Some(self.0.error_recovery_state.borrow().clone()?.ident())
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
                    .get_mut(reducible_item)
                    .unwrap()
                    .remove(shift_symbol);
            } else if reducible_item.associativity() == Associativity::Left {
                shift_list.remove(shift_symbol);
            } else if reducible_item.has_error_recovery_tail() {
                grammar_items
                    .get_mut(reducible_item)
                    .unwrap()
                    .remove(shift_symbol);
            } else {
                // Default: resolve in favour of shift but mark as unresolved
                // to give the user the option of accepting this resolution
                grammar_items
                    .get_mut(reducible_item)
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
                    if intersection.len() > 0 {
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

    pub fn is_recovery_state_for_token(&self, token: &Token) -> bool {
        if let Some(recovery_state) = self.0.error_recovery_state.borrow().clone() {
            if recovery_state
                .0
                .grammar_items
                .borrow()
                .error_recovery_look_ahead_set_contains(token)
            {
                return true;
            }
        };
        false
    }

    pub fn look_ahead_set(&self) -> TokenSet {
        self.0
            .grammar_items
            .borrow()
            .reducible_look_ahead_set()
            .union(&self.0.shift_list.borrow().keys().cloned().collect())
            .cloned()
            .collect()
    }

    pub fn write_next_action_code<W: Write>(
        &self,
        wtr: &mut W,
        indent: &str,
    ) -> std::io::Result<()> {
        let reductions = self.0.grammar_items.borrow().reductions();
        wtr.write_fmt(format_args!(
            "{}{} => match aa_tag {{\n",
            indent,
            self.ident()
        ))?;
        for (token, state) in self.0.shift_list.borrow().iter() {
            wtr.write_fmt(format_args!(
                "{}    {} => Action::Shift({}),\n",
                indent,
                token.name(),
                state.ident()
            ))?;
        }
        for (productions, look_ahead_set) in reductions.reductions() {
            for production in productions {
                wtr.write_fmt(format_args!("{indent}    // {production}\n"))?;
                if production.is_start_production() {
                    wtr.write_fmt(format_args!(
                        "{}    {} => Action::Accept,\n",
                        indent,
                        look_ahead_set.formated_as_or_list(),
                    ))?;
                } else {
                    wtr.write_fmt(format_args!(
                        "{}    {} => Action::Reduce({}),\n",
                        indent,
                        look_ahead_set.formated_as_or_list(),
                        production.ident(),
                    ))?;
                }
            }
        }
        wtr.write_fmt(format_args!("{indent}    _ => Action::SyntaxError,\n",))?;
        wtr.write_fmt(format_args!("{indent}}},\n"))?;
        Ok(())
    }

    pub fn write_goto_table_code<W: Write>(
        &self,
        wtr: &mut W,
        indent: &str,
    ) -> std::io::Result<()> {
        if self.0.goto_table.borrow().len() > 0 {
            wtr.write_fmt(format_args!("{}{} => match lhs {{\n", indent, self.ident()))?;
            for (non_terminal, state) in self.0.goto_table.borrow().iter() {
                wtr.write_fmt(format_args!(
                    "{}    AANonTerminal::{} => {},\n",
                    indent,
                    non_terminal.name(),
                    state.ident()
                ))?;
            }
            wtr.write_fmt(format_args!(
                "{indent}    _ => panic!(\"Malformed goto table: ({{}}, {{}})\", lhs, current_state),\n"
            ))?;
            wtr.write_fmt(format_args!("{indent}}},\n"))?;
        };
        Ok(())
    }

    pub fn description(&self) -> String {
        let mut string = format!("\nState<{}>:\n  Grammar Items:\n", self.0.ident);
        for (key, look_ahead_set) in self.0.grammar_items.borrow().iter() {
            string += &format!("    {key}: {look_ahead_set}\n");
        }
        string += "  Parser Action Table:\n";
        let mut empty = true;
        let shift_list = self.0.shift_list.borrow();
        if shift_list.len() > 0 {
            empty = false;
            string += "    Shifts:\n";
            for (token, state) in shift_list.iter() {
                string += &format!("      {} -> State<{}>\n", token, state.ident());
            }
        }
        let reductions = self.0.grammar_items.borrow().reductions();
        if reductions.len() > 0 {
            empty = false;
            string += "    Reductions:\n";
            for (productions, look_ahead_set) in reductions.reductions() {
                for production in productions.iter() {
                    if productions.len() == 1 && production.is_start_production() {
                        string += &format!(
                            "      {}: accept {}\n",
                            look_ahead_set.display_as_or_list(),
                            production
                        );
                    } else {
                        string += &format!(
                            "      {}: reduce {}\n",
                            look_ahead_set.display_as_or_list(),
                            production
                        );
                    }
                }
            }
        }
        if empty {
            string += "    <empty>\n";
        }
        string += "  Go To Table:\n";
        if self.0.goto_table.borrow().len() == 0 {
            string += "    <empty>\n";
        } else {
            for (non_terminal, state) in self.0.goto_table.borrow().iter() {
                string += &format!("    {} -> State<{}>\n", non_terminal.name(), state.ident());
            }
        }
        if let Some(ref state) = self.0.error_recovery_state.borrow().clone() {
            string += &format!("  Error Recovery State: State<{}>\n", state.ident());
            string += &format!("    Look Ahead: {}\n", state.look_ahead_set());
        } else {
            string += "  Error Recovery State: <none>\n";
        }
        if self.0.shift_reduce_conflicts.borrow().len() > 0 {
            string += "  Shift/Reduce Conflicts:\n";
            for (shift_token, goto_state, reducible_item, look_ahead_set) in
                self.0.shift_reduce_conflicts.borrow().iter()
            {
                string += &format!("    {shift_token}:\n");
                string += &format!("      shift -> State<{}>\n", goto_state.ident());
                string += &format!(
                    "      reduce {}: {}",
                    reducible_item.production(),
                    look_ahead_set
                );
            }
        }
        if self.0.reduce_reduce_conflicts.borrow().len() > 0 {
            string += "  Reduce/Reduce Conflicts:\n";
            for (items, intersection) in self.0.reduce_reduce_conflicts.borrow().iter() {
                string += &format!("    {intersection}\n");
                string += &format!(
                    "      reduce {} : {}\n",
                    items.0,
                    &self.0.grammar_items.borrow()[&items.0]
                );
                string += &format!(
                    "      reduce {} : {}\n",
                    items.1,
                    &self.0.grammar_items.borrow()[&items.1]
                );
            }
        }
        string
    }
}
