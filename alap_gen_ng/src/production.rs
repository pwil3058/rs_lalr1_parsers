// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::symbol::{non_terminal::NonTerminal, terminal::TokenSet, Associativity, Symbol};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct ProductionTailData {
    right_hand_side: Vec<Symbol>,
    predicate: Option<String>,
    associativity: Associativity,
    precedence: u16,
    action: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct ProductionTail(Rc<ProductionTailData>);

impl ProductionTail {
    pub fn new(
        right_hand_side: &[Symbol],
        o_predicate: Option<&str>,
        associative_precedence: Option<(Associativity, u16)>,
        o_action: Option<&str>,
    ) -> Self {
        let predicate = if let Some(predicate) = o_predicate {
            Some(predicate.to_string())
        } else {
            None
        };
        let action = if let Some(action) = o_action {
            Some(action.to_string())
        } else {
            None
        };
        let (associativity, precedence) = if let Some(tuple) = associative_precedence {
            tuple
        } else if let Some(tuple) = rhs_associated_precedence(&right_hand_side) {
            tuple
        } else {
            (Associativity::default(), 0)
        };
        Self(Rc::new(ProductionTailData {
            right_hand_side: right_hand_side.to_vec(),
            predicate,
            action,
            associativity,
            precedence,
        }))
    }
}

fn rhs_associated_precedence(symbols: &[Symbol]) -> Option<(Associativity, u16)> {
    for symbol in symbols.iter() {
        match symbol {
            Symbol::Terminal(token) => {
                return Some(token.associativity_and_precedence());
            }
            _ => (),
        }
    }
    None
}

#[derive(Debug)]
pub struct ProductionData {
    pub ident: u32,
    left_hand_side: NonTerminal,
    tail: ProductionTail,
}

#[derive(Debug, Clone)]
pub struct Production(Rc<ProductionData>);

impl Production {
    pub fn new(ident: u32, left_hand_side: NonTerminal, tail: ProductionTail) -> Self {
        Self(Rc::new(ProductionData {
            ident,
            left_hand_side,
            tail,
        }))
    }

    pub fn len(&self) -> usize {
        self.0.tail.0.right_hand_side.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn left_hand_side(&self) -> &NonTerminal {
        &self.0.left_hand_side
    }

    pub fn associativity(&self) -> Associativity {
        self.0.tail.0.associativity
    }

    pub fn precedence(&self) -> u16 {
        self.0.tail.0.precedence
    }

    pub fn has_error_recovery_tail(&self) -> bool {
        if let Some(symbol) = self.0.tail.0.right_hand_side.last() {
            match symbol {
                Symbol::Terminal(_) => false,
                Symbol::NonTerminal(non_terminal) => non_terminal.is_error(),
            }
        } else {
            false
        }
    }
}

impl PartialEq for Production {
    fn eq(&self, other: &Self) -> bool {
        self.0.ident == other.0.ident
    }
}

impl Eq for Production {}

impl PartialOrd for Production {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.ident.partial_cmp(&other.0.ident)
    }
}

impl Ord for Production {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug, Default)]
struct Reductions {
    reductions: BTreeMap<BTreeSet<Production>, TokenSet>,
    expected_tokens: TokenSet,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub struct GrammarItemKey {
    production: Production,
    dot: usize,
}

impl From<&Production> for GrammarItemKey {
    fn from(production: &Production) -> Self {
        Self {
            production: production.clone(),
            dot: 0,
        }
    }
}

impl GrammarItemKey {
    pub fn shifted(&self) -> Self {
        debug_assert!(self.dot < self.production.len());
        let dot = self.dot + 1;
        Self {
            production: self.production.clone(),
            dot,
        }
    }

    pub fn is_closable(&self) -> bool {
        if let Some(symbol) = self.production.0.tail.0.right_hand_side.get(self.dot) {
            symbol.is_non_terminal()
        } else {
            false
        }
    }

    pub fn is_kernel_item(&self) -> bool {
        self.dot > 0 || self.production.0.left_hand_side.is_start()
    }

    pub fn is_reducible(&self) -> bool {
        self.dot >= self.production.0.tail.0.right_hand_side.len()
    }

    pub fn next_symbol(&self) -> Option<&Symbol> {
        self.production.0.tail.0.right_hand_side.get(self.dot)
    }

    pub fn next_symbol_is(&self, symbol: &Symbol) -> bool {
        if let Some(next_symbol) = self.next_symbol() {
            next_symbol == symbol
        } else {
            false
        }
    }

    pub fn rhs_tail(&self) -> &[Symbol] {
        &self.production.0.tail.0.right_hand_side[self.dot + 1..]
    }

    pub fn associativity(&self) -> Associativity {
        self.production.associativity()
    }

    pub fn precedence(&self) -> u16 {
        self.production.precedence()
    }

    pub fn has_no_predicate(&self) -> bool {
        self.production.0.tail.0.predicate.is_none()
    }

    pub fn has_error_recovery_tail(&self) -> bool {
        self.production.has_error_recovery_tail()
    }
}

#[derive(Debug, Default)]
pub struct GrammarItemSet(BTreeMap<GrammarItemKey, TokenSet>);

impl From<BTreeMap<GrammarItemKey, TokenSet>> for GrammarItemSet {
    fn from(map: BTreeMap<GrammarItemKey, TokenSet>) -> Self {
        Self(map)
    }
}

impl GrammarItemSet {
    pub fn iter(&self) -> impl Iterator<Item = (&GrammarItemKey, &TokenSet)> {
        self.0.iter()
    }

    pub fn closable_set(&self) -> Vec<(GrammarItemKey, TokenSet)> {
        let mut closables = vec![];
        for (key, set) in self.0.iter().filter(|x| x.0.is_closable()) {
            closables.push((key.clone(), set.clone()));
        }
        closables
    }

    pub fn generate_goto_kernel(&self, symbol: &Symbol) -> GrammarItemSet {
        // TODO: be more itery
        let mut map = BTreeMap::new();
        for (item_key, look_ahead_set) in self.0.iter() {
            if item_key.next_symbol_is(symbol) {
                map.insert(item_key.shifted(), look_ahead_set.clone());
            }
        }
        GrammarItemSet(map)
    }

    pub fn kernel_key_set(&self) -> BTreeSet<GrammarItemKey> {
        // TODO: be more itery
        let mut keys = BTreeSet::new();
        for key in self.0.keys().filter(|x| x.is_kernel_item()) {
            keys.insert(key.clone());
        }
        keys
    }

    pub fn irreducible_key_set(&self) -> BTreeSet<GrammarItemKey> {
        self.0
            .keys()
            .filter(|x| !x.is_reducible())
            .cloned()
            .collect()
    }

    pub fn reducible_key_set(&self) -> BTreeSet<GrammarItemKey> {
        self.0
            .keys()
            .filter(|x| x.is_reducible())
            .cloned()
            .collect()
    }

    pub fn get_mut(&mut self, key: &GrammarItemKey) -> Option<&mut TokenSet> {
        self.0.get_mut(key)
    }

    pub fn insert(&mut self, key: GrammarItemKey, look_ahead_set: TokenSet) -> Option<TokenSet> {
        self.0.insert(key, look_ahead_set)
    }

    pub fn look_ahead_intersection(
        &self,
        key1: &GrammarItemKey,
        key2: &GrammarItemKey,
    ) -> TokenSet {
        self.0
            .get(key1)
            .unwrap()
            .intersection(self.0.get(key2).unwrap())
            .cloned()
            .collect()
    }

    pub fn remove_look_ahead_symbols(&mut self, key: &GrammarItemKey, symbols: &TokenSet) {
        let look_ahead_set = self.0.get_mut(key).unwrap();
        *look_ahead_set = look_ahead_set.difference(symbols).cloned().collect();
    }
}
