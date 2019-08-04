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

#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    fields: HashMap<String, (String, lexan::Location)>,
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

    pub fn add_field(
        &mut self,
        name: &str,
        field_type: &str,
        location: &lexan::Location,
    ) -> Result<(), Error> {
        if let Some((_, location)) = self
            .fields
            .insert(name.to_string(), (field_type.to_string(), location.clone()))
        {
            Err(Error::AlreadyDefined(name.to_string(), location.clone()))
        } else {
            Ok(())
        }
    }
}
