//! Merchant→category rule engine.
//!
//! Rules are stored in the DB (`merchant_rules`); this module is the pure-logic
//! match step. Repos load rules once per import and run them across all parsed
//! transactions. Lower `priority` wins; ties broken by insertion order (the
//! caller is expected to ORDER BY priority ASC).

use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Rule {
    pub pattern: String,
    pub pattern_kind: PatternKind,
    pub category_code: String,
    pub is_business: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternKind {
    Substring,
    Regex,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub category_code: String,
    pub is_business: bool,
}

/// Compiled rule set keyed for one pass over a transaction batch. Regexes are
/// compiled once; substring patterns are pre-lowercased for case-insensitive
/// match against the normalized merchant string.
pub struct CompiledRules {
    compiled: Vec<CompiledRule>,
}

struct CompiledRule {
    needle: Needle,
    category_code: String,
    is_business: bool,
}

enum Needle {
    Substring(String),
    Regex(Regex),
}

impl CompiledRules {
    pub fn compile(rules: &[Rule]) -> Result<Self, regex::Error> {
        let mut compiled = Vec::with_capacity(rules.len());
        for r in rules {
            let needle = match r.pattern_kind {
                PatternKind::Substring => Needle::Substring(r.pattern.to_lowercase()),
                PatternKind::Regex => Needle::Regex(Regex::new(&r.pattern)?),
            };
            compiled.push(CompiledRule {
                needle,
                category_code: r.category_code.clone(),
                is_business: r.is_business,
            });
        }
        Ok(Self { compiled })
    }

    /// Match the first rule that fires against `merchant_normalized`.
    pub fn match_one(&self, merchant_normalized: &str) -> Option<Match> {
        for r in &self.compiled {
            let hit = match &r.needle {
                Needle::Substring(s) => merchant_normalized.contains(s),
                Needle::Regex(re) => re.is_match(merchant_normalized),
            };
            if hit {
                return Some(Match {
                    category_code: r.category_code.clone(),
                    is_business: r.is_business,
                });
            }
        }
        None
    }

    /// Batch-apply: returns parallel vec of `Option<Match>` for each merchant.
    pub fn match_all<'a, I: IntoIterator<Item = &'a str>>(&self, merchants: I) -> Vec<Option<Match>> {
        merchants.into_iter().map(|m| self.match_one(m)).collect()
    }

    /// Indexed batch-apply: returns map of `merchant_normalized → Match`
    /// containing only the merchants that hit a rule. Useful for bulk SQL UPDATE.
    pub fn hit_map<'a, I: IntoIterator<Item = &'a str>>(
        &self,
        merchants: I,
    ) -> HashMap<String, Match> {
        let mut out = HashMap::new();
        for m in merchants {
            if let Some(hit) = self.match_one(m) {
                out.entry(m.to_string()).or_insert(hit);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(pat: &str, cat: &str) -> Rule {
        Rule {
            pattern: pat.into(),
            pattern_kind: PatternKind::Substring,
            category_code: cat.into(),
            is_business: true,
        }
    }

    #[test]
    fn substring_first_match_wins() {
        let rules = CompiledRules::compile(&[r("uber", "travel"), r("uber eats", "meals_50")])
            .unwrap();
        // Caller is responsible for ordering. "uber" comes first so it fires
        // on "uber eats" too — caller's job to put more-specific patterns first.
        let hit = rules.match_one("uber eats").unwrap();
        assert_eq!(hit.category_code, "travel");
    }

    #[test]
    fn regex_pattern_works() {
        let rules = CompiledRules::compile(&[Rule {
            pattern: r"^(chevron|shell|exxon|bp)\b".into(),
            pattern_kind: PatternKind::Regex,
            category_code: "car_truck".into(),
            is_business: true,
        }])
        .unwrap();
        assert_eq!(rules.match_one("chevron 12345").unwrap().category_code, "car_truck");
        assert_eq!(rules.match_one("shell oil").unwrap().category_code, "car_truck");
        assert!(rules.match_one("seashell store").is_none());
    }

    #[test]
    fn no_match_returns_none() {
        let rules = CompiledRules::compile(&[r("staples", "office")]).unwrap();
        assert!(rules.match_one("home depot").is_none());
    }
}
