// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

#[cfg(not(feature = "bootstrap"))]
use crate::alap_gen_ng::AATerminal;
#[cfg(feature = "bootstrap")]
use crate::bootstrap::AATerminal;
use crate::production::{GrammarItemKey, GrammarItemSet, Production, ProductionTail};
use crate::state::ParserState;
use crate::symbol::non_terminal::NonTerminal;
use crate::symbol::terminal::{Token, TokenSet};
use crate::symbol::{Symbol, SymbolTable};
use lalr1_plus::Parser;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;
use std::io;
use std::io::{stderr, Write};
use std::path::Path;

pub fn report_error(location: &lexan::Location, what: &str) {
    writeln!(stderr(), "{}: Error: {}.", location, what).expect("what?");
}

pub fn report_warning(location: &lexan::Location, what: &str) {
    writeln!(stderr(), "{}: Warning: {}.", location, what).expect("what?");
}

#[derive(Debug, Default)]
pub struct Specification {
    pub symbol_table: SymbolTable,
    productions: Vec<Production>,
    preamble: String,
    pub attribute_type: String,
    pub target_type: String,
    pub error_count: u32,
    pub warning_count: u32,
    pub expected_rr_conflicts: u32,
    pub expected_sr_conflicts: u32,
}

impl lalr1_plus::ReportError<AATerminal> for Specification {}

impl Specification {
    pub fn new(text: String, label: String) -> Result<Self, lalr1_plus::Error<AATerminal>> {
        let mut spec = Specification::default();
        spec.attribute_type = "AttributeData".to_string();
        spec.target_type = "Specification".to_string();
        spec.parse_text(text, label)?;
        // Add dummy error production last so that it has lowest precedence during conflict resolution
        let symbol = spec.symbol_table.error_non_terminal.clone();
        let ident = spec.productions.len() as u32;
        let tail = ProductionTail::default();
        if !spec.symbol_table.error_non_terminal().is_unused() {
            spec.productions.push(Production::new(ident, symbol, tail));
        }
        spec.symbol_table
            .start_non_terminal()
            .set_firsts_data(&spec.productions);
        spec.symbol_table
            .error_non_terminal()
            .set_firsts_data(&spec.productions);
        for non_terminal in spec.symbol_table.non_terminals() {
            //if non_terminal.firsts_data_is_none() {
            non_terminal.set_firsts_data(&spec.productions)
            //}
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

    pub fn new_production(&mut self, left_hand_side: &NonTerminal, tail: &ProductionTail) {
        if self.productions.len() == 0 {
            let location = left_hand_side
                .first_definition()
                .expect("should be defined");
            left_hand_side.add_used_at(&location);
            let start_symbol = self.symbol_table.start_non_terminal_used_at(&location);
            let start_tail = ProductionTail::new(&[left_hand_side.into()], None, None, None);
            let start_production = Production::new(0, start_symbol, start_tail);
            self.productions.push(start_production);
        }
        let ident = self.productions.len() as u32;
        self.productions
            .push(Production::new(ident, left_hand_side.clone(), tail.clone()));
    }

    fn closure(&self, mut closure_set: GrammarItemSet) -> GrammarItemSet {
        let mut additions_made = true;
        while additions_made {
            additions_made = false;
            // Closables extraction as a new separate map necessary to avoid borrow conflict
            for (item_key, look_ahead_set) in closure_set.closable_set() {
                if let Some(symbol) = item_key.next_symbol() {
                    match symbol {
                        Symbol::Terminal(_) => debug_assert!(!item_key.is_closable()),
                        Symbol::NonTerminal(prospective_lhs) => {
                            debug_assert!(item_key.is_closable());
                            for look_ahead_symbol in look_ahead_set.iter() {
                                let firsts = TokenSet::first_all_caps(
                                    item_key.rhs_tail(),
                                    look_ahead_symbol,
                                );
                                for production in self
                                    .productions
                                    .iter()
                                    .filter(|x| x.left_hand_side() == prospective_lhs)
                                {
                                    let prospective_key = GrammarItemKey::from(production);
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
                } else {
                    debug_assert!(!item_key.is_closable());
                }
            }
        }
        closure_set
    }

    pub fn write_preamble_text<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write(self.preamble.as_bytes())?;
        wtr.write(b"\n")?;
        Ok(())
    }

    pub fn write_production_data_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write(b"    fn production_data(production_id: u32) -> (AANonTerminal, usize) {\n")?;
        wtr.write(b"        match production_id {\n")?;
        for production in self.productions.iter() {
            wtr.write_fmt(format_args!(
                "            {} => (AANonTerminal::{}, {}),\n",
                production.ident(),
                production.left_hand_side().name(),
                production.len(),
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
                wtr.write_fmt(format_args!("            {} => {{\n", production.ident()))?;
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
    specification: Specification,
    parser_states: Vec<ParserState>,
}

#[derive(Debug)]
pub enum Error {
    TooManyErrors(u32),
    UndefinedSymbols(u32),
    UnexpectedSRConflicts(u32, u32, String),
    UnexpectedRRConflicts(u32, u32, String),
}

impl TryFrom<Specification> for Grammar {
    type Error = Error;

    fn try_from(specification: Specification) -> Result<Self, Error> {
        for token in specification
            .symbol_table
            .tokens()
            .filter(|t| t.is_unused())
        {
            report_warning(
                token.defined_at(),
                &format!("Token \"{}\" is not used", token.name()),
            )
        }

        for tag in specification.symbol_table.tags().filter(|t| t.is_unused()) {
            report_warning(
                tag.defined_at(),
                &format!("Tag \"{}\" is not used", tag.name()),
            )
        }

        for non_terminal in specification
            .symbol_table
            .non_terminals()
            .filter(|t| t.is_unused())
        {
            report_warning(
                &non_terminal
                    .first_definition()
                    .expect("can't be both unused and undefined"),
                &format!("Non terminal \"{}\" is not used", non_terminal.name()),
            )
        }

        let mut undefined_symbols = 0;
        for non_terminal in specification
            .symbol_table
            .non_terminals()
            .filter(|t| t.is_undefined())
        {
            for location in non_terminal.used_at() {
                report_error(
                    &location,
                    &format!("Non terminal \"{}\" is not defined", non_terminal.name()),
                );
            }
            undefined_symbols += 1;
        }

        if undefined_symbols > 0 {
            Err(Error::UndefinedSymbols(undefined_symbols))
        } else if specification.error_count > 0 {
            Err(Error::TooManyErrors(specification.error_count))
        } else {
            let start_item_key = GrammarItemKey::from(&specification.productions[0]);
            let mut start_look_ahead_set = TokenSet::new();
            start_look_ahead_set.insert(&Token::EndToken);
            let mut map = BTreeMap::<GrammarItemKey, TokenSet>::new();
            map.insert(start_item_key, start_look_ahead_set);
            let start_kernel = specification.closure(GrammarItemSet::from(map));
            let mut grammar = Self {
                specification,
                parser_states: vec![],
            };
            grammar.new_parser_state(start_kernel);
            while let Some(unprocessed_state) = grammar.first_unprocessed_state() {
                let first_time = !unprocessed_state.needs_reprocessing();
                unprocessed_state.mark_as_processed();
                let mut already_done = BTreeSet::<Symbol>::new();
                for item_key in unprocessed_state.non_kernel_key_set().iter() {
                    let symbol_x = item_key.next_symbol().expect("not reducible");
                    if !already_done.insert(symbol_x.clone()) {
                        continue;
                    };
                    let kernel_x = unprocessed_state.generate_goto_kernel(&symbol_x);
                    let item_set_x = grammar.specification.closure(kernel_x);
                    let goto_state =
                        if let Some(equivalent_state) = grammar.equivalent_state(&item_set_x) {
                            equivalent_state.merge_lookahead_sets(&item_set_x);
                            equivalent_state.clone()
                        } else {
                            grammar.new_parser_state(item_set_x)
                        };
                    if first_time {
                        match symbol_x {
                            Symbol::Terminal(token) => {
                                unprocessed_state.add_shift_action(token.clone(), goto_state)
                            }
                            Symbol::NonTerminal(non_terminal) => {
                                if non_terminal.is_error() {
                                    unprocessed_state.set_error_recovery_state(&goto_state);
                                }
                                unprocessed_state.add_goto(non_terminal.clone(), goto_state);
                            }
                        }
                    }
                }
            }
            let (sr_conflicts, rr_conflicts) = grammar.resolve_conflicts();
            if sr_conflicts != grammar.specification.expected_sr_conflicts {
                Err(Error::UnexpectedSRConflicts(
                    sr_conflicts,
                    grammar.specification.expected_sr_conflicts,
                    grammar.describe_sr_conflict_states(),
                ))
            } else if rr_conflicts != grammar.specification.expected_rr_conflicts {
                Err(Error::UnexpectedRRConflicts(
                    rr_conflicts,
                    grammar.specification.expected_rr_conflicts,
                    grammar.describe_rr_conflict_states(),
                ))
            } else {
                Ok(grammar)
            }
        }
    }
}

impl Grammar {
    fn resolve_conflicts(&mut self) -> (u32, u32) {
        let mut sr_conflicts = 0_u32;
        let mut rr_conflicts = 0_u32;
        for parser_state in self.parser_states.iter_mut() {
            sr_conflicts += parser_state.resolve_shift_reduce_conflicts() as u32;
            rr_conflicts += parser_state.resolve_reduce_reduce_conflicts() as u32;
        }
        (sr_conflicts, rr_conflicts)
    }

    fn first_unprocessed_state(&self) -> Option<ParserState> {
        Some(
            self.parser_states
                .iter()
                .find(|x| !x.is_processed())?
                .clone(),
        )
    }

    fn new_parser_state(&mut self, grammar_items: GrammarItemSet) -> ParserState {
        let ident = self.parser_states.len() as u32;
        let parser_state = ParserState::new(ident, grammar_items);
        self.parser_states.push(parser_state.clone());
        parser_state
    }

    fn equivalent_state(&self, item_set: &GrammarItemSet) -> Option<&ParserState> {
        let target_key_set = item_set.kernel_key_set();
        if target_key_set.len() > 0 {
            for parser_state in self.parser_states.iter() {
                if target_key_set == parser_state.kernel_key_set() {
                    return Some(parser_state);
                }
            }
        };
        None
    }
}

impl Grammar {
    fn write_parser_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write(b"// generated by alap_gen_ng.\n\n")?;

        self.specification.write_preamble_text(wtr)?;
        self.write_symbol_enum_code(wtr)?;
        self.write_parser_implementation_code(wtr)?;
        Ok(())
    }

    pub fn write_parser_code_to_file(&self, file_path: &Path) -> io::Result<()> {
        let mut file = std::fs::File::create(file_path)?;
        self.write_parser_code(&mut file)?;
        Ok(())
    }

    fn write_symbol_enum_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        let special_tokens = [Token::EndToken];
        let special_non_terminals = self.specification.symbol_table.used_non_terminal_specials();

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
        for token in special_tokens
            .iter()
            .chain(self.specification.symbol_table.tokens())
        {
            wtr.write_fmt(format_args!("    {},\n", token.name()))?;
        }
        wtr.write(b"}\n\n")?;
        wtr.write(b"impl std::fmt::Display for AATerminal {\n")?;
        wtr.write(b"    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {\n")?;
        wtr.write(b"        match self {\n")?;
        for token in special_tokens
            .iter()
            .chain(self.specification.symbol_table.tokens())
        {
            wtr.write(b"            AATerminal::")?;
            match token {
                Token::Literal(token_data) => {
                    wtr.write_fmt(format_args!(
                        "{} => write!(f, r###\"{}\"###),\n",
                        token_data.name, token_data.text
                    ))?;
                }
                Token::Regex(token_data) => {
                    wtr.write_fmt(format_args!(
                        "{} => write!(f, r###\"{}\"###),\n",
                        token_data.name, token_data.name
                    ))?;
                }
                Token::EndToken => {
                    wtr.write_fmt(format_args!(
                        "{} => write!(f, r###\"{}\"###),\n",
                        token.name(),
                        token.name()
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
        for non_terminal in special_non_terminals
            .iter()
            .chain(self.specification.symbol_table.non_terminals())
        {
            wtr.write_fmt(format_args!("    {},\n", non_terminal.name()))?;
        }
        wtr.write(b"}\n\n")?;
        wtr.write(b"impl std::fmt::Display for AANonTerminal {\n")?;
        wtr.write(b"    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {\n")?;
        wtr.write(b"        match self {\n")?;
        for non_terminal in special_non_terminals
            .iter()
            .chain(self.specification.symbol_table.non_terminals())
        {
            wtr.write(b"            AANonTerminal::")?;
            let name = non_terminal.name();
            wtr.write_fmt(format_args!("{} => write!(f, r\"{}\"),\n", name, name))?;
        }
        wtr.write(b"        }\n")?;
        wtr.write(b"    }\n")?;
        wtr.write(b"}\n\n")?;
        Ok(())
    }

    fn write_lexical_analyzer_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write(b"lazy_static! {\n")?;
        wtr.write(b"    static ref AALEXAN: lexan::LexicalAnalyzer<AATerminal> = {\n")?;
        wtr.write(b"        use AATerminal::*;\n")?;
        wtr.write(b"        lexan::LexicalAnalyzer::new(\n")?;
        wtr.write(b"            &[\n")?;
        for token in self.specification.symbol_table.literal_tokens() {
            wtr.write(b"                ")?;
            wtr.write_fmt(format_args!(
                "({}, r###{}###),\n",
                token.name(),
                token.text()
            ))?;
        }
        wtr.write(b"            ],\n")?;
        wtr.write(b"            &[\n")?;
        for token in self.specification.symbol_table.regex_tokens() {
            wtr.write(b"                ")?;
            wtr.write_fmt(format_args!(
                "({}, r###\"{}\"###),\n",
                token.name(),
                token.text()
            ))?;
        }
        wtr.write(b"            ],\n")?;
        wtr.write(b"            &[\n")?;
        for skip_rule in self.specification.symbol_table.skip_rules() {
            wtr.write(b"                ")?;
            wtr.write_fmt(format_args!("r###\"{}\"###,\n", skip_rule))?;
        }
        wtr.write(b"            ],\n")?;
        wtr.write_fmt(format_args!("            {},\n", Token::EndToken.name()))?;
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

    fn error_recovery_state_set_for_token(&self, token: &Token) -> BTreeSet<u32> {
        self.parser_states
            .iter()
            .filter(|x| x.is_recovery_state_for_token(token))
            .map(|x| x.ident())
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
        for token in [Token::EndToken]
            .iter()
            .chain(self.specification.symbol_table.tokens())
        {
            let set = self.error_recovery_state_set_for_token(token);
            if set.len() > 0 {
                wtr.write_fmt(format_args!(
                    "            AATerminal::{} => {},\n",
                    token.name(),
                    Self::format_u32_set(&set)
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
                    parser_state.ident(),
                    goto_state_id
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
                parser_state.ident(),
                parser_state.look_ahead_set().formated_as_macro_call()
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
        file.write(b"\nProductions:\n")?;
        for production in self.specification.productions.iter() {
            file.write_fmt(format_args!("  {}\n", production))?;
        }
        for parser_state in self.parser_states.iter() {
            file.write(parser_state.description().as_bytes())?;
        }
        Ok(())
    }

    pub fn describe_sr_conflict_states(&self) -> String {
        let mut string = String::new();
        for parser_state in self.parser_states.iter() {
            if parser_state.shift_reduce_conflict_count() > 0 {
                string += &parser_state.description().as_str();
            }
        }
        string
    }

    pub fn describe_rr_conflict_states(&self) -> String {
        let mut string = String::new();
        for parser_state in self.parser_states.iter() {
            if parser_state.reduce_reduce_conflict_count() > 0 {
                string += &parser_state.description().as_str();
            }
        }
        string
    }
}
