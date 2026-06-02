//! Lease renewal offer timing and disclosure framework
//! — when must a landlord send a renewal offer (or
//! non-renewal notice), with what content, and by what
//! method? Trader-landlord critical because failure to
//! provide timely renewal offer can EXTEND tenancy as
//! month-to-month, INVALIDATE rent increase, or
//! FORFEIT landlord's right to non-renew. Distinct
//! from sibling `rent_increase_notice_period`
//! (notice requirements for periodic-tenancy rent
//! increases).
//!
//! Companion to lease_auto_renewal, lease_succession,
//! lease_assignment_consent, lease_copy_delivery,
//! rent_increase_notice_period.
//!
//! **Four-jurisdiction framework**:
//!
//! **New York (rent-stabilized)** — DHCR Form RTP-8
//! plus Rent Stabilization Code § 2523.5: owner must
//! provide written renewal offer by mail or personal
//! delivery NOT MORE THAN 150 DAYS and NOT LESS THAN
//! 90 DAYS before existing lease expires; renewal MUST
//! be on DHCR-promulgated Form RTP-8; tenant has 60
//! DAYS to accept; failure by owner FORFEITS right to
//! raise rent until proper notice given and tenant
//! continues in possession at then-current rent.
//!
//! **New York (HSTPA 2019 NON-STABILIZED)** — NY RPL
//! § 226-c added by HSTPA requires written advance
//! notice for non-renewal OR rent increase >= 5%:
//! 30 days under 1 year, 60 days for 1-2 years, 90
//! days for 2+ years. Failure invalidates non-renewal/
//! increase until compliant notice.
//!
//! **California TPA (AB 1482, 2019)** — Cal. Civ. Code
//! § 1946.2: just-cause eviction regime covering most
//! rentals more than 15 years old; non-renewal requires
//! JUST CAUSE plus written notice; relocation
//! assistance equal to one month's rent for no-fault
//! non-renewal (§ 1946.2(d)).
//!
//! **DC** — D.C. Code § 42-3505.54: Rental Housing Act
//! of 1985 (as amended); landlord must offer 12-month
//! renewal at lease expiration except for specific
//! just-cause grounds enumerated in § 42-3505.01; rent
//! increase notice required 30 days in advance; CPI-
//! tied annual increase cap.
//!
//! **NY Rent-Stabilized renewal content requirements
//! (RTP-8)**:
//! 1. Offer of 1-year OR 2-year renewal term at
//!    tenant's option;
//! 2. Current legal regulated rent;
//! 3. Proposed new rent based on RGB increase
//!    percentages for chosen term;
//! 4. Tenant's right to renew without surcharge
//!    (HSTPA repealed vacancy bonus + longevity
//!    bonus);
//! 5. Notice of major capital improvement (MCI) or
//!    individual apartment improvement (IAI)
//!    surcharge if any;
//! 6. Tenant's right to file challenge with DHCR.
//!
//! **Trader-landlord critical fact patterns**:
//! 1. NYC trader-landlord owns rent-stabilized unit;
//!    sends renewal offer 60 days before expiry —
//!    UNTIMELY under § 2523.5; rent increase
//!    INVALIDATED until proper notice; tenant
//!    continues at current rent.
//! 2. NY non-stabilized trader sends 2-year tenant
//!    45-day non-renewal notice — UNTIMELY under
//!    RPL § 226-c (requires 90 days); tenant entitled
//!    to additional 45 days at current rent until
//!    statutory notice complete.
//! 3. CA trader-landlord seeks non-renewal of TPA-
//!    covered unit without just cause — VIOLATION;
//!    § 1946.2(d) requires one month's rent
//!    relocation assistance.
//! 4. DC trader-landlord seeks non-renewal of 12-
//!    month lease without § 42-3505.01 just cause —
//!    statutory presumption FAVOR OF RENEWAL.
//! 5. NYC trader uses outdated RTP-8 form — RENEWAL
//!    INVALID; tenant retains rights under prior
//!    lease until proper form delivered.
//!
//! Citations: NY DHCR Form RTP-8 (latest version 2024);
//! NY Rent Stabilization Code (9 NYCRR) § 2523.5;
//! NY RPL § 226-c (HSTPA 2019); NY DHCR Fact Sheet
//! #4; HSTPA of 2019 (NY Laws 2019, ch. 36); Cal. Civ.
//! Code § 1946.2 (Tenant Protection Act of 2019 — AB
//! 1482); D.C. Code § 42-3505.01; D.C. Code
//! § 42-3505.54; D.C. Rental Housing Act of 1985 (as
//! amended).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewYorkRentStabilized,
    NewYorkNonStabilized,
    CaliforniaTpa,
    DistrictOfColumbia,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LeaseRenewalOfferTimingInput {
    pub jurisdiction: Jurisdiction,
    /// Days before lease expiration that renewal offer
    /// or non-renewal notice was given.
    pub days_before_expiration_notice_given: u32,
    /// Whether renewal offer was given at all.
    pub renewal_offer_given: bool,
    /// Whether NY rent-stabilized renewal used the
    /// current DHCR Form RTP-8.
    pub ny_used_current_rtp8_form: bool,
    /// Whether renewal offered both 1-year and 2-year
    /// terms at tenant's option (NY rent-stabilized
    /// requirement).
    pub ny_offered_both_one_and_two_year_terms: bool,
    /// Months of tenancy occupied (NY RPL § 226-c
    /// tier determination).
    pub months_of_tenancy: u32,
    /// Whether rent increase 5% or more (HSTPA RPL
    /// § 226-c trigger).
    pub rent_increase_five_percent_or_more: bool,
    /// Whether landlord seeking non-renewal of TPA-
    /// covered (CA) or RHA-covered (DC) unit has just
    /// cause.
    pub just_cause_for_non_renewal: bool,
    /// Whether CA TPA relocation assistance (one month's
    /// rent) was paid for no-fault non-renewal.
    pub ca_relocation_assistance_paid: bool,
    /// Method of delivery (mail or personal delivery).
    pub mail_or_personal_delivery: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LeaseRenewalOfferTimingResult {
    pub required_notice_window_days_min: u32,
    pub required_notice_window_days_max: u32,
    pub timing_compliant: bool,
    pub form_compliant: bool,
    pub content_compliant: bool,
    pub method_compliant: bool,
    pub rent_increase_invalidated: bool,
    pub non_renewal_invalidated: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &LeaseRenewalOfferTimingInput,
) -> LeaseRenewalOfferTimingResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let (window_min, window_max) = match input.jurisdiction {
        Jurisdiction::NewYorkRentStabilized => (90, 150),
        Jurisdiction::NewYorkNonStabilized => match input.months_of_tenancy {
            0..=11 => (30, 365),
            12..=23 => (60, 365),
            _ => (90, 365),
        },
        Jurisdiction::CaliforniaTpa => (60, 365),
        Jurisdiction::DistrictOfColumbia => (30, 365),
        Jurisdiction::Default => (30, 365),
    };

    let timing_compliant = input.days_before_expiration_notice_given >= window_min
        && input.days_before_expiration_notice_given <= window_max;

    let form_compliant = match input.jurisdiction {
        Jurisdiction::NewYorkRentStabilized => input.ny_used_current_rtp8_form,
        _ => true,
    };

    let content_compliant = match input.jurisdiction {
        Jurisdiction::NewYorkRentStabilized => input.ny_offered_both_one_and_two_year_terms,
        _ => true,
    };

    let method_compliant = match input.jurisdiction {
        Jurisdiction::NewYorkRentStabilized => input.mail_or_personal_delivery,
        _ => true,
    };

    let rent_increase_invalidated = !timing_compliant
        && (matches!(input.jurisdiction, Jurisdiction::NewYorkRentStabilized)
            || (matches!(input.jurisdiction, Jurisdiction::NewYorkNonStabilized)
                && input.rent_increase_five_percent_or_more));

    let non_renewal_invalidated = !timing_compliant
        || (matches!(
            input.jurisdiction,
            Jurisdiction::CaliforniaTpa | Jurisdiction::DistrictOfColumbia
        ) && !input.just_cause_for_non_renewal);

    if !input.renewal_offer_given {
        failure_reasons.push(
            "No renewal offer or non-renewal notice given; landlord cannot proceed with non-renewal or rent increase until proper notice delivered".to_string(),
        );
    }

    if !timing_compliant {
        match input.jurisdiction {
            Jurisdiction::NewYorkRentStabilized => failure_reasons.push(format!(
                "NY Rent Stabilization Code (9 NYCRR) § 2523.5 — renewal offer given {} days before expiration; required NOT LESS THAN 90 DAYS and NOT MORE THAN 150 DAYS; rent increase INVALIDATED until proper notice; tenant continues at current rent",
                input.days_before_expiration_notice_given
            )),
            Jurisdiction::NewYorkNonStabilized => failure_reasons.push(format!(
                "NY RPL § 226-c (HSTPA 2019) — notice given {} days before expiration; tenancy {} months requires {} days minimum advance notice; non-renewal/increase ≥ 5% INVALIDATED until compliant notice",
                input.days_before_expiration_notice_given,
                input.months_of_tenancy,
                window_min
            )),
            Jurisdiction::CaliforniaTpa => failure_reasons.push(format!(
                "Cal. Civ. Code § 1946.2 (TPA / AB 1482) — non-renewal notice given {} days before expiration; just-cause grounds plus written notice required; failure invalidates non-renewal",
                input.days_before_expiration_notice_given
            )),
            Jurisdiction::DistrictOfColumbia => failure_reasons.push(format!(
                "D.C. Code § 42-3505.54 + § 42-3505.01 — non-renewal notice given {} days before expiration; 12-month renewal must be offered except for enumerated just-cause grounds",
                input.days_before_expiration_notice_given
            )),
            Jurisdiction::Default => failure_reasons.push(format!(
                "Common-law reasonable-notice standard — {} days insufficient; minimum 30 days advance notice typically required",
                input.days_before_expiration_notice_given
            )),
        }
    }

    if !form_compliant && matches!(input.jurisdiction, Jurisdiction::NewYorkRentStabilized) {
        failure_reasons.push(
            "NY DHCR Form RTP-8 (latest version 2024) — renewal offer NOT on current DHCR-promulgated form; renewal INVALID; tenant retains rights under prior lease until proper form delivered".to_string(),
        );
    }

    if !content_compliant && matches!(input.jurisdiction, Jurisdiction::NewYorkRentStabilized) {
        failure_reasons.push(
            "NY Rent Stabilization Code § 2523.5 — renewal must offer BOTH 1-year AND 2-year terms at tenant's option; failure to offer both terms INVALIDATES renewal".to_string(),
        );
    }

    if !method_compliant && matches!(input.jurisdiction, Jurisdiction::NewYorkRentStabilized) {
        failure_reasons.push(
            "NY Rent Stabilization Code § 2523.5 — renewal must be delivered by MAIL or PERSONAL DELIVERY; alternative methods (email, posting, third-party) INVALID".to_string(),
        );
    }

    if matches!(input.jurisdiction, Jurisdiction::CaliforniaTpa)
        && !input.just_cause_for_non_renewal
    {
        failure_reasons.push(
            "Cal. Civ. Code § 1946.2(b) — TPA-covered unit non-renewal requires JUST CAUSE (at-fault or no-fault grounds enumerated); without just cause, non-renewal VOID".to_string(),
        );
        if !input.ca_relocation_assistance_paid {
            failure_reasons.push(
                "Cal. Civ. Code § 1946.2(d) — no-fault non-renewal requires RELOCATION ASSISTANCE equal to ONE MONTH'S RENT (or rent waiver for final month); failure independently INVALIDATES non-renewal".to_string(),
            );
        }
    }

    if matches!(input.jurisdiction, Jurisdiction::DistrictOfColumbia)
        && !input.just_cause_for_non_renewal
    {
        failure_reasons.push(
            "D.C. Code § 42-3505.01 — RHA-covered unit landlord MUST OFFER 12-MONTH RENEWAL except for enumerated just-cause grounds (nonpayment + violation + landlord personal use + demolition + substantial rehabilitation + sale to qualifying purchaser); presumption FAVOR OF RENEWAL".to_string(),
        );
    }

    if rent_increase_invalidated {
        failure_reasons.push(
            "RENT INCREASE INVALIDATED — tenant continues at CURRENT RENT until landlord delivers compliant renewal/notice; landlord cannot collect increased rent until cure".to_string(),
        );
    }

    if non_renewal_invalidated {
        failure_reasons.push(
            "NON-RENEWAL INVALIDATED — landlord cannot evict at lease expiration; tenancy continues as month-to-month or under prior lease terms until compliant notice given".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "Four-jurisdiction framework: NEW YORK RENT-STABILIZED (NY Rent Stabilization Code (9 NYCRR) § 2523.5 — 90-150 DAY window + DHCR Form RTP-8 + 1-year/2-year option); NEW YORK NON-STABILIZED (NY RPL § 226-c HSTPA 2019 — 30/60/90 day tiers based on tenancy length); CALIFORNIA TPA (Cal. Civ. Code § 1946.2 AB 1482 — just-cause + § 1946.2(d) relocation assistance); D.C. (D.C. Code § 42-3505.54 + § 42-3505.01 — RHA mandatory 12-month renewal except just-cause)".to_string(),
        "NY Rent Stabilization Code (9 NYCRR) § 2523.5 — renewal offer by MAIL or PERSONAL DELIVERY NOT MORE THAN 150 DAYS and NOT LESS THAN 90 DAYS before lease expiration; MUST be on DHCR-promulgated Form RTP-8; tenant has 60 DAYS to accept; failure FORFEITS right to raise rent until proper notice given".to_string(),
        "NY RPL § 226-c (HSTPA 2019) advance notice tiers for NON-STABILIZED units when rent increase ≥ 5% OR non-renewal: (1) tenancy < 1 year → 30 days; (2) 1-2 years → 60 days; (3) 2+ years → 90 days; failure INVALIDATES non-renewal/increase until compliant notice".to_string(),
        "NY DHCR Form RTP-8 (latest 2024) renewal offer content: (1) 1-year OR 2-year term at tenant's option; (2) current legal regulated rent; (3) proposed new rent based on RGB increase percentages; (4) tenant right to renew without vacancy/longevity bonus (HSTPA repealed); (5) notice of MCI or IAI surcharge if any; (6) tenant right to file challenge with DHCR".to_string(),
        "Cal. Civ. Code § 1946.2 TPA (AB 1482, 2019) — JUST-CAUSE EVICTION regime for most rentals more than 15 years old; non-renewal requires (a) at-fault just cause (nonpayment + breach + nuisance + criminal activity + assignment-without-consent) or (b) no-fault just cause (withdrawal from rental market + owner move-in + rehabilitation + government order); § 1946.2(d) requires ONE MONTH'S RENT relocation assistance for no-fault grounds".to_string(),
        "D.C. Code § 42-3505.01 + § 42-3505.54 — Rental Housing Act of 1985 (as amended); landlord MUST OFFER 12-MONTH RENEWAL at lease expiration except for enumerated just-cause grounds; rent increase notice required 30 days in advance; CPI-tied annual increase cap under Rental Housing Commission rules".to_string(),
        "Trader-landlord critical fact patterns: (1) NYC trader sends rent-stabilized renewal 60 days before expiry — UNTIMELY; rent increase INVALIDATED; tenant continues at current rent; (2) NY non-stabilized trader sends 2-year tenant 45-day non-renewal — UNTIMELY under RPL § 226-c (requires 90 days); (3) CA trader seeks TPA non-renewal without just cause — § 1946.2(d) ONE MONTH RELOCATION assistance required for no-fault; (4) DC trader seeks non-renewal without just cause — presumption FAVOR OF RENEWAL; (5) NYC trader uses outdated RTP-8 form — RENEWAL INVALID".to_string(),
        "Companion to lease_auto_renewal (auto-renewal disclosure on initial lease) + lease_succession (post-death/incapacity successor rights) + lease_assignment_consent (assignment vs renewal) + lease_copy_delivery (initial lease delivery) + rent_increase_notice_period (periodic-tenancy rent increase notice)".to_string(),
    ];

    LeaseRenewalOfferTimingResult {
        required_notice_window_days_min: window_min,
        required_notice_window_days_max: window_max,
        timing_compliant,
        form_compliant,
        content_compliant,
        method_compliant,
        rent_increase_invalidated,
        non_renewal_invalidated,
        failure_reasons,
        citation: "NY DHCR Form RTP-8 (2024); NY Rent Stabilization Code (9 NYCRR) § 2523.5; NY RPL § 226-c (HSTPA 2019); NY DHCR Fact Sheet #4; HSTPA of 2019 (NY Laws 2019, ch. 36); Cal. Civ. Code § 1946.2 (Tenant Protection Act of 2019 — AB 1482); D.C. Code § 42-3505.01; D.C. Code § 42-3505.54; D.C. Rental Housing Act of 1985",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ny_stabilized_compliant() -> LeaseRenewalOfferTimingInput {
        LeaseRenewalOfferTimingInput {
            jurisdiction: Jurisdiction::NewYorkRentStabilized,
            days_before_expiration_notice_given: 120,
            renewal_offer_given: true,
            ny_used_current_rtp8_form: true,
            ny_offered_both_one_and_two_year_terms: true,
            months_of_tenancy: 36,
            rent_increase_five_percent_or_more: true,
            just_cause_for_non_renewal: true,
            ca_relocation_assistance_paid: true,
            mail_or_personal_delivery: true,
        }
    }

    #[test]
    fn ny_stabilized_120_day_compliant() {
        let r = check(&ny_stabilized_compliant());
        assert_eq!(r.required_notice_window_days_min, 90);
        assert_eq!(r.required_notice_window_days_max, 150);
        assert!(r.timing_compliant);
        assert!(r.form_compliant);
        assert!(r.content_compliant);
        assert!(r.method_compliant);
        assert!(!r.rent_increase_invalidated);
    }

    #[test]
    fn ny_stabilized_60_day_untimely() {
        let mut i = ny_stabilized_compliant();
        i.days_before_expiration_notice_given = 60;
        let r = check(&i);
        assert!(!r.timing_compliant);
        assert!(r.rent_increase_invalidated);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 2523.5")
            && f.contains("NOT LESS THAN 90 DAYS")));
    }

    #[test]
    fn ny_stabilized_160_day_too_early() {
        let mut i = ny_stabilized_compliant();
        i.days_before_expiration_notice_given = 160;
        let r = check(&i);
        assert!(!r.timing_compliant);
    }

    #[test]
    fn ny_stabilized_90_day_boundary_compliant() {
        let mut i = ny_stabilized_compliant();
        i.days_before_expiration_notice_given = 90;
        let r = check(&i);
        assert!(r.timing_compliant);
    }

    #[test]
    fn ny_stabilized_150_day_boundary_compliant() {
        let mut i = ny_stabilized_compliant();
        i.days_before_expiration_notice_given = 150;
        let r = check(&i);
        assert!(r.timing_compliant);
    }

    #[test]
    fn ny_stabilized_no_rtp8_form_invalid() {
        let mut i = ny_stabilized_compliant();
        i.ny_used_current_rtp8_form = false;
        let r = check(&i);
        assert!(!r.form_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("DHCR Form RTP-8")
            && f.contains("INVALID")));
    }

    #[test]
    fn ny_stabilized_no_both_terms_invalid() {
        let mut i = ny_stabilized_compliant();
        i.ny_offered_both_one_and_two_year_terms = false;
        let r = check(&i);
        assert!(!r.content_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 2523.5")
            && f.contains("BOTH 1-year AND 2-year")));
    }

    #[test]
    fn ny_stabilized_email_delivery_invalid() {
        let mut i = ny_stabilized_compliant();
        i.mail_or_personal_delivery = false;
        let r = check(&i);
        assert!(!r.method_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 2523.5")
            && f.contains("MAIL or PERSONAL DELIVERY")));
    }

    #[test]
    fn ny_non_stabilized_under_one_year_30_day_tier() {
        let mut i = ny_stabilized_compliant();
        i.jurisdiction = Jurisdiction::NewYorkNonStabilized;
        i.months_of_tenancy = 6;
        let r = check(&i);
        assert_eq!(r.required_notice_window_days_min, 30);
    }

    #[test]
    fn ny_non_stabilized_one_to_two_year_60_day_tier() {
        let mut i = ny_stabilized_compliant();
        i.jurisdiction = Jurisdiction::NewYorkNonStabilized;
        i.months_of_tenancy = 18;
        let r = check(&i);
        assert_eq!(r.required_notice_window_days_min, 60);
    }

    #[test]
    fn ny_non_stabilized_over_two_year_90_day_tier() {
        let mut i = ny_stabilized_compliant();
        i.jurisdiction = Jurisdiction::NewYorkNonStabilized;
        i.months_of_tenancy = 36;
        let r = check(&i);
        assert_eq!(r.required_notice_window_days_min, 90);
    }

    #[test]
    fn ny_non_stabilized_under_5_percent_no_invalidation() {
        let mut i = ny_stabilized_compliant();
        i.jurisdiction = Jurisdiction::NewYorkNonStabilized;
        i.rent_increase_five_percent_or_more = false;
        i.days_before_expiration_notice_given = 30;
        i.months_of_tenancy = 36;
        let r = check(&i);
        assert!(!r.rent_increase_invalidated);
    }

    #[test]
    fn ny_non_stabilized_5_plus_percent_invalidated() {
        let mut i = ny_stabilized_compliant();
        i.jurisdiction = Jurisdiction::NewYorkNonStabilized;
        i.rent_increase_five_percent_or_more = true;
        i.days_before_expiration_notice_given = 30;
        i.months_of_tenancy = 36;
        let r = check(&i);
        assert!(!r.timing_compliant);
        assert!(r.rent_increase_invalidated);
    }

    #[test]
    fn ca_tpa_no_just_cause_invalidates() {
        let mut i = ny_stabilized_compliant();
        i.jurisdiction = Jurisdiction::CaliforniaTpa;
        i.just_cause_for_non_renewal = false;
        let r = check(&i);
        assert!(r.non_renewal_invalidated);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 1946.2(b)")
            && f.contains("JUST CAUSE")));
    }

    #[test]
    fn ca_tpa_no_relocation_assistance_invalidates() {
        let mut i = ny_stabilized_compliant();
        i.jurisdiction = Jurisdiction::CaliforniaTpa;
        i.just_cause_for_non_renewal = false;
        i.ca_relocation_assistance_paid = false;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 1946.2(d)")
            && f.contains("ONE MONTH'S RENT")));
    }

    #[test]
    fn dc_no_just_cause_renewal_required() {
        let mut i = ny_stabilized_compliant();
        i.jurisdiction = Jurisdiction::DistrictOfColumbia;
        i.just_cause_for_non_renewal = false;
        let r = check(&i);
        assert!(r.non_renewal_invalidated);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 42-3505.01")
            && f.contains("12-MONTH RENEWAL")));
    }

    #[test]
    fn no_renewal_offer_given_at_all() {
        let mut i = ny_stabilized_compliant();
        i.renewal_offer_given = false;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("No renewal offer or non-renewal notice given")));
    }

    #[test]
    fn jurisdiction_window_truth_table_five_cells() {
        let cases = [
            (Jurisdiction::NewYorkRentStabilized, 36, 90, 150),
            (Jurisdiction::NewYorkNonStabilized, 6, 30, 365),
            (Jurisdiction::NewYorkNonStabilized, 18, 60, 365),
            (Jurisdiction::NewYorkNonStabilized, 36, 90, 365),
            (Jurisdiction::CaliforniaTpa, 36, 60, 365),
            (Jurisdiction::DistrictOfColumbia, 36, 30, 365),
            (Jurisdiction::Default, 36, 30, 365),
        ];
        for (j, months, exp_min, exp_max) in cases {
            let mut i = ny_stabilized_compliant();
            i.jurisdiction = j;
            i.months_of_tenancy = months;
            let r = check(&i);
            assert_eq!(r.required_notice_window_days_min, exp_min, "j={:?} months={}", j, months);
            assert_eq!(r.required_notice_window_days_max, exp_max, "j={:?} months={}", j, months);
        }
    }

    #[test]
    fn ny_stabilized_uniquely_requires_rtp8_form_invariant() {
        let mut ny_st = ny_stabilized_compliant();
        ny_st.ny_used_current_rtp8_form = false;
        let r_ny_st = check(&ny_st);
        assert!(!r_ny_st.form_compliant);

        for j in [
            Jurisdiction::NewYorkNonStabilized,
            Jurisdiction::CaliforniaTpa,
            Jurisdiction::DistrictOfColumbia,
            Jurisdiction::Default,
        ] {
            let mut i = ny_stabilized_compliant();
            i.jurisdiction = j;
            i.ny_used_current_rtp8_form = false;
            let r = check(&i);
            assert!(r.form_compliant, "j={:?}", j);
        }
    }

    #[test]
    fn ca_uniquely_requires_relocation_assistance_invariant() {
        let mut ca = ny_stabilized_compliant();
        ca.jurisdiction = Jurisdiction::CaliforniaTpa;
        ca.just_cause_for_non_renewal = false;
        ca.ca_relocation_assistance_paid = false;
        let r_ca = check(&ca);
        assert!(r_ca.failure_reasons.iter().any(|f|
            f.contains("RELOCATION ASSISTANCE")));

        for j in [
            Jurisdiction::NewYorkRentStabilized,
            Jurisdiction::NewYorkNonStabilized,
            Jurisdiction::DistrictOfColumbia,
            Jurisdiction::Default,
        ] {
            let mut i = ny_stabilized_compliant();
            i.jurisdiction = j;
            i.just_cause_for_non_renewal = false;
            i.ca_relocation_assistance_paid = false;
            let r = check(&i);
            assert!(!r.failure_reasons.iter().any(|f|
                f.contains("RELOCATION ASSISTANCE")), "j={:?}", j);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.citation.contains("DHCR Form RTP-8 (2024)"));
        assert!(r.citation.contains("9 NYCRR"));
        assert!(r.citation.contains("§ 2523.5"));
        assert!(r.citation.contains("NY RPL § 226-c"));
        assert!(r.citation.contains("HSTPA"));
        assert!(r.citation.contains("Cal. Civ. Code § 1946.2"));
        assert!(r.citation.contains("AB 1482"));
        assert!(r.citation.contains("D.C. Code § 42-3505.01"));
        assert!(r.citation.contains("D.C. Code § 42-3505.54"));
        assert!(r.citation.contains("Rental Housing Act of 1985"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Four-jurisdiction framework")
            && n.contains("NEW YORK RENT-STABILIZED")
            && n.contains("90-150 DAY")
            && n.contains("CALIFORNIA TPA")
            && n.contains("D.C.")));
    }

    #[test]
    fn note_pins_ny_2523_5() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 2523.5")
            && n.contains("NOT MORE THAN 150 DAYS")
            && n.contains("NOT LESS THAN 90 DAYS")
            && n.contains("60 DAYS to accept")));
    }

    #[test]
    fn note_pins_ny_rpl_226c_tiers() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("NY RPL § 226-c")
            && n.contains("(1) tenancy < 1 year → 30 days")
            && n.contains("(2) 1-2 years → 60 days")
            && n.contains("(3) 2+ years → 90 days")));
    }

    #[test]
    fn note_pins_rtp8_content_six_elements() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("DHCR Form RTP-8")
            && n.contains("1-year OR 2-year term")
            && n.contains("RGB increase percentages")
            && n.contains("MCI or IAI surcharge")));
    }

    #[test]
    fn note_pins_ca_tpa_just_cause_grounds() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 1946.2 TPA")
            && n.contains("at-fault just cause")
            && n.contains("no-fault just cause")
            && n.contains("ONE MONTH'S RENT relocation")));
    }

    #[test]
    fn note_pins_dc_rha_just_cause() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 42-3505.01")
            && n.contains("12-MONTH RENEWAL")
            && n.contains("Rental Housing Act of 1985")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Trader-landlord critical fact patterns")
            && n.contains("NYC trader sends rent-stabilized renewal 60 days")
            && n.contains("RPL § 226-c")
            && n.contains("ONE MONTH RELOCATION")
            && n.contains("RENEWAL INVALID")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Companion to lease_auto_renewal")
            && n.contains("lease_succession")
            && n.contains("rent_increase_notice_period")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = ny_stabilized_compliant();
        i.days_before_expiration_notice_given = 30;
        i.ny_used_current_rtp8_form = false;
        i.ny_offered_both_one_and_two_year_terms = false;
        i.mail_or_personal_delivery = false;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 4);
    }
}
