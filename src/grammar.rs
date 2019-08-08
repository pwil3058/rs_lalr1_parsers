use std::{
    io::{stderr, Write},
    rc::Rc,
};

use lexan;

use crate::symbols::{
    AssociativePrecedence, SpecialSymbols, Symbol, SymbolTable,
};

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
}

#[derive(Debug, Default, Clone)]
pub struct ParserSpecification {
    pub symbol_table: SymbolTable,
    productions: Vec<Production>,
    preamble: String,
    error_count: u32,
    warning_count: u32,
}

impl ParserSpecification {
    pub fn new() -> Self {
        let symbol_table = SymbolTable::new();
        let start_symbol = symbol_table.special_symbol(&SpecialSymbols::Start);
        let production = Production::new(0, Rc::clone(start_symbol), ProductionTail::default());
        Self {
            symbol_table,
            productions: vec![production],
            preamble: String::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn is_allowable_name(name: &str) -> bool {
        !(name.starts_with("aa") || name.starts_with("AA"))
    }

    pub fn error(&mut self, location: &lexan::Location, what: &str) {
        writeln!(stderr(), "{}:Error: {}.", location, what).expect("what?");
        self.error_count += 1;
    }

    pub fn warning(&mut self, location: &lexan::Location, what: &str) {
        writeln!(stderr(), "{}:Warning: {}.", location, what).expect("what?");
        self.warning_count += 1;
    }

    pub fn set_preamble(&mut self, preamble: &str) {
        self.preamble = preamble.to_string();
    }

    pub fn new_production(&mut self, left_hand_side: Rc<Symbol>, tail: ProductionTail) {
        let ident = self.productions.len() as u32;
        if ident == 1 {
            self.productions[0].tail.right_hand_side = vec![Rc::clone(&left_hand_side)];
        }
        self.productions
            .push(Production::new(ident, left_hand_side, tail));
    }
}
