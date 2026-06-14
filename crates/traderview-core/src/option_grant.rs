//! Stock option grant (ISO / NSO) — a corporate-equity document distinct from a
//! cap table (ownership snapshot) or a stock subscription (purchase of issued
//! shares). It computes a time-based vesting schedule (a cliff, then monthly
//! vesting over the remaining term) and the exercise economics: shares vested as
//! of a date, the in-the-money spread, and — for an ISO — the AMT preference
//! item created on exercise. Drafting aid, not legal/tax advice.

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct OptionGrantInput {
    pub company_name: String,
    pub optionee_name: String,
    /// Total options granted.
    pub total_options: f64,
    /// Exercise (strike) price per share.
    pub strike_usd: f64,
    /// Current fair market value per share (409A / last round).
    pub fmv_usd: f64,
    /// "ISO" (incentive stock option) or "NSO" (non-qualified).
    #[serde(default)]
    pub option_type: String,
    /// Total vesting term in months (e.g. 48 for a 4-year vest).
    pub vesting_months: u32,
    /// Cliff length in months before any vesting (e.g. 12).
    pub cliff_months: u32,
    pub grant_date: String,
    /// Date to value vesting against.
    pub as_of_date: String,
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OptionGrant {
    pub title: String,
    pub total_options: f64,
    pub months_elapsed: i64,
    pub vested_options: f64,
    pub unvested_options: f64,
    /// In-the-money spread per share, max(fmv - strike, 0).
    pub spread_per_share_usd: f64,
    /// Spread on the currently-vested options.
    pub vested_spread_usd: f64,
    /// Cost to exercise all vested options at the strike price.
    pub vested_exercise_cost_usd: f64,
    /// AMT preference item on exercising vested ISOs (0 for an NSO).
    pub iso_amt_preference_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

/// Whole months from `start` up to and including `end` (0 if end precedes start).
pub fn months_between(start: NaiveDate, end: NaiveDate) -> i64 {
    if end < start {
        return 0;
    }
    let mut months =
        (end.year() - start.year()) * 12 + (end.month() as i32 - start.month() as i32);
    // Not a full month yet if the day-of-month hasn't been reached.
    if end.day() < start.day() {
        months -= 1;
    }
    months.max(0) as i64
}

/// Standard cliff + monthly vest: nothing before the cliff; at/after the cliff,
/// the linearly-accrued (floored) amount, capped at the total at full term.
pub fn vested_shares(total: f64, vesting_months: u32, cliff_months: u32, elapsed: i64) -> f64 {
    if vesting_months == 0 || elapsed < cliff_months as i64 {
        return 0.0;
    }
    if elapsed >= vesting_months as i64 {
        return total;
    }
    (total * elapsed as f64 / vesting_months as f64).floor()
}

pub fn generate(i: &OptionGrantInput) -> OptionGrant {
    let grant = NaiveDate::parse_from_str(&i.grant_date, "%Y-%m-%d");
    let as_of = NaiveDate::parse_from_str(&i.as_of_date, "%Y-%m-%d");
    let elapsed = match (grant, as_of) {
        (Ok(g), Ok(a)) => months_between(g, a),
        _ => 0,
    };

    let vested = vested_shares(i.total_options, i.vesting_months, i.cliff_months, elapsed);
    let unvested = (i.total_options - vested).max(0.0);

    let spread_per_share = (i.fmv_usd - i.strike_usd).max(0.0);
    let vested_spread = cents(spread_per_share * vested);
    let exercise_cost = cents(i.strike_usd * vested);

    let is_iso = i.option_type.trim().eq_ignore_ascii_case("ISO");
    // For an ISO, the bargain element at exercise is an AMT preference item.
    let amt_pref = if is_iso { vested_spread } else { 0.0 };

    let type_label = if is_iso {
        "Incentive Stock Option (ISO)"
    } else {
        "Non-Qualified Stock Option (NSO)"
    };

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("This grant is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This grant is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    let years = i.vesting_months as f64 / 12.0;
    let vesting_body = format!(
        "The Option vests over {} months ({:.2} years) with a {}-month cliff. No options vest before the cliff; thereafter options vest monthly in equal installments. As of {}, {} month(s) have elapsed and {:.0} of {:.0} options are vested ({:.0} unvested).",
        i.vesting_months, years, i.cliff_months, i.as_of_date, elapsed, vested, i.total_options, unvested
    );

    let tax_body = if is_iso {
        format!(
            "Exercising the {:.0} vested ISOs at the strike of {} (cost {}) when fair market value is {} creates a bargain element of {} per share. The total spread of {} is an alternative minimum tax (AMT) preference item in the year of exercise; it is not ordinary income for regular tax if ISO holding periods are met.",
            vested, money(i.strike_usd), money(exercise_cost), money(i.fmv_usd), money(spread_per_share), money(vested_spread)
        )
    } else {
        format!(
            "Exercising the {:.0} vested NSOs at the strike of {} (cost {}) when fair market value is {} produces ordinary income equal to the {} per-share spread, {} in total, taxable and withheld at exercise.",
            vested, money(i.strike_usd), money(exercise_cost), money(i.fmv_usd), money(spread_per_share), money(vested_spread)
        )
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Company: {}\nOptionee: {}\nGrant date: {}",
                i.company_name, i.optionee_name, i.grant_date
            ),
        },
        DocClause {
            heading: "1. Grant".into(),
            body: format!(
                "The Company grants the Optionee an option to purchase {:.0} shares of common stock at an exercise price of {} per share. This option is a {}.",
                i.total_options, money(i.strike_usd), type_label
            ),
        },
        DocClause { heading: "2. Vesting".into(), body: vesting_body },
        DocClause {
            heading: "3. Exercise".into(),
            body: "Vested options may be exercised in whole or in part by written notice and payment of the exercise price. Unvested options are forfeited on termination of service; vested options must be exercised within the post-termination exercise window stated in the plan.".into(),
        },
        DocClause { heading: "4. Tax Treatment".into(), body: tax_body },
        DocClause {
            heading: "5. Transfer Restrictions".into(),
            body: "The option is non-transferable except by will or the laws of descent. Shares acquired on exercise are subject to the Company's right of first refusal and applicable securities-law transfer restrictions.".into(),
        },
        DocClause { heading: "6. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Company: ____________________  Date: __________\n{}\n\nOptionee: ____________________  Date: __________\n{}",
                i.company_name, i.optionee_name
            ),
        },
    ];

    OptionGrant {
        title: format!("Stock Option Grant — {}", type_label),
        total_options: i.total_options,
        months_elapsed: elapsed,
        vested_options: vested,
        unvested_options: unvested,
        spread_per_share_usd: cents(spread_per_share),
        vested_spread_usd: vested_spread,
        vested_exercise_cost_usd: exercise_cost,
        iso_amt_preference_usd: amt_pref,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> OptionGrantInput {
        OptionGrantInput {
            company_name: "Startup Inc".into(),
            optionee_name: "Jane Engineer".into(),
            total_options: 48_000.0,
            strike_usd: 1.00,
            fmv_usd: 5.00,
            option_type: "ISO".into(),
            vesting_months: 48,
            cliff_months: 12,
            grant_date: "2024-01-01".into(),
            as_of_date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn vesting_30_months_in() {
        let g = generate(&base());
        assert_eq!(g.months_elapsed, 30);
        assert!(close(g.vested_options, 30_000.0));
        assert!(close(g.unvested_options, 18_000.0));
    }

    #[test]
    fn pre_cliff_zero() {
        let g = generate(&OptionGrantInput { as_of_date: "2024-07-01".into(), ..base() });
        assert_eq!(g.months_elapsed, 6);
        assert!(close(g.vested_options, 0.0));
    }

    #[test]
    fn at_cliff_25_percent() {
        let g = generate(&OptionGrantInput { as_of_date: "2025-01-01".into(), ..base() });
        assert_eq!(g.months_elapsed, 12);
        assert!(close(g.vested_options, 12_000.0));
    }

    #[test]
    fn fully_vested_caps_at_total() {
        let g = generate(&OptionGrantInput { as_of_date: "2030-01-01".into(), ..base() });
        assert!(close(g.vested_options, 48_000.0));
        assert!(close(g.unvested_options, 0.0));
    }

    #[test]
    fn iso_spread_and_amt() {
        let g = generate(&base());
        assert!(close(g.spread_per_share_usd, 4.00));
        assert!(close(g.vested_spread_usd, 120_000.0));
        assert!(close(g.vested_exercise_cost_usd, 30_000.0));
        // ISO bargain element is an AMT preference item.
        assert!(close(g.iso_amt_preference_usd, 120_000.0));
    }

    #[test]
    fn nso_has_no_amt_preference() {
        let g = generate(&OptionGrantInput { option_type: "NSO".into(), ..base() });
        assert!(close(g.iso_amt_preference_usd, 0.0));
        assert!(g.clauses.iter().any(|c| c.body.contains("ordinary income")));
    }

    #[test]
    fn underwater_option_zero_spread() {
        let g = generate(&OptionGrantInput { fmv_usd: 0.50, ..base() });
        assert!(close(g.spread_per_share_usd, 0.0));
        assert!(close(g.vested_spread_usd, 0.0));
    }

    #[test]
    fn statute_citation_echoed() {
        let g = generate(&OptionGrantInput { statute_citation: "26 U.S.C. § 422".into(), ..base() });
        assert_eq!(g.statutory_citation, "26 U.S.C. § 422");
        assert!(g.clauses.iter().any(|c| c.body.contains("26 U.S.C. § 422")));
    }
}
