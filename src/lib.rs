extern crate lexan;

pub mod parser;

#[cfg(test)]
mod tests {
    use super::parser;

    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::convert::From;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum Terminal {
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
        Expr,
    }

    #[derive(Debug, Clone)]
    struct AttributeData {
        id: String,
        value: f64,
    }

    struct Calc {
        lexical_analyzer: lexan::LexicalAnalyzer<Terminal>,
        state_stack: RefCell<Vec<(parser::Symbol<Terminal, NonTerminal>, u32)>>,
        attributes: Vec<AttributeData>,
        errors: u32,
        variables: HashMap<String, f64>,
    }

    impl Calc {
        pub fn new() -> Self {
            use Terminal::*;
            let lexical_analyzer = lexan::LexicalAnalyzer::new(
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
            );
            Self {
                lexical_analyzer,
                attributes: vec![],
                state_stack: RefCell::new(vec![]),
                errors: 0,
                variables: HashMap::new(),
            }
        }
    }

    macro_rules! syntax_error {
        ( $token:expr; $( $symbol:expr),* ) => {
            parser::Error::SyntaxError(
                *$token.symbol(),
                vec![ $( $symbol),* ],
                $token.location().to_string(),
            )
        };
    }

    impl parser::Parser<Terminal, NonTerminal, AttributeData> for Calc {
        fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<Terminal> {
            &self.lexical_analyzer
        }

        fn attribute<'b>(&'b self, attr_num: usize, num_attrs: usize) -> &'b AttributeData {
            let index = self.attributes.len() - num_attrs - 1 + attr_num;
            &self.attributes[index]
        }

        fn pop_attributes(&mut self, n: usize) -> Vec<AttributeData> {
            self.attributes.split_off(n)
        }


        fn current_state(&self) -> u32 {
            self.state_stack.borrow().last().unwrap().1
        }

        fn push_state(&self, state: u32, symbol: parser::Symbol<Terminal, NonTerminal>) {
            self.state_stack.borrow_mut().push((symbol, state));
        }

        fn next_action<'a>(
            &self,
            state: u32,
            token: &lexan::Token<'a, Terminal>,
        ) -> Result<parser::Action, parser::Error<'a, Terminal>> {
            use Terminal::*;
            let symbol = *token.symbol();
            return match state {
                0 => match symbol {
                    Minus | LPR | Number | Id => Ok(parser::Action::Reduce(8)),
                    _ => Err(syntax_error!(token; Minus, LPR, Number, Id)),
                },
                1 => match symbol {
                    EOL => Ok(parser::Action::Shift(4)),
                    _ => Err(syntax_error!(token; EOL)),
                },
                2 => match symbol {
                    Minus => Ok(parser::Action::Shift(8)),
                    LPR => Ok(parser::Action::Shift(7)),
                    Number => Ok(parser::Action::Shift(9)),
                    Id => Ok(parser::Action::Shift(6)),
                    _ => Err(syntax_error!(token; Minus, LPR, Number, Id)),
                },
                3 => match symbol {
                    EOL => Ok(parser::Action::Reduce(7)),
                    _ => Err(syntax_error!(token; EOL)),
                },
                4 => match symbol {
                    EOL => Ok(parser::Action::Reduce(6)),
                    Minus | Number | Id | LPR => Ok(parser::Action::Reduce(8)),
                    _ => Err(syntax_error!(token; EOL, Minus, Number, Id, LPR)),
                },
                5 => match symbol {
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
                6 => match symbol {
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
                7 | 8 => match symbol {
                    Minus => Ok(parser::Action::Shift(8)),
                    LPR => Ok(parser::Action::Shift(17)),
                    Number => Ok(parser::Action::Shift(9)),
                    Id => Ok(parser::Action::Shift(17)),
                    _ => Err(syntax_error!(token; Minus, Number, Id, LPR)),
                },
                9 => match symbol {
                    EOL | Plus | Minus | Times | Divide | RPR => Ok(parser::Action::Reduce(25)),
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                },
                10 => match symbol {
                    EOL => Ok(parser::Action::Reduce(5)),
                    _ => Err(syntax_error!(token; EOL)),
                },
                11 | 12 | 13 | 14 | 15 => match symbol {
                    Minus => Ok(parser::Action::Shift(8)),
                    LPR => Ok(parser::Action::Shift(7)),
                    Number => Ok(parser::Action::Shift(9)),
                    Id => Ok(parser::Action::Shift(17)),
                    _ => Err(syntax_error!(token; Minus, Number, Id, LPR)),
                },
                16 => match symbol {
                    Plus => Ok(parser::Action::Shift(11)),
                    Minus => Ok(parser::Action::Shift(12)),
                    Times => Ok(parser::Action::Shift(13)),
                    Divide => Ok(parser::Action::Shift(14)),
                    RPR => Ok(parser::Action::Shift(24)),
                    _ => Err(syntax_error!(token; Plus, Minus, Times, Divide, RPR)),
                },
                17 => match symbol {
                    EOL | Plus | Minus | Times | Divide | RPR => {
                        if self.variables.contains_key(&self.attribute(2, 1).id) {
                            Ok(parser::Action::Reduce(26))
                        } else {
                            Ok(parser::Action::Reduce(27))
                        }
                    }
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                },
                18 => match symbol {
                    EOL | Plus | Minus | Times | Divide | RPR => Ok(parser::Action::Reduce(24)),
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                },
                19 => match symbol {
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
                20 => match symbol {
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
                21 => match symbol {
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
                22 => match symbol {
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
                23 => match symbol {
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
                    }
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                },
                24 => match symbol {
                    EOL | Plus | Minus | Times | Divide | RPR => Ok(parser::Action::Reduce(22)),
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                },
                _ => panic!("illegal state: {}", state),
            };
        }

        fn next_coda(&self, state: u32) -> parser::Coda {
            return match state {
                1 => parser::Coda::Accept,
                3 => parser::Coda::Reduce(7),
                4 => parser::Coda::Reduce(6),
                5 => {
                    if self.errors > 0 {
                        parser::Coda::Reduce(1)
                    } else {
                        parser::Coda::Reduce(2)
                    }
                }
                6 | 17 => {
                    if self.variables.contains_key(&self.attribute(2, 1).id) {
                        parser::Coda::Reduce(26)
                    } else {
                        parser::Coda::Reduce(27)
                    }
                }
                9 => parser::Coda::Reduce(25),
                10 => parser::Coda::Reduce(15),
                18 => parser::Coda::Reduce(24),
                19 => {
                    if self.attribute(4, 1).value == 0.0 {
                        parser::Coda::Reduce(9)
                    } else if self.attribute(4, 3).value == 0.0 {
                        parser::Coda::Reduce(10)
                    } else {
                        parser::Coda::Reduce(11)
                    }
                }
                20 => {
                    if self.attribute(4, 1).value == 0.0 {
                        parser::Coda::Reduce(12)
                    } else if self.attribute(4, 3).value == 0.0 {
                        parser::Coda::Reduce(13)
                    } else {
                        parser::Coda::Reduce(14)
                    }
                }
                21 => {
                    if self.attribute(4, 1).value == 0.0 || self.attribute(4, 3).value == 0.0 {
                        parser::Coda::Reduce(15)
                    } else if self.attribute(4, 1).value == 1.0 {
                        parser::Coda::Reduce(16)
                    } else if self.attribute(4, 3).value == 1.0 {
                        parser::Coda::Reduce(17)
                    } else {
                        parser::Coda::Reduce(18)
                    }
                }
                22 => {
                    if self.attribute(4, 1).value == 0.0 || self.attribute(4, 3).value == 0.0 {
                        parser::Coda::Reduce(19)
                    } else if self.attribute(4, 1).value == 1.0 {
                        parser::Coda::Reduce(20)
                    } else if self.attribute(4, 3).value == 1.0 {
                        parser::Coda::Reduce(21)
                    } else {
                        parser::Coda::Reduce(22)
                    }
                }
                23 => {
                    if self.errors == 0 {
                        parser::Coda::Reduce(3)
                    } else {
                        parser::Coda::Reduce(4)
                    }
                }
                24 => parser::Coda::Reduce(23),
                _ => parser::Coda::UnexpectedEndOfInput,
            };
        }
    }

    #[test]
    fn calc_works() {
        use crate::parser::Parser;
        let mut calc = Calc::new();
        assert!(calc.parse_text("a = 3 + 4", "raw").is_err());
    }
}
