# LALR(1) Parser Generation Tools

This workspace contains three crates:
1. *lalr1_plus* defines a public trait `Parser` which is (when implemented) is an **LALR(1)** parser, 
2. *lexan* is a configurable lexical analyser `LexicalAnalyzer<T>` where `T` is a lexical token, and
3. *alap_gen* is a binary crate whose binary file implements the `Parser` trait from *lalr1_plus*
  (including a configured `LexicalAnalyser<T>`) on nominated type from a specification file.
   
which can be used to create **LALR(1)** parsers from specification files.

The crates *tests/calc* and *tests/calc_no_er* are test parsers.

The file *alap_gen/src/alap_gen.alaps* is also an example of a grammar specification file and was used
to generate *alap_gen*'s internal parser.

