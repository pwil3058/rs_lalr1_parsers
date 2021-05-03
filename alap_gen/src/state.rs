use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    fmt,
    io::Write,
    rc::Rc,
    str::FromStr,
};

use crate::symbols::{format_as_or_list, AssociativePrecedence, Associativity, Symbol, SymbolSet};

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

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
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

#[derive(Debug)]
struct Reductions {
    reductions: BTreeMap<BTreeSet<Rc<Production>>, SymbolSet>,
    expected_tokens: SymbolSet,
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
        for (_, look_ahead_set) in self
            .0
            .iter()
            .filter(|x| x.0.has_reducible_error_recovery_tail())
        {
            if look_ahead_set.contains(token) {
                return true;
            }
        }
        false
    }

    fn reducible_look_ahead_set(&self) -> SymbolSet {
        let mut set = SymbolSet::new();
        for (_, look_ahead_set) in self.0.iter().filter(|x| x.0.is_reducible()) {
            set |= look_ahead_set;
        }
        set
    }

    fn reductions(&self) -> Reductions {
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
            look_ahead_set.insert(Rc::clone(token));
        }
        Reductions {
            reductions,
            expected_tokens,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ProcessedState {
    Unprocessed,
    NeedsReprocessing,
    Processed,
}

pub struct ParserState {
    pub ident: u32,
    grammar_items: RefCell<GrammarItemSet>,
    shift_list: RefCell<BTreeMap<Rc<Symbol>, Rc<ParserState>>>,
    goto_table: RefCell<BTreeMap<Rc<Symbol>, Rc<ParserState>>>,
    error_recovery_state: RefCell<Option<Rc<ParserState>>>,
    processed_state: Cell<ProcessedState>,
    shift_reduce_conflicts:
        RefCell<Vec<(Rc<Symbol>, Rc<ParserState>, Rc<GrammarItemKey>, SymbolSet)>>,
    reduce_reduce_conflicts: RefCell<Vec<((Rc<GrammarItemKey>, Rc<GrammarItemKey>), SymbolSet)>>,
}

impl fmt::Debug for ParserState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "State#{}({:?}):",
            self.ident,
            self.grammar_items.borrow().keys()
        )
    }
}

impl_ident_cmp!(ParserState);

impl ParserState {
    pub fn new(ident: u32, grammar_items: GrammarItemSet) -> Rc<Self> {
        Rc::new(Self {
            ident,
            grammar_items: RefCell::new(grammar_items),
            shift_list: RefCell::new(BTreeMap::new()),
            goto_table: RefCell::new(BTreeMap::new()),
            error_recovery_state: RefCell::new(None),
            processed_state: Cell::new(ProcessedState::Unprocessed),
            shift_reduce_conflicts: RefCell::new(vec![]),
            reduce_reduce_conflicts: RefCell::new(vec![]),
        })
    }

    pub fn is_processed(&self) -> bool {
        match self.processed_state.get() {
            ProcessedState::Processed => true,
            _ => false,
        }
    }

    pub fn is_unprocessed(&self) -> bool {
        match self.processed_state.get() {
            ProcessedState::Unprocessed => true,
            _ => false,
        }
    }

    pub fn mark_as_processed(&self) {
        self.processed_state.set(ProcessedState::Processed)
    }

    pub fn merge_lookahead_sets(&self, item_set: &GrammarItemSet) {
        let mut additions = 0;
        for (key, other_look_ahead_set) in item_set.iter().filter(|(k, _)| k.is_kernel_item()) {
            if let Some(look_ahead_set) = self.grammar_items.borrow_mut().get_mut(key) {
                let current_len = look_ahead_set.len();
                *look_ahead_set |= other_look_ahead_set;
                additions += look_ahead_set.len() - current_len;
            } else {
                panic!("key sets should be identical to get here")
            }
        }
        if additions > 0 && self.is_processed() {
            self.processed_state.set(ProcessedState::NeedsReprocessing);
        }
    }

    pub fn add_shift_action(&self, token: Rc<Symbol>, state: Rc<ParserState>) {
        self.shift_list.borrow_mut().insert(token, state);
    }

    pub fn add_goto(&self, token: Rc<Symbol>, state: Rc<ParserState>) {
        self.goto_table.borrow_mut().insert(token, state);
    }

    pub fn set_error_recovery_state(&self, state: &Rc<ParserState>) {
        *self.error_recovery_state.borrow_mut() = Some(Rc::clone(state));
    }

    pub fn error_goto_state_ident(&self) -> Option<u32> {
        if let Some(error_recovery_state) = self.error_recovery_state.borrow().clone() {
            Some(error_recovery_state.ident)
        } else {
            None
        }
    }

    pub fn kernel_keys(&self) -> BTreeSet<Rc<GrammarItemKey>> {
        self.grammar_items.borrow().kernel_keys()
    }

    pub fn non_kernel_keys(&self) -> BTreeSet<Rc<GrammarItemKey>> {
        self.grammar_items.borrow().irreducible_keys()
    }

    pub fn generate_goto_kernel(&self, symbol: &Rc<Symbol>) -> GrammarItemSet {
        self.grammar_items.borrow().generate_goto_kernel(symbol)
    }

    pub fn resolve_shift_reduce_conflicts(&self) -> usize {
        // do this in two stages to avoid borrow/access conflicts
        let mut conflicts = vec![];
        for (shift_symbol, goto_state) in self.shift_list.borrow().iter() {
            for (item, look_ahead_set) in self.grammar_items.borrow().iter() {
                if item.is_reducible() && look_ahead_set.contains(shift_symbol) {
                    conflicts.push((
                        Rc::clone(shift_symbol),
                        Rc::clone(goto_state),
                        Rc::clone(item),
                        look_ahead_set.clone(),
                    ))
                }
            }
        }
        let mut shift_reduce_conflicts = self.shift_reduce_conflicts.borrow_mut();
        let mut shift_list = self.shift_list.borrow_mut();
        let mut grammar_items = self.grammar_items.borrow_mut();
        for (shift_symbol, goto_state, reducible_item, look_ahead_set) in conflicts.iter() {
            if shift_symbol.precedence() < reducible_item.precedence() {
                shift_list.remove(shift_symbol);
            } else if shift_symbol.precedence() > reducible_item.precedence() {
                grammar_items
                    .get_mut(&Rc::clone(reducible_item))
                    .unwrap()
                    .remove(shift_symbol);
            } else if reducible_item.associativity() == Associativity::Left {
                shift_list.remove(shift_symbol);
            } else if reducible_item.has_error_recovery_tail() {
                grammar_items
                    .get_mut(&Rc::clone(reducible_item))
                    .unwrap()
                    .remove(shift_symbol);
            } else {
                // Default: resolve in favour of shift but mark as unresolved
                // to give the user the option of accepting this resolution
                grammar_items
                    .get_mut(&Rc::clone(reducible_item))
                    .unwrap()
                    .remove(shift_symbol);
                shift_reduce_conflicts.push((
                    Rc::clone(shift_symbol),
                    Rc::clone(goto_state),
                    Rc::clone(reducible_item),
                    look_ahead_set.clone(),
                ))
            }
        }
        shift_reduce_conflicts.len()
    }

    pub fn resolve_reduce_reduce_conflicts(&self) -> usize {
        // TODO: think about moving reduce/reduce conflict resolution inside GrammarItemSet
        let reducible_key_set = self.grammar_items.borrow().reducible_keys();
        if reducible_key_set.len() < 2 {
            return 0;
        }

        let mut reduce_reduce_conflicts = self.reduce_reduce_conflicts.borrow_mut();
        let reducible_key_set_2 = reducible_key_set.clone();
        for key_1 in reducible_key_set.iter() {
            for key_2 in reducible_key_set_2.iter() {
                if key_2 > key_1 {
                    let intersection = self
                        .grammar_items
                        .borrow()
                        .look_ahead_intersection(key_1, key_2);
                    if intersection.len() > 0 && key_1.predicate().is_none() {
                        if key_1.has_error_recovery_tail() {
                            self.grammar_items
                                .borrow_mut()
                                .remove_look_ahead_symbols(key_1, &intersection);
                        } else if key_2.has_error_recovery_tail() {
                            self.grammar_items
                                .borrow_mut()
                                .remove_look_ahead_symbols(key_2, &intersection);
                        } else {
                            // Default: resolve in favour of first declared production
                            // but mark unresolved to give the user some options
                            self.grammar_items
                                .borrow_mut()
                                .remove_look_ahead_symbols(key_2, &intersection);
                            reduce_reduce_conflicts
                                .push(((Rc::clone(key_1), Rc::clone(key_2)), intersection))
                        }
                    }
                }
            }
        }
        reduce_reduce_conflicts.len()
    }

    pub fn is_recovery_state_for_token(&self, token: &Rc<Symbol>) -> bool {
        if let Some(recovery_state) = self.error_recovery_state.borrow().clone() {
            if recovery_state
                .grammar_items
                .borrow()
                .error_recovery_look_ahead_set_contains(token)
            {
                return true;
            }
        };
        false
    }

    pub fn look_ahead_set(&self) -> SymbolSet {
        self.grammar_items
            .borrow()
            .reducible_look_ahead_set()
            .union(&self.shift_list.borrow().keys().cloned().collect())
            .cloned()
            .collect()
    }

    pub fn write_next_action_code<W: Write>(
        &self,
        wtr: &mut W,
        indent: &str,
    ) -> std::io::Result<()> {
        let reductions = self.grammar_items.borrow().reductions();
        wtr.write_fmt(format_args!(
            "{}{} => match aa_tag {{\n",
            indent, self.ident
        ))?;
        for (token, state) in self.shift_list.borrow().iter() {
            wtr.write_fmt(format_args!(
                "{}    {} => Action::Shift({}),\n",
                indent,
                token.name(),
                state.ident
            ))?;
        }
        for (productions, look_ahead_set) in reductions.reductions.iter() {
            if productions.len() == 1 {
                let production = productions.iter().next().expect("len() == 1");
                debug_assert!(production.predicate().is_none());
                wtr.write_fmt(format_args!(
                    "{}    // {}\n",
                    indent,
                    production.as_comment_string()
                ))?;
                if production.ident == 0 {
                    wtr.write_fmt(format_args!(
                        "{}    {} => Action::Accept,\n",
                        indent,
                        format_as_or_list(&look_ahead_set),
                    ))?;
                } else {
                    wtr.write_fmt(format_args!(
                        "{}    {} => Action::Reduce({}),\n",
                        indent,
                        format_as_or_list(&look_ahead_set),
                        production.ident,
                    ))?;
                }
            } else {
                wtr.write_fmt(format_args!(
                    "{}    {} => {{\n",
                    indent,
                    format_as_or_list(&look_ahead_set)
                ))?;
                for (i, production) in productions.iter().enumerate() {
                    if i == 0 {
                        wtr.write_fmt(format_args!(
                            "{}        if {} {{\n",
                            indent,
                            production.expanded_predicate().expect("more than one")
                        ))?;
                    } else if production.predicate().is_some() {
                        wtr.write_fmt(format_args!(
                            "{}        }} else if {} {{\n",
                            indent,
                            production.expanded_predicate().expect("more than one")
                        ))?;
                    } else {
                        wtr.write_fmt(format_args!("{}        }} else {{\n", indent,))?;
                    }
                    wtr.write_fmt(format_args!(
                        "{}            // {}\n",
                        indent,
                        production.as_comment_string()
                    ))?;
                    if production.ident == 0 {
                        wtr.write_fmt(format_args!("{}            Action::Accept\n", indent,))?;
                    } else {
                        wtr.write_fmt(format_args!(
                            "{}            Action::Reduce({})\n",
                            indent, production.ident,
                        ))?;
                    }
                }
                wtr.write_fmt(format_args!("{}        }}\n", indent))?;
                wtr.write_fmt(format_args!("{}    }}\n", indent))?;
            }
        }
        wtr.write_fmt(format_args!("{}    _ => Action::SyntaxError,\n", indent,))?;
        wtr.write_fmt(format_args!("{}}},\n", indent))?;
        Ok(())
    }

    pub fn write_goto_table_code<W: Write>(
        &self,
        wtr: &mut W,
        indent: &str,
    ) -> std::io::Result<()> {
        if self.goto_table.borrow().len() > 0 {
            wtr.write_fmt(format_args!("{}{} => match lhs {{\n", indent, self.ident))?;
            for (symbol, state) in self.goto_table.borrow().iter() {
                wtr.write_fmt(format_args!(
                    "{}    AANonTerminal::{} => {},\n",
                    indent, symbol, state.ident
                ))?;
            }
            wtr.write_fmt(format_args!(
                "{}    _ => panic!(\"Malformed goto table: ({{}}, {{}})\", lhs, current_state),\n",
                indent
            ))?;
            wtr.write_fmt(format_args!("{}}},\n", indent))?;
        };
        Ok(())
    }

    pub fn description(&self) -> String {
        let mut string = format!("State<{}>:\n  Grammar Items:\n", self.ident);
        for (key, look_ahead_set) in self.grammar_items.borrow().iter() {
            string += &format!("    {}: {}\n", key, look_ahead_set);
        }
        string += "  Parser Action Table:\n";
        let look_ahead_set = self.look_ahead_set();
        if look_ahead_set.len() == 0 {
            string += "    <empty>\n";
        } else {
            for token in look_ahead_set.iter() {
                if let Some(state) = self.shift_list.borrow().get(token) {
                    string += &format!("    {}: shift: -> State<{}>\n", token, state.ident);
                } else {
                    for (key, lahs) in self
                        .grammar_items
                        .borrow()
                        .0
                        .iter()
                        .filter(|x| x.0.is_reducible())
                    {
                        if lahs.contains(token) {
                            string += &format!("    {}: reduce: {}\n", token, key.production);
                        }
                    }
                }
            }
        }
        string += "  Go To Table:\n";
        if self.goto_table.borrow().len() == 0 {
            string += "    <empty>\n";
        } else {
            for (symbol, state) in self.goto_table.borrow().iter() {
                string += &format!("    {} -> State<{}>\n", symbol, state.ident);
            }
        }
        if let Some(ref state) = self.error_recovery_state.borrow().clone() {
            string += &format!("  Error Recovery State: State<{}>\n", state.ident);
            string += &format!("    Look Ahead: {}\n", state.look_ahead_set());
        } else {
            string += "  Error Recovery State: <none>\n";
        }
        if self.shift_reduce_conflicts.borrow().len() > 0 {
            string += "  Shift/Reduce Conflicts:\n";
            for (shift_symbol, goto_state, reducible_item, look_ahead_set) in
                self.shift_reduce_conflicts.borrow().iter()
            {
                string += &format!("    {}:\n", shift_symbol);
                string += &format!("      shift -> State<{}>\n", goto_state.ident);
                string += &format!(
                    "      reduce {}: {}",
                    reducible_item.production, look_ahead_set
                );
            }
        }
        if self.reduce_reduce_conflicts.borrow().len() > 0 {
            string += "  Reduce/Reduce Conflicts:\n";
            for (items, intersection) in self.reduce_reduce_conflicts.borrow().iter() {
                string += &format!("    {}\n", intersection);
                string += &format!(
                    "      reduce {} : {}\n",
                    items.0,
                    &self.grammar_items.borrow().0[&Rc::clone(&items.0)]
                );
                string += &format!(
                    "      reduce {} : {}\n",
                    items.1,
                    &self.grammar_items.borrow().0[&Rc::clone(&items.1)]
                );
            }
        }
        string
    }
}
