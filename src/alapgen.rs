
use std::{fmt, fs::File, io::Read, rc::Rc};

use crate::{
    attributes::*,
    grammar::GrammarSpecification,
    state::ProductionTail,
    symbols::{AssociativePrecedence, Associativity, SpecialSymbols},
};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum AATerminal {
    AAEND,
    REGEX,
    LITERAL,
    TOKEN,
    LEFT,
    RIGHT,
    NONASSOC,
    PRECEDENCE,
    SKIP,
    ERROR,
    INJECT,
    NEWSECTION,
    COLON,
    VBAR,
    DOT,
    IDENT,
    PREDICATE,
    ACTION,
    RUSTCODE,
}

impl std::fmt::Display for AATerminal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
        AATerminal::AAEND => write!(f, r###"AAEND"###),
        AATerminal::REGEX => write!(f, r###"REGEX"###),
        AATerminal::LITERAL => write!(f, r###"LITERAL"###),
        AATerminal::TOKEN => write!(f, r###""%token""###),
        AATerminal::LEFT => write!(f, r###""%left""###),
        AATerminal::RIGHT => write!(f, r###""%right""###),
        AATerminal::NONASSOC => write!(f, r###""%nonassoc""###),
        AATerminal::PRECEDENCE => write!(f, r###""%prec""###),
        AATerminal::SKIP => write!(f, r###""%skip""###),
        AATerminal::ERROR => write!(f, r###""%error""###),
        AATerminal::INJECT => write!(f, r###""%inject""###),
        AATerminal::NEWSECTION => write!(f, r###""%%""###),
        AATerminal::COLON => write!(f, r###"":""###),
        AATerminal::VBAR => write!(f, r###""|""###),
        AATerminal::DOT => write!(f, r###"".""###),
        AATerminal::IDENT => write!(f, r###"IDENT"###),
        AATerminal::PREDICATE => write!(f, r###"PREDICATE"###),
        AATerminal::ACTION => write!(f, r###"ACTION"###),
        AATerminal::RUSTCODE => write!(f, r###"RUSTCODE"###),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum AANonTerminal {
    AASTART,
    AALEXICALERROR,
    AASYNTAXERROR,
    AASEMANTICERROR,
    specification,
    preamble,
    definitions,
    production_rules,
    oinjection,
    injection,
    injection_head,
    token_definitions,
    skip_definitions,
    precedence_definitions,
    token_definition,
    new_token_name,
    pattern,
    skip_definition,
    precedence_definition,
    tag_list,
    tag,
    production_group,
    production_group_head,
    production_tail_list,
    production_tail,
    action,
    predicate,
    symbol_list,
    tagged_precedence,
    symbol,
}

impl std::fmt::Display for AANonTerminal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
        AANonTerminal::AASTART => write!(f, r"AASTART"),
        AANonTerminal::AALEXICALERROR => write!(f, r"AALEXICALERROR"),
        AANonTerminal::AASYNTAXERROR => write!(f, r"AASYNTAXERROR"),
        AANonTerminal::AASEMANTICERROR => write!(f, r"AASEMANTICERROR"),
        AANonTerminal::specification => write!(f, r"specification"),
        AANonTerminal::preamble => write!(f, r"preamble"),
        AANonTerminal::definitions => write!(f, r"definitions"),
        AANonTerminal::production_rules => write!(f, r"production_rules"),
        AANonTerminal::oinjection => write!(f, r"oinjection"),
        AANonTerminal::injection => write!(f, r"injection"),
        AANonTerminal::injection_head => write!(f, r"injection_head"),
        AANonTerminal::token_definitions => write!(f, r"token_definitions"),
        AANonTerminal::skip_definitions => write!(f, r"skip_definitions"),
        AANonTerminal::precedence_definitions => write!(f, r"precedence_definitions"),
        AANonTerminal::token_definition => write!(f, r"token_definition"),
        AANonTerminal::new_token_name => write!(f, r"new_token_name"),
        AANonTerminal::pattern => write!(f, r"pattern"),
        AANonTerminal::skip_definition => write!(f, r"skip_definition"),
        AANonTerminal::precedence_definition => write!(f, r"precedence_definition"),
        AANonTerminal::tag_list => write!(f, r"tag_list"),
        AANonTerminal::tag => write!(f, r"tag"),
        AANonTerminal::production_group => write!(f, r"production_group"),
        AANonTerminal::production_group_head => write!(f, r"production_group_head"),
        AANonTerminal::production_tail_list => write!(f, r"production_tail_list"),
        AANonTerminal::production_tail => write!(f, r"production_tail"),
        AANonTerminal::action => write!(f, r"action"),
        AANonTerminal::predicate => write!(f, r"predicate"),
        AANonTerminal::symbol_list => write!(f, r"symbol_list"),
        AANonTerminal::tagged_precedence => write!(f, r"tagged_precedence"),
        AANonTerminal::symbol => write!(f, r"symbol"),
        }
    }
}

lazy_static! {
    static ref AALEXAN: lexan::LexicalAnalyzer<AATerminal> = {
        use AATerminal::*;
        lexan::LexicalAnalyzer::new(
            &[
                (TOKEN, r###"%token"###),
                (LEFT, r###"%left"###),
                (RIGHT, r###"%right"###),
                (NONASSOC, r###"%nonassoc"###),
                (PRECEDENCE, r###"%prec"###),
                (SKIP, r###"%skip"###),
                (ERROR, r###"%error"###),
                (INJECT, r###"%inject"###),
                (NEWSECTION, r###"%%"###),
                (COLON, r###":"###),
                (VBAR, r###"|"###),
                (DOT, r###"."###),
            ],
            &[
                (REGEX, r###"(\(.+\)(?=\s))"###),
                (LITERAL, r###"("(\\"|[^"\t\r\n\v\f])*")"###),
                (IDENT, r###"([a-zA-Z]+[a-zA-Z0-9_]*)"###),
                (PREDICATE, r###"(\?\((.|[\n\r])*?\?\))"###),
                (ACTION, r###"(!\{(.|[\n\r])*?!\})"###),
                (RUSTCODE, r###"(%\{(.|[\n\r])*?%\})"###),
            ],
            &[
                r###"(/\*(.|[\n\r])*?\*/)"###,
                r###"(//[^\n\r]*)"###,
                r###"(\s+)"###,
            ],
            AAEND,
        )
    };
}

impl lalr1plus::Parser<AATerminal, AANonTerminal, AttributeData> for Calc {
    fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<AATerminal> {
        &AALEXAN
    }

    fn production_data(production_id: u32) -> (AANonTerminal, usize) {
        match production_id {
            0 => (AANonTerminal::AASTART, 1),
            1 => (AANonTerminal::specification, 4),
            2 => (AANonTerminal::oinjection, 0),
            3 => (AANonTerminal::oinjection, 1),
            4 => (AANonTerminal::injection_head, 2),
            5 => (AANonTerminal::injection, 2),
            6 => (AANonTerminal::preamble, 0),
            7 => (AANonTerminal::preamble, 3),
            8 => (AANonTerminal::definitions, 3),
            9 => (AANonTerminal::token_definitions, 2),
            10 => (AANonTerminal::token_definitions, 4),
            11 => (AANonTerminal::token_definition, 3),
            12 => (AANonTerminal::new_token_name, 1),
            13 => (AANonTerminal::new_token_name, 1),
            14 => (AANonTerminal::pattern, 1),
            15 => (AANonTerminal::pattern, 1),
            16 => (AANonTerminal::skip_definitions, 0),
            17 => (AANonTerminal::skip_definitions, 4),
            18 => (AANonTerminal::skip_definition, 2),
            19 => (AANonTerminal::precedence_definitions, 0),
            20 => (AANonTerminal::precedence_definitions, 4),
            21 => (AANonTerminal::precedence_definition, 2),
            22 => (AANonTerminal::precedence_definition, 2),
            23 => (AANonTerminal::precedence_definition, 2),
            24 => (AANonTerminal::tag_list, 1),
            25 => (AANonTerminal::tag_list, 2),
            26 => (AANonTerminal::tag, 1),
            27 => (AANonTerminal::tag, 1),
            28 => (AANonTerminal::production_rules, 3),
            29 => (AANonTerminal::production_rules, 3),
            30 => (AANonTerminal::production_group, 3),
            31 => (AANonTerminal::production_group_head, 2),
            32 => (AANonTerminal::production_tail_list, 1),
            33 => (AANonTerminal::production_tail_list, 3),
            34 => (AANonTerminal::production_tail, 0),
            35 => (AANonTerminal::production_tail, 1),
            36 => (AANonTerminal::production_tail, 2),
            37 => (AANonTerminal::production_tail, 1),
            38 => (AANonTerminal::production_tail, 4),
            39 => (AANonTerminal::production_tail, 3),
            40 => (AANonTerminal::production_tail, 3),
            41 => (AANonTerminal::production_tail, 2),
            42 => (AANonTerminal::production_tail, 3),
            43 => (AANonTerminal::production_tail, 2),
            44 => (AANonTerminal::production_tail, 2),
            45 => (AANonTerminal::production_tail, 1),
            46 => (AANonTerminal::action, 1),
            47 => (AANonTerminal::predicate, 1),
            48 => (AANonTerminal::tagged_precedence, 2),
            49 => (AANonTerminal::tagged_precedence, 2),
            50 => (AANonTerminal::symbol_list, 1),
            51 => (AANonTerminal::symbol_list, 2),
            52 => (AANonTerminal::symbol, 1),
            53 => (AANonTerminal::symbol, 1),
            54 => (AANonTerminal::symbol, 1),
            _ => panic!("malformed production data table"),
        }
    }

    fn do_semantic_action(
        &mut self,
        aa_production_id: u32,
        aa_rhs: Vec<AttributeData>,
        aa_token_stream: &mut lexan::TokenStream<AATerminal>,
    ) -> AttributeData {
        let mut aa_lhs = if let Some(a) = aa_rhs.first() {
            a.clone()
        } else {
            AttributeData::default()
        };
        match aa_production_id {
            2 => {
                // oinjection: <empty>
                 // no injection so nothing to do 
            }
            4 => {
                // injection_head: "%inject" LITERAL
                
            let (text, location) = aa_rhs[1].text_and_location().unwrap();
            let file_path = text.trim_matches('"');
            match File::open(&file_path) {
                Ok(mut file) => {
                    let mut text = String::new();
                    if let Err(err) = file.read_to_string(&mut text) {
                        self.error(&location, &format!("Injecting: {}", err));
                    } else if text.len() == 0 {
                        self.error(&location, &format!("Injected file \"{}\" is empty.", file_path));
                    } else {
                        aa_token_stream.inject(text, file_path.to_string());
                    }
                }
                Err(err) => self.error(&location, &format!("Injecting: {}.", err)),
            };
        
            }
            6 => {
                // preamble: <empty>
                
            // no preamble defined so there's nothing to do
        
            }
            7 => {
                // preamble: oinjection RUSTCODE oinjection
                
            let text = aa_rhs[1].matched_text.unwrap();
            self.set_preamble(&text[2..text.len() - 2]);
        
            }
            11 => {
                // token_definition: "%token" new_token_name pattern
                
            let (name, location) = aa_rhs[1].text_and_location().unwrap();
            let pattern = aa_rhs[2].matched_text().unwrap();
            if let Err(err) = self.symbol_table.new_token(name, pattern, location) {
                self.error(location, &err.to_string());
            }
        
            }
            12 => {
                // new_token_name: IDENT ?( !Self::is_allowable_name($1.matched_text().unwrap()) ?)
                
            let (name, location) = aa_rhs[0].text_and_location().unwrap();
            self.warning(
                location,
                &format!("token name \"{}\" may clash with generated code", name),
            );
        
            }
            16 => {
                // skip_definitions: <empty>
                
            // do nothing
        
            }
            18 => {
                // skip_definition: "%skip" REGEX
                
            let skip_rule = aa_rhs[1].matched_text().unwrap();
            self.symbol_table.add_skip_rule(skip_rule);
        
            }
            19 => {
                // precedence_definitions: <empty>
                
            // do nothing
        
            }
            21 => {
                // precedence_definition: "%left" tag_list
                
            let mut tag_list = aa_rhs[1].symbol_list().clone();
            self.symbol_table
                .set_precedences(Associativity::Left, &mut tag_list);
        
            }
            22 => {
                // precedence_definition: "%right" tag_list
                
            let mut tag_list = aa_rhs[1].symbol_list().clone();
            self.symbol_table
                .set_precedences(Associativity::Right, &mut tag_list);
        
            }
            23 => {
                // precedence_definition: "%nonassoc" tag_list
                
            let mut tag_list = aa_rhs[1].symbol_list().clone();
            self.symbol_table
                .set_precedences(Associativity::NonAssoc, &mut tag_list);
        
            }
            24 => {
                // tag_list: tag
                
            let tag = aa_rhs[0].symbol().unwrap();
            aa_lhs = AttributeData::<AATerminal>::SymbolList(vec![Rc::clone(tag)]);
        
            }
            25 => {
                // tag_list: tag_list tag
                
            let mut tag_list = aa_rhs[0].symbol_list().clone();
            let tag = aa_rhs[1].symbol().unwrap();
            tag_list.push(Rc::clone(tag));
            aa_lhs = AttributeData::<AATerminal>::SymbolList(tag_list);
        
            }
            26 => {
                // tag: LITERAL
                
            let (text, location) = aa_rhs[0].text_and_location().unwrap();
            if let Some(symbol) = self.symbol_table.get_literal_token(text, location) {
                aa_lhs = AttributeData::<AATerminal>::Symbol(Some(Rc::clone(symbol)));
            } else {
                let symbol = self.symbol_table.special_symbol(Some(SpecialSymbols::LexicalError));
                aa_lhs = AttributeData::<AATerminal>::Symbol(Some(Rc::clone(symbol)));
                let msg = format!("Literal token \"{}\" is not known", text);
                self.error(location, &msg);
            }
        
            }
            27 => {
                // tag: IDENT
                
            let (name, location) = aa_rhs[0].text_and_location().unwrap();
            if let Some(symbol) = self.symbol_table.use_symbol_named(name, location) {
                aa_lhs = AttributeData::<AATerminal>::Symbol(Some(Rc::clone(symbol)));
                if symbol.is_non_terminal() {
                    self.error(
                        location,
                        &format!(
                            "Non terminal \"{}\" cannot be used as precedence tag.",
                            name
                        ),
                    )
                }
            } else {
                if !Self::is_allowable_name(name) {
                    self.warning(
                        location,
                        &format!("tag name \"{}\" may clash with generated code", name),
                    );
                };
                match self.symbol_table.new_tag(name, location) {
                    Ok(symbol) => aa_lhs = AttributeData::<AATerminal>::Symbol(Some(symbol)),
                    Err(err) => self.error(location, &err.to_string()),
                }
            }
        
            }
            30 => {
                // production_group: production_group_head production_tail_list "."
                
            let lhs = aa_rhs[0].left_hand_side();
            let tails = aa_rhs[1].production_tail_list();
            for tail in tails.iter() {
                self.new_production(Rc::clone(&lhs), tail.clone());
            }
        
            }
            31 => {
                // production_group_head: IDENT ":"
                
            let (name, location) = aa_rhs[0].text_and_location().unwrap();
            if let Some(symbol) = self.symbol_table.use_symbol_named(name, location) {
                aa_lhs = AttributeData::<AATerminal>::LeftHandSide(Rc::clone(symbol));
                if !symbol.is_non_terminal() {
                    self.error(
                        location,
                        &format!(
                            "Token/tag \"{}\" cannot be used as left hand side of production.",
                            name
                        ),
                    )
                }
            } else {
                if !Self::is_allowable_name(name) {
                    self.warning(
                        location,
                        &format!("Non terminal name \"{}\" may clash with generated code", name),
                    );
                };
                let non_terminal = self.symbol_table.define_non_terminal(name, location);
                aa_lhs = AttributeData::<AATerminal>::LeftHandSide(non_terminal);
            }
        
            }
            32 => {
                // production_tail_list: production_tail
                
            let production_tail = aa_rhs[0].production_tail().clone();
            aa_lhs = AttributeData::<AATerminal>::ProductionTailList(vec![production_tail]);
        
            }
            33 => {
                // production_tail_list: production_tail_list "|" production_tail
                
            let mut production_tail_list = aa_rhs[0].production_tail_list().clone();
            let production_tail = aa_rhs[2].production_tail().clone();
            production_tail_list.push(production_tail);
            aa_lhs = AttributeData::<AATerminal>::ProductionTailList(production_tail_list);
        
            }
            34 => {
                // production_tail: <empty>
                
            let tail = ProductionTail::new(vec![], None, None, None);
            aa_lhs = AttributeData::<AATerminal>::ProductionTail(tail)
        
            }
            35 => {
                // production_tail: action
                
            let action = aa_rhs[0].action().to_string();
            let tail = ProductionTail::new(vec![], None, None, Some(action));
            aa_lhs = AttributeData::<AATerminal>::ProductionTail(tail)
        
            }
            36 => {
                // production_tail: predicate action
                
            let predicate = aa_rhs[0].predicate().to_string();
            let action = aa_rhs[1].action().to_string();
            let tail = ProductionTail::new(vec![], Some(predicate), None, Some(action));
            aa_lhs = AttributeData::<AATerminal>::ProductionTail(tail)
        
            }
            37 => {
                // production_tail: predicate
                
            let predicate = aa_rhs[0].predicate().to_string();
            let tail = ProductionTail::new(vec![], Some(predicate), None, None);
            aa_lhs = AttributeData::<AATerminal>::ProductionTail(tail)
        
            }
            38 => {
                // production_tail: symbol_list predicate tagged_precedence action
                
            let rhs = aa_rhs[0].symbol_list().clone();
            let predicate = aa_rhs[1].predicate().to_string();
            let tagged_precedence = aa_rhs[2].associative_precedence().clone();
            let action = aa_rhs[3].action().to_string();
            let tail = ProductionTail::new(rhs, Some(predicate), Some(tagged_precedence), Some(action));
            aa_lhs = AttributeData::<AATerminal>::ProductionTail(tail)
        
            }
            39 => {
                // production_tail: symbol_list predicate tagged_precedence
                
            let lhs = aa_rhs[0].symbol_list().clone();
            let predicate = aa_rhs[1].predicate().to_string();
            let tagged_precedence = aa_rhs[2].associative_precedence().clone();
            let tail = ProductionTail::new(lhs, Some(predicate), Some(tagged_precedence), None);
            aa_lhs = AttributeData::<AATerminal>::ProductionTail(tail)
        
            }
            40 => {
                // production_tail: symbol_list predicate action
                
            let lhs = aa_rhs[0].symbol_list().clone();
            let predicate = aa_rhs[1].predicate().to_string();
            let action = aa_rhs[2].action().to_string();
            let tail = ProductionTail::new(lhs, Some(predicate), None, Some(action));
            aa_lhs = AttributeData::<AATerminal>::ProductionTail(tail)
        
            }
            41 => {
                // production_tail: symbol_list predicate
                
            let lhs = aa_rhs[0].symbol_list().clone();
            let predicate = aa_rhs[1].predicate().to_string();
            let tail = ProductionTail::new(lhs, Some(predicate), None, None);
            aa_lhs = AttributeData::<AATerminal>::ProductionTail(tail)
        
            }
            42 => {
                // production_tail: symbol_list tagged_precedence action
                
            let lhs = aa_rhs[0].symbol_list().clone();
            let tagged_precedence = aa_rhs[1].associative_precedence().clone();
            let action = aa_rhs[2].action().to_string();
            let tail = ProductionTail::new(lhs, None, Some(tagged_precedence), Some(action));
            aa_lhs = AttributeData::<AATerminal>::ProductionTail(tail)
        
            }
            43 => {
                // production_tail: symbol_list tagged_precedence
                
            let lhs = aa_rhs[0].symbol_list().clone();
            let tagged_precedence = aa_rhs[1].associative_precedence().clone();
            let tail = ProductionTail::new(lhs, None, Some(tagged_precedence), None);
            aa_lhs = AttributeData::<AATerminal>::ProductionTail(tail)
        
            }
            44 => {
                // production_tail: symbol_list action
                
            let lhs = aa_rhs[0].symbol_list().clone();
            let action = aa_rhs[1].action().to_string();
            let tail = ProductionTail::new(lhs, None, None, Some(action));
            aa_lhs = AttributeData::<AATerminal>::ProductionTail(tail)
        
            }
            45 => {
                // production_tail: symbol_list
                
            let lhs = aa_rhs[0].symbol_list().clone();
            let tail = ProductionTail::new(lhs, None, None, None);
            aa_lhs = AttributeData::<AATerminal>::ProductionTail(tail)
        
            }
            46 => {
                // action: ACTION
                
            let text = aa_rhs[0].matched_text().unwrap();
            aa_lhs = AttributeData::<AATerminal>::Action(text[2..text.len() - 2].to_string());
        
            }
            47 => {
                // predicate: PREDICATE
                
            let text = aa_rhs[0].matched_text().unwrap();
            aa_lhs = AttributeData::<AATerminal>::Predicate(text[2..text.len() - 2].to_string());
        
            }
            48 => {
                // tagged_precedence: "%prec" IDENT
                
            let (name, location) = aa_rhs[1].text_and_location();
            let mut ap = AssociativePrecedence::default();
            if let Some(symbol) = self.symbol_table.use_symbol_named(name, location) {
                if symbol.is_non_terminal() {
                    self.error(
                        location,
                        &format!("{}: illegal precedence tag (must be token or tag)", name),
                    );
                } else {
                    ap = symbol.associative_precedence();
                }
            } else {
                self.error(location, &format!("{}: unknown symbol", name));
            };
            aa_lhs = AttributeData::<AATerminal>::AssociativePrecedence(ap);
        
            }
            49 => {
                // tagged_precedence: "%prec" LITERAL
                
            let (lexeme, location) = aa_rhs[1].text_and_location();
            let mut ap = AssociativePrecedence::default();
            if let Some(symbol) = self.symbol_table.get_literal_token(lexeme, location) {
                if symbol.is_non_terminal() {
                    self.error(
                        location,
                        &format!("{}: illegal precedence tag (must be token or tag)", lexeme),
                    );
                } else {
                    ap = symbol.associative_precedence();
                }
            } else {
                self.error(location, &format!("{}: unknown literal", lexeme));
            };
            aa_lhs = AttributeData::<AATerminal>::AssociativePrecedence(ap);
        
            }
            50 => {
                // symbol_list: symbol
                
            let symbol = aa_rhs[0].symbol().unwrap();
            aa_lhs = AttributeData::<AATerminal>::SymbolList(vec![Rc::clone(&symbol)]);
        
            }
            51 => {
                // symbol_list: symbol_list symbol
                
            let symbol = aa_rhs[1].symbol().unwrap();
            let mut symbol_list = aa_rhs[1].symbol_list().clone();
            symbol_list.push(Rc::clone(symbol));
            aa_lhs = AttributeData::<AATerminal>::SymbolList(symbol_list);
        
            }
            52 => {
                // symbol: IDENT
                
            let (name, location) = aa_rhs[0].text_and_location();
            if let Some(symbol) = self.symbol_table.use_symbol_named(name, location) {
                aa_lhs = AttributeData::<AATerminal>::Symbol(Some(Rc::clone(symbol)));
            } else {
                let symbol = self.symbol_table.use_new_non_terminal(name, location);
                aa_lhs = AttributeData::<AATerminal>::Symbol(Some(symbol));
            }
        
            }
            53 => {
                // symbol: LITERAL
                
            let (lexeme, location) = aa_rhs[0].text_and_location();
            if let Some(symbol) = self.symbol_table.get_literal_token(lexeme, location) {
                aa_lhs = AttributeData::<AATerminal>::Symbol(Some(Rc::clone(symbol)));
            } else {
                self.error(location, &format!("{}: unknown literal)", lexeme));
                aa_lhs = AttributeData::<AATerminal>::Symbol(Some(SpecialSymbols::LexicalError));
            }
        
            }
            54 => {
                // symbol: "%error"
                
            let symbol = self
                .symbol_table
                .special_symbol(&SpecialSymbols::SyntaxError);
            aa_lhs = AttributeData::<AATerminal>::Symbol(Some(symbol));
        
            }
            _ => (),
        };
        aa_lhs
    }

    fn goto_state(lhs: &AANonTerminal, current_state: u32) -> u32 {
        return match current_state {
            0 => match lhs {
                AANonTerminal::specification => 1,
                AANonTerminal::preamble => 2,
                AANonTerminal::oinjection => 6,
                AANonTerminal::injection => 3,
                AANonTerminal::injection_head => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            2 => match lhs {
                AANonTerminal::definitions => 7,
                AANonTerminal::oinjection => 9,
                AANonTerminal::injection => 3,
                AANonTerminal::injection_head => 5,
                AANonTerminal::token_definitions => 8,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            8 => match lhs {
                AANonTerminal::oinjection => 15,
                AANonTerminal::injection => 3,
                AANonTerminal::injection_head => 5,
                AANonTerminal::skip_definitions => 14,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            9 => match lhs {
                AANonTerminal::token_definition => 16,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            12 => match lhs {
                AANonTerminal::oinjection => 18,
                AANonTerminal::injection => 3,
                AANonTerminal::injection_head => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            13 => match lhs {
                AANonTerminal::production_rules => 19,
                AANonTerminal::oinjection => 20,
                AANonTerminal::injection => 3,
                AANonTerminal::injection_head => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            14 => match lhs {
                AANonTerminal::oinjection => 22,
                AANonTerminal::injection => 3,
                AANonTerminal::injection_head => 5,
                AANonTerminal::precedence_definitions => 21,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            15 => match lhs {
                AANonTerminal::token_definition => 23,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            17 => match lhs {
                AANonTerminal::new_token_name => 24,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            19 => match lhs {
                AANonTerminal::production_group => 26,
                AANonTerminal::production_group_head => 27,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            20 => match lhs {
                AANonTerminal::production_group => 29,
                AANonTerminal::production_group_head => 27,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            21 => match lhs {
                AANonTerminal::oinjection => 30,
                AANonTerminal::injection => 3,
                AANonTerminal::injection_head => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            22 => match lhs {
                AANonTerminal::skip_definition => 31,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            23 => match lhs {
                AANonTerminal::oinjection => 33,
                AANonTerminal::injection => 3,
                AANonTerminal::injection_head => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            24 => match lhs {
                AANonTerminal::pattern => 34,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            26 => match lhs {
                AANonTerminal::oinjection => 37,
                AANonTerminal::injection => 3,
                AANonTerminal::injection_head => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            27 => match lhs {
                AANonTerminal::production_tail_list => 38,
                AANonTerminal::production_tail => 39,
                AANonTerminal::action => 40,
                AANonTerminal::predicate => 41,
                AANonTerminal::symbol_list => 42,
                AANonTerminal::symbol => 45,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            29 => match lhs {
                AANonTerminal::oinjection => 50,
                AANonTerminal::injection => 3,
                AANonTerminal::injection_head => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            30 => match lhs {
                AANonTerminal::precedence_definition => 51,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            31 => match lhs {
                AANonTerminal::oinjection => 55,
                AANonTerminal::injection => 3,
                AANonTerminal::injection_head => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            41 => match lhs {
                AANonTerminal::action => 59,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            42 => match lhs {
                AANonTerminal::action => 62,
                AANonTerminal::predicate => 60,
                AANonTerminal::tagged_precedence => 61,
                AANonTerminal::symbol => 64,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            51 => match lhs {
                AANonTerminal::oinjection => 65,
                AANonTerminal::injection => 3,
                AANonTerminal::injection_head => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            52 => match lhs {
                AANonTerminal::tag_list => 66,
                AANonTerminal::tag => 67,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            53 => match lhs {
                AANonTerminal::tag_list => 70,
                AANonTerminal::tag => 67,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            54 => match lhs {
                AANonTerminal::tag_list => 71,
                AANonTerminal::tag => 67,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            58 => match lhs {
                AANonTerminal::production_tail => 72,
                AANonTerminal::action => 40,
                AANonTerminal::predicate => 41,
                AANonTerminal::symbol_list => 42,
                AANonTerminal::symbol => 45,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            60 => match lhs {
                AANonTerminal::action => 74,
                AANonTerminal::tagged_precedence => 73,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            61 => match lhs {
                AANonTerminal::action => 75,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            66 => match lhs {
                AANonTerminal::tag => 78,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            70 => match lhs {
                AANonTerminal::tag => 78,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            71 => match lhs {
                AANonTerminal::tag => 78,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            73 => match lhs {
                AANonTerminal::action => 79,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
        }
    }

    fn viable_error_recovery_states(token: &AATerminal) -> Vec<u32> {
        use AATerminal::*;
        match token {
            _ => vec![],
        }
    }

    fn error_goto_state(state: u32) -> u32 {
        match state {
            _ => panic!("No error go to state for {}", state),
        }
    }

    fn next_action(
        &self,
        state: u32,
        aa_attributes: &lalr1plus::ParseStack<AATerminal, AANonTerminal, AttributeData>,
        token: &lexan::Token<AATerminal>,
    ) -> lalr1plus::Action<AATerminal> {
        use lalr1plus::Action;
        use AATerminal::*;
        let aa_tag = *token.tag();
        return match state {
            0 => match aa_tag {
                INJECT => Action::Shift(4),
                // oinjection: <empty>
                RUSTCODE => Action::Reduce(2),
                // preamble: <empty>
                TOKEN => Action::Reduce(6),
                _ => Action::SyntaxError(vec![TOKEN, INJECT, RUSTCODE]),
            },
            1 => match aa_tag {
                // AASTART: specification
                AAEND => Action::Accept,
                _ => Action::SyntaxError(vec![AAEND]),
            },
            2 => match aa_tag {
                INJECT => Action::Shift(4),
                // oinjection: <empty>
                TOKEN => Action::Reduce(2),
                _ => Action::SyntaxError(vec![TOKEN, INJECT]),
            },
            3 => match aa_tag {
                // oinjection: injection
                AAEND | TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION | IDENT | RUSTCODE => Action::Reduce(3),
                _ => Action::SyntaxError(vec![AAEND, TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION, IDENT, RUSTCODE]),
            },
            4 => match aa_tag {
                LITERAL => Action::Shift(10),
                _ => Action::SyntaxError(vec![LITERAL]),
            },
            5 => match aa_tag {
                _ => Action::SyntaxError(vec![]),
            },
            6 => match aa_tag {
                RUSTCODE => Action::Shift(12),
                _ => Action::SyntaxError(vec![RUSTCODE]),
            },
            7 => match aa_tag {
                NEWSECTION => Action::Shift(13),
                _ => Action::SyntaxError(vec![NEWSECTION]),
            },
            8 => match aa_tag {
                INJECT => Action::Shift(4),
                // oinjection: <empty>
                TOKEN => Action::Reduce(2),
                // skip_definitions: <empty>
                LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(16),
                _ => Action::SyntaxError(vec![TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            9 => match aa_tag {
                TOKEN => Action::Shift(17),
                _ => Action::SyntaxError(vec![TOKEN]),
            },
            10 => match aa_tag {
                // injection_head: "%inject" LITERAL
                DOT => Action::Reduce(4),
                _ => Action::SyntaxError(vec![DOT]),
            },
            11 => match aa_tag {
                // injection: injection_head "."
                AAEND | TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION | IDENT | RUSTCODE => Action::Reduce(5),
                _ => Action::SyntaxError(vec![AAEND, TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION, IDENT, RUSTCODE]),
            },
            12 => match aa_tag {
                INJECT => Action::Shift(4),
                // oinjection: <empty>
                TOKEN => Action::Reduce(2),
                _ => Action::SyntaxError(vec![TOKEN, INJECT]),
            },
            13 => match aa_tag {
                INJECT => Action::Shift(4),
                // oinjection: <empty>
                IDENT => Action::Reduce(2),
                _ => Action::SyntaxError(vec![INJECT, IDENT]),
            },
            14 => match aa_tag {
                INJECT => Action::Shift(4),
                // oinjection: <empty>
                SKIP => Action::Reduce(2),
                // precedence_definitions: <empty>
                LEFT | RIGHT | NONASSOC | NEWSECTION => Action::Reduce(19),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            15 => match aa_tag {
                TOKEN => Action::Shift(17),
                _ => Action::SyntaxError(vec![TOKEN]),
            },
            16 => match aa_tag {
                // token_definitions: oinjection token_definition
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(9),
                _ => Action::SyntaxError(vec![TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            17 => match aa_tag {
                IDENT => Action::Shift(25),
                _ => Action::SyntaxError(vec![IDENT]),
            },
            18 => match aa_tag {
                // preamble: oinjection RUSTCODE oinjection
                TOKEN | INJECT => Action::Reduce(7),
                _ => Action::SyntaxError(vec![TOKEN, INJECT]),
            },
            19 => match aa_tag {
                IDENT => Action::Shift(28),
                // specification: preamble definitions "%%" production_rules
                AAEND => Action::Reduce(1),
                _ => Action::SyntaxError(vec![AAEND, IDENT]),
            },
            20 => match aa_tag {
                IDENT => Action::Shift(28),
                _ => Action::SyntaxError(vec![IDENT]),
            },
            21 => match aa_tag {
                INJECT => Action::Shift(4),
                // oinjection: <empty>
                LEFT | RIGHT | NONASSOC => Action::Reduce(2),
                // definitions: token_definitions skip_definitions precedence_definitions
                NEWSECTION => Action::Reduce(8),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION]),
            },
            22 => match aa_tag {
                SKIP => Action::Shift(32),
                _ => Action::SyntaxError(vec![SKIP]),
            },
            23 => match aa_tag {
                INJECT => Action::Shift(4),
                // oinjection: <empty>
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError(vec![TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            24 => match aa_tag {
                REGEX => Action::Shift(35),
                LITERAL => Action::Shift(36),
                _ => Action::SyntaxError(vec![REGEX, LITERAL]),
            },
            25 => match aa_tag {
                REGEX | LITERAL => {
                    if  !Self::is_allowable_name(aa_attributes.at_len_minus_n(1).matched_text().unwrap())  {
                        // new_token_name: IDENT ?( !Self::is_allowable_name($1.matched_text().unwrap()) ?)
                        Action::Reduce(12)
                    } else {
                        // new_token_name: IDENT
                        Action::Reduce(13)
                    }
                }
                _ => Action::SyntaxError(vec![REGEX, LITERAL]),
            },
            26 => match aa_tag {
                INJECT => Action::Shift(4),
                // oinjection: <empty>
                AAEND | IDENT => Action::Reduce(2),
                _ => Action::SyntaxError(vec![AAEND, INJECT, IDENT]),
            },
            27 => match aa_tag {
                LITERAL => Action::Shift(47),
                ERROR => Action::Shift(48),
                IDENT => Action::Shift(46),
                PREDICATE => Action::Shift(44),
                ACTION => Action::Shift(43),
                // production_tail: <empty>
                VBAR | DOT => Action::Reduce(34),
                _ => Action::SyntaxError(vec![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            28 => match aa_tag {
                COLON => Action::Shift(49),
                _ => Action::SyntaxError(vec![COLON]),
            },
            29 => match aa_tag {
                INJECT => Action::Shift(4),
                // oinjection: <empty>
                AAEND | IDENT => Action::Reduce(2),
                _ => Action::SyntaxError(vec![AAEND, INJECT, IDENT]),
            },
            30 => match aa_tag {
                LEFT => Action::Shift(52),
                RIGHT => Action::Shift(53),
                NONASSOC => Action::Shift(54),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC]),
            },
            31 => match aa_tag {
                INJECT => Action::Shift(4),
                // oinjection: <empty>
                LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            32 => match aa_tag {
                REGEX => Action::Shift(56),
                _ => Action::SyntaxError(vec![REGEX]),
            },
            33 => match aa_tag {
                // token_definitions: token_definitions oinjection token_definition oinjection
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(10),
                _ => Action::SyntaxError(vec![TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            34 => match aa_tag {
                // token_definition: "%token" new_token_name pattern
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(11),
                _ => Action::SyntaxError(vec![TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            35 => match aa_tag {
                // pattern: REGEX
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(14),
                _ => Action::SyntaxError(vec![TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            36 => match aa_tag {
                // pattern: LITERAL
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(15),
                _ => Action::SyntaxError(vec![TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            37 => match aa_tag {
                // production_rules: production_rules production_group oinjection
                AAEND | IDENT => Action::Reduce(29),
                _ => Action::SyntaxError(vec![AAEND, IDENT]),
            },
            38 => match aa_tag {
                VBAR => Action::Shift(58),
                DOT => Action::Shift(57),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            39 => match aa_tag {
                // production_tail_list: production_tail
                VBAR | DOT => Action::Reduce(32),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            40 => match aa_tag {
                // production_tail: action
                VBAR | DOT => Action::Reduce(35),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            41 => match aa_tag {
                ACTION => Action::Shift(43),
                // production_tail: predicate
                VBAR | DOT => Action::Reduce(37),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            42 => match aa_tag {
                LITERAL => Action::Shift(47),
                PRECEDENCE => Action::Shift(63),
                ERROR => Action::Shift(48),
                IDENT => Action::Shift(46),
                PREDICATE => Action::Shift(44),
                ACTION => Action::Shift(43),
                // production_tail: symbol_list
                VBAR | DOT => Action::Reduce(45),
                _ => Action::SyntaxError(vec![LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            43 => match aa_tag {
                // action: ACTION
                VBAR | DOT => Action::Reduce(46),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            44 => match aa_tag {
                // predicate: PREDICATE
                PRECEDENCE | VBAR | DOT | ACTION => Action::Reduce(47),
                _ => Action::SyntaxError(vec![PRECEDENCE, VBAR, DOT, ACTION]),
            },
            45 => match aa_tag {
                // symbol_list: symbol
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => Action::Reduce(50),
                _ => Action::SyntaxError(vec![LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            46 => match aa_tag {
                // symbol: IDENT
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => Action::Reduce(52),
                _ => Action::SyntaxError(vec![LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            47 => match aa_tag {
                // symbol: LITERAL
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => Action::Reduce(53),
                _ => Action::SyntaxError(vec![LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            48 => match aa_tag {
                // symbol: "%error"
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => Action::Reduce(54),
                _ => Action::SyntaxError(vec![LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            49 => match aa_tag {
                // production_group_head: IDENT ":"
                LITERAL | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => Action::Reduce(31),
                _ => Action::SyntaxError(vec![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            50 => match aa_tag {
                // production_rules: oinjection production_group oinjection
                AAEND | IDENT => Action::Reduce(28),
                _ => Action::SyntaxError(vec![AAEND, IDENT]),
            },
            51 => match aa_tag {
                INJECT => Action::Shift(4),
                // oinjection: <empty>
                LEFT | RIGHT | NONASSOC | NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION]),
            },
            52 => match aa_tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            53 => match aa_tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            54 => match aa_tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            55 => match aa_tag {
                // skip_definitions: skip_definitions oinjection skip_definition oinjection
                LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(17),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            56 => match aa_tag {
                // skip_definition: "%skip" REGEX
                LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(18),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            57 => match aa_tag {
                // production_group: production_group_head production_tail_list "."
                AAEND | INJECT | IDENT => Action::Reduce(30),
                _ => Action::SyntaxError(vec![AAEND, INJECT, IDENT]),
            },
            58 => match aa_tag {
                LITERAL => Action::Shift(47),
                ERROR => Action::Shift(48),
                IDENT => Action::Shift(46),
                PREDICATE => Action::Shift(44),
                ACTION => Action::Shift(43),
                // production_tail: <empty>
                VBAR | DOT => Action::Reduce(34),
                _ => Action::SyntaxError(vec![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            59 => match aa_tag {
                // production_tail: predicate action
                VBAR | DOT => Action::Reduce(36),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            60 => match aa_tag {
                PRECEDENCE => Action::Shift(63),
                ACTION => Action::Shift(43),
                // production_tail: symbol_list predicate
                VBAR | DOT => Action::Reduce(41),
                _ => Action::SyntaxError(vec![PRECEDENCE, VBAR, DOT, ACTION]),
            },
            61 => match aa_tag {
                ACTION => Action::Shift(43),
                // production_tail: symbol_list tagged_precedence
                VBAR | DOT => Action::Reduce(43),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            62 => match aa_tag {
                // production_tail: symbol_list action
                VBAR | DOT => Action::Reduce(44),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            63 => match aa_tag {
                LITERAL => Action::Shift(77),
                IDENT => Action::Shift(76),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            64 => match aa_tag {
                // symbol_list: symbol_list symbol
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => Action::Reduce(51),
                _ => Action::SyntaxError(vec![LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            65 => match aa_tag {
                // precedence_definitions: precedence_definitions oinjection precedence_definition oinjection
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(20),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION]),
            },
            66 => match aa_tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                // precedence_definition: "%left" tag_list
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(21),
                _ => Action::SyntaxError(vec![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT]),
            },
            67 => match aa_tag {
                // tag_list: tag
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => Action::Reduce(24),
                _ => Action::SyntaxError(vec![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT]),
            },
            68 => match aa_tag {
                // tag: LITERAL
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => Action::Reduce(26),
                _ => Action::SyntaxError(vec![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT]),
            },
            69 => match aa_tag {
                // tag: IDENT
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => Action::Reduce(27),
                _ => Action::SyntaxError(vec![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT]),
            },
            70 => match aa_tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                // precedence_definition: "%right" tag_list
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(22),
                _ => Action::SyntaxError(vec![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT]),
            },
            71 => match aa_tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                // precedence_definition: "%nonassoc" tag_list
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(23),
                _ => Action::SyntaxError(vec![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT]),
            },
            72 => match aa_tag {
                // production_tail_list: production_tail_list "|" production_tail
                VBAR | DOT => Action::Reduce(33),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            73 => match aa_tag {
                ACTION => Action::Shift(43),
                // production_tail: symbol_list predicate tagged_precedence
                VBAR | DOT => Action::Reduce(39),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            74 => match aa_tag {
                // production_tail: symbol_list predicate action
                VBAR | DOT => Action::Reduce(40),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            75 => match aa_tag {
                // production_tail: symbol_list tagged_precedence action
                VBAR | DOT => Action::Reduce(42),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            76 => match aa_tag {
                // tagged_precedence: "%prec" IDENT
                VBAR | DOT | ACTION => Action::Reduce(48),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            77 => match aa_tag {
                // tagged_precedence: "%prec" LITERAL
                VBAR | DOT | ACTION => Action::Reduce(49),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            78 => match aa_tag {
                // tag_list: tag_list tag
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => Action::Reduce(25),
                _ => Action::SyntaxError(vec![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT]),
            },
            79 => match aa_tag {
                // production_tail: symbol_list predicate tagged_precedence action
                VBAR | DOT => Action::Reduce(38),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            _ => panic!("illegal state: {}", state),
        }
    }

}
