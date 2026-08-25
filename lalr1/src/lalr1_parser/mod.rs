// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

#[cfg(feature = "bootstrap")]
pub mod bootstrap;
#[cfg(not(feature = "bootstrap"))]
pub mod parser;
