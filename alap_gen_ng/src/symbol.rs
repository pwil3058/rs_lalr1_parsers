// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>
use std::{collections::BTreeMap, fmt};

use crate::symbol::non_terminal::NonTerminal;
use crate::symbol::tag::{Tag, TagOrToken};
use crate::symbol::terminal::Token;

pub mod non_terminal;
pub mod tag;
pub mod terminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    NonAssoc,
    Left,
    Right,
}

impl Default for Associativity {
    fn default() -> Self {
        Associativity::NonAssoc
    }
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    Terminal(Token),
    NonTerminal(NonTerminal),
}

impl From<&Token> for Symbol {
    fn from(token: &Token) -> Self {
        Symbol::Terminal(token.clone())
    }
}

impl From<&NonTerminal> for Symbol {
    fn from(non_terminal: &NonTerminal) -> Self {
        Symbol::NonTerminal(non_terminal.clone())
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Symbol::NonTerminal(non_terminal) => write!(f, "{}", non_terminal.name()),
            Symbol::Terminal(token) => match token {
                Token::Literal(token_data) => write!(f, "{}", token_data.text),
                _ => write!(f, "{}", token.name()),
            },
        }
    }
}

impl Symbol {
    pub fn is_non_terminal(&self) -> bool {
        match self {
            Symbol::NonTerminal(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    DuplicateTag(Tag),
    DuplicateToken(Token),
    DuplicateTokenDefinition(Token),
    ConflictsWithToken(Token),
    DuplicateSkipRule(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DuplicateTag(tag) => {
                write!(
                    f,
                    "Tag \"{}\" already defined at {}",
                    tag.name(),
                    tag.defined_at(),
                )
            }
            Error::DuplicateToken(token) => {
                write!(
                    f,
                    "Token \"{}\" already defined at {}",
                    token.name(),
                    token.defined_at(),
                )
            }
            Error::DuplicateTokenDefinition(token) => {
                write!(
                    f,
                    "Token \"{}\" defined at {} has same definition",
                    token.name(),
                    token.defined_at(),
                )
            }
            Error::ConflictsWithToken(token) => {
                write!(
                    f,
                    "NonTerminal \"{}\" conflicts with token defined at {}.",
                    token.name(),
                    token.defined_at(),
                )
            }
            Error::DuplicateSkipRule(string) => {
                write!(f, "Skip rule \"{}\" already defined.", string,)
            }
        }
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    tags: BTreeMap<String, Tag>,
    tokens: BTreeMap<String, Token>,
    literal_tokens: BTreeMap<String, Token>,
    regex_tokens: BTreeMap<String, Token>,
    non_terminals: BTreeMap<String, NonTerminal>,
    skip_rules: Vec<String>,
    next_precedence: u16,
    start_non_terminal: NonTerminal,
    pub error_non_terminal: NonTerminal,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self {
            tags: BTreeMap::new(),
            tokens: BTreeMap::new(),
            literal_tokens: BTreeMap::new(),
            regex_tokens: BTreeMap::new(),
            non_terminals: BTreeMap::new(),
            skip_rules: Vec::new(),
            next_precedence: u16::MAX,
            start_non_terminal: NonTerminal::new_start(),
            error_non_terminal: NonTerminal::new_error(),
        }
    }
}

impl SymbolTable {
    pub fn start_non_terminal(&self) -> &NonTerminal {
        &self.start_non_terminal
    }
    pub fn error_non_terminal(&self) -> &NonTerminal {
        &self.error_non_terminal
    }

    pub fn new_tag(&mut self, name: &str, defined_at: &lexan::Location) -> Result<Tag, Error> {
        let tag = Tag::new(name, defined_at);
        if let Some(other) = self.tags.insert(name.to_string(), tag.clone()) {
            Err(Error::DuplicateTag(other))
        } else {
            Ok(tag)
        }
    }

    pub fn get_tag(&self, name: &str) -> Option<&Tag> {
        self.tags.get(name)
    }

    pub fn tags(&self) -> impl Iterator<Item = &Tag> {
        self.tags.values()
    }

    pub fn new_literal_token(
        &mut self,
        name: &str,
        text: &str,
        defined_at: &lexan::Location,
    ) -> Result<Token, Error> {
        let token = Token::new_literal_token(name, text, defined_at);
        if let Some(other) = self.tokens.insert(name.to_string(), token.clone()) {
            Err(Error::DuplicateToken(other))
        } else if let Some(other) = self.literal_tokens.insert(text.to_string(), token.clone()) {
            Err(Error::DuplicateTokenDefinition(other))
        } else {
            Ok(token)
        }
    }

    pub fn new_regex_token(
        &mut self,
        name: &str,
        text: &str,
        defined_at: &lexan::Location,
    ) -> Result<Token, Error> {
        let token = Token::new_regex_token(name, text, defined_at);
        if let Some(other) = self.tokens.insert(name.to_string(), token.clone()) {
            Err(Error::DuplicateToken(other))
        } else if let Some(other) = self.regex_tokens.insert(text.to_string(), token.clone()) {
            Err(Error::DuplicateTokenDefinition(other))
        } else {
            Ok(token)
        }
    }

    pub fn get_token(&self, name: &str) -> Option<&Token> {
        self.tokens.get(name)
    }

    pub fn get_literal_token(&self, lexeme: &str) -> Option<&Token> {
        self.literal_tokens.get(lexeme)
    }

    pub fn tokens(&self) -> impl Iterator<Item = &Token> {
        self.tokens.values()
    }

    pub fn literal_tokens(&self) -> impl Iterator<Item = &Token> {
        self.literal_tokens.values()
    }

    pub fn regex_tokens(&self) -> impl Iterator<Item = &Token> {
        self.regex_tokens.values()
    }

    pub fn non_terminal_defined_at(
        &mut self,
        name: &str,
        defined_at: &lexan::Location,
    ) -> Result<NonTerminal, Error> {
        if let Some(non_terminal) = self.non_terminals.get(name) {
            non_terminal.add_defined_at(defined_at);
            Ok(non_terminal.clone())
        } else if let Some(token) = self.tokens.get(name) {
            Err(Error::ConflictsWithToken(token.clone()))
        } else {
            let non_terminal = NonTerminal::new_defined(name, defined_at);
            self.non_terminals
                .insert(name.to_string(), non_terminal.clone());
            Ok(non_terminal)
        }
    }

    pub fn non_terminals(&self) -> impl Iterator<Item = &NonTerminal> {
        self.non_terminals.values()
    }

    pub fn symbol_used_at(&mut self, name: &str, used_at: &lexan::Location) -> Symbol {
        if let Some(token) = self.tokens.get(name) {
            token.add_used_at(used_at);
            Symbol::Terminal(token.clone())
        } else if let Some(non_terminal) = self.non_terminals.get(name) {
            non_terminal.add_used_at(used_at);
            Symbol::NonTerminal(non_terminal.clone())
        } else {
            let non_terminal = NonTerminal::new_used(name, used_at);
            self.non_terminals
                .insert(name.to_string(), non_terminal.clone());
            Symbol::NonTerminal(non_terminal)
        }
    }

    pub fn error_symbol_used_at(&self, used_at: &lexan::Location) -> Symbol {
        self.error_non_terminal.add_used_at(used_at);
        Symbol::from(&self.error_non_terminal)
    }

    pub fn start_non_terminal_used_at(&self, used_at: &lexan::Location) -> NonTerminal {
        self.start_non_terminal.add_used_at(used_at);
        self.start_non_terminal.clone()
    }

    pub fn add_skip_rule(&mut self, skip_rule: &String) -> Result<(), Error> {
        if self.skip_rules.contains(skip_rule) {
            Err(Error::DuplicateSkipRule(skip_rule.to_string()))
        } else {
            self.skip_rules.push(skip_rule.to_string());
            Ok(())
        }
    }

    pub fn skip_rules(&self) -> impl Iterator<Item = &String> {
        self.skip_rules.iter()
    }

    pub fn set_precedences(
        &mut self,
        associativity: Associativity,
        tag_or_token_list: &[TagOrToken],
    ) {
        let precedence = self.next_precedence;
        self.next_precedence -= 1;
        for tag_or_token in tag_or_token_list.iter() {
            match tag_or_token {
                TagOrToken::Tag(tag) => {
                    tag.set_associativity(associativity);
                    tag.set_precedence(precedence);
                }
                TagOrToken::Token(token) => {
                    token.set_associativity(associativity);
                    token.set_precedence(precedence);
                }
                TagOrToken::Invalid => (),
            }
        }
    }

    pub fn description(&self) -> String {
        let mut string = "Symbols:\n".to_string();
        string += "  Tokens:\n";
        for token in [Token::EndToken].iter().chain(self.tokens()) {
            string += &format!(
                "    {}({}): {}({})\n",
                token.name(),
                token.text(),
                token.associativity(),
                token.precedence()
            );
        }
        string += "  Tags:\n";
        for tag in self.tags.values() {
            string += &format!(
                "    {}: {}({})\n",
                tag.name(),
                tag.associativity(),
                tag.precedence()
            );
        }
        string += "  Non Terminal Symbols:\n";
        for non_terminal in [
            self.start_non_terminal.clone(),
            self.error_non_terminal.clone(),
        ]
        .iter()
        .chain(self.non_terminals())
        {
            string += &format!(
                "    {}: {}\n",
                non_terminal.name(),
                non_terminal.firsts_data()
            );
        }
        string
    }
}
