# Augmented Lexical Analyzer and Parser Generator
 
**alapgen** is an augmented lexical analyser and parser generation
tool (in the vein of *yacc*, *bison*, *dunnart*, *lex*, *flex*, etc) written in
and targeted at Rust.
It takes a specification file as input and uses it to implement the
[`lalr1plus::Parser`](https://github.com/pwil3058/rs_lalr1plus)
trait for a specified Rust type and to generate an instance of
[`lexan::LexicalAnalyzer`](https://github.com/pwil3058/rs_lexan)
for use by the parser.  The **alapgen** module that parses the specification file
was itself generated from an **alapgen** specification file by an iterative
process whereby the first version was hand written using **dunnart**'s equivalent
(also written by me, so no plagiarism) as a guide.

The augmentations of **alapgen** with respect to other similar tools are:
1. the lexical analyzer and parser are both generated from a single specification,
2. predicates may be attached to grammar productions in order to (among other things)
resolve conflicts, and
3. extra text may be injected into the parser's input stream from within productions'
action code.

## Synopsis

```
Augmented Lexical Analyzer and Parser Generator 

USAGE:
    alapgen [FLAGS] [OPTIONS] <specification>

FLAGS:
    -f, --force      overwrite the output files (if they exist)
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --expect <expect>    the exact number of shift/reduce and/or reduce/reduce conflicts to expect

ARGS:
    <specification>    the path of the file containing the grammar specification
```

## Example Specification

```bash
%{
use std::collections::HashMap;
use std::convert::From;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum AttributeData {
    Token(lexan::Token<AATerminal>),
    Error(lalr1plus::Error<AATerminal>),
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

impl From<lalr1plus::Error<AATerminal>> for AttributeData {
    fn from(error: lalr1plus::Error<AATerminal>) -> Self {
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

impl lalr1plus::ReportError<AATerminal> for Calc {}

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
%}

%attr   AttributeData
%target Calc

%%

%token          EOL     (\n)
%token          PLUS    "+"
%token          MINUS   "-"
%token          TIMES   "*"
%token          DIVIDE  "/"
%token          ASSIGN  "="
%token          NUMBER  ([0-9]+(\.[0-9]+){0,1})
%token          ID      ([a-zA-Z]+)
%token          LPR     "("
%token          RPR     ")"

%skip   ([\t\r ]+)

%right  UMINUS
%left   "*" "/"
%left   "+" "-"
%left   EOL

%%
Line: SetUp Expr ?(self.errors > 0?) !{self.report_errors();!}
    | SetUp Expr !{println!("{}", $2.value());!}
    | SetUp ID "=" Expr ?(self.errors == 0?) !{self.variables.insert($2.id().clone(), $4.value());!}
    | SetUp ID "=" Expr !{self.report_errors();!}
    | Line EOL Line
    | Line EOL
    | %error !{self.errors |= SYNTAX_ERROR;!}
    .

SetUp: !{self.errors = 0;!}.

Expr: Expr "+" Expr ?($1.value() == 0.0?) !{$$ = AttributeData::Value($3.value());!}
    | Expr "+" Expr ?($3.value() == 0.0?) !{$$ = AttributeData::Value($1.value());!}
    | Expr "+" Expr !{$$ = AttributeData::Value($1.value() + $3.value());!}
    | Expr "-" Expr ?($1.value() == 0.0?) !{$$ = AttributeData::Value(-$3.value());!}
    | Expr "-" Expr ?($3.value() == 0.0?) !{$$ = AttributeData::Value($1.value());!}
    | Expr "-" Expr !{$$ = AttributeData::Value($1.value() - $3.value());!}
    | Expr "*" Expr ?($1.value() == 0.0 || $3.value() == 0.0?) !{$$ = AttributeData::Value(-$3.value());!}
    | Expr "*" Expr ?($1.value() == 1.0?) !{$$ = AttributeData::Value($3.value());!}
    | Expr "*" Expr ?($3.value() == 1.0?) !{$$ = AttributeData::Value($1.value());!}
    | Expr "*" Expr !{$$ = AttributeData::Value($1.value() * $3.value());!}
    | Expr "/" Expr ?($3.value() == 1.0?) !{$$ = AttributeData::Value($1.value());!}
    | Expr "/" Expr ?($3.value() == 0.0?) !{self.errors |= DIVIDE_BY_ZERO;!}
    | Expr "/" Expr ?($1.value() == 0.0?) !{$$ = AttributeData::Value(0.0);!}
    | Expr "/" Expr !{$$ = AttributeData::Value($1.value() / $3.value());!}
    | "(" Expr ")" !{$$ = AttributeData::Value($2.value());!}
    | "-" Expr %prec UMINUS !{$$ = AttributeData::Value(-$2.value());!}
    | NUMBER !{$$ = AttributeData::Value($1.value());!}
    | ID ?(self.variables.contains_key($1.id())?) !{$$ = AttributeData::Value(self.variables[$1.id()]);!}
    | ID !{self.errors |= UNDEFINED_VARIABLE; $$ = AttributeData::Value(0.0);!}
    .
```