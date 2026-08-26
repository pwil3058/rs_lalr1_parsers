// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use crate::symbol::terminal::Token;
use crate::symbol::{Associativity, Symbol, non_terminal::NonTerminal, terminal::TokenSet};

use lalr1::OrderedSet;

use lazy_static::lazy_static;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::io;
use std::io::Write;
use std::iter::FromIterator;
use std::ops::Index;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::atomic::{self, AtomicU32};

#[derive(Debug, Default, Clone)]
pub struct ProductionTailData {
    right_hand_side: Box<[Symbol]>,
    associativity: Associativity,
    precedence: u16,
    action: Option<String>,
}

#[derive(Debug, Default)]
pub struct ProductionTail(Rc<ProductionTailData>);

impl Clone for ProductionTail {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl ProductionTail {
    pub fn new(
        right_hand_side: &[Symbol],
        associative_precedence: Option<(Associativity, u16)>,
        o_action: Option<&str>,
    ) -> Self {
        let action = o_action.map(|action| action.to_string());
        let (associativity, precedence) = if let Some(tuple) = associative_precedence {
            tuple
        } else {
            rhs_associated_precedence(right_hand_side).unwrap_or_default()
        };
        Self(Rc::new(ProductionTailData {
            right_hand_side: right_hand_side.to_vec().into(),
            action,
            associativity,
            precedence,
        }))
    }
}

fn rhs_associated_precedence(symbols: &[Symbol]) -> Option<(Associativity, u16)> {
    for symbol in symbols.iter() {
        if let Symbol::Terminal(token) = symbol {
            return Some(token.associativity_and_precedence());
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProductionId(u32);

impl ProductionId {
    fn new() -> Self {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        ProductionId(NEXT_ID.fetch_add(1, atomic::Ordering::Relaxed))
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Display for ProductionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct ProductionData {
    pub ident: ProductionId,
    left_hand_side: NonTerminal,
    tail: ProductionTail,
}

impl PartialEq for ProductionData {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident
    }
}

impl Eq for ProductionData {}

impl Ord for ProductionData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ident.cmp(&other.ident)
    }
}

impl PartialOrd for ProductionData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Production(Rc<ProductionData>);

impl Clone for Production {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

lazy_static! {
    static ref RHS_CRE: regex::Regex = regex::Regex::new(r"\$(\d+)").unwrap();
}

impl Production {
    pub fn new(left_hand_side: NonTerminal, tail: ProductionTail) -> Self {
        Self(Rc::new(ProductionData {
            ident: ProductionId::new(),
            left_hand_side,
            tail,
        }))
    }

    pub fn ident(&self) -> ProductionId {
        self.0.ident
    }

    pub fn is_start_production(&self) -> bool {
        self.0.ident.is_zero()
    }

    pub fn len(&self) -> usize {
        self.0.tail.0.right_hand_side.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn left_hand_side(&self) -> &NonTerminal {
        &self.0.left_hand_side
    }

    pub fn right_hand_side_symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.0.tail.0.right_hand_side.iter()
    }

    pub fn associativity(&self) -> Associativity {
        self.0.tail.0.associativity
    }

    pub fn precedence(&self) -> u16 {
        self.0.tail.0.precedence
    }

    pub fn expanded_action(&self) -> Option<String> {
        // TODO: move action expansion to RHS creation
        if let Some(action) = &self.0.tail.0.action {
            let string = action.replace("$$", "aa_lhs");
            let string = string.replace("$INJECT", "aa_inject");
            let string = RHS_CRE
                .replace_all(&string, |caps: &regex::Captures| {
                    format!("aa_rhs[{}]", usize::from_str(&caps[1]).unwrap() - 1)
                })
                .to_string();
            Some(string)
        } else {
            None
        }
    }

    pub fn has_error_recovery_tail(&self) -> bool {
        if let Some(symbol) = self.0.tail.0.right_hand_side.last() {
            match symbol {
                Symbol::Terminal(_) => false,
                Symbol::NonTerminal(non_terminal) => non_terminal.is_error(),
            }
        } else {
            false
        }
    }
}

impl Display for Production {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string = format!("{}:", self.left_hand_side().name());
        if self.0.tail.0.right_hand_side.is_empty() {
            string += " <empty>";
        } else {
            for symbol in self.0.tail.0.right_hand_side.iter() {
                string += &format!(" {symbol}");
            }
        };
        string += &format!(" #({}, {})", self.associativity(), self.precedence());
        write!(f, "{string}")
    }
}

#[derive(Debug, Default)]
pub struct Productions(Vec<Production>);

impl Productions {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = &Production> {
        self.0.iter()
    }

    pub fn push(&mut self, production: Production) {
        self.0.push(production);
    }

    pub fn base(&self) -> &Production {
        self.0.first().expect("Productions is empty")
    }

    pub fn closure(&self, mut closure_set: GrammarItemSet) -> GrammarItemSet {
        let mut additions_made = true;
        while additions_made {
            additions_made = false;
            // Closables extraction as a new separate map necessary to avoid borrow conflict
            for (item_key, look_ahead_set) in closure_set.closable_set() {
                if let Some(symbol) = item_key.next_symbol() {
                    match symbol {
                        Symbol::Terminal(_) => debug_assert!(!item_key.is_closable()),
                        Symbol::NonTerminal(prospective_lhs) => {
                            debug_assert!(item_key.is_closable());
                            for look_ahead_symbol in look_ahead_set.iter() {
                                let firsts = TokenSet::first_all_caps(
                                    item_key.rhs_tail(),
                                    look_ahead_symbol,
                                );
                                for production in self
                                    .0
                                    .iter()
                                    .filter(|x| x.left_hand_side() == prospective_lhs)
                                {
                                    let prospective_key = GrammarItemKey::from(production);
                                    if let Some(set) = closure_set.get_mut(&prospective_key) {
                                        let len = set.len();
                                        *set |= &firsts;
                                        additions_made = additions_made || set.len() > len;
                                    } else {
                                        closure_set.insert(prospective_key, firsts.clone());
                                        additions_made = true;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    debug_assert!(!item_key.is_closable());
                }
            }
        }
        closure_set
    }
}

impl Productions {
    pub fn write_production_data_code<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        wtr.write_all(b"    fn production_data(production_id: u32) -> (AANonTerminal, usize) {\n")?;
        wtr.write_all(b"        match production_id {\n")?;
        for production in self.0.iter() {
            wtr.write_fmt(format_args!(
                "            {} => (AANonTerminal::{}, {}),\n",
                production.ident(),
                production.left_hand_side().name(),
                production.len(),
            ))?;
        }
        wtr.write_all(b"            _ => panic!(\"malformed production data table\"),\n")?;
        wtr.write_all(b"        }\n")?;
        wtr.write_all(b"    }\n\n")?;
        Ok(())
    }

    pub fn write_semantic_action_code<W: Write>(
        &self,
        wtr: &mut W,
        attribute_type: &str,
    ) -> io::Result<()> {
        wtr.write_all(b"    fn do_semantic_action<F: FnMut(String, String)>(\n")?;
        wtr.write_all(b"        &mut self,\n")?;
        wtr.write_all(b"        aa_production_id: u32,\n")?;
        wtr.write_fmt(format_args!("        aa_rhs: Vec<{}>,\n", attribute_type))?;
        wtr.write_all(b"        mut aa_inject: F,\n")?;
        wtr.write_fmt(format_args!("    ) -> {} {{\n", attribute_type))?;
        wtr.write_all(b"        let mut aa_lhs = if let Some(a) = aa_rhs.first() {\n")?;
        wtr.write_all(b"            a.clone()\n")?;
        wtr.write_all(b"        } else {\n")?;
        wtr.write_fmt(format_args!("           {}::default()\n", attribute_type))?;
        wtr.write_all(b"        };\n")?;
        wtr.write_all(b"        match aa_production_id {\n")?;
        for production in self.0.iter() {
            if let Some(action_code) = production.expanded_action() {
                wtr.write_fmt(format_args!("            {} => {{\n", production.ident()))?;
                wtr.write_fmt(format_args!("                // {production}\n"))?;
                wtr.write_fmt(format_args!("                {action_code}\n"))?;
                wtr.write_all(b"            }\n")?;
            }
        }
        wtr.write_all(b"            _ => aa_inject(String::new(), String::new()),\n")?;
        wtr.write_all(b"        };\n")?;
        wtr.write_all(b"        aa_lhs\n")?;
        wtr.write_all(b"    }\n\n")?;
        Ok(())
    }

    pub fn write_description<W: Write>(&self, wtr: &mut W) -> io::Result<()> {
        for production in self.0.iter() {
            wtr.write_fmt(format_args!("  {production}\n"))?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Reductions {
    reductions: BTreeMap<OrderedSet<Production>, TokenSet>,
}

impl Reductions {
    pub fn len(&self) -> usize {
        self.reductions.len()
    }

    pub fn reductions(&self) -> impl Iterator<Item = (&OrderedSet<Production>, &TokenSet)> {
        self.reductions.iter()
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub struct GrammarItemKey {
    production: Production,
    dot: usize,
}

impl Display for GrammarItemKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string = format!("{}:", self.production.0.left_hand_side.name());
        if self.production.0.tail.0.right_hand_side.is_empty() {
            string += " . <empty>";
        } else {
            for (index, symbol) in self.production.0.tail.0.right_hand_side.iter().enumerate() {
                if index == self.dot {
                    string += &format!(" . {symbol}");
                } else {
                    string += &format!(" {symbol}");
                }
            }
            if self.dot >= self.production.0.tail.0.right_hand_side.len() {
                string += " . ";
            }
        };
        string += &format!(
            " #({}, {})",
            self.production.associativity(),
            self.production.precedence()
        );
        write!(f, "{string}")
    }
}

impl From<&Production> for GrammarItemKey {
    fn from(production: &Production) -> Self {
        Self {
            production: production.clone(),
            dot: 0,
        }
    }
}

impl GrammarItemKey {
    pub fn production(&self) -> &Production {
        &self.production
    }

    pub fn shifted(&self) -> Self {
        debug_assert!(self.dot < self.production.len());
        let dot = self.dot + 1;
        Self {
            production: self.production.clone(),
            dot,
        }
    }

    pub fn is_closable(&self) -> bool {
        if let Some(symbol) = self.production.0.tail.0.right_hand_side.get(self.dot) {
            symbol.is_non_terminal()
        } else {
            false
        }
    }

    pub fn is_kernel_item(&self) -> bool {
        self.dot > 0 || self.production.0.left_hand_side.is_start()
    }

    pub fn is_reducible(&self) -> bool {
        self.dot >= self.production.0.tail.0.right_hand_side.len()
    }

    pub fn next_symbol(&self) -> Option<&Symbol> {
        self.production.0.tail.0.right_hand_side.get(self.dot)
    }

    pub fn next_symbol_is(&self, symbol: &Symbol) -> bool {
        if let Some(next_symbol) = self.next_symbol() {
            next_symbol == symbol
        } else {
            false
        }
    }

    pub fn rhs_tail(&self) -> &[Symbol] {
        &self.production.0.tail.0.right_hand_side[self.dot + 1..]
    }

    pub fn associativity(&self) -> Associativity {
        self.production.associativity()
    }

    pub fn precedence(&self) -> u16 {
        self.production.precedence()
    }

    pub fn has_error_recovery_tail(&self) -> bool {
        self.production.has_error_recovery_tail()
    }

    pub fn has_reducible_error_recovery_tail(&self) -> bool {
        self.is_reducible() && self.production.has_error_recovery_tail()
    }
}

#[derive(Debug, Default)]
pub struct GrammarItemSet(BTreeMap<GrammarItemKey, TokenSet>);

impl From<BTreeMap<GrammarItemKey, TokenSet>> for GrammarItemSet {
    fn from(map: BTreeMap<GrammarItemKey, TokenSet>) -> Self {
        Self(map)
    }
}

impl Index<&GrammarItemKey> for GrammarItemSet {
    type Output = TokenSet;

    fn index(&self, key: &GrammarItemKey) -> &TokenSet {
        self.0.index(key)
    }
}

impl FromIterator<(GrammarItemKey, TokenSet)> for GrammarItemSet {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (GrammarItemKey, TokenSet)>,
    {
        Self(BTreeMap::<GrammarItemKey, TokenSet>::from_iter(iter))
    }
}

impl GrammarItemSet {
    pub fn iter(&self) -> impl Iterator<Item = (&GrammarItemKey, &TokenSet)> {
        self.0.iter()
    }

    pub fn closable_set(&self) -> Vec<(GrammarItemKey, TokenSet)> {
        let mut closables = vec![];
        for (key, set) in self.0.iter().filter(|x| x.0.is_closable()) {
            closables.push((key.clone(), set.clone()));
        }
        closables
    }

    pub fn generate_goto_kernel(&self, symbol: &Symbol) -> GrammarItemSet {
        self.0
            .iter()
            .filter(|t| t.0.next_symbol_is(symbol))
            .map(|t| (t.0.shifted(), t.1.clone()))
            .collect()
    }

    pub fn kernel_key_set(&self) -> OrderedSet<GrammarItemKey> {
        self.0
            .keys()
            .filter(|x| x.is_kernel_item())
            .cloned()
            .collect()
    }

    pub fn irreducible_key_set(&self) -> OrderedSet<GrammarItemKey> {
        self.0
            .keys()
            .filter(|x| !x.is_reducible())
            .cloned()
            .collect()
    }

    pub fn reducible_key_set(&self) -> OrderedSet<GrammarItemKey> {
        self.0
            .keys()
            .filter(|x| x.is_reducible())
            .cloned()
            .collect()
    }

    pub fn get_mut(&mut self, key: &GrammarItemKey) -> Option<&mut TokenSet> {
        self.0.get_mut(key)
    }

    pub fn insert(&mut self, key: GrammarItemKey, look_ahead_set: TokenSet) -> Option<TokenSet> {
        self.0.insert(key, look_ahead_set)
    }

    pub fn look_ahead_intersection(
        &self,
        key1: &GrammarItemKey,
        key2: &GrammarItemKey,
    ) -> TokenSet {
        self.0
            .get(key1)
            .unwrap()
            .intersection(self.0.get(key2).unwrap())
            .cloned()
            .collect()
    }

    pub fn remove_look_ahead_symbols(&mut self, key: &GrammarItemKey, symbols: &TokenSet) {
        let look_ahead_set = self.0.get_mut(key).unwrap();
        *look_ahead_set = look_ahead_set.difference(symbols).cloned().collect();
    }

    pub fn error_recovery_look_ahead_set_contains(&self, token: &Token) -> bool {
        for look_ahead_set in self
            .0
            .iter()
            .filter(|x| x.0.has_reducible_error_recovery_tail())
            .map(|x| x.1)
        {
            if look_ahead_set.contains(token) {
                return true;
            }
        }
        false
    }

    pub fn reducible_look_ahead_set(&self) -> TokenSet {
        let mut set = TokenSet::new();
        for (_, look_ahead_set) in self.0.iter().filter(|x| x.0.is_reducible()) {
            set |= look_ahead_set;
        }
        set
    }

    pub fn reductions(&self) -> Reductions {
        let expected_tokens = self.reducible_look_ahead_set();
        #[allow(clippy::mutable_key_type)]
        let mut reductions: BTreeMap<OrderedSet<Production>, TokenSet> = BTreeMap::new();
        for token in expected_tokens.iter() {
            let mut productions: OrderedSet<Production> = OrderedSet::new();
            for (item_key, look_ahead_set) in self.0.iter().filter(|x| x.0.is_reducible()) {
                if look_ahead_set.contains(token) {
                    productions.insert(item_key.production.clone());
                }
            }
            let look_ahead_set = reductions.entry(productions).or_default();
            look_ahead_set.insert(token);
        }
        Reductions { reductions }
    }
}
