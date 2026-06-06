//! Rental rooftop solar PV disclosure framework — distinct from sibling
//! `tenant_solar_installation` (tenant right-to-install). Covers landlord
//! disclosure of EXISTING rooftop solar arrangement at lease signing: owned
//! vs leased panel ownership, PPA (Power Purchase Agreement) third-party
//! financing structure, net-metering credit allocation, SREC (Solar Renewable
//! Energy Credit) ownership, monthly lease/PPA payment pass-through, panel
//! maintenance responsibility, roof-warranty interaction, and rooftop-
//! encumbrance disclosure.
//!
//! Five financing structures: (1) outright-owned by landlord; (2) financed
//! loan owned by landlord; (3) solar lease (fixed monthly payment regardless
//! of production); (4) solar PPA (per-kWh payment based on actual production
//! at fixed rate); (5) community solar subscription (off-site offtake).
//!
//! Five failure modes a trader-landlord faces: (1) tenant utility bill
//! confusion — tenant assumes solar = free power; (2) net-metering credit
//! captured by landlord while tenant pays full utility bill; (3) PPA
//! monthly payment pass-through unauthorized under lease; (4) roof penetration
//! voids landlord's roof warranty; (5) PPA acceleration on default makes the
//! tenant responsible for remainder.
//!
//! California: Cal. Civ. Code § 1102.6c (solar contract disclosure on sale)
//! plus Pub. Util. Code § 2827 (NEM 1.0/2.0) plus NEM 3.0 NBT (Net Billing
//! Tariff) effective April 15, 2023 governing residential net-metering
//! credit allocation. Cal. Civ. Code § 1947.13 implicit pass-through
//! limitation. New Jersey: N.J.S.A. 48:3-87.13 community-solar program plus
//! N.J.A.C. 14:4-9.1 PPA disclosure rules. Massachusetts: 220 CMR 18.00 net
//! metering plus SMART (Solar Massachusetts Renewable Target) declining-
//! block program effective November 26, 2018 governing SREC successor.
//! Arizona: A.A.C. R14-2-1801 et seq. plus House Bill 2675 (2021)
//! requiring written disclosure of solar lease/PPA terms before sale or
//! lease transfer. Hawaii: H.R.S. § 269-101.5 (interconnection) plus Customer
//! Self-Supply (CSS) plus Customer Grid-Supply Plus (CGS+) tariffs since
//! October 21, 2015 (NEM closed to new customers). Federal: 26 U.S.C.
//! § 48 ITC (Investment Tax Credit) — only the OWNER of the system can
//! claim the credit; PPA host customers (tenants or landlords paying per-
//! kWh) do NOT qualify per Notice 2018-59. CERCLA / EPA Hazardous Waste
//! considerations for cracked-panel cadmium-telluride disposal (Universal
//! Waste rule 40 C.F.R. Part 273) on tenant turnover.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    Massachusetts,
    NewJersey,
    Arizona,
    Hawaii,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolarFinancingStructure {
    NoSolar,
    OutrightOwnedByLandlord,
    FinancedLoanOwnedByLandlord,
    SolarLeaseFixedMonthly,
    SolarPpaPerKwh,
    CommunitySolarSubscription,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetMeteringCreditBeneficiary {
    LandlordOnly,
    TenantOnly,
    SplitProRata,
    NoNetMetering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Compliant,
    DisclosureRequiredAtLeaseSigning,
    NetMeteringCreditMisallocation,
    PpaPassThroughUnauthorized,
    RoofWarrantyVoidRisk,
    PpaAccelerationTenantExposure,
    UniversalWasteDisposalRiskOnPanelDamage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub financing_structure: SolarFinancingStructure,
    pub net_metering_credit: NetMeteringCreditBeneficiary,
    pub system_size_kw: u32,
    pub monthly_lease_or_ppa_payment_cents: u64,
    pub estimated_monthly_generation_kwh: u32,
    pub disclosure_provided_at_lease_signing: bool,
    pub disclosure_includes_financing_structure: bool,
    pub disclosure_includes_net_metering_allocation: bool,
    pub disclosure_includes_pass_through_charge: bool,
    pub lease_authorizes_pass_through_charge: bool,
    pub roof_warranty_active_with_installer_endorsement: bool,
    pub annual_rent_cents: u64,
    pub tenant_aware_of_panel_damage_universal_waste_rule: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub annual_rent_at_risk_cents: u64,
    pub estimated_net_metering_credit_to_misallocated_party_cents: u64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const ESTIMATED_RETAIL_KWH_RATE_CENTS: u64 = 25;
pub const ROOF_WARRANTY_AVERAGE_REMAINING_YEARS: u32 = 12;
pub const PPA_DEFAULT_REMAINING_TERM_MONTHS_AVG: u32 = 240;

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;
    let mut credit_misallocation_cents: u64 = 0;

    if matches!(input.financing_structure, SolarFinancingStructure::NoSolar) {
        notes.push(
            "No solar PV system on premises; framework inapplicable. If tenant requests \
             portable / plug-in solar, route to [[tenant_solar_installation]] right-to-install \
             module."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            estimated_net_metering_credit_to_misallocated_party_cents: 0,
            citation: "n/a",
            notes,
        };
    }

    let is_third_party_financed = matches!(
        input.financing_structure,
        SolarFinancingStructure::SolarLeaseFixedMonthly | SolarFinancingStructure::SolarPpaPerKwh
    );

    let net_meter_misallocation = matches!(
        input.net_metering_credit,
        NetMeteringCreditBeneficiary::LandlordOnly
    ) && !input.disclosure_includes_net_metering_allocation;

    if net_meter_misallocation {
        let monthly_credit =
            u64::from(input.estimated_monthly_generation_kwh) * ESTIMATED_RETAIL_KWH_RATE_CENTS;
        credit_misallocation_cents = monthly_credit.saturating_mul(12);
        severity = Severity::NetMeteringCreditMisallocation;
        actions.push(format!(
            "Tenant pays utility bill while landlord receives net-metering credit; this \
             constitutes an undisclosed economic offset of approximately {} cents per year. \
             Either (1) restructure lease so tenant receives credit, (2) reduce rent by \
             corresponding amount, or (3) provide explicit disclosure at lease signing per \
             Cal. Civ. Code § 1947.13 (CA), N.J.A.C. 14:4-9.1 (NJ), or A.R.S. § 33-1310 (AZ).",
            credit_misallocation_cents
        ));
    } else if is_third_party_financed
        && input.monthly_lease_or_ppa_payment_cents > 0
        && !input.disclosure_includes_pass_through_charge
    {
        severity = Severity::DisclosureRequiredAtLeaseSigning;
        actions.push(
            "Third-party solar lease or PPA monthly payment NOT disclosed at lease signing; \
             provide written disclosure containing (1) financing structure, (2) monthly \
             payment amount, (3) escalator clause, (4) remaining term length, (5) buyout \
             provisions, (6) acceleration-on-default provisions, (7) panel ownership at \
             lease termination per California Senate Bill 1340 (2018) plus Cal. Civ. Code \
             § 1102.6c sale-disclosure analog."
                .to_string(),
        );
    } else if is_third_party_financed
        && input.monthly_lease_or_ppa_payment_cents > 0
        && !input.lease_authorizes_pass_through_charge
    {
        severity = Severity::PpaPassThroughUnauthorized;
        actions.push(
            "Solar PPA / lease monthly payment passed through to tenant WITHOUT explicit lease \
             clause authorizing the charge; in CA this violates Cal. Civ. Code § 1947 / § \
             1947.13 (rent is the full agreed amount — additional charges require lease \
             authorization). Cease pass-through; refund any tenant payments collected without \
             lease authorization."
                .to_string(),
        );
    } else if matches!(
        input.financing_structure,
        SolarFinancingStructure::SolarPpaPerKwh
    ) && !input.tenant_aware_of_panel_damage_universal_waste_rule
    {
        severity = Severity::UniversalWasteDisposalRiskOnPanelDamage;
        actions.push(
            "Tenant not advised of cracked / damaged solar panel Universal Waste protocol \
             under 40 C.F.R. Part 273; CdTe (cadmium-telluride) thin-film panels require \
             RCRA-compliant disposal at landlord cost; provide written notice that tenant \
             MUST NOT dispose of broken panels in normal trash."
                .to_string(),
        );
    } else if is_third_party_financed && !input.roof_warranty_active_with_installer_endorsement {
        severity = Severity::RoofWarrantyVoidRisk;
        actions.push(format!(
            "Solar mount penetrations through roof voided installer-warranty endorsement; \
             approximately {} years of remaining roof warranty at risk. Obtain installer-\
             endorsed warranty rider before claiming compliance; otherwise reserve for \
             roof-replacement cost on tenant-attributable leak claim.",
            ROOF_WARRANTY_AVERAGE_REMAINING_YEARS
        ));
    } else if is_third_party_financed && !input.disclosure_provided_at_lease_signing {
        severity = Severity::DisclosureRequiredAtLeaseSigning;
        actions.push(
            "Solar lease / PPA arrangement exists but no written disclosure provided at \
             lease signing; create disclosure addendum covering financing structure, monthly \
             payment, net-metering allocation, panel-damage responsibility, and PPA \
             acceleration clause."
                .to_string(),
        );
    } else if is_third_party_financed && !input.disclosure_includes_financing_structure {
        severity = Severity::DisclosureRequiredAtLeaseSigning;
        actions.push(
            "Disclosure incomplete: financing structure (lease vs PPA vs community subscription) \
             must be specified explicitly so tenant understands monthly-payment vs per-kWh \
             distinction."
                .to_string(),
        );
    } else if is_third_party_financed && input.monthly_lease_or_ppa_payment_cents > 0 {
        severity = Severity::PpaAccelerationTenantExposure;
        actions.push(format!(
            "Lease authorizes PPA pass-through; advise tenant that PPA default acceleration \
             clause may accelerate remaining ~{} months of PPA payments on early termination. \
             Cap tenant exposure to current-period payment only; landlord bears acceleration \
             risk per lease addendum.",
            PPA_DEFAULT_REMAINING_TERM_MONTHS_AVG
        ));
    } else {
        severity = Severity::Compliant;
        actions.push(
            "Disclosure framework compliant; retain solar financing documentation (PPA \
             contract, lease, monthly invoices, net-metering credit statements) for full \
             tenancy plus statute-of-limitations window."
                .to_string(),
        );
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            notes.push(
                "Cal. Civ. Code § 1102.6c (solar contract disclosure on transfer — analog \
                 applied to rental); Cal. Pub. Util. Code § 2827 (NEM 1.0 + NEM 2.0); NEM \
                 3.0 / NBT Net Billing Tariff effective April 15, 2023 governing new \
                 residential interconnections (export credits roughly 75% lower than NEM \
                 2.0 retail rate). Cal. Civ. Code § 1947 / § 1947.13 implicit limitation \
                 on additional charges beyond agreed rent. CA Senate Bill 1340 (2018) \
                 solar consumer protection."
                    .to_string(),
            );
        }
        Jurisdiction::Massachusetts => {
            notes.push(
                "220 CMR 18.00 (net metering); SMART (Solar Massachusetts Renewable Target) \
                 declining-block program effective November 26, 2018 superseding SREC II for \
                 new installations; M.G.L. ch. 164 § 138-140 net-metering framework. \
                 Massachusetts DPU Docket 11-11 governs net metering credit transfers."
                    .to_string(),
            );
        }
        Jurisdiction::NewJersey => {
            notes.push(
                "N.J.S.A. 48:3-87.13 (community solar pilot program); N.J.A.C. 14:4-9.1 PPA \
                 disclosure rules; N.J. Solar Successor Incentive Program (SuSI) replaced \
                 SREC market effective August 28, 2021 with TREC (Transition Renewable Energy \
                 Certificate) plus SREC-II programs."
                    .to_string(),
            );
        }
        Jurisdiction::Arizona => {
            notes.push(
                "A.A.C. R14-2-1801 et seq. (ACC solar rules); A.R.S. § 33-1310 (residential \
                 landlord-tenant act); Arizona House Bill 2675 (2021) requiring written \
                 disclosure of solar lease / PPA terms before sale or lease transfer including \
                 monthly payment, escalator clause, remaining term, buyout amount."
                    .to_string(),
            );
        }
        Jurisdiction::Hawaii => {
            notes.push(
                "H.R.S. § 269-101.5 (interconnection); Customer Self-Supply (CSS) plus Customer \
                 Grid-Supply Plus (CGS+) tariffs since October 21, 2015 (NEM closed to new \
                 residential customers). Smart Export tariff approved March 25, 2020. PUC \
                 Docket 2014-0192 governs successor tariffs."
                    .to_string(),
            );
        }
        Jurisdiction::NewYork => {
            notes.push(
                "16 NYCRR Part 96 (NY PSC net metering); Value of Distributed Energy Resources \
                 (VDER) Value Stack tariff superseding net metering effective March 9, 2017 \
                 for non-residential plus community DG; residential remains traditional NEM. \
                 NY Real Property Law § 235-b implicit warranty of habitability standard."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(
                "Federal 26 U.S.C. § 48 ITC — only system owner claims credit; PPA host \
                 customers do NOT qualify per Notice 2018-59. Universal Waste rule 40 C.F.R. \
                 Part 273 governs cracked-panel cadmium-telluride disposal. State-level net \
                 metering policy varies widely; consult state PUC tariff schedule."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordination with [[tenant_solar_installation]] (tenant right-to-install on portable \
         / plug-in systems where landlord has no existing PV), [[rental_propane_tank_lease_\
         disclosure]] (lease-pass-through analog for energy infrastructure), [[rental_oil_\
         tank_replacement_disclosure]] (energy-infrastructure disclosure pattern), [[ev_charger_\
         installation]] (parallel tenant-right-to-charge framework)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::NetMeteringCreditMisallocation | Severity::PpaPassThroughUnauthorized => {
            input.annual_rent_cents
        }
        Severity::DisclosureRequiredAtLeaseSigning
        | Severity::RoofWarrantyVoidRisk
        | Severity::PpaAccelerationTenantExposure
        | Severity::UniversalWasteDisposalRiskOnPanelDamage => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        estimated_net_metering_credit_to_misallocated_party_cents: credit_misallocation_cents,
        citation: match input.jurisdiction {
            Jurisdiction::California => {
                "Cal. Civ. Code § 1102.6c + § 1947.13 + § 2827 PUC + SB 1340"
            }
            Jurisdiction::Massachusetts => "220 CMR 18.00 + M.G.L. ch. 164 § 138-140 + SMART",
            Jurisdiction::NewJersey => "N.J.S.A. 48:3-87.13 + N.J.A.C. 14:4-9.1 + SuSI",
            Jurisdiction::Arizona => "A.A.C. R14-2-1801 + A.R.S. § 33-1310 + HB 2675",
            Jurisdiction::Hawaii => "H.R.S. § 269-101.5 + CSS + CGS+ tariffs",
            Jurisdiction::NewYork => "16 NYCRR Part 96 + VDER Value Stack + § 235-b",
            Jurisdiction::Default => "26 U.S.C. § 48 ITC + 40 C.F.R. Part 273 + state PUC",
        },
        notes,
    }
}

pub type RentalSolarPanelDisclosureInput = Input;
pub type RentalSolarPanelDisclosureResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            financing_structure: SolarFinancingStructure::OutrightOwnedByLandlord,
            net_metering_credit: NetMeteringCreditBeneficiary::TenantOnly,
            system_size_kw: 6,
            monthly_lease_or_ppa_payment_cents: 0,
            estimated_monthly_generation_kwh: 750,
            disclosure_provided_at_lease_signing: true,
            disclosure_includes_financing_structure: true,
            disclosure_includes_net_metering_allocation: true,
            disclosure_includes_pass_through_charge: true,
            lease_authorizes_pass_through_charge: true,
            roof_warranty_active_with_installer_endorsement: true,
            annual_rent_cents: 36_000_00,
            tenant_aware_of_panel_damage_universal_waste_rule: true,
        }
    }

    #[test]
    fn no_solar_returns_not_applicable() {
        let mut i = baseline();
        i.financing_structure = SolarFinancingStructure::NoSolar;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert_eq!(r.citation, "n/a");
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_solar_installation")));
    }

    #[test]
    fn outright_owned_with_tenant_credit_compliant() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::Compliant));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn net_metering_credit_landlord_undisclosed_triggers_misallocation() {
        let mut i = baseline();
        i.net_metering_credit = NetMeteringCreditBeneficiary::LandlordOnly;
        i.disclosure_includes_net_metering_allocation = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::NetMeteringCreditMisallocation
        ));
        let expected: u64 =
            u64::from(i.estimated_monthly_generation_kwh) * ESTIMATED_RETAIL_KWH_RATE_CENTS * 12;
        assert_eq!(
            r.estimated_net_metering_credit_to_misallocated_party_cents,
            expected
        );
    }

    #[test]
    fn net_metering_landlord_with_disclosure_compliant() {
        let mut i = baseline();
        i.net_metering_credit = NetMeteringCreditBeneficiary::LandlordOnly;
        i.disclosure_includes_net_metering_allocation = true;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::Compliant));
    }

    #[test]
    fn ppa_no_disclosure_at_lease_signing_triggers_required_status() {
        let mut i = baseline();
        i.financing_structure = SolarFinancingStructure::SolarPpaPerKwh;
        i.monthly_lease_or_ppa_payment_cents = 150_00;
        i.disclosure_includes_pass_through_charge = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DisclosureRequiredAtLeaseSigning
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("California Senate Bill 1340")));
    }

    #[test]
    fn ppa_pass_through_without_lease_authorization_violation() {
        let mut i = baseline();
        i.financing_structure = SolarFinancingStructure::SolarPpaPerKwh;
        i.monthly_lease_or_ppa_payment_cents = 150_00;
        i.disclosure_includes_pass_through_charge = true;
        i.lease_authorizes_pass_through_charge = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::PpaPassThroughUnauthorized));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Cal. Civ. Code § 1947")));
    }

    #[test]
    fn solar_lease_with_pass_through_authorized_and_roof_warranty_compliant() {
        let mut i = baseline();
        i.financing_structure = SolarFinancingStructure::SolarLeaseFixedMonthly;
        i.monthly_lease_or_ppa_payment_cents = 175_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::PpaAccelerationTenantExposure
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("acceleration")));
    }

    #[test]
    fn ppa_universal_waste_unaware_triggers_disposal_risk() {
        let mut i = baseline();
        i.financing_structure = SolarFinancingStructure::SolarPpaPerKwh;
        i.monthly_lease_or_ppa_payment_cents = 0;
        i.tenant_aware_of_panel_damage_universal_waste_rule = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::UniversalWasteDisposalRiskOnPanelDamage
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("40 C.F.R. Part 273")));
    }

    #[test]
    fn third_party_financed_no_roof_warranty_endorsement_risk() {
        let mut i = baseline();
        i.financing_structure = SolarFinancingStructure::SolarLeaseFixedMonthly;
        i.monthly_lease_or_ppa_payment_cents = 0;
        i.roof_warranty_active_with_installer_endorsement = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::RoofWarrantyVoidRisk));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains(&ROOF_WARRANTY_AVERAGE_REMAINING_YEARS.to_string())));
    }

    #[test]
    fn community_solar_subscription_no_panel_on_premises_compliant_if_disclosed() {
        let mut i = baseline();
        i.financing_structure = SolarFinancingStructure::CommunitySolarSubscription;
        i.monthly_lease_or_ppa_payment_cents = 40_00;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::Compliant));
    }

    #[test]
    fn ca_jurisdiction_pins_nem_3_nbt_effective_date() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("April 15, 2023")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Cal. Civ. Code § 1102.6c")));
        assert!(r.citation.contains("SB 1340"));
    }

    #[test]
    fn ca_jurisdiction_pins_section_1947_13() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 1947.13")));
    }

    #[test]
    fn ma_jurisdiction_pins_smart_program() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("SMART")));
        assert!(r.notes.iter().any(|n| n.contains("220 CMR 18.00")));
        assert!(r.notes.iter().any(|n| n.contains("November 26, 2018")));
    }

    #[test]
    fn nj_jurisdiction_pins_susi_replacement_for_srec() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewJersey;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("SuSI")));
        assert!(r.notes.iter().any(|n| n.contains("N.J.S.A. 48:3-87.13")));
        assert!(r.notes.iter().any(|n| n.contains("August 28, 2021")));
    }

    #[test]
    fn az_jurisdiction_pins_hb_2675_disclosure_rule() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Arizona;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("House Bill 2675")));
        assert!(r.notes.iter().any(|n| n.contains("A.R.S. § 33-1310")));
    }

    #[test]
    fn hi_jurisdiction_pins_css_cgs_closure_of_nem() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Hawaii;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Customer Self-Supply")));
        assert!(r.notes.iter().any(|n| n.contains("October 21, 2015")));
        assert!(r.notes.iter().any(|n| n.contains("Smart Export")));
    }

    #[test]
    fn ny_jurisdiction_pins_vder_value_stack() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("VDER")));
        assert!(r.notes.iter().any(|n| n.contains("16 NYCRR Part 96")));
        assert!(r.notes.iter().any(|n| n.contains("March 9, 2017")));
    }

    #[test]
    fn default_jurisdiction_pins_federal_itc_notice_2018_59() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("26 U.S.C. § 48")));
        assert!(r.notes.iter().any(|n| n.contains("Notice 2018-59")));
        assert!(r.notes.iter().any(|n| n.contains("40 C.F.R. Part 273")));
    }

    #[test]
    fn coordination_note_references_tenant_solar_install_and_oil_tank() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_solar_installation")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_oil_tank_replacement_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_propane_tank_lease_disclosure")));
    }

    #[test]
    fn coordination_note_pinned_in_all_jurisdictions() {
        for j in [
            Jurisdiction::California,
            Jurisdiction::Massachusetts,
            Jurisdiction::NewJersey,
            Jurisdiction::Arizona,
            Jurisdiction::Hawaii,
            Jurisdiction::NewYork,
            Jurisdiction::Default,
        ] {
            let mut i = baseline();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("tenant_solar_installation")),
                "coordination missing for {j:?}"
            );
        }
    }

    #[test]
    fn full_rent_at_risk_for_credit_misallocation() {
        let mut i = baseline();
        i.net_metering_credit = NetMeteringCreditBeneficiary::LandlordOnly;
        i.disclosure_includes_net_metering_allocation = false;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
    }

    #[test]
    fn full_rent_at_risk_for_unauthorized_pass_through() {
        let mut i = baseline();
        i.financing_structure = SolarFinancingStructure::SolarPpaPerKwh;
        i.monthly_lease_or_ppa_payment_cents = 100_00;
        i.lease_authorizes_pass_through_charge = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::PpaPassThroughUnauthorized));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
    }

    #[test]
    fn half_rent_at_risk_for_disclosure_required() {
        let mut i = baseline();
        i.financing_structure = SolarFinancingStructure::SolarPpaPerKwh;
        i.monthly_lease_or_ppa_payment_cents = 100_00;
        i.disclosure_includes_pass_through_charge = false;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
    }

    #[test]
    fn saturating_mul_does_not_panic_on_extreme_generation_kwh() {
        let mut i = baseline();
        i.net_metering_credit = NetMeteringCreditBeneficiary::LandlordOnly;
        i.disclosure_includes_net_metering_allocation = false;
        i.estimated_monthly_generation_kwh = u32::MAX;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::NetMeteringCreditMisallocation
        ));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.net_metering_credit = NetMeteringCreditBeneficiary::LandlordOnly;
        i.disclosure_includes_net_metering_allocation = false;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn rate_constant_pins_25_cents_per_kwh() {
        assert_eq!(ESTIMATED_RETAIL_KWH_RATE_CENTS, 25);
    }

    #[test]
    fn ppa_default_remaining_term_pins_240_months() {
        assert_eq!(PPA_DEFAULT_REMAINING_TERM_MONTHS_AVG, 240);
    }

    #[test]
    fn financed_loan_owned_treated_as_owned_compliant() {
        let mut i = baseline();
        i.financing_structure = SolarFinancingStructure::FinancedLoanOwnedByLandlord;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::Compliant));
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let ca = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::California;
            i
        });
        let ma = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Massachusetts;
            i
        });
        let nj = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::NewJersey;
            i
        });
        let az = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Arizona;
            i
        });
        let hi = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Hawaii;
            i
        });
        let ny = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::NewYork;
            i
        });
        let de = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Default;
            i
        });
        assert!(ca.citation.contains("Cal. Civ. Code"));
        assert!(ma.citation.contains("SMART"));
        assert!(nj.citation.contains("SuSI"));
        assert!(az.citation.contains("HB 2675"));
        assert!(hi.citation.contains("CSS"));
        assert!(ny.citation.contains("VDER"));
        assert!(de.citation.contains("ITC"));
    }
}
