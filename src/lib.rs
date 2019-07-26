extern crate lexan;

pub mod parser;

#[cfg(test)]
mod tests {
    use super::parser;

    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::convert::From;

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

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum NonTerminal {
        Line,
        SetUp,
        Expr
    }

    #[derive(Debug, Clone)]
    struct AttributeData {
        id: String,
        value: f64,
    }

    struct Calc {
        lexicon: lexan::Lexicon<Handle>,
        state_stack: RefCell<Vec<(parser::Symbol<Handle, NonTerminal>, u32)>>,
        attributes: Vec<AttributeData>,
        errors: u32,
        variables: HashMap<String, f64>,
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
                &[r"\A([\t\r ]+)"],
            )
            .unwrap();
            Self {
                lexicon,
                attributes: vec![],
                state_stack: RefCell::new(vec![]),
                errors: 0,
                variables: HashMap::new(),
            }
        }
    }

    macro_rules! syntax_error {
        ( $token:expr; $( $handle:expr),* ) => {
            parser::Error::SyntaxError(
                *$token.handle(),
                vec![ $( $handle),* ],
                $token.location().to_string(),
            )
        };
    }

    impl parser::Parser<Handle, NonTerminal, AttributeData> for Calc {
        fn lexicon(&self) -> &lexan::Lexicon<Handle> {
            &self.lexicon
        }

        fn attribute<'b>(&'b self, attr_num: usize, num_attrs: usize) -> &'b AttributeData {
            let index = self.attributes.len() - num_attrs - 1 + attr_num;
            &self.attributes[index]
        }

        fn current_state(&self) -> u32 {
            self.state_stack.borrow().last().unwrap().1
        }

        fn push_state(&self, state:u32, symbol: parser::Symbol<Handle, NonTerminal>) {
            self.state_stack.borrow_mut().push((symbol, state));
        }

        fn next_action<'a>(
            &self,
            state: u32,
            o_token: Option<&lexan::Token<'a, Handle>>,
        ) -> Result<parser::Action, parser::Error<Handle>> {
            if let Some(token) = o_token {
                use Handle::*;
                let handle = *token.handle();
                return match state {
                    0 => match handle {
                        Minus | LPR | Number | Id => Ok(parser::Action::Reduce(8)),
                        _ => Err(syntax_error!(token; Minus, LPR, Number, Id)),
                    },
                    1 => match handle {
                        EOL => Ok(parser::Action::Shift(4)),
                        _ => Err(syntax_error!(token; EOL)),
                    },
                    2 => match handle {
                        Minus => Ok(parser::Action::Shift(8)),
                        LPR => Ok(parser::Action::Shift(7)),
                        Number => Ok(parser::Action::Shift(9)),
                        Id => Ok(parser::Action::Shift(6)),
                        _ => Err(syntax_error!(token; Minus, LPR, Number, Id)),
                    },
                    3 => match handle {
                        EOL => Ok(parser::Action::Reduce(7)),
                        _ => Err(syntax_error!(token; EOL)),
                    },
                    4 => match handle {
                        EOL => Ok(parser::Action::Reduce(6)),
                        Minus | Number | Id | LPR => Ok(parser::Action::Reduce(8)),
                        _ => Err(syntax_error!(token; EOL, Minus, Number, Id, LPR)),
                    },
                    5 => match handle {
                        Plus => Ok(parser::Action::Shift(11)),
                        Minus => Ok(parser::Action::Shift(12)),
                        Times => Ok(parser::Action::Shift(13)),
                        Divide => Ok(parser::Action::Shift(14)),
                        EOL => {
                            if self.errors > 0 {
                                Ok(parser::Action::Reduce(1))
                            } else {
                                Ok(parser::Action::Reduce(2))
                            }
                        }
                        _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide)),
                    },
                    6 => match handle {
                        Assign => Ok(parser::Action::Shift(15)),
                        EOL | Plus | Minus | Times | Divide => {
                            if self.variables.contains_key(&self.attribute(2, 1).id) {
                                Ok(parser::Action::Reduce(26))
                            } else {
                                Ok(parser::Action::Reduce(27))
                            }
                        }
                        _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, Assign)),
                    },
                    7 | 8 => match handle {
                        Minus => Ok(parser::Action::Shift(8)),
                        LPR => Ok(parser::Action::Shift(17)),
                        Number => Ok(parser::Action::Shift(9)),
                        Id => Ok(parser::Action::Shift(17)),
                        _ => Err(syntax_error!(token; Minus, Number, Id, LPR)),
                    },
                    9 => match handle {
                        EOL | Plus | Minus | Times | Divide | RPR => Ok(parser::Action::Reduce(25)),
                        _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                    },
                    10 => match handle {
                        EOL => Ok(parser::Action::Reduce(5)),
                        _ => Err(syntax_error!(token; EOL)),
                    },
                    11 | 12 | 13 | 14 | 15 => match handle {
                        Minus => Ok(parser::Action::Shift(8)),
                        LPR => Ok(parser::Action::Shift(7)),
                        Number => Ok(parser::Action::Shift(9)),
                        Id => Ok(parser::Action::Shift(17)),
                        _ => Err(syntax_error!(token; Minus, Number, Id, LPR)),
                    },
                    16 => match handle {
                        Plus => Ok(parser::Action::Shift(11)),
                        Minus => Ok(parser::Action::Shift(12)),
                        Times => Ok(parser::Action::Shift(13)),
                        Divide => Ok(parser::Action::Shift(14)),
                        RPR => Ok(parser::Action::Shift(24)),
                        _ => Err(syntax_error!(token; Plus, Minus, Times, Divide, RPR)),
                    },
                    17 => match handle {
                        EOL | Plus | Minus | Times | Divide | RPR => {
                            if self.variables.contains_key(&self.attribute(2, 1).id) {
                                Ok(parser::Action::Reduce(26))
                            } else {
                                Ok(parser::Action::Reduce(27))
                            }
                        }
                        _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                    },
                    18 => match handle {
                        EOL | Plus | Minus | Times | Divide | RPR => Ok(parser::Action::Reduce(24)),
                        _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                    },
                    19 => match handle {
                        Times => Ok(parser::Action::Shift(13)),
                        Divide => Ok(parser::Action::Shift(14)),
                        EOL | Plus | Minus | RPR => {
                            if self.attribute(4, 1).value == 0.0 {
                                Ok(parser::Action::Reduce(9))
                            } else if self.attribute(4, 3).value == 0.0 {
                                Ok(parser::Action::Reduce(10))
                            } else {
                                Ok(parser::Action::Reduce(11))
                            }
                        }
                        _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                    },
                    20 => match handle {
                        Times => Ok(parser::Action::Shift(13)),
                        Divide => Ok(parser::Action::Shift(14)),
                        EOL | Plus | Minus | RPR => {
                            if self.attribute(4, 1).value == 0.0 {
                                Ok(parser::Action::Reduce(12))
                            } else if self.attribute(4, 3).value == 0.0 {
                                Ok(parser::Action::Reduce(13))
                            } else {
                                Ok(parser::Action::Reduce(14))
                            }
                        }
                        _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                    },
                    21 => match handle {
                        EOL | Plus | Minus | Times | Divide | RPR => {
                            if self.attribute(4, 1).value == 0.0 || self.attribute(4, 3).value == 0.0 {
                                Ok(parser::Action::Reduce(15))
                            } else if self.attribute(4, 1).value == 1.0 {
                                Ok(parser::Action::Reduce(16))
                            } else if self.attribute(4, 3).value == 1.0 {
                                Ok(parser::Action::Reduce(17))
                            } else {
                                Ok(parser::Action::Reduce(18))
                            }
                        }
                        _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                    },
                    22 => match handle {
                        EOL | Plus | Minus | Times | Divide | RPR => {
                            if self.attribute(4, 1).value == 0.0 || self.attribute(4, 3).value == 0.0 {
                                Ok(parser::Action::Reduce(19))
                            } else if self.attribute(4, 1).value == 1.0 {
                                Ok(parser::Action::Reduce(20))
                            } else if self.attribute(4, 3).value == 1.0 {
                                Ok(parser::Action::Reduce(21))
                            } else {
                                Ok(parser::Action::Reduce(22))
                            }
                        }
                        _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                    },
                    23 => match handle {
                        Plus => Ok(parser::Action::Shift(11)),
                        Minus => Ok(parser::Action::Shift(12)),
                        Times => Ok(parser::Action::Shift(13)),
                        Divide => Ok(parser::Action::Shift(14)),
                        EOL => {
                            if self.errors == 0 {
                                Ok(parser::Action::Reduce(3))
                            } else {
                                Ok(parser::Action::Reduce(4))
                            }
                        },
                        _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                    },
                    24 => match handle {
                        EOL | Plus | Minus | Times | Divide | RPR => Ok(parser::Action::Reduce(22)),
                        _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                    }
                    _ => panic!("illegal state: {}", state),
                };
            } else {
                return match state {
                    1 => Ok(parser::Action::Accept),
                    3 => Ok(parser::Action::Reduce(7)),
                    4 => Ok(parser::Action::Reduce(6)),
                    5 => {
                        if self.errors > 0 {
                            Ok(parser::Action::Reduce(1))
                        } else {
                            Ok(parser::Action::Reduce(2))
                        }
                    }
                    6 | 17 => {
                        if self.variables.contains_key(&self.attribute(2, 1).id) {
                            Ok(parser::Action::Reduce(26))
                        } else {
                            Ok(parser::Action::Reduce(27))
                        }
                    }
                    9 => Ok(parser::Action::Reduce(25)),
                    10 => Ok(parser::Action::Reduce(15)),
                    18 => Ok(parser::Action::Reduce(24)),
                    19 => {
                        if self.attribute(4, 1).value == 0.0 {
                            Ok(parser::Action::Reduce(9))
                        } else if self.attribute(4, 3).value == 0.0 {
                            Ok(parser::Action::Reduce(10))
                        } else {
                            Ok(parser::Action::Reduce(11))
                        }
                    }
                    20 => {
                        if self.attribute(4, 1).value == 0.0 {
                            Ok(parser::Action::Reduce(12))
                        } else if self.attribute(4, 3).value == 0.0 {
                            Ok(parser::Action::Reduce(13))
                        } else {
                            Ok(parser::Action::Reduce(14))
                        }
                    }
                    21 => {
                        if self.attribute(4, 1).value == 0.0 || self.attribute(4, 3).value == 0.0 {
                            Ok(parser::Action::Reduce(15))
                        } else if self.attribute(4, 1).value == 1.0 {
                            Ok(parser::Action::Reduce(16))
                        } else if self.attribute(4, 3).value == 1.0 {
                            Ok(parser::Action::Reduce(17))
                        } else {
                            Ok(parser::Action::Reduce(18))
                        }
                    }
                    22 => {
                        if self.attribute(4, 1).value == 0.0 || self.attribute(4, 3).value == 0.0 {
                            Ok(parser::Action::Reduce(19))
                        } else if self.attribute(4, 1).value == 1.0 {
                            Ok(parser::Action::Reduce(20))
                        } else if self.attribute(4, 3).value == 1.0 {
                            Ok(parser::Action::Reduce(21))
                        } else {
                            Ok(parser::Action::Reduce(22))
                        }
                    }
                    23 => {
                        if self.errors == 0 {
                            Ok(parser::Action::Reduce(3))
                        } else {
                            Ok(parser::Action::Reduce(4))
                        }
                    },
                    24 => Ok(parser::Action::Reduce(23)),
                    _ => Err(parser::Error::UnexpectedEndOfInput),
                };
            }
        }
    }

    #[test]
    fn calc_works() {
        use crate::parser::Parser;
        let mut calc = Calc::new();
        assert!(!calc.parse_text("a = 3 + 4", "raw"));
    }
}
