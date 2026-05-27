//! §199A Qualified Business Income (QBI) deduction calculator.
//!
//! 20% deduction on QBI for pass-through businesses (Schedule C, S-corp,
//! partnership). Phase-out kicks in above the income thresholds, and
//! "specified service trades" (SSTBs — health, law, consulting, athletics,
//! financial services) lose the deduction entirely past the threshold.
//!
//! **Traders ARE SSTBs** under final §199A regs (financial services).
//! That means full QBI under the threshold, full phase-out above the
//! ceiling, partial in between.
//!
//! 2024 thresholds: $191,950 single / $383,900 MFJ (start of phase-out).
//! 2025: $197,300 single / $394,600 MFJ. Phase-out range is $50k single /
//! $100k MFJ wide.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    MarriedJoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QbiInput {
    /// Net profit from Schedule C (QBI before the deduction).
    pub qbi: Decimal,
    /// Total taxable income for the year (used for threshold tests).
    pub taxable_income: Decimal,
    pub filing_status: FilingStatus,
    /// True for traders (financial services = SSTB under §199A regs).
    pub is_sstb: bool,
    pub tax_year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QbiReport {
    /// Final §199A deduction. Goes on Form 1040 line 13.
    pub deduction: Decimal,
    /// Cap from 20% × QBI.
    pub component_pct_qbi: Decimal,
    /// Cap from 20% × (taxable income − net capital gains). We don't model
    /// net capital gains explicitly — caller subtracts if relevant.
    pub component_pct_taxable: Decimal,
    /// Phase-out fraction applied (0.0 = no phase-out, 1.0 = fully out).
    pub phase_out_fraction: f64,
    pub note: String,
}

fn threshold(year: i32, status: FilingStatus) -> Decimal {
    let d = |s: &str| Decimal::from_str(s).unwrap();
    match (year, status) {
        (2024, FilingStatus::Single)       => d("191950"),
        (2024, FilingStatus::MarriedJoint) => d("383900"),
        (2025, FilingStatus::Single)       => d("197300"),
        (2025, FilingStatus::MarriedJoint) => d("394600"),
        (2026, FilingStatus::Single)       => d("203500"),     // estimate
        (2026, FilingStatus::MarriedJoint) => d("407000"),     // estimate
        // Default: use 2026 estimate. Don't panic on future years.
        (_,    FilingStatus::Single)       => d("203500"),
        (_,    FilingStatus::MarriedJoint) => d("407000"),
    }
}

/// Phase-out range width above the threshold.
fn phase_out_range(status: FilingStatus) -> Decimal {
    match status {
        FilingStatus::Single       => Decimal::from(50_000),
        FilingStatus::MarriedJoint => Decimal::from(100_000),
    }
}

pub fn compute(input: &QbiInput) -> QbiReport {
    let mut r = QbiReport::default();
    let twenty_pct = Decimal::from_str("0.20").unwrap();
    r.component_pct_qbi = input.qbi.max(Decimal::ZERO) * twenty_pct;
    r.component_pct_taxable = input.taxable_income.max(Decimal::ZERO) * twenty_pct;

    let lower = threshold(input.tax_year, input.filing_status);
    let upper = lower + phase_out_range(input.filing_status);

    if input.taxable_income <= lower {
        // Fully under the threshold — full 20% (SSTB or not).
        let raw = r.component_pct_qbi.min(r.component_pct_taxable);
        r.deduction = raw.max(Decimal::ZERO);
        r.note = "below threshold — full 20%".into();
        return r;
    }

    if input.is_sstb && input.taxable_income >= upper {
        // SSTBs fully phase out above the ceiling — trader gets $0.
        r.deduction = Decimal::ZERO;
        r.phase_out_fraction = 1.0;
        r.note = "SSTB fully phased out (trader = financial services)".into();
        return r;
    }

    if input.is_sstb {
        // Linear phase-out for SSTBs between threshold and ceiling.
        let over = input.taxable_income - lower;
        let frac_phased_out = over / phase_out_range(input.filing_status);
        let frac_remaining = Decimal::ONE - frac_phased_out;
        let raw = r.component_pct_qbi.min(r.component_pct_taxable);
        r.deduction = (raw * frac_remaining).max(Decimal::ZERO);
        r.phase_out_fraction = frac_phased_out.to_string().parse().unwrap_or(0.0);
        r.note = format!("SSTB phase-out: {} remaining",
            (Decimal::ONE - frac_phased_out));
        return r;
    }

    // Non-SSTB above threshold: W-2-wages / UBIA limit kicks in (not
    // modeled — caller supplies a manual override if needed). For now
    // grant the full 20% capped at taxable.
    let raw = r.component_pct_qbi.min(r.component_pct_taxable);
    r.deduction = raw.max(Decimal::ZERO);
    r.note = "non-SSTB above threshold (W-2/UBIA cap not modeled)".into();
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    #[test]
    fn under_threshold_full_20_percent_qbi() {
        let r = compute(&QbiInput {
            qbi: d("100000"),
            taxable_income: d("150000"),    // well under 2024 single threshold
            filing_status: FilingStatus::Single,
            is_sstb: true,
            tax_year: 2024,
        });
        assert_eq!(r.deduction, d("20000.00"));
        assert_eq!(r.phase_out_fraction, 0.0);
    }

    #[test]
    fn sstb_fully_phased_out_at_ceiling() {
        // Trader (SSTB) with $300k taxable in 2024 single → past
        // $191,950 + $50,000 = $241,950 ceiling → $0 deduction.
        let r = compute(&QbiInput {
            qbi: d("250000"),
            taxable_income: d("300000"),
            filing_status: FilingStatus::Single,
            is_sstb: true,
            tax_year: 2024,
        });
        assert_eq!(r.deduction, Decimal::ZERO);
        assert_eq!(r.phase_out_fraction, 1.0);
    }

    #[test]
    fn sstb_partial_phase_out_in_the_middle() {
        // Single, $216,950 taxable (= threshold + $25k). 50% phased out.
        let r = compute(&QbiInput {
            qbi: d("100000"),
            taxable_income: d("216950"),
            filing_status: FilingStatus::Single,
            is_sstb: true,
            tax_year: 2024,
        });
        // Raw deduction 20% × $100k = $20k. Half phased out → $10k.
        assert_eq!(r.deduction, d("10000.0"));
        assert!((r.phase_out_fraction - 0.5).abs() < 0.01);
    }

    #[test]
    fn married_joint_uses_doubled_threshold() {
        // $300k taxable MFJ — under the $383.9k threshold → full 20%.
        let r = compute(&QbiInput {
            qbi: d("100000"),
            taxable_income: d("300000"),
            filing_status: FilingStatus::MarriedJoint,
            is_sstb: true,
            tax_year: 2024,
        });
        assert_eq!(r.deduction, d("20000.00"));
    }

    #[test]
    fn deduction_capped_at_20_percent_taxable_income() {
        // Tiny taxable income ($10k) but huge QBI claim — the
        // taxable-income cap kicks in.
        let r = compute(&QbiInput {
            qbi: d("100000"),
            taxable_income: d("10000"),
            filing_status: FilingStatus::Single,
            is_sstb: false,
            tax_year: 2024,
        });
        // 20% × $10k = $2k, not 20% × $100k = $20k.
        assert_eq!(r.deduction, d("2000.00"));
    }

    #[test]
    fn negative_qbi_yields_zero_deduction() {
        let r = compute(&QbiInput {
            qbi: d("-5000"),
            taxable_income: d("50000"),
            filing_status: FilingStatus::Single,
            is_sstb: false,
            tax_year: 2024,
        });
        assert_eq!(r.deduction, Decimal::ZERO);
    }

    #[test]
    fn non_sstb_above_threshold_keeps_deduction() {
        // Plumber (non-SSTB) with $300k taxable single — full 20%
        // (W-2/UBIA cap not modeled, but doesn't phase out for non-SSTB).
        let r = compute(&QbiInput {
            qbi: d("250000"),
            taxable_income: d("300000"),
            filing_status: FilingStatus::Single,
            is_sstb: false,
            tax_year: 2024,
        });
        assert!(r.deduction > Decimal::ZERO);
        assert_eq!(r.phase_out_fraction, 0.0);
    }

    #[test]
    fn future_year_threshold_extrapolated_no_panic() {
        let r = compute(&QbiInput {
            qbi: d("100000"),
            taxable_income: d("50000"),
            filing_status: FilingStatus::Single,
            is_sstb: true,
            tax_year: 2099,
        });
        assert_eq!(r.deduction, d("10000.00"));
    }

    #[test]
    fn boundary_at_threshold_exactly_grants_full() {
        let r = compute(&QbiInput {
            qbi: d("100000"),
            taxable_income: d("191950"),      // exactly at 2024 single
            filing_status: FilingStatus::Single,
            is_sstb: true,
            tax_year: 2024,
        });
        // Inclusive — "<= threshold" path.
        assert_eq!(r.deduction, d("20000.00"));
    }
}
