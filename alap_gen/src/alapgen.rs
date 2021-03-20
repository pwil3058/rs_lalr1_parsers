use std::{fs::File, io::Read, rc::Rc};

use crate::{
    attributes::*,
    grammar::GrammarSpecification,
    state::ProductionTail,
    symbols::{AssociativePrecedence, Associativity, SymbolType},
};

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
    AAError,
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
            AANonTerminal::AAError => write!(f, r"AAError"),
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

impl lalr1_plus::Parser<AATerminal, AANonTerminal, AttributeData> for GrammarSpecification {
    fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<AATerminal> {
        &AALEXAN
    }

    fn viable_error_recovery_states(token: &AATerminal) -> BTreeSet<u32> {
        match token {
            _ => btree_set![],
        }
    }

    fn error_goto_state(state: u32) -> u32 {
        match state {
            _ => panic!("No error go to state for {}", state),
        }
    }

    fn look_ahead_set(state: u32) -> BTreeSet<AATerminal> {
        use AATerminal::*;
        return match state {
            0 => btree_set![ATTR, TARGET, INJECT, RUSTCODE],
            1 => btree_set![AAEnd],
            2 => btree_set![ATTR, TARGET],
            3 => btree_set![
                AAEnd, ATTR, TARGET, TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION, IDENT,
                RUSTCODE
            ],
            4 => btree_set![LITERAL],
            5 => btree_set![DOT],
            6 => btree_set![RUSTCODE],
            7 => btree_set![NEWSECTION],
            8 => btree_set![TARGET, INJECT],
            9 => btree_set![ATTR, INJECT],
            10 => btree_set![IDENT],
            11 => btree_set![IDENT],
            12 => btree_set![DOT],
            13 => btree_set![
                AAEnd, ATTR, TARGET, TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION, IDENT,
                RUSTCODE
            ],
            14 => btree_set![ATTR, TARGET, INJECT],
            15 => btree_set![TOKEN, INJECT],
            16 => btree_set![TARGET],
            17 => btree_set![ATTR],
            18 => btree_set![TARGET, INJECT, NEWSECTION],
            19 => btree_set![ATTR, INJECT, NEWSECTION],
            20 => btree_set![ATTR, TARGET],
            21 => btree_set![NEWSECTION],
            22 => btree_set![TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION],
            23 => btree_set![TOKEN],
            24 => btree_set![INJECT, NEWSECTION],
            25 => btree_set![INJECT, NEWSECTION],
            26 => btree_set![INJECT, IDENT],
            27 => btree_set![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION],
            28 => btree_set![TOKEN],
            29 => btree_set![TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION],
            30 => btree_set![IDENT],
            31 => btree_set![NEWSECTION],
            32 => btree_set![NEWSECTION],
            33 => btree_set![AAEnd, IDENT],
            34 => btree_set![IDENT],
            35 => btree_set![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION],
            36 => btree_set![SKIP],
            37 => btree_set![TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION],
            38 => btree_set![REGEX, LITERAL],
            39 => btree_set![REGEX, LITERAL],
            40 => btree_set![AAEnd, INJECT, IDENT],
            41 => btree_set![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION],
            42 => btree_set![COLON],
            43 => btree_set![AAEnd, INJECT, IDENT],
            44 => btree_set![LEFT, RIGHT, NONASSOC],
            45 => btree_set![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION],
            46 => btree_set![REGEX],
            47 => btree_set![TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION],
            48 => btree_set![TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION],
            49 => btree_set![TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION],
            50 => btree_set![AAEnd, IDENT],
            51 => btree_set![VBAR, DOT],
            52 => btree_set![VBAR, DOT],
            53 => btree_set![VBAR, DOT],
            54 => btree_set![VBAR, DOT, ACTION],
            55 => btree_set![LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION],
            56 => btree_set![VBAR, DOT],
            57 => btree_set![PRECEDENCE, VBAR, DOT, ACTION],
            58 => btree_set![LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION],
            59 => btree_set![LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION],
            60 => btree_set![LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION],
            61 => btree_set![LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION],
            62 => btree_set![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION],
            63 => btree_set![AAEnd, IDENT],
            64 => btree_set![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION],
            65 => btree_set![LITERAL, IDENT],
            66 => btree_set![LITERAL, IDENT],
            67 => btree_set![LITERAL, IDENT],
            68 => btree_set![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION],
            69 => btree_set![LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION],
            70 => btree_set![AAEnd, INJECT, IDENT],
            71 => btree_set![LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION],
            72 => btree_set![VBAR, DOT],
            73 => btree_set![PRECEDENCE, VBAR, DOT, ACTION],
            74 => btree_set![VBAR, DOT, ACTION],
            75 => btree_set![VBAR, DOT],
            76 => btree_set![LITERAL, IDENT],
            77 => btree_set![LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION],
            78 => btree_set![LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION],
            79 => btree_set![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT],
            80 => btree_set![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT],
            81 => btree_set![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT],
            82 => btree_set![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT],
            83 => btree_set![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT],
            84 => btree_set![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT],
            85 => btree_set![VBAR, DOT],
            86 => btree_set![VBAR, DOT, ACTION],
            87 => btree_set![VBAR, DOT],
            88 => btree_set![VBAR, DOT],
            89 => btree_set![VBAR, DOT, ACTION],
            90 => btree_set![VBAR, DOT, ACTION],
            91 => btree_set![LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT],
            92 => btree_set![VBAR, DOT],
            _ => panic!("illegal state: {}", state),
        };
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
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                RUSTCODE => Action::Reduce(2),
                // Preamble: <empty>
                ATTR | TARGET => Action::Reduce(6),
                _ => Action::SyntaxError,
            },
            1 => match aa_tag {
                // AAStart: Specification
                AAEnd => Action::Accept,
                _ => Action::SyntaxError,
            },
            2 => match aa_tag {
                ATTR => Action::Shift(10),
                TARGET => Action::Shift(11),
                _ => Action::SyntaxError,
            },
            3 => match aa_tag {
                // OptionalInjection: Injection
                AAEnd | ATTR | TARGET | TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT
                | NEWSECTION | IDENT | RUSTCODE => Action::Reduce(3),
                _ => Action::SyntaxError,
            },
            4 => match aa_tag {
                LITERAL => Action::Shift(12),
                _ => Action::SyntaxError,
            },
            5 => match aa_tag {
                DOT => Action::Shift(13),
                _ => Action::SyntaxError,
            },
            6 => match aa_tag {
                RUSTCODE => Action::Shift(14),
                _ => Action::SyntaxError,
            },
            7 => match aa_tag {
                NEWSECTION => Action::Shift(15),
                _ => Action::SyntaxError,
            },
            8 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                TARGET => Action::Reduce(2),
                _ => Action::SyntaxError,
            },
            9 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                ATTR => Action::Reduce(2),
                _ => Action::SyntaxError,
            },
            10 => match aa_tag {
                IDENT => Action::Shift(18),
                _ => Action::SyntaxError,
            },
            11 => match aa_tag {
                IDENT => Action::Shift(19),
                _ => Action::SyntaxError,
            },
            12 => match aa_tag {
                // InjectionHead: "%inject" LITERAL
                DOT => Action::Reduce(4),
                _ => Action::SyntaxError,
            },
            13 => match aa_tag {
                // Injection: InjectionHead "."
                AAEnd | ATTR | TARGET | TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT
                | NEWSECTION | IDENT | RUSTCODE => Action::Reduce(5),
                _ => Action::SyntaxError,
            },
            14 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                ATTR | TARGET => Action::Reduce(2),
                _ => Action::SyntaxError,
            },
            15 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                TOKEN => Action::Reduce(2),
                _ => Action::SyntaxError,
            },
            16 => match aa_tag {
                TARGET => Action::Shift(11),
                _ => Action::SyntaxError,
            },
            17 => match aa_tag {
                ATTR => Action::Shift(10),
                _ => Action::SyntaxError,
            },
            18 => match aa_tag {
                // AttributeType: "%attr" IDENT
                TARGET | INJECT | NEWSECTION => Action::Reduce(10),
                _ => Action::SyntaxError,
            },
            19 => match aa_tag {
                // TargetType: "%target" IDENT
                ATTR | INJECT | NEWSECTION => Action::Reduce(11),
                _ => Action::SyntaxError,
            },
            20 => match aa_tag {
                // Preamble: OptionalInjection RUSTCODE OptionalInjection
                ATTR | TARGET => Action::Reduce(7),
                _ => Action::SyntaxError,
            },
            21 => match aa_tag {
                NEWSECTION => Action::Shift(26),
                _ => Action::SyntaxError,
            },
            22 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                TOKEN => Action::Reduce(2),
                // SkipDefinitions: <empty>
                LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(19),
                _ => Action::SyntaxError,
            },
            23 => match aa_tag {
                TOKEN => Action::Shift(30),
                _ => Action::SyntaxError,
            },
            24 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError,
            },
            25 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError,
            },
            26 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                IDENT => Action::Reduce(2),
                _ => Action::SyntaxError,
            },
            27 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                SKIP => Action::Reduce(2),
                // PrecedenceDefinitions: <empty>
                LEFT | RIGHT | NONASSOC | NEWSECTION => Action::Reduce(22),
                _ => Action::SyntaxError,
            },
            28 => match aa_tag {
                TOKEN => Action::Shift(30),
                _ => Action::SyntaxError,
            },
            29 => match aa_tag {
                // TokenDefinitions: OptionalInjection TokenDefinition
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(13),
                _ => Action::SyntaxError,
            },
            30 => match aa_tag {
                IDENT => Action::Shift(39),
                _ => Action::SyntaxError,
            },
            31 => match aa_tag {
                // Configuration: AttributeType OptionalInjection TargetType OptionalInjection
                NEWSECTION => Action::Reduce(8),
                _ => Action::SyntaxError,
            },
            32 => match aa_tag {
                // Configuration: TargetType OptionalInjection AttributeType OptionalInjection
                NEWSECTION => Action::Reduce(9),
                _ => Action::SyntaxError,
            },
            33 => match aa_tag {
                IDENT => Action::Shift(42),
                // Specification: Preamble Configuration "%%" Definitions "%%" ProductionRules
                AAEnd => Action::Reduce(1),
                _ => Action::SyntaxError,
            },
            34 => match aa_tag {
                IDENT => Action::Shift(42),
                _ => Action::SyntaxError,
            },
            35 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                LEFT | RIGHT | NONASSOC => Action::Reduce(2),
                // Definitions: TokenDefinitions SkipDefinitions PrecedenceDefinitions
                NEWSECTION => Action::Reduce(12),
                _ => Action::SyntaxError,
            },
            36 => match aa_tag {
                SKIP => Action::Shift(46),
                _ => Action::SyntaxError,
            },
            37 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError,
            },
            38 => match aa_tag {
                REGEX => Action::Shift(49),
                LITERAL => Action::Shift(48),
                _ => Action::SyntaxError,
            },
            39 => match aa_tag {
                REGEX | LITERAL => {
                    if !Self::is_allowable_name(aa_attributes.at_len_minus_n(1).matched_text()) {
                        // NewTokenName: IDENT ?( !Self::is_allowable_name($1.matched_text()) ?)
                        Action::Reduce(17)
                    } else {
                        // NewTokenName: IDENT
                        Action::Reduce(18)
                    }
                }
                _ => Action::SyntaxError,
            },
            40 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                AAEnd | IDENT => Action::Reduce(2),
                _ => Action::SyntaxError,
            },
            41 => match aa_tag {
                LITERAL => Action::Shift(60),
                ERROR => Action::Shift(61),
                IDENT => Action::Shift(59),
                PREDICATE => Action::Shift(57),
                ACTION => Action::Shift(56),
                // ProductionTail: <empty>
                VBAR | DOT => Action::Reduce(37),
                _ => Action::SyntaxError,
            },
            42 => match aa_tag {
                COLON => Action::Shift(62),
                _ => Action::SyntaxError,
            },
            43 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                AAEnd | IDENT => Action::Reduce(2),
                _ => Action::SyntaxError,
            },
            44 => match aa_tag {
                LEFT => Action::Shift(65),
                RIGHT => Action::Shift(66),
                NONASSOC => Action::Shift(67),
                _ => Action::SyntaxError,
            },
            45 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError,
            },
            46 => match aa_tag {
                REGEX => Action::Shift(69),
                _ => Action::SyntaxError,
            },
            47 => match aa_tag {
                // TokenDefinitions: TokenDefinitions OptionalInjection TokenDefinition OptionalInjection
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(14),
                _ => Action::SyntaxError,
            },
            48 => match aa_tag {
                // TokenDefinition: "%token" NewTokenName LITERAL
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(15),
                _ => Action::SyntaxError,
            },
            49 => match aa_tag {
                // TokenDefinition: "%token" NewTokenName REGEX
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(16),
                _ => Action::SyntaxError,
            },
            50 => match aa_tag {
                // ProductionRules: ProductionRules ProductionGroup OptionalInjection
                AAEnd | IDENT => Action::Reduce(32),
                _ => Action::SyntaxError,
            },
            51 => match aa_tag {
                VBAR => Action::Shift(71),
                DOT => Action::Shift(70),
                _ => Action::SyntaxError,
            },
            52 => match aa_tag {
                // ProductionTailList: ProductionTail
                VBAR | DOT => Action::Reduce(35),
                _ => Action::SyntaxError,
            },
            53 => match aa_tag {
                // ProductionTail: Action
                VBAR | DOT => Action::Reduce(38),
                _ => Action::SyntaxError,
            },
            54 => match aa_tag {
                ACTION => Action::Shift(56),
                // ProductionTail: Predicate
                VBAR | DOT => Action::Reduce(40),
                _ => Action::SyntaxError,
            },
            55 => match aa_tag {
                LITERAL => Action::Shift(60),
                PRECEDENCE => Action::Shift(76),
                ERROR => Action::Shift(61),
                IDENT => Action::Shift(59),
                PREDICATE => Action::Shift(57),
                ACTION => Action::Shift(56),
                // ProductionTail: SymbolList
                VBAR | DOT => Action::Reduce(48),
                _ => Action::SyntaxError,
            },
            56 => match aa_tag {
                // Action: ACTION
                VBAR | DOT => Action::Reduce(49),
                _ => Action::SyntaxError,
            },
            57 => match aa_tag {
                // Predicate: PREDICATE
                PRECEDENCE | VBAR | DOT | ACTION => Action::Reduce(50),
                _ => Action::SyntaxError,
            },
            58 => match aa_tag {
                // SymbolList: Symbol
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(53)
                }
                _ => Action::SyntaxError,
            },
            59 => match aa_tag {
                // Symbol: IDENT
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(55)
                }
                _ => Action::SyntaxError,
            },
            60 => match aa_tag {
                // Symbol: LITERAL
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(56)
                }
                _ => Action::SyntaxError,
            },
            61 => match aa_tag {
                // Symbol: "%error"
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(57)
                }
                _ => Action::SyntaxError,
            },
            62 => match aa_tag {
                // ProductionGroupHead: IDENT ":"
                LITERAL | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => Action::Reduce(34),
                _ => Action::SyntaxError,
            },
            63 => match aa_tag {
                // ProductionRules: OptionalInjection ProductionGroup OptionalInjection
                AAEnd | IDENT => Action::Reduce(31),
                _ => Action::SyntaxError,
            },
            64 => match aa_tag {
                INJECT => Action::Shift(4),
                // OptionalInjection: <empty>
                LEFT | RIGHT | NONASSOC | NEWSECTION => Action::Reduce(2),
                _ => Action::SyntaxError,
            },
            65 => match aa_tag {
                LITERAL => Action::Shift(81),
                IDENT => Action::Shift(82),
                _ => Action::SyntaxError,
            },
            66 => match aa_tag {
                LITERAL => Action::Shift(81),
                IDENT => Action::Shift(82),
                _ => Action::SyntaxError,
            },
            67 => match aa_tag {
                LITERAL => Action::Shift(81),
                IDENT => Action::Shift(82),
                _ => Action::SyntaxError,
            },
            68 => match aa_tag {
                // SkipDefinitions: SkipDefinitions OptionalInjection SkipDefinition OptionalInjection
                LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(20),
                _ => Action::SyntaxError,
            },
            69 => match aa_tag {
                // SkipDefinition: "%skip" REGEX
                LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => Action::Reduce(21),
                _ => Action::SyntaxError,
            },
            70 => match aa_tag {
                // ProductionGroup: ProductionGroupHead ProductionTailList "."
                AAEnd | INJECT | IDENT => Action::Reduce(33),
                _ => Action::SyntaxError,
            },
            71 => match aa_tag {
                LITERAL => Action::Shift(60),
                ERROR => Action::Shift(61),
                IDENT => Action::Shift(59),
                PREDICATE => Action::Shift(57),
                ACTION => Action::Shift(56),
                // ProductionTail: <empty>
                VBAR | DOT => Action::Reduce(37),
                _ => Action::SyntaxError,
            },
            72 => match aa_tag {
                // ProductionTail: Predicate Action
                VBAR | DOT => Action::Reduce(39),
                _ => Action::SyntaxError,
            },
            73 => match aa_tag {
                PRECEDENCE => Action::Shift(76),
                ACTION => Action::Shift(56),
                // ProductionTail: SymbolList Predicate
                VBAR | DOT => Action::Reduce(44),
                _ => Action::SyntaxError,
            },
            74 => match aa_tag {
                ACTION => Action::Shift(56),
                // ProductionTail: SymbolList TaggedPrecedence
                VBAR | DOT => Action::Reduce(46),
                _ => Action::SyntaxError,
            },
            75 => match aa_tag {
                // ProductionTail: SymbolList Action
                VBAR | DOT => Action::Reduce(47),
                _ => Action::SyntaxError,
            },
            76 => match aa_tag {
                LITERAL => Action::Shift(90),
                IDENT => Action::Shift(89),
                _ => Action::SyntaxError,
            },
            77 => match aa_tag {
                // SymbolList: SymbolList Symbol
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    Action::Reduce(54)
                }
                _ => Action::SyntaxError,
            },
            78 => match aa_tag {
                // PrecedenceDefinitions: PrecedenceDefinitions OptionalInjection PrecedenceDefinition OptionalInjection
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(23),
                _ => Action::SyntaxError,
            },
            79 => match aa_tag {
                LITERAL => Action::Shift(81),
                IDENT => Action::Shift(82),
                // PrecedenceDefinition: "%left" TagList
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(24),
                _ => Action::SyntaxError,
            },
            80 => match aa_tag {
                // TagList: Tag
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(27)
                }
                _ => Action::SyntaxError,
            },
            81 => match aa_tag {
                // Tag: LITERAL
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(29)
                }
                _ => Action::SyntaxError,
            },
            82 => match aa_tag {
                // Tag: IDENT
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(30)
                }
                _ => Action::SyntaxError,
            },
            83 => match aa_tag {
                LITERAL => Action::Shift(81),
                IDENT => Action::Shift(82),
                // PrecedenceDefinition: "%right" TagList
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(25),
                _ => Action::SyntaxError,
            },
            84 => match aa_tag {
                LITERAL => Action::Shift(81),
                IDENT => Action::Shift(82),
                // PrecedenceDefinition: "%nonassoc" TagList
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => Action::Reduce(26),
                _ => Action::SyntaxError,
            },
            85 => match aa_tag {
                // ProductionTailList: ProductionTailList "|" ProductionTail
                VBAR | DOT => Action::Reduce(36),
                _ => Action::SyntaxError,
            },
            86 => match aa_tag {
                ACTION => Action::Shift(56),
                // ProductionTail: SymbolList Predicate TaggedPrecedence
                VBAR | DOT => Action::Reduce(42),
                _ => Action::SyntaxError,
            },
            87 => match aa_tag {
                // ProductionTail: SymbolList Predicate Action
                VBAR | DOT => Action::Reduce(43),
                _ => Action::SyntaxError,
            },
            88 => match aa_tag {
                // ProductionTail: SymbolList TaggedPrecedence Action
                VBAR | DOT => Action::Reduce(45),
                _ => Action::SyntaxError,
            },
            89 => match aa_tag {
                // TaggedPrecedence: "%prec" IDENT
                VBAR | DOT | ACTION => Action::Reduce(51),
                _ => Action::SyntaxError,
            },
            90 => match aa_tag {
                // TaggedPrecedence: "%prec" LITERAL
                VBAR | DOT | ACTION => Action::Reduce(52),
                _ => Action::SyntaxError,
            },
            91 => match aa_tag {
                // TagList: TagList Tag
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    Action::Reduce(28)
                }
                _ => Action::SyntaxError,
            },
            92 => match aa_tag {
                // ProductionTail: SymbolList Predicate TaggedPrecedence Action
                VBAR | DOT => Action::Reduce(41),
                _ => Action::SyntaxError,
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
            16 => (AANonTerminal::TokenDefinition, 3),
            17 => (AANonTerminal::NewTokenName, 1),
            18 => (AANonTerminal::NewTokenName, 1),
            19 => (AANonTerminal::SkipDefinitions, 0),
            20 => (AANonTerminal::SkipDefinitions, 4),
            21 => (AANonTerminal::SkipDefinition, 2),
            22 => (AANonTerminal::PrecedenceDefinitions, 0),
            23 => (AANonTerminal::PrecedenceDefinitions, 4),
            24 => (AANonTerminal::PrecedenceDefinition, 2),
            25 => (AANonTerminal::PrecedenceDefinition, 2),
            26 => (AANonTerminal::PrecedenceDefinition, 2),
            27 => (AANonTerminal::TagList, 1),
            28 => (AANonTerminal::TagList, 2),
            29 => (AANonTerminal::Tag, 1),
            30 => (AANonTerminal::Tag, 1),
            31 => (AANonTerminal::ProductionRules, 3),
            32 => (AANonTerminal::ProductionRules, 3),
            33 => (AANonTerminal::ProductionGroup, 3),
            34 => (AANonTerminal::ProductionGroupHead, 2),
            35 => (AANonTerminal::ProductionTailList, 1),
            36 => (AANonTerminal::ProductionTailList, 3),
            37 => (AANonTerminal::ProductionTail, 0),
            38 => (AANonTerminal::ProductionTail, 1),
            39 => (AANonTerminal::ProductionTail, 2),
            40 => (AANonTerminal::ProductionTail, 1),
            41 => (AANonTerminal::ProductionTail, 4),
            42 => (AANonTerminal::ProductionTail, 3),
            43 => (AANonTerminal::ProductionTail, 3),
            44 => (AANonTerminal::ProductionTail, 2),
            45 => (AANonTerminal::ProductionTail, 3),
            46 => (AANonTerminal::ProductionTail, 2),
            47 => (AANonTerminal::ProductionTail, 2),
            48 => (AANonTerminal::ProductionTail, 1),
            49 => (AANonTerminal::Action, 1),
            50 => (AANonTerminal::Predicate, 1),
            51 => (AANonTerminal::TaggedPrecedence, 2),
            52 => (AANonTerminal::TaggedPrecedence, 2),
            53 => (AANonTerminal::SymbolList, 1),
            54 => (AANonTerminal::SymbolList, 2),
            55 => (AANonTerminal::Symbol, 1),
            56 => (AANonTerminal::Symbol, 1),
            57 => (AANonTerminal::Symbol, 1),
            58 => (AANonTerminal::AAError, 0),
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
            40 => match lhs {
                AANonTerminal::OptionalInjection => 50,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            41 => match lhs {
                AANonTerminal::ProductionTailList => 51,
                AANonTerminal::ProductionTail => 52,
                AANonTerminal::Action => 53,
                AANonTerminal::Predicate => 54,
                AANonTerminal::SymbolList => 55,
                AANonTerminal::Symbol => 58,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            43 => match lhs {
                AANonTerminal::OptionalInjection => 63,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            44 => match lhs {
                AANonTerminal::PrecedenceDefinition => 64,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            45 => match lhs {
                AANonTerminal::OptionalInjection => 68,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            54 => match lhs {
                AANonTerminal::Action => 72,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            55 => match lhs {
                AANonTerminal::Action => 75,
                AANonTerminal::Predicate => 73,
                AANonTerminal::TaggedPrecedence => 74,
                AANonTerminal::Symbol => 77,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            64 => match lhs {
                AANonTerminal::OptionalInjection => 78,
                AANonTerminal::Injection => 3,
                AANonTerminal::InjectionHead => 5,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            65 => match lhs {
                AANonTerminal::TagList => 79,
                AANonTerminal::Tag => 80,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            66 => match lhs {
                AANonTerminal::TagList => 83,
                AANonTerminal::Tag => 80,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            67 => match lhs {
                AANonTerminal::TagList => 84,
                AANonTerminal::Tag => 80,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            71 => match lhs {
                AANonTerminal::ProductionTail => 85,
                AANonTerminal::Action => 53,
                AANonTerminal::Predicate => 54,
                AANonTerminal::SymbolList => 55,
                AANonTerminal::Symbol => 58,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            73 => match lhs {
                AANonTerminal::Action => 87,
                AANonTerminal::TaggedPrecedence => 86,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            74 => match lhs {
                AANonTerminal::Action => 88,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            79 => match lhs {
                AANonTerminal::Tag => 91,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            83 => match lhs {
                AANonTerminal::Tag => 91,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            84 => match lhs {
                AANonTerminal::Tag => 91,
                _ => panic!("Malformed goto table: ({}, {})", lhs, current_state),
            },
            86 => match lhs {
                AANonTerminal::Action => 92,
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
                // TokenDefinition: "%token" NewTokenName LITERAL

                let (name, location) = aa_rhs[1].text_and_location();
                let symbol_type = SymbolType::LiteralToken(aa_rhs[2].matched_text().to_string());
                if let Err(err) = self.symbol_table.new_token(name, symbol_type, location) {
                    self.error(location, &err.to_string());
                }
            }
            16 => {
                // TokenDefinition: "%token" NewTokenName REGEX

                let (name, location) = aa_rhs[1].text_and_location();
                let symbol_type = SymbolType::RegExToken(aa_rhs[2].matched_text().to_string());
                if let Err(err) = self.symbol_table.new_token(name, symbol_type, location) {
                    self.error(location, &err.to_string());
                }
            }
            17 => {
                // NewTokenName: IDENT ?( !Self::is_allowable_name($1.matched_text()) ?)

                let (name, location) = aa_rhs[0].text_and_location();
                self.warning(
                    location,
                    &format!("token name \"{}\" may clash with generated code", name),
                );
            }
            19 => {
                // SkipDefinitions: <empty>

                // do nothing
            }
            21 => {
                // SkipDefinition: "%skip" REGEX

                let skip_rule = aa_rhs[1].matched_text();
                self.symbol_table.add_skip_rule(skip_rule);
            }
            22 => {
                // PrecedenceDefinitions: <empty>

                // do nothing
            }
            24 => {
                // PrecedenceDefinition: "%left" TagList

                let tag_list = aa_rhs[1].symbol_list();
                self.symbol_table
                    .set_precedences(Associativity::Left, tag_list);
            }
            25 => {
                // PrecedenceDefinition: "%right" TagList

                let tag_list = aa_rhs[1].symbol_list();
                self.symbol_table
                    .set_precedences(Associativity::Right, tag_list);
            }
            26 => {
                // PrecedenceDefinition: "%nonassoc" TagList

                let tag_list = aa_rhs[1].symbol_list();
                self.symbol_table
                    .set_precedences(Associativity::NonAssoc, tag_list);
            }
            27 => {
                // TagList: Tag

                let tag = aa_rhs[0].symbol();
                aa_lhs = AttributeData::SymbolList(vec![Rc::clone(&tag)]);
            }
            28 => {
                // TagList: TagList Tag

                let tag = aa_rhs[1].symbol();
                aa_lhs.symbol_list_mut().push(Rc::clone(tag));
            }
            29 => {
                // Tag: LITERAL

                let (text, location) = aa_rhs[0].text_and_location();
                if let Some(symbol) = self.symbol_table.get_literal_token(text, location) {
                    aa_lhs = AttributeData::Symbol(Rc::clone(symbol));
                } else {
                    let symbol = self
                        .symbol_table
                        .use_symbol_named(&"AAInvalidTag".to_string(), location)
                        .unwrap();
                    aa_lhs = AttributeData::Symbol(symbol);
                    let msg = format!("Literal token \"{}\" is not known", text);
                    self.error(location, &msg);
                }
            }
            30 => {
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
            33 => {
                // ProductionGroup: ProductionGroupHead ProductionTailList "."

                let lhs = aa_rhs[0].left_hand_side();
                let tails = aa_rhs[1].production_tail_list();
                for tail in tails.iter() {
                    self.new_production(Rc::clone(&lhs), tail.clone());
                }
            }
            34 => {
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
            35 => {
                // ProductionTailList: ProductionTail

                let production_tail = aa_rhs[0].production_tail().clone();
                aa_lhs = AttributeData::ProductionTailList(vec![production_tail]);
            }
            36 => {
                // ProductionTailList: ProductionTailList "|" ProductionTail

                let mut production_tail_list = aa_rhs[0].production_tail_list().clone();
                let production_tail = aa_rhs[2].production_tail().clone();
                production_tail_list.push(production_tail);
                aa_lhs = AttributeData::ProductionTailList(production_tail_list);
            }
            37 => {
                // ProductionTail: <empty>

                let tail = ProductionTail::new(vec![], None, None, None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            38 => {
                // ProductionTail: Action

                let action = aa_rhs[0].action().to_string();
                let tail = ProductionTail::new(vec![], None, None, Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            39 => {
                // ProductionTail: Predicate Action

                let predicate = aa_rhs[0].predicate().to_string();
                let action = aa_rhs[1].action().to_string();
                let tail = ProductionTail::new(vec![], Some(predicate), None, Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            40 => {
                // ProductionTail: Predicate

                let predicate = aa_rhs[0].predicate().to_string();
                let tail = ProductionTail::new(vec![], Some(predicate), None, None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            41 => {
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
            42 => {
                // ProductionTail: SymbolList Predicate TaggedPrecedence

                let rhs = aa_rhs[0].symbol_list().clone();
                let predicate = aa_rhs[1].predicate().to_string();
                let tagged_precedence = aa_rhs[2].associative_precedence().clone();
                let tail = ProductionTail::new(rhs, Some(predicate), Some(tagged_precedence), None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            43 => {
                // ProductionTail: SymbolList Predicate Action

                let rhs = aa_rhs[0].symbol_list().clone();
                let predicate = aa_rhs[1].predicate().to_string();
                let action = aa_rhs[2].action().to_string();
                let tail = ProductionTail::new(rhs, Some(predicate), None, Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            44 => {
                // ProductionTail: SymbolList Predicate

                let rhs = aa_rhs[0].symbol_list().clone();
                let predicate = aa_rhs[1].predicate().to_string();
                let tail = ProductionTail::new(rhs, Some(predicate), None, None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            45 => {
                // ProductionTail: SymbolList TaggedPrecedence Action

                let rhs = aa_rhs[0].symbol_list().clone();
                let tagged_precedence = aa_rhs[1].associative_precedence().clone();
                let action = aa_rhs[2].action().to_string();
                let tail = ProductionTail::new(rhs, None, Some(tagged_precedence), Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            46 => {
                // ProductionTail: SymbolList TaggedPrecedence

                let rhs = aa_rhs[0].symbol_list().clone();
                let tagged_precedence = aa_rhs[1].associative_precedence().clone();
                let tail = ProductionTail::new(rhs, None, Some(tagged_precedence), None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            47 => {
                // ProductionTail: SymbolList Action

                let rhs = aa_rhs[0].symbol_list().clone();
                let action = aa_rhs[1].action().to_string();
                let tail = ProductionTail::new(rhs, None, None, Some(action));
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            48 => {
                // ProductionTail: SymbolList

                let rhs = aa_rhs[0].symbol_list().clone();
                let tail = ProductionTail::new(rhs, None, None, None);
                aa_lhs = AttributeData::ProductionTail(tail)
            }
            49 => {
                // Action: ACTION

                let text = aa_rhs[0].matched_text();
                aa_lhs = AttributeData::Action(text[2..text.len() - 2].to_string());
            }
            50 => {
                // Predicate: PREDICATE

                let text = aa_rhs[0].matched_text();
                aa_lhs = AttributeData::Predicate(text[2..text.len() - 2].to_string());
            }
            51 => {
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
            52 => {
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
            53 => {
                // SymbolList: Symbol

                let symbol = aa_rhs[0].symbol();
                aa_lhs = AttributeData::SymbolList(vec![Rc::clone(&symbol)]);
            }
            54 => {
                // SymbolList: SymbolList Symbol

                let symbol = aa_rhs[1].symbol();
                aa_lhs.symbol_list_mut().push(Rc::clone(&symbol));
            }
            55 => {
                // Symbol: IDENT

                let (name, location) = aa_rhs[0].text_and_location();
                if let Some(symbol) = self.symbol_table.use_symbol_named(name, location) {
                    aa_lhs = AttributeData::Symbol(symbol);
                } else {
                    let symbol = self.symbol_table.use_new_non_terminal(name, location);
                    aa_lhs = AttributeData::Symbol(symbol);
                }
            }
            56 => {
                // Symbol: LITERAL

                let (lexeme, location) = aa_rhs[0].text_and_location();
                if let Some(symbol) = self.symbol_table.get_literal_token(lexeme, location) {
                    aa_lhs = AttributeData::Symbol(Rc::clone(symbol));
                } else {
                    self.error(location, &format!("{}: unknown literal)", lexeme));
                    let symbol = self
                        .symbol_table
                        .use_symbol_named(&AANonTerminal::AAError.to_string(), location)
                        .unwrap();
                    aa_lhs = AttributeData::Symbol(symbol);
                }
            }
            57 => {
                // Symbol: "%error"

                let location = aa_rhs[0].location();
                let symbol = self
                    .symbol_table
                    .use_symbol_named(&AANonTerminal::AAError.to_string(), location)
                    .unwrap();
                aa_lhs = AttributeData::Symbol(symbol);
            }
            _ => aa_inject(String::new(), String::new()),
        };
        aa_lhs
    }
}
