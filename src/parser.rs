use std::fmt::Debug;

use lexan;

#[derive(Debug, Clone)]
pub enum Action<H:  Ord + Copy + Debug> {
    Shift(u32),
    Reduce(u32),
    Accept,
    SyntaxError(H, Vec<H>),
    UnexpectedEndOfInput,
}

pub trait Parser<H: Ord + Copy + Debug, A> {
    fn lexicon(&self) -> &lexan::Lexicon<H>;
    fn attributes(&self) -> &Vec<A>;
    fn next_action(&self, state: u32, o_handle: Option<H>) -> Action<H>;

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
                    lexan::Token::Valid(handle, text, location) => {
                        match self.next_action(state, Some(handle)) {
                            Action::Shift(state) => println!("shift: {}", state),
                            Action::Reduce(production) => println!("reduce: {}", production),
                            Action::SyntaxError(found, expected) => {
                                println!("syntax error: expected {:?} found {:?}", expected, found);
                                result = false;
                                if Self::short_circuit() {
                                    return result
                                }
                            }
                            _ => panic!("unexpected action"),
                        }
                    }
                }
            } else {
                match self.next_action(state, None) {
                    Action::Reduce(production) => println!("reduce: {}", production),
                    Action::Accept => break,
                    Action::UnexpectedEndOfInput => {
                        println!("unexpected end of input");
                        return false
                    }
                    _ => panic!("unexpected action"),
                }
            }
            o_token = tokens.next();
        }
        result
    }
}
