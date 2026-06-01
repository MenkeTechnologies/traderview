//! IRC § 41 — Credit for Increasing Research Activities (R&D credit).
//!
//! Practical for algorithmic traders building custom trading systems,
//! data pipelines, machine-learning models, and other software that
//! qualifies as "research" under § 41(d) (technological in nature,
//! intended to develop new or improved business component, with
//! elements of experimentation, and substantially all activities
//! constitute process of experimentation).
//!
//! Two computation methods:
//!
//! § 41(a) Regular Credit — 20% × (current QRE − base amount), where
//! base amount = greater of (i) fixed_base_percentage × average gross
//! receipts of 4 prior years, OR (ii) 50% × current-year QRE. The
//! fixed-base percentage is capped at 16%. Companies with no QRE in
//! any of the 3 prior tax years use a "start-up" 3% fixed-base
//! percentage.
//!
//! § 41(c)(4) Alternative Simplified Credit (ASC) — 14% × (current
//! QRE − 50% × average QRE for prior 3 years). If the taxpayer had no
//! QRE in any of the 3 prior years, ASC = 6% × current QRE (startup
//! path under § 41(c)(4)(B)). ASC election must be made on a timely-
//! filed ORIGINAL return — cannot be elected on amended return.
//!
//! § 280C(c) interaction — by default, § 280C(c)(1) requires taxpayer
//! to REDUCE the § 174 deduction by the amount of the § 41 credit
//! (anti-double-dip). § 280C(c)(2) lets taxpayer elect REDUCED CREDIT
//! = full credit × (1 − 21%), where 21% is the max corporate rate, and
//! keep the full § 174 deduction in exchange. Reduced-credit election
//! must be on the original return per § 280C(c)(3) — invalid if made
//! on an amended return per IRS.
//!
//! § 174 capitalization (post-TCJA 2022) — R&D expenses MUST be
//! capitalized and amortized (5 years domestic / 15 years foreign).
//! § 41 credit is computed on the FULL QRE despite § 174 capitalization
//! — the credit is independent of the deduction timing.
//!
//! Citations: 26 U.S.C. § 41; § 41(a)(1) (regular 20% credit);
//! § 41(c)(1) (fixed-base-percentage definition); § 41(c)(3) (fixed-
//! base-percentage cap at 16%); § 41(c)(4)(A) (ASC 14%); § 41(c)(4)(B)
//! (ASC 6% startup); § 41(d) (qualified research definition);
//! § 280C(c)(1) (default deduction reduction); § 280C(c)(2) (reduced-
//! credit election); § 280C(c)(3) (election must be on original return);
//! § 174 (R&D capitalization post-TCJA 2022).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditMethod {
    Regular,
    AlternativeSimplified,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section41Input {
    pub credit_method: CreditMethod,
    pub current_year_qre_cents: i64,
    /// Average qualified research expenses for the 3 preceding tax
    /// years. Drives ASC base = 50% × this average.
    pub prior_3_year_avg_qre_cents: i64,
    /// Whether taxpayer had no QRE in any of the 3 preceding tax years.
    /// Triggers ASC 6% startup path under § 41(c)(4)(B).
    pub no_prior_3_year_qre: bool,
    /// Regular-method input: fixed-base percentage × 100 (so 0.05 = 500
    /// for 5%). Capped at 16% (1600 basis points × 100).
    pub fixed_base_percentage_bp: u32,
    /// Regular-method input: average gross receipts for the 4 preceding
    /// tax years.
    pub prior_4_year_avg_gross_receipts_cents: i64,
    /// Regular-method input: whether the taxpayer is a "start-up" with
    /// no QRE in any of the 3 prior years. Triggers 3% fixed-base
    /// percentage under § 41(c)(3)(B).
    pub regular_method_startup: bool,
    /// Whether taxpayer elects § 280C(c)(2) reduced-credit election to
    /// preserve the full § 174 deduction. Reduces credit by 21%.
    pub elects_280c_reduced_credit: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section41Result {
    pub method: CreditMethod,
    pub base_amount_cents: i64,
    pub gross_credit_cents: i64,
    pub credit_after_280c_election_cents: i64,
    pub final_credit_cents: i64,
    pub deduction_reduction_required: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section41Input) -> Section41Result {
    let qre = input.current_year_qre_cents.max(0);
    let prior_3_avg = input.prior_3_year_avg_qre_cents.max(0);

    let (gross_credit, base_amount, citation_method) = match input.credit_method {
        CreditMethod::AlternativeSimplified => {
            // § 41(c)(4): ASC = 14% × (current QRE − 50% × prior 3-year avg QRE).
            // § 41(c)(4)(B) startup: 6% × current QRE if no QRE in 3 prior years.
            if input.no_prior_3_year_qre {
                let credit = (qre as i128 * 6 / 100) as i64;
                (
                    credit,
                    0_i64,
                    "26 U.S.C. § 41(c)(4)(B) — ASC startup path: 6% × current QRE when no QRE in any of 3 prior years",
                )
            } else {
                let base = ((prior_3_avg as i128) / 2) as i64;
                let excess = (qre - base).max(0);
                let credit = (excess as i128 * 14 / 100) as i64;
                (
                    credit,
                    base,
                    "26 U.S.C. § 41(c)(4)(A) — Alternative Simplified Credit: 14% × (QRE − 50% × prior 3-year average QRE)",
                )
            }
        }
        CreditMethod::Regular => {
            // § 41(a): 20% × (current QRE − base amount).
            // Base amount = greater of (fixed-base % × avg gross receipts) OR (50% × current QRE).
            // § 41(c)(3) fixed-base % capped at 16%. Startup uses 3% under § 41(c)(3)(B).
            let fbp_bp = if input.regular_method_startup {
                300 // 3% expressed in bp×100 form
            } else {
                input.fixed_base_percentage_bp.min(1600)
            };
            let avg_gross_receipts = input.prior_4_year_avg_gross_receipts_cents.max(0);
            let fbp_base =
                ((avg_gross_receipts as i128) * (fbp_bp as i128) / 10_000) as i64;
            let fifty_pct_qre = ((qre as i128) / 2) as i64;
            let base = fbp_base.max(fifty_pct_qre);
            let excess = (qre - base).max(0);
            let credit = ((excess as i128) * 20 / 100) as i64;
            (
                credit,
                base,
                "26 U.S.C. § 41(a)(1)/(c)(1) — Regular Credit: 20% × (QRE − greater of fixed-base-% × avg gross receipts or 50% × current QRE)",
            )
        }
    };

    // § 280C(c)(2) reduced-credit election: reduce credit by 21%.
    let after_280c = if input.elects_280c_reduced_credit {
        ((gross_credit as i128) * 79 / 100) as i64
    } else {
        gross_credit
    };
    let deduction_reduction_required = !input.elects_280c_reduced_credit;

    let citation = if input.elects_280c_reduced_credit {
        "26 U.S.C. § 280C(c)(2) — reduced-credit election: credit × (1 − 21%) and taxpayer keeps full § 174 deduction; § 280C(c)(3) election must be on original return"
    } else {
        citation_method
    };

    let note = format!(
        "Method = {:?}. Current QRE = {} cents. Prior 3-year avg QRE = {} cents. Base amount = {} cents. Gross credit = {} cents. § 280C(c) election made = {}. Credit after § 280C = {} cents. Default § 280C(c)(1) deduction-reduction {}.",
        input.credit_method,
        qre,
        prior_3_avg,
        base_amount,
        gross_credit,
        input.elects_280c_reduced_credit,
        after_280c,
        if deduction_reduction_required { "APPLIES (reduces § 174 deduction)" } else { "WAIVED via reduced-credit election" },
    );

    Section41Result {
        method: input.credit_method,
        base_amount_cents: base_amount,
        gross_credit_cents: gross_credit,
        credit_after_280c_election_cents: after_280c,
        final_credit_cents: after_280c,
        deduction_reduction_required,
        citation,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        method: CreditMethod,
        qre: i64,
        prior_3_avg: i64,
        no_prior: bool,
        fbp_bp: u32,
        prior_4_gross: i64,
        startup: bool,
        elects_280c: bool,
    ) -> Section41Input {
        Section41Input {
            credit_method: method,
            current_year_qre_cents: qre,
            prior_3_year_avg_qre_cents: prior_3_avg,
            no_prior_3_year_qre: no_prior,
            fixed_base_percentage_bp: fbp_bp,
            prior_4_year_avg_gross_receipts_cents: prior_4_gross,
            regular_method_startup: startup,
            elects_280c_reduced_credit: elects_280c,
        }
    }

    #[test]
    fn asc_standard_14_percent_excess_over_50_pct_prior_avg() {
        // QRE $200K, prior 3-year avg $100K. ASC base = 50% × $100K = $50K.
        // Excess = $200K − $50K = $150K. Credit = 14% × $150K = $21K.
        let r = compute(&input(
            CreditMethod::AlternativeSimplified,
            200_000_00,
            100_000_00,
            false,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.base_amount_cents, 50_000_00);
        assert_eq!(r.gross_credit_cents, 21_000_00);
        assert_eq!(r.final_credit_cents, 21_000_00);
    }

    #[test]
    fn asc_startup_6_percent_when_no_prior_qre() {
        // No QRE in 3 prior years → ASC 6% × current QRE.
        let r = compute(&input(
            CreditMethod::AlternativeSimplified,
            100_000_00,
            0,
            true,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.gross_credit_cents, 6_000_00);
        assert!(r.citation.contains("startup"));
        assert!(r.citation.contains("6%"));
    }

    #[test]
    fn asc_qre_below_50_pct_prior_avg_no_credit() {
        // QRE $40K, prior avg $100K. Base = $50K. QRE < base → 0 credit.
        let r = compute(&input(
            CreditMethod::AlternativeSimplified,
            40_000_00,
            100_000_00,
            false,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.gross_credit_cents, 0);
    }

    #[test]
    fn regular_20_percent_excess_over_fixed_base() {
        // QRE $200K, FBP 5% × $1M gross = $50K. 50% × QRE = $100K. Base
        // = max($50K, $100K) = $100K. Excess = $100K. Credit = 20% × $100K
        // = $20K.
        let r = compute(&input(
            CreditMethod::Regular,
            200_000_00,
            0,
            false,
            500,
            1_000_000_00,
            false,
            false,
        ));
        assert_eq!(r.base_amount_cents, 100_000_00);
        assert_eq!(r.gross_credit_cents, 20_000_00);
    }

    #[test]
    fn regular_fixed_base_percentage_capped_at_16() {
        // FBP attempted 20% (2000 bp). Cap = 16% (1600 bp).
        // FBP base = 16% × $1M gross = $160K. 50% × QRE $200K = $100K.
        // Base = max($160K, $100K) = $160K. Excess = $40K. Credit = $8K.
        let r = compute(&input(
            CreditMethod::Regular,
            200_000_00,
            0,
            false,
            2000,
            1_000_000_00,
            false,
            false,
        ));
        assert_eq!(r.base_amount_cents, 160_000_00);
        assert_eq!(r.gross_credit_cents, 8_000_00);
    }

    #[test]
    fn regular_startup_uses_3_percent_fbp() {
        // Startup FBP forced to 3% (300 bp) regardless of input fbp.
        // 3% × $1M = $30K. 50% × $200K = $100K. Base = max($30K, $100K)
        // = $100K. Excess = $100K. Credit = $20K.
        let r = compute(&input(
            CreditMethod::Regular,
            200_000_00,
            0,
            false,
            500, // input ignored when startup
            1_000_000_00,
            true,
            false,
        ));
        // Base = max(3% × $1M = $30K, 50% × $200K = $100K) = $100K.
        assert_eq!(r.base_amount_cents, 100_000_00);
    }

    #[test]
    fn regular_50_pct_qre_floor_applies_when_higher_than_fbp_base() {
        // FBP 1% × $1M = $10K. 50% × QRE $200K = $100K. Base = max(_,
        // $100K) = $100K. Excess = $100K. Credit = $20K.
        let r = compute(&input(
            CreditMethod::Regular,
            200_000_00,
            0,
            false,
            100,
            1_000_000_00,
            false,
            false,
        ));
        assert_eq!(r.base_amount_cents, 100_000_00);
        assert_eq!(r.gross_credit_cents, 20_000_00);
    }

    #[test]
    fn section_280c_reduced_credit_reduces_by_21_percent() {
        // Without election: $20K credit. With election: $20K × 79% = $15,800.
        let r = compute(&input(
            CreditMethod::AlternativeSimplified,
            200_000_00,
            0,
            true,
            0,
            0,
            false,
            true,
        ));
        // Startup path 6% × $200K = $12K. With 280C reduced: $12K × 79% = $9,480.
        assert_eq!(r.gross_credit_cents, 12_000_00);
        assert_eq!(r.final_credit_cents, 9_480_00);
        assert!(!r.deduction_reduction_required);
        assert!(r.citation.contains("§ 280C(c)(2)"));
    }

    #[test]
    fn without_280c_election_deduction_reduction_applies() {
        let r = compute(&input(
            CreditMethod::AlternativeSimplified,
            200_000_00,
            0,
            true,
            0,
            0,
            false,
            false,
        ));
        assert!(r.deduction_reduction_required);
        assert_eq!(r.gross_credit_cents, r.final_credit_cents);
    }

    #[test]
    fn asc_method_selection_path() {
        let r = compute(&input(
            CreditMethod::AlternativeSimplified,
            100_000_00,
            50_000_00,
            false,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.method, CreditMethod::AlternativeSimplified);
    }

    #[test]
    fn regular_method_selection_path() {
        let r = compute(&input(
            CreditMethod::Regular,
            100_000_00,
            0,
            false,
            500,
            1_000_000_00,
            false,
            false,
        ));
        assert_eq!(r.method, CreditMethod::Regular);
    }

    #[test]
    fn worked_example_algo_trader_asc() {
        // Algo trader QRE $500K (custom trading engine + ML model), prior
        // avg $200K. ASC base = $100K. Excess = $400K. Credit = $56K.
        let r = compute(&input(
            CreditMethod::AlternativeSimplified,
            500_000_00,
            200_000_00,
            false,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.gross_credit_cents, 56_000_00);
    }

    #[test]
    fn worked_example_algo_trader_with_280c_election() {
        // Same as above with 280C election: $56K × 79% = $44,240.
        let r = compute(&input(
            CreditMethod::AlternativeSimplified,
            500_000_00,
            200_000_00,
            false,
            0,
            0,
            false,
            true,
        ));
        assert_eq!(r.final_credit_cents, 44_240_00);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let r_asc = compute(&input(
            CreditMethod::AlternativeSimplified,
            200_000_00,
            100_000_00,
            false,
            0,
            0,
            false,
            false,
        ));
        assert!(r_asc.citation.contains("§ 41(c)(4)(A)"));

        let r_asc_startup = compute(&input(
            CreditMethod::AlternativeSimplified,
            200_000_00,
            0,
            true,
            0,
            0,
            false,
            false,
        ));
        assert!(r_asc_startup.citation.contains("§ 41(c)(4)(B)"));

        let r_regular = compute(&input(
            CreditMethod::Regular,
            200_000_00,
            0,
            false,
            500,
            1_000_000_00,
            false,
            false,
        ));
        assert!(r_regular.citation.contains("§ 41(a)(1)"));

        let r_280c = compute(&input(
            CreditMethod::AlternativeSimplified,
            200_000_00,
            100_000_00,
            false,
            0,
            0,
            false,
            true,
        ));
        assert!(r_280c.citation.contains("§ 280C(c)(2)"));
        assert!(r_280c.citation.contains("21%"));
    }

    #[test]
    fn negative_inputs_clamped() {
        let r = compute(&input(
            CreditMethod::AlternativeSimplified,
            -1,
            -1,
            false,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.gross_credit_cents, 0);
    }

    #[test]
    fn asc_startup_280c_combination() {
        // No prior QRE + 280C election. 6% × $200K = $12K → 79% = $9,480.
        let r = compute(&input(
            CreditMethod::AlternativeSimplified,
            200_000_00,
            0,
            true,
            0,
            0,
            false,
            true,
        ));
        assert_eq!(r.gross_credit_cents, 12_000_00);
        assert_eq!(r.final_credit_cents, 9_480_00);
    }

    #[test]
    fn regular_startup_with_280c_election() {
        // Startup forced 3% FBP + 280C reduced credit.
        let r = compute(&input(
            CreditMethod::Regular,
            200_000_00,
            0,
            false,
            500, // ignored due to startup
            1_000_000_00,
            true,
            true,
        ));
        // Base = max(3% × $1M = $30K, 50% × $200K = $100K) = $100K.
        // Excess = $100K. Credit = $20K. After 280C: $20K × 79% = $15.8K.
        assert_eq!(r.base_amount_cents, 100_000_00);
        assert_eq!(r.final_credit_cents, 15_800_00);
    }

    #[test]
    fn asc_zero_qre_zero_credit() {
        let r = compute(&input(
            CreditMethod::AlternativeSimplified,
            0,
            100_000_00,
            false,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.gross_credit_cents, 0);
    }

    #[test]
    fn deduction_reduction_flag_inverse_of_280c_election() {
        let without = compute(&input(
            CreditMethod::AlternativeSimplified,
            200_000_00,
            100_000_00,
            false,
            0,
            0,
            false,
            false,
        ));
        let with_election = compute(&input(
            CreditMethod::AlternativeSimplified,
            200_000_00,
            100_000_00,
            false,
            0,
            0,
            false,
            true,
        ));
        assert!(without.deduction_reduction_required);
        assert!(!with_election.deduction_reduction_required);
    }

    #[test]
    fn regular_at_16_pct_cap_boundary() {
        // FBP exactly 16% (1600 bp). Cap allows this; not capped further.
        let r = compute(&input(
            CreditMethod::Regular,
            200_000_00,
            0,
            false,
            1600,
            1_000_000_00,
            false,
            false,
        ));
        // 16% × $1M = $160K. 50% × $200K = $100K. Base = $160K.
        assert_eq!(r.base_amount_cents, 160_000_00);
    }

    #[test]
    fn asc_50_pct_base_calculation_correctness() {
        // Prior 3-year avg $50K → ASC base = $25K.
        let r = compute(&input(
            CreditMethod::AlternativeSimplified,
            100_000_00,
            50_000_00,
            false,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.base_amount_cents, 25_000_00);
        // Credit = 14% × ($100K − $25K) = 14% × $75K = $10,500.
        assert_eq!(r.gross_credit_cents, 10_500_00);
    }
}
