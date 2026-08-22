# alalr1: An Augmented LALR (1) parser library.

~~~
// Copyright 2026 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

%token  RegEx           (\(.+\))
%token  Literal         ("(\\"|[^"\t\r\n\v\f])*")
%token  Ident           ([a-zA-Z]+[a-zA-Z0-9_]*)
%token  PredicateExpr   (\?\((.|[\n\r])*?\?\))
%token  ActionCode      (!\{(.|[\n\r])*?!\})
%token  RustCode        (%\{(.|[\n\r])*?%\})
%token  NumberExpr      ([0-9]+)

Specification: Preamble Configuration "%%" Definitions "%%" ProductionRules.

OptionalInjection:
    | Injection .

InjectionHead: "%inject" Literal .

Injection: InjectionHead "." .

Preamble:
    | OptionalInjection RustCode OptionalInjection .

Configuration: AttributeType OptionalInjection TargetType OptionalInjection ExpectedConflicts OptionalInjection
    | TargetType OptionalInjection AttributeType OptionalInjection ExpectedConflicts OptionalInjection
    .

AttributeType: "%attr" Ident .

TargetType: "%target" Ident .

ExpectedConflicts:
    | ExpectedRRConflicts OptionalInjection  ExpectedSRConflicts
    | ExpectedSRConflicts OptionalInjection  ExpectedRRConflicts
    | ExpectedRRConflicts
    | ExpectedSRConflicts
    .

ExpectedRRConflicts: "%reduce_reduce" Number .

ExpectedSRConflicts: "%shift_reduce" Number .

Number: NumberExpr .

Definitions : TokenDefinitions SkipDefinitions PrecedenceDefinitions .

TokenDefinitions : OptionalInjection TokenDefinition
    | TokenDefinitions OptionalInjection TokenDefinition OptionalInjection
    .

TokenDefinition: "%token" NewTokenName Literal
    | "%token" NewTokenName RegularExpression
    .

RegularExpression: RegEx .

NewTokenName: Ident .

SkipDefinitions:
    | SkipDefinitions OptionalInjection SkipDefinition OptionalInjection
    .

SkipDefinition: "%skip" RegularExpression .

PrecedenceDefinitions:
    | PrecedenceDefinitions OptionalInjection PrecedenceDefinition OptionalInjection
    .

PrecedenceDefinition: "%left" TagList
    | "%right" TagList
    | "%nonassoc" TagList
    .

TagList: Tag
    | TagList Tag
    .

Tag: Literal
    | Ident
    .

ProductionRules: OptionalInjection ProductionGroup OptionalInjection
    | ProductionRules ProductionGroup OptionalInjection
    .

ProductionGroup: ProductionGroupHead ProductionTailList "." .

ProductionGroupHead: Ident ":" .

ProductionTailList: ProductionTail
    | ProductionTailList "|" ProductionTail
    .

ProductionTail:
    | Action
    | Predicate Action
    | Predicate
    | SymbolList Predicate TaggedPrecedence Action
    | SymbolList Predicate TaggedPrecedence
    | SymbolList Predicate Action
    | SymbolList Predicate
    | SymbolList TaggedPrecedence Action
    | SymbolList TaggedPrecedence
    | SymbolList Action
    | SymbolList
    .

Action: ActionCode .

Predicate: PredicateExpr .

TaggedPrecedence: "%prec" Ident
    | "%prec" Literal
    .

SymbolList: Symbol
    | SymbolList Symbol
    .

Symbol: Ident
    | Literal
    | "%error"
    .
~~~
