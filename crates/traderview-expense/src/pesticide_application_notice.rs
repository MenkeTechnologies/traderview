//! State landlord pesticide-application notice compliance check.
//!
//! Operational concern for any landlord using a pest-control
//! company or self-applying pesticides in a rental unit. Federal
//! FIFRA imposes labeling and applicator-licensing requirements but
//! does NOT require advance notice to occupants — that is purely
//! state law. The states diverge sharply: California requires
//! 24-hour written advance notice to the tenant (plus adjacent
//! units for broadcast/fogger/aerosol applications); Massachusetts
//! requires 48-hour posting at building entrances; Oregon requires
//! 24-hour warning-sign posting; New Jersey requires on-application
//! label-information delivery by the applicator; New York has NO
//! statutory pre-notice requirement (only on-request delivery for
//! multi-dwelling buildings via the owner).
//!
//! Six regimes:
//!
//!   - **California** — Cal. Civ. Code § 1940.8.5 + 14 CCR § 6740.
//!     At least 24 hours BEFORE application, landlord or authorized
//!     agent must provide written notice to the tenant of the
//!     treated dwelling. For BROADCAST applications, total release
//!     foggers, or aerosol sprays, notice must also go to any
//!     tenant in an adjacent dwelling unit that could reasonably be
//!     impacted. Notice must include the pest, the pesticide product
//!     name, manufacturer, EPA registration number, area treated,
//!     date/time, and applicator or licensee name.
//!
//!   - **NewJersey** — N.J.A.C. § 7:30-9.12. Pesticide applicator
//!     must provide written information AT APPLICATION (not in
//!     advance) — label safety instructions, applicator name +
//!     address + phone, National Pesticide Information Center
//!     phone, NJDEP Pesticide Control Program phone. For multi-
//!     family buildings, applicator provides to owner; owner
//!     distributes to occupants on request.
//!
//!   - **NewYork** — N.Y. ECL § 33-1004 (Pesticide Reporting Law /
//!     Neighbor Notification Law). For one- or two-family
//!     dwellings, applicator provides written label information
//!     AT TIME OF APPLICATION (no statutory pre-notice). For
//!     multiple dwellings or nonresidential buildings, applicator
//!     provides label information to the owner/agent, who must
//!     then provide it to occupants ON REQUEST. No statewide
//!     advance-notice mandate for residential rentals.
//!
//!   - **Massachusetts** — Mass. G.L. c. 132B § 9 + 333 CMR 13.04.
//!     For non-emergency pesticide applications in residential
//!     rental buildings, posting required at building entrances and
//!     written notice to occupants at least 48 hours in advance.
//!     Integrated pest management plans required for schools and
//!     day care under § 132B (carve-out outside rental scope).
//!
//!   - **Oregon** — ORS § 634.740 + § 634.700. Warning signs must
//!     be posted around pesticide application areas at least 24
//!     hours before the application begins and remain at least 72
//!     hours after. Signs must read "Warning: pesticide-treated
//!     area," include the expected or actual date/time of
//!     application, and provide a contact person's telephone
//!     number.
//!
//!   - **Default** — federal FIFRA labeling and applicator-
//!     licensing requirements apply; no statewide advance-notice
//!     mandate.
//!
//! Citations: Cal. Civ. Code § 1940.8.5 (24-hour written notice);
//! 14 CCR § 6740 (CA pesticide regulation); N.J.A.C. § 7:30-9.12
//! (NJ structural pest control notice); N.Y. ECL § 33-1004 (NY
//! Pesticide Reporting Law); Mass. G.L. c. 132B § 9 + 333 CMR
//! 13.04 (MA notice + IPM); ORS § 634.740 (OR warning-sign
//! posting); 7 U.S.C. § 136 et seq. (FIFRA labeling federal floor).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewJersey,
    NewYork,
    Massachusetts,
    Oregon,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationType {
    /// Targeted (spot) application — typical structural pest
    /// control. CA notice goes to applied unit only.
    Targeted,
    /// Broadcast application — sprayed over a wide area.
    Broadcast,
    /// Total-release fogger ("bug bomb").
    TotalReleaseFogger,
    /// Aerosol spray.
    AerosolSpray,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BuildingType {
    /// Single-family dwelling (NY treats specially).
    SingleFamily,
    /// Multi-family residential building.
    MultiFamily,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub application_type: ApplicationType,
    pub building_type: BuildingType,
    /// Hours of advance notice given to the tenant of the treated
    /// dwelling unit. Zero if notice given only at the time of
    /// application.
    pub hours_advance_notice_to_treated_tenant: u32,
    /// Hours of advance notice given to adjacent tenants (CA
    /// broadcast/fogger/aerosol requirement).
    pub hours_advance_notice_to_adjacent_tenants: u32,
    /// Hours of advance posting at building entrances (MA + OR
    /// requirements).
    pub hours_advance_posting_at_building: u32,
    /// True if notice contains pesticide product name +
    /// manufacturer + EPA registration number (CA requires;
    /// FIFRA-derived label info elsewhere).
    pub notice_contains_pesticide_product_info: bool,
    /// True if notice contains applicator/licensee name + phone
    /// (CA + NJ).
    pub notice_contains_applicator_info: bool,
    /// True if notice contains National Pesticide Information
    /// Center (NPIC) phone (NJ-specific).
    pub notice_contains_npic_phone: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    pub compliant: bool,
    /// Required hours of advance notice to the tenant of the
    /// treated dwelling (zero where the regime imposes no
    /// pre-notice mandate).
    pub required_advance_notice_hours: u32,
    /// True if the regime requires notice to ADJACENT-unit tenants
    /// for the application type at issue (CA broadcast / fogger /
    /// aerosol only).
    pub adjacent_notice_required: bool,
    /// True if the regime requires posting at building entrances
    /// (MA + OR signal this).
    pub building_posting_required: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// California 14 CCR § 6740 advance-notice window.
pub const CA_ADVANCE_NOTICE_HOURS: u32 = 24;
/// Massachusetts 333 CMR 13.04 advance-notice window.
pub const MA_ADVANCE_NOTICE_HOURS: u32 = 48;
/// Oregon ORS § 634.740 sign-posting advance window.
pub const OR_ADVANCE_POSTING_HOURS: u32 = 24;

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let (
        required_advance_notice_hours,
        adjacent_notice_required,
        building_posting_required,
        citation,
    ) = match input.regime {
        Regime::California => {
            // CA § 1940.8.5 — 24-hour advance written notice to
            // treated unit; adjacent units also if broadcast,
            // fogger, or aerosol.
            let adjacent = !matches!(input.application_type, ApplicationType::Targeted);
            (
                CA_ADVANCE_NOTICE_HOURS,
                adjacent,
                false,
                "Cal. Civ. Code § 1940.8.5 (24-hour advance written notice to treated tenant; \
                 adjacent units for broadcast / total-release fogger / aerosol spray); 14 CCR \
                 § 6740 (notice content — pesticide product + manufacturer + EPA registration \
                 + area + date/time + applicator)",
            )
        }
        Regime::NewJersey => (
            0,
            false,
            false,
            "N.J.A.C. § 7:30-9.12 (structural pest control — applicator provides label safety \
             info + applicator name/address/phone + National Pesticide Information Center phone \
             + NJDEP Pesticide Control Program phone AT TIME OF APPLICATION; multi-family \
             building owner distributes on request)",
        ),
        Regime::NewYork => (
            0,
            false,
            false,
            "N.Y. ECL § 33-1004 (Pesticide Reporting Law / Neighbor Notification Law — for \
             one- or two-family dwellings applicator provides label info AT TIME OF \
             APPLICATION; for multiple dwellings applicator provides to owner who provides to \
             occupants ON REQUEST; no statewide advance-notice mandate for residential rentals)",
        ),
        Regime::Massachusetts => (
            MA_ADVANCE_NOTICE_HOURS,
            false,
            true,
            "Mass. G.L. c. 132B § 9 (integrated pest management framework); 333 CMR 13.04 \
             (residential rental application — 48-hour advance written notice to occupants + \
             posting at building entrances)",
        ),
        Regime::Oregon => (
            0,
            false,
            true,
            "ORS § 634.740 (warning-sign posting — beginning at least 24 hours before \
             application and ending no earlier than 72 hours after; sign reads \"Warning: \
             pesticide-treated area,\" includes date/time and contact phone); ORS § 634.700 \
             (definitions)",
        ),
        Regime::Default => (
            0,
            false,
            false,
            "FIFRA — 7 U.S.C. § 136 et seq. (federal labeling + applicator licensing; no \
             statewide advance-notice mandate)",
        ),
    };

    // Advance-notice compliance to treated tenant.
    if required_advance_notice_hours > 0
        && input.hours_advance_notice_to_treated_tenant < required_advance_notice_hours
    {
        violations.push(format!(
            "Advance notice to treated tenant was {} hours; regime requires {} hours.",
            input.hours_advance_notice_to_treated_tenant, required_advance_notice_hours,
        ));
    }

    // CA adjacent-unit notice for broadcast / fogger / aerosol.
    if adjacent_notice_required
        && input.hours_advance_notice_to_adjacent_tenants < required_advance_notice_hours
    {
        violations.push(format!(
            "Adjacent-unit advance notice was {} hours; California § 1940.8.5 requires {} \
             hours for {:?} application.",
            input.hours_advance_notice_to_adjacent_tenants,
            required_advance_notice_hours,
            input.application_type,
        ));
    }

    // Building posting (MA + OR).
    if building_posting_required {
        let required_posting = match input.regime {
            Regime::Massachusetts => MA_ADVANCE_NOTICE_HOURS,
            Regime::Oregon => OR_ADVANCE_POSTING_HOURS,
            _ => 0,
        };
        if input.hours_advance_posting_at_building < required_posting {
            violations.push(format!(
                "Building-entrance posting was {} hours; regime requires {} hours.",
                input.hours_advance_posting_at_building, required_posting,
            ));
        }
    }

    // California content requirements.
    if matches!(input.regime, Regime::California) {
        if !input.notice_contains_pesticide_product_info {
            violations.push(
                "California 14 CCR § 6740: notice must include pesticide product name + \
                 manufacturer + EPA registration number; missing."
                    .to_string(),
            );
        }
        if !input.notice_contains_applicator_info {
            violations.push(
                "California 14 CCR § 6740: notice must include applicator/licensee name; \
                 missing."
                    .to_string(),
            );
        }
    }

    // New Jersey content requirements.
    if matches!(input.regime, Regime::NewJersey) {
        if !input.notice_contains_applicator_info {
            violations.push(
                "N.J.A.C. § 7:30-9.12: applicator must provide name + address + phone; \
                 missing."
                    .to_string(),
            );
        }
        if !input.notice_contains_npic_phone {
            violations.push(
                "N.J.A.C. § 7:30-9.12: notice must include National Pesticide Information \
                 Center phone; missing."
                    .to_string(),
            );
        }
    }

    // NY multi-dwelling regime note.
    if matches!(input.regime, Regime::NewYork)
        && matches!(input.building_type, BuildingType::MultiFamily)
    {
        notes.push(
            "N.Y. multi-dwelling: applicator provides label info to owner; owner must provide \
             to occupants ON REQUEST. No automatic pre-notice obligation."
                .to_string(),
        );
    }

    notes.push(
        "Federal FIFRA (7 U.S.C. § 136 et seq.) provides labeling + applicator-licensing \
         floor; state law layers advance-notice mandates on top where applicable."
            .to_string(),
    );

    CheckResult {
        compliant: violations.is_empty(),
        required_advance_notice_hours,
        adjacent_notice_required,
        building_posting_required,
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
            application_type: ApplicationType::Targeted,
            building_type: BuildingType::MultiFamily,
            hours_advance_notice_to_treated_tenant: 24,
            hours_advance_notice_to_adjacent_tenants: 24,
            hours_advance_posting_at_building: 48,
            notice_contains_pesticide_product_info: true,
            notice_contains_applicator_info: true,
            notice_contains_npic_phone: true,
        }
    }

    // ── California § 1940.8.5 + 14 CCR § 6740 ───────────────────

    #[test]
    fn california_targeted_24h_compliant() {
        let r = check(&base(Regime::California));
        assert!(r.compliant);
        assert_eq!(r.required_advance_notice_hours, 24);
        assert!(!r.adjacent_notice_required);
        assert!(r.citation.contains("§ 1940.8.5"));
        assert!(r.citation.contains("14 CCR § 6740"));
    }

    #[test]
    fn california_broadcast_requires_adjacent_unit_notice() {
        let mut i = base(Regime::California);
        i.application_type = ApplicationType::Broadcast;
        let r = check(&i);
        assert!(r.adjacent_notice_required);
        assert!(r.compliant);
    }

    #[test]
    fn california_fogger_requires_adjacent_unit_notice() {
        let mut i = base(Regime::California);
        i.application_type = ApplicationType::TotalReleaseFogger;
        let r = check(&i);
        assert!(r.adjacent_notice_required);
    }

    #[test]
    fn california_aerosol_requires_adjacent_unit_notice() {
        let mut i = base(Regime::California);
        i.application_type = ApplicationType::AerosolSpray;
        let r = check(&i);
        assert!(r.adjacent_notice_required);
    }

    #[test]
    fn california_under_24h_violation() {
        let mut i = base(Regime::California);
        i.hours_advance_notice_to_treated_tenant = 23;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("23") && v.contains("24")));
    }

    #[test]
    fn california_at_24h_boundary_compliant() {
        let mut i = base(Regime::California);
        i.hours_advance_notice_to_treated_tenant = 24;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn california_broadcast_missing_adjacent_notice_violation() {
        let mut i = base(Regime::California);
        i.application_type = ApplicationType::Broadcast;
        i.hours_advance_notice_to_adjacent_tenants = 0;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Adjacent-unit") && v.contains("Broadcast")));
    }

    #[test]
    fn california_missing_product_info_violation() {
        let mut i = base(Regime::California);
        i.notice_contains_pesticide_product_info = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("14 CCR § 6740") && v.contains("pesticide product")));
    }

    #[test]
    fn california_missing_applicator_info_violation() {
        let mut i = base(Regime::California);
        i.notice_contains_applicator_info = false;
        let r = check(&i);
        assert!(!r.compliant);
    }

    // ── New Jersey § 7:30-9.12 ──────────────────────────────────

    #[test]
    fn new_jersey_no_advance_notice_compliant() {
        let mut i = base(Regime::NewJersey);
        i.hours_advance_notice_to_treated_tenant = 0;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.required_advance_notice_hours, 0);
        assert!(r.citation.contains("§ 7:30-9.12"));
    }

    #[test]
    fn new_jersey_missing_npic_phone_violation() {
        let mut i = base(Regime::NewJersey);
        i.notice_contains_npic_phone = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 7:30-9.12") && v.contains("National Pesticide")));
    }

    #[test]
    fn new_jersey_missing_applicator_info_violation() {
        let mut i = base(Regime::NewJersey);
        i.notice_contains_applicator_info = false;
        let r = check(&i);
        assert!(!r.compliant);
    }

    // ── New York § 33-1004 ──────────────────────────────────────

    #[test]
    fn new_york_no_statutory_advance_notice_compliant() {
        let mut i = base(Regime::NewYork);
        i.hours_advance_notice_to_treated_tenant = 0;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.required_advance_notice_hours, 0);
        assert!(r.citation.contains("§ 33-1004"));
    }

    #[test]
    fn new_york_multi_dwelling_note_present() {
        let mut i = base(Regime::NewYork);
        i.building_type = BuildingType::MultiFamily;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("multi-dwelling") && n.contains("ON REQUEST")));
    }

    #[test]
    fn new_york_single_family_no_multi_dwelling_note() {
        let mut i = base(Regime::NewYork);
        i.building_type = BuildingType::SingleFamily;
        let r = check(&i);
        assert!(!r.notes.iter().any(|n| n.contains("multi-dwelling")));
    }

    // ── Massachusetts c. 132B § 9 + 333 CMR 13.04 ────────────────

    #[test]
    fn massachusetts_48h_compliant() {
        let mut i = base(Regime::Massachusetts);
        i.hours_advance_notice_to_treated_tenant = 48;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.required_advance_notice_hours, 48);
        assert!(r.building_posting_required);
        assert!(r.citation.contains("c. 132B § 9"));
        assert!(r.citation.contains("333 CMR 13.04"));
    }

    #[test]
    fn massachusetts_under_48h_violation() {
        let mut i = base(Regime::Massachusetts);
        i.hours_advance_notice_to_treated_tenant = 47;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn massachusetts_at_48h_boundary_compliant() {
        let mut i = base(Regime::Massachusetts);
        i.hours_advance_notice_to_treated_tenant = 48;
        i.hours_advance_posting_at_building = 48;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn massachusetts_missing_building_posting_violation() {
        let mut i = base(Regime::Massachusetts);
        i.hours_advance_posting_at_building = 0;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("posting") && v.contains("48")));
    }

    // ── Oregon § 634.740 ────────────────────────────────────────

    #[test]
    fn oregon_24h_posting_compliant() {
        let mut i = base(Regime::Oregon);
        i.hours_advance_posting_at_building = 24;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.building_posting_required);
        assert!(r.citation.contains("§ 634.740"));
    }

    #[test]
    fn oregon_under_24h_posting_violation() {
        let mut i = base(Regime::Oregon);
        i.hours_advance_posting_at_building = 23;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn oregon_no_advance_notice_to_tenant_required() {
        // OR requires SIGN POSTING but not direct tenant notice.
        let mut i = base(Regime::Oregon);
        i.hours_advance_notice_to_treated_tenant = 0;
        i.hours_advance_posting_at_building = 24;
        let r = check(&i);
        assert_eq!(r.required_advance_notice_hours, 0);
        assert!(r.compliant);
    }

    // ── Default — federal FIFRA only ───────────────────────────

    #[test]
    fn default_federal_fifra_only() {
        let mut i = base(Regime::Default);
        i.hours_advance_notice_to_treated_tenant = 0;
        i.hours_advance_posting_at_building = 0;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.required_advance_notice_hours, 0);
        assert!(r.citation.contains("FIFRA"));
    }

    // ── Regression-critical multi-regime invariants ────────────

    #[test]
    fn only_california_requires_adjacent_unit_notice_invariant() {
        // CA broadcast triggers adjacent_notice_required; no other
        // regime does, even for the same application type.
        let mut ca = base(Regime::California);
        ca.application_type = ApplicationType::Broadcast;
        assert!(check(&ca).adjacent_notice_required);

        for &regime in &[
            Regime::NewJersey,
            Regime::NewYork,
            Regime::Massachusetts,
            Regime::Oregon,
            Regime::Default,
        ] {
            let mut i = base(regime);
            i.application_type = ApplicationType::Broadcast;
            assert!(
                !check(&i).adjacent_notice_required,
                "{:?}: must NOT require adjacent-unit notice",
                regime,
            );
        }
    }

    #[test]
    fn only_ma_and_or_require_building_posting_invariant() {
        assert!(check(&base(Regime::Massachusetts)).building_posting_required);
        assert!(check(&base(Regime::Oregon)).building_posting_required);
        for &regime in &[
            Regime::California,
            Regime::NewJersey,
            Regime::NewYork,
            Regime::Default,
        ] {
            assert!(
                !check(&base(regime)).building_posting_required,
                "{:?}: must NOT require building posting",
                regime,
            );
        }
    }

    #[test]
    fn only_ca_and_ma_have_advance_notice_to_tenant_invariant() {
        assert_eq!(
            check(&base(Regime::California)).required_advance_notice_hours,
            24,
        );
        assert_eq!(
            check(&base(Regime::Massachusetts)).required_advance_notice_hours,
            48,
        );
        for &regime in &[
            Regime::NewJersey,
            Regime::NewYork,
            Regime::Oregon,
            Regime::Default,
        ] {
            assert_eq!(
                check(&base(regime)).required_advance_notice_hours,
                0,
                "{:?}: must NOT require tenant advance notice",
                regime,
            );
        }
    }

    #[test]
    fn only_nj_requires_npic_phone_invariant() {
        // Drop NPIC phone — only NJ should violate.
        let mut nj = base(Regime::NewJersey);
        nj.notice_contains_npic_phone = false;
        assert!(!check(&nj).compliant);

        for &regime in &[
            Regime::California,
            Regime::NewYork,
            Regime::Massachusetts,
            Regime::Oregon,
            Regime::Default,
        ] {
            let mut i = base(regime);
            i.notice_contains_npic_phone = false;
            // Need to set MA-specific posting/notice to compliant
            // baseline; targeted CA without product info still ok.
            // For each, check that NPIC absence alone doesn't
            // trigger a violation.
            let r = check(&i);
            assert!(
                !r.violations
                    .iter()
                    .any(|v| v.contains("National Pesticide Information Center")),
                "{:?}: must NOT cite NPIC phone",
                regime,
            );
        }
    }

    #[test]
    fn massachusetts_has_strictest_advance_notice_window() {
        let ma = check(&base(Regime::Massachusetts)).required_advance_notice_hours;
        let ca = check(&base(Regime::California)).required_advance_notice_hours;
        let or_required = check(&base(Regime::Oregon)).required_advance_notice_hours;
        let nj_required = check(&base(Regime::NewJersey)).required_advance_notice_hours;
        let ny_required = check(&base(Regime::NewYork)).required_advance_notice_hours;
        assert!(ma > ca);
        assert!(ma > or_required);
        assert!(ma > nj_required);
        assert!(ma > ny_required);
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(check(&base(Regime::California))
            .citation
            .contains("§ 1940.8.5"));
        assert!(check(&base(Regime::NewJersey))
            .citation
            .contains("§ 7:30-9.12"));
        assert!(check(&base(Regime::NewYork)).citation.contains("§ 33-1004"));
        assert!(check(&base(Regime::Massachusetts))
            .citation
            .contains("c. 132B § 9"));
        assert!(check(&base(Regime::Oregon)).citation.contains("§ 634.740"));
        assert!(check(&base(Regime::Default)).citation.contains("FIFRA"));
    }

    #[test]
    fn fifra_floor_note_present_across_all_regimes() {
        for &regime in &[
            Regime::California,
            Regime::NewJersey,
            Regime::NewYork,
            Regime::Massachusetts,
            Regime::Oregon,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("FIFRA") && n.contains("labeling")),
                "{:?}: FIFRA federal-floor note must be present",
                regime,
            );
        }
    }
}
