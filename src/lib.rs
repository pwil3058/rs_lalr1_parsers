extern crate lexan;

pub mod parser;

#[cfg(test)]
mod tests {
    use super::parser;

    use std::collections::HashMap;
    use std::convert::From;
    use std::fmt;
    use std::str::FromStr;

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

    impl fmt::Display for Terminal {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Terminal::Plus => write!(f, "+"),
                Terminal::Minus => write!(f, "-"),
                Terminal::Times => write!(f, "*"),
                Terminal::Divide => write!(f, "/"),
                Terminal::Assign => write!(f, "="),
                Terminal::LPR => write!(f, "("),
                Terminal::RPR => write!(f, ")"),
                Terminal::EOL => write!(f, "EOL"),
                Terminal::Number => write!(f, "Number"),
                Terminal::Id => write!(f, "Id"),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum NonTerminal {
        Line,
        SetUp,
        Expr,
    }

    impl fmt::Display for NonTerminal {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                NonTerminal::Line => write!(f, "Line"),
                NonTerminal::SetUp => write!(f, "SetUp"),
                NonTerminal::Expr => write!(f, "Expr"),
            }
        }
    }

    #[derive(Debug, Default, Clone)]
    struct AttributeData {
        id: String,
        value: f64,
    }

    impl From<(Terminal, String)> for AttributeData {
        fn from(input: (Terminal, String)) -> Self {
            let mut attr = AttributeData::default();
            match input.0 {
                Terminal::Number => {
                    attr.value = f64::from_str(&input.1).unwrap();
                }
                Terminal::Id => {
                    attr.id = input.1;
                }
                _ => (),
            };
            attr
        }
    }

    struct Calc {
        lexical_analyzer: lexan::LexicalAnalyzer<Terminal>,
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
                errors: 0,
                variables: HashMap::new(),
            }
        }
    }

    macro_rules! syntax_error {
        ( $token:expr; $( $tag:expr),* ) => {
            parser::Error::SyntaxError(
                *$token.tag(),
                vec![ $( $tag),* ],
                $token.location().to_string(),
            )
        };
    }

    impl parser::Parser<Terminal, NonTerminal, AttributeData> for Calc {
        fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<Terminal> {
            &self.lexical_analyzer
        }

        fn next_action<'a>(
            &self,
            state: u32,
            attributes: &parser::ParseStack<Terminal, NonTerminal, AttributeData>,
            token: &lexan::Token<'a, Terminal>,
        ) -> Result<parser::Action, parser::Error<'a, Terminal>> {
            use Terminal::*;
            let tag = *token.tag();
            return match state {
                0 => match tag {
                    Minus | LPR | Number | Id => Ok(parser::Action::Reduce(8)),
                    _ => Err(syntax_error!(token; Minus, LPR, Number, Id)),
                },
                1 => match tag {
                    EOL => Ok(parser::Action::Shift(4)),
                    _ => Err(syntax_error!(token; EOL)),
                },
                2 => match tag {
                    Minus => Ok(parser::Action::Shift(8)),
                    LPR => Ok(parser::Action::Shift(7)),
                    Number => Ok(parser::Action::Shift(9)),
                    Id => Ok(parser::Action::Shift(6)),
                    _ => Err(syntax_error!(token; Minus, LPR, Number, Id)),
                },
                3 => match tag {
                    EOL => Ok(parser::Action::Reduce(7)),
                    _ => Err(syntax_error!(token; EOL)),
                },
                4 => match tag {
                    EOL => Ok(parser::Action::Reduce(6)),
                    Minus | Number | Id | LPR => Ok(parser::Action::Reduce(8)),
                    _ => Err(syntax_error!(token; EOL, Minus, Number, Id, LPR)),
                },
                5 => match tag {
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
                6 => match tag {
                    Assign => Ok(parser::Action::Shift(15)),
                    EOL | Plus | Minus | Times | Divide => {
                        if self.variables.contains_key(&attributes.attribute_n_from_end(2 - 1).id) {
                            Ok(parser::Action::Reduce(26))
                        } else {
                            Ok(parser::Action::Reduce(27))
                        }
                    }
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, Assign)),
                },
                7 | 8 => match tag {
                    Minus => Ok(parser::Action::Shift(8)),
                    LPR => Ok(parser::Action::Shift(7)),
                    Number => Ok(parser::Action::Shift(9)),
                    Id => Ok(parser::Action::Shift(17)),
                    _ => Err(syntax_error!(token; Minus, Number, Id, LPR)),
                },
                9 => match tag {
                    EOL | Plus | Minus | Times | Divide | RPR => Ok(parser::Action::Reduce(25)),
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                },
                10 => match tag {
                    EOL => Ok(parser::Action::Reduce(5)),
                    _ => Err(syntax_error!(token; EOL)),
                },
                11 | 12 | 13 | 14 | 15 => match tag {
                    Minus => Ok(parser::Action::Shift(8)),
                    LPR => Ok(parser::Action::Shift(7)),
                    Number => Ok(parser::Action::Shift(9)),
                    Id => Ok(parser::Action::Shift(17)),
                    _ => Err(syntax_error!(token; Minus, Number, Id, LPR)),
                },
                16 => match tag {
                    Plus => Ok(parser::Action::Shift(11)),
                    Minus => Ok(parser::Action::Shift(12)),
                    Times => Ok(parser::Action::Shift(13)),
                    Divide => Ok(parser::Action::Shift(14)),
                    RPR => Ok(parser::Action::Shift(24)),
                    _ => Err(syntax_error!(token; Plus, Minus, Times, Divide, RPR)),
                },
                17 => match tag {
                    EOL | Plus | Minus | Times | Divide | RPR => {
                        if self.variables.contains_key(&attributes.attribute_n_from_end(2 - 1).id) {
                            Ok(parser::Action::Reduce(26))
                        } else {
                            Ok(parser::Action::Reduce(27))
                        }
                    }
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                },
                18 => match tag {
                    EOL | Plus | Minus | Times | Divide | RPR => Ok(parser::Action::Reduce(24)),
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                },
                19 => match tag {
                    Times => Ok(parser::Action::Shift(13)),
                    Divide => Ok(parser::Action::Shift(14)),
                    EOL | Plus | Minus | RPR => {
                        if attributes.attribute_n_from_end(4 - 1).value == 0.0 {
                            Ok(parser::Action::Reduce(9))
                        } else if attributes.attribute_n_from_end(4 - 3).value == 0.0 {
                            Ok(parser::Action::Reduce(10))
                        } else {
                            Ok(parser::Action::Reduce(11))
                        }
                    }
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                },
                20 => match tag {
                    Times => Ok(parser::Action::Shift(13)),
                    Divide => Ok(parser::Action::Shift(14)),
                    EOL | Plus | Minus | RPR => {
                        if attributes.attribute_n_from_end(4 - 1).value == 0.0 {
                            Ok(parser::Action::Reduce(12))
                        } else if attributes.attribute_n_from_end(4 - 3).value == 0.0 {
                            Ok(parser::Action::Reduce(13))
                        } else {
                            Ok(parser::Action::Reduce(14))
                        }
                    }
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                },
                21 => match tag {
                    EOL | Plus | Minus | Times | Divide | RPR => {
                        if attributes.attribute_n_from_end(4 - 1).value == 0.0 || attributes.attribute_n_from_end(4 - 3).value == 0.0 {
                            Ok(parser::Action::Reduce(15))
                        } else if attributes.attribute_n_from_end(4 - 1).value == 1.0 {
                            Ok(parser::Action::Reduce(16))
                        } else if attributes.attribute_n_from_end(4 - 3).value == 1.0 {
                            Ok(parser::Action::Reduce(17))
                        } else {
                            Ok(parser::Action::Reduce(18))
                        }
                    }
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                },
                22 => match tag {
                    EOL | Plus | Minus | Times | Divide | RPR => {
                        if attributes.attribute_n_from_end(4 - 1).value == 0.0 || attributes.attribute_n_from_end(4 - 3).value == 0.0 {
                            Ok(parser::Action::Reduce(19))
                        } else if attributes.attribute_n_from_end(4 - 1).value == 1.0 {
                            Ok(parser::Action::Reduce(20))
                        } else if attributes.attribute_n_from_end(4 - 3).value == 1.0 {
                            Ok(parser::Action::Reduce(21))
                        } else {
                            Ok(parser::Action::Reduce(22))
                        }
                    }
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                },
                23 => match tag {
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
                24 => match tag {
                    EOL | Plus | Minus | Times | Divide | RPR => Ok(parser::Action::Reduce(23)),
                    _ => Err(syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR)),
                },
                _ => panic!("illegal state: {}", state),
            };
        }

        fn next_coda(
            &self,
            state: u32,
            attributes: &parser::ParseStack<Terminal, NonTerminal, AttributeData>,
        ) -> parser::Coda {
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
                    if self.variables.contains_key(&attributes.attribute_n_from_end(2 - 1).id) {
                        parser::Coda::Reduce(26)
                    } else {
                        parser::Coda::Reduce(27)
                    }
                }
                9 => parser::Coda::Reduce(25),
                10 => parser::Coda::Reduce(5),
                18 => parser::Coda::Reduce(24),
                19 => {
                    if attributes.attribute_n_from_end(4 - 1).value == 0.0 {
                        parser::Coda::Reduce(9)
                    } else if attributes.attribute_n_from_end(4 - 3).value == 0.0 {
                        parser::Coda::Reduce(10)
                    } else {
                        parser::Coda::Reduce(11)
                    }
                }
                20 => {
                    if attributes.attribute_n_from_end(4 - 1).value == 0.0 {
                        parser::Coda::Reduce(12)
                    } else if attributes.attribute_n_from_end(4 - 3).value == 0.0 {
                        parser::Coda::Reduce(13)
                    } else {
                        parser::Coda::Reduce(14)
                    }
                }
                21 => {
                    if attributes.attribute_n_from_end(4 - 1).value == 0.0 || attributes.attribute_n_from_end(4 - 3).value == 0.0 {
                        parser::Coda::Reduce(15)
                    } else if attributes.attribute_n_from_end(4 - 1).value == 1.0 {
                        parser::Coda::Reduce(16)
                    } else if attributes.attribute_n_from_end(4 - 3).value == 1.0 {
                        parser::Coda::Reduce(17)
                    } else {
                        parser::Coda::Reduce(18)
                    }
                }
                22 => {
                    if attributes.attribute_n_from_end(4 - 1).value == 0.0 || attributes.attribute_n_from_end(4 - 3).value == 0.0 {
                        parser::Coda::Reduce(19)
                    } else if attributes.attribute_n_from_end(4 - 1).value == 1.0 {
                        parser::Coda::Reduce(20)
                    } else if attributes.attribute_n_from_end(4 - 3).value == 1.0 {
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

        fn production_data(&mut self, production_id: u32) -> (NonTerminal, usize) {
            match production_id {
                1 => (NonTerminal::Line, 2),
                2 => (NonTerminal::Line, 2),
                3 => (NonTerminal::Line, 4),
                4 => (NonTerminal::Line, 4),
                5 => (NonTerminal::Line, 3),
                6 => (NonTerminal::Line, 2),
                7 => (NonTerminal::Line, 1),
                8 => (NonTerminal::SetUp, 0),
                9 => (NonTerminal::Expr, 3),
                10 => (NonTerminal::Expr, 3),
                11 => (NonTerminal::Expr, 3),
                12 => (NonTerminal::Expr, 3),
                13 => (NonTerminal::Expr, 3),
                14 => (NonTerminal::Expr, 3),
                15 => (NonTerminal::Expr, 3),
                16 => (NonTerminal::Expr, 3),
                17 => (NonTerminal::Expr, 3),
                18 => (NonTerminal::Expr, 3),
                19 => (NonTerminal::Expr, 3),
                20 => (NonTerminal::Expr, 3),
                21 => (NonTerminal::Expr, 3),
                22 => (NonTerminal::Expr, 3),
                23 => (NonTerminal::Expr, 3),
                24 => (NonTerminal::Expr, 2),
                25 => (NonTerminal::Expr, 1),
                26 => (NonTerminal::Expr, 1),
                27 => (NonTerminal::Expr, 1),
                _ => panic!("malformed production data table"),
            }
        }

        fn goto_state(lhs: &NonTerminal, current_state: u32) -> u32 {
            match current_state {
                0 => match lhs {
                    NonTerminal::Line => 1,
                    NonTerminal::SetUp => 2,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                2 => match lhs {
                    NonTerminal::Expr => 5,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                4 => match lhs {
                    NonTerminal::Line => 10,
                    NonTerminal::SetUp => 2,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                7 => match lhs {
                    NonTerminal::Expr => 16,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                8 => match lhs {
                    NonTerminal::Expr => 18,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                11 => match lhs {
                    NonTerminal::Expr => 19,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                12 => match lhs {
                    NonTerminal::Expr => 20,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                13 => match lhs {
                    NonTerminal::Expr => 21,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                14 => match lhs {
                    NonTerminal::Expr => 22,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                15 => match lhs {
                    NonTerminal::Expr => 23,
                    _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
                },
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            }
        }
    }

    #[test]
    fn calc_works() {
        use crate::parser::Parser;
        let mut calc = Calc::new();
        assert!(calc.parse_text("a = (3 + 4)\n", "raw").is_ok());
    }
}
