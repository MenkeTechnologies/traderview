//! Rental property organic waste collection disclosure
//! compliance — when must a trader-landlord provide organics
//! collection bins, tenant education, and move-in disclosure
//! before billing for waste service or claiming compliance
//! with state organic waste diversion mandates? Trader-
//! landlord operational concern: SB 1383-covered multifamily
//! owners face per-incident contamination fines + per-tenant
//! education-failure penalties + state-jurisdiction
//! enforcement. Distinct from siblings `rental_energy_
//! benchmarking` (energy/GHG), `rental_water_submetering_
//! disclosure` (water billing), `rental_gas_appliance_ban`
//! (electrification).
//!
//! **Four regimes**:
//!
//! **California — SB 1383 of 2016 (Short-Lived Climate
//! Pollutant Reduction Strategy, eff. January 1, 2022; 14
//! CCR §§ 18984-18984.13)**:
//! - Multifamily properties with **5+ units** must provide
//!   organics collection containers for tenant food waste +
//!   yard debris.
//! - **Annual tenant education** on organic waste sorting
//!   required.
//! - **New tenant information within 14 days** of occupancy.
//! - Move-out notification on organic waste rules.
//! - Containers must be properly labeled/color-coded.
//! - Local jurisdictions (CalRecycle) impose per-incident
//!   contamination fines.
//! - 75% organic waste diversion target by 2025; 20%
//!   surplus food edible-recovery target.
//!
//! **Vermont — Universal Recycling Law (Act 148 of 2012,
//! fully effective July 1, 2020; 10 V.S.A. § 6605k)**:
//! - **Bans food scraps from landfill statewide**.
//! - Landlords must provide opportunity for tenants to
//!   recycle/compost food waste.
//! - Civil penalties **$200 to $1,000** for first offense
//!   under 10 V.S.A. § 8007.
//! - Most stringent statewide ban among comparators.
//!
//! **Seattle — Seattle Municipal Code Ch. 21.36.082 (eff.
//! January 1, 2015)**:
//! - Multifamily **5+ units** required to participate in
//!   compost service.
//! - **$50 contamination fine per pickup** when
//!   recyclables or food waste contaminate landfill bin.
//! - 1% maximum contamination threshold.
//!
//! **Default — common-law warranty of habitability + state
//! solid waste regulations**:
//! - No specific statewide organic waste mandate.
//! - Local municipal ordinances may impose composting
//!   requirements (e.g., NYC LL97 organic waste rules,
//!   Boston organic waste pilot, Portland composting
//!   requirements).
//! - Federal RCRA Subtitle D solid waste regulations apply.
//!
//! Citations: Cal. SB 1383 of 2016; 14 CCR §§ 18984-
//! 18984.13; Vermont Act 148 of 2012 (10 V.S.A. § 6605k);
//! 10 V.S.A. § 8007; Seattle Municipal Code Ch. 21.36.082;
//! CalRecycle SB 1383 Regulations; RCRA Subtitle D (42 USC
//! § 6941 et seq.).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Vermont,
    Seattle,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalOrganicWasteCollectionDisclosureInput {
    pub regime: Regime,
    /// Number of dwelling units in the property.
    pub unit_count: u32,
    /// Whether organic waste collection containers are
    /// provided to tenants (CA SB 1383 + Seattle).
    pub organics_bins_provided: bool,
    /// Whether containers are properly labeled/color-coded.
    pub containers_properly_labeled: bool,
    /// Whether annual tenant education was provided (CA SB
    /// 1383 + Seattle).
    pub annual_tenant_education_provided: bool,
    /// Days from new tenant occupancy to organic waste
    /// information delivery (CA SB 1383 14-day deadline).
    pub days_to_new_tenant_information: u32,
    /// Whether opportunity to recycle/compost is provided to
    /// tenants (Vermont Act 148).
    pub recycling_compost_opportunity_provided: bool,
    /// Vermont per-violation civil penalty assessed in cents.
    pub vt_civil_penalty_cents: u64,
    /// Number of Seattle contamination incidents (each
    /// triggers $50 fine).
    pub seattle_contamination_incidents: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalOrganicWasteCollectionDisclosureResult {
    pub disclosure_compliant: bool,
    pub bin_provision_required: bool,
    pub annual_education_required: bool,
    pub fourteen_day_new_tenant_window_compliant: bool,
    pub vt_penalty_in_statutory_range: bool,
    pub seattle_contamination_fine_cents: u64,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &RentalOrganicWasteCollectionDisclosureInput,
) -> RentalOrganicWasteCollectionDisclosureResult {
    match input.regime {
        Regime::California => check_ca(input),
        Regime::Vermont => check_vt(input),
        Regime::Seattle => check_seattle(input),
        Regime::Default => check_default(input),
    }
}

fn check_ca(
    input: &RentalOrganicWasteCollectionDisclosureInput,
) -> RentalOrganicWasteCollectionDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. SB 1383 of 2016 (Short-Lived Climate Pollutant Reduction Strategy, eff. January 1, 2022) — multifamily properties with 5+ units must provide organics collection containers".to_string(),
        "14 CCR §§ 18984-18984.13 — containers must be properly labeled/color-coded; CalRecycle imposes per-incident contamination fines via local jurisdictions".to_string(),
        "Cal. SB 1383 — annual tenant education on organic waste sorting required + new tenant information within 14 days of occupancy".to_string(),
        "Cal. SB 1383 75% organic waste diversion target by 2025; 20% surplus food edible-recovery target".to_string(),
        "Cal. SB 1383 — move-out notification on organic waste rules required for departing tenants".to_string(),
    ];

    let in_scope = input.unit_count >= 5;

    if in_scope && !input.organics_bins_provided {
        violations.push(
            "Cal. SB 1383 + 14 CCR § 18984 — multifamily property with 5+ units must provide organics collection containers".to_string(),
        );
    }

    if in_scope && input.organics_bins_provided && !input.containers_properly_labeled {
        violations.push(
            "14 CCR § 18984.7 — organics containers must be properly labeled and color-coded (green for organics)".to_string(),
        );
    }

    if in_scope && !input.annual_tenant_education_provided {
        violations.push(
            "Cal. SB 1383 — annual tenant education on organic waste sorting required".to_string(),
        );
    }

    let fourteen_day_compliant = !in_scope || input.days_to_new_tenant_information <= 14;
    if in_scope && !fourteen_day_compliant {
        violations.push(
            "Cal. SB 1383 — new tenant organic waste information must be provided within 14 days of occupancy".to_string(),
        );
    }

    RentalOrganicWasteCollectionDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        bin_provision_required: in_scope,
        annual_education_required: in_scope,
        fourteen_day_new_tenant_window_compliant: fourteen_day_compliant,
        vt_penalty_in_statutory_range: true,
        seattle_contamination_fine_cents: 0,
        violations,
        citation: "Cal. SB 1383 of 2016; 14 CCR §§ 18984-18984.13; CalRecycle Regulations",
        notes,
    }
}

fn check_vt(
    input: &RentalOrganicWasteCollectionDisclosureInput,
) -> RentalOrganicWasteCollectionDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Vermont Universal Recycling Law (Act 148 of 2012, fully eff. July 1, 2020) — bans food scraps from landfill statewide".to_string(),
        "10 V.S.A. § 6605k — landlords must provide opportunity for tenants to recycle/compost food waste".to_string(),
        "10 V.S.A. § 8007 — civil penalties $200 to $1,000 for first offense".to_string(),
        "Vermont most stringent statewide ban among comparators (unlike CA SB 1383 which is unit-size-gated)".to_string(),
        "Vermont applies to ALL property regardless of unit count (no 5+ unit threshold like CA + Seattle)".to_string(),
    ];

    if !input.recycling_compost_opportunity_provided {
        violations.push(
            "10 V.S.A. § 6605k — landlord must provide opportunity for tenants to recycle/compost food waste; food scraps banned from landfill statewide".to_string(),
        );
    }

    let penalty_in_range = input.vt_civil_penalty_cents == 0
        || (input.vt_civil_penalty_cents >= 20_000
            && input.vt_civil_penalty_cents <= 100_000);
    if !penalty_in_range {
        violations.push(
            "10 V.S.A. § 8007 — civil penalty must be between $200 and $1,000 for first offense".to_string(),
        );
    }

    RentalOrganicWasteCollectionDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        bin_provision_required: true,
        annual_education_required: false,
        fourteen_day_new_tenant_window_compliant: true,
        vt_penalty_in_statutory_range: penalty_in_range,
        seattle_contamination_fine_cents: 0,
        violations,
        citation: "Vermont Act 148 of 2012; 10 V.S.A. § 6605k + § 8007",
        notes,
    }
}

fn check_seattle(
    input: &RentalOrganicWasteCollectionDisclosureInput,
) -> RentalOrganicWasteCollectionDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Seattle Municipal Code Ch. 21.36.082 (eff. January 1, 2015) — multifamily 5+ units required to participate in compost service".to_string(),
        "SMC 21.36.082 — $50 contamination fine per pickup when recyclables or food waste contaminate landfill bin".to_string(),
        "Seattle 1% maximum contamination threshold; fines compound per incident".to_string(),
        "Seattle parallels CA 5+ unit threshold but adds per-pickup contamination fine mechanism".to_string(),
        "Seattle Solid Waste Director enforces via Seattle Public Utilities".to_string(),
    ];

    let in_scope = input.unit_count >= 5;

    if in_scope && !input.organics_bins_provided {
        violations.push(
            "SMC 21.36.082 — multifamily 5+ units must participate in compost service".to_string(),
        );
    }

    const SEATTLE_FINE_PER_INCIDENT_CENTS: u64 = 5_000;
    let seattle_fine = SEATTLE_FINE_PER_INCIDENT_CENTS
        .saturating_mul(input.seattle_contamination_incidents as u64);

    RentalOrganicWasteCollectionDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        bin_provision_required: in_scope,
        annual_education_required: false,
        fourteen_day_new_tenant_window_compliant: true,
        vt_penalty_in_statutory_range: true,
        seattle_contamination_fine_cents: seattle_fine,
        violations,
        citation: "Seattle Municipal Code Ch. 21.36.082",
        notes,
    }
}

fn check_default(
    input: &RentalOrganicWasteCollectionDisclosureInput,
) -> RentalOrganicWasteCollectionDisclosureResult {
    let violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Default — no specific statewide organic waste mandate".to_string(),
        "Default — local municipal ordinances may impose composting requirements (NYC LL97 organic waste rules, Boston organic waste pilot, Portland composting requirements)".to_string(),
        "Default — federal RCRA Subtitle D (42 USC § 6941 et seq.) solid waste regulations apply".to_string(),
        "Default — common-law warranty of habitability implies basic waste collection but not composting".to_string(),
        "Default — verify local jurisdiction municipal solid waste ordinance for specific composting + organic waste mandate".to_string(),
    ];

    let _ = input;

    RentalOrganicWasteCollectionDisclosureResult {
        disclosure_compliant: true,
        bin_provision_required: false,
        annual_education_required: false,
        fourteen_day_new_tenant_window_compliant: true,
        vt_penalty_in_statutory_range: true,
        seattle_contamination_fine_cents: 0,
        violations,
        citation: "Default state solid waste regulations + RCRA Subtitle D (42 USC § 6941); verify local jurisdiction",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_clean() -> RentalOrganicWasteCollectionDisclosureInput {
        RentalOrganicWasteCollectionDisclosureInput {
            regime: Regime::California,
            unit_count: 10,
            organics_bins_provided: true,
            containers_properly_labeled: true,
            annual_tenant_education_provided: true,
            days_to_new_tenant_information: 7,
            recycling_compost_opportunity_provided: true,
            vt_civil_penalty_cents: 0,
            seattle_contamination_incidents: 0,
        }
    }

    fn vt_clean() -> RentalOrganicWasteCollectionDisclosureInput {
        let mut i = ca_clean();
        i.regime = Regime::Vermont;
        i
    }

    fn seattle_clean() -> RentalOrganicWasteCollectionDisclosureInput {
        let mut i = ca_clean();
        i.regime = Regime::Seattle;
        i
    }

    fn default_clean() -> RentalOrganicWasteCollectionDisclosureInput {
        let mut i = ca_clean();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_5_unit_in_scope_compliant() {
        let mut i = ca_clean();
        i.unit_count = 5;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(r.bin_provision_required);
    }

    #[test]
    fn ca_4_unit_out_of_scope() {
        let mut i = ca_clean();
        i.unit_count = 4;
        i.organics_bins_provided = false;
        i.annual_tenant_education_provided = false;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(!r.bin_provision_required);
    }

    #[test]
    fn ca_no_bins_violation() {
        let mut i = ca_clean();
        i.organics_bins_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("SB 1383") && v.contains("organics collection containers")));
    }

    #[test]
    fn ca_unlabeled_containers_violation() {
        let mut i = ca_clean();
        i.containers_properly_labeled = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 18984.7") && v.contains("color-coded")));
    }

    #[test]
    fn ca_no_annual_education_violation() {
        let mut i = ca_clean();
        i.annual_tenant_education_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("annual tenant education")));
    }

    #[test]
    fn ca_14_day_new_tenant_boundary_compliant() {
        let mut i = ca_clean();
        i.days_to_new_tenant_information = 14;
        let r = check(&i);
        assert!(r.fourteen_day_new_tenant_window_compliant);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn ca_15_day_new_tenant_violation() {
        let mut i = ca_clean();
        i.days_to_new_tenant_information = 15;
        let r = check(&i);
        assert!(!r.fourteen_day_new_tenant_window_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("14 days")));
    }

    #[test]
    fn vt_clean_compliant() {
        let r = check(&vt_clean());
        assert!(r.disclosure_compliant);
        assert!(r.bin_provision_required);
    }

    #[test]
    fn vt_no_recycling_opportunity_violation() {
        let mut i = vt_clean();
        i.recycling_compost_opportunity_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6605k") && v.contains("banned from landfill")));
    }

    #[test]
    fn vt_penalty_at_minimum_in_range() {
        let mut i = vt_clean();
        i.vt_civil_penalty_cents = 20_000;
        let r = check(&i);
        assert!(r.vt_penalty_in_statutory_range);
    }

    #[test]
    fn vt_penalty_at_maximum_in_range() {
        let mut i = vt_clean();
        i.vt_civil_penalty_cents = 100_000;
        let r = check(&i);
        assert!(r.vt_penalty_in_statutory_range);
    }

    #[test]
    fn vt_penalty_below_minimum_violation() {
        let mut i = vt_clean();
        i.vt_civil_penalty_cents = 19_999;
        let r = check(&i);
        assert!(!r.vt_penalty_in_statutory_range);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 8007") && v.contains("$200")));
    }

    #[test]
    fn vt_penalty_above_maximum_violation() {
        let mut i = vt_clean();
        i.vt_civil_penalty_cents = 100_001;
        let r = check(&i);
        assert!(!r.vt_penalty_in_statutory_range);
    }

    #[test]
    fn vt_applies_to_all_properties_regardless_of_unit_count() {
        let mut i = vt_clean();
        i.unit_count = 1;
        i.recycling_compost_opportunity_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
    }

    #[test]
    fn seattle_5_unit_in_scope_compliant() {
        let r = check(&seattle_clean());
        assert!(r.disclosure_compliant);
        assert!(r.bin_provision_required);
    }

    #[test]
    fn seattle_no_bins_violation() {
        let mut i = seattle_clean();
        i.organics_bins_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("SMC 21.36.082")));
    }

    #[test]
    fn seattle_contamination_fine_per_incident_50_dollars() {
        let mut i = seattle_clean();
        i.seattle_contamination_incidents = 1;
        let r = check(&i);
        assert_eq!(r.seattle_contamination_fine_cents, 5_000);
    }

    #[test]
    fn seattle_contamination_fine_multiple_incidents() {
        let mut i = seattle_clean();
        i.seattle_contamination_incidents = 5;
        let r = check(&i);
        assert_eq!(r.seattle_contamination_fine_cents, 25_000);
    }

    #[test]
    fn seattle_defensive_contamination_overflow_saturates() {
        let mut i = seattle_clean();
        i.seattle_contamination_incidents = u32::MAX;
        let r = check(&i);
        let expected = 5_000_u64.saturating_mul(u32::MAX as u64);
        assert_eq!(r.seattle_contamination_fine_cents, expected);
    }

    #[test]
    fn default_no_violations_regardless_of_inputs() {
        let mut i = default_clean();
        i.organics_bins_provided = false;
        i.annual_tenant_education_provided = false;
        i.days_to_new_tenant_information = 365;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(!r.bin_provision_required);
    }

    #[test]
    fn citation_pins_ca_authority() {
        let r = check(&ca_clean());
        assert!(r.citation.contains("SB 1383"));
        assert!(r.citation.contains("§§ 18984"));
    }

    #[test]
    fn citation_pins_vt_authority() {
        let r = check(&vt_clean());
        assert!(r.citation.contains("Vermont Act 148"));
        assert!(r.citation.contains("§ 6605k"));
        assert!(r.citation.contains("§ 8007"));
    }

    #[test]
    fn citation_pins_seattle_authority() {
        let r = check(&seattle_clean());
        assert!(r.citation.contains("21.36.082"));
    }

    #[test]
    fn citation_pins_default_authority() {
        let r = check(&default_clean());
        assert!(r.citation.contains("RCRA Subtitle D"));
        assert!(r.citation.contains("42 USC § 6941"));
    }

    #[test]
    fn note_pins_ca_january_2022_effective_date() {
        let r = check(&ca_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("January 1, 2022") && n.contains("SB 1383")));
    }

    #[test]
    fn note_pins_ca_75_percent_diversion_target() {
        let r = check(&ca_clean());
        assert!(r.notes.iter().any(|n| n.contains("75%")
            && n.contains("2025")));
    }

    #[test]
    fn note_pins_vt_act_148_july_2020_effective_date() {
        let r = check(&vt_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("July 1, 2020") && n.contains("Act 148")));
    }

    #[test]
    fn note_pins_seattle_50_dollar_contamination_fine() {
        let r = check(&seattle_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("$50 contamination fine")));
    }

    #[test]
    fn note_pins_default_rcra_subtitle_d() {
        let r = check(&default_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("RCRA Subtitle D")));
    }

    #[test]
    fn vt_uniquely_no_unit_threshold_invariant() {
        let mut i_ca = ca_clean();
        i_ca.unit_count = 1;
        i_ca.organics_bins_provided = false;
        let r_ca = check(&i_ca);
        assert!(r_ca.disclosure_compliant);

        let mut i_vt = vt_clean();
        i_vt.unit_count = 1;
        i_vt.recycling_compost_opportunity_provided = false;
        let r_vt = check(&i_vt);
        assert!(!r_vt.disclosure_compliant);
    }

    #[test]
    fn ca_uniquely_requires_annual_education_invariant() {
        let mut i_ca = ca_clean();
        i_ca.annual_tenant_education_provided = false;
        let r_ca = check(&i_ca);
        assert!(!r_ca.disclosure_compliant);

        let mut i_vt = vt_clean();
        i_vt.annual_tenant_education_provided = false;
        let r_vt = check(&i_vt);
        assert!(r_vt.disclosure_compliant);

        let mut i_seattle = seattle_clean();
        i_seattle.annual_tenant_education_provided = false;
        let r_seattle = check(&i_seattle);
        assert!(r_seattle.disclosure_compliant);
    }

    #[test]
    fn seattle_uniquely_per_pickup_fine_invariant() {
        let mut i_ca = ca_clean();
        i_ca.seattle_contamination_incidents = 5;
        let r_ca = check(&i_ca);
        assert_eq!(r_ca.seattle_contamination_fine_cents, 0);

        let mut i_seattle = seattle_clean();
        i_seattle.seattle_contamination_incidents = 5;
        let r_seattle = check(&i_seattle);
        assert!(r_seattle.seattle_contamination_fine_cents > 0);
    }

    #[test]
    fn ca_unit_threshold_truth_table() {
        for (units, exp_in_scope) in [
            (1, false),
            (3, false),
            (4, false),
            (5, true),
            (10, true),
            (100, true),
        ] {
            let mut i = ca_clean();
            i.unit_count = units;
            let r = check(&i);
            assert_eq!(
                r.bin_provision_required, exp_in_scope,
                "units={} expected in_scope={}",
                units, exp_in_scope
            );
        }
    }

    #[test]
    fn multiple_ca_violations_stack() {
        let mut i = ca_clean();
        i.organics_bins_provided = false;
        i.annual_tenant_education_provided = false;
        i.days_to_new_tenant_information = 20;
        let r = check(&i);
        assert_eq!(r.violations.len(), 3);
    }
}
