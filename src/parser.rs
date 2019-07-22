use std::fmt::Debug;

use lexan;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Shift,
    Reduce,
    Accept,
    SyntaxError,
    UnexpectedText,
}

pub trait Parser<H: Ord + Copy + Debug> {
    fn lexicon(&self) -> &lexan::Lexicon<H>;

    fn next_action(&self, o_token: Option<lexan::Token<H>>) -> Action;

    fn parse_text(&self, text: &str, label: &str) -> bool {
        let mut tokens = self.lexicon().injectable_token_stream(text, label);
        let mut o_token = tokens.next();
        loop {
            match self.next_action(o_token) {
                Action::Shift => println!("shift"),
                Action::Reduce => println!("reduce"),
                Action::Accept => break,
                _ => return false,
            };
            o_token = tokens.next();
        }
        true
    }
}
