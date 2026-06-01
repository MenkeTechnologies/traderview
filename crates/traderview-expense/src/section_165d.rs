//! IRC §165(d) — Wagering losses.
//!
//! Trader-relevant when derivative positions or prediction-market
//! activities are characterized as wagering rather than capital-
//! asset trading. Pre-OBBBA: deduction limited to losses up to the
//! amount of winnings (100% of losses, capped at winnings).
//! Post-OBBBA: deduction further restricted to 90% of losses,
//! still capped at winnings. Creates the "phantom income" problem
//! where a gambler who breaks even economically can still owe tax.
//!
//! **Pre-2026 baseline (TCJA + earlier)**: §165(d) allowed gambling
//! losses to be deducted as an itemized deduction up to the amount
//! of gambling winnings reported as income. Effective rate of
//! deduction = 100% of losses, capped at winnings.
//!
//! **Post-2025 OBBBA (eff. 2026-01-01)**: P.L. 119-21 (One Big
//! Beautiful Bill Act, signed 2025-07-04) amended §165(d) to limit
//! deductible losses to **90% of losses incurred** AND capped at
//! winnings. Both amateur and professional gamblers affected.
//!
//! Example (winnings $3,000; losses $4,000):
//! - Pre-OBBBA: deductible = min($4,000, $3,000) = $3,000
//! - Post-OBBBA: 90% × $4,000 = $3,600; deductible = min($3,600,
//!   $3,000) = $3,000 (winnings cap still binds)
//!
//! Example showing phantom income (winnings $3,000; losses $3,000):
//! - Pre-OBBBA: deductible $3,000; taxable wagering income $0
//! - Post-OBBBA: 90% × $3,000 = $2,700; deductible $2,700; taxable
//!   wagering income = $3,000 − $2,700 = **$300 phantom income**
//!   despite breaking even economically.
//!
//! **Professional gambler additional deductions**: §162 trade-or-
//! business expenses (travel, entry fees, supplies, etc.) remain
//! deductible separately. OBBBA preserved this carve-out.
//!
//! **Itemized deduction requirement**: §165(d) losses are itemized
//! Schedule A deductions; only taxpayers who itemize benefit. With
//! TCJA's expanded standard deduction made permanent by OBBBA,
//! only ~14% of taxpayers itemize in 2026.
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 165](https://www.law.cornell.edu/uscode/text/26/165),
//! [Tax Foundation — OBBBA unequal gambling-loss treatment](https://taxfoundation.org/blog/gambling-losses-tax-big-beautiful-bill/),
//! [Blackburn Childers & Steagall — Gambling Loss Deductions Under OBBBA](https://www.bcscpa.com/understanding-gambling-loss-deductions-under-the-one-big-beautiful-bill-act/),
//! [Greenleaf Trust — OBBBA Impact on Gambling](https://greenleaftrust.com/missives/obbba-and-gambling-losses-surprise/),
//! [Barnes Dennig — OBBBA Major Changes to Gambling Loss Deductions](https://www.barnesdennig.com/obbba-major-changes-to-gambling-loss-deductions/).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section165dInput {
    pub tax_year: i32,
    pub gambling_winnings_dollars: i64,
    pub gambling_losses_dollars: i64,
    /// True if the taxpayer is a professional gambler treating
    /// activity as a trade or business under §162.
    pub taxpayer_is_professional_gambler: bool,
    /// Additional §162 trade-or-business expenses incurred by a
    /// professional gambler (travel, supplies, entry fees, etc.).
    /// Zero for amateurs.
    pub professional_gambler_business_expenses_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section165dResult {
    pub post_obbba: bool,
    /// Effective loss-deduction percentage in basis points
    /// (10000 = 100% pre-OBBBA; 9000 = 90% post-OBBBA).
    pub loss_deduction_pct_bp: u32,
    /// Adjusted losses after percentage limitation, before
    /// winnings-cap.
    pub adjusted_losses_after_percentage_dollars: i64,
    /// Final §165(d) deduction = min(adjusted_losses, winnings).
    pub gambling_loss_deduction_dollars: i64,
    /// Additional §162 business-expense deduction for professional
    /// gamblers.
    pub professional_gambler_expense_deduction_dollars: i64,
    /// Total deductible amount = §165(d) + §162 if professional.
    pub total_deduction_dollars: i64,
    /// Phantom-income amount = winnings − §165(d) deduction. Equals
    /// zero pre-OBBBA when losses ≥ winnings.
    pub phantom_taxable_income_dollars: i64,
    /// Net taxable wagering income = winnings − total deduction.
    pub net_taxable_wagering_income_dollars: i64,
    pub citation: String,
    pub note: String,
}

const OBBBA_EFFECTIVE_YEAR: i32 = 2026;
const POST_OBBBA_LOSS_DEDUCTION_BP: u32 = 9000; // 90%
const PRE_OBBBA_LOSS_DEDUCTION_BP: u32 = 10_000; // 100%

pub fn compute(input: &Section165dInput) -> Section165dResult {
    let post_obbba = input.tax_year >= OBBBA_EFFECTIVE_YEAR;
    let loss_pct_bp = if post_obbba {
        POST_OBBBA_LOSS_DEDUCTION_BP
    } else {
        PRE_OBBBA_LOSS_DEDUCTION_BP
    };

    let losses = input.gambling_losses_dollars.max(0);
    let winnings = input.gambling_winnings_dollars.max(0);

    // Apply percentage limitation.
    let adjusted_losses =
        ((losses as i128) * (loss_pct_bp as i128) / 10_000) as i64;

    // §165(d) winnings cap.
    let loss_deduction = adjusted_losses.min(winnings);

    // §162 professional gambler expenses.
    let professional_expenses = if input.taxpayer_is_professional_gambler {
        input.professional_gambler_business_expenses_dollars.max(0)
    } else {
        0
    };

    let total_deduction = loss_deduction + professional_expenses;

    // Phantom income: winnings minus the §165(d) deduction (positive
    // only when 90% of losses < winnings ≤ losses).
    let phantom_income = (winnings - loss_deduction).max(0);

    // Net taxable wagering income — can go negative for professional
    // gamblers with significant §162 expenses.
    let net_taxable = winnings - total_deduction;

    let regime_label = if post_obbba {
        "post-OBBBA 2025 90% loss limitation"
    } else {
        "pre-OBBBA 100% loss-up-to-winnings limitation"
    };

    let note = format!(
        "Tax year {}; {}; winnings ${}; losses ${}; loss deduction percentage {}.{}%; adjusted losses ${}; §165(d) deduction (capped at winnings) ${}; {} §162 expenses ${}; total deduction ${}; phantom taxable income ${}; net taxable wagering income ${}.",
        input.tax_year,
        regime_label,
        winnings,
        losses,
        loss_pct_bp / 100,
        loss_pct_bp % 100,
        adjusted_losses,
        loss_deduction,
        if input.taxpayer_is_professional_gambler { "professional" } else { "amateur (no §162)" },
        professional_expenses,
        total_deduction,
        phantom_income,
        net_taxable,
    );

    Section165dResult {
        post_obbba,
        loss_deduction_pct_bp: loss_pct_bp,
        adjusted_losses_after_percentage_dollars: adjusted_losses,
        gambling_loss_deduction_dollars: loss_deduction,
        professional_gambler_expense_deduction_dollars: professional_expenses,
        total_deduction_dollars: total_deduction,
        phantom_taxable_income_dollars: phantom_income,
        net_taxable_wagering_income_dollars: net_taxable,
        citation:
            "IRC §165(d) wagering loss deduction — historically 100% of losses up to winnings; One Big Beautiful Bill Act (P.L. 119-21, signed 2025-07-04) effective tax years beginning after 2025-12-31 limits to 90% of losses + still capped at winnings; both amateur and professional gamblers affected; §162 trade-or-business expense carve-out preserved for professional gamblers; itemized Schedule A deduction (Schedule C for professional); phantom-income problem when 90% × losses < winnings ≤ losses"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section165dInput {
        Section165dInput {
            tax_year: 2026,
            gambling_winnings_dollars: 3_000,
            gambling_losses_dollars: 4_000,
            taxpayer_is_professional_gambler: false,
            professional_gambler_business_expenses_dollars: 0,
        }
    }

    // ── Pre-OBBBA 100% baseline ────────────────────────────────────

    #[test]
    fn pre_obbba_2025_loss_full_100_pct_capped_at_winnings() {
        // Winnings $3k, losses $4k → deduction = min($4k, $3k) = $3k.
        let mut i = base();
        i.tax_year = 2025;
        let r = compute(&i);
        assert!(!r.post_obbba);
        assert_eq!(r.loss_deduction_pct_bp, 10_000);
        assert_eq!(r.adjusted_losses_after_percentage_dollars, 4_000);
        assert_eq!(r.gambling_loss_deduction_dollars, 3_000);
        assert_eq!(r.phantom_taxable_income_dollars, 0);
        assert_eq!(r.net_taxable_wagering_income_dollars, 0);
    }

    #[test]
    fn pre_obbba_losses_below_winnings_full_loss_deductible() {
        let mut i = base();
        i.tax_year = 2025;
        i.gambling_winnings_dollars = 10_000;
        i.gambling_losses_dollars = 4_000;
        let r = compute(&i);
        assert_eq!(r.gambling_loss_deduction_dollars, 4_000);
        assert_eq!(r.net_taxable_wagering_income_dollars, 6_000);
    }

    // ── Post-OBBBA 90% limitation ──────────────────────────────────

    #[test]
    fn post_obbba_2026_loss_limited_to_90_pct() {
        let r = compute(&base());
        assert!(r.post_obbba);
        assert_eq!(r.loss_deduction_pct_bp, 9000);
        // 90% × $4k = $3,600; still capped at $3k winnings.
        assert_eq!(r.adjusted_losses_after_percentage_dollars, 3_600);
        assert_eq!(r.gambling_loss_deduction_dollars, 3_000);
    }

    #[test]
    fn post_obbba_break_even_creates_phantom_income() {
        // Winnings $3k, losses $3k. Pre-OBBBA: deduction $3k → net 0.
        // Post-OBBBA: 90% × $3k = $2,700; deduction $2,700; phantom
        // income $300 despite breaking even economically.
        let mut i = base();
        i.tax_year = 2026;
        i.gambling_winnings_dollars = 3_000;
        i.gambling_losses_dollars = 3_000;
        let r = compute(&i);
        assert_eq!(r.gambling_loss_deduction_dollars, 2_700);
        assert_eq!(r.phantom_taxable_income_dollars, 300);
        assert_eq!(r.net_taxable_wagering_income_dollars, 300);
    }

    #[test]
    fn post_obbba_losses_well_below_winnings_full_90_pct_deductible() {
        // Winnings $100k, losses $10k → 90% × $10k = $9k.
        let mut i = base();
        i.tax_year = 2026;
        i.gambling_winnings_dollars = 100_000;
        i.gambling_losses_dollars = 10_000;
        let r = compute(&i);
        assert_eq!(r.gambling_loss_deduction_dollars, 9_000);
        // Even though losses are below winnings, the 90% still
        // applies — only 90% of $10k is deductible.
        assert_eq!(r.net_taxable_wagering_income_dollars, 91_000);
    }

    // ── Year boundary ──────────────────────────────────────────────

    #[test]
    fn year_2025_pre_obbba() {
        let mut i = base();
        i.tax_year = 2025;
        let r = compute(&i);
        assert!(!r.post_obbba);
        assert_eq!(r.loss_deduction_pct_bp, 10_000);
    }

    #[test]
    fn year_2026_post_obbba() {
        let r = compute(&base());
        assert!(r.post_obbba);
        assert_eq!(r.loss_deduction_pct_bp, 9000);
    }

    #[test]
    fn future_year_2030_post_obbba() {
        let mut i = base();
        i.tax_year = 2030;
        let r = compute(&i);
        assert!(r.post_obbba);
    }

    // ── Professional gambler §162 expenses ─────────────────────────

    #[test]
    fn professional_gambler_additional_162_expenses_deductible() {
        let mut i = base();
        i.tax_year = 2026;
        i.gambling_winnings_dollars = 100_000;
        i.gambling_losses_dollars = 50_000;
        i.taxpayer_is_professional_gambler = true;
        i.professional_gambler_business_expenses_dollars = 20_000;
        let r = compute(&i);
        // §165(d) = 90% × 50k = $45k (within winnings cap).
        // §162 = $20k.
        // Total = $65k.
        assert_eq!(r.gambling_loss_deduction_dollars, 45_000);
        assert_eq!(r.professional_gambler_expense_deduction_dollars, 20_000);
        assert_eq!(r.total_deduction_dollars, 65_000);
        assert_eq!(r.net_taxable_wagering_income_dollars, 35_000);
    }

    #[test]
    fn amateur_no_162_expenses_even_if_input_provided() {
        // Module zeroes out §162 expenses for amateurs.
        let mut i = base();
        i.taxpayer_is_professional_gambler = false;
        i.professional_gambler_business_expenses_dollars = 5_000;
        let r = compute(&i);
        assert_eq!(r.professional_gambler_expense_deduction_dollars, 0);
    }

    #[test]
    fn professional_gambler_with_162_can_create_net_loss() {
        // High §162 expenses can drive net taxable below zero.
        let mut i = base();
        i.tax_year = 2026;
        i.gambling_winnings_dollars = 10_000;
        i.gambling_losses_dollars = 5_000;
        i.taxpayer_is_professional_gambler = true;
        i.professional_gambler_business_expenses_dollars = 8_000;
        let r = compute(&i);
        // §165(d) = 90% × $5k = $4,500; §162 = $8k; total $12,500.
        // Winnings $10k − $12,500 = −$2,500.
        assert_eq!(r.net_taxable_wagering_income_dollars, -2_500);
    }

    // ── Defensive ──────────────────────────────────────────────────

    #[test]
    fn zero_winnings_zero_losses_no_op() {
        let mut i = base();
        i.gambling_winnings_dollars = 0;
        i.gambling_losses_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.gambling_loss_deduction_dollars, 0);
        assert_eq!(r.phantom_taxable_income_dollars, 0);
        assert_eq!(r.net_taxable_wagering_income_dollars, 0);
    }

    #[test]
    fn negative_inputs_clamped_to_zero() {
        let mut i = base();
        i.gambling_winnings_dollars = -1_000;
        i.gambling_losses_dollars = -2_000;
        let r = compute(&i);
        assert_eq!(r.gambling_loss_deduction_dollars, 0);
        assert_eq!(r.net_taxable_wagering_income_dollars, 0);
    }

    #[test]
    fn very_large_winnings_no_precision_loss() {
        let mut i = base();
        i.gambling_winnings_dollars = 1_000_000_000;
        i.gambling_losses_dollars = 800_000_000;
        let r = compute(&i);
        // 90% × $800M = $720M.
        assert_eq!(r.gambling_loss_deduction_dollars, 720_000_000);
        assert_eq!(r.net_taxable_wagering_income_dollars, 280_000_000);
    }

    // ── Citation ───────────────────────────────────────────────────

    #[test]
    fn citation_mentions_165_d_and_obbba_2025_07_04() {
        let r = compute(&base());
        assert!(r.citation.contains("§165(d)"));
        assert!(r.citation.contains("One Big Beautiful Bill"));
        assert!(r.citation.contains("P.L. 119-21"));
        assert!(r.citation.contains("2025-07-04"));
    }

    #[test]
    fn citation_mentions_90_pct_and_phantom_income() {
        let r = compute(&base());
        assert!(r.citation.contains("90% of losses"));
        assert!(r.citation.contains("phantom-income"));
    }

    #[test]
    fn citation_mentions_162_carveout_for_professionals() {
        let r = compute(&base());
        assert!(r.citation.contains("§162 trade-or-business"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn pre_obbba_note_describes_100_pct_regime() {
        let mut i = base();
        i.tax_year = 2025;
        let r = compute(&i);
        assert!(r.note.contains("pre-OBBBA 100% loss-up-to-winnings"));
    }

    #[test]
    fn post_obbba_note_describes_90_pct_regime() {
        let r = compute(&base());
        assert!(r.note.contains("post-OBBBA 2025 90% loss limitation"));
    }

    #[test]
    fn note_professional_path_describes_162() {
        let mut i = base();
        i.taxpayer_is_professional_gambler = true;
        let r = compute(&i);
        assert!(r.note.contains("professional"));
    }

    #[test]
    fn note_amateur_path_says_no_162() {
        let r = compute(&base());
        assert!(r.note.contains("amateur (no §162)"));
    }
}
