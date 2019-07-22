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
            Self { lexicon }
        }
    }

    impl parser::Parser<Handle> for Calc {
        fn lexicon(&self) -> &lexan::Lexicon<Handle> {
            &self.lexicon
        }

        fn next_action(&self, o_token: Option<lexan::Token<Handle>>) -> parser::Action {
            if let Some(token) = o_token {
                return parser::Action::Shift
            } else {
                return parser::Action::Accept
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
