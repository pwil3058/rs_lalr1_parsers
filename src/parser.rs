use std::fmt::Debug;

use lexan;

#[derive(Debug)]
pub enum Error<'a, H: Copy + Debug> {
    SyntaxError(H, Vec<H>),
    UnexpectedEndOfInput,
    UnexpectedInput(&'a str),
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
    fn next_action(&self, state: u32, o_handle: Option<H>) -> Result<Action, Error<H>>;

    fn report_error(_error: &Error<H>, _location: Option<lexan::Location>) {

    }

    fn short_circuit() -> bool {
        false
    }

    fn parse_text(&self, text: &str, label: &str) -> bool {
        let mut tokens = self.lexicon().injectable_token_stream(text, label);
        let mut o_token = tokens.next();
        let mut state: u32 = 0;
        let mut result = true;
        loop {
            if let Some(token) = o_token {
                match token {
                    lexan::Token::UnexpectedText(text, location) => {
                        println!("Unexpected text \"{}\" at: {}", text, location);
                        if Self::short_circuit() {
                            return false
                        }
                    }
                    lexan::Token::Valid(handle, _text, location) => {
                        match self.next_action(state, Some(handle)) {
                            Ok(action) => match action {
                                Action::Shift(state) => println!("shift: {}", state),
                                Action::Reduce(production) => println!("reduce: {}", production),
                                _ => panic!("unexpected action"),
                            }
                            Err(err) => {
                                Self::report_error(&err, Some(location));
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
                        Self::report_error(&err, None);
                        return false
                    }
                }
            }
            o_token = tokens.next();
        }
        result
    }
}
