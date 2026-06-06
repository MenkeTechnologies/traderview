//! Master Limited Partnership K-1 Unrelated Business Taxable Income
//! tracker for IRAs and qualified plans.
//!
//! Traders holding MLPs (ET, KMI, MPLX, EPD, NGL) in IRAs face a
//! gotcha the broker doesn't flag: under IRC §511-514, the IRA itself
//! is taxable on the share of MLP operating income passed through on
//! the K-1. The tax is owed by the IRA custodian via **Form 990-T**,
//! NOT by the IRA holder personally — but the cash comes out of the
//! IRA, eroding the retirement balance.
//!
//! Mechanics:
//!
//!   * **K-1 Box 1 — ordinary business income** flows directly to
//!     UBTI (the IRA's share of MLP operating P&L).
//!   * **§512(b) exclusions** — dividends, interest, royalties,
//!     short/long-term capital gains shown on the K-1 are NOT UBTI
//!     (they're passive investment income to the IRA, untouched by
//!     §511). These appear in K-1 Boxes 5, 6a, 8, 9a respectively.
//!   * **§514 debt-financed UBTI** — if the MLP holds acquisition
//!     indebtedness, even excluded income becomes UBTI to the extent
//!     of the debt-financed ratio. Caller supplies the ratio from
//!     K-1 Box 20V or the partner's footnote.
//!   * **§512(b)(12) specific deduction** — first $1,000 of total
//!     UBTI is deductible before tax applies.
//!   * **Trust-rate tax per §511(b)(2)** — IRAs and qualified plans
//!     use the compressed trust tax brackets (not corp rates). 2024
//!     trust brackets: 10% to $3,100, 24% to $11,150, 35% to $15,200,
//!     37% over $15,200. Compressed brackets mean a $20k UBTI year
//!     pays ~$6k in tax — a real bite.
//!
//! Pure compute. Caller passes per-MLP K-1 line items; we aggregate,
//! exclude per §512(b), apply §514 debt ratio, subtract §512(b)(12),
//! and run trust brackets.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// One MLP holding's K-1 line items relevant to UBTI computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlpK1 {
    pub mlp_name: String,
    /// K-1 Box 1 — ordinary business income/loss. Always UBTI.
    pub box_1_ordinary_business_income: Decimal,
    /// K-1 Box 5 — interest income. Excluded under §512(b)(1) unless
    /// debt-financed.
    pub box_5_interest_income: Decimal,
    /// K-1 Box 6a — ordinary dividends. Excluded under §512(b)(1).
    pub box_6a_dividends: Decimal,
    /// K-1 Box 8 — net short-term capital gain. Excluded under §512(b)(5).
    pub box_8_short_term_capital_gain: Decimal,
    /// K-1 Box 9a — net long-term capital gain. Excluded under §512(b)(5).
    pub box_9a_long_term_capital_gain: Decimal,
    /// K-1 Box 13 — §179 / other deductions allocable to UBTI activity.
    /// Positive number = deduction.
    pub box_13_deductions: Decimal,
    /// K-1 Box 20V — debt-financed income inclusion (§514 ratio applied
    /// at the partnership level). Caller supplies the dollar amount;
    /// we treat it as additive UBTI on top of Box 1.
    pub box_20v_debt_financed_ubti: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlpUbtiInput {
    pub tax_year: i32,
    pub mlps: Vec<MlpK1>,
    /// Specific-deduction override. Defaults to $1,000 per §512(b)(12).
    /// Caller can pass a custom amount if a particular trust regime
    /// applies (e.g. some pension plans have different thresholds).
    pub specific_deduction_override: Option<Decimal>,
    /// True to use trust brackets per §511(b)(2). False uses corp
    /// brackets per §511(a)(1) — only relevant for §501(c)(2) title-
    /// holding corporations and similar; IRAs always use trust.
    pub use_trust_brackets: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MlpUbtiResult {
    pub tax_year: i32,
    pub per_mlp: Vec<PerMlpUbti>,
    pub gross_ubti: Decimal,
    pub specific_deduction_applied: Decimal,
    pub taxable_ubti: Decimal,
    pub estimated_tax: Decimal,
    pub form_990t_required: bool,
    pub note: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerMlpUbti {
    pub mlp_name: String,
    pub ubti_contribution: Decimal,
    pub excluded_passive_income: Decimal,
    pub note: String,
}

fn dollar(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

/// 2024 trust tax brackets per §1(e). Returns the total tax owed at
/// the given taxable income. Compressed brackets — UBTI tax bites
/// hard above $15k.
pub fn trust_tax_2024(taxable_income: Decimal) -> Decimal {
    let ti = taxable_income.max(Decimal::ZERO);
    let mut tax = Decimal::ZERO;
    let brackets = [
        (dollar("3100"), dollar("0.10")),
        (dollar("11150"), dollar("0.24")),
        (dollar("15200"), dollar("0.35")),
    ];
    let top_rate = dollar("0.37");
    let mut prior_top = Decimal::ZERO;
    let mut remaining = ti;
    for (bracket_top, rate) in brackets {
        if remaining <= Decimal::ZERO {
            return tax.round_dp(2);
        }
        let width = bracket_top - prior_top;
        let taxed_here = remaining.min(width);
        tax += taxed_here * rate;
        remaining -= taxed_here;
        prior_top = bracket_top;
    }
    if remaining > Decimal::ZERO {
        tax += remaining * top_rate;
    }
    tax.round_dp(2)
}

pub fn compute(input: &MlpUbtiInput) -> MlpUbtiResult {
    let mut r = MlpUbtiResult {
        tax_year: input.tax_year,
        ..MlpUbtiResult::default()
    };

    for mlp in &input.mlps {
        // §511(a)(1) UBTI = Box 1 ordinary business income (always UBTI)
        // plus Box 20V debt-financed inclusion, minus Box 13 deductions
        // (only deductions allocable to UBTI activity — caller must
        // pre-filter, but we don't over-validate).
        let ubti = mlp.box_1_ordinary_business_income + mlp.box_20v_debt_financed_ubti
            - mlp.box_13_deductions;
        let excluded_passive = mlp.box_5_interest_income
            + mlp.box_6a_dividends
            + mlp.box_8_short_term_capital_gain
            + mlp.box_9a_long_term_capital_gain;
        r.gross_ubti += ubti;
        r.per_mlp.push(PerMlpUbti {
            mlp_name: mlp.mlp_name.clone(),
            ubti_contribution: ubti,
            excluded_passive_income: excluded_passive,
            note: format!(
                "Box 1 ${} + Box 20V ${} - Box 13 ${} = ${} UBTI; passive excluded ${}",
                mlp.box_1_ordinary_business_income,
                mlp.box_20v_debt_financed_ubti,
                mlp.box_13_deductions,
                ubti,
                excluded_passive,
            ),
        });
    }

    // §512(b)(12) specific deduction — first $1,000 NOT taxed.
    let sd = input
        .specific_deduction_override
        .unwrap_or_else(|| dollar("1000"));
    r.specific_deduction_applied = sd.min(r.gross_ubti.max(Decimal::ZERO));
    r.taxable_ubti = (r.gross_ubti - r.specific_deduction_applied).max(Decimal::ZERO);

    if input.use_trust_brackets {
        r.estimated_tax = trust_tax_2024(r.taxable_ubti);
    } else {
        // Flat 21% corp rate per §511(a)(1).
        r.estimated_tax = (r.taxable_ubti * dollar("0.21")).round_dp(2);
    }

    // §6011 + Reg. §1.6033-2(g): Form 990-T required if gross UBTI ≥ $1,000.
    r.form_990t_required = r.gross_ubti >= dollar("1000");

    r.note = if r.gross_ubti <= Decimal::ZERO {
        "no UBTI generated by these MLPs this year".into()
    } else if !r.form_990t_required {
        format!(
            "gross UBTI ${} < $1,000 — no Form 990-T required; no tax owed",
            r.gross_ubti
        )
    } else if r.taxable_ubti <= Decimal::ZERO {
        format!(
            "gross UBTI ${} absorbed by §512(b)(12) ${} specific deduction; Form 990-T required but no tax owed",
            r.gross_ubti, r.specific_deduction_applied
        )
    } else {
        format!(
            "gross UBTI ${} - §512(b)(12) ${} = ${} taxable; ${} estimated tax at {} rates; Form 990-T required",
            r.gross_ubti,
            r.specific_deduction_applied,
            r.taxable_ubti,
            r.estimated_tax,
            if input.use_trust_brackets {
                "trust"
            } else {
                "corp 21%"
            },
        )
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn mlp(name: &str, box_1: Decimal) -> MlpK1 {
        MlpK1 {
            mlp_name: name.into(),
            box_1_ordinary_business_income: box_1,
            box_5_interest_income: Decimal::ZERO,
            box_6a_dividends: Decimal::ZERO,
            box_8_short_term_capital_gain: Decimal::ZERO,
            box_9a_long_term_capital_gain: Decimal::ZERO,
            box_13_deductions: Decimal::ZERO,
            box_20v_debt_financed_ubti: Decimal::ZERO,
        }
    }

    fn input(mlps: Vec<MlpK1>) -> MlpUbtiInput {
        MlpUbtiInput {
            tax_year: 2024,
            mlps,
            specific_deduction_override: None,
            use_trust_brackets: true,
        }
    }

    #[test]
    fn single_mlp_below_1000_no_form_990t() {
        // $500 of UBTI — below threshold, no Form 990-T, no tax.
        let r = compute(&input(vec![mlp("ET", dec!(500))]));
        assert_eq!(r.gross_ubti, dec!(500));
        assert!(!r.form_990t_required);
        assert_eq!(r.estimated_tax, Decimal::ZERO);
        assert!(r.note.contains("$1,000"));
    }

    #[test]
    fn single_mlp_exactly_1000_triggers_form_but_zero_tax() {
        // $1,000 UBTI = at threshold (>=). Form required. §512(b)(12)
        // absorbs the whole amount → taxable $0.
        let r = compute(&input(vec![mlp("KMI", dec!(1000))]));
        assert!(r.form_990t_required);
        assert_eq!(r.specific_deduction_applied, dec!(1000));
        assert_eq!(r.taxable_ubti, Decimal::ZERO);
        assert_eq!(r.estimated_tax, Decimal::ZERO);
    }

    #[test]
    fn aggregate_across_multiple_mlps() {
        let r = compute(&input(vec![
            mlp("ET", dec!(800)),
            mlp("KMI", dec!(700)),
            mlp("EPD", dec!(500)),
        ]));
        assert_eq!(r.gross_ubti, dec!(2000));
        assert!(r.form_990t_required);
        assert_eq!(r.specific_deduction_applied, dec!(1000));
        assert_eq!(r.taxable_ubti, dec!(1000));
        // Trust 10% bracket: $1k × 10% = $100.
        assert_eq!(r.estimated_tax, dec!(100));
    }

    #[test]
    fn passive_income_excluded_from_ubti() {
        // $200 Box 1 ordinary + $5k dividends + $3k capital gains.
        // UBTI is only the $200 — the passive items are excluded.
        let mut m = mlp("MPLX", dec!(200));
        m.box_6a_dividends = dec!(5000);
        m.box_9a_long_term_capital_gain = dec!(3000);
        let r = compute(&input(vec![m]));
        assert_eq!(r.gross_ubti, dec!(200));
        assert_eq!(r.per_mlp[0].excluded_passive_income, dec!(8000));
        assert!(!r.form_990t_required);
    }

    #[test]
    fn debt_financed_inclusion_adds_to_ubti() {
        // $300 Box 1 + $1500 Box 20V debt-financed = $1800 UBTI.
        let mut m = mlp("EPD", dec!(300));
        m.box_20v_debt_financed_ubti = dec!(1500);
        let r = compute(&input(vec![m]));
        assert_eq!(r.gross_ubti, dec!(1800));
        assert!(r.form_990t_required);
        // $1800 - $1000 = $800 taxable. Trust 10% = $80.
        assert_eq!(r.taxable_ubti, dec!(800));
        assert_eq!(r.estimated_tax, dec!(80));
    }

    #[test]
    fn box_13_deductions_reduce_ubti() {
        let mut m = mlp("KMI", dec!(2500));
        m.box_13_deductions = dec!(800);
        let r = compute(&input(vec![m]));
        assert_eq!(r.gross_ubti, dec!(1700));
        // $1700 - $1000 = $700 taxable. Trust 10% = $70.
        assert_eq!(r.estimated_tax, dec!(70));
    }

    #[test]
    fn negative_ubti_doesnt_create_negative_specific_deduction() {
        // Big Box 13 deductions create UBTI loss. Specific deduction
        // should not artificially increase the loss.
        let mut m = mlp("ET", dec!(500));
        m.box_13_deductions = dec!(2000);
        let r = compute(&input(vec![m]));
        assert_eq!(r.gross_ubti, dec!(-1500));
        assert_eq!(r.specific_deduction_applied, Decimal::ZERO);
        assert_eq!(r.taxable_ubti, Decimal::ZERO);
        assert_eq!(r.estimated_tax, Decimal::ZERO);
    }

    #[test]
    fn trust_brackets_2024_compressed_correctly() {
        // 2024 trust brackets:
        //   $0–$3,100 at 10%
        //   $3,100–$11,150 at 24%
        //   $11,150–$15,200 at 35%
        //   $15,200+ at 37%
        // At $20k income:
        //   10% × $3,100 = $310
        //   24% × $8,050 = $1,932
        //   35% × $4,050 = $1,417.50
        //   37% × $4,800 = $1,776
        //   Total = $5,435.50
        let t = trust_tax_2024(dec!(20000));
        assert_eq!(t, dec!(5435.50));
    }

    #[test]
    fn trust_tax_at_each_bracket_threshold() {
        assert_eq!(trust_tax_2024(dec!(3100)), dec!(310));
        // Up to $11,150: $310 + 24% × $8,050 = $310 + $1,932 = $2,242.
        assert_eq!(trust_tax_2024(dec!(11150)), dec!(2242));
        // Up to $15,200: $2,242 + 35% × $4,050 = $2,242 + $1,417.50 = $3,659.50.
        assert_eq!(trust_tax_2024(dec!(15200)), dec!(3659.50));
    }

    #[test]
    fn trust_tax_zero_income_zero_tax() {
        assert_eq!(trust_tax_2024(Decimal::ZERO), Decimal::ZERO);
    }

    #[test]
    fn trust_tax_negative_clamped_to_zero() {
        assert_eq!(trust_tax_2024(dec!(-5000)), Decimal::ZERO);
    }

    #[test]
    fn corp_rate_flat_21pct_when_trust_brackets_false() {
        let mut i = input(vec![mlp("ET", dec!(20000))]);
        i.use_trust_brackets = false;
        let r = compute(&i);
        // $20k - $1k = $19k × 21% = $3,990.
        assert_eq!(r.estimated_tax, dec!(3990));
    }

    #[test]
    fn specific_deduction_override_replaces_1000_default() {
        let mut i = input(vec![mlp("ET", dec!(5000))]);
        i.specific_deduction_override = Some(dec!(2500));
        let r = compute(&i);
        assert_eq!(r.specific_deduction_applied, dec!(2500));
        assert_eq!(r.taxable_ubti, dec!(2500));
    }

    #[test]
    fn empty_mlp_list_no_ubti() {
        let r = compute(&input(vec![]));
        assert_eq!(r.gross_ubti, Decimal::ZERO);
        assert!(!r.form_990t_required);
        assert!(r.note.contains("no UBTI"));
    }

    #[test]
    fn loss_with_passive_income_still_zero_ubti() {
        // Big passive income, small loss in Box 1. UBTI = -$500
        // (loss), passive excluded from UBTI even though it's positive.
        let mut m = mlp("KMI", dec!(-500));
        m.box_6a_dividends = dec!(10000);
        let r = compute(&input(vec![m]));
        assert_eq!(r.gross_ubti, dec!(-500));
        assert_eq!(r.taxable_ubti, Decimal::ZERO);
        assert!(!r.form_990t_required);
    }

    #[test]
    fn per_mlp_breakdown_preserves_names() {
        let r = compute(&input(vec![mlp("ET", dec!(800)), mlp("EPD", dec!(700))]));
        assert_eq!(r.per_mlp.len(), 2);
        assert_eq!(r.per_mlp[0].mlp_name, "ET");
        assert_eq!(r.per_mlp[1].mlp_name, "EPD");
    }

    #[test]
    fn high_ubti_year_uses_compressed_trust_top_bracket() {
        // $30k UBTI: gross $30k, minus $1k = $29k taxable.
        // Trust brackets above $15,200 are 37%.
        let r = compute(&input(vec![mlp("ET", dec!(30000))]));
        // Tax to $15,200: $3,659.50 (from earlier).
        // 37% × ($29,000 - $15,200) = 37% × $13,800 = $5,106.
        // Total: $8,765.50.
        assert_eq!(r.estimated_tax, dec!(8765.50));
    }
}
