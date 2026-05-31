//! IRC §108 — Income from discharge of indebtedness.
//!
//! Critical for distressed debt traders, underwater real estate
//! investors, mortgage workouts, and credit card settlement scenarios.
//! Default rule under §61(a)(12) is that cancelled debt is gross income;
//! §108(a) provides narrow exclusions with mandatory attribute-reduction
//! consequences.
//!
//! **Five exclusions under §108(a)(1), in priority order:**
//!
//! 1. **§108(a)(1)(A) — Title 11 bankruptcy**: full exclusion when the
//!    debt is discharged in a Title 11 bankruptcy case. Highest
//!    priority — overrides ALL others.
//! 2. **§108(a)(1)(E) — Qualified principal residence indebtedness
//!    (QPRI)**: home-mortgage discharge under a written arrangement
//!    entered into before January 1, 2026. Takes priority over
//!    insolvency UNLESS the taxpayer elects insolvency under §108(a)(2)(C).
//! 3. **§108(a)(1)(B) — Insolvency**: excluded to the extent the
//!    taxpayer is insolvent immediately before the discharge. Insolvency
//!    = liabilities > FMV of assets per §108(d)(3).
//! 4. **§108(a)(1)(C) — Qualified farm indebtedness**: for qualified
//!    farmers (50%+ income from farming for the 3 prior years). Doesn't
//!    apply to the extent (B) insolvency already excluded the COD.
//! 5. **§108(a)(1)(D) — Qualified real property business indebtedness
//!    (QRPBI)**: for non-C-corp taxpayers with real-property business
//!    debt. Doesn't apply to the extent (B) insolvency already excluded.
//!    C-corporations cannot use (D).
//!
//! **§108(d)(3) Insolvency test**: insolvency amount = liabilities -
//! FMV of all assets, measured immediately before the discharge.
//! Negative result means solvent (no exclusion). Insolvency excludes
//! ONLY up to the insolvency amount — solvent taxpayers must include
//! the entire COD income.
//!
//! **§108(b) Attribute reduction**: when an exclusion applies, the
//! taxpayer must reduce tax attributes by the excluded amount. Reduces
//! NOL carryovers, GBC, minimum tax credit, capital loss carryovers,
//! basis of property, passive activity loss carryovers, FTC carryovers
//! (in that order). This module reports the required reduction total;
//! ordering and bucket allocation is a downstream Form 982 / detailed
//! attribute-by-attribute computation the caller performs.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section108Input {
    pub canceled_debt_amount: Decimal,
    /// True if the discharge occurs in a Title 11 bankruptcy case.
    pub debtor_in_bankruptcy_case: bool,
    /// FMV of ALL debtor's assets immediately before discharge.
    pub debtor_assets_fmv: Decimal,
    /// ALL debtor's liabilities immediately before discharge.
    pub debtor_liabilities: Decimal,
    /// True if the cancelled debt is qualified principal residence
    /// indebtedness (acquisition / improvement of principal residence).
    pub debt_is_qualified_principal_residence_indebtedness: bool,
    /// §108(a)(1)(E) requires the discharge to be subject to a written
    /// arrangement entered into before January 1, 2026. Discharges in
    /// 2026+ on pre-2026 arrangements still qualify; new arrangements
    /// in 2026+ do not.
    pub qpri_arrangement_entered_before_2026: bool,
    /// §108(a)(2)(C) election: taxpayer may elect to apply insolvency
    /// exclusion (B) instead of QPRI (E) — useful when attribute
    /// reduction under (B) is more favorable than basis reduction
    /// under (E).
    pub elect_insolvency_over_qpri: bool,
    /// True if cancelled debt is qualified farm indebtedness.
    pub debt_is_qualified_farm_indebtedness: bool,
    /// §108(g)(1)(B): 50%+ of income for the 3 taxable years preceding
    /// the discharge year must be from farming.
    pub debtor_is_qualified_farmer: bool,
    /// True if cancelled debt is qualified real property business
    /// indebtedness (acquisition/improvement of real property used in
    /// non-C-corp trade or business).
    pub debt_is_qualified_real_property_business_indebtedness: bool,
    /// C-corporations cannot use §108(a)(1)(D) exclusion.
    pub debtor_is_c_corporation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section108Rule {
    /// §108(a)(1)(A) — Title 11 bankruptcy. Full exclusion.
    BankruptcyFullExclusion,
    /// §108(a)(1)(E) — QPRI. Excluded by writing-before-2026 rule.
    QualifiedPrincipalResidenceExclusion,
    /// §108(a)(1)(B) — Insolvency. Excluded only up to insolvency amount.
    InsolvencyExclusion,
    /// §108(a)(1)(C) — Qualified farm.
    QualifiedFarmExclusion,
    /// §108(a)(1)(D) — Qualified real property business.
    QualifiedRealPropertyBusinessExclusion,
    /// No exclusion applies; full canceled debt is COD income.
    NoExclusionApplies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section108Result {
    pub gross_cod_income: Decimal,
    pub exclusion_amount: Decimal,
    pub includible_cod_income: Decimal,
    pub exclusion_path: Section108Rule,
    /// Insolvency amount = liabilities - assets FMV. Negative clamped to
    /// 0 (solvent).
    pub insolvency_amount: Decimal,
    /// §108(b) attribute reduction required = excluded amount (the
    /// "deferred tax" cost of taking the exclusion).
    pub attribute_reduction_required: Decimal,
    pub note: String,
}

pub fn compute(input: &Section108Input) -> Section108Result {
    let gross = input.canceled_debt_amount;

    // §108(d)(3) insolvency amount.
    let insolvency_amount =
        (input.debtor_liabilities - input.debtor_assets_fmv).max(Decimal::ZERO);

    // Priority 1: bankruptcy → full exclusion under §108(a)(1)(A).
    if input.debtor_in_bankruptcy_case {
        return Section108Result {
            gross_cod_income: gross,
            exclusion_amount: gross,
            includible_cod_income: Decimal::ZERO,
            exclusion_path: Section108Rule::BankruptcyFullExclusion,
            insolvency_amount,
            attribute_reduction_required: gross,
            note: format!(
                "§108(a)(1)(A) Title 11 bankruptcy — full ${} exclusion; mandatory §108(b) attribute reduction = ${}",
                gross.round_dp(2),
                gross.round_dp(2)
            ),
        };
    }

    // Priority 2: QPRI under §108(a)(1)(E) — UNLESS taxpayer elects
    // insolvency under §108(a)(2)(C).
    if input.debt_is_qualified_principal_residence_indebtedness
        && input.qpri_arrangement_entered_before_2026
        && !input.elect_insolvency_over_qpri
    {
        return Section108Result {
            gross_cod_income: gross,
            exclusion_amount: gross,
            includible_cod_income: Decimal::ZERO,
            exclusion_path: Section108Rule::QualifiedPrincipalResidenceExclusion,
            insolvency_amount,
            attribute_reduction_required: gross,
            note: format!(
                "§108(a)(1)(E) QPRI — full ${} exclusion (pre-2026 written arrangement); basis-of-residence reduction required",
                gross.round_dp(2)
            ),
        };
    }

    // Priority 3: Insolvency under §108(a)(1)(B). Excluded up to
    // insolvency amount.
    if insolvency_amount > Decimal::ZERO {
        let insol_exclusion = gross.min(insolvency_amount);
        let remaining_after_insolvency = gross - insol_exclusion;

        if remaining_after_insolvency == Decimal::ZERO {
            return Section108Result {
                gross_cod_income: gross,
                exclusion_amount: insol_exclusion,
                includible_cod_income: Decimal::ZERO,
                exclusion_path: Section108Rule::InsolvencyExclusion,
                insolvency_amount,
                attribute_reduction_required: insol_exclusion,
                note: format!(
                    "§108(a)(1)(B) insolvency — full ${} exclusion (insolvency amount ${} ≥ canceled debt); attribute reduction ${}",
                    gross.round_dp(2),
                    insolvency_amount.round_dp(2),
                    insol_exclusion.round_dp(2)
                ),
            };
        }

        // Insolvency exclusion partial. Check if farm or real-property-
        // business can further exclude the remainder.
        let (further_excl, further_rule) = further_exclusion_for_remainder(
            input,
            remaining_after_insolvency,
        );
        let total_excl = insol_exclusion + further_excl;
        let includible = (gross - total_excl).max(Decimal::ZERO);

        let path = if further_excl > Decimal::ZERO {
            further_rule
        } else {
            Section108Rule::InsolvencyExclusion
        };

        return Section108Result {
            gross_cod_income: gross,
            exclusion_amount: total_excl,
            includible_cod_income: includible,
            exclusion_path: path,
            insolvency_amount,
            attribute_reduction_required: total_excl,
            note: format!(
                "§108(a)(1)(B) insolvency partial — ${} excluded by insolvency, ${} further excluded; ${} includible COD income",
                insol_exclusion.round_dp(2),
                further_excl.round_dp(2),
                includible.round_dp(2),
            ),
        };
    }

    // Priority 4 / 5: Farm or real-property-business indebtedness
    // (when not insolvent and no QPRI/bankruptcy).
    let (further_excl, further_rule) = further_exclusion_for_remainder(input, gross);
    if further_excl > Decimal::ZERO {
        return Section108Result {
            gross_cod_income: gross,
            exclusion_amount: further_excl,
            includible_cod_income: gross - further_excl,
            exclusion_path: further_rule,
            insolvency_amount,
            attribute_reduction_required: further_excl,
            note: format!(
                "${} excluded under qualified-debt category; ${} includible COD income",
                further_excl.round_dp(2),
                (gross - further_excl).round_dp(2)
            ),
        };
    }

    // No exclusion applies — full COD income under §61(a)(12).
    Section108Result {
        gross_cod_income: gross,
        exclusion_amount: Decimal::ZERO,
        includible_cod_income: gross,
        exclusion_path: Section108Rule::NoExclusionApplies,
        insolvency_amount,
        attribute_reduction_required: Decimal::ZERO,
        note: format!(
            "no §108 exclusion applies — full ${} included in gross income under §61(a)(12)",
            gross.round_dp(2)
        ),
    }
}

/// Returns (excluded amount, rule path) when one of the qualified-debt
/// categories applies; otherwise returns (0, NoExclusionApplies).
fn further_exclusion_for_remainder(
    input: &Section108Input,
    remainder: Decimal,
) -> (Decimal, Section108Rule) {
    if input.debt_is_qualified_farm_indebtedness && input.debtor_is_qualified_farmer {
        return (remainder, Section108Rule::QualifiedFarmExclusion);
    }
    if input.debt_is_qualified_real_property_business_indebtedness
        && !input.debtor_is_c_corporation
    {
        return (
            remainder,
            Section108Rule::QualifiedRealPropertyBusinessExclusion,
        );
    }
    (Decimal::ZERO, Section108Rule::NoExclusionApplies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section108Input {
        Section108Input {
            canceled_debt_amount: dec!(50_000),
            debtor_in_bankruptcy_case: false,
            debtor_assets_fmv: dec!(80_000),
            debtor_liabilities: dec!(70_000),
            debt_is_qualified_principal_residence_indebtedness: false,
            qpri_arrangement_entered_before_2026: false,
            elect_insolvency_over_qpri: false,
            debt_is_qualified_farm_indebtedness: false,
            debtor_is_qualified_farmer: false,
            debt_is_qualified_real_property_business_indebtedness: false,
            debtor_is_c_corporation: false,
        }
    }

    #[test]
    fn no_exclusion_solvent_taxpayer_full_inclusion() {
        // Solvent (assets $80k > liab $70k) → no exclusion. Full COD
        // income.
        let r = compute(&base());
        assert_eq!(r.exclusion_path, Section108Rule::NoExclusionApplies);
        assert_eq!(r.includible_cod_income, dec!(50_000));
        assert_eq!(r.exclusion_amount, Decimal::ZERO);
        assert_eq!(r.insolvency_amount, Decimal::ZERO);
    }

    #[test]
    fn bankruptcy_full_exclusion_highest_priority() {
        // Title 11 bankruptcy → full exclusion regardless of other facts.
        let mut i = base();
        i.debtor_in_bankruptcy_case = true;
        let r = compute(&i);
        assert_eq!(r.exclusion_path, Section108Rule::BankruptcyFullExclusion);
        assert_eq!(r.exclusion_amount, dec!(50_000));
        assert_eq!(r.includible_cod_income, Decimal::ZERO);
        assert_eq!(r.attribute_reduction_required, dec!(50_000));
    }

    #[test]
    fn insolvency_full_exclusion_when_amount_ge_debt() {
        // Liabilities $200k, assets $100k → insolvent by $100k.
        // Canceled debt $50k ≤ insolvency → full exclusion.
        let mut i = base();
        i.debtor_assets_fmv = dec!(100_000);
        i.debtor_liabilities = dec!(200_000);
        let r = compute(&i);
        assert_eq!(r.exclusion_path, Section108Rule::InsolvencyExclusion);
        assert_eq!(r.exclusion_amount, dec!(50_000));
        assert_eq!(r.includible_cod_income, Decimal::ZERO);
        assert_eq!(r.insolvency_amount, dec!(100_000));
    }

    #[test]
    fn insolvency_partial_exclusion_with_remainder_includible() {
        // Liabilities $130k, assets $100k → insolvent by $30k. Canceled
        // debt $50k → exclude $30k, include $20k.
        let mut i = base();
        i.debtor_assets_fmv = dec!(100_000);
        i.debtor_liabilities = dec!(130_000);
        let r = compute(&i);
        assert_eq!(r.exclusion_amount, dec!(30_000));
        assert_eq!(r.includible_cod_income, dec!(20_000));
        assert_eq!(r.insolvency_amount, dec!(30_000));
    }

    #[test]
    fn qpri_pre_2026_exclusion_overrides_insolvency() {
        // QPRI with pre-2026 arrangement → full exclusion under
        // §108(a)(1)(E), takes priority over insolvency (B) by default.
        let mut i = base();
        i.debt_is_qualified_principal_residence_indebtedness = true;
        i.qpri_arrangement_entered_before_2026 = true;
        i.debtor_assets_fmv = dec!(100_000);
        i.debtor_liabilities = dec!(200_000); // also insolvent
        let r = compute(&i);
        assert_eq!(
            r.exclusion_path,
            Section108Rule::QualifiedPrincipalResidenceExclusion
        );
        assert_eq!(r.exclusion_amount, dec!(50_000));
    }

    #[test]
    fn qpri_post_2026_arrangement_does_not_exclude() {
        // QPRI flag set but arrangement entered in 2026+ → exclusion
        // doesn't apply. Falls through to insolvency / other paths.
        let mut i = base();
        i.debt_is_qualified_principal_residence_indebtedness = true;
        i.qpri_arrangement_entered_before_2026 = false;
        let r = compute(&i);
        assert_ne!(
            r.exclusion_path,
            Section108Rule::QualifiedPrincipalResidenceExclusion
        );
        // Solvent → NoExclusionApplies.
        assert_eq!(r.exclusion_path, Section108Rule::NoExclusionApplies);
    }

    #[test]
    fn qpri_election_to_use_insolvency_routes_through_insolvency() {
        // §108(a)(2)(C) election — taxpayer elects insolvency instead
        // of QPRI. Routes through insolvency path.
        let mut i = base();
        i.debt_is_qualified_principal_residence_indebtedness = true;
        i.qpri_arrangement_entered_before_2026 = true;
        i.elect_insolvency_over_qpri = true;
        i.debtor_assets_fmv = dec!(100_000);
        i.debtor_liabilities = dec!(200_000);
        let r = compute(&i);
        assert_eq!(r.exclusion_path, Section108Rule::InsolvencyExclusion);
    }

    #[test]
    fn bankruptcy_overrides_qpri() {
        // Bankruptcy has highest priority. QPRI flags don't matter.
        let mut i = base();
        i.debtor_in_bankruptcy_case = true;
        i.debt_is_qualified_principal_residence_indebtedness = true;
        i.qpri_arrangement_entered_before_2026 = true;
        let r = compute(&i);
        assert_eq!(r.exclusion_path, Section108Rule::BankruptcyFullExclusion);
    }

    #[test]
    fn qualified_farm_indebtedness_solvent_excludes() {
        // Solvent farmer with qualified farm debt → §108(a)(1)(C).
        let mut i = base();
        i.debt_is_qualified_farm_indebtedness = true;
        i.debtor_is_qualified_farmer = true;
        let r = compute(&i);
        assert_eq!(r.exclusion_path, Section108Rule::QualifiedFarmExclusion);
        assert_eq!(r.exclusion_amount, dec!(50_000));
    }

    #[test]
    fn farm_debt_without_qualified_farmer_status_no_exclusion() {
        // Has qualified farm debt but doesn't meet 50%-farming-income
        // test → no farm exclusion.
        let mut i = base();
        i.debt_is_qualified_farm_indebtedness = true;
        i.debtor_is_qualified_farmer = false;
        let r = compute(&i);
        assert_eq!(r.exclusion_path, Section108Rule::NoExclusionApplies);
    }

    #[test]
    fn qrpbi_for_non_c_corp_excludes() {
        // Qualified real property business indebtedness for non-C-corp.
        let mut i = base();
        i.debt_is_qualified_real_property_business_indebtedness = true;
        i.debtor_is_c_corporation = false;
        let r = compute(&i);
        assert_eq!(
            r.exclusion_path,
            Section108Rule::QualifiedRealPropertyBusinessExclusion
        );
    }

    #[test]
    fn c_corp_cannot_use_qrpbi_exclusion() {
        // C-corp with QRPBI flag → no exclusion (statutorily blocked).
        let mut i = base();
        i.debt_is_qualified_real_property_business_indebtedness = true;
        i.debtor_is_c_corporation = true;
        let r = compute(&i);
        assert_eq!(r.exclusion_path, Section108Rule::NoExclusionApplies);
    }

    #[test]
    fn insolvency_partial_plus_farm_excludes_remainder() {
        // Insolvent by $30k, canceled $50k, qualified farm + farmer.
        // Insolvency excludes $30k; farm excludes remaining $20k.
        let mut i = base();
        i.debtor_assets_fmv = dec!(100_000);
        i.debtor_liabilities = dec!(130_000);
        i.debt_is_qualified_farm_indebtedness = true;
        i.debtor_is_qualified_farmer = true;
        let r = compute(&i);
        assert_eq!(r.exclusion_amount, dec!(50_000));
        assert_eq!(r.includible_cod_income, Decimal::ZERO);
        assert_eq!(r.exclusion_path, Section108Rule::QualifiedFarmExclusion);
    }

    #[test]
    fn insolvency_partial_plus_qrpbi_excludes_remainder() {
        // Same setup with QRPBI instead of farm.
        let mut i = base();
        i.debtor_assets_fmv = dec!(100_000);
        i.debtor_liabilities = dec!(130_000);
        i.debt_is_qualified_real_property_business_indebtedness = true;
        let r = compute(&i);
        assert_eq!(r.exclusion_amount, dec!(50_000));
        assert_eq!(r.includible_cod_income, Decimal::ZERO);
    }

    #[test]
    fn attribute_reduction_equals_excluded_amount() {
        // §108(b) attribute reduction = excluded amount, always.
        let mut i = base();
        i.debtor_assets_fmv = dec!(100_000);
        i.debtor_liabilities = dec!(130_000);
        let r = compute(&i);
        assert_eq!(r.attribute_reduction_required, r.exclusion_amount);
    }

    #[test]
    fn insolvency_amount_clamps_to_zero_when_solvent() {
        // Solvent → insolvency_amount = 0 even though liabilities > 0.
        let r = compute(&base());
        assert_eq!(r.insolvency_amount, Decimal::ZERO);
    }

    #[test]
    fn insolvency_amount_at_exact_zero_solvent() {
        // Liabilities exactly equal assets → solvent (not insolvent).
        let mut i = base();
        i.debtor_assets_fmv = dec!(100_000);
        i.debtor_liabilities = dec!(100_000);
        let r = compute(&i);
        assert_eq!(r.insolvency_amount, Decimal::ZERO);
        assert_eq!(r.exclusion_path, Section108Rule::NoExclusionApplies);
    }

    #[test]
    fn priority_bankruptcy_over_insolvency() {
        // Both flags set → bankruptcy wins.
        let mut i = base();
        i.debtor_in_bankruptcy_case = true;
        i.debtor_assets_fmv = dec!(100_000);
        i.debtor_liabilities = dec!(500_000);
        let r = compute(&i);
        assert_eq!(r.exclusion_path, Section108Rule::BankruptcyFullExclusion);
        // attribute reduction = full canceled debt under (A).
        assert_eq!(r.attribute_reduction_required, dec!(50_000));
    }

    #[test]
    fn priority_qpri_over_insolvency_without_election() {
        // QPRI + insolvent + no election → QPRI wins (priority 2 over 3).
        let mut i = base();
        i.debt_is_qualified_principal_residence_indebtedness = true;
        i.qpri_arrangement_entered_before_2026 = true;
        i.debtor_assets_fmv = dec!(100_000);
        i.debtor_liabilities = dec!(200_000);
        let r = compute(&i);
        assert_eq!(
            r.exclusion_path,
            Section108Rule::QualifiedPrincipalResidenceExclusion
        );
    }

    #[test]
    fn zero_canceled_debt_no_op() {
        let mut i = base();
        i.canceled_debt_amount = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.includible_cod_income, Decimal::ZERO);
        assert_eq!(r.exclusion_amount, Decimal::ZERO);
    }

    #[test]
    fn very_large_canceled_debt_no_precision_loss() {
        // $10B distressed debt write-off in bankruptcy.
        let mut i = base();
        i.canceled_debt_amount = dec!(10_000_000_000);
        i.debtor_in_bankruptcy_case = true;
        let r = compute(&i);
        assert_eq!(r.exclusion_amount, dec!(10_000_000_000));
        assert_eq!(r.includible_cod_income, Decimal::ZERO);
    }

    #[test]
    fn note_describes_section_per_path() {
        let bankruptcy = {
            let mut i = base();
            i.debtor_in_bankruptcy_case = true;
            compute(&i)
        };
        assert!(bankruptcy.note.contains("§108(a)(1)(A)"));

        let qpri = {
            let mut i = base();
            i.debt_is_qualified_principal_residence_indebtedness = true;
            i.qpri_arrangement_entered_before_2026 = true;
            compute(&i)
        };
        assert!(qpri.note.contains("§108(a)(1)(E)"));

        let insolvency = {
            let mut i = base();
            i.debtor_assets_fmv = dec!(100_000);
            i.debtor_liabilities = dec!(200_000);
            compute(&i)
        };
        assert!(insolvency.note.contains("§108(a)(1)(B)"));

        let no_excl = compute(&base());
        assert!(no_excl.note.contains("§61(a)(12)"));
    }
}
