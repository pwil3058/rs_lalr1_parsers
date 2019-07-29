use std::{
    convert::From,
    default::Default,
    fmt::{Debug, Display},
};

use lexan;

#[derive(Debug)]
pub enum Error<'a, T: Copy + Debug> {
    LexicalError(lexan::Error<'a, T>),
    SyntaxError(T, Vec<T>, String),
    UnexpectedEndOfInput,
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
    T: Copy + Ord + Debug,
    A: From<(T, String)>,
{
    states: Vec<(Symbol<T, N>, u32)>,
    attributes: Vec<A>,
}

impl<T, N, A> ParseStack<T, N, A>
where
    T: Copy + Ord + Debug,
    A: From<(T, String)>,
{
    fn new() -> Self {
        Self {
            states: vec![(Symbol::Start, 0)],
            attributes: vec![],
        }
    }

    fn current_state(&self) -> u32 {
        self.states.last().unwrap().1
    }

    pub fn attribute_n_from_end<'a>(&'a self, n: usize) -> &'a A {
        let len = self.attributes.len();
        &self.attributes[len - n]
    }

    fn pop_n(&mut self, n: usize) -> Vec<A> {
        let len = self.states.len();
        self.states.truncate(len - n);
        let len = self.attributes.len();
        self.attributes.split_off(len - n)
    }

    fn push_terminal(&mut self, terminal: T, string: &str, new_state: u32) {
        let attribute = A::from((terminal, string.into()));
        self.attributes.push(attribute);
        self.states.push((Symbol::Terminal(terminal), new_state));
    }

    fn push_non_terminal(&mut self, non_terminal: N, new_state: u32) {
        self.states
            .push((Symbol::NonTerminal(non_terminal), new_state));
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Shift(u32),
    Reduce(u32),
}

#[derive(Debug, Clone)]
pub enum Coda {
    Reduce(u32),
    Accept,
    UnexpectedEndOfInput,
}

pub trait Parser<'t, T: Ord + Copy + Debug, N, A>
where
    T: Ord + Copy + Debug,
    N: Ord + Display + Debug,
    A: Default + From<(T, String)>,
{
    fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<T>;
    fn attribute<'b>(&'b self, attr_num: usize, num_attrs: usize) -> &'b A;
    fn pop_attributes(&mut self, n: usize) -> Vec<A>;
    fn push_attribute(&mut self, attribute: A);
    fn current_state(&self) -> u32;
    fn push_state(&mut self, state: u32, symbol: Symbol<T, N>);
    fn pop_states(&mut self, n: usize);
    fn next_action<'a>(
        &self,
        state: u32,
        attributes: &ParseStack<T, N, A>,
        o_token: &lexan::Token<'a, T>,
    ) -> Result<Action, Error<'a, T>>;
    fn next_coda<'a>(&self, state: u32, attributes: &ParseStack<T, N, A>) -> Coda;
    fn production_data(&mut self, production_id: u32) -> (N, usize);
    fn goto_state(lhs: &N, current_state: u32) -> u32;

    fn report_error(error: &Error<T>) {
        match error {
            Error::LexicalError(lex_err) => println!("Lexical Error: {}.", lex_err),
            Error::SyntaxError(found, expected, location) => println!(
                "Syntax Error: expected: {:?} found: {:?} at: {}.",
                expected, found, location
            ),
            Error::UnexpectedEndOfInput => println!("unexpected end of input."),
        }
    }

    fn short_circuit() -> bool {
        false
    }

    fn parse_text<'a>(&mut self, text: &'a str, label: &'a str) -> Result<(), Error<'a, T>> {
        let mut tokens = self.lexical_analyzer().injectable_token_stream(text, label);
        let mut parse_stack = ParseStack::<T, N, A>::new();
        let mut result: Result<(), Error<'a, T>> = Ok(());

        let mut o_r_token = tokens.next();
        while let Some(ref r_token) = o_r_token {
            match r_token {
                Err(err) => {
                    let err = Error::LexicalError(err.clone());
                    Self::report_error(&err);
                    result = Err(err);
                    if Self::short_circuit() {
                        return result;
                    }
                }
                Ok(token) => {
                    match self.next_action(parse_stack.current_state(), &parse_stack, &token) {
                        Ok(action) => match action {
                            Action::Shift(next_state) => {
                                parse_stack.push_terminal(*token.tag(), token.lexeme(), next_state);
                                o_r_token = tokens.next();
                            }
                            Action::Reduce(production_id) => {
                                let (lhs, rhs_len) = self.production_data(production_id);
                                let _rhs = parse_stack.pop_n(rhs_len);
                                let next_state =
                                    Self::goto_state(&lhs, parse_stack.current_state());
                                parse_stack.push_non_terminal(lhs, next_state);
                                self.push_attribute(A::default());
                                // TODO: do semantic actions here
                            }
                        },
                        Err(err) => {
                            Self::report_error(&err);
                            result = Err(err);
                            if Self::short_circuit() {
                                return result;
                            }
                        }
                    }
                }
            };
        }
        let mut coda = self.next_coda(parse_stack.current_state(), &parse_stack);
        while let Coda::Reduce(production_id) = coda {
            let (lhs, rhs_len) = self.production_data(production_id);
            let _rhs = parse_stack.pop_n(rhs_len);
            let next_state = Self::goto_state(&lhs, parse_stack.current_state());
            parse_stack.push_non_terminal(lhs, next_state);
            self.push_attribute(A::default());
            // TODO: do semantic actions here

            coda = self.next_coda(parse_stack.current_state(), &parse_stack);
        }
        if let Coda::UnexpectedEndOfInput = coda {
            let err = Error::UnexpectedEndOfInput;
            Self::report_error(&err);
            return Err(err);
        }
        result
    }
}
