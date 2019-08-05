use std::{collections::HashMap, fmt};

pub enum Error {
    AlreadyDefined(String, lexan::Location),
}

impl fmt::Display for Error {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::AlreadyDefined(name, location) => {
                write!(dest, "\"{}\" already defined at {}", name, location)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Associativity {
    NonAssoc,
    Left,
    Right,
    Default,
}

#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    tokens: HashMap<String, (String, lexan::Location)>,
    skip_rules: Vec<String>,
}

impl SymbolTable {
    pub fn is_known_non_terminal(&self, _name: &str) -> bool {
        false
    }

    pub fn is_known_tag(&self, _name: &str) -> bool {
        false
    }

    pub fn is_known_token(&self, _name: &str) -> bool {
        false
    }

    pub fn add_token(
        &mut self,
        name: &str,
        pattern: &str,
        location: &lexan::Location,
    ) -> Result<(), Error> {
        if let Some((_, location)) = self
            .tokens
            .insert(name.to_string(), (pattern.to_string(), location.clone()))
        {
            Err(Error::AlreadyDefined(name.to_string(), location.clone()))
        } else {
            Ok(())
        }
    }

    pub fn add_skip_rule(&mut self, rule: &str) {
        self.skip_rules.push(rule.to_string());
    }

    pub fn set_precedence(&mut self, associativity: Associativity, tags: &Vec<String>) {
        panic!("not yet implemented")
    }
}
