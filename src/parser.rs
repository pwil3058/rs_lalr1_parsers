use std::fmt::Debug;

use lexan;

#[derive(Debug)]
pub enum Error<'a, H: Copy + Debug> {
    LexicalError(lexan::Error<'a, H>),
    SyntaxError(H, Vec<H>, lexan::Location<'a>),
    UnexpectedEndOfInput,
}

#[derive(Debug, Clone)]
pub enum Action {
    Shift(u32),
    Reduce(u32),
    Accept,
}

pub trait Parser<H: Ord + Copy + Debug, A> {
    fn lexicon(&self) -> &lexan::Lexicon<H>;
    fn attributes(&self) -> &Vec<A>;
    fn next_action<'a>(&'a self, state: u32, o_token: Option<&'a lexan::Token<'a, H>>) -> Result<Action, Error<'a, H>>;

    fn report_error<'a>(error: &Error<'a, H>) {
        match error {
            Error::LexicalError(lex_err) => println!("Lexical Error: {}.", lex_err),
            Error::SyntaxError(found, expected, location) =>
                println!("Syntax Error: expected: {:?} found: {:?} at: {}.", expected, found, location),
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
                            return result
                        }
                    }
                    Ok(token) => {
                        match self.next_action(state, Some(&token)) {
                            Ok(action) => match action {
                                Action::Shift(state) => println!("shift: {}", state),
                                Action::Reduce(production) => println!("reduce: {}", production),
                                _ => panic!("unexpected action"),
                            }
                            Err(err) => {
                                Self::report_error(&err);
                                result = false;
                                if Self::short_circuit() {
                                    return result
                                }
                            }
                        }
                    }
                }
            } else {
                match self.next_action(state, None) {
                    Ok(action) => match action {
                        Action::Reduce(production) => println!("reduce: {}", production),
                        Action::Accept => break,
                        _ => panic!("unexpected action"),
                    }
                    Err(err) => {
                        Self::report_error(&err);
                        return false
                    }
                }
            }
            o_r_token = tokens.next();
        }
        result
    }
}
