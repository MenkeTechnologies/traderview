//! Orchestrator: `TaxReturn` → `TaxResult`.
//!
//! Walks the 1040 sections in order:
//!   1. Total income (wages + interest + dividends + SE net + rental net + cap gains)
//!   2. Adjustments → AGI (half SE tax, HSA, IRA, student loan interest)
//!   3. Deduction (std vs itemized, picks larger)
//!   4. QBI § 199A deduction
//!   5. Taxable income → bracket tax
//!   6. Other taxes (SE tax)
//!   7. Credits (nonrefundable → refundable)
//!   8. Refund/owed (vs payments + withholding)

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{brackets, credits, qbi, se_tax};

/// 1040 filing status checkboxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    #[default]
    Single,
    /// Married filing jointly (and qualifying surviving spouse).
    Mfj,
    /// Married filing separately.
    Mfs,
    /// Head of household.
    Hoh,
}

/// Single-W-2 entry. Maps 1:1 to W-2 Form boxes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct W2 {
    pub employer_name: String,
    pub box_1_wages: Decimal,
    pub box_2_federal_income_tax_withheld: Decimal,
    pub box_3_ss_wages: Decimal,
    pub box_4_ss_tax_withheld: Decimal,
    pub box_5_medicare_wages: Decimal,
    pub box_6_medicare_tax_withheld: Decimal,
    pub box_17_state_income_tax: Decimal,
}

/// Schedule C summary — auto-populated from receipts where the user
/// has tagged items with `tax_bucket='business'`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduleC {
    pub gross_receipts: Decimal,
    pub total_expenses: Decimal,
    pub net_profit: Decimal,
}

/// Schedule E — rental real estate per property.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduleE {
    pub gross_rents: Decimal,
    pub total_expenses: Decimal,
    pub net_income: Decimal,
}

/// Itemized deductions — collected on Schedule A. When zero, the
/// engine picks the standard deduction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Itemized {
    pub medical_over_7_5_pct_agi: Decimal,
    pub state_and_local_taxes_capped_at_10k: Decimal,
    pub mortgage_interest: Decimal,
    pub charitable_gifts: Decimal,
    pub casualty_losses: Decimal,
}

impl Itemized {
    pub fn total(&self) -> Decimal {
        self.medical_over_7_5_pct_agi
            + self
                .state_and_local_taxes_capped_at_10k
                .min(Decimal::from(10_000))
            + self.mortgage_interest
            + self.charitable_gifts
            + self.casualty_losses
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaxReturn {
    pub tax_year: i32,
    pub status: FilingStatus,

    // Income.
    pub w2s: Vec<W2>,
    pub interest_income: Decimal,
    pub ordinary_dividends: Decimal,
    pub qualified_dividends: Decimal,
    pub net_long_term_capital_gain: Decimal,
    pub schedule_c: ScheduleC,
    pub schedule_e: ScheduleE,
    pub other_income: Decimal,

    // Adjustments above the line.
    pub hsa_deduction: Decimal,
    pub ira_deduction: Decimal,
    pub student_loan_interest: Decimal,
    pub other_adjustments: Decimal,

    // Below-the-line deductions.
    pub itemized: Itemized,
    pub force_standard_deduction: bool,

    // § 199A inputs.
    pub qbi_is_sstb: bool,

    // Credits.
    pub qualifying_children_under_17: u32,
    pub other_dependents: u32,

    // Payments + withholding.
    pub estimated_tax_payments: Decimal,
    pub eitc_claim: Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaxResult {
    pub total_income: Decimal,
    pub adjustments_total: Decimal,
    pub agi: Decimal,
    pub deduction_used: Decimal,
    pub deduction_label: String, // "standard" | "itemized"
    pub qbi_deduction: Decimal,
    pub qbi_needs_manual_review: bool,
    pub taxable_income: Decimal,
    pub ordinary_tax: Decimal,
    /// QDCGTW breakdown when preferential income is present; `None`
    /// when LTCG and qualified dividends are both zero.
    pub capital_gains: Option<crate::capital_gains::QdcgtwResult>,
    pub se_tax: se_tax::SeResult,
    pub additional_medicare: Decimal,
    pub ctc: credits::CtcResult,
    pub tax_after_credits: Decimal,
    /// Net investment income tax (IRC § 1411, 3.8%).
    pub niit: crate::niit::NiitResult,
    pub total_payments: Decimal,
    pub refund_due: Decimal,
    pub tax_owed: Decimal,
}

pub fn compute(r: &TaxReturn) -> TaxResult {
    let mut result = TaxResult::default();

    // ── 1) Total income ────────────────────────────────────────────────
    let w2_wages: Decimal = r.w2s.iter().map(|w| w.box_1_wages).sum();
    let w2_ss_wages: Decimal = r.w2s.iter().map(|w| w.box_3_ss_wages).sum();
    let w2_medicare_wages: Decimal = r.w2s.iter().map(|w| w.box_5_medicare_wages).sum();
    let w2_fed_withheld: Decimal = r
        .w2s
        .iter()
        .map(|w| w.box_2_federal_income_tax_withheld)
        .sum();

    result.total_income = w2_wages
        + r.interest_income
        + r.ordinary_dividends
        + r.net_long_term_capital_gain
        + r.schedule_c.net_profit
        + r.schedule_e.net_income
        + r.other_income;

    // ── 2) Adjustments → AGI ───────────────────────────────────────────
    let se = se_tax::compute(
        r.schedule_c.net_profit,
        w2_ss_wages,
        w2_medicare_wages,
        r.status,
    );
    result.se_tax = se;

    result.adjustments_total = se.above_line_deduction
        + r.hsa_deduction
        + r.ira_deduction
        + r.student_loan_interest
        + r.other_adjustments;
    result.agi = (result.total_income - result.adjustments_total).max(Decimal::ZERO);

    // ── 3) Deduction (std vs itemized, picks larger unless forced) ────
    let std_ded = brackets::standard_deduction(r.status);
    let itemized_total = r.itemized.total();
    if r.force_standard_deduction || itemized_total <= std_ded {
        result.deduction_used = std_ded;
        result.deduction_label = "standard".into();
    } else {
        result.deduction_used = itemized_total;
        result.deduction_label = "itemized".into();
    }

    // ── 4) QBI § 199A ──────────────────────────────────────────────────
    // QBI = positive net SE income (Schedule C profit). Negative or
    // zero net SE → no QBI.
    let qbi_amount = r.schedule_c.net_profit.max(Decimal::ZERO);
    let ti_before_qbi = (result.agi - result.deduction_used).max(Decimal::ZERO);
    let net_cap_gain = r.net_long_term_capital_gain + r.qualified_dividends;
    let qbi_result = qbi::compute(qbi::QbiInput {
        qualified_business_income: qbi_amount,
        taxable_income_before_qbi: ti_before_qbi,
        net_capital_gain: net_cap_gain,
        is_sstb: r.qbi_is_sstb,
        status: r.status,
    });
    result.qbi_deduction = qbi_result.deduction;
    result.qbi_needs_manual_review = qbi_result.needs_manual_review;

    // ── 5) Taxable income → bracket tax ────────────────────────────────
    // Routes through the Qualified Dividends and Capital Gain Tax
    // Worksheet (IRC § 1(h) — preferential 0/15/20% rates) when the
    // taxpayer has LTCG or qualified dividends; otherwise straight
    // bracket tax. `ordinary_tax` always reflects the *total* income
    // tax bill (ordinary + preferential), matching Form 1040 line 16.
    result.taxable_income = (ti_before_qbi - result.qbi_deduction).max(Decimal::ZERO);
    if crate::capital_gains::has_preferential_income(
        r.qualified_dividends,
        r.net_long_term_capital_gain,
    ) {
        let cg = crate::capital_gains::compute(crate::capital_gains::QdcgtwInput {
            taxable_income: result.taxable_income,
            net_long_term_capital_gain: r.net_long_term_capital_gain,
            qualified_dividends: r.qualified_dividends,
            status: r.status,
        });
        result.ordinary_tax = cg.total_tax;
        result.capital_gains = Some(cg);
    } else {
        result.ordinary_tax =
            brackets::ordinary_income_tax(result.taxable_income, r.status).round_dp(2);
        result.capital_gains = None;
    }

    // ── 6) Credits ─────────────────────────────────────────────────────
    let ctc = credits::child_tax_credit(credits::CtcInput {
        qualifying_children_under_17: r.qualifying_children_under_17,
        other_dependents: r.other_dependents,
        agi: result.agi,
        status: r.status,
    });
    result.ctc = ctc;
    // CTC + ODC offset ordinary tax (non-refundable). The refundable
    // portion of CTC (Additional Child Tax Credit) goes into payments.
    let nonref_credit = (ctc.total - ctc.refundable_portion).min(result.ordinary_tax);

    // ── 6.5) NIIT (IRC § 1411) — 3.8% surtax on the lesser of NII or
    // (MAGI - statutory threshold). Self-employment income is NOT NII
    // (it's subject to SE tax instead). Rental on Schedule E counts as
    // NII when passive (the default for most users; material-participation
    // is rare and not modeled in v1).
    let nii = (r.interest_income
        + r.ordinary_dividends
        + r.net_long_term_capital_gain
        + r.schedule_e.net_income)
        .max(Decimal::ZERO);
    let niit = crate::niit::compute(crate::niit::NiitInput {
        net_investment_income: nii,
        magi: result.agi, // MAGI ≈ AGI absent foreign-earned-income exclusion
        status: r.status,
    });
    result.niit = niit;

    result.tax_after_credits =
        (result.ordinary_tax - nonref_credit + se.total + niit.tax).max(Decimal::ZERO);
    result.additional_medicare = se.additional_medicare_tax;

    // ── 7) Payments + withholding ─────────────────────────────────────
    result.total_payments =
        w2_fed_withheld + r.estimated_tax_payments + ctc.refundable_portion + r.eitc_claim;

    // ── 8) Refund vs owed ──────────────────────────────────────────────
    if result.total_payments >= result.tax_after_credits {
        result.refund_due = (result.total_payments - result.tax_after_credits).round_dp(2);
        result.tax_owed = Decimal::ZERO;
    } else {
        result.tax_owed = (result.tax_after_credits - result.total_payments).round_dp(2);
        result.refund_due = Decimal::ZERO;
    }

    result
}

#[cfg(test)]
mod itemized_tests {
    use super::*;

    /// SALT cap exactly at $10k — the cap permits the full amount.
    #[test]
    fn salt_at_cap_boundary_uses_full_amount() {
        let i = Itemized {
            medical_over_7_5_pct_agi: Decimal::ZERO,
            state_and_local_taxes_capped_at_10k: Decimal::from(10_000),
            mortgage_interest: Decimal::ZERO,
            charitable_gifts: Decimal::ZERO,
            casualty_losses: Decimal::ZERO,
        };
        assert_eq!(i.total(), Decimal::from(10_000));
    }

    /// $9,999 — under cap, full amount counts.
    #[test]
    fn salt_under_cap_uses_full_amount() {
        let i = Itemized {
            state_and_local_taxes_capped_at_10k: Decimal::from(9_999),
            ..Itemized::default()
        };
        assert_eq!(i.total(), Decimal::from(9_999));
    }

    /// $10,001 — cap binds, drops to $10k.
    #[test]
    fn salt_over_cap_clamps_to_10k() {
        let i = Itemized {
            state_and_local_taxes_capped_at_10k: Decimal::from(10_001),
            ..Itemized::default()
        };
        assert_eq!(
            i.total(),
            Decimal::from(10_000),
            "$10,001 SALT must cap at $10,000 (TCJA § 164(b)(6))"
        );
    }

    /// Big SALT entry — cap still binds, doesn't pass through.
    #[test]
    fn salt_big_value_still_capped() {
        let i = Itemized {
            state_and_local_taxes_capped_at_10k: Decimal::from(50_000),
            ..Itemized::default()
        };
        assert_eq!(i.total(), Decimal::from(10_000));
    }

    /// Other lines pass through uncapped — only SALT has the $10k cap
    /// in this code path. Mortgage interest can be tens of thousands.
    #[test]
    fn non_salt_lines_pass_through_uncapped() {
        let i = Itemized {
            medical_over_7_5_pct_agi: Decimal::from(8_000),
            state_and_local_taxes_capped_at_10k: Decimal::ZERO,
            mortgage_interest: Decimal::from(35_000),
            charitable_gifts: Decimal::from(12_000),
            casualty_losses: Decimal::from(5_000),
        };
        // 8k + 0 + 35k + 12k + 5k = 60k.
        assert_eq!(i.total(), Decimal::from(60_000));
    }

    /// All-zero default. Itemized::default() must produce total = 0.
    #[test]
    fn default_itemized_totals_zero() {
        assert_eq!(Itemized::default().total(), Decimal::ZERO);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty(status: FilingStatus) -> TaxReturn {
        TaxReturn {
            tax_year: 2025,
            status,
            ..Default::default()
        }
    }

    #[test]
    fn empty_return_no_income_no_tax_no_refund() {
        let r = empty(FilingStatus::Single);
        let res = compute(&r);
        assert_eq!(res.total_income, Decimal::ZERO);
        assert_eq!(res.agi, Decimal::ZERO);
        assert_eq!(res.tax_owed, Decimal::ZERO);
        assert_eq!(res.refund_due, Decimal::ZERO);
        // Standard deduction kicks in even with $0 income.
        assert_eq!(res.deduction_used, Decimal::from(15_000));
        assert_eq!(res.deduction_label, "standard");
    }

    #[test]
    fn single_w2_only_modest_refund() {
        let mut r = empty(FilingStatus::Single);
        r.w2s.push(W2 {
            employer_name: "ACME".into(),
            box_1_wages: Decimal::from(50_000),
            box_2_federal_income_tax_withheld: Decimal::from(6_000),
            box_3_ss_wages: Decimal::from(50_000),
            box_4_ss_tax_withheld: Decimal::from(3_100),
            box_5_medicare_wages: Decimal::from(50_000),
            box_6_medicare_tax_withheld: Decimal::from(725),
            box_17_state_income_tax: Decimal::ZERO,
        });
        let res = compute(&r);
        assert_eq!(res.total_income, Decimal::from(50_000));
        // No SE tax → no above-line deduction → AGI = $50,000.
        assert_eq!(res.agi, Decimal::from(50_000));
        // After $15k std deduction → TI = $35,000 (no QBI).
        assert_eq!(res.taxable_income, Decimal::from(35_000));
        // Brackets: 11,925 @ 10% + 23,075 @ 12% = 1192.5 + 2769 = 3961.5
        assert_eq!(res.ordinary_tax, "3961.5".parse::<Decimal>().unwrap());
        // Withheld $6,000 > $3,961.50 owed → refund $2,038.50.
        assert_eq!(res.refund_due, "2038.5".parse::<Decimal>().unwrap());
        assert_eq!(res.tax_owed, Decimal::ZERO);
    }

    #[test]
    fn self_employed_only_owes_se_tax_plus_income_tax() {
        let mut r = empty(FilingStatus::Single);
        r.schedule_c = ScheduleC {
            gross_receipts: Decimal::from(100_000),
            total_expenses: Decimal::from(20_000),
            net_profit: Decimal::from(80_000),
        };
        let res = compute(&r);
        // SE tax computed: $80k * 0.9235 = 73,880 base; SS 12.4% = 9,161.12;
        // Medicare 2.9% = 2,142.52; total = 11,303.64; half = 5,651.82.
        assert!(res.se_tax.total > Decimal::ZERO);
        assert!(res.adjustments_total > Decimal::ZERO);
        // QBI deduction reduces taxable income.
        assert!(res.qbi_deduction > Decimal::ZERO);
        // Owes — no withholding.
        assert!(res.tax_owed > Decimal::ZERO);
        assert_eq!(res.refund_due, Decimal::ZERO);
    }

    #[test]
    fn itemized_beats_standard_when_larger() {
        let mut r = empty(FilingStatus::Single);
        r.w2s.push(W2 {
            box_1_wages: Decimal::from(80_000),
            box_2_federal_income_tax_withheld: Decimal::from(8_000),
            ..Default::default()
        });
        r.itemized = Itemized {
            medical_over_7_5_pct_agi: Decimal::from(5_000),
            state_and_local_taxes_capped_at_10k: Decimal::from(10_000),
            mortgage_interest: Decimal::from(8_000),
            charitable_gifts: Decimal::from(2_000),
            casualty_losses: Decimal::ZERO,
        };
        // Itemized total = 25,000 > std $15,000.
        let res = compute(&r);
        assert_eq!(res.deduction_label, "itemized");
        assert_eq!(res.deduction_used, Decimal::from(25_000));
    }

    #[test]
    fn force_standard_overrides_itemized() {
        let mut r = empty(FilingStatus::Single);
        r.itemized.mortgage_interest = Decimal::from(50_000);
        r.force_standard_deduction = true;
        let res = compute(&r);
        assert_eq!(res.deduction_label, "standard");
    }

    #[test]
    fn ctc_two_kids_offsets_tax() {
        let mut r = empty(FilingStatus::Mfj);
        r.w2s.push(W2 {
            box_1_wages: Decimal::from(80_000),
            box_2_federal_income_tax_withheld: Decimal::from(5_000),
            ..Default::default()
        });
        r.qualifying_children_under_17 = 2;
        let res = compute(&r);
        // CTC = $4,000 ($3,400 refundable). Brings tax_after_credits down.
        assert_eq!(res.ctc.total, Decimal::from(4_000));
    }
}
