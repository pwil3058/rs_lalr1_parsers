// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use std::io::Write;
use std::{
    default::Default,
    fmt::{self, Debug, Display},
};
use thiserror::Error;

use lexan::TokenStream;

// Create a wrapper around BTreeSet so we can implement Display on it
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OrderedSet<T: Display + Ord + Clone>(pub std::collections::BTreeSet<T>);

impl<T: Display + Clone + Ord> OrderedSet<T> {
    pub fn new() -> Self {
        Self(std::collections::BTreeSet::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, item: &T) -> bool {
        self.0.contains(item)
    }

    pub fn insert(&mut self, item: T) -> bool {
        self.0.insert(item)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
}

impl<T: Ord + Display + Clone> FromIterator<T> for OrderedSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let btree_set = std::collections::BTreeSet::from_iter(iter);
        Self(btree_set)
    }
}
impl<'a, T: Ord + Display + Clone> IntoIterator for &'a OrderedSet<T> {
    type Item = &'a T;
    type IntoIter = <&'a std::collections::BTreeSet<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (self.0).iter()
    }
}

impl<T: Display + Ord + Clone> Display for OrderedSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();
        for (index, item) in self.0.iter().enumerate() {
            if index == 0 {
                string += &item.to_string();
            } else {
                string += ", ";
                string += &item.to_string()
            }
        }
        write!(f, "{}", string)
    }
}

#[derive(Debug, Clone, Error)]
pub enum Error<T: Ord + Clone + Copy + Debug + Display + Eq> {
    #[error("Lexical error: {0} expected {1}.")]
    LexicalError(lexan::Error<T>, OrderedSet<T>),
    #[error("Syntax error: {0} expected {1}.")]
    SyntaxError(lexan::Token<T>, OrderedSet<T>),
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

#[derive(Debug, Error)]
pub enum GrammarError {
    #[error("Too many errors: {0}")]
    TooManyErrors(u32),
    #[error("{0} undefined symbols")]
    UndefinedSymbols(u32),
    #[error("Unexpected Shift/Reduce conflicts: {0} {1} {2}")]
    UnexpectedSRConflicts(u32, u32, String),
    #[error("Unexpected Reduce/Reduce conflicts: {0} {1} {2}")]
    UnexpectedRRConflicts(u32, u32, String),
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

    pub fn current_state(&self) -> u32 {
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

    fn distance_to_viable_state<F: Fn(&T) -> OrderedSet<u32>>(
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
    fn next_action(&self, parse_stack: &ParseStack<T, N, A>, o_token: &lexan::Token<T>) -> Action;
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

    fn viable_error_recovery_states(tag: &T) -> OrderedSet<u32>;

    fn error_goto_state(state: u32) -> u32 {
        panic!("No error go to state for {state}")
    }

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
                Ok(token) => match self.next_action(&parse_stack, &token) {
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
mod tests;
