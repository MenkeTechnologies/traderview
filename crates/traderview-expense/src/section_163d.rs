//! IRC §163(d) — Limitation on investment interest expense.
//!
//! Investors (taxpayers who have NOT made the §475(f) trader-in-
//! securities election) face this rather than §163(j) from iter 16.
//! Investment interest expense (margin interest, interest on loans
//! to buy securities, etc.) is deductible only up to **net
//! investment income** per §163(d)(1). Excess carries forward
//! INDEFINITELY under §163(d)(2).
//!
//! **Net investment income** per §163(d)(4) includes:
//!
//!   * Interest income from investments (taxable bond interest,
//!     T-bill interest, money-market interest).
//!   * Ordinary (non-qualified) dividends.
//!   * Net short-term capital gain (always counted).
//!   * **Net long-term capital gain** — ONLY if the taxpayer
//!     ELECTS under §163(d)(4)(B) to treat it as investment income.
//!     The election triggers loss of LTCG preferential rates on the
//!     elected portion.
//!   * **Qualified dividends** — likewise only if elected under
//!     §1(h)(11)(D)(i). Election forfeits QD preferential rate.
//!
//! Net investment income is REDUCED by:
//!
//!   * Investment expenses other than interest (§163(d)(4)(C)).
//!     Caller passes this as a single aggregate.
//!
//! The election tradeoff is the non-obvious part: a taxpayer with
//! large unused QDs / LTCG can elect them into net investment
//! income to unlock the interest deduction NOW, but pays ordinary-
//! income rates instead of preferential capital-gain rates on the
//! elected portion. Worth it if marginal ordinary rate × elected QD <
//! deferred interest deduction × ordinary rate. Model both paths and
//! let the caller compare.
//!
//! Pure compute. Caller asserts dollar amounts + election flags;
//! we compute the limit, current-year deduction, carryforward, and
//! the dollar amounts of QD/LTCG that lost preferential treatment.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section163dInput {
    pub tax_year: i32,
    /// Current-year investment interest expense (margin interest +
    /// loan-to-carry interest).
    pub investment_interest_expense: Decimal,
    /// Interest income from investments.
    pub interest_income: Decimal,
    /// Ordinary (non-qualified) dividends.
    pub ordinary_dividends: Decimal,
    /// Qualified dividends — only counted toward net investment
    /// income if `elect_qualified_dividends_as_investment_income` is
    /// true. Otherwise these get LTCG preferential rates and stay
    /// out of the §163(d) numerator.
    pub qualified_dividends: Decimal,
    /// Net short-term capital gain (always counted toward investment
    /// income — STCG is ordinary regardless).
    pub net_short_term_capital_gain: Decimal,
    /// Net long-term capital gain — only counted if
    /// `elect_long_term_capital_gain_as_investment_income`.
    pub net_long_term_capital_gain: Decimal,
    /// Investment expenses other than interest (§163(d)(4)(C)) — e.g.
    /// portion of management fees deductible as investment expense.
    /// Reduces net investment income.
    pub other_investment_expenses: Decimal,
    /// Carryforward of disallowed investment interest from prior years
    /// per §163(d)(2). Added to current-year expense.
    pub prior_year_carryforward: Decimal,
    /// §1(h)(11)(D)(i) election: treat QDs as investment income.
    /// Boosts §163(d) limit; QDs forfeit preferential rate.
    pub elect_qualified_dividends_as_investment_income: bool,
    /// §163(d)(4)(B)(iii) election: treat net LTCG as investment
    /// income. Same tradeoff applies.
    pub elect_long_term_capital_gain_as_investment_income: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section163dResult {
    pub tax_year: i32,
    /// Sum of all items counted in net investment income (pre-expense).
    pub gross_investment_income: Decimal,
    pub net_investment_income: Decimal,
    pub total_expense_available: Decimal,
    pub deductible_this_year: Decimal,
    pub carryforward_to_next_year: Decimal,
    /// QDs that lost preferential rate by being elected as investment
    /// income (sub them in at ordinary rates instead).
    pub qualified_dividends_lost_preferential_rate: Decimal,
    /// LTCG that lost preferential rate similarly.
    pub long_term_capital_gain_lost_preferential_rate: Decimal,
    pub note: String,
}

pub fn compute(input: &Section163dInput) -> Section163dResult {
    let mut r = Section163dResult {
        tax_year: input.tax_year,
        ..Section163dResult::default()
    };

    // Gross investment income depends on what's elected in.
    let qd_in =
        if input.elect_qualified_dividends_as_investment_income {
            input.qualified_dividends.max(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };
    let ltcg_in =
        if input.elect_long_term_capital_gain_as_investment_income {
            input.net_long_term_capital_gain.max(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };

    r.gross_investment_income = input.interest_income
        + input.ordinary_dividends
        + input.net_short_term_capital_gain.max(Decimal::ZERO)
        + qd_in
        + ltcg_in;

    r.net_investment_income = (r.gross_investment_income
        - input.other_investment_expenses.max(Decimal::ZERO))
        .max(Decimal::ZERO);

    r.qualified_dividends_lost_preferential_rate = qd_in;
    r.long_term_capital_gain_lost_preferential_rate = ltcg_in;

    r.total_expense_available =
        input.investment_interest_expense + input.prior_year_carryforward;

    if r.total_expense_available <= Decimal::ZERO {
        r.note = "no investment interest to limit".into();
        return r;
    }

    r.deductible_this_year = r.total_expense_available.min(r.net_investment_income);
    r.carryforward_to_next_year =
        (r.total_expense_available - r.deductible_this_year).max(Decimal::ZERO);

    r.note = if r.carryforward_to_next_year > Decimal::ZERO {
        let mut note = format!(
            "§163(d) limit ${} (net investment income), ${} deducted, ${} carries forward",
            r.net_investment_income, r.deductible_this_year, r.carryforward_to_next_year
        );
        if r.qualified_dividends_lost_preferential_rate > Decimal::ZERO
            || r.long_term_capital_gain_lost_preferential_rate > Decimal::ZERO
        {
            note.push_str(&format!(
                " (tradeoff: ${} QD + ${} LTCG forfeited preferential rate per §1(h)(11)(D)(i) election)",
                r.qualified_dividends_lost_preferential_rate,
                r.long_term_capital_gain_lost_preferential_rate
            ));
        }
        note
    } else {
        format!(
            "§163(d): full ${} deducted within ${} net investment income limit",
            r.deductible_this_year, r.net_investment_income
        )
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section163dInput {
        Section163dInput {
            tax_year: 2024,
            investment_interest_expense: dec!(10000),
            interest_income: dec!(3000),
            ordinary_dividends: dec!(1000),
            qualified_dividends: dec!(5000),
            net_short_term_capital_gain: dec!(2000),
            net_long_term_capital_gain: dec!(8000),
            other_investment_expenses: Decimal::ZERO,
            prior_year_carryforward: Decimal::ZERO,
            elect_qualified_dividends_as_investment_income: false,
            elect_long_term_capital_gain_as_investment_income: false,
        }
    }

    #[test]
    fn baseline_without_elections_uses_interest_ord_div_stcg() {
        // Interest $3k + ordinary dividends $1k + STCG $2k = $6k NII.
        // $10k expense → $6k deducted, $4k carries forward.
        let r = compute(&base());
        assert_eq!(r.gross_investment_income, dec!(6000));
        assert_eq!(r.net_investment_income, dec!(6000));
        assert_eq!(r.deductible_this_year, dec!(6000));
        assert_eq!(r.carryforward_to_next_year, dec!(4000));
        assert_eq!(r.qualified_dividends_lost_preferential_rate, Decimal::ZERO);
        assert_eq!(r.long_term_capital_gain_lost_preferential_rate, Decimal::ZERO);
    }

    #[test]
    fn qd_election_boosts_limit_and_forfeits_preferential_rate() {
        let mut i = base();
        i.elect_qualified_dividends_as_investment_income = true;
        let r = compute(&i);
        // Now NII = $6k + $5k QD = $11k. Expense $10k → fully deductible.
        assert_eq!(r.net_investment_income, dec!(11000));
        assert_eq!(r.deductible_this_year, dec!(10000));
        assert_eq!(r.carryforward_to_next_year, Decimal::ZERO);
        // The $5k QD pays ordinary rate now instead of QD preferential rate.
        assert_eq!(r.qualified_dividends_lost_preferential_rate, dec!(5000));
    }

    #[test]
    fn ltcg_election_boosts_limit_and_forfeits_preferential_rate() {
        let mut i = base();
        i.elect_long_term_capital_gain_as_investment_income = true;
        let r = compute(&i);
        // NII = $6k + $8k LTCG = $14k. Expense $10k → fully deductible.
        assert_eq!(r.net_investment_income, dec!(14000));
        assert_eq!(r.deductible_this_year, dec!(10000));
        assert_eq!(r.long_term_capital_gain_lost_preferential_rate, dec!(8000));
    }

    #[test]
    fn both_elections_stack() {
        let mut i = base();
        i.elect_qualified_dividends_as_investment_income = true;
        i.elect_long_term_capital_gain_as_investment_income = true;
        let r = compute(&i);
        // NII = $6k + $5k + $8k = $19k. Expense $10k → fully deductible.
        assert_eq!(r.net_investment_income, dec!(19000));
        assert_eq!(r.deductible_this_year, dec!(10000));
        assert_eq!(r.qualified_dividends_lost_preferential_rate, dec!(5000));
        assert_eq!(r.long_term_capital_gain_lost_preferential_rate, dec!(8000));
    }

    #[test]
    fn other_investment_expenses_reduce_nii() {
        let mut i = base();
        i.other_investment_expenses = dec!(2000);
        let r = compute(&i);
        // NII = $6k gross - $2k expenses = $4k. Expense $10k → $4k deductible, $6k carries.
        assert_eq!(r.net_investment_income, dec!(4000));
        assert_eq!(r.deductible_this_year, dec!(4000));
        assert_eq!(r.carryforward_to_next_year, dec!(6000));
    }

    #[test]
    fn nii_cannot_go_negative_other_expenses_exceed_gross() {
        let mut i = base();
        i.other_investment_expenses = dec!(20000);
        let r = compute(&i);
        assert_eq!(r.net_investment_income, Decimal::ZERO);
        assert_eq!(r.deductible_this_year, Decimal::ZERO);
        assert_eq!(r.carryforward_to_next_year, dec!(10000));
    }

    #[test]
    fn prior_carryforward_stacks_with_current_expense() {
        // $5k current + $15k prior = $20k available. NII $6k → $6k deductible, $14k carries.
        let mut i = base();
        i.investment_interest_expense = dec!(5000);
        i.prior_year_carryforward = dec!(15000);
        let r = compute(&i);
        assert_eq!(r.total_expense_available, dec!(20000));
        assert_eq!(r.deductible_this_year, dec!(6000));
        assert_eq!(r.carryforward_to_next_year, dec!(14000));
    }

    #[test]
    fn no_expense_no_op() {
        let mut i = base();
        i.investment_interest_expense = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.deductible_this_year, Decimal::ZERO);
        assert!(r.note.contains("no investment interest"));
    }

    #[test]
    fn expense_fully_under_nii_no_carryforward() {
        let mut i = base();
        i.investment_interest_expense = dec!(3000);
        let r = compute(&i);
        assert_eq!(r.deductible_this_year, dec!(3000));
        assert_eq!(r.carryforward_to_next_year, Decimal::ZERO);
        assert!(r.note.contains("full"));
    }

    #[test]
    fn negative_stcg_treated_as_zero_in_nii() {
        // Net STCG loss should not REDUCE other investment income.
        let mut i = base();
        i.net_short_term_capital_gain = dec!(-5000);
        let r = compute(&i);
        // STCG floor = 0. NII = $3k interest + $1k ord div + 0 = $4k.
        assert_eq!(r.gross_investment_income, dec!(4000));
    }

    #[test]
    fn no_qd_election_keeps_qd_at_preferential_rate() {
        let r = compute(&base());
        assert_eq!(r.qualified_dividends_lost_preferential_rate, Decimal::ZERO);
        // $5k QD untouched — still gets the preferential rate.
    }

    #[test]
    fn multi_year_chain_carryforward_eventually_absorbs() {
        // Year 1: $10k expense, $6k NII → $4k carries.
        // Year 2: $0 new expense, $5k NII → $4k available, $4k deducted.
        let y1 = compute(&base());
        assert_eq!(y1.carryforward_to_next_year, dec!(4000));

        let mut y2_in = base();
        y2_in.tax_year = 2025;
        y2_in.investment_interest_expense = Decimal::ZERO;
        y2_in.prior_year_carryforward = y1.carryforward_to_next_year;
        y2_in.interest_income = dec!(5000);
        y2_in.ordinary_dividends = Decimal::ZERO;
        y2_in.net_short_term_capital_gain = Decimal::ZERO;
        let y2 = compute(&y2_in);
        assert_eq!(y2.deductible_this_year, dec!(4000));
        assert_eq!(y2.carryforward_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn note_reflects_election_tradeoff_when_applicable() {
        let mut i = base();
        i.elect_qualified_dividends_as_investment_income = true;
        i.investment_interest_expense = dec!(20000); // forces carryforward
        let r = compute(&i);
        assert!(r.note.contains("preferential rate") || r.note.contains("§1(h)"));
    }

    #[test]
    fn zero_income_zero_nii_full_carryforward() {
        let mut i = base();
        i.interest_income = Decimal::ZERO;
        i.ordinary_dividends = Decimal::ZERO;
        i.qualified_dividends = Decimal::ZERO;
        i.net_short_term_capital_gain = Decimal::ZERO;
        i.net_long_term_capital_gain = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.net_investment_income, Decimal::ZERO);
        assert_eq!(r.deductible_this_year, Decimal::ZERO);
        assert_eq!(r.carryforward_to_next_year, dec!(10000));
    }

    #[test]
    fn nii_equals_gross_when_no_other_expenses() {
        let r = compute(&base());
        assert_eq!(r.net_investment_income, r.gross_investment_income);
    }

    #[test]
    fn carryforward_never_negative_under_stress() {
        let mut i = base();
        i.investment_interest_expense = dec!(0);
        i.prior_year_carryforward = dec!(100);
        i.interest_income = dec!(1000);
        let r = compute(&i);
        assert_eq!(r.deductible_this_year, dec!(100));
        assert_eq!(r.carryforward_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn election_with_zero_amount_doesnt_increase_nii() {
        let mut i = base();
        i.qualified_dividends = Decimal::ZERO;
        i.elect_qualified_dividends_as_investment_income = true;
        let r = compute(&i);
        // Election asserted but no QD to elect — no change.
        assert_eq!(r.qualified_dividends_lost_preferential_rate, Decimal::ZERO);
    }
}
