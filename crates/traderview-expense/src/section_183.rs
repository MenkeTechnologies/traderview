//! IRC §183 — Activities not engaged in for profit (the "hobby
//! loss" rules).
//!
//! Trader-critical because a trader who loses money for years
//! risks IRS recharacterization of the activity as a hobby rather
//! than a §162 trade or business — losing access to ordinary-loss
//! deductions for all trading expenses (data feeds, hardware,
//! office, professional fees). For post-2017 tax years (made
//! permanent by OBBBA 2025), the §67(g) suspension of
//! miscellaneous itemized deductions means hobby expenses are
//! effectively non-deductible — the taxpayer recognizes the
//! income but receives no offsetting deductions.
//!
//! **§183(a) general rule**: deductions for an activity not
//! engaged in for profit are allowed only to the extent provided
//! in §183(b).
//!
//! **§183(b)(1)**: deductions allowable WITHOUT regard to profit
//! motive (property tax, mortgage interest, etc.) are always
//! deductible to the extent otherwise allowable.
//!
//! **§183(b)(2)**: other deductions are allowed up to the excess
//! of gross income from the activity over the §183(b)(1)
//! deductions — capped at zero (never negative). Post-TCJA /
//! OBBBA: these are miscellaneous itemized deductions suspended
//! through 2025 and beyond by §67(g), so effectively ZERO.
//!
//! **§183(c)**: "activity not engaged in for profit" = any
//! activity other than one with respect to which deductions are
//! allowable under §162 or §212.
//!
//! **§183(d) profit-motive presumption**:
//! - Standard activities: profit in 3 or more taxable years of a
//!   5-consecutive-year period creates presumption of profit motive.
//! - Horse-related activities (breeding / training / showing /
//!   racing): profit in 2 of 7 consecutive years.
//!
//! **§183(e) election** to defer the presumption determination
//! until after the first 5 (or 7) years.
//!
//! **Reg. § 1.183-2(b) nine-factor test** for cases where the
//! §183(d) presumption is not met or rebutted:
//! 1. Manner in which the activity is carried on
//! 2. Expertise of the taxpayer or advisors
//! 3. Time and effort expended
//! 4. Expectation that assets used may appreciate
//! 5. Success of taxpayer in similar activities
//! 6. Taxpayer's history of income or losses
//! 7. Amount of occasional profits
//! 8. Financial status of taxpayer
//! 9. Elements of personal pleasure or recreation
//!
//! No single factor is controlling; all must be weighed together.
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 183](https://www.law.cornell.edu/uscode/text/26/183),
//! [IRS Pub. 5558 — Activities Not Engaged in for Profit ATG](https://www.irs.gov/pub/irs-pdf/p5558.pdf),
//! [IRS FS-08-23 — Is Your Hobby a For-Profit Endeavor?](https://www.irs.gov/pub/irs-news/fs-08-23.pdf),
//! [Meadows Collier — Hobby Loss and Ranches §183 Overview](https://www.meadowscollier.com/hobby-loss-and-ranches-an-overview-of-section-183).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    /// Standard activity (3-of-5-year presumption window).
    Standard,
    /// Horse-related activity (2-of-7-year presumption window).
    HorseRelated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfitMotiveDetermination {
    /// §183(d) presumption met (3 of 5 / 2 of 7 years profit).
    /// Activity treated as §162 trade or business; all deductions
    /// allowable.
    PresumptionMet,
    /// Taxpayer establishes profit motive via the 9-factor test
    /// even though §183(d) presumption was NOT met.
    NineFactorEstablished,
    /// Neither presumption nor 9-factor analysis supports profit
    /// motive — §183 hobby treatment applies.
    HobbyTreatment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section183Input {
    pub activity_type: ActivityType,
    /// Number of profit years in the relevant window (5 years
    /// standard / 7 years horse).
    pub profit_years_in_window: u32,
    /// Number of the 9 Reg. §1.183-2(b) factors that favor profit
    /// motive on these facts (0-9).
    pub nine_factors_favoring_profit: u32,
    /// True if the §183(d) presumption test has not yet been
    /// applied because the activity is in its first 5 (or 7)
    /// years — §183(e) deferral election available.
    pub section_183e_deferral_election_made: bool,
    /// Gross income from the activity.
    pub gross_income_from_activity_dollars: i64,
    /// §183(b)(1) deductions otherwise allowable regardless of
    /// profit motive (property tax, mortgage interest, etc.).
    pub section_183b1_deductions_dollars: i64,
    /// Other deductions attributable to the activity (would be
    /// §183(b)(2)-limited).
    pub other_activity_deductions_dollars: i64,
    /// True if the tax year is 2018+ (TCJA / OBBBA §67(g)
    /// suspension of miscellaneous itemized deductions). Module
    /// uses this to zero §183(b)(2) deductions for post-TCJA years.
    pub post_2017_section_67g_suspension_applies: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section183Result {
    pub determination: ProfitMotiveDetermination,
    pub presumption_threshold_profit_years: u32,
    pub presumption_met: bool,
    pub section_183b1_deductions_allowed_dollars: i64,
    pub section_183b2_deductions_allowed_dollars: i64,
    /// True if §67(g) suspended the §183(b)(2) deductions to zero.
    pub section_67g_disallowance_applied: bool,
    pub total_deductions_allowed_dollars: i64,
    pub net_taxable_activity_income_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section183Input) -> Section183Result {
    let threshold = match input.activity_type {
        ActivityType::Standard => 3,
        ActivityType::HorseRelated => 2,
    };

    let presumption_met =
        !input.section_183e_deferral_election_made && input.profit_years_in_window >= threshold;

    // Determination of profit motive:
    // 1. §183(d) presumption met → profit motive
    // 2. Nine-factor test shows majority of factors favor → profit motive
    // 3. Otherwise → hobby treatment
    let determination = if presumption_met {
        ProfitMotiveDetermination::PresumptionMet
    } else if input.nine_factors_favoring_profit >= 5 {
        ProfitMotiveDetermination::NineFactorEstablished
    } else {
        ProfitMotiveDetermination::HobbyTreatment
    };

    let is_hobby = matches!(determination, ProfitMotiveDetermination::HobbyTreatment);

    let b1_deductions = input.section_183b1_deductions_dollars.max(0);
    let other_deductions = input.other_activity_deductions_dollars.max(0);
    let gross_income = input.gross_income_from_activity_dollars.max(0);

    let (b1_allowed, b2_allowed, section_67g_applied) = if is_hobby {
        // §67(g) post-TCJA / OBBBA: §183(b)(2) deductions are
        // miscellaneous itemized deductions and are SUSPENDED.
        let s67g = input.post_2017_section_67g_suspension_applies;
        let b2_cap = (gross_income - b1_deductions).max(0).min(other_deductions);
        let b2 = if s67g { 0 } else { b2_cap };
        (b1_deductions, b2, s67g)
    } else {
        // Profit motive established → all expenses allowable under
        // §162 / §212 (no §183 cap).
        (b1_deductions, other_deductions, false)
    };

    let total_allowed = b1_allowed + b2_allowed;
    let net_taxable = gross_income - total_allowed;

    let determination_label = match determination {
        ProfitMotiveDetermination::PresumptionMet => {
            "§183(d) presumption met — profit motive established (§162 trade or business)"
        }
        ProfitMotiveDetermination::NineFactorEstablished => {
            "Reg. § 1.183-2(b) 9-factor analysis establishes profit motive (despite §183(d) presumption not met)"
        }
        ProfitMotiveDetermination::HobbyTreatment => {
            "§183 hobby treatment applies — neither presumption nor 9-factor analysis supports profit motive"
        }
    };

    let note = format!(
        "Activity type: {:?}; profit years in window: {} (threshold {}); presumption met: {}; 9 factors favoring profit: {}; determination: {}; §183(b)(1) allowed ${}; §183(b)(2) allowed ${} ({}); total deductions allowed ${}; net taxable activity income ${}.",
        input.activity_type,
        input.profit_years_in_window,
        threshold,
        presumption_met,
        input.nine_factors_favoring_profit,
        determination_label,
        b1_allowed,
        b2_allowed,
        if section_67g_applied {
            "ZEROED by §67(g) post-TCJA suspension of misc. itemized deductions"
        } else if is_hobby {
            "limited by §183(b)(2) income cap"
        } else {
            "no §183 cap — §162 trade/business"
        },
        total_allowed,
        net_taxable,
    );

    Section183Result {
        determination,
        presumption_threshold_profit_years: threshold,
        presumption_met,
        section_183b1_deductions_allowed_dollars: b1_allowed,
        section_183b2_deductions_allowed_dollars: b2_allowed,
        section_67g_disallowance_applied: section_67g_applied,
        total_deductions_allowed_dollars: total_allowed,
        net_taxable_activity_income_dollars: net_taxable,
        citation:
            "IRC §183(a) hobby loss general rule; §183(b)(1) deductions allowable regardless of profit motive; §183(b)(2) other deductions capped at gross income − §183(b)(1) (effectively ZERO post-TCJA via §67(g) miscellaneous-itemized-deduction suspension made permanent by OBBBA 2025); §183(c) definition; §183(d) presumption (3 of 5 standard / 2 of 7 horse-related); §183(e) deferral election; Reg. § 1.183-2(b) 9-factor test"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section183Input {
        Section183Input {
            activity_type: ActivityType::Standard,
            profit_years_in_window: 3,
            nine_factors_favoring_profit: 5,
            section_183e_deferral_election_made: false,
            gross_income_from_activity_dollars: 10_000,
            section_183b1_deductions_dollars: 2_000,
            other_activity_deductions_dollars: 15_000,
            post_2017_section_67g_suspension_applies: true,
        }
    }

    // ── §183(d) presumption — standard 3 of 5 ──────────────────────

    #[test]
    fn standard_3_of_5_presumption_met() {
        let r = compute(&base());
        assert_eq!(r.presumption_threshold_profit_years, 3);
        assert!(r.presumption_met);
        assert_eq!(r.determination, ProfitMotiveDetermination::PresumptionMet);
    }

    #[test]
    fn standard_4_of_5_presumption_met() {
        let mut i = base();
        i.profit_years_in_window = 4;
        let r = compute(&i);
        assert!(r.presumption_met);
    }

    #[test]
    fn standard_2_of_5_presumption_not_met() {
        let mut i = base();
        i.profit_years_in_window = 2;
        i.nine_factors_favoring_profit = 0;
        let r = compute(&i);
        assert!(!r.presumption_met);
        assert_eq!(r.determination, ProfitMotiveDetermination::HobbyTreatment);
    }

    // ── §183(d) horse-related 2 of 7 ───────────────────────────────

    #[test]
    fn horse_2_of_7_presumption_met() {
        let mut i = base();
        i.activity_type = ActivityType::HorseRelated;
        i.profit_years_in_window = 2;
        let r = compute(&i);
        assert_eq!(r.presumption_threshold_profit_years, 2);
        assert!(r.presumption_met);
    }

    #[test]
    fn horse_1_of_7_presumption_not_met() {
        let mut i = base();
        i.activity_type = ActivityType::HorseRelated;
        i.profit_years_in_window = 1;
        i.nine_factors_favoring_profit = 0;
        let r = compute(&i);
        assert!(!r.presumption_met);
    }

    // ── §183(e) deferral election ──────────────────────────────────

    #[test]
    fn deferral_election_suppresses_presumption() {
        let mut i = base();
        i.section_183e_deferral_election_made = true;
        i.profit_years_in_window = 5; // would otherwise pass
        i.nine_factors_favoring_profit = 0;
        let r = compute(&i);
        assert!(!r.presumption_met);
    }

    // ── Reg. § 1.183-2(b) 9-factor backup ──────────────────────────

    #[test]
    fn nine_factor_5_of_9_establishes_profit_motive() {
        let mut i = base();
        i.profit_years_in_window = 1; // presumption fails
        i.nine_factors_favoring_profit = 5;
        let r = compute(&i);
        assert_eq!(
            r.determination,
            ProfitMotiveDetermination::NineFactorEstablished
        );
    }

    #[test]
    fn nine_factor_4_of_9_falls_to_hobby() {
        let mut i = base();
        i.profit_years_in_window = 1;
        i.nine_factors_favoring_profit = 4;
        let r = compute(&i);
        assert_eq!(r.determination, ProfitMotiveDetermination::HobbyTreatment);
    }

    // ── Hobby treatment + §67(g) zero deductions ───────────────────

    #[test]
    fn hobby_with_67g_suspension_only_b1_allowed() {
        let mut i = base();
        i.profit_years_in_window = 1;
        i.nine_factors_favoring_profit = 0;
        i.post_2017_section_67g_suspension_applies = true;
        let r = compute(&i);
        assert!(r.section_67g_disallowance_applied);
        // Only §183(b)(1) $2k allowed; §183(b)(2) zeroed.
        assert_eq!(r.section_183b1_deductions_allowed_dollars, 2_000);
        assert_eq!(r.section_183b2_deductions_allowed_dollars, 0);
        // Net taxable = $10k − $2k = $8k.
        assert_eq!(r.net_taxable_activity_income_dollars, 8_000);
    }

    #[test]
    fn hobby_pre_2018_b2_allowed_up_to_income_cap() {
        let mut i = base();
        i.profit_years_in_window = 1;
        i.nine_factors_favoring_profit = 0;
        i.post_2017_section_67g_suspension_applies = false;
        let r = compute(&i);
        assert!(!r.section_67g_disallowance_applied);
        // §183(b)(2) cap: $10k income − $2k (b)(1) = $8k available;
        // other deductions $15k → capped at $8k.
        assert_eq!(r.section_183b2_deductions_allowed_dollars, 8_000);
        assert_eq!(r.net_taxable_activity_income_dollars, 0);
    }

    #[test]
    fn hobby_pre_2018_b1_below_income_b2_can_zero_net() {
        // Smaller other deductions → b2 not capped.
        let mut i = base();
        i.profit_years_in_window = 1;
        i.nine_factors_favoring_profit = 0;
        i.post_2017_section_67g_suspension_applies = false;
        i.other_activity_deductions_dollars = 3_000;
        let r = compute(&i);
        // Cap = $8k; other = $3k → §183(b)(2) = $3k (no cap binding).
        assert_eq!(r.section_183b2_deductions_allowed_dollars, 3_000);
        // Net = $10k − $2k − $3k = $5k.
        assert_eq!(r.net_taxable_activity_income_dollars, 5_000);
    }

    // ── Profit motive established → no §183 cap ────────────────────

    #[test]
    fn presumption_met_all_expenses_allowed_no_cap() {
        let r = compute(&base());
        assert_eq!(r.determination, ProfitMotiveDetermination::PresumptionMet);
        // All $15k other deductions allowed.
        assert_eq!(r.section_183b2_deductions_allowed_dollars, 15_000);
        // Net = $10k − $2k − $15k = −$7k (NOL territory).
        assert_eq!(r.net_taxable_activity_income_dollars, -7_000);
    }

    #[test]
    fn nine_factor_established_all_expenses_allowed() {
        let mut i = base();
        i.profit_years_in_window = 0;
        i.nine_factors_favoring_profit = 9;
        let r = compute(&i);
        assert_eq!(
            r.determination,
            ProfitMotiveDetermination::NineFactorEstablished
        );
        assert_eq!(r.section_183b2_deductions_allowed_dollars, 15_000);
    }

    // ── §183(b)(1) always allowed ──────────────────────────────────

    #[test]
    fn b1_deductions_always_allowed_even_for_hobby() {
        let mut i = base();
        i.profit_years_in_window = 0;
        i.nine_factors_favoring_profit = 0;
        let r = compute(&i);
        assert_eq!(r.section_183b1_deductions_allowed_dollars, 2_000);
    }

    // ── Citation ───────────────────────────────────────────────────

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§183(a)"));
        assert!(r.citation.contains("§183(b)(1)"));
        assert!(r.citation.contains("§183(b)(2)"));
        assert!(r.citation.contains("§183(c)"));
        assert!(r.citation.contains("§183(d)"));
        assert!(r.citation.contains("§183(e)"));
        assert!(r.citation.contains("§ 1.183-2(b)"));
        assert!(r.citation.contains("§67(g)"));
        assert!(r.citation.contains("OBBBA 2025"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn presumption_note_mentions_162_trade_business() {
        let r = compute(&base());
        assert!(r.note.contains("§162 trade or business"));
    }

    #[test]
    fn hobby_with_67g_note_says_zeroed() {
        let mut i = base();
        i.profit_years_in_window = 0;
        i.nine_factors_favoring_profit = 0;
        let r = compute(&i);
        assert!(r.note.contains("ZEROED by §67(g)"));
    }

    #[test]
    fn nine_factor_note_says_established() {
        let mut i = base();
        i.profit_years_in_window = 0;
        i.nine_factors_favoring_profit = 7;
        let r = compute(&i);
        assert!(r
            .note
            .contains("9-factor analysis establishes profit motive"));
    }

    // ── Precision ──────────────────────────────────────────────────

    #[test]
    fn very_large_activity_no_precision_loss() {
        let mut i = base();
        i.gross_income_from_activity_dollars = 1_000_000_000;
        i.section_183b1_deductions_dollars = 100_000_000;
        i.other_activity_deductions_dollars = 500_000_000;
        let r = compute(&i);
        // Presumption met → all allowed.
        assert_eq!(r.section_183b2_deductions_allowed_dollars, 500_000_000);
        // Net = $1B − $100M − $500M = $400M.
        assert_eq!(r.net_taxable_activity_income_dollars, 400_000_000);
    }
}
