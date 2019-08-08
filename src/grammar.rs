use std::{
    io::{stderr, Write},
    rc::Rc,
};

use lexan;

use crate::symbols::{
    self, AssociativePrecedence, Associativity, SpecialSymbols, Symbol, SymbolTable,
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

    pub fn is_known_tag(&self, name: &str) -> bool {
        self.symbol_table.is_known_tag(name)
    }

    pub fn is_known_token(&self, name: &str) -> bool {
        self.symbol_table.is_known_token(name)
    }

    pub fn is_known_non_terminal(&self, name: &str) -> bool {
        self.symbol_table.is_known_non_terminal(name)
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

    pub fn new_token(
        &mut self,
        name: &str,
        pattern: &str,
        location: &lexan::Location,
    ) -> Result<(), symbols::Error> {
        self.symbol_table.new_token(name, pattern, location)
    }

    pub fn new_tag(
        &mut self,
        name: &str,
        location: &lexan::Location,
    ) -> Result<(), symbols::Error> {
        self.symbol_table.new_tag(name, location)
    }

    pub fn new_production(&mut self, left_hand_side: Rc<Symbol>, tail: ProductionTail) {
        let ident = self.productions.len() as u32;
        if ident == 1 {
            self.productions[0].tail.right_hand_side = vec![Rc::clone(&left_hand_side)];
        }
        self.productions
            .push(Production::new(ident, left_hand_side, tail));
    }

    pub fn add_skip_rule(&mut self, rule: &str) {
        self.symbol_table.add_skip_rule(rule);
    }

    pub fn set_precedences(
        &mut self,
        associativity: Associativity,
        tags: &Vec<Rc<symbols::Symbol>>,
    ) {
        self.symbol_table.set_precedences(associativity, tags);
    }

    pub fn get_literal_token(&self, text: &str, location: &lexan::Location) -> Option<&Rc<Symbol>> {
        self.symbol_table.get_literal_token(text, location)
    }

    pub fn get_token(&self, name: &str, location: &lexan::Location) -> Option<&Rc<Symbol>> {
        self.symbol_table.get_token(name, location)
    }

    pub fn declaration_location(&self, symbol_name: &str) -> Option<lexan::Location> {
        self.symbol_table.declaration_location(symbol_name)
    }

    pub fn special_symbol(&self, t: &SpecialSymbols) -> &Rc<Symbol> {
        self.symbol_table.special_symbol(t)
    }

    pub fn define_non_terminal(&mut self, name: &str, location: &lexan::Location) -> &Rc<Symbol> {
        self.symbol_table.define_non_terminal(name, location)
    }
    pub fn use_symbol_named(&mut self, symbol_name: &str, location: &lexan::Location) -> Option<&Rc<Symbol>> {
        self.symbol_table.use_symbol_named(symbol_name, location)
    }
}
