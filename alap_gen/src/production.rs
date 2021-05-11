use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
    str::FromStr,
};

use crate::symbols::{AssociativePrecedence, Associativity, Symbol, SymbolSet};
use std::ops::Index;

#[derive(Debug, Clone, Default)]
pub struct ProductionTail {
    right_hand_side: Vec<Rc<Symbol>>,
    predicate: Option<String>,
    associative_precedence: AssociativePrecedence,
    action: Option<String>,
}

impl ProductionTail {
    pub fn new(
        right_hand_side: Vec<Rc<Symbol>>,
        predicate: Option<String>,
        associative_precedence: Option<AssociativePrecedence>,
        action: Option<String>,
    ) -> Self {
        let associative_precedence = if let Some(associative_precedence) = associative_precedence {
            associative_precedence
        } else if let Some(associative_precedence) = rhs_associated_precedence(&right_hand_side) {
            associative_precedence
        } else {
            AssociativePrecedence::default()
        };
        Self {
            right_hand_side,
            predicate,
            action,
            associative_precedence,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Production {
    pub ident: u32,
    left_hand_side: Rc<Symbol>,
    tail: ProductionTail,
}

impl_ident_cmp!(Production);

lazy_static! {
    static ref RHS_CRE: regex::Regex = regex::Regex::new(r"\$(\d+)").unwrap();
}

fn rhs_associated_precedence(symbols: &[Rc<Symbol>]) -> Option<AssociativePrecedence> {
    for symbol in symbols.iter() {
        if symbol.is_token() {
            let associative_precedence = symbol.associative_precedence();
            return Some(associative_precedence);
        }
    }
    None
}

impl Production {
    pub fn new(ident: u32, left_hand_side: Rc<Symbol>, tail: ProductionTail) -> Self {
        Self {
            ident,
            left_hand_side,
            tail,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tail.right_hand_side.len() == 0
    }

    pub fn left_hand_side(&self) -> &Rc<Symbol> {
        &self.left_hand_side
    }

    pub fn right_hand_side_len(&self) -> usize {
        self.tail.right_hand_side.len()
    }

    pub fn right_hand_side_symbols(&self) -> impl Iterator<Item = &Rc<Symbol>> {
        self.tail.right_hand_side.iter()
    }

    pub fn associativity(&self) -> Associativity {
        self.tail.associative_precedence.associativity
    }

    pub fn precedence(&self) -> u32 {
        self.tail.associative_precedence.precedence
    }

    pub fn predicate(&self) -> Option<&String> {
        if let Some(ref string) = self.tail.predicate {
            Some(string)
        } else {
            None
        }
    }

    pub fn expanded_predicate(&self) -> Option<String> {
        if let Some(predicate) = &self.tail.predicate {
            let rhs_len = self.tail.right_hand_side.len();
            let string = RHS_CRE
                .replace_all(&predicate, |caps: &regex::Captures| {
                    format!(
                        "aa_attributes.at_len_minus_n({})",
                        rhs_len + 1 - usize::from_str(&caps[1]).unwrap()
                    )
                })
                .to_string();
            let string = string.replace("$?", "aa_tag");
            Some(string)
        } else {
            None
        }
    }

    pub fn expanded_action(&self) -> Option<String> {
        if let Some(action) = &self.tail.action {
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
        if let Some(symbol) = self.tail.right_hand_side.last() {
            symbol.is_error_symbol()
        } else {
            false
        }
    }

    pub fn as_comment_string(&self) -> String {
        let mut string = format!("{}:", self.left_hand_side.name());
        if self.tail.right_hand_side.len() == 0 {
            string += " <empty>";
        } else {
            for symbol in self.tail.right_hand_side.iter() {
                string += &format!(" {}", symbol);
            }
        };
        if let Some(predicate) = &self.tail.predicate {
            string += &format!(" ?({}?)", predicate);
        };
        string
    }
}

impl std::fmt::Display for Production {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string = format!("{}:", self.left_hand_side);
        if self.tail.right_hand_side.len() == 0 {
            string += " <empty>";
        } else {
            for symbol in self.tail.right_hand_side.iter() {
                string += &format!(" {}", symbol);
            }
        };
        if let Some(predicate) = &self.tail.predicate {
            string += &format!(" ?({}?)", predicate);
        };
        write!(f, "{}", string)
    }
}

#[derive(Debug)]
pub struct Reductions {
    reductions: BTreeMap<BTreeSet<Rc<Production>>, SymbolSet>,
    expected_tokens: SymbolSet,
}

impl Reductions {
    pub fn reductions(&self) -> impl Iterator<Item = (&BTreeSet<Rc<Production>>, &SymbolSet)> {
        self.reductions.iter()
    }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
pub struct GrammarItemKey {
    production: Rc<Production>,
    dot: usize,
}

impl std::fmt::Display for GrammarItemKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string = format!("{}:", self.production.left_hand_side.name());
        if self.production.tail.right_hand_side.len() == 0 {
            string += " . <empty>";
        } else {
            for (index, symbol) in self.production.tail.right_hand_side.iter().enumerate() {
                if index == self.dot {
                    string += &format!(" . {}", symbol);
                } else {
                    string += &format!(" {}", symbol);
                }
            }
            if self.dot >= self.production.tail.right_hand_side.len() {
                string += " . ";
            }
        };
        if let Some(predicate) = &self.production.tail.predicate {
            string += &format!(" ?({}?)", predicate);
        };
        write!(f, "{}", string)
    }
}

impl From<&Rc<Production>> for GrammarItemKey {
    fn from(production: &Rc<Production>) -> Self {
        Self {
            production: Rc::clone(production),
            dot: 0,
        }
    }
}

impl GrammarItemKey {
    pub fn new(production: Rc<Production>) -> Rc<Self> {
        Rc::new(Self { production, dot: 0 })
    }

    pub fn production(&self) -> &Rc<Production> {
        &self.production
    }

    pub fn shifted(&self) -> Rc<Self> {
        let production = Rc::clone(&self.production);
        let dot = self.dot + 1;
        Rc::new(Self { production, dot })
    }

    pub fn is_closable(&self) -> bool {
        if let Some(symbol) = self.production.tail.right_hand_side.get(self.dot) {
            symbol.is_non_terminal()
        } else {
            false
        }
    }

    pub fn is_kernel_item(&self) -> bool {
        self.dot > 0 || self.production.left_hand_side.is_start_symbol()
    }

    pub fn is_reducible(&self) -> bool {
        self.dot >= self.production.tail.right_hand_side.len()
    }

    pub fn next_symbol(&self) -> Option<&Rc<Symbol>> {
        self.production.tail.right_hand_side.get(self.dot)
    }

    pub fn next_symbol_is(&self, symbol: &Rc<Symbol>) -> bool {
        if let Some(next_symbol) = self.next_symbol() {
            next_symbol == symbol
        } else {
            false
        }
    }

    pub fn rhs_tail(&self) -> &[Rc<Symbol>] {
        &self.production.tail.right_hand_side[self.dot + 1..]
    }

    pub fn associativity(&self) -> Associativity {
        self.production.associativity()
    }

    pub fn precedence(&self) -> u32 {
        self.production.precedence()
    }

    pub fn predicate(&self) -> Option<&String> {
        self.production.predicate()
    }

    pub fn has_error_recovery_tail(&self) -> bool {
        self.production.has_error_recovery_tail()
    }

    pub fn has_reducible_error_recovery_tail(&self) -> bool {
        self.is_reducible() && self.production.has_error_recovery_tail()
    }
}

pub struct GrammarItemSet(BTreeMap<Rc<GrammarItemKey>, SymbolSet>);

impl From<BTreeMap<Rc<GrammarItemKey>, SymbolSet>> for GrammarItemSet {
    fn from(key_look_ahead_set_map: BTreeMap<Rc<GrammarItemKey>, SymbolSet>) -> Self {
        Self(key_look_ahead_set_map)
    }
}

pub fn format_set<T: Ord + std::fmt::Display>(set: &BTreeSet<T>) -> String {
    let mut set_string = "Set{".to_string();
    for (index, item) in set.iter().enumerate() {
        if index == 0 {
            set_string += &format!("{}", item);
        } else {
            set_string += &format!(", {}", item);
        }
    }
    set_string += "}";
    set_string
}

impl Index<&Rc<GrammarItemKey>> for GrammarItemSet {
    type Output = SymbolSet;

    fn index(&self, key: &Rc<GrammarItemKey>) -> &SymbolSet {
        self.0.index(key)
    }
}

impl std::fmt::Display for GrammarItemSet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string = "GrammarItemSet{\n".to_string();
        for (key, set) in self.0.iter() {
            string += &format!("    {}: {}\n", key, set);
        }
        string += "}";
        write!(f, "{}", string)
    }
}

impl GrammarItemSet {
    pub fn iter(&self) -> impl Iterator<Item = (&Rc<GrammarItemKey>, &SymbolSet)> {
        self.0.iter()
    }

    pub fn closables(&self) -> Vec<(Rc<GrammarItemKey>, SymbolSet)> {
        let mut closables = vec![];
        for (key, set) in self.0.iter().filter(|x| x.0.is_closable()) {
            closables.push((Rc::clone(key), set.clone()));
        }
        closables
    }

    pub fn generate_goto_kernel(&self, symbol: &Rc<Symbol>) -> GrammarItemSet {
        let mut map = BTreeMap::new();
        for (item_key, look_ahead_set) in self.0.iter() {
            if item_key.next_symbol_is(symbol) {
                map.insert(item_key.shifted(), look_ahead_set.clone());
            }
        }
        GrammarItemSet(map)
    }

    pub fn kernel_keys(&self) -> BTreeSet<Rc<GrammarItemKey>> {
        let mut keys = BTreeSet::new();
        for key in self.0.keys().filter(|x| x.is_kernel_item()) {
            keys.insert(Rc::clone(key));
        }
        keys
    }

    pub fn irreducible_keys(&self) -> BTreeSet<Rc<GrammarItemKey>> {
        self.0
            .keys()
            .filter(|x| !x.is_reducible())
            .cloned()
            .collect()
    }

    pub fn reducible_keys(&self) -> BTreeSet<Rc<GrammarItemKey>> {
        self.0
            .keys()
            .filter(|x| x.is_reducible())
            .cloned()
            .collect()
    }

    pub fn keys(&self) -> BTreeSet<Rc<GrammarItemKey>> {
        self.0.keys().cloned().collect()
    }

    pub fn get_mut(&mut self, key: &Rc<GrammarItemKey>) -> Option<&mut SymbolSet> {
        self.0.get_mut(key)
    }

    pub fn insert(
        &mut self,
        key: Rc<GrammarItemKey>,
        look_ahead_set: SymbolSet,
    ) -> Option<SymbolSet> {
        self.0.insert(key, look_ahead_set)
    }

    pub fn look_ahead_intersection(
        &self,
        key1: &GrammarItemKey,
        key2: &GrammarItemKey,
    ) -> SymbolSet {
        self.0
            .get(key1)
            .unwrap()
            .intersection(self.0.get(key2).unwrap())
            .cloned()
            .collect()
    }

    pub fn remove_look_ahead_symbols(&mut self, key: &GrammarItemKey, symbols: &SymbolSet) {
        let look_ahead_set = self.0.get_mut(key).unwrap();
        *look_ahead_set = look_ahead_set.difference(symbols).cloned().collect();
    }

    pub fn error_recovery_look_ahead_set_contains(&self, token: &Rc<Symbol>) -> bool {
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

    pub fn reducible_look_ahead_set(&self) -> SymbolSet {
        let mut set = SymbolSet::new();
        for (_, look_ahead_set) in self.0.iter().filter(|x| x.0.is_reducible()) {
            set |= look_ahead_set;
        }
        set
    }

    pub fn reductions(&self) -> Reductions {
        let expected_tokens = self.reducible_look_ahead_set();
        let mut reductions: BTreeMap<BTreeSet<Rc<Production>>, SymbolSet> = BTreeMap::new();
        for token in expected_tokens.iter() {
            let mut productions: BTreeSet<Rc<Production>> = BTreeSet::new();
            for (item_key, look_ahead_set) in self.0.iter().filter(|x| x.0.is_reducible()) {
                if look_ahead_set.contains(token) {
                    productions.insert(Rc::clone(&item_key.production));
                }
            }
            let look_ahead_set = reductions.entry(productions).or_insert(SymbolSet::new());
            look_ahead_set.insert(token);
        }
        Reductions {
            reductions,
            expected_tokens,
        }
    }
}
