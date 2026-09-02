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
### Preamble
The preamble consists of arbitrary **Rust** code between a **%{**  **%}** pair which is copied verbatim
into the start of the generated file.
### Configuration.
#### Attribute Type (%attr)
The attribute type statement
```
%attr Ident
```
specifies the **Rust** type that is to be used within **action code** to represent the attributes held the
components of the **production** with which the **action code** is associated.
#### Target Type (%target)
The target type statement
```
%target Ident
```
specifies the **Rust** type for which the `Parser` trait is to be implemented.
The complexity of this type can vary greatly depending on the needs of the parser being generated.
For example, the target type used to build the parser in this package
```
#[derive(Debug, Default)]
pub struct Specification {
    pub symbol_table: SymbolTable,
    productions: Productions,
    preamble: String,
    pub attribute_type: String,
    pub target_type: String,
    pub error_count: u32,
    pub warning_count: u32,
    pub expected_rr_conflicts: u32,
    pub expected_sr_conflicts: u32,
}

```
is of medium complexity and one for use building a parser for a programming language compiler would
need to be more complex.
On the other hand, one for a simple arithmetical expression parser
```
pub struct Calc {
    errors: u32,
    variables: HashMap<String, f64>,
}

```
could be vary simple.
The purpose of this type is to hold the data extracted from the input text in a form amenable to further
processing.
As an aside, it takes 2574 lines of **Rust** code to implement the `Parser` trait from the data held in a `Specification`.
####Expected Conflicts
The expected conflicts statements specify the number of reduce-reduce conflicts
```
%reduce_reduce Number
```
and shift-reduce conflicts
``` 
%shift_reduce Number
```
that are expected when implementing `Parser`. If not specified they default to zero.
###Definitions
####Token Definitions
The token definitions define the lexicon of the `Parser` definition.
There are two types of tokens: literal tokens
```
%token TokenName ("(\\"|[^"\t\r\n\v\f])*")
```
where a literal string enclosed in `""` quotes is assigned a name, e.g.
```
%token Plus "+"
```
in the ensuing `Production` definitions either the name or the literal value (e.g. "+") may be used.
Using the literal value tends to make the `Production` definitions more readable.
The second type of token is the `Regex` token
```
%token TokenName (\(.+\))
```
where a `Regex` regular expression enclosed in `()` brackets is associated with a name, e.g.
```
%token Id ([a-zA-Z]+[a-zA-Z0-9_]*)
```
and in `Production` definitions they may only be referred by their names.

These definitions are used to define a `enum AATerminal`, `lexan::Token<AATerminal>` and an
 `lalr1::Error<AATerminal>` and the type nominated as the attribute type in the `%attr` statement
must implement `From<lexan::Token<AATerminal>` and `From<lalr1::Error<AATerminal>`.

Tokens have three methods:
```
    pub fn tag(&self) -> &AATerminal;
    pub fn lexeme(&self) -> &String;
    pub fn location(&self) -> &Location;
```
which provide information which may be used in these conversions. The `String` pointed to by `lexeme()`
is the sub string in the input text which was consumed by the token and the `Location` contains data
re where the token was found. It has three useful methods:
```
    pub fn line_number(&self) -> usize;
    pub fn offset(&self) -> usize;
    pub fn label(&self) -> &String;
```
where the `String` pointed to by `label()` is the label associated with the initial input text or
an injected text. This information is useful for error messages.

####Skip Definitions
The `%skip` definitions:
```
%skip (\(.+\))
```
are unnamed `Regex` regular expressions that define text in the input that is to be skipped, e.g. whitespace and comments. There can be as many `%skip` statements as are necessary.
The token and skip definitions are then used to implement a `lexan::TokenStream` for the input text
and its associated label within the `Parser::parse_text(text, label)` method.

The `lexan::TokenStream` (in
addition to its primary task of supplying a stream of tokens) has the ability to have labelled
texts injected into the stream at any point. This feature can be used via an  
```
%inject Literlal
```
in the specification to include text as part of the specification or using an `$INJECT(text, label)` method call within a `Production`'s action code to include text as part of the text being parsed.
#### Precedence and Associativity Section
This section consists of a list of associativity statements
```
%left   TagList
%right  TalList
```
or
```
%nonassoc TagList
```
allocating associativity to the items in the TagLists which may be literal tokens or tags to be used in
the precedence/associativity section of `Production` definitions
and precedence is determined by the (ascending) order of the statements, , e.g.

```
%left   "+" "-"
%left   "*" "/"
%right  UMINUS
```
would specify conventional precedence and associativity for arithmetic.
Tokens not mentioned have no precedence or associativity.
The `UMINUS` tag in the abve example could be used like this.

```
Expr: "-" Expr %prec UMINUS !{$$ = AttributeData::Value(-$2.value());!}

```
in a `Production` definition.

### Production Rules
Production rules define the syntax of the "language" to be parsed.