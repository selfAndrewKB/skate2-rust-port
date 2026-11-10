//! Completion rules as data: how many players must prove a mission's objectives. One table,
//! read from `coop-rules.json` beside the saves when it exists, else the built-in default
//! (`design.md` §3.4, §7: tutorials `Everyone`, everything else `Anyone`, tuned in play). The
//! objective system asks it at its one evaluation point; nothing else decides.
//!
//! M2 applies `Everyone` and `Anyone`; `Combined`, `BestOf` and `Compete` are named so the
//! table can hold them before M7 applies them.

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rule {
    /// Every player's own copy of the objectives must complete.
    Everyone,
    /// The first player's completion completes the mission for all.
    Anyone,
    /// The players' results add up (M7).
    Combined,
    /// The best single result counts (M7).
    BestOf,
    /// The players compete; the best human result counts, or every human must win (M7).
    Compete { every_human_must_win: bool },
}

impl Rule {
    fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "Everyone" => Self::Everyone,
            "Anyone" => Self::Anyone,
            "Combined" => Self::Combined,
            "BestOf" => Self::BestOf,
            "Compete" => Self::Compete { every_human_must_win: false },
            "CompeteAllWin" => Self::Compete { every_human_must_win: true },
            _ => return None,
        })
    }
}

/// The table: a rule by challenge key, else by the challenge's name prefix, else by its family
/// (`eChallengeTypes`), else the default.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RuleTable {
    pub default: Rule,
    pub families: BTreeMap<i32, Rule>,
    pub prefixes: Vec<(String, Rule)>,
    pub challenges: BTreeMap<u64, Rule>,
    /// Where it came from, for the log.
    pub source: String,
}

impl RuleTable {
    /// The built-in table: the opening tutorial missions (`m_1a_*`) are everyone's, the rest
    /// anyone's. The tutorials' family number is not yet known ([open items]).
    pub fn builtin() -> Self {
        Self {
            default: Rule::Anyone,
            families: BTreeMap::new(),
            prefixes: vec![("m_1a_".to_owned(), Rule::Everyone)],
            challenges: BTreeMap::new(),
            source: "built-in".to_owned(),
        }
    }

    /// `coop-rules.json` beside the saves (`SKATE2_SAVE_ROOT`, else the working directory), or
    /// `SKATE2_COOP_RULES=<file>`; the built-in table when neither exists. A file that does not
    /// parse is reported and the built-in table used, so a typo never splits the consoles
    /// silently.
    pub fn load() -> (Self, Option<String>) {
        let path = match std::env::var_os("SKATE2_COOP_RULES") {
            Some(path) => std::path::PathBuf::from(path),
            None => match std::env::var_os("SKATE2_SAVE_ROOT") {
                Some(root) => std::path::PathBuf::from(root),
                None => std::env::current_dir().unwrap_or_default(),
            }
            .join("coop-rules.json"),
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => match Self::parse(&text) {
                Ok(mut table) => {
                    table.source = path.display().to_string();
                    (table, None)
                }
                Err(error) => (Self::builtin(), Some(format!("{}: {error}; the built-in table is used", path.display()))),
            },
            Err(_) => (Self::builtin(), None),
        }
    }

    /// The rule for a challenge: by key, then name prefix, then family, then the default.
    pub fn rule_for(&self, key: u64, name: &str, family: i32) -> Rule {
        if let Some(&rule) = self.challenges.get(&key) {
            return rule;
        }
        if let Some((_, rule)) = self.prefixes.iter().find(|(prefix, _)| name.starts_with(prefix.as_str())) {
            return *rule;
        }
        self.families.get(&family).copied().unwrap_or(self.default)
    }

    /// One number every console can compare in its log.
    pub fn hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        (&self.default, &self.families, &self.prefixes, &self.challenges).hash(&mut hasher);
        hasher.finish()
    }

    /// `{"default": "Anyone", "families": {"34": "Everyone"}, "prefixes": {"m_1a_": "Everyone"},
    /// "challenges": {"0123456789abcdef": "Everyone"}}`; every key is optional.
    pub fn parse(text: &str) -> Result<Self, String> {
        let value = json::parse(text)?;
        let json::Value::Object(fields) = value else { return Err("the rules file is not an object".into()) };
        let rule_of = |value: &json::Value, what: &str| -> Result<Rule, String> {
            match value {
                json::Value::String(text) => Rule::parse(text).ok_or_else(|| format!("{what}: unknown rule {text:?}")),
                _ => Err(format!("{what}: a rule is a string")),
            }
        };
        let mut table = Self::builtin();
        table.prefixes.clear();
        table.source = "file".to_owned();
        for (key, value) in &fields {
            match key.as_str() {
                "default" => table.default = rule_of(value, "default")?,
                "families" | "prefixes" | "challenges" => {
                    let json::Value::Object(entries) = value else { return Err(format!("{key}: an object")) };
                    for (entry, rule) in entries {
                        let rule = rule_of(rule, entry)?;
                        match key.as_str() {
                            "families" => {
                                table.families.insert(entry.parse().map_err(|_| format!("families: {entry:?} is no number"))?, rule);
                            }
                            "prefixes" => table.prefixes.push((entry.clone(), rule)),
                            _ => {
                                table.challenges.insert(u64::from_str_radix(entry, 16).map_err(|_| format!("challenges: {entry:?} is no hex key"))?, rule);
                            }
                        }
                    }
                }
                other => return Err(format!("unknown field {other:?}")),
            }
        }
        Ok(table)
    }
}

/// The campaign as the challenge world sees it: the table and the players by challenge index.
#[derive(Clone, Debug)]
pub struct CampaignWorld {
    pub rules: RuleTable,
    pub players: Vec<i32>,
}

/// A small JSON reader for the rules file: objects, strings, numbers, booleans and null.
pub mod json {
    use std::collections::BTreeMap;

    #[derive(Clone, Debug, PartialEq)]
    pub enum Value {
        Object(BTreeMap<String, Value>),
        Array(Vec<Value>),
        String(String),
        Number(f64),
        Bool(bool),
        Null,
    }

    pub fn parse(text: &str) -> Result<Value, String> {
        let mut p = Parser { bytes: text.as_bytes(), at: 0 };
        let value = p.value()?;
        p.skip_space();
        if p.at != p.bytes.len() {
            return Err(format!("trailing text at byte {}", p.at));
        }
        Ok(value)
    }

    struct Parser<'a> {
        bytes: &'a [u8],
        at: usize,
    }

    impl Parser<'_> {
        fn skip_space(&mut self) {
            while self.bytes.get(self.at).is_some_and(u8::is_ascii_whitespace) {
                self.at += 1;
            }
        }
        fn peek(&self) -> Option<u8> {
            self.bytes.get(self.at).copied()
        }
        fn expect(&mut self, byte: u8) -> Result<(), String> {
            self.skip_space();
            if self.peek() == Some(byte) {
                self.at += 1;
                Ok(())
            } else {
                Err(format!("expected {:?} at byte {}", byte as char, self.at))
            }
        }
        fn value(&mut self) -> Result<Value, String> {
            self.skip_space();
            match self.peek() {
                Some(b'{') => {
                    self.at += 1;
                    let mut fields = BTreeMap::new();
                    self.skip_space();
                    if self.peek() == Some(b'}') {
                        self.at += 1;
                        return Ok(Value::Object(fields));
                    }
                    loop {
                        self.skip_space();
                        let key = self.string()?;
                        self.expect(b':')?;
                        let value = self.value()?;
                        fields.insert(key, value);
                        self.skip_space();
                        match self.peek() {
                            Some(b',') => self.at += 1,
                            Some(b'}') => {
                                self.at += 1;
                                return Ok(Value::Object(fields));
                            }
                            _ => return Err(format!("expected ',' or '}}' at byte {}", self.at)),
                        }
                    }
                }
                Some(b'[') => {
                    self.at += 1;
                    let mut items = Vec::new();
                    self.skip_space();
                    if self.peek() == Some(b']') {
                        self.at += 1;
                        return Ok(Value::Array(items));
                    }
                    loop {
                        items.push(self.value()?);
                        self.skip_space();
                        match self.peek() {
                            Some(b',') => self.at += 1,
                            Some(b']') => {
                                self.at += 1;
                                return Ok(Value::Array(items));
                            }
                            _ => return Err(format!("expected ',' or ']' at byte {}", self.at)),
                        }
                    }
                }
                Some(b'"') => Ok(Value::String(self.string()?)),
                Some(b't') if self.bytes[self.at..].starts_with(b"true") => {
                    self.at += 4;
                    Ok(Value::Bool(true))
                }
                Some(b'f') if self.bytes[self.at..].starts_with(b"false") => {
                    self.at += 5;
                    Ok(Value::Bool(false))
                }
                Some(b'n') if self.bytes[self.at..].starts_with(b"null") => {
                    self.at += 4;
                    Ok(Value::Null)
                }
                Some(byte) if byte == b'-' || byte.is_ascii_digit() => {
                    let start = self.at;
                    while self.peek().is_some_and(|b| b == b'-' || b == b'+' || b == b'.' || b == b'e' || b == b'E' || b.is_ascii_digit()) {
                        self.at += 1;
                    }
                    let text = std::str::from_utf8(&self.bytes[start..self.at]).map_err(|e| e.to_string())?;
                    text.parse().map(Value::Number).map_err(|_| format!("bad number {text:?}"))
                }
                _ => Err(format!("unexpected text at byte {}", self.at)),
            }
        }
        fn string(&mut self) -> Result<String, String> {
            if self.peek() != Some(b'"') {
                return Err(format!("expected a string at byte {}", self.at));
            }
            self.at += 1;
            let mut out = String::new();
            loop {
                let byte = self.peek().ok_or("unterminated string")?;
                self.at += 1;
                match byte {
                    b'"' => return Ok(out),
                    b'\\' => {
                        let escaped = self.peek().ok_or("unterminated escape")?;
                        self.at += 1;
                        out.push(match escaped {
                            b'n' => '\n',
                            b't' => '\t',
                            b'r' => '\r',
                            b'"' => '"',
                            b'\\' => '\\',
                            b'/' => '/',
                            _ => return Err("unsupported escape".into()),
                        });
                    }
                    _ => {
                        let start = self.at - 1;
                        let mut end = self.at;
                        while end < self.bytes.len() && self.bytes[end] != b'"' && self.bytes[end] != b'\\' {
                            end += 1;
                        }
                        out.push_str(std::str::from_utf8(&self.bytes[start..end]).map_err(|e| e.to_string())?);
                        self.at = end;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_table_asks_everyone_for_the_opening_tutorials() {
        let table = RuleTable::builtin();
        assert_eq!(table.rule_for(1, "m_1a_01_goto_slappys", 7), Rule::Everyone);
        assert_eq!(table.rule_for(2, "m_2b_race", 7), Rule::Anyone);
    }

    #[test]
    fn a_rules_file_overrides_by_key_then_prefix_then_family() {
        let table = RuleTable::parse(
            r#"{"default": "Anyone", "families": {"34": "Everyone"}, "prefixes": {"m_1a_": "Everyone", "m_2": "BestOf"},
                "challenges": {"00000000000000ff": "CompeteAllWin"}}"#,
        )
        .expect("parses");
        assert_eq!(table.rule_for(0xff, "m_1a_x", 34), Rule::Compete { every_human_must_win: true });
        assert_eq!(table.rule_for(1, "m_1a_x", 34), Rule::Everyone);
        assert_eq!(table.rule_for(1, "m_2_x", 34), Rule::BestOf);
        assert_eq!(table.rule_for(1, "other", 34), Rule::Everyone);
        assert_eq!(table.rule_for(1, "other", 35), Rule::Anyone);
        assert_ne!(table.hash(), RuleTable::builtin().hash());
    }

    #[test]
    fn a_bad_rules_file_is_refused_with_a_reason() {
        assert!(RuleTable::parse(r#"{"default": "Sometimes"}"#).unwrap_err().contains("unknown rule"));
        assert!(RuleTable::parse(r#"{"families": {"x": "Anyone"}}"#).unwrap_err().contains("no number"));
        assert!(RuleTable::parse(r#"{"defaults": "Anyone"}"#).unwrap_err().contains("unknown field"));
        assert!(RuleTable::parse(r#"[1,2]"#).is_err());
        assert!(RuleTable::parse(r#"{"default": "Anyone"} x"#).is_err());
    }

    #[test]
    fn the_json_reader_handles_the_shapes_the_file_takes() {
        let value = json::parse(r#" {"a": [1, 2.5, -3e2, true, false, null, "x\"y\n"], "b": {}} "#).expect("parses");
        let json::Value::Object(fields) = value else { panic!() };
        assert_eq!(fields["b"], json::Value::Object(Default::default()));
        let json::Value::Array(items) = &fields["a"] else { panic!() };
        assert_eq!(items[2], json::Value::Number(-300.));
        assert_eq!(items[6], json::Value::String("x\"y\n".into()));
    }
}
