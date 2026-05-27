//! Symbol allow/blocklist enforcer.
//!
//! Compliance / personal-trading-restrictions filter:
//!   - **Allowlist mode**: only listed symbols permitted.
//!   - **Blocklist mode**: listed symbols blocked, all others allowed.
//!
//! Supports glob/prefix matching (e.g. "TSLA*" matches TSLA, TSLAQ
//! options). Use cases: insider-restricted lists, company-policy
//! blocked sectors, trader's "never trade these" personal list.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterMode {
    Allowlist,
    Blocklist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolFilter {
    pub mode: FilterMode,
    /// Patterns; trailing '*' = prefix match, otherwise exact match.
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterDecision {
    Permit,
    Block,
}

impl SymbolFilter {
    pub fn check(&self, symbol: &str) -> FilterDecision {
        let matched = self.patterns.iter().any(|p| matches(p, symbol));
        match (self.mode, matched) {
            (FilterMode::Allowlist, true) => FilterDecision::Permit,
            (FilterMode::Allowlist, false) => FilterDecision::Block,
            (FilterMode::Blocklist, true) => FilterDecision::Block,
            (FilterMode::Blocklist, false) => FilterDecision::Permit,
        }
    }
}

fn matches(pattern: &str, symbol: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        symbol.starts_with(prefix)
    } else {
        symbol == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allow(patterns: &[&str]) -> SymbolFilter {
        SymbolFilter {
            mode: FilterMode::Allowlist,
            patterns: patterns.iter().map(|s| s.to_string()).collect(),
        }
    }
    fn block(patterns: &[&str]) -> SymbolFilter {
        SymbolFilter {
            mode: FilterMode::Blocklist,
            patterns: patterns.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn allowlist_matched_symbol_permits() {
        let f = allow(&["AAPL"]);
        assert_eq!(f.check("AAPL"), FilterDecision::Permit);
    }

    #[test]
    fn allowlist_unmatched_symbol_blocks() {
        let f = allow(&["AAPL"]);
        assert_eq!(f.check("TSLA"), FilterDecision::Block);
    }

    #[test]
    fn blocklist_matched_symbol_blocks() {
        let f = block(&["TSLA"]);
        assert_eq!(f.check("TSLA"), FilterDecision::Block);
    }

    #[test]
    fn blocklist_unmatched_symbol_permits() {
        let f = block(&["TSLA"]);
        assert_eq!(f.check("AAPL"), FilterDecision::Permit);
    }

    #[test]
    fn empty_allowlist_blocks_everything() {
        let f = allow(&[]);
        assert_eq!(f.check("AAPL"), FilterDecision::Block);
    }

    #[test]
    fn empty_blocklist_permits_everything() {
        let f = block(&[]);
        assert_eq!(f.check("AAPL"), FilterDecision::Permit);
    }

    #[test]
    fn prefix_glob_matches_multiple_symbols() {
        let f = block(&["TSLA*"]);
        assert_eq!(f.check("TSLA"), FilterDecision::Block);
        assert_eq!(f.check("TSLAQ"), FilterDecision::Block);
        assert_eq!(f.check("TSLY"), FilterDecision::Permit);
    }

    #[test]
    fn exact_match_does_not_glob() {
        let f = block(&["TSLA"]);
        // No trailing star → exact only. TSLAQ permitted.
        assert_eq!(f.check("TSLAQ"), FilterDecision::Permit);
    }

    #[test]
    fn multiple_patterns_any_match_counts() {
        let f = allow(&["AAPL", "MSFT", "GOOGL"]);
        assert_eq!(f.check("MSFT"), FilterDecision::Permit);
        assert_eq!(f.check("META"), FilterDecision::Block);
    }
}
