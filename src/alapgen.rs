use std::{fs::File, io::Read, rc::Rc};

use crate::{
    attributes::*,
    grammar::GrammarSpecification,
    state::ProductionTail,
    symbols::{AssociativePrecedence, Associativity},
};

use lalr1plus;
use lexan;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum AATerminal {
    AAEnd,
    REGEX,
    LITERAL,
    ATTR,
    TARGET,
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
            AATerminal::AAEnd => write!(f, r###"AAEnd"###),
            AATerminal::REGEX => write!(f, r###"REGEX"###),
            AATerminal::LITERAL => write!(f, r###"LITERAL"###),
            AATerminal::ATTR => write!(f, r###""%attr""###),
            AATerminal::TARGET => write!(f, r###""%target""###),
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
                (ATTR, r###"%attr"###),
                (TARGET, r###"%target"###),
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
            AAEnd,
        )
    };
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum AANonTerminal {
    AAStart,
    AASyntaxError,
    AALexicalError,
    AASemanticError,
    Specification,
    Preamble,
    Configuration,
    Definitions,
    ProductionRules,
    OptionalInjection,
    Injection,
    InjectionHead,
    AttributeType,
    TargetType,
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
            AANonTerminal::AAStart => write!(f, r"AAStart"),
            AANonTerminal::AASyntaxError => write!(f, r"AASyntaxError"),
            AANonTerminal::AALexicalError => write!(f, r"AALexicalError"),
            AANonTerminal::AASemanticError => write!(f, r"AASemanticError"),
            AANonTerminal::Specification => write!(f, r"Specification"),
            AANonTerminal::Preamble => write!(f, r"Preamble"),
            AANonTerminal::Configuration => write!(f, r"Configuration"),
            AANonTerminal::Definitions => write!(f, r"Definitions"),
            AANonTerminal::ProductionRules => write!(f, r"ProductionRules"),
            AANonTerminal::OptionalInjection => write!(f, r"OptionalInjection"),
            AANonTerminal::Injection => write!(f, r"Injection"),
            AANonTerminal::InjectionHead => write!(f, r"InjectionHead"),
            AANonTerminal::AttributeType => write!(f, r"AttributeType"),
            AANonTerminal::TargetType => write!(f, r"TargetType"),
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
        aa_state: u32,
        aa_attributes: &lalr1plus::ParseStack<AATerminal, AANonTerminal, AttributeData>,
        aa_token: &lexan::Token<AATerminal>,
    ) -> lalr1plus::Action<AATerminal> {
        use lalr1plus::Action;
        use AATerminal::*;
        let aa_tag = *aa_token.tag();
        return match aa_state {
            0 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                RUSTCODE => Action::Reduce(2),
                // Preamble: <empty>
                ATTR | TARGET => Action::Reduce(6),
                _ => Action::SyntaxError(vec![ATTR, TARGET, INJECT, RUSTCODE]),
            },
            1 => match aa_tag {
                // AAStart: Specification
                AAEnd => Action::Accept,
                _ => Action::SyntaxError(vec![AAEnd]),
            },
            2 => match aa_tag {
                ATTR => Action::Shift(10),
                TARGET => Action::Shift(11),
                _ => Action::SyntaxError(vec![ATTR, TARGET]),
            },
            3 => match aa_tag {
                // OptionalInjection: Injection
                AAEnd | ATTR | TARGET | TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT
                | NEWSECTION | IDENT | RUSTCODE => Action::Reduce(3),
                _ => Action::SyntaxError(vec![
                    AAEnd, ATTR, TARGET, TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                    IDENT, RUSTCODE,
                ]),
            },
            4 => match aa_tag {
                LITERAL => Action::Shift(12),
                _ => Action::SyntaxError(vec![LITERAL]),
            },
            5 => match aa_tag {
                DOT => Action::Shift(13),
                _ => Action::SyntaxError(vec![DOT]),
            },
            6 => match aa_tag {
                RUSTCODE => Action::Shift(14),
                _ => Action::SyntaxError(vec![RUSTCODE]),
            },
            7 => match aa_tag {
                NEWSECTION => Action::Shift(15),
                _ => Action::SyntaxError(vec![NEWSECTION]),
            },
            8 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                TARGET => Action::Reduce(2),
                _ => Action::SyntaxError(vec![TARGET, INJECT]),
            },
            9 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                ATTR => Action::Reduce(2),
                _ => Action::SyntaxError(vec![ATTR, INJECT]),
            },
            10 => match aa_tag {
                IDENT => Action::Shift(18),
                _ => Action::SyntaxError(vec![IDENT]),
            },
            11 => match aa_tag {
                IDENT => Action::Shift(19),
                _ => Action::SyntaxError(vec![IDENT]),
            },
            12 => match aa_tag {
                // InjectionHead: "%inject" LITERAL
                DOT => Action::Reduce(4),
                _ => Action::SyntaxError(vec![DOT]),
            },
            13 => match aa_tag {
                // Injection: InjectionHead "."
                AAEnd | ATTR | TARGET | TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT
                | NEWSECTION | IDENT | RUSTCODE => Action::Reduce(5),
                _ => Action::SyntaxError(vec![
                    AAEnd, ATTR, TARGET, TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                    IDENT, RUSTCODE,
                ]),
            },
            14 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                ATTR | TARGET => Action::Reduce(2),
                _ => Action::SyntaxError(vec![ATTR, TARGET, INJECT]),
            },
            15 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                TOKEN => Action::Reduce(2),
                _ => Action::SyntaxError(vec![TOKEN, INJECT]),
            },
            16 => match aa_tag {
                TARGET => Action::Shift(11),
                _ => Action::SyntaxError(vec![TARGET]),
            },
            17 => match aa_tag {
                ATTR => Action::Shift(10),
                _ => Action::SyntaxError(vec![ATTR]),
            },
            18 => match aa_tag {
                // AttributeType: "%attr" IDENT
                TARGET | INJECT | NEWSECTION => Action::Reduce(10),
                _ => Action::SyntaxError(vec![TARGET, INJECT, NEWSECTION]),
            },
            19 => match aa_tag {
                // TargetType: "%target" IDENT
                ATTR | INJECT | NEWSECTION => Action::Reduce(11),
                _ => Action::SyntaxError(vec![ATTR, INJECT, NEWSECTION]),
            },
            20 => match aa_tag {
                // Preamble: OptionalInjection RUSTCODE OptionalInjection
                ATTR | TARGET => Action::Reduce(7),
                _ => Action::SyntaxError(vec![ATTR, TARGET]),
            },
            21 => match aa_tag {
                NEWSECTION => Action::Shift(26),
                _ => Action::SyntaxError(vec![NEWSECTION]),
            },
            22 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                TOKEN => Action::Reduce(2),
                // SkipDefinitions: <empty>
                LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(20),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            23 => match aa_tag {
                TOKEN => Action::Shift(30),
                _ => Action::SyntaxError(vec![TOKEN]),
            },
            24 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError(vec![INJECT, NEWSECTION]),
            },
            25 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError(vec![INJECT, NEWSECTION]),
            },
            26 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                IDENT => Action::Reduce(2),
                _ => Action::SyntaxError(vec![INJECT, IDENT]),
            },
            27 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                SKIP => Action::Reduce(2),
                // PrecedenceDefinitions: <empty>
                LEFT | RIGHT | NONASSOC | NEWSECTION => Action::Reduce(23),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            28 => match aa_tag {
                TOKEN => Action::Shift(30),
                _ => Action::SyntaxError(vec![TOKEN]),
            },
            29 => match aa_tag {
                // TokenDefinitions: OptionalInjection TokenDefinition
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(13),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            30 => match aa_tag {
                IDENT => Action::Shift(39),
                _ => Action::SyntaxError(vec![IDENT]),
            },
            31 => match aa_tag {
                // Configuration: AttributeType OptionalInjection TargetType OptionalInjection
                NEWSECTION => Action::Reduce(8),
                _ => Action::SyntaxError(vec![NEWSECTION]),
            },
            32 => match aa_tag {
                // Configuration: TargetType OptionalInjection AttributeType OptionalInjection
                NEWSECTION => Action::Reduce(9),
                _ => Action::SyntaxError(vec![NEWSECTION]),
            },
            33 => match aa_tag {
                IDENT => Action::Shift(42),
                // Specification: Preamble Configuration "%%" Definitions "%%" ProductionRules
                AAEnd => Action::Reduce(1),
                _ => Action::SyntaxError(vec![AAEnd, IDENT]),
            },
            34 => match aa_tag {
                IDENT => Action::Shift(42),
                _ => Action::SyntaxError(vec![IDENT]),
            },
            35 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                LEFT | RIGHT | NONASSOC => Action::Reduce(2),
                // Definitions: TokenDefinitions SkipDefinitions PrecedenceDefinitions
                NEWSECTION => Action::Reduce(12),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION]),
            },
            36 => match aa_tag {
                SKIP => Action::Shift(46),
                _ => Action::SyntaxError(vec![SKIP]),
            },
            37 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            38 => match aa_tag {
                REGEX => Action::Shift(49),
                LITERAL => Action::Shift(50),
                _ => Action::SyntaxError(vec![REGEX, LITERAL]),
            },
            39 => match aa_tag {
                REGEX | LITERAL => {
                    if !Self::is_allowable_name(aa_attributes.at_len_minus_n(1).matched_text()) {
                        // NewTokenName: IDENT ?( !Self::is_allowable_name($1.matched_text()) ?)
                        Action::Reduce(16)
                    } else {
                        // NewTokenName: IDENT
                        Action::Reduce(17)
                    }
                }
                _ => Action::SyntaxError(vec![REGEX, LITERAL]),
            },
            40 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                AAEnd | IDENT => Action::Reduce(2),
                _ => Action::SyntaxError(vec![AAEnd, INJECT, IDENT]),
            },
            41 => match aa_tag {
                LITERAL => Action::Shift(61),
                ERROR => Action::Shift(62),
                IDENT => Action::Shift(60),
                PREDICATE => Action::Shift(58),
                ACTION => Action::Shift(57),
                // ProductionTail: <empty>
                VBAR | DOT => Action::Reduce(38),
                _ => Action::SyntaxError(vec![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            42 => match aa_tag {
                COLON => Action::Shift(63),
                _ => Action::SyntaxError(vec![COLON]),
            },
            43 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                AAEnd | IDENT => Action::Reduce(2),
                _ => Action::SyntaxError(vec![AAEnd, INJECT, IDENT]),
            },
            44 => match aa_tag {
                LEFT => Action::Shift(66),
                RIGHT => Action::Shift(67),
                NONASSOC => Action::Shift(68),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC]),
            },
            45 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            46 => match aa_tag {
                REGEX => Action::Shift(70),
                _ => Action::SyntaxError(vec![REGEX]),
            },
            47 => match aa_tag {
                // TokenDefinitions: TokenDefinitions OptionalInjection TokenDefinition OptionalInjection
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(14),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            48 => match aa_tag {
                // TokenDefinition: "%token" NewTokenName Pattern
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(15),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            49 => match aa_tag {
                // Pattern: REGEX
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(18),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            50 => match aa_tag {
                // Pattern: LITERAL
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(19),
                _ => Action::SyntaxError(vec![
                    TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION,
                ]),
            },
            51 => match aa_tag {
                // ProductionRules: ProductionRules ProductionGroup OptionalInjection
                AAEnd | IDENT => Action::Reduce(33),
                _ => Action::SyntaxError(vec![AAEnd, IDENT]),
            },
            52 => match aa_tag {
                VBAR => Action::Shift(72),
                DOT => Action::Shift(71),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            53 => match aa_tag {
                // ProductionTailList: ProductionTail
                VBAR | DOT => Action::Reduce(36),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            54 => match aa_tag {
                // ProductionTail: Action
                VBAR | DOT => Action::Reduce(39),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            55 => match aa_tag {
                ACTION => Action::Shift(57),
                // ProductionTail: Predicate
                VBAR | DOT => Action::Reduce(41),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            56 => match aa_tag {
                LITERAL => Action::Shift(61),
                PRECEDENCE => Action::Shift(77),
                ERROR => Action::Shift(62),
                IDENT => Action::Shift(60),
                PREDICATE => Action::Shift(58),
                ACTION => Action::Shift(57),
                // ProductionTail: SymbolList
                VBAR | DOT => Action::Reduce(49),
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            57 => match aa_tag {
                // Action: ACTION
                VBAR | DOT => Action::Reduce(50),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            58 => match aa_tag {
                // Predicate: PREDICATE
                PRECEDENCE | VBAR | DOT | ACTION => Action::Reduce(51),
                _ => Action::SyntaxError(vec![PRECEDENCE, VBAR, DOT, ACTION]),
            },
            59 => match aa_tag {
                // SymbolList: Symbol
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(54)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            60 => match aa_tag {
                // Symbol: IDENT
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(56)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            61 => match aa_tag {
                // Symbol: LITERAL
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(57)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            62 => match aa_tag {
                // Symbol: "%error"
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(58)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            63 => match aa_tag {
                // ProductionGroupHead: IDENT ":"
                LITERAL | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => Action::Reduce(35),
                _ => Action::SyntaxError(vec![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            64 => match aa_tag {
                // ProductionRules: OptionalInjection ProductionGroup OptionalInjection
                AAEnd | IDENT => Action::Reduce(32),
                _ => Action::SyntaxError(vec![AAEnd, IDENT]),
            },
            65 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                LEFT | RIGHT | NONASSOC | NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION]),
            },
            66 => match aa_tag {
                LITERAL => Action::Shift(82),
                IDENT => Action::Shift(83),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            67 => match aa_tag {
                LITERAL => Action::Shift(82),
                IDENT => Action::Shift(83),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            68 => match aa_tag {
                LITERAL => Action::Shift(82),
                IDENT => Action::Shift(83),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            69 => match aa_tag {
                // SkipDefinitions: SkipDefinitions OptionalInjection SkipDefinition OptionalInjection
                LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(21),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            70 => match aa_tag {
                // SkipDefinition: "%skip" REGEX
                LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(22),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION]),
            },
            71 => match aa_tag {
                // ProductionGroup: ProductionGroupHead ProductionTailList "."
                AAEnd | INJECT | IDENT => Action::Reduce(34),
                _ => Action::SyntaxError(vec![AAEnd, INJECT, IDENT]),
            },
            72 => match aa_tag {
                LITERAL => Action::Shift(61),
                ERROR => Action::Shift(62),
                IDENT => Action::Shift(60),
                PREDICATE => Action::Shift(58),
                ACTION => Action::Shift(57),
                // ProductionTail: <empty>
                VBAR | DOT => Action::Reduce(38),
                _ => Action::SyntaxError(vec![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION]),
            },
            73 => match aa_tag {
                // ProductionTail: Predicate Action
                VBAR | DOT => Action::Reduce(40),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            74 => match aa_tag {
                PRECEDENCE => Action::Shift(77),
                ACTION => Action::Shift(57),
                // ProductionTail: SymbolList Predicate
                VBAR | DOT => Action::Reduce(45),
                _ => Action::SyntaxError(vec![PRECEDENCE, VBAR, DOT, ACTION]),
            },
            75 => match aa_tag {
                ACTION => Action::Shift(57),
                // ProductionTail: SymbolList TaggedPrecedence
                VBAR | DOT => Action::Reduce(47),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            76 => match aa_tag {
                // ProductionTail: SymbolList Action
                VBAR | DOT => Action::Reduce(48),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            77 => match aa_tag {
                LITERAL => Action::Shift(91),
                IDENT => Action::Shift(90),
                _ => Action::SyntaxError(vec![LITERAL, IDENT]),
            },
            78 => match aa_tag {
                // SymbolList: SymbolList Symbol
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(55)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION,
                ]),
            },
            79 => match aa_tag {
                // PrecedenceDefinitions: PrecedenceDefinitions OptionalInjection PrecedenceDefinition OptionalInjection
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(24),
                _ => Action::SyntaxError(vec![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION]),
            },
            80 => match aa_tag {
                LITERAL => Action::Shift(82),
                IDENT => Action::Shift(83),
                // PrecedenceDefinition: "%left" TagList
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(25),
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            81 => match aa_tag {
                // TagList: Tag
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(28)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            82 => match aa_tag {
                // Tag: LITERAL
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(30)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            83 => match aa_tag {
                // Tag: IDENT
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(31)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            84 => match aa_tag {
                LITERAL => Action::Shift(82),
                IDENT => Action::Shift(83),
                // PrecedenceDefinition: "%right" TagList
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(26),
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            85 => match aa_tag {
                LITERAL => Action::Shift(82),
                IDENT => Action::Shift(83),
                // PrecedenceDefinition: "%nonassoc" TagList
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(27),
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            86 => match aa_tag {
                // ProductionTailList: ProductionTailList "|" ProductionTail
                VBAR | DOT => Action::Reduce(37),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            87 => match aa_tag {
                ACTION => Action::Shift(57),
                // ProductionTail: SymbolList Predicate TaggedPrecedence
                VBAR | DOT => Action::Reduce(43),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            88 => match aa_tag {
                // ProductionTail: SymbolList Predicate Action
                VBAR | DOT => Action::Reduce(44),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            89 => match aa_tag {
                // ProductionTail: SymbolList TaggedPrecedence Action
                VBAR | DOT => Action::Reduce(46),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            90 => match aa_tag {
                // TaggedPrecedence: "%prec" IDENT
                VBAR | DOT | ACTION => Action::Reduce(52),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            91 => match aa_tag {
                // TaggedPrecedence: "%prec" LITERAL
                VBAR | DOT | ACTION => Action::Reduce(53),
                _ => Action::SyntaxError(vec![VBAR, DOT, ACTION]),
            },
            92 => match aa_tag {
                // TagList: TagList Tag
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(29)
                }
                _ => Action::SyntaxError(vec![
                    LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT,
                ]),
            },
            93 => match aa_tag {
                // ProductionTail: SymbolList Predicate TaggedPrecedence Action
                VBAR | DOT => Action::Reduce(42),
                _ => Action::SyntaxError(vec![VBAR, DOT]),
            },
            _ => panic!("illegal state: {}", aa_state),
        };
    }

    fn production_data(production_id: u32) -> (AANonTerminal, usize) {
        match production_id {
            0 => (AANonTerminal::AAStart, 1),
            1 => (AANonTerminal::Specification, 6),
            2 => (AANonTerminal::OptionalInjection, 0),
            3 => (AANonTerminal::OptionalInjection, 1),
            4 => (AANonTerminal::InjectionHead, 2),
            5 => (AANonTerminal::Injection, 2),
            6 => (AANonTerminal::Preamble, 0),
            7 => (AANonTerminal::Preamble, 3),
            8 => (AANonTerminal::Configuration, 4),
            9 => (AANonTerminal::Configuration, 4),
            10 => (AANonTerminal::AttributeType, 2),
            11 => (AANonTerminal::TargetType, 2),
            12 => (AANonTerminal::Definitions, 3),
            13 => (AANonTerminal::TokenDefinitions, 2),
            14 => (AANonTerminal::TokenDefinitions, 4),
            15 => (AANonTerminal::TokenDefinition, 3),
            16 => (AANonTerminal::NewTokenName, 1),
            17 => (AANonTerminal::NewTokenName, 1),
            18 => (AANonTerminal::Pattern, 1),
            19 => (AANonTerminal::Pattern, 1),
            20 => (AANonTerminal::SkipDefinitions, 0),
            21 => (AANonTerminal::SkipDefinitions, 4),
            22 => (AANonTerminal::SkipDefinition, 2),
            23 => (AANonTerminal::PrecedenceDefinitions, 0),
            24 => (AANonTerminal::PrecedenceDefinitions, 4),
            25 => (AANonTerminal::PrecedenceDefinition, 2),
            26 => (AANonTerminal::PrecedenceDefinition, 2),
            27 => (AANonTerminal::PrecedenceDefinition, 2),
            28 => (AANonTerminal::TagList, 1),
            29 => (AANonTerminal::TagList, 2),
            30 => (AANonTerminal::Tag, 1),
            31 => (AANonTerminal::Tag, 1),
            32 => (AANonTerminal::ProductionRules, 3),
            33 => (AANonTerminal::ProductionRules, 3),
            34 => (AANonTerminal::ProductionGroup, 3),
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
            59 => (AANonTerminal::AASyntaxError, 0),
            60 => (AANonTerminal::AALexicalError, 0),
            61 => (AANonTerminal::AASemanticError, 0),
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
                AANonTerminal::Configuration => 7,
                AANonTerminal::AttributeType => 8,
                AANonTerminal::TargetType => 9,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            8 => match lhs {
                AANonTerminal::OptionalInjection => 16,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            9 => match lhs {
                AANonTerminal::OptionalInjection => 17,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            14 => match lhs {
                AANonTerminal::OptionalInjection => 20,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            15 => match lhs {
                AANonTerminal::Definitions => 21,
                AANonTerminal::OptionalInjection => 23,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                AANonTerminal::TokenDefinitions => 22,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            16 => match lhs {
                AANonTerminal::TargetType => 24,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            17 => match lhs {
                AANonTerminal::AttributeType => 25,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            22 => match lhs {
                AANonTerminal::OptionalInjection => 28,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                AANonTerminal::SkipDefinitions => 27,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            23 => match lhs {
                AANonTerminal::TokenDefinition => 29,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            24 => match lhs {
                AANonTerminal::OptionalInjection => 31,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            25 => match lhs {
                AANonTerminal::OptionalInjection => 32,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            26 => match lhs {
                AANonTerminal::ProductionRules => 33,
                AANonTerminal::OptionalInjection => 34,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            27 => match lhs {
                AANonTerminal::OptionalInjection => 36,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                AANonTerminal::PrecedenceDefinitions => 35,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            28 => match lhs {
                AANonTerminal::TokenDefinition => 37,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            30 => match lhs {
                AANonTerminal::NewTokenName => 38,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            33 => match lhs {
                AANonTerminal::ProductionGroup => 40,
                AANonTerminal::ProductionGroupHead => 41,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            34 => match lhs {
                AANonTerminal::ProductionGroup => 43,
                AANonTerminal::ProductionGroupHead => 41,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            35 => match lhs {
                AANonTerminal::OptionalInjection => 44,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            36 => match lhs {
                AANonTerminal::SkipDefinition => 45,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            37 => match lhs {
                AANonTerminal::OptionalInjection => 47,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            38 => match lhs {
                AANonTerminal::Pattern => 48,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            40 => match lhs {
                AANonTerminal::OptionalInjection => 51,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            41 => match lhs {
                AANonTerminal::ProductionTailList => 52,
                AANonTerminal::ProductionTail => 53,
                AANonTerminal::Action => 54,
                AANonTerminal::Predicate => 55,
                AANonTerminal::SymbolList => 56,
                AANonTerminal::Symbol => 59,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            43 => match lhs {
                AANonTerminal::OptionalInjection => 64,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            44 => match lhs {
                AANonTerminal::PrecedenceDefinition => 65,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            45 => match lhs {
                AANonTerminal::OptionalInjection => 69,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            55 => match lhs {
                AANonTerminal::Action => 73,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            56 => match lhs {
                AANonTerminal::Action => 76,
                AANonTerminal::Predicate => 74,
                AANonTerminal::TaggedPrecedence => 75,
                AANonTerminal::Symbol => 78,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            65 => match lhs {
                AANonTerminal::OptionalInjection => 79,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            66 => match lhs {
                AANonTerminal::TagList => 80,
                AANonTerminal::Tag => 81,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            67 => match lhs {
                AANonTerminal::TagList => 84,
                AANonTerminal::Tag => 81,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            68 => match lhs {
                AANonTerminal::TagList => 85,
                AANonTerminal::Tag => 81,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            72 => match lhs {
                AANonTerminal::ProductionTail => 86,
                AANonTerminal::Action => 54,
                AANonTerminal::Predicate => 55,
                AANonTerminal::SymbolList => 56,
                AANonTerminal::Symbol => 59,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            74 => match lhs {
                AANonTerminal::Action => 88,
                AANonTerminal::TaggedPrecedence => 87,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            75 => match lhs {
                AANonTerminal::Action => 89,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            80 => match lhs {
                AANonTerminal::Tag => 92,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            84 => match lhs {
                AANonTerminal::Tag => 92,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            85 => match lhs {
                AANonTerminal::Tag => 92,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            87 => match lhs {
                AANonTerminal::Action => 93,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
        };
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
            2 => {
                // OptionalInjection: <empty>
                // no injection so nothing to do
            }
            4 => {
                // InjectionHead: "%inject" LITERAL

                let (text, location) = aa_rhs[1].text_and_location();
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
                            aa_inject(text, file_path.to_string());
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

                let text = aa_rhs[1].matched_text();
                self.set_preamble(&text[2..text.len() - 2]);
            }
            10 => {
                // AttributeType: "%attr" IDENT

                self.attribute_type = aa_rhs[1].matched_text().to_string();
            }
            11 => {
                // TargetType: "%target" IDENT

                self.target_type = aa_rhs[1].matched_text().to_string();
            }
            15 => {
                // TokenDefinition: "%token" NewTokenName Pattern

                let (name, location) = aa_rhs[1].text_and_location();
                let pattern = aa_rhs[2].matched_text();
                if let Err(err) = self.symbol_table.new_token(name, pattern, location) {
                    self.error(location, &err.to_string());
                }
            }
            16 => {
                // NewTokenName: IDENT ?( !Self::is_allowable_name($1.matched_text()) ?)

                let (name, location) = aa_rhs[0].text_and_location();
                self.warning(
                    location,
                    &format!("token name \"{}\" may clash with generated code", name),
                );
            }
            20 => {
                // SkipDefinitions: <empty>

                // do nothing

            }
            22 => {
                // SkipDefinition: "%skip" REGEX

                let skip_rule = aa_rhs[1].matched_text();
                self.symbol_table.add_skip_rule(skip_rule);
            }
            23 => {
                // PrecedenceDefinitions: <empty>

                // do nothing

            }
            25 => {
                // PrecedenceDefinition: "%left" TagList

                let tag_list = aa_rhs[1].symbol_list();
                self.symbol_table
                    .set_precedences(Associativity::Left, tag_list);
            }
            26 => {
                // PrecedenceDefinition: "%right" TagList

                let tag_list = aa_rhs[1].symbol_list();
                self.symbol_table
                    .set_precedences(Associativity::Right, tag_list);
            }
            27 => {
                // PrecedenceDefinition: "%nonassoc" TagList

                let tag_list = aa_rhs[1].symbol_list();
                self.symbol_table
                    .set_precedences(Associativity::NonAssoc, tag_list);
            }
            28 => {
                // TagList: Tag

                let tag = aa_rhs[0].symbol();
                aa_lhs = AttributeData::SymbolList(vec![Rc::clone(&tag)]);
            }
            29 => {
                // TagList: TagList Tag

                let tag = aa_rhs[1].symbol();
                aa_lhs.symbol_list_mut().push(Rc::clone(tag));
            }
            30 => {
                // Tag: LITERAL

                let (text, location) = aa_rhs[0].text_and_location();
                if let Some(symbol) = self.symbol_table.get_literal_token(text, location) {
                    aa_lhs = AttributeData::Symbol(Rc::clone(symbol));
                } else {
                    let symbol = self
                        .symbol_table
                        .use_symbol_named(&AANonTerminal::AALexicalError.to_string(), location)
                        .unwrap();
                    aa_lhs = AttributeData::Symbol(symbol);
                    let msg = format!("Literal token \"{}\" is not known", text);
                    self.error(location, &msg);
                }
            }
            31 => {
                // Tag: IDENT

                let (name, location) = aa_rhs[0].text_and_location();
                if let Some(symbol) = self.symbol_table.use_symbol_named(name, location) {
                    if symbol.is_non_terminal() {
                        self.error(
                            location,
                            &format!(
                                "Non terminal \"{}\" cannot be used as precedence tag.",
                                name
                            ),
                        )
                    }
                    aa_lhs = AttributeData::Symbol(symbol);
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
            34 => {
                // ProductionGroup: ProductionGroupHead ProductionTailList "."

                let lhs = aa_rhs[0].left_hand_side();
                let tails = aa_rhs[1].production_tail_list();
                for tail in tails.iter() {
                    self.new_production(Rc::clone(&lhs), tail.clone());
                }
            }
            35 => {
                // ProductionGroupHead: IDENT ":"

                let (name, location) = aa_rhs[0].text_and_location();
                if let Some(symbol) = self.symbol_table.use_symbol_named(name, location) {
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
                    aa_lhs = AttributeData::LeftHandSide(symbol);
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
            36 => {
                // ProductionTailList: ProductionTail

                let production_tail = aa_rhs[0].production_tail().clone();
                aa_lhs = AttributeData::ProductionTailList(vec![production_tail]);
            }
            37 => {
                // ProductionTailList: ProductionTailList "|" ProductionTail

                let mut production_tail_list = aa_rhs[0].production_tail_list().clone();
                let production_tail = aa_rhs[2].production_tail().clone();
                production_tail_list.push(production_tail);
                aa_lhs = AttributeData::ProductionTailList(production_tail_list);
            }
            38 => {
                // ProductionTail: <empty>

                let tail = ProductionTail::new(vec![], None, None, None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            39 => {
                // ProductionTail: Action

                let action = aa_rhs[0].action().to_string();
                let tail = ProductionTail::new(vec![], None, None, Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            40 => {
                // ProductionTail: Predicate Action

                let predicate = aa_rhs[0].predicate().to_string();
                let action = aa_rhs[1].action().to_string();
                let tail = ProductionTail::new(vec![], Some(predicate), None, Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            41 => {
                // ProductionTail: Predicate

                let predicate = aa_rhs[0].predicate().to_string();
                let tail = ProductionTail::new(vec![], Some(predicate), None, None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            42 => {
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
            43 => {
                // ProductionTail: SymbolList Predicate TaggedPrecedence

                let rhs = aa_rhs[0].symbol_list().clone();
                let predicate = aa_rhs[1].predicate().to_string();
                let tagged_precedence = aa_rhs[2].associative_precedence().clone();
                let tail = ProductionTail::new(rhs, Some(predicate), Some(tagged_precedence), None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            44 => {
                // ProductionTail: SymbolList Predicate Action

                let rhs = aa_rhs[0].symbol_list().clone();
                let predicate = aa_rhs[1].predicate().to_string();
                let action = aa_rhs[2].action().to_string();
                let tail = ProductionTail::new(rhs, Some(predicate), None, Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            45 => {
                // ProductionTail: SymbolList Predicate

                let rhs = aa_rhs[0].symbol_list().clone();
                let predicate = aa_rhs[1].predicate().to_string();
                let tail = ProductionTail::new(rhs, Some(predicate), None, None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            46 => {
                // ProductionTail: SymbolList TaggedPrecedence Action

                let rhs = aa_rhs[0].symbol_list().clone();
                let tagged_precedence = aa_rhs[1].associative_precedence().clone();
                let action = aa_rhs[2].action().to_string();
                let tail = ProductionTail::new(rhs, None, Some(tagged_precedence), Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            47 => {
                // ProductionTail: SymbolList TaggedPrecedence

                let rhs = aa_rhs[0].symbol_list().clone();
                let tagged_precedence = aa_rhs[1].associative_precedence().clone();
                let tail = ProductionTail::new(rhs, None, Some(tagged_precedence), None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            48 => {
                // ProductionTail: SymbolList Action

                let rhs = aa_rhs[0].symbol_list().clone();
                let action = aa_rhs[1].action().to_string();
                let tail = ProductionTail::new(rhs, None, None, Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            49 => {
                // ProductionTail: SymbolList

                let rhs = aa_rhs[0].symbol_list().clone();
                let tail = ProductionTail::new(rhs, None, None, None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            50 => {
                // Action: ACTION

                let text = aa_rhs[0].matched_text();
                aa_lhs = AttributeData::Action(text[2..text.len() - 2].to_string());
            }
            51 => {
                // Predicate: PREDICATE

                let text = aa_rhs[0].matched_text();
                aa_lhs = AttributeData::Predicate(text[2..text.len() - 2].to_string());
            }
            52 => {
                // TaggedPrecedence: "%prec" IDENT

                let (name, location) = aa_rhs[1].text_and_location();
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
            53 => {
                // TaggedPrecedence: "%prec" LITERAL

                let (lexeme, location) = aa_rhs[1].text_and_location();
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
            54 => {
                // SymbolList: Symbol

                let symbol = aa_rhs[0].symbol();
                aa_lhs = AttributeData::SymbolList(vec![Rc::clone(&symbol)]);
            }
            55 => {
                // SymbolList: SymbolList Symbol

                let symbol = aa_rhs[1].symbol();
                aa_lhs.symbol_list_mut().push(Rc::clone(&symbol));
            }
            56 => {
                // Symbol: IDENT

                let (name, location) = aa_rhs[0].text_and_location();
                if let Some(symbol) = self.symbol_table.use_symbol_named(name, location) {
                    aa_lhs = AttributeData::Symbol(symbol);
                } else {
                    let symbol = self.symbol_table.use_new_non_terminal(name, location);
                    aa_lhs = AttributeData::Symbol(symbol);
                }
            }
            57 => {
                // Symbol: LITERAL

                let (lexeme, location) = aa_rhs[0].text_and_location();
                if let Some(symbol) = self.symbol_table.get_literal_token(lexeme, location) {
                    aa_lhs = AttributeData::Symbol(Rc::clone(symbol));
                } else {
                    self.error(location, &format!("{}: unknown literal)", lexeme));
                    let symbol = self
                        .symbol_table
                        .use_symbol_named(&AANonTerminal::AALexicalError.to_string(), location)
                        .unwrap();
                    aa_lhs = AttributeData::Symbol(symbol);
                }
            }
            58 => {
                // Symbol: "%error"

                let location = aa_rhs[0].location();
                let symbol = self
                    .symbol_table
                    .use_symbol_named(&AANonTerminal::AASyntaxError.to_string(), location)
                    .unwrap();
                aa_lhs = AttributeData::Symbol(symbol);
            }
            _ => aa_inject(String::new(), String::new()),
        };
        aa_lhs
    }
}
