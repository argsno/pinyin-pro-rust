//! Global mutable state mirroring the JS module singletons:
//! `DICT1` (FastDictFactory), the AC tree, custom dicts, origin-restore map
//! and the traditional-char map.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use crate::data::DICT1_ENTRIES;
use crate::segmentit::{AcAutomaton, DICT_BUILTIN};

// ---------------------------------------------------------------------------
// char dict (FastDictFactory)
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct DictStore {
    singles: HashMap<char, String>,
    strings: HashMap<String, String>,
}

impl DictStore {
    pub fn get(&self, word: &str) -> Option<&String> {
        if word.chars().count() > 1 {
            self.strings.get(word)
        } else {
            word.chars().next().and_then(|c| self.singles.get(&c))
        }
    }

    pub fn set(&mut self, word: &str, pinyin: String) {
        if word.chars().count() > 1 {
            self.strings.insert(word.to_string(), pinyin);
        } else if let Some(c) = word.chars().next() {
            self.singles.insert(c, pinyin);
        }
    }

    pub fn remove(&mut self, word: &str) {
        if word.chars().count() > 1 {
            self.strings.remove(word);
        } else if let Some(c) = word.chars().next() {
            self.singles.remove(&c);
        }
    }

    pub fn clear(&mut self) {
        self.singles.clear();
        self.strings.clear();
    }

    /// JS truthiness: missing and empty string both count as absent.
    pub fn get_truthy(&self, word: &str) -> Option<String> {
        self.get(word)
            .and_then(|s| if s.is_empty() { None } else { Some(s.clone()) })
    }
}

fn build_dict1() -> DictStore {
    let mut d = DictStore::default();
    for (ch, py) in DICT1_ENTRIES {
        if let Some(c) = ch.chars().next() {
            d.singles.insert(c, py.to_string());
        }
    }
    d
}

static DICT1: OnceLock<RwLock<DictStore>> = OnceLock::new();

pub fn dict1() -> &'static RwLock<DictStore> {
    DICT1.get_or_init(|| RwLock::new(build_dict1()))
}

// custom multiple / polyphonic single-char overrides
static CUSTOM_MULTIPLE: OnceLock<RwLock<DictStore>> = OnceLock::new();
static CUSTOM_POLYPHONIC: OnceLock<RwLock<DictStore>> = OnceLock::new();

pub fn custom_multiple() -> &'static RwLock<DictStore> {
    CUSTOM_MULTIPLE.get_or_init(|| RwLock::new(DictStore::default()))
}

pub fn custom_polyphonic() -> &'static RwLock<DictStore> {
    CUSTOM_POLYPHONIC.get_or_init(|| RwLock::new(DictStore::default()))
}

// ---------------------------------------------------------------------------
// AC tree
// ---------------------------------------------------------------------------

static AC: OnceLock<RwLock<AcAutomaton>> = OnceLock::new();

/// Build the initial trie once (built-ins + fail links).
pub fn ensure_ac_built() {
    AC.get_or_init(|| {
        let mut ac = AcAutomaton::new();
        ac.extend_trie(crate::data::builtin_patterns());
        ac.build_fail();
        RwLock::new(ac)
    });
}

pub fn ac() -> &'static RwLock<AcAutomaton> {
    ensure_ac_built();
    AC.get().expect("ac initialized")
}

// ---------------------------------------------------------------------------
// named dict ids (add_dict(name))
// ---------------------------------------------------------------------------

static DICT_IDS: OnceLock<RwLock<DictIdGen>> = OnceLock::new();

struct DictIdGen {
    map: HashMap<String, u64>,
    next: u64,
}

pub fn named_dict_id(name: &str) -> u64 {
    let gen = DICT_IDS.get_or_init(|| {
        RwLock::new(DictIdGen {
            map: HashMap::new(),
            next: 100,
        })
    });
    // Fast path: existing id.
    if let Some(id) = gen.read().unwrap().map.get(name) {
        return *id;
    }
    let mut g = gen.write().unwrap();
    if let Some(id) = g.map.get(name) {
        return *id;
    }
    let id = g.next;
    g.next += 1;
    g.map.insert(name.to_string(), id);
    id
}

pub const DEFAULT_DICT_NAME: &str = "pinyin-pro-default";

// ---------------------------------------------------------------------------
// origin restore map for add_dict single-char overrides
// ---------------------------------------------------------------------------
static ORIGIN_DICT: OnceLock<RwLock<OriginDictMap>> = OnceLock::new();

type OriginDictMap = HashMap<u64, HashMap<String, Option<String>>>;

pub fn origin_dict() -> &'static RwLock<OriginDictMap> {
    ORIGIN_DICT.get_or_init(|| RwLock::new(HashMap::new()))
}

// ---------------------------------------------------------------------------
// customPinyin word map (insertion-ordered, mirroring the JS object)
// ---------------------------------------------------------------------------

static CUSTOM_WORDS: OnceLock<RwLock<Vec<(String, String)>>> = OnceLock::new();

pub fn custom_words() -> &'static RwLock<Vec<(String, String)>> {
    CUSTOM_WORDS.get_or_init(|| RwLock::new(Vec::new()))
}

// ---------------------------------------------------------------------------
// traditional dict
// ---------------------------------------------------------------------------

static TRADITIONAL: OnceLock<RwLock<HashMap<char, char>>> = OnceLock::new();

pub fn traditional() -> &'static RwLock<HashMap<char, char>> {
    TRADITIONAL.get_or_init(|| RwLock::new(HashMap::new()))
}

#[allow(dead_code)]
pub fn builtin_dict_id() -> u64 {
    DICT_BUILTIN
}
