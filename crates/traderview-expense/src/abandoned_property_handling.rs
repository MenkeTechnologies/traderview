//! State abandoned-tenant-personal-property landlord compliance check.
//!
//! Distinct from `tenant_abandonment` (which addresses when the tenant
//! has vacated and the landlord may declare the unit abandoned) — this
//! module addresses WHAT the landlord must do with BELONGINGS the tenant
//! left behind. Three jurisdictions ship detailed statutory procedures
//! with materially different timeline + value thresholds + sale duties.
//!
//! California (Cal. Civ. Code §§ 1980-1991) — landlord must serve a
//! Notice of Right to Reclaim Abandoned Property. Reclaim clock is 15
//! days from personal delivery OR 18 days from first-class mailing.
//! After clock expires: if total value < $700, landlord may keep or
//! dispose; if total value >= $700, landlord MUST conduct a public
//! auction with the proceeds (less storage costs and auction expenses)
//! going to the county treasury. Cannot offset against unpaid rent by
//! keeping items.
//!
//! Texas (Tex. Prop. Code § 54.044 + § 92.0081) — landlord may remove
//! contents on tenant's abandonment without prior notice. To sell
//! abandoned property, landlord must give 30-day notice by FIRST-CLASS
//! mail AND CERTIFIED mail (return receipt) to tenant's last known
//! address before sale.
//!
//! Washington (RCW 59.18.310) — landlord serves notice of intent to sell
//! or dispose; clock depends on value: 45 days if total value >= $250;
//! only 7 days if value < $250 (with the EXCEPTION that personal papers,
//! family pictures, and keepsakes can NEVER be thrown out under the
//! 7-day rule — those always require the 45-day procedure).
//!
//! Default — common-law abandonment; landlord must give reasonable
//! notice and use reasonable means to dispose; no statutory dollar
//! threshold.
//!
//! Citations: Cal. Civ. Code § 1980 (definitions); § 1983 (notice of
//! right to reclaim); § 1984 (15-day personal-delivery / 18-day mail);
//! § 1988 ($700 auction threshold); Tex. Prop. Code § 54.044 (seizure);
//! § 92.0081 (residential abandoned property + 30-day notice before
//! sale); RCW 59.18.310 (45-day standard / 7-day under $250 with
//! keepsake carve-out).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Texas,
    Washington,
    Default,
}

impl Regime {
    pub fn for_state(state: &str) -> Self {
        match state.trim().to_ascii_uppercase().as_str() {
            "CA" => Self::California,
            "TX" => Self::Texas,
            "WA" => Self::Washington,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeDelivery {
    PersonalDelivery,
    FirstClassMail,
    CertifiedMail,
    BothFirstClassAndCertified,
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AbandonedPropertyInput {
    pub regime: Regime,
    pub written_notice_provided: bool,
    pub notice_delivery: NoticeDelivery,
    pub days_since_notice: u32,
    pub total_property_value_cents: i64,
    /// Whether the property includes personal papers, family pictures,
    /// or other keepsakes — Washington's RCW 59.18.310 carve-out
    /// requires 45-day procedure for these regardless of value.
    pub includes_personal_papers_or_keepsakes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DispositionRoute {
    /// Landlord may sell at public auction (proceeds go to specified party).
    PublicAuction,
    /// Landlord may keep, sell privately, or dispose at will.
    KeepOrDispose,
    /// Landlord cannot yet act — notice or waiting period not satisfied.
    NotYetPermitted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    MissingWrittenNotice,
    InsufficientNoticeDelivery,
    InsufficientWaitingPeriod,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AbandonedPropertyResult {
    pub regime: Regime,
    pub minimum_waiting_period_days: u32,
    pub statutory_value_threshold_cents: i64,
    pub above_value_threshold: bool,
    pub disposition_route: DispositionRoute,
    pub violation: ViolationType,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &AbandonedPropertyInput) -> AbandonedPropertyResult {
    match input.regime {
        Regime::California => ca_check(input),
        Regime::Texas => tx_check(input),
        Regime::Washington => wa_check(input),
        Regime::Default => default_check(input),
    }
}

fn ca_check(input: &AbandonedPropertyInput) -> AbandonedPropertyResult {
    if !input.written_notice_provided {
        return AbandonedPropertyResult {
            regime: Regime::California,
            minimum_waiting_period_days: 15,
            statutory_value_threshold_cents: 70000,
            above_value_threshold: input.total_property_value_cents >= 70000,
            disposition_route: DispositionRoute::NotYetPermitted,
            violation: ViolationType::MissingWrittenNotice,
            landlord_compliant: false,
            citation: "Cal. Civ. Code § 1983 — landlord must serve Notice of Right to Reclaim Abandoned Property",
            note: "Required § 1983 notice not provided. Landlord cannot dispose of abandoned property.".to_string(),
        };
    }
    let required_days = match input.notice_delivery {
        NoticeDelivery::PersonalDelivery => 15,
        NoticeDelivery::FirstClassMail
        | NoticeDelivery::CertifiedMail
        | NoticeDelivery::BothFirstClassAndCertified => 18,
        NoticeDelivery::None => {
            return AbandonedPropertyResult {
                regime: Regime::California,
                minimum_waiting_period_days: 15,
                statutory_value_threshold_cents: 70000,
                above_value_threshold: input.total_property_value_cents >= 70000,
                disposition_route: DispositionRoute::NotYetPermitted,
                violation: ViolationType::InsufficientNoticeDelivery,
                landlord_compliant: false,
                citation: "Cal. Civ. Code § 1984 — notice must be delivered by personal delivery (15-day clock) or first-class mail (18-day clock)",
                note: "Notice not delivered by an authorized method under § 1984.".to_string(),
            };
        }
    };
    if input.days_since_notice < required_days {
        return AbandonedPropertyResult {
            regime: Regime::California,
            minimum_waiting_period_days: required_days,
            statutory_value_threshold_cents: 70000,
            above_value_threshold: input.total_property_value_cents >= 70000,
            disposition_route: DispositionRoute::NotYetPermitted,
            violation: ViolationType::InsufficientWaitingPeriod,
            landlord_compliant: false,
            citation: "Cal. Civ. Code § 1984 — 15-day clock for personal delivery / 18-day clock for first-class mail",
            note: format!(
                "Only {} days since notice; § 1984 requires at least {} days for the chosen delivery method.",
                input.days_since_notice, required_days
            ),
        };
    }
    let above_threshold = input.total_property_value_cents >= 70000;
    let (route, citation) = if above_threshold {
        (
            DispositionRoute::PublicAuction,
            "Cal. Civ. Code § 1988 — property valued at $700 or more MUST be sold at public auction (cannot offset against unpaid rent)",
        )
    } else {
        (
            DispositionRoute::KeepOrDispose,
            "Cal. Civ. Code § 1988 — property valued under $700 may be kept, sold privately, or disposed by landlord",
        )
    };
    AbandonedPropertyResult {
        regime: Regime::California,
        minimum_waiting_period_days: required_days,
        statutory_value_threshold_cents: 70000,
        above_value_threshold: above_threshold,
        disposition_route: route,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation,
        note: format!(
            "CA § 1980-1991 compliance OK: notice served + {} days elapsed (≥ {}). Total value {} cents {} $700 threshold; disposition route: {:?}.",
            input.days_since_notice,
            required_days,
            input.total_property_value_cents,
            if above_threshold { "≥" } else { "<" },
            route,
        ),
    }
}

fn tx_check(input: &AbandonedPropertyInput) -> AbandonedPropertyResult {
    if !input.written_notice_provided {
        return AbandonedPropertyResult {
            regime: Regime::Texas,
            minimum_waiting_period_days: 30,
            statutory_value_threshold_cents: 0,
            above_value_threshold: false,
            disposition_route: DispositionRoute::NotYetPermitted,
            violation: ViolationType::MissingWrittenNotice,
            landlord_compliant: false,
            citation: "Tex. Prop. Code § 92.0081 — 30-day notice required before SALE of abandoned property",
            note: "Required 30-day notice before sale not provided.".to_string(),
        };
    }
    // TX requires both first-class AND certified mail for sale notice.
    if !matches!(
        input.notice_delivery,
        NoticeDelivery::BothFirstClassAndCertified
    ) {
        return AbandonedPropertyResult {
            regime: Regime::Texas,
            minimum_waiting_period_days: 30,
            statutory_value_threshold_cents: 0,
            above_value_threshold: false,
            disposition_route: DispositionRoute::NotYetPermitted,
            violation: ViolationType::InsufficientNoticeDelivery,
            landlord_compliant: false,
            citation: "Tex. Prop. Code § 92.0081 — sale notice must be sent by FIRST-CLASS mail AND CERTIFIED mail (return receipt) to tenant's last known address",
            note: "Notice must be sent by BOTH first-class mail AND certified mail return-receipt under TX § 92.0081.".to_string(),
        };
    }
    if input.days_since_notice < 30 {
        return AbandonedPropertyResult {
            regime: Regime::Texas,
            minimum_waiting_period_days: 30,
            statutory_value_threshold_cents: 0,
            above_value_threshold: false,
            disposition_route: DispositionRoute::NotYetPermitted,
            violation: ViolationType::InsufficientWaitingPeriod,
            landlord_compliant: false,
            citation: "Tex. Prop. Code § 92.0081 — 30-day notice required before sale",
            note: format!(
                "Only {} days since notice; TX § 92.0081 requires 30 days before sale.",
                input.days_since_notice
            ),
        };
    }
    AbandonedPropertyResult {
        regime: Regime::Texas,
        minimum_waiting_period_days: 30,
        statutory_value_threshold_cents: 0,
        above_value_threshold: false,
        disposition_route: DispositionRoute::PublicAuction,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "Tex. Prop. Code §§ 54.044 + 92.0081 — 30-day dual-method notice satisfied; landlord may sell",
        note: format!(
            "TX § 54.044 + § 92.0081 compliance OK: 30-day notice served by first-class AND certified mail; {} days elapsed. Landlord may sell at auction.",
            input.days_since_notice
        ),
    }
}

fn wa_check(input: &AbandonedPropertyInput) -> AbandonedPropertyResult {
    if !input.written_notice_provided {
        return AbandonedPropertyResult {
            regime: Regime::Washington,
            minimum_waiting_period_days: 45,
            statutory_value_threshold_cents: 25000,
            above_value_threshold: input.total_property_value_cents >= 25000,
            disposition_route: DispositionRoute::NotYetPermitted,
            violation: ViolationType::MissingWrittenNotice,
            landlord_compliant: false,
            citation: "RCW 59.18.310 — written notice of intent to sell/dispose required",
            note: "Required RCW 59.18.310 notice of intent to sell/dispose not provided.".to_string(),
        };
    }
    // 7-day rule applies only if value < $250 AND no personal-papers
    // carve-out. Otherwise the standard 45-day rule applies.
    let above_threshold = input.total_property_value_cents >= 25000;
    let required_days = if above_threshold || input.includes_personal_papers_or_keepsakes {
        45
    } else {
        7
    };
    if input.days_since_notice < required_days {
        return AbandonedPropertyResult {
            regime: Regime::Washington,
            minimum_waiting_period_days: required_days,
            statutory_value_threshold_cents: 25000,
            above_value_threshold: above_threshold,
            disposition_route: DispositionRoute::NotYetPermitted,
            violation: ViolationType::InsufficientWaitingPeriod,
            landlord_compliant: false,
            citation: if required_days == 45 {
                "RCW 59.18.310 — 45-day waiting period when total value ≥ $250 OR property includes personal papers / family pictures / keepsakes"
            } else {
                "RCW 59.18.310 — 7-day waiting period when total value < $250 (except for personal papers / family pictures / keepsakes which always require 45 days)"
            },
            note: format!(
                "Only {} days since notice; RCW 59.18.310 requires {} days for the applicable category.",
                input.days_since_notice, required_days
            ),
        };
    }
    let route = if above_threshold {
        DispositionRoute::PublicAuction
    } else {
        DispositionRoute::KeepOrDispose
    };
    AbandonedPropertyResult {
        regime: Regime::Washington,
        minimum_waiting_period_days: required_days,
        statutory_value_threshold_cents: 25000,
        above_value_threshold: above_threshold,
        disposition_route: route,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: if required_days == 45 {
            "RCW 59.18.310 — 45-day waiting period satisfied"
        } else {
            "RCW 59.18.310 — 7-day waiting period satisfied (value < $250, no keepsakes)"
        },
        note: format!(
            "WA RCW 59.18.310 compliance OK: notice served + {} days elapsed (≥ {}). Disposition route: {:?}.",
            input.days_since_notice, required_days, route,
        ),
    }
}

fn default_check(input: &AbandonedPropertyInput) -> AbandonedPropertyResult {
    let permitted = input.written_notice_provided && input.days_since_notice >= 30;
    AbandonedPropertyResult {
        regime: Regime::Default,
        minimum_waiting_period_days: 30,
        statutory_value_threshold_cents: 0,
        above_value_threshold: false,
        disposition_route: if permitted {
            DispositionRoute::KeepOrDispose
        } else {
            DispositionRoute::NotYetPermitted
        },
        violation: if permitted {
            ViolationType::None
        } else if !input.written_notice_provided {
            ViolationType::MissingWrittenNotice
        } else {
            ViolationType::InsufficientWaitingPeriod
        },
        landlord_compliant: permitted,
        citation:
            "No statewide statutory procedure identified — common-law abandonment requires reasonable notice and reasonable disposal",
        note:
            "Default regime: common law applies. Reasonable written notice + reasonable waiting period (30 days here as proxy) recommended.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        notice: bool,
        delivery: NoticeDelivery,
        days: u32,
        value: i64,
        keepsakes: bool,
    ) -> AbandonedPropertyInput {
        AbandonedPropertyInput {
            regime,
            written_notice_provided: notice,
            notice_delivery: delivery,
            days_since_notice: days,
            total_property_value_cents: value,
            includes_personal_papers_or_keepsakes: keepsakes,
        }
    }

    #[test]
    fn ca_personal_delivery_15_days_under_700_keep() {
        let r = check(&input(
            Regime::California,
            true,
            NoticeDelivery::PersonalDelivery,
            15,
            500_00,
            false,
        ));
        assert_eq!(r.disposition_route, DispositionRoute::KeepOrDispose);
        assert_eq!(r.violation, ViolationType::None);
        assert_eq!(r.minimum_waiting_period_days, 15);
    }

    #[test]
    fn ca_first_class_mail_18_days_above_700_auction() {
        let r = check(&input(
            Regime::California,
            true,
            NoticeDelivery::FirstClassMail,
            18,
            1_000_00,
            false,
        ));
        assert_eq!(r.disposition_route, DispositionRoute::PublicAuction);
        assert!(r.above_value_threshold);
        assert!(r.citation.contains("§ 1988"));
        assert!(r.citation.contains("$700"));
    }

    #[test]
    fn ca_at_700_boundary_must_auction() {
        let r = check(&input(
            Regime::California,
            true,
            NoticeDelivery::PersonalDelivery,
            15,
            700_00,
            false,
        ));
        assert!(r.above_value_threshold);
        assert_eq!(r.disposition_route, DispositionRoute::PublicAuction);
    }

    #[test]
    fn ca_at_699_99_under_threshold_keep() {
        let r = check(&input(
            Regime::California,
            true,
            NoticeDelivery::PersonalDelivery,
            15,
            699_99,
            false,
        ));
        assert!(!r.above_value_threshold);
        assert_eq!(r.disposition_route, DispositionRoute::KeepOrDispose);
    }

    #[test]
    fn ca_14_days_personal_delivery_insufficient() {
        let r = check(&input(
            Regime::California,
            true,
            NoticeDelivery::PersonalDelivery,
            14,
            500_00,
            false,
        ));
        assert_eq!(r.violation, ViolationType::InsufficientWaitingPeriod);
    }

    #[test]
    fn ca_17_days_first_class_insufficient_18_required() {
        let r = check(&input(
            Regime::California,
            true,
            NoticeDelivery::FirstClassMail,
            17,
            500_00,
            false,
        ));
        assert_eq!(r.violation, ViolationType::InsufficientWaitingPeriod);
        assert!(r.note.contains("18 days"));
    }

    #[test]
    fn ca_no_notice_blocks() {
        let r = check(&input(
            Regime::California,
            false,
            NoticeDelivery::None,
            30,
            500_00,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingWrittenNotice);
        assert!(r.citation.contains("§ 1983"));
    }

    #[test]
    fn tx_30_day_dual_method_notice_satisfied() {
        let r = check(&input(
            Regime::Texas,
            true,
            NoticeDelivery::BothFirstClassAndCertified,
            30,
            500_00,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert_eq!(r.disposition_route, DispositionRoute::PublicAuction);
        assert!(r.citation.contains("92.0081"));
    }

    #[test]
    fn tx_only_first_class_insufficient_delivery() {
        let r = check(&input(
            Regime::Texas,
            true,
            NoticeDelivery::FirstClassMail,
            30,
            500_00,
            false,
        ));
        assert_eq!(r.violation, ViolationType::InsufficientNoticeDelivery);
        assert!(r.citation.contains("CERTIFIED"));
    }

    #[test]
    fn tx_only_certified_insufficient_delivery() {
        let r = check(&input(
            Regime::Texas,
            true,
            NoticeDelivery::CertifiedMail,
            30,
            500_00,
            false,
        ));
        assert_eq!(r.violation, ViolationType::InsufficientNoticeDelivery);
    }

    #[test]
    fn tx_29_days_insufficient() {
        let r = check(&input(
            Regime::Texas,
            true,
            NoticeDelivery::BothFirstClassAndCertified,
            29,
            500_00,
            false,
        ));
        assert_eq!(r.violation, ViolationType::InsufficientWaitingPeriod);
    }

    #[test]
    fn wa_45_day_above_250_threshold() {
        let r = check(&input(
            Regime::Washington,
            true,
            NoticeDelivery::FirstClassMail,
            45,
            500_00,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert_eq!(r.minimum_waiting_period_days, 45);
        assert_eq!(r.disposition_route, DispositionRoute::PublicAuction);
    }

    #[test]
    fn wa_7_day_under_250_no_keepsakes() {
        let r = check(&input(
            Regime::Washington,
            true,
            NoticeDelivery::FirstClassMail,
            7,
            200_00,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert_eq!(r.minimum_waiting_period_days, 7);
        assert_eq!(r.disposition_route, DispositionRoute::KeepOrDispose);
    }

    #[test]
    fn wa_under_250_with_keepsakes_requires_45_days() {
        // Keepsake carve-out forces 45-day rule even though value is low.
        let r = check(&input(
            Regime::Washington,
            true,
            NoticeDelivery::FirstClassMail,
            7,
            100_00,
            true,
        ));
        assert_eq!(r.minimum_waiting_period_days, 45);
        assert_eq!(r.violation, ViolationType::InsufficientWaitingPeriod);
        assert!(r.citation.contains("keepsakes"));
    }

    #[test]
    fn wa_at_250_boundary_uses_45_day() {
        let r = check(&input(
            Regime::Washington,
            true,
            NoticeDelivery::FirstClassMail,
            7,
            250_00,
            false,
        ));
        assert_eq!(r.minimum_waiting_period_days, 45);
        assert_eq!(r.violation, ViolationType::InsufficientWaitingPeriod);
    }

    #[test]
    fn wa_above_250_with_keepsakes_45_day() {
        let r = check(&input(
            Regime::Washington,
            true,
            NoticeDelivery::FirstClassMail,
            44,
            500_00,
            true,
        ));
        assert_eq!(r.minimum_waiting_period_days, 45);
        assert_eq!(r.violation, ViolationType::InsufficientWaitingPeriod);
    }

    #[test]
    fn wa_no_notice_blocks() {
        let r = check(&input(
            Regime::Washington,
            false,
            NoticeDelivery::None,
            45,
            500_00,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingWrittenNotice);
    }

    #[test]
    fn default_30_days_compliant() {
        let r = check(&input(
            Regime::Default,
            true,
            NoticeDelivery::FirstClassMail,
            30,
            500_00,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.citation.contains("common-law"));
    }

    #[test]
    fn default_no_notice_blocks() {
        let r = check(&input(
            Regime::Default,
            false,
            NoticeDelivery::None,
            45,
            500_00,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingWrittenNotice);
    }

    #[test]
    fn state_routing_ca_tx_wa_default() {
        assert_eq!(Regime::for_state("CA"), Regime::California);
        assert_eq!(Regime::for_state("TX"), Regime::Texas);
        assert_eq!(Regime::for_state("WA"), Regime::Washington);
        assert_eq!(Regime::for_state("NY"), Regime::Default);
    }

    #[test]
    fn state_routing_case_insensitive() {
        assert_eq!(Regime::for_state("ca"), Regime::California);
        assert_eq!(Regime::for_state("tx"), Regime::Texas);
        assert_eq!(Regime::for_state("wa"), Regime::Washington);
    }

    #[test]
    fn only_ca_has_700_dollar_auction_threshold() {
        let ca = check(&input(
            Regime::California,
            true,
            NoticeDelivery::PersonalDelivery,
            15,
            1_000_00,
            false,
        ));
        let tx = check(&input(
            Regime::Texas,
            true,
            NoticeDelivery::BothFirstClassAndCertified,
            30,
            1_000_00,
            false,
        ));
        let wa = check(&input(
            Regime::Washington,
            true,
            NoticeDelivery::FirstClassMail,
            45,
            1_000_00,
            false,
        ));
        assert_eq!(ca.statutory_value_threshold_cents, 700_00);
        assert_eq!(tx.statutory_value_threshold_cents, 0);
        assert_eq!(wa.statutory_value_threshold_cents, 250_00);
    }

    #[test]
    fn only_wa_has_keepsake_carve_out() {
        // Same low-value-with-keepsakes input across regimes.
        let wa = check(&input(
            Regime::Washington,
            true,
            NoticeDelivery::FirstClassMail,
            7,
            100_00,
            true,
        ));
        let ca = check(&input(
            Regime::California,
            true,
            NoticeDelivery::PersonalDelivery,
            15,
            100_00,
            true,
        ));
        // WA: keepsakes force 45-day wait, only 7 days elapsed → violation.
        assert_eq!(wa.violation, ViolationType::InsufficientWaitingPeriod);
        // CA: no keepsake carve-out, 15 days personal delivery is enough.
        assert_eq!(ca.violation, ViolationType::None);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let ca = check(&input(
            Regime::California,
            true,
            NoticeDelivery::PersonalDelivery,
            15,
            1_000_00,
            false,
        ));
        assert!(ca.citation.contains("§ 1988"));

        let tx = check(&input(
            Regime::Texas,
            true,
            NoticeDelivery::BothFirstClassAndCertified,
            30,
            500_00,
            false,
        ));
        assert!(tx.citation.contains("92.0081"));

        let wa = check(&input(
            Regime::Washington,
            true,
            NoticeDelivery::FirstClassMail,
            45,
            500_00,
            false,
        ));
        assert!(wa.citation.contains("RCW 59.18.310"));
    }

    #[test]
    fn ca_personal_delivery_15_lt_first_class_mail_18() {
        // Two delivery methods for same property = different clocks.
        let personal = check(&input(
            Regime::California,
            true,
            NoticeDelivery::PersonalDelivery,
            15,
            500_00,
            false,
        ));
        let mail = check(&input(
            Regime::California,
            true,
            NoticeDelivery::FirstClassMail,
            15,
            500_00,
            false,
        ));
        // Personal delivery compliant at 15 days; first-class mail needs 18.
        assert_eq!(personal.violation, ViolationType::None);
        assert_eq!(mail.violation, ViolationType::InsufficientWaitingPeriod);
    }
}
