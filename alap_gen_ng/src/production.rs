// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::symbol::{non_terminal::NonTerminal, Associativity, Symbol};
use std::cmp::Ordering;
use std::rc::Rc;

#[derive(Debug, Clone, Default)]
pub struct ProductionTail {
    right_hand_side: Vec<Symbol>,
    predicate: Option<String>,
    associativity: Associativity,
    precedence: u16,
    action: Option<String>,
}

impl ProductionTail {
    pub fn new(
        right_hand_side: Vec<Symbol>,
        predicate: Option<String>,
        associative_precedence: Option<(Associativity, u16)>,
        action: Option<String>,
    ) -> Self {
        let (associativity, precedence) = if let Some(tuple) = associative_precedence {
            tuple
        } else if let Some(tuple) = rhs_associated_precedence(&right_hand_side) {
            tuple
        } else {
            (Associativity::default(), 0)
        };
        Self {
            right_hand_side,
            predicate,
            action,
            associativity,
            precedence,
        }
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
