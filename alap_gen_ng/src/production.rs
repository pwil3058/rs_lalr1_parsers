// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::symbol::terminal::Token;
use crate::symbol::{non_terminal::NonTerminal, terminal::TokenSet, Associativity, Symbol};
use lazy_static::lazy_static;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Index;
use std::rc::Rc;
use std::str::FromStr;

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

lazy_static! {
    static ref RHS_CRE: regex::Regex = regex::Regex::new(r"\$(\d+)").unwrap();
}

impl Production {
    pub fn new(ident: u32, left_hand_side: NonTerminal, tail: ProductionTail) -> Self {
        Self(Rc::new(ProductionData {
            ident,
            left_hand_side,
            tail,
        }))
    }

    pub fn ident(&self) -> u32 {
        self.0.ident
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

    pub fn right_hand_side_symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.0.tail.0.right_hand_side.iter()
    }

    pub fn associativity(&self) -> Associativity {
        self.0.tail.0.associativity
    }

    pub fn precedence(&self) -> u16 {
        self.0.tail.0.precedence
    }

    pub fn has_predicate(&self) -> bool {
        self.0.tail.0.predicate.is_some()
    }

    pub fn expanded_predicate(&self) -> Option<String> {
        if let Some(predicate) = &self.0.tail.0.predicate {
            let rhs_len = self.0.tail.0.right_hand_side.len();
            let string = RHS_CRE
                .replace_all(&predicate, |caps: &regex::Captures| {
                    format!(
                        "aa_attributes.at_len_minus_n({})",
                        rhs_len + 1 - usize::from_str(&caps[1]).unwrap()
                    )
                })
                .to_string();
            let string = string.replace("$?", "aa_tag");
            Some(string)
        } else {
            None
        }
    }

    pub fn expanded_action(&self) -> Option<String> {
        // TODO: move action expansion to RHS creation
        if let Some(action) = &self.0.tail.0.action {
            let string = action.replace("$$", "aa_lhs");
            let string = string.replace("$INJECT", "aa_inject");
            let string = RHS_CRE
                .replace_all(&string, |caps: &regex::Captures| {
                    format!("aa_rhs[{}]", usize::from_str(&caps[1]).unwrap() - 1)
                })
                .to_string();
            Some(string)
        } else {
            None
        }
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

impl std::fmt::Display for Production {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string = format!("{}:", self.left_hand_side().name());
        if self.0.tail.0.right_hand_side.len() == 0 {
            string += " <empty>";
        } else {
            for symbol in self.0.tail.0.right_hand_side.iter() {
                string += &format!(" {}", symbol);
            }
        };
        if let Some(predicate) = &self.0.tail.0.predicate {
            string += &format!(" ?({}?)", predicate);
        };
        write!(f, "{}", string)
    }
}

#[derive(Debug, Default)]
pub struct Reductions {
    reductions: BTreeMap<BTreeSet<Production>, TokenSet>,
    expected_tokens: TokenSet,
}

impl Reductions {
    pub fn reductions(&self) -> impl Iterator<Item = (&BTreeSet<Production>, &TokenSet)> {
        self.reductions.iter()
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub struct GrammarItemKey {
    production: Production,
    dot: usize,
}

impl std::fmt::Display for GrammarItemKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string = format!("{}:", self.production.0.left_hand_side.name());
        if self.production.0.tail.0.right_hand_side.len() == 0 {
            string += " . <empty>";
        } else {
            for (index, symbol) in self.production.0.tail.0.right_hand_side.iter().enumerate() {
                if index == self.dot {
                    string += &format!(" . {}", symbol);
                } else {
                    string += &format!(" {}", symbol);
                }
            }
            if self.dot >= self.production.0.tail.0.right_hand_side.len() {
                string += " . ";
            }
        };
        if let Some(predicate) = &self.production.0.tail.0.predicate {
            string += &format!(" ?({}?)", predicate);
        };
        write!(f, "{}", string)
    }
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
    pub fn production(&self) -> &Production {
        &self.production
    }

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

    pub fn has_reducible_error_recovery_tail(&self) -> bool {
        self.is_reducible() && self.production.has_error_recovery_tail()
    }
}

#[derive(Debug, Default)]
pub struct GrammarItemSet(BTreeMap<GrammarItemKey, TokenSet>);

impl From<BTreeMap<GrammarItemKey, TokenSet>> for GrammarItemSet {
    fn from(map: BTreeMap<GrammarItemKey, TokenSet>) -> Self {
        Self(map)
    }
}

impl Index<&GrammarItemKey> for GrammarItemSet {
    type Output = TokenSet;

    fn index(&self, key: &GrammarItemKey) -> &TokenSet {
        self.0.index(key)
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

    pub fn error_recovery_look_ahead_set_contains(&self, token: &Token) -> bool {
        for look_ahead_set in self
            .0
            .iter()
            .filter(|x| x.0.has_reducible_error_recovery_tail())
            .map(|x| x.1)
        {
            if look_ahead_set.contains(token) {
                return true;
            }
        }
        false
    }

    pub fn reducible_look_ahead_set(&self) -> TokenSet {
        let mut set = TokenSet::new();
        for (_, look_ahead_set) in self.0.iter().filter(|x| x.0.is_reducible()) {
            set |= look_ahead_set;
        }
        set
    }

    pub fn reductions(&self) -> Reductions {
        let expected_tokens = self.reducible_look_ahead_set();
        let mut reductions: BTreeMap<BTreeSet<Production>, TokenSet> = BTreeMap::new();
        for token in expected_tokens.iter() {
            let mut productions: BTreeSet<Production> = BTreeSet::new();
            for (item_key, look_ahead_set) in self.0.iter().filter(|x| x.0.is_reducible()) {
                if look_ahead_set.contains(token) {
                    productions.insert(item_key.production.clone());
                }
            }
            let look_ahead_set = reductions.entry(productions).or_insert(TokenSet::new());
            look_ahead_set.insert(token);
        }
        Reductions {
            reductions,
            expected_tokens,
        }
    }
}
