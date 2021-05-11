use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
    io::{self, stderr, Write},
    path::Path,
    rc::Rc,
};

use lalr1_plus::{self, Parser};
use lexan;

use crate::production::{GrammarItemKey, GrammarItemSet, Production, ProductionTail};
use crate::state::ParserState;
use crate::symbols::{
    format_as_macro_call, SetFirstsData, Symbol, SymbolSet, SymbolTable, SymbolType,
};

#[cfg(not(feature = "bootstrap"))]
use crate::alapgen::*;
#[cfg(feature = "bootstrap")]
use crate::bootstrap::*;

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
    pub attribute_type: String,
    pub target_type: String,
    pub error_count: u32,
    pub warning_count: u32,
}

impl lalr1_plus::ReportError<AATerminal> for GrammarSpecification {}

impl GrammarSpecification {
    pub fn new(text: String, label: String) -> Result<Self, lalr1_plus::Error<AATerminal>> {
        let symbol_table = SymbolTable::new();
        let mut spec = Self {
            symbol_table,
            productions: vec![],
            preamble: String::new(),
            attribute_type: "AttributeData".to_string(),
            target_type: "GrammarSpecification".to_string(),
            error_count: 0,
            warning_count: 0,
        };
        spec.parse_text(text, label)?;
        let location = lexan::Location::default();
        // Add dummy error production last so that it has lowest precedence during conflict resolution
        let symbol = spec
            .symbol_table
            .use_symbol_named(&AANonTerminal::AAError.to_string(), &location)
            .unwrap();
        let ident = spec.productions.len() as u32;
        let tail = ProductionTail::default();
        spec.productions
            .push(Rc::new(Production::new(ident, symbol, tail)));
        for symbol in spec.symbol_table.non_terminal_symbols() {
            if symbol.firsts_data_is_none() {
                symbol.set_firsts_data(&spec.productions)
            }
        }
        Ok(spec)
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

    pub fn write_preamble_text<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write(self.preamble.as_bytes())?;
        wtr.write(b"\n")?;
        Ok(())
    }

    pub fn new_production(&mut self, left_hand_side: Rc<Symbol>, tail: ProductionTail) {
        if self.productions.len() == 0 {
            let location = left_hand_side.defined_at().expect("should be defined");
            left_hand_side.add_used_at(&location);
            let start_symbol = self
                .symbol_table
                .use_symbol_named(&AANonTerminal::AAStart.to_string(), &location)
                .unwrap();
            let start_tail =
                ProductionTail::new(vec![Rc::clone(&left_hand_side)], None, None, None);
            let start_production = Production::new(0, start_symbol, start_tail);
            self.productions.push(Rc::new(start_production));
        }
        let ident = self.productions.len() as u32;
        self.productions
            .push(Rc::new(Production::new(ident, left_hand_side, tail)));
    }

    fn closure(&self, mut closure_set: GrammarItemSet) -> GrammarItemSet {
        let mut additions_made = true;
        while additions_made {
            additions_made = false;
            // Closables extraction as a new separate map necessary to avoid borrow conflict
            for (item_key, look_ahead_set) in closure_set.closables() {
                let prospective_lhs = item_key.next_symbol().expect("it's closable");
                for look_ahead_symbol in look_ahead_set.iter() {
                    let firsts = SymbolSet::first_all_caps(item_key.rhs_tail(), look_ahead_symbol);
                    for production in self
                        .productions
                        .iter()
                        .filter(|x| x.left_hand_side() == prospective_lhs)
                    {
                        let prospective_key = GrammarItemKey::new(Rc::clone(production));
                        if let Some(set) = closure_set.get_mut(&prospective_key) {
                            let len = set.len();
                            *set |= &firsts;
                            additions_made = additions_made || set.len() > len;
                        } else {
                            closure_set.insert(prospective_key, firsts.clone());
                            additions_made = true;
                        }
                    }
                }
            }
        }
        closure_set
    }

    pub fn write_production_data_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write(b"    fn production_data(production_id: u32) -> (AANonTerminal, usize) {\n")?;
        wtr.write(b"        match production_id {\n")?;
        for production in self.productions.iter() {
            wtr.write_fmt(format_args!(
                "            {} => (AANonTerminal::{}, {}),\n",
                production.ident,
                production.left_hand_side(),
                production.right_hand_side_len(),
            ))?;
        }
        wtr.write(b"            _ => panic!(\"malformed production data table\"),\n")?;
        wtr.write(b"        }\n")?;
        wtr.write(b"    }\n\n")?;
        Ok(())
    }

    pub fn write_semantic_action_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write(b"    fn do_semantic_action<F: FnMut(String, String)>(\n")?;
        wtr.write(b"        &mut self,\n")?;
        wtr.write(b"        aa_production_id: u32,\n")?;
        wtr.write_fmt(format_args!(
            "        aa_rhs: Vec<{}>,\n",
            self.attribute_type
        ))?;
        wtr.write(b"        mut aa_inject: F,\n")?;
        wtr.write_fmt(format_args!("    ) -> {} {{\n", self.attribute_type))?;
        wtr.write(b"        let mut aa_lhs = if let Some(a) = aa_rhs.first() {\n")?;
        wtr.write(b"            a.clone()\n")?;
        wtr.write(b"        } else {\n")?;
        wtr.write_fmt(format_args!(
            "           {}::default()\n",
            self.attribute_type
        ))?;
        wtr.write(b"        };\n")?;
        wtr.write(b"        match aa_production_id {\n")?;
        for production in self.productions.iter() {
            if let Some(action_code) = production.expanded_action() {
                wtr.write_fmt(format_args!("            {} => {{\n", production.ident))?;
                wtr.write_fmt(format_args!("                // {}\n", production))?;
                wtr.write_fmt(format_args!("                {}\n", action_code))?;
                wtr.write(b"            }\n")?;
            }
        }
        wtr.write(b"            _ => aa_inject(String::new(), String::new()),\n")?;
        wtr.write(b"        };\n")?;
        wtr.write(b"        aa_lhs\n")?;
        wtr.write(b"    }\n\n")?;
        Ok(())
    }
}

pub struct Grammar {
    specification: GrammarSpecification,
    parser_states: Vec<Rc<ParserState>>,
    unresolved_sr_conflicts: usize,
    unresolved_rr_conflicts: usize,
}

#[derive(Debug)]
pub enum Error {
    TooManyErrors(u32),
    UndefinedSymbols(u32),
}

impl TryFrom<GrammarSpecification> for Grammar {
    type Error = Error;

    fn try_from(mut specification: GrammarSpecification) -> Result<Self, Error> {
        for symbol in specification.symbol_table.unused_symbols() {
            let location = symbol.defined_at().unwrap();
            report_warning(
                &location,
                &format!("Symbol \"{}\" is not used", symbol.name()),
            );
        }

        let mut undefined_symbols = 0;
        for symbol in specification.symbol_table.undefined_symbols() {
            for location in symbol.used_at() {
                report_error(
                    &location,
                    &format!("Symbol \"{}\" is not defined", symbol.name()),
                );
            }
            undefined_symbols += 1;
        }

        if undefined_symbols > 0 {
            Err(Error::UndefinedSymbols(undefined_symbols))
        } else if specification.error_count > 0 {
            Err(Error::TooManyErrors(specification.error_count))
        } else {
            let start_item_key = Rc::new(GrammarItemKey::from(&specification.productions[0]));
            let end_symbol = specification
                .symbol_table
                .use_symbol_named(&AATerminal::AAEnd.to_string(), &lexan::Location::default())
                .unwrap();
            let mut start_look_ahead_set = SymbolSet::new();
            start_look_ahead_set.insert(&end_symbol);
            let mut map = BTreeMap::<Rc<GrammarItemKey>, SymbolSet>::new();
            map.insert(start_item_key, start_look_ahead_set);
            let start_kernel = specification.closure(GrammarItemSet::from(map));
            let mut grammar = Self {
                specification,
                parser_states: vec![],
                unresolved_rr_conflicts: 0,
                unresolved_sr_conflicts: 0,
            };
            grammar.new_parser_state(start_kernel);
            while let Some(unprocessed_state) = grammar.first_unprocessed_state() {
                let first_time = unprocessed_state.is_unprocessed();
                unprocessed_state.mark_as_processed();
                let mut already_done = BTreeSet::<Rc<Symbol>>::new();
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
                        if symbol_x.is_error_symbol() {
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
            grammar.resolve_conflicts();
            Ok(grammar)
        }
    }
}

impl Grammar {
    fn resolve_conflicts(&mut self) {
        for parser_state in self.parser_states.iter_mut() {
            self.unresolved_sr_conflicts += parser_state.resolve_shift_reduce_conflicts();
            self.unresolved_rr_conflicts += parser_state.resolve_reduce_reduce_conflicts();
        }
    }

    fn first_unprocessed_state(&self) -> Option<Rc<ParserState>> {
        match self
            .parser_states
            .iter()
            .filter(|x| !x.is_processed())
            .next()
        {
            Some(unprocessed_state) => Some(Rc::clone(unprocessed_state)),
            None => None,
        }
    }

    fn new_parser_state(&mut self, grammar_items: GrammarItemSet) -> Rc<ParserState> {
        let ident = self.parser_states.len() as u32;
        let parser_state = ParserState::new(ident, grammar_items);
        self.parser_states.push(Rc::clone(&parser_state));
        parser_state
    }

    fn equivalent_state(&self, item_set: &GrammarItemSet) -> Option<&Rc<ParserState>> {
        let target_keys = item_set.kernel_keys();
        if target_keys.len() > 0 {
            for parser_state in self.parser_states.iter() {
                if target_keys == parser_state.kernel_keys() {
                    return Some(parser_state);
                }
            }
        };
        None
    }

    pub fn total_unresolved_conflicts(&self) -> usize {
        self.unresolved_rr_conflicts + self.unresolved_sr_conflicts
    }

    pub fn write_parser_code(&self, file_path: &Path) -> io::Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        self.specification.write_preamble_text(&mut file)?;
        self.write_symbol_enum_code(&mut file)?;
        self.write_parser_implementation_code(&mut file)?;
        Ok(())
    }

    fn write_symbol_enum_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        let tokens = self.specification.symbol_table.token_set();
        wtr.write(b"use std::collections::BTreeSet;\n\n")?;

        wtr.write(b"macro_rules! btree_set {\n")?;
        wtr.write(b"    () => { BTreeSet::new() };\n")?;
        wtr.write(b"    ( $( $x:expr ),* ) => {\n")?;
        wtr.write(b"        {\n")?;
        wtr.write(b"            let mut set = BTreeSet::new();\n")?;
        wtr.write(b"            $( set.insert($x); )*\n")?;
        wtr.write(b"            set\n")?;
        wtr.write(b"        }\n")?;
        wtr.write(b"    };\n")?;
        wtr.write(b"    ( $( $x:expr ),+ , ) => {\n")?;
        wtr.write(b"        btree_set![ $( $x ), * ]\n")?;
        wtr.write(b"    };\n")?;
        wtr.write(b"}\n\n")?;
        wtr.write(b"use lalr1_plus;\n")?;
        wtr.write(b"use lexan;\n\n")?;
        wtr.write(b"#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]\n")?;
        wtr.write(b"pub enum AATerminal {\n")?;
        for token in tokens.iter() {
            wtr.write_fmt(format_args!("    {},\n", token.name()))?;
        }
        wtr.write(b"}\n\n")?;
        wtr.write(b"impl std::fmt::Display for AATerminal {\n")?;
        wtr.write(b"    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {\n")?;
        wtr.write(b"        match self {\n")?;
        for token in tokens.iter() {
            wtr.write(b"            AATerminal::")?;
            let name = token.name();
            match token.symbol_type() {
                SymbolType::LiteralToken(literal) => {
                    wtr.write_fmt(format_args!(
                        "{} => write!(f, r###\"{}\"###),\n",
                        name, literal
                    ))?;
                }
                _ => {
                    wtr.write_fmt(format_args!(
                        "{} => write!(f, r###\"{}\"###),\n",
                        name, name
                    ))?;
                }
            }
        }
        wtr.write(b"        }\n")?;
        wtr.write(b"    }\n")?;
        wtr.write(b"}\n\n")?;
        self.write_lexical_analyzer_code(wtr)?;
        wtr.write(b"#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]\n")?;
        wtr.write(b"pub enum AANonTerminal {\n")?;
        let non_terminal_symbols = self.specification.symbol_table.non_terminal_symbol_set();
        for symbol in non_terminal_symbols.iter() {
            wtr.write_fmt(format_args!("    {},\n", symbol.name()))?;
        }
        wtr.write(b"}\n\n")?;
        wtr.write(b"impl std::fmt::Display for AANonTerminal {\n")?;
        wtr.write(b"    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {\n")?;
        wtr.write(b"        match self {\n")?;
        for symbol in non_terminal_symbols.iter() {
            wtr.write(b"            AANonTerminal::")?;
            let name = symbol.name();
            wtr.write_fmt(format_args!("{} => write!(f, r\"{}\"),\n", name, name))?;
        }
        wtr.write(b"        }\n")?;
        wtr.write(b"    }\n")?;
        wtr.write(b"}\n\n")?;
        Ok(())
    }

    fn write_lexical_analyzer_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        let tokens = self.specification.symbol_table.token_set();
        wtr.write(b"lazy_static! {\n")?;
        wtr.write(b"    static ref AALEXAN: lexan::LexicalAnalyzer<AATerminal> = {\n")?;
        wtr.write(b"        use AATerminal::*;\n")?;
        wtr.write(b"        lexan::LexicalAnalyzer::new(\n")?;
        wtr.write(b"            &[\n")?;
        for token in tokens.iter() {
            if let SymbolType::LiteralToken(literal) = token.symbol_type() {
                wtr.write(b"                ")?;
                wtr.write_fmt(format_args!("({}, r###{}###),\n", token.name(), literal))?;
            }
        }
        wtr.write(b"            ],\n")?;
        wtr.write(b"            &[\n")?;
        for token in tokens.iter() {
            if let SymbolType::RegExToken(regex) = token.symbol_type() {
                wtr.write(b"                ")?;
                wtr.write_fmt(format_args!("({}, r###\"{}\"###),\n", token.name(), regex))?;
            }
        }
        wtr.write(b"            ],\n")?;
        wtr.write(b"            &[\n")?;
        for skip_rule in self.specification.symbol_table.skip_rules() {
            wtr.write(b"                ")?;
            wtr.write_fmt(format_args!("r###\"{}\"###,\n", skip_rule))?;
        }
        wtr.write(b"            ],\n")?;
        wtr.write_fmt(format_args!("            {},\n", AATerminal::AAEnd))?;
        wtr.write(b"        )\n")?;
        wtr.write(b"    };\n")?;
        wtr.write(b"}\n\n")?;
        Ok(())
    }

    fn write_parser_implementation_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        let attr = &self.specification.attribute_type;
        let parser = &self.specification.target_type;
        let text = format!(
            "impl lalr1_plus::Parser<AATerminal, AANonTerminal, {}> for {} {{\n",
            attr, parser
        );
        wtr.write(text.as_bytes())?;
        wtr.write(b"    fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<AATerminal> {\n")?;
        wtr.write(b"        &AALEXAN\n")?;
        wtr.write(b"    }\n\n")?;
        self.write_error_recovery_code(wtr)?;
        self.write_look_ahead_set_code(wtr)?;
        self.write_next_action_code(wtr)?;
        self.specification.write_production_data_code(wtr)?;
        self.write_goto_table_code(wtr)?;
        self.specification.write_semantic_action_code(wtr)?;
        wtr.write(b"}\n")?;
        Ok(())
    }

    fn error_recovery_states_for_token(&self, token: &Rc<Symbol>) -> BTreeSet<u32> {
        self.parser_states
            .iter()
            .filter(|x| x.is_recovery_state_for_token(token))
            .map(|x| x.ident)
            .collect()
    }

    fn format_u32_set(set: &BTreeSet<u32>) -> String {
        let mut string = "btree_set![".to_string();
        for (index, number) in set.iter().enumerate() {
            if index == 0 {
                string += &format!("{}", number);
            } else {
                string += &format!(", {}", number);
            }
        }
        string += "]";
        string
    }

    fn write_error_recovery_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write(b"    fn viable_error_recovery_states(token: &AATerminal) -> BTreeSet<u32> {\n")?;
        wtr.write(b"        match token {\n")?;
        let mut default_required = false;
        for token in self.specification.symbol_table.token_set() {
            let set = self.error_recovery_states_for_token(token);
            if set.len() > 0 {
                let set_str = Self::format_u32_set(&set);
                wtr.write_fmt(format_args!(
                    "            AATerminal::{} => {},\n",
                    token.name(),
                    set_str
                ))?;
            } else {
                default_required = true;
            }
        }
        if default_required {
            wtr.write(b"            _ => btree_set![],\n")?;
        }
        wtr.write(b"        }\n")?;
        wtr.write(b"    }\n\n")?;
        wtr.write(b"    fn error_goto_state(state: u32) -> u32 {\n")?;
        wtr.write(b"        match state {\n")?;
        for parser_state in self.parser_states.iter() {
            if let Some(goto_state_id) = parser_state.error_goto_state_ident() {
                wtr.write_fmt(format_args!(
                    "            {:1} => {:1},\n",
                    parser_state.ident, goto_state_id
                ))?;
            }
        }
        wtr.write(b"            _ => panic!(\"No error go to state for {}\", state),\n")?;
        wtr.write(b"        }\n")?;
        wtr.write(b"    }\n\n")?;
        Ok(())
    }

    fn write_look_ahead_set_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write(b"    fn look_ahead_set(state: u32) -> BTreeSet<AATerminal> {\n")?;
        wtr.write(b"        use AATerminal::*;\n")?;
        wtr.write(b"        return match state {\n")?;
        for parser_state in self.parser_states.iter() {
            wtr.write_fmt(format_args!(
                "            {} => {},\n",
                parser_state.ident,
                format_as_macro_call(&parser_state.look_ahead_set())
            ))?;
        }
        wtr.write(b"            _ => panic!(\"illegal state: {}\", state),\n")?;
        wtr.write(b"        }\n")?;
        wtr.write(b"    }\n\n")?;
        Ok(())
    }

    fn write_next_action_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write(b"    fn next_action(\n")?;
        wtr.write(b"        &self,\n")?;
        wtr.write(b"        aa_state: u32,\n")?;
        wtr.write_fmt(format_args!(
            "        aa_attributes: &lalr1_plus::ParseStack<AATerminal, AANonTerminal, {}>,\n",
            self.specification.attribute_type
        ))?;
        wtr.write(b"        aa_token: &lexan::Token<AATerminal>,\n")?;
        wtr.write(b"    ) -> lalr1_plus::Action {\n")?;
        wtr.write(b"        use lalr1_plus::Action;\n")?;
        wtr.write(b"        use AATerminal::*;\n")?;
        wtr.write(b"        let aa_tag = *aa_token.tag();\n")?;
        wtr.write(b"        return match aa_state {\n")?;
        for parser_state in self.parser_states.iter() {
            parser_state.write_next_action_code(wtr, "            ")?;
        }
        wtr.write(b"            _ => panic!(\"illegal state: {}\", aa_state),\n")?;
        wtr.write(b"        }\n")?;
        wtr.write(b"    }\n\n")?;
        Ok(())
    }

    fn write_goto_table_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write(b"    fn goto_state(lhs: &AANonTerminal, current_state: u32) -> u32 {\n")?;
        wtr.write(b"        return match current_state {\n")?;
        for parser_state in self.parser_states.iter() {
            parser_state.write_goto_table_code(wtr, "            ")?;
        }
        wtr.write(
            b"            _ => panic!(\"Malformed goto table: ({}, {})\", lhs, current_state),\n",
        )?;
        wtr.write(b"        }\n")?;
        wtr.write(b"    }\n\n")?;
        Ok(())
    }

    pub fn write_description(&self, file_path: &Path) -> io::Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        file.write(self.specification.symbol_table.description().as_bytes())?;
        for parser_state in self.parser_states.iter() {
            file.write(parser_state.description().as_bytes())?;
        }
        Ok(())
    }
}
