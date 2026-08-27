# Look Ahead Left Right 1 (lalr1) Parser Generator

This library provides a mechanism for producing a **Rust** module that implements the `pub trait Parser<T, N, A>`
(see `lalr1` library) for a nominated target type. It uses the `lexan` library internally for lexical analysis of the
input text. The details of the actions to be performed by the `Parser` trait are specified by a text following the
format described below where

```
%token  RegEx           (\(.+\))
%token  Literal         ("(\\"|[^"\t\r\n\v\f])*")
%token  Ident           ([a-zA-Z]+[a-zA-Z0-9_]*)
%token  ActionCode      (!\{(.|[\n\r])*?!\})
%token  RustCode        (%\{(.|[\n\r])*?%\})
%token  NumberExpr      ([0-9]+)
```

are tokens described by the associated regular expression and used within the description. The parser used within this
library was produced by this library using a boostrap procedure where the first version was written by hand. The first
version was (obviously) very simple and did just enough to build itself. After that the parser was refined by gradually
adding features via the specification file.

## Parser Specification

```
Specification: Preamble Configuration "%%" Definitions "%%" ProductionRules.

OptionalInjection: | Injection .

Injection: InjectionHead "." .

InjectionHead: "%inject" Literal .

Preamble: | OptionalInjection RustCode OptionalInjection .

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

SkipDefinitions :  | SkipDefinitions OptionalInjection SkipDefinition OptionalInjection .

SkipDefinition: "%skip" RegularExpression .

PrecedenceDefinitions : | PrecedenceDefinitions OptionalInjection PrecedenceDefinition OptionalInjection .

PrecedenceDefinition: "%left" TagList
    | "%right" TagList
    | "%nonassoc" TagList
    .

TagList: Tag
    | TagList Tag .

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
    | SymbolList TaggedPrecedence Action
    | SymbolList TaggedPrecedence
    | SymbolList Action
    | SymbolList
    .

Action: ActionCode .

TaggedPrecedence: "%prec" Ident
    | "%prec" Literal
    .

SymbolList: Symbol
    | SymbolList Symbol
    .

Symbol: Ident
    | Literal
    | "%error"
        !{
            let location = aa_rhs[0].location();
            let symbol = self.symbol_table.error_symbol_used_at(location);
            $$ = AttributeData::Symbol(symbol);
        !}
    .
```