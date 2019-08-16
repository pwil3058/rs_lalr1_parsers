use std::{
    cell::{Cell, RefCell},
    fmt,
    io::{stderr, Write},
    rc::Rc,
};

use ordered_collections::{
    ordered_set::ord_set_iterators::{Selection, SkipAheadIterator, ToSet},
    OrderedMap, OrderedSet,
};

use crate::symbols::{AssociativePrecedence, Associativity, Symbol};

#[derive(Debug, Clone, Default)]
pub struct ProductionTail {
    right_hand_side: Vec<Rc<Symbol>>,
    predicate: Option<String>,
    associative_precedence: AssociativePrecedence,
    action: Option<String>,
}

impl ProductionTail {
    pub fn new(
        right_hand_side: Vec<Rc<Symbol>>,
        predicate: Option<String>,
        associative_precedence: Option<AssociativePrecedence>,
        action: Option<String>,
    ) -> Self {
        let associative_precedence = if let Some(associative_precedence) = associative_precedence {
            associative_precedence
        } else if let Some(associative_precedence) = rhs_associated_precedence(&right_hand_side) {
            associative_precedence
        } else {
            AssociativePrecedence::default()
        };
        Self {
            right_hand_side,
            predicate,
            action,
            associative_precedence,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Production {
    ident: u32,
    left_hand_side: Rc<Symbol>,
    tail: ProductionTail,
}

impl_ident_cmp!(Production);

fn rhs_associated_precedence(symbols: &[Rc<Symbol>]) -> Option<AssociativePrecedence> {
    for symbol in symbols.iter() {
        if symbol.is_token() {
            let associative_precedence = symbol.associative_precedence();
            return Some(associative_precedence);
        }
    }
    None
}

impl Production {
    pub fn new(ident: u32, left_hand_side: Rc<Symbol>, tail: ProductionTail) -> Self {
        Self {
            ident,
            left_hand_side,
            tail,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tail.right_hand_side.len() == 0
    }

    pub fn left_hand_side(&self) -> &Rc<Symbol> {
        &self.left_hand_side
    }

    pub fn right_hand_side_symbols(&self) -> impl Iterator<Item = &Rc<Symbol>> {
        self.tail.right_hand_side.iter()
    }

    pub fn associativity(&self) -> Associativity {
        self.tail.associative_precedence.associativity
    }

    pub fn precedence(&self) -> u32 {
        self.tail.associative_precedence.precedence
    }

    pub fn predicate(&self) -> Option<&String> {
        if let Some(ref string) = self.tail.predicate {
            Some(string)
        } else {
            None
        }
    }

    pub fn has_error_recovery_tail(&self) -> bool {
        if let Some(symbol) = self.tail.right_hand_side.last() {
            symbol.is_syntax_error()
        } else {
            false
        }
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct GrammarItemKey {
    production: Rc<Production>,
    dot: usize,
}

impl GrammarItemKey {
    pub fn new(production: Rc<Production>) -> Rc<Self> {
        Rc::new(Self { production, dot: 0 })
    }

    pub fn shifted(&self) -> Rc<Self> {
        let production = Rc::clone(&self.production);
        let dot = self.dot + 1;
        Rc::new(Self { production, dot })
    }

    pub fn is_closable(&self) -> bool {
        if let Some(symbol) = self.production.tail.right_hand_side.get(self.dot) {
            symbol.is_non_terminal()
        } else {
            false
        }
    }

    pub fn is_kernel_item(&self) -> bool {
        self.dot > 0 || self.production.left_hand_side.is_start_symbol()
    }

    pub fn is_reducible(&self) -> bool {
        self.dot >= self.production.tail.right_hand_side.len()
    }

    pub fn next_symbol(&self) -> Option<&Rc<Symbol>> {
        self.production.tail.right_hand_side.get(self.dot)
    }

    pub fn next_symbol_is(&self, symbol: &Rc<Symbol>) -> bool {
        if let Some(next_symbol) = self.next_symbol() {
            next_symbol == symbol
        } else {
            false
        }
    }

    pub fn rhs_tail(&self) -> &[Rc<Symbol>] {
        &self.production.tail.right_hand_side[self.dot + 1..]
    }

    pub fn associativity(&self) -> Associativity {
        self.production.associativity()
    }

    pub fn precedence(&self) -> u32 {
        self.production.precedence()
    }

    pub fn predicate(&self) -> Option<&String> {
        self.production.predicate()
    }

    pub fn has_error_recovery_tail(&self) -> bool {
        self.production.has_error_recovery_tail()
    }

    pub fn has_reducible_error_recovery_tail(&self) -> bool {
        self.is_reducible() && self.production.has_error_recovery_tail()
    }
}

pub struct GrammarItemSet(OrderedMap<Rc<GrammarItemKey>, OrderedSet<Rc<Symbol>>>);

impl GrammarItemSet {
    pub fn new(map: OrderedMap<Rc<GrammarItemKey>, OrderedSet<Rc<Symbol>>>) -> Self {
        GrammarItemSet(map)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn closables(&self) -> Vec<(Rc<GrammarItemKey>, OrderedSet<Rc<Symbol>>)> {
        let mut closables = vec![];
        for (key, set) in self.0.iter().filter(|x| x.0.is_closable()) {
            closables.push((Rc::clone(key), set.clone()));
        }
        closables
    }

    pub fn generate_goto_kernel(&self, symbol: &Rc<Symbol>) -> GrammarItemSet {
        let mut map = OrderedMap::new();
        for (item_key, look_ahead_set) in self.0.iter() {
            if item_key.next_symbol_is(symbol) {
                map.insert(item_key.shifted(), look_ahead_set.clone());
            }
        }
        GrammarItemSet(map)
    }

    pub fn kernel_keys(&self) -> OrderedSet<Rc<GrammarItemKey>> {
        let mut keys = OrderedSet::new();
        for key in self.0.keys().filter(|x| x.is_kernel_item()) {
            keys.insert(Rc::clone(key));
        }
        keys
    }

    pub fn irreducible_keys(&self) -> OrderedSet<Rc<GrammarItemKey>> {
        self.0.keys().select(|x| !x.is_reducible()).to_set()
    }

    pub fn reducible_keys(&self) -> OrderedSet<Rc<GrammarItemKey>> {
        self.0.keys().select(|x| x.is_reducible()).to_set()
    }

    pub fn keys(&self) -> OrderedSet<Rc<GrammarItemKey>> {
        self.0.keys().to_set()
    }

    pub fn get_mut(&mut self, key: &Rc<GrammarItemKey>) -> Option<&mut OrderedSet<Rc<Symbol>>> {
        self.0.get_mut(key)
    }

    pub fn insert(
        &mut self,
        key: Rc<GrammarItemKey>,
        look_ahead_set: OrderedSet<Rc<Symbol>>,
    ) -> Option<OrderedSet<Rc<Symbol>>> {
        self.0.insert(key, look_ahead_set)
    }

    pub fn look_ahead_intersection(
        &self,
        key1: &GrammarItemKey,
        key2: &GrammarItemKey,
    ) -> OrderedSet<Rc<Symbol>> {
        self.0
            .get(key1)
            .unwrap()
            .intersection(self.0.get(key2).unwrap())
            .to_set()
    }

    pub fn remove_look_ahead_symbol(&mut self, key: &GrammarItemKey, symbol: &Rc<Symbol>) {
        self.0.get_mut(key).unwrap().remove(symbol);
    }

    pub fn remove_look_ahead_symbols(
        &mut self,
        key: &GrammarItemKey,
        symbols: &OrderedSet<Rc<Symbol>>,
    ) {
        let look_ahead_set = self.0.get_mut(key).unwrap();
        *look_ahead_set = look_ahead_set.difference(symbols).to_set();
    }

    pub fn error_recovery_look_ahead_set_contains(&self, token: &Rc<Symbol>) -> bool {
        for (item_key, look_ahead_set) in self
            .0
            .iter()
            .filter(|x| x.0.has_reducible_error_recovery_tail())
        {
            if look_ahead_set.contains(token) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ProcessedState {
    Unprocessed,
    NeedsReprocessing,
    Processed,
}

pub struct ParserState {
    pub ident: u32,
    grammar_items: RefCell<GrammarItemSet>,
    shift_list: RefCell<OrderedMap<Rc<Symbol>, Rc<ParserState>>>,
    goto_table: RefCell<OrderedMap<Rc<Symbol>, Rc<ParserState>>>,
    error_recovery_state: RefCell<Option<Rc<ParserState>>>,
    processed_state: Cell<ProcessedState>,
    shift_reduce_conflicts: RefCell<
        Vec<(
            Rc<Symbol>,
            Rc<ParserState>,
            Rc<GrammarItemKey>,
            OrderedSet<Rc<Symbol>>,
        )>,
    >,
    reduce_reduce_conflicts: RefCell<
        Vec<(
            (Rc<GrammarItemKey>, Rc<GrammarItemKey>),
            OrderedSet<Rc<Symbol>>,
        )>,
    >,
}

impl fmt::Debug for ParserState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "State#{}({:?}):",
            self.ident,
            self.grammar_items.borrow().keys()
        )
    }
}

impl_ident_cmp!(ParserState);

impl ParserState {
    pub fn new(ident: u32, grammar_items: GrammarItemSet) -> Rc<Self> {
        Rc::new(Self {
            ident,
            grammar_items: RefCell::new(grammar_items),
            shift_list: RefCell::new(OrderedMap::new()),
            goto_table: RefCell::new(OrderedMap::new()),
            error_recovery_state: RefCell::new(None),
            processed_state: Cell::new(ProcessedState::Unprocessed),
            shift_reduce_conflicts: RefCell::new(vec![]),
            reduce_reduce_conflicts: RefCell::new(vec![]),
        })
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
        for (key, other_look_ahead_set) in item_set.0.iter().filter(|(k, _)| k.is_kernel_item()) {
            if let Some(look_ahead_set) = self.grammar_items.borrow_mut().0.get_mut(key) {
                let current_len = look_ahead_set.len();
                *look_ahead_set = look_ahead_set.union(other_look_ahead_set).to_set();
                additions += look_ahead_set.len() - current_len;
            } else {
                panic!("key sets should be identical to get here")
            }
        }
        if additions > 0 {
            self.processed_state.set(ProcessedState::NeedsReprocessing);
        }
    }

    pub fn add_shift_action(&self, token: Rc<Symbol>, state: Rc<ParserState>) {
        self.shift_list.borrow_mut().insert(token, state);
    }

    pub fn add_goto(&self, token: Rc<Symbol>, state: Rc<ParserState>) {
        self.goto_table.borrow_mut().insert(token, state);
    }

    pub fn set_error_recovery_state(&self, state: &Rc<ParserState>) {
        //self.error_recovery_state.set(Some(Rc::clone(state)));
        *self.error_recovery_state.borrow_mut() = Some(Rc::clone(state));
    }

    pub fn error_goto_state_ident(&self) -> Option<u32> {
        if let Some(error_recovery_state) = self.error_recovery_state.borrow().clone() {
            Some(error_recovery_state.ident)
        } else {
            None
        }
    }

    pub fn has_empty_look_ahead_set(&self) -> bool {
        if self.shift_list.borrow().len() > 0 {
            return false;
        } else {
            for (key, look_ahead_set) in self.grammar_items.borrow().0.iter() {
                if key.is_reducible() && look_ahead_set.len() > 0 {
                    return false;
                }
            }
        };
        true
    }

    pub fn kernel_keys(&self) -> OrderedSet<Rc<GrammarItemKey>> {
        self.grammar_items.borrow().kernel_keys()
    }

    pub fn non_kernel_keys(&self) -> OrderedSet<Rc<GrammarItemKey>> {
        self.grammar_items.borrow().irreducible_keys()
    }

    pub fn generate_goto_kernel(&self, symbol: &Rc<Symbol>) -> GrammarItemSet {
        self.grammar_items.borrow().generate_goto_kernel(symbol)
    }

    pub fn resolve_shift_reduce_conflicts(&self) -> usize {
        // do this in two stages to avoid borrow/access conflicts
        let mut conflicts = vec![];
        for (shift_symbol, goto_state) in self.shift_list.borrow().iter() {
            for (item, look_ahead_set) in self.grammar_items.borrow().0.iter() {
                if item.is_reducible() && look_ahead_set.contains(shift_symbol) {
                    conflicts.push((
                        Rc::clone(shift_symbol),
                        Rc::clone(goto_state),
                        Rc::clone(item),
                        look_ahead_set.clone(),
                    ))
                }
            }
        }
        let mut shift_reduce_conflicts = self.shift_reduce_conflicts.borrow_mut();
        let mut shift_list = self.shift_list.borrow_mut();
        let mut grammar_items = self.grammar_items.borrow_mut();
        for (shift_symbol, goto_state, reducible_item, look_ahead_set) in conflicts.iter() {
            if shift_symbol.precedence() < reducible_item.precedence() {
                shift_list.remove(shift_symbol);
            } else if shift_symbol.precedence() > reducible_item.precedence() {
                grammar_items.0[Rc::clone(reducible_item)].remove(shift_symbol);
            } else if reducible_item.associativity() == Associativity::Left {
                shift_list.remove(shift_symbol);
            } else if reducible_item.has_error_recovery_tail() {
                grammar_items.0[Rc::clone(reducible_item)].remove(shift_symbol);
            } else {
                // Default: resolve in favour of shift but mark as unresolved
                // to give the user the option of accepting this resolution
                grammar_items.0[Rc::clone(reducible_item)].remove(shift_symbol);
                shift_reduce_conflicts.push((
                    Rc::clone(shift_symbol),
                    Rc::clone(goto_state),
                    Rc::clone(reducible_item),
                    look_ahead_set.clone(),
                ))
            }
        }
        shift_reduce_conflicts.len()
    }

    pub fn resolve_reduce_reduce_conflicts(&self) -> usize {
        let reducible_key_set = self.grammar_items.borrow().reducible_keys();
        if reducible_key_set.len() < 2 {
            return 0;
        }

        let mut reduce_reduce_conflicts = self.reduce_reduce_conflicts.borrow_mut();
        let reducible_key_set_2 = reducible_key_set.clone();
        for key_1 in reducible_key_set.iter() {
            for key_2 in reducible_key_set_2.iter().advance_past(key_1) {
                let intersection = self
                    .grammar_items
                    .borrow()
                    .look_ahead_intersection(key_1, key_2);
                if intersection.len() > 0 && key_1.predicate().is_none() {
                    if key_1.has_error_recovery_tail() {
                        self.grammar_items
                            .borrow_mut()
                            .remove_look_ahead_symbols(key_1, &intersection);
                    } else if key_2.has_error_recovery_tail() {
                        self.grammar_items
                            .borrow_mut()
                            .remove_look_ahead_symbols(key_2, &intersection);
                    } else {
                        // Default: resolve in favour of first declared production
                        // but mark unresolved to give the user some options
                        self.grammar_items
                            .borrow_mut()
                            .remove_look_ahead_symbols(key_2, &intersection);
                        reduce_reduce_conflicts
                            .push(((Rc::clone(key_1), Rc::clone(key_2)), intersection))
                    }
                }
            }
        }
        reduce_reduce_conflicts.len()
    }

    pub fn is_recovery_state_for_token(&self, token: &Rc<Symbol>) -> bool {
        if let Some(recovery_state) = self.error_recovery_state.borrow().clone() {
            if recovery_state
                .grammar_items
                .borrow()
                .error_recovery_look_ahead_set_contains(token)
            {
                return true;
            }
        };
        false
    }
}
