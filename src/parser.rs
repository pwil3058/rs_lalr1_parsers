use std::fmt::Debug;

use lexan;

#[derive(Debug)]
pub enum Error<H: Copy + Debug> {
    LexicalError(lexan::Error<H>),
    SyntaxError(H, Vec<H>, String),
    UnexpectedEndOfInput,
}

#[derive(Debug, Clone)]
pub enum Action {
    Shift(u32),
    Reduce(u32),
    Accept,
}

#[derive(Debug, Clone)]
pub struct SyntaxErrorData<'a, H> {
    unexpected_handle: H,
    matched_text: &'a str,
}

pub trait Parser<H: Ord + Copy + Debug, A> {
    fn lexicon(&self) -> &lexan::Lexicon<H>;
    fn attribute<'b>(&'b self, attr_num: usize, num_attrs: usize) -> &'b A;
    fn next_action<'a>(
        &self,
        state: u32,
        o_token: Option<&lexan::Token<'a, H>>,
    ) -> Result<Action, Error<H>>;

    fn report_error(error: &Error<H>) {
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

    fn parse_text<'b>(&'b self, text: &'b str, label: &'b str) -> bool {
        let mut tokens = self.lexicon().injectable_token_stream(text, label);
        let mut o_r_token = tokens.next();
        let mut state: u32 = 0;
        let mut result: bool = true;
        loop {
            if let Some(r_token) = o_r_token {
                match r_token {
                    Err(err) => {
                        let err = Error::LexicalError(err);
                        Self::report_error(&err);
                        result = false;
                        if Self::short_circuit() {
                            return result;
                        }
                    }
                    Ok(token) => match self.next_action(state, Some(&token)) {
                        Ok(action) => match action {
                            Action::Shift(state) => println!("shift: {}", state),
                            Action::Reduce(production) => println!("reduce: {}", production),
                            _ => panic!("unexpected action"),
                        },
                        Err(err) => {
                            Self::report_error(&err);
                            result = false;
                            if Self::short_circuit() {
                                return result;
                            }
                        }
                    },
                }
            } else {
                match self.next_action(state, None) {
                    Ok(action) => match action {
                        Action::Reduce(production) => println!("reduce: {}", production),
                        Action::Accept => break,
                        _ => panic!("unexpected action"),
                    },
                    Err(err) => {
                        Self::report_error(&err);
                        return false;
                    }
                }
            }
            o_r_token = tokens.next();
        }
        result
    }
}
