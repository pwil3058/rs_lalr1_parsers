extern crate lexan;

mod parser;

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

        fn next_action<'a>(&self, state: u32, o_token: Option<lexan::Token<'a, Handle>>) -> parser::Action<'a> {
            if let Some(token) = o_token {
                use Handle::*;
                match token {
                    lexan::Token::UnexpectedText(text,location) => {
                        return parser::Action::UnexpectedText(text, location);
                    }
                    lexan::Token::Valid(handle, text, location) => match state {
                        0 => match handle {
                            Minus => return parser::Action::Reduce(8),
                            LPR => return parser::Action::Reduce(8),
                            Number => return parser::Action::Reduce(8),
                            Id => return parser::Action::Reduce(8),
                            _ => return parser::Action::SyntaxError,
                        },
                        1 => match handle {
                            EOL => return parser::Action::Shift(4),
                            _ => return parser::Action::SyntaxError,
                        },
                        100 => match handle {
                            Plus => return parser::Action::Shift(0),
                            Minus => return parser::Action::Shift(0),
                            Times => return parser::Action::Shift(0),
                            Divide => return parser::Action::Shift(0),
                            Assign => return parser::Action::Shift(0),
                            LPR => return parser::Action::Shift(0),
                            RPR => return parser::Action::Shift(0),
                            EOL => return parser::Action::Shift(0),
                            Number => return parser::Action::Shift(0),
                            Id => return parser::Action::Shift(0),
                        },
                        _ => panic!("illegal state: {}", state)
                    }
                }
            } else {
                match state {
                    1 => return parser::Action::Accept,
                    _ => return parser::Action::UnexpectedEndOfInput,
                }
            }
        }
    }

    #[test]
    fn calc_works() {
        use crate::parser::Parser;
        let calc = Calc::new();
        assert!(calc.parse_text("a = 3 + 4", "raw"));
    }
}
