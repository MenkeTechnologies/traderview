//! Default merchant→category rules shipped on first run.
//!
//! Conservative list — covers the recurring merchants that show up across
//! most US business expense reports. Users can edit, disable, or add their
//! own via the rules manager. Substring patterns match against
//! `merchant_normalized` (already lowercased + stripped of processor prefixes).
//!
//! Order matters: more-specific patterns must precede their less-specific
//! parents (e.g. "uber eats" before "uber"), since the rule engine returns
//! the first hit.

use crate::rules::{PatternKind, Rule};

pub fn seed() -> Vec<Rule> {
    let s = |pat: &str, cat: &str| Rule {
        pattern: pat.into(),
        pattern_kind: PatternKind::Substring,
        category_code: cat.into(),
        is_business: true,
    };

    vec![
        // --- meals (specific) before travel ---
        s("uber eats", "meals_50"),
        s("doordash", "meals_50"),
        s("grubhub", "meals_50"),
        s("postmates", "meals_50"),
        s("starbucks", "meals_50"),
        s("blue bottle", "meals_50"),
        s("chipotle", "meals_50"),

        // --- travel ---
        s("uber", "travel"),
        s("lyft", "travel"),
        s("united.com", "travel"),
        s("united airlines", "travel"),
        s("delta air", "travel"),
        s("american air", "travel"),
        s("southwest air", "travel"),
        s("alaska air", "travel"),
        s("jetblue", "travel"),
        s("airbnb", "travel"),
        s("marriott", "travel"),
        s("hilton", "travel"),
        s("hyatt", "travel"),

        // --- car / fuel ---
        s("chevron", "car_truck"),
        s("shell oil", "car_truck"),
        s("shell service", "car_truck"),
        s("76 gas", "car_truck"),
        s("exxon", "car_truck"),
        s("mobil", "car_truck"),
        s("bp gas", "car_truck"),
        s("valero", "car_truck"),
        s("arco", "car_truck"),

        // --- office / software ---
        s("amzn mktp", "office"),
        s("amazon mktp", "office"),
        s("amazon.com", "office"),
        s("staples", "office"),
        s("office depot", "office"),
        s("officemax", "office"),
        s("adobe", "office"),
        s("jetbrains", "office"),
        s("github", "office"),
        s("digitalocean", "office"),
        s("linode", "office"),
        s("aws", "office"),
        s("amazon web", "office"),
        s("anthropic", "office"),
        s("openai", "office"),
        s("google workspace", "office"),
        s("google cloud", "office"),
        s("cursor.com", "office"),
        s("notion", "office"),

        // --- shipping (office expense per Schedule C; UPS/FedEx of business pkgs) ---
        s("usps", "office"),
        s("ups ", "office"),
        s("fedex", "office"),

        // --- utilities ---
        s("comcast", "utilities"),
        s("xfinity", "utilities"),
        s("at&t", "utilities"),
        s("verizon", "utilities"),
        s("t-mobile", "utilities"),
        s("pg&e", "utilities"),
        s("sce ", "utilities"),
        s("sdg&e", "utilities"),

        // --- legal / professional ---
        s("legalzoom", "legal"),
        s("rocketlawyer", "legal"),

        // --- advertising ---
        s("google ads", "advertising"),
        s("facebook ads", "advertising"),
        s("meta platforms", "advertising"),
        s("linkedin", "advertising"),

        // --- insurance ---
        s("geico", "insurance"),
        s("state farm", "insurance"),
        s("allstate", "insurance"),
        s("progressive", "insurance"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Categories that the shipped rules MAY emit. Adding a new category
    /// here is fine; emitting one not in this set fails the test so the
    /// frontend's category picker doesn't quietly desync.
    const KNOWN_CATEGORIES: &[&str] = &[
        "meals_50", "travel", "car_truck", "office", "utilities",
        "legal", "advertising", "insurance",
    ];

    #[test]
    fn every_seed_rule_uses_a_known_category() {
        let known: HashSet<&str> = KNOWN_CATEGORIES.iter().copied().collect();
        for r in seed() {
            assert!(
                known.contains(r.category_code.as_str()),
                "rule for pattern `{}` uses unknown category `{}` — add to KNOWN_CATEGORIES or fix the typo",
                r.pattern, r.category_code,
            );
        }
    }

    #[test]
    fn seed_rules_all_default_to_business() {
        // The seed is a *business* expense ruleset; nothing personal should
        // ship by default.
        for r in seed() {
            assert!(r.is_business,
                "non-business default rule slipped in: `{}` → `{}`",
                r.pattern, r.category_code);
        }
    }

    #[test]
    fn no_duplicate_patterns() {
        // First-match-wins; a duplicate is dead code and a confusion source.
        let mut seen: HashSet<String> = HashSet::new();
        for r in seed() {
            assert!(seen.insert(r.pattern.clone()),
                "duplicate pattern `{}` — second occurrence is unreachable",
                r.pattern);
        }
    }

    #[test]
    fn specific_meals_precede_generic_uber() {
        // "uber eats" → meals_50 must be encountered BEFORE bare "uber" →
        // travel, or every Uber Eats charge gets misclassified as travel.
        let rules = seed();
        let eats_idx = rules.iter().position(|r| r.pattern == "uber eats")
            .expect("`uber eats` rule missing");
        let uber_idx = rules.iter().position(|r| r.pattern == "uber")
            .expect("`uber` rule missing");
        assert!(eats_idx < uber_idx,
            "rule order broken: `uber eats` must precede `uber`");
    }

    #[test]
    fn substring_kind_is_the_default_for_all_seed_rules() {
        // The substring matcher is the cheapest + matches every shipped
        // rule. If a future entry uses Regex, that requires extra
        // compilation paths — call it out explicitly so it's a conscious
        // decision.
        for r in seed() {
            assert!(matches!(r.pattern_kind, PatternKind::Substring),
                "rule `{}` uses non-Substring kind — verify intentional",
                r.pattern);
        }
    }

    #[test]
    fn patterns_are_lowercase() {
        // Patterns are matched against merchant_normalized which is already
        // lowercased; an uppercase pattern would silently never match.
        for r in seed() {
            assert_eq!(r.pattern, r.pattern.to_lowercase(),
                "uppercase chars in pattern `{}` would never match", r.pattern);
        }
    }

    #[test]
    fn seed_is_non_trivial() {
        // Sanity floor — if someone deletes the whole list by accident,
        // catch it.
        assert!(seed().len() > 20);
    }
}
