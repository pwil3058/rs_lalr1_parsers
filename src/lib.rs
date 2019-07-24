extern crate lexan;

pub mod parser;

#[cfg(test)]
mod tests {
    use super::parser;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum Handle {
        Plus,
        Minus,
        Times,
        Divide,
        Assign,
        LPR,
        RPR,
        EOL,
        Number,
        Id,
    }

    struct Calc {
        lexicon: lexan::Lexicon<Handle>,
        attributes: Vec<u32>,
    }

    impl Calc {
        pub fn new() -> Self {
            use Handle::*;
            let lexicon = lexan::Lexicon::new(
                &[
                    (Plus, "+"),
                    (Minus, "-"),
                    (Times, "*"),
                    (Divide, "/"),
                    (Assign, "="),
                    (LPR, "("),
                    (RPR, ")"),
                ],
                &[
                    (EOL, r"\A(\n)"),
                    (Number, r"\A([0-9]+(\.[0-9]+){0,1})"),
                    (Id, r"\A([a-zA-Z]+)"),
                ],
                &[
                    r"\A([\t\r ]+)",
                ]
            ).unwrap();
            Self { lexicon, attributes: vec![] }
        }
    }

    impl parser::Parser<Handle, u32> for Calc {
        fn lexicon(&self) -> &lexan::Lexicon<Handle> {
            &self.lexicon
        }

        fn attributes(&self) -> &Vec<u32> {
            &self.attributes
        }

        fn next_action<'a>(&self, state: u32, o_token: Option<&lexan::Token<'a, Handle>>) -> Result<parser::Action, parser::Error<Handle>> {
            if let Some(token) = o_token {
                use Handle::*;
                let handle = *token.handle();
                match state {
                    0 => match handle {
                        Minus => return Ok(parser::Action::Reduce(8)),
                        LPR => return Ok(parser::Action::Reduce(8)),
                        Number => return Ok(parser::Action::Reduce(8)),
                        Id => return Ok(parser::Action::Reduce(8)),
                        _ => return Err(parser::Error::SyntaxError(handle, vec![Minus, LPR, Number, Id], token.location().to_string())),
                    },
                    1 => match handle {
                        EOL => return Ok(parser::Action::Shift(4)),
                        _ => return Err(parser::Error::SyntaxError(handle, vec![EOL], token.location().to_string())),
                    },
                    100 => match handle {
                        Plus => return Ok(parser::Action::Shift(0)),
                        Minus => return Ok(parser::Action::Shift(0)),
                        Times => return Ok(parser::Action::Shift(0)),
                        Divide => return Ok(parser::Action::Shift(0)),
                        Assign => return Ok(parser::Action::Shift(0)),
                        LPR => return Ok(parser::Action::Shift(0)),
                        RPR => return Ok(parser::Action::Shift(0)),
                        EOL => return Ok(parser::Action::Shift(0)),
                        Number => return Ok(parser::Action::Shift(0)),
                        Id => return Ok(parser::Action::Shift(0)),
                    },
                    _ => panic!("illegal state: {}", state)
                }
            } else {
                match state {
                    1 => return Ok(parser::Action::Accept),
                    _ => return Err(parser::Error::UnexpectedEndOfInput),
                }
            }
        }
    }

    #[test]
    fn calc_works() {
        use crate::parser::Parser;
        let calc = Calc::new();
        assert!(!calc.parse_text("a = 3 + 4", "raw"));
    }
}
