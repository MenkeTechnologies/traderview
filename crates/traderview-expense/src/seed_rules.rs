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
