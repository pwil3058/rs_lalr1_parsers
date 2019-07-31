#[macro_use]
extern crate lazy_static;
extern crate lexan;

pub mod parser;

pub use parser::*;

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

    impl From<parser::Error<Terminal>> for AttributeData {
        fn from(_error: parser::Error<Terminal>) -> Self {
            AttributeData::default()
        }
    }

    const UNDEFINED_VARIABLE: u32 = 1 << 0;
    const DIVIDE_BY_ZERO: u32 = 1 << 1;
    const SYNTAX_ERROR: u32 = 1 << 2;
    const LEXICAL_ERROR: u32 = 1 << 3;

    struct Calc {
        errors: u32,
        variables: HashMap<String, f64>,
    }

    lazy_static! {
        static ref AALEXAN: lexan::LexicalAnalyzer<Terminal> = {
            use Terminal::*;
            lexan::LexicalAnalyzer::new(
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
                    (EOL, r"(\n)"),
                    (Number, r"([0-9]+(\.[0-9]+){0,1})"),
                    (Id, r"([a-zA-Z]+)"),
                ],
                &[r"([\t\r ]+)"],
            )
        };
    }

    impl Calc {
        pub fn new() -> Self {
            Self {
                errors: 0,
                variables: HashMap::new(),
            }
        }

        fn report_errors(&self) {
            if self.errors == 0 {
                println!("no errrs")
            } else {
                if self.errors & UNDEFINED_VARIABLE == UNDEFINED_VARIABLE {
                    println!("undefined variable errors")
                }
                if self.errors & DIVIDE_BY_ZERO == DIVIDE_BY_ZERO {
                    println!("divide by zero errors")
                }
                if self.errors & SYNTAX_ERROR == SYNTAX_ERROR {
                    println!("syntax errors")
                }
                if self.errors & LEXICAL_ERROR == LEXICAL_ERROR {
                    println!("lexical errors")
                }
            }
            println!("#errors = {}", self.errors)
        }
    }

    macro_rules! syntax_error {
        ( $token:expr; $( $tag:expr),* ) => {
            parser::Action::SyntaxError(
                *$token.tag(),
                vec![ $( $tag),* ],
                $token.location().to_string(),
            )
        };
    }

    impl parser::Parser<Terminal, NonTerminal, AttributeData> for Calc {
        fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<Terminal> {
            &AALEXAN
        }

        fn viable_error_recovery_states(tag: &Terminal) -> Vec<u32> {
            use Terminal::*;
            match tag {
                EOL => vec![0, 4],
                _ => vec![],
            }
        }

        fn error_go_state(state: u32) -> u32 {
            match state {
                0 | 4 => 3,
                _ => panic!("No error go to state for {}", state),
            }
        }

        fn next_action<'a>(
            &self,
            state: u32,
            attributes: &parser::ParseStack<Terminal, NonTerminal, AttributeData>,
            token: &lexan::Token<'a, Terminal>,
        ) -> parser::Action<Terminal> {
            use Terminal::*;
            let tag = *token.tag();
            return match state {
                0 => match tag {
                    Minus | LPR | Number | Id => parser::Action::Reduce(8),
                    _ => syntax_error!(token; Minus, LPR, Number, Id),
                },
                1 => match tag {
                    EOL => parser::Action::Shift(4),
                    _ => syntax_error!(token; EOL),
                },
                2 => match tag {
                    Minus => parser::Action::Shift(8),
                    LPR => parser::Action::Shift(7),
                    Number => parser::Action::Shift(9),
                    Id => parser::Action::Shift(6),
                    _ => syntax_error!(token; Minus, LPR, Number, Id),
                },
                3 => match tag {
                    EOL => parser::Action::Reduce(7),
                    _ => syntax_error!(token; EOL),
                },
                4 => match tag {
                    EOL => parser::Action::Reduce(6),
                    Minus | Number | Id | LPR => parser::Action::Reduce(8),
                    _ => syntax_error!(token; EOL, Minus, Number, Id, LPR),
                },
                5 => match tag {
                    Plus => parser::Action::Shift(11),
                    Minus => parser::Action::Shift(12),
                    Times => parser::Action::Shift(13),
                    Divide => parser::Action::Shift(14),
                    EOL => {
                        if self.errors > 0 {
                            parser::Action::Reduce(1)
                        } else {
                            parser::Action::Reduce(2)
                        }
                    }
                    _ => syntax_error!(token; EOL, Plus, Minus, Times, Divide),
                },
                6 => match tag {
                    Assign => parser::Action::Shift(15),
                    EOL | Plus | Minus | Times | Divide => {
                        if self
                            .variables
                            .contains_key(&attributes.attribute_n_from_end(2 - 1).id)
                        {
                            parser::Action::Reduce(26)
                        } else {
                            parser::Action::Reduce(27)
                        }
                    }
                    _ => syntax_error!(token; EOL, Plus, Minus, Times, Divide, Assign),
                },
                7 | 8 => match tag {
                    Minus => parser::Action::Shift(8),
                    LPR => parser::Action::Shift(7),
                    Number => parser::Action::Shift(9),
                    Id => parser::Action::Shift(17),
                    _ => syntax_error!(token; Minus, Number, Id, LPR),
                },
                9 => match tag {
                    EOL | Plus | Minus | Times | Divide | RPR => parser::Action::Reduce(25),
                    _ => syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR),
                },
                10 => match tag {
                    EOL => parser::Action::Reduce(5),
                    _ => syntax_error!(token; EOL),
                },
                11 | 12 | 13 | 14 | 15 => match tag {
                    Minus => parser::Action::Shift(8),
                    LPR => parser::Action::Shift(7),
                    Number => parser::Action::Shift(9),
                    Id => parser::Action::Shift(17),
                    _ => syntax_error!(token; Minus, Number, Id, LPR),
                },
                16 => match tag {
                    Plus => parser::Action::Shift(11),
                    Minus => parser::Action::Shift(12),
                    Times => parser::Action::Shift(13),
                    Divide => parser::Action::Shift(14),
                    RPR => parser::Action::Shift(24),
                    _ => syntax_error!(token; Plus, Minus, Times, Divide, RPR),
                },
                17 => match tag {
                    EOL | Plus | Minus | Times | Divide | RPR => {
                        if self
                            .variables
                            .contains_key(&attributes.attribute_n_from_end(2 - 1).id)
                        {
                            parser::Action::Reduce(26)
                        } else {
                            parser::Action::Reduce(27)
                        }
                    }
                    _ => syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR),
                },
                18 => match tag {
                    EOL | Plus | Minus | Times | Divide | RPR => parser::Action::Reduce(24),
                    _ => syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR),
                },
                19 => match tag {
                    Times => parser::Action::Shift(13),
                    Divide => parser::Action::Shift(14),
                    EOL | Plus | Minus | RPR => {
                        if attributes.attribute_n_from_end(4 - 1).value == 0.0 {
                            parser::Action::Reduce(9)
                        } else if attributes.attribute_n_from_end(4 - 3).value == 0.0 {
                            parser::Action::Reduce(10)
                        } else {
                            parser::Action::Reduce(11)
                        }
                    }
                    _ => syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR),
                },
                20 => match tag {
                    Times => parser::Action::Shift(13),
                    Divide => parser::Action::Shift(14),
                    EOL | Plus | Minus | RPR => {
                        if attributes.attribute_n_from_end(4 - 1).value == 0.0 {
                            parser::Action::Reduce(12)
                        } else if attributes.attribute_n_from_end(4 - 3).value == 0.0 {
                            parser::Action::Reduce(13)
                        } else {
                            parser::Action::Reduce(14)
                        }
                    }
                    _ => syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR),
                },
                21 => match tag {
                    EOL | Plus | Minus | Times | Divide | RPR => {
                        if attributes.attribute_n_from_end(4 - 1).value == 0.0
                            || attributes.attribute_n_from_end(4 - 3).value == 0.0
                        {
                            parser::Action::Reduce(15)
                        } else if attributes.attribute_n_from_end(4 - 1).value == 1.0 {
                            parser::Action::Reduce(16)
                        } else if attributes.attribute_n_from_end(4 - 3).value == 1.0 {
                            parser::Action::Reduce(17)
                        } else {
                            parser::Action::Reduce(18)
                        }
                    }
                    _ => syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR),
                },
                22 => match tag {
                    EOL | Plus | Minus | Times | Divide | RPR => {
                        if attributes.attribute_n_from_end(4 - 1).value == 0.0
                            || attributes.attribute_n_from_end(4 - 3).value == 0.0
                        {
                            parser::Action::Reduce(19)
                        } else if attributes.attribute_n_from_end(4 - 1).value == 1.0 {
                            parser::Action::Reduce(20)
                        } else if attributes.attribute_n_from_end(4 - 3).value == 1.0 {
                            parser::Action::Reduce(21)
                        } else {
                            parser::Action::Reduce(22)
                        }
                    }
                    _ => syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR),
                },
                23 => match tag {
                    Plus => parser::Action::Shift(11),
                    Minus => parser::Action::Shift(12),
                    Times => parser::Action::Shift(13),
                    Divide => parser::Action::Shift(14),
                    EOL => {
                        if self.errors == 0 {
                            parser::Action::Reduce(3)
                        } else {
                            parser::Action::Reduce(4)
                        }
                    }
                    _ => syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR),
                },
                24 => match tag {
                    EOL | Plus | Minus | Times | Divide | RPR => parser::Action::Reduce(23),
                    _ => syntax_error!(token; EOL, Plus, Minus, Times, Divide, RPR),
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
                    if self
                        .variables
                        .contains_key(&attributes.attribute_n_from_end(2 - 1).id)
                    {
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
                    if attributes.attribute_n_from_end(4 - 1).value == 0.0
                        || attributes.attribute_n_from_end(4 - 3).value == 0.0
                    {
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
                    if attributes.attribute_n_from_end(4 - 1).value == 0.0
                        || attributes.attribute_n_from_end(4 - 3).value == 0.0
                    {
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

        fn do_semantic_action(
            &mut self,
            production_id: u32,
            rhs: Vec<AttributeData>,
            token_stream: &mut lexan::TokenStream<Terminal>,
        ) -> AttributeData {
            let mut lhs = AttributeData::default();
            token_stream.inject("", "");
            match production_id {
                1 | 4 => {
                    self.report_errors();
                }
                2 => {
                    println!("{}", rhs[2 - 1].value);
                }
                3 => {
                    self.variables
                        .insert(rhs[2 - 1].id.clone(), rhs[4 - 1].value);
                }
                7 => {
                    self.errors |= SYNTAX_ERROR;
                }
                8 => {
                    self.errors = 0;
                }
                9 => {
                    lhs.value = rhs[3 - 1].value;
                }
                10 => {
                    lhs.value = rhs[1 - 1].value;
                }
                11 => {
                    lhs.value = rhs[1 - 1].value + rhs[3 - 1].value;
                }
                12 => {
                    lhs.value = -rhs[3 - 1].value;
                }
                13 => {
                    lhs.value = rhs[1 - 1].value;
                }
                14 => {
                    lhs.value = rhs[1 - 1].value - rhs[3 - 1].value;
                }
                15 => {
                    lhs.value = -rhs[3 - 1].value;
                }
                16 => {
                    lhs.value = rhs[3 - 1].value;
                }
                17 => {
                    lhs.value = rhs[1 - 1].value;
                }
                18 => {
                    lhs.value = rhs[1 - 1].value * rhs[3 - 1].value;
                }
                19 => {
                    lhs.value = rhs[1 - 1].value;
                }
                20 => {
                    self.errors |= DIVIDE_BY_ZERO;
                }
                21 => {
                    lhs.value = 0.0;
                }
                22 => {
                    lhs.value = rhs[1 - 1].value / rhs[3 - 1].value;
                }
                23 => {
                    lhs.value = rhs[2 - 1].value;
                }
                24 => {
                    lhs.value = -rhs[2 - 1].value;
                }
                25 => {
                    lhs.value = rhs[1 - 1].value;
                }
                26 => {
                    lhs.value = *self.variables.get(&rhs[1 - 1].id).unwrap();
                }
                27 => {
                    self.errors |= UNDEFINED_VARIABLE;
                    lhs.value = 0.0;
                }
                _ => (),
            }
            lhs
        }
    }

    #[test]
    fn calc_works() {
        use crate::parser::Parser;
        let mut calc = Calc::new();
        assert!(calc.parse_text("a = (3 + 4)\n", "raw").is_ok());
        assert_eq!(calc.variables.get("a"), Some(&7.0));
        assert!(calc.parse_text("b = a * 5\n", "raw").is_ok());
        assert_eq!(calc.variables.get("b"), Some(&35.0));
    }
}
