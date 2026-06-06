//! IRC § 1375 — Tax imposed when passive investment income of
//! S corporation having accumulated earnings and profits exceeds
//! 25 percent of gross receipts.
//!
//! Completes the S-corp cluster after § 1361 (definition), § 1367
//! (basis), § 1368 (distributions), and § 1374 (built-in gains).
//! § 1375 is the passive-investment-income tax — applies ONLY to
//! S-corps with accumulated E&P from a prior C-corp period AND
//! passive investment income exceeding 25% of gross receipts.
//! Companion to § 1362(d)(3) S-election termination after three
//! consecutive years of the same condition.
//!
//! § 1375(a) ENGAGEMENT — TWO conditions, both required:
//!   (1) S-corp has accumulated earnings and profits at close of
//!       the taxable year (carried over from prior C-corp period);
//!   (2) Gross receipts more than 25% of which are passive
//!       investment income.
//! S-corps that never were C-corps have zero accumulated E&P and
//! are never subject to § 1375 regardless of passive income mix.
//!
//! § 1375(b) TAX COMPUTATION:
//!   Tax = Excess Net Passive Income × highest § 11(b) corporate
//!         rate (21% post-TCJA).
//!
//! § 1375(b)(1)(B) EXCESS NET PASSIVE INCOME (ENPI) formula:
//!   ENPI = Net Passive Income × [(PII − 25% × GR) / PII]
//! Where:
//!   PII = Passive Investment Income (§ 1362(d)(3)(C) — rents,
//!         royalties, dividends, interest, annuities);
//!   GR  = Gross Receipts (§ 1362(d)(3)(B) — corporation's
//!         total receipts);
//!   NPI = Net Passive Income (PII less deductions directly
//!         connected with production of such passive income).
//!
//! § 1375(b)(1)(A) CAP: ENPI cannot exceed the corporation's
//! taxable income for the year (preserves S-corp ability to
//! survive even when entire passive income is "excess").
//!
//! § 1375(d) WAIVER: Secretary may waive the tax if the
//! corporation establishes that it determined in good faith that
//! it had no E&P at close of the year and that the E&P was
//! distributed within a reasonable period.
//!
//! § 1362(d)(3) — RELATED S-ELECTION TERMINATION RULE: An S
//! election terminates if for THREE CONSECUTIVE TAXABLE YEARS the
//! corporation has accumulated E&P at close of each year AND
//! gross receipts more than 25% of which are passive investment
//! income. Three years = termination; first year = warning;
//! second year = elevated warning. This is a distinct provision
//! from § 1375 tax but with identical engagement triggers.
//!
//! Citations: 26 U.S.C. § 1375 (general tax); 26 U.S.C. § 1375(a)
//! (engagement — E&P + 25% PII); 26 U.S.C. § 1375(b) (tax
//! computation); 26 U.S.C. § 1375(b)(1)(A) (ENPI capped at
//! taxable income); 26 U.S.C. § 1375(b)(1)(B) (ENPI formula);
//! 26 U.S.C. § 1375(d) (Secretary waiver authority); 26 U.S.C.
//! § 11(b) (highest corporate rate — 21% post-TCJA); 26 U.S.C.
//! § 1362(d)(3) (three-year termination); 26 U.S.C. § 1362(d)(3)(B)
//! (gross receipts definition); 26 U.S.C. § 1362(d)(3)(C)
//! (passive investment income definition); Treas. Reg.
//! § 1.1375-1 (tax regulations). Sibling modules: § 1361
//! (definition); § 1366 (pass-through); § 1367 (basis); § 1368
//! (distributions); § 1374 (built-in gains tax).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section1375Input {
    /// Accumulated E&P from prior C-corp period at close of
    /// taxable year. Tax engages only if > 0.
    pub accumulated_e_and_p_cents: i64,
    /// Passive investment income for the year — § 1362(d)(3)(C)
    /// (rents, royalties, dividends, interest, annuities).
    pub passive_investment_income_cents: i64,
    /// Gross receipts — § 1362(d)(3)(B) (corporation's total
    /// receipts).
    pub gross_receipts_cents: i64,
    /// Net passive income — PII less deductions directly
    /// connected with production of passive income.
    pub net_passive_income_cents: i64,
    /// Corporation's taxable income — caps ENPI under
    /// § 1375(b)(1)(A).
    pub taxable_income_cents: i64,
    /// Highest § 11(b) corporate rate in basis points
    /// (default 2100 = 21% post-TCJA).
    pub corporate_tax_rate_bps: i64,
    /// Number of CONSECUTIVE taxable years (counting current)
    /// in which both § 1362(d)(3) conditions have been met.
    /// Used to flag S-election termination risk.
    pub consecutive_years_pii_above_threshold: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1375Result {
    /// True if both § 1375(a) engagement conditions met.
    pub tax_engaged: bool,
    /// PII as percentage of gross receipts in basis points.
    pub pii_percentage_of_gross_receipts_bps: i64,
    /// True if PII exceeds 25% of gross receipts.
    pub pii_exceeds_25_percent_threshold: bool,
    /// Raw ENPI before § 1375(b)(1)(A) taxable-income cap.
    pub excess_net_passive_income_raw_cents: i64,
    /// ENPI after § 1375(b)(1)(A) cap (used for tax calc).
    pub excess_net_passive_income_capped_cents: i64,
    /// § 1375 tax (cents).
    pub section_1375_tax_cents: i64,
    /// True if § 1362(d)(3) three-year termination rule has
    /// been triggered.
    pub s_election_terminated: bool,
    /// Warning level for § 1362(d)(3) approach — 0 (no risk),
    /// 1 (one year), 2 (two years — termination next year).
    pub termination_risk_warning_level: i64,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// 25% threshold in basis points (2500 bps).
pub const PASSIVE_INCOME_THRESHOLD_BPS: i64 = 2500;
/// § 11(b) post-TCJA default corporate rate (2100 bps = 21%).
pub const DEFAULT_CORPORATE_RATE_BPS: i64 = 2100;
/// Basis-point denominator.
pub const BPS_DENOMINATOR: i64 = 10_000;
/// § 1362(d)(3) consecutive-year termination threshold.
pub const TERMINATION_CONSECUTIVE_YEARS: i64 = 3;

pub fn compute(input: &Section1375Input) -> Section1375Result {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let e_and_p = input.accumulated_e_and_p_cents.max(0);
    let pii = input.passive_investment_income_cents.max(0);
    let gross_receipts = input.gross_receipts_cents.max(0);
    let net_passive_income = input.net_passive_income_cents.max(0);
    let taxable_income = input.taxable_income_cents.max(0);
    let rate_bps = input.corporate_tax_rate_bps.clamp(0, BPS_DENOMINATOR);
    let consecutive_years = input.consecutive_years_pii_above_threshold.max(0);

    // PII percentage of gross receipts (bps) — display only.
    let pii_percentage_bps = if gross_receipts > 0 {
        pii.saturating_mul(BPS_DENOMINATOR) / gross_receipts
    } else {
        0
    };
    // Engagement test uses exact-integer comparison to avoid
    // sub-bps precision loss at the threshold boundary.
    // PII > 25% × GR  ≡  4 × PII > GR. Guard against GR = 0
    // (statute requires gross receipts > 0 of which 25% is
    // passive income).
    let pii_exceeds_25_percent = gross_receipts > 0 && pii.saturating_mul(4) > gross_receipts;

    // § 1375(a) engagement: BOTH conditions required.
    let tax_engaged = e_and_p > 0 && pii_exceeds_25_percent;

    // § 1375(b)(1)(B) ENPI calculation.
    let (enpi_raw, enpi_capped, tax) = if tax_engaged && pii > 0 {
        // ENPI = NPI × (PII - 25% × GR) / PII
        // Compute (PII − 25% × GR) — note: 25% × GR = GR / 4.
        let twenty_five_percent_gr = gross_receipts / 4;
        let pii_excess_over_threshold = pii.saturating_sub(twenty_five_percent_gr);
        // ENPI = NPI × pii_excess_over_threshold / PII (with overflow protection).
        let raw = net_passive_income.saturating_mul(pii_excess_over_threshold) / pii.max(1);
        let capped = raw.min(taxable_income);
        let t = capped.saturating_mul(rate_bps) / BPS_DENOMINATOR;
        (raw, capped, t)
    } else {
        (0, 0, 0)
    };

    // § 1362(d)(3) consecutive-year termination check.
    let s_election_terminated =
        pii_exceeds_25_percent && e_and_p > 0 && consecutive_years >= TERMINATION_CONSECUTIVE_YEARS;
    let termination_risk_warning_level = if pii_exceeds_25_percent && e_and_p > 0 {
        consecutive_years.min(TERMINATION_CONSECUTIVE_YEARS - 1)
    } else {
        0
    };

    if s_election_terminated {
        violations.push(format!(
            "§ 1362(d)(3) — S election TERMINATED. Corporation has had E&P + >25% passive \
             income for {} consecutive years (≥ 3). Election terminates effective close \
             of third taxable year. Corporation reverts to C-corp status unless re-\
             election under § 1362(g) (5-year waiting period unless IRS consents).",
            consecutive_years,
        ));
    }

    if tax_engaged {
        violations.push(format!(
            "§ 1375 — passive investment income tax engaged. PII {} cents is {}% of gross \
             receipts {} cents (exceeds 25% threshold), AND corporation has accumulated \
             E&P {} cents. Excess net passive income: {} cents (capped at taxable income \
             {} cents = {} cents); tax = {} cents at {}% rate.",
            pii,
            pii_percentage_bps / 100,
            gross_receipts,
            e_and_p,
            enpi_raw,
            taxable_income,
            enpi_capped,
            tax,
            rate_bps / 100,
        ));
    }

    // Engagement notes.
    if e_and_p == 0 {
        notes.push(
            "§ 1375 does NOT apply — corporation has no accumulated E&P. S-corps that \
             never were C-corps (or that have fully distributed prior E&P) are never \
             subject to § 1375 regardless of passive income mix. Companion: § 1362(d)(3) \
             also does NOT apply for the same reason."
                .to_string(),
        );
    } else if !pii_exceeds_25_percent {
        notes.push(format!(
            "§ 1375 does NOT apply — PII at {}% of gross receipts does not exceed 25% \
             threshold. Tax engages only when PII exceeds 25%. Monitor PII mix; \
             § 1362(d)(3) S-election termination risk engages on same 25%+ threshold.",
            pii_percentage_bps / 100,
        ));
    }

    notes.push(format!(
        "§ 1375(b)(1)(B) ENPI formula: NPI {} × (PII {} − 25% × GR {}) / PII {}. PII as \
         %% of GR: {}%. Threshold: 25%%.",
        net_passive_income,
        pii,
        gross_receipts,
        pii.max(1),
        pii_percentage_bps / 100,
    ));

    if tax_engaged && enpi_raw > taxable_income {
        notes.push(format!(
            "§ 1375(b)(1)(A) ENPI CAP engaged — raw ENPI {} cents capped at taxable \
             income {} cents. Preserves S-corp ability to survive when entire passive \
             income would otherwise be 'excess'.",
            enpi_raw, taxable_income,
        ));
    }

    if termination_risk_warning_level == 1 {
        notes.push(
            "§ 1362(d)(3) WARNING — first year of E&P + >25% PII pattern. Two more \
             consecutive years terminate the S election. Monitor PII ratio; consider \
             § 1368(e)(3) election to distribute E&P first and purge accumulated E&P \
             ahead of three-year clock."
                .to_string(),
        );
    } else if termination_risk_warning_level == 2 {
        notes.push(
            "§ 1362(d)(3) ELEVATED WARNING — second consecutive year of E&P + >25% PII. \
             One more year terminates the S election. Strongly consider § 1368(e)(3) \
             election to distribute E&P first, or restructure income mix below 25% \
             passive threshold."
                .to_string(),
        );
    }

    notes.push(
        "Sibling S-corp cluster: § 1361 (definition + eligibility); § 1366 (pass-through \
         items); § 1367 (basis adjustments); § 1368 (distributions — including § 1368(e)(3) \
         election to distribute E&P first, which can purge accumulated E&P and avoid \
         § 1375 + § 1362(d)(3) exposure); § 1374 (built-in gains tax — separate tax on \
         C-to-S conversion gains, often coincides with § 1375 exposure for newly \
         converted S-corps). § 1375 + § 1362(d)(3) are the two principal anti-abuse \
         rules targeting S-corps that operate as passive holding vehicles after C-corp \
         conversion."
            .to_string(),
    );

    let compliant = violations.is_empty();

    Section1375Result {
        tax_engaged,
        pii_percentage_of_gross_receipts_bps: pii_percentage_bps,
        pii_exceeds_25_percent_threshold: pii_exceeds_25_percent,
        excess_net_passive_income_raw_cents: enpi_raw,
        excess_net_passive_income_capped_cents: enpi_capped,
        section_1375_tax_cents: tax,
        s_election_terminated,
        termination_risk_warning_level,
        compliant,
        violations,
        citation: "26 U.S.C. § 1375 (general tax); 26 U.S.C. § 1375(a) (engagement — \
                   E&P + 25% PII); 26 U.S.C. § 1375(b) (tax computation); 26 U.S.C. \
                   § 1375(b)(1)(A) (ENPI capped at taxable income); 26 U.S.C. \
                   § 1375(b)(1)(B) (ENPI formula); 26 U.S.C. § 1375(d) (Secretary \
                   waiver authority); 26 U.S.C. § 11(b) (highest corporate rate — 21% \
                   post-TCJA); 26 U.S.C. § 1362(d)(3) (three-year S-election \
                   termination); 26 U.S.C. § 1362(d)(3)(B) (gross receipts definition); \
                   26 U.S.C. § 1362(d)(3)(C) (passive investment income definition); \
                   Treas. Reg. § 1.1375-1 (tax regulations); 26 U.S.C. § 1362(g) \
                   (5-year re-election waiting period)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Section1375Input {
        Section1375Input {
            accumulated_e_and_p_cents: 10_000_000,       // $100K E&P
            passive_investment_income_cents: 50_000_000, // $500K PII
            gross_receipts_cents: 100_000_000,           // $1M GR (50% PII)
            net_passive_income_cents: 40_000_000,        // $400K NPI
            taxable_income_cents: 50_000_000,            // $500K TI
            corporate_tax_rate_bps: DEFAULT_CORPORATE_RATE_BPS,
            consecutive_years_pii_above_threshold: 1,
        }
    }

    // ── Engagement triggers ────────────────────────────────────

    #[test]
    fn no_e_and_p_no_tax_engagement() {
        let mut b = input();
        b.accumulated_e_and_p_cents = 0;
        let r = compute(&b);
        assert!(!r.tax_engaged);
        assert_eq!(r.section_1375_tax_cents, 0);
        assert!(!r.s_election_terminated);
    }

    #[test]
    fn pii_at_25_percent_exactly_no_engagement() {
        let mut b = input();
        b.passive_investment_income_cents = 25_000_000; // $250K = 25% of $1M
        b.gross_receipts_cents = 100_000_000;
        let r = compute(&b);
        assert!(!r.pii_exceeds_25_percent_threshold);
        assert!(!r.tax_engaged);
    }

    #[test]
    fn pii_just_above_25_percent_engages() {
        let mut b = input();
        b.passive_investment_income_cents = 25_000_001;
        b.gross_receipts_cents = 100_000_000;
        let r = compute(&b);
        assert!(r.pii_exceeds_25_percent_threshold);
        assert!(r.tax_engaged);
    }

    #[test]
    fn pii_below_25_percent_no_engagement() {
        let mut b = input();
        b.passive_investment_income_cents = 20_000_000; // 20% of $1M
        let r = compute(&b);
        assert!(!r.pii_exceeds_25_percent_threshold);
        assert!(!r.tax_engaged);
    }

    // ── ENPI formula math ─────────────────────────────────────

    #[test]
    fn enpi_baseline_math() {
        let r = compute(&input());
        // GR $1M, PII $500K (50%), NPI $400K. 25% × GR = $250K.
        // PII - 25%GR = $500K - $250K = $250K.
        // ENPI = $400K × $250K / $500K = $200K.
        assert_eq!(r.excess_net_passive_income_raw_cents, 20_000_000);
        assert_eq!(r.excess_net_passive_income_capped_cents, 20_000_000);
        // Tax = $200K × 21% = $42K.
        assert_eq!(r.section_1375_tax_cents, 4_200_000);
    }

    #[test]
    fn enpi_capped_at_taxable_income() {
        let mut b = input();
        b.taxable_income_cents = 5_000_000; // $50K TI, below ENPI $200K
        let r = compute(&b);
        assert_eq!(r.excess_net_passive_income_raw_cents, 20_000_000);
        assert_eq!(r.excess_net_passive_income_capped_cents, 5_000_000);
        // Tax = $50K × 21% = $10.5K
        assert_eq!(r.section_1375_tax_cents, 1_050_000);
    }

    #[test]
    fn enpi_zero_when_no_engagement() {
        let mut b = input();
        b.accumulated_e_and_p_cents = 0;
        let r = compute(&b);
        assert_eq!(r.excess_net_passive_income_raw_cents, 0);
        assert_eq!(r.section_1375_tax_cents, 0);
    }

    #[test]
    fn enpi_full_passive_income_when_pii_is_all_of_gross_receipts() {
        let mut b = input();
        b.passive_investment_income_cents = 100_000_000; // 100% PII
        b.gross_receipts_cents = 100_000_000;
        b.net_passive_income_cents = 80_000_000;
        b.taxable_income_cents = 100_000_000;
        // PII - 25%GR = $1M - $250K = $750K.
        // ENPI = $800K × $750K / $1M = $600K.
        let r = compute(&b);
        assert_eq!(r.excess_net_passive_income_raw_cents, 60_000_000);
    }

    // ── § 1362(d)(3) three-year termination ───────────────────

    #[test]
    fn consecutive_year_1_warning_level_1() {
        let mut b = input();
        b.consecutive_years_pii_above_threshold = 1;
        let r = compute(&b);
        assert_eq!(r.termination_risk_warning_level, 1);
        assert!(!r.s_election_terminated);
    }

    #[test]
    fn consecutive_year_2_warning_level_2() {
        let mut b = input();
        b.consecutive_years_pii_above_threshold = 2;
        let r = compute(&b);
        assert_eq!(r.termination_risk_warning_level, 2);
        assert!(!r.s_election_terminated);
    }

    #[test]
    fn consecutive_year_3_election_terminated() {
        let mut b = input();
        b.consecutive_years_pii_above_threshold = 3;
        let r = compute(&b);
        assert_eq!(r.termination_risk_warning_level, 2); // clamps at 2 — display
        assert!(r.s_election_terminated);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 1362(d)(3)")));
    }

    #[test]
    fn consecutive_years_only_count_when_engagement_present() {
        let mut b = input();
        b.accumulated_e_and_p_cents = 0;
        b.consecutive_years_pii_above_threshold = 5; // doesn't matter without E&P
        let r = compute(&b);
        assert!(!r.s_election_terminated);
        assert_eq!(r.termination_risk_warning_level, 0);
    }

    // ── Tax rate variations ───────────────────────────────────

    #[test]
    fn pre_tcja_35_percent_rate() {
        let mut b = input();
        b.corporate_tax_rate_bps = 3500; // 35% pre-TCJA
        let r = compute(&b);
        // ENPI $200K × 35% = $70K
        assert_eq!(r.section_1375_tax_cents, 7_000_000);
    }

    #[test]
    fn zero_rate_no_tax() {
        let mut b = input();
        b.corporate_tax_rate_bps = 0;
        let r = compute(&b);
        assert_eq!(r.section_1375_tax_cents, 0);
        // Still engaged.
        assert!(r.tax_engaged);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn engagement_requires_both_triggers_truth_table() {
        // 4-cell truth table: E&P × PII>25%.
        let cells = [
            (10_000_000, 50_000_000, true),  // both → engaged
            (10_000_000, 20_000_000, false), // E&P only, PII below
            (0, 50_000_000, false),          // PII above, no E&P
            (0, 20_000_000, false),          // neither
        ];
        for (e_and_p, pii, expected_engaged) in cells.iter() {
            let mut b = input();
            b.accumulated_e_and_p_cents = *e_and_p;
            b.passive_investment_income_cents = *pii;
            b.gross_receipts_cents = 100_000_000;
            let r = compute(&b);
            assert_eq!(
                r.tax_engaged, *expected_engaged,
                "e_and_p={} pii={}",
                e_and_p, pii
            );
        }
    }

    #[test]
    fn pii_percentage_calculation_bps_invariant() {
        let mut b = input();
        b.passive_investment_income_cents = 25_000_000;
        b.gross_receipts_cents = 100_000_000;
        let r = compute(&b);
        assert_eq!(r.pii_percentage_of_gross_receipts_bps, 2500); // 25%

        b.passive_investment_income_cents = 30_000_000;
        let r2 = compute(&b);
        assert_eq!(r2.pii_percentage_of_gross_receipts_bps, 3000); // 30%
    }

    #[test]
    fn termination_threshold_constant_invariant() {
        assert_eq!(PASSIVE_INCOME_THRESHOLD_BPS, 2500); // 25%
        assert_eq!(DEFAULT_CORPORATE_RATE_BPS, 2100); // 21% post-TCJA
        assert_eq!(TERMINATION_CONSECUTIVE_YEARS, 3);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input());
        assert!(r.citation.contains("§ 1375"));
        assert!(r.citation.contains("§ 1375(a)"));
        assert!(r.citation.contains("§ 1375(b)"));
        assert!(r.citation.contains("§ 1375(b)(1)(A)"));
        assert!(r.citation.contains("§ 1375(b)(1)(B)"));
        assert!(r.citation.contains("§ 1375(d)"));
        assert!(r.citation.contains("§ 11(b)"));
        assert!(r.citation.contains("§ 1362(d)(3)"));
        assert!(r.citation.contains("§ 1362(d)(3)(B)"));
        assert!(r.citation.contains("§ 1362(d)(3)(C)"));
        assert!(r.citation.contains("§ 1.1375-1"));
        assert!(r.citation.contains("§ 1362(g)"));
    }

    #[test]
    fn sibling_modules_note_present() {
        let r = compute(&input());
        assert!(
            r.notes.iter().any(|n| n.contains("§ 1361")
                && n.contains("§ 1366")
                && n.contains("§ 1367")
                && n.contains("§ 1368")
                && n.contains("§ 1368(e)(3)")
                && n.contains("§ 1374")
                && n.contains("§ 1362(d)(3)")),
            "S-corp cluster note must reference all 5 sibling statutes + § 1368(e)(3) election + § 1362(d)(3)"
        );
    }

    // ── Defensive input clamping ───────────────────────────────

    #[test]
    fn defensive_negative_inputs_clamped() {
        let mut b = input();
        b.accumulated_e_and_p_cents = -10_000_000;
        b.passive_investment_income_cents = -5_000_000;
        b.gross_receipts_cents = -100_000_000;
        b.net_passive_income_cents = -1_000_000;
        b.taxable_income_cents = -10_000_000;
        let r = compute(&b);
        // All clamped to 0 → no engagement.
        assert!(!r.tax_engaged);
        assert_eq!(r.section_1375_tax_cents, 0);
    }

    #[test]
    fn defensive_zero_gross_receipts_no_overflow() {
        let mut b = input();
        b.gross_receipts_cents = 0;
        b.passive_investment_income_cents = 50_000_000;
        let r = compute(&b);
        // PII percentage = 0 when GR = 0 (avoid division by zero).
        assert_eq!(r.pii_percentage_of_gross_receipts_bps, 0);
        assert!(!r.tax_engaged);
    }

    #[test]
    fn defensive_rate_above_100_percent_clamped() {
        let mut b = input();
        b.corporate_tax_rate_bps = 15_000; // 150% nonsense
        let r = compute(&b);
        // Rate clamped to 100% = 10000 bps; tax = ENPI × 100%.
        assert_eq!(r.section_1375_tax_cents, 20_000_000); // $200K
    }

    #[test]
    fn enpi_formula_high_pii_low_npi() {
        let mut b = input();
        b.passive_investment_income_cents = 80_000_000;
        b.gross_receipts_cents = 100_000_000;
        b.net_passive_income_cents = 10_000_000; // small NPI relative to PII
        b.taxable_income_cents = 80_000_000;
        // PII - 25%GR = $800K - $250K = $550K.
        // ENPI = $100K × $550K / $800K = $68.75K.
        let r = compute(&b);
        assert_eq!(r.excess_net_passive_income_raw_cents, 6_875_000);
    }

    #[test]
    fn taxable_income_zero_caps_tax_at_zero() {
        let mut b = input();
        b.taxable_income_cents = 0;
        let r = compute(&b);
        assert_eq!(r.excess_net_passive_income_capped_cents, 0);
        assert_eq!(r.section_1375_tax_cents, 0);
        // Still engaged but capped to zero.
        assert!(r.tax_engaged);
    }
}
