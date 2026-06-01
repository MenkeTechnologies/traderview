//! IRC §6651 — Failure to file tax return or to pay tax.
//!
//! The most commonly asserted civil tax penalty. Two parallel
//! tracks — late filing (§6651(a)(1)) and late paying
//! (§6651(a)(2)) — that interact via a reduction rule
//! (§6651(c)(1)) and a minimum-penalty floor for returns over 60
//! days late (§6651(g)). Fraud doubles the late-filing rate via
//! §6651(f).
//!
//! **§6651(a)(1) failure to file**: 5% of the tax required to be
//! shown on the return per month (or fraction) the return is
//! late, capped at **25%** (i.e., 5 months × 5% = 25% max).
//!
//! **§6651(a)(2) failure to pay**: 0.5% of unpaid tax per month
//! (or fraction), capped at **25%** (50 months max). Reduced to
//! **0.25%/month** under §6651(h) when (i) the return was filed
//! by the due date including extensions AND (ii) a §6159
//! installment agreement is in effect.
//!
//! **§6651(c)(1) interaction**: when both penalties apply for the
//! same month, the failure-to-file penalty is reduced by the
//! failure-to-pay penalty amount. Effect: 4.5%/month FTF +
//! 0.5%/month FTP = 5%/month combined (instead of 5.5%/month).
//!
//! **§6651(f) fraud uplift**: if the failure to file is
//! fraudulent, the rate increases to **15%/month** with a **75%
//! maximum** (5 months × 15%).
//!
//! **§6651(g) minimum penalty** (returns more than 60 days late):
//! the minimum penalty is the LESSER of (i) the inflation-adjusted
//! statutory amount published annually by Rev. Proc. (2024: $485;
//! 2025: $510) OR (ii) 100% of the tax required to be shown.
//!
//! **Reasonable-cause defense** (§6651(a) flush language): no
//! penalty when failure is due to reasonable cause and not willful
//! neglect. Defense is unavailable for §6651(f) fraud.
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 6651](https://www.law.cornell.edu/uscode/text/26/6651),
//! [IRS — Failure to File Penalty](https://www.irs.gov/payments/failure-to-file-penalty),
//! [Tom Talks Taxes — Overview of the Failure to File Penalty](https://www.tomtalkstaxes.com/p/an-overview-of-the-failure-to-file),
//! [Wolters Kluwer AnswerConnect — Interaction of FTF / FTP penalties](https://answerconnect.cch.com/document/arp283d1401907b6f1000b334001b78be8c780170/federal/irc/explanation/interaction-of-failure-to-file-or-pay-penalties).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section6651Input {
    /// Tax required to be shown on the return (numerator for both
    /// penalties).
    pub tax_required_dollars: i64,
    /// Months (or fraction of months) the return is late. Capped
    /// at the statutory max internally.
    pub months_late_filing: u32,
    /// Months (or fraction) the tax has gone unpaid.
    pub months_late_paying: u32,
    /// True if the return is more than 60 days late (triggers
    /// §6651(g) minimum-penalty floor).
    pub return_more_than_60_days_late: bool,
    /// Inflation-adjusted minimum-penalty amount under §6651(g)
    /// for the relevant tax year (Rev. Proc.: 2024 = $485;
    /// 2025 = $510). Caller-supplied so the module remains
    /// year-agnostic.
    pub minimum_penalty_inflation_adjusted_dollars: i64,
    /// True if a §6159 installment agreement is in effect.
    pub installment_agreement_in_effect: bool,
    /// True if the return was filed by the due date including
    /// extensions (precondition for §6651(h) installment-rate
    /// reduction).
    pub timely_filed_with_extension: bool,
    /// True if the failure to file is fraudulent (§6651(f)).
    pub failure_to_file_is_fraudulent: bool,
    /// True if reasonable-cause-and-not-willful-neglect defense
    /// is established. Unavailable for §6651(f) fraud.
    pub reasonable_cause_established: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section6651Result {
    /// §6651(a)(1) failure-to-file penalty before §6651(c) reduction.
    pub failure_to_file_penalty_dollars: i64,
    /// §6651(a)(2) failure-to-pay penalty.
    pub failure_to_pay_penalty_dollars: i64,
    /// §6651(c)(1) reduction amount applied to FTF for months
    /// where both apply.
    pub section_6651c1_reduction_dollars: i64,
    /// FTF after §6651(c)(1) reduction.
    pub net_failure_to_file_penalty_dollars: i64,
    /// True if §6651(f) fraud uplift to 15%/month was applied.
    pub fraud_uplift_applied: bool,
    /// True if §6651(g) minimum-penalty floor was triggered.
    pub minimum_penalty_floor_triggered: bool,
    /// Final minimum-penalty amount (lesser of inflation-adjusted
    /// amount or 100% of tax).
    pub minimum_penalty_amount_dollars: i64,
    /// True if §6651(h) installment-rate reduction to 0.25%/month
    /// applied.
    pub installment_rate_reduction_applied: bool,
    /// True if reasonable-cause defense zeroed the FTF/FTP
    /// penalty. Cannot zero the §6651(f) fraud uplift.
    pub reasonable_cause_defense_applies: bool,
    /// Final penalty after all adjustments.
    pub total_penalty_dollars: i64,
    pub citation: String,
    pub note: String,
}

const FTF_RATE_BP: u32 = 500; // 5%/month
const FTF_MAX_BP: u32 = 2500; // 25%
const FTP_RATE_BP: u32 = 50; // 0.5%/month
const FTP_INSTALLMENT_RATE_BP: u32 = 25; // 0.25%/month
const FTP_MAX_BP: u32 = 2500; // 25%
const FRAUD_FTF_RATE_BP: u32 = 1500; // 15%/month
const FRAUD_FTF_MAX_BP: u32 = 7500; // 75%

pub fn compute(input: &Section6651Input) -> Section6651Result {
    let tax = input.tax_required_dollars.max(0);

    // §6651(h): installment-rate reduction to 0.25%/month when
    // installment agreement in effect AND return filed timely with
    // extension.
    let installment_rate_applies =
        input.installment_agreement_in_effect && input.timely_filed_with_extension;
    let ftp_rate_bp = if installment_rate_applies {
        FTP_INSTALLMENT_RATE_BP
    } else {
        FTP_RATE_BP
    };

    // §6651(a)(2) FTP: 0.5% × months, capped at 25%.
    let ftp_total_bp = (input.months_late_paying * ftp_rate_bp).min(FTP_MAX_BP);
    let ftp_penalty = ((tax as i128) * (ftp_total_bp as i128) / 10_000) as i64;

    // §6651(a)(1) FTF or §6651(f) fraud-uplift FTF.
    let (ftf_rate_bp, ftf_max_bp) = if input.failure_to_file_is_fraudulent {
        (FRAUD_FTF_RATE_BP, FRAUD_FTF_MAX_BP)
    } else {
        (FTF_RATE_BP, FTF_MAX_BP)
    };
    let ftf_total_bp = (input.months_late_filing * ftf_rate_bp).min(ftf_max_bp);
    let ftf_penalty_gross = ((tax as i128) * (ftf_total_bp as i128) / 10_000) as i64;

    // §6651(c)(1) reduction — FTF reduced by FTP amount during
    // overlapping months. Approximation: reduce FTF by FTP penalty
    // for the months where both apply. Simplified: reduce by FTP
    // amount up to FTF amount (no double-counting). Not applicable
    // to fraud path (which already uses higher rate).
    let overlap_months = input.months_late_filing.min(input.months_late_paying);
    let section_6651c1_reduction = if input.failure_to_file_is_fraudulent {
        0
    } else {
        let overlap_ftp_bp =
            (overlap_months * ftp_rate_bp).min(FTP_MAX_BP);
        ((tax as i128) * (overlap_ftp_bp as i128) / 10_000) as i64
    };

    let ftf_penalty_net = (ftf_penalty_gross - section_6651c1_reduction).max(0);

    // §6651(g) minimum penalty for returns > 60 days late:
    // lesser of inflation-adjusted amount or 100% of tax.
    let (minimum_floor_triggered, minimum_penalty) = if input.return_more_than_60_days_late {
        let floor = input
            .minimum_penalty_inflation_adjusted_dollars
            .max(0)
            .min(tax);
        (true, floor)
    } else {
        (false, 0)
    };

    // Reasonable-cause defense — not applicable to fraud.
    let reasonable_cause_defense_applies = input.reasonable_cause_established
        && !input.failure_to_file_is_fraudulent;

    let mut total_penalty = if reasonable_cause_defense_applies {
        0
    } else {
        ftf_penalty_net + ftp_penalty
    };

    // Apply minimum-penalty floor — total cannot fall below the
    // §6651(g) amount when triggered.
    if minimum_floor_triggered && !reasonable_cause_defense_applies && total_penalty < minimum_penalty
    {
        total_penalty = minimum_penalty;
    }

    let mut note_parts =
        vec![format!("Tax required ${}; months late filing {}; months late paying {}", tax, input.months_late_filing, input.months_late_paying)];
    if installment_rate_applies {
        note_parts.push("§6651(h) installment-rate 0.25%/month applies".to_string());
    }
    if input.failure_to_file_is_fraudulent {
        note_parts.push("§6651(f) fraud uplift 15%/month / 75% max applied".to_string());
    }
    note_parts.push(format!(
        "FTF (gross) ${}; FTP ${}; §6651(c)(1) reduction ${}; FTF (net) ${}",
        ftf_penalty_gross, ftp_penalty, section_6651c1_reduction, ftf_penalty_net,
    ));
    if minimum_floor_triggered {
        note_parts.push(format!(
            "§6651(g) minimum-penalty floor ${} (lesser of inflation amount ${} or 100% tax ${})",
            minimum_penalty,
            input.minimum_penalty_inflation_adjusted_dollars,
            tax,
        ));
    }
    if reasonable_cause_defense_applies {
        note_parts.push("reasonable-cause defense applies — penalty zeroed".to_string());
    }
    note_parts.push(format!("total penalty ${}", total_penalty));

    Section6651Result {
        failure_to_file_penalty_dollars: ftf_penalty_gross,
        failure_to_pay_penalty_dollars: ftp_penalty,
        section_6651c1_reduction_dollars: section_6651c1_reduction,
        net_failure_to_file_penalty_dollars: ftf_penalty_net,
        fraud_uplift_applied: input.failure_to_file_is_fraudulent,
        minimum_penalty_floor_triggered: minimum_floor_triggered,
        minimum_penalty_amount_dollars: minimum_penalty,
        installment_rate_reduction_applied: installment_rate_applies,
        reasonable_cause_defense_applies,
        total_penalty_dollars: total_penalty,
        citation:
            "IRC §6651(a)(1) failure-to-file penalty (5%/month / 25% max); §6651(a)(2) failure-to-pay penalty (0.5%/month / 25% max); §6651(c)(1) FTF reduced by FTP for overlapping months (net 4.5%/month FTF + 0.5%/month FTP = 5%/month combined); §6651(f) fraud uplift to 15%/month / 75% max; §6651(g) minimum-penalty floor for returns > 60 days late (lesser of inflation-adjusted amount or 100% tax); §6651(h) installment-rate reduction to 0.25%/month when timely-filed-with-extension + §6159 agreement in effect; reasonable-cause-and-not-willful-neglect defense (unavailable for §6651(f) fraud)"
                .to_string(),
        note: note_parts.join("; ") + ".",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section6651Input {
        Section6651Input {
            tax_required_dollars: 100_000,
            months_late_filing: 0,
            months_late_paying: 0,
            return_more_than_60_days_late: false,
            minimum_penalty_inflation_adjusted_dollars: 510,
            installment_agreement_in_effect: false,
            timely_filed_with_extension: false,
            failure_to_file_is_fraudulent: false,
            reasonable_cause_established: false,
        }
    }

    // ── §6651(a)(1) failure-to-file ─────────────────────────────────

    #[test]
    fn ftf_one_month_5_pct() {
        let mut i = base();
        i.months_late_filing = 1;
        let r = compute(&i);
        // 5% × $100k = $5k.
        assert_eq!(r.failure_to_file_penalty_dollars, 5_000);
    }

    #[test]
    fn ftf_three_months_15_pct() {
        let mut i = base();
        i.months_late_filing = 3;
        let r = compute(&i);
        assert_eq!(r.failure_to_file_penalty_dollars, 15_000);
    }

    #[test]
    fn ftf_max_capped_at_25_pct() {
        let mut i = base();
        i.months_late_filing = 100;
        let r = compute(&i);
        // 25% cap → $25k.
        assert_eq!(r.failure_to_file_penalty_dollars, 25_000);
    }

    // ── §6651(a)(2) failure-to-pay ─────────────────────────────────

    #[test]
    fn ftp_one_month_half_pct() {
        let mut i = base();
        i.months_late_paying = 1;
        let r = compute(&i);
        // 0.5% × $100k = $500.
        assert_eq!(r.failure_to_pay_penalty_dollars, 500);
    }

    #[test]
    fn ftp_max_capped_at_25_pct_50_months() {
        let mut i = base();
        i.months_late_paying = 100;
        let r = compute(&i);
        assert_eq!(r.failure_to_pay_penalty_dollars, 25_000);
    }

    // ── §6651(h) installment reduction ─────────────────────────────

    #[test]
    fn installment_agreement_and_timely_filing_reduces_ftp_to_025_pct() {
        let mut i = base();
        i.months_late_paying = 4;
        i.installment_agreement_in_effect = true;
        i.timely_filed_with_extension = true;
        let r = compute(&i);
        assert!(r.installment_rate_reduction_applied);
        // 0.25% × 4 × $100k = $1,000.
        assert_eq!(r.failure_to_pay_penalty_dollars, 1_000);
    }

    #[test]
    fn installment_without_timely_filing_no_reduction() {
        let mut i = base();
        i.months_late_paying = 4;
        i.installment_agreement_in_effect = true;
        i.timely_filed_with_extension = false;
        let r = compute(&i);
        assert!(!r.installment_rate_reduction_applied);
        // 0.5% × 4 × $100k = $2,000.
        assert_eq!(r.failure_to_pay_penalty_dollars, 2_000);
    }

    // ── §6651(c)(1) interaction ────────────────────────────────────

    #[test]
    fn combined_ftf_ftp_one_month_5_pct_combined() {
        // Both 1 month late: FTF gross = $5k, FTP = $500, reduction
        // = $500 (overlap 1 month × 0.5% × $100k), FTF net = $4,500.
        // Combined = $5,000.
        let mut i = base();
        i.months_late_filing = 1;
        i.months_late_paying = 1;
        let r = compute(&i);
        assert_eq!(r.failure_to_file_penalty_dollars, 5_000);
        assert_eq!(r.failure_to_pay_penalty_dollars, 500);
        assert_eq!(r.section_6651c1_reduction_dollars, 500);
        assert_eq!(r.net_failure_to_file_penalty_dollars, 4_500);
        assert_eq!(r.total_penalty_dollars, 5_000);
    }

    #[test]
    fn combined_three_months_15_pct_combined() {
        // 3 months both: FTF gross $15k − $1,500 reduction = $13,500
        // net; FTP $1,500. Combined $15,000.
        let mut i = base();
        i.months_late_filing = 3;
        i.months_late_paying = 3;
        let r = compute(&i);
        assert_eq!(r.section_6651c1_reduction_dollars, 1_500);
        assert_eq!(r.total_penalty_dollars, 15_000);
    }

    // ── §6651(f) fraud uplift ──────────────────────────────────────

    #[test]
    fn fraud_uplift_15_pct_per_month() {
        let mut i = base();
        i.months_late_filing = 1;
        i.failure_to_file_is_fraudulent = true;
        let r = compute(&i);
        assert!(r.fraud_uplift_applied);
        // 15% × $100k = $15k.
        assert_eq!(r.failure_to_file_penalty_dollars, 15_000);
    }

    #[test]
    fn fraud_uplift_capped_at_75_pct() {
        let mut i = base();
        i.months_late_filing = 10;
        i.failure_to_file_is_fraudulent = true;
        let r = compute(&i);
        // 75% × $100k = $75k.
        assert_eq!(r.failure_to_file_penalty_dollars, 75_000);
    }

    #[test]
    fn fraud_no_c1_reduction() {
        // §6651(c)(1) reduction does not apply to fraud path.
        let mut i = base();
        i.months_late_filing = 3;
        i.months_late_paying = 3;
        i.failure_to_file_is_fraudulent = true;
        let r = compute(&i);
        assert_eq!(r.section_6651c1_reduction_dollars, 0);
    }

    // ── §6651(g) minimum penalty ───────────────────────────────────

    #[test]
    fn minimum_penalty_triggered_when_return_60_days_late() {
        let mut i = base();
        i.tax_required_dollars = 100_000;
        i.months_late_filing = 3;
        i.return_more_than_60_days_late = true;
        i.minimum_penalty_inflation_adjusted_dollars = 510;
        let r = compute(&i);
        assert!(r.minimum_penalty_floor_triggered);
        // 3% × $100k = $15k > $510 floor → uses higher of the two.
        assert_eq!(r.total_penalty_dollars, 15_000);
    }

    #[test]
    fn minimum_penalty_floor_binds_when_normal_penalty_smaller() {
        let mut i = base();
        i.tax_required_dollars = 5_000;
        i.months_late_filing = 1;
        i.return_more_than_60_days_late = true;
        i.minimum_penalty_inflation_adjusted_dollars = 510;
        let r = compute(&i);
        // FTF 5% × $5k = $250 < $510 floor → uses $510.
        assert_eq!(r.minimum_penalty_amount_dollars, 510);
        assert_eq!(r.total_penalty_dollars, 510);
    }

    #[test]
    fn minimum_penalty_capped_at_100_pct_tax() {
        // When tax < inflation amount, floor is the 100% tax cap.
        let mut i = base();
        i.tax_required_dollars = 100;
        i.months_late_filing = 1;
        i.return_more_than_60_days_late = true;
        i.minimum_penalty_inflation_adjusted_dollars = 510;
        let r = compute(&i);
        // FTF 5% × $100 = $5; floor is min($510, $100) = $100.
        assert_eq!(r.minimum_penalty_amount_dollars, 100);
        assert_eq!(r.total_penalty_dollars, 100);
    }

    #[test]
    fn not_more_than_60_days_late_no_minimum_floor() {
        let mut i = base();
        i.tax_required_dollars = 1_000;
        i.months_late_filing = 1;
        i.return_more_than_60_days_late = false;
        let r = compute(&i);
        assert!(!r.minimum_penalty_floor_triggered);
        // FTF 5% × $1k = $50.
        assert_eq!(r.total_penalty_dollars, 50);
    }

    // ── Reasonable-cause defense ───────────────────────────────────

    #[test]
    fn reasonable_cause_zeros_normal_penalty() {
        let mut i = base();
        i.months_late_filing = 3;
        i.months_late_paying = 3;
        i.reasonable_cause_established = true;
        let r = compute(&i);
        assert!(r.reasonable_cause_defense_applies);
        assert_eq!(r.total_penalty_dollars, 0);
    }

    #[test]
    fn reasonable_cause_does_not_zero_fraud_penalty() {
        let mut i = base();
        i.months_late_filing = 3;
        i.failure_to_file_is_fraudulent = true;
        i.reasonable_cause_established = true;
        let r = compute(&i);
        assert!(!r.reasonable_cause_defense_applies);
        assert_eq!(r.failure_to_file_penalty_dollars, 45_000);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§6651(a)(1)"));
        assert!(r.citation.contains("§6651(a)(2)"));
        assert!(r.citation.contains("§6651(c)(1)"));
        assert!(r.citation.contains("§6651(f)"));
        assert!(r.citation.contains("§6651(g)"));
        assert!(r.citation.contains("§6651(h)"));
        assert!(r.citation.contains("5%/month"));
        assert!(r.citation.contains("0.5%/month"));
        assert!(r.citation.contains("15%/month"));
        assert!(r.citation.contains("0.25%/month"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_includes_installment_path() {
        let mut i = base();
        i.months_late_paying = 1;
        i.installment_agreement_in_effect = true;
        i.timely_filed_with_extension = true;
        let r = compute(&i);
        assert!(r.note.contains("§6651(h)"));
    }

    #[test]
    fn note_includes_fraud_path() {
        let mut i = base();
        i.months_late_filing = 1;
        i.failure_to_file_is_fraudulent = true;
        let r = compute(&i);
        assert!(r.note.contains("§6651(f)"));
    }

    #[test]
    fn note_includes_minimum_floor_when_triggered() {
        let mut i = base();
        i.tax_required_dollars = 5_000;
        i.months_late_filing = 1;
        i.return_more_than_60_days_late = true;
        let r = compute(&i);
        assert!(r.note.contains("§6651(g) minimum-penalty floor"));
    }

    // ── Precision ──────────────────────────────────────────────────

    #[test]
    fn very_large_billion_dollar_tax_precision() {
        let mut i = base();
        i.tax_required_dollars = 1_000_000_000;
        i.months_late_filing = 5;
        i.months_late_paying = 5;
        let r = compute(&i);
        // FTF gross = 25% × $1B = $250M; FTP = 2.5% × $1B = $25M;
        // Reduction = $25M; FTF net = $225M; total = $250M.
        assert_eq!(r.failure_to_file_penalty_dollars, 250_000_000);
        assert_eq!(r.failure_to_pay_penalty_dollars, 25_000_000);
        assert_eq!(r.total_penalty_dollars, 250_000_000);
    }
}
