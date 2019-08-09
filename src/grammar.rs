use std::{
    collections::{HashMap, HashSet},
    io::{stderr, Write},
    rc::Rc,
};

use ordered_collections::{OrderedMap, OrderedSet};

use lexan;

use crate::symbols::{AssociativePrecedence, SpecialSymbols, Symbol, SymbolTable};

#[derive(Debug)]
pub struct Error {}

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
}

pub fn report_error(location: &lexan::Location, what: &str) {
    writeln!(stderr(), "{}: Error: {}.", location, what).expect("what?");
}

pub fn report_warning(location: &lexan::Location, what: &str) {
    writeln!(stderr(), "{}: Warning: {}.", location, what).expect("what?");
}

#[derive(Debug, Default, Clone)]
pub struct GrammarSpecification {
    pub symbol_table: SymbolTable,
    productions: Vec<Rc<Production>>,
    preamble: String,
    pub error_count: u32,
    pub warning_count: u32,
}

impl GrammarSpecification {
    pub fn new() -> Self {
        let symbol_table = SymbolTable::new();
        Self {
            symbol_table,
            productions: vec![],
            preamble: String::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn is_allowable_name(name: &str) -> bool {
        !(name.starts_with("aa") || name.starts_with("AA"))
    }

    pub fn error(&mut self, location: &lexan::Location, what: &str) {
        report_error(location, what);
        self.error_count += 1;
    }

    pub fn warning(&mut self, location: &lexan::Location, what: &str) {
        report_warning(location, what);
        self.warning_count += 1;
    }

    pub fn set_preamble(&mut self, preamble: &str) {
        self.preamble = preamble.to_string();
    }

    pub fn new_production(&mut self, left_hand_side: Rc<Symbol>, tail: ProductionTail) {
        if self.productions.len() == 0 {
            let start_symbol = self.symbol_table.special_symbol(&SpecialSymbols::Start);
            let start_tail =
                ProductionTail::new(vec![Rc::clone(&left_hand_side)], None, None, None);
            let start_production = Production::new(0, start_symbol, start_tail);
            self.productions.push(Rc::new(start_production));
        }
        let ident = self.productions.len() as u32;
        self.productions
            .push(Rc::new(Production::new(ident, left_hand_side, tail)));
    }

    fn first_allcaps(&self, symbol_string: &[Rc<Symbol>], token: &Rc<Symbol>) -> OrderedSet<Rc<Symbol>> {
        let token_set: OrderedSet<Rc<Symbol>> = OrderedSet::new();
        token_set
    }

    pub fn closure(&self, mut closure_set: GrammarItemSet) -> GrammarItemSet {
        loop {
            let additions = 0;
            for (item_key, look_ahead_set) in closure_set.0.iter().filter(|x| x.0.is_closable()) {
                let prospective_lhs = item_key.next_symbol().expect("it's closable");
                for look_ahead_symbol in look_ahead_set.iter() {
                    let firsts = self.first_allcaps(item_key.rhs_tail(), look_ahead_symbol);
                }
            }
            if additions == 0 {
                break;
            }
        }
        closure_set
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
struct GrammarItemKey {
    production: Rc<Production>,
    dot: usize,
}

impl GrammarItemKey {
    fn new(production: Rc<Production>) -> Rc<Self> {
        Rc::new(Self { production, dot: 0 })
    }

    fn shifted(&self) -> Rc<Self> {
        let production = Rc::clone(&self.production);
        let dot = self.dot + 1;
        Rc::new(Self { production, dot })
    }

    fn is_closable(&self) -> bool {
        if let Some(symbol) = self.production.tail.right_hand_side.get(self.dot) {
            symbol.is_non_terminal()
        } else {
            false
        }
    }

    fn next_symbol(&self) -> Option<&Rc<Symbol>> {
        self.production.tail.right_hand_side.get(self.dot)
    }

    fn rhs_tail(&self) -> &[Rc<Symbol>] {
        &self.production.tail.right_hand_side[self.dot + 1..]
    }
}

struct GrammarItemSet(OrderedMap<Rc<GrammarItemKey>, OrderedSet<Rc<Symbol>>>);

struct ParserState {
    ident: u32,
    grammar_items: GrammarItemSet,
    shift_list: HashMap<Rc<Symbol>, Rc<ParserState>>,
    goto_table: HashMap<Rc<Symbol>, Rc<ParserState>>,
    error_recovery_state: Option<Rc<ParserState>>,
}

pub struct Grammar {
    specification: GrammarSpecification,
    parser_states: Vec<ParserState>,
    goto_table: OrderedMap<Rc<Symbol>, HashMap<ParserState, HashSet<ParserState>>>,
    empty_look_ahead_sets: Vec<u32>,
    unresolved_sr_conflicts: usize,
    unresolved_rr_conflicts: usize,
}

impl Grammar {
    pub fn new(specification: GrammarSpecification) -> Result<Self, Error> {
        let mut grammar = Self {
            specification,
            parser_states: vec![],
            goto_table: OrderedMap::new(),
            empty_look_ahead_sets: vec![],
            unresolved_rr_conflicts: 0,
            unresolved_sr_conflicts: 0,
        };
        let start_item_key = GrammarItemKey::new(Rc::clone(&grammar.specification.productions[0]));
        let mut start_look_ahead_set: OrderedSet<Rc<Symbol>> = OrderedSet::new();
        start_look_ahead_set.insert(
            grammar
                .specification
                .symbol_table
                .special_symbol(&SpecialSymbols::End),
        );
        let mut kernel: OrderedMap<Rc<GrammarItemKey>, OrderedSet<Rc<Symbol>>> = OrderedMap::new();
        kernel.insert(start_item_key, start_look_ahead_set);

        Ok(grammar)
    }
}
