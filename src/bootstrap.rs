use std::{fmt, fs::File, io::Read, rc::Rc};

use lalr1plus;
use lexan;

use crate::{
    attributes::*,
    grammar::ParserSpecification,
    symbols::Associativity,
};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum AATerminal {
    AAEND,
    REGEX,
    LITERAL,
    TOKEN,
    LEFT,
    RIGHT,
    NONASSOC,
    PRECEDENCE,
    SKIP,
    ERROR,
    INJECT,
    NEWSECTION,
    COLON,
    VBAR,
    DOT,
    IDENT,
    PREDICATE,
    ACTION,
    RUSTCODE,
}

impl fmt::Display for AATerminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AATerminal::*;
        match self {
            AAEND => write!(f, "AAEND"),
            REGEX => write!(f, "REGEX"),
            LITERAL => write!(f, "LITERAL"),
            TOKEN => write!(f, "%token"),
            LEFT => write!(f, "%left"),
            RIGHT => write!(f, "%right"),
            NONASSOC => write!(f, "%nonassoc"),
            PRECEDENCE => write!(f, "%prec"),
            SKIP => write!(f, "%skip"),
            ERROR => write!(f, "%error"),
            INJECT => write!(f, "%inject"),
            NEWSECTION => write!(f, "%%"),
            COLON => write!(f, ":"),
            VBAR => write!(f, "|"),
            DOT => write!(f, "."),
            IDENT => write!(f, "IDENT"),
            PREDICATE => write!(f, "PREDICATE"),
            ACTION => write!(f, "ACTION"),
            RUSTCODE => write!(f, "RUSTCODE"),
        }
    }
}

lazy_static! {
    static ref AALEXAN: lexan::LexicalAnalyzer<AATerminal> = {
        use AATerminal::*;
        lexan::LexicalAnalyzer::new(
            &[
                (TOKEN, "%token"),
                (LEFT, "%left"),
                (RIGHT, "%right"),
                (NONASSOC, "%nonassoc"),
                (PRECEDENCE, "%prec"),
                (SKIP, "%skip"),
                (ERROR, "%error"),
                (INJECT, "%inject"),
                (NEWSECTION, "%%"),
                (COLON, ":"),
                (VBAR, "|"),
                (DOT, "."),
            ],
            &[
                (REGEX, r###"(\(.+\)(?=\s))"###),
                (LITERAL, r###"("(\\"|[^"\t\r\n\v\f])*")"###),
                (IDENT, r###"([a-zA-Z]+[a-zA-Z0-9_]*)"###),
                (PREDICATE, r###"(\?\((.|[\n\r])*?\?\))"###),
                (ACTION, r###"(!\{(.|[\n\r])*?!\})"###),
                (RUSTCODE, r###"(%\{(.|[\n\r])*?%\})"###),
            ],
            &[
                r###"(/\*(.|[\n\r])*?\*/)"###,
                r###"(//[^\n\r]*)"###,
                r###"(\s+)"###,
            ],
            AAEND,
        )
    };
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum AANonTerminal {
    Specification,
    Preamble,
    Definitions,
    ProductionRules,
    OInjection,
    Injection,
    InjectionHead,
    TokenDefinitions,
    SkipDefinitions,
    PrecedenceDefinitions,
    TokenDefinition,
    NewTokenName,
    Pattern,
    SkipDefinition,
    PrecedenceDefinition,
    TagList,
    Tag,
    ProductionGroup,
    ProductionGroupHead,
    ProductionTailList,
    ProductionTail,
    Action,
    Predicate,
    SymbolList,
    TaggedPrecedence,
    Symbol,
}

impl fmt::Display for AANonTerminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AANonTerminal::*;
        match self {
            Specification => write!(f, "Specification"),
            Preamble => write!(f, "Preamble"),
            Definitions => write!(f, "Definitions"),
            ProductionRules => write!(f, "ProductionRules"),
            OInjection => write!(f, "OInjection"),
            Injection => write!(f, "Injection"),
            InjectionHead => write!(f, "InjectionHead"),
            TokenDefinitions => write!(f, "TokenDefinitions"),
            SkipDefinitions => write!(f, "SkipDefinitions"),
            PrecedenceDefinitions => write!(f, "PrecedenceDefinitions"),
            TokenDefinition => write!(f, "TokenDefinition"),
            NewTokenName => write!(f, "NewTokenName"),
            Pattern => write!(f, "Pattern"),
            SkipDefinition => write!(f, "SkipDefinition"),
            PrecedenceDefinition => write!(f, "PrecedenceDefinition"),
            TagList => write!(f, "TagList"),
            Tag => write!(f, "Tag"),
            ProductionGroup => write!(f, "ProductionGroup"),
            ProductionGroupHead => write!(f, "ProductionGroupHead"),
            ProductionTailList => write!(f, "ProductionTailList"),
            ProductionTail => write!(f, "ProductionTail"),
            Action => write!(f, "Action"),
            Predicate => write!(f, "Predicate"),
            SymbolList => write!(f, "SymbolList"),
            TaggedPrecedence => write!(f, "TaggedPrecedence"),
            Symbol => write!(f, "Symbol"),
        }
    }
}

impl lalr1plus::Parser<AATerminal, AANonTerminal, AttributeData<AATerminal>> for ParserSpecification {
    fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<AATerminal> {
        &AALEXAN
    }

    fn viable_error_recovery_states(_tag: &AATerminal) -> Vec<u32> {
        vec![]
    }

    fn error_go_state(state: u32) -> u32 {
        panic!("No error go to state for {}", state)
    }

    fn next_action<'a>(
        &self,
        state: u32,
        aa_attributes: &lalr1plus::ParseStack<AATerminal, AANonTerminal, AttributeData<AATerminal>>,
        token: &lexan::Token<AATerminal>,
    ) -> lalr1plus::Action<AATerminal> {
        use lalr1plus::Action;
        use AATerminal::*;
        let tag = *token.tag();
        match state {
            0 => match tag {
                INJECT => Action::Shift(4),
                TOKEN => Action::Reduce(6),    // preamble: <empty>
                RUSTCODE => Action::Reduce(2), // oinjection: <empty>
                _ => Action::SyntaxError(vec![TOKEN, INJECT, RUSTCODE]),
            },
            1 => match tag {
                AAEND => Action::Accept,
                _ => Action::SyntaxError(vec![AAEND]),
            },
            2 => match tag {
                INJECT => Action::Shift(4),
                TOKEN => Action::Reduce(2), // oinjection: <empty>
                _ => Action::SyntaxError(vec![TOKEN, INJECT]),
            },
            3 => match tag {
                AAEND | TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION | IDENT
                | RUSTCODE => Action::Reduce(3), // oinjection: injection
                _ => Action::SyntaxError(vec![
                    AAEND, TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION, IDENT, RUSTCODE,
                ]),
            },
            4 => match tag {
                LITERAL => Action::Shift(10),
                _ => Action::SyntaxError(vec![LITERAL]),
            },
            5 => match tag {
                DOT => Action::Shift(11),
                _ => Action::SyntaxError(vec![DOT]),
            },
            6 => match tag {
                RUSTCODE => Action::Shift(12),
                _ => Action::SyntaxError(vec![RUSTCODE]),
            },
            7 => match tag {
                NEWSECTION => Action::Shift(13),
                _ => Action::SyntaxError(vec![NEWSECTION]),
            },
            8 => match tag {
                INJECT => Action::Shift(4),
                TOKEN => Action::Reduce(2), // oinjection: <empty>
                LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(16), // skip_definitions: <empty>
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            9 => match tag {
                TOKEN => Action::Shift(17),
                _ => Action::SyntaxError(vec![TOKEN]),
            },
            10 => match tag {
                DOT => Action::Reduce(4), // injection_head: "%inject" LITERAL
                _ => Action::SyntaxError(vec![DOT]),
            },
            11 => match tag {
                AAEND | TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION | IDENT
                | RUSTCODE => Action::Reduce(5), // injection: injection_head "."
                _ => Action::SyntaxError(vec![
                    AAEND, TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION, IDENT, RUSTCODE,
                ]),
            },
            12 => match tag {
                INJECT => Action::Shift(4),
                TOKEN => Action::Reduce(2), // oinjection: <empty>
                _ => Action::SyntaxError(vec![TOKEN, INJECT]),
            },
            13 => match tag {
                INJECT => Action::Shift(4),
                IDENT => Action::Reduce(2), // oinjection: <empty>
                _ => Action::SyntaxError(vec![INJECT, IDENT]),
            },
            14 => match tag {
                INJECT => Action::Shift(4),
                LEFT | RIGHT | NONASSOC | NEWSECTION => Action::Reduce(19), // precedence_definitions: <empty>
                SKIP => Action::Reduce(2),                                  // oinjection: <empty>
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            15 => match tag {
                TOKEN => Action::Shift(17),
                _ => Action::SyntaxError(vec![TOKEN]),
            },
            16 => match tag {
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(9), // token_definitions: oinjection token_definition
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            17 => match tag {
                IDENT => Action::Shift(25),
                _ => Action::SyntaxError(vec![IDENT]),
            },
            18 => match tag {
                TOKEN | INJECT => Action::Reduce(7), // preamble: oinjection RUSTCODE oinjection
                _ => Action::SyntaxError(vec![TOKEN, INJECT]),
            },
            19 => match tag {
                IDENT => Action::Shift(28),
                AAEND => Action::Reduce(1), // specification: preamble definitions "%%" production_rules
                _ => Action::SyntaxError(vec![AAEND, IDENT]),
            },
            20 => match tag {
                IDENT => Action::Shift(28),
                _ => Action::SyntaxError(vec![IDENT]),
            },
            21 => match tag {
                INJECT => Action::Shift(4),
                LEFT | RIGHT | NONASSOC => Action::Reduce(2), // oinjection: <empty>
                NEWSECTION => Action::Reduce(8), // definitions: token_definitions skip_definitions precedence_definitions
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION]),
            },
            22 => match tag {
                SKIP => Action::Shift(32),
                _ => Action::SyntaxError(vec![SKIP]),
            },
            23 => match tag {
                INJECT => Action::Shift(4),
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(2), // oinjection: <empty>
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            24 => match tag {
                REGEX => Action::Shift(35),
                LITERAL => Action::Shift(36),
                _ => Action::SyntaxError(vec![REGEX, LITERAL]),
            },
            25 => match tag {
                REGEX | LITERAL => {
                    if !Self::is_allowable_name(
                        aa_attributes.attribute_n_from_end(2 - 1).matched_text(),
                    ) {
                        Action::Reduce(12) // new_token_name: IDENT ?(  !is_allowable_name($1.matched_text())  ?)
                    } else {
                        Action::Reduce(13) // new_token_name: IDENT
                    }
                }
                _ => Action::SyntaxError(vec![REGEX, LITERAL]),
            },
            26 => match tag {
                INJECT => Action::Shift(4),
                AAEND | IDENT => Action::Reduce(2), // oinjection: <empty>
                _ => Action::SyntaxError(vec![AAEND, INJECT, IDENT]),
            },
            27 => match tag {
                LITERAL => Action::Shift(47),
                ERROR => Action::Shift(48),
                IDENT => Action::Shift(46),
                PREDICATE => Action::Shift(44),
                ACTION => Action::Shift(43),
                VBAR | DOT => Action::Reduce(38), // production_tail: <empty>
                _ => Action::SyntaxError(vec![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            28 => match tag {
                COLON => Action::Shift(49),
                _ => Action::SyntaxError(vec![COLON]),
            },
            29 => match tag {
                INJECT => Action::Shift(4),
                AAEND | IDENT => Action::Reduce(2), // oinjection: <empty>
                _ => Action::SyntaxError(vec![AAEND, INJECT, IDENT]),
            },
            30 => match tag {
                LEFT => Action::Shift(52),
                RIGHT => Action::Shift(53),
                NONASSOC => Action::Shift(54),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC]),
            },
            31 => match tag {
                INJECT => Action::Shift(4),
                LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(2), // oinjection: <empty>
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            32 => match tag {
                REGEX => Action::Shift(56),
                _ => Action::SyntaxError(vec![REGEX]),
            },
            33 => match tag {
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(10), // token_definitions: token_definitions oinjection token_definition oinjection
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            34 => match tag {
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(11), // token_definition: "%token" new_token_name pattern
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            35 => match tag {
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(14), // pattern: REGEX
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            36 => match tag {
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(15), // pattern: LITERAL
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            37 => match tag {
                AAEND | IDENT => Action::Reduce(31), // production_rules: production_rules production_group oinjection
                _ => Action::SyntaxError(vec![AAEND, IDENT]),
            },
            38 => match tag {
                VBAR => Action::Shift(58),
                DOT => Action::Shift(57),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            39 => match tag {
                VBAR | DOT => Action::Reduce(36), // production_tail_list: production_tail
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            40 => match tag {
                VBAR | DOT => Action::Reduce(39), // production_tail: action
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            41 => match tag {
                ACTION => Action::Shift(43),
                VBAR | DOT => Action::Reduce(41), // production_tail: predicate
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            42 => match tag {
                LITERAL => Action::Shift(47),
                PRECEDENCE => Action::Shift(63),
                ERROR => Action::Shift(48),
                IDENT => Action::Shift(46),
                PREDICATE => Action::Shift(44),
                ACTION => Action::Shift(43),
                VBAR | DOT => Action::Reduce(49), // production_tail: symbol_list
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            43 => match tag {
                VBAR | DOT => Action::Reduce(50), // action: ACTION
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            44 => match tag {
                PRECEDENCE | VBAR | DOT | ACTION => Action::Reduce(51), // predicate: PREDICATE
                _ => Action::SyntaxError(vec![PRECEDENCE, VBAR, DOT, ACTION]),
            },
            45 => match tag {
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(54)
                } // symbol_list: symbol
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            46 => match tag {
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(56)
                } // symbol: IDENT
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            47 => match tag {
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(57)
                } // symbol: LITERAL
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            48 => match tag {
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(58)
                } // symbol: "%error"
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            49 => match tag {
                LITERAL | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    if self.is_known_token(aa_attributes.attribute_n_from_end(3 - 1).matched_text())
                    {
                        Action::Reduce(33) // production_group_head: IDENT ":" ?(  self.is_known_token($1.matched_text())  ?)
                    } else if self
                        .is_known_tag(aa_attributes.attribute_n_from_end(3 - 1).matched_text())
                    {
                        Action::Reduce(34) // production_group_head: IDENT ":" ?(  self.is_known_tag($1.matched_text())  ?)
                    } else {
                        Action::Reduce(35) // production_group_head: IDENT ":"
                    }
                }
                _ => Action::SyntaxError(vec![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            50 => match tag {
                AAEND | IDENT => Action::Reduce(30), // production_rules: oinjection production_group oinjection
                _ => Action::SyntaxError(vec![AAEND, IDENT]),
            },
            51 => match tag {
                INJECT => Action::Shift(4),
                LEFT | RIGHT | NONASSOC | NEWSECTION => Action::Reduce(2), // oinjection: <empty>
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION]),
            },
            52 => match tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            53 => match tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            54 => match tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            55 => match tag {
                LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(17), // skip_definitions: skip_definitions oinjection skip_definition oinjection
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            56 => match tag {
                LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(18), // skip_definition: "%skip" REGEX
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            57 => match tag {
                AAEND | INJECT | IDENT => Action::Reduce(32), // production_group: production_group_head production_tail_list "."
                _ => Action::SyntaxError(vec![AAEND, INJECT, IDENT]),
            },
            58 => match tag {
                LITERAL => Action::Shift(47),
                ERROR => Action::Shift(48),
                IDENT => Action::Shift(46),
                PREDICATE => Action::Shift(44),
                ACTION => Action::Shift(43),
                VBAR | DOT => Action::Reduce(38), // production_tail: <empty>
                _ => Action::SyntaxError(vec![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            59 => match tag {
                VBAR | DOT => Action::Reduce(40), // production_tail: predicate action
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            60 => match tag {
                PRECEDENCE => Action::Shift(63),
                ACTION => Action::Shift(43),
                VBAR | DOT => Action::Reduce(45), // production_tail: symbol_list predicate
                _ => Action::SyntaxError(vec![PRECEDENCE, VBAR, DOT, ACTION]),
            },
            61 => match tag {
                ACTION => Action::Shift(43),
                VBAR | DOT => Action::Reduce(47), // production_tail: symbol_list tagged_precedence
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            62 => match tag {
                VBAR | DOT => Action::Reduce(48), // production_tail: symbol_list action
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            63 => match tag {
                LITERAL => Action::Shift(77),
                IDENT => Action::Shift(76),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            64 => match tag {
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(55)
                } // symbol_list: symbol_list symbol
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            65 => match tag {
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(20), // precedence_definitions: precedence_definitions oinjection precedence_definition oinjection
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION]),
            },
            66 => match tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(21), // precedence_definition: "%left" tag_list
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            67 => match tag {
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(24)
                } // tag_list: tag
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            68 => match tag {
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(26)
                } // tag: LITERAL
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            69 => match tag {
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    if self.is_known_token(aa_attributes.attribute_n_from_end(2 - 1).matched_text())
                    {
                        Action::Reduce(27) // tag: IDENT ?(  self.is_known_token($1.matched_text())  ?)
                    } else if self.is_known_non_terminal(
                        aa_attributes.attribute_n_from_end(2 - 1).matched_text(),
                    ) {
                        Action::Reduce(28) // tag: IDENT ?(  self.is_known_non_terminal($1.matched_text())  ?)
                    } else {
                        Action::Reduce(29) // tag: IDENT
                    }
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            70 => match tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(22), // precedence_definition: "%right" tag_list
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            71 => match tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(23), // precedence_definition: "%nonassoc" tag_list
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            72 => match tag {
                VBAR | DOT => Action::Reduce(37), // production_tail_list: production_tail_list "|" production_tail
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            73 => match tag {
                ACTION => Action::Shift(43),
                VBAR | DOT => Action::Reduce(43), // production_tail: symbol_list predicate tagged_precedence
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            74 => match tag {
                VBAR | DOT => Action::Reduce(44), // production_tail: symbol_list predicate action
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            75 => match tag {
                VBAR | DOT => Action::Reduce(46), // production_tail: symbol_list tagged_precedence action
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            76 => match tag {
                VBAR | DOT | ACTION => Action::Reduce(52), // tagged_precedence: "%prec" IDENT
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            77 => match tag {
                VBAR | DOT | ACTION => Action::Reduce(53), // tagged_precedence: "%prec" LITERAL
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            78 => match tag {
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(25)
                } // tag_list: tag_list tag
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            79 => match tag {
                VBAR | DOT => Action::Reduce(42), // production_tail: symbol_list predicate tagged_precedence action
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },

            _ => panic!("{}: invalid parser state.", state),
        }
    }

    fn production_data(production_id: u32) -> (AANonTerminal, usize) {
        match production_id {
            1 => (AANonTerminal::Specification, 5),
            2 => (AANonTerminal::OInjection, 0),
            3 => (AANonTerminal::OInjection, 1),
            4 => (AANonTerminal::InjectionHead, 2),
            5 => (AANonTerminal::Injection, 2),
            6 => (AANonTerminal::Preamble, 0),
            7 => (AANonTerminal::Preamble, 3),
            8 => (AANonTerminal::Definitions, 3),
            9 => (AANonTerminal::TokenDefinitions, 2),
            10 => (AANonTerminal::TokenDefinitions, 4),
            11 => (AANonTerminal::TokenDefinition, 3),
            12 => (AANonTerminal::NewTokenName, 1),
            13 => (AANonTerminal::NewTokenName, 1),
            14 => (AANonTerminal::Pattern, 1),
            15 => (AANonTerminal::Pattern, 1),
            16 => (AANonTerminal::SkipDefinitions, 0),
            17 => (AANonTerminal::SkipDefinitions, 4),
            18 => (AANonTerminal::SkipDefinition, 2),
            19 => (AANonTerminal::PrecedenceDefinitions, 0),
            20 => (AANonTerminal::PrecedenceDefinitions, 4),
            21 => (AANonTerminal::PrecedenceDefinition, 2),
            22 => (AANonTerminal::PrecedenceDefinition, 2),
            23 => (AANonTerminal::PrecedenceDefinition, 2),
            24 => (AANonTerminal::TagList, 1),
            25 => (AANonTerminal::TagList, 2),
            26 => (AANonTerminal::Tag, 1),
            27 => (AANonTerminal::Tag, 1),
            28 => (AANonTerminal::Tag, 1),
            29 => (AANonTerminal::Tag, 1),
            30 => (AANonTerminal::ProductionRules, 3),
            31 => (AANonTerminal::ProductionRules, 3),
            32 => (AANonTerminal::ProductionGroup, 3),
            33 => (AANonTerminal::ProductionGroupHead, 2),
            34 => (AANonTerminal::ProductionGroupHead, 2),
            35 => (AANonTerminal::ProductionGroupHead, 2),
            36 => (AANonTerminal::ProductionTailList, 1),
            37 => (AANonTerminal::ProductionTailList, 3),
            38 => (AANonTerminal::ProductionTail, 0),
            39 => (AANonTerminal::ProductionTail, 1),
            40 => (AANonTerminal::ProductionTail, 2),
            41 => (AANonTerminal::ProductionTail, 1),
            42 => (AANonTerminal::ProductionTail, 4),
            43 => (AANonTerminal::ProductionTail, 3),
            44 => (AANonTerminal::ProductionTail, 3),
            45 => (AANonTerminal::ProductionTail, 2),
            46 => (AANonTerminal::ProductionTail, 3),
            47 => (AANonTerminal::ProductionTail, 2),
            48 => (AANonTerminal::ProductionTail, 2),
            49 => (AANonTerminal::ProductionTail, 1),
            50 => (AANonTerminal::Action, 1),
            51 => (AANonTerminal::Predicate, 1),
            52 => (AANonTerminal::TaggedPrecedence, 2),
            53 => (AANonTerminal::TaggedPrecedence, 2),
            54 => (AANonTerminal::SymbolList, 1),
            55 => (AANonTerminal::SymbolList, 2),
            56 => (AANonTerminal::Symbol, 1),
            57 => (AANonTerminal::Symbol, 1),
            58 => (AANonTerminal::Symbol, 1),
            _ => panic!("Malformed production data table"),
        }
    }

    fn goto_state(lhs: &AANonTerminal, current_state: u32) -> u32 {
        use AANonTerminal::*;
        match current_state {
            0 => match lhs {
                Specification => 1,
                Preamble => 2,
                OInjection => 6,
                Injection => 3,
                InjectionHead => 5,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            2 => match lhs {
                Definitions => 7,
                OInjection => 9,
                Injection => 3,
                InjectionHead => 5,
                TokenDefinitions => 8,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            8 => match lhs {
                OInjection => 15,
                Injection => 3,
                InjectionHead => 5,
                TokenDefinitions => 14,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            9 => match lhs {
                TokenDefinitions => 15,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            12 => match lhs {
                OInjection => 18,
                Injection => 3,
                InjectionHead => 5,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            13 => match lhs {
                ProductionRules => 19,
                OInjection => 20,
                Injection => 3,
                InjectionHead => 5,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            14 => match lhs {
                OInjection => 22,
                Injection => 3,
                InjectionHead => 5,
                PrecedenceDefinitions => 21,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            15 => match lhs {
                TokenDefinition => 23,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            17 => match lhs {
                NewTokenName => 24,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            19 => match lhs {
                ProductionGroup => 26,
                ProductionGroupHead => 27,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            20 => match lhs {
                ProductionGroup => 29,
                ProductionGroupHead => 27,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            21 => match lhs {
                OInjection => 30,
                Injection => 3,
                InjectionHead => 5,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            22 => match lhs {
                SkipDefinition => 31,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            23 => match lhs {
                OInjection => 33,
                Injection => 3,
                InjectionHead => 5,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            24 => match lhs {
                Pattern => 34,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            26 => match lhs {
                OInjection => 37,
                Injection => 3,
                InjectionHead => 5,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            27 => match lhs {
                ProductionTailList => 38,
                ProductionTail => 39,
                Action => 40,
                Predicate => 41,
                SymbolList => 42,
                Symbol => 45,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            29 => match lhs {
                OInjection => 50,
                Injection => 3,
                InjectionHead => 5,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            30 => match lhs {
                PrecedenceDefinition => 51,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            31 => match lhs {
                OInjection => 55,
                Injection => 3,
                InjectionHead => 5,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            41 => match lhs {
                Action => 59,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            42 => match lhs {
                Action => 62,
                Predicate => 60,
                TaggedPrecedence => 61,
                Symbol => 64,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            51 => match lhs {
                OInjection => 65,
                Injection => 3,
                InjectionHead => 5,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            52 => match lhs {
                TagList => 66,
                Tag => 67,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            53 => match lhs {
                TagList => 70,
                Tag => 67,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            54 => match lhs {
                TagList => 71,
                Tag => 67,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            58 => match lhs {
                ProductionTail => 72,
                Action => 40,
                Predicate => 41,
                SymbolList => 42,
                Symbol => 45,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            60 => match lhs {
                Action => 74,
                TaggedPrecedence => 73,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            61 => match lhs {
                Action => 75,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            66 => match lhs {
                Tag => 78,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            70 => match lhs {
                Tag => 78,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            71 => match lhs {
                Tag => 78,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            73 => match lhs {
                Action => 79,
                _ => panic!(
                    "Malformed goto table: no entry for ({} , {})",
                    lhs, current_state
                ),
            },
            _ => panic!(
                "Malformed goto table: no entry for ({}, {}).",
                lhs, current_state
            ),
        }
    }

    fn do_semantic_action(
        &mut self,
        aa_production_id: u32,
        aa_rhs: Vec<AttributeData<AATerminal>>,
        aa_token_stream: &mut lexan::TokenStream<AATerminal>,
    ) -> AttributeData<AATerminal> {
        let mut aa_lhs = AttributeData::default();
        match aa_production_id {
            4 => {
                // injection_head: "%inject" LITERAL
                let file_path = aa_rhs[2 - 1].matched_text().trim_matches('"');
                match File::open(&file_path) {
                    Ok(mut file) => {
                        let mut text = String::new();
                        if let Err(err) = file.read_to_string(&mut text) {
                            self.error(aa_rhs[2 - 1].location(), &format!("Injecting: {}", err));
                        } else if text.len() == 0 {
                            self.error(
                                aa_rhs[2 - 1].location(),
                                &format!("Injected file \"{}\" is empty.", file_path),
                            );
                        } else {
                            aa_token_stream.inject(text, file_path.to_string());
                        }
                    }
                    Err(err) => {
                        self.error(aa_rhs[2 - 1].location(), &format!("Injecting: {}.", err))
                    }
                };
            }
            7 => {
                // preamble: oinjection RUSTCODE oinjection
                let text = aa_rhs[2 - 1].matched_text();
                self.set_preamble(&text[2..text.len() - 2]);
            }
            11 => {
                // token_definition: "%token" new_token_name pattern
                let name = aa_rhs[2 - 1].matched_text();
                let pattern = aa_rhs[3 - 1].matched_text();
                let location = aa_rhs[3 - 1].location();
                if let Err(err) = self.add_token(name, pattern, location) {
                    self.error(location, &err.to_string())
                }
            }
            12 => {
                // new_token_name: IDENT ?( !is_allowable_name($1.matched_text()) ?)
                let name = aa_rhs[1 - 1].matched_text();
                let location = aa_rhs[1 - 1].location();
                self.warning(
                    location,
                    &format!("token name \"{}\" may clash with generated code", name),
                );
                aa_lhs = aa_rhs[1 - 1].clone();
            }
            18 => {
                // skip_definition: "%skip" REGEX
                let skip_rule = aa_rhs[2 - 1].matched_text();
                self.add_skip_rule(skip_rule);
            }
            21 => {
                //  precedence_definition: "%left" tag_list
                let mut tag_list = aa_rhs[2 - 1].symbol_list().clone();
                self.set_precedences(Associativity::Left, &mut tag_list);
                aa_lhs = AttributeData::SymbolList(tag_list);
            }
            22 => {
                //  precedence_definition: "%right" tag_list
                let mut tag_list = aa_rhs[2 - 1].symbol_list().clone();
                self.set_precedences(Associativity::Right, &mut tag_list);
                aa_lhs = AttributeData::SymbolList(tag_list);
            }
            23 => {
                //  precedence_definition: "%nonassoc" tag_list
                let mut tag_list = aa_rhs[2 - 1].symbol_list().clone();
                self.set_precedences(Associativity::NonAssoc, &mut tag_list);
                aa_lhs = AttributeData::SymbolList(tag_list);
            }
            24 => {
                // tag_list: tag
                aa_lhs = if let Some(tag) = aa_rhs[1 - 1].symbol() {
                    AttributeData::SymbolList(vec![tag.clone()])
                } else {
                    AttributeData::SymbolList(vec![])
                }
            }
            25 => {
                // tag_list: tag_list tag
                let mut tag_list = aa_rhs[1 - 1].symbol_list().clone();
                aa_lhs = if let Some(tag) = aa_rhs[2 - 1].symbol() {
                    tag_list.push(tag.clone());
                    AttributeData::SymbolList(tag_list)
                } else {
                    AttributeData::SymbolList(tag_list)
                }
            }
            26 => {
                // tag: LITERAL
                //let name = aa_rhs[1 - 1].matched_text();
                //let location = aa_rhs[1 - 1].location();
                //if let Some(symbol) = self.get_literal_token(name, location) {
                //    aa_lhs = AttributeData::Symbol(Rc::clone(symbol))
                //}
            }
            _ => (),
        }
        aa_lhs
    }
}
