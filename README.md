# LALR(1) Parser Generation Tools

This workspace contains three crates:
1. *lalr1* defines a public trait `Parser` which is (when implemented) is an **LALR(1)** parser,
2. *lalr1_plus* defines a public trait `Parser` which is (when implemented) is an augmented **LALR(1)** parser,
3. *lexan* is a configurable lexical analyser `LexicalAnalyzer<T>` where `T` is a lexical token, and
4. *lap_gen* is a binary crate whose binary file implements the `Parser` trait from *lalr1*
   (including a configured `LexicalAnalyser<T>`) on nominated type from a specification file.
5. *alap_gen* is a binary crate whose binary file implements the `Parser` trait from *lalr1_plus*
   (including a configured `LexicalAnalyser<T>`) on nominated type from a specification file.
   
which can be used to create **LALR(1)** parsers from specification files.

The crate *examples/calc_no_aug* is an example 'lalr1' parser and
the crates *examples/calc* and *examples/calc_no_er* are example 'lalr1_plus' parsers.

The file *lap_gen/src/lap_gen.laps* is also an example of a 'lap_gen' grammar specification file and was used
to generate *lap_gen*'s internal parser (*lap_gen/src/lap_gen.rs*) and
the file *alap_gen/src/alap_gen.alaps* is also an example of an 'alap_gen' grammar specification file and was used
to generate *alap_gen*'s internal parser (*alap_gen/src/alap_gen.rs*).

