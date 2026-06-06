//! IRC §280E — Expenditures in connection with the illegal sale of
//! drugs.
//!
//! Disallows any deduction or credit for amounts paid or incurred in
//! carrying on any trade or business that consists of trafficking in
//! controlled substances under Schedule I or II of the Controlled
//! Substances Act. Enacted by P.L. 97-248 § 351 (1982) in response
//! to *Edmondson v. Commissioner*. The defining tax restriction on
//! the entire U.S. state-legal cannabis industry.
//!
//! **Pre-2025 baseline**: marijuana classified as Schedule I →
//! §280E applies to state-licensed cannabis cultivators,
//! processors, distributors, and retailers regardless of state
//! legalization. Effective tax rates on state-legal operators
//! routinely exceed 70% because ordinary §162 expenses (rent,
//! wages, marketing) are disallowed.
//!
//! **2025-2026 rescheduling status** (verified post-cutoff):
//! - 2025-12-18: Executive Order 14370 signed by President Trump
//!   directing the U.S. Attorney General to complete DEA rulemaking
//!   moving marijuana from Schedule I to Schedule III "in the most
//!   expeditious manner permitted by law."
//! - As of 2026-02, the administrative process is ongoing and the
//!   final order has not yet taken effect.
//! - DOJ Final Order (partial): FDA-approved marijuana drug
//!   products + state-licensed medical marijuana + certain
//!   marijuana extracts / naturally derived delta-9-THC moved to
//!   Schedule III. Bulk marijuana, unlicensed crops, and any
//!   marijuana not incorporated into an FDA-approved drug product
//!   remain Schedule I.
//! - Effective date for tax: rescheduling applies first to the
//!   business's full taxable year that includes the effective date
//!   of the Final Order.
//!
//! **What §280E does NOT disallow**:
//! - **Cost of Goods Sold (COGS)** — COGS reduces gross income
//!   under §61, not a §162 deduction. *Californians Helping to
//!   Alleviate Med. Probs., Inc. v. Comm'r (Champ)* (T.C. 2007)
//!   established that §280E does not apply to COGS.
//! - **Bifurcated non-trafficking business expenses** — *Champ*
//!   established that when a business conducts both a trafficking
//!   activity AND a separate legal trade or business (e.g.,
//!   caregiving services bundled with medical marijuana), expenses
//!   allocable to the non-trafficking activity remain deductible.
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 280E](https://www.law.cornell.edu/uscode/text/26/280E),
//! [U.S. Treasury — Process for Tax Guidance Following DOJ Final Order on Medical Marijuana Rescheduling](https://home.treasury.gov/news/press-releases/sb0471),
//! [Current Federal Tax Developments — Executive Order 14370 and §280E sunset](https://www.currentfederaltaxdevelopments.com/blog/2025/12/19/tax-alert-executive-action-on-marijuana-scheduling-and-the-potential-sunset-of-irc-section-280e),
//! [CRS R46709 — Application of §280E to Marijuana Businesses](https://www.congress.gov/crs-product/R46709).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessActivityType {
    /// Traditional Schedule I/II trafficking — bulk marijuana, raw
    /// flower for adult-use markets, illegal drugs. §280E applies
    /// fully.
    TraditionalTrafficking,
    /// FDA-approved Schedule III marijuana drug product (e.g.,
    /// Epidiolex — pre-existed the broader rescheduling). §280E
    /// does not apply.
    FdaApprovedScheduleIII,
    /// State-licensed medical marijuana qualifying under the DOJ
    /// Final Order partial rescheduling to Schedule III. §280E
    /// applies until the Final Order's effective date is reached
    /// for the business's taxable year.
    StateLicensedMedicalRescheduled,
    /// Bifurcated non-trafficking activity within a cannabis-
    /// adjacent business under *Champ v. Comm'r* — caregiving,
    /// consulting, branded merchandise. §280E does not apply.
    NonTraffickingBifurcated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section280eInput {
    pub tax_year: i32,
    pub business_activity_type: BusinessActivityType,
    pub gross_revenue_dollars: i64,
    /// Cost of Goods Sold — always allowed as gross-income reducer
    /// regardless of §280E.
    pub cogs_dollars: i64,
    /// Ordinary §162 expenses (rent, wages, marketing, etc.)
    /// attributable to the trafficking activity.
    pub trafficking_business_expenses_dollars: i64,
    /// §162 expenses attributable to a separate non-trafficking
    /// activity under the *Champ* bifurcation doctrine. Always
    /// allowed.
    pub non_trafficking_bifurcated_expenses_dollars: i64,
    /// True if the DOJ Final Order rescheduling marijuana to
    /// Schedule III has been finalized AND its effective date falls
    /// within or before the taxpayer's current taxable year.
    pub doj_final_order_effective_for_year: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section280eResult {
    pub section_280e_applies: bool,
    pub trafficking_expenses_allowed_dollars: i64,
    pub trafficking_expenses_disallowed_dollars: i64,
    pub non_trafficking_expenses_allowed_dollars: i64,
    pub cogs_allowed_dollars: i64,
    pub gross_income_dollars: i64,
    pub total_allowed_deductions_dollars: i64,
    pub taxable_income_dollars: i64,
    /// Effective tax disadvantage — the §162 deductions that would
    /// have been allowed but for §280E. Useful for measuring the
    /// pre-rescheduling cost of the section.
    pub effective_tax_disadvantage_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section280eInput) -> Section280eResult {
    // Determine whether §280E applies based on activity type and
    // rescheduling status.
    let applies = match input.business_activity_type {
        BusinessActivityType::TraditionalTrafficking => true,
        BusinessActivityType::FdaApprovedScheduleIII => false,
        BusinessActivityType::StateLicensedMedicalRescheduled => {
            !input.doj_final_order_effective_for_year
        }
        BusinessActivityType::NonTraffickingBifurcated => false,
    };

    let trafficking_allowed = if applies {
        0
    } else {
        input.trafficking_business_expenses_dollars
    };
    let trafficking_disallowed = if applies {
        input.trafficking_business_expenses_dollars
    } else {
        0
    };

    // *Champ* bifurcation: non-trafficking activity expenses always
    // allowed.
    let non_trafficking_allowed = input.non_trafficking_bifurcated_expenses_dollars;

    // COGS always allowed — it reduces gross income, not a §162
    // deduction. §280E does not apply.
    let cogs_allowed = input.cogs_dollars;

    let gross_income = input.gross_revenue_dollars - cogs_allowed;
    let total_allowed = trafficking_allowed + non_trafficking_allowed;
    let taxable_income = gross_income - total_allowed;

    // Effective tax disadvantage: the §162 amount disallowed by §280E.
    let effective_disadvantage = trafficking_disallowed;

    let activity_label = match input.business_activity_type {
        BusinessActivityType::TraditionalTrafficking => "Traditional Schedule I/II trafficking",
        BusinessActivityType::FdaApprovedScheduleIII => "FDA-approved Schedule III drug product",
        BusinessActivityType::StateLicensedMedicalRescheduled => {
            "State-licensed medical marijuana (DOJ Final Order partial rescheduling)"
        }
        BusinessActivityType::NonTraffickingBifurcated => {
            "Non-trafficking bifurcated activity (Champ)"
        }
    };

    let rescheduling_status = if input.doj_final_order_effective_for_year {
        "DOJ Final Order in effect for this taxable year"
    } else {
        "DOJ Final Order not yet in effect for this taxable year"
    };

    let note = format!(
        "Tax year {}; {} ({}); gross revenue ${} − COGS ${} = gross income ${}; trafficking expenses ${} ({}); non-trafficking bifurcated ${} (always allowed under Champ); taxable income ${}; §280E effective tax disadvantage ${}. {}.",
        input.tax_year,
        activity_label,
        rescheduling_status,
        input.gross_revenue_dollars,
        cogs_allowed,
        gross_income,
        input.trafficking_business_expenses_dollars,
        if applies { "DISALLOWED by §280E" } else { "allowed" },
        non_trafficking_allowed,
        taxable_income,
        effective_disadvantage,
        if applies {
            "§280E applies — only COGS and bifurcated non-trafficking expenses reduce taxable income"
        } else {
            "§280E does NOT apply — full §162 deduction available"
        },
    );

    Section280eResult {
        section_280e_applies: applies,
        trafficking_expenses_allowed_dollars: trafficking_allowed,
        trafficking_expenses_disallowed_dollars: trafficking_disallowed,
        non_trafficking_expenses_allowed_dollars: non_trafficking_allowed,
        cogs_allowed_dollars: cogs_allowed,
        gross_income_dollars: gross_income,
        total_allowed_deductions_dollars: total_allowed,
        taxable_income_dollars: taxable_income,
        effective_tax_disadvantage_dollars: effective_disadvantage,
        citation:
            "IRC §280E Expenditures in connection with the illegal sale of drugs (P.L. 97-248 § 351 (1982), in response to Edmondson v. Comm'r); disallows §162 deductions for trafficking in Schedule I/II controlled substances regardless of state legalization. Does NOT disallow COGS (Champ T.C. 2007) or non-trafficking bifurcated activity expenses. EO 14370 (2025-12-18) directs DEA rescheduling Schedule I → Schedule III; DOJ Final Order partially reschedules FDA-approved + state-licensed medical marijuana but leaves bulk / unlicensed in Schedule I"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section280eInput {
        Section280eInput {
            tax_year: 2026,
            business_activity_type: BusinessActivityType::TraditionalTrafficking,
            gross_revenue_dollars: 1_000_000,
            cogs_dollars: 400_000,
            trafficking_business_expenses_dollars: 300_000,
            non_trafficking_bifurcated_expenses_dollars: 0,
            doj_final_order_effective_for_year: false,
        }
    }

    // ── §280E applies — traditional cannabis ───────────────────────

    #[test]
    fn traditional_schedule_i_ii_section_280e_applies() {
        let r = compute(&baseline());
        assert!(r.section_280e_applies);
        // Trafficking expenses fully disallowed.
        assert_eq!(r.trafficking_expenses_allowed_dollars, 0);
        assert_eq!(r.trafficking_expenses_disallowed_dollars, 300_000);
    }

    #[test]
    fn traditional_cannabis_taxable_income_is_gross_income_minus_cogs_only() {
        // Revenue $1M − COGS $400k = $600k gross income.
        // Trafficking expenses $300k DISALLOWED.
        // Taxable income = $600k.
        let r = compute(&baseline());
        assert_eq!(r.gross_income_dollars, 600_000);
        assert_eq!(r.total_allowed_deductions_dollars, 0);
        assert_eq!(r.taxable_income_dollars, 600_000);
    }

    #[test]
    fn traditional_cannabis_effective_disadvantage_equals_disallowed() {
        let r = compute(&baseline());
        assert_eq!(r.effective_tax_disadvantage_dollars, 300_000);
    }

    // ── COGS always allowed ────────────────────────────────────────

    #[test]
    fn cogs_always_reduces_gross_income_under_280e() {
        // COGS reduces gross income BEFORE §280E gates §162 expenses.
        let r = compute(&baseline());
        assert_eq!(
            r.cogs_allowed_dollars, 400_000,
            "COGS reduces gross income under §61, not a §162 deduction"
        );
        assert_eq!(r.gross_income_dollars, 600_000);
    }

    #[test]
    fn very_large_cogs_still_allowed() {
        // $10M revenue, $9M COGS → $1M gross income.
        let mut i = baseline();
        i.gross_revenue_dollars = 10_000_000;
        i.cogs_dollars = 9_000_000;
        let r = compute(&i);
        assert_eq!(r.cogs_allowed_dollars, 9_000_000);
        assert_eq!(r.gross_income_dollars, 1_000_000);
    }

    // ── Champ bifurcation ──────────────────────────────────────────

    #[test]
    fn champ_bifurcation_non_trafficking_expenses_always_allowed() {
        // Traditional cannabis $300k trafficking expenses
        // (disallowed); $50k non-trafficking expenses (allowed under
        // Champ).
        let mut i = baseline();
        i.non_trafficking_bifurcated_expenses_dollars = 50_000;
        let r = compute(&i);
        assert!(r.section_280e_applies);
        assert_eq!(r.trafficking_expenses_allowed_dollars, 0);
        assert_eq!(r.non_trafficking_expenses_allowed_dollars, 50_000);
        assert_eq!(r.taxable_income_dollars, 600_000 - 50_000);
    }

    #[test]
    fn pure_non_trafficking_business_no_section_280e() {
        // A separate trade or business that does not traffic in
        // controlled substances is fully outside §280E.
        let mut i = baseline();
        i.business_activity_type = BusinessActivityType::NonTraffickingBifurcated;
        i.trafficking_business_expenses_dollars = 0;
        i.non_trafficking_bifurcated_expenses_dollars = 200_000;
        let r = compute(&i);
        assert!(!r.section_280e_applies);
        assert_eq!(r.non_trafficking_expenses_allowed_dollars, 200_000);
    }

    // ── FDA-approved Schedule III ──────────────────────────────────

    #[test]
    fn fda_approved_schedule_iii_no_280e() {
        // FDA-approved drug products (e.g., Epidiolex, Marinol) are
        // Schedule III and outside §280E.
        let mut i = baseline();
        i.business_activity_type = BusinessActivityType::FdaApprovedScheduleIII;
        let r = compute(&i);
        assert!(!r.section_280e_applies);
        assert_eq!(r.trafficking_expenses_allowed_dollars, 300_000);
        // Taxable income = 600k gross − 300k expenses = 300k.
        assert_eq!(r.taxable_income_dollars, 300_000);
    }

    // ── DOJ Final Order partial rescheduling ───────────────────────

    #[test]
    fn state_licensed_medical_pre_final_order_still_280e() {
        // DOJ Final Order not yet in effect → §280E applies even
        // for state-licensed medical marijuana.
        let mut i = baseline();
        i.business_activity_type = BusinessActivityType::StateLicensedMedicalRescheduled;
        i.doj_final_order_effective_for_year = false;
        let r = compute(&i);
        assert!(r.section_280e_applies);
        assert_eq!(r.trafficking_expenses_allowed_dollars, 0);
    }

    #[test]
    fn state_licensed_medical_post_final_order_no_280e() {
        // DOJ Final Order in effect for this taxable year → §280E
        // no longer applies to state-licensed medical marijuana.
        let mut i = baseline();
        i.business_activity_type = BusinessActivityType::StateLicensedMedicalRescheduled;
        i.doj_final_order_effective_for_year = true;
        let r = compute(&i);
        assert!(!r.section_280e_applies);
        assert_eq!(r.trafficking_expenses_allowed_dollars, 300_000);
        assert_eq!(r.taxable_income_dollars, 300_000);
    }

    #[test]
    fn bulk_unlicensed_still_280e_after_partial_rescheduling() {
        // Bulk marijuana / unlicensed crops remain Schedule I even
        // post-Final-Order. §280E still applies.
        let mut i = baseline();
        i.business_activity_type = BusinessActivityType::TraditionalTrafficking;
        i.doj_final_order_effective_for_year = true;
        let r = compute(&i);
        assert!(r.section_280e_applies);
    }

    // ── Effective tax-disadvantage measurement ────────────────────

    #[test]
    fn no_280e_zero_effective_disadvantage() {
        let mut i = baseline();
        i.business_activity_type = BusinessActivityType::FdaApprovedScheduleIII;
        let r = compute(&i);
        assert_eq!(r.effective_tax_disadvantage_dollars, 0);
    }

    #[test]
    fn higher_trafficking_expenses_higher_disadvantage() {
        let mut i = baseline();
        i.trafficking_business_expenses_dollars = 500_000;
        let r = compute(&i);
        assert_eq!(r.effective_tax_disadvantage_dollars, 500_000);
    }

    // ── Zero / edge cases ──────────────────────────────────────────

    #[test]
    fn zero_revenue_zero_income() {
        let mut i = baseline();
        i.gross_revenue_dollars = 0;
        i.cogs_dollars = 0;
        i.trafficking_business_expenses_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.gross_income_dollars, 0);
        assert_eq!(r.taxable_income_dollars, 0);
    }

    #[test]
    fn zero_trafficking_expenses_no_disadvantage() {
        let mut i = baseline();
        i.trafficking_business_expenses_dollars = 0;
        let r = compute(&i);
        assert!(r.section_280e_applies);
        assert_eq!(r.effective_tax_disadvantage_dollars, 0);
    }

    // ── Activity-type combinations ─────────────────────────────────

    #[test]
    fn champ_bifurcation_with_fda_approved_full_deduction() {
        // FDA-approved + bifurcated non-trafficking — both allowed.
        let mut i = baseline();
        i.business_activity_type = BusinessActivityType::FdaApprovedScheduleIII;
        i.non_trafficking_bifurcated_expenses_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(r.trafficking_expenses_allowed_dollars, 300_000);
        assert_eq!(r.non_trafficking_expenses_allowed_dollars, 100_000);
        assert_eq!(r.total_allowed_deductions_dollars, 400_000);
        assert_eq!(r.taxable_income_dollars, 600_000 - 400_000);
    }

    // ── Citation ────────────────────────────────────────────────────

    #[test]
    fn citation_mentions_280e_origin() {
        let r = compute(&baseline());
        assert!(r.citation.contains("P.L. 97-248 § 351"));
        assert!(r.citation.contains("Edmondson"));
    }

    #[test]
    fn citation_mentions_champ_cogs_carveout() {
        let r = compute(&baseline());
        assert!(r.citation.contains("Champ"));
        assert!(r.citation.contains("COGS"));
    }

    #[test]
    fn citation_mentions_eo_14370_and_doj_final_order() {
        let r = compute(&baseline());
        assert!(r.citation.contains("EO 14370"));
        assert!(r.citation.contains("DOJ Final Order"));
    }

    #[test]
    fn citation_mentions_2025_12_18_executive_action() {
        let r = compute(&baseline());
        assert!(r.citation.contains("2025-12-18"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_traditional_says_280e_applies() {
        let r = compute(&baseline());
        assert!(r.note.contains("§280E applies"));
    }

    #[test]
    fn note_fda_says_full_162_deduction_available() {
        let mut i = baseline();
        i.business_activity_type = BusinessActivityType::FdaApprovedScheduleIII;
        let r = compute(&i);
        assert!(r.note.contains("§280E does NOT apply"));
    }

    #[test]
    fn note_rescheduling_status_mentioned() {
        let mut i = baseline();
        i.doj_final_order_effective_for_year = true;
        let r = compute(&i);
        assert!(r.note.contains("DOJ Final Order in effect"));
    }

    // ── Precision ──────────────────────────────────────────────────

    #[test]
    fn very_large_cannabis_revenue_no_precision_loss() {
        // $1B revenue, $400M COGS, $300M §162 expenses, §280E applies.
        let mut i = baseline();
        i.gross_revenue_dollars = 1_000_000_000;
        i.cogs_dollars = 400_000_000;
        i.trafficking_business_expenses_dollars = 300_000_000;
        let r = compute(&i);
        assert_eq!(r.gross_income_dollars, 600_000_000);
        assert_eq!(r.taxable_income_dollars, 600_000_000);
        assert_eq!(r.effective_tax_disadvantage_dollars, 300_000_000);
    }
}
