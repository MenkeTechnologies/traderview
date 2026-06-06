//! State snow / ice removal landlord responsibility compliance check.
//!
//! Snow + ice removal liability is one of the most heavily litigated
//! premises-liability topics in cold-weather states. Three jurisdictions
//! have meaningfully different rules: Massachusetts abolished the
//! natural-vs-unnatural accumulation distinction in Papadopoulos
//! (2010) and imposes a reasonable-care duty; Illinois grants statutory
//! immunity to residential owners who voluntarily clear sidewalks; New
//! York City mandates a 4-hour timeline by municipal ordinance and
//! holds owner liable for tickets even when tenant lease-delegated the
//! task.
//!
//! Massachusetts (Papadopoulos v. Target Corp. 457 Mass. 368 (2010) +
//! 105 CMR 410 State Sanitary Code) — landlord owes a general duty of
//! reasonable care to keep premises reasonably safe; natural-vs-
//! unnatural distinction is gone; the State Sanitary Code requires
//! landlord PRIMARY responsibility for means of egress (exterior
//! doors, walkways, outdoor staircases); landlord cannot delegate to
//! tenant via lease UNLESS tenant has an independent, private entrance
//! that is not shared with anyone else.
//!
//! Illinois (745 ILCS 75/ Snow and Ice Removal Act, enacted 1979) —
//! grants IMMUNITY to residential property owners who voluntarily
//! clear snow / ice from sidewalks. No civil liability for snow / ice
//! removal efforts unless WILLFUL or WANTON. Critical limits: immunity
//! does NOT extend to private property (driveway, walkway to front
//! door, garage entrance) and does NOT excuse failing to correct
//! dangerous conditions that cause accumulation on the public sidewalk.
//!
//! New York City (NYC Admin Code § 16-123) — owner + tenant + lessee
//! of any building lot adjacent to a sidewalk must remove snow + ice
//! within 4 HOURS of when snow stops, EXCEPT during 9 PM–7 AM (then
//! 4 hours after 7 AM). Multi-unit-building landlords fully
//! responsible. Single-family leases MAY delegate to tenant, but the
//! OWNER still receives any city ticket.
//!
//! Default — common-law habitability; varies by state. No statewide
//! statutory liability regime identified.
//!
//! Citations: Papadopoulos v. Target Corp., 457 Mass. 368 (2010);
//! Mass. State Sanitary Code 105 CMR 410.452 (means-of-egress duty);
//! 745 ILCS 75/ Snow and Ice Removal Act; NYC Admin Code § 16-123.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Massachusetts,
    Illinois,
    NewYorkCity,
    Default,
}

impl Regime {
    pub fn for_jurisdiction(state: &str, city: &str) -> Self {
        let st = state.trim().to_ascii_uppercase();
        let ct = city.trim().to_ascii_lowercase();
        match (st.as_str(), ct.as_str()) {
            ("MA", _) => Self::Massachusetts,
            ("IL", _) => Self::Illinois,
            ("NY", "new york") | ("NY", "nyc") => Self::NewYorkCity,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocationType {
    /// Public sidewalk adjacent to the property.
    PublicSidewalk,
    /// Means of egress — exterior doors, walkways, outdoor staircases.
    /// Massachusetts Sanitary Code primary-landlord-duty applies.
    MeansOfEgress,
    /// Private property — driveway, walkway to front door, garage entrance.
    /// Illinois immunity does NOT extend here.
    PrivateProperty,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnowRemovalInput {
    pub regime: Regime,
    pub location: LocationType,
    pub hours_since_snow_stopped: u32,
    pub snow_removed: bool,
    /// Whether the lease purports to delegate snow removal to the tenant.
    pub lease_delegates_to_tenant: bool,
    /// Whether the building has multiple units (i.e., multi-family
    /// rental rather than single-family). NYC: multi-unit landlord
    /// always primarily responsible; MA: relevant to whether tenant
    /// has independent private entrance.
    pub multi_unit_building: bool,
    /// Massachusetts: whether the tenant has an INDEPENDENT, PRIVATE
    /// entrance that is not shared with anyone else (the only path to
    /// valid lease delegation under MA Sanitary Code).
    pub tenant_has_independent_private_entrance: bool,
    /// Illinois: whether the landlord's snow removal effort was
    /// WILLFUL or WANTON (Illinois immunity exception).
    pub willful_or_wanton_conduct: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    NotClearedWithinTimeline,
    InvalidLeaseDelegation,
    /// Illinois: willful or wanton conduct overrides immunity.
    WillfulOrWantonOverrideImmunity,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SnowRemovalResult {
    pub regime: Regime,
    pub landlord_has_duty: bool,
    pub statutory_timeline_hours: u32,
    pub immunity_applies: bool,
    pub delegation_to_tenant_valid: bool,
    pub violation: ViolationType,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &SnowRemovalInput) -> SnowRemovalResult {
    match input.regime {
        Regime::Massachusetts => ma_check(input),
        Regime::Illinois => il_check(input),
        Regime::NewYorkCity => nyc_check(input),
        Regime::Default => default_check(input),
    }
}

fn ma_check(input: &SnowRemovalInput) -> SnowRemovalResult {
    // MA State Sanitary Code 105 CMR 410.452: landlord primary duty for
    // means of egress. Delegation valid only if tenant has independent
    // private entrance.
    let primary_landlord_duty = matches!(
        input.location,
        LocationType::MeansOfEgress | LocationType::PublicSidewalk
    ) || input.multi_unit_building;
    let delegation_valid = input.lease_delegates_to_tenant
        && input.tenant_has_independent_private_entrance
        && !input.multi_unit_building;
    if input.lease_delegates_to_tenant && !delegation_valid {
        return SnowRemovalResult {
            regime: Regime::Massachusetts,
            landlord_has_duty: primary_landlord_duty,
            statutory_timeline_hours: 0,
            immunity_applies: false,
            delegation_to_tenant_valid: false,
            violation: ViolationType::InvalidLeaseDelegation,
            landlord_compliant: false,
            citation: "Mass. State Sanitary Code 105 CMR 410.452 — landlord's primary means-of-egress duty cannot be delegated to tenant via lease UNLESS tenant has an independent, private entrance and is not in a multi-unit building",
            note: "Lease delegation to tenant is INVALID under MA Sanitary Code: tenant lacks independent private entrance OR property is multi-unit.".to_string(),
        };
    }
    if primary_landlord_duty && !input.snow_removed {
        return SnowRemovalResult {
            regime: Regime::Massachusetts,
            landlord_has_duty: true,
            statutory_timeline_hours: 0,
            immunity_applies: false,
            delegation_to_tenant_valid: delegation_valid,
            violation: ViolationType::NotClearedWithinTimeline,
            landlord_compliant: false,
            citation: "Papadopoulos v. Target Corp. 457 Mass. 368 (2010) — landlord owes a duty of REASONABLE CARE to keep premises reasonably safe; natural-vs-unnatural accumulation distinction abolished",
            note: format!(
                "Snow stopped {} hours ago and remains uncleared. MA reasonable-care duty (Papadopoulos) applies — failure breaches duty and may support premises-liability claim.",
                input.hours_since_snow_stopped
            ),
        };
    }
    SnowRemovalResult {
        regime: Regime::Massachusetts,
        landlord_has_duty: primary_landlord_duty,
        statutory_timeline_hours: 0,
        immunity_applies: false,
        delegation_to_tenant_valid: delegation_valid,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "Papadopoulos v. Target Corp. + Mass. State Sanitary Code 105 CMR 410.452 — compliance OK",
        note: format!(
            "MA reasonable-care duty: snow {} cleared. Delegation to tenant {} valid.",
            if input.snow_removed { "is" } else { "is not" },
            if delegation_valid { "is" } else { "is not" },
        ),
    }
}

fn il_check(input: &SnowRemovalInput) -> SnowRemovalResult {
    // Illinois 745 ILCS 75/ — immunity for VOLUNTARY clearing of public
    // sidewalks UNLESS willful or wanton. Immunity does NOT extend to
    // private property (driveway, walkway).
    if input.willful_or_wanton_conduct {
        return SnowRemovalResult {
            regime: Regime::Illinois,
            landlord_has_duty: false,
            statutory_timeline_hours: 0,
            immunity_applies: false,
            delegation_to_tenant_valid: false,
            violation: ViolationType::WillfulOrWantonOverrideImmunity,
            landlord_compliant: false,
            citation: "745 ILCS 75/ — Illinois Snow and Ice Removal Act immunity does NOT apply to willful or wanton conduct",
            note: "Willful or wanton conduct overrides statutory immunity. Landlord exposed to liability.".to_string(),
        };
    }
    let immunity = matches!(input.location, LocationType::PublicSidewalk);
    if !immunity {
        // Private property: no immunity. Standard premises-liability
        // rules apply. Failure to clear may support negligence claim
        // depending on circumstances.
        return SnowRemovalResult {
            regime: Regime::Illinois,
            landlord_has_duty: !input.snow_removed,
            statutory_timeline_hours: 0,
            immunity_applies: false,
            delegation_to_tenant_valid: input.lease_delegates_to_tenant,
            violation: if input.snow_removed {
                ViolationType::None
            } else {
                ViolationType::NotClearedWithinTimeline
            },
            landlord_compliant: input.snow_removed,
            citation: "745 ILCS 75/ — Illinois immunity does NOT extend to private property (driveway / walkway to front door / garage entrance); standard premises-liability rules apply",
            note: format!(
                "Private-property location — Illinois Snow Removal Act immunity does not apply. Standard premises-liability analysis. Snow {} cleared.",
                if input.snow_removed { "is" } else { "is not" }
            ),
        };
    }
    // Public sidewalk + non-willful: immunity applies regardless of
    // whether voluntarily cleared.
    SnowRemovalResult {
        regime: Regime::Illinois,
        landlord_has_duty: false,
        statutory_timeline_hours: 0,
        immunity_applies: true,
        delegation_to_tenant_valid: input.lease_delegates_to_tenant,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "745 ILCS 75/ Snow and Ice Removal Act — residential owner IMMUNE from liability for voluntary snow/ice removal efforts on PUBLIC SIDEWALK unless willful or wanton",
        note: format!(
            "Public sidewalk + non-willful conduct → IL Snow Removal Act immunity applies. Snow {} cleared.",
            if input.snow_removed { "is" } else { "is not" }
        ),
    }
}

fn nyc_check(input: &SnowRemovalInput) -> SnowRemovalResult {
    // NYC § 16-123: 4-hour window after snow stops, except 9 PM–7 AM
    // (caller's hours_since_snow_stopped should account for the night
    // pause). Multi-unit landlord = always primarily responsible.
    // Single-family lease may delegate but owner still gets ticket.
    let timeline = 4;
    let delegation_valid = input.lease_delegates_to_tenant && !input.multi_unit_building;
    if input.lease_delegates_to_tenant && !delegation_valid {
        return SnowRemovalResult {
            regime: Regime::NewYorkCity,
            landlord_has_duty: true,
            statutory_timeline_hours: timeline,
            immunity_applies: false,
            delegation_to_tenant_valid: false,
            violation: ViolationType::InvalidLeaseDelegation,
            landlord_compliant: false,
            citation: "NYC Admin Code § 16-123 — in MULTI-UNIT buildings, landlord is FULLY responsible; lease delegation to tenant invalid",
            note: "Lease purports to delegate snow removal to tenant in a multi-unit building. § 16-123 makes landlord fully responsible regardless.".to_string(),
        };
    }
    if !input.snow_removed && input.hours_since_snow_stopped > timeline {
        return SnowRemovalResult {
            regime: Regime::NewYorkCity,
            landlord_has_duty: true,
            statutory_timeline_hours: timeline,
            immunity_applies: false,
            delegation_to_tenant_valid: delegation_valid,
            violation: ViolationType::NotClearedWithinTimeline,
            landlord_compliant: false,
            citation: "NYC Admin Code § 16-123 — snow + ice must be removed within 4 hours of snow stopping (except 9 PM–7 AM window)",
            note: format!(
                "Snow stopped {} hours ago (> 4-hour timeline). § 16-123 violated. Note: even with valid single-family delegation, OWNER still receives any city ticket.",
                input.hours_since_snow_stopped
            ),
        };
    }
    SnowRemovalResult {
        regime: Regime::NewYorkCity,
        landlord_has_duty: true,
        statutory_timeline_hours: timeline,
        immunity_applies: false,
        delegation_to_tenant_valid: delegation_valid,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "NYC Admin Code § 16-123 — compliance OK; multi-unit landlord fully responsible; single-family delegation permitted but owner receives any ticket",
        note: format!(
            "NYC § 16-123 compliance OK: snow {} cleared within 4-hour window. Delegation to tenant {} valid (single-family only).",
            if input.snow_removed { "is" } else { "is not" },
            if delegation_valid { "is" } else { "is not" },
        ),
    }
}

fn default_check(input: &SnowRemovalInput) -> SnowRemovalResult {
    SnowRemovalResult {
        regime: Regime::Default,
        landlord_has_duty: !input.snow_removed,
        statutory_timeline_hours: 0,
        immunity_applies: false,
        delegation_to_tenant_valid: input.lease_delegates_to_tenant,
        // Default doesn't quantify per-state premises-liability standards;
        // common-law analysis would apply by jurisdiction. No violation
        // surfaced here regardless of clearing status.
        violation: ViolationType::None,
        landlord_compliant: true,
        citation:
            "No statewide snow/ice removal statute identified — common-law habitability + premises-liability rules vary by jurisdiction",
        note: "Default regime: common law applies. Landlord premises-liability exposure depends on local jurisdiction.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        location: LocationType,
        hours: u32,
        removed: bool,
        delegated: bool,
        multi_unit: bool,
        independent_entrance: bool,
        willful: bool,
    ) -> SnowRemovalInput {
        SnowRemovalInput {
            regime,
            location,
            hours_since_snow_stopped: hours,
            snow_removed: removed,
            lease_delegates_to_tenant: delegated,
            multi_unit_building: multi_unit,
            tenant_has_independent_private_entrance: independent_entrance,
            willful_or_wanton_conduct: willful,
        }
    }

    #[test]
    fn ma_means_of_egress_landlord_duty() {
        let r = check(&input(
            Regime::Massachusetts,
            LocationType::MeansOfEgress,
            6,
            true,
            false,
            true,
            false,
            false,
        ));
        assert!(r.landlord_has_duty);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn ma_uncleared_egress_violation() {
        let r = check(&input(
            Regime::Massachusetts,
            LocationType::MeansOfEgress,
            6,
            false,
            false,
            true,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::NotClearedWithinTimeline);
        assert!(r.citation.contains("Papadopoulos"));
    }

    #[test]
    fn ma_lease_delegation_invalid_in_multi_unit() {
        let r = check(&input(
            Regime::Massachusetts,
            LocationType::MeansOfEgress,
            2,
            false,
            true,
            true,
            true,
            false,
        ));
        assert_eq!(r.violation, ViolationType::InvalidLeaseDelegation);
        assert!(!r.delegation_to_tenant_valid);
    }

    #[test]
    fn ma_lease_delegation_valid_single_family_with_independent_entrance() {
        let r = check(&input(
            Regime::Massachusetts,
            LocationType::MeansOfEgress,
            2,
            true,
            true,
            false,
            true,
            false,
        ));
        assert!(r.delegation_to_tenant_valid);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn ma_natural_accumulation_distinction_abolished() {
        // Even pure natural accumulation imposes duty under Papadopoulos.
        let r = check(&input(
            Regime::Massachusetts,
            LocationType::MeansOfEgress,
            12,
            false,
            false,
            true,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::NotClearedWithinTimeline);
    }

    #[test]
    fn il_public_sidewalk_immunity_applies() {
        let r = check(&input(
            Regime::Illinois,
            LocationType::PublicSidewalk,
            12,
            false,
            false,
            false,
            false,
            false,
        ));
        assert!(r.immunity_applies);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.citation.contains("745 ILCS 75/"));
    }

    #[test]
    fn il_willful_wanton_overrides_immunity() {
        let r = check(&input(
            Regime::Illinois,
            LocationType::PublicSidewalk,
            6,
            true,
            false,
            false,
            false,
            true,
        ));
        assert!(!r.immunity_applies);
        assert_eq!(r.violation, ViolationType::WillfulOrWantonOverrideImmunity);
    }

    #[test]
    fn il_private_property_no_immunity() {
        let r = check(&input(
            Regime::Illinois,
            LocationType::PrivateProperty,
            12,
            false,
            false,
            false,
            false,
            false,
        ));
        assert!(!r.immunity_applies);
        assert!(r.citation.contains("does NOT extend to private property"));
    }

    #[test]
    fn il_private_property_cleared_compliant() {
        let r = check(&input(
            Regime::Illinois,
            LocationType::PrivateProperty,
            2,
            true,
            false,
            false,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn nyc_within_4_hour_window_compliant() {
        let r = check(&input(
            Regime::NewYorkCity,
            LocationType::PublicSidewalk,
            3,
            true,
            false,
            true,
            false,
            false,
        ));
        assert_eq!(r.statutory_timeline_hours, 4);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn nyc_at_4_hour_boundary_compliant() {
        let r = check(&input(
            Regime::NewYorkCity,
            LocationType::PublicSidewalk,
            4,
            false,
            false,
            true,
            false,
            false,
        ));
        // 4 hours not exceeded (> 4 required for violation).
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn nyc_at_5_hours_violation() {
        let r = check(&input(
            Regime::NewYorkCity,
            LocationType::PublicSidewalk,
            5,
            false,
            false,
            true,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::NotClearedWithinTimeline);
        assert!(r.citation.contains("4 hours"));
    }

    #[test]
    fn nyc_multi_unit_invalid_delegation() {
        let r = check(&input(
            Regime::NewYorkCity,
            LocationType::PublicSidewalk,
            2,
            true,
            true,
            true,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::InvalidLeaseDelegation);
    }

    #[test]
    fn nyc_single_family_delegation_valid() {
        let r = check(&input(
            Regime::NewYorkCity,
            LocationType::PublicSidewalk,
            2,
            true,
            true,
            false,
            true,
            false,
        ));
        assert!(r.delegation_to_tenant_valid);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn default_no_specific_obligation() {
        let r = check(&input(
            Regime::Default,
            LocationType::PublicSidewalk,
            12,
            false,
            false,
            false,
            false,
            false,
        ));
        assert!(r.citation.contains("common-law habitability"));
    }

    #[test]
    fn jurisdiction_routing_ma_il_nyc_default() {
        assert_eq!(
            Regime::for_jurisdiction("MA", "Boston"),
            Regime::Massachusetts
        );
        assert_eq!(Regime::for_jurisdiction("IL", "Chicago"), Regime::Illinois);
        assert_eq!(
            Regime::for_jurisdiction("NY", "New York"),
            Regime::NewYorkCity
        );
        assert_eq!(Regime::for_jurisdiction("NY", "NYC"), Regime::NewYorkCity);
        assert_eq!(Regime::for_jurisdiction("NY", "Buffalo"), Regime::Default);
        assert_eq!(Regime::for_jurisdiction("CA", "LA"), Regime::Default);
    }

    #[test]
    fn jurisdiction_routing_case_insensitive() {
        assert_eq!(
            Regime::for_jurisdiction("ma", "anywhere"),
            Regime::Massachusetts
        );
        assert_eq!(Regime::for_jurisdiction("il", "any"), Regime::Illinois);
    }

    #[test]
    fn only_il_has_voluntary_clearing_immunity() {
        // Same uncleared-public-sidewalk scenario across regimes.
        let il = check(&input(
            Regime::Illinois,
            LocationType::PublicSidewalk,
            12,
            false,
            false,
            false,
            false,
            false,
        ));
        let ma = check(&input(
            Regime::Massachusetts,
            LocationType::PublicSidewalk,
            12,
            false,
            false,
            false,
            false,
            false,
        ));
        let nyc = check(&input(
            Regime::NewYorkCity,
            LocationType::PublicSidewalk,
            12,
            false,
            false,
            false,
            false,
            false,
        ));
        assert!(il.immunity_applies);
        assert!(!ma.immunity_applies);
        assert!(!nyc.immunity_applies);
    }

    #[test]
    fn only_nyc_has_4_hour_timeline() {
        let nyc = check(&input(
            Regime::NewYorkCity,
            LocationType::PublicSidewalk,
            5,
            false,
            false,
            true,
            false,
            false,
        ));
        let ma = check(&input(
            Regime::Massachusetts,
            LocationType::PublicSidewalk,
            5,
            false,
            false,
            true,
            false,
            false,
        ));
        let il = check(&input(
            Regime::Illinois,
            LocationType::PublicSidewalk,
            5,
            false,
            false,
            true,
            false,
            false,
        ));
        assert_eq!(nyc.statutory_timeline_hours, 4);
        assert_eq!(ma.statutory_timeline_hours, 0);
        assert_eq!(il.statutory_timeline_hours, 0);
    }

    #[test]
    fn only_ma_requires_independent_entrance_for_delegation() {
        // Same single-family lease-delegating-to-tenant scenario WITHOUT
        // independent entrance.
        let ma = check(&input(
            Regime::Massachusetts,
            LocationType::MeansOfEgress,
            2,
            true,
            true,
            false,
            false, // no independent entrance
            false,
        ));
        let nyc = check(&input(
            Regime::NewYorkCity,
            LocationType::PublicSidewalk,
            2,
            true,
            true,
            false,
            false,
            false,
        ));
        // MA: delegation invalid without independent entrance.
        assert!(!ma.delegation_to_tenant_valid);
        // NYC: delegation valid (single-family can delegate; entrance
        // not required).
        assert!(nyc.delegation_to_tenant_valid);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let ma = check(&input(
            Regime::Massachusetts,
            LocationType::MeansOfEgress,
            6,
            false,
            false,
            true,
            false,
            false,
        ));
        assert!(ma.citation.contains("Papadopoulos"));

        let il = check(&input(
            Regime::Illinois,
            LocationType::PublicSidewalk,
            6,
            true,
            false,
            false,
            false,
            false,
        ));
        assert!(il.citation.contains("745 ILCS 75/"));

        let nyc = check(&input(
            Regime::NewYorkCity,
            LocationType::PublicSidewalk,
            2,
            true,
            false,
            true,
            false,
            false,
        ));
        assert!(nyc.citation.contains("§ 16-123"));
    }
}
