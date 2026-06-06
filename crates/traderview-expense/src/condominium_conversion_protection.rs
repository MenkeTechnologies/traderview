//! State landlord condominium-conversion tenant protection
//! compliance check.
//!
//! When a rental building is converted to condominium (or
//! cooperative) ownership, tenants' rental tenancies are at risk.
//! Five states + DC have layered protections — first-refusal
//! purchase rights, relocation assistance, notice periods, and
//! (in DC) a tenant-election prerequisite to conversion at all.
//! Other jurisdictions rely on lease enforcement plus common law
//! with no specific statutory protection.
//!
//! Six regimes:
//!
//!   - **DistrictOfColumbia** — DC Code § 42-3401.01 et seq.
//!     (Rental Housing Conversion and Sale Act) + § 42-1904.08
//!     (notice + tenant right to purchase). The owner cannot
//!     convert without (a) Mayor certifying compliance AND (b) a
//!     MAJORITY of tenants voting for conversion in a certified
//!     election. Pairs with the DC Tenant Opportunity to Purchase
//!     Act (TOPA) right-of-first-refusal. Displaced tenants get
//!     statutory relocation assistance.
//!
//!   - **Massachusetts** — G.L. c. 527 of the Acts of 1983
//!     (Massachusetts Condo Conversion Law). Applies only to
//!     buildings of 4+ rental units. Tenants get a 90-day
//!     first-refusal grace period at the same or better terms as
//!     offered to the public. Relocation assistance: $750 per
//!     tenant or $1,000 for elderly / disabled / low-or-moderate-
//!     income tenants. Notice period: 1 year for standard tenants
//!     extending up to 4 years for elderly or disabled tenants.
//!
//!   - **NewJersey** — N.J.S.A. 2A:18-61.22 et seq. (Senior
//!     Citizens and Disabled Protected Tenancy Act) + Tenant
//!     Protection Act of 1992. Senior (62+) and disabled tenants
//!     obtain PROTECTED TENANCY status — landlord may not bring
//!     an action for possession against a protected tenant for
//!     the duration of the protected tenancy period (up to 40
//!     years). Effectively bars condo-conversion eviction of
//!     these classes.
//!
//!   - **NewYork** — N.Y. Gen. Bus. Law § 352-e (plan filing with
//!     Attorney General) + § 352-eee (state-wide conversion) +
//!     § 352-eeee (NYC-specific). Two plan types: EVICTION PLAN
//!     (requires 51% tenant purchase commitment; allows eviction
//!     of non-purchasers) and NON-EVICTION PLAN (requires 15%
//!     tenant purchase commitment; tenants who don't buy may
//!     remain as rent-regulated). Eligible senior + disabled
//!     tenants receive 99-year non-eviction tenure regardless of
//!     plan type.
//!
//!   - **MarylandMontgomery** — Montgomery County Code Ch. 11A
//!     (Condominium Conversion Tenant Displacement). Landlord must
//!     deliver 180-day notice of conversion + offer right of first
//!     refusal + provide relocation assistance to non-purchasing
//!     tenants. Maryland state law (Md. Code Real Property
//!     § 11-102.1) requires 180-day pre-recording notice; counties
//!     may layer more.
//!
//!   - **Default** — no statewide tenant protection on conversion;
//!     common-law plus lease enforcement controls.
//!
//! Citations: DC Code § 42-3401.01 et seq. (Rental Housing
//! Conversion and Sale Act); DC Code § 42-1904.08 (notice + tenant
//! right to purchase + § 42-3402.02 conversion procedures);
//! G.L. c. 527 of the Acts of 1983 (MA Condo Conversion Law);
//! N.J.S.A. 2A:18-61.22 (NJ Senior Citizens and Disabled Protected
//! Tenancy Act); N.Y. Gen. Bus. Law § 352-e (NY filing) +
//! § 352-eee + § 352-eeee (NY conversion plans); Mont. County Code
//! Ch. 11A (MD Montgomery County condo conversion); Md. Code Real
//! Property § 11-102.1 (MD pre-recording notice).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    DistrictOfColumbia,
    Massachusetts,
    NewJersey,
    NewYork,
    MarylandMontgomery,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TenantClass {
    /// Standard tenant — no elevated protection class.
    Standard,
    /// Senior tenant — typically age 62 or older (NJ + NY) or 60
    /// or older (MA varies).
    Senior,
    /// Disabled tenant — disability-defined narrowly by each
    /// regime's statutory test.
    Disabled,
    /// Low- or moderate-income tenant — Massachusetts elevated
    /// relocation tier.
    LowModerateIncome,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NyPlanType {
    /// Not in NY — irrelevant.
    NotApplicable,
    /// N.Y. Gen. Bus. Law § 352-eeee EVICTION PLAN — requires 51%
    /// tenant purchase commitment; allows eviction of non-purchasers
    /// after a transition period.
    EvictionPlan,
    /// N.Y. Gen. Bus. Law § 352-eeee NON-EVICTION PLAN — requires
    /// 15% tenant purchase commitment; non-purchasers may remain
    /// as rent-regulated tenants.
    NonEvictionPlan,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub building_unit_count: u32,
    /// DC § 42-3401.01: majority of tenants voted for conversion
    /// in a certified election. Required as a prerequisite to
    /// conversion at all.
    pub tenant_majority_vote_obtained: bool,
    pub tenant_class: TenantClass,
    /// Days of advance notice delivered to the tenant before
    /// conversion / possession termination.
    pub notice_days_before_conversion: u32,
    /// Relocation assistance paid to the tenant (cents).
    pub relocation_assistance_paid_cents: i64,
    /// Whether the landlord offered the tenant the right of first
    /// refusal to purchase the converted unit.
    pub right_of_first_refusal_offered: bool,
    /// Days the tenant was given to accept the first-refusal offer.
    pub days_for_first_refusal_acceptance: u32,
    /// NY-only — plan type filed with the Attorney General.
    pub ny_plan_type: NyPlanType,
    /// NY-only — actual tenant purchase commitment percentage of
    /// units (basis points × 100; e.g., 5100 = 51.00%).
    pub ny_tenant_purchase_commitment_bp: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    pub compliant: bool,
    /// Required advance notice days for the regime + tenant class.
    pub required_notice_days: u32,
    /// Required first-refusal acceptance window (days).
    pub required_first_refusal_days: u32,
    /// Required relocation assistance (cents) — varies by regime
    /// and tenant class. Zero where the regime imposes no
    /// statutory assistance.
    pub required_relocation_assistance_cents: i64,
    /// Whether the conversion is barred under the present facts
    /// (e.g., NJ protected tenancy; DC majority-vote not obtained).
    pub conversion_barred: bool,
    /// Whether the building is large enough to trigger the regime
    /// (MA c. 527 requires 4+ units; others have no size threshold).
    pub size_threshold_met: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut conversion_barred = false;

    let (
        required_notice_days,
        required_first_refusal_days,
        required_relocation_assistance_cents,
        size_threshold_met,
        citation,
    ): (u32, u32, i64, bool, &'static str) = match input.regime {
        Regime::DistrictOfColumbia => {
            // DC majority-vote prerequisite — failure bars conversion.
            if !input.tenant_majority_vote_obtained {
                conversion_barred = true;
                violations.push(
                    "DC Code § 42-3401.01 et seq.: conversion requires a MAJORITY of tenants \
                     voting for conversion in a certified election; the majority vote has not \
                     been obtained — conversion is BARRED."
                        .to_string(),
                );
            }
            (
                120,
                60,
                0,
                true,
                "DC Code § 42-3401.01 et seq. (Rental Housing Conversion and Sale Act — \
                 majority tenant-vote prerequisite to conversion + Mayor certification); DC \
                 Code § 42-1904.08 (tenant right to purchase + notice to vacate); § 42-3402.02 \
                 (conversion procedures); TOPA coordination",
            )
        }
        Regime::Massachusetts => {
            // Massachusetts c. 527 applies only to buildings of
            // 4+ rental units.
            let size_ok = input.building_unit_count >= 4;
            if !size_ok {
                notes.push(
                    "Massachusetts G.L. c. 527 applies only to buildings of 4 or more rental \
                     units; this building falls below the threshold and the statute does not \
                     reach the conversion."
                        .to_string(),
                );
            }
            // Notice + relocation vary by tenant class.
            let (notice, relocation) = match input.tenant_class {
                TenantClass::Senior | TenantClass::Disabled => (
                    1460,    // 4 years
                    100_000, // $1,000
                ),
                TenantClass::LowModerateIncome => (
                    730,     // 2 years
                    100_000, // $1,000
                ),
                TenantClass::Standard => (
                    365,    // 1 year
                    75_000, // $750
                ),
            };
            (
                notice,
                90,
                relocation,
                size_ok,
                "G.L. c. 527 of the Acts of 1983 (Massachusetts Condo Conversion Law — applies \
                 to buildings of 4+ rental units; 90-day first-refusal grace period at same or \
                 better terms; relocation $750 standard / $1,000 elderly + disabled + \
                 low-moderate-income; notice 1 year standard / 2 years low-moderate / 4 years \
                 elderly + disabled)",
            )
        }
        Regime::NewJersey => {
            // NJ — protected tenancy bars conversion eviction of
            // senior + disabled tenants.
            if matches!(
                input.tenant_class,
                TenantClass::Senior | TenantClass::Disabled
            ) {
                conversion_barred = true;
                notes.push(
                    "N.J.S.A. 2A:18-61.22 et seq. — Senior Citizens and Disabled Protected \
                     Tenancy Act bars the landlord from bringing an action for possession \
                     against a senior or disabled tenant during the protected tenancy period \
                     (up to 40 years). Conversion eviction of these classes is BARRED."
                        .to_string(),
                );
            }
            (
                365,
                60,
                0,
                true,
                "N.J.S.A. 2A:18-61.22 et seq. (Senior Citizens and Disabled Protected Tenancy \
                 Act — up to 40-year protected tenancy period bars conversion eviction); \
                 N.J.S.A. Tenant Protection Act of 1992",
            )
        }
        Regime::NewYork => {
            // NY plan-type threshold check.
            let (notice, first_refusal, plan_note) = match input.ny_plan_type {
                NyPlanType::EvictionPlan => {
                    // Eviction plan requires 51% tenant purchase
                    // commitment.
                    if input.ny_tenant_purchase_commitment_bp < 5_100 {
                        violations.push(format!(
                            "N.Y. Gen. Bus. Law § 352-eeee EVICTION PLAN: 51% tenant purchase \
                             commitment required; actual commitment is {} basis points \
                             ({}%) — plan does not satisfy the threshold.",
                            input.ny_tenant_purchase_commitment_bp,
                            input.ny_tenant_purchase_commitment_bp as f64 / 100.0,
                        ));
                    }
                    (
                        1095, // 3 years
                        90,
                        "EVICTION PLAN — 51% tenant purchase commitment required; allows \
                         eviction of non-purchasers after transition period",
                    )
                }
                NyPlanType::NonEvictionPlan => {
                    // Non-eviction plan requires 15% commitment.
                    if input.ny_tenant_purchase_commitment_bp < 1_500 {
                        violations.push(format!(
                            "N.Y. Gen. Bus. Law § 352-eeee NON-EVICTION PLAN: 15% tenant \
                             purchase commitment required; actual commitment is {} basis \
                             points ({}%) — plan does not satisfy the threshold.",
                            input.ny_tenant_purchase_commitment_bp,
                            input.ny_tenant_purchase_commitment_bp as f64 / 100.0,
                        ));
                    }
                    (
                        365,
                        90,
                        "NON-EVICTION PLAN — 15% tenant purchase commitment required; \
                         non-purchasers may remain as rent-regulated tenants",
                    )
                }
                NyPlanType::NotApplicable => (365, 90, "no plan type specified"),
            };
            // 99-year non-eviction tenure for eligible senior +
            // disabled under § 352-eeee, regardless of plan type.
            if matches!(
                input.tenant_class,
                TenantClass::Senior | TenantClass::Disabled
            ) {
                notes.push(
                    "N.Y. Gen. Bus. Law § 352-eeee — eligible senior and disabled tenants \
                     receive 99-year non-eviction tenure regardless of EVICTION or \
                     NON-EVICTION plan type."
                        .to_string(),
                );
            }
            notes.push(format!("Plan type: {}.", plan_note));
            (
                notice,
                first_refusal,
                0,
                true,
                "N.Y. Gen. Bus. Law § 352-e (Attorney General plan filing); § 352-eee + \
                 § 352-eeee (NY conversion plans — EVICTION PLAN 51% threshold; NON-EVICTION \
                 PLAN 15% threshold; senior + disabled 99-year non-eviction tenure)",
            )
        }
        Regime::MarylandMontgomery => (
            180,
            60,
            0,
            true,
            "Mont. County Code Ch. 11A (Condominium Conversion Tenant Displacement — 180-day \
             notice + right of first refusal + relocation assistance); Md. Code Real Property \
             § 11-102.1 (state-level 180-day pre-recording notice)",
        ),
        Regime::Default => (
            0,
            0,
            0,
            true,
            "No statewide tenant protection on condominium conversion; common-law and lease \
             enforcement control",
        ),
    };

    // Notice compliance.
    if required_notice_days > 0
        && size_threshold_met
        && !conversion_barred
        && input.notice_days_before_conversion < required_notice_days
    {
        violations.push(format!(
            "Advance notice was {} days; regime requires {} days for {:?} tenant class.",
            input.notice_days_before_conversion, required_notice_days, input.tenant_class,
        ));
    }

    // First-refusal compliance.
    if required_first_refusal_days > 0 && size_threshold_met && !conversion_barred {
        if !input.right_of_first_refusal_offered {
            violations.push(
                "Landlord did not offer the tenant the statutory right of first refusal to \
                 purchase the converted unit."
                    .to_string(),
            );
        } else if input.days_for_first_refusal_acceptance < required_first_refusal_days {
            violations.push(format!(
                "First-refusal acceptance window was {} days; regime requires {} days.",
                input.days_for_first_refusal_acceptance, required_first_refusal_days
            ));
        }
    }

    // Relocation assistance compliance.
    if required_relocation_assistance_cents > 0
        && size_threshold_met
        && !conversion_barred
        && input.relocation_assistance_paid_cents < required_relocation_assistance_cents
    {
        violations.push(format!(
            "Relocation assistance paid {} cents; regime requires {} cents for {:?} tenant \
             class.",
            input.relocation_assistance_paid_cents,
            required_relocation_assistance_cents,
            input.tenant_class,
        ));
    }

    notes.push(
        "Companion to tenant_relocation_assistance (which addresses general post-displacement \
         relocation aid) and foreclosure_tenant_rights (which addresses tenant rights on owner \
         default). This module focuses specifically on the OWNER-INITIATED conversion of the \
         building from rental to condominium / cooperative ownership."
            .to_string(),
    );

    CheckResult {
        compliant: violations.is_empty() && !conversion_barred,
        required_notice_days,
        required_first_refusal_days,
        required_relocation_assistance_cents,
        conversion_barred,
        size_threshold_met,
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(regime: Regime) -> Input {
        Input {
            regime,
            building_unit_count: 12,
            tenant_majority_vote_obtained: true,
            tenant_class: TenantClass::Standard,
            notice_days_before_conversion: 1500,
            relocation_assistance_paid_cents: 150_000,
            right_of_first_refusal_offered: true,
            days_for_first_refusal_acceptance: 120,
            ny_plan_type: NyPlanType::NotApplicable,
            ny_tenant_purchase_commitment_bp: 0,
        }
    }

    // ── DC Rental Housing Conversion and Sale Act ───────────────

    #[test]
    fn dc_majority_vote_obtained_compliant() {
        let r = check(&base(Regime::DistrictOfColumbia));
        assert!(r.compliant);
        assert!(!r.conversion_barred);
        assert!(r.citation.contains("§ 42-3401.01"));
        assert!(r.citation.contains("§ 42-1904.08"));
    }

    #[test]
    fn dc_no_majority_vote_conversion_barred() {
        let mut i = base(Regime::DistrictOfColumbia);
        i.tenant_majority_vote_obtained = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.conversion_barred);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("MAJORITY") && v.contains("BARRED")));
    }

    // ── Massachusetts G.L. c. 527 ───────────────────────────────

    #[test]
    fn massachusetts_4_plus_units_standard_tenant_compliant() {
        let r = check(&base(Regime::Massachusetts));
        assert!(r.compliant);
        assert!(r.size_threshold_met);
        assert_eq!(r.required_notice_days, 365);
        assert_eq!(r.required_relocation_assistance_cents, 75_000);
        assert!(r.citation.contains("c. 527"));
    }

    #[test]
    fn massachusetts_3_units_below_threshold_no_statute() {
        let mut i = base(Regime::Massachusetts);
        i.building_unit_count = 3;
        i.notice_days_before_conversion = 0;
        i.relocation_assistance_paid_cents = 0;
        i.right_of_first_refusal_offered = false;
        let r = check(&i);
        assert!(!r.size_threshold_met);
        // Below threshold → no statutory violation.
        assert!(r.compliant);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("4 or more rental units") && n.contains("below the threshold")));
    }

    #[test]
    fn massachusetts_4_units_at_threshold_statute_applies() {
        let mut i = base(Regime::Massachusetts);
        i.building_unit_count = 4;
        let r = check(&i);
        assert!(r.size_threshold_met);
        assert!(r.compliant);
    }

    #[test]
    fn massachusetts_senior_4_year_notice_required() {
        let mut i = base(Regime::Massachusetts);
        i.tenant_class = TenantClass::Senior;
        i.notice_days_before_conversion = 1460;
        i.relocation_assistance_paid_cents = 100_000;
        let r = check(&i);
        assert_eq!(r.required_notice_days, 1460);
        assert_eq!(r.required_relocation_assistance_cents, 100_000);
        assert!(r.compliant);
    }

    #[test]
    fn massachusetts_disabled_4_year_notice_and_1000_relocation() {
        let mut i = base(Regime::Massachusetts);
        i.tenant_class = TenantClass::Disabled;
        i.notice_days_before_conversion = 1460;
        i.relocation_assistance_paid_cents = 100_000;
        let r = check(&i);
        assert_eq!(r.required_notice_days, 1460);
        assert_eq!(r.required_relocation_assistance_cents, 100_000);
        assert!(r.compliant);
    }

    #[test]
    fn massachusetts_low_moderate_income_2_year_notice_and_1000_relocation() {
        let mut i = base(Regime::Massachusetts);
        i.tenant_class = TenantClass::LowModerateIncome;
        i.notice_days_before_conversion = 730;
        i.relocation_assistance_paid_cents = 100_000;
        let r = check(&i);
        assert_eq!(r.required_notice_days, 730);
        assert_eq!(r.required_relocation_assistance_cents, 100_000);
        assert!(r.compliant);
    }

    #[test]
    fn massachusetts_under_notice_violation() {
        let mut i = base(Regime::Massachusetts);
        i.notice_days_before_conversion = 364;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("365") && v.contains("364")));
    }

    #[test]
    fn massachusetts_relocation_below_threshold_violation() {
        let mut i = base(Regime::Massachusetts);
        i.relocation_assistance_paid_cents = 50_000; // $500
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Relocation assistance") && v.contains("75000")));
    }

    // ── New Jersey 2A:18-61.22 Protected Tenancy Act ────────────

    #[test]
    fn new_jersey_senior_protected_tenancy_bars_conversion() {
        let mut i = base(Regime::NewJersey);
        i.tenant_class = TenantClass::Senior;
        let r = check(&i);
        assert!(r.conversion_barred);
        assert!(!r.compliant);
        assert!(r.citation.contains("2A:18-61.22"));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("40 years") && n.contains("BARRED")));
    }

    #[test]
    fn new_jersey_disabled_protected_tenancy_bars_conversion() {
        let mut i = base(Regime::NewJersey);
        i.tenant_class = TenantClass::Disabled;
        let r = check(&i);
        assert!(r.conversion_barred);
    }

    #[test]
    fn new_jersey_standard_tenant_no_bar() {
        let r = check(&base(Regime::NewJersey));
        assert!(!r.conversion_barred);
        assert!(r.compliant);
    }

    // ── New York § 352-eeee plan types ──────────────────────────

    #[test]
    fn new_york_eviction_plan_at_51_percent_compliant() {
        let mut i = base(Regime::NewYork);
        i.ny_plan_type = NyPlanType::EvictionPlan;
        i.ny_tenant_purchase_commitment_bp = 5_100;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.citation.contains("§ 352-eeee"));
    }

    #[test]
    fn new_york_eviction_plan_below_51_percent_violation() {
        let mut i = base(Regime::NewYork);
        i.ny_plan_type = NyPlanType::EvictionPlan;
        i.ny_tenant_purchase_commitment_bp = 5_000;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("EVICTION PLAN") && v.contains("51%")));
    }

    #[test]
    fn new_york_non_eviction_plan_at_15_percent_compliant() {
        let mut i = base(Regime::NewYork);
        i.ny_plan_type = NyPlanType::NonEvictionPlan;
        i.ny_tenant_purchase_commitment_bp = 1_500;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn new_york_non_eviction_plan_below_15_percent_violation() {
        let mut i = base(Regime::NewYork);
        i.ny_plan_type = NyPlanType::NonEvictionPlan;
        i.ny_tenant_purchase_commitment_bp = 1_400;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("NON-EVICTION PLAN") && v.contains("15%")));
    }

    #[test]
    fn new_york_senior_99_year_non_eviction_note() {
        let mut i = base(Regime::NewYork);
        i.tenant_class = TenantClass::Senior;
        i.ny_plan_type = NyPlanType::NonEvictionPlan;
        i.ny_tenant_purchase_commitment_bp = 1_500;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("99-year non-eviction")));
    }

    // ── Maryland Montgomery County ─────────────────────────────

    #[test]
    fn maryland_montgomery_180_day_notice_compliant() {
        let r = check(&base(Regime::MarylandMontgomery));
        assert!(r.compliant);
        assert_eq!(r.required_notice_days, 180);
        assert!(r.citation.contains("Mont. County Code Ch. 11A"));
    }

    #[test]
    fn maryland_montgomery_under_180_days_violation() {
        let mut i = base(Regime::MarylandMontgomery);
        i.notice_days_before_conversion = 179;
        let r = check(&i);
        assert!(!r.compliant);
    }

    // ── Default — no statewide protection ──────────────────────

    #[test]
    fn default_no_statutory_protection() {
        let mut i = base(Regime::Default);
        i.notice_days_before_conversion = 0;
        i.right_of_first_refusal_offered = false;
        i.relocation_assistance_paid_cents = 0;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.required_notice_days, 0);
        assert!(r.citation.contains("No statewide tenant protection"));
    }

    // ── First-refusal compliance ────────────────────────────────

    #[test]
    fn missing_first_refusal_offer_violation() {
        let mut i = base(Regime::Massachusetts);
        i.right_of_first_refusal_offered = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("right of first refusal")));
    }

    #[test]
    fn first_refusal_acceptance_window_too_short_violation() {
        let mut i = base(Regime::Massachusetts);
        i.days_for_first_refusal_acceptance = 89;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("First-refusal") && v.contains("90")));
    }

    // ── Regression-critical multi-regime invariants ────────────

    #[test]
    fn only_dc_requires_tenant_majority_vote_invariant() {
        let mut dc = base(Regime::DistrictOfColumbia);
        dc.tenant_majority_vote_obtained = false;
        assert!(check(&dc).conversion_barred);

        for &regime in &[
            Regime::Massachusetts,
            Regime::NewJersey,
            Regime::NewYork,
            Regime::MarylandMontgomery,
            Regime::Default,
        ] {
            let mut i = base(regime);
            i.tenant_majority_vote_obtained = false;
            i.ny_plan_type = NyPlanType::NonEvictionPlan;
            i.ny_tenant_purchase_commitment_bp = 1_500;
            // Should not bar conversion solely on majority-vote
            // absence in non-DC regimes.
            let r = check(&i);
            if !matches!(regime, Regime::NewJersey) {
                assert!(
                    !r.conversion_barred,
                    "{:?}: must NOT bar on majority-vote absence",
                    regime,
                );
            }
        }
    }

    #[test]
    fn only_nj_bars_conversion_for_protected_senior_disabled_invariant() {
        let mut nj_senior = base(Regime::NewJersey);
        nj_senior.tenant_class = TenantClass::Senior;
        assert!(check(&nj_senior).conversion_barred);

        for &regime in &[
            Regime::DistrictOfColumbia,
            Regime::Massachusetts,
            Regime::NewYork,
            Regime::MarylandMontgomery,
            Regime::Default,
        ] {
            let mut i = base(regime);
            i.tenant_class = TenantClass::Senior;
            i.ny_plan_type = NyPlanType::NonEvictionPlan;
            i.ny_tenant_purchase_commitment_bp = 1_500;
            assert!(
                !check(&i).conversion_barred,
                "{:?}: must NOT statutorily bar conversion for senior tenants",
                regime,
            );
        }
    }

    #[test]
    fn only_massachusetts_has_size_threshold_invariant() {
        let mut ma = base(Regime::Massachusetts);
        ma.building_unit_count = 3;
        assert!(!check(&ma).size_threshold_met);

        for &regime in &[
            Regime::DistrictOfColumbia,
            Regime::NewJersey,
            Regime::NewYork,
            Regime::MarylandMontgomery,
            Regime::Default,
        ] {
            let mut i = base(regime);
            i.building_unit_count = 3;
            i.ny_plan_type = NyPlanType::NonEvictionPlan;
            i.ny_tenant_purchase_commitment_bp = 1_500;
            assert!(
                check(&i).size_threshold_met,
                "{:?}: must NOT have a size threshold",
                regime,
            );
        }
    }

    #[test]
    fn only_ny_uses_plan_type_threshold_branching_invariant() {
        for plan_type in [NyPlanType::EvictionPlan, NyPlanType::NonEvictionPlan] {
            let mut ny = base(Regime::NewYork);
            ny.ny_plan_type = plan_type;
            ny.ny_tenant_purchase_commitment_bp = 0;
            // Should violate for NY when commitment is 0.
            assert!(!check(&ny).compliant);
        }

        for &regime in &[
            Regime::DistrictOfColumbia,
            Regime::Massachusetts,
            Regime::NewJersey,
            Regime::MarylandMontgomery,
            Regime::Default,
        ] {
            let mut a = base(regime);
            a.ny_plan_type = NyPlanType::EvictionPlan;
            a.ny_tenant_purchase_commitment_bp = 0;
            let mut b = base(regime);
            b.ny_plan_type = NyPlanType::NonEvictionPlan;
            b.ny_tenant_purchase_commitment_bp = 1_500;
            // Non-NY regimes ignore plan type entirely — compliance
            // should not depend on it.
            assert_eq!(
                check(&a).compliant,
                check(&b).compliant,
                "{:?}: plan-type branching must be a no-op",
                regime,
            );
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(check(&base(Regime::DistrictOfColumbia))
            .citation
            .contains("§ 42-3401.01"));
        assert!(check(&base(Regime::Massachusetts))
            .citation
            .contains("c. 527"));
        assert!(check(&base(Regime::NewJersey))
            .citation
            .contains("2A:18-61.22"));
        assert!(check(&base(Regime::NewYork)).citation.contains("§ 352-e"));
        assert!(check(&base(Regime::MarylandMontgomery))
            .citation
            .contains("Mont. County Code Ch. 11A"));
        assert!(check(&base(Regime::Default))
            .citation
            .contains("No statewide"));
    }

    #[test]
    fn sibling_module_note_present_across_all_regimes() {
        for &regime in &[
            Regime::DistrictOfColumbia,
            Regime::Massachusetts,
            Regime::NewJersey,
            Regime::NewYork,
            Regime::MarylandMontgomery,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("tenant_relocation_assistance")
                        && n.contains("foreclosure_tenant_rights")
                        && n.contains("OWNER-INITIATED")),
                "{:?}: sibling-module note must be present",
                regime,
            );
        }
    }
}
