# LALR(1) Parser Generation Tools

This workspace contains three crates:
1. *lalr1_plus* defines a public trait `Parser` which is (when implemented) is an **LALR(1)** parser, 
2. *lexan* is a configurable lexical analyser `LexicalAnalyzer<T>` where `T` is a lexical token, and
3. *alap_gen* is a binary crate whose binary file implements a `Parser` (including a configured `LexicalAnalyser<T>`)
   from a specification file.
4. *alap_gen_ng* is a binary crate whose binary file implements a `Parser` (including a configured `LexicalAnalyser<T>`)
   from a specification file it is an improved version of *alap_gen* and will soon replace it.
   
which can be used to create **LALR(1)** parsers from specification files.

The crate *test_calc* is an test parser.

