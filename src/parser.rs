pub use std::{
    convert::From,
    default::Default,
    fmt::{self, Debug, Display},
    io::Write,
};

use lexan;

#[derive(Debug, Clone)]
pub enum Error<T: Copy + Debug + Display + Eq> {
    LexicalError(lexan::Error<T>),
    SyntaxError(lexan::Token<T>, Vec<T>),
}

fn format_vec<T: Display>(vec: &Vec<T>) -> String {
    let mut string = String::new();
    let last = vec.len() - 1;
    for (index, item) in vec.iter().enumerate() {
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

impl<T: Copy + Debug + Display + Eq> Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::LexicalError(lex_err) => write!(f, "Lexical Error: {}.", lex_err),
            Error::SyntaxError(found, expected) => write!(
                f,
                "Syntax Error: expected: {} found: {} at: {}.",
                format_vec(&expected),
                found.tag(),
                found.location()
            ),
        }
    }
}

pub trait ReportError<T: Copy + Debug + Display + Eq> {
    fn report_error(&mut self, error: &Error<T>) {
        let message = error.to_string();
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

    fn distance_to_viable_state(&mut self, viable_states: &[u32]) -> Option<usize> {
        if viable_states.len() > 0 {
            for sub in 1..self.states.len() {
                let candidate = self.states[self.states.len() - 1].1;
                if viable_states.contains(&candidate) {
                    if let Some(last_error_state) = self.last_error_state {
                        if candidate == last_error_state {
                            continue;
                        }
                    }
                    self.last_error_state = Some(candidate);
                    return Some(sub - 1);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub enum Action<T: Copy + Debug> {
    Shift(u32),
    Reduce(u32),
    Accept,
    SyntaxError(Vec<T>),
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
    ) -> Action<T>;
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

    fn parse_text(&mut self, text: String, label: String) -> Result<(), Error<T>> {
        let mut tokens = self.lexical_analyzer().token_stream(text, label);
        let mut parse_stack = ParseStack::<T, N, A>::new();
        let mut result: Result<(), Error<T>> = Ok(());

        loop {
            match tokens.front() {
                Err(err) => {
                    let error = Error::LexicalError(err);
                    self.report_error(&error);
                    result = Err(error.clone());
                    // Sensible thing is to just skip the bad data but ...
                    // TODO: think about some error recovery stuff here
                    parse_stack.push_error(parse_stack.current_state(), error);
                    tokens.advance();
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
                        Action::SyntaxError(expected) => {
                            let error = Error::SyntaxError(token.clone(), expected);
                            self.report_error(&error);
                            result = Err(error.clone());
                            let viable_states = Self::viable_error_recovery_states(token.tag());
                            let mut distance = parse_stack.distance_to_viable_state(&viable_states);
                            while distance.is_none() && !tokens.is_empty() {
                                if let Ok(token) = tokens.advance_front() {
                                    let viable_states =
                                        Self::viable_error_recovery_states(token.tag());
                                    distance = parse_stack.distance_to_viable_state(&viable_states);
                                }
                            }
                            if let Some(distance) = distance {
                                parse_stack.pop_n(distance);
                                let next_state =
                                    Self::error_goto_state(parse_stack.current_state());
                                parse_stack.push_error(next_state, error);
                            } else {
                                return result;
                            }
                        }
                    }
                }
            };
        }
    }
}
