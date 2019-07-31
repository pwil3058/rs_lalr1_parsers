use std::fmt;

use lalr1plus;
use lexan;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum AATerminal {
    TAG,
    REGEX,
    LITERAL,
}

impl fmt::Display for AATerminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AATerminal::*;
        match self {
            TAG => write!(f, "%tag"),
            REGEX => write!(f, "REGEX"),
            LITERAL => write!(f, "LITERAL"),
        }
    }
}

lazy_static! {
    static ref AALEXAN: lexan::LexicalAnalyzer<AATerminal> = {
        use AATerminal::*;
        lexan::LexicalAnalyzer::new(
            &[(TAG, "%tag")],
            &[
                (REGEX, r###"\A(\\A\(.+\)(?=\s))"###),
                (LITERAL, r###"\A("(\\"|[^"\t\r\n\v\f])*")"###),
            ],
            &[],
        )
    };
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum AANonTerminal {
    Specification,
    Preamble,
    Definitions,
}

impl fmt::Display for AANonTerminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AANonTerminal::*;
        match self {
            Specification => write!(f, "Specification"),
            Preamble => write!(f, "Preamble"),
            Definitions => write!(f, "Definitions"),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AAAttributeData {}

impl From<(AATerminal, String)> for AAAttributeData {
    fn from(input: (AATerminal, String)) -> Self {
        AAAttributeData::default()
    }
}

impl From<lalr1plus::Error<AATerminal>> for AAAttributeData {
    fn from(_error: lalr1plus::Error<AATerminal>) -> Self {
        AAAttributeData::default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ParserSpecification {}

impl lalr1plus::Parser<AATerminal, AANonTerminal, AAAttributeData> for ParserSpecification {
    fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<AATerminal> {
        &AALEXAN
    }

    fn viable_error_recovery_states(_tag: &AATerminal) -> Vec<u32> {
        vec![]
    }

    fn error_go_state(state: u32) -> u32 {
        state
    }

    fn next_action<'a>(
        &self,
        _state: u32,
        _attributes: &lalr1plus::ParseStack<AATerminal, AANonTerminal, AAAttributeData>,
        _token: &lexan::Token<'a, AATerminal>,
    ) -> lalr1plus::Action<AATerminal> {
        lalr1plus::Action::Shift(0)
    }

    fn next_coda(
        &self,
        _state: u32,
        _attributes: &lalr1plus::ParseStack<AATerminal, AANonTerminal, AAAttributeData>,
    ) -> lalr1plus::Coda {
        lalr1plus::Coda::Accept
    }

    fn production_data(&mut self, _production_id: u32) -> (AANonTerminal, usize) {
        (AANonTerminal::Specification, 0)
    }

    fn goto_state(_lhs: &AANonTerminal, _current_state: u32) -> u32 {
        0
    }

    fn do_semantic_action(
        &mut self,
        _production_id: u32,
        _rhs: Vec<AAAttributeData>,
        _token_stream: &mut lexan::TokenStream<AATerminal>,
    ) -> AAAttributeData {
        AAAttributeData::default()
    }
}
