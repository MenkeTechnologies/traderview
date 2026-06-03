//! Rent concession disclosure framework — when may a
//! landlord offer "free rent" / "first month free" /
//! preferential rent concessions, and how must those
//! concessions be DISCLOSED and CALCULATED? Trader-
//! landlord critical because rent concessions are
//! widely used to ATTRACT TENANTS in soft markets but
//! can become a MAJOR FRAUD-LIABILITY EXPOSURE in
//! rent-regulated jurisdictions where concessions
//! disguise the true legal regulated rent.
//!
//! Companion to lease_disclosures, lease_copy_delivery,
//! tenant_rights_statement_disclosure, lease_waiver_
//! enforceability, lease_renewal_offer_timing (iter
//! 439), landlord_identification_disclosure.
//!
//! **Three-jurisdiction framework**:
//!
//! NEW YORK (rent-stabilized): post-HSTPA 2019,
//! preferential rents CANNOT BE REVOKED during tenancy;
//! all percentage rent increases must be calculated on
//! PREFERENTIAL RENT (not legal regulated rent);
//! DHCR Operational Bulletin 2016-1 (as modified) +
//! Fact Sheet #40 (May 2024) require concession to be
//! AMORTIZED OVER LEASE TERM to determine EFFECTIVE
//! NET RENT for DHCR registration purposes;
//! misrepresentation of concession structure = FRAUD
//! exposing landlord to STATUTORY OVERCHARGE +
//! TREBLE DAMAGES under RSL § 26-516(a).
//!
//! NEW YORK (non-rent-stabilized): rent concessions
//! must be clearly disclosed in the lease (RPL § 235-
//! a + UDAP § 349); concessions cannot disguise the
//! true rent for purposes of rent-payment history,
//! credit reporting, or future renewal increase
//! calculations.
//!
//! CALIFORNIA: Cal. Civ. Code § 1947.15 governs
//! concession-based rent increases in TPA-covered
//! units; AB 1482 § 1947.12 caps annual rent increases
//! at lower of CPI+5% or 10% of LOWEST gross rent
//! charged at any time during prior 12 months;
//! concession disclosure required to prevent gaming
//! the cap.
//!
//! DEFAULT / common law: concessions enforceable as
//! contract modifications but must be in writing if
//! lease exceeds Statute of Frauds threshold; UDAP
//! disclosure rules apply.
//!
//! **NY HSTPA 2019 reforms** (NY Laws 2019, ch. 36):
//! 1. RSL § 26-511(c)(14) — preferential rents
//!    CANNOT be revoked during tenancy;
//! 2. RSL § 26-511(c)(14) — renewal percentage
//!    increases calculated on PREFERENTIAL RENT (not
//!    legal regulated rent);
//! 3. RSL § 26-516(a) — overcharge damages extended
//!    from 4-year to 6-year lookback;
//! 4. RSL § 26-516(a)(2) — TREBLE DAMAGES on
//!    overcharge if landlord cannot prove no willful
//!    overcharge.
//!
//! **DHCR Operational Bulletin 2016-1 + Fact Sheet
//! #40 (May 2024) — concession amortization formula**:
//! ```text
//! Net Effective Monthly Rent = (Gross Monthly Rent ×
//!     Lease Term Months − Total Concession Value)
//!     ÷ Lease Term Months
//! ```
//!
//! Trader-critical fact patterns:
//!
//! 1. NY rent-stabilized: landlord offers 2 months
//!    free on a 12-month lease at $3,000/month; gross
//!    annual rent $36,000 minus $6,000 concession
//!    = $30,000 effective annual; $2,500/month net
//!    effective rent must be registered with DHCR as
//!    the preferential rent and used for future
//!    renewal increases.
//! 2. NY rent-stabilized: landlord registers gross
//!    $3,000/month with DHCR but writes "preferential
//!    rent $2,500/month" in lease — FAILURE TO
//!    AMORTIZE concession = FRAUD; tenant can sue
//!    for treble damages on 6-year lookback under
//!    RSL § 26-516(a).
//! 3. NY rent-stabilized: post-HSTPA, landlord
//!    cannot revoke preferential rent at renewal —
//!    rent increase calculated on preferential rent,
//!    not legal regulated rent; pre-HSTPA "two-tier"
//!    structure ELIMINATED.
//! 4. CA TPA: landlord offers $500/month concession;
//!    annual increase computed on LOWEST gross rent
//!    in prior 12 months — concession floor caps
//!    future increases.
//! 5. NY non-stabilized: landlord credit-reports
//!    full $3,000/month with concession unwound at
//!    end of lease — tenant has UDAP § 349 claim if
//!    credit history misrepresented.
//!
//! Citations: NY RSL (Rent Stabilization Law)
//! § 26-511(c)(14); NY RSL § 26-516(a); NY RSL
//! § 26-516(a)(2); HSTPA of 2019 (NY Laws 2019,
//! ch. 36); DHCR Operational Bulletin 2016-1
//! (as modified by 2019 Bulletin); DHCR Fact Sheet
//! #40 (Preferential Rents, May 2024); NY RPL
//! § 235-a; NY GBL § 349 (UDAP); Cal. Civ. Code
//! § 1947.12 (AB 1482 Tenant Protection Act);
//! Cal. Civ. Code § 1947.15.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewYorkRentStabilized,
    NewYorkNonStabilized,
    California,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentConcessionDisclosureInput {
    pub jurisdiction: Jurisdiction,
    /// Gross monthly rent in cents.
    pub gross_monthly_rent_cents: u64,
    /// Total concession value in cents (e.g. 2 months
    /// free = 2 × gross_monthly_rent).
    pub total_concession_value_cents: u64,
    /// Lease term in months.
    pub lease_term_months: u64,
    /// Whether lease clearly discloses the concession
    /// (RPL § 235-a + UDAP requirements).
    pub lease_discloses_concession: bool,
    /// Whether NY rent-stabilized landlord registered
    /// the NET EFFECTIVE rent (amortized) with DHCR.
    pub ny_registered_net_effective_with_dhcr: bool,
    /// Whether landlord attempts to revoke preferential
    /// rent at renewal (post-HSTPA prohibited).
    pub landlord_attempts_revoke_preferential_at_renewal: bool,
    /// Whether renewal rent increase calculated on
    /// preferential (post-HSTPA correct) or legal
    /// regulated rent (post-HSTPA incorrect).
    pub renewal_increase_on_preferential: bool,
    /// Whether landlord credit-reports gross (not
    /// effective) rent — relevant for non-stabilized
    /// UDAP claim.
    pub credit_reports_gross_not_effective: bool,
    /// Lowest gross rent charged in prior 12 months
    /// in cents (CA AB 1482 cap basis).
    pub ca_lowest_gross_rent_prior_12_months_cents: u64,
    /// Proposed CA renewal rent in cents.
    pub ca_proposed_renewal_rent_cents: u64,
    /// CPI percentage for CA AB 1482 cap (typically 3-5%).
    pub ca_cpi_percent: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentConcessionDisclosureResult {
    pub net_effective_monthly_rent_cents: u64,
    pub disclosure_compliant: bool,
    pub ny_dhcr_registration_compliant: bool,
    pub ny_preferential_rent_revocation_prohibited: bool,
    pub ny_renewal_calculation_compliant: bool,
    pub ca_ab1482_cap_cents: u64,
    pub ca_rent_increase_compliant: bool,
    pub fraud_exposure_engaged: bool,
    pub treble_damages_available: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &RentConcessionDisclosureInput,
) -> RentConcessionDisclosureResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let gross_total = input
        .gross_monthly_rent_cents
        .saturating_mul(input.lease_term_months);
    let net_total = gross_total.saturating_sub(input.total_concession_value_cents);
    let net_effective_monthly_rent_cents = net_total
        .checked_div(input.lease_term_months)
        .unwrap_or(input.gross_monthly_rent_cents);

    let disclosure_compliant = input.lease_discloses_concession;

    let ny_dhcr_registration_compliant = match input.jurisdiction {
        Jurisdiction::NewYorkRentStabilized => input.ny_registered_net_effective_with_dhcr,
        _ => true,
    };

    let ny_preferential_rent_revocation_prohibited = matches!(
        input.jurisdiction,
        Jurisdiction::NewYorkRentStabilized
    );

    let ny_renewal_calculation_compliant = match input.jurisdiction {
        Jurisdiction::NewYorkRentStabilized => {
            !input.landlord_attempts_revoke_preferential_at_renewal
                && input.renewal_increase_on_preferential
        }
        _ => true,
    };

    let ca_ab1482_cap_cents = if matches!(input.jurisdiction, Jurisdiction::California) {
        let cpi_plus_5 = input
            .ca_lowest_gross_rent_prior_12_months_cents
            .saturating_mul((input.ca_cpi_percent as u64) + 5)
            / 100;
        let ten_percent = input
            .ca_lowest_gross_rent_prior_12_months_cents
            .saturating_mul(10)
            / 100;
        let lower_cap = cpi_plus_5.min(ten_percent);
        input
            .ca_lowest_gross_rent_prior_12_months_cents
            .saturating_add(lower_cap)
    } else {
        0
    };

    let ca_rent_increase_compliant = match input.jurisdiction {
        Jurisdiction::California => input.ca_proposed_renewal_rent_cents <= ca_ab1482_cap_cents,
        _ => true,
    };

    let fraud_exposure_engaged = matches!(input.jurisdiction, Jurisdiction::NewYorkRentStabilized)
        && !ny_dhcr_registration_compliant;

    let treble_damages_available = fraud_exposure_engaged;

    if !input.lease_discloses_concession {
        failure_reasons.push(
            "Lease FAILS to clearly disclose concession; NY RPL § 235-a + NY GBL § 349 (UDAP) require clear disclosure; tenant has UDAP claim for deceptive practices; statutory damages + attorney's fees available".to_string(),
        );
    }

    if matches!(input.jurisdiction, Jurisdiction::NewYorkRentStabilized) {
        if !input.ny_registered_net_effective_with_dhcr {
            failure_reasons.push(format!(
                "NY DHCR Operational Bulletin 2016-1 + Fact Sheet #40 (May 2024) — landlord FAILED to register NET EFFECTIVE monthly rent ({} cents) with DHCR; registering only gross monthly rent disguises true preferential rent; FRAUD EXPOSURE under HSTPA 2019",
                net_effective_monthly_rent_cents
            ));
        }

        if input.landlord_attempts_revoke_preferential_at_renewal {
            failure_reasons.push(
                "NY RSL § 26-511(c)(14) (HSTPA 2019) — landlord CANNOT REVOKE preferential rent during tenancy; pre-HSTPA two-tier structure ELIMINATED; preferential rent locked in as base for all future renewal increases".to_string(),
            );
        }

        if !input.renewal_increase_on_preferential {
            failure_reasons.push(
                "NY RSL § 26-511(c)(14) (HSTPA 2019) — renewal percentage increase MUST be calculated on PREFERENTIAL RENT (not legal regulated rent); calculating on legal regulated rent VIOLATES HSTPA + exposes landlord to overcharge claim".to_string(),
            );
        }

        if fraud_exposure_engaged {
            failure_reasons.push(
                "NY RSL § 26-516(a) (HSTPA 2019) — overcharge damages calculated on 6-YEAR LOOKBACK (extended from prior 4-year limit); RSL § 26-516(a)(2) — TREBLE DAMAGES on overcharge if landlord cannot prove no willful overcharge; failure to register NET EFFECTIVE rent = presumptive willful overcharge".to_string(),
            );
        }
    }

    if matches!(input.jurisdiction, Jurisdiction::NewYorkNonStabilized)
        && input.credit_reports_gross_not_effective
    {
        failure_reasons.push(
            "NY GBL § 349 (UDAP) — landlord credit-reporting GROSS rent (not net effective rent) misrepresents tenant's payment history to consumer reporting agencies; deceptive practice exposes landlord to UDAP § 349 claim with statutory damages + attorney's fees".to_string(),
        );
    }

    if matches!(input.jurisdiction, Jurisdiction::California) && !ca_rent_increase_compliant {
        failure_reasons.push(format!(
            "Cal. Civ. Code § 1947.12 (AB 1482 Tenant Protection Act) — proposed renewal rent {} cents EXCEEDS AB 1482 cap of {} cents (LOWEST gross rent in prior 12 months + lower of CPI+5% or 10%); excess INVALID and tenant entitled to refund + statutory penalty",
            input.ca_proposed_renewal_rent_cents, ca_ab1482_cap_cents
        ));
    }

    let notes: Vec<String> = vec![
        "Three-jurisdiction framework: NEW YORK RENT-STABILIZED (NY RSL § 26-511(c)(14) HSTPA 2019 preferential-rent lock-in + DHCR Operational Bulletin 2016-1 amortization formula); NEW YORK NON-STABILIZED (NY RPL § 235-a + NY GBL § 349 UDAP disclosure); CALIFORNIA (Cal. Civ. Code § 1947.12 AB 1482 lowest-rent-prior-12-months cap)".to_string(),
        "NY HSTPA 2019 (NY Laws 2019, ch. 36) reforms: (1) RSL § 26-511(c)(14) preferential rents CANNOT be revoked during tenancy; (2) renewal percentage increases calculated on PREFERENTIAL RENT (not legal regulated rent); (3) RSL § 26-516(a) overcharge damages extended from 4-YEAR to 6-YEAR LOOKBACK; (4) RSL § 26-516(a)(2) TREBLE DAMAGES on overcharge if landlord cannot prove no willful overcharge".to_string(),
        "DHCR Operational Bulletin 2016-1 + Fact Sheet #40 (May 2024) — concession amortization formula: Net Effective Monthly Rent = (Gross Monthly Rent × Lease Term Months − Total Concession Value) ÷ Lease Term Months; failure to register NET EFFECTIVE rent with DHCR disguises preferential rent and triggers HSTPA fraud exposure".to_string(),
        "Trader-critical fact patterns: (1) NY rent-stabilized 2 months free on 12-month $3K lease → $30K annual = $2,500/month net effective preferential rent must be registered with DHCR; (2) registering gross $3,000 with side-letter preferential = FRAUD; treble damages on 6-year lookback; (3) post-HSTPA cannot revoke preferential at renewal — increase calculated on preferential rent; (4) CA TPA $500/month concession → annual increase computed on LOWEST gross prior 12 months; (5) NY non-stabilized credit reports gross not effective = UDAP § 349 claim".to_string(),
        "Cal. Civ. Code § 1947.12 (AB 1482 Tenant Protection Act of 2019) — caps annual rent increases at LOWER of CPI+5% or 10% of LOWEST gross rent charged at any time during prior 12 months; concession disclosure required to prevent gaming the cap; § 1947.15 governs how concessions interact with cap calculation".to_string(),
        "NY RPL § 235-a + NY GBL § 349 (UDAP) — clear concession disclosure required in lease; failure to disclose or misrepresentation of concession structure exposes landlord to UDAP claim with statutory damages + attorney's fees; deceptive practices include credit-reporting gross rent when net effective rent is in fact lower".to_string(),
        "Pre-HSTPA two-tier structure ELIMINATED — pre-2019, NY landlords could charge preferential rent in initial lease and revoke at renewal (charging full legal regulated rent); HSTPA § 26-511(c)(14) locks in preferential rent as base for all future renewal increases".to_string(),
        "Companion to lease_disclosures (initial lease delivery + disclosures) + lease_copy_delivery + tenant_rights_statement_disclosure + lease_waiver_enforceability + lease_renewal_offer_timing (renewal procedure) + landlord_identification_disclosure".to_string(),
    ];

    RentConcessionDisclosureResult {
        net_effective_monthly_rent_cents,
        disclosure_compliant,
        ny_dhcr_registration_compliant,
        ny_preferential_rent_revocation_prohibited,
        ny_renewal_calculation_compliant,
        ca_ab1482_cap_cents,
        ca_rent_increase_compliant,
        fraud_exposure_engaged,
        treble_damages_available,
        failure_reasons,
        citation: "NY RSL (Rent Stabilization Law) § 26-511(c)(14); NY RSL § 26-516(a); NY RSL § 26-516(a)(2); HSTPA of 2019 (NY Laws 2019, ch. 36); DHCR Operational Bulletin 2016-1 (as modified 2019); DHCR Fact Sheet #40 (Preferential Rents, May 2024); NY RPL § 235-a; NY GBL § 349 (UDAP); Cal. Civ. Code § 1947.12 (AB 1482 Tenant Protection Act); Cal. Civ. Code § 1947.15",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ny_stabilized_compliant() -> RentConcessionDisclosureInput {
        RentConcessionDisclosureInput {
            jurisdiction: Jurisdiction::NewYorkRentStabilized,
            gross_monthly_rent_cents: 300_000,
            total_concession_value_cents: 600_000,
            lease_term_months: 12,
            lease_discloses_concession: true,
            ny_registered_net_effective_with_dhcr: true,
            landlord_attempts_revoke_preferential_at_renewal: false,
            renewal_increase_on_preferential: true,
            credit_reports_gross_not_effective: false,
            ca_lowest_gross_rent_prior_12_months_cents: 0,
            ca_proposed_renewal_rent_cents: 0,
            ca_cpi_percent: 0,
        }
    }

    #[test]
    fn ny_stabilized_net_effective_correct() {
        let r = check(&ny_stabilized_compliant());
        assert_eq!(r.net_effective_monthly_rent_cents, 250_000);
    }

    #[test]
    fn ny_stabilized_compliant_passes() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.disclosure_compliant);
        assert!(r.ny_dhcr_registration_compliant);
        assert!(r.ny_renewal_calculation_compliant);
        assert!(!r.fraud_exposure_engaged);
    }

    #[test]
    fn ny_stabilized_failure_to_register_net_effective_fraud() {
        let mut i = ny_stabilized_compliant();
        i.ny_registered_net_effective_with_dhcr = false;
        let r = check(&i);
        assert!(!r.ny_dhcr_registration_compliant);
        assert!(r.fraud_exposure_engaged);
        assert!(r.treble_damages_available);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("DHCR Operational Bulletin 2016-1")
            && f.contains("NET EFFECTIVE")
            && f.contains("FRAUD")));
    }

    #[test]
    fn ny_stabilized_revoke_preferential_at_renewal_prohibited() {
        let mut i = ny_stabilized_compliant();
        i.landlord_attempts_revoke_preferential_at_renewal = true;
        let r = check(&i);
        assert!(!r.ny_renewal_calculation_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 26-511(c)(14)")
            && f.contains("CANNOT REVOKE preferential rent")));
    }

    #[test]
    fn ny_stabilized_renewal_on_legal_not_preferential_violates_hstpa() {
        let mut i = ny_stabilized_compliant();
        i.renewal_increase_on_preferential = false;
        let r = check(&i);
        assert!(!r.ny_renewal_calculation_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 26-511(c)(14)")
            && f.contains("PREFERENTIAL RENT")
            && f.contains("legal regulated rent")));
    }

    #[test]
    fn ny_stabilized_treble_damages_six_year_lookback() {
        let mut i = ny_stabilized_compliant();
        i.ny_registered_net_effective_with_dhcr = false;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 26-516(a)")
            && f.contains("6-YEAR LOOKBACK")
            && f.contains("TREBLE DAMAGES")));
    }

    #[test]
    fn ny_non_stabilized_credit_report_gross_udap_claim() {
        let mut i = ny_stabilized_compliant();
        i.jurisdiction = Jurisdiction::NewYorkNonStabilized;
        i.credit_reports_gross_not_effective = true;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 349 (UDAP)")
            && f.contains("credit-reporting GROSS rent")
            && f.contains("deceptive practice")));
    }

    #[test]
    fn no_disclosure_in_lease_udap_violation() {
        let mut i = ny_stabilized_compliant();
        i.lease_discloses_concession = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("FAILS to clearly disclose concession")
            && f.contains("UDAP")));
    }

    #[test]
    fn ca_ab1482_cap_calculation_cpi_3_percent() {
        let mut i = ny_stabilized_compliant();
        i.jurisdiction = Jurisdiction::California;
        i.ca_lowest_gross_rent_prior_12_months_cents = 200_000;
        i.ca_proposed_renewal_rent_cents = 215_000;
        i.ca_cpi_percent = 3;
        let r = check(&i);
        assert_eq!(r.ca_ab1482_cap_cents, 200_000 + (200_000 * 8 / 100));
        assert!(r.ca_rent_increase_compliant);
    }

    #[test]
    fn ca_ab1482_cap_cpi_10_percent_capped_at_10_floor() {
        let mut i = ny_stabilized_compliant();
        i.jurisdiction = Jurisdiction::California;
        i.ca_lowest_gross_rent_prior_12_months_cents = 200_000;
        i.ca_proposed_renewal_rent_cents = 220_000;
        i.ca_cpi_percent = 10;
        let r = check(&i);
        assert_eq!(r.ca_ab1482_cap_cents, 200_000 + (200_000 * 10 / 100));
        assert!(r.ca_rent_increase_compliant);
    }

    #[test]
    fn ca_ab1482_excess_violation() {
        let mut i = ny_stabilized_compliant();
        i.jurisdiction = Jurisdiction::California;
        i.ca_lowest_gross_rent_prior_12_months_cents = 200_000;
        i.ca_proposed_renewal_rent_cents = 230_000;
        i.ca_cpi_percent = 3;
        let r = check(&i);
        assert!(!r.ca_rent_increase_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 1947.12")
            && f.contains("AB 1482")
            && f.contains("EXCEEDS AB 1482 cap")));
    }

    #[test]
    fn three_month_free_24_month_lease_eighth_off() {
        let mut i = ny_stabilized_compliant();
        i.gross_monthly_rent_cents = 400_000;
        i.total_concession_value_cents = 1_200_000;
        i.lease_term_months = 24;
        let r = check(&i);
        assert_eq!(r.net_effective_monthly_rent_cents, 350_000);
    }

    #[test]
    fn no_concession_net_equals_gross() {
        let mut i = ny_stabilized_compliant();
        i.total_concession_value_cents = 0;
        let r = check(&i);
        assert_eq!(r.net_effective_monthly_rent_cents, 300_000);
    }

    #[test]
    fn zero_lease_term_uses_gross() {
        let mut i = ny_stabilized_compliant();
        i.lease_term_months = 0;
        let r = check(&i);
        assert_eq!(r.net_effective_monthly_rent_cents, 300_000);
    }

    #[test]
    fn ny_stabilized_uniquely_prohibits_revocation_invariant() {
        let ny_st = ny_stabilized_compliant();
        let r_ny_st = check(&ny_st);
        assert!(r_ny_st.ny_preferential_rent_revocation_prohibited);

        for j in [
            Jurisdiction::NewYorkNonStabilized,
            Jurisdiction::California,
            Jurisdiction::Default,
        ] {
            let mut i = ny_stabilized_compliant();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(!r.ny_preferential_rent_revocation_prohibited, "j={:?}", j);
        }
        // Touch to silence unused-mut warnings if any
        let _ = ny_st.jurisdiction;
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        for (j, expect_fraud_possible) in [
            (Jurisdiction::NewYorkRentStabilized, true),
            (Jurisdiction::NewYorkNonStabilized, false),
            (Jurisdiction::California, false),
            (Jurisdiction::Default, false),
        ] {
            let mut i = ny_stabilized_compliant();
            i.jurisdiction = j;
            i.ny_registered_net_effective_with_dhcr = false;
            let r = check(&i);
            assert_eq!(
                r.fraud_exposure_engaged, expect_fraud_possible,
                "j={:?}", j
            );
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.citation.contains("NY RSL"));
        assert!(r.citation.contains("§ 26-511(c)(14)"));
        assert!(r.citation.contains("§ 26-516(a)"));
        assert!(r.citation.contains("§ 26-516(a)(2)"));
        assert!(r.citation.contains("HSTPA of 2019"));
        assert!(r.citation.contains("DHCR Operational Bulletin 2016-1"));
        assert!(r.citation.contains("DHCR Fact Sheet #40"));
        assert!(r.citation.contains("NY RPL § 235-a"));
        assert!(r.citation.contains("NY GBL § 349"));
        assert!(r.citation.contains("Cal. Civ. Code § 1947.12"));
        assert!(r.citation.contains("AB 1482"));
        assert!(r.citation.contains("Cal. Civ. Code § 1947.15"));
    }

    #[test]
    fn note_pins_three_jurisdiction_framework() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Three-jurisdiction framework")
            && n.contains("NEW YORK RENT-STABILIZED")
            && n.contains("NEW YORK NON-STABILIZED")
            && n.contains("CALIFORNIA")));
    }

    #[test]
    fn note_pins_hstpa_four_reforms() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("NY HSTPA 2019")
            && n.contains("§ 26-511(c)(14)")
            && n.contains("CANNOT be revoked")
            && n.contains("§ 26-516(a)")
            && n.contains("4-YEAR to 6-YEAR LOOKBACK")
            && n.contains("TREBLE DAMAGES")));
    }

    #[test]
    fn note_pins_dhcr_amortization_formula() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("DHCR Operational Bulletin 2016-1")
            && n.contains("Fact Sheet #40")
            && n.contains("Net Effective Monthly Rent")
            && n.contains("Lease Term Months")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Trader-critical fact patterns")
            && n.contains("2 months free on 12-month")
            && n.contains("FRAUD")
            && n.contains("UDAP § 349")));
    }

    #[test]
    fn note_pins_ca_ab1482() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 1947.12")
            && n.contains("AB 1482 Tenant Protection Act")
            && n.contains("CPI+5% or 10%")
            && n.contains("LOWEST gross rent")));
    }

    #[test]
    fn note_pins_ny_rpl_235a_udap() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("NY RPL § 235-a")
            && n.contains("NY GBL § 349 (UDAP)")
            && n.contains("deceptive practices")));
    }

    #[test]
    fn note_pins_pre_hstpa_two_tier_elimination() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Pre-HSTPA two-tier structure ELIMINATED")
            && n.contains("locks in preferential rent")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&ny_stabilized_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Companion to lease_disclosures")
            && n.contains("lease_renewal_offer_timing")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = ny_stabilized_compliant();
        i.lease_discloses_concession = false;
        i.ny_registered_net_effective_with_dhcr = false;
        i.landlord_attempts_revoke_preferential_at_renewal = true;
        i.renewal_increase_on_preferential = false;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 4);
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = ny_stabilized_compliant();
        i.gross_monthly_rent_cents = u64::MAX;
        i.lease_term_months = u64::MAX;
        let r = check(&i);
        let _ = r.net_effective_monthly_rent_cents;
    }
}
