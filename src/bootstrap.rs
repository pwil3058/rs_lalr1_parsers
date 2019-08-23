use std::{fs::File, io::Read, rc::Rc};

use crate::{
    attributes::*,
    grammar::GrammarSpecification,
    state::ProductionTail,
    symbols::{AssociativePrecedence, Associativity, SpecialSymbols},
};

use lalr1plus;
use lexan;

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

impl std::fmt::Display for AATerminal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AATerminal::AAEND => write!(f, r###"AAEND"###),
            AATerminal::REGEX => write!(f, r###"REGEX"###),
            AATerminal::LITERAL => write!(f, r###"LITERAL"###),
            AATerminal::TOKEN => write!(f, r###""%token""###),
            AATerminal::LEFT => write!(f, r###""%left""###),
            AATerminal::RIGHT => write!(f, r###""%right""###),
            AATerminal::NONASSOC => write!(f, r###""%nonassoc""###),
            AATerminal::PRECEDENCE => write!(f, r###""%prec""###),
            AATerminal::SKIP => write!(f, r###""%skip""###),
            AATerminal::ERROR => write!(f, r###""%error""###),
            AATerminal::INJECT => write!(f, r###""%inject""###),
            AATerminal::NEWSECTION => write!(f, r###""%%""###),
            AATerminal::COLON => write!(f, r###"":""###),
            AATerminal::VBAR => write!(f, r###""|""###),
            AATerminal::DOT => write!(f, r###"".""###),
            AATerminal::IDENT => write!(f, r###"IDENT"###),
            AATerminal::PREDICATE => write!(f, r###"PREDICATE"###),
            AATerminal::ACTION => write!(f, r###"ACTION"###),
            AATerminal::RUSTCODE => write!(f, r###"RUSTCODE"###),
        }
    }
}

lazy_static! {
    static ref AALEXAN: lexan::LexicalAnalyzer<AATerminal> = {
        use AATerminal::*;
        lexan::LexicalAnalyzer::new(
            &[
                (TOKEN, r###"%token"###),
                (LEFT, r###"%left"###),
                (RIGHT, r###"%right"###),
                (NONASSOC, r###"%nonassoc"###),
                (PRECEDENCE, r###"%prec"###),
                (SKIP, r###"%skip"###),
                (ERROR, r###"%error"###),
                (INJECT, r###"%inject"###),
                (NEWSECTION, r###"%%"###),
                (COLON, r###":"###),
                (VBAR, r###"|"###),
                (DOT, r###"."###),
            ],
            &[
                (REGEX, r###"(\(.+\))"###),
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
    AASTART,
    AALEXICALERROR,
    AASYNTAXERROR,
    AASEMANTICERROR,
    Specification,
    Preamble,
    Definitions,
    ProductionRules,
    OptionalInjection,
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

impl std::fmt::Display for AANonTerminal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AANonTerminal::AASTART => write!(f, r"AASTART"),
            AANonTerminal::AALEXICALERROR => write!(f, r"AALEXICALERROR"),
            AANonTerminal::AASYNTAXERROR => write!(f, r"AASYNTAXERROR"),
            AANonTerminal::AASEMANTICERROR => write!(f, r"AASEMANTICERROR"),
            AANonTerminal::Specification => write!(f, r"Specification"),
            AANonTerminal::Preamble => write!(f, r"Preamble"),
            AANonTerminal::Definitions => write!(f, r"Definitions"),
            AANonTerminal::ProductionRules => write!(f, r"ProductionRules"),
            AANonTerminal::OptionalInjection => write!(f, r"OptionalInjection"),
            AANonTerminal::Injection => write!(f, r"Injection"),
            AANonTerminal::InjectionHead => write!(f, r"InjectionHead"),
            AANonTerminal::TokenDefinitions => write!(f, r"TokenDefinitions"),
            AANonTerminal::SkipDefinitions => write!(f, r"SkipDefinitions"),
            AANonTerminal::PrecedenceDefinitions => write!(f, r"PrecedenceDefinitions"),
            AANonTerminal::TokenDefinition => write!(f, r"TokenDefinition"),
            AANonTerminal::NewTokenName => write!(f, r"NewTokenName"),
            AANonTerminal::Pattern => write!(f, r"Pattern"),
            AANonTerminal::SkipDefinition => write!(f, r"SkipDefinition"),
            AANonTerminal::PrecedenceDefinition => write!(f, r"PrecedenceDefinition"),
            AANonTerminal::TagList => write!(f, r"TagList"),
            AANonTerminal::Tag => write!(f, r"Tag"),
            AANonTerminal::ProductionGroup => write!(f, r"ProductionGroup"),
            AANonTerminal::ProductionGroupHead => write!(f, r"ProductionGroupHead"),
            AANonTerminal::ProductionTailList => write!(f, r"ProductionTailList"),
            AANonTerminal::ProductionTail => write!(f, r"ProductionTail"),
            AANonTerminal::Action => write!(f, r"Action"),
            AANonTerminal::Predicate => write!(f, r"Predicate"),
            AANonTerminal::SymbolList => write!(f, r"SymbolList"),
            AANonTerminal::TaggedPrecedence => write!(f, r"TaggedPrecedence"),
            AANonTerminal::Symbol => write!(f, r"Symbol"),
        }
    }
}

impl lalr1plus::Parser<AATerminal, AANonTerminal, AttributeData> for GrammarSpecification {
    fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<AATerminal> {
        &AALEXAN
    }

    fn viable_error_recovery_states(token: &AATerminal) -> Vec<u32> {
        match token {
            _ => vec![],
        }
    }

    fn error_goto_state(state: u32) -> u32 {
        match state {
            _ => panic!("No error go to state for {}", state),
        }
    }

    fn next_action(
        &self,
        state: u32,
        aa_attributes: &lalr1plus::ParseStack<AATerminal, AANonTerminal, AttributeData>,
        token: &lexan::Token<AATerminal>,
    ) -> lalr1plus::Action<AATerminal> {
        use lalr1plus::Action;
        use AATerminal::*;
        let aa_tag = *token.tag();
        return match state {
            0 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                RUSTCODE => Action::Reduce(2),
                // Preamble: <empty>
                TOKEN => Action::Reduce(6),
                _ => Action::SyntaxError(vec![TOKEN, INJECT, RUSTCODE]),
            },
            1 => match aa_tag {
                // AASTART: Specification
                AAEND => Action::Accept,
                _ => Action::SyntaxError(vec![AAEND]),
            },
            2 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                TOKEN => Action::Reduce(2),
                _ => Action::SyntaxError(vec![TOKEN, INJECT]),
            },
            3 => match aa_tag {
                // OptionalInjection: Injection
                AAEND | TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION | IDENT
                | RUSTCODE => Action::Reduce(3),
                _ => Action::SyntaxError(vec![
                    AAEND, TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION, IDENT, RUSTCODE,
                ]),
            },
            4 => match aa_tag {
                LITERAL => Action::Shift(10),
                _ => Action::SyntaxError(vec![LITERAL]),
            },
            5 => match aa_tag {
                _ => Action::SyntaxError(vec![]),
            },
            6 => match aa_tag {
                RUSTCODE => Action::Shift(12),
                _ => Action::SyntaxError(vec![RUSTCODE]),
            },
            7 => match aa_tag {
                NEWSECTION => Action::Shift(13),
                _ => Action::SyntaxError(vec![NEWSECTION]),
            },
            8 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                TOKEN => Action::Reduce(2),
                // SkipDefinitions: <empty>
                LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(16),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            9 => match aa_tag {
                TOKEN => Action::Shift(17),
                _ => Action::SyntaxError(vec![TOKEN]),
            },
            10 => match aa_tag {
                // InjectionHead: "%inject" LITERAL
                DOT => Action::Reduce(4),
                _ => Action::SyntaxError(vec![DOT]),
            },
            11 => match aa_tag {
                // Injection: InjectionHead "."
                AAEND | TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION | IDENT
                | RUSTCODE => Action::Reduce(5),
                _ => Action::SyntaxError(vec![
                    AAEND, TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION, IDENT, RUSTCODE,
                ]),
            },
            12 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                TOKEN => Action::Reduce(2),
                _ => Action::SyntaxError(vec![TOKEN, INJECT]),
            },
            13 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                IDENT => Action::Reduce(2),
                _ => Action::SyntaxError(vec![INJECT, IDENT]),
            },
            14 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                SKIP => Action::Reduce(2),
                // PrecedenceDefinitions: <empty>
                LEFT | RIGHT | NONASSOC | NEWSECTION => Action::Reduce(19),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            15 => match aa_tag {
                TOKEN => Action::Shift(17),
                _ => Action::SyntaxError(vec![TOKEN]),
            },
            16 => match aa_tag {
                // TokenDefinitions: OptionalInjection TokenDefinition
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(9),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            17 => match aa_tag {
                IDENT => Action::Shift(25),
                _ => Action::SyntaxError(vec![IDENT]),
            },
            18 => match aa_tag {
                // Preamble: OptionalInjection RUSTCODE OptionalInjection
                TOKEN | INJECT => Action::Reduce(7),
                _ => Action::SyntaxError(vec![TOKEN, INJECT]),
            },
            19 => match aa_tag {
                IDENT => Action::Shift(28),
                // Specification: Preamble Definitions "%%" ProductionRules
                AAEND => Action::Reduce(1),
                _ => Action::SyntaxError(vec![AAEND, IDENT]),
            },
            20 => match aa_tag {
                IDENT => Action::Shift(28),
                _ => Action::SyntaxError(vec![IDENT]),
            },
            21 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                LEFT | RIGHT | NONASSOC => Action::Reduce(2),
                // Definitions: TokenDefinitions SkipDefinitions PrecedenceDefinitions
                NEWSECTION => Action::Reduce(8),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION]),
            },
            22 => match aa_tag {
                SKIP => Action::Shift(32),
                _ => Action::SyntaxError(vec![SKIP]),
            },
            23 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            24 => match aa_tag {
                REGEX => Action::Shift(35),
                LITERAL => Action::Shift(36),
                _ => Action::SyntaxError(vec![REGEX, LITERAL]),
            },
            25 => match aa_tag {
                REGEX | LITERAL => {
                    if !Self::is_allowable_name(
                        aa_attributes.at_len_minus_n(1).matched_text().unwrap(),
                    ) {
                        // NewTokenName: IDENT ?( !Self::is_allowable_name($1.matched_text().unwrap()) ?)
                        Action::Reduce(12)
                    } else {
                        // NewTokenName: IDENT
                        Action::Reduce(13)
                    }
                }
                _ => Action::SyntaxError(vec![REGEX, LITERAL]),
            },
            26 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                AAEND | IDENT => Action::Reduce(2),
                _ => Action::SyntaxError(vec![AAEND, INJECT, IDENT]),
            },
            27 => match aa_tag {
                LITERAL => Action::Shift(47),
                ERROR => Action::Shift(48),
                IDENT => Action::Shift(46),
                PREDICATE => Action::Shift(44),
                ACTION => Action::Shift(43),
                // ProductionTail: <empty>
                VBAR | DOT => Action::Reduce(34),
                _ => Action::SyntaxError(vec![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            28 => match aa_tag {
                COLON => Action::Shift(49),
                _ => Action::SyntaxError(vec![COLON]),
            },
            29 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                AAEND | IDENT => Action::Reduce(2),
                _ => Action::SyntaxError(vec![AAEND, INJECT, IDENT]),
            },
            30 => match aa_tag {
                LEFT => Action::Shift(52),
                RIGHT => Action::Shift(53),
                NONASSOC => Action::Shift(54),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC]),
            },
            31 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            32 => match aa_tag {
                REGEX => Action::Shift(56),
                _ => Action::SyntaxError(vec![REGEX]),
            },
            33 => match aa_tag {
                // TokenDefinitions: TokenDefinitions OptionalInjection TokenDefinition OptionalInjection
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(10),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            34 => match aa_tag {
                // TokenDefinition: "%token" NewTokenName Pattern
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(11),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            35 => match aa_tag {
                // Pattern: REGEX
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(14),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            36 => match aa_tag {
                // Pattern: LITERAL
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(15),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            37 => match aa_tag {
                // ProductionRules: ProductionRules ProductionGroup OptionalInjection
                AAEND | IDENT => Action::Reduce(29),
                _ => Action::SyntaxError(vec![AAEND, IDENT]),
            },
            38 => match aa_tag {
                VBAR => Action::Shift(58),
                DOT => Action::Shift(57),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            39 => match aa_tag {
                // ProductionTailList: ProductionTail
                VBAR | DOT => Action::Reduce(32),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            40 => match aa_tag {
                // ProductionTail: Action
                VBAR | DOT => Action::Reduce(35),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            41 => match aa_tag {
                ACTION => Action::Shift(43),
                // ProductionTail: Predicate
                VBAR | DOT => Action::Reduce(37),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            42 => match aa_tag {
                LITERAL => Action::Shift(47),
                PRECEDENCE => Action::Shift(63),
                ERROR => Action::Shift(48),
                IDENT => Action::Shift(46),
                PREDICATE => Action::Shift(44),
                ACTION => Action::Shift(43),
                // ProductionTail: SymbolList
                VBAR | DOT => Action::Reduce(45),
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            43 => match aa_tag {
                // Action: ACTION
                VBAR | DOT => Action::Reduce(46),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            44 => match aa_tag {
                // Predicate: PREDICATE
                PRECEDENCE | VBAR | DOT | ACTION => Action::Reduce(47),
                _ => Action::SyntaxError(vec![PRECEDENCE, VBAR, DOT, ACTION]),
            },
            45 => match aa_tag {
                // SymbolList: Symbol
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(50)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            46 => match aa_tag {
                // Symbol: IDENT
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(52)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            47 => match aa_tag {
                // Symbol: LITERAL
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(53)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            48 => match aa_tag {
                // Symbol: "%error"
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(54)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            49 => match aa_tag {
                // ProductionGroupHead: IDENT ":"
                LITERAL | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => Action::Reduce(31),
                _ => Action::SyntaxError(vec![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            50 => match aa_tag {
                // ProductionRules: OptionalInjection ProductionGroup OptionalInjection
                AAEND | IDENT => Action::Reduce(28),
                _ => Action::SyntaxError(vec![AAEND, IDENT]),
            },
            51 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                LEFT | RIGHT | NONASSOC | NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION]),
            },
            52 => match aa_tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            53 => match aa_tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            54 => match aa_tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            55 => match aa_tag {
                // SkipDefinitions: SkipDefinitions OptionalInjection SkipDefinition OptionalInjection
                LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(17),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            56 => match aa_tag {
                // SkipDefinition: "%skip" REGEX
                LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(18),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            57 => match aa_tag {
                // ProductionGroup: ProductionGroupHead ProductionTailList "."
                AAEND | INJECT | IDENT => Action::Reduce(30),
                _ => Action::SyntaxError(vec![AAEND, INJECT, IDENT]),
            },
            58 => match aa_tag {
                LITERAL => Action::Shift(47),
                ERROR => Action::Shift(48),
                IDENT => Action::Shift(46),
                PREDICATE => Action::Shift(44),
                ACTION => Action::Shift(43),
                // ProductionTail: <empty>
                VBAR | DOT => Action::Reduce(34),
                _ => Action::SyntaxError(vec![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            59 => match aa_tag {
                // ProductionTail: Predicate Action
                VBAR | DOT => Action::Reduce(36),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            60 => match aa_tag {
                PRECEDENCE => Action::Shift(63),
                ACTION => Action::Shift(43),
                // ProductionTail: SymbolList Predicate
                VBAR | DOT => Action::Reduce(41),
                _ => Action::SyntaxError(vec![PRECEDENCE, VBAR, DOT, ACTION]),
            },
            61 => match aa_tag {
                ACTION => Action::Shift(43),
                // ProductionTail: SymbolList TaggedPrecedence
                VBAR | DOT => Action::Reduce(43),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            62 => match aa_tag {
                // ProductionTail: SymbolList Action
                VBAR | DOT => Action::Reduce(44),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            63 => match aa_tag {
                LITERAL => Action::Shift(77),
                IDENT => Action::Shift(76),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            64 => match aa_tag {
                // SymbolList: SymbolList Symbol
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(51)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            65 => match aa_tag {
                // PrecedenceDefinitions: PrecedenceDefinitions OptionalInjection PrecedenceDefinition OptionalInjection
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(20),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION]),
            },
            66 => match aa_tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                // PrecedenceDefinition: "%left" TagList
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(21),
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            67 => match aa_tag {
                // TagList: Tag
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(24)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            68 => match aa_tag {
                // Tag: LITERAL
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(26)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            69 => match aa_tag {
                // Tag: IDENT
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(27)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            70 => match aa_tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                // PrecedenceDefinition: "%right" TagList
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(22),
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            71 => match aa_tag {
                LITERAL => Action::Shift(68),
                IDENT => Action::Shift(69),
                // PrecedenceDefinition: "%nonassoc" TagList
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(23),
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            72 => match aa_tag {
                // ProductionTailList: ProductionTailList "|" ProductionTail
                VBAR | DOT => Action::Reduce(33),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            73 => match aa_tag {
                ACTION => Action::Shift(43),
                // ProductionTail: SymbolList Predicate TaggedPrecedence
                VBAR | DOT => Action::Reduce(39),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            74 => match aa_tag {
                // ProductionTail: SymbolList Predicate Action
                VBAR | DOT => Action::Reduce(40),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            75 => match aa_tag {
                // ProductionTail: SymbolList TaggedPrecedence Action
                VBAR | DOT => Action::Reduce(42),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            76 => match aa_tag {
                // TaggedPrecedence: "%prec" IDENT
                VBAR | DOT | ACTION => Action::Reduce(48),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            77 => match aa_tag {
                // TaggedPrecedence: "%prec" LITERAL
                VBAR | DOT | ACTION => Action::Reduce(49),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            78 => match aa_tag {
                // TagList: TagList Tag
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(25)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            79 => match aa_tag {
                // ProductionTail: SymbolList Predicate TaggedPrecedence Action
                VBAR | DOT => Action::Reduce(38),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            _ => panic!("illegal state: {}", state),
        };
    }

    fn production_data(production_id: u32) -> (AANonTerminal, usize) {
        match production_id {
            0 => (AANonTerminal::AASTART, 1),
            1 => (AANonTerminal::Specification, 4),
            2 => (AANonTerminal::OptionalInjection, 0),
            3 => (AANonTerminal::OptionalInjection, 1),
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
            28 => (AANonTerminal::ProductionRules, 3),
            29 => (AANonTerminal::ProductionRules, 3),
            30 => (AANonTerminal::ProductionGroup, 3),
            31 => (AANonTerminal::ProductionGroupHead, 2),
            32 => (AANonTerminal::ProductionTailList, 1),
            33 => (AANonTerminal::ProductionTailList, 3),
            34 => (AANonTerminal::ProductionTail, 0),
            35 => (AANonTerminal::ProductionTail, 1),
            36 => (AANonTerminal::ProductionTail, 2),
            37 => (AANonTerminal::ProductionTail, 1),
            38 => (AANonTerminal::ProductionTail, 4),
            39 => (AANonTerminal::ProductionTail, 3),
            40 => (AANonTerminal::ProductionTail, 3),
            41 => (AANonTerminal::ProductionTail, 2),
            42 => (AANonTerminal::ProductionTail, 3),
            43 => (AANonTerminal::ProductionTail, 2),
            44 => (AANonTerminal::ProductionTail, 2),
            45 => (AANonTerminal::ProductionTail, 1),
            46 => (AANonTerminal::Action, 1),
            47 => (AANonTerminal::Predicate, 1),
            48 => (AANonTerminal::TaggedPrecedence, 2),
            49 => (AANonTerminal::TaggedPrecedence, 2),
            50 => (AANonTerminal::SymbolList, 1),
            51 => (AANonTerminal::SymbolList, 2),
            52 => (AANonTerminal::Symbol, 1),
            53 => (AANonTerminal::Symbol, 1),
            54 => (AANonTerminal::Symbol, 1),
            _ => panic!("malformed production data table"),
        }
    }

    fn goto_state(lhs: &AANonTerminal, current_state: u32) -> u32 {
        return match current_state {
            0 => match lhs {
                AANonTerminal::Specification => 1,
                AANonTerminal::Preamble => 2,
                AANonTerminal::OptionalInjection => 6,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            2 => match lhs {
                AANonTerminal::Definitions => 7,
                AANonTerminal::OptionalInjection => 9,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                AANonTerminal::TokenDefinitions => 8,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            8 => match lhs {
                AANonTerminal::OptionalInjection => 15,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                AANonTerminal::SkipDefinitions => 14,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            9 => match lhs {
                AANonTerminal::TokenDefinition => 16,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            12 => match lhs {
                AANonTerminal::OptionalInjection => 18,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            13 => match lhs {
                AANonTerminal::ProductionRules => 19,
                AANonTerminal::OptionalInjection => 20,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            14 => match lhs {
                AANonTerminal::OptionalInjection => 22,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                AANonTerminal::PrecedenceDefinitions => 21,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            15 => match lhs {
                AANonTerminal::TokenDefinition => 23,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            17 => match lhs {
                AANonTerminal::NewTokenName => 24,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            19 => match lhs {
                AANonTerminal::ProductionGroup => 26,
                AANonTerminal::ProductionGroupHead => 27,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            20 => match lhs {
                AANonTerminal::ProductionGroup => 29,
                AANonTerminal::ProductionGroupHead => 27,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            21 => match lhs {
                AANonTerminal::OptionalInjection => 30,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            22 => match lhs {
                AANonTerminal::SkipDefinition => 31,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            23 => match lhs {
                AANonTerminal::OptionalInjection => 33,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            24 => match lhs {
                AANonTerminal::Pattern => 34,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            26 => match lhs {
                AANonTerminal::OptionalInjection => 37,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            27 => match lhs {
                AANonTerminal::ProductionTailList => 38,
                AANonTerminal::ProductionTail => 39,
                AANonTerminal::Action => 40,
                AANonTerminal::Predicate => 41,
                AANonTerminal::SymbolList => 42,
                AANonTerminal::Symbol => 45,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            29 => match lhs {
                AANonTerminal::OptionalInjection => 50,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            30 => match lhs {
                AANonTerminal::PrecedenceDefinition => 51,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            31 => match lhs {
                AANonTerminal::OptionalInjection => 55,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            41 => match lhs {
                AANonTerminal::Action => 59,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            42 => match lhs {
                AANonTerminal::Action => 62,
                AANonTerminal::Predicate => 60,
                AANonTerminal::TaggedPrecedence => 61,
                AANonTerminal::Symbol => 64,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            51 => match lhs {
                AANonTerminal::OptionalInjection => 65,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            52 => match lhs {
                AANonTerminal::TagList => 66,
                AANonTerminal::Tag => 67,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            53 => match lhs {
                AANonTerminal::TagList => 70,
                AANonTerminal::Tag => 67,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            54 => match lhs {
                AANonTerminal::TagList => 71,
                AANonTerminal::Tag => 67,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            58 => match lhs {
                AANonTerminal::ProductionTail => 72,
                AANonTerminal::Action => 40,
                AANonTerminal::Predicate => 41,
                AANonTerminal::SymbolList => 42,
                AANonTerminal::Symbol => 45,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            60 => match lhs {
                AANonTerminal::Action => 74,
                AANonTerminal::TaggedPrecedence => 73,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            61 => match lhs {
                AANonTerminal::Action => 75,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            66 => match lhs {
                AANonTerminal::Tag => 78,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            70 => match lhs {
                AANonTerminal::Tag => 78,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            71 => match lhs {
                AANonTerminal::Tag => 78,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            73 => match lhs {
                AANonTerminal::Action => 79,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
        };
    }

    fn do_semantic_action(
        &mut self,
        aa_production_id: u32,
        aa_rhs: Vec<AttributeData>,
        aa_token_stream: &mut lexan::TokenStream<AATerminal>,
    ) -> AttributeData {
        let mut aa_lhs = if let Some(a) = aa_rhs.first() {
            a.clone()
        } else {
            AttributeData::default()
        };
        match aa_production_id {
            2 => {
                // OptionalInjection: <empty>
                // no injection so nothing to do
            }
            4 => {
                // InjectionHead: "%inject" LITERAL

                let (text, location) = aa_rhs[1].text_and_location().unwrap();
                let file_path = text.trim_matches('"');
                match File::open(&file_path) {
                    Ok(mut file) => {
                        let mut text = String::new();
                        if let Err(err) = file.read_to_string(&mut text) {
                            self.error(&location, &format!("Injecting: {}", err));
                        } else if text.len() == 0 {
                            self.error(
                                &location,
                                &format!("Injected file \"{}\" is empty.", file_path),
                            );
                        } else {
                            aa_token_stream.inject(text, file_path.to_string());
                        }
                    }
                    Err(err) => self.error(&location, &format!("Injecting: {}.", err)),
                };
            }
            6 => {
                // Preamble: <empty>

                // no Preamble defined so there's nothing to do

            }
            7 => {
                // Preamble: OptionalInjection RUSTCODE OptionalInjection

                let text = aa_rhs[1].matched_text().unwrap();
                self.set_preamble(&text[2..text.len() - 2]);
            }
            11 => {
                // TokenDefinition: "%token" NewTokenName Pattern

                let (name, location) = aa_rhs[1].text_and_location().unwrap();
                let pattern = aa_rhs[2].matched_text().unwrap();
                if let Err(err) = self.symbol_table.new_token(name, pattern, location) {
                    self.error(location, &err.to_string());
                }
            }
            12 => {
                // NewTokenName: IDENT ?( !Self::is_allowable_name($1.matched_text().unwrap()) ?)

                let (name, location) = aa_rhs[0].text_and_location().unwrap();
                self.warning(
                    location,
                    &format!("token name \"{}\" may clash with generated code", name),
                );
            }
            16 => {
                // SkipDefinitions: <empty>

                // do nothing

            }
            18 => {
                // SkipDefinition: "%skip" REGEX

                let skip_rule = aa_rhs[1].matched_text().unwrap();
                self.symbol_table.add_skip_rule(skip_rule);
            }
            19 => {
                // PrecedenceDefinitions: <empty>

                // do nothing

            }
            21 => {
                // PrecedenceDefinition: "%left" TagList

                let mut tag_list = aa_rhs[1].symbol_list().clone();
                self.symbol_table
                    .set_precedences(Associativity::Left, &mut tag_list);
            }
            22 => {
                // PrecedenceDefinition: "%right" TagList

                let mut tag_list = aa_rhs[1].symbol_list().clone();
                self.symbol_table
                    .set_precedences(Associativity::Right, &mut tag_list);
            }
            23 => {
                // PrecedenceDefinition: "%nonassoc" TagList

                let mut tag_list = aa_rhs[1].symbol_list().clone();
                self.symbol_table
                    .set_precedences(Associativity::NonAssoc, &mut tag_list);
            }
            24 => {
                // TagList: Tag

                let tag = aa_rhs[0].symbol();
                aa_lhs = AttributeData::SymbolList(vec![Rc::clone(&tag)]);
            }
            25 => {
                // TagList: TagList Tag

                let mut tag_list = aa_rhs[0].symbol_list().clone();
                let tag = aa_rhs[1].symbol();
                tag_list.push(Rc::clone(&tag));
                aa_lhs = AttributeData::SymbolList(tag_list);
            }
            26 => {
                // Tag: LITERAL

                let (text, location) = aa_rhs[0].text_and_location().unwrap();
                if let Some(symbol) = self.symbol_table.get_literal_token(text, location) {
                    aa_lhs = AttributeData::Symbol(Rc::clone(symbol));
                } else {
                    let symbol = self
                        .symbol_table
                        .special_symbol(&SpecialSymbols::LexicalError);
                    aa_lhs = AttributeData::Symbol(symbol);
                    let msg = format!("Literal token \"{}\" is not known", text);
                    self.error(location, &msg);
                }
            }
            27 => {
                // Tag: IDENT

                let (name, location) = aa_rhs[0].text_and_location().unwrap();
                if let Some(symbol) = self.symbol_table.use_symbol_named(name, location) {
                    aa_lhs = AttributeData::Symbol(Rc::clone(symbol));
                    if symbol.is_non_terminal() {
                        self.error(
                            location,
                            &format!(
                                "Non terminal \"{}\" cannot be used as precedence tag.",
                                name
                            ),
                        )
                    }
                } else {
                    if !Self::is_allowable_name(name) {
                        self.warning(
                            location,
                            &format!("tag name \"{}\" may clash with generated code", name),
                        );
                    };
                    match self.symbol_table.new_tag(name, location) {
                        Ok(symbol) => aa_lhs = AttributeData::Symbol(symbol),
                        Err(err) => self.error(location, &err.to_string()),
                    }
                }
            }
            30 => {
                // ProductionGroup: ProductionGroupHead ProductionTailList "."

                let lhs = aa_rhs[0].left_hand_side();
                let tails = aa_rhs[1].production_tail_list();
                for tail in tails.iter() {
                    self.new_production(Rc::clone(&lhs), tail.clone());
                }
            }
            31 => {
                // ProductionGroupHead: IDENT ":"

                let (name, location) = aa_rhs[0].text_and_location().unwrap();
                if let Some(symbol) = self.symbol_table.use_symbol_named(name, location) {
                    aa_lhs = AttributeData::LeftHandSide(Rc::clone(symbol));
                    if symbol.is_non_terminal() {
                        symbol.set_defined_at(location);
                    } else {
                        self.error(
                            location,
                            &format!(
                                "Token/tag \"{}\" cannot be used as left hand side of production.",
                                name
                            ),
                        );
                    }
                } else {
                    if !Self::is_allowable_name(name) {
                        self.warning(
                            location,
                            &format!(
                                "Non terminal name \"{}\" may clash with generated code",
                                name
                            ),
                        );
                    };
                    let non_terminal = self.symbol_table.define_non_terminal(name, location);
                    aa_lhs = AttributeData::LeftHandSide(non_terminal);
                }
            }
            32 => {
                // ProductionTailList: ProductionTail

                let production_tail = aa_rhs[0].production_tail().clone();
                aa_lhs = AttributeData::ProductionTailList(vec![production_tail]);
            }
            33 => {
                // ProductionTailList: ProductionTailList "|" ProductionTail

                let mut production_tail_list = aa_rhs[0].production_tail_list().clone();
                let production_tail = aa_rhs[2].production_tail().clone();
                production_tail_list.push(production_tail);
                aa_lhs = AttributeData::ProductionTailList(production_tail_list);
            }
            34 => {
                // ProductionTail: <empty>

                let tail = ProductionTail::new(vec![], None, None, None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            35 => {
                // ProductionTail: Action

                let action = aa_rhs[0].action().to_string();
                let tail = ProductionTail::new(vec![], None, None, Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            36 => {
                // ProductionTail: Predicate Action

                let predicate = aa_rhs[0].predicate().to_string();
                let action = aa_rhs[1].action().to_string();
                let tail = ProductionTail::new(vec![], Some(predicate), None, Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            37 => {
                // ProductionTail: Predicate

                let predicate = aa_rhs[0].predicate().to_string();
                let tail = ProductionTail::new(vec![], Some(predicate), None, None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            38 => {
                // ProductionTail: SymbolList Predicate TaggedPrecedence Action

                let rhs = aa_rhs[0].symbol_list().clone();
                let predicate = aa_rhs[1].predicate().to_string();
                let tagged_precedence = aa_rhs[2].associative_precedence().clone();
                let action = aa_rhs[3].action().to_string();
                let tail = ProductionTail::new(
                    rhs,
                    Some(predicate),
                    Some(tagged_precedence),
                    Some(action),
                );
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            39 => {
                // ProductionTail: SymbolList Predicate TaggedPrecedence

                let lhs = aa_rhs[0].symbol_list().clone();
                let predicate = aa_rhs[1].predicate().to_string();
                let tagged_precedence = aa_rhs[2].associative_precedence().clone();
                let tail = ProductionTail::new(lhs, Some(predicate), Some(tagged_precedence), None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            40 => {
                // ProductionTail: SymbolList Predicate Action

                let lhs = aa_rhs[0].symbol_list().clone();
                let predicate = aa_rhs[1].predicate().to_string();
                let action = aa_rhs[2].action().to_string();
                let tail = ProductionTail::new(lhs, Some(predicate), None, Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            41 => {
                // ProductionTail: SymbolList Predicate

                let lhs = aa_rhs[0].symbol_list().clone();
                let predicate = aa_rhs[1].predicate().to_string();
                let tail = ProductionTail::new(lhs, Some(predicate), None, None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            42 => {
                // ProductionTail: SymbolList TaggedPrecedence Action

                let lhs = aa_rhs[0].symbol_list().clone();
                let tagged_precedence = aa_rhs[1].associative_precedence().clone();
                let action = aa_rhs[2].action().to_string();
                let tail = ProductionTail::new(lhs, None, Some(tagged_precedence), Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            43 => {
                // ProductionTail: SymbolList TaggedPrecedence

                let lhs = aa_rhs[0].symbol_list().clone();
                let tagged_precedence = aa_rhs[1].associative_precedence().clone();
                let tail = ProductionTail::new(lhs, None, Some(tagged_precedence), None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            44 => {
                // ProductionTail: SymbolList Action

                let lhs = aa_rhs[0].symbol_list().clone();
                let action = aa_rhs[1].action().to_string();
                let tail = ProductionTail::new(lhs, None, None, Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            45 => {
                // ProductionTail: SymbolList

                let lhs = aa_rhs[0].symbol_list().clone();
                let tail = ProductionTail::new(lhs, None, None, None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            46 => {
                // Action: ACTION

                let text = aa_rhs[0].matched_text().unwrap();
                aa_lhs = AttributeData::Action(text[2..text.len() - 2].to_string());
            }
            47 => {
                // Predicate: PREDICATE

                let text = aa_rhs[0].matched_text().unwrap();
                aa_lhs = AttributeData::Predicate(text[2..text.len() - 2].to_string());
            }
            48 => {
                // TaggedPrecedence: "%prec" IDENT

                let (name, location) = aa_rhs[1].text_and_location().unwrap();
                let mut ap = AssociativePrecedence::default();
                if let Some(symbol) = self.symbol_table.use_symbol_named(name, location) {
                    if symbol.is_non_terminal() {
                        self.error(
                            location,
                            &format!("{}: illegal precedence tag (must be token or tag)", name),
                        );
                    } else {
                        ap = symbol.associative_precedence();
                    }
                } else {
                    self.error(location, &format!("{}: unknown symbol", name));
                };
                aa_lhs = AttributeData::AssociativePrecedence(ap);
            }
            49 => {
                // TaggedPrecedence: "%prec" LITERAL

                let (lexeme, location) = aa_rhs[1].text_and_location().unwrap();
                let mut ap = AssociativePrecedence::default();
                if let Some(symbol) = self.symbol_table.get_literal_token(lexeme, location) {
                    if symbol.is_non_terminal() {
                        self.error(
                            location,
                            &format!("{}: illegal precedence tag (must be token or tag)", lexeme),
                        );
                    } else {
                        ap = symbol.associative_precedence();
                    }
                } else {
                    self.error(location, &format!("{}: unknown literal", lexeme));
                };
                aa_lhs = AttributeData::AssociativePrecedence(ap);
            }
            50 => {
                // SymbolList: Symbol

                let symbol = aa_rhs[0].symbol();
                aa_lhs = AttributeData::SymbolList(vec![Rc::clone(&symbol)]);
            }
            51 => {
                // SymbolList: SymbolList Symbol

                let symbol = aa_rhs[1].symbol();
                let mut symbol_list = aa_rhs[0].symbol_list().clone();
                symbol_list.push(Rc::clone(&symbol));
                aa_lhs = AttributeData::SymbolList(symbol_list);
            }
            52 => {
                // Symbol: IDENT

                let (name, location) = aa_rhs[0].text_and_location().unwrap();
                if let Some(symbol) = self.symbol_table.use_symbol_named(name, location) {
                    aa_lhs = AttributeData::Symbol(Rc::clone(symbol));
                } else {
                    let symbol = self.symbol_table.use_new_non_terminal(name, location);
                    aa_lhs = AttributeData::Symbol(symbol);
                }
            }
            53 => {
                // Symbol: LITERAL

                let (lexeme, location) = aa_rhs[0].text_and_location().unwrap();
                if let Some(symbol) = self.symbol_table.get_literal_token(lexeme, location) {
                    aa_lhs = AttributeData::Symbol(Rc::clone(symbol));
                } else {
                    self.error(location, &format!("{}: unknown literal)", lexeme));
                    let symbol = self
                        .symbol_table
                        .special_symbol(&SpecialSymbols::LexicalError);
                    aa_lhs = AttributeData::Symbol(symbol);
                }
            }
            54 => {
                // Symbol: "%error"

                let symbol = self
                    .symbol_table
                    .special_symbol(&SpecialSymbols::SyntaxError);
                aa_lhs = AttributeData::Symbol(symbol);
            }
            _ => (),
        };
        aa_lhs
    }
}
