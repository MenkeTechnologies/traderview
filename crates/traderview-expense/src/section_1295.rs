//! IRC §1295 — Qualified Electing Fund (QEF) election for PFIC stock.
//!
//! Companion to iter 22's `section_1296` PFIC mark-to-market election.
//! When a U.S. shareholder holds Passive Foreign Investment Company
//! stock, three regimes are available:
//!
//!   1. **§1291 default** — punitive "excess distribution" rules:
//!      distributions and gains taxed at highest historical marginal
//!      rate + deferred-interest charge. This is what catches retail
//!      international ETF holders.
//!
//!   2. **§1296 MTM** (iter 22) — annual mark-to-market reported as
//!      ordinary income or loss. Loss limited to "unreversed
//!      inclusions" (cumulative prior gain). Only available for
//!      MARKETABLE PFIC stock.
//!
//!   3. **§1295 QEF** (this module) — shareholder included as
//!      partner-equivalent: each year reports pro-rata share of the
//!      PFIC's ordinary earnings AND net capital gain per §1293(a).
//!      **Character preserved**: capital gain stays LTCG. No
//!      deferred-interest charge. No "loss limitation" — but losses
//!      are also not currently deductible (they reduce basis only
//!      to zero, then go nowhere). QEF requires the PFIC to provide
//!      a **PFIC Annual Information Statement** — many PFICs don't.
//!
//! Mechanics per §1293/§1295/§1297:
//!
//!   * Annual inclusion = pro_rata_ordinary_earnings (ordinary) +
//!     pro_rata_net_capital_gain (LTCG character).
//!   * **Basis increased** by the inclusion per §1293(d)(1). This
//!     prevents double tax when the gain is eventually distributed.
//!   * **Distributions from previously taxed income (PTI)** are
//!     excluded from gross income per §1293(c). The shareholder
//!     tracks PTI via the basis-adjustment mechanism.
//!   * Basis decreased by such PTI distributions. New PTI account
//!     ends the year = beginning PTI + current-year inclusion -
//!     PTI distributions.
//!   * Distributions in excess of PTI are taxed as regular dividends
//!     (qualified or not depending on holding period).
//!
//! Pure compute. Caller passes the PFIC's pro-rata report numbers plus holdings plus distributions; we compute the inclusion, the basis evolution, and the taxable-distribution split.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1295Input {
    pub tax_year: i32,
    /// Shareholder's pro-rata share of the PFIC's ordinary earnings
    /// for the year, per the PFIC Annual Information Statement.
    pub pro_rata_ordinary_earnings: Decimal,
    /// Shareholder's pro-rata share of the PFIC's net capital gain
    /// (LTCG-character preserved per §1293(a)(1)(B)).
    pub pro_rata_net_capital_gain: Decimal,
    /// Adjusted basis at the START of the year. Year 1 of holding =
    /// cost basis.
    pub adjusted_basis_year_start: Decimal,
    /// Total distributions received from the PFIC this year.
    pub distributions_received: Decimal,
    /// Cumulative previously-taxed income (PTI) at the START of the
    /// year — sum of prior years' inclusions minus prior PTI
    /// distributions. Year 1 = 0.
    pub pti_account_year_start: Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section1295Result {
    pub tax_year: i32,
    /// Reportable ordinary income (Form 8621 + Schedule 1).
    pub ordinary_income_inclusion: Decimal,
    /// Reportable LTCG (Form 8621 + Schedule D — character preserved).
    pub long_term_capital_gain_inclusion: Decimal,
    /// Distributions from PTI — excluded from gross income.
    pub distribution_from_pti: Decimal,
    /// Distributions in excess of PTI — taxed as dividends this year.
    pub taxable_dividend_distribution: Decimal,
    /// Basis at year end after the §1293(d) adjustments.
    pub adjusted_basis_year_end: Decimal,
    /// PTI account at year end.
    pub pti_account_year_end: Decimal,
    pub note: String,
}

pub fn compute(input: &Section1295Input) -> Section1295Result {
    let mut r = Section1295Result {
        tax_year: input.tax_year,
        ordinary_income_inclusion: input.pro_rata_ordinary_earnings.max(Decimal::ZERO),
        long_term_capital_gain_inclusion: input.pro_rata_net_capital_gain.max(Decimal::ZERO),
        ..Section1295Result::default()
    };

    let total_inclusion = r.ordinary_income_inclusion + r.long_term_capital_gain_inclusion;

    // PTI account before distributions = prior PTI + current inclusion.
    let pti_pool_before_distributions =
        (input.pti_account_year_start + total_inclusion).max(Decimal::ZERO);

    // Distributions consume PTI first (excluded from gross income),
    // then become taxable dividends.
    let distributions_for_year = input.distributions_received.max(Decimal::ZERO);
    r.distribution_from_pti = distributions_for_year.min(pti_pool_before_distributions);
    r.taxable_dividend_distribution =
        (distributions_for_year - r.distribution_from_pti).max(Decimal::ZERO);

    r.pti_account_year_end = pti_pool_before_distributions - r.distribution_from_pti;

    // Basis evolves: +inclusion, -PTI distributions per §1293(d).
    r.adjusted_basis_year_end = (input.adjusted_basis_year_start + total_inclusion
        - r.distribution_from_pti)
        .max(Decimal::ZERO);

    r.note = if total_inclusion == Decimal::ZERO && distributions_for_year == Decimal::ZERO {
        "no QEF inclusion or distribution this year".into()
    } else if r.taxable_dividend_distribution > Decimal::ZERO {
        format!(
            "QEF: ${} ordinary + ${} LTCG inclusion; ${} distributions covered by PTI, ${} taxable as dividend; basis ${} → ${}; PTI ${} → ${}",
            r.ordinary_income_inclusion,
            r.long_term_capital_gain_inclusion,
            r.distribution_from_pti,
            r.taxable_dividend_distribution,
            input.adjusted_basis_year_start,
            r.adjusted_basis_year_end,
            input.pti_account_year_start,
            r.pti_account_year_end,
        )
    } else {
        format!(
            "QEF: ${} ordinary + ${} LTCG inclusion (character preserved); basis ${} → ${}; PTI ${} → ${}",
            r.ordinary_income_inclusion,
            r.long_term_capital_gain_inclusion,
            input.adjusted_basis_year_start,
            r.adjusted_basis_year_end,
            input.pti_account_year_start,
            r.pti_account_year_end,
        )
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section1295Input {
        Section1295Input {
            tax_year: 2024,
            pro_rata_ordinary_earnings: dec!(3000),
            pro_rata_net_capital_gain: dec!(2000),
            adjusted_basis_year_start: dec!(50000),
            distributions_received: Decimal::ZERO,
            pti_account_year_start: Decimal::ZERO,
        }
    }

    #[test]
    fn year_1_inclusion_preserves_character() {
        let r = compute(&base());
        assert_eq!(r.ordinary_income_inclusion, dec!(3000));
        assert_eq!(r.long_term_capital_gain_inclusion, dec!(2000));
    }

    #[test]
    fn basis_steps_up_by_total_inclusion_year_1() {
        let r = compute(&base());
        // $50k + $5k inclusion = $55k.
        assert_eq!(r.adjusted_basis_year_end, dec!(55000));
    }

    #[test]
    fn pti_account_year_end_equals_total_inclusion_when_no_distribution() {
        let r = compute(&base());
        assert_eq!(r.pti_account_year_end, dec!(5000));
    }

    #[test]
    fn no_distribution_no_taxable_dividend() {
        let r = compute(&base());
        assert_eq!(r.taxable_dividend_distribution, Decimal::ZERO);
        assert_eq!(r.distribution_from_pti, Decimal::ZERO);
    }

    #[test]
    fn distribution_fully_absorbed_by_pti_no_taxable_dividend() {
        // $5k inclusion + $0 prior PTI = $5k PTI pool. $3k distribution
        // fully absorbed.
        let mut i = base();
        i.distributions_received = dec!(3000);
        let r = compute(&i);
        assert_eq!(r.distribution_from_pti, dec!(3000));
        assert_eq!(r.taxable_dividend_distribution, Decimal::ZERO);
        assert_eq!(r.pti_account_year_end, dec!(2000));
    }

    #[test]
    fn distribution_exceeds_pti_excess_taxable_as_dividend() {
        // $5k PTI pool, $8k distribution → $5k PTI absorbed, $3k taxable.
        let mut i = base();
        i.distributions_received = dec!(8000);
        let r = compute(&i);
        assert_eq!(r.distribution_from_pti, dec!(5000));
        assert_eq!(r.taxable_dividend_distribution, dec!(3000));
        assert_eq!(r.pti_account_year_end, Decimal::ZERO);
    }

    #[test]
    fn prior_pti_carries_into_current_year() {
        // $10k prior PTI + $5k current = $15k pool. $8k distribution →
        // $8k from PTI, $0 taxable, $7k PTI remains.
        let mut i = base();
        i.pti_account_year_start = dec!(10000);
        i.distributions_received = dec!(8000);
        let r = compute(&i);
        assert_eq!(r.distribution_from_pti, dec!(8000));
        assert_eq!(r.taxable_dividend_distribution, Decimal::ZERO);
        assert_eq!(r.pti_account_year_end, dec!(7000));
    }

    #[test]
    fn basis_decreases_by_pti_distribution() {
        // Basis $50k + inclusion $5k - $3k PTI distribution = $52k.
        let mut i = base();
        i.distributions_received = dec!(3000);
        let r = compute(&i);
        assert_eq!(r.adjusted_basis_year_end, dec!(52000));
    }

    #[test]
    fn basis_doesnt_decrease_for_taxable_dividend_portion() {
        // Taxable dividend portion comes from earnings excess of PTI —
        // doesn't reduce basis. Distribution $8k = $5k PTI + $3k dividend.
        // Basis $50k + $5k inclusion - $5k PTI = $50k. The $3k dividend
        // hits dividend bucket, not basis.
        let mut i = base();
        i.distributions_received = dec!(8000);
        let r = compute(&i);
        assert_eq!(r.adjusted_basis_year_end, dec!(50000));
    }

    #[test]
    fn multi_year_chain_basis_and_pti_evolve_correctly() {
        // Year 1: $5k inclusion, no dist → basis $55k, PTI $5k.
        let y1 = compute(&base());
        assert_eq!(y1.adjusted_basis_year_end, dec!(55000));
        assert_eq!(y1.pti_account_year_end, dec!(5000));

        // Year 2: $6k inclusion, $4k distribution.
        let y2 = compute(&Section1295Input {
            tax_year: 2025,
            pro_rata_ordinary_earnings: dec!(3000),
            pro_rata_net_capital_gain: dec!(3000),
            adjusted_basis_year_start: y1.adjusted_basis_year_end,
            distributions_received: dec!(4000),
            pti_account_year_start: y1.pti_account_year_end,
        });
        // PTI pool = $5k + $6k = $11k. $4k distribution → all PTI.
        // Basis = $55k + $6k - $4k = $57k. PTI ends = $7k.
        assert_eq!(y2.distribution_from_pti, dec!(4000));
        assert_eq!(y2.adjusted_basis_year_end, dec!(57000));
        assert_eq!(y2.pti_account_year_end, dec!(7000));
    }

    #[test]
    fn zero_inclusion_zero_distribution_no_op() {
        let i = Section1295Input {
            tax_year: 2024,
            pro_rata_ordinary_earnings: Decimal::ZERO,
            pro_rata_net_capital_gain: Decimal::ZERO,
            adjusted_basis_year_start: dec!(50000),
            distributions_received: Decimal::ZERO,
            pti_account_year_start: Decimal::ZERO,
        };
        let r = compute(&i);
        assert_eq!(r.adjusted_basis_year_end, dec!(50000));
        assert!(r.note.contains("no QEF inclusion"));
    }

    #[test]
    fn negative_pfic_earnings_treated_as_zero() {
        // QEF inclusion is bounded below at zero per §1293(a) — a
        // PFIC loss isn't currently deductible to the shareholder.
        let mut i = base();
        i.pro_rata_ordinary_earnings = dec!(-1000);
        i.pro_rata_net_capital_gain = dec!(-500);
        let r = compute(&i);
        assert_eq!(r.ordinary_income_inclusion, Decimal::ZERO);
        assert_eq!(r.long_term_capital_gain_inclusion, Decimal::ZERO);
        assert_eq!(r.adjusted_basis_year_end, dec!(50000)); // unchanged
    }

    #[test]
    fn character_preserved_unlike_section_1296_mtm() {
        // The whole point: §1295 keeps LTCG as LTCG.
        let r = compute(&base());
        assert!(r.long_term_capital_gain_inclusion > Decimal::ZERO);
        // Compare to §1296 which would lump everything into ordinary.
    }

    #[test]
    fn pti_account_never_negative_when_distributions_exceed_pool() {
        let mut i = base();
        i.distributions_received = dec!(100000); // way over pool
        let r = compute(&i);
        assert_eq!(r.pti_account_year_end, Decimal::ZERO);
        assert!(r.pti_account_year_end >= Decimal::ZERO);
    }

    #[test]
    fn basis_never_negative_when_distributions_exceed_inclusion_plus_basis() {
        // Edge: tiny basis, big PTI distribution. Basis floors at zero.
        let mut i = base();
        i.adjusted_basis_year_start = dec!(100);
        i.distributions_received = dec!(5000);
        let r = compute(&i);
        assert!(r.adjusted_basis_year_end >= Decimal::ZERO);
    }

    #[test]
    fn note_text_distinguishes_distribution_vs_no_distribution_paths() {
        let no_dist = compute(&base());
        assert!(no_dist.note.contains("character preserved"));

        let mut i = base();
        i.distributions_received = dec!(8000);
        let with_dividend = compute(&i);
        assert!(with_dividend.note.contains("taxable as dividend"));
    }

    #[test]
    fn ordinary_only_pfic_still_includes_ordinary() {
        let mut i = base();
        i.pro_rata_net_capital_gain = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.ordinary_income_inclusion, dec!(3000));
        assert_eq!(r.long_term_capital_gain_inclusion, Decimal::ZERO);
    }

    #[test]
    fn ltcg_only_pfic_still_includes_ltcg() {
        let mut i = base();
        i.pro_rata_ordinary_earnings = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.ordinary_income_inclusion, Decimal::ZERO);
        assert_eq!(r.long_term_capital_gain_inclusion, dec!(2000));
    }
}
