//! Landlord water and heat emergency response
//! timeline framework — multi-jurisdictional landlord
//! obligation timeline for true habitability
//! emergencies (burst pipes, flooding, sewage
//! backups, no heat during heat season, no hot water).
//! Distinct from sibling habitability_remedies
//! (general repair-and-deduct framework),
//! landlord_repair_response_timeframe (general
//! written-response framework), landlord_pest_
//! extermination_timeline (iter 449), heat_
//! requirements (winter heat compliance standards
//! independently), cooling_requirements (summer max
//! temperature).
//!
//! Trader-landlord critical because true habitability
//! emergencies trigger SHORT statutory response
//! windows (24-72 hours), high daily civil penalties
//! ($250-$500 per day in NYC), and tenant
//! constructive-eviction or rent-abatement remedies.
//! Companion to habitability_remedies (general
//! framework), landlord_repair_response_timeframe
//! (written-response timeline), landlord_pest_
//! extermination_timeline (pest emergencies),
//! detector_requirements (smoke/CO emergencies).
//!
//! **Four-jurisdiction framework**:
//!
//! NEW YORK CITY — NYC HMC § 27-2029 (heat and hot
//! water minimums); NYC HMC § 27-2028 through § 27-
//! 2033 (Article 8 — heat, cooling, hot water);
//! emergency violations correctable within 24 HOURS
//! to avoid HPD Class C civil penalty $250-$500 per
//! day. Hot water 120°F minimum 365 days per year.
//! Heat season October 1 - May 31, 68°F daytime
//! (6 AM - 10 PM) when outside < 55°F, 62°F nighttime
//! (10 PM - 6 AM).
//!
//! CALIFORNIA — Cal. Civ. Code § 1941.1(a)(1)
//! requires WATERPROOFING and weather protection;
//! § 1941.1(a)(2) requires hot and cold running
//! water with appropriate fixtures connected to
//! sewer or septic; § 1941.3 security devices.
//! Reasonable-time standard with 24-72 hour
//! benchmark for habitability emergencies.
//!
//! TEXAS — Tex. Prop. Code § 92.052 requires
//! landlord to repair conditions materially
//! affecting physical health or safety within
//! REASONABLE TIME (typically 7 days under
//! § 92.056); EMERGENCY conditions (no heat in
//! winter, sewage backup, gas leak, no water)
//! treated as 24-72 hour response under § 92.056(d)
//! reasonable-time standard.
//!
//! DEFAULT / URLTA / common law — URLTA § 4.103(a)
//! provides EMERGENCY REPAIR-AND-DEDUCT for
//! conditions involving FIRE OR HEALTH SAFETY HAZARD
//! after 14-day notice OR LESS in true emergency;
//! Restatement (Second) of Property § 5.4 implied
//! warranty of habitability with constructive
//! eviction remedy for true emergencies.
//!
//! **Emergency severity categories**:
//!
//! TIER 1 IMMEDIATE EMERGENCY (24-hour response):
//! 1. NO HEAT during heat season (NYC October 1 -
//!    May 31; similar windows in other cold-climate
//!    jurisdictions);
//! 2. NO HOT WATER (year-round);
//! 3. SEWAGE BACKUP into dwelling unit;
//! 4. ACTIVE FLOODING (burst pipe, foundation
//!    failure);
//! 5. GAS LEAK;
//! 6. SMOKE / CARBON MONOXIDE DETECTOR FAILURE
//!    (detector_requirements module);
//! 7. UNSECURED EXTERIOR DOOR (post-burglary);
//! 8. ELECTRICAL HAZARD risking fire;
//! 9. STRUCTURAL COLLAPSE risk;
//! 10. TEMPERATURE > 95°F in unit where elderly /
//!     infant / medical-condition tenants present.
//!
//! TIER 2 URGENT (72-hour response):
//! 1. Hot water reduced flow / inconsistent
//!    temperature;
//! 2. Partial loss of heat (one room only);
//! 3. Slow water leak causing minor damage;
//! 4. Refrigerator or stove failure;
//! 5. Non-functioning toilet (with backup toilet
//!    available);
//! 6. Window unable to close securely (security
//!    risk but not active break-in).
//!
//! TIER 3 STANDARD (7-14 day response):
//! 1. Cosmetic water staining;
//! 2. Working but slow plumbing;
//! 3. HVAC inefficiency without complete failure;
//! 4. Pest extermination (defer to landlord_pest_
//!    extermination_timeline iter 449).
//!
//! **NYC HMC Article 8 (heat and hot water) — civil
//! penalties**:
//! 1. Heat violation: $250-$500 per day per
//!    apartment per heat-season day under HMC
//!    § 27-2115;
//! 2. Hot water violation: $250-$500 per day under
//!    HMC § 27-2115;
//! 3. HPD Heat-Season Hotline 311 + on-line filing
//!    at HPDONLINE.NYC.GOV;
//! 4. 24/7 emergency contact required for owners
//!    of multi-unit buildings.
//!
//! **Cal. Civ. Code § 1942 repair-and-deduct (CA)**:
//! 1. Tenant gives REASONABLE NOTICE to landlord;
//! 2. Landlord fails to make repair within REASONABLE
//!    TIME (typically 30 days but 24-72 hours for
//!    true emergency);
//! 3. Tenant may repair and deduct from rent up to
//!    ONE MONTH'S RENT;
//! 4. Limit twice in any 12-month period;
//! 5. Tenant may NOT use repair-and-deduct if tenant
//!    caused condition.
//!
//! **Texas Property Code § 92.056 tenant remedies**:
//! 1. Written notice of need to repair specifying
//!    condition;
//! 2. 7-day waiting period for response (less in
//!    emergency);
//! 3. Tenant may TERMINATE LEASE; OR
//! 4. Tenant may obtain judicial order for repair;
//!    OR
//! 5. Tenant may sue for actual damages + one
//!    month's rent + $500 civil penalty +
//!    attorney's fees + court costs.
//!
//! **Trader-landlord critical fact patterns**:
//!
//! NYC trader-owned 12-unit building loses heat
//! November 15 (heat season); landlord delays 48
//! hours to dispatch HVAC tech; HPD inspection
//! within 24 hours of tenant 311 complaint; HMC
//! § 27-2029 Class C violation $250-$500 per day
//! per apartment; 12 apartments × 2 days × $500 =
//! $12,000 maximum daily penalty.
//!
//! CA trader-owned SFR burst pipe floods kitchen;
//! tenant gives written notice; landlord delays 5
//! days to dispatch plumber; tenant invokes Cal.
//! Civ. Code § 1942 repair-and-deduct hires
//! emergency plumber for $2,500 and deducts from
//! next month's rent ($2,500 cap = one month's
//! rent for $2,500 unit).
//!
//! TX trader-owned duplex sewage backup;
//! tenant gives written § 92.056 notice; landlord
//! delays 14 days; tenant terminates lease + sues
//! for actual damages + one month rent + $500
//! civil penalty + attorney's fees under
//! § 92.056(b).
//!
//! NYC trader fails to provide 24/7 emergency
//! contact for building; HPD issues HMC § 27-2018
//! Class B violation with $25-$250 daily civil
//! penalty plus tenant constructive eviction claim
//! for severe non-response.
//!
//! Default jurisdiction trader — URLTA § 4.103(a)
//! emergency repair-and-deduct: tenant gives notice;
//! landlord fails to act within 14 days (less in
//! true emergency); tenant repairs and deducts up
//! to greater of $300 or 1/2 month rent.
//!
//! Citations: NYC HMC § 27-2028 (heat and hot water
//! generally); NYC HMC § 27-2029 (minimum
//! temperature); NYC HMC § 27-2030 (hot water);
//! NYC HMC § 27-2031 (cooling requirements); NYC
//! HMC § 27-2032 (definitions); NYC HMC § 27-2033
//! (variances); NYC HMC § 27-2115 (civil penalties);
//! NYC HMC § 27-2018 (rodent and insect eradication
//! — pest-extermination companion); Cal. Civ. Code
//! § 1941.1(a)(1)-(2) (waterproofing + hot/cold
//! water); Cal. Civ. Code § 1941.3 (security
//! devices); Cal. Civ. Code § 1942 (repair-and-
//! deduct); Tex. Prop. Code § 92.052 (landlord
//! duty to repair); Tex. Prop. Code § 92.056
//! (tenant remedies); URLTA § 2.104 (landlord
//! maintenance duty); URLTA § 4.103 (emergency
//! repair-and-deduct); Restatement (Second) of
//! Property: Landlord and Tenant § 5.4 (implied
//! warranty of habitability and constructive
//! eviction); Park West Mgmt. Corp. v. Mitchell,
//! 47 N.Y.2d 316 (1979); Green v. Superior Court,
//! 10 Cal. 3d 616 (1974); Boston Housing Auth. v.
//! Hemingway, 363 Mass. 184 (1973).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewYorkCity,
    California,
    Texas,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmergencyTier {
    /// 24-hour response (no heat in heat season, no
    /// hot water, sewage backup, active flooding,
    /// gas leak, electrical hazard, etc.).
    Tier1Immediate,
    /// 72-hour response (reduced hot water, partial
    /// heat loss, slow water leak, appliance failure).
    Tier2Urgent,
    /// 7-14 day response (cosmetic water staining,
    /// slow plumbing, HVAC inefficiency).
    Tier3Standard,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NycHpdViolationClass {
    ClassA,
    ClassB,
    ClassC,
    NoViolation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LandlordWaterHeatEmergencyResponseInput {
    pub jurisdiction: Jurisdiction,
    pub emergency_tier: EmergencyTier,
    /// Hours since tenant reported emergency.
    pub hours_since_tenant_report: u32,
    /// Whether landlord has dispatched a repair crew /
    /// commenced repair.
    pub repair_dispatched: bool,
    /// Whether NYC heat season (October 1 - May 31).
    pub nyc_heat_season: bool,
    /// Whether landlord maintains 24/7 emergency
    /// contact for multi-unit building.
    pub nyc_24_7_emergency_contact: bool,
    /// NYC HPD violation class issued.
    pub nyc_hpd_violation_class: NycHpdViolationClass,
    /// Apartments affected by emergency (NYC daily
    /// penalty multiplier).
    pub apartments_affected: u32,
    /// Whether tenant gave written notice to landlord
    /// (required for Cal. § 1942 + Tex. § 92.056).
    pub tenant_written_notice_given: bool,
    /// Whether tenant invoked CA Civ. Code § 1942
    /// repair-and-deduct.
    pub ca_repair_and_deduct_invoked: bool,
    /// Whether tenant terminated lease under Tex.
    /// Prop. Code § 92.056.
    pub tx_lease_termination_invoked: bool,
    /// Whether tenant invokes constructive eviction.
    pub constructive_eviction_invoked: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LandlordWaterHeatEmergencyResponseResult {
    pub required_response_hours: u32,
    pub response_timely: bool,
    pub habitability_warranty_breached: bool,
    pub nyc_24_7_contact_required: bool,
    pub nyc_24_7_contact_compliant: bool,
    pub nyc_daily_civil_penalty_max_cents: u64,
    pub nyc_daily_civil_penalty_total_cents: u64,
    pub tenant_remedies_available: Vec<String>,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &LandlordWaterHeatEmergencyResponseInput,
) -> LandlordWaterHeatEmergencyResponseResult {
    let mut failure_reasons: Vec<String> = Vec::new();
    let mut tenant_remedies_available: Vec<String> = Vec::new();

    let required_response_hours: u32 = match input.emergency_tier {
        EmergencyTier::Tier1Immediate => 24,
        EmergencyTier::Tier2Urgent => 72,
        EmergencyTier::Tier3Standard => 7 * 24,
    };

    let response_timely =
        input.repair_dispatched && input.hours_since_tenant_report <= required_response_hours;

    let habitability_warranty_breached = !response_timely;

    let nyc_24_7_contact_required = matches!(input.jurisdiction, Jurisdiction::NewYorkCity)
        && input.apartments_affected >= 2;

    let nyc_24_7_contact_compliant = !nyc_24_7_contact_required || input.nyc_24_7_emergency_contact;

    let nyc_daily_civil_penalty_max_cents: u64 = match input.nyc_hpd_violation_class {
        NycHpdViolationClass::ClassC => 50_000,
        NycHpdViolationClass::ClassB => 25_000,
        NycHpdViolationClass::ClassA => 0,
        NycHpdViolationClass::NoViolation => 0,
    };

    let nyc_daily_civil_penalty_total_cents = if matches!(input.jurisdiction, Jurisdiction::NewYorkCity) {
        nyc_daily_civil_penalty_max_cents.saturating_mul(input.apartments_affected as u64)
    } else {
        0
    };

    if !response_timely {
        let tier_label = match input.emergency_tier {
            EmergencyTier::Tier1Immediate => "TIER 1 IMMEDIATE EMERGENCY (no heat in heat season, no hot water, sewage backup, active flooding, gas leak, electrical hazard, structural collapse risk)",
            EmergencyTier::Tier2Urgent => "TIER 2 URGENT (reduced hot water, partial heat loss, slow water leak, appliance failure)",
            EmergencyTier::Tier3Standard => "TIER 3 STANDARD (cosmetic water staining, slow plumbing, HVAC inefficiency)",
        };
        let jurisdiction_statute = match input.jurisdiction {
            Jurisdiction::NewYorkCity => "NYC HMC § 27-2029 (heat and hot water minimum temperature) + § 27-2115 (civil penalties)",
            Jurisdiction::California => "Cal. Civ. Code § 1941.1 (habitability) + § 1942 (repair-and-deduct)",
            Jurisdiction::Texas => "Tex. Prop. Code § 92.052 (landlord duty to repair) + § 92.056 (tenant remedies)",
            Jurisdiction::Default => "URLTA § 2.104 (landlord maintenance duty) + § 4.103 (emergency repair-and-deduct); Restatement (Second) of Property § 5.4",
        };
        failure_reasons.push(format!(
            "{} — landlord failed to commence repair within {} hours for {}; {} hours elapsed since tenant report; habitability warranty BREACHED",
            jurisdiction_statute, required_response_hours, tier_label, input.hours_since_tenant_report
        ));
    }

    if !nyc_24_7_contact_compliant {
        failure_reasons.push(
            "NYC HMC § 27-2018 + 24/7 EMERGENCY CONTACT REQUIREMENT — owners of multi-unit buildings (2+ apartments) must maintain 24/7 emergency contact for breakdowns; failure exposes landlord to HPD Class B violation $25-$250 daily civil penalty plus tenant constructive eviction claim".to_string(),
        );
    }

    if !matches!(input.nyc_hpd_violation_class, NycHpdViolationClass::NoViolation)
        && matches!(input.jurisdiction, Jurisdiction::NewYorkCity)
    {
        let class_label = match input.nyc_hpd_violation_class {
            NycHpdViolationClass::ClassA => "CLASS A (non-hazardous) — 90-day correction window; no daily civil penalty",
            NycHpdViolationClass::ClassB => "CLASS B (hazardous) — 30-day correction window; $25-$250 daily civil penalty per apartment",
            NycHpdViolationClass::ClassC => "CLASS C (immediately hazardous) — 24-hour correction window; $250-$500 daily civil penalty per apartment under NYC HMC § 27-2115",
            NycHpdViolationClass::NoViolation => "(none)",
        };
        failure_reasons.push(format!(
            "NYC HPD Violation issued — {}; total daily penalty across {} apartments: {} cents/day",
            class_label, input.apartments_affected, nyc_daily_civil_penalty_total_cents
        ));
    }

    if matches!(input.jurisdiction, Jurisdiction::NewYorkCity) && input.nyc_heat_season {
        failure_reasons.push(
            "NYC HMC § 27-2029 HEAT SEASON October 1 - May 31 — landlord must maintain temperature of at least 68°F daytime (6 AM - 10 PM) when outside temperature falls below 55°F, and 62°F nighttime (10 PM - 6 AM); hot water 120°F minimum 365 days/year under NYC HMC § 27-2030".to_string(),
        );
    }

    if matches!(input.jurisdiction, Jurisdiction::California)
        && input.tenant_written_notice_given
        && input.ca_repair_and_deduct_invoked
        && !response_timely
    {
        failure_reasons.push(
            "Cal. Civ. Code § 1942 REPAIR-AND-DEDUCT INVOKED — tenant gave reasonable notice; landlord failed to make repair within reasonable time (typically 30 days but 24-72 hours for true emergency); tenant may repair and deduct from rent UP TO ONE MONTH'S RENT; limit twice in any 12-month period; tenant may NOT use repair-and-deduct if tenant caused condition".to_string(),
        );
    }

    if matches!(input.jurisdiction, Jurisdiction::Texas)
        && input.tenant_written_notice_given
        && input.tx_lease_termination_invoked
        && !response_timely
    {
        failure_reasons.push(
            "Tex. Prop. Code § 92.056 TENANT REMEDIES INVOKED — written notice given; 7-day waiting period elapsed (or less in emergency); tenant may TERMINATE LEASE, obtain judicial order for repair, OR sue for actual damages + ONE MONTH'S RENT + $500 CIVIL PENALTY + attorney's fees + court costs".to_string(),
        );
    }

    if input.constructive_eviction_invoked && habitability_warranty_breached {
        failure_reasons.push(
            "CONSTRUCTIVE EVICTION INVOKED — tenant entitled to vacate premises with full rent relief; Restatement (Second) of Property § 5.4; supported by Park West Mgmt. Corp. v. Mitchell, 47 N.Y.2d 316 (1979) in NY; Green v. Superior Court, 10 Cal. 3d 616 (1974) in CA; Boston Housing Auth. v. Hemingway, 363 Mass. 184 (1973) in MA".to_string(),
        );
    }

    if habitability_warranty_breached {
        tenant_remedies_available.push(
            "RENT WITHHOLDING under implied warranty of habitability".to_string(),
        );
        tenant_remedies_available.push(
            "REPAIR AND DEDUCT — CA Cal. Civ. Code § 1942 up to ONE MONTH'S RENT; URLTA § 4.103 up to greater of $300 or 1/2 month rent".to_string(),
        );
        tenant_remedies_available.push(
            "RENT ABATEMENT — 50-100% of rent during habitability breach period (varies by jurisdiction)".to_string(),
        );
        tenant_remedies_available.push(
            "CONSTRUCTIVE EVICTION — vacate premises with full rent relief under Restatement (Second) of Property § 5.4".to_string(),
        );
        tenant_remedies_available.push(
            "TEXAS § 92.056 STATUTORY REMEDIES — termination + actual damages + one month's rent + $500 civil penalty + attorney's fees + court costs (Texas only)".to_string(),
        );
    }

    if matches!(input.jurisdiction, Jurisdiction::NewYorkCity) {
        tenant_remedies_available.push(
            "NYC HPD HEAT-SEASON HOTLINE 311 + ON-LINE FILING at HPDONLINE.NYC.GOV — tenant complaint triggers HPD inspection within 24 hours; potential Class A/B/C violation issuance with civil penalties per HMC § 27-2115".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "Four-jurisdiction framework: NEW YORK CITY (NYC HMC § 27-2029 heat + § 27-2030 hot water + § 27-2115 civil penalties); CALIFORNIA (Cal. Civ. Code § 1941.1 habitability + § 1942 repair-and-deduct); TEXAS (Tex. Prop. Code § 92.052 landlord duty + § 92.056 tenant remedies); DEFAULT (URLTA § 2.104 + § 4.103 + Restatement (Second) of Property § 5.4)".to_string(),
        "Emergency severity categories: TIER 1 IMMEDIATE EMERGENCY (24-hour response — no heat in heat season, no hot water, sewage backup, active flooding, gas leak, smoke/CO detector failure, unsecured exterior door, electrical hazard, structural collapse risk, > 95°F temperature with vulnerable tenants); TIER 2 URGENT (72-hour response — reduced hot water flow, partial heat loss in one room, slow water leak with minor damage, refrigerator or stove failure, non-functioning toilet with backup, window security risk); TIER 3 STANDARD (7-14 day response — cosmetic water staining, slow plumbing, HVAC inefficiency)".to_string(),
        "NYC HMC § 27-2029 HEAT SEASON October 1 - May 31: 68°F minimum daytime (6 AM - 10 PM) when outside < 55°F; 62°F minimum nighttime (10 PM - 6 AM); NYC HMC § 27-2030 hot water 120°F minimum 365 days per year; NYC HMC § 27-2115 civil penalties $250-$500/day per apartment for Class C violations".to_string(),
        "NYC HPD violation classes: CLASS A (non-hazardous) 90-day correction no daily penalty; CLASS B (hazardous) 30-day correction $25-$250 daily; CLASS C (immediately hazardous) 24-hour correction $250-$500 daily; heat and hot water violations typically CLASS C immediately hazardous".to_string(),
        "NYC 24/7 EMERGENCY CONTACT REQUIREMENT — owners of multi-unit buildings (2+ apartments) must maintain 24/7 emergency contact for breakdowns; failure exposes landlord to additional HPD violations plus tenant constructive-eviction claims".to_string(),
        "Cal. Civ. Code § 1942 REPAIR-AND-DEDUCT: (1) tenant gives reasonable notice; (2) landlord fails to repair within reasonable time (typically 30 days but 24-72 hours for true emergency); (3) tenant may repair and deduct from rent UP TO ONE MONTH'S RENT; (4) limit twice in any 12-month period; (5) tenant may NOT invoke if tenant caused condition".to_string(),
        "Tex. Prop. Code § 92.056 TENANT REMEDIES: (1) written notice of need to repair specifying condition; (2) 7-day waiting period for response (less in true emergency under § 92.056(d)); (3) tenant may TERMINATE LEASE; (4) OR obtain judicial order for repair; (5) OR sue for actual damages + ONE MONTH'S RENT + $500 CIVIL PENALTY + attorney's fees + court costs".to_string(),
        "Tenant remedies under habitability warranty: (1) RENT WITHHOLDING (Park West Mgmt. Corp. v. Mitchell 47 N.Y.2d 316 (1979) in NY; Green v. Superior Court 10 Cal. 3d 616 (1974) in CA; Boston Housing Auth. v. Hemingway 363 Mass. 184 (1973) in MA); (2) REPAIR AND DEDUCT under state thresholds; (3) RENT ABATEMENT 50-100% of rent during breach; (4) CONSTRUCTIVE EVICTION with full rent relief under Restatement (Second) of Property § 5.4; (5) TX § 92.056 statutory remedies (termination + damages + one month rent + $500 + fees)".to_string(),
        "Trader-landlord critical fact patterns: (1) NYC 12-unit building loses heat November 15 — 48-hour delay = HMC § 27-2029 Class C violation $250-$500/day × 12 apartments × 2 days = up to $12,000 maximum daily penalty; (2) CA SFR burst pipe — landlord 5-day delay → § 1942 repair-and-deduct hires emergency plumber up to one month rent; (3) TX duplex sewage backup — § 92.056 written notice + 14-day delay → tenant termination + damages + one month rent + $500 civil penalty + fees; (4) NYC missing 24/7 emergency contact — Class B violation + constructive eviction risk; (5) URLTA jurisdiction § 4.103 emergency repair-and-deduct up to greater of $300 or 1/2 month rent".to_string(),
        "Companion to habitability_remedies (general repair-and-deduct framework comprehensive) + landlord_repair_response_timeframe (general written-response timeline) + landlord_pest_extermination_timeline (iter 449 pest emergencies) + detector_requirements (smoke/CO detector emergencies) + heat_requirements (winter heat compliance) + cooling_requirements (summer max temperature) + rental_basement_water_intrusion_disclosure (water intrusion DISCLOSURE not repair)".to_string(),
    ];

    LandlordWaterHeatEmergencyResponseResult {
        required_response_hours,
        response_timely,
        habitability_warranty_breached,
        nyc_24_7_contact_required,
        nyc_24_7_contact_compliant,
        nyc_daily_civil_penalty_max_cents,
        nyc_daily_civil_penalty_total_cents,
        tenant_remedies_available,
        failure_reasons,
        citation: "NYC HMC § 27-2028 (heat and hot water generally); NYC HMC § 27-2029 (minimum temperature); NYC HMC § 27-2030 (hot water); NYC HMC § 27-2031 (cooling requirements); NYC HMC § 27-2032 (definitions); NYC HMC § 27-2033 (variances); NYC HMC § 27-2115 (civil penalties); NYC HMC § 27-2018 (rodent and insect eradication); Cal. Civ. Code § 1941.1(a)(1)-(2); Cal. Civ. Code § 1941.3 (security devices); Cal. Civ. Code § 1942 (repair-and-deduct); Tex. Prop. Code § 92.052 (landlord duty to repair); Tex. Prop. Code § 92.056 (tenant remedies); URLTA § 2.104 (landlord maintenance duty); URLTA § 4.103 (emergency repair-and-deduct); Restatement (Second) of Property: Landlord and Tenant § 5.4 (implied warranty of habitability and constructive eviction); Park West Mgmt. Corp. v. Mitchell, 47 N.Y.2d 316 (1979); Green v. Superior Court, 10 Cal. 3d 616 (1974); Boston Housing Auth. v. Hemingway, 363 Mass. 184 (1973)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nyc_tier1_timely() -> LandlordWaterHeatEmergencyResponseInput {
        LandlordWaterHeatEmergencyResponseInput {
            jurisdiction: Jurisdiction::NewYorkCity,
            emergency_tier: EmergencyTier::Tier1Immediate,
            hours_since_tenant_report: 12,
            repair_dispatched: true,
            nyc_heat_season: true,
            nyc_24_7_emergency_contact: true,
            nyc_hpd_violation_class: NycHpdViolationClass::NoViolation,
            apartments_affected: 12,
            tenant_written_notice_given: true,
            ca_repair_and_deduct_invoked: false,
            tx_lease_termination_invoked: false,
            constructive_eviction_invoked: false,
        }
    }

    #[test]
    fn nyc_tier1_within_24_hours_timely() {
        let r = check(&nyc_tier1_timely());
        assert_eq!(r.required_response_hours, 24);
        assert!(r.response_timely);
        assert!(!r.habitability_warranty_breached);
    }

    #[test]
    fn nyc_tier1_past_24_hours_breach() {
        let mut i = nyc_tier1_timely();
        i.hours_since_tenant_report = 48;
        let r = check(&i);
        assert!(!r.response_timely);
        assert!(r.habitability_warranty_breached);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("NYC HMC § 27-2029")
            && f.contains("TIER 1 IMMEDIATE EMERGENCY")
            && f.contains("48 hours elapsed")));
    }

    #[test]
    fn tier2_72_hour_window() {
        let mut i = nyc_tier1_timely();
        i.emergency_tier = EmergencyTier::Tier2Urgent;
        let r = check(&i);
        assert_eq!(r.required_response_hours, 72);
    }

    #[test]
    fn tier3_7_day_window() {
        let mut i = nyc_tier1_timely();
        i.emergency_tier = EmergencyTier::Tier3Standard;
        let r = check(&i);
        assert_eq!(r.required_response_hours, 7 * 24);
    }

    #[test]
    fn no_repair_dispatched_breach() {
        let mut i = nyc_tier1_timely();
        i.repair_dispatched = false;
        let r = check(&i);
        assert!(!r.response_timely);
    }

    #[test]
    fn nyc_multi_unit_24_7_contact_required() {
        let r = check(&nyc_tier1_timely());
        assert!(r.nyc_24_7_contact_required);
        assert!(r.nyc_24_7_contact_compliant);
    }

    #[test]
    fn nyc_single_unit_no_24_7_requirement() {
        let mut i = nyc_tier1_timely();
        i.apartments_affected = 1;
        let r = check(&i);
        assert!(!r.nyc_24_7_contact_required);
    }

    #[test]
    fn nyc_missing_24_7_contact_violation() {
        let mut i = nyc_tier1_timely();
        i.nyc_24_7_emergency_contact = false;
        let r = check(&i);
        assert!(!r.nyc_24_7_contact_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("24/7 EMERGENCY CONTACT")
            && f.contains("Class B violation")));
    }

    #[test]
    fn nyc_class_c_civil_penalty_500_per_apartment() {
        let mut i = nyc_tier1_timely();
        i.nyc_hpd_violation_class = NycHpdViolationClass::ClassC;
        let r = check(&i);
        assert_eq!(r.nyc_daily_civil_penalty_max_cents, 50_000);
        assert_eq!(r.nyc_daily_civil_penalty_total_cents, 50_000 * 12);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("CLASS C")
            && f.contains("$250-$500 daily")
            && f.contains("§ 27-2115")));
    }

    #[test]
    fn nyc_class_b_civil_penalty_250_per_apartment() {
        let mut i = nyc_tier1_timely();
        i.nyc_hpd_violation_class = NycHpdViolationClass::ClassB;
        let r = check(&i);
        assert_eq!(r.nyc_daily_civil_penalty_max_cents, 25_000);
        assert_eq!(r.nyc_daily_civil_penalty_total_cents, 25_000 * 12);
    }

    #[test]
    fn nyc_class_a_no_civil_penalty() {
        let mut i = nyc_tier1_timely();
        i.nyc_hpd_violation_class = NycHpdViolationClass::ClassA;
        let r = check(&i);
        assert_eq!(r.nyc_daily_civil_penalty_max_cents, 0);
    }

    #[test]
    fn nyc_heat_season_disclosure() {
        let r = check(&nyc_tier1_timely());
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("HEAT SEASON October 1 - May 31")
            && f.contains("68°F")
            && f.contains("62°F")
            && f.contains("120°F")));
    }

    #[test]
    fn ca_repair_and_deduct_breach_invokes() {
        let mut i = nyc_tier1_timely();
        i.jurisdiction = Jurisdiction::California;
        i.hours_since_tenant_report = 48;
        i.ca_repair_and_deduct_invoked = true;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 1942 REPAIR-AND-DEDUCT INVOKED")
            && f.contains("ONE MONTH'S RENT")
            && f.contains("twice in any 12-month")));
    }

    #[test]
    fn tx_lease_termination_breach_invokes() {
        let mut i = nyc_tier1_timely();
        i.jurisdiction = Jurisdiction::Texas;
        i.hours_since_tenant_report = 48;
        i.tx_lease_termination_invoked = true;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 92.056 TENANT REMEDIES INVOKED")
            && f.contains("ONE MONTH'S RENT")
            && f.contains("$500 CIVIL PENALTY")
            && f.contains("attorney's fees")));
    }

    #[test]
    fn constructive_eviction_invokes_with_case_law() {
        let mut i = nyc_tier1_timely();
        i.hours_since_tenant_report = 48;
        i.constructive_eviction_invoked = true;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("CONSTRUCTIVE EVICTION INVOKED")
            && f.contains("Park West")
            && f.contains("Green v. Superior Court")
            && f.contains("Hemingway")));
    }

    #[test]
    fn habitability_breach_unlocks_remedies() {
        let mut i = nyc_tier1_timely();
        i.hours_since_tenant_report = 48;
        let r = check(&i);
        assert!(r.tenant_remedies_available.iter().any(|t|
            t.contains("RENT WITHHOLDING")));
        assert!(r.tenant_remedies_available.iter().any(|t|
            t.contains("REPAIR AND DEDUCT")
            && t.contains("ONE MONTH'S RENT")));
        assert!(r.tenant_remedies_available.iter().any(|t|
            t.contains("CONSTRUCTIVE EVICTION")));
        assert!(r.tenant_remedies_available.iter().any(|t|
            t.contains("TEXAS § 92.056")));
    }

    #[test]
    fn nyc_unlocks_hpd_hotline_remedy() {
        let r = check(&nyc_tier1_timely());
        assert!(r.tenant_remedies_available.iter().any(|t|
            t.contains("NYC HPD HEAT-SEASON HOTLINE 311")
            && t.contains("HPDONLINE.NYC.GOV")
            && t.contains("HMC § 27-2115")));
    }

    #[test]
    fn jurisdiction_tier_truth_table() {
        let cases = [
            (Jurisdiction::NewYorkCity, EmergencyTier::Tier1Immediate, 24),
            (Jurisdiction::NewYorkCity, EmergencyTier::Tier2Urgent, 72),
            (Jurisdiction::NewYorkCity, EmergencyTier::Tier3Standard, 168),
            (Jurisdiction::California, EmergencyTier::Tier1Immediate, 24),
            (Jurisdiction::California, EmergencyTier::Tier2Urgent, 72),
            (Jurisdiction::Texas, EmergencyTier::Tier1Immediate, 24),
            (Jurisdiction::Texas, EmergencyTier::Tier2Urgent, 72),
            (Jurisdiction::Default, EmergencyTier::Tier1Immediate, 24),
            (Jurisdiction::Default, EmergencyTier::Tier3Standard, 168),
        ];
        for (j, tier, exp) in cases {
            let mut i = nyc_tier1_timely();
            i.jurisdiction = j;
            i.emergency_tier = tier;
            let r = check(&i);
            assert_eq!(r.required_response_hours, exp, "j={:?} tier={:?}", j, tier);
        }
    }

    #[test]
    fn tier1_uniquely_24_hour_invariant() {
        let tier1 = check(&nyc_tier1_timely());
        assert_eq!(tier1.required_response_hours, 24);

        let mut tier2 = nyc_tier1_timely();
        tier2.emergency_tier = EmergencyTier::Tier2Urgent;
        assert!(check(&tier2).required_response_hours > 24);

        let mut tier3 = nyc_tier1_timely();
        tier3.emergency_tier = EmergencyTier::Tier3Standard;
        assert!(check(&tier3).required_response_hours > 24);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&nyc_tier1_timely());
        assert!(r.citation.contains("NYC HMC § 27-2028"));
        assert!(r.citation.contains("NYC HMC § 27-2029"));
        assert!(r.citation.contains("NYC HMC § 27-2030"));
        assert!(r.citation.contains("NYC HMC § 27-2115"));
        assert!(r.citation.contains("Cal. Civ. Code § 1941.1(a)(1)-(2)"));
        assert!(r.citation.contains("Cal. Civ. Code § 1942"));
        assert!(r.citation.contains("Tex. Prop. Code § 92.052"));
        assert!(r.citation.contains("Tex. Prop. Code § 92.056"));
        assert!(r.citation.contains("URLTA § 2.104"));
        assert!(r.citation.contains("URLTA § 4.103"));
        assert!(r.citation.contains("Restatement (Second) of Property: Landlord and Tenant § 5.4"));
        assert!(r.citation.contains("Park West Mgmt. Corp. v. Mitchell"));
        assert!(r.citation.contains("Green v. Superior Court"));
        assert!(r.citation.contains("Boston Housing Auth. v. Hemingway"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let r = check(&nyc_tier1_timely());
        assert!(r.notes.iter().any(|n|
            n.contains("Four-jurisdiction framework")
            && n.contains("NEW YORK CITY")
            && n.contains("CALIFORNIA")
            && n.contains("TEXAS")
            && n.contains("DEFAULT")));
    }

    #[test]
    fn note_pins_three_emergency_tiers() {
        let r = check(&nyc_tier1_timely());
        assert!(r.notes.iter().any(|n|
            n.contains("Emergency severity categories")
            && n.contains("TIER 1 IMMEDIATE EMERGENCY")
            && n.contains("TIER 2 URGENT")
            && n.contains("TIER 3 STANDARD")
            && n.contains("structural collapse")
            && n.contains("vulnerable tenants")));
    }

    #[test]
    fn note_pins_nyc_heat_season_temperatures() {
        let r = check(&nyc_tier1_timely());
        assert!(r.notes.iter().any(|n|
            n.contains("NYC HMC § 27-2029 HEAT SEASON")
            && n.contains("October 1 - May 31")
            && n.contains("68°F minimum daytime")
            && n.contains("62°F minimum nighttime")
            && n.contains("120°F minimum")));
    }

    #[test]
    fn note_pins_nyc_hpd_violation_classes() {
        let r = check(&nyc_tier1_timely());
        assert!(r.notes.iter().any(|n|
            n.contains("NYC HPD violation classes")
            && n.contains("CLASS C")
            && n.contains("$250-$500")
            && n.contains("immediately hazardous")));
    }

    #[test]
    fn note_pins_nyc_24_7_emergency_contact() {
        let r = check(&nyc_tier1_timely());
        assert!(r.notes.iter().any(|n|
            n.contains("NYC 24/7 EMERGENCY CONTACT REQUIREMENT")
            && n.contains("2+ apartments")));
    }

    #[test]
    fn note_pins_ca_section_1942_repair_and_deduct_five_elements() {
        let r = check(&nyc_tier1_timely());
        assert!(r.notes.iter().any(|n|
            n.contains("Cal. Civ. Code § 1942 REPAIR-AND-DEDUCT")
            && n.contains("ONE MONTH'S RENT")
            && n.contains("twice in any 12-month period")
            && n.contains("tenant caused condition")));
    }

    #[test]
    fn note_pins_tx_section_92_056_five_elements() {
        let r = check(&nyc_tier1_timely());
        assert!(r.notes.iter().any(|n|
            n.contains("Tex. Prop. Code § 92.056 TENANT REMEDIES")
            && n.contains("7-day waiting period")
            && n.contains("TERMINATE LEASE")
            && n.contains("$500 CIVIL PENALTY")
            && n.contains("attorney's fees")));
    }

    #[test]
    fn note_pins_habitability_remedies_with_case_law() {
        let r = check(&nyc_tier1_timely());
        assert!(r.notes.iter().any(|n|
            n.contains("Tenant remedies under habitability warranty")
            && n.contains("Park West Mgmt. Corp. v. Mitchell")
            && n.contains("Green v. Superior Court")
            && n.contains("Boston Housing Auth. v. Hemingway")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&nyc_tier1_timely());
        assert!(r.notes.iter().any(|n|
            n.contains("Trader-landlord critical fact patterns")
            && n.contains("12-unit building loses heat")
            && n.contains("burst pipe")
            && n.contains("sewage backup")
            && n.contains("URLTA jurisdiction")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&nyc_tier1_timely());
        assert!(r.notes.iter().any(|n|
            n.contains("Companion to habitability_remedies")
            && n.contains("landlord_pest_extermination_timeline")
            && n.contains("detector_requirements")
            && n.contains("heat_requirements")
            && n.contains("rental_basement_water_intrusion_disclosure")));
    }
}
