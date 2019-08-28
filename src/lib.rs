#[cfg(test)]
#[macro_use]
extern crate lazy_static;
extern crate lexan;
extern crate ordered_collections;

pub use std::{
    convert::From,
    default::Default,
    fmt::{self, Debug, Display},
    io::Write,
};

use lexan::TokenStream;
use ordered_collections::OrderedSet;

#[derive(Debug, Clone)]
pub enum Error<T: Ord + Copy + Debug + Display + Eq> {
    LexicalError(lexan::Error<T>, OrderedSet<T>),
    SyntaxError(lexan::Token<T>, OrderedSet<T>),
}

fn format_set<T: Ord + Display>(set: &OrderedSet<T>) -> String {
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
            Error::LexicalError(lex_err, expected) => {
                write!(f, "Lexical Error: {}: expected: {}.", lex_err, expected)
            }
            Error::SyntaxError(found, expected) => write!(
                f,
                "Syntax Error: expected: {} found: {} at: {}.",
                format_set(&expected),
                found.tag(),
                found.location()
            ),
        }
    }
}

pub trait ReportError<T: Ord + Copy + Debug + Display + Eq> {
    fn report_error(&mut self, error: &Error<T>) {
        let message = error.to_string();
        if let Error::LexicalError(lex_err, _) = error {
            if let lexan::Error::AmbiguousMatches(_, _, _) = lex_err {
                panic!("Fatal Error: {}!!", message);
            }
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
    End,
    Error,
    Invalid,
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

    pub fn at_len_minus_n<'a>(&'a self, n: usize) -> &'a A {
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

    fn distance_to_viable_state<F: Fn(&T) -> Vec<u32>>(
        &mut self,
        tokens: &mut lexan::TokenStream<T>,
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
    fn next_action(
        &self,
        state: u32,
        attributes: &ParseStack<T, N, A>,
        o_token: &lexan::Token<T>,
    ) -> Action;
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

    fn viable_error_recovery_states(tag: &T) -> Vec<u32>;

    fn error_goto_state(state: u32) -> u32;

    fn look_ahead_set(state: u32) -> OrderedSet<T>;

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

    fn parse_text(&mut self, text: String, label: String) -> Result<(), Error<T>> {
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
                Ok(token) => {
                    match self.next_action(parse_stack.current_state(), &parse_stack, &token) {
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
                                .do_semantic_action(production_id, rhs, |s, l| tokens.inject(s, l));
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
                    }
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ReportError;
    use ordered_collections::OrderedSet;
    use std::collections::HashMap;
    use std::convert::From;
    use std::fmt;
    use std::str::FromStr;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum Terminal {
        Plus,
        Minus,
        Times,
        Divide,
        Assign,
        LPR,
        RPR,
        EOL,
        Number,
        Id,
        EndMarker,
    }

    impl fmt::Display for Terminal {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Terminal::Plus => write!(f, "+"),
                Terminal::Minus => write!(f, "-"),
                Terminal::Times => write!(f, "*"),
                Terminal::Divide => write!(f, "/"),
                Terminal::Assign => write!(f, "="),
                Terminal::LPR => write!(f, "("),
                Terminal::RPR => write!(f, ")"),
                Terminal::EOL => write!(f, "EOL"),
                Terminal::Number => write!(f, "Number"),
                Terminal::Id => write!(f, "Id"),
                Terminal::EndMarker => write!(f, "EndMarker"),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum NonTerminal {
        Line,
        SetUp,
        Expr,
    }

    impl fmt::Display for NonTerminal {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                NonTerminal::Line => write!(f, "Line"),
                NonTerminal::SetUp => write!(f, "SetUp"),
                NonTerminal::Expr => write!(f, "Expr"),
            }
        }
    }

    #[derive(Debug, Default, Clone)]
    struct AttributeData {
        id: String,
        value: f64,
    }

    impl From<lexan::Token<Terminal>> for AttributeData {
        fn from(input: lexan::Token<Terminal>) -> Self {
            let mut attr = AttributeData::default();
            match input.tag() {
                Terminal::Number => {
                    attr.value = f64::from_str(input.lexeme()).unwrap();
                }
                Terminal::Id => {
                    attr.id = input.lexeme().to_string();
                }
                _ => (),
            };
            attr
        }
    }

    impl From<crate::Error<Terminal>> for AttributeData {
        fn from(_error: crate::Error<Terminal>) -> Self {
            AttributeData::default()
        }
    }

    const UNDEFINED_VARIABLE: u32 = 1 << 0;
    const DIVIDE_BY_ZERO: u32 = 1 << 1;
    const SYNTAX_ERROR: u32 = 1 << 2;
    const LEXICAL_ERROR: u32 = 1 << 3;

    struct Calc {
        errors: u32,
        variables: HashMap<String, f64>,
    }

    impl ReportError<Terminal> for Calc {}

    lazy_static! {
        static ref AALEXAN: lexan::LexicalAnalyzer<Terminal> = {
            use Terminal::*;
            lexan::LexicalAnalyzer::new(
                &[
                    (Plus, "+"),
                    (Minus, "-"),
                    (Times, "*"),
                    (Divide, "/"),
                    (Assign, "="),
                    (LPR, "("),
                    (RPR, ")"),
                ],
                &[
                    (EOL, r"(\n)"),
                    (Number, r"([0-9]+(\.[0-9]+){0,1})"),
                    (Id, r"([a-zA-Z]+)"),
                ],
                &[r"([\t\r ]+)"],
                EndMarker,
            )
        };
    }

    impl Calc {
        pub fn new() -> Self {
            Self {
                errors: 0,
                variables: HashMap::new(),
            }
        }

        fn report_errors(&self) {
            if self.errors == 0 {
                println!("no errrs")
            } else {
                if self.errors & UNDEFINED_VARIABLE == UNDEFINED_VARIABLE {
                    println!("undefined variable errors")
                }
                if self.errors & DIVIDE_BY_ZERO == DIVIDE_BY_ZERO {
                    println!("divide by zero errors")
                }
                if self.errors & SYNTAX_ERROR == SYNTAX_ERROR {
                    println!("syntax errors")
                }
                if self.errors & LEXICAL_ERROR == LEXICAL_ERROR {
                    println!("lexical errors")
                }
            }
            println!("#errors = {}", self.errors)
        }
    }

    impl crate::Parser<Terminal, NonTerminal, AttributeData> for Calc {
        fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<Terminal> {
            &AALEXAN
        }

        fn viable_error_recovery_states(tag: &Terminal) -> Vec<u32> {
            use Terminal::*;
            match tag {
                EOL => vec![0, 4],
                _ => vec![],
            }
        }

        fn error_goto_state(state: u32) -> u32 {
            match state {
                0 | 4 => 3,
                _ => panic!("No error go to state for {}", state),
            }
        }

        fn look_ahead_set(state: u32) -> OrderedSet<Terminal> {
            use Terminal::*;
            return match state {
                0 => vec![Minus, LPR, Number, Id].into(),
                1 => vec![EndMarker, EOL].into(),
                2 => vec![Minus, LPR, Number, Id].into(),
                3 => vec![EndMarker, EOL].into(),
                4 => vec![EndMarker, EOL, Minus, Number, Id, LPR].into(),
                5 => vec![EndMarker, EOL, Plus, Minus, Times, Divide].into(),
                6 => vec![EndMarker, EOL, Plus, Minus, Times, Divide, Assign].into(),
                7 | 8 => vec![Minus, Number, Id, LPR].into(),
                9 => vec![EndMarker, EOL, Plus, Minus, Times, Divide, RPR].into(),
                10 => vec![EndMarker, EOL].into(),
                11 | 12 | 13 | 14 | 15 => vec![Minus, Number, Id, LPR].into(),
                16 => vec![Plus, Minus, Times, Divide, RPR].into(),
                17 => vec![EndMarker, EOL, Plus, Minus, Times, Divide, RPR].into(),
                18 => vec![EndMarker, EOL, Plus, Minus, Times, Divide, RPR].into(),
                19 => vec![EndMarker, EOL, Plus, Minus, Times, Divide, RPR].into(),
                20 => vec![EndMarker, EOL, Plus, Minus, Times, Divide, RPR].into(),
                21 => vec![EndMarker, EOL, Plus, Minus, Times, Divide, RPR].into(),
                22 => vec![EndMarker, EOL, Plus, Minus, Times, Divide, RPR].into(),
                23 => vec![EndMarker, EOL, Plus, Minus, Times, Divide, RPR].into(),
                24 => vec![EndMarker, EOL, Plus, Minus, Times, Divide, RPR].into(),
                _ => panic!("illegal state: {}", state),
            };
        }

        fn next_action(
            &self,
            state: u32,
            attributes: &crate::ParseStack<Terminal, NonTerminal, AttributeData>,
            token: &lexan::Token<Terminal>,
        ) -> crate::Action {
            use crate::Action;
            use Terminal::*;
            let tag = *token.tag();
            return match state {
                0 => match tag {
                    Minus | LPR | Number | Id => Action::Reduce(8),
                    _ => Action::SyntaxError,
                },
                1 => match tag {
                    EndMarker => Action::Accept,
                    EOL => Action::Shift(4),
                    _ => Action::SyntaxError,
                },
                2 => match tag {
                    Minus => Action::Shift(8),
                    LPR => Action::Shift(7),
                    Number => Action::Shift(9),
                    Id => Action::Shift(6),
                    _ => Action::SyntaxError,
                },
                3 => match tag {
                    EndMarker | EOL => Action::Reduce(7),
                    _ => Action::SyntaxError,
                },
                4 => match tag {
                    EndMarker | EOL => Action::Reduce(6),
                    Minus | Number | Id | LPR => Action::Reduce(8),
                    _ => Action::SyntaxError,
                },
                5 => match tag {
                    Plus => Action::Shift(11),
                    Minus => Action::Shift(12),
                    Times => Action::Shift(13),
                    Divide => Action::Shift(14),
                    EndMarker | EOL => {
                        if self.errors > 0 {
                            Action::Reduce(1)
                        } else {
                            Action::Reduce(2)
                        }
                    }
                    _ => Action::SyntaxError,
                },
                6 => match tag {
                    Assign => Action::Shift(15),
                    EndMarker | EOL | Plus | Minus | Times | Divide => {
                        if self
                            .variables
                            .contains_key(&attributes.at_len_minus_n(2 - 1).id)
                        {
                            Action::Reduce(26)
                        } else {
                            Action::Reduce(27)
                        }
                    }
                    _ => Action::SyntaxError,
                },
                7 | 8 => match tag {
                    Minus => Action::Shift(8),
                    LPR => Action::Shift(7),
                    Number => Action::Shift(9),
                    Id => Action::Shift(17),
                    _ => Action::SyntaxError,
                },
                9 => match tag {
                    EndMarker | EOL | Plus | Minus | Times | Divide | RPR => Action::Reduce(25),
                    _ => Action::SyntaxError,
                },
                10 => match tag {
                    EndMarker | EOL => Action::Reduce(5),
                    _ => Action::SyntaxError,
                },
                11 | 12 | 13 | 14 | 15 => match tag {
                    Minus => Action::Shift(8),
                    LPR => Action::Shift(7),
                    Number => Action::Shift(9),
                    Id => Action::Shift(17),
                    _ => Action::SyntaxError,
                },
                16 => match tag {
                    Plus => Action::Shift(11),
                    Minus => Action::Shift(12),
                    Times => Action::Shift(13),
                    Divide => Action::Shift(14),
                    RPR => Action::Shift(24),
                    _ => Action::SyntaxError,
                },
                17 => match tag {
                    EndMarker | EOL | Plus | Minus | Times | Divide | RPR => {
                        if self
                            .variables
                            .contains_key(&attributes.at_len_minus_n(2 - 1).id)
                        {
                            Action::Reduce(26)
                        } else {
                            Action::Reduce(27)
                        }
                    }
                    _ => Action::SyntaxError,
                },
                18 => match tag {
                    EndMarker | EOL | Plus | Minus | Times | Divide | RPR => Action::Reduce(24),
                    _ => Action::SyntaxError,
                },
                19 => match tag {
                    Times => Action::Shift(13),
                    Divide => Action::Shift(14),
                    EndMarker | EOL | Plus | Minus | RPR => {
                        if attributes.at_len_minus_n(4 - 1).value == 0.0 {
                            Action::Reduce(9)
                        } else if attributes.at_len_minus_n(4 - 3).value == 0.0 {
                            Action::Reduce(10)
                        } else {
                            Action::Reduce(11)
                        }
                    }
                    _ => Action::SyntaxError,
                },
                20 => match tag {
                    Times => Action::Shift(13),
                    Divide => Action::Shift(14),
                    EndMarker | EOL | Plus | Minus | RPR => {
                        if attributes.at_len_minus_n(4 - 1).value == 0.0 {
                            Action::Reduce(12)
                        } else if attributes.at_len_minus_n(4 - 3).value == 0.0 {
                            Action::Reduce(13)
                        } else {
                            Action::Reduce(14)
                        }
                    }
                    _ => Action::SyntaxError,
                },
                21 => match tag {
                    EndMarker | EOL | Plus | Minus | Times | Divide | RPR => {
                        if attributes.at_len_minus_n(4 - 1).value == 0.0
                            || attributes.at_len_minus_n(4 - 3).value == 0.0
                        {
                            Action::Reduce(15)
                        } else if attributes.at_len_minus_n(4 - 1).value == 1.0 {
                            Action::Reduce(16)
                        } else if attributes.at_len_minus_n(4 - 3).value == 1.0 {
                            Action::Reduce(17)
                        } else {
                            Action::Reduce(18)
                        }
                    }
                    _ => Action::SyntaxError,
                },
                22 => match tag {
                    EndMarker | EOL | Plus | Minus | Times | Divide | RPR => {
                        if attributes.at_len_minus_n(4 - 1).value == 0.0
                            || attributes.at_len_minus_n(4 - 3).value == 0.0
                        {
                            Action::Reduce(19)
                        } else if attributes.at_len_minus_n(4 - 1).value == 1.0 {
                            Action::Reduce(20)
                        } else if attributes.at_len_minus_n(4 - 3).value == 1.0 {
                            Action::Reduce(21)
                        } else {
                            Action::Reduce(22)
                        }
                    }
                    _ => Action::SyntaxError,
                },
                23 => match tag {
                    Plus => Action::Shift(11),
                    Minus => Action::Shift(12),
                    Times => Action::Shift(13),
                    Divide => Action::Shift(14),
                    EndMarker | EOL => {
                        if self.errors == 0 {
                            Action::Reduce(3)
                        } else {
                            Action::Reduce(4)
                        }
                    }
                    _ => Action::SyntaxError,
                },
                24 => match tag {
                    EndMarker | EOL | Plus | Minus | Times | Divide | RPR => Action::Reduce(23),
                    _ => Action::SyntaxError,
                },
                _ => panic!("illegal state: {}", state),
            };
        }

        fn production_data(production_id: u32) -> (NonTerminal, usize) {
            match production_id {
                1 => (NonTerminal::Line, 2),
                2 => (NonTerminal::Line, 2),
                3 => (NonTerminal::Line, 4),
                4 => (NonTerminal::Line, 4),
                5 => (NonTerminal::Line, 3),
                6 => (NonTerminal::Line, 2),
                7 => (NonTerminal::Line, 1),
                8 => (NonTerminal::SetUp, 0),
                9 => (NonTerminal::Expr, 3),
                10 => (NonTerminal::Expr, 3),
                11 => (NonTerminal::Expr, 3),
                12 => (NonTerminal::Expr, 3),
                13 => (NonTerminal::Expr, 3),
                14 => (NonTerminal::Expr, 3),
                15 => (NonTerminal::Expr, 3),
                16 => (NonTerminal::Expr, 3),
                17 => (NonTerminal::Expr, 3),
                18 => (NonTerminal::Expr, 3),
                19 => (NonTerminal::Expr, 3),
                20 => (NonTerminal::Expr, 3),
                21 => (NonTerminal::Expr, 3),
                22 => (NonTerminal::Expr, 3),
                23 => (NonTerminal::Expr, 3),
                24 => (NonTerminal::Expr, 2),
                25 => (NonTerminal::Expr, 1),
                26 => (NonTerminal::Expr, 1),
                27 => (NonTerminal::Expr, 1),
                _ => panic!("malformed production data table"),
            }
        }

        fn goto_state(lhs: &NonTerminal, current_state: u32) -> u32 {
            match current_state {
                0 => match lhs {
                    NonTerminal::Line => 1,
                    NonTerminal::SetUp => 2,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                2 => match lhs {
                    NonTerminal::Expr => 5,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                4 => match lhs {
                    NonTerminal::Line => 10,
                    NonTerminal::SetUp => 2,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                7 => match lhs {
                    NonTerminal::Expr => 16,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                8 => match lhs {
                    NonTerminal::Expr => 18,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                11 => match lhs {
                    NonTerminal::Expr => 19,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                12 => match lhs {
                    NonTerminal::Expr => 20,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                13 => match lhs {
                    NonTerminal::Expr => 21,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                14 => match lhs {
                    NonTerminal::Expr => 22,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                15 => match lhs {
                    NonTerminal::Expr => 23,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            }
        }

        fn do_semantic_action<F: FnMut(String, String)>(
            &mut self,
            production_id: u32,
            rhs: Vec<AttributeData>,
            mut inject: F,
        ) -> AttributeData {
            let mut lhs = AttributeData::default();
            // test that multiple injects are OK
            inject(String::new(), String::new());
            inject(String::new(), String::new());
            match production_id {
                1 | 4 => {
                    self.report_errors();
                }
                2 => {
                    println!("{}", rhs[2 - 1].value);
                }
                3 => {
                    self.variables
                        .insert(rhs[2 - 1].id.clone(), rhs[4 - 1].value);
                }
                7 => {
                    self.errors |= SYNTAX_ERROR;
                }
                8 => {
                    self.errors = 0;
                }
                9 => {
                    lhs.value = rhs[3 - 1].value;
                }
                10 => {
                    lhs.value = rhs[1 - 1].value;
                }
                11 => {
                    lhs.value = rhs[1 - 1].value + rhs[3 - 1].value;
                }
                12 => {
                    lhs.value = -rhs[3 - 1].value;
                }
                13 => {
                    lhs.value = rhs[1 - 1].value;
                }
                14 => {
                    lhs.value = rhs[1 - 1].value - rhs[3 - 1].value;
                }
                15 => {
                    lhs.value = -rhs[3 - 1].value;
                }
                16 => {
                    lhs.value = rhs[3 - 1].value;
                }
                17 => {
                    lhs.value = rhs[1 - 1].value;
                }
                18 => {
                    lhs.value = rhs[1 - 1].value * rhs[3 - 1].value;
                }
                19 => {
                    lhs.value = rhs[1 - 1].value;
                }
                20 => {
                    self.errors |= DIVIDE_BY_ZERO;
                }
                21 => {
                    lhs.value = 0.0;
                }
                22 => {
                    lhs.value = rhs[1 - 1].value / rhs[3 - 1].value;
                }
                23 => {
                    lhs.value = rhs[2 - 1].value;
                }
                24 => {
                    lhs.value = -rhs[2 - 1].value;
                }
                25 => {
                    lhs.value = rhs[1 - 1].value;
                }
                26 => {
                    lhs.value = *self.variables.get(&rhs[1 - 1].id).unwrap();
                }
                27 => {
                    self.errors |= UNDEFINED_VARIABLE;
                    lhs.value = 0.0;
                }
                _ => (),
            }
            lhs
        }
    }

    #[test]
    fn calc_works() {
        use crate::Parser;
        let mut calc = Calc::new();
        assert!(calc
            .parse_text("a = (3 + 4)\n".to_string(), "raw".to_string())
            .is_ok());
        assert_eq!(calc.variables.get("a"), Some(&7.0));
        assert!(calc
            .parse_text("b = a * 5\n".to_string(), "raw".to_string())
            .is_ok());
        assert_eq!(calc.variables.get("b"), Some(&35.0));
    }
}
