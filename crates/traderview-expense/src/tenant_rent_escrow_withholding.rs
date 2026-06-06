//! Multi-jurisdictional tenant rent escrow / rent
//! withholding for habitability violations framework.
//! Trader-landlord critical because the implied
//! warranty of habitability is among the most powerful
//! tenant defenses — a tenant who establishes
//! habitability violation can REDUCE OR WITHHOLD RENT
//! ENTIRELY without losing possession, and many state
//! waivers are VOID as contrary to public policy. Many
//! states ALSO authorize TENANT REPAIR-AND-DEDUCT with
//! statutory damage caps per repair and per year.
//!
//! Companion to rental_carbon_monoxide_detector,
//! rental_basement_water_intrusion_disclosure,
//! rental_bedroom_egress_window, landlord_repair_response_
//! timeframe.
//!
//! **California Civil Code § 1942 + § 1942.4 — Repair
//! and Deduct + Rent Withholding**:
//! 1. § 1942(a) — tenant may repair and deduct cost from
//!    rent up to ONE MONTH'S RENT per repair after
//!    landlord fails to repair within reasonable time.
//! 2. § 1942(a) — limited to TWICE in any 12-MONTH
//!    PERIOD.
//! 3. § 1942(b) — 30-DAY notice creates reasonable-time
//!    PRESUMPTION (rebuttable based on circumstances).
//! 4. § 1942.4 — landlord PROHIBITED from demanding rent,
//!    increasing rent, OR terminating tenancy when:
//!    - Building substandard per Health & Safety Code
//!      § 17920.3; AND
//!    - 35 DAYS since governmental notice to landlord;
//!      AND
//!    - Violation not caused by tenant.
//!
//! § 1942.4 violation entitles tenant to ACTUAL DAMAGES
//! AND STATUTORY DAMAGES OF $100 to $5,000 AND reasonable
//! attorney fees AND costs (per § 1942.4(b)).
//!
//! **New York Real Property Law § 235-b — Warranty of
//! Habitability + Rent Withholding**:
//! 1. § 235-b(1) — IMPLIED warranty in EVERY written or
//!    oral lease that premises are fit for human
//!    habitation and not dangerous/hazardous/detrimental
//!    to life/health/safety.
//! 2. § 235-b(2) — WAIVER VOID as contrary to public
//!    policy.
//! 3. § 235-b(3) — tenant remedies include:
//!    - RENT ABATEMENT (court-determined reduction);
//!    - REPAIR AND DEDUCT (in extenuating circumstances);
//!    - RENT WITHHOLDING (defense to nonpayment summary
//!      proceeding under RPAPL § 711);
//!    - DAMAGES action;
//!    - ATTORNEY FEES and costs.
//!
//! Established by N.Y. Court of Appeals in Park West
//! Management Corp. v. Mitchell, 47 N.Y.2d 316 (1979).
//!
//! **Massachusetts G.L. c. 239 § 8A — Rent Withholding
//! Defense**:
//! 1. Tenant may withhold rent if:
//!    - Conditions ENDANGER or MATERIALLY IMPAIR health,
//!      safety, or well-being;
//!    - Tenant has reported to LOCAL BOARD OF HEALTH or
//!      similar code enforcement agency;
//!    - Landlord has been notified IN WRITING (or has
//!      actual notice via inspection).
//! 2. Tenant must pay rent INTO ESCROW (court or attorney
//!    trust) during proceedings.
//! 3. State Sanitary Code 105 CMR 410 sets minimum
//!    standards.
//! 4. Mass. G.L. c. 111 § 127L coordinates with local
//!    board of health enforcement.
//!
//! **Illinois — Chicago RLTO § 5-12-110 — Rent
//! Abatement**:
//! 1. Tenant may notify landlord in writing of repair
//!    needed for habitability violation;
//! 2. 14-day cure window for landlord;
//! 3. If unrepaired, tenant may:
//!    - REDUCE RENT by reasonable amount reflecting
//!      reduction in value of dwelling unit;
//!    - REPAIR AND DEDUCT up to ONE-HALF MONTH'S RENT or
//!      $500 (whichever greater);
//!    - TERMINATE rental agreement.
//!
//! **Pennsylvania — Pugh v. Holmes, 486 Pa. 272 (1979)**:
//! 1. COMMON-LAW IMPLIED warranty of habitability;
//! 2. Tenant may WITHHOLD RENT as defense to ejectment;
//! 3. Court determines damages based on rental-value
//!    diminution from substantial impairment;
//! 4. Repair-and-deduct also recognized.
//!
//! **Default — common-law habitability framework** —
//! virtually every state recognizes implied warranty of
//! habitability under common law; varies by state on
//! specific remedies + escrow requirements + thresholds.
//!
//! Citations: Cal. Civ. Code § 1942; Cal. Civ. Code
//! § 1942.4; Cal. Health & Safety Code § 17920.3; N.Y.
//! Real Prop. Law § 235-b; Park West Management Corp. v.
//! Mitchell, 47 N.Y.2d 316 (1979); N.Y. RPAPL § 711;
//! Mass. G.L. c. 239 § 8A; Mass. State Sanitary Code 105
//! CMR 410; Mass. G.L. c. 111 § 127L; Chicago RLTO
//! § 5-12-110; Pugh v. Holmes, 486 Pa. 272 (1979).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Massachusetts,
    IllinoisChicago,
    Pennsylvania,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TenantAction {
    /// Repair-and-deduct (CA § 1942, IL Chicago, PA
    /// Pugh).
    RepairAndDeduct,
    /// Rent withholding (NY § 235-b, MA § 8A, PA Pugh).
    RentWithholding,
    /// Rent abatement (NY § 235-b, IL Chicago, judicial
    /// award).
    RentAbatement,
    /// Damages action.
    DamagesAction,
    /// Lease termination (IL Chicago option).
    LeaseTermination,
    /// No tenant action taken.
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantRentEscrowWithholdingInput {
    pub jurisdiction: Jurisdiction,
    pub tenant_action: TenantAction,
    /// Whether habitability violation exists (materially
    /// impairs health/safety).
    pub habitability_violation_exists: bool,
    /// Days since tenant gave landlord written notice.
    pub days_since_landlord_notice: u32,
    /// Days since governmental notice to landlord (CA
    /// § 1942.4(b) 35-day trigger).
    pub days_since_governmental_notice: u32,
    /// Whether tenant reported to local board of health
    /// (MA + Chicago requirement).
    pub reported_to_local_code_enforcement: bool,
    /// Monthly rent in cents.
    pub monthly_rent_cents: u64,
    /// Repair cost claimed by tenant in cents.
    pub repair_cost_cents: u64,
    /// Number of times tenant invoked repair-and-deduct
    /// in last 12 months (CA twice-per-12-months limit).
    pub repair_and_deduct_count_12_months: u32,
    /// Whether tenant paid withheld rent INTO ESCROW (MA
    /// requirement; some jurisdictions).
    pub rent_paid_into_escrow: bool,
    /// Whether violation caused by tenant (defeats CA
    /// § 1942.4 protection).
    pub violation_caused_by_tenant: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantRentEscrowWithholdingResult {
    pub jurisdiction: Jurisdiction,
    pub tenant_remedy_available: bool,
    pub repair_and_deduct_compliant: bool,
    pub rent_withholding_compliant: bool,
    pub repair_cost_within_cap: bool,
    pub annual_limit_compliant: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &TenantRentEscrowWithholdingInput) -> TenantRentEscrowWithholdingResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let mut tenant_remedy_available = input.habitability_violation_exists;
    let mut repair_and_deduct_compliant = true;
    let mut rent_withholding_compliant = true;
    let mut repair_cost_within_cap = true;
    let mut annual_limit_compliant = true;

    match input.jurisdiction {
        Jurisdiction::California => {
            if matches!(input.tenant_action, TenantAction::RepairAndDeduct) {
                if input.repair_cost_cents > input.monthly_rent_cents {
                    repair_cost_within_cap = false;
                    failure_reasons.push(
                        "Cal. Civ. Code § 1942(a) — tenant repair-and-deduct cost MAY NOT EXCEED ONE MONTH'S RENT per repair".to_string(),
                    );
                }
                if input.repair_and_deduct_count_12_months >= 2 {
                    annual_limit_compliant = false;
                    failure_reasons.push(
                        "Cal. Civ. Code § 1942(a) — tenant repair-and-deduct LIMITED to TWICE in any 12-MONTH PERIOD".to_string(),
                    );
                }
                if input.days_since_landlord_notice < 30 {
                    repair_and_deduct_compliant = false;
                    failure_reasons.push(format!(
                        "Cal. Civ. Code § 1942(b) — 30-DAY notice creates reasonable-time PRESUMPTION; only {} days since landlord notice",
                        input.days_since_landlord_notice
                    ));
                }
            }
            if input.habitability_violation_exists
                && input.days_since_governmental_notice >= 35
                && !input.violation_caused_by_tenant
            {
                failure_reasons.push(
                    "Cal. Civ. Code § 1942.4 — landlord PROHIBITED from demanding rent, increasing rent, OR terminating tenancy when (1) building substandard per Health & Safety Code § 17920.3 AND (2) 35 DAYS since governmental notice AND (3) violation not caused by tenant; tenant entitled to ACTUAL DAMAGES + $100-$5,000 STATUTORY DAMAGES + reasonable attorney fees + costs per § 1942.4(b)".to_string(),
                );
            }
        }
        Jurisdiction::NewYork => {
            if input.habitability_violation_exists {
                failure_reasons.push(
                    "N.Y. Real Prop. Law § 235-b — IMPLIED warranty of habitability in every lease; remedies: rent abatement + repair-and-deduct (extenuating circumstances) + rent withholding (defense to RPAPL § 711 nonpayment proceeding) + damages + attorney fees; established by Park West Management Corp. v. Mitchell, 47 N.Y.2d 316 (1979)".to_string(),
                );
                failure_reasons.push(
                    "N.Y. Real Prop. Law § 235-b(2) — WAIVER VOID as contrary to public policy; lease provision attempting to waive warranty of habitability is unenforceable".to_string(),
                );
            }
        }
        Jurisdiction::Massachusetts => {
            if matches!(input.tenant_action, TenantAction::RentWithholding) {
                if !input.reported_to_local_code_enforcement {
                    rent_withholding_compliant = false;
                    failure_reasons.push(
                        "Mass. G.L. c. 239 § 8A — tenant must REPORT TO LOCAL BOARD OF HEALTH or similar code enforcement agency before withholding rent; State Sanitary Code 105 CMR 410 sets minimum standards".to_string(),
                    );
                }
                if !input.rent_paid_into_escrow {
                    rent_withholding_compliant = false;
                    failure_reasons.push(
                        "Mass. G.L. c. 239 § 8A — withheld rent must be PAID INTO ESCROW (court or attorney trust) during summary process proceedings".to_string(),
                    );
                }
            }
        }
        Jurisdiction::IllinoisChicago => {
            if matches!(input.tenant_action, TenantAction::RepairAndDeduct) {
                let cap_cents = (input.monthly_rent_cents / 2).max(50_000);
                if input.repair_cost_cents > cap_cents {
                    repair_cost_within_cap = false;
                    failure_reasons.push(format!(
                        "Chicago RLTO § 5-12-110 — repair-and-deduct cost CAPPED at ONE-HALF MONTH'S RENT OR $500 (whichever greater); cap = {} cents",
                        cap_cents
                    ));
                }
                if input.days_since_landlord_notice < 14 {
                    repair_and_deduct_compliant = false;
                    failure_reasons.push(format!(
                        "Chicago RLTO § 5-12-110 — 14-DAY cure window for landlord required before tenant repair-and-deduct; only {} days since notice",
                        input.days_since_landlord_notice
                    ));
                }
            }
        }
        Jurisdiction::Pennsylvania => {
            if input.habitability_violation_exists {
                failure_reasons.push(
                    "Pugh v. Holmes, 486 Pa. 272 (1979) — COMMON-LAW IMPLIED warranty of habitability; tenant may WITHHOLD RENT as defense to ejectment; court determines damages based on rental-value diminution from substantial impairment; repair-and-deduct also recognized".to_string(),
                );
            }
        }
        Jurisdiction::Default => {
            if !input.habitability_violation_exists {
                tenant_remedy_available = false;
            }
        }
    }

    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1942(a) — tenant may REPAIR AND DEDUCT cost from rent up to ONE MONTH'S RENT per repair after landlord fails to repair within reasonable time; LIMITED to TWICE in any 12-MONTH PERIOD".to_string(),
        "Cal. Civ. Code § 1942(b) — 30-DAY notice creates reasonable-time PRESUMPTION (rebuttable based on circumstances)".to_string(),
        "Cal. Civ. Code § 1942.4 — landlord PROHIBITED from demanding rent / increasing rent / terminating tenancy when (1) building substandard per Health & Safety Code § 17920.3; (2) 35 DAYS since governmental notice to landlord; (3) violation not caused by tenant; tenant remedy: ACTUAL DAMAGES + $100-$5,000 STATUTORY DAMAGES + reasonable attorney fees + costs".to_string(),
        "N.Y. Real Prop. Law § 235-b(1) — IMPLIED warranty in EVERY written or oral lease that premises are FIT FOR HUMAN HABITATION and not dangerous/hazardous/detrimental to life/health/safety".to_string(),
        "N.Y. Real Prop. Law § 235-b(2) — WAIVER VOID as contrary to public policy; lease provision attempting to waive warranty of habitability is unenforceable".to_string(),
        "N.Y. Real Prop. Law § 235-b(3) — tenant remedies include RENT ABATEMENT (court-determined) + REPAIR AND DEDUCT (extenuating circumstances) + RENT WITHHOLDING (defense to RPAPL § 711 nonpayment summary proceeding) + DAMAGES action + ATTORNEY FEES; established by Park West Management Corp. v. Mitchell, 47 N.Y.2d 316 (1979)".to_string(),
        "Mass. G.L. c. 239 § 8A — tenant may withhold rent if (1) conditions ENDANGER or MATERIALLY IMPAIR health/safety/well-being; (2) tenant reported to LOCAL BOARD OF HEALTH or similar code enforcement agency; (3) landlord notified IN WRITING; tenant must pay rent INTO ESCROW during proceedings; Mass. State Sanitary Code 105 CMR 410 sets minimum standards".to_string(),
        "Chicago RLTO § 5-12-110 — tenant may notify landlord in writing of repair needed; 14-DAY cure window for landlord; if unrepaired: (1) REDUCE RENT by reasonable amount reflecting reduction in value; (2) REPAIR AND DEDUCT up to ONE-HALF MONTH'S RENT OR $500 (whichever greater); (3) TERMINATE rental agreement".to_string(),
        "Pugh v. Holmes, 486 Pa. 272 (1979) — COMMON-LAW IMPLIED warranty of habitability; tenant may WITHHOLD RENT as defense to ejectment; court determines damages based on rental-value diminution; repair-and-deduct also recognized".to_string(),
        "Default — virtually every state recognizes implied warranty of habitability under common law; varies by state on specific remedies + escrow requirements + thresholds".to_string(),
        "Trader-landlord critical because (1) implied warranty of habitability is among the most powerful tenant defenses; (2) tenant who establishes habitability violation can REDUCE OR WITHHOLD RENT without losing possession; (3) many state waivers are VOID as contrary to public policy; (4) many states authorize TENANT REPAIR-AND-DEDUCT with statutory damage caps".to_string(),
        "Cross-jurisdictional architecture: California uses REPAIR-AND-DEDUCT + § 1942.4 STATUTORY DAMAGES; New York uses IMPLIED WARRANTY + WAIVER VOID + multi-remedy menu; Massachusetts uses RENT WITHHOLDING + ESCROW + CODE ENFORCEMENT trigger; Chicago uses 14-DAY CURE + HALF-MONTH/$500 REPAIR CAP + TERMINATION OPTION; Pennsylvania uses COMMON-LAW PUGH v. HOLMES; Default uses common-law habitability framework".to_string(),
    ];

    TenantRentEscrowWithholdingResult {
        jurisdiction: input.jurisdiction,
        tenant_remedy_available,
        repair_and_deduct_compliant,
        rent_withholding_compliant,
        repair_cost_within_cap,
        annual_limit_compliant,
        failure_reasons,
        citation: "Cal. Civ. Code § 1942 and § 1942.4; Cal. Health & Safety Code § 17920.3; N.Y. Real Prop. Law § 235-b; Park West Management Corp. v. Mitchell, 47 N.Y.2d 316 (1979); N.Y. RPAPL § 711; Mass. G.L. c. 239 § 8A; Mass. State Sanitary Code 105 CMR 410; Mass. G.L. c. 111 § 127L; Chicago RLTO § 5-12-110; Pugh v. Holmes, 486 Pa. 272 (1979)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_repair_compliant() -> TenantRentEscrowWithholdingInput {
        TenantRentEscrowWithholdingInput {
            jurisdiction: Jurisdiction::California,
            tenant_action: TenantAction::RepairAndDeduct,
            habitability_violation_exists: true,
            days_since_landlord_notice: 35,
            days_since_governmental_notice: 0,
            reported_to_local_code_enforcement: false,
            monthly_rent_cents: 300_000,
            repair_cost_cents: 200_000,
            repair_and_deduct_count_12_months: 0,
            rent_paid_into_escrow: false,
            violation_caused_by_tenant: false,
        }
    }

    #[test]
    fn ca_repair_within_one_month_cap_compliant() {
        let r = check(&ca_repair_compliant());
        assert!(r.repair_cost_within_cap);
        assert!(r.repair_and_deduct_compliant);
        assert!(r.annual_limit_compliant);
    }

    #[test]
    fn ca_repair_exceeds_one_month_cap_violation() {
        let mut i = ca_repair_compliant();
        i.repair_cost_cents = 400_000;
        let r = check(&i);
        assert!(!r.repair_cost_within_cap);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1942(a)") && f.contains("ONE MONTH'S RENT")));
    }

    #[test]
    fn ca_twice_per_year_limit_violation() {
        let mut i = ca_repair_compliant();
        i.repair_and_deduct_count_12_months = 2;
        let r = check(&i);
        assert!(!r.annual_limit_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1942(a)") && f.contains("TWICE") && f.contains("12-MONTH")));
    }

    #[test]
    fn ca_under_30_day_notice_violation() {
        let mut i = ca_repair_compliant();
        i.days_since_landlord_notice = 29;
        let r = check(&i);
        assert!(!r.repair_and_deduct_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1942(b)") && f.contains("30-DAY")));
    }

    #[test]
    fn ca_1942_4_35_day_governmental_notice_engages() {
        let mut i = ca_repair_compliant();
        i.tenant_action = TenantAction::RentWithholding;
        i.days_since_governmental_notice = 35;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1942.4") && f.contains("35 DAYS") && f.contains("$100-$5,000")));
    }

    #[test]
    fn ca_1942_4_violation_caused_by_tenant_no_protection() {
        let mut i = ca_repair_compliant();
        i.days_since_governmental_notice = 35;
        i.violation_caused_by_tenant = true;
        let r = check(&i);
        assert!(!r.failure_reasons.iter().any(|f| f.contains("§ 1942.4")));
    }

    #[test]
    fn ny_warranty_engages_on_habitability_violation() {
        let mut i = ca_repair_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r.tenant_remedy_available);
        assert!(
            r.failure_reasons
                .iter()
                .any(|f| f.contains("§ 235-b")
                    && f.contains("Park West Management Corp. v. Mitchell"))
        );
    }

    #[test]
    fn ny_waiver_void_note() {
        let mut i = ca_repair_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 235-b(2)") && f.contains("WAIVER VOID")));
    }

    #[test]
    fn ma_rent_withholding_requires_code_enforcement_report() {
        let mut i = ca_repair_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.tenant_action = TenantAction::RentWithholding;
        i.reported_to_local_code_enforcement = false;
        i.rent_paid_into_escrow = true;
        let r = check(&i);
        assert!(!r.rent_withholding_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("c. 239 § 8A") && f.contains("LOCAL BOARD OF HEALTH")));
    }

    #[test]
    fn ma_rent_withholding_requires_escrow_payment() {
        let mut i = ca_repair_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.tenant_action = TenantAction::RentWithholding;
        i.reported_to_local_code_enforcement = true;
        i.rent_paid_into_escrow = false;
        let r = check(&i);
        assert!(!r.rent_withholding_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("c. 239 § 8A") && f.contains("PAID INTO ESCROW")));
    }

    #[test]
    fn ma_rent_withholding_full_compliance() {
        let mut i = ca_repair_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.tenant_action = TenantAction::RentWithholding;
        i.reported_to_local_code_enforcement = true;
        i.rent_paid_into_escrow = true;
        let r = check(&i);
        assert!(r.rent_withholding_compliant);
    }

    #[test]
    fn chicago_repair_within_half_month_cap_compliant() {
        let mut i = ca_repair_compliant();
        i.jurisdiction = Jurisdiction::IllinoisChicago;
        i.monthly_rent_cents = 200_000;
        i.repair_cost_cents = 100_000;
        i.days_since_landlord_notice = 14;
        let r = check(&i);
        assert!(r.repair_cost_within_cap);
        assert!(r.repair_and_deduct_compliant);
    }

    #[test]
    fn chicago_repair_exceeds_half_month_violation() {
        let mut i = ca_repair_compliant();
        i.jurisdiction = Jurisdiction::IllinoisChicago;
        i.monthly_rent_cents = 200_000;
        i.repair_cost_cents = 150_000;
        i.days_since_landlord_notice = 14;
        let r = check(&i);
        assert!(!r.repair_cost_within_cap);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 5-12-110") && f.contains("ONE-HALF MONTH'S RENT OR $500")));
    }

    #[test]
    fn chicago_500_dollar_floor_overrides_low_rent() {
        let mut i = ca_repair_compliant();
        i.jurisdiction = Jurisdiction::IllinoisChicago;
        i.monthly_rent_cents = 50_000;
        i.repair_cost_cents = 49_000;
        i.days_since_landlord_notice = 14;
        let r = check(&i);
        assert!(r.repair_cost_within_cap);
    }

    #[test]
    fn chicago_under_14_day_cure_violation() {
        let mut i = ca_repair_compliant();
        i.jurisdiction = Jurisdiction::IllinoisChicago;
        i.days_since_landlord_notice = 13;
        i.monthly_rent_cents = 200_000;
        i.repair_cost_cents = 50_000;
        let r = check(&i);
        assert!(!r.repair_and_deduct_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 5-12-110") && f.contains("14-DAY")));
    }

    #[test]
    fn pa_pugh_v_holmes_engages_on_violation() {
        let mut i = ca_repair_compliant();
        i.jurisdiction = Jurisdiction::Pennsylvania;
        let r = check(&i);
        assert!(r.tenant_remedy_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("Pugh v. Holmes")
                && f.contains("486 Pa. 272 (1979)")
                && f.contains("COMMON-LAW IMPLIED warranty")));
    }

    #[test]
    fn default_no_violation_no_remedy() {
        let mut i = ca_repair_compliant();
        i.jurisdiction = Jurisdiction::Default;
        i.habitability_violation_exists = false;
        let r = check(&i);
        assert!(!r.tenant_remedy_available);
    }

    #[test]
    fn jurisdiction_truth_table_six_cells() {
        for jur in [
            Jurisdiction::California,
            Jurisdiction::NewYork,
            Jurisdiction::Massachusetts,
            Jurisdiction::IllinoisChicago,
            Jurisdiction::Pennsylvania,
            Jurisdiction::Default,
        ] {
            let mut i = ca_repair_compliant();
            i.jurisdiction = jur;
            let r = check(&i);
            assert_eq!(r.jurisdiction, jur);
        }
    }

    #[test]
    fn ny_uniquely_engages_waiver_void_invariant() {
        let mut ny = ca_repair_compliant();
        ny.jurisdiction = Jurisdiction::NewYork;
        let r_ny = check(&ny);
        assert!(r_ny
            .failure_reasons
            .iter()
            .any(|f| f.contains("WAIVER VOID")));

        for jur in [
            Jurisdiction::California,
            Jurisdiction::Massachusetts,
            Jurisdiction::IllinoisChicago,
            Jurisdiction::Pennsylvania,
            Jurisdiction::Default,
        ] {
            let mut i = ca_repair_compliant();
            i.jurisdiction = jur;
            let r = check(&i);
            assert!(
                !r.failure_reasons.iter().any(|f| f.contains("WAIVER VOID")),
                "jur={:?}",
                jur
            );
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&ca_repair_compliant());
        assert!(r.citation.contains("Cal. Civ. Code § 1942 and § 1942.4"));
        assert!(r.citation.contains("Cal. Health & Safety Code § 17920.3"));
        assert!(r.citation.contains("N.Y. Real Prop. Law § 235-b"));
        assert!(r
            .citation
            .contains("Park West Management Corp. v. Mitchell"));
        assert!(r.citation.contains("47 N.Y.2d 316 (1979)"));
        assert!(r.citation.contains("N.Y. RPAPL § 711"));
        assert!(r.citation.contains("Mass. G.L. c. 239 § 8A"));
        assert!(r.citation.contains("105 CMR 410"));
        assert!(r.citation.contains("Mass. G.L. c. 111 § 127L"));
        assert!(r.citation.contains("Chicago RLTO § 5-12-110"));
        assert!(r.citation.contains("Pugh v. Holmes, 486 Pa. 272 (1979)"));
    }

    #[test]
    fn note_pins_ca_1942_repair_and_deduct() {
        let r = check(&ca_repair_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1942(a)")
            && n.contains("REPAIR AND DEDUCT")
            && n.contains("ONE MONTH'S RENT")
            && n.contains("TWICE")
            && n.contains("12-MONTH")));
    }

    #[test]
    fn note_pins_ca_1942_30_day_presumption() {
        let r = check(&ca_repair_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1942(b)")
            && n.contains("30-DAY notice")
            && n.contains("PRESUMPTION")));
    }

    #[test]
    fn note_pins_ca_1942_4_35_day_statutory_damages() {
        let r = check(&ca_repair_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1942.4")
            && n.contains("35 DAYS")
            && n.contains("$100-$5,000")
            && n.contains("§ 17920.3")));
    }

    #[test]
    fn note_pins_ny_235b_implied_warranty() {
        let r = check(&ca_repair_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 235-b(1)")
            && n.contains("IMPLIED warranty")
            && n.contains("FIT FOR HUMAN HABITATION")));
    }

    #[test]
    fn note_pins_ny_235b_waiver_void() {
        let r = check(&ca_repair_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 235-b(2)") && n.contains("WAIVER VOID")));
    }

    #[test]
    fn note_pins_ny_park_west_management() {
        let r = check(&ca_repair_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 235-b(3)")
            && n.contains("Park West Management Corp. v. Mitchell")
            && n.contains("47 N.Y.2d 316 (1979)")));
    }

    #[test]
    fn note_pins_ma_8a_code_enforcement_escrow() {
        let r = check(&ca_repair_compliant());
        assert!(r.notes.iter().any(|n| n.contains("c. 239 § 8A")
            && n.contains("ENDANGER or MATERIALLY IMPAIR")
            && n.contains("LOCAL BOARD OF HEALTH")
            && n.contains("INTO ESCROW")
            && n.contains("105 CMR 410")));
    }

    #[test]
    fn note_pins_chicago_5_12_110_14_day_cure() {
        let r = check(&ca_repair_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 5-12-110")
            && n.contains("14-DAY cure window")
            && n.contains("ONE-HALF MONTH'S RENT OR $500")
            && n.contains("TERMINATE rental agreement")));
    }

    #[test]
    fn note_pins_pa_pugh_v_holmes() {
        let r = check(&ca_repair_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Pugh v. Holmes")
            && n.contains("486 Pa. 272 (1979)")
            && n.contains("COMMON-LAW IMPLIED warranty")));
    }

    #[test]
    fn note_pins_default_common_law_framework() {
        let r = check(&ca_repair_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Default")
            && n.contains("virtually every state")
            && n.contains("implied warranty")));
    }

    #[test]
    fn note_pins_trader_landlord_powerful_tenant_defenses() {
        let r = check(&ca_repair_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-landlord critical")
                && n.contains("most powerful tenant defenses")
                && n.contains("REDUCE OR WITHHOLD RENT")
                && n.contains("VOID as contrary to public policy")));
    }

    #[test]
    fn note_pins_cross_jurisdictional_architecture() {
        let r = check(&ca_repair_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Cross-jurisdictional architecture")
                && n.contains("REPAIR-AND-DEDUCT + § 1942.4 STATUTORY DAMAGES")
                && n.contains("IMPLIED WARRANTY + WAIVER VOID")
                && n.contains("RENT WITHHOLDING + ESCROW")
                && n.contains("14-DAY CURE")
                && n.contains("PUGH v. HOLMES")));
    }

    #[test]
    fn multiple_ca_failures_stack() {
        let mut i = ca_repair_compliant();
        i.repair_cost_cents = 500_000;
        i.repair_and_deduct_count_12_months = 2;
        i.days_since_landlord_notice = 10;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 3);
    }
}
