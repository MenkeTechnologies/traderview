//! Multi-jurisdictional LANDLORD POST-EVICTION TENANT
//! PROPERTY storage and disposal compliance framework.
//! When a tenant vacates (voluntarily, by eviction, or
//! by abandonment) and leaves personal property in or
//! around the rental unit, what notice procedures,
//! storage durations, valuation thresholds, and disposal
//! pathways apply, and what failure-mode liabilities
//! expose landlord to conversion + statutory damages?
//!
//! Distinct from sibling modules: tenant_abandonment
//! (premises abandonment without eviction), landlord_
//! self_help_eviction_prohibition (illegal lockout/
//! property hold), squatter_unauthorized_occupant_removal
//! (squatter scenario), abandoned_property_handling
//! (general framework), landlord_retaliation_damages.
//!
//! Four-jurisdiction framework:
//!
//! 1. CALIFORNIA (most procedurally detailed) — Cal. Civ.
//!    Code §§ 1980-1991 ("Disposition of Personal Property
//!    Remaining on Premises at Termination of Tenancy"):
//!    - § 1983 — written notice to former tenant + any
//!      other person reasonably believed to be owner;
//!      response deadline at least 15 DAYS after PERSONAL
//!      delivery OR 18 DAYS after MAILING
//!    - § 1984 — notice content: description of property
//!      sufficient for owner identification + location +
//!      reasonable storage cost
//!    - § 1985 — $700 VALUATION THRESHOLD: property
//!      reasonably believed to be worth less than $700 may
//!      be RETAINED OR DISPOSED OF in any manner; property
//!      worth $700 or more requires PUBLIC SALE through
//!      competitive bidding with newspaper notice
//!    - § 1988 — landlord entitled to reimbursement of
//!      storage cost
//!    - § 1989 — tenant RIGHT OF REDEMPTION upon payment
//!      of reasonable storage costs before sale
//!
//! 2. TEXAS — Tex. Prop. Code § 92.0081 (Removal of
//!    Property and Exclusion of Residential Tenant);
//!    § 92.014 (security deposit retained amounts):
//!    landlord may remove and store tenant property
//!    following lawful eviction or abandonment; written
//!    notice must be provided + reasonable storage period
//!    + post-storage sale or disposal procedure.
//!
//! 3. NEW YORK — N.Y. Real Property Actions and
//!    Proceedings Law (RPAPL) § 749(3) post-eviction
//!    property retention duty: warrant of eviction
//!    AUTHORIZES sheriff/marshal to physically remove
//!    tenant property to designated storage location;
//!    landlord must hold property at tenant's reasonable
//!    expense; N.Y. RPL § 235-b implied warranty
//!    overlay; NYC Admin. Code § 26-521 unlawful
//!    eviction protections.
//!
//! 4. DEFAULT — Common-law CONVERSION liability for
//!    unauthorized exercise of dominion over tenant
//!    property; common-law BAILEE LIABILITY for property
//!    held for tenant; tort negligence for damaged or
//!    lost property; implied warranty of habitability
//!    overlay per Hilder v. St. Peter, 478 A.2d 202
//!    (Vt. 1984); state implied warranty statutes such
//!    as Cal. Civ. Code § 1941.1.
//!
//! Universal five failure-mode liability framework:
//! 1. DISPOSAL WITHOUT NOTICE → common-law conversion
//!    liability + state statutory damages; California
//!    Cal. Civ. Code § 1989 minimum $250 statutory damages
//!    plus actual damages plus reasonable attorney fees
//! 2. NOTICE DEFICIENT — missing property description /
//!    location / deadline → notice ineffective + same
//!    conversion liability as no-notice scenario
//! 3. PREMATURE DISPOSAL (before statutory storage window
//!    expires) → conversion + tort negligence + statutory
//!    damages
//! 4. SALE BELOW FAIR MARKET VALUE without public-sale
//!    procedure on $700+ property → conversion + breach
//!    of duty to obtain reasonable value
//! 5. LOSS OR DAMAGE DURING STORAGE → common-law bailee
//!    liability (ordinary care duty) + insurance dispute
//!    over coverage of held-for-tenant property
//!
//! Trader-landlord critical because (1) conversion
//! liability under California Cal. Civ. Code § 1989 is
//! strict-liability — even good-faith disposal without
//! proper notice triggers $250 minimum plus actual
//! damages plus attorney fees; (2) California's $700
//! threshold determines whether public sale required —
//! must value property in good faith before disposal;
//! (3) Texas § 92.0081 requires notice but allows
//! discard if no response; (4) New York requires
//! sheriff-supervised removal under RPAPL § 749(3) —
//! landlord cannot unilaterally remove; (5) bailee
//! liability for property held in storage means landlord
//! owes ordinary-care duty during storage period;
//! (6) cumulative storage costs over months for valuable
//! items can exceed property value, motivating tenant
//! abandonment of redemption.
//!
//! Authority: Cal. Civ. Code §§ 1980-1991 ("Disposition
//! of Personal Property Remaining on Premises at
//! Termination of Tenancy"); Cal. Civ. Code § 1983
//! (notice); Cal. Civ. Code § 1984 (notice content);
//! Cal. Civ. Code § 1985 ($700 threshold); Cal. Civ.
//! Code § 1988 (storage cost reimbursement); Cal. Civ.
//! Code § 1989 (statutory damages); Cal. Civ. Code
//! § 1941.1; Tex. Prop. Code § 92.0081 (removal and
//! exclusion); Tex. Prop. Code § 92.014; N.Y. RPAPL
//! § 749(3) (post-eviction property retention); N.Y.
//! RPL § 235-b (implied warranty of habitability); NYC
//! Admin. Code § 26-521 (unlawful eviction); Hilder v.
//! St. Peter, 478 A.2d 202 (Vt. 1984); Green v. Superior
//! Court, 10 Cal. 3d 616 (1974).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    Texas,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeDeliveryMethod {
    NoneServed,
    PersonalDelivery,
    Mail,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub property_value_cents: u64,
    pub notice_method: NoticeDeliveryMethod,
    pub notice_includes_property_description: bool,
    pub notice_includes_location: bool,
    pub notice_includes_response_deadline: bool,
    pub days_since_notice_served: u32,
    pub tenant_demanded_return_within_deadline: bool,
    pub property_returned_to_tenant: bool,
    pub disposal_action_taken: bool,
    pub public_sale_with_newspaper_notice_completed: bool,
    pub property_damaged_or_lost_during_storage: bool,
    pub ny_sheriff_supervised_removal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Compliant,
    NoticeRequired,
    StorageWindowActive,
    PrematureDisposalConversion,
    PublicSaleRequired,
    BaileeLiability,
    ConversionLiability,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub jurisdiction_specific_actions: Vec<String>,
    pub notes: Vec<String>,
}

pub const CA_PERSONAL_DELIVERY_DEADLINE_DAYS: u32 = 15;
pub const CA_MAIL_DEADLINE_DAYS: u32 = 18;
pub const CA_PUBLIC_SALE_THRESHOLD_CENTS: u64 = 70_000;

pub type LandlordPostEvictionTenantPropertyStorageDisposalInput = Input;
pub type LandlordPostEvictionTenantPropertyStorageDisposalResult = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "Four-jurisdiction framework: California (most procedurally detailed — Cal. Civ. Code §§ 1980-1991 'Disposition of Personal Property Remaining on Premises at Termination of Tenancy': § 1983 written notice + § 1984 content + § 1985 $700 valuation threshold determining public-sale requirement + § 1988 storage cost reimbursement + § 1989 tenant redemption right + $250 minimum statutory damages); Texas (Tex. Prop. Code § 92.0081 removal and exclusion + § 92.014 — written notice required, reasonable storage period, post-storage sale or disposal); New York (N.Y. RPAPL § 749(3) sheriff/marshal supervised removal + N.Y. RPL § 235-b implied warranty + NYC Admin. Code § 26-521 unlawful eviction protections); Default (common-law conversion + bailee liability + tort negligence + implied warranty of habitability per Hilder v. St. Peter 478 A.2d 202 (Vt. 1984) + Cal. Civ. Code § 1941.1).".to_string(),
        "California response deadline per Cal. Civ. Code § 1983: at least 15 DAYS after PERSONAL DELIVERY or 18 DAYS after MAILING of notice. Cal. Civ. Code § 1985 $700 VALUATION THRESHOLD: property reasonably valued under $700 may be retained or disposed of in any manner; property valued $700+ requires public sale through competitive bidding with newspaper notice.".to_string(),
        "Cal. Civ. Code § 1984 notice content requirements: (a) description of property sufficient for owner identification; (b) location where property may be claimed; (c) reasonable storage cost; (d) date by which tenant must respond.".to_string(),
        "Five universal failure-mode liabilities: (1) DISPOSAL WITHOUT NOTICE → common-law conversion + Cal. Civ. Code § 1989 minimum $250 statutory damages plus actual damages plus attorney fees; (2) NOTICE DEFICIENT (missing description/location/deadline) → notice ineffective + same conversion liability; (3) PREMATURE DISPOSAL (before statutory window expires) → conversion + tort negligence; (4) SALE BELOW FAIR MARKET VALUE without public-sale procedure on $700+ property → conversion + breach of duty; (5) LOSS OR DAMAGE during storage → common-law bailee ordinary-care duty breach + insurance dispute.".to_string(),
        "New York sheriff-supervised removal per N.Y. RPAPL § 749(3): warrant of eviction authorizes sheriff/marshal to physically remove tenant property to designated storage location; landlord cannot unilaterally remove; NYC Admin. Code § 26-521 unlawful-eviction protections supplement.".to_string(),
        "Companion modules: tenant_abandonment (premises abandonment without eviction), landlord_self_help_eviction_prohibition (illegal lockout/property hold), squatter_unauthorized_occupant_removal, abandoned_property_handling (general framework), landlord_retaliation_damages.".to_string(),
    ];
    let mut actions: Vec<String> = Vec::new();

    if input.property_returned_to_tenant {
        let mut n = notes;
        n.push("Property returned to tenant — disposal pathway not engaged; compliant.".to_string());
        return Output {
            severity: Severity::Compliant,
            jurisdiction_specific_actions: actions,
            notes: n,
        };
    }

    // Property damaged/lost during storage — bailee liability regardless of notice path
    if input.property_damaged_or_lost_during_storage {
        actions.push("Property damaged or lost during storage: common-law bailee liability (ordinary care duty); cannot rely on disposal-pathway compliance to cure damage-during-storage exposure; insurance dispute likely.".to_string());
    }

    let notice_complete = !matches!(input.notice_method, NoticeDeliveryMethod::NoneServed)
        && input.notice_includes_property_description
        && input.notice_includes_location
        && input.notice_includes_response_deadline;

    if !notice_complete && input.disposal_action_taken {
        actions.push("Disposal action taken without complete notice: common-law conversion exposure; Cal. Civ. Code § 1989 minimum $250 statutory damages + actual damages + attorney fees in California; analogous tort liability in other jurisdictions.".to_string());
    }

    if !notice_complete && !input.disposal_action_taken {
        let missing = if matches!(input.notice_method, NoticeDeliveryMethod::NoneServed) {
            "notice not yet served"
        } else if !input.notice_includes_property_description {
            "notice missing property description"
        } else if !input.notice_includes_location {
            "notice missing location"
        } else {
            "notice missing response deadline"
        };
        actions.push(format!(
            "Notice required before disposal: {} — Cal. Civ. Code § 1984 content requirements (description + location + response deadline + reasonable storage cost).",
            missing
        ));
    }

    let required_deadline_days = match input.notice_method {
        NoticeDeliveryMethod::PersonalDelivery => CA_PERSONAL_DELIVERY_DEADLINE_DAYS,
        NoticeDeliveryMethod::Mail => CA_MAIL_DEADLINE_DAYS,
        NoticeDeliveryMethod::NoneServed => 0,
    };

    let storage_window_active = notice_complete
        && input.days_since_notice_served < required_deadline_days
        && !input.disposal_action_taken;

    let premature_disposal = notice_complete
        && input.disposal_action_taken
        && input.days_since_notice_served < required_deadline_days;

    if premature_disposal {
        actions.push(format!(
            "Premature disposal: only {} days since notice served, less than required {} day window for {:?} delivery; conversion + tort negligence + statutory damages.",
            input.days_since_notice_served, required_deadline_days, input.notice_method
        ));
    }

    let public_sale_required = notice_complete
        && input.property_value_cents >= CA_PUBLIC_SALE_THRESHOLD_CENTS
        && matches!(input.jurisdiction, Jurisdiction::California)
        && input.disposal_action_taken
        && !input.public_sale_with_newspaper_notice_completed;

    if public_sale_required {
        actions.push(format!(
            "California public sale required: property valued ${}.{:02} at or above $700 § 1985 threshold; competitive-bidding public sale with newspaper notice required (NOT direct disposal). Failure exposes landlord to conversion + breach of duty to obtain reasonable value.",
            input.property_value_cents / 100,
            input.property_value_cents % 100
        ));
    }

    if input.tenant_demanded_return_within_deadline && !input.property_returned_to_tenant {
        actions.push("Tenant demanded return within deadline: Cal. Civ. Code § 1989 tenant redemption right upon payment of reasonable storage costs; deny only for non-payment of storage costs.".to_string());
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            actions.push("California: Cal. Civ. Code §§ 1980-1991 'Disposition of Personal Property Remaining on Premises at Termination of Tenancy' — § 1983 notice + § 1984 content + § 1985 $700 threshold + § 1988 storage cost reimbursement + § 1989 redemption right + $250 minimum statutory damages.".to_string());
        }
        Jurisdiction::Texas => {
            actions.push("Texas: Tex. Prop. Code § 92.0081 removal and exclusion of residential tenant + § 92.014 — written notice required + reasonable storage period + post-storage sale or disposal procedure.".to_string());
        }
        Jurisdiction::NewYork => {
            if !input.ny_sheriff_supervised_removal {
                actions.push("New York: N.Y. RPAPL § 749(3) sheriff/marshal supervised removal REQUIRED — landlord cannot unilaterally remove tenant property; warrant of eviction authorizes sheriff/marshal physical removal to designated storage; NYC Admin. Code § 26-521 unlawful eviction.".to_string());
            } else {
                actions.push("New York: sheriff-supervised removal completed per N.Y. RPAPL § 749(3); landlord holds property at tenant's reasonable expense; NYC Admin. Code § 26-521 + N.Y. RPL § 235-b implied warranty overlay.".to_string());
            }
        }
        Jurisdiction::Default => {
            actions.push("Default jurisdiction: common-law conversion liability + bailee ordinary-care duty + tort negligence + implied warranty of habitability per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Cal. Civ. Code § 1941.1.".to_string());
        }
    }

    let ny_unilateral_removal_violation = matches!(input.jurisdiction, Jurisdiction::NewYork)
        && !input.ny_sheriff_supervised_removal
        && input.disposal_action_taken;

    let severity = if input.property_damaged_or_lost_during_storage {
        Severity::BaileeLiability
    } else if (!notice_complete && input.disposal_action_taken) || ny_unilateral_removal_violation {
        Severity::ConversionLiability
    } else if premature_disposal {
        Severity::PrematureDisposalConversion
    } else if public_sale_required {
        Severity::PublicSaleRequired
    } else if storage_window_active {
        Severity::StorageWindowActive
    } else if !notice_complete && !input.disposal_action_taken {
        Severity::NoticeRequired
    } else {
        Severity::Compliant
    };

    Output {
        severity,
        jurisdiction_specific_actions: actions,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            property_value_cents: 500_00, // $500 under $700
            notice_method: NoticeDeliveryMethod::PersonalDelivery,
            notice_includes_property_description: true,
            notice_includes_location: true,
            notice_includes_response_deadline: true,
            days_since_notice_served: 20, // past 15-day deadline
            tenant_demanded_return_within_deadline: false,
            property_returned_to_tenant: false,
            disposal_action_taken: true,
            public_sale_with_newspaper_notice_completed: false,
            property_damaged_or_lost_during_storage: false,
            ny_sheriff_supervised_removal: false,
        }
    }

    #[test]
    fn property_returned_compliant() {
        let mut i = baseline();
        i.property_returned_to_tenant = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn ca_under_700_compliant_after_15_days_personal_delivery() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn ca_personal_delivery_window_active_under_15_days() {
        let mut i = baseline();
        i.days_since_notice_served = 10;
        i.disposal_action_taken = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::StorageWindowActive);
    }

    #[test]
    fn ca_mail_window_active_under_18_days() {
        let mut i = baseline();
        i.notice_method = NoticeDeliveryMethod::Mail;
        i.days_since_notice_served = 16;
        i.disposal_action_taken = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::StorageWindowActive);
    }

    #[test]
    fn ca_mail_window_exactly_18_days_compliant() {
        let mut i = baseline();
        i.notice_method = NoticeDeliveryMethod::Mail;
        i.days_since_notice_served = 18;
        i.disposal_action_taken = true;
        let out = check(&i);
        // 18 days = window has expired (window is days < required, so 18 is past)
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn no_notice_served_disposal_taken_conversion_liability() {
        let mut i = baseline();
        i.notice_method = NoticeDeliveryMethod::NoneServed;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ConversionLiability);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Cal. Civ. Code § 1989"));
        assert!(joined.contains("$250"));
    }

    #[test]
    fn notice_missing_description_conversion_when_disposed() {
        let mut i = baseline();
        i.notice_includes_property_description = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ConversionLiability);
    }

    #[test]
    fn notice_missing_location_conversion_when_disposed() {
        let mut i = baseline();
        i.notice_includes_location = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ConversionLiability);
    }

    #[test]
    fn notice_missing_deadline_conversion_when_disposed() {
        let mut i = baseline();
        i.notice_includes_response_deadline = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ConversionLiability);
    }

    #[test]
    fn notice_required_when_no_disposal_taken_yet() {
        let mut i = baseline();
        i.notice_method = NoticeDeliveryMethod::NoneServed;
        i.disposal_action_taken = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NoticeRequired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("notice not yet served"));
    }

    #[test]
    fn premature_disposal_5_days_personal_delivery() {
        let mut i = baseline();
        i.days_since_notice_served = 5;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PrematureDisposalConversion);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("5 days"));
        assert!(joined.contains("15 day window"));
    }

    #[test]
    fn ca_over_700_no_public_sale_violation() {
        let mut i = baseline();
        i.property_value_cents = 1_500_00; // $1,500 over $700
        let out = check(&i);
        assert_eq!(out.severity, Severity::PublicSaleRequired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("$700 § 1985 threshold"));
        assert!(joined.contains("public sale"));
    }

    #[test]
    fn ca_over_700_with_public_sale_compliant() {
        let mut i = baseline();
        i.property_value_cents = 1_500_00;
        i.public_sale_with_newspaper_notice_completed = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn ca_exactly_700_triggers_public_sale_requirement() {
        let mut i = baseline();
        i.property_value_cents = 700_00; // exactly $700
        let out = check(&i);
        assert_eq!(out.severity, Severity::PublicSaleRequired);
    }

    #[test]
    fn ca_under_700_no_public_sale_required() {
        let mut i = baseline();
        i.property_value_cents = 699_99; // $699.99
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn property_damaged_during_storage_bailee_liability() {
        let mut i = baseline();
        i.property_damaged_or_lost_during_storage = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::BaileeLiability);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("bailee liability"));
        assert!(joined.contains("ordinary care"));
    }

    #[test]
    fn ny_no_sheriff_supervised_removal_unilateral_conversion() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        i.ny_sheriff_supervised_removal = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ConversionLiability);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 749(3)"));
        assert!(joined.contains("sheriff/marshal"));
    }

    #[test]
    fn ny_with_sheriff_supervised_removal_compliant() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        i.ny_sheriff_supervised_removal = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn texas_compliant_baseline() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Texas;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Tex. Prop. Code § 92.0081"));
    }

    #[test]
    fn default_jurisdiction_common_law_conversion() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("common-law conversion"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("§ 1941.1"));
    }

    #[test]
    fn tenant_demanded_return_redemption_right() {
        let mut i = baseline();
        i.tenant_demanded_return_within_deadline = true;
        i.days_since_notice_served = 10;
        i.disposal_action_taken = false;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("redemption right"));
        assert!(joined.contains("§ 1989"));
    }

    #[test]
    fn severity_priority_bailee_above_conversion_above_premature_above_public_sale() {
        let mut i = baseline();
        i.property_damaged_or_lost_during_storage = true;
        i.notice_method = NoticeDeliveryMethod::NoneServed;
        i.days_since_notice_served = 5;
        i.property_value_cents = 1_500_00;
        let out = check(&i);
        // Bailee wins
        assert_eq!(out.severity, Severity::BaileeLiability);
    }

    #[test]
    fn severity_conversion_above_premature_above_public_sale() {
        let mut i = baseline();
        i.notice_method = NoticeDeliveryMethod::NoneServed;
        i.days_since_notice_served = 5;
        i.property_value_cents = 1_500_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ConversionLiability);
    }

    #[test]
    fn severity_premature_above_public_sale() {
        let mut i = baseline();
        i.days_since_notice_served = 5;
        i.property_value_cents = 1_500_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PrematureDisposalConversion);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Cal. Civ. Code §§ 1980-1991"));
        assert!(joined.contains("§ 1983"));
        assert!(joined.contains("§ 1984"));
        assert!(joined.contains("§ 1985"));
        assert!(joined.contains("§ 1988"));
        assert!(joined.contains("§ 1989"));
        assert!(joined.contains("Tex. Prop. Code § 92.0081"));
        assert!(joined.contains("§ 92.014"));
        assert!(joined.contains("N.Y. RPAPL § 749(3)"));
        assert!(joined.contains("RPL § 235-b"));
        assert!(joined.contains("NYC Admin. Code § 26-521"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
        assert!(joined.contains("§ 1941.1"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("California (most procedurally detailed"));
        assert!(joined.contains("Texas"));
        assert!(joined.contains("New York"));
        assert!(joined.contains("Default"));
    }

    #[test]
    fn note_pins_ca_15_18_day_response_window() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("15 DAYS after PERSONAL DELIVERY"));
        assert!(joined.contains("18 DAYS after MAILING"));
    }

    #[test]
    fn note_pins_700_valuation_threshold() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("$700 VALUATION THRESHOLD"));
        assert!(joined.contains("public sale"));
        assert!(joined.contains("competitive bidding"));
    }

    #[test]
    fn note_pins_1984_notice_content() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("description of property"));
        assert!(joined.contains("location where"));
        assert!(joined.contains("reasonable storage cost"));
        assert!(joined.contains("date by which"));
    }

    #[test]
    fn note_pins_five_failure_modes() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("DISPOSAL WITHOUT NOTICE"));
        assert!(joined.contains("NOTICE DEFICIENT"));
        assert!(joined.contains("PREMATURE DISPOSAL"));
        assert!(joined.contains("SALE BELOW FAIR MARKET VALUE"));
        assert!(joined.contains("LOSS OR DAMAGE"));
    }

    #[test]
    fn note_pins_ny_sheriff_supervised_removal() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("N.Y. RPAPL § 749(3)"));
        assert!(joined.contains("sheriff/marshal"));
        assert!(joined.contains("warrant of eviction"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("tenant_abandonment"));
        assert!(joined.contains("landlord_self_help_eviction_prohibition"));
        assert!(joined.contains("squatter_unauthorized_occupant_removal"));
        assert!(joined.contains("abandoned_property_handling"));
        assert!(joined.contains("landlord_retaliation_damages"));
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        let ca = check(&Input {
            jurisdiction: Jurisdiction::California,
            ..baseline()
        });
        let tx = check(&Input {
            jurisdiction: Jurisdiction::Texas,
            ..baseline()
        });
        let ny = check(&Input {
            jurisdiction: Jurisdiction::NewYork,
            ny_sheriff_supervised_removal: true,
            ..baseline()
        });
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            ..baseline()
        });
        assert_eq!(ca.severity, Severity::Compliant);
        assert_eq!(tx.severity, Severity::Compliant);
        assert_eq!(ny.severity, Severity::Compliant);
        assert_eq!(de.severity, Severity::Compliant);
    }

    #[test]
    fn ca_uniquely_700_public_sale_invariant() {
        // CA: $1500 + no public sale → PublicSaleRequired
        let ca = check(&Input {
            jurisdiction: Jurisdiction::California,
            property_value_cents: 1_500_00,
            public_sale_with_newspaper_notice_completed: false,
            ..baseline()
        });
        // TX: $1500 + no public sale → Compliant (no public sale rule modeled)
        let tx = check(&Input {
            jurisdiction: Jurisdiction::Texas,
            property_value_cents: 1_500_00,
            public_sale_with_newspaper_notice_completed: false,
            ..baseline()
        });
        // Only CA triggers public-sale requirement at $700+ threshold
        assert_eq!(ca.severity, Severity::PublicSaleRequired);
        assert_eq!(tx.severity, Severity::Compliant);
    }

    #[test]
    fn multiple_violations_stack_in_actions() {
        let mut i = baseline();
        i.notice_method = NoticeDeliveryMethod::NoneServed;
        i.tenant_demanded_return_within_deadline = true;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        // Includes both conversion and redemption-right notes
        assert!(joined.contains("Cal. Civ. Code § 1989"));
        assert!(joined.contains("redemption"));
    }
}
