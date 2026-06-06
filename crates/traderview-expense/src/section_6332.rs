//! IRC § 6332 — Surrender of property subject to levy.
//! Third-party (bank + employer + broker + financial
//! institution) holding taxpayer property who receives IRS
//! levy notice must surrender or face personal liability +
//! 50% penalty. Trader-relevant on both sides:
//! - Trader-traders facing IRS levy on brokerage accounts:
//!   broker must surrender after 21-day bank hold (if
//!   treated as bank under regs).
//! - Trader-landlords as third-party levy recipients
//!   (employers, vendors, etc.) face § 6332 surrender
//!   obligations.
//!
//! Procedural-companion to § 6331 (levy authority), § 6321
//! (lien attachment), § 6303 (notice and demand), § 6323
//! (lien priority), § 6325 (release/discharge), § 6334
//! (exempt property), § 6330 (CDP for levies), § 7426
//! (third-party wrongful levy — INVERSE pathway when third
//! party believes levy is wrongful), § 7421 (Anti-Injunction
//! Act), and § 7433 (civil damages for unauthorized
//! collection).
//!
//! **§ 6332(a) general surrender obligation** — any person
//! in possession of (or obligated with respect to) property
//! or rights to property subject to levy upon which a levy
//! has been made MUST surrender to Secretary upon demand.
//!
//! **§ 6332(c) 21-day bank hold** — any bank shall surrender
//! deposits (including interest) ONLY AFTER 21 days after
//! service of levy. Allows error-correction window + taxpayer
//! to challenge before bank actually releases funds.
//!
//! **§ 6332(b) wage/salary continuous levy** — special rule
//! for salary/wages cross-references § 6331(e) continuous
//! wage levy attachment.
//!
//! **§ 6332(d)(1) personal liability for failure to
//! surrender** — third party liable in own person and estate
//! to United States in sum equal to value of property NOT
//! surrendered, but NOT exceeding the amount of taxes for
//! collection of which levy was made + costs + interest at
//! § 6621 underpayment rate from date of levy.
//!
//! **§ 6332(d)(2) 50% penalty for failure to surrender
//! without reasonable cause** — additional liability =
//! 50% of amount recoverable under § 6332(d)(1). NO credit
//! against underlying tax liability for which levy made.
//!
//! **§ 6332(e) discharge from competing liability** — third
//! party who surrenders property upon Secretary's demand
//! shall be DISCHARGED from any obligation or liability to
//! delinquent taxpayer and any other person with respect to
//! such property arising from surrender or payment. Provides
//! safe harbor for compliant levy recipients.
//!
//! Citations: 26 USC § 6332(a)-(e); 26 CFR § 301.6332-1;
//! IRM 5.17.3 (Levy and Sale); § 6331 (levy authority); §
//! 6621 (underpayment interest rate); § 7426 (third-party
//! wrongful levy — inverse pathway).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    /// § 6332(c) bank deposit — 21-day hold rule applies.
    BankDeposit,
    /// § 6332(b) wage or salary — § 6331(e) continuous levy.
    WageOrSalary,
    /// General tangible or intangible property.
    GeneralProperty,
    /// Life insurance and endowment contracts (§ 6332(b)
    /// special provisions).
    LifeInsurance,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6332Input {
    pub property_type: PropertyType,
    /// Days since IRS served levy on third party.
    pub days_since_levy_served: u32,
    /// Whether third party (bank, employer, broker) has
    /// surrendered property.
    pub property_surrendered: bool,
    /// Whether failure to surrender was WITHOUT reasonable
    /// cause (engages § 6332(d)(2) 50% penalty).
    pub failure_without_reasonable_cause: bool,
    /// Value of property held by third party, in cents.
    pub property_value_cents: i64,
    /// Amount of taxes for collection of which levy was
    /// made, in cents (cap on § 6332(d)(1) personal
    /// liability).
    pub tax_liability_amount_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6332Result {
    pub surrender_required_now: bool,
    pub twenty_one_day_hold_engaged: bool,
    pub surrender_compliant: bool,
    pub third_party_discharged: bool,
    pub personal_liability_cents: i64,
    pub fifty_percent_penalty_cents: i64,
    pub total_third_party_exposure_cents: i64,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6332Input) -> Section6332Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let twenty_one_day_hold = matches!(input.property_type, PropertyType::BankDeposit)
        && input.days_since_levy_served < 21;

    let surrender_required_now = !twenty_one_day_hold;

    let property_value = input.property_value_cents.max(0);
    let tax_liability = input.tax_liability_amount_cents.max(0);

    let personal_liability_cap = property_value.min(tax_liability);

    let personal_liability = if surrender_required_now && !input.property_surrendered {
        personal_liability_cap
    } else {
        0
    };

    let fifty_percent_penalty = if personal_liability > 0 && input.failure_without_reasonable_cause
    {
        personal_liability.saturating_mul(50) / 100
    } else {
        0
    };

    if surrender_required_now && !input.property_surrendered {
        failure_reasons.push(
            "26 USC § 6332(a) — third party in possession of property subject to levy must surrender upon demand by Secretary; failure exposes third party to § 6332(d)(1) personal liability"
                .to_string(),
        );

        if input.failure_without_reasonable_cause {
            failure_reasons.push(
                "26 USC § 6332(d)(2) — failure to surrender WITHOUT REASONABLE CAUSE triggers additional 50% penalty on amount recoverable under § 6332(d)(1); NO credit against underlying tax liability"
                    .to_string(),
            );
        }
    }

    let surrender_compliant = !surrender_required_now || input.property_surrendered;
    let third_party_discharged = surrender_compliant && input.property_surrendered;

    let total_exposure = personal_liability.saturating_add(fifty_percent_penalty);

    let notes: Vec<String> = vec![
        "26 USC § 6332(a) — any person in possession of (or obligated with respect to) property/rights to property subject to levy must surrender to Secretary upon demand"
            .to_string(),
        "26 USC § 6332(c) — 21-day bank hold: banks surrender deposits ONLY AFTER 21 days after service of levy (error-correction window for taxpayer to challenge)"
            .to_string(),
        "26 USC § 6332(b) — wage/salary continuous levy cross-reference § 6331(e); attaches to wages earned + advances + future wages until levy released"
            .to_string(),
        "26 USC § 6332(d)(1) personal liability — failure to surrender exposes third party to liability equal to value of property NOT surrendered, capped at tax + costs + § 6621 underpayment interest from date of levy"
            .to_string(),
        "26 USC § 6332(d)(2) 50% additional penalty — failure WITHOUT REASONABLE CAUSE triggers 50% of § 6332(d)(1) recoverable amount; NO credit against underlying tax"
            .to_string(),
        "26 USC § 6332(e) discharge safe harbor — compliant surrender DISCHARGES third party from any obligation or liability to delinquent taxpayer arising from surrender or payment"
            .to_string(),
    ];

    Section6332Result {
        surrender_required_now,
        twenty_one_day_hold_engaged: twenty_one_day_hold,
        surrender_compliant,
        third_party_discharged,
        personal_liability_cents: personal_liability,
        fifty_percent_penalty_cents: fifty_percent_penalty,
        total_third_party_exposure_cents: total_exposure,
        failure_reasons,
        citation: "26 USC § 6332(a)-(e); 26 CFR § 301.6332-1; IRM 5.17.3; § 6331; § 6621; § 7426",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compliant_base() -> Section6332Input {
        Section6332Input {
            property_type: PropertyType::GeneralProperty,
            days_since_levy_served: 30,
            property_surrendered: true,
            failure_without_reasonable_cause: false,
            property_value_cents: 10_000_000_000,
            tax_liability_amount_cents: 5_000_000_000,
        }
    }

    fn bank_compliant_base() -> Section6332Input {
        let mut i = compliant_base();
        i.property_type = PropertyType::BankDeposit;
        i.days_since_levy_served = 25;
        i
    }

    #[test]
    fn general_property_surrendered_compliant() {
        let r = check(&compliant_base());
        assert!(r.surrender_required_now);
        assert!(r.surrender_compliant);
        assert!(r.third_party_discharged);
        assert_eq!(r.personal_liability_cents, 0);
        assert_eq!(r.fifty_percent_penalty_cents, 0);
        assert!(r.failure_reasons.is_empty());
    }

    #[test]
    fn general_property_not_surrendered_triggers_personal_liability() {
        let mut i = compliant_base();
        i.property_surrendered = false;
        let r = check(&i);
        assert!(!r.surrender_compliant);
        assert!(!r.third_party_discharged);
        assert_eq!(r.personal_liability_cents, 5_000_000_000);
        assert_eq!(r.fifty_percent_penalty_cents, 0);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6332(a)") && f.contains("must surrender")));
    }

    #[test]
    fn failure_without_reasonable_cause_triggers_50_percent_penalty() {
        let mut i = compliant_base();
        i.property_surrendered = false;
        i.failure_without_reasonable_cause = true;
        let r = check(&i);
        assert_eq!(r.personal_liability_cents, 5_000_000_000);
        assert_eq!(r.fifty_percent_penalty_cents, 2_500_000_000);
        assert_eq!(r.total_third_party_exposure_cents, 7_500_000_000);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6332(d)(2)")
            && f.contains("REASONABLE CAUSE")
            && f.contains("50%")));
    }

    #[test]
    fn personal_liability_capped_at_tax_liability() {
        let mut i = compliant_base();
        i.property_surrendered = false;
        i.property_value_cents = 100_000_000_000;
        i.tax_liability_amount_cents = 5_000_000_000;
        let r = check(&i);
        assert_eq!(r.personal_liability_cents, 5_000_000_000);
    }

    #[test]
    fn personal_liability_capped_at_property_value() {
        let mut i = compliant_base();
        i.property_surrendered = false;
        i.property_value_cents = 2_000_000_000;
        i.tax_liability_amount_cents = 100_000_000_000;
        let r = check(&i);
        assert_eq!(r.personal_liability_cents, 2_000_000_000);
    }

    #[test]
    fn bank_deposit_within_21_day_hold_not_required_to_surrender() {
        let r = check(&bank_compliant_base());
        let mut i = bank_compliant_base();
        i.days_since_levy_served = 10;
        i.property_surrendered = false;
        let r2 = check(&i);
        assert!(!r2.surrender_required_now);
        assert!(r2.twenty_one_day_hold_engaged);
        assert_eq!(r2.personal_liability_cents, 0);
        assert!(r.surrender_compliant);
    }

    #[test]
    fn bank_deposit_at_21_day_boundary_required_to_surrender() {
        let mut i = bank_compliant_base();
        i.days_since_levy_served = 21;
        let r = check(&i);
        assert!(r.surrender_required_now);
        assert!(!r.twenty_one_day_hold_engaged);
    }

    #[test]
    fn bank_deposit_at_20_days_within_hold() {
        let mut i = bank_compliant_base();
        i.days_since_levy_served = 20;
        i.property_surrendered = false;
        let r = check(&i);
        assert!(r.twenty_one_day_hold_engaged);
        assert!(!r.surrender_required_now);
        assert_eq!(r.personal_liability_cents, 0);
    }

    #[test]
    fn bank_deposit_after_21_day_hold_and_not_surrendered_triggers_liability() {
        let mut i = bank_compliant_base();
        i.days_since_levy_served = 22;
        i.property_surrendered = false;
        let r = check(&i);
        assert!(r.surrender_required_now);
        assert!(!r.twenty_one_day_hold_engaged);
        assert_eq!(r.personal_liability_cents, 5_000_000_000);
    }

    #[test]
    fn wage_salary_no_21_day_hold() {
        let mut i = compliant_base();
        i.property_type = PropertyType::WageOrSalary;
        i.days_since_levy_served = 5;
        let r = check(&i);
        assert!(!r.twenty_one_day_hold_engaged);
        assert!(r.surrender_required_now);
    }

    #[test]
    fn life_insurance_no_21_day_hold() {
        let mut i = compliant_base();
        i.property_type = PropertyType::LifeInsurance;
        i.days_since_levy_served = 5;
        let r = check(&i);
        assert!(!r.twenty_one_day_hold_engaged);
        assert!(r.surrender_required_now);
    }

    #[test]
    fn discharge_only_when_surrendered_invariant() {
        let mut i_surrendered = compliant_base();
        i_surrendered.property_surrendered = true;
        let r_surrendered = check(&i_surrendered);
        assert!(r_surrendered.third_party_discharged);

        let mut i_not = compliant_base();
        i_not.property_surrendered = false;
        let r_not = check(&i_not);
        assert!(!r_not.third_party_discharged);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&compliant_base());
        assert!(r.citation.contains("§ 6332(a)-(e)"));
        assert!(r.citation.contains("§ 301.6332-1"));
        assert!(r.citation.contains("IRM 5.17.3"));
        assert!(r.citation.contains("§ 6331"));
        assert!(r.citation.contains("§ 6621"));
        assert!(r.citation.contains("§ 7426"));
    }

    #[test]
    fn note_pins_21_day_hold() {
        let r = check(&compliant_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6332(c)")
            && n.contains("21-day bank hold")
            && n.contains("error-correction")));
    }

    #[test]
    fn note_pins_50_percent_penalty() {
        let r = check(&compliant_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6332(d)(2)")
            && n.contains("50%")
            && n.contains("WITHOUT REASONABLE CAUSE")));
    }

    #[test]
    fn note_pins_discharge_safe_harbor() {
        let r = check(&compliant_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6332(e)")
            && n.contains("DISCHARGES")
            && n.contains("safe harbor")));
    }

    #[test]
    fn property_type_truth_table_21_day_hold() {
        for (property, day_5_holds) in [
            (PropertyType::BankDeposit, true),
            (PropertyType::WageOrSalary, false),
            (PropertyType::GeneralProperty, false),
            (PropertyType::LifeInsurance, false),
        ] {
            let mut i = compliant_base();
            i.property_type = property;
            i.days_since_levy_served = 5;
            let r = check(&i);
            assert_eq!(r.twenty_one_day_hold_engaged, day_5_holds);
        }
    }

    #[test]
    fn fifty_percent_penalty_truth_table() {
        for (surrendered, reasonable_cause, exp_penalty_engaged) in [
            (true, true, false),
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            let mut i = compliant_base();
            i.property_surrendered = surrendered;
            i.failure_without_reasonable_cause = !reasonable_cause;
            let r = check(&i);
            assert_eq!(r.fifty_percent_penalty_cents > 0, exp_penalty_engaged);
        }
    }

    #[test]
    fn defensive_negative_property_value_clamped() {
        let mut i = compliant_base();
        i.property_value_cents = -1_000_000_000;
        i.property_surrendered = false;
        let r = check(&i);
        assert_eq!(r.personal_liability_cents, 0);
    }

    #[test]
    fn defensive_negative_tax_liability_clamped() {
        let mut i = compliant_base();
        i.tax_liability_amount_cents = -1_000_000_000;
        i.property_surrendered = false;
        let r = check(&i);
        assert_eq!(r.personal_liability_cents, 0);
    }

    #[test]
    fn defensive_overflow_50_percent_saturating() {
        let mut i = compliant_base();
        i.property_surrendered = false;
        i.failure_without_reasonable_cause = true;
        i.property_value_cents = i64::MAX;
        i.tax_liability_amount_cents = i64::MAX - 1_000;
        let r = check(&i);
        assert!(r.personal_liability_cents > 0);
    }

    #[test]
    fn bank_uniquely_engages_21_day_hold_invariant() {
        for property in [
            PropertyType::WageOrSalary,
            PropertyType::GeneralProperty,
            PropertyType::LifeInsurance,
        ] {
            let mut i = compliant_base();
            i.property_type = property;
            i.days_since_levy_served = 5;
            let r = check(&i);
            assert!(!r.twenty_one_day_hold_engaged);
        }

        let mut i_bank = compliant_base();
        i_bank.property_type = PropertyType::BankDeposit;
        i_bank.days_since_levy_served = 5;
        let r_bank = check(&i_bank);
        assert!(r_bank.twenty_one_day_hold_engaged);
    }

    #[test]
    fn twenty_one_day_boundary_precision_invariant() {
        let mut i_20 = bank_compliant_base();
        i_20.days_since_levy_served = 20;
        let r_20 = check(&i_20);
        assert!(r_20.twenty_one_day_hold_engaged);

        let mut i_21 = bank_compliant_base();
        i_21.days_since_levy_served = 21;
        let r_21 = check(&i_21);
        assert!(!r_21.twenty_one_day_hold_engaged);
    }

    #[test]
    fn total_exposure_combines_liability_and_penalty() {
        let mut i = compliant_base();
        i.property_surrendered = false;
        i.failure_without_reasonable_cause = true;
        let r = check(&i);
        assert_eq!(
            r.total_third_party_exposure_cents,
            r.personal_liability_cents + r.fifty_percent_penalty_cents
        );
    }

    #[test]
    fn discharge_engages_only_for_general_compliance_invariant() {
        let mut i_general = compliant_base();
        let r_general = check(&i_general);
        assert!(r_general.third_party_discharged);

        i_general.property_surrendered = false;
        let r_general_fail = check(&i_general);
        assert!(!r_general_fail.third_party_discharged);
    }

    #[test]
    fn within_21_day_hold_no_failure_reasons_for_bank() {
        let mut i = bank_compliant_base();
        i.days_since_levy_served = 5;
        i.property_surrendered = false;
        let r = check(&i);
        assert!(r.failure_reasons.is_empty());
    }
}
