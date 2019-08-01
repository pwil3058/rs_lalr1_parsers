use std::fmt;

use lalr1plus;
use lexan;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum AATerminal {
    REGEX,
    LITERAL,
    TOKEN,
    FIELD,
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
    FIELDNAME,
    PREDICATE,
    ACTION,
    DCODE,
}

impl fmt::Display for AATerminal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AATerminal::*;
        match self {
            REGEX => write!(f, "REGEX"),
            LITERAL => write!(f, "LITERAL"),
            TOKEN => write!(f, "%token"),
            FIELD => write!(f, "%field"),
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
            FIELDNAME => write!(f, "FIELDNAME"),
            PREDICATE => write!(f, "PREDICATE"),
            ACTION => write!(f, "ACTION"),
            DCODE => write!(f, "DCODE"),
        }
    }
}

lazy_static! {
    static ref AALEXAN: lexan::LexicalAnalyzer<AATerminal> = {
        use AATerminal::*;
        lexan::LexicalAnalyzer::new(
            &[
                (TOKEN, "%token"),
                (FIELD, "%field"),
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
                (FIELDNAME, r###"(<[a-zA-Z]+[a-zA-Z0-9_]*>)"###),
                (PREDICATE, r###"(\?\((.|[\n\r])*?\?\))"###),
                (ACTION, r###"(!\{(.|[\n\r])*?!\})"###),
                (DCODE, r###"(%\{(.|[\n\r])*?%\})"###),
            ],
            &[
                r###"(/\*(.|[\n\r])*?\*/)"###,
                r###"(//[^\n\r]*)"###,
                r###"(\s+)"###,
            ],
        )
    };
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum AANonTerminal {
    Specification,
    Preamble,
    Definitions,
    ProductionRules,
    Coda,
    OInjection,
    Injection,
    InjectionHead,
    FieldDefinitions,
    TokenDefinitions,
    SkipDefinitions,
    PrecedenceDefinitions,
    FieldDefinition,
    FieldType,
    FieldName,
    FieldConversionFunction,
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
            Coda => write!(f, "Coda"),
            OInjection => write!(f, "OInjection"),
            Injection => write!(f, "Injection"),
            InjectionHead => write!(f, "InjectionHead"),
            FieldDefinitions => write!(f, "FieldDefinitions"),
            TokenDefinitions => write!(f, "TokenDefinitions"),
            SkipDefinitions => write!(f, "SkipDefinitions"),
            PrecedenceDefinitions => write!(f, "PrecedenceDefinitions"),
            FieldDefinition => write!(f, "FieldDefinition"),
            FieldType => write!(f, "FieldType"),
            FieldName => write!(f, "FieldName"),
            FieldConversionFunction => write!(f, "FieldConversionFunction"),
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

#[derive(Debug, Default, Clone)]
pub struct AAAttributeData {}

impl AAAttributeData {
    fn matched_text<'a>(&'a self) -> &'a str {
        "what"
    }
}

impl From<(AATerminal, String)> for AAAttributeData {
    fn from(_input: (AATerminal, String)) -> Self {
        AAAttributeData::default()
    }
}

impl From<lalr1plus::Error<AATerminal>> for AAAttributeData {
    fn from(_error: lalr1plus::Error<AATerminal>) -> Self {
        AAAttributeData::default()
    }
}

#[derive(Debug, Default, Clone)]
pub struct SymbolTable {}

impl SymbolTable {
    fn is_known_non_terminal(&self, _name: &str) -> bool {
        false
    }

    fn is_known_tag(&self, _name: &str) -> bool {
        false
    }

    fn is_known_token(&self, _name: &str) -> bool {
        false
    }
}

#[derive(Debug, Default, Clone)]
pub struct ParserSpecification {
    symbol_table: SymbolTable,
}

impl ParserSpecification {
    fn is_allowable_name(_name: &str) -> bool {
        false
    }
}

macro_rules! aa_syntax_error {
    ( $token:expr; $( $tag:expr),* ) => ({
        lalr1plus::Action::SyntaxError(
            *$token.tag(),
            vec![ $( $tag),* ],
            $token.location().to_string(),
        )
    });
}

impl lalr1plus::Parser<AATerminal, AANonTerminal, AAAttributeData> for ParserSpecification {
    fn lexical_analyzer(&self) -> &lexan::LexicalAnalyzer<AATerminal> {
        &AALEXAN
    }

    fn viable_error_recovery_states(_tag: &AATerminal) -> Vec<u32> {
        vec![]
    }

    fn error_go_state(state: u32) -> u32 {
        state
    }

    fn next_action<'a>(
        &self,
        state: u32,
        aa_attributes: &lalr1plus::ParseStack<AATerminal, AANonTerminal, AAAttributeData>,
        token: &lexan::Token<'a, AATerminal>,
    ) -> lalr1plus::Action<AATerminal> {
        use AATerminal::*;
        let tag = *token.tag();
        match state {
            0 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                TOKEN | FIELD => lalr1plus::Action::Reduce(6), // preamble: <empty>
                DCODE => lalr1plus::Action::Reduce(2),         // oinjection: <empty>
                _ => aa_syntax_error!(token; TOKEN, FIELD, INJECT, DCODE),
            },
            1 => aa_syntax_error!(token; ),
            2 => match tag {
                TOKEN | FIELD | INJECT => lalr1plus::Action::Reduce(12), // field_definitions: <empty>
                _ => aa_syntax_error!(token; TOKEN, FIELD, INJECT),
            },
            3 => match tag {
                TOKEN | FIELD | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION | IDENT
                | DCODE => lalr1plus::Action::Reduce(3), // oinjection: injection
                _ => aa_syntax_error!(token; TOKEN, FIELD, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION, IDENT, DCODE)
            },
            4 => match tag {
                LITERAL => lalr1plus::Action::Shift(9),
                _ => aa_syntax_error!(token; LITERAL),
            },
            5 => match tag {
                DOT => lalr1plus::Action::Shift(10),
                _ => aa_syntax_error!(token; DOT),
            },
            6 => match tag {
                DCODE => lalr1plus::Action::Shift(11),
                _ => aa_syntax_error!(token; DCODE),
            },
            7 => match tag {
                NEWSECTION => lalr1plus::Action::Shift(12),
                _ => aa_syntax_error!(token; NEWSECTION),
            },
            8 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                TOKEN | FIELD => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                _ => aa_syntax_error!(token; TOKEN, FIELD, INJECT),
            },
            9 => match tag {
                DOT => lalr1plus::Action::Reduce(4), // injection_head: "%inject" LITERAL
                _ => aa_syntax_error!(token; DOT),
            },
            10 => match tag {
                TOKEN | FIELD | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION | IDENT
                | DCODE => lalr1plus::Action::Reduce(5), // injection: injection_head "."
                _ => aa_syntax_error!(token; TOKEN, FIELD, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION, IDENT, DCODE)
            },
            11 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                TOKEN | FIELD | DCODE => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                _ => aa_syntax_error!(token; TOKEN, FIELD, INJECT, DCODE),
            },
            12 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                IDENT => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                _ => aa_syntax_error!(token; INJECT, IDENT),
            },
            13 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                TOKEN => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => lalr1plus::Action::Reduce(30), // skip_definitions: <empty>
                _ => aa_syntax_error!(token; TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION)
            },
            14 => match tag {
                TOKEN => lalr1plus::Action::Shift(23),
                FIELD => lalr1plus::Action::Shift(21),
                _ => aa_syntax_error!(token; TOKEN, FIELD),
            },
            15 => match tag {
                DCODE => lalr1plus::Action::Shift(24),
                TOKEN | FIELD | INJECT => lalr1plus::Action::Reduce(7), // preamble: oinjection DCODE oinjection
                _ => aa_syntax_error!(token; TOKEN, FIELD, INJECT, DCODE),
            },
            16 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                IDENT => lalr1plus::Action::Shift(29),
                DCODE => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                _ => aa_syntax_error!(token; INJECT, IDENT, DCODE),
            },
            17 => match tag {
                IDENT => lalr1plus::Action::Shift(29),
                _ => aa_syntax_error!(token; IDENT),
            },
            18 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                LEFT | RIGHT | NONASSOC | NEWSECTION => lalr1plus::Action::Reduce(33), // precedence_definitions: <empty>
                SKIP => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                _ => aa_syntax_error!(token; LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION),
            },
            19 => match tag {
                TOKEN => lalr1plus::Action::Shift(23),
                _ => aa_syntax_error!(token; TOKEN),
            },
            20 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                TOKEN | FIELD => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                _ => aa_syntax_error!(token; TOKEN, FIELD, INJECT),
            },
            21 => match tag {
                IDENT => lalr1plus::Action::Shift(36),
                _ => aa_syntax_error!(token; IDENT),
            },
            22 => match tag {
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION =>
                    lalr1plus::Action::Reduce(22),
                // token_definitions: oinjection token_definition
                _ => aa_syntax_error!(token; TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION)
            },
            23 => match tag {
                IDENT => lalr1plus::Action::Shift(39),
                FIELDNAME => lalr1plus::Action::Shift(38),
                _ => aa_syntax_error!(token; IDENT, FIELDNAME),
            },
            24 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                TOKEN | FIELD => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                _ => aa_syntax_error!(token; TOKEN, FIELD, INJECT),
            },
            25 => aa_syntax_error!(token; ),
            26 => match tag {
                DCODE => lalr1plus::Action::Shift(41),
                _ => aa_syntax_error!(token; DCODE),
            },
            27 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                IDENT | DCODE => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                _ => aa_syntax_error!(token; INJECT, IDENT, DCODE),
            },
            28 => match tag {
                LITERAL => lalr1plus::Action::Shift(52),
                ERROR => lalr1plus::Action::Shift(53),
                IDENT => lalr1plus::Action::Shift(51),
                PREDICATE => lalr1plus::Action::Shift(49),
                ACTION => lalr1plus::Action::Shift(48),
                VBAR | DOT => lalr1plus::Action::Reduce(52), // production_tail: <empty>
                _ => aa_syntax_error!(token; LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION),
            },
            29 => match tag {
                COLON => lalr1plus::Action::Shift(54),
                _ => aa_syntax_error!(token; COLON),
            },
            30 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                IDENT | DCODE => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                _ => aa_syntax_error!(token; INJECT, IDENT, DCODE),
            },
            31 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                LEFT | RIGHT | NONASSOC => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                NEWSECTION => lalr1plus::Action::Reduce(11), // definitions: field_definitions token_definitions skip_definitions precedence_definitions
                _ => aa_syntax_error!(token; LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION),
            },
            32 => match tag {
                SKIP => lalr1plus::Action::Shift(58),
                _ => aa_syntax_error!(token; SKIP),
            },
            33 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                _ => aa_syntax_error!(token; TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION)
            },
            34 => match tag {
                TOKEN | FIELD | INJECT => lalr1plus::Action::Reduce(13), // field_definitions: field_definitions oinjection field_definition oinjection
                _ => aa_syntax_error!(token; TOKEN, FIELD, INJECT),
            },
            35 => match tag {
                IDENT => lalr1plus::Action::Shift(61),
                _ => aa_syntax_error!(token; IDENT),
            },
            36 => match tag {
                IDENT => {
                    if !Self::is_allowable_name(
                        aa_attributes.attribute_n_from_end(2 - 1).matched_text(),
                    ) {
                        lalr1plus::Action::Reduce(16) // field_type: IDENT ?(  !is_allowable_name($1.matched_text())  ?)
                    } else {
                        lalr1plus::Action::Reduce(17) // field_type: IDENT
                    }
                }
                _ => aa_syntax_error!(token; IDENT),
            },
            37 => match tag {
                REGEX => lalr1plus::Action::Shift(63),
                LITERAL => lalr1plus::Action::Shift(64),
                _ => aa_syntax_error!(token; REGEX, LITERAL),
            },
            38 => match tag {
                IDENT => lalr1plus::Action::Shift(39),
                _ => aa_syntax_error!(token; IDENT),
            },
            39 => match tag {
                REGEX | LITERAL => {
                    if !Self::is_allowable_name(
                        aa_attributes.attribute_n_from_end(2 - 1).matched_text(),
                    ) {
                        lalr1plus::Action::Reduce(26) // new_token_name: IDENT ?(  !is_allowable_name($1.matched_text())  ?)
                    } else {
                        lalr1plus::Action::Reduce(27) // new_token_name: IDENT
                    }
                }
                _ => aa_syntax_error!(token; REGEX, LITERAL),
            },
            40 => match tag {
                TOKEN | FIELD | INJECT => lalr1plus::Action::Reduce(8), // preamble: oinjection DCODE oinjection DCODE oinjection
                _ => aa_syntax_error!(token; TOKEN, FIELD, INJECT),
            },
            41 => aa_syntax_error!(token; ),
            42 => match tag {
                INJECT | IDENT | DCODE => lalr1plus::Action::Reduce(45), // production_rules: production_rules production_group oinjection
                _ => aa_syntax_error!(token; INJECT, IDENT, DCODE),
            },
            43 => match tag {
                VBAR => lalr1plus::Action::Shift(67),
                DOT => lalr1plus::Action::Shift(66),
                _ => aa_syntax_error!(token; VBAR, DOT),
            },
            44 => match tag {
                VBAR | DOT => lalr1plus::Action::Reduce(50), // production_tail_list: production_tail
                _ => aa_syntax_error!(token; VBAR, DOT),
            },
            45 => match tag {
                VBAR | DOT => lalr1plus::Action::Reduce(53), // production_tail: action
                _ => aa_syntax_error!(token; VBAR, DOT),
            },
            46 => match tag {
                ACTION => lalr1plus::Action::Shift(48),
                VBAR | DOT => lalr1plus::Action::Reduce(55), // production_tail: predicate
                _ => aa_syntax_error!(token; VBAR, DOT, ACTION),
            },
            47 => match tag {
                LITERAL => lalr1plus::Action::Shift(52),
                PRECEDENCE => lalr1plus::Action::Shift(72),
                ERROR => lalr1plus::Action::Shift(53),
                IDENT => lalr1plus::Action::Shift(51),
                PREDICATE => lalr1plus::Action::Shift(49),
                ACTION => lalr1plus::Action::Shift(48),
                VBAR | DOT => lalr1plus::Action::Reduce(63), // production_tail: symbol_list
                _ => aa_syntax_error!(token; LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION)
            },
            48 => match tag {
                VBAR | DOT => lalr1plus::Action::Reduce(64), // action: ACTION
                _ => aa_syntax_error!(token; VBAR, DOT),
            },
            49 => match tag {
                PRECEDENCE | VBAR | DOT | ACTION => lalr1plus::Action::Reduce(65), // predicate: PREDICATE
                _ => aa_syntax_error!(token; PRECEDENCE, VBAR, DOT, ACTION),
            },
            50 => match tag {
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    lalr1plus::Action::Reduce(68)
                } // symbol_list: symbol
                _ => aa_syntax_error!(token; LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION)
            },
            51 => match tag {
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    lalr1plus::Action::Reduce(70)
                } // symbol: IDENT
                _ => aa_syntax_error!(token; LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION)
            },
            52 => match tag {
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    lalr1plus::Action::Reduce(71)
                } // symbol: LITERAL
                _ => aa_syntax_error!(token; LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION)
            },
            53 => match tag {
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    lalr1plus::Action::Reduce(72)
                } // symbol: "%error"
                _ => aa_syntax_error!(token; LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION)
            },
            54 => match tag {
                LITERAL | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    if self
                        .symbol_table
                        .is_known_token(aa_attributes.attribute_n_from_end(3 - 1).matched_text())
                    {
                        lalr1plus::Action::Reduce(47) // production_group_head: IDENT ":" ?(  self.symbol_table.is_known_token($1.matched_text())  ?)
                    } else if self
                        .symbol_table
                        .is_known_tag(aa_attributes.attribute_n_from_end(3 - 1).matched_text())
                    {
                        lalr1plus::Action::Reduce(48) // production_group_head: IDENT ":" ?(  self.symbol_table.is_known_tag($1.matched_text())  ?)
                    } else {
                        lalr1plus::Action::Reduce(49) // production_group_head: IDENT ":"
                    }
                }
                _ => aa_syntax_error!(token; LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION),
            },
            55 => match tag {
                INJECT | IDENT | DCODE => lalr1plus::Action::Reduce(44), // production_rules: oinjection production_group oinjection
                _ => aa_syntax_error!(token; INJECT, IDENT, DCODE),
            },
            56 => match tag {
                LEFT => lalr1plus::Action::Shift(75),
                RIGHT => lalr1plus::Action::Shift(76),
                NONASSOC => lalr1plus::Action::Shift(77),
                _ => aa_syntax_error!(token; LEFT, RIGHT, NONASSOC),
            },
            57 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                LEFT | RIGHT | NONASSOC | SKIP | NEWSECTION => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                _ => aa_syntax_error!(token; LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION),
            },
            58 => match tag {
                REGEX => lalr1plus::Action::Shift(79),
                _ => aa_syntax_error!(token; REGEX),
            },
            59 => match tag {
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => {
                    lalr1plus::Action::Reduce(23)
                } // token_definitions: token_definitions oinjection token_definition oinjection
                _ => aa_syntax_error!(token; TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION)
            },
            60 => match tag {
                IDENT => lalr1plus::Action::Shift(81),
                TOKEN | FIELD | INJECT => lalr1plus::Action::Reduce(14), // field_definition: "%field" field_type field_name
                _ => aa_syntax_error!(token; TOKEN, FIELD, INJECT, IDENT),
            },
            61 => match tag {
                TOKEN | FIELD | INJECT | IDENT => {
                    if !Self::is_allowable_name(
                        aa_attributes.attribute_n_from_end(2 - 1).matched_text(),
                    ) {
                        lalr1plus::Action::Reduce(18) // field_name: IDENT ?(  !is_allowable_name($1.matched_text())  ?)
                    } else {
                        lalr1plus::Action::Reduce(19) // field_name: IDENT
                    }
                }
                _ => aa_syntax_error!(token; TOKEN, FIELD, INJECT, IDENT),
            },
            62 => match tag {
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => {
                    lalr1plus::Action::Reduce(24)
                } // token_definition: "%token" new_token_name pattern
                _ => aa_syntax_error!(token; TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION)
            },
            63 => match tag {
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => {
                    lalr1plus::Action::Reduce(28)
                } // pattern: REGEX
                _ => aa_syntax_error!(token; TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION)
            },
            64 => match tag {
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => {
                    lalr1plus::Action::Reduce(29)
                } // pattern: LITERAL
                _ => {
                    aa_syntax_error!(token; TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION)
                }
            },
            65 => match tag {
                REGEX => lalr1plus::Action::Shift(63),
                LITERAL => lalr1plus::Action::Shift(64),
                _ => aa_syntax_error!(token; REGEX, LITERAL),
            },
            66 => match tag {
                INJECT | IDENT | DCODE => lalr1plus::Action::Reduce(46), // production_group: production_group_head production_tail_list "."
                _ => aa_syntax_error!(token; INJECT, IDENT, DCODE),
            },
            67 => match tag {
                LITERAL => lalr1plus::Action::Shift(52),
                ERROR => lalr1plus::Action::Shift(53),
                IDENT => lalr1plus::Action::Shift(51),
                PREDICATE => lalr1plus::Action::Shift(49),
                ACTION => lalr1plus::Action::Shift(48),
                VBAR | DOT => lalr1plus::Action::Reduce(52), // production_tail: <empty>
                _ => aa_syntax_error!(token; LITERAL, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION),
            },
            68 => match tag {
                VBAR | DOT => lalr1plus::Action::Reduce(54), // production_tail: predicate action
                _ => aa_syntax_error!(token; VBAR, DOT),
            },
            69 => match tag {
                PRECEDENCE => lalr1plus::Action::Shift(72),
                ACTION => lalr1plus::Action::Shift(48),
                VBAR | DOT => lalr1plus::Action::Reduce(59), // production_tail: symbol_list predicate
                _ => aa_syntax_error!(token; PRECEDENCE, VBAR, DOT, ACTION),
            },
            70 => match tag {
                ACTION => lalr1plus::Action::Shift(48),
                VBAR | DOT => lalr1plus::Action::Reduce(61), // production_tail: symbol_list tagged_precedence
                _ => aa_syntax_error!(token; VBAR, DOT, ACTION),
            },
            71 => match tag {
                VBAR | DOT => lalr1plus::Action::Reduce(62), // production_tail: symbol_list action
                _ => aa_syntax_error!(token; VBAR, DOT),
            },
            72 => match tag {
                LITERAL => lalr1plus::Action::Shift(88),
                IDENT => lalr1plus::Action::Shift(87),
                _ => aa_syntax_error!(token; LITERAL, IDENT),
            },
            73 => match tag {
                LITERAL | PRECEDENCE | ERROR | VBAR | DOT | IDENT | PREDICATE | ACTION => {
                    lalr1plus::Action::Reduce(69)
                } // symbol_list: symbol_list symbol
                _ => aa_syntax_error!(token; LITERAL, PRECEDENCE, ERROR, VBAR, DOT, IDENT, PREDICATE, ACTION)
            },
            74 => match tag {
                INJECT => lalr1plus::Action::Shift(4),
                LEFT | RIGHT | NONASSOC | NEWSECTION => lalr1plus::Action::Reduce(2), // oinjection: <empty>
                _ => aa_syntax_error!(token; LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION),
            },
            75 => match tag {
                LITERAL => lalr1plus::Action::Shift(92),
                IDENT => lalr1plus::Action::Shift(93),
                _ => aa_syntax_error!(token; LITERAL, IDENT),
            },
            76 => match tag {
                LITERAL => lalr1plus::Action::Shift(92),
                IDENT => lalr1plus::Action::Shift(93),
                _ => aa_syntax_error!(token; LITERAL, IDENT),
            },
            77 => match tag {
                LITERAL => lalr1plus::Action::Shift(92),
                IDENT => lalr1plus::Action::Shift(93),
                _ => aa_syntax_error!(token; LITERAL, IDENT),
            },
            78 => match tag {
                LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => {
                    lalr1plus::Action::Reduce(31)
                } // skip_definitions: skip_definitions oinjection skip_definition oinjection
                _ => aa_syntax_error!(token; LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION),
            },
            79 => match tag {
                LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => {
                    lalr1plus::Action::Reduce(32)
                } // skip_definition: "%skip" REGEX
                _ => aa_syntax_error!(token; LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION),
            },
            80 => match tag {
                TOKEN | FIELD | INJECT => lalr1plus::Action::Reduce(15), // field_definition: "%field" field_type field_name field_conversion_function
                _ => aa_syntax_error!(token; TOKEN, FIELD, INJECT),
            },
            81 => match tag {
                TOKEN | FIELD | INJECT => {
                    if !Self::is_allowable_name(
                        aa_attributes.attribute_n_from_end(2 - 1).matched_text(),
                    ) {
                        lalr1plus::Action::Reduce(20) // field_conversion_function: IDENT ?(  !is_allowable_name($1.matched_text())  ?)
                    } else {
                        lalr1plus::Action::Reduce(21) // field_conversion_function: IDENT
                    }
                }
                _ => aa_syntax_error!(token; TOKEN, FIELD, INJECT),
            },
            82 => match tag {
                TOKEN | LEFT | RIGHT | NONASSOC | SKIP | INJECT | NEWSECTION => {
                    lalr1plus::Action::Reduce(25)
                } // token_definition: "%token" FIELDNAME new_token_name pattern
                _ => {
                    aa_syntax_error!(token; TOKEN, LEFT, RIGHT, NONASSOC, SKIP, INJECT, NEWSECTION)
                }
            },
            83 => match tag {
                VBAR | DOT => lalr1plus::Action::Reduce(51), // production_tail_list: production_tail_list "|" production_tail
                _ => aa_syntax_error!(token; VBAR, DOT),
            },
            84 => match tag {
                ACTION => lalr1plus::Action::Shift(48),
                VBAR | DOT => lalr1plus::Action::Reduce(57), // production_tail: symbol_list predicate tagged_precedence
                _ => aa_syntax_error!(token; VBAR, DOT, ACTION),
            },
            85 => match tag {
                VBAR | DOT => lalr1plus::Action::Reduce(58), // production_tail: symbol_list predicate action
                _ => aa_syntax_error!(token; VBAR, DOT),
            },
            86 => match tag {
                VBAR | DOT => lalr1plus::Action::Reduce(60), // production_tail: symbol_list tagged_precedence action
                _ => aa_syntax_error!(token; VBAR, DOT),
            },
            87 => match tag {
                VBAR | DOT | ACTION => lalr1plus::Action::Reduce(66), // tagged_precedence: "%prec" IDENT
                _ => aa_syntax_error!(token; VBAR, DOT, ACTION),
            },
            88 => match tag {
                VBAR | DOT | ACTION => lalr1plus::Action::Reduce(67), // tagged_precedence: "%prec" LITERAL
                _ => aa_syntax_error!(token; VBAR, DOT, ACTION),
            },
            89 => match tag {
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => lalr1plus::Action::Reduce(34), // precedence_definitions: precedence_definitions oinjection precedence_definition oinjection
                _ => aa_syntax_error!(token; LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION),
            },
            90 => match tag {
                LITERAL => lalr1plus::Action::Shift(92),
                IDENT => lalr1plus::Action::Shift(93),
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => lalr1plus::Action::Reduce(35), // precedence_definition: "%left" tag_list
                _ => aa_syntax_error!(token; LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT)
            },
            91 => match tag {
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    lalr1plus::Action::Reduce(38)
                } // tag_list: tag
                _ => aa_syntax_error!(token; LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT)
            },
            92 => match tag {
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    lalr1plus::Action::Reduce(40)
                } // tag: LITERAL
                _ => aa_syntax_error!(token; LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT)
            },
            93 => match tag {
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT => {
                    if self
                        .symbol_table
                        .is_known_token(aa_attributes.attribute_n_from_end(2 - 1).matched_text())
                    {
                        lalr1plus::Action::Reduce(41) // tag: IDENT ?(  self.symbol_table.is_known_token($1.matched_text())  ?)
                    } else if self.symbol_table.is_known_non_terminal(
                        aa_attributes.attribute_n_from_end(2 - 1).matched_text(),
                    ) {
                        lalr1plus::Action::Reduce(42) // tag: IDENT ?(  self.symbol_table.is_known_non_terminal($1.matched_text())  ?)
                    } else {
                        lalr1plus::Action::Reduce(43) // tag: IDENT
                    }
                }
                _ => aa_syntax_error!(token; LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT)
            },
            94 => match tag {
                LITERAL => lalr1plus::Action::Shift(92),
                IDENT => lalr1plus::Action::Shift(93),
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION => lalr1plus::Action::Reduce(36), // precedence_definition: "%right" tag_list
                _ => aa_syntax_error!(token; LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT)
            },
            95 => match tag {
                LITERAL => lalr1plus::Action::Shift(92),
                IDENT => lalr1plus::Action::Shift(93),
                LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION =>
                    // precedence_definition: "%nonassoc" tag_list
                    lalr1plus::Action::Reduce(37),
                _ => aa_syntax_error!(token; LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT)
            },
            96 => match tag {
                VBAR | DOT => lalr1plus::Action::Reduce(56), // production_tail: symbol_list predicate tagged_precedence action
                _ => aa_syntax_error!(token; VBAR, DOT),
            },
            97 => match tag {
                LITERAL | LEFT | RIGHT | NONASSOC | INJECT | NEWSECTION | IDENT =>
                    lalr1plus::Action::Reduce(39), // tag_list: tag_list tag
                _ => aa_syntax_error!(token; LITERAL, LEFT, RIGHT, NONASSOC, INJECT, NEWSECTION, IDENT)
            },
            _ => panic!("{}: invalid parser state.", state),
        }
    }

    fn next_coda(
        &self,
        state: u32,
        _attributes: &lalr1plus::ParseStack<AATerminal, AANonTerminal, AAAttributeData>,
    ) -> lalr1plus::Coda {
        match state {
            1 => lalr1plus::Coda::Accept,
            3 => lalr1plus::Coda::Reduce(3), // oinjection: injection
            10 => lalr1plus::Coda::Reduce(5), // injection: injection_head "."
            16 => lalr1plus::Coda::Reduce(9), // coda: <empty>
            25 => lalr1plus::Coda::Reduce(1), // specification: preamble definitions "%%" production_rules coda
            27 => lalr1plus::Coda::Reduce(2), // oinjection: <empty>
            30 => lalr1plus::Coda::Reduce(2), // oinjection: <empty>
            41 => lalr1plus::Coda::Reduce(10), // coda: oinjection DCODE
            42 => lalr1plus::Coda::Reduce(45), // production_rules: production_rules production_group oinjection
            55 => lalr1plus::Coda::Reduce(44), // production_rules: oinjection production_group oinjection
            66 => lalr1plus::Coda::Reduce(46), // production_group: production_group_head production_tail_list "."
            _ => lalr1plus::Coda::UnexpectedEndOfInput,
        }
    }

    fn production_data(&mut self, production_id: u32) -> (AANonTerminal, usize) {
        match production_id {
            1 => (AANonTerminal::Specification, 5),
            2 => (AANonTerminal::OInjection, 0),
            3 => (AANonTerminal::OInjection, 1),
            4 => (AANonTerminal::InjectionHead, 2),
            5 => (AANonTerminal::Injection, 2),
            6 => (AANonTerminal::Preamble, 0),
            7 => (AANonTerminal::Preamble, 3),
            8 => (AANonTerminal::Preamble, 5),
            9 => (AANonTerminal::Coda, 0),
            10 => (AANonTerminal::Coda, 2),
            11 => (AANonTerminal::Definitions, 4),
            12 => (AANonTerminal::FieldDefinitions, 0),
            13 => (AANonTerminal::FieldDefinitions, 4),
            14 => (AANonTerminal::FieldDefinition, 3),
            15 => (AANonTerminal::FieldDefinition, 4),
            16 => (AANonTerminal::FieldType, 1),
            17 => (AANonTerminal::FieldType, 1),
            18 => (AANonTerminal::FieldName, 1),
            19 => (AANonTerminal::FieldName, 1),
            20 => (AANonTerminal::FieldConversionFunction, 1),
            21 => (AANonTerminal::FieldConversionFunction, 1),
            22 => (AANonTerminal::TokenDefinitions, 2),
            23 => (AANonTerminal::TokenDefinitions, 4),
            24 => (AANonTerminal::TokenDefinition, 3),
            25 => (AANonTerminal::TokenDefinition, 4),
            26 => (AANonTerminal::NewTokenName, 1),
            27 => (AANonTerminal::NewTokenName, 1),
            28 => (AANonTerminal::Pattern, 1),
            29 => (AANonTerminal::Pattern, 1),
            30 => (AANonTerminal::SkipDefinitions, 0),
            31 => (AANonTerminal::SkipDefinitions, 4),
            32 => (AANonTerminal::SkipDefinition, 2),
            33 => (AANonTerminal::PrecedenceDefinitions, 0),
            34 => (AANonTerminal::PrecedenceDefinitions, 4),
            35 => (AANonTerminal::PrecedenceDefinition, 2),
            36 => (AANonTerminal::PrecedenceDefinition, 2),
            37 => (AANonTerminal::PrecedenceDefinition, 2),
            38 => (AANonTerminal::TagList, 1),
            39 => (AANonTerminal::TagList, 2),
            40 => (AANonTerminal::Tag, 1),
            41 => (AANonTerminal::Tag, 1),
            42 => (AANonTerminal::Tag, 1),
            43 => (AANonTerminal::Tag, 1),
            44 => (AANonTerminal::ProductionRules, 3),
            45 => (AANonTerminal::ProductionRules, 3),
            46 => (AANonTerminal::ProductionGroup, 3),
            47 => (AANonTerminal::ProductionGroupHead, 2),
            48 => (AANonTerminal::ProductionGroupHead, 2),
            49 => (AANonTerminal::ProductionGroupHead, 2),
            50 => (AANonTerminal::ProductionTailList, 1),
            51 => (AANonTerminal::ProductionTailList, 3),
            52 => (AANonTerminal::ProductionTail, 0),
            53 => (AANonTerminal::ProductionTail, 1),
            54 => (AANonTerminal::ProductionTail, 2),
            55 => (AANonTerminal::ProductionTail, 1),
            56 => (AANonTerminal::ProductionTail, 4),
            57 => (AANonTerminal::ProductionTail, 3),
            58 => (AANonTerminal::ProductionTail, 3),
            59 => (AANonTerminal::ProductionTail, 2),
            60 => (AANonTerminal::ProductionTail, 3),
            61 => (AANonTerminal::ProductionTail, 2),
            62 => (AANonTerminal::ProductionTail, 2),
            63 => (AANonTerminal::ProductionTail, 1),
            64 => (AANonTerminal::Action, 1),
            65 => (AANonTerminal::Predicate, 1),
            66 => (AANonTerminal::TaggedPrecedence, 2),
            67 => (AANonTerminal::TaggedPrecedence, 2),
            68 => (AANonTerminal::SymbolList, 1),
            69 => (AANonTerminal::SymbolList, 2),
            70 => (AANonTerminal::Symbol, 1),
            71 => (AANonTerminal::Symbol, 1),
            72 => (AANonTerminal::Symbol, 1),
            _ => panic!("Malformed production data table"),
        }
    }

    fn goto_state(_lhs: &AANonTerminal, _current_state: u32) -> u32 {
        0
    }

    fn do_semantic_action(
        &mut self,
        _production_id: u32,
        _rhs: Vec<AAAttributeData>,
        _token_stream: &mut lexan::TokenStream<AATerminal>,
    ) -> AAAttributeData {
        AAAttributeData::default()
    }
}
