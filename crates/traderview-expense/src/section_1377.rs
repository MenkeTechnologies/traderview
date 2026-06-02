//! IRC § 1377 — Definitions and special rule for S
//! corporation pro rata share computation, terminating
//! election on shareholder departure, and post-termination
//! transition period (PTTP). Direct S-corporation-cluster
//! companion to section_1361 (S corp election), section_1366
//! (pass-thru — iter 488), section_1367 (basis adjustments),
//! section_1368 (distributions), section_1374 (built-in
//! gains), section_1375 (passive investment income).
//!
//! § 1377(a)(1) PRO RATA SHARE GENERAL RULE: each
//! shareholder's pro rata share of any S corporation
//! item for any taxable year is determined by assigning
//! an equal portion of the item to EACH DAY of the S
//! corporation's taxable year, then dividing that portion
//! PRO RATA among shares outstanding on that day. This
//! is the DAILY ASSIGNMENT + PER-SHARE allocation method.
//!
//! Example: If S corp has $365,000 of income for a
//! 365-day year and 100 shares outstanding throughout
//! the year, each share gets $10/day × 365 = $3,650
//! annual share. A 25-share shareholder gets $91,250
//! pro rata share.
//!
//! Special-day rules per 26 C.F.R. § 1.1377-1(a)(2):
//! 1. Shareholder who DISPOSES of stock is treated as
//!    the shareholder for the DAY OF DISPOSITION
//! 2. Shareholder who DIES is treated as the shareholder
//!    for the DAY OF DEATH
//!
//! § 1377(a)(2) TERMINATING ELECTION ("closing of the
//! books") — if a shareholder's interest in an S
//! corporation is TERMINATED during the taxable year
//! AND ALL affected shareholders agree, the corporation
//! may elect to apply § 1377(a)(1) AS IF the taxable
//! year consisted of TWO TAXABLE YEARS, the first of
//! which ends on the date of termination. This avoids
//! the daily-assignment method's averaging effect when
//! actual operating results swing dramatically before
//! and after termination.
//!
//! § 1377(a)(2) eligibility requires:
//! 1. Shareholder's ENTIRE interest must terminate
//!    during the taxable year — not partial sale
//! 2. ALL AFFECTED SHAREHOLDERS must consent — defined
//!    in 26 C.F.R. § 1.1377-1(b)(2) as terminating
//!    shareholder + acquiring shareholder
//! 3. Election filed by attaching statement to timely
//!    Form 1120-S (including extensions)
//!
//! Two situations qualify for § 1377(a)(2) terminating
//! election:
//! 1. § 1377(a)(2)(A) — Shareholder's stock FULLY
//!    DISPOSED OF (sale, exchange, gift, or other
//!    transfer)
//! 2. § 1377(a)(2)(B) — Shareholder's stock REDEEMED
//!    in a transaction qualifying under § 302
//!    (substantially disproportionate redemption) or
//!    § 303 (estate-tax stock redemption)
//!
//! § 1377(b) POST-TERMINATION TRANSITION PERIOD (PTTP)
//! definitions:
//! 1. § 1377(b)(1)(A) — 1-YEAR PERIOD beginning on the
//!    date the corporation ceases to be an S corporation
//! 2. § 1377(b)(1)(B) — 120-DAY PERIOD beginning on the
//!    date of a "determination" that the corporation's
//!    S election terminated for a prior year
//! 3. § 1377(b)(1)(C) — 120-DAY PERIOD beginning on the
//!    date of a determination that the corporation had
//!    insufficient adjusted earnings and profits for
//!    a year
//!
//! § 1377(b)(2) DISTRIBUTION DURING PTTP — cash
//! distributions during PTTP are treated as reducing
//! the corporation's accumulated adjustments account
//! (AAA) under § 1368(c) FIRST, then E&P (with
//! dividend tax treatment). This allows shareholders to
//! withdraw remaining AAA tax-free during PTTP after
//! S election termination.
//!
//! § 1377(b)(3) DETERMINATION DEFINITIONS:
//! 1. § 1377(b)(3)(A) — a determination as defined in
//!    § 1313(a)(1) (final IRS or court determination)
//! 2. § 1377(b)(3)(B) — a determination by the
//!    Secretary that the corporation's election under
//!    § 1362 has terminated
//! 3. § 1377(b)(3)(C) — an agreement between the
//!    corporation and the Secretary that the
//!    corporation's S election has terminated
//!
//! § 1377(c) AUTHORITY TO PRESCRIBE REGULATIONS — IRS
//! authorized to prescribe regulations consistent with
//! § 1377 to address allocations among shareholders.
//!
//! Trader-business-owner critical because (1) § 1377(a)(1)
//! daily-assignment method works fine in stable years
//! but creates DISTORTION when annual results swing —
//! e.g., trader-CEO sells stock to partner mid-year with
//! H1 $5M loss and H2 $5M gain; without § 1377(a)(2)
//! election, BOTH parties get pro rata share of $0 net
//! annual income; (2) § 1377(a)(2) election allows
//! economic results to be matched with actual ownership
//! periods; (3) PTTP under § 1377(b) is the critical
//! window during which former S shareholders can extract
//! AAA tax-free before C-corp dividend treatment kicks
//! in; (4) § 1377(b) PTTP coordination with § 1366(d)(3)
//! suspended-loss carryover is key estate planning
//! opportunity when S election terminates due to
//! shareholder death.
//!
//! Distinction from § 706 (partnership): partnerships
//! use varying interest rules under § 706(d) that allow
//! interim closing or proration; S corporations are
//! restricted to § 1377(a)(1) daily method UNLESS
//! § 1377(a)(2) election made on entire-interest
//! termination.
//!
//! Authority: 26 U.S.C. § 1377; § 1377(a)(1) (pro rata
//! share daily method); § 1377(a)(2)(A) (full
//! disposition election); § 1377(a)(2)(B) (§ 302/§ 303
//! redemption election); § 1377(b)(1)(A) (1-year PTTP);
//! § 1377(b)(1)(B) (120-day determination PTTP);
//! § 1377(b)(1)(C) (120-day E&P determination PTTP);
//! § 1377(b)(2) (PTTP AAA distribution); § 1377(b)(3)
//! (determination definitions); § 1377(c) (regulatory
//! authority); § 1361 (S corp election); § 1366 (pass-
//! thru — iter 488); § 1367 (basis adjustments); § 1368
//! (distributions); § 1368(c) (AAA treatment); § 1313(a)(1)
//! (determination); § 1362 (S election); § 1374 (built-
//! in gains); § 1375 (passive investment income); § 302
//! (stock redemption); § 303 (stock redemption); § 706(d)
//! (partnership varying interests); 26 C.F.R. § 1.1377-1
//! (pro rata share); 26 C.F.R. § 1.1377-2; 26 C.F.R.
//! § 1.1377-3 (effective dates); Subchapter S Revision
//! Act of 1982, Pub. L. 97-354 — current § 1377
//! framework.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AllocationMethod {
    Section1377a1DailyMethod,
    Section1377a2TerminatingElection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminationEvent {
    NoTermination,
    FullDisposition,
    Section302Redemption,
    Section303Redemption,
    PartialDisposition, // does not qualify for § 1377(a)(2)
    Death,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PttpPeriod {
    None,
    OneYearAfterCeases,
    OneHundredTwentyDaysDetermination,
    OneHundredTwentyDaysEnpDetermination,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub allocation_method: AllocationMethod,
    pub termination_event: TerminationEvent,
    pub all_affected_shareholders_consented_section_1377a2: bool,
    pub election_filed_with_timely_form_1120s: bool,
    pub shareholder_held_pct_basis_points: u32, // 0-10,000 bps
    pub days_held_in_year: u32, // 0-365 (or 366 leap year)
    pub days_in_year: u32,
    pub corp_annual_income_cents: u64,
    pub pttp_period: PttpPeriod,
    pub distribution_during_pttp_cents: u64,
    pub aaa_balance_cents: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    DailyMethodApplied,
    TerminatingElectionValid,
    TerminatingElectionInvalid,
    PttpDistributionAaaReduction,
    PttpDistributionEnpDividendTreatment,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub shareholder_pro_rata_share_cents: u64,
    pub pttp_distribution_against_aaa_cents: u64,
    pub pttp_distribution_against_enp_cents: u64,
    pub notes: Vec<String>,
}

pub type Section1377Input = Input;
pub type Section1377Result = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "§ 1377(a)(1) PRO RATA SHARE GENERAL RULE: each shareholder's pro rata share of any S corporation item is determined by assigning an equal portion to EACH DAY of the S corporation's taxable year, then dividing that portion PRO RATA among shares outstanding on that day (DAILY ASSIGNMENT + PER-SHARE allocation). Special-day rules per 26 C.F.R. § 1.1377-1(a)(2): shareholder who DISPOSES of stock treated as shareholder for DAY OF DISPOSITION; shareholder who DIES treated as shareholder for DAY OF DEATH.".to_string(),
        "§ 1377(a)(2) TERMINATING ELECTION ('closing of the books'): if shareholder's ENTIRE INTEREST terminates AND ALL AFFECTED SHAREHOLDERS consent, corporation may elect to apply § 1377(a)(1) AS IF taxable year consisted of TWO TAXABLE YEARS, first ending on termination date. Election filed by attaching statement to timely Form 1120-S (including extensions). Two qualifying situations: § 1377(a)(2)(A) full disposition (sale + exchange + gift + other transfer); § 1377(a)(2)(B) § 302 or § 303 redemption.".to_string(),
        "§ 1377(b) POST-TERMINATION TRANSITION PERIOD (PTTP) definitions: § 1377(b)(1)(A) 1-YEAR PERIOD beginning on date corporation ceases to be S corp; § 1377(b)(1)(B) 120-DAY PERIOD beginning on date of determination that S election terminated for prior year; § 1377(b)(1)(C) 120-DAY PERIOD beginning on date of determination that corporation had insufficient adjusted E&P.".to_string(),
        "§ 1377(b)(2) DISTRIBUTION DURING PTTP: cash distributions treated as reducing accumulated adjustments account (AAA) under § 1368(c) FIRST, then E&P (with dividend tax treatment). This allows shareholders to withdraw remaining AAA TAX-FREE during PTTP after S election termination.".to_string(),
        "§ 1377(b)(3) DETERMINATION DEFINITIONS: (A) § 1313(a)(1) final IRS or court determination; (B) Secretary's determination that § 1362 election has terminated; (C) agreement between corporation and Secretary that S election has terminated.".to_string(),
        "Trader-business-owner significance: § 1377(a)(1) DAILY method creates DISTORTION when annual results swing — trader-CEO sells stock mid-year with H1 $5M loss and H2 $5M gain; without § 1377(a)(2) election, BOTH parties get pro rata share of $0 net annual income. § 1377(a)(2) election allows economic results to be matched with actual ownership periods. PTTP under § 1377(b) is critical window for AAA-tax-free distribution after S election termination.".to_string(),
        "Distinction from § 706 (partnership): partnerships use varying interest rules under § 706(d) that allow interim closing or proration; S corporations are restricted to § 1377(a)(1) daily method UNLESS § 1377(a)(2) election made on entire-interest termination.".to_string(),
        "Companion: section_1361 (S corp election), section_1366 (pass-thru — iter 488), section_1367 (basis adjustments), section_1368 (distributions + § 1368(c) AAA), section_1374 (built-in gains), section_1375 (passive investment income); also references § 302 + § 303 + § 1313(a)(1) + § 1362 + § 706(d).".to_string(),
    ];

    // Validate terminating election
    if matches!(input.allocation_method, AllocationMethod::Section1377a2TerminatingElection) {
        let qualifying_event = matches!(
            input.termination_event,
            TerminationEvent::FullDisposition
                | TerminationEvent::Section302Redemption
                | TerminationEvent::Section303Redemption
                | TerminationEvent::Death
        );
        let consent_and_filing = input.all_affected_shareholders_consented_section_1377a2
            && input.election_filed_with_timely_form_1120s;
        if !qualifying_event || !consent_and_filing {
            let mut n = notes;
            n.push(format!(
                "§ 1377(a)(2) terminating election INVALID: qualifying event {} + consent_and_filing {}.",
                qualifying_event, consent_and_filing
            ));
            return Output {
                severity: Severity::TerminatingElectionInvalid,
                shareholder_pro_rata_share_cents: 0,
                pttp_distribution_against_aaa_cents: 0,
                pttp_distribution_against_enp_cents: 0,
                notes: n,
            };
        }

        // Compute terminating-election pro rata share (closing of books at termination date)
        let pro_rata = (input.corp_annual_income_cents as u128)
            .saturating_mul(input.shareholder_held_pct_basis_points as u128)
            .saturating_mul(input.days_held_in_year as u128)
            .checked_div(10_000_u128.saturating_mul(input.days_in_year.max(1) as u128))
            .unwrap_or(0) as u64;

        let (against_aaa, against_enp) = compute_pttp_distribution(input);

        let mut n = notes;
        n.push(format!(
            "§ 1377(a)(2) terminating election VALID: shareholder pro rata share computed using two-period method ${}.{:02} (corp annual income ${}.{:02} × {} bps × {} of {} days).",
            pro_rata / 100, pro_rata % 100,
            input.corp_annual_income_cents / 100, input.corp_annual_income_cents % 100,
            input.shareholder_held_pct_basis_points,
            input.days_held_in_year, input.days_in_year
        ));

        return Output {
            severity: Severity::TerminatingElectionValid,
            shareholder_pro_rata_share_cents: pro_rata,
            pttp_distribution_against_aaa_cents: against_aaa,
            pttp_distribution_against_enp_cents: against_enp,
            notes: n,
        };
    }

    // Default: § 1377(a)(1) daily method
    let days_in_year = input.days_in_year.max(1);
    let pro_rata = (input.corp_annual_income_cents as u128)
        .saturating_mul(input.shareholder_held_pct_basis_points as u128)
        .saturating_mul(input.days_held_in_year as u128)
        .checked_div(10_000_u128.saturating_mul(days_in_year as u128))
        .unwrap_or(0) as u64;

    let (against_aaa, against_enp) = compute_pttp_distribution(input);

    let mut n = notes;
    n.push(format!(
        "§ 1377(a)(1) DAILY METHOD applied: shareholder pro rata share ${}.{:02} (corp annual income ${}.{:02} × {} bps × {} of {} days).",
        pro_rata / 100, pro_rata % 100,
        input.corp_annual_income_cents / 100, input.corp_annual_income_cents % 100,
        input.shareholder_held_pct_basis_points,
        input.days_held_in_year, days_in_year
    ));

    let severity = if matches!(input.pttp_period, PttpPeriod::None) {
        Severity::DailyMethodApplied
    } else if against_enp > 0 {
        Severity::PttpDistributionEnpDividendTreatment
    } else if against_aaa > 0 {
        Severity::PttpDistributionAaaReduction
    } else {
        Severity::DailyMethodApplied
    };

    Output {
        severity,
        shareholder_pro_rata_share_cents: pro_rata,
        pttp_distribution_against_aaa_cents: against_aaa,
        pttp_distribution_against_enp_cents: against_enp,
        notes: n,
    }
}

fn compute_pttp_distribution(input: &Input) -> (u64, u64) {
    if matches!(input.pttp_period, PttpPeriod::None) || input.distribution_during_pttp_cents == 0 {
        return (0, 0);
    }
    // PTTP: § 1377(b)(2) — distribution reduces AAA first, then E&P
    let against_aaa = input.distribution_during_pttp_cents.min(input.aaa_balance_cents);
    let against_enp = input
        .distribution_during_pttp_cents
        .saturating_sub(against_aaa);
    (against_aaa, against_enp)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            allocation_method: AllocationMethod::Section1377a1DailyMethod,
            termination_event: TerminationEvent::NoTermination,
            all_affected_shareholders_consented_section_1377a2: false,
            election_filed_with_timely_form_1120s: false,
            shareholder_held_pct_basis_points: 2500, // 25%
            days_held_in_year: 365,
            days_in_year: 365,
            corp_annual_income_cents: 365_000_00, // $365K annual income
            pttp_period: PttpPeriod::None,
            distribution_during_pttp_cents: 0,
            aaa_balance_cents: 0,
        }
    }

    #[test]
    fn daily_method_full_year_25_percent_shareholder() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::DailyMethodApplied);
        // 25% × $365K × 365/365 = $91,250
        assert_eq!(out.shareholder_pro_rata_share_cents, 91_250_00);
    }

    #[test]
    fn daily_method_half_year_25_percent_shareholder() {
        let mut i = baseline();
        i.days_held_in_year = 183; // first half of year
        let out = check(&i);
        // 25% × $365K × 183/365 = $45,750
        assert_eq!(out.shareholder_pro_rata_share_cents, 45_750_00);
    }

    #[test]
    fn daily_method_50_percent_full_year() {
        let mut i = baseline();
        i.shareholder_held_pct_basis_points = 5000;
        let out = check(&i);
        // 50% × $365K = $182,500
        assert_eq!(out.shareholder_pro_rata_share_cents, 182_500_00);
    }

    #[test]
    fn terminating_election_full_disposition_valid() {
        let mut i = baseline();
        i.allocation_method = AllocationMethod::Section1377a2TerminatingElection;
        i.termination_event = TerminationEvent::FullDisposition;
        i.all_affected_shareholders_consented_section_1377a2 = true;
        i.election_filed_with_timely_form_1120s = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::TerminatingElectionValid);
    }

    #[test]
    fn terminating_election_partial_disposition_invalid() {
        let mut i = baseline();
        i.allocation_method = AllocationMethod::Section1377a2TerminatingElection;
        i.termination_event = TerminationEvent::PartialDisposition;
        i.all_affected_shareholders_consented_section_1377a2 = true;
        i.election_filed_with_timely_form_1120s = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::TerminatingElectionInvalid);
    }

    #[test]
    fn terminating_election_no_consent_invalid() {
        let mut i = baseline();
        i.allocation_method = AllocationMethod::Section1377a2TerminatingElection;
        i.termination_event = TerminationEvent::FullDisposition;
        i.all_affected_shareholders_consented_section_1377a2 = false;
        i.election_filed_with_timely_form_1120s = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::TerminatingElectionInvalid);
    }

    #[test]
    fn terminating_election_no_form_1120s_invalid() {
        let mut i = baseline();
        i.allocation_method = AllocationMethod::Section1377a2TerminatingElection;
        i.termination_event = TerminationEvent::FullDisposition;
        i.all_affected_shareholders_consented_section_1377a2 = true;
        i.election_filed_with_timely_form_1120s = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::TerminatingElectionInvalid);
    }

    #[test]
    fn terminating_election_section_302_redemption_valid() {
        let mut i = baseline();
        i.allocation_method = AllocationMethod::Section1377a2TerminatingElection;
        i.termination_event = TerminationEvent::Section302Redemption;
        i.all_affected_shareholders_consented_section_1377a2 = true;
        i.election_filed_with_timely_form_1120s = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::TerminatingElectionValid);
    }

    #[test]
    fn terminating_election_section_303_redemption_valid() {
        let mut i = baseline();
        i.allocation_method = AllocationMethod::Section1377a2TerminatingElection;
        i.termination_event = TerminationEvent::Section303Redemption;
        i.all_affected_shareholders_consented_section_1377a2 = true;
        i.election_filed_with_timely_form_1120s = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::TerminatingElectionValid);
    }

    #[test]
    fn terminating_election_death_valid() {
        let mut i = baseline();
        i.allocation_method = AllocationMethod::Section1377a2TerminatingElection;
        i.termination_event = TerminationEvent::Death;
        i.all_affected_shareholders_consented_section_1377a2 = true;
        i.election_filed_with_timely_form_1120s = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::TerminatingElectionValid);
    }

    #[test]
    fn pttp_distribution_against_aaa() {
        let mut i = baseline();
        i.pttp_period = PttpPeriod::OneYearAfterCeases;
        i.distribution_during_pttp_cents = 100_000_00;
        i.aaa_balance_cents = 150_000_00; // ample AAA
        let out = check(&i);
        assert_eq!(out.pttp_distribution_against_aaa_cents, 100_000_00);
        assert_eq!(out.pttp_distribution_against_enp_cents, 0);
        assert_eq!(out.severity, Severity::PttpDistributionAaaReduction);
    }

    #[test]
    fn pttp_distribution_partial_against_aaa_then_enp() {
        let mut i = baseline();
        i.pttp_period = PttpPeriod::OneYearAfterCeases;
        i.distribution_during_pttp_cents = 100_000_00;
        i.aaa_balance_cents = 30_000_00; // small AAA
        let out = check(&i);
        assert_eq!(out.pttp_distribution_against_aaa_cents, 30_000_00);
        assert_eq!(out.pttp_distribution_against_enp_cents, 70_000_00);
        assert_eq!(out.severity, Severity::PttpDistributionEnpDividendTreatment);
    }

    #[test]
    fn pttp_120_day_determination_period() {
        let mut i = baseline();
        i.pttp_period = PttpPeriod::OneHundredTwentyDaysDetermination;
        i.distribution_during_pttp_cents = 50_000_00;
        i.aaa_balance_cents = 100_000_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PttpDistributionAaaReduction);
    }

    #[test]
    fn pttp_120_day_enp_determination_period() {
        let mut i = baseline();
        i.pttp_period = PttpPeriod::OneHundredTwentyDaysEnpDetermination;
        i.distribution_during_pttp_cents = 50_000_00;
        i.aaa_balance_cents = 100_000_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PttpDistributionAaaReduction);
    }

    #[test]
    fn no_pttp_no_distribution_daily_method() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::DailyMethodApplied);
        assert_eq!(out.pttp_distribution_against_aaa_cents, 0);
        assert_eq!(out.pttp_distribution_against_enp_cents, 0);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1377(a)(1)"));
        assert!(joined.contains("§ 1377(a)(2)"));
        assert!(joined.contains("§ 1377(a)(2)(A)"));
        assert!(joined.contains("§ 1377(a)(2)(B)"));
        assert!(joined.contains("§ 1377(b)"));
        assert!(joined.contains("§ 1377(b)(1)(A)"));
        assert!(joined.contains("§ 1377(b)(1)(B)"));
        assert!(joined.contains("§ 1377(b)(1)(C)"));
        assert!(joined.contains("§ 1377(b)(2)"));
        assert!(joined.contains("§ 1377(b)(3)"));
        assert!(joined.contains("§ 1313(a)(1)"));
        assert!(joined.contains("§ 1362"));
        assert!(joined.contains("§ 1368(c)"));
        assert!(joined.contains("§ 302"));
        assert!(joined.contains("§ 303"));
        assert!(joined.contains("§ 706(d)"));
        assert!(joined.contains("26 C.F.R. § 1.1377-1"));
    }

    #[test]
    fn note_pins_daily_assignment_method() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("DAILY ASSIGNMENT"));
        assert!(joined.contains("PER-SHARE"));
        assert!(joined.contains("DAY OF DISPOSITION"));
        assert!(joined.contains("DAY OF DEATH"));
    }

    #[test]
    fn note_pins_terminating_election_requirements() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("closing of the books"));
        assert!(joined.contains("ENTIRE INTEREST"));
        assert!(joined.contains("ALL AFFECTED SHAREHOLDERS"));
        assert!(joined.contains("TWO TAXABLE YEARS"));
        assert!(joined.contains("Form 1120-S"));
    }

    #[test]
    fn note_pins_two_qualifying_situations() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1377(a)(2)(A) full disposition"));
        assert!(joined.contains("§ 1377(a)(2)(B) § 302 or § 303 redemption"));
    }

    #[test]
    fn note_pins_three_pttp_periods() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("1-YEAR PERIOD"));
        assert!(joined.contains("120-DAY PERIOD"));
    }

    #[test]
    fn note_pins_pttp_aaa_distribution_tax_free() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("DISTRIBUTION DURING PTTP"));
        assert!(joined.contains("TAX-FREE"));
        assert!(joined.contains("dividend tax treatment"));
    }

    #[test]
    fn note_pins_section_706d_distinction() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 706 (partnership)"));
        assert!(joined.contains("§ 706(d)"));
        assert!(joined.contains("interim closing or proration"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("section_1361"));
        assert!(joined.contains("section_1366"));
        assert!(joined.contains("section_1367"));
        assert!(joined.contains("section_1368"));
        assert!(joined.contains("section_1374"));
        assert!(joined.contains("section_1375"));
    }

    #[test]
    fn truth_table_five_severity_cells() {
        // Daily method
        let c1 = check(&baseline());
        assert_eq!(c1.severity, Severity::DailyMethodApplied);

        // Terminating election valid
        let mut c2_input = baseline();
        c2_input.allocation_method = AllocationMethod::Section1377a2TerminatingElection;
        c2_input.termination_event = TerminationEvent::FullDisposition;
        c2_input.all_affected_shareholders_consented_section_1377a2 = true;
        c2_input.election_filed_with_timely_form_1120s = true;
        let c2 = check(&c2_input);
        assert_eq!(c2.severity, Severity::TerminatingElectionValid);

        // Terminating election invalid
        let mut c3_input = baseline();
        c3_input.allocation_method = AllocationMethod::Section1377a2TerminatingElection;
        c3_input.termination_event = TerminationEvent::PartialDisposition;
        let c3 = check(&c3_input);
        assert_eq!(c3.severity, Severity::TerminatingElectionInvalid);

        // PTTP AAA reduction
        let mut c4_input = baseline();
        c4_input.pttp_period = PttpPeriod::OneYearAfterCeases;
        c4_input.distribution_during_pttp_cents = 50_000_00;
        c4_input.aaa_balance_cents = 100_000_00;
        let c4 = check(&c4_input);
        assert_eq!(c4.severity, Severity::PttpDistributionAaaReduction);

        // PTTP E&P dividend
        let mut c5_input = baseline();
        c5_input.pttp_period = PttpPeriod::OneYearAfterCeases;
        c5_input.distribution_during_pttp_cents = 200_000_00;
        c5_input.aaa_balance_cents = 50_000_00;
        let c5 = check(&c5_input);
        assert_eq!(c5.severity, Severity::PttpDistributionEnpDividendTreatment);
    }

    #[test]
    fn saturating_arithmetic_overflow_defense() {
        let mut i = baseline();
        i.corp_annual_income_cents = u64::MAX;
        i.shareholder_held_pct_basis_points = 10000;
        i.days_held_in_year = 365;
        let out = check(&i);
        // No panic; saturating math
        assert_eq!(out.severity, Severity::DailyMethodApplied);
    }

    #[test]
    fn zero_days_zero_share() {
        let mut i = baseline();
        i.days_held_in_year = 0;
        let out = check(&i);
        assert_eq!(out.shareholder_pro_rata_share_cents, 0);
    }

    #[test]
    fn zero_corp_income_zero_share() {
        let mut i = baseline();
        i.corp_annual_income_cents = 0;
        let out = check(&i);
        assert_eq!(out.shareholder_pro_rata_share_cents, 0);
    }

    #[test]
    fn realistic_mid_year_shareholder_swap() {
        // Trader-CEO with 50% interest sells mid-year (day 183 of 365)
        // Corp has $1M annual income; without § 1377(a)(2) daily method
        // assigns equal portion to each day
        let mut i = baseline();
        i.allocation_method = AllocationMethod::Section1377a1DailyMethod;
        i.shareholder_held_pct_basis_points = 5000;
        i.days_held_in_year = 183;
        i.corp_annual_income_cents = 1_000_000_00;
        let out = check(&i);
        // 50% × $1M × 183/365 ≈ $250,684.93
        // u128 calc: 1_000_000_00 × 5000 × 183 = 9.15 × 10^14
        // / (10000 × 365) = 9.15e14 / 3.65e6 = 2.5068e8 = $250,684.93
        assert_eq!(out.shareholder_pro_rata_share_cents, 250_684_93);
    }
}
