// Copyright 2022 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

pub use std::{
    collections::BTreeSet,
    convert::From,
    default::Default,
    fmt::{self, Debug, Display},
    io::Write,
};

use lexan::TokenStream;

#[derive(Debug, Clone)]
pub enum Error<T: Ord + Copy + Debug + Display + Eq> {
    LexicalError(lexan::Error<T>, BTreeSet<T>),
    SyntaxError(lexan::Token<T>, BTreeSet<T>),
}

fn format_set<T: Ord + Display>(set: &BTreeSet<T>) -> String {
    let mut string = String::new();
    let last = set.len() - 1;
    for (index, item) in set.iter().enumerate() {
        if index == 0 {
            string += &item.to_string();
        } else {
            if index == last {
                string += " or ";
            } else {
                string += ", ";
            };
            string += &item.to_string()
        }
    }
    string
}

impl<T: Ord + Copy + Debug + Display + Eq> Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::LexicalError(lex_err, expected) => write!(
                f,
                "Lexical Error: {}: expected: {}.",
                lex_err,
                format_set(expected)
            ),
            Error::SyntaxError(found, expected) => write!(
                f,
                "Syntax Error: expected: {} found: {} at: {}.",
                format_set(expected),
                found.tag(),
                found.location()
            ),
        }
    }
}

pub trait ReportError<T: Ord + Copy + Debug + Display + Eq> {
    fn report_error(&mut self, error: &Error<T>) {
        let message = error.to_string();
        if let Error::LexicalError(lexan::Error::AmbiguousMatches(_, _, _), _) = error {
            panic!("Fatal Error: {message}!!");
        };
        std::io::stderr()
            .write_all(message.as_bytes())
            .expect("Nowhere to go here!!!");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol<T, N> {
    Terminal(T),
    NonTerminal(N),
    Start,
    Error,
}

#[derive(Debug)]
pub struct ParseStack<T, N, A>
where
    T: Copy + Ord + Debug + Display,
    A: From<lexan::Token<T>> + From<Error<T>>,
{
    states: Vec<(Symbol<T, N>, u32)>,
    attributes: Vec<A>,
    last_error_state: Option<u32>,
}

impl<T, N, A> ParseStack<T, N, A>
where
    T: Copy + Ord + Debug + Display,
    A: From<lexan::Token<T>> + From<Error<T>>,
{
    fn new() -> Self {
        Self {
            states: vec![(Symbol::Start, 0)],
            attributes: vec![],
            last_error_state: None,
        }
    }

    fn current_state(&self) -> u32 {
        self.states.last().unwrap().1
    }

    pub fn at_len_minus_n(&self, n: usize) -> &A {
        let len = self.attributes.len();
        &self.attributes[len - n]
    }

    fn pop_n(&mut self, n: usize) -> Vec<A> {
        let len = self.states.len();
        self.states.truncate(len - n);
        let len = self.attributes.len();
        self.attributes.split_off(len - n)
    }

    fn push_error(&mut self, state: u32, error: Error<T>) {
        self.states.push((Symbol::Error, state));
        self.attributes.push(A::from(error))
    }

    fn push_terminal(&mut self, token: lexan::Token<T>, new_state: u32) {
        self.states
            .push((Symbol::Terminal(*token.tag()), new_state));
        self.attributes.push(A::from(token));
    }

    fn push_non_terminal(&mut self, non_terminal: N, attribute: A, new_state: u32) {
        self.attributes.push(attribute);
        self.states
            .push((Symbol::NonTerminal(non_terminal), new_state));
    }

    fn is_last_error_state(&self, state: u32) -> bool {
        if let Some(last_error_state) = self.last_error_state {
            state == last_error_state
        } else {
            false
        }
    }

    fn distance_to_viable_state<F: Fn(&T) -> BTreeSet<u32>>(
        &mut self,
        tokens: &mut TokenStream<T>,
        viable_error_recovery_states: F,
    ) -> Option<usize> {
        while !tokens.is_empty() {
            if let Ok(token) = tokens.front() {
                let viable_states = viable_error_recovery_states(token.tag());
                for sub in 1..self.states.len() {
                    let candidate = self.states[self.states.len() - sub].1;
                    if !self.is_last_error_state(candidate) && viable_states.contains(&candidate) {
                        self.last_error_state = Some(candidate);
                        return Some(sub - 1);
                    }
                }
            };
            tokens.advance();
        }
        None
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Shift(u32),
    Reduce(u32),
    Accept,
    SyntaxError,
}

pub trait Parser<T: Ord + Copy + Debug, N, A>
where
    T: Ord + Copy + Debug + Display,
    N: Ord + Display + Debug,
    A: Default + From<lexan::Token<T>> + From<Error<T>>,
    Self: ReportError<T>,
{
    fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<T>;
    fn next_action(&self, state: u32, o_token: &lexan::Token<T>) -> Action;
    fn production_data(production_id: u32) -> (N, usize);
    fn goto_state(lhs: &N, current_state: u32) -> u32;
    fn do_semantic_action<F: FnMut(String, String)>(
        &mut self,
        _production_id: u32,
        _attributes: Vec<A>,
        mut inject: F,
    ) -> A {
        // NB: required in order to cop with issue #35203
        inject(String::new(), String::new());
        // confirm multiple injects OK.
        inject(String::new(), String::new());
        A::default()
    }

    fn viable_error_recovery_states(tag: &T) -> BTreeSet<u32>;

    fn error_goto_state(state: u32) -> u32 {
        panic!("No error go to state for {state}")
    }

    fn look_ahead_set(state: u32) -> BTreeSet<T>;

    fn recover_from_error(
        error: Error<T>,
        parse_stack: &mut ParseStack<T, N, A>,
        tokens: &mut TokenStream<T>,
    ) -> bool {
        if let Some(distance) =
            parse_stack.distance_to_viable_state(tokens, |t| Self::viable_error_recovery_states(t))
        {
            parse_stack.pop_n(distance);
            let next_state = Self::error_goto_state(parse_stack.current_state());
            parse_stack.push_error(next_state, error);
            true
        } else {
            false
        }
    }

    fn parse_text(&mut self, text: &str, label: &str) -> Result<(), Error<T>> {
        let mut tokens = self.lexical_analyzer().token_stream(text, label);
        let mut parse_stack = ParseStack::<T, N, A>::new();
        let mut result: Result<(), Error<T>> = Ok(());

        loop {
            match tokens.front() {
                Err(err) => {
                    let expected_tokens = Self::look_ahead_set(parse_stack.current_state());
                    let error = Error::LexicalError(err, expected_tokens);
                    self.report_error(&error);
                    result = Err(error.clone());
                    if !Self::recover_from_error(error, &mut parse_stack, &mut tokens) {
                        return result;
                    }
                }
                Ok(token) => match self.next_action(parse_stack.current_state(), &token) {
                    Action::Accept => return result,
                    Action::Shift(next_state) => {
                        parse_stack.push_terminal(token, next_state);
                        tokens.advance();
                    }
                    Action::Reduce(production_id) => {
                        let (lhs, rhs_len) = Self::production_data(production_id);
                        let rhs = parse_stack.pop_n(rhs_len);
                        let next_state = Self::goto_state(&lhs, parse_stack.current_state());
                        let attribute = self
                            .do_semantic_action(production_id, rhs, |s, l| tokens.inject(&s, &l));
                        parse_stack.push_non_terminal(lhs, attribute, next_state);
                    }
                    Action::SyntaxError => {
                        let expected_tokens = Self::look_ahead_set(parse_stack.current_state());
                        let error = Error::SyntaxError(token.clone(), expected_tokens);
                        self.report_error(&error);
                        result = Err(error.clone());
                        if !Self::recover_from_error(error, &mut parse_stack, &mut tokens) {
                            return result;
                        }
                    }
                },
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::convert::From;
    use std::str::FromStr;

    use lazy_static::lazy_static;

    use crate as lalr1;

    #[derive(Debug, Clone)]
    pub enum AttributeData {
        Token(lexan::Token<AATerminal>),
        Error(lalr1::Error<AATerminal>),
        Value(f64),
        Id(String),
        Default,
    }

    impl Default for AttributeData {
        fn default() -> Self {
            AttributeData::Default
        }
    }

    impl AttributeData {
        fn id(&self) -> &String {
            match self {
                AttributeData::Id(id) => id,
                _ => panic!("invalid variant"),
            }
        }

        fn value(&self) -> f64 {
            match self {
                AttributeData::Value(value) => *value,
                _ => panic!("invalid variant"),
            }
        }
    }

    impl From<lexan::Token<AATerminal>> for AttributeData {
        fn from(input: lexan::Token<AATerminal>) -> Self {
            match input.tag() {
                AATerminal::NUMBER => {
                    let value = f64::from_str(input.lexeme()).unwrap();
                    AttributeData::Value(value)
                }
                AATerminal::ID => {
                    let id = input.lexeme().to_string();
                    AttributeData::Id(id)
                }
                _ => AttributeData::Token(input.clone()),
            }
        }
    }

    impl From<lalr1::Error<AATerminal>> for AttributeData {
        fn from(error: lalr1::Error<AATerminal>) -> Self {
            AttributeData::Error(error)
        }
    }

    const UNDEFINED_VARIABLE: u32 = 1 << 0;
    const DIVIDE_BY_ZERO: u32 = 1 << 1;
    const SYNTAX_ERROR: u32 = 1 << 2;
    const LEXICAL_ERROR: u32 = 1 << 3;

    pub struct Calc {
        errors: u32,
        variables: HashMap<String, f64>,
    }

    impl lalr1::ReportError<AATerminal> for Calc {}

    impl Calc {
        pub fn new() -> Self {
            Self {
                errors: 0,
                variables: HashMap::new(),
            }
        }

        pub fn variable(&self, name: &str) -> Option<f64> {
            self.variables.get(name).copied()
        }

        fn report_errors(&self) {
            if self.errors & UNDEFINED_VARIABLE != 0 {
                println!("Undefined variable(s).")
            };
            if self.errors & DIVIDE_BY_ZERO != 0 {
                println!("Divide by zero.")
            };
            if self.errors & SYNTAX_ERROR != 0 {
                println!("Syntax error.")
            };
            if self.errors & LEXICAL_ERROR != 0 {
                println!("Lexical error.")
            };
        }
    }

    use std::collections::BTreeSet;

    macro_rules! btree_set {
        () => { BTreeSet::new() };
        ( $( $x:expr ),* ) => {
            {
                let mut set = BTreeSet::new();
                $( set.insert($x); )*
                set
            }
        };
        ( $( $x:expr ),+ , ) => {
            btree_set![ $( $x ), * ]
        };
    }

    #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
    pub enum AATerminal {
        AAEnd,
        ASSIGN,
        DIVIDE,
        EOL,
        ID,
        LPR,
        MINUS,
        NUMBER,
        PLUS,
        RPR,
        TIMES,
    }

    impl std::fmt::Display for AATerminal {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                AATerminal::AAEnd => write!(f, r###"AAEnd"###),
                AATerminal::ASSIGN => write!(f, r###""=""###),
                AATerminal::DIVIDE => write!(f, r###""/""###),
                AATerminal::EOL => write!(f, r###"EOL"###),
                AATerminal::ID => write!(f, r###"ID"###),
                AATerminal::LPR => write!(f, r###""(""###),
                AATerminal::MINUS => write!(f, r###""-""###),
                AATerminal::NUMBER => write!(f, r###"NUMBER"###),
                AATerminal::PLUS => write!(f, r###""+""###),
                AATerminal::RPR => write!(f, r###"")""###),
                AATerminal::TIMES => write!(f, r###""*""###),
            }
        }
    }

    lazy_static! {
        static ref AALEXAN: lexan::LexicalAnalyzer<AATerminal> = {
            use AATerminal::*;
            lexan::LexicalAnalyzer::new(
                &[
                    (LPR, r###"("###),
                    (RPR, r###")"###),
                    (TIMES, r###"*"###),
                    (PLUS, r###"+"###),
                    (MINUS, r###"-"###),
                    (DIVIDE, r###"/"###),
                    (ASSIGN, r###"="###),
                ],
                &[
                    (NUMBER, r###"([0-9]+(\.[0-9]+){0,1})"###),
                    (ID, r###"([a-zA-Z]+)"###),
                    (EOL, r###"(\n)"###),
                ],
                &[r###"([\t\r ]+)"###],
                AAEnd,
            )
        };
    }

    #[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
    pub enum AANonTerminal {
        AAStart,
        AAError,
        Expr,
        Line,
        SetUp,
    }

    impl std::fmt::Display for AANonTerminal {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            match self {
                AANonTerminal::AAStart => write!(f, r"AAStart"),
                AANonTerminal::AAError => write!(f, r"AAError"),
                AANonTerminal::Expr => write!(f, r"Expr"),
                AANonTerminal::Line => write!(f, r"Line"),
                AANonTerminal::SetUp => write!(f, r"SetUp"),
            }
        }
    }

    impl lalr1::Parser<AATerminal, AANonTerminal, AttributeData> for Calc {
        fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<AATerminal> {
            &AALEXAN
        }

        fn viable_error_recovery_states(token: &AATerminal) -> BTreeSet<u32> {
            match token {
                AATerminal::AAEnd => btree_set![0, 4],
                AATerminal::EOL => btree_set![0, 4],
                _ => btree_set![],
            }
        }

        fn error_goto_state(state: u32) -> u32 {
            match state {
                0 => 3,
                4 => 3,
                _ => panic!("No error go to state for {state}"),
            }
        }

        fn look_ahead_set(state: u32) -> BTreeSet<AATerminal> {
            use AATerminal::*;
            return match state {
                0 => btree_set![LPR, MINUS, EOL, ID, NUMBER, AAEnd],
                1 => btree_set![EOL, AAEnd],
                2 => btree_set![LPR, MINUS, ID, NUMBER],
                3 => btree_set![EOL, AAEnd],
                4 => btree_set![LPR, MINUS, EOL, ID, NUMBER, AAEnd],
                5 => btree_set![DIVIDE, MINUS, PLUS, TIMES, EOL, AAEnd],
                6 => btree_set![ASSIGN, DIVIDE, MINUS, PLUS, TIMES, EOL, AAEnd],
                7 => btree_set![LPR, MINUS, ID, NUMBER],
                8 => btree_set![LPR, MINUS, ID, NUMBER],
                9 => btree_set![DIVIDE, MINUS, PLUS, RPR, TIMES, EOL, AAEnd],
                10 => btree_set![EOL, AAEnd],
                11 => btree_set![LPR, MINUS, ID, NUMBER],
                12 => btree_set![LPR, MINUS, ID, NUMBER],
                13 => btree_set![LPR, MINUS, ID, NUMBER],
                14 => btree_set![LPR, MINUS, ID, NUMBER],
                15 => btree_set![LPR, MINUS, ID, NUMBER],
                16 => btree_set![DIVIDE, MINUS, PLUS, RPR, TIMES],
                17 => btree_set![DIVIDE, MINUS, PLUS, RPR, TIMES, EOL, AAEnd],
                18 => btree_set![DIVIDE, MINUS, PLUS, RPR, TIMES, EOL, AAEnd],
                19 => btree_set![DIVIDE, MINUS, PLUS, RPR, TIMES, EOL, AAEnd],
                20 => btree_set![DIVIDE, MINUS, PLUS, RPR, TIMES, EOL, AAEnd],
                21 => btree_set![DIVIDE, MINUS, PLUS, RPR, TIMES, EOL, AAEnd],
                22 => btree_set![DIVIDE, MINUS, PLUS, RPR, TIMES, EOL, AAEnd],
                23 => btree_set![DIVIDE, MINUS, PLUS, TIMES, EOL, AAEnd],
                24 => btree_set![DIVIDE, MINUS, PLUS, RPR, TIMES, EOL, AAEnd],
                _ => panic!("illegal state: {state}"),
            };
        }

        fn next_action(&self, aa_state: u32, aa_token: &lexan::Token<AATerminal>) -> lalr1::Action {
            use lalr1::Action;
            use AATerminal::*;
            let aa_tag = *aa_token.tag();
            return match aa_state {
                0 => match aa_tag {
                    // SetUp: <empty> #(NonAssoc, 0)
                    LPR | MINUS | ID | NUMBER => Action::Reduce(6),
                    // AAError: <empty> #(NonAssoc, 0)
                    EOL | AAEnd => Action::Reduce(15),
                    _ => Action::SyntaxError,
                },
                1 => match aa_tag {
                    EOL => Action::Shift(4),
                    // AAStart: Line #(NonAssoc, 0)
                    AAEnd => Action::Accept,
                    _ => Action::SyntaxError,
                },
                2 => match aa_tag {
                    LPR => Action::Shift(7),
                    MINUS => Action::Shift(8),
                    ID => Action::Shift(6),
                    NUMBER => Action::Shift(9),
                    _ => Action::SyntaxError,
                },
                3 => match aa_tag {
                    // Line: AAError #(NonAssoc, 0)
                    EOL | AAEnd => Action::Reduce(5),
                    _ => Action::SyntaxError,
                },
                4 => match aa_tag {
                    // Line: Line EOL #(Left, 1)
                    EOL | AAEnd => Action::Reduce(4),
                    // SetUp: <empty> #(NonAssoc, 0)
                    LPR | MINUS | ID | NUMBER => Action::Reduce(6),
                    _ => Action::SyntaxError,
                },
                5 => match aa_tag {
                    DIVIDE => Action::Shift(14),
                    MINUS => Action::Shift(12),
                    PLUS => Action::Shift(11),
                    TIMES => Action::Shift(13),
                    // Line: SetUp Expr #(NonAssoc, 0)
                    EOL | AAEnd => Action::Reduce(1),
                    _ => Action::SyntaxError,
                },
                6 => match aa_tag {
                    ASSIGN => Action::Shift(15),
                    // Expr: ID #(NonAssoc, 0)
                    DIVIDE | MINUS | PLUS | TIMES | EOL | AAEnd => Action::Reduce(14),
                    _ => Action::SyntaxError,
                },
                7 => match aa_tag {
                    LPR => Action::Shift(7),
                    MINUS => Action::Shift(8),
                    ID => Action::Shift(17),
                    NUMBER => Action::Shift(9),
                    _ => Action::SyntaxError,
                },
                8 => match aa_tag {
                    LPR => Action::Shift(7),
                    MINUS => Action::Shift(8),
                    ID => Action::Shift(17),
                    NUMBER => Action::Shift(9),
                    _ => Action::SyntaxError,
                },
                9 => match aa_tag {
                    // Expr: NUMBER #(NonAssoc, 0)
                    DIVIDE | MINUS | PLUS | RPR | TIMES | EOL | AAEnd => Action::Reduce(13),
                    _ => Action::SyntaxError,
                },
                10 => match aa_tag {
                    // Line: Line EOL Line #(Left, 1)
                    EOL | AAEnd => Action::Reduce(3),
                    _ => Action::SyntaxError,
                },
                11 => match aa_tag {
                    LPR => Action::Shift(7),
                    MINUS => Action::Shift(8),
                    ID => Action::Shift(17),
                    NUMBER => Action::Shift(9),
                    _ => Action::SyntaxError,
                },
                12 => match aa_tag {
                    LPR => Action::Shift(7),
                    MINUS => Action::Shift(8),
                    ID => Action::Shift(17),
                    NUMBER => Action::Shift(9),
                    _ => Action::SyntaxError,
                },
                13 => match aa_tag {
                    LPR => Action::Shift(7),
                    MINUS => Action::Shift(8),
                    ID => Action::Shift(17),
                    NUMBER => Action::Shift(9),
                    _ => Action::SyntaxError,
                },
                14 => match aa_tag {
                    LPR => Action::Shift(7),
                    MINUS => Action::Shift(8),
                    ID => Action::Shift(17),
                    NUMBER => Action::Shift(9),
                    _ => Action::SyntaxError,
                },
                15 => match aa_tag {
                    LPR => Action::Shift(7),
                    MINUS => Action::Shift(8),
                    ID => Action::Shift(17),
                    NUMBER => Action::Shift(9),
                    _ => Action::SyntaxError,
                },
                16 => match aa_tag {
                    DIVIDE => Action::Shift(14),
                    MINUS => Action::Shift(12),
                    PLUS => Action::Shift(11),
                    RPR => Action::Shift(24),
                    TIMES => Action::Shift(13),
                    _ => Action::SyntaxError,
                },
                17 => match aa_tag {
                    // Expr: ID #(NonAssoc, 0)
                    DIVIDE | MINUS | PLUS | RPR | TIMES | EOL | AAEnd => Action::Reduce(14),
                    _ => Action::SyntaxError,
                },
                18 => match aa_tag {
                    // Expr: "-" Expr #(Right, 4)
                    DIVIDE | MINUS | PLUS | RPR | TIMES | EOL | AAEnd => Action::Reduce(12),
                    _ => Action::SyntaxError,
                },
                19 => match aa_tag {
                    DIVIDE => Action::Shift(14),
                    TIMES => Action::Shift(13),
                    // Expr: Expr "+" Expr #(Left, 2)
                    MINUS | PLUS | RPR | EOL | AAEnd => Action::Reduce(7),
                    _ => Action::SyntaxError,
                },
                20 => match aa_tag {
                    DIVIDE => Action::Shift(14),
                    TIMES => Action::Shift(13),
                    // Expr: Expr "-" Expr #(Left, 2)
                    MINUS | PLUS | RPR | EOL | AAEnd => Action::Reduce(8),
                    _ => Action::SyntaxError,
                },
                21 => match aa_tag {
                    // Expr: Expr "*" Expr #(Left, 3)
                    DIVIDE | MINUS | PLUS | RPR | TIMES | EOL | AAEnd => Action::Reduce(9),
                    _ => Action::SyntaxError,
                },
                22 => match aa_tag {
                    // Expr: Expr "/" Expr #(Left, 3)
                    DIVIDE | MINUS | PLUS | RPR | TIMES | EOL | AAEnd => Action::Reduce(10),
                    _ => Action::SyntaxError,
                },
                23 => match aa_tag {
                    DIVIDE => Action::Shift(14),
                    MINUS => Action::Shift(12),
                    PLUS => Action::Shift(11),
                    TIMES => Action::Shift(13),
                    // Line: SetUp ID "=" Expr #(NonAssoc, 0)
                    EOL | AAEnd => Action::Reduce(2),
                    _ => Action::SyntaxError,
                },
                24 => match aa_tag {
                    // Expr: "(" Expr ")" #(NonAssoc, 0)
                    DIVIDE | MINUS | PLUS | RPR | TIMES | EOL | AAEnd => Action::Reduce(11),
                    _ => Action::SyntaxError,
                },
                _ => panic!("illegal state: {aa_state}"),
            };
        }

        fn production_data(production_id: u32) -> (AANonTerminal, usize) {
            match production_id {
                0 => (AANonTerminal::AAStart, 1),
                1 => (AANonTerminal::Line, 2),
                2 => (AANonTerminal::Line, 4),
                3 => (AANonTerminal::Line, 3),
                4 => (AANonTerminal::Line, 2),
                5 => (AANonTerminal::Line, 1),
                6 => (AANonTerminal::SetUp, 0),
                7 => (AANonTerminal::Expr, 3),
                8 => (AANonTerminal::Expr, 3),
                9 => (AANonTerminal::Expr, 3),
                10 => (AANonTerminal::Expr, 3),
                11 => (AANonTerminal::Expr, 3),
                12 => (AANonTerminal::Expr, 2),
                13 => (AANonTerminal::Expr, 1),
                14 => (AANonTerminal::Expr, 1),
                15 => (AANonTerminal::AAError, 0),
                _ => panic!("malformed production data table"),
            }
        }

        fn goto_state(lhs: &AANonTerminal, current_state: u32) -> u32 {
            return match current_state {
                0 => match lhs {
                    AANonTerminal::Line => 1,
                    AANonTerminal::SetUp => 2,
                    AANonTerminal::AAError => 3,
                    _ => panic!("Malformed goto table: ({lhs}, {current_state})"),
                },
                2 => match lhs {
                    AANonTerminal::Expr => 5,
                    _ => panic!("Malformed goto table: ({lhs}, {current_state})"),
                },
                4 => match lhs {
                    AANonTerminal::Line => 10,
                    AANonTerminal::SetUp => 2,
                    AANonTerminal::AAError => 3,
                    _ => panic!("Malformed goto table: ({lhs}, {current_state})"),
                },
                7 => match lhs {
                    AANonTerminal::Expr => 16,
                    _ => panic!("Malformed goto table: ({lhs}, {current_state})"),
                },
                8 => match lhs {
                    AANonTerminal::Expr => 18,
                    _ => panic!("Malformed goto table: ({lhs}, {current_state})"),
                },
                11 => match lhs {
                    AANonTerminal::Expr => 19,
                    _ => panic!("Malformed goto table: ({lhs}, {current_state})"),
                },
                12 => match lhs {
                    AANonTerminal::Expr => 20,
                    _ => panic!("Malformed goto table: ({lhs}, {current_state})"),
                },
                13 => match lhs {
                    AANonTerminal::Expr => 21,
                    _ => panic!("Malformed goto table: ({lhs}, {current_state})"),
                },
                14 => match lhs {
                    AANonTerminal::Expr => 22,
                    _ => panic!("Malformed goto table: ({lhs}, {current_state})"),
                },
                15 => match lhs {
                    AANonTerminal::Expr => 23,
                    _ => panic!("Malformed goto table: ({lhs}, {current_state})"),
                },
                _ => panic!("Malformed goto table: ({lhs}, {current_state})"),
            };
        }

        fn do_semantic_action<F: FnMut(String, String)>(
            &mut self,
            aa_production_id: u32,
            aa_rhs: Vec<AttributeData>,
            mut aa_inject: F,
        ) -> AttributeData {
            let mut aa_lhs = if let Some(a) = aa_rhs.first() {
                a.clone()
            } else {
                AttributeData::default()
            };
            match aa_production_id {
                1 => {
                    // Line: SetUp Expr #(NonAssoc, 0)

                    if self.errors > 0 {
                        self.report_errors();
                    } else {
                        println!("{}", aa_rhs[1].value());
                    }
                }
                2 => {
                    // Line: SetUp ID "=" Expr #(NonAssoc, 0)

                    if self.errors > 0 {
                        self.report_errors();
                    } else {
                        self.variables
                            .insert(aa_rhs[1].id().clone(), aa_rhs[3].value());
                    }
                }
                5 => {
                    // Line: AAError #(NonAssoc, 0)
                    self.errors |= SYNTAX_ERROR;
                }
                6 => {
                    // SetUp: <empty> #(NonAssoc, 0)
                    self.errors = 0;
                }
                7 => {
                    // Expr: Expr "+" Expr #(Left, 2)
                    aa_lhs = AttributeData::Value(aa_rhs[0].value() + aa_rhs[2].value());
                }
                8 => {
                    // Expr: Expr "-" Expr #(Left, 2)
                    aa_lhs = AttributeData::Value(aa_rhs[0].value() - aa_rhs[2].value());
                }
                9 => {
                    // Expr: Expr "*" Expr #(Left, 3)
                    aa_lhs = AttributeData::Value(aa_rhs[0].value() * aa_rhs[2].value());
                }
                10 => {
                    // Expr: Expr "/" Expr #(Left, 3)

                    if aa_rhs[2].value() == 0.0 {
                        self.errors |= DIVIDE_BY_ZERO;
                    } else {
                        aa_lhs = AttributeData::Value(aa_rhs[0].value() / aa_rhs[2].value());
                    }
                }
                11 => {
                    // Expr: "(" Expr ")" #(NonAssoc, 0)
                    aa_lhs = AttributeData::Value(aa_rhs[1].value());
                }
                12 => {
                    // Expr: "-" Expr #(Right, 4)
                    aa_lhs = AttributeData::Value(-aa_rhs[1].value());
                }
                13 => {
                    // Expr: NUMBER #(NonAssoc, 0)
                    aa_lhs = AttributeData::Value(aa_rhs[0].value());
                }
                14 => {
                    // Expr: ID #(NonAssoc, 0)

                    match self.variable(aa_rhs[0].id()) {
                        Some(value) => aa_lhs = AttributeData::Value(value),
                        None => {
                            aa_lhs = AttributeData::Value(0.0);
                            self.errors |= UNDEFINED_VARIABLE;
                        }
                    }
                }
                _ => aa_inject(String::new(), String::new()),
            };
            aa_lhs
        }
    }

    #[test]
    fn calc_works() {
        use crate::Parser;
        let mut calc = Calc::new();
        assert!(calc.parse_text("a = (3 + 4)\n", "raw").is_ok());
        assert_eq!(calc.variables.get("a"), Some(&7.0));
        assert!(calc.parse_text("b = a * 5\n", "raw").is_ok());
        assert_eq!(calc.variables.get("b"), Some(&35.0));
    }
}
