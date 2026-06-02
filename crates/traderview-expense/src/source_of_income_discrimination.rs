//! Source-of-income (SOI) discrimination ban — when may a
//! landlord lawfully refuse to accept Housing Choice Voucher
//! (Section 8), HUD-VASH, SSI/SSDI, child support, alimony,
//! TANF, or other lawful non-wage income as the basis for
//! rejecting an applicant? Trader-landlord critical: refusing
//! a Section 8 voucher in a SOI-protected jurisdiction can
//! trigger five-figure penalties + actual damages + attorney
//! fees, while in non-protected jurisdictions the federal
//! Fair Housing Act does NOT make voucher refusal a per se
//! violation. Distinct from `fair_chance_housing` (criminal
//! background screening) and `tenant_in_foreclosure_
//! protection` (post-foreclosure tenant protections).
//!
//! **Four regimes**:
//!
//! **California — SB 329 (Housing Opportunities Act of 2019,
//! eff. 2020) + Cal. Gov. Code § 12955 (FEHA)**. Section 8
//! Housing Choice Vouchers explicitly added to "source of
//! income" definition. Landlord refusal solely because of
//! Section 8 voucher participation triggers FEHA liability:
//! actual damages + emotional distress + civil penalty
//! (Cal. Gov. Code § 12989.2 + § 12987) + attorney fees +
//! injunctive relief. Small-landlord (single-family-dwelling
//! owner-occupied with ≤4 units) carve-out available but
//! narrow.
//!
//! **New Jersey — NJ Law Against Discrimination (LAD),
//! N.J.S.A. 10:5-12.5 + Division on Civil Rights enforcement**.
//! Source-of-income covers Section 8 vouchers, public
//! assistance. Penalties from $1,000 to $5,000 initial DCR
//! administrative, escalating to up to $10,000 first offense
//! / $25,000 subsequent offense via LAD enforcement. Actual
//! damages + attorney fees + punitive damages available
//! through private right of action.
//!
//! **New York — NY State Human Rights Law N.Y. Exec. Law §
//! 296(5)(a)(1) (state-level SOI ban eff. April 2019) + NYC
//! Admin. Code § 8-107(5)(a)(5) (city-level since 2008)**.
//! State-level SOI includes "lawful source of income"
//! including federal, state, or local public assistance,
//! housing assistance, child support, or alimony. NYC
//! Commission on Human Rights has obtained over $780K in
//! damages + penalties since 2014 enforcement.
//!
//! **Default — federal Fair Housing Act, 42 USC § 3604**. No
//! per se SOI protection at federal level. Section 8 voucher
//! refusal is NOT a federal FHA violation absent disparate-
//! impact / disparate-treatment showing. Local protections
//! may apply (Cook County IL, Seattle, DC, etc.) but not
//! universal.
//!
//! Citations: Cal. Gov. Code §§ 12955, 12987, 12989.2; SB 329
//! (2019); N.J.S.A. 10:5-12.5; N.Y. Exec. Law § 296(5)(a)(1);
//! NYC Admin. Code § 8-107(5)(a)(5); 42 USC § 3604 (federal
//! FHA baseline).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewJersey,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IncomeSourceType {
    /// Housing Choice Voucher (Section 8) — primary SOI
    /// litigation flashpoint.
    Section8HousingChoiceVoucher,
    /// HUD Veterans Affairs Supportive Housing.
    HudVash,
    /// Social Security disability income.
    SsiSsdi,
    /// Child support or alimony.
    ChildSupportOrAlimony,
    /// Temporary Assistance for Needy Families.
    Tanf,
    /// Employment wages — SOI protection does NOT apply
    /// (income-amount screening lawful).
    EmploymentWages,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RefusalReason {
    /// Landlord refused solely because of voucher/SOI status.
    VoucherSoleReason,
    /// Landlord refused with stated income-multiple
    /// rationale BUT applied to voucher in disparate manner.
    IncomeMultipleAppliedDisparately,
    /// Landlord refused for unrelated legitimate reason
    /// (criminal background per fair-chance, no SOI nexus).
    UnrelatedLegitimateReason,
    /// Landlord refused because property is owner-occupied
    /// single-family or qualifies for narrow carve-out.
    SmallLandlordCarveout,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SourceOfIncomeInput {
    pub regime: Regime,
    pub income_source: IncomeSourceType,
    pub refusal_reason: RefusalReason,
    /// Number of prior SOI discrimination findings against
    /// this landlord (escalates NJ LAD penalty tier).
    pub prior_findings_count: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SourceOfIncomeResult {
    pub violation: bool,
    pub soi_protection_engaged: bool,
    pub max_civil_penalty_cents: i64,
    pub actual_damages_available: bool,
    pub attorney_fees_available: bool,
    pub punitive_damages_available: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &SourceOfIncomeInput) -> SourceOfIncomeResult {
    let prior_count = input.prior_findings_count.min(1_000);

    match input.regime {
        Regime::California => check_california(input, prior_count),
        Regime::NewJersey => check_new_jersey(input, prior_count),
        Regime::NewYork => check_new_york(input, prior_count),
        Regime::Default => check_default(input),
    }
}

fn soi_protected(source: IncomeSourceType) -> bool {
    !matches!(source, IncomeSourceType::EmploymentWages)
}

fn check_california(
    input: &SourceOfIncomeInput,
    _prior_count: u32,
) -> SourceOfIncomeResult {
    let mut failure_reasons: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "California SB 329 (Housing Opportunities Act of 2019, eff. 2020) — Section 8 Housing Choice Vouchers explicitly added to FEHA source-of-income definition (Cal. Gov. Code § 12955)"
            .to_string(),
        "Cal. Gov. Code § 12987 + § 12989.2 — FEHA remedies: actual damages + emotional distress + civil penalty + attorney fees + injunctive relief"
            .to_string(),
    ];

    let protected = soi_protected(input.income_source);
    let mut violation = false;

    if protected {
        match input.refusal_reason {
            RefusalReason::VoucherSoleReason => {
                violation = true;
                failure_reasons.push(
                    "Cal. Gov. Code § 12955 + SB 329 — landlord may NOT refuse applicant solely because of Section 8 voucher participation".to_string(),
                );
            }
            RefusalReason::IncomeMultipleAppliedDisparately => {
                violation = true;
                failure_reasons.push(
                    "Cal. Gov. Code § 12955 — applying income-multiple test to voucher portion (instead of tenant's portion) is disparate treatment".to_string(),
                );
            }
            RefusalReason::SmallLandlordCarveout => {
                failure_reasons.push(
                    "Cal. Gov. Code § 12955 narrow owner-occupied single-family-dwelling carve-out (≤4 units) — verify applicability before relying on it".to_string(),
                );
            }
            RefusalReason::UnrelatedLegitimateReason => {}
        }
    }

    SourceOfIncomeResult {
        violation,
        soi_protection_engaged: protected,
        max_civil_penalty_cents: 5_000_000,
        actual_damages_available: violation,
        attorney_fees_available: violation,
        punitive_damages_available: violation,
        failure_reasons,
        citation: "Cal. Gov. Code §§ 12955, 12987, 12989.2; SB 329 (2019)",
        notes,
    }
}

fn check_new_jersey(
    input: &SourceOfIncomeInput,
    prior_count: u32,
) -> SourceOfIncomeResult {
    let mut failure_reasons: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "N.J.S.A. 10:5-12.5 (NJ Law Against Discrimination) — source of income covers Section 8 vouchers + public assistance; Division on Civil Rights administrative enforcement"
            .to_string(),
        "NJ LAD penalty tiers — DCR administrative $1,000-$5,000 initial; LAD enforcement up to $10,000 first offense / $25,000 subsequent offense + actual damages + attorney fees + punitive damages via private right of action"
            .to_string(),
    ];

    let protected = soi_protected(input.income_source);
    let mut violation = false;

    if protected
        && matches!(
            input.refusal_reason,
            RefusalReason::VoucherSoleReason
                | RefusalReason::IncomeMultipleAppliedDisparately
        )
    {
        violation = true;
        failure_reasons.push(
            "N.J.S.A. 10:5-12.5 — landlord may NOT refuse applicant solely because of Section 8 voucher participation; LAD source-of-income protection".to_string(),
        );
    }

    let max_penalty: i64 = if prior_count == 0 {
        1_000_000
    } else {
        2_500_000
    };

    SourceOfIncomeResult {
        violation,
        soi_protection_engaged: protected,
        max_civil_penalty_cents: max_penalty,
        actual_damages_available: violation,
        attorney_fees_available: violation,
        punitive_damages_available: violation,
        failure_reasons,
        citation: "N.J.S.A. 10:5-12.5 (NJ Law Against Discrimination)",
        notes,
    }
}

fn check_new_york(
    input: &SourceOfIncomeInput,
    _prior_count: u32,
) -> SourceOfIncomeResult {
    let mut failure_reasons: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "N.Y. Exec. Law § 296(5)(a)(1) (NY State Human Rights Law, SOI eff. April 2019) — landlord may not refuse applicant based on lawful source of income including federal, state, or local public assistance + housing assistance + child support + alimony"
            .to_string(),
        "NYC Admin. Code § 8-107(5)(a)(5) (NYC Human Rights Law, SOI eff. 2008) — NYC Commission has obtained over $780K in damages and penalties since 2014 enforcement"
            .to_string(),
    ];

    let protected = soi_protected(input.income_source);
    let mut violation = false;

    if protected
        && matches!(
            input.refusal_reason,
            RefusalReason::VoucherSoleReason
                | RefusalReason::IncomeMultipleAppliedDisparately
        )
    {
        violation = true;
        failure_reasons.push(
            "N.Y. Exec. Law § 296(5)(a)(1) + NYC Admin. Code § 8-107(5)(a)(5) — landlord may NOT refuse applicant based on lawful source of income".to_string(),
        );
    }

    SourceOfIncomeResult {
        violation,
        soi_protection_engaged: protected,
        max_civil_penalty_cents: 25_000_000,
        actual_damages_available: violation,
        attorney_fees_available: violation,
        punitive_damages_available: violation,
        failure_reasons,
        citation: "N.Y. Exec. Law § 296(5)(a)(1); NYC Admin. Code § 8-107(5)(a)(5)",
        notes,
    }
}

fn check_default(_input: &SourceOfIncomeInput) -> SourceOfIncomeResult {
    let notes: Vec<String> = vec![
        "default rule — federal Fair Housing Act, 42 USC § 3604, provides NO per se source-of-income protection; Section 8 voucher refusal not federal FHA violation absent disparate-treatment / disparate-impact showing"
            .to_string(),
        "default rule — local protections may apply (Cook County IL, Seattle WA, DC, etc.); verify jurisdiction-specific ordinances before relying on default rule"
            .to_string(),
    ];

    SourceOfIncomeResult {
        violation: false,
        soi_protection_engaged: false,
        max_civil_penalty_cents: 0,
        actual_damages_available: false,
        attorney_fees_available: false,
        punitive_damages_available: false,
        failure_reasons: Vec::new(),
        citation: "42 USC § 3604 (federal FHA baseline)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_base() -> SourceOfIncomeInput {
        SourceOfIncomeInput {
            regime: Regime::California,
            income_source: IncomeSourceType::Section8HousingChoiceVoucher,
            refusal_reason: RefusalReason::VoucherSoleReason,
            prior_findings_count: 0,
        }
    }

    fn nj_base() -> SourceOfIncomeInput {
        let mut i = ca_base();
        i.regime = Regime::NewJersey;
        i
    }

    fn ny_base() -> SourceOfIncomeInput {
        let mut i = ca_base();
        i.regime = Regime::NewYork;
        i
    }

    fn default_base() -> SourceOfIncomeInput {
        let mut i = ca_base();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_voucher_refusal_violates() {
        let r = check(&ca_base());
        assert!(r.violation);
        assert!(r.soi_protection_engaged);
        assert!(r.actual_damages_available);
        assert!(r.attorney_fees_available);
    }

    #[test]
    fn ca_employment_wages_no_protection() {
        let mut i = ca_base();
        i.income_source = IncomeSourceType::EmploymentWages;
        let r = check(&i);
        assert!(!r.violation);
        assert!(!r.soi_protection_engaged);
    }

    #[test]
    fn ca_disparate_income_multiple_violates() {
        let mut i = ca_base();
        i.refusal_reason = RefusalReason::IncomeMultipleAppliedDisparately;
        let r = check(&i);
        assert!(r.violation);
        assert!(r.failure_reasons.iter().any(|f| f.contains("disparate treatment")));
    }

    #[test]
    fn ca_unrelated_legitimate_reason_no_violation() {
        let mut i = ca_base();
        i.refusal_reason = RefusalReason::UnrelatedLegitimateReason;
        let r = check(&i);
        assert!(!r.violation);
        assert!(r.soi_protection_engaged);
    }

    #[test]
    fn ca_citation_pins_sb329_and_subsections() {
        let r = check(&ca_base());
        assert!(r.citation.contains("§§ 12955, 12987, 12989.2"));
        assert!(r.citation.contains("SB 329"));
    }

    #[test]
    fn ca_max_penalty_50k_cents() {
        let r = check(&ca_base());
        assert_eq!(r.max_civil_penalty_cents, 5_000_000);
    }

    #[test]
    fn nj_voucher_refusal_violates() {
        let r = check(&nj_base());
        assert!(r.violation);
        assert!(r.failure_reasons.iter().any(|f| f.contains("N.J.S.A. 10:5-12.5")));
    }

    #[test]
    fn nj_first_offense_penalty_10k() {
        let r = check(&nj_base());
        assert_eq!(r.max_civil_penalty_cents, 1_000_000);
    }

    #[test]
    fn nj_subsequent_offense_penalty_25k() {
        let mut i = nj_base();
        i.prior_findings_count = 1;
        let r = check(&i);
        assert_eq!(r.max_civil_penalty_cents, 2_500_000);
    }

    #[test]
    fn nj_citation_pins_lad() {
        let r = check(&nj_base());
        assert!(r.citation.contains("N.J.S.A. 10:5-12.5"));
        assert!(r.citation.contains("NJ Law Against Discrimination"));
    }

    #[test]
    fn ny_voucher_refusal_violates() {
        let r = check(&ny_base());
        assert!(r.violation);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 296(5)(a)(1)")));
    }

    #[test]
    fn ny_citation_pins_state_and_nyc() {
        let r = check(&ny_base());
        assert!(r.citation.contains("N.Y. Exec. Law § 296(5)(a)(1)"));
        assert!(r.citation.contains("§ 8-107(5)(a)(5)"));
    }

    #[test]
    fn ny_nyc_enforcement_note_pinned() {
        let r = check(&ny_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("$780K") && n.contains("2014")));
    }

    #[test]
    fn default_no_protection_no_violation() {
        let r = check(&default_base());
        assert!(!r.violation);
        assert!(!r.soi_protection_engaged);
        assert_eq!(r.max_civil_penalty_cents, 0);
    }

    #[test]
    fn default_citation_pins_fha_baseline() {
        let r = check(&default_base());
        assert!(r.citation.contains("42 USC § 3604"));
    }

    #[test]
    fn protected_source_truth_table() {
        for (source, protected) in [
            (IncomeSourceType::Section8HousingChoiceVoucher, true),
            (IncomeSourceType::HudVash, true),
            (IncomeSourceType::SsiSsdi, true),
            (IncomeSourceType::ChildSupportOrAlimony, true),
            (IncomeSourceType::Tanf, true),
            (IncomeSourceType::EmploymentWages, false),
        ] {
            let mut i = ca_base();
            i.income_source = source;
            let r = check(&i);
            assert_eq!(r.soi_protection_engaged, protected);
        }
    }

    #[test]
    fn four_regimes_routed_correctly() {
        for regime in [
            Regime::California,
            Regime::NewJersey,
            Regime::NewYork,
            Regime::Default,
        ] {
            let mut i = ca_base();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn protected_regimes_all_punish_voucher_refusal() {
        for regime in [Regime::California, Regime::NewJersey, Regime::NewYork] {
            let mut i = ca_base();
            i.regime = regime;
            let r = check(&i);
            assert!(r.violation);
            assert!(r.actual_damages_available);
        }
    }

    #[test]
    fn default_uniquely_no_protection_invariant() {
        let r_default = check(&default_base());
        assert!(!r_default.soi_protection_engaged);

        for regime in [Regime::California, Regime::NewJersey, Regime::NewYork] {
            let mut i = ca_base();
            i.regime = regime;
            let r = check(&i);
            assert!(r.soi_protection_engaged);
        }
    }

    #[test]
    fn refusal_reason_truth_table_california() {
        for (reason, expect_violation) in [
            (RefusalReason::VoucherSoleReason, true),
            (RefusalReason::IncomeMultipleAppliedDisparately, true),
            (RefusalReason::UnrelatedLegitimateReason, false),
            (RefusalReason::SmallLandlordCarveout, false),
        ] {
            let mut i = ca_base();
            i.refusal_reason = reason;
            let r = check(&i);
            assert_eq!(r.violation, expect_violation);
        }
    }

    #[test]
    fn ca_small_landlord_carveout_pins_warning_note() {
        let mut i = ca_base();
        i.refusal_reason = RefusalReason::SmallLandlordCarveout;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("owner-occupied") && f.contains("≤4 units")));
    }

    #[test]
    fn nj_uniquely_tiered_penalty_invariant() {
        let mut i_first = nj_base();
        i_first.prior_findings_count = 0;
        let r_first = check(&i_first);

        let mut i_subsequent = nj_base();
        i_subsequent.prior_findings_count = 5;
        let r_subsequent = check(&i_subsequent);

        assert_eq!(r_first.max_civil_penalty_cents, 1_000_000);
        assert_eq!(r_subsequent.max_civil_penalty_cents, 2_500_000);
        assert!(r_subsequent.max_civil_penalty_cents > r_first.max_civil_penalty_cents);

        let r_ca = check(&ca_base());
        assert_eq!(r_ca.max_civil_penalty_cents, 5_000_000);
    }

    #[test]
    fn defensive_prior_findings_overflow_clamped() {
        let mut i = nj_base();
        i.prior_findings_count = u32::MAX;
        let r = check(&i);
        assert!(r.violation);
        assert_eq!(r.max_civil_penalty_cents, 2_500_000);
    }

    #[test]
    fn ssi_ssdi_protected_in_ca() {
        let mut i = ca_base();
        i.income_source = IncomeSourceType::SsiSsdi;
        let r = check(&i);
        assert!(r.violation);
    }

    #[test]
    fn child_support_protected_in_ny() {
        let mut i = ny_base();
        i.income_source = IncomeSourceType::ChildSupportOrAlimony;
        let r = check(&i);
        assert!(r.violation);
    }

    #[test]
    fn tanf_protected_in_nj() {
        let mut i = nj_base();
        i.income_source = IncomeSourceType::Tanf;
        let r = check(&i);
        assert!(r.violation);
    }

    #[test]
    fn hud_vash_protected_in_ca() {
        let mut i = ca_base();
        i.income_source = IncomeSourceType::HudVash;
        let r = check(&i);
        assert!(r.violation);
    }

    #[test]
    fn ca_note_pins_sb329_eff_2020() {
        let r = check(&ca_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("SB 329") && n.contains("eff. 2020")));
    }

    #[test]
    fn nj_note_pins_dcr_penalty_range() {
        let r = check(&nj_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("DCR") && n.contains("$1,000-$5,000") && n.contains("$25,000")));
    }

    #[test]
    fn ny_note_pins_eff_april_2019() {
        let r = check(&ny_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("April 2019")));
    }

    #[test]
    fn default_note_pins_local_protections_warning() {
        let r = check(&default_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Cook County") && n.contains("Seattle")));
    }

    #[test]
    fn three_remedies_engage_together_in_protected_regimes() {
        for regime in [Regime::California, Regime::NewJersey, Regime::NewYork] {
            let mut i = ca_base();
            i.regime = regime;
            let r = check(&i);
            assert!(r.actual_damages_available);
            assert!(r.attorney_fees_available);
            assert!(r.punitive_damages_available);
        }
    }
}
