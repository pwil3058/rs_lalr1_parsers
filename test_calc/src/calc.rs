
use std::collections::HashMap;
use std::convert::From;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum AttributeData {
    Token(lexan::Token<AATerminal>),
    Error(lalr1_plus::Error<AATerminal>),
    Value(f64),
    Id(String),
    Default
}

impl Default for AttributeData {
    fn default() -> Self {
        AttributeData::Default
    }
}

impl AttributeData {
    fn id(&self) -> &String {
        match self {
            AttributeData::Id(id) => id,
            _ => panic!("invalid variant"),
        }
    }

    fn value(&self) -> f64 {
        match self {
            AttributeData::Value(value) => *value,
            _ => panic!("invalid variant"),
        }
    }
}

impl From<lexan::Token<AATerminal>> for AttributeData {
    fn from(input: lexan::Token<AATerminal>) -> Self {
        match input.tag() {
            AATerminal::NUMBER => {
                let value = f64::from_str(input.lexeme()).unwrap();
                AttributeData::Value(value)
            }
            AATerminal::ID => {
                let id = input.lexeme().to_string();
                AttributeData::Id(id)
            }
            _ => AttributeData::Token(input.clone()),
        }
    }
}

impl From<lalr1_plus::Error<AATerminal>> for AttributeData {
    fn from(error: lalr1_plus::Error<AATerminal>) -> Self {
        AttributeData::Error(error.clone())
    }
}

const UNDEFINED_VARIABLE: u32 = 1 << 0;
const DIVIDE_BY_ZERO: u32 = 1 << 1;
const SYNTAX_ERROR: u32 = 1 << 2;
const LEXICAL_ERROR: u32 = 1 << 3;


pub struct Calc {
    errors: u32,
    variables: HashMap<String, f64>,
}

impl lalr1_plus::ReportError<AATerminal> for Calc {}

impl Calc {
    pub fn new() -> Self {
        Self { errors: 0, variables: HashMap::new() }
    }

    pub fn variable(&self, name: &str) -> Option<f64> {
        if let Some(value) = self.variables.get(name) {
            Some(*value)
        } else {
            None
        }
    }

    fn report_errors(&self) {
        if self.errors & UNDEFINED_VARIABLE != 0 {
            println!("Undefined variable(s).")
        };
        if self.errors & DIVIDE_BY_ZERO != 0 {
            println!("Divide by zero.")
        };
        if self.errors & SYNTAX_ERROR != 0 {
            println!("Syntax error.")
        };
        if self.errors & LEXICAL_ERROR != 0 {
            println!("Lexical error.")
        };
    }
}

use std::collections::BTreeSet;

macro_rules! btree_set {
    () => { BTreeSet::new() };
    ( $( $x:expr ),* ) => {
        {
            let mut set = BTreeSet::new();
            $( set.insert($x); )*
            set
        }
    };
    ( $( $x:expr ),+ , ) => {
        btree_set![ $( $x ), * ]
    };
}

use lalr1_plus;
use lexan;
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum AATerminal {
    AAEnd,
    EOL,
    PLUS,
    MINUS,
    TIMES,
    DIVIDE,
    ASSIGN,
    NUMBER,
    ID,
    LPR,
    RPR,
}

impl std::fmt::Display for AATerminal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AATerminal::AAEnd => write!(f, r###"AAEnd"###),
            AATerminal::EOL => write!(f, r###"EOL"###),
            AATerminal::PLUS => write!(f, r###""+""###),
            AATerminal::MINUS => write!(f, r###""-""###),
            AATerminal::TIMES => write!(f, r###""*""###),
            AATerminal::DIVIDE => write!(f, r###""/""###),
            AATerminal::ASSIGN => write!(f, r###""=""###),
            AATerminal::NUMBER => write!(f, r###"NUMBER"###),
            AATerminal::ID => write!(f, r###"ID"###),
            AATerminal::LPR => write!(f, r###""(""###),
            AATerminal::RPR => write!(f, r###"")""###),
        }
    }
}

lazy_static! {
    static ref AALEXAN: lexan::LexicalAnalyzer<AATerminal> = {
        use AATerminal::*;
        lexan::LexicalAnalyzer::new(
            &[
                (PLUS, r###"+"###),
                (MINUS, r###"-"###),
                (TIMES, r###"*"###),
                (DIVIDE, r###"/"###),
                (ASSIGN, r###"="###),
                (LPR, r###"("###),
                (RPR, r###")"###),
            ],
            &[
                (EOL, r###"(\n)"###),
                (NUMBER, r###"([0-9]+(\.[0-9]+){0,1})"###),
                (ID, r###"([a-zA-Z]+)"###),
            ],
            &[
                r###"([\t\r ]+)"###,
            ],
            AAEnd,
        )
    };
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum AANonTerminal {
    AAStart,
    AAError,
    Line,
    SetUp,
    Expr,
}

impl std::fmt::Display for AANonTerminal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AANonTerminal::AAStart => write!(f, r"AAStart"),
            AANonTerminal::AAError => write!(f, r"AAError"),
            AANonTerminal::Line => write!(f, r"Line"),
            AANonTerminal::SetUp => write!(f, r"SetUp"),
            AANonTerminal::Expr => write!(f, r"Expr"),
        }
    }
}

impl lalr1_plus::Parser<AATerminal, AANonTerminal, AttributeData> for Calc {
    fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<AATerminal> {
        &AALEXAN
    }

    fn viable_error_recovery_states(token: &AATerminal) -> BTreeSet<u32> {
        match token {
            AATerminal::AAEnd => btree_set![0, 4],
            AATerminal::EOL => btree_set![0, 4],
            _ => btree_set![],
        }
    }

    fn error_goto_state(state: u32) -> u32 {
        match state {
            0 => 3,
            4 => 3,
            _ => panic!("No error go to state for {}", state),
        }
    }

    fn look_ahead_set(state: u32) -> BTreeSet<AATerminal> {
        use AATerminal::*;
        return match state {
            0 => btree_set![AAEnd, EOL, MINUS, NUMBER, ID, LPR],
            1 => btree_set![AAEnd, EOL],
            2 => btree_set![MINUS, NUMBER, ID, LPR],
            3 => btree_set![AAEnd, EOL],
            4 => btree_set![AAEnd, EOL, MINUS, NUMBER, ID, LPR],
            5 => btree_set![AAEnd, EOL, PLUS, MINUS, TIMES, DIVIDE],
            6 => btree_set![AAEnd, EOL, PLUS, MINUS, TIMES, DIVIDE, ASSIGN],
            7 => btree_set![MINUS, NUMBER, ID, LPR],
            8 => btree_set![MINUS, NUMBER, ID, LPR],
            9 => btree_set![AAEnd, EOL, PLUS, MINUS, TIMES, DIVIDE, RPR],
            10 => btree_set![AAEnd, EOL],
            11 => btree_set![MINUS, NUMBER, ID, LPR],
            12 => btree_set![MINUS, NUMBER, ID, LPR],
            13 => btree_set![MINUS, NUMBER, ID, LPR],
            14 => btree_set![MINUS, NUMBER, ID, LPR],
            15 => btree_set![MINUS, NUMBER, ID, LPR],
            16 => btree_set![PLUS, MINUS, TIMES, DIVIDE, RPR],
            17 => btree_set![AAEnd, EOL, PLUS, MINUS, TIMES, DIVIDE, RPR],
            18 => btree_set![AAEnd, EOL, PLUS, MINUS, TIMES, DIVIDE, RPR],
            19 => btree_set![AAEnd, EOL, PLUS, MINUS, TIMES, DIVIDE, RPR],
            20 => btree_set![AAEnd, EOL, PLUS, MINUS, TIMES, DIVIDE, RPR],
            21 => btree_set![AAEnd, EOL, PLUS, MINUS, TIMES, DIVIDE, RPR],
            22 => btree_set![AAEnd, EOL, PLUS, MINUS, TIMES, DIVIDE, RPR],
            23 => btree_set![AAEnd, EOL, PLUS, MINUS, TIMES, DIVIDE],
            24 => btree_set![AAEnd, EOL, PLUS, MINUS, TIMES, DIVIDE, RPR],
            _ => panic!("illegal state: {}", state),
        }
    }

    fn next_action(
        &self,
        aa_state: u32,
        aa_attributes: &lalr1_plus::ParseStack<AATerminal, AANonTerminal, AttributeData>,
        aa_token: &lexan::Token<AATerminal>,
    ) -> lalr1_plus::Action {
        use lalr1_plus::Action;
        use AATerminal::*;
        let aa_tag = *aa_token.tag();
        return match aa_state {
            0 => match aa_tag {
                // SetUp: <empty>
                MINUS | NUMBER | ID | LPR => Action::Reduce(8),
                // AAError: <empty>
                AAEnd | EOL => Action::Reduce(28),
                _ => Action::SyntaxError,
            },
            1 => match aa_tag {
                EOL => Action::Shift(4),
                // AAStart: Line
                AAEnd => Action::Accept,
                _ => Action::SyntaxError,
            },
            2 => match aa_tag {
                MINUS => Action::Shift(8),
                NUMBER => Action::Shift(9),
                ID => Action::Shift(6),
                LPR => Action::Shift(7),
                _ => Action::SyntaxError,
            },
            3 => match aa_tag {
                // Line: AAError
                AAEnd | EOL => Action::Reduce(7),
                _ => Action::SyntaxError,
            },
            4 => match aa_tag {
                // Line: Line EOL
                AAEnd | EOL => Action::Reduce(6),
                // SetUp: <empty>
                MINUS | NUMBER | ID | LPR => Action::Reduce(8),
                _ => Action::SyntaxError,
            },
            5 => match aa_tag {
                PLUS => Action::Shift(11),
                MINUS => Action::Shift(12),
                TIMES => Action::Shift(13),
                DIVIDE => Action::Shift(14),
                AAEnd | EOL => {
                    if self.errors > 0 {
                        // Line: SetUp Expr ?(self.errors > 0?)
                        Action::Reduce(1)
                    } else {
                        // Line: SetUp Expr
                        Action::Reduce(2)
                    }
                }
                _ => Action::SyntaxError,
            },
            6 => match aa_tag {
                ASSIGN => Action::Shift(15),
                AAEnd | EOL | PLUS | MINUS | TIMES | DIVIDE => {
                    if self.variables.contains_key(aa_attributes.at_len_minus_n(1).id()) {
                        // Expr: ID ?(self.variables.contains_key($1.id())?)
                        Action::Reduce(26)
                    } else {
                        // Expr: ID
                        Action::Reduce(27)
                    }
                }
                _ => Action::SyntaxError,
            },
            7 => match aa_tag {
                MINUS => Action::Shift(8),
                NUMBER => Action::Shift(9),
                ID => Action::Shift(17),
                LPR => Action::Shift(7),
                _ => Action::SyntaxError,
            },
            8 => match aa_tag {
                MINUS => Action::Shift(8),
                NUMBER => Action::Shift(9),
                ID => Action::Shift(17),
                LPR => Action::Shift(7),
                _ => Action::SyntaxError,
            },
            9 => match aa_tag {
                // Expr: NUMBER
                AAEnd | EOL | PLUS | MINUS | TIMES | DIVIDE | RPR => Action::Reduce(25),
                _ => Action::SyntaxError,
            },
            10 => match aa_tag {
                // Line: Line EOL Line
                AAEnd | EOL => Action::Reduce(5),
                _ => Action::SyntaxError,
            },
            11 => match aa_tag {
                MINUS => Action::Shift(8),
                NUMBER => Action::Shift(9),
                ID => Action::Shift(17),
                LPR => Action::Shift(7),
                _ => Action::SyntaxError,
            },
            12 => match aa_tag {
                MINUS => Action::Shift(8),
                NUMBER => Action::Shift(9),
                ID => Action::Shift(17),
                LPR => Action::Shift(7),
                _ => Action::SyntaxError,
            },
            13 => match aa_tag {
                MINUS => Action::Shift(8),
                NUMBER => Action::Shift(9),
                ID => Action::Shift(17),
                LPR => Action::Shift(7),
                _ => Action::SyntaxError,
            },
            14 => match aa_tag {
                MINUS => Action::Shift(8),
                NUMBER => Action::Shift(9),
                ID => Action::Shift(17),
                LPR => Action::Shift(7),
                _ => Action::SyntaxError,
            },
            15 => match aa_tag {
                MINUS => Action::Shift(8),
                NUMBER => Action::Shift(9),
                ID => Action::Shift(17),
                LPR => Action::Shift(7),
                _ => Action::SyntaxError,
            },
            16 => match aa_tag {
                PLUS => Action::Shift(11),
                MINUS => Action::Shift(12),
                TIMES => Action::Shift(13),
                DIVIDE => Action::Shift(14),
                RPR => Action::Shift(24),
                _ => Action::SyntaxError,
            },
            17 => match aa_tag {
                AAEnd | EOL | PLUS | MINUS | TIMES | DIVIDE | RPR => {
                    if self.variables.contains_key(aa_attributes.at_len_minus_n(1).id()) {
                        // Expr: ID ?(self.variables.contains_key($1.id())?)
                        Action::Reduce(26)
                    } else {
                        // Expr: ID
                        Action::Reduce(27)
                    }
                }
                _ => Action::SyntaxError,
            },
            18 => match aa_tag {
                // Expr: "-" Expr
                AAEnd | EOL | PLUS | MINUS | TIMES | DIVIDE | RPR => Action::Reduce(24),
                _ => Action::SyntaxError,
            },
            19 => match aa_tag {
                TIMES => Action::Shift(13),
                DIVIDE => Action::Shift(14),
                AAEnd | EOL | PLUS | MINUS | RPR => {
                    if aa_attributes.at_len_minus_n(3).value() == 0.0 {
                        // Expr: Expr "+" Expr ?($1.value() == 0.0?)
                        Action::Reduce(9)
                    } else if aa_attributes.at_len_minus_n(1).value() == 0.0 {
                        // Expr: Expr "+" Expr ?($3.value() == 0.0?)
                        Action::Reduce(10)
                    } else {
                        // Expr: Expr "+" Expr
                        Action::Reduce(11)
                    }
                }
                _ => Action::SyntaxError,
            },
            20 => match aa_tag {
                TIMES => Action::Shift(13),
                DIVIDE => Action::Shift(14),
                AAEnd | EOL | PLUS | MINUS | RPR => {
                    if aa_attributes.at_len_minus_n(3).value() == 0.0 {
                        // Expr: Expr "-" Expr ?($1.value() == 0.0?)
                        Action::Reduce(12)
                    } else if aa_attributes.at_len_minus_n(1).value() == 0.0 {
                        // Expr: Expr "-" Expr ?($3.value() == 0.0?)
                        Action::Reduce(13)
                    } else {
                        // Expr: Expr "-" Expr
                        Action::Reduce(14)
                    }
                }
                _ => Action::SyntaxError,
            },
            21 => match aa_tag {
                AAEnd | EOL | PLUS | MINUS | TIMES | DIVIDE | RPR => {
                    if aa_attributes.at_len_minus_n(3).value() == 0.0 || aa_attributes.at_len_minus_n(1).value() == 0.0 {
                        // Expr: Expr "*" Expr ?($1.value() == 0.0 || $3.value() == 0.0?)
                        Action::Reduce(15)
                    } else if aa_attributes.at_len_minus_n(3).value() == 1.0 {
                        // Expr: Expr "*" Expr ?($1.value() == 1.0?)
                        Action::Reduce(16)
                    } else if aa_attributes.at_len_minus_n(1).value() == 1.0 {
                        // Expr: Expr "*" Expr ?($3.value() == 1.0?)
                        Action::Reduce(17)
                    } else {
                        // Expr: Expr "*" Expr
                        Action::Reduce(18)
                    }
                }
                _ => Action::SyntaxError,
            },
            22 => match aa_tag {
                AAEnd | EOL | PLUS | MINUS | TIMES | DIVIDE | RPR => {
                    if aa_attributes.at_len_minus_n(1).value() == 1.0 {
                        // Expr: Expr "/" Expr ?($3.value() == 1.0?)
                        Action::Reduce(19)
                    } else if aa_attributes.at_len_minus_n(1).value() == 0.0 {
                        // Expr: Expr "/" Expr ?($3.value() == 0.0?)
                        Action::Reduce(20)
                    } else if aa_attributes.at_len_minus_n(3).value() == 0.0 {
                        // Expr: Expr "/" Expr ?($1.value() == 0.0?)
                        Action::Reduce(21)
                    } else {
                        // Expr: Expr "/" Expr
                        Action::Reduce(22)
                    }
                }
                _ => Action::SyntaxError,
            },
            23 => match aa_tag {
                PLUS => Action::Shift(11),
                MINUS => Action::Shift(12),
                TIMES => Action::Shift(13),
                DIVIDE => Action::Shift(14),
                AAEnd | EOL => {
                    if self.errors == 0 {
                        // Line: SetUp ID "=" Expr ?(self.errors == 0?)
                        Action::Reduce(3)
                    } else {
                        // Line: SetUp ID "=" Expr
                        Action::Reduce(4)
                    }
                }
                _ => Action::SyntaxError,
            },
            24 => match aa_tag {
                // Expr: "(" Expr ")"
                AAEnd | EOL | PLUS | MINUS | TIMES | DIVIDE | RPR => Action::Reduce(23),
                _ => Action::SyntaxError,
            },
            _ => panic!("illegal state: {}", aa_state),
        }
    }

    fn production_data(production_id: u32) -> (AANonTerminal, usize) {
        match production_id {
            0 => (AANonTerminal::AAStart, 1),
            1 => (AANonTerminal::Line, 2),
            2 => (AANonTerminal::Line, 2),
            3 => (AANonTerminal::Line, 4),
            4 => (AANonTerminal::Line, 4),
            5 => (AANonTerminal::Line, 3),
            6 => (AANonTerminal::Line, 2),
            7 => (AANonTerminal::Line, 1),
            8 => (AANonTerminal::SetUp, 0),
            9 => (AANonTerminal::Expr, 3),
            10 => (AANonTerminal::Expr, 3),
            11 => (AANonTerminal::Expr, 3),
            12 => (AANonTerminal::Expr, 3),
            13 => (AANonTerminal::Expr, 3),
            14 => (AANonTerminal::Expr, 3),
            15 => (AANonTerminal::Expr, 3),
            16 => (AANonTerminal::Expr, 3),
            17 => (AANonTerminal::Expr, 3),
            18 => (AANonTerminal::Expr, 3),
            19 => (AANonTerminal::Expr, 3),
            20 => (AANonTerminal::Expr, 3),
            21 => (AANonTerminal::Expr, 3),
            22 => (AANonTerminal::Expr, 3),
            23 => (AANonTerminal::Expr, 3),
            24 => (AANonTerminal::Expr, 2),
            25 => (AANonTerminal::Expr, 1),
            26 => (AANonTerminal::Expr, 1),
            27 => (AANonTerminal::Expr, 1),
            28 => (AANonTerminal::AAError, 0),
            _ => panic!("malformed production data table"),
        }
    }

    fn goto_state(lhs: &AANonTerminal, current_state: u32) -> u32 {
        return match current_state {
            0 => match lhs {
                AANonTerminal::AAError => 3,
                AANonTerminal::Line => 1,
                AANonTerminal::SetUp => 2,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            2 => match lhs {
                AANonTerminal::Expr => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            4 => match lhs {
                AANonTerminal::AAError => 3,
                AANonTerminal::Line => 10,
                AANonTerminal::SetUp => 2,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            7 => match lhs {
                AANonTerminal::Expr => 16,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            8 => match lhs {
                AANonTerminal::Expr => 18,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            11 => match lhs {
                AANonTerminal::Expr => 19,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            12 => match lhs {
                AANonTerminal::Expr => 20,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            13 => match lhs {
                AANonTerminal::Expr => 21,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            14 => match lhs {
                AANonTerminal::Expr => 22,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            15 => match lhs {
                AANonTerminal::Expr => 23,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
        }
    }

    fn do_semantic_action<F: FnMut(String, String)>(
        &mut self,
        aa_production_id: u32,
        aa_rhs: Vec<AttributeData>,
        mut aa_inject: F,
    ) -> AttributeData {
        let mut aa_lhs = if let Some(a) = aa_rhs.first() {
            a.clone()
        } else {
           AttributeData::default()
        };
        match aa_production_id {
            1 => {
                // Line: SetUp Expr ?(self.errors > 0?)
                self.report_errors();
            }
            2 => {
                // Line: SetUp Expr
                println!("{}", aa_rhs[1].value());
            }
            3 => {
                // Line: SetUp ID "=" Expr ?(self.errors == 0?)
                self.variables.insert(aa_rhs[1].id().clone(), aa_rhs[3].value());
            }
            4 => {
                // Line: SetUp ID "=" Expr
                self.report_errors();
            }
            7 => {
                // Line: AAError
                self.errors |= SYNTAX_ERROR;
            }
            8 => {
                // SetUp: <empty>
                self.errors = 0;
            }
            9 => {
                // Expr: Expr "+" Expr ?($1.value() == 0.0?)
                aa_lhs = AttributeData::Value(aa_rhs[2].value());
            }
            10 => {
                // Expr: Expr "+" Expr ?($3.value() == 0.0?)
                aa_lhs = AttributeData::Value(aa_rhs[0].value());
            }
            11 => {
                // Expr: Expr "+" Expr
                aa_lhs = AttributeData::Value(aa_rhs[0].value() + aa_rhs[2].value());
            }
            12 => {
                // Expr: Expr "-" Expr ?($1.value() == 0.0?)
                aa_lhs = AttributeData::Value(-aa_rhs[2].value());
            }
            13 => {
                // Expr: Expr "-" Expr ?($3.value() == 0.0?)
                aa_lhs = AttributeData::Value(aa_rhs[0].value());
            }
            14 => {
                // Expr: Expr "-" Expr
                aa_lhs = AttributeData::Value(aa_rhs[0].value() - aa_rhs[2].value());
            }
            15 => {
                // Expr: Expr "*" Expr ?($1.value() == 0.0 || $3.value() == 0.0?)
                aa_lhs = AttributeData::Value(-aa_rhs[2].value());
            }
            16 => {
                // Expr: Expr "*" Expr ?($1.value() == 1.0?)
                aa_lhs = AttributeData::Value(aa_rhs[2].value());
            }
            17 => {
                // Expr: Expr "*" Expr ?($3.value() == 1.0?)
                aa_lhs = AttributeData::Value(aa_rhs[0].value());
            }
            18 => {
                // Expr: Expr "*" Expr
                aa_lhs = AttributeData::Value(aa_rhs[0].value() * aa_rhs[2].value());
            }
            19 => {
                // Expr: Expr "/" Expr ?($3.value() == 1.0?)
                aa_lhs = AttributeData::Value(aa_rhs[0].value());
            }
            20 => {
                // Expr: Expr "/" Expr ?($3.value() == 0.0?)
                self.errors |= DIVIDE_BY_ZERO;
            }
            21 => {
                // Expr: Expr "/" Expr ?($1.value() == 0.0?)
                aa_lhs = AttributeData::Value(0.0);
            }
            22 => {
                // Expr: Expr "/" Expr
                aa_lhs = AttributeData::Value(aa_rhs[0].value() / aa_rhs[2].value());
            }
            23 => {
                // Expr: "(" Expr ")"
                aa_lhs = AttributeData::Value(aa_rhs[1].value());
            }
            24 => {
                // Expr: "-" Expr
                aa_lhs = AttributeData::Value(-aa_rhs[1].value());
            }
            25 => {
                // Expr: NUMBER
                aa_lhs = AttributeData::Value(aa_rhs[0].value());
            }
            26 => {
                // Expr: ID ?(self.variables.contains_key($1.id())?)
                aa_lhs = AttributeData::Value(self.variables[aa_rhs[0].id()]);
            }
            27 => {
                // Expr: ID
                self.errors |= UNDEFINED_VARIABLE; aa_lhs = AttributeData::Value(0.0);
            }
            _ => aa_inject(String::new(), String::new()),
        };
        aa_lhs
    }

}
