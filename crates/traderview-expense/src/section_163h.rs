//! IRC §163(h) — Qualified residence interest deduction for home
//! mortgage interest.
//!
//! Universal to any home-owning taxpayer. The TCJA (P.L. 115-97)
//! capped acquisition indebtedness at **$750k** for mortgages
//! originated after **2017-12-15**, killed the home equity
//! indebtedness deduction 2018-2025, and was scheduled to sunset at
//! end of 2025. The **One Big Beautiful Bill Act of 2025** (§ 70108
//! of OBBBA) **permanently extended** both: the $750k cap and the
//! home equity indebtedness disallowance no longer sunset. OBBBA also
//! **permanently reinstated** mortgage insurance premiums (PMI) as
//! qualified mortgage interest for tax years beginning after 2025
//! (i.e., 2026+).
//!
//! **Grandfathered acquisition indebtedness** (§ 163(h)(3)(F)(i)):
//! mortgages originated **before 2017-12-16** retain the **$1M cap**
//! ($500k MFS). Subject to a binding-contract carve-out — a mortgage
//! originated under a binding contract entered before 2017-12-15 and
//! closed before **2018-04-01** is also grandfathered.
//!
//! **Caps by filing status:**
//!
//! | Origination date              | Single/MFJ/HOH | MFS    |
//! |-------------------------------|----------------|--------|
//! | Pre-2017-12-16 (grandfathered) | $1,000,000    | $500,000 |
//! | Post-2017-12-15 (TCJA + OBBBA permanent) | $750,000 | $375,000 |
//!
//! **Refinance** (§ 163(h)(3)(F)(iii)): a refinance of a grandfathered
//! mortgage retains the grandfathered $1M cap to the extent of the
//! pre-refinance balance. Any cash-out / new money on top falls under
//! the new $750k cap. Cap applied PROPORTIONALLY by this module.
//!
//! **Home equity indebtedness** (§ 163(h)(3)(C), as amended by TCJA +
//! OBBBA): interest on home equity debt NOT used to buy, build, or
//! substantially improve the home that secures the loan is NEVER
//! DEDUCTIBLE. If the home equity loan IS used for acquisition /
//! substantial improvement, it counts toward the acquisition
//! indebtedness cap (not as a separate $100k bucket — the pre-TCJA
//! separate $100k bucket is permanently gone under OBBBA).
//!
//! **PMI / mortgage insurance premiums** (§ 163(h)(4)(E) reinstated by
//! OBBBA): for tax years beginning after 2025, PMI on acquisition debt
//! is qualified mortgage interest. Subject to phase-out for taxpayers
//! with AGI > $100k ($50k MFS) at 10% per $1k of AGI above the
//! threshold under pre-TCJA rules — the module surfaces the raw
//! premium and lets the caller apply the phase-out.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    SingleOrMfjOrHoh,
    MarriedFilingSeparately,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section163hInput {
    pub tax_year: i32,
    pub filing_status: FilingStatus,
    /// True if the underlying mortgage was originated before
    /// 2017-12-16 (or under the binding-contract carve-out — contract
    /// pre-2017-12-15 AND closing pre-2018-04-01).
    pub mortgage_grandfathered: bool,
    /// Acquisition indebtedness balance (used to buy / build /
    /// substantially improve the home).
    pub acquisition_indebtedness_balance: Decimal,
    /// Home equity indebtedness balance NOT used for acquisition /
    /// substantial improvement (TCJA + OBBBA: never deductible).
    pub non_acquisition_home_equity_balance: Decimal,
    pub interest_paid_acquisition: Decimal,
    pub interest_paid_non_acquisition_home_equity: Decimal,
    /// PMI premiums paid in the tax year. Deductible 2026+; ignored
    /// for tax years ≤ 2025.
    pub mortgage_insurance_premiums_paid: Decimal,
    /// Refinance proportional-allocation hook: pre-refinance balance
    /// that retains grandfathered treatment under § 163(h)(3)(F)(iii).
    /// If `None` or 0, no refinance allocation applied.
    pub grandfathered_refinance_portion: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section163hResult {
    pub effective_acquisition_debt_cap: Decimal,
    pub deductible_acquisition_interest: Decimal,
    pub disallowed_acquisition_interest_over_cap: Decimal,
    /// Always zero post-TCJA + OBBBA — never deductible.
    pub deductible_home_equity_interest: Decimal,
    pub disallowed_home_equity_interest: Decimal,
    pub deductible_pmi_premiums: Decimal,
    pub total_qualified_residence_interest: Decimal,
    pub note: String,
}

const TCJA_CAP_SINGLE: i64 = 750_000;
const TCJA_CAP_MFS: i64 = 375_000;
const GRANDFATHERED_CAP_SINGLE: i64 = 1_000_000;
const GRANDFATHERED_CAP_MFS: i64 = 500_000;

pub fn compute(input: &Section163hInput) -> Section163hResult {
    let mfs = input.filing_status == FilingStatus::MarriedFilingSeparately;
    let grandfathered_cap = Decimal::from(if mfs {
        GRANDFATHERED_CAP_MFS
    } else {
        GRANDFATHERED_CAP_SINGLE
    });
    let tcja_cap = Decimal::from(if mfs { TCJA_CAP_MFS } else { TCJA_CAP_SINGLE });

    // Refinance: pro-rate the cap by the grandfathered portion of the
    // current balance vs the new-money portion.
    let effective_cap = match input.grandfathered_refinance_portion {
        Some(pre) if pre > Decimal::ZERO => {
            // Grandfathered portion at $1M cap, remainder at $750k cap.
            // Effective cap = pre (capped at grandfathered) + (current
            // balance − pre) (capped at TCJA cap).
            let grand_part = pre.min(grandfathered_cap);
            let new_money = (input.acquisition_indebtedness_balance - pre).max(Decimal::ZERO);
            grand_part + new_money.min(tcja_cap)
        }
        _ => {
            if input.mortgage_grandfathered {
                grandfathered_cap
            } else {
                tcja_cap
            }
        }
    };

    // Acquisition interest deductibility: pro-rate by cap / balance.
    let bal = input.acquisition_indebtedness_balance;
    let deductible_acq = if bal <= Decimal::ZERO {
        Decimal::ZERO
    } else if bal <= effective_cap {
        input.interest_paid_acquisition
    } else {
        input.interest_paid_acquisition * effective_cap / bal
    };
    let disallowed_acq = (input.interest_paid_acquisition - deductible_acq).max(Decimal::ZERO);

    // Home equity (non-acquisition) interest: TCJA + OBBBA never
    // deductible.
    let disallowed_he = input
        .interest_paid_non_acquisition_home_equity
        .max(Decimal::ZERO);

    // PMI: only deductible for tax years ≥ 2026 per OBBBA reinstatement.
    let deductible_pmi = if input.tax_year >= 2026 {
        input.mortgage_insurance_premiums_paid.max(Decimal::ZERO)
    } else {
        Decimal::ZERO
    };

    let total = deductible_acq + deductible_pmi;

    let cap_basis = if input.grandfathered_refinance_portion.unwrap_or_default() > Decimal::ZERO {
        format!(
            "blended refinance cap: ${} grandfathered + remaining at ${} TCJA",
            input.grandfathered_refinance_portion.unwrap_or_default(),
            tcja_cap,
        )
    } else if input.mortgage_grandfathered {
        format!("${} grandfathered pre-2017-12-16", grandfathered_cap)
    } else {
        format!(
            "${} TCJA cap (made permanent by OBBBA 2025 § 70108)",
            tcja_cap
        )
    };
    let note = format!(
        "§163(h) qualified residence interest. Cap basis: {}. Effective cap ${}. Acquisition interest: ${} deductible / ${} disallowed. Home equity interest ${} permanently disallowed (TCJA + OBBBA). PMI ${} deductible{}.",
        cap_basis,
        effective_cap.round_dp(2),
        deductible_acq.round_dp(2),
        disallowed_acq.round_dp(2),
        disallowed_he.round_dp(2),
        deductible_pmi.round_dp(2),
        if input.tax_year < 2026 {
            " (PMI only deductible 2026+ per OBBBA § 70108 reinstatement)"
        } else {
            ""
        },
    );

    Section163hResult {
        effective_acquisition_debt_cap: effective_cap,
        deductible_acquisition_interest: deductible_acq,
        disallowed_acquisition_interest_over_cap: disallowed_acq,
        deductible_home_equity_interest: Decimal::ZERO,
        disallowed_home_equity_interest: disallowed_he,
        deductible_pmi_premiums: deductible_pmi,
        total_qualified_residence_interest: total,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section163hInput {
        Section163hInput {
            tax_year: 2026,
            filing_status: FilingStatus::SingleOrMfjOrHoh,
            mortgage_grandfathered: false,
            acquisition_indebtedness_balance: dec!(500_000),
            non_acquisition_home_equity_balance: Decimal::ZERO,
            interest_paid_acquisition: dec!(25_000),
            interest_paid_non_acquisition_home_equity: Decimal::ZERO,
            mortgage_insurance_premiums_paid: Decimal::ZERO,
            grandfathered_refinance_portion: None,
        }
    }

    #[test]
    fn standard_post_tcja_under_cap_full_deduction() {
        // $500k balance < $750k cap → full $25k interest deductible.
        let r = compute(&base());
        assert_eq!(r.effective_acquisition_debt_cap, dec!(750_000));
        assert_eq!(r.deductible_acquisition_interest, dec!(25_000));
        assert_eq!(r.disallowed_acquisition_interest_over_cap, Decimal::ZERO);
    }

    #[test]
    fn over_750k_cap_pro_rata_deduction() {
        // $1.5M balance × $750k cap / $1.5M balance = 50% deductible.
        let mut i = base();
        i.acquisition_indebtedness_balance = dec!(1_500_000);
        i.interest_paid_acquisition = dec!(60_000);
        let r = compute(&i);
        assert_eq!(r.deductible_acquisition_interest, dec!(30_000));
        assert_eq!(r.disallowed_acquisition_interest_over_cap, dec!(30_000));
    }

    #[test]
    fn grandfathered_pre_2017_full_1m_cap() {
        // $900k grandfathered mortgage → full deduction (under $1M cap).
        let mut i = base();
        i.mortgage_grandfathered = true;
        i.acquisition_indebtedness_balance = dec!(900_000);
        i.interest_paid_acquisition = dec!(40_000);
        let r = compute(&i);
        assert_eq!(r.effective_acquisition_debt_cap, dec!(1_000_000));
        assert_eq!(r.deductible_acquisition_interest, dec!(40_000));
    }

    #[test]
    fn grandfathered_over_1m_cap_partial() {
        let mut i = base();
        i.mortgage_grandfathered = true;
        i.acquisition_indebtedness_balance = dec!(2_000_000);
        i.interest_paid_acquisition = dec!(80_000);
        let r = compute(&i);
        // $80k × $1M / $2M = $40k deductible.
        assert_eq!(r.deductible_acquisition_interest, dec!(40_000));
    }

    #[test]
    fn mfs_filing_status_half_cap_post_tcja() {
        let mut i = base();
        i.filing_status = FilingStatus::MarriedFilingSeparately;
        let r = compute(&i);
        assert_eq!(r.effective_acquisition_debt_cap, dec!(375_000));
    }

    #[test]
    fn mfs_grandfathered_half_cap() {
        let mut i = base();
        i.filing_status = FilingStatus::MarriedFilingSeparately;
        i.mortgage_grandfathered = true;
        let r = compute(&i);
        assert_eq!(r.effective_acquisition_debt_cap, dec!(500_000));
    }

    #[test]
    fn home_equity_interest_never_deductible_post_tcja() {
        // $10k HE interest → all disallowed under TCJA + OBBBA.
        let mut i = base();
        i.non_acquisition_home_equity_balance = dec!(100_000);
        i.interest_paid_non_acquisition_home_equity = dec!(10_000);
        let r = compute(&i);
        assert_eq!(r.deductible_home_equity_interest, Decimal::ZERO);
        assert_eq!(r.disallowed_home_equity_interest, dec!(10_000));
    }

    #[test]
    fn pmi_deductible_starting_2026_only() {
        let mut i = base();
        i.tax_year = 2025;
        i.mortgage_insurance_premiums_paid = dec!(2_000);
        let r = compute(&i);
        assert_eq!(r.deductible_pmi_premiums, Decimal::ZERO);

        let mut i2 = base();
        i2.tax_year = 2026;
        i2.mortgage_insurance_premiums_paid = dec!(2_000);
        let r2 = compute(&i2);
        assert_eq!(r2.deductible_pmi_premiums, dec!(2_000));
    }

    #[test]
    fn pmi_2026_added_to_total() {
        let mut i = base();
        i.mortgage_insurance_premiums_paid = dec!(2_000);
        let r = compute(&i);
        // $25k interest + $2k PMI = $27k total.
        assert_eq!(r.total_qualified_residence_interest, dec!(27_000));
    }

    #[test]
    fn refinance_pro_rata_grandfathered_plus_new_money() {
        // Pre-refi $900k grandfathered, post-refi $1.5M → $600k new money.
        // Effective cap = $900k grandfathered + min($600k, $750k) = $1.5M.
        let mut i = base();
        i.mortgage_grandfathered = false; // The refinance itself isn't pre-12/15/17 — only the original.
        i.acquisition_indebtedness_balance = dec!(1_500_000);
        i.grandfathered_refinance_portion = Some(dec!(900_000));
        i.interest_paid_acquisition = dec!(60_000);
        let r = compute(&i);
        assert_eq!(r.effective_acquisition_debt_cap, dec!(1_500_000));
        assert_eq!(r.deductible_acquisition_interest, dec!(60_000));
    }

    #[test]
    fn refinance_new_money_caps_at_tcja_limit() {
        // Pre-refi $500k grand, refi to $2M → $1.5M new money.
        // Effective cap = $500k + min($1.5M, $750k) = $1.25M.
        let mut i = base();
        i.mortgage_grandfathered = false;
        i.acquisition_indebtedness_balance = dec!(2_000_000);
        i.grandfathered_refinance_portion = Some(dec!(500_000));
        i.interest_paid_acquisition = dec!(80_000);
        let r = compute(&i);
        assert_eq!(r.effective_acquisition_debt_cap, dec!(1_250_000));
        // $80k × $1.25M / $2M = $50k.
        assert_eq!(r.deductible_acquisition_interest, dec!(50_000));
    }

    #[test]
    fn zero_acquisition_balance_no_deduction() {
        let mut i = base();
        i.acquisition_indebtedness_balance = Decimal::ZERO;
        i.interest_paid_acquisition = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.deductible_acquisition_interest, Decimal::ZERO);
    }

    #[test]
    fn at_exact_cap_full_deduction() {
        // Balance exactly at $750k cap → full deduction.
        let mut i = base();
        i.acquisition_indebtedness_balance = dec!(750_000);
        i.interest_paid_acquisition = dec!(30_000);
        let r = compute(&i);
        assert_eq!(r.deductible_acquisition_interest, dec!(30_000));
        assert_eq!(r.disallowed_acquisition_interest_over_cap, Decimal::ZERO);
    }

    #[test]
    fn one_dollar_over_cap_proportional_haircut() {
        let mut i = base();
        i.acquisition_indebtedness_balance = dec!(750_001);
        i.interest_paid_acquisition = dec!(30_000);
        let r = compute(&i);
        // $30k × $750k / $750_001 ≈ $29999.96.
        assert!(r.deductible_acquisition_interest < dec!(30_000));
        assert!(r.deductible_acquisition_interest > dec!(29_999));
    }

    #[test]
    fn combined_acquisition_plus_home_equity_calculation() {
        // $500k acquisition + $50k home equity. Acquisition full deductible.
        // Home equity entirely disallowed under TCJA + OBBBA.
        let mut i = base();
        i.non_acquisition_home_equity_balance = dec!(50_000);
        i.interest_paid_non_acquisition_home_equity = dec!(3_000);
        let r = compute(&i);
        assert_eq!(r.deductible_acquisition_interest, dec!(25_000));
        assert_eq!(r.deductible_home_equity_interest, Decimal::ZERO);
        assert_eq!(r.disallowed_home_equity_interest, dec!(3_000));
    }

    #[test]
    fn note_describes_obbba_permanence() {
        let r = compute(&base());
        assert!(r.note.contains("OBBBA"));
        assert!(r.note.contains("permanent"));
    }

    #[test]
    fn note_2025_calls_out_pmi_only_2026_plus() {
        let mut i = base();
        i.tax_year = 2025;
        i.mortgage_insurance_premiums_paid = dec!(1_000);
        let r = compute(&i);
        assert!(r.note.contains("PMI only deductible 2026+"));
    }

    #[test]
    fn note_grandfathered_path_described() {
        let mut i = base();
        i.mortgage_grandfathered = true;
        let r = compute(&i);
        assert!(r.note.contains("grandfathered"));
    }

    #[test]
    fn note_refinance_blended_path_described() {
        let mut i = base();
        i.grandfathered_refinance_portion = Some(dec!(500_000));
        i.acquisition_indebtedness_balance = dec!(1_000_000);
        let r = compute(&i);
        assert!(r.note.contains("blended refinance"));
    }

    #[test]
    fn large_balance_precision_path() {
        // $5M balance / $750k cap → 15% deductible.
        let mut i = base();
        i.acquisition_indebtedness_balance = dec!(5_000_000);
        i.interest_paid_acquisition = dec!(200_000);
        let r = compute(&i);
        // $200k × $750k / $5M = $30k.
        assert_eq!(r.deductible_acquisition_interest, dec!(30_000));
    }

    #[test]
    fn mfs_over_cap_pro_rata() {
        let mut i = base();
        i.filing_status = FilingStatus::MarriedFilingSeparately;
        i.acquisition_indebtedness_balance = dec!(750_000);
        i.interest_paid_acquisition = dec!(30_000);
        let r = compute(&i);
        // $30k × $375k / $750k = $15k.
        assert_eq!(r.deductible_acquisition_interest, dec!(15_000));
    }
}
