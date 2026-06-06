//! IRC § 6038D — Information with respect to foreign financial
//! assets (Form 8938).
//!
//! Trader-critical for anyone with offshore brokerage accounts,
//! foreign mutual fund holdings (PFICs covered by § 1297/1298),
//! foreign retirement accounts, foreign-issued bonds, or interests
//! in foreign entities. § 6038D requires individuals to attach
//! Form 8938 disclosing such assets when aggregate value crosses
//! the applicable filing threshold.
//!
//! Direct companion to:
//!   - `section_1297` (PFIC classification — many foreign assets
//!     trigger BOTH § 6038D Form 8938 AND § 1298(f) Form 8621
//!     filings).
//!   - `section_1298` (PFIC attribution + Form 8621 annual
//!     reporting).
//!
//! Distinct from FinCEN Form 114 (FBAR) — though both regimes
//! often apply to the same taxpayer, § 6038D is an income-tax
//! filing under Title 26 while FBAR is a separate BSA filing
//! under Title 31. Each has its own threshold, content
//! requirements, and penalty regime.
//!
//! § 6038D operative provisions:
//!
//!   § 6038D(a) — GENERAL RULE: Any individual who, during any
//!     taxable year, holds any interest in a SPECIFIED FOREIGN
//!     FINANCIAL ASSET shall attach Form 8938 to such person's
//!     income-tax return if the aggregate value of all such
//!     assets exceeds the threshold under § 6038D(b).
//!
//!   § 6038D(b) — THRESHOLD: Statutory baseline $50,000.
//!     Treas. Reg. § 1.6038D-2 tiers the threshold by filing
//!     status AND domestic/abroad residency, with separate
//!     year-end and any-time-during-year amounts:
//!       - Single / MFS Domestic: $50k year-end OR $75k any time
//!       - MFJ Domestic: $100k year-end OR $150k any time
//!       - Single / MFS Abroad: $200k year-end OR $300k any time
//!       - MFJ Abroad: $400k year-end OR $600k any time
//!
//!   § 6038D(c) — REQUIRED INFORMATION: Institution name +
//!     address + account number for each financial account;
//!     issuer name/info for each non-account asset (stocks,
//!     securities); maximum value of the asset during the
//!     taxable year.
//!
//!   § 6038D(d) — PENALTY: $10,000 per failure to disclose.
//!
//!   § 6038D(e) — CONTINUING FAILURE PENALTY: After IRS notice
//!     of failure, additional $10,000 for each 30-day period (or
//!     fraction thereof) the failure continues, capped at
//!     $50,000 additional.
//!
//!   § 6038D(g) — REASONABLE-CAUSE EXCEPTION: No penalty for
//!     failure shown to be due to reasonable cause AND not due
//!     to willful neglect.
//!
//! Citations: 26 U.S.C. § 6038D(a) (general filing rule);
//! § 6038D(b) (threshold — $50,000 statutory baseline); Treas.
//! Reg. § 1.6038D-2 (tiered thresholds by filing status +
//! residency); § 6038D(c) (required information); § 6038D(d)
//! ($10,000 initial penalty); § 6038D(e) (continuing $10k/30-day
//! up to $50k cap); § 6038D(g) (reasonable-cause + no-willful-
//! neglect exception); Treas. Reg. § 1.6038D-3 (specified
//! foreign financial assets); 31 U.S.C. § 5314 + 31 CFR § 1010.350
//! (separate FBAR / FinCEN Form 114 BSA reporting regime).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    /// Single or married-filing-separately, residence in U.S.:
    /// $50,000 year-end OR $75,000 any time during year.
    SingleOrMFSDomestic,
    /// Married-filing-jointly, residence in U.S.: $100,000
    /// year-end OR $150,000 any time during year.
    MarriedFilingJointlyDomestic,
    /// Single or married-filing-separately, residence ABROAD:
    /// $200,000 year-end OR $300,000 any time during year.
    SingleOrMFSAbroad,
    /// Married-filing-jointly, residence ABROAD: $400,000 year-end
    /// OR $600,000 any time during year.
    MarriedFilingJointlyAbroad,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6038DInput {
    pub filing_status: FilingStatus,
    /// Aggregate value of specified foreign financial assets on
    /// the LAST day of the taxable year (cents).
    pub aggregate_value_year_end_cents: i64,
    /// Aggregate value of specified foreign financial assets at
    /// ANY time during the taxable year (cents). High-water mark.
    pub aggregate_value_any_time_during_year_cents: i64,
    /// Whether Form 8938 was filed with the income-tax return.
    pub form_8938_filed: bool,
    /// Whether Form 8938 contained all § 6038D(c) required
    /// information (institution name + address + account number
    /// + issuer info + maximum value).
    pub form_8938_includes_required_information: bool,
    /// Days elapsed since IRS issued § 6038D(e) notice of failure.
    /// Continuing penalty accrues at $10,000 per 30-day period
    /// (or fraction thereof) after notice.
    pub days_since_irs_notice: u32,
    /// Whether the failure to disclose was due to REASONABLE CAUSE
    /// (§ 6038D(g) prong 1).
    pub reasonable_cause: bool,
    /// Whether the failure was due to WILLFUL NEGLECT (§ 6038D(g)
    /// prong 2 — if true, reasonable-cause exception does NOT
    /// apply).
    pub willful_neglect: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6038DResult {
    /// Treas. Reg. § 1.6038D-2 year-end threshold for this filing
    /// status (cents).
    pub year_end_threshold_cents: i64,
    /// Treas. Reg. § 1.6038D-2 any-time-during-year threshold
    /// (cents).
    pub any_time_threshold_cents: i64,
    /// True if Form 8938 filing is required (either threshold
    /// exceeded).
    pub filing_required: bool,
    /// § 6038D(d) initial $10,000 penalty (cents). Triggered when
    /// filing required AND not filed (and reasonable-cause
    /// exception does NOT apply).
    pub initial_penalty_cents: i64,
    /// § 6038D(e) continuing failure penalty (cents). $10,000 per
    /// 30-day period (or fraction) after IRS notice, capped at
    /// $50,000.
    pub continuing_penalty_cents: i64,
    /// Total penalty (cents) = initial + continuing.
    pub total_penalty_cents: i64,
    /// True if § 6038D(g) reasonable-cause exception applies
    /// (reasonable cause AND not willful neglect).
    pub reasonable_cause_excuses: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// § 6038D(d) — $10,000 initial penalty.
pub const SECTION_6038D_INITIAL_PENALTY_CENTS: i64 = 1_000_000;
/// § 6038D(e) — $10,000 per 30-day period continuing penalty.
pub const SECTION_6038D_CONTINUING_PENALTY_PER_PERIOD_CENTS: i64 = 1_000_000;
/// § 6038D(e) — $50,000 cap on continuing penalty.
pub const SECTION_6038D_CONTINUING_PENALTY_CAP_CENTS: i64 = 5_000_000;
/// § 6038D(e) — 30-day period length.
pub const SECTION_6038D_CONTINUING_PERIOD_DAYS: u32 = 30;
/// § 6038D(e) — 90-day grace period after IRS notice before
/// continuing penalty begins to accrue.
pub const SECTION_6038D_NOTICE_GRACE_DAYS: u32 = 90;

pub fn compute(input: &Section6038DInput) -> Section6038DResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let (year_end_threshold, any_time_threshold) = match input.filing_status {
        FilingStatus::SingleOrMFSDomestic => (5_000_000, 7_500_000),
        FilingStatus::MarriedFilingJointlyDomestic => (10_000_000, 15_000_000),
        FilingStatus::SingleOrMFSAbroad => (20_000_000, 30_000_000),
        FilingStatus::MarriedFilingJointlyAbroad => (40_000_000, 60_000_000),
    };

    let filing_required = input.aggregate_value_year_end_cents > year_end_threshold
        || input.aggregate_value_any_time_during_year_cents > any_time_threshold;

    let reasonable_cause_excuses = input.reasonable_cause && !input.willful_neglect;

    let (initial_penalty, continuing_penalty) = if !filing_required {
        (0, 0)
    } else if input.form_8938_filed && input.form_8938_includes_required_information {
        // Filed and complete.
        (0, 0)
    } else if reasonable_cause_excuses {
        // § 6038D(g) exception applies.
        notes.push(
            "§ 6038D(g) — reasonable-cause exception applies (reasonable cause AND not willful \
             neglect); no penalty imposed."
                .to_string(),
        );
        (0, 0)
    } else {
        // Penalty applies.
        let initial = SECTION_6038D_INITIAL_PENALTY_CENTS;
        // § 6038D(e) — continuing $10k per 30-day period after
        // 90-day notice grace.
        let continuing = if input.days_since_irs_notice > SECTION_6038D_NOTICE_GRACE_DAYS {
            let days_past_grace = input.days_since_irs_notice - SECTION_6038D_NOTICE_GRACE_DAYS;
            // Number of 30-day periods or fractions.
            let periods = days_past_grace.div_ceil(SECTION_6038D_CONTINUING_PERIOD_DAYS) as i64;
            let raw = periods.saturating_mul(SECTION_6038D_CONTINUING_PENALTY_PER_PERIOD_CENTS);
            raw.min(SECTION_6038D_CONTINUING_PENALTY_CAP_CENTS)
        } else {
            0
        };

        violations.push(
            "§ 6038D(d) — Form 8938 required but not filed (or incomplete); $10,000 initial \
             penalty applies."
                .to_string(),
        );
        if continuing > 0 {
            violations.push(format!(
                "§ 6038D(e) — continuing failure penalty: {} cents ({} days past 90-day IRS \
                 notice grace).",
                continuing,
                input.days_since_irs_notice - SECTION_6038D_NOTICE_GRACE_DAYS,
            ));
        }

        (initial, continuing)
    };

    let total_penalty = initial_penalty.saturating_add(continuing_penalty);

    // Note for filing-required-but-filed cases.
    if filing_required && input.form_8938_filed && !input.form_8938_includes_required_information {
        violations.push(
            "§ 6038D(c) — Form 8938 must include institution name, address, account number, \
             issuer information, and maximum value of asset during taxable year; required \
             information is missing."
                .to_string(),
        );
    }

    if input.willful_neglect {
        notes.push(
            "§ 6038D(g) — willful neglect bars the reasonable-cause exception; reasonable \
             cause alone is not sufficient where willful neglect is established."
                .to_string(),
        );
    }

    notes.push(
        "Companion to section_1297 (PFIC classification) and section_1298 (PFIC attribution + \
         Form 8621 annual reporting). Many foreign assets trigger BOTH § 6038D Form 8938 AND \
         § 1298(f) Form 8621 filings."
            .to_string(),
    );
    notes.push(
        "Distinct from FinCEN Form 114 (FBAR) under 31 U.S.C. § 5314 + 31 CFR § 1010.350 — \
         FBAR is a separate Bank Secrecy Act filing with its own $10,000 aggregate threshold \
         and penalty regime. Both filings often apply to the same taxpayer."
            .to_string(),
    );

    Section6038DResult {
        year_end_threshold_cents: year_end_threshold,
        any_time_threshold_cents: any_time_threshold,
        filing_required,
        initial_penalty_cents: initial_penalty,
        continuing_penalty_cents: continuing_penalty,
        total_penalty_cents: total_penalty,
        reasonable_cause_excuses,
        compliant: violations.is_empty(),
        violations,
        citation: "26 U.S.C. § 6038D(a) (general filing rule); § 6038D(b) (threshold — $50,000 \
                   statutory baseline); Treas. Reg. § 1.6038D-2 (tiered thresholds by filing \
                   status + residency); § 6038D(c) (required information — institution + \
                   issuer + maximum value); § 6038D(d) ($10,000 initial penalty); § 6038D(e) \
                   (continuing $10k/30-day up to $50k cap after 90-day notice); § 6038D(g) \
                   (reasonable-cause AND no-willful-neglect exception); Treas. Reg. \
                   § 1.6038D-3 (specified foreign financial assets); 31 U.S.C. § 5314 + 31 \
                   CFR § 1010.350 (separate FBAR / FinCEN Form 114)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        status: FilingStatus,
        year_end: i64,
        any_time: i64,
        filed: bool,
        complete: bool,
        days_since_notice: u32,
        reasonable: bool,
        willful: bool,
    ) -> Section6038DInput {
        Section6038DInput {
            filing_status: status,
            aggregate_value_year_end_cents: year_end,
            aggregate_value_any_time_during_year_cents: any_time,
            form_8938_filed: filed,
            form_8938_includes_required_information: complete,
            days_since_irs_notice: days_since_notice,
            reasonable_cause: reasonable,
            willful_neglect: willful,
        }
    }

    // ── § 6038D(b) thresholds + § 6038D(a) filing requirement ──

    #[test]
    fn single_domestic_below_threshold_no_filing_required() {
        // Year-end $40k, any-time $60k — both below thresholds.
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            4_000_000,
            6_000_000,
            false,
            false,
            0,
            false,
            false,
        ));
        assert!(!r.filing_required);
        assert!(r.compliant);
        assert_eq!(r.total_penalty_cents, 0);
    }

    #[test]
    fn single_domestic_year_end_just_above_50k_requires_filing() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            5_000_001,
            5_000_001,
            false,
            false,
            0,
            false,
            false,
        ));
        assert!(r.filing_required);
        assert!(!r.compliant);
        assert_eq!(r.initial_penalty_cents, 1_000_000);
    }

    #[test]
    fn single_domestic_year_end_exactly_50k_no_filing_required() {
        // § 6038D(a) says "exceeds" — at threshold not exceeded.
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            5_000_000,
            5_000_000,
            false,
            false,
            0,
            false,
            false,
        ));
        assert!(!r.filing_required);
    }

    #[test]
    fn single_domestic_any_time_above_75k_requires_filing() {
        // Year-end $40k (below 50k), any-time $80k (above 75k).
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            4_000_000,
            8_000_000,
            false,
            false,
            0,
            false,
            false,
        ));
        assert!(r.filing_required);
    }

    #[test]
    fn mfj_domestic_threshold_100k_year_end_150k_any_time() {
        let r = compute(&input(
            FilingStatus::MarriedFilingJointlyDomestic,
            10_000_000,
            10_000_000,
            false,
            false,
            0,
            false,
            false,
        ));
        assert_eq!(r.year_end_threshold_cents, 10_000_000);
        assert_eq!(r.any_time_threshold_cents, 15_000_000);
        assert!(!r.filing_required); // at not above
    }

    #[test]
    fn single_abroad_threshold_200k_300k() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSAbroad,
            0,
            0,
            false,
            false,
            0,
            false,
            false,
        ));
        assert_eq!(r.year_end_threshold_cents, 20_000_000);
        assert_eq!(r.any_time_threshold_cents, 30_000_000);
    }

    #[test]
    fn mfj_abroad_threshold_400k_600k() {
        let r = compute(&input(
            FilingStatus::MarriedFilingJointlyAbroad,
            0,
            0,
            false,
            false,
            0,
            false,
            false,
        ));
        assert_eq!(r.year_end_threshold_cents, 40_000_000);
        assert_eq!(r.any_time_threshold_cents, 60_000_000);
    }

    // ── § 6038D(d) initial $10,000 penalty ─────────────────────

    #[test]
    fn filing_required_not_filed_10k_penalty() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            false,
            false,
            0,
            false,
            false,
        ));
        assert_eq!(r.initial_penalty_cents, 1_000_000);
        assert_eq!(r.total_penalty_cents, 1_000_000);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6038D(d)") && v.contains("$10,000")));
    }

    #[test]
    fn filing_required_and_complete_filing_no_penalty() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            true,
            true,
            0,
            false,
            false,
        ));
        assert!(r.compliant);
        assert_eq!(r.total_penalty_cents, 0);
    }

    #[test]
    fn filed_but_incomplete_violation() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            true,
            false,
            0,
            false,
            false,
        ));
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6038D(c)") && v.contains("required information is missing")));
    }

    // ── § 6038D(e) continuing failure penalty ──────────────────

    #[test]
    fn within_90_day_grace_no_continuing_penalty() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            false,
            false,
            89, // within 90-day grace
            false,
            false,
        ));
        assert_eq!(r.continuing_penalty_cents, 0);
        assert_eq!(r.total_penalty_cents, 1_000_000);
    }

    #[test]
    fn at_90_day_boundary_no_continuing_penalty() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            false,
            false,
            90,
            false,
            false,
        ));
        assert_eq!(r.continuing_penalty_cents, 0);
    }

    #[test]
    fn day_91_one_period_continuing_penalty_10k() {
        // 1 day past 90-day grace = 1 fractional period = $10k.
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            false,
            false,
            91,
            false,
            false,
        ));
        assert_eq!(r.continuing_penalty_cents, 1_000_000);
        assert_eq!(r.total_penalty_cents, 2_000_000); // $20k total
    }

    #[test]
    fn day_120_two_periods_continuing_penalty_20k() {
        // 30 days past grace = 1 full period = $10k.
        // 31 days past grace = 2nd fractional period started = $20k.
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            false,
            false,
            120, // 30 days past 90-day grace
            false,
            false,
        ));
        // 30 days / 30 days per period = 1 period. div_ceil(30, 30) = 1.
        assert_eq!(r.continuing_penalty_cents, 1_000_000);
    }

    #[test]
    fn day_121_three_periods_started_continuing_penalty_20k() {
        // 31 days past grace → div_ceil(31, 30) = 2 periods = $20k.
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            false,
            false,
            121,
            false,
            false,
        ));
        assert_eq!(r.continuing_penalty_cents, 2_000_000);
    }

    #[test]
    fn continuing_penalty_caps_at_50k() {
        // 200 days past grace → div_ceil(200, 30) = 7 periods = $70k
        // but capped at $50k.
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            false,
            false,
            290, // 200 days past 90-day grace
            false,
            false,
        ));
        assert_eq!(r.continuing_penalty_cents, 5_000_000);
        assert_eq!(r.total_penalty_cents, 6_000_000); // $10k initial + $50k continuing
    }

    #[test]
    fn continuing_penalty_caps_at_50k_for_extreme_delay() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            false,
            false,
            10_000,
            false,
            false,
        ));
        assert_eq!(r.continuing_penalty_cents, 5_000_000);
    }

    // ── § 6038D(g) reasonable-cause exception ───────────────────

    #[test]
    fn reasonable_cause_without_willful_neglect_excuses_penalty() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            false,
            false,
            120,
            true,
            false,
        ));
        assert!(r.reasonable_cause_excuses);
        assert_eq!(r.total_penalty_cents, 0);
        assert!(r.compliant);
    }

    #[test]
    fn reasonable_cause_with_willful_neglect_does_not_excuse() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            false,
            false,
            0,
            true,
            true,
        ));
        assert!(!r.reasonable_cause_excuses);
        assert_eq!(r.total_penalty_cents, 1_000_000);
    }

    #[test]
    fn willful_neglect_alone_does_not_excuse() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            false,
            false,
            0,
            false,
            true,
        ));
        assert!(!r.reasonable_cause_excuses);
        assert_eq!(r.total_penalty_cents, 1_000_000);
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn filing_required_iff_either_threshold_exceeded_invariant() {
        // For Single/MFS Domestic ($50k/$75k):
        for (year_end, any_time, expected) in [
            (4_000_000_i64, 6_000_000_i64, false),
            (5_000_001, 6_000_000, true),  // year-end above
            (4_000_000, 7_500_001, true),  // any-time above
            (5_000_001, 7_500_001, true),  // both above
            (5_000_000, 7_500_000, false), // both exactly at threshold
        ] {
            let r = compute(&input(
                FilingStatus::SingleOrMFSDomestic,
                year_end,
                any_time,
                false,
                false,
                0,
                false,
                false,
            ));
            assert_eq!(
                r.filing_required, expected,
                "year_end={} any_time={} expected={}",
                year_end, any_time, expected,
            );
        }
    }

    #[test]
    fn reasonable_cause_excuses_only_when_no_willful_neglect_invariant() {
        // 4-cell truth table.
        for (reasonable, willful, expected_excuses) in [
            (false, false, false),
            (true, false, true),
            (false, true, false),
            (true, true, false),
        ] {
            let r = compute(&input(
                FilingStatus::SingleOrMFSDomestic,
                10_000_000,
                10_000_000,
                false,
                false,
                0,
                reasonable,
                willful,
            ));
            assert_eq!(
                r.reasonable_cause_excuses, expected_excuses,
                "reasonable={} willful={} expected_excuses={}",
                reasonable, willful, expected_excuses,
            );
        }
    }

    #[test]
    fn continuing_penalty_capped_at_50k_invariant() {
        for days_past_notice in [200_u32, 500, 1_000, 10_000] {
            let r = compute(&input(
                FilingStatus::SingleOrMFSDomestic,
                10_000_000,
                10_000_000,
                false,
                false,
                days_past_notice,
                false,
                false,
            ));
            assert!(
                r.continuing_penalty_cents <= 5_000_000,
                "days={} continuing={} exceeded $50k cap",
                days_past_notice,
                r.continuing_penalty_cents,
            );
        }
    }

    #[test]
    fn thresholds_per_filing_status_invariant() {
        let cases = [
            (
                FilingStatus::SingleOrMFSDomestic,
                5_000_000_i64,
                7_500_000_i64,
            ),
            (
                FilingStatus::MarriedFilingJointlyDomestic,
                10_000_000,
                15_000_000,
            ),
            (FilingStatus::SingleOrMFSAbroad, 20_000_000, 30_000_000),
            (
                FilingStatus::MarriedFilingJointlyAbroad,
                40_000_000,
                60_000_000,
            ),
        ];
        for (status, ye, any) in cases {
            let r = compute(&input(status, 0, 0, false, false, 0, false, false));
            assert_eq!(r.year_end_threshold_cents, ye, "{:?}", status);
            assert_eq!(r.any_time_threshold_cents, any, "{:?}", status);
        }
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            false,
            false,
            0,
            false,
            false,
        ));
        assert!(r.citation.contains("§ 6038D(a)"));
        assert!(r.citation.contains("§ 6038D(b)"));
        assert!(r.citation.contains("§ 6038D(c)"));
        assert!(r.citation.contains("§ 6038D(d)"));
        assert!(r.citation.contains("§ 6038D(e)"));
        assert!(r.citation.contains("§ 6038D(g)"));
        assert!(r.citation.contains("§ 1.6038D-2"));
        assert!(r.citation.contains("31 U.S.C. § 5314"));
    }

    #[test]
    fn fbar_distinction_note_present() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            true,
            true,
            0,
            false,
            false,
        ));
        assert!(
            r.notes.iter().any(|n| n.contains("FBAR")
                && n.contains("31 U.S.C. § 5314")
                && n.contains("Bank Secrecy Act")),
            "FBAR distinction note must be present"
        );
    }

    #[test]
    fn pfic_companion_note_present() {
        let r = compute(&input(
            FilingStatus::SingleOrMFSDomestic,
            10_000_000,
            10_000_000,
            true,
            true,
            0,
            false,
            false,
        ));
        assert!(
            r.notes.iter().any(|n| n.contains("section_1297")
                && n.contains("section_1298")
                && n.contains("Form 8621")),
            "PFIC companion note must be present"
        );
    }
}
