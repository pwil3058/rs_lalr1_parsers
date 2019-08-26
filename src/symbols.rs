use std::{cell::RefCell, fmt, rc::Rc};

use lexan;
use ordered_collections::{OrderedMap, OrderedSet};

#[cfg(not(feature = "bootstrap"))]
use crate::alapgen::{AANonTerminal, AATerminal};
#[cfg(feature = "bootstrap")]
use crate::bootstrap::{AANonTerminal, AATerminal};

#[derive(Debug)]
pub enum Error {
    AlreadyDefined(Rc<Symbol>),
}

impl fmt::Display for Error {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::AlreadyDefined(symbol) => {
                if let Some(location) = symbol.defined_at() {
                    write!(
                        dest,
                        "\"{}\" already defined at {}",
                        symbol.name(),
                        location
                    )
                } else {
                    write!(dest, "\"{}\" already defined", symbol.name())
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Associativity {
    NonAssoc,
    Left,
    Right,
}

impl std::fmt::Display for Associativity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Associativity::*;
        match self {
            NonAssoc => write!(f, "NonAssoc"),
            Left => write!(f, "Left"),
            Right => write!(f, "Right"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AssociativePrecedence {
    pub associativity: Associativity,
    pub precedence: u32,
}

impl std::fmt::Display for AssociativePrecedence {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:({})", self.associativity, self.precedence)
    }
}

impl Default for AssociativePrecedence {
    fn default() -> Self {
        Self {
            associativity: Associativity::NonAssoc,
            precedence: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FirstsData {
    pub token_set: OrderedSet<Rc<Symbol>>,
    pub transparent: bool,
}

impl FirstsData {
    pub fn new(token_set: OrderedSet<Rc<Symbol>>, transparent: bool) -> Self {
        Self {
            token_set,
            transparent,
        }
    }
}

impl fmt::Display for FirstsData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:({})", self.token_set, self.transparent)
    }
}

#[derive(Debug, Clone)]
pub struct SymbolMutableData {
    associative_precedence: AssociativePrecedence,
    defined_at: Option<lexan::Location>,
    pub firsts_data: Option<FirstsData>,
    used_at: Vec<lexan::Location>,
}

impl Default for SymbolMutableData {
    fn default() -> Self {
        Self {
            associative_precedence: AssociativePrecedence::default(),
            defined_at: None,
            firsts_data: None,
            used_at: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum SymbolType {
    Token,
    Tag,
    NonTerminal,
}

impl SymbolType {
    pub fn is_token(&self) -> bool {
        match self {
            SymbolType::Token => true,
            _ => false,
        }
    }

    pub fn is_non_terminal(&self) -> bool {
        match self {
            SymbolType::NonTerminal => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct Symbol {
    ident: u32,
    name: String,
    symbol_type: SymbolType,
    pattern: String,
    pub mutable_data: RefCell<SymbolMutableData>,
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Symbol({}):", self.name)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.symbol_type {
            SymbolType::Token => {
                if self.pattern.starts_with('"') {
                    write!(f, "{}", self.pattern)
                } else {
                    write!(f, "{}", self.name)
                }
            }
            _ => write!(f, "{}", self.name),
        }
    }
}

impl_ident_cmp!(Symbol);

impl Symbol {
    pub fn new_non_terminal_at(ident: u32, name: &str, location: &lexan::Location) -> Rc<Symbol> {
        let mutable_data = RefCell::new(SymbolMutableData {
            associative_precedence: AssociativePrecedence::default(),
            firsts_data: None,
            defined_at: Some(location.clone()),
            used_at: vec![],
        });
        Rc::new(Self {
            ident,
            name: name.to_string(),
            pattern: String::new(),
            symbol_type: SymbolType::NonTerminal,
            mutable_data,
        })
    }

    pub fn new_non_terminal_used_at(
        ident: u32,
        name: &str,
        location: &lexan::Location,
    ) -> Rc<Symbol> {
        let mutable_data = RefCell::new(SymbolMutableData {
            associative_precedence: AssociativePrecedence::default(),
            firsts_data: None,
            defined_at: None,
            used_at: vec![location.clone()],
        });
        Rc::new(Self {
            ident,
            name: name.to_string(),
            pattern: String::new(),
            symbol_type: SymbolType::NonTerminal,
            mutable_data,
        })
    }

    pub fn new_tag_at(ident: u32, name: &str, location: &lexan::Location) -> Rc<Symbol> {
        let mutable_data = RefCell::new(SymbolMutableData {
            associative_precedence: AssociativePrecedence::default(),
            firsts_data: None,
            defined_at: Some(location.clone()),
            used_at: vec![],
        });
        Rc::new(Self {
            ident,
            name: name.to_string(),
            pattern: String::new(),
            symbol_type: SymbolType::Tag,
            mutable_data,
        })
    }

    pub fn new_token_at(
        ident: u32,
        name: &str,
        pattern: &str,
        location: &lexan::Location,
    ) -> Rc<Symbol> {
        let mutable_data = RefCell::new(SymbolMutableData {
            associative_precedence: AssociativePrecedence::default(),
            firsts_data: None,
            defined_at: Some(location.clone()),
            used_at: vec![],
        });
        let token = Rc::new(Self {
            ident,
            name: name.to_string(),
            pattern: pattern.to_string(),
            symbol_type: SymbolType::Token,
            mutable_data,
        });
        let mut token_set: OrderedSet<Rc<Symbol>> = OrderedSet::new();
        token_set.insert(Rc::clone(&token));
        token.set_firsts_data(FirstsData {
            token_set,
            transparent: false,
        });
        token
    }

    pub fn is_start_symbol(&self) -> bool {
        self.name == AANonTerminal::AAStart.to_string()
    }

    pub fn is_syntax_error(&self) -> bool {
        self.name == AANonTerminal::AASyntaxError.to_string()
    }

    fn is_special_symbol(&self) -> bool {
        self.ident < NUM_SPECIAL_SYMBOLS
    }

    pub fn is_token(&self) -> bool {
        self.symbol_type.is_token()
    }

    pub fn is_non_terminal(&self) -> bool {
        self.symbol_type.is_non_terminal()
    }

    pub fn is_undefined(&self) -> bool {
        self.mutable_data.borrow().defined_at.is_none() && !self.is_special_symbol()
    }

    pub fn is_unused(&self) -> bool {
        self.mutable_data.borrow().used_at.len() == 0 && !self.is_special_symbol()
    }

    pub fn used_at(&self) -> Vec<lexan::Location> {
        self.mutable_data.borrow().used_at.iter().cloned().collect()
    }

    pub fn precedence(&self) -> u32 {
        self.mutable_data.borrow().associative_precedence.precedence
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn pattern(&self) -> &String {
        &self.pattern
    }

    pub fn defined_at(&self) -> Option<lexan::Location> {
        if let Some(location) = &self.mutable_data.borrow().defined_at {
            Some(location.clone())
        } else {
            None
        }
    }

    pub fn set_associative_precedence(&self, associativity: Associativity, precedence: u32) {
        self.mutable_data.borrow_mut().associative_precedence = AssociativePrecedence {
            associativity,
            precedence,
        }
    }

    pub fn associative_precedence(&self) -> AssociativePrecedence {
        self.mutable_data.borrow().associative_precedence
    }

    pub fn add_used_at(&self, location: &lexan::Location) {
        self.mutable_data
            .borrow_mut()
            .used_at
            .push(location.clone())
    }

    pub fn set_defined_at(&self, location: &lexan::Location) {
        self.mutable_data.borrow_mut().defined_at = Some(location.clone());
    }

    pub fn firsts_data(&self) -> FirstsData {
        let msg = format!("{} :should be set", self.name);
        self.mutable_data.borrow().firsts_data.clone().expect(&msg)
    }

    pub fn firsts_data_is_none(&self) -> bool {
        self.mutable_data.borrow().firsts_data.is_none()
    }

    pub fn set_firsts_data(&self, firsts_data: FirstsData) {
        self.mutable_data.borrow_mut().firsts_data = Some(firsts_data);
    }
}

pub fn format_as_vec(symbol_set: &OrderedSet<Rc<Symbol>>) -> String {
    let mut string = "vec![".to_string();
    for (index, symbol) in symbol_set.iter().enumerate() {
        if index == 0 {
            string += &format!("{}", symbol.name());
        } else {
            string += &format!(", {}", symbol.name());
        }
    }
    string += "]";
    string
}

pub fn format_as_or_list(symbol_set: &OrderedSet<Rc<Symbol>>) -> String {
    let mut string = "".to_string();
    for (index, symbol) in symbol_set.iter().enumerate() {
        if index == 0 {
            string += &format!("{}", symbol.name());
        } else {
            string += &format!(" | {}", symbol.name());
        }
    }
    string
}

const NUM_SPECIAL_SYMBOLS: u32 = 5;

#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    tokens: OrderedMap<String, Rc<Symbol>>, // indexed by token name
    literal_tokens: OrderedMap<String, Rc<Symbol>>, // indexed by token name
    tags: OrderedMap<String, Rc<Symbol>>,   // indexed by tag name
    non_terminals: OrderedMap<String, Rc<Symbol>>, // indexed by tag name
    skip_rules: Vec<String>,
    next_precedence: u32,
    next_ident: u32,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut st = Self {
            tokens: OrderedMap::new(),
            literal_tokens: OrderedMap::new(),
            tags: OrderedMap::new(),
            non_terminals: OrderedMap::new(),
            skip_rules: Vec::new(),
            next_precedence: u32::max_value(),
            next_ident: 0,
        };
        let start_location = lexan::Location::default();

        st.define_non_terminal(&AANonTerminal::AAStart.to_string(), &start_location);
        st.new_token(&AATerminal::AAEnd.to_string(), "", &start_location)
            .expect("There should be no naming conflicts yet.");
        st.define_non_terminal(&AANonTerminal::AASyntaxError.to_string(), &start_location);
        st.define_non_terminal(&AANonTerminal::AALexicalError.to_string(), &start_location);
        st.define_non_terminal(&AANonTerminal::AASemanticError.to_string(), &start_location);

        assert_eq!(NUM_SPECIAL_SYMBOLS, st.next_ident);

        st
    }

    pub fn undefined_symbols(&self) -> impl Iterator<Item = &Rc<Symbol>> {
        self.tokens
            .values()
            .chain(self.tags.values())
            .chain(self.non_terminals.values())
            .filter(|s| s.is_undefined())
    }

    pub fn unused_symbols(&self) -> impl Iterator<Item = &Rc<Symbol>> {
        self.tokens
            .values()
            .chain(self.tags.values())
            .chain(self.non_terminals.values())
            .filter(|s| s.is_unused())
    }

    pub fn non_terminal_symbols(&self) -> impl Iterator<Item = &Rc<Symbol>> {
        self.non_terminals.values()
    }

    pub fn tokens_sorted(&self) -> Vec<&Rc<Symbol>> {
        let mut tokens: Vec<&Rc<Symbol>> = self.tokens.values().collect();
        tokens.sort();
        tokens
    }

    pub fn non_terminal_symbols_sorted(&self) -> Vec<&Rc<Symbol>> {
        let mut non_terminal_symbols: Vec<&Rc<Symbol>> = self.non_terminals.values().collect();
        non_terminal_symbols.sort();
        non_terminal_symbols
    }

    pub fn skip_rules(&self) -> impl Iterator<Item = &String> {
        self.skip_rules.iter()
    }

    pub fn use_symbol_named(
        &mut self,
        symbol_name: &String,
        location: &lexan::Location,
    ) -> Option<Rc<Symbol>> {
        if let Some(token) = self.tokens.get(symbol_name) {
            token.add_used_at(location);
            Some(Rc::clone(token))
        } else if let Some(tag) = self.tags.get(symbol_name) {
            tag.add_used_at(location);
            Some(Rc::clone(tag))
        } else if let Some(non_terminal) = self.non_terminals.get(symbol_name) {
            non_terminal.add_used_at(location);
            Some(Rc::clone(non_terminal))
        } else {
            None
        }
    }

    pub fn new_tag(&mut self, name: &str, location: &lexan::Location) -> Result<Rc<Symbol>, Error> {
        let tag = Symbol::new_tag_at(self.next_ident, name, location);
        self.next_ident += 1;
        if let Some(tag) = self.tags.insert(name.to_string(), Rc::clone(&tag)) {
            Err(Error::AlreadyDefined(Rc::clone(&tag)))
        } else {
            Ok(tag)
        }
    }

    pub fn new_token(
        &mut self,
        name: &str,
        pattern: &str,
        location: &lexan::Location,
    ) -> Result<Rc<Symbol>, Error> {
        let token = Symbol::new_token_at(self.next_ident, name, pattern, location);
        self.next_ident += 1;
        if let Some(token) = self.tokens.insert(name.to_string(), Rc::clone(&token)) {
            Err(Error::AlreadyDefined(Rc::clone(&token)))
        } else if pattern.starts_with('"') {
            if let Some(token) = self
                .literal_tokens
                .insert(pattern.to_string(), Rc::clone(&token))
            {
                Err(Error::AlreadyDefined(Rc::clone(&token)))
            } else {
                Ok(token)
            }
        } else {
            Ok(token)
        }
    }

    pub fn define_non_terminal(&mut self, name: &str, location: &lexan::Location) -> Rc<Symbol> {
        if let Some(non_terminal) = self.non_terminals.get_mut(name) {
            non_terminal.set_defined_at(location);
            Rc::clone(non_terminal)
        } else {
            let ident = self.next_ident;
            self.next_ident += 1;
            let non_terminal = Symbol::new_non_terminal_at(ident, name, location);
            self.non_terminals
                .insert(name.to_string(), Rc::clone(&non_terminal));
            non_terminal
        }
    }

    pub fn use_new_non_terminal(
        &mut self,
        name: &String,
        location: &lexan::Location,
    ) -> Rc<Symbol> {
        let ident = self.next_ident;
        self.next_ident += 1;
        let non_terminal = Symbol::new_non_terminal_used_at(ident, name, location);
        self.non_terminals
            .insert(name.to_string(), Rc::clone(&non_terminal));
        non_terminal
    }

    pub fn add_skip_rule(&mut self, rule: &String) {
        self.skip_rules.push(rule.to_string());
    }

    pub fn set_precedences(&mut self, associativity: Associativity, tags: &Vec<Rc<Symbol>>) {
        let precedence = self.next_precedence;
        for symbol in tags.iter() {
            symbol.set_associative_precedence(associativity, precedence);
        }
        self.next_precedence -= 1;
    }

    pub fn get_literal_token(
        &self,
        text: &String,
        location: &lexan::Location,
    ) -> Option<&Rc<Symbol>> {
        if let Some(token) = self.literal_tokens.get(text) {
            token.add_used_at(location);
            Some(token)
        } else {
            None
        }
    }

    pub fn description(&self) -> String {
        let mut string = "Symbols:\n".to_string();
        string += "  Tokens:\n";
        for token in self.tokens_sorted() {
            string += &format!(
                "    {}({}): {} {}\n",
                token.name,
                token.pattern,
                token.associative_precedence(),
                token.firsts_data()
            );
        }
        string += "  Tags:\n";
        for (_, tag) in self.tags.iter() {
            string += &format!("    {}: {}\n", tag.name, tag.associative_precedence(),);
        }
        string += "  Non Terminal Symbols:\n";
        for symbol in self.non_terminal_symbols_sorted() {
            string += &format!(
                "    {}: {} {}\n",
                symbol.name,
                symbol.associative_precedence(),
                symbol.firsts_data()
            );
        }
        string
    }
}
