use std::fmt::Debug;

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

#[derive(Debug, Clone)]
pub struct SyntaxErrorData<'a, T> {
    unexpected_symbol: T,
    matched_text: &'a str,
}

pub trait Parser<T: Ord + Copy + Debug, N, A> {
    fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<T>;
    fn attribute<'b>(&'b self, attr_num: usize, num_attrs: usize) -> &'b A;
    fn current_state(&self) -> u32;
    fn push_state(&self, state: u32, symbol: Symbol<T, N>);
    fn next_action<'a>(
        &self,
        state: u32,
        o_token: &lexan::Token<'a, T>,
    ) -> Result<Action, Error<'a, T>>;
    fn next_coda<'a>(&self, state: u32) -> Coda;

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
        self.push_state(0, Symbol::Start);
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
                Ok(token) => match self.next_action(self.current_state(), &token) {
                    Ok(action) => match action {
                        Action::Shift(state) => {
                            println!("shift: {}", state);
                            self.push_state(state, Symbol::Terminal(*token.handle()));
                        }
                        Action::Reduce(production) => {
                            println!("reduce: {}", production);
                            //continue;
                        }
                    },
                    Err(err) => {
                        Self::report_error(&err);
                        result = Err(err);
                        if Self::short_circuit() {
                            return result;
                        }
                    }
                },
            };
            o_r_token = tokens.next();
        }
        let mut coda = self.next_coda(self.current_state());
        while let Coda::Reduce(production) = coda {
            println!("reduce: {}", production);
            coda = self.next_coda(self.current_state());
        }
        if let Coda::UnexpectedEndOfInput = coda {
            let err = Error::UnexpectedEndOfInput;
            Self::report_error(&err);
            return Err(err);
        }
        result
    }
}
