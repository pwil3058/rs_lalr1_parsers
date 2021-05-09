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

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
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
}

#[derive(Debug, Default)]
pub struct GrammarItemSet(BTreeMap<GrammarItemKey, TokenSet>);
