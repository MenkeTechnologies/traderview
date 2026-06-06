//! Multi-jurisdictional residential tenant late fee cap
//! framework. Trader-landlord critical because late-fee
//! over-collection is one of the most common landlord
//! mistakes — each over-cap charge can trigger statutory
//! damages plus attorney fees. The four-state regime
//! comparison reveals dramatically different rule
//! architectures: NY hard cap (lesser of $50 OR 5%), CA
//! reasonableness/liquidated-damages test, TX percentage
//! safe-harbor with treble damages, FL manufactured-home
//! park reasonableness regime. Companion to
//! rental_security_deposit_interest,
//! landlord_self_help_eviction_prohibition,
//! landlord_retaliation_damages,
//! rental_junk_fee_transparency.
//!
//! **California Civ. Code § 1671(d) + Orozco v. Casimiro,
//! 121 Cal.App.4th Supp. 7 (Cal. App. Dep't Super. Ct.
//! 2004)** — late fees in residential leases treated as
//! LIQUIDATED DAMAGES. Two-prong validity test:
//! 1. It was IMPRACTICABLE OR EXTREMELY DIFFICULT to fix
//!    actual damage at lease execution; AND
//! 2. The amount is a REASONABLE ENDEAVOR to estimate FAIR
//!    AVERAGE COMPENSATION for loss sustained.
//!
//! Orozco v. Casimiro invalidated a $50 automatic late
//! fee — court held that the landlord failed to establish
//! either prong. California courts consistently invalidate
//! late fees exceeding **5-6% of monthly rent** as
//! unreasonable liquidated damages. Late fee struck down
//! is unenforceable in toto — landlord recovers $0 in late
//! fees on that lease.
//!
//! **New York Real Prop. Law § 238-a (HSTPA of 2019,
//! effective June 14, 2019)** — STATUTORY HARD CAP:
//! 1. Late fee may NOT exceed **LESSER of $50 OR 5% of
//!    monthly rent**;
//! 2. Mandatory **5-day GRACE PERIOD** — no late fee may
//!    be demanded unless rent is unpaid for at least 5
//!    days past due date;
//! 3. Landlord CANNOT EVICT solely for unpaid late fees
//!    (warrant of eviction tracks rent + permissible
//!    fees only).
//!
//! Strictest residential late-fee regime in the country.
//! Applies to all residential tenancies, including rent-
//! regulated and rent-stabilized apartments.
//!
//! **Florida Stat. § 83.808 (manufactured home park
//! rentals)** — reasonable late fee permitted. Common
//! guidance: $20 OR 20% of monthly rent, whichever is
//! greater. General residential FL (Chapter 83 Part II)
//! has NO statutory cap on late fees — court reasonable
//! standard governs. Lease must specify late fee amount;
//! § 83.49 deposit framework analogously requires
//! disclosure.
//!
//! **Texas Prop. Code § 92.019 (revised 2019)** —
//! safe-harbor percentage framework:
//! 1. **12% of rent** if structure contains 4 OR FEWER
//!    dwelling units;
//! 2. **10% of rent** if structure contains MORE THAN 4
//!    dwelling units;
//! 3. Mandatory **2-day GRACE PERIOD** — no late fee for
//!    rent unpaid until 2 full days after rent due date;
//! 4. Initial fee + daily fee combined within single cap.
//!
//! § 92.019 violation remedy: tenant entitled to **$100 +
//! 3X (TREBLE) the late fee collected in violation + reasonable
//! attorney's fees**.
//!
//! **Default — common-law liquidated damages doctrine** —
//! late fee must (1) bear reasonable relationship to
//! actual or anticipated damages; (2) NOT operate as a
//! penalty. Punitive fees void as against public policy.
//! Restatement (Second) of Contracts § 356.
//!
//! Citations: Cal. Civ. Code § 1671(d); Orozco v.
//! Casimiro, 121 Cal.App.4th Supp. 7 (2004); N.Y. Real
//! Prop. Law § 238-a (HSTPA of 2019); Fla. Stat. § 83.808;
//! Tex. Prop. Code § 92.019; Restatement (Second) of
//! Contracts § 356.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Florida,
    Texas,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantLateFeeCapInput {
    pub jurisdiction: Jurisdiction,
    /// Monthly rent in cents.
    pub monthly_rent_cents: u64,
    /// Late fee charged by landlord in cents.
    pub late_fee_charged_cents: u64,
    /// Days since rent due date when landlord first
    /// charged the late fee (grace-period gate).
    pub days_after_due_when_charged: u32,
    /// Whether lease specifies the late fee amount
    /// (CA + FL require disclosure).
    pub lease_specifies_late_fee: bool,
    /// Number of dwelling units in structure (TX 4-unit
    /// pivot).
    pub dwelling_unit_count: u32,
    /// Whether late fee provision was a REASONABLE
    /// ENDEAVOR to estimate fair average compensation for
    /// loss sustained (CA Orozco prong 2).
    pub reasonable_endeavor_to_estimate: bool,
    /// Whether actual damage was IMPRACTICABLE OR
    /// EXTREMELY DIFFICULT to fix at lease execution (CA
    /// Orozco prong 1).
    pub impracticable_to_fix_actual_damage: bool,
    /// Whether property is manufactured home park (FL
    /// § 83.808 vs general FL Chapter 83 Part II).
    pub manufactured_home_park: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantLateFeeCapResult {
    pub jurisdiction: Jurisdiction,
    pub statutory_cap_cents: u64,
    pub late_fee_compliant: bool,
    pub late_fee_excess_cents: u64,
    pub grace_period_satisfied: bool,
    pub tenant_damages_cents: u64,
    pub treble_damages_engaged: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &TenantLateFeeCapInput) -> TenantLateFeeCapResult {
    let mut failure_reasons: Vec<String> = Vec::new();
    let statutory_cap_cents: u64;
    let grace_period_days_required: u32;
    let mut tenant_damages_cents: u64 = 0;
    let mut treble_damages_engaged = false;

    match input.jurisdiction {
        Jurisdiction::California => {
            statutory_cap_cents = input.monthly_rent_cents.saturating_mul(600) / 10_000;
            grace_period_days_required = 0;
            if !input.impracticable_to_fix_actual_damage || !input.reasonable_endeavor_to_estimate {
                failure_reasons.push(
                    "Cal. Civ. Code § 1671(d) + Orozco v. Casimiro, 121 Cal.App.4th Supp. 7 (2004) — late fee VOID as unenforceable liquidated damages unless landlord proves BOTH (1) it was impracticable or extremely difficult to fix actual damage at lease execution AND (2) the fee was a reasonable endeavor to estimate fair average compensation for loss sustained".to_string(),
                );
                if input.late_fee_charged_cents > 0 {
                    tenant_damages_cents = input.late_fee_charged_cents;
                }
            }
        }
        Jurisdiction::NewYork => {
            let five_percent = input.monthly_rent_cents.saturating_mul(500) / 10_000;
            statutory_cap_cents = five_percent.min(5_000);
            grace_period_days_required = 5;
        }
        Jurisdiction::Florida => {
            statutory_cap_cents = if input.manufactured_home_park {
                let twenty_percent = input.monthly_rent_cents.saturating_mul(2000) / 10_000;
                twenty_percent.max(2_000)
            } else {
                input.monthly_rent_cents.saturating_mul(2000) / 10_000
            };
            grace_period_days_required = 0;
        }
        Jurisdiction::Texas => {
            let safe_harbor_bps: u32 = if input.dwelling_unit_count > 4 {
                1000
            } else {
                1200
            };
            statutory_cap_cents = input
                .monthly_rent_cents
                .saturating_mul(safe_harbor_bps as u64)
                / 10_000;
            grace_period_days_required = 2;
        }
        Jurisdiction::Default => {
            statutory_cap_cents = input.monthly_rent_cents.saturating_mul(600) / 10_000;
            grace_period_days_required = 0;
        }
    }

    let late_fee_excess_cents = input
        .late_fee_charged_cents
        .saturating_sub(statutory_cap_cents);
    let late_fee_compliant_amount = input.late_fee_charged_cents <= statutory_cap_cents;
    let grace_period_satisfied = input.days_after_due_when_charged >= grace_period_days_required;

    let mut late_fee_compliant = late_fee_compliant_amount && grace_period_satisfied;

    if input.jurisdiction == Jurisdiction::California
        && (!input.impracticable_to_fix_actual_damage || !input.reasonable_endeavor_to_estimate)
    {
        late_fee_compliant = false;
    }

    if late_fee_excess_cents > 0 {
        match input.jurisdiction {
            Jurisdiction::NewYork => {
                failure_reasons.push(format!(
                    "N.Y. Real Prop. Law § 238-a (HSTPA of 2019) — late fee EXCEEDS STATUTORY HARD CAP of LESSER of $50 OR 5% of monthly rent (cap = ${} cents; charged ${} cents; excess ${} cents)",
                    statutory_cap_cents, input.late_fee_charged_cents, late_fee_excess_cents
                ));
            }
            Jurisdiction::Texas => {
                tenant_damages_cents =
                    10_000_u64.saturating_add(input.late_fee_charged_cents.saturating_mul(3));
                treble_damages_engaged = true;
                failure_reasons.push(format!(
                    "Tex. Prop. Code § 92.019(d) — late fee EXCEEDS safe-harbor cap ({} bps of monthly rent for {}-unit structure); tenant remedy: $100 + TREBLE the late fee collected + reasonable attorney's fees ({} cents total)",
                    if input.dwelling_unit_count > 4 { 1000 } else { 1200 },
                    input.dwelling_unit_count,
                    tenant_damages_cents
                ));
            }
            Jurisdiction::Florida => {
                failure_reasons.push(format!(
                    "Fla. Stat. {} — late fee EXCEEDS reasonable cap; common standard $20 or 20% of monthly rent (whichever {})",
                    if input.manufactured_home_park { "§ 83.808" } else { "Chapter 83 Part II (no statutory cap; court reasonableness)" },
                    if input.manufactured_home_park { "greater for manufactured home park" } else { "applies" }
                ));
            }
            Jurisdiction::Default => {
                failure_reasons.push(
                    "Common-law liquidated damages doctrine (Restatement (Second) of Contracts § 356) — late fee must (1) bear reasonable relationship to actual or anticipated damages and (2) NOT operate as a penalty; punitive fees void as against public policy".to_string(),
                );
            }
            Jurisdiction::California => {}
        }
    }

    if !grace_period_satisfied {
        match input.jurisdiction {
            Jurisdiction::NewYork => {
                failure_reasons.push(format!(
                    "N.Y. Real Prop. Law § 238-a — mandatory 5-DAY GRACE PERIOD; no late fee may be demanded unless rent unpaid for at least 5 DAYS past due date ({} days)",
                    input.days_after_due_when_charged
                ));
            }
            Jurisdiction::Texas => {
                failure_reasons.push(format!(
                    "Tex. Prop. Code § 92.019(a)(2) — mandatory 2-DAY GRACE PERIOD; no late fee for rent unpaid until 2 full days after rent due date ({} days)",
                    input.days_after_due_when_charged
                ));
            }
            _ => {}
        }
    }

    if input.jurisdiction == Jurisdiction::NewYork && input.late_fee_charged_cents > 0 {
        failure_reasons.push(
            "N.Y. Real Prop. Law § 238-a — landlord CANNOT EVICT solely for unpaid late fees; warrant of eviction tracks rent + permissible fees only".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1671(d) + Orozco v. Casimiro, 121 Cal.App.4th Supp. 7 (2004) — residential late fees treated as LIQUIDATED DAMAGES; two-prong validity test (impracticable to fix actual damage AND reasonable endeavor to estimate fair average compensation)".to_string(),
        "Cal. Civ. Code § 1671(d) — California courts consistently invalidate late fees EXCEEDING 5-6% of monthly rent as unreasonable liquidated damages; invalidated fee is void in toto (landlord recovers $0)".to_string(),
        "N.Y. Real Prop. Law § 238-a (HSTPA of 2019, effective June 14, 2019) — STATUTORY HARD CAP: late fee may NOT exceed LESSER of $50 OR 5% of monthly rent".to_string(),
        "N.Y. Real Prop. Law § 238-a — mandatory 5-DAY GRACE PERIOD; landlord CANNOT EVICT solely for unpaid late fees".to_string(),
        "Fla. Stat. § 83.808 — manufactured home park rentals: reasonable late fee permitted; common guidance: $20 OR 20% of monthly rent (whichever GREATER)".to_string(),
        "Fla. Chapter 83 Part II — general residential Florida: NO statutory cap on late fees; court reasonableness standard governs; lease must specify late fee amount".to_string(),
        "Tex. Prop. Code § 92.019 (revised 2019) — safe-harbor percentage framework: 12% for structures of 4 or fewer units, 10% for structures of more than 4 units".to_string(),
        "Tex. Prop. Code § 92.019(a)(2) — mandatory 2-DAY GRACE PERIOD; no late fee until rent unpaid 2 full days past due".to_string(),
        "Tex. Prop. Code § 92.019(d) — violation remedy: tenant entitled to $100 PLUS TREBLE the late fee collected in violation PLUS reasonable attorney's fees".to_string(),
        "Default — common-law liquidated damages doctrine (Restatement (Second) of Contracts § 356): late fee must (1) bear reasonable relationship to actual or anticipated damages AND (2) NOT operate as a penalty; punitive fees void as against public policy".to_string(),
        "Cross-jurisdictional invariant: California uses LIQUIDATED-DAMAGES REASONABLENESS test; New York uses STATUTORY HARD CAP; Texas uses PERCENTAGE SAFE HARBOR; Florida uses REASONABLENESS unless manufactured home park; Default uses common-law liquidated damages".to_string(),
    ];

    TenantLateFeeCapResult {
        jurisdiction: input.jurisdiction,
        statutory_cap_cents,
        late_fee_compliant,
        late_fee_excess_cents,
        grace_period_satisfied,
        tenant_damages_cents,
        treble_damages_engaged,
        failure_reasons,
        citation: "Cal. Civ. Code § 1671(d); Orozco v. Casimiro, 121 Cal.App.4th Supp. 7 (2004); N.Y. Real Prop. Law § 238-a (HSTPA of 2019); Fla. Stat. § 83.808 (manufactured home park); Tex. Prop. Code § 92.019 (revised 2019); Restatement (Second) of Contracts § 356",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant() -> TenantLateFeeCapInput {
        TenantLateFeeCapInput {
            jurisdiction: Jurisdiction::California,
            monthly_rent_cents: 200_000,
            late_fee_charged_cents: 10_000,
            days_after_due_when_charged: 5,
            lease_specifies_late_fee: true,
            dwelling_unit_count: 4,
            reasonable_endeavor_to_estimate: true,
            impracticable_to_fix_actual_damage: true,
            manufactured_home_park: false,
        }
    }

    #[test]
    fn ca_5_percent_late_fee_with_both_prongs_compliant() {
        let r = check(&ca_compliant());
        assert!(r.late_fee_compliant);
        assert_eq!(r.statutory_cap_cents, 12_000);
        assert_eq!(r.late_fee_excess_cents, 0);
    }

    #[test]
    fn ca_missing_impracticable_prong_voids_fee() {
        let mut i = ca_compliant();
        i.impracticable_to_fix_actual_damage = false;
        let r = check(&i);
        assert!(!r.late_fee_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1671(d)")
            && f.contains("Orozco")
            && f.contains("impracticable")));
    }

    #[test]
    fn ca_missing_reasonable_endeavor_prong_voids_fee() {
        let mut i = ca_compliant();
        i.reasonable_endeavor_to_estimate = false;
        let r = check(&i);
        assert!(!r.late_fee_compliant);
    }

    #[test]
    fn ca_void_fee_recovers_charged_amount_as_damages() {
        let mut i = ca_compliant();
        i.impracticable_to_fix_actual_damage = false;
        i.late_fee_charged_cents = 50_000;
        let r = check(&i);
        assert_eq!(r.tenant_damages_cents, 50_000);
    }

    #[test]
    fn ny_50_dollar_or_5_percent_lesser_cap() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.monthly_rent_cents = 200_000;
        let r = check(&i);
        assert_eq!(r.statutory_cap_cents, 5_000);
    }

    #[test]
    fn ny_low_rent_5_percent_lower_than_50_dollars() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.monthly_rent_cents = 50_000;
        let r = check(&i);
        assert_eq!(r.statutory_cap_cents, 2_500);
    }

    #[test]
    fn ny_late_fee_at_50_dollar_cap_exactly_compliant() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.monthly_rent_cents = 200_000;
        i.late_fee_charged_cents = 5_000;
        i.days_after_due_when_charged = 5;
        let r = check(&i);
        assert!(r.late_fee_compliant);
    }

    #[test]
    fn ny_late_fee_one_cent_above_cap_violation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.monthly_rent_cents = 200_000;
        i.late_fee_charged_cents = 5_001;
        i.days_after_due_when_charged = 5;
        let r = check(&i);
        assert!(!r.late_fee_compliant);
        assert_eq!(r.late_fee_excess_cents, 1);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 238-a") && f.contains("LESSER of $50 OR 5%")));
    }

    #[test]
    fn ny_4_day_grace_period_violation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.days_after_due_when_charged = 4;
        i.late_fee_charged_cents = 3_000;
        let r = check(&i);
        assert!(!r.grace_period_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 238-a") && f.contains("5-DAY GRACE PERIOD")));
    }

    #[test]
    fn ny_5_day_grace_period_boundary_compliant() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.days_after_due_when_charged = 5;
        i.late_fee_charged_cents = 3_000;
        let r = check(&i);
        assert!(r.grace_period_satisfied);
    }

    #[test]
    fn ny_eviction_restriction_note_present() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(
            |f| f.contains("§ 238-a") && f.contains("CANNOT EVICT solely for unpaid late fees")
        ));
    }

    #[test]
    fn fl_manufactured_home_park_20_percent_or_20_dollar_floor() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Florida;
        i.manufactured_home_park = true;
        i.monthly_rent_cents = 50_000;
        let r = check(&i);
        assert_eq!(r.statutory_cap_cents, 10_000);
    }

    #[test]
    fn fl_low_rent_20_dollar_floor_overrides_20_percent() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Florida;
        i.manufactured_home_park = true;
        i.monthly_rent_cents = 5_000;
        let r = check(&i);
        assert_eq!(r.statutory_cap_cents, 2_000);
    }

    #[test]
    fn tx_4_or_fewer_units_12_percent_safe_harbor() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Texas;
        i.dwelling_unit_count = 4;
        i.monthly_rent_cents = 200_000;
        let r = check(&i);
        assert_eq!(r.statutory_cap_cents, 24_000);
    }

    #[test]
    fn tx_5_or_more_units_10_percent_safe_harbor() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Texas;
        i.dwelling_unit_count = 5;
        i.monthly_rent_cents = 200_000;
        let r = check(&i);
        assert_eq!(r.statutory_cap_cents, 20_000);
    }

    #[test]
    fn tx_late_fee_above_cap_triggers_treble_plus_100() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Texas;
        i.dwelling_unit_count = 4;
        i.monthly_rent_cents = 200_000;
        i.late_fee_charged_cents = 30_000;
        i.days_after_due_when_charged = 5;
        let r = check(&i);
        assert!(!r.late_fee_compliant);
        assert!(r.treble_damages_engaged);
        assert_eq!(r.tenant_damages_cents, 10_000 + 30_000 * 3);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 92.019(d)") && f.contains("$100 + TREBLE")));
    }

    #[test]
    fn tx_1_day_grace_period_violation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Texas;
        i.dwelling_unit_count = 4;
        i.monthly_rent_cents = 200_000;
        i.late_fee_charged_cents = 20_000;
        i.days_after_due_when_charged = 1;
        let r = check(&i);
        assert!(!r.grace_period_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 92.019(a)(2)") && f.contains("2-DAY GRACE PERIOD")));
    }

    #[test]
    fn tx_2_day_grace_period_boundary_compliant() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Texas;
        i.dwelling_unit_count = 4;
        i.monthly_rent_cents = 200_000;
        i.late_fee_charged_cents = 20_000;
        i.days_after_due_when_charged = 2;
        let r = check(&i);
        assert!(r.grace_period_satisfied);
    }

    #[test]
    fn default_jurisdiction_5_percent_cap() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Default;
        i.monthly_rent_cents = 200_000;
        let r = check(&i);
        assert_eq!(r.statutory_cap_cents, 12_000);
    }

    #[test]
    fn default_above_cap_violation_cites_restatement() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Default;
        i.late_fee_charged_cents = 50_000;
        let r = check(&i);
        assert!(!r.late_fee_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("Common-law liquidated damages")
                && f.contains("Restatement (Second) of Contracts § 356")));
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("Cal. Civ. Code § 1671(d)"));
        assert!(r.citation.contains("Orozco v. Casimiro"));
        assert!(r.citation.contains("121 Cal.App.4th Supp. 7 (2004)"));
        assert!(r.citation.contains("N.Y. Real Prop. Law § 238-a"));
        assert!(r.citation.contains("HSTPA of 2019"));
        assert!(r.citation.contains("Fla. Stat. § 83.808"));
        assert!(r.citation.contains("Tex. Prop. Code § 92.019"));
        assert!(r
            .citation
            .contains("Restatement (Second) of Contracts § 356"));
    }

    #[test]
    fn note_pins_california_two_prong_test() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1671(d)")
            && n.contains("Orozco v. Casimiro")
            && n.contains("LIQUIDATED DAMAGES")
            && n.contains("two-prong validity test")));
    }

    #[test]
    fn note_pins_california_5_to_6_percent_threshold() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1671(d)")
            && n.contains("5-6% of monthly rent")
            && n.contains("void in toto")));
    }

    #[test]
    fn note_pins_new_york_50_dollar_or_5_percent_hard_cap() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 238-a")
            && n.contains("HSTPA of 2019")
            && n.contains("June 14, 2019")
            && n.contains("LESSER of $50 OR 5%")));
    }

    #[test]
    fn note_pins_new_york_5_day_grace_no_eviction() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 238-a")
            && n.contains("5-DAY GRACE PERIOD")
            && n.contains("CANNOT EVICT solely for unpaid late fees")));
    }

    #[test]
    fn note_pins_florida_manufactured_home_park_20_dollar_or_20_percent() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 83.808")
            && n.contains("manufactured home park")
            && n.contains("$20 OR 20%")
            && n.contains("GREATER")));
    }

    #[test]
    fn note_pins_florida_general_no_statutory_cap() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Chapter 83 Part II")
            && n.contains("NO statutory cap")
            && n.contains("court reasonableness")));
    }

    #[test]
    fn note_pins_texas_12_10_safe_harbor() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 92.019")
            && n.contains("12% for structures of 4 or fewer units")
            && n.contains("10% for structures of more than 4 units")));
    }

    #[test]
    fn note_pins_texas_2_day_grace() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 92.019(a)(2)") && n.contains("2-DAY GRACE PERIOD")));
    }

    #[test]
    fn note_pins_texas_treble_plus_100_remedy() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 92.019(d)")
            && n.contains("$100 PLUS TREBLE")
            && n.contains("reasonable attorney's fees")));
    }

    #[test]
    fn note_pins_default_restatement_356() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Restatement (Second) of Contracts § 356")
                && n.contains("NOT operate as a penalty")));
    }

    #[test]
    fn note_pins_cross_jurisdictional_invariant() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Cross-jurisdictional invariant")
                && n.contains("LIQUIDATED-DAMAGES REASONABLENESS")
                && n.contains("STATUTORY HARD CAP")
                && n.contains("PERCENTAGE SAFE HARBOR")));
    }

    #[test]
    fn ny_uniquely_hard_caps_at_50_dollars_invariant() {
        let make = |jur, rent_cents| TenantLateFeeCapInput {
            jurisdiction: jur,
            monthly_rent_cents: rent_cents,
            late_fee_charged_cents: 0,
            days_after_due_when_charged: 10,
            lease_specifies_late_fee: true,
            dwelling_unit_count: 4,
            reasonable_endeavor_to_estimate: true,
            impracticable_to_fix_actual_damage: true,
            manufactured_home_park: false,
        };
        let ny = check(&make(Jurisdiction::NewYork, 10_000_000));
        let ca = check(&make(Jurisdiction::California, 10_000_000));
        let tx = check(&make(Jurisdiction::Texas, 10_000_000));
        assert_eq!(ny.statutory_cap_cents, 5_000);
        assert!(ca.statutory_cap_cents > ny.statutory_cap_cents);
        assert!(tx.statutory_cap_cents > ny.statutory_cap_cents);
    }

    #[test]
    fn tx_uniquely_engages_treble_damages_invariant() {
        let make = |jur| TenantLateFeeCapInput {
            jurisdiction: jur,
            monthly_rent_cents: 200_000,
            late_fee_charged_cents: 30_000,
            days_after_due_when_charged: 10,
            lease_specifies_late_fee: true,
            dwelling_unit_count: 4,
            reasonable_endeavor_to_estimate: true,
            impracticable_to_fix_actual_damage: true,
            manufactured_home_park: false,
        };
        let tx = check(&make(Jurisdiction::Texas));
        let ny = check(&make(Jurisdiction::NewYork));
        let ca = check(&make(Jurisdiction::California));
        let fl = check(&make(Jurisdiction::Florida));
        let de = check(&make(Jurisdiction::Default));
        assert!(tx.treble_damages_engaged);
        assert!(!ny.treble_damages_engaged);
        assert!(!ca.treble_damages_engaged);
        assert!(!fl.treble_damages_engaged);
        assert!(!de.treble_damages_engaged);
    }

    #[test]
    fn jurisdiction_truth_table_five_cells() {
        for jur in [
            Jurisdiction::California,
            Jurisdiction::NewYork,
            Jurisdiction::Florida,
            Jurisdiction::Texas,
            Jurisdiction::Default,
        ] {
            let mut i = ca_compliant();
            i.jurisdiction = jur;
            let r = check(&i);
            assert_eq!(r.jurisdiction, jur);
        }
    }

    #[test]
    fn defensive_overflow_clamped_with_saturating_mul() {
        let mut i = ca_compliant();
        i.monthly_rent_cents = u64::MAX;
        let r = check(&i);
        let _ = r.statutory_cap_cents;
        assert!(r.statutory_cap_cents > 0);
    }
}
