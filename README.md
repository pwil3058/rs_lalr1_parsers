# LALR (1) Parser Generation Tools

This workspace contains six crates:

1. *lalr1* defines a public trait `ParserGenerator` which is (when implemented) is an **LALR (1)** parser,
2. *lexan* is a library containing a configurable lexical analyser `LexicalAnalyzer<T>` where `T` is a lexical token,
3. *lalr1_lib* is a library containing a parser generator `ParserGenerator`
   which can be used to produce parsers(including a configured `LexicalAnalyser<T>`) on a specified file from a
   specification file,
4. *lalr1_gen* is a binary crate whose binary file implements the `Parser` trait from *lalr1*
   (including a configured `LexicalAnalyser<T>`) on a nominated type from a specification file,
5. lalr1_lib* is a library containing a parser generator `ParserGenerator`
   which can be used to produce augmented parsers(including a configured `LexicalAnalyser<T>`) on a specified file from
   a specification file
6. *alalr1_gen* is a binary crate whose binary file implements the `Parser` trait from *lalr1*
   (including a configured `LexicalAnalyser<T>`) on nominated type from a specification file.

which can be used to create **LALR (1)** parsers from specification files.

The crate *examples/calc_no_aug* is an example plain 'lalr1' parser and the crates *examples/calc* and
*examples/calc_no_er*
are examples of augmented 'lalr1' parsers.

The file *lalr1_lib/src/parser.laps* is also an example of a plain 'lalr1' grammar specification file and was used to
generate *lalr1_lib*'s internal parser (*lalr1/src/parser.rs*).
The file *alalr1_lib/src/parser.alaps* is an
example of an augmented 'alalr1' grammar specification file and was used to generate *alalr1_lib*'s internal parser
(*alap_lib/src/parser.rs*).

