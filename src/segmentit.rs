//! Aho-Corasick tokenizer ported from `lib/common/segmentit/`.
//!
//! Faithful notes:
//! - `match` keeps at most one pattern per trie node per end position (the
//!   first in priority order), plus matches along the fail chain.
//! - `build()` (initial load, `add_dict`, `custom_pinyin`) extends the trie
//!   and then rebuilds fail links for the newly added nodes only; old nodes
//!   keep their existing fail links.

use std::collections::HashMap;

use crate::options::SurnameMode;

// ---------------------------------------------------------------------------
// constants (lib/common/constant.ts)
// ---------------------------------------------------------------------------

pub const PROB_UNKNOWN: f64 = 1e-13;
pub const PROB_RULE: f64 = 1e-12;
pub const PROB_DICT: f64 = 2e-8;
pub const PROB_SURNAME: f64 = 1.0;
pub const PROB_CUSTOM: f64 = 1.0;

pub const PRIO_NORMAL: u8 = 1;
pub const PRIO_SURNAME: u8 = 10;
pub const PRIO_CUSTOM: u8 = 100;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TokenizationAlgorithm {
    ReverseMaxMatch = 1,
    #[default]
    MaxProbability = 2,
    MinTokenization = 3,
}

// ---------------------------------------------------------------------------
// patterns
// ---------------------------------------------------------------------------

/// Shape of the auto-generated static tables.
pub struct RawPattern {
    pub zh: &'static str,
    pub pinyin: &'static str,
    pub prob: f64,
    pub len: usize,
    pub priority: u8,
}

/// Owned pattern inserted into the trie.
#[derive(Clone, Debug)]
pub struct NewPattern {
    pub zh: String,
    pub pinyin: String,
    pub prob: f64,
    pub len: usize,
    pub prio: u8,
    pub dict: u64,
    pub pos: String,
}

#[derive(Clone, Debug)]
pub struct Pattern {
    pub zh_chars: Vec<char>,
    pub pinyin: String,
    pub prob: f64,
    pub len: usize,
    pub prio: u8,
    pub dict: u64,
    pub pos: String,
}

#[derive(Clone, Debug)]
pub struct MatchPattern {
    pub zh: String,
    pub pinyin: String,
    pub prob: f64,
    pub len: usize,
    pub prio: u8,
    pub dict: u64,
    pub pos: String,
    pub index: usize,
}

// dict ids: built-ins share one id (never removed by name)
pub const DICT_BUILTIN: u64 = 0;
pub const DICT_DEFAULT: u64 = 1;
pub const DICT_CUSTOM: u64 = 2;

// ---------------------------------------------------------------------------
// trie
// ---------------------------------------------------------------------------

struct Node {
    children: HashMap<char, usize>,
    fail: Option<usize>,
    patterns: Vec<usize>, // indices into AcAutomaton::patterns, priority-ordered
    parent: usize,
    key: char,
}

pub struct AcAutomaton {
    nodes: Vec<Node>,
    patterns: Vec<Pattern>,
    /// New node indices by depth since the last fail build (mirrors `queues`).
    queues: Vec<Vec<usize>>,
}

impl Default for AcAutomaton {
    fn default() -> Self {
        Self::new()
    }
}

impl AcAutomaton {
    pub fn new() -> Self {
        AcAutomaton {
            nodes: vec![Node {
                children: HashMap::new(),
                fail: None,
                patterns: Vec::new(),
                parent: 0,
                key: '\0',
            }],
            patterns: Vec::new(),
            queues: Vec::new(),
        }
    }

    fn queue_node(&mut self, idx: usize, depth: usize) {
        if self.queues.len() <= depth {
            self.queues.resize(depth + 1, Vec::new());
        }
        self.queues[depth].push(idx);
    }

    /// Insert patterns into the trie (no fail-link updates).
    pub fn extend_trie(&mut self, items: Vec<NewPattern>) {
        for item in items {
            let zh_chars: Vec<char> = item.zh.chars().collect();
            let len = if item.len == 0 {
                zh_chars.len()
            } else {
                item.len
            };
            let pat_idx = self.patterns.len();
            self.patterns.push(Pattern {
                zh_chars: zh_chars.clone(),
                pinyin: item.pinyin,
                prob: item.prob,
                len,
                prio: item.prio,
                dict: item.dict,
                pos: item.pos,
            });
            let mut cur = 0;
            for (depth0, &c) in zh_chars.iter().enumerate() {
                let depth = depth0 + 1;
                let next = if let Some(&n) = self.nodes[cur].children.get(&c) {
                    n
                } else {
                    let n = self.nodes.len();
                    self.nodes.push(Node {
                        children: HashMap::new(),
                        fail: None,
                        patterns: Vec::new(),
                        parent: cur,
                        key: c,
                    });
                    self.nodes[cur].children.insert(c, n);
                    self.queue_node(n, depth);
                    n
                };
                cur = next;
            }
            insert_sorted(&mut self.nodes[cur].patterns, &self.patterns, pat_idx);
        }
    }

    /// Build fail links for nodes queued since the last call, shallowest
    /// first. Called once for the initial load and after every `add_dict` /
    /// `custom_pinyin` (mirrors `build()` = buildTrie + buildFailPointer).
    /// Old nodes keep their existing fail links.
    pub fn build_fail(&mut self) {
        let queues = std::mem::take(&mut self.queues);
        for level in queues {
            for n in level {
                let (parent, key) = (self.nodes[n].parent, self.nodes[n].key);
                let mut f = self.nodes[parent].fail;
                loop {
                    match f {
                        None => {
                            self.nodes[n].fail = Some(0);
                            break;
                        }
                        Some(fidx) => {
                            if let Some(&next) = self.nodes[fidx].children.get(&key) {
                                self.nodes[n].fail = Some(next);
                                break;
                            }
                            f = self.nodes[fidx].fail;
                        }
                    }
                }
            }
        }
    }

    /// Remove every pattern registered under `dict`.
    pub fn remove_dict(&mut self, dict: u64) {
        for node in self.nodes.iter_mut() {
            node.patterns.retain(|&p| self.patterns[p].dict != dict);
        }
    }

    fn surname_ok(prio: u8, len: usize, end: usize, surname: SurnameMode) -> bool {
        match surname {
            SurnameMode::Off => prio != PRIO_SURNAME,
            // `item.length - 1 - i === 0`: match must start at text head.
            SurnameMode::Head => len == end + 1,
            SurnameMode::All => true,
        }
    }

    fn emit(&self, node_idx: usize, end: usize, surname: SurnameMode, out: &mut Vec<MatchPattern>) {
        let node = &self.nodes[node_idx];
        if let Some(p) = node
            .patterns
            .iter()
            .map(|&pi| &self.patterns[pi])
            .find(|p| Self::surname_ok(p.prio, p.len, end, surname))
        {
            out.push(MatchPattern {
                zh: p.zh_chars.iter().collect(),
                pinyin: p.pinyin.clone(),
                prob: p.prob,
                len: p.len,
                prio: p.prio,
                dict: p.dict,
                pos: p.pos.clone(),
                index: end + 1 - p.len,
            });
        }
    }

    /// Raw AC scan. `chars` are already traditional-mapped when applicable.
    pub fn match_text(&self, chars: &[char], surname: SurnameMode) -> Vec<MatchPattern> {
        let mut out = Vec::new();
        let mut cur = 0usize;
        for (i, &c) in chars.iter().enumerate() {
            let mut next = self.nodes[cur].children.get(&c).copied();
            while next.is_none() && cur != 0 {
                match self.nodes[cur].fail {
                    None => {
                        cur = 0;
                        break;
                    }
                    Some(f) => {
                        cur = f;
                        next = self.nodes[cur].children.get(&c).copied();
                    }
                }
            }
            if let Some(n) = next {
                cur = n;
                self.emit(n, i, surname, &mut out);
                let mut f = self.nodes[n].fail;
                while let Some(ff) = f {
                    self.emit(ff, i, surname, &mut out);
                    f = self.nodes[ff].fail;
                }
            }
        }
        out
    }

    pub fn search(
        &self,
        chars: &[char],
        surname: SurnameMode,
        algo: TokenizationAlgorithm,
    ) -> Vec<MatchPattern> {
        let patterns = self.match_text(chars, surname);
        match algo {
            TokenizationAlgorithm::ReverseMaxMatch => reverse_max_match(&patterns),
            TokenizationAlgorithm::MinTokenization => min_tokenization(&patterns, chars.len()),
            TokenizationAlgorithm::MaxProbability => max_probability(&patterns, chars.len()),
        }
    }
}

/// Priority-ordered insert: descending priority, then descending probability;
/// later inserts win ties (mirrors `insertPattern`).
fn insert_sorted(slot: &mut Vec<usize>, patterns: &[Pattern], pat_idx: usize) {
    let (prio, prob) = (patterns[pat_idx].prio, patterns[pat_idx].prob);
    // Scan from the end; shift right while the new pattern outranks the
    // existing one, then insert. Net effect equals the JS in-place rotation.
    let mut i = slot.len();
    while i > 0 {
        let q = &patterns[slot[i - 1]];
        if (prio == q.prio && prob >= q.prob) || prio > q.prio {
            i -= 1;
        } else {
            break;
        }
    }
    slot.insert(i, pat_idx);
}

// ---------------------------------------------------------------------------
// max-probability (lib/common/segmentit/max-probability.ts)
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct ProbItem {
    prob: f64,
    decimal: i64,
    pat: Option<usize>,
    next: usize,
}

fn check_decimal(p: &mut ProbItem) {
    if p.prob < 1e-300 {
        p.prob *= 1e300;
        p.decimal += 1;
    }
}

fn pattern_decimal(p: &MatchPattern) -> i64 {
    let l = p.len as i64;
    if p.prio == PRIO_CUSTOM {
        -(l * l * 100)
    } else if p.prio == PRIO_SURNAME {
        -(l * l * 10)
    } else {
        0
    }
}

fn max_of(a: Option<ProbItem>, b: ProbItem) -> ProbItem {
    match a {
        None => b,
        Some(a) => {
            if a.decimal < b.decimal {
                a
            } else if a.decimal == b.decimal {
                if a.prob > b.prob {
                    a
                } else {
                    b
                }
            } else {
                b
            }
        }
    }
}

pub fn max_probability(patterns: &[MatchPattern], length: usize) -> Vec<MatchPattern> {
    let mut dp: Vec<Option<ProbItem>> = vec![None; length];
    let terminal = ProbItem {
        prob: 1.0,
        decimal: 0,
        pat: None,
        next: length,
    };
    let mut p_idx = patterns.len();
    for i in (0..length).rev() {
        let suffix: ProbItem = if i + 1 >= length {
            terminal.clone()
        } else {
            dp[i + 1].clone().expect("dp hole")
        };
        let next_index = if suffix.pat.is_some() {
            i + 1
        } else {
            suffix.next
        };
        while p_idx > 0 && patterns[p_idx - 1].index + patterns[p_idx - 1].len - 1 == i {
            p_idx -= 1;
            let pat = &patterns[p_idx];
            let mut cur = ProbItem {
                prob: pat.prob * suffix.prob,
                decimal: suffix.decimal + pattern_decimal(pat),
                pat: Some(p_idx),
                next: next_index,
            };
            check_decimal(&mut cur);
            let prev = dp[pat.index].take();
            dp[pat.index] = Some(max_of(prev, cur));
        }
        let mut idp = ProbItem {
            prob: PROB_UNKNOWN * suffix.prob,
            decimal: 0,
            pat: None,
            next: next_index,
        };
        check_decimal(&mut idp);
        let prev = dp[i].take();
        dp[i] = Some(max_of(prev, idp));
    }
    let mut out = Vec::new();
    let mut index = 0;
    while index < length {
        let item = dp[index].clone().expect("dp hole");
        if let Some(p) = item.pat {
            out.push(patterns[p].clone());
        }
        index = item.next;
    }
    out
}

// ---------------------------------------------------------------------------
// min-tokenization (lib/common/segmentit/min-tokenization.ts)
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Tok {
    count: i64,
    patterns: Vec<MatchPattern>,
    concat: Option<MatchPattern>,
}

fn min_of(a: Option<Tok>, b: Tok) -> Tok {
    match a {
        None => b,
        Some(a) => {
            if a.count <= b.count {
                a
            } else {
                b
            }
        }
    }
}

fn pattern_count(p: &MatchPattern) -> i64 {
    let l = p.len as i64;
    if p.prio == PRIO_CUSTOM {
        -(l * l * 100000)
    } else if p.prio == PRIO_SURNAME {
        -(l * l * 100)
    } else {
        1
    }
}

pub fn min_tokenization(patterns: &[MatchPattern], length: usize) -> Vec<MatchPattern> {
    let mut dp: Vec<Option<Tok>> = vec![None; length];
    let mut p_idx = patterns.len();
    for i in (0..length).rev() {
        let suffix: Tok = if i + 1 >= length {
            Tok {
                count: 0,
                patterns: Vec::new(),
                concat: None,
            }
        } else {
            dp[i + 1].clone().expect("dp hole")
        };
        while p_idx > 0 && patterns[p_idx - 1].index + patterns[p_idx - 1].len - 1 == i {
            p_idx -= 1;
            let pat = &patterns[p_idx];
            let cur = Tok {
                count: pattern_count(pat) + suffix.count,
                patterns: suffix.patterns.clone(),
                concat: Some(pat.clone()),
            };
            let prev = dp[pat.index].take();
            dp[pat.index] = Some(min_of(prev, cur));
        }
        let idp = Tok {
            count: 1 + suffix.count,
            patterns: suffix.patterns.clone(),
            concat: None,
        };
        let prev = dp[i].take();
        dp[i] = Some(min_of(prev, idp));
        if dp[i].as_ref().unwrap().concat.is_some() {
            let c = dp[i].as_mut().unwrap().concat.take().unwrap();
            dp[i].as_mut().unwrap().patterns.push(c);
        }
    }
    let mut res = dp[0].clone().expect("dp hole").patterns;
    res.reverse();
    res
}

// ---------------------------------------------------------------------------
// reverse-max-match (lib/common/segmentit/reverse-max-match.ts)
// ---------------------------------------------------------------------------

fn ignorable(cur: &MatchPattern, pre: &MatchPattern) -> bool {
    if pre.index + pre.len <= cur.index {
        return false;
    }
    if pre.prio > cur.prio {
        return false;
    }
    if pre.prio == cur.prio && pre.len > cur.len {
        return false;
    }
    true
}

pub fn reverse_max_match(patterns: &[MatchPattern]) -> Vec<MatchPattern> {
    let mut out = Vec::new();
    let mut i = patterns.len() as isize - 1;
    while i >= 0 {
        let index = patterns[i as usize].index;
        let mut j = i - 1;
        while j >= 0 && ignorable(&patterns[i as usize], &patterns[j as usize]) {
            j -= 1;
        }
        if j < 0 || patterns[j as usize].index + patterns[j as usize].len <= index {
            out.push(patterns[i as usize].clone());
        }
        i = j;
    }
    out.reverse();
    out
}
