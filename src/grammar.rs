use std::{
    cell::{Cell, RefCell},
    io::{stderr, Write},
    rc::Rc,
};

use ordered_collections::{ordered_set::ord_set_iterators::ToSet, OrderedMap, OrderedSet};

use lexan;

use crate::state::{GrammarItemKey, GrammarItemSet, ParserState, Production, ProductionTail};
use crate::symbols::{AssociativePrecedence, FirstsData, SpecialSymbols, Symbol, SymbolTable};

#[derive(Debug)]
pub struct Error {}

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

    fn first_allcaps(
        &self,
        symbol_string: &[Rc<Symbol>],
        token: &Rc<Symbol>,
    ) -> OrderedSet<Rc<Symbol>> {
        let mut token_set: OrderedSet<Rc<Symbol>> = OrderedSet::new();
        for symbol in symbol_string.iter() {
            let firsts_data = self.firsts_data(symbol);
            token_set = token_set.union(&firsts_data.token_set).to_set();
            if !firsts_data.transparent {
                return token_set;
            }
        }
        token_set.insert(Rc::clone(token));
        token_set
    }

    fn firsts_data(&self, symbol: &Rc<Symbol>) -> FirstsData {
        if symbol.mutable_data.borrow().firsts_data.is_none() {
            self.set_firsts_data(symbol);
        };
        symbol.mutable_data.borrow().firsts_data.clone().unwrap()
    }

    fn set_firsts_data(&self, symbol: &Rc<Symbol>) {
        assert!(symbol.mutable_data.borrow().firsts_data.is_none());
        if !symbol.is_non_terminal() {
            let set: OrderedSet<Rc<Symbol>> = vec![Rc::clone(symbol)].into();
            symbol.set_firsts_data(FirstsData::new(set, false));
        } else {
            let relevant_productions: Vec<&Rc<Production>> = self
                .productions
                .iter()
                .filter(|x| x.left_hand_side() == symbol)
                .collect();
            let mut transparent = relevant_productions.iter().any(|x| x.is_empty());
            let mut token_set = OrderedSet::<Rc<Symbol>>::new();
            let mut transparency_changed = true;
            while transparency_changed {
                transparency_changed = false;
                for production in relevant_productions.iter() {
                    let mut transparent_production = true;
                    for rhs_symbol in production.right_hand_side_symbols() {
                        if rhs_symbol == symbol {
                            if transparent {
                                continue;
                            } else {
                                transparent_production = false;
                                break;
                            }
                        }
                        if rhs_symbol.mutable_data.borrow().firsts_data.is_none() {
                            self.set_firsts_data(rhs_symbol)
                        }
                        let firsts_data = rhs_symbol
                            .mutable_data
                            .borrow()
                            .firsts_data
                            .clone()
                            .unwrap();
                        token_set = token_set.union(&firsts_data.token_set).to_set();
                        if !firsts_data.transparent {
                            transparent_production = false;
                            break;
                        }
                    }
                    if transparent_production {
                        transparency_changed = !transparent;
                        transparent = true;
                    }
                }
            }
            let firsts_data = FirstsData::new(token_set, transparent);
            symbol.set_firsts_data(firsts_data);
        }
    }

    fn closure(&self, mut closure_set: GrammarItemSet) -> GrammarItemSet {
        loop {
            let mut additions = 0;
            for (item_key, look_ahead_set) in closure_set.closables() {
                let prospective_lhs = item_key.next_symbol().expect("it's closable");
                for look_ahead_symbol in look_ahead_set.iter() {
                    let firsts = self.first_allcaps(item_key.rhs_tail(), look_ahead_symbol);
                    for production in self
                        .productions
                        .iter()
                        .filter(|x| x.left_hand_side() == prospective_lhs)
                    {
                        let prospective_key = GrammarItemKey::new(Rc::clone(production));
                        if let Some(set) = closure_set.get_mut(&prospective_key) {
                            if !set.is_superset(&firsts) {
                                additions += 1;
                                *set = set.union(&firsts).to_set();
                            }
                        } else {
                            additions += 1;
                            closure_set.insert(prospective_key, firsts.clone());
                        }
                    }
                }
            }
            if additions == 0 {
                break;
            }
        }
        closure_set
    }
}

pub struct Grammar {
    specification: GrammarSpecification,
    parser_states: OrderedSet<Rc<ParserState>>,
    goto_table: OrderedMap<Rc<Symbol>, OrderedMap<ParserState, OrderedSet<ParserState>>>,
    empty_look_ahead_sets: Vec<u32>,
    unresolved_sr_conflicts: usize,
    unresolved_rr_conflicts: usize,
}

impl Grammar {
    pub fn new(specification: GrammarSpecification) -> Result<Self, Error> {
        let mut grammar = Self {
            specification,
            parser_states: OrderedSet::new(),
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
        let mut map: OrderedMap<Rc<GrammarItemKey>, OrderedSet<Rc<Symbol>>> = OrderedMap::new();
        map.insert(start_item_key, start_look_ahead_set);
        let start_kernel = grammar.specification.closure(GrammarItemSet::new(map));
        let start_state = grammar.new_parser_state(start_kernel);

        loop {
            let unprocessed_state = if let Some(state) = grammar
                .parser_states
                .iter()
                .filter(|x| !x.is_processed())
                .next()
            {
                Rc::clone(state)
            } else {
                break;
            };
            let first_time = unprocessed_state.is_unprocessed();
            unprocessed_state.mark_as_processed();
            let mut already_done: OrderedSet<Rc<Symbol>> = OrderedSet::new();
            for item_key in unprocessed_state.non_kernel_keys().iter() {
                let symbol_x = item_key.next_symbol().expect("not reducible");
                if !already_done.insert(Rc::clone(symbol_x)) {
                    continue;
                };
                let kernel_x = unprocessed_state.generate_goto_kernel(&symbol_x);
                let item_set_x = grammar.specification.closure(kernel_x);
                let goto_state =
                    if let Some(equivalent_state) = grammar.equivalent_state(&item_set_x) {
                        equivalent_state.merge_lookahead_sets(&item_set_x);
                        Rc::clone(equivalent_state)
                    } else {
                        grammar.new_parser_state(item_set_x)
                    };
                if first_time {
                    if symbol_x.is_syntax_error() {
                        unprocessed_state.set_error_recovery_state(&goto_state)
                    };
                    if symbol_x.is_token() {
                        unprocessed_state.add_shift_action(Rc::clone(symbol_x), goto_state);
                    } else {
                        unprocessed_state.add_goto(Rc::clone(symbol_x), goto_state);
                    };
                }
            }
        }
        panic!("conflicts not resolved");

        Ok(grammar)
    }

    fn new_parser_state(&mut self, grammar_items: GrammarItemSet) -> Rc<ParserState> {
        let ident = self.parser_states.len() as u32;
        let parser_state = ParserState::new(ident, grammar_items);
        self.parser_states.insert(Rc::clone(&parser_state));
        parser_state
    }

    fn equivalent_state(&self, item_set: &GrammarItemSet) -> Option<&Rc<ParserState>> {
        let target_keys = item_set.kernel_keys();
        for parser_state in self.parser_states.iter() {
            if target_keys == parser_state.kernel_keys() {
                return Some(parser_state);
            }
        }
        None
    }
}
