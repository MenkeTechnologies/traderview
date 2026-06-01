//! State tenant-death lease termination compliance.
//!
//! When a sole-occupant residential tenant dies before lease
//! expiration, four distinct statutory regimes govern whether and
//! when the lease terminates and what the estate / landlord must do.
//! The default common-law rule (in states without a death-
//! termination statute) is that the lease is a contract that
//! survives death and the deceased tenant's estate remains liable
//! for remaining rent through the end of the term.
//!
//! Five regimes:
//!
//! `EstateRepresentativeTerminatesWith30DayNotice`: TX. Tex. Prop.
//! Code § 92.0162 — a representative of the estate of a sole-
//! occupant tenant who dies before lease expiration may terminate
//! the lease by providing written notice to the landlord, removing
//! the deceased's property, and signing an inventory if required.
//! Termination is effective on the later of the 30th day after
//! notice or the date all conditions are met.
//!
//! `MonthToMonthAutoTerminationOnLastRent`: CA. Cal. Civ. Code
//! § 1934 — when the tenant on a month-to-month tenancy dies, the
//! tenancy is deemed terminated 30 days after the last rent payment
//! made by the deceased; no 30-day notice from the estate is
//! required. Fixed-term leases fall back to common-law contract
//! continuation.
//!
//! `LeaseAutoTerminatesOnDateOfDeath`: VA. Va. Code § 55.1-1256 —
//! the rental agreement is deemed terminated AS OF THE DATE OF
//! DEATH of a sole tenant still residing in the dwelling unit; the
//! landlord need not seek a court order of possession. A separate
//! 10-day written notice goes to the tenant's authorized contact
//! person before the landlord may dispose of personal property.
//!
//! `MultiNoticeStorageRegime`: WA. RCW 59.18.595 — landlord must
//! mail a first written notice to known representatives / emergency
//! contacts / successors; if property is placed in storage, a
//! second notice is required, with the landlord able to sell or
//! dispose on or after a date at least 45 days after the second
//! notice. The lease itself terminates by operation of the act
//! once these procedural steps are satisfied.
//!
//! `NoSpecificStatuteCommonLawContract`: 46 other states + DC. No
//! state-specific tenant-death termination statute confirmed; the
//! lease is a contract that survives the tenant's death and the
//! estate is liable for remaining rent through the end of the term
//! (subject to landlord's general common-law duty to mitigate
//! damages where the jurisdiction recognizes it).
//!
//! Sources:
//! [Tex. Prop. Code § 92.0162 (Justia)](https://law.justia.com/codes/texas/property-code/title-8/chapter-92/subchapter-a/section-92-0162/),
//! [Va. Code § 55.1-1256 (Justia)](https://law.justia.com/codes/virginia/2022/title-55-1/chapter-12/section-55-1-1256/),
//! [RCW 59.18.595 (Justia)](https://law.justia.com/codes/washington/title-59/chapter-59-18/section-59-18-595/),
//! [Cal. Civ. Code § 1934 — month-to-month tenancy on death](https://www.struthers.legal/post/landlord-responsibilities-tenant-death).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantDeathRegime {
    EstateRepresentativeTerminatesWith30DayNotice,
    MonthToMonthAutoTerminationOnLastRent,
    LeaseAutoTerminatesOnDateOfDeath,
    MultiNoticeStorageRegime,
    NoSpecificStatuteCommonLawContract,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: TenantDeathRegime,
    /// Notice period (days) for the estate / representative to
    /// terminate the lease, where the regime requires affirmative
    /// notice. Zero when the lease auto-terminates or the regime
    /// uses a different mechanism (storage, last-rent date).
    pub estate_termination_notice_days: u32,
    /// Property-disposition notice days the landlord must provide
    /// after death before disposing of belongings. Zero where the
    /// regime is silent.
    pub property_disposition_notice_days: u32,
    /// True when the lease is deemed automatically terminated by
    /// operation of law without any affirmative estate notice.
    pub lease_auto_terminates: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: TenantDeathRegime,
    estate_termination_notice_days: u32,
    property_disposition_notice_days: u32,
    lease_auto_terminates: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        estate_termination_notice_days,
        property_disposition_notice_days,
        lease_auto_terminates,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use TenantDeathRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "TX",
        rule(
            EstateRepresentativeTerminatesWith30DayNotice,
            30,
            0,
            false,
            "Tex. Prop. Code § 92.0162 — sole-occupant tenant dies; estate representative may terminate by written notice + property removal + signed inventory; effective on later of 30th day after notice or date all conditions met",
        ),
    );

    m.insert(
        "CA",
        rule(
            MonthToMonthAutoTerminationOnLastRent,
            0,
            0,
            true,
            "Cal. Civ. Code § 1934 — month-to-month tenant dies; tenancy deemed terminated 30 days after last rent payment made by the deceased; no 30-day notice from estate required; fixed-term leases fall to common-law contract continuation",
        ),
    );

    m.insert(
        "VA",
        rule(
            LeaseAutoTerminatesOnDateOfDeath,
            0,
            10,
            true,
            "Va. Code § 55.1-1256 — rental agreement deemed terminated as of the date of death of sole tenant still residing in the dwelling unit; landlord need not seek court order of possession; 10-day property-disposition notice to authorized contact person required before disposal",
        ),
    );

    m.insert(
        "WA",
        rule(
            MultiNoticeStorageRegime,
            0,
            45,
            true,
            "RCW 59.18.595 — landlord must mail first notice to representatives / emergency contacts / successors; if property is placed in storage a second notice is required; landlord may sell or dispose on or after at least 45 days after second notice",
        ),
    );

    // NoSpecificStatuteCommonLawContract default — 46 other states + DC.
    let no_state = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE",
        "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
        "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS",
        "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY",
        "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
        "SD", "TN", "UT", "VT", "WV", "WI", "WY",
    ];
    for code in no_state {
        m.insert(
            code,
            rule(
                NoSpecificStatuteCommonLawContract,
                0,
                0,
                false,
                "No specific tenant-death termination statute confirmed; lease is a contract that survives death; estate remains liable for remaining rent subject to landlord's common-law duty to mitigate where the jurisdiction recognizes it",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDeathInput {
    pub state_code: String,
    /// True if the deceased was the sole occupant of the unit
    /// (regimes 1, 3 require sole-occupancy).
    pub sole_occupant: bool,
    /// True if the lease was a month-to-month tenancy (CA regime
    /// only applies to month-to-month — fixed-term leases fall to
    /// common-law continuation in CA).
    pub month_to_month_tenancy: bool,
    /// Whether the estate representative has provided written notice
    /// to the landlord (TX regime).
    pub estate_provided_written_notice: bool,
    /// Days elapsed since the estate's written notice was served.
    pub days_since_estate_notice: u32,
    /// Whether the deceased's property has been removed from the
    /// premises (TX regime condition).
    pub property_removed_from_premises: bool,
    /// Whether the estate representative has signed the property
    /// inventory if the landlord required one (TX regime condition).
    pub inventory_signed_if_required: bool,
    /// Whether the landlord required a signed inventory in the
    /// first place (TX regime — only relevant when inventory is
    /// required).
    pub inventory_required_by_landlord: bool,
    /// Days elapsed since the date of death.
    pub days_since_date_of_death: u32,
    /// Days elapsed since the last rent payment made by the
    /// deceased tenant (CA month-to-month regime).
    pub days_since_last_rent_payment: u32,
    /// Whether the landlord mailed first notice to representative /
    /// emergency contact / successor (WA regime requirement).
    pub landlord_mailed_first_notice: bool,
    /// Whether the landlord placed property in storage and mailed
    /// the second WA-regime notice.
    pub landlord_mailed_second_notice_with_storage: bool,
    /// Days elapsed since landlord's WA second notice was mailed.
    pub days_since_wa_second_notice: u32,
    /// Days elapsed since landlord's property-disposition notice
    /// (VA regime — 10-day notice to authorized contact).
    pub days_since_property_disposition_notice: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDeathResult {
    pub regime: TenantDeathRegime,
    pub lease_terminated: bool,
    /// True when the landlord may take action to dispose of the
    /// deceased tenant's personal property (regime-specific timing).
    pub landlord_may_dispose_property: bool,
    /// True when the estate has fulfilled all regime-specific
    /// conditions for termination.
    pub estate_conditions_complete: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &TenantDeathInput) -> TenantDeathResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: TenantDeathRegime::NoSpecificStatuteCommonLawContract,
        estate_termination_notice_days: 0,
        property_disposition_notice_days: 0,
        lease_auto_terminates: false,
        citation: "Unknown state code; common-law contract continuation assumed",
    });

    // TX § 92.0162 requires: written notice + property removal +
    // inventory signed if required.
    let tx_conditions_met = input.estate_provided_written_notice
        && input.property_removed_from_premises
        && (!input.inventory_required_by_landlord || input.inventory_signed_if_required);

    let (terminated, may_dispose, estate_complete) = match rule.regime {
        TenantDeathRegime::EstateRepresentativeTerminatesWith30DayNotice => {
            // TX: effective on later of (30 days after notice) or
            // (date all conditions met). Must be sole occupant.
            let sole = input.sole_occupant;
            let notice_30_passed = input.estate_provided_written_notice
                && input.days_since_estate_notice >= rule.estate_termination_notice_days;
            let term = sole && notice_30_passed && tx_conditions_met;
            // Landlord disposition follows termination + property already
            // removed by estate, so this is effectively concurrent.
            (term, term, tx_conditions_met)
        }
        TenantDeathRegime::MonthToMonthAutoTerminationOnLastRent => {
            // CA § 1934: only month-to-month + 30 days after last rent
            // payment.
            if !input.month_to_month_tenancy {
                // Fixed-term lease — common-law continuation.
                (false, false, false)
            } else {
                let term = input.days_since_last_rent_payment >= 30;
                (term, term, true)
            }
        }
        TenantDeathRegime::LeaseAutoTerminatesOnDateOfDeath => {
            // VA § 55.1-1256: lease terminated as of date of death
            // for any sole-occupant tenant. Landlord may dispose of
            // property only after 10-day notice to authorized contact.
            let term = input.sole_occupant;
            let may_dispose = term
                && input.days_since_property_disposition_notice
                    >= rule.property_disposition_notice_days;
            (term, may_dispose, true)
        }
        TenantDeathRegime::MultiNoticeStorageRegime => {
            // WA § 59.18.595: lease terminated by operation; storage
            // disposition requires first notice + second notice + 45 days.
            let term = input.landlord_mailed_first_notice;
            let may_dispose = term
                && input.landlord_mailed_second_notice_with_storage
                && input.days_since_wa_second_notice >= rule.property_disposition_notice_days;
            (term, may_dispose, term)
        }
        TenantDeathRegime::NoSpecificStatuteCommonLawContract => {
            // No statutory termination; estate liable through end of
            // term. Not "terminated" by death.
            (false, false, false)
        }
    };

    let regime_label = match rule.regime {
        TenantDeathRegime::EstateRepresentativeTerminatesWith30DayNotice => {
            "estate-representative 30-day notice"
        }
        TenantDeathRegime::MonthToMonthAutoTerminationOnLastRent => {
            "month-to-month auto-termination on last rent"
        }
        TenantDeathRegime::LeaseAutoTerminatesOnDateOfDeath => "lease auto-terminates on date of death",
        TenantDeathRegime::MultiNoticeStorageRegime => "multi-notice storage regime",
        TenantDeathRegime::NoSpecificStatuteCommonLawContract => {
            "no specific statute — common-law contract continuation"
        }
    };

    let note = if terminated {
        format!(
            "State applies {} regime; lease has terminated on these facts{}.",
            regime_label,
            if may_dispose {
                "; landlord may dispose of property"
            } else { "" },
        )
    } else {
        format!(
            "State applies {} regime; lease NOT yet terminated on these facts (conditions / time period not satisfied or no statutory termination available).",
            regime_label,
        )
    };

    TenantDeathResult {
        regime: rule.regime,
        lease_terminated: terminated,
        landlord_may_dispose_property: may_dispose,
        estate_conditions_complete: estate_complete,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> TenantDeathInput {
        TenantDeathInput {
            state_code: state.to_string(),
            sole_occupant: true,
            month_to_month_tenancy: false,
            estate_provided_written_notice: false,
            days_since_estate_notice: 0,
            property_removed_from_premises: false,
            inventory_signed_if_required: false,
            inventory_required_by_landlord: false,
            days_since_date_of_death: 0,
            days_since_last_rent_payment: 0,
            landlord_mailed_first_notice: false,
            landlord_mailed_second_notice_with_storage: false,
            days_since_wa_second_notice: 0,
            days_since_property_disposition_notice: 0,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn tx_estate_30_day_regime() {
        let r = check(&baseline("TX"));
        assert_eq!(
            r.regime,
            TenantDeathRegime::EstateRepresentativeTerminatesWith30DayNotice
        );
    }

    #[test]
    fn ca_month_to_month_regime() {
        let r = check(&baseline("CA"));
        assert_eq!(
            r.regime,
            TenantDeathRegime::MonthToMonthAutoTerminationOnLastRent
        );
    }

    #[test]
    fn va_auto_terminates_on_death_regime() {
        let r = check(&baseline("VA"));
        assert_eq!(
            r.regime,
            TenantDeathRegime::LeaseAutoTerminatesOnDateOfDeath
        );
    }

    #[test]
    fn wa_multi_notice_storage_regime() {
        let r = check(&baseline("WA"));
        assert_eq!(r.regime, TenantDeathRegime::MultiNoticeStorageRegime);
    }

    #[test]
    fn default_state_common_law_regime() {
        for s in ["AL", "FL", "NY", "PA", "DC", "WY"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                TenantDeathRegime::NoSpecificStatuteCommonLawContract,
                "expected {s} common-law regime"
            );
        }
    }

    // ── TX § 92.0162 conditions ─────────────────────────────────────

    #[test]
    fn tx_all_conditions_met_30_days_terminates() {
        let mut i = baseline("TX");
        i.estate_provided_written_notice = true;
        i.days_since_estate_notice = 30;
        i.property_removed_from_premises = true;
        i.inventory_required_by_landlord = false;
        let r = check(&i);
        assert!(r.lease_terminated);
        assert!(r.estate_conditions_complete);
    }

    #[test]
    fn tx_notice_under_30_days_not_yet_terminated() {
        let mut i = baseline("TX");
        i.estate_provided_written_notice = true;
        i.days_since_estate_notice = 29;
        i.property_removed_from_premises = true;
        let r = check(&i);
        assert!(!r.lease_terminated);
    }

    #[test]
    fn tx_no_property_removal_not_yet_terminated() {
        let mut i = baseline("TX");
        i.estate_provided_written_notice = true;
        i.days_since_estate_notice = 30;
        i.property_removed_from_premises = false;
        let r = check(&i);
        assert!(!r.lease_terminated);
        assert!(!r.estate_conditions_complete);
    }

    #[test]
    fn tx_inventory_required_unsigned_not_yet_terminated() {
        let mut i = baseline("TX");
        i.estate_provided_written_notice = true;
        i.days_since_estate_notice = 30;
        i.property_removed_from_premises = true;
        i.inventory_required_by_landlord = true;
        i.inventory_signed_if_required = false;
        let r = check(&i);
        assert!(!r.estate_conditions_complete);
        assert!(!r.lease_terminated);
    }

    #[test]
    fn tx_inventory_required_signed_terminates() {
        let mut i = baseline("TX");
        i.estate_provided_written_notice = true;
        i.days_since_estate_notice = 30;
        i.property_removed_from_premises = true;
        i.inventory_required_by_landlord = true;
        i.inventory_signed_if_required = true;
        let r = check(&i);
        assert!(r.lease_terminated);
    }

    #[test]
    fn tx_not_sole_occupant_no_termination_right() {
        let mut i = baseline("TX");
        i.sole_occupant = false;
        i.estate_provided_written_notice = true;
        i.days_since_estate_notice = 60;
        i.property_removed_from_premises = true;
        let r = check(&i);
        assert!(
            !r.lease_terminated,
            "§ 92.0162 is sole-occupant only"
        );
    }

    // ── CA § 1934 month-to-month ───────────────────────────────────

    #[test]
    fn ca_month_to_month_30_days_after_last_rent_terminates() {
        let mut i = baseline("CA");
        i.month_to_month_tenancy = true;
        i.days_since_last_rent_payment = 30;
        let r = check(&i);
        assert!(r.lease_terminated);
    }

    #[test]
    fn ca_month_to_month_29_days_not_yet_terminated() {
        let mut i = baseline("CA");
        i.month_to_month_tenancy = true;
        i.days_since_last_rent_payment = 29;
        let r = check(&i);
        assert!(!r.lease_terminated);
    }

    #[test]
    fn ca_fixed_term_lease_falls_to_common_law() {
        // § 1934 only applies to month-to-month — fixed-term leaves
        // the lease in force, estate liable.
        let mut i = baseline("CA");
        i.month_to_month_tenancy = false;
        i.days_since_last_rent_payment = 90;
        let r = check(&i);
        assert!(
            !r.lease_terminated,
            "fixed-term leases in CA fall to common-law continuation"
        );
    }

    // ── VA § 55.1-1256 auto-terminate on date of death ─────────────

    #[test]
    fn va_lease_terminated_on_date_of_death() {
        let mut i = baseline("VA");
        i.sole_occupant = true;
        i.days_since_date_of_death = 0;
        let r = check(&i);
        assert!(r.lease_terminated);
    }

    #[test]
    fn va_dispose_property_only_after_10_day_notice() {
        let mut i = baseline("VA");
        i.sole_occupant = true;
        i.days_since_date_of_death = 30;
        i.days_since_property_disposition_notice = 9;
        let r = check(&i);
        assert!(r.lease_terminated);
        assert!(
            !r.landlord_may_dispose_property,
            "VA requires 10-day notice before disposal"
        );
    }

    #[test]
    fn va_dispose_property_after_10_days_allowed() {
        let mut i = baseline("VA");
        i.sole_occupant = true;
        i.days_since_property_disposition_notice = 10;
        let r = check(&i);
        assert!(r.landlord_may_dispose_property);
    }

    #[test]
    fn va_not_sole_occupant_no_auto_termination() {
        let mut i = baseline("VA");
        i.sole_occupant = false;
        let r = check(&i);
        assert!(
            !r.lease_terminated,
            "VA § 55.1-1256 is sole-occupant only"
        );
    }

    // ── WA RCW 59.18.595 multi-notice ──────────────────────────────

    #[test]
    fn wa_first_notice_alone_terminates_lease() {
        let mut i = baseline("WA");
        i.landlord_mailed_first_notice = true;
        let r = check(&i);
        assert!(r.lease_terminated);
    }

    #[test]
    fn wa_no_first_notice_no_termination() {
        let mut i = baseline("WA");
        i.landlord_mailed_first_notice = false;
        let r = check(&i);
        assert!(!r.lease_terminated);
    }

    #[test]
    fn wa_dispose_requires_second_notice_plus_45_days() {
        let mut i = baseline("WA");
        i.landlord_mailed_first_notice = true;
        i.landlord_mailed_second_notice_with_storage = true;
        i.days_since_wa_second_notice = 45;
        let r = check(&i);
        assert!(r.landlord_may_dispose_property);
    }

    #[test]
    fn wa_dispose_under_45_days_not_yet_allowed() {
        let mut i = baseline("WA");
        i.landlord_mailed_first_notice = true;
        i.landlord_mailed_second_notice_with_storage = true;
        i.days_since_wa_second_notice = 44;
        let r = check(&i);
        assert!(
            !r.landlord_may_dispose_property,
            "WA second notice requires ≥ 45 days before disposal"
        );
    }

    #[test]
    fn wa_dispose_without_second_notice_not_allowed() {
        let mut i = baseline("WA");
        i.landlord_mailed_first_notice = true;
        i.landlord_mailed_second_notice_with_storage = false;
        i.days_since_wa_second_notice = 100;
        let r = check(&i);
        assert!(!r.landlord_may_dispose_property);
    }

    // ── Default common-law states ──────────────────────────────────

    #[test]
    fn default_state_lease_continues_after_death() {
        // No state termination; estate remains liable.
        let r = check(&baseline("FL"));
        assert!(!r.lease_terminated);
    }

    // ── Citations ──────────────────────────────────────────────────

    #[test]
    fn tx_citation_mentions_92_0162_and_30_days() {
        let r = check(&baseline("TX"));
        assert!(r.citation.contains("§ 92.0162"));
        assert!(r.citation.contains("30th day"));
    }

    #[test]
    fn ca_citation_mentions_1934_and_30_days() {
        let r = check(&baseline("CA"));
        assert!(r.citation.contains("§ 1934"));
        assert!(r.citation.contains("30 days"));
    }

    #[test]
    fn va_citation_mentions_55_1_1256() {
        let r = check(&baseline("VA"));
        assert!(r.citation.contains("§ 55.1-1256"));
        assert!(r.citation.contains("10-day"));
    }

    #[test]
    fn wa_citation_mentions_59_18_595_and_45_days() {
        let r = check(&baseline("WA"));
        assert!(r.citation.contains("59.18.595"));
        assert!(r.citation.contains("45 days"));
    }

    // ── Coverage / invariants ──────────────────────────────────────

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let count = RULES.len();
        assert_eq!(count, 51, "expected 50 states + DC, got {count}");
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} empty citation");
        }
    }

    #[test]
    fn tx_is_only_estate_30_day_regime_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    TenantDeathRegime::EstateRepresentativeTerminatesWith30DayNotice
                )
            })
            .count();
        assert_eq!(count, 1, "expected TX only");
    }

    #[test]
    fn ca_is_only_month_to_month_regime_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    TenantDeathRegime::MonthToMonthAutoTerminationOnLastRent
                )
            })
            .count();
        assert_eq!(count, 1, "expected CA only");
    }

    #[test]
    fn va_is_only_auto_terminate_regime_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(r.regime, TenantDeathRegime::LeaseAutoTerminatesOnDateOfDeath)
            })
            .count();
        assert_eq!(count, 1, "expected VA only");
    }

    #[test]
    fn wa_is_only_multi_notice_regime_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| matches!(r.regime, TenantDeathRegime::MultiNoticeStorageRegime))
            .count();
        assert_eq!(count, 1, "expected WA only");
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_terminated_describes_regime() {
        let mut i = baseline("CA");
        i.month_to_month_tenancy = true;
        i.days_since_last_rent_payment = 30;
        let r = check(&i);
        assert!(r.note.contains("month-to-month auto-termination"));
        assert!(r.note.contains("terminated"));
    }

    #[test]
    fn note_not_terminated_describes_regime() {
        let r = check(&baseline("FL"));
        assert!(r.note.contains("common-law contract continuation"));
        assert!(r.note.contains("NOT yet terminated"));
    }

    // ── Normalization ──────────────────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let mut i = baseline("tx");
        i.sole_occupant = true;
        i.estate_provided_written_notice = true;
        i.days_since_estate_notice = 30;
        i.property_removed_from_premises = true;
        let r = check(&i);
        assert_eq!(
            r.regime,
            TenantDeathRegime::EstateRepresentativeTerminatesWith30DayNotice
        );
    }
}
