use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use lexan;

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

#[derive(Debug, Clone, Copy)]
pub enum Associativity {
    NonAssoc,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct AssociativePrecedence {
    associativity: Associativity,
    precedence: u32,
}

impl Default for AssociativePrecedence {
    fn default() -> Self {
        Self {
            associativity: Associativity::NonAssoc,
            precedence: 0,
        }
    }
}

impl AssociativePrecedence {
    pub fn is_explicitly_set(&self) -> bool {
        self.precedence != 0
    }
}

#[derive(Debug, Clone)]
struct SymbolMutableData {
    associative_precedence: AssociativePrecedence,
    defined_at: Option<lexan::Location>,
    used_at: Vec<lexan::Location>,
}

impl Default for SymbolMutableData {
    fn default() -> Self {
        Self {
            associative_precedence: AssociativePrecedence::default(),
            defined_at: None,
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
    pub fn is_tag(&self) -> bool {
        match self {
            SymbolType::Tag => true,
            _ => false,
        }
    }

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

#[derive(Debug, Clone)]
pub struct Symbol {
    ident: u32,
    name: String,
    symbol_type: SymbolType,
    pattern: String,
    mutable_data: RefCell<SymbolMutableData>,
}

impl Symbol {
    pub fn new(ident: u32, name: String, symbol_type: SymbolType, pattern: String) -> Rc<Self> {
        Rc::new(Self {
            ident,
            name,
            symbol_type,
            pattern,
            mutable_data: RefCell::new(SymbolMutableData::default()),
        })
    }

    pub fn new_tag_at(ident: u32, name: &str, location: &lexan::Location) -> Rc<Symbol> {
        let mutable_data = RefCell::new(SymbolMutableData {
            associative_precedence: AssociativePrecedence::default(),
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

    pub fn is_tag(&self) -> bool {
        self.symbol_type.is_tag()
    }

    pub fn is_token(&self) -> bool {
        self.symbol_type.is_token()
    }

    pub fn is_non_terminal(&self) -> bool {
        self.symbol_type.is_non_terminal()
    }

    pub fn new_token_at(
        ident: u32,
        name: &str,
        pattern: &str,
        location: &lexan::Location,
    ) -> Rc<Symbol> {
        let mutable_data = RefCell::new(SymbolMutableData {
            associative_precedence: AssociativePrecedence::default(),
            defined_at: Some(location.clone()),
            used_at: vec![],
        });
        Rc::new(Self {
            ident,
            name: name.to_string(),
            pattern: pattern.to_string(),
            symbol_type: SymbolType::Token,
            mutable_data,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new_non_terminal_at(ident: u32, name: &str, location: &lexan::Location) -> Rc<Symbol> {
        let mutable_data = RefCell::new(SymbolMutableData {
            associative_precedence: AssociativePrecedence::default(),
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

    pub fn new_non_terminal_used_at(ident: u32, name: &str, location: &lexan::Location) -> Rc<Symbol> {
        let mutable_data = RefCell::new(SymbolMutableData {
            associative_precedence: AssociativePrecedence::default(),
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

    pub fn add_reference(&self, location: &lexan::Location) {
        self.mutable_data
            .borrow_mut()
            .used_at
            .push(location.clone())
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
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SpecialSymbols {
    Start,
    End,
    LexicalError,
    SyntaxError,
    SemanticError,
}

#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    special_symbols: HashMap<SpecialSymbols, Rc<Symbol>>,
    tokens: HashMap<String, Rc<Symbol>>, // indexed by token name
    literal_tokens: HashMap<String, Rc<Symbol>>, // indexed by token name
    tags: HashMap<String, Rc<Symbol>>,   // indexed by tag name
    non_terminals: HashMap<String, Rc<Symbol>>, // indexed by tag name
    skip_rules: Vec<String>,
    next_precedence: u32,
    next_ident: u32,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut st = Self {
            special_symbols: HashMap::new(),
            tokens: HashMap::new(),
            literal_tokens: HashMap::new(),
            tags: HashMap::new(),
            non_terminals: HashMap::new(),
            skip_rules: Vec::new(),
            next_precedence: u32::max_value(),
            next_ident: 5,
        };
        let symbol = Symbol::new(
            0,
            "AASTART".to_string(),
            SymbolType::NonTerminal,
            String::new(),
        );
        st.special_symbols.insert(SpecialSymbols::Start, symbol);
        let symbol = Symbol::new(1, "AAEND".to_string(), SymbolType::Token, String::new());
        st.special_symbols.insert(SpecialSymbols::End, symbol);
        let symbol = Symbol::new(
            2,
            "AALEXICALERROR".to_string(),
            SymbolType::NonTerminal,
            String::new(),
        );
        st.special_symbols
            .insert(SpecialSymbols::LexicalError, symbol);
        let symbol = Symbol::new(
            3,
            "AASYNTAXERROR".to_string(),
            SymbolType::NonTerminal,
            String::new(),
        );
        st.special_symbols
            .insert(SpecialSymbols::SyntaxError, symbol);
        let symbol = Symbol::new(
            4,
            "AASEMANTICERROR".to_string(),
            SymbolType::NonTerminal,
            String::new(),
        );
        st.special_symbols
            .insert(SpecialSymbols::SemanticError, symbol);
        st
    }

    pub fn special_symbol(&self, t: &SpecialSymbols) -> &Rc<Symbol> {
        self.special_symbols.get(t).unwrap()
    }

    pub fn is_known_non_terminal(&self, _name: &str) -> bool {
        false
    }

    pub fn is_known_tag(&self, name: &str) -> bool {
        self.tags.contains_key(name)
    }

    pub fn is_known_token(&self, name: &str) -> bool {
        self.tokens.contains_key(name)
    }

    pub fn use_symbol_named(&mut self, symbol_name: &str, location: &lexan::Location) -> Option<&Rc<Symbol>> {
        if let Some(token) = self.tokens.get(symbol_name) {
            token.add_used_at(location);
            Some(token)
        } else if let Some(tag) = self.tags.get(symbol_name) {
            tag.add_used_at(location);
            Some(tag)
        } else if let Some(non_terminal) = self.non_terminals.get(symbol_name) {
            non_terminal.add_used_at(location);
            Some(non_terminal)
        } else {
            None
        }
    }

    pub fn declaration_location(&self, symbol_name: &str) -> Option<lexan::Location> {
        if let Some(token) = self.tokens.get(symbol_name) {
            token.defined_at()
        } else if let Some(tag) = self.tags.get(symbol_name) {
            tag.defined_at()
        } else if let Some(non_terminal) = self.non_terminals.get(symbol_name) {
            non_terminal.defined_at()
        } else {
            None
        }
    }

    pub fn new_tag(&mut self, name: &str, location: &lexan::Location) -> Result<(), Error> {
        let tag = Symbol::new_tag_at(self.next_ident, name, location);
        self.next_ident += 1;
        if let Some(tag) = self.tokens.insert(name.to_string(), tag) {
            Err(Error::AlreadyDefined(Rc::clone(&tag)))
        } else {
            Ok(())
        }
    }

    pub fn new_token(
        &mut self,
        name: &str,
        pattern: &str,
        location: &lexan::Location,
    ) -> Result<(), Error> {
        let token = Symbol::new_token_at(self.next_ident, name, pattern, location);
        self.next_ident += 1;
        if let Some(token) = self.tokens.insert(name.to_string(), token.clone()) {
            Err(Error::AlreadyDefined(Rc::clone(&token)))
        } else if pattern.starts_with('"') {
            if let Some(token) = self.literal_tokens.insert(pattern.to_string(), token) {
                Err(Error::AlreadyDefined(Rc::clone(&token)))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub fn define_non_terminal(&mut self, name: &str, location: &lexan::Location) -> &Rc<Symbol> {
        if let Some(non_terminal) = self.non_terminals.get_mut(name) {
            non_terminal.add_reference(location);
        } else {
            let ident = self.next_ident;
            self.non_terminals.insert(
                name.to_string(),
                Symbol::new_non_terminal_at(ident, name, location),
            );
            self.next_ident += 1;
        };
        self.non_terminals.get(name).unwrap()
        //self.non_terminals.entry(name.to_string()).or_insert_with(
        //    || Symbol::new_non_terminal_at(ident, name, location)
        //)
    }

    pub fn use_new_non_terminal(&mut self, name: &str, location: &lexan::Location) -> &Rc<Symbol> {
        let symbol = Symbol::new_non_terminal_used_at(self.next_ident, name, location);
        self.next_ident += 1;
        self.non_terminals.insert(name.to_string(), symbol);
        self.non_terminals.get(name).unwrap()
    }

    pub fn add_skip_rule(&mut self, rule: &str) {
        self.skip_rules.push(rule.to_string());
    }

    pub fn set_precedences(&mut self, associativity: Associativity, tags: &Vec<Rc<Symbol>>) {
        let precedence = self.next_precedence;
        for symbol in tags.iter() {
            symbol.set_associative_precedence(associativity, precedence);
        }
        self.next_precedence -= 1;
    }

    pub fn get_literal_token(&self, text: &str, location: &lexan::Location) -> Option<&Rc<Symbol>> {
        if let Some(token) = self.literal_tokens.get(text) {
            token.add_used_at(location);
            Some(token)
        } else {
            None
        }
    }

    pub fn get_token(&self, name: &str, location: &lexan::Location) -> Option<&Rc<Symbol>> {
        if let Some(token) = self.tokens.get(name) {
            token.add_used_at(location);
            Some(token)
        } else {
            None
        }
    }
}
