use std::fmt::Debug;

use lexan;

#[derive(Debug, Clone, Copy)]
pub enum Action<'a> {
    Shift(u32),
    Reduce(u32),
    Accept,
    SyntaxError,
    UnexpectedText(&'a str, lexan::Location<'a>),
    UnexpectedEndOfInput,
}

pub trait Parser<H: Ord + Copy + Debug, A> {
    fn lexicon(&self) -> &lexan::Lexicon<H>;
    fn attributes(&self) -> &Vec<A>;
    fn next_action<'a>(&self, state: u32, o_token: Option<lexan::Token<'a, H>>) -> Action<'a>;

    fn parse_text(&self, text: &str, label: &str) -> bool {
        let mut tokens = self.lexicon().injectable_token_stream(text, label);
        let mut o_token = tokens.next();
        let mut state: u32 = 0;
        loop {
            match self.next_action(state, o_token) {
                Action::Shift(state) => println!("shift: {}", state),
                Action::Reduce(production) => println!("reduce: {}", production),
                Action::Accept => break,
                _ => return false,
            };
            o_token = tokens.next();
        }
        true
    }
}
