use std::fmt;

use lalr1plus;
use lexan;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum AATerminal {
    REGEX,
    LITERAL,
    TOKEN,
    FIELD,
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
    FIELDNAME,
    PREDICATE,
    ACTION,
    DCODE,
}

impl fmt::Display for AATerminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AATerminal::*;
        match self {
            REGEX => write!(f, "REGEX"),
            LITERAL => write!(f, "LITERAL"),
            TOKEN => write!(f, "%token"),
            FIELD => write!(f, "%field"),
            LEFT => write!(f, "%left"),
            RIGHT => write!(f, "%right"),
            NONASSOC => write!(f, "%nonassoc"),
            PRECEDENCE => write!(f, "%prec"),
            SKIP => write!(f, "%skip"),
            ERROR => write!(f, "%error"),
            INJECT => write!(f, "%inject"),
            NEWSECTION => write!(f, "%%"),
            COLON => write!(f, ":"),
            VBAR => write!(f, "|"),
            DOT => write!(f, "."),
            IDENT => write!(f, "IDENT"),
            FIELDNAME => write!(f, "FIELDNAME"),
            PREDICATE => write!(f, "PREDICATE"),
            ACTION => write!(f, "ACTION"),
            DCODE => write!(f, "DCODE"),
        }
    }
}

lazy_static! {
    static ref AALEXAN: lexan::LexicalAnalyzer<AATerminal> = {
        use AATerminal::*;
        lexan::LexicalAnalyzer::new(
            &[
                (TOKEN, "%token"),
                (FIELD, "%field"),
                (LEFT, "%left"),
                (RIGHT, "%right"),
                (NONASSOC, "%nonassoc"),
                (PRECEDENCE, "%prec"),
                (SKIP, "%skip"),
                (ERROR, "%error"),
                (INJECT, "%inject"),
                (NEWSECTION, "%%"),
                (COLON, ":"),
                (VBAR, "|"),
                (DOT, "."),
            ],
            &[
                (REGEX, r###"\A(\(.+\)(?=\s))"###),
                (LITERAL, r###"\A("(\\"|[^"\t\r\n\v\f])*")"###),
                (IDENT, r###"\A([a-zA-Z]+[a-zA-Z0-9_]*)"###),
                (FIELDNAME, r###"\A(<[a-zA-Z]+[a-zA-Z0-9_]*>)"###),
                (PREDICATE, r###"\A(\?\((.|[\n\r])*?\?\))"###),
                (ACTION, r###"\A(!\{(.|[\n\r])*?!\})"###),
                (DCODE, r###"\A(%\{(.|[\n\r])*?%\})"###),
            ],
            &[
                r###"\A(/\*(.|[\n\r])*?\*/)"###,
                r###"\A(//[^\n\r]*)"###,
                r###"\A(\s+)"###,
            ],
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
