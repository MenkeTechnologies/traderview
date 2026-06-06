//! IRC § 6707A — Penalty for failure to include reportable
//! transaction information with return. Trader-relevant
//! because traders often get caught up in reportable / listed
//! transactions through partnerships (syndicated conservation
//! easements + micro-captive insurance + monetized installment
//! sales + § 831(b) captive arrangements), and § 6707A
//! penalties STACK on top of § 6662A accuracy-related penalty
//! (paired module). Directly tied to § 7421 Anti-Injunction
//! Act jurisprudence — CIC Services, LLC v. IRS, 593 U.S. 209
//! (2021) addressed pre-enforcement challenge to micro-captive
//! reporting requirements that trigger § 6707A.
//!
//! **§ 6707A(b)(1) base computation** — penalty = 75% of the
//! decrease in tax shown on the return as a result of the
//! transaction (or which would have resulted from the
//! transaction if such transaction were respected for federal
//! tax purposes).
//!
//! **§ 6707A(b)(2) maximum penalty**:
//! - Listed transaction (defined § 6707A(c)(2)): $200,000
//!   entity / $100,000 natural person
//! - Other reportable transaction: $50,000 entity / $10,000
//!   natural person
//!
//! **§ 6707A(b)(3) minimum penalty**: $10,000 entity / $5,000
//! natural person.
//!
//! **§ 6707A(c)(1) reportable transaction** — any transaction
//! with respect to which information is required to be
//! included with a return or statement because, as determined
//! under regulations prescribed under § 6011, such transaction
//! is of a type which the Secretary determines as having a
//! potential for tax avoidance or evasion.
//!
//! **§ 6707A(c)(2) listed transaction** — a reportable
//! transaction which is the same as, or substantially similar
//! to, a transaction specifically identified by the Secretary
//! as a tax avoidance transaction for purposes of § 6011.
//!
//! Citations: 26 USC § 6707A(b)(1), (b)(2)(A)+(B), (b)(3),
//! (c)(1), (c)(2); Small Business Jobs Act of 2010 (75%
//! formula); 26 CFR § 301.6707A-1; CIC Services, LLC v. IRS,
//! 593 U.S. 209 (2021).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxpayerType {
    /// Natural person (lower minimum and lower maximum caps).
    NaturalPerson,
    /// Entity (corporation, partnership, trust) — higher
    /// minimum and higher maximum caps.
    Entity,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// Listed transaction — § 6707A(c)(2); subject to higher
    /// maximum cap.
    Listed,
    /// Other reportable transaction — § 6707A(c)(1); subject
    /// to lower maximum cap.
    OtherReportable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6707AInput {
    pub taxpayer_type: TaxpayerType,
    pub transaction_type: TransactionType,
    /// Decrease in tax shown on the return as a result of the
    /// transaction (the 75% base).
    pub decrease_in_tax_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6707AResult {
    pub penalty_cents: i64,
    pub base_75_percent_cents: i64,
    pub floor_engaged: bool,
    pub cap_engaged: bool,
    pub floor_cents: i64,
    pub cap_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6707AInput) -> Section6707AResult {
    let decrease = input.decrease_in_tax_cents.max(0);

    let base_75 = decrease.saturating_mul(3) / 4;

    let floor: i64 = match input.taxpayer_type {
        TaxpayerType::NaturalPerson => 500_000,
        TaxpayerType::Entity => 1_000_000,
    };

    let cap: i64 = match (input.taxpayer_type, input.transaction_type) {
        (TaxpayerType::NaturalPerson, TransactionType::Listed) => 10_000_000,
        (TaxpayerType::Entity, TransactionType::Listed) => 20_000_000,
        (TaxpayerType::NaturalPerson, TransactionType::OtherReportable) => 1_000_000,
        (TaxpayerType::Entity, TransactionType::OtherReportable) => 5_000_000,
    };

    let after_floor = base_75.max(floor);
    let penalty = after_floor.min(cap);

    let floor_engaged = base_75 < floor;
    let cap_engaged = after_floor > cap;

    let notes: Vec<String> = vec![
        "26 USC § 6707A(b)(1) — penalty base = 75% of decrease in tax shown on return as result of transaction"
            .to_string(),
        "26 USC § 6707A(b)(3) — minimum penalty: $10,000 entity / $5,000 natural person; § 6707A(b)(2) — maximum penalty: listed = $200,000 entity / $100,000 natural person; other reportable = $50,000 entity / $10,000 natural person"
            .to_string(),
        "26 USC § 6707A(c)(2) — listed transaction = reportable transaction substantially similar to transaction specifically identified by Secretary as tax avoidance transaction under § 6011"
            .to_string(),
        "CIC Services, LLC v. IRS, 593 U.S. 209 (2021) — pre-enforcement challenge to micro-captive reporting requirements that trigger § 6707A is NOT barred by § 7421(a) Anti-Injunction Act"
            .to_string(),
    ];

    Section6707AResult {
        penalty_cents: penalty,
        base_75_percent_cents: base_75,
        floor_engaged,
        cap_engaged,
        floor_cents: floor,
        cap_cents: cap,
        citation: "26 USC § 6707A(b)(1), (b)(2), (b)(3), (c)(1), (c)(2); 26 CFR § 301.6707A-1",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn np_listed_base() -> Section6707AInput {
        Section6707AInput {
            taxpayer_type: TaxpayerType::NaturalPerson,
            transaction_type: TransactionType::Listed,
            decrease_in_tax_cents: 5_000_000,
        }
    }

    fn entity_listed_base() -> Section6707AInput {
        let mut i = np_listed_base();
        i.taxpayer_type = TaxpayerType::Entity;
        i
    }

    fn np_other_base() -> Section6707AInput {
        let mut i = np_listed_base();
        i.transaction_type = TransactionType::OtherReportable;
        i
    }

    fn entity_other_base() -> Section6707AInput {
        let mut i = np_listed_base();
        i.taxpayer_type = TaxpayerType::Entity;
        i.transaction_type = TransactionType::OtherReportable;
        i
    }

    #[test]
    fn np_listed_75_percent_mid_range() {
        let mut i = np_listed_base();
        i.decrease_in_tax_cents = 4_000_000;
        let r = check(&i);
        assert_eq!(r.base_75_percent_cents, 3_000_000);
        assert_eq!(r.penalty_cents, 3_000_000);
        assert!(!r.floor_engaged);
        assert!(!r.cap_engaged);
    }

    #[test]
    fn np_listed_floor_engaged_low_decrease() {
        let mut i = np_listed_base();
        i.decrease_in_tax_cents = 100_000;
        let r = check(&i);
        assert_eq!(r.base_75_percent_cents, 75_000);
        assert!(r.floor_engaged);
        assert_eq!(r.penalty_cents, 500_000);
    }

    #[test]
    fn np_listed_cap_engaged_high_decrease() {
        let mut i = np_listed_base();
        i.decrease_in_tax_cents = 100_000_000;
        let r = check(&i);
        assert_eq!(r.base_75_percent_cents, 75_000_000);
        assert!(r.cap_engaged);
        assert_eq!(r.penalty_cents, 10_000_000);
    }

    #[test]
    fn entity_listed_floor_10k() {
        let mut i = entity_listed_base();
        i.decrease_in_tax_cents = 0;
        let r = check(&i);
        assert!(r.floor_engaged);
        assert_eq!(r.penalty_cents, 1_000_000);
    }

    #[test]
    fn entity_listed_cap_200k() {
        let mut i = entity_listed_base();
        i.decrease_in_tax_cents = 100_000_000;
        let r = check(&i);
        assert!(r.cap_engaged);
        assert_eq!(r.penalty_cents, 20_000_000);
    }

    #[test]
    fn np_other_cap_10k() {
        let mut i = np_other_base();
        i.decrease_in_tax_cents = 10_000_000;
        let r = check(&i);
        assert!(r.cap_engaged);
        assert_eq!(r.penalty_cents, 1_000_000);
    }

    #[test]
    fn entity_other_cap_50k() {
        let mut i = entity_other_base();
        i.decrease_in_tax_cents = 50_000_000;
        let r = check(&i);
        assert!(r.cap_engaged);
        assert_eq!(r.penalty_cents, 5_000_000);
    }

    #[test]
    fn np_listed_at_floor_boundary_compliant() {
        let mut i = np_listed_base();
        i.decrease_in_tax_cents = 666_667;
        let r = check(&i);
        assert_eq!(r.penalty_cents, 500_000);
        assert!(!r.cap_engaged);
    }

    #[test]
    fn np_listed_at_cap_boundary() {
        let mut i = np_listed_base();
        i.decrease_in_tax_cents = 13_333_333;
        let r = check(&i);
        assert_eq!(r.penalty_cents, 9_999_999);
        assert!(!r.cap_engaged);
    }

    #[test]
    fn entity_listed_floor_at_minimum_decrease() {
        let mut i = entity_listed_base();
        i.decrease_in_tax_cents = 1_333_333;
        let r = check(&i);
        assert_eq!(r.base_75_percent_cents, 999_999);
        assert!(r.floor_engaged);
        assert_eq!(r.penalty_cents, 1_000_000);
    }

    #[test]
    fn entity_listed_above_floor() {
        let mut i = entity_listed_base();
        i.decrease_in_tax_cents = 1_500_000;
        let r = check(&i);
        assert_eq!(r.base_75_percent_cents, 1_125_000);
        assert!(!r.floor_engaged);
        assert_eq!(r.penalty_cents, 1_125_000);
    }

    #[test]
    fn citation_pins_subsections() {
        let r = check(&np_listed_base());
        assert!(r.citation.contains("§ 6707A(b)(1)"));
        assert!(r.citation.contains("(b)(2)"));
        assert!(r.citation.contains("(b)(3)"));
        assert!(r.citation.contains("(c)(1)"));
        assert!(r.citation.contains("(c)(2)"));
        assert!(r.citation.contains("§ 301.6707A-1"));
    }

    #[test]
    fn note_pins_75_percent_base() {
        let r = check(&np_listed_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6707A(b)(1)") && n.contains("75%")));
    }

    #[test]
    fn note_pins_all_4_floor_cap_amounts() {
        let r = check(&np_listed_base());
        assert!(r.notes.iter().any(|n| n.contains("$10,000 entity")
            && n.contains("$5,000 natural person")
            && n.contains("$200,000 entity")
            && n.contains("$100,000 natural person")
            && n.contains("$50,000 entity")
            && n.contains("$10,000 natural person")));
    }

    #[test]
    fn note_pins_listed_transaction_definition() {
        let r = check(&np_listed_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6707A(c)(2)") && n.contains("substantially similar")));
    }

    #[test]
    fn note_pins_cic_services_pre_enforcement() {
        let r = check(&np_listed_base());
        assert!(r.notes.iter().any(|n| n.contains("CIC Services")
            && n.contains("593 U.S. 209 (2021)")
            && n.contains("§ 7421(a)")));
    }

    #[test]
    fn defensive_negative_decrease_clamped() {
        let mut i = np_listed_base();
        i.decrease_in_tax_cents = -1_000_000;
        let r = check(&i);
        assert_eq!(r.base_75_percent_cents, 0);
        assert!(r.floor_engaged);
        assert_eq!(r.penalty_cents, 500_000);
    }

    #[test]
    fn defensive_overflow_base_saturating() {
        let mut i = entity_listed_base();
        i.decrease_in_tax_cents = i64::MAX;
        let r = check(&i);
        assert!(r.cap_engaged);
        assert_eq!(r.penalty_cents, 20_000_000);
    }

    #[test]
    fn floor_amounts_truth_table() {
        for (taxpayer, expected_floor) in [
            (TaxpayerType::NaturalPerson, 500_000_i64),
            (TaxpayerType::Entity, 1_000_000_i64),
        ] {
            let mut i = np_listed_base();
            i.taxpayer_type = taxpayer;
            i.decrease_in_tax_cents = 0;
            let r = check(&i);
            assert_eq!(r.floor_cents, expected_floor);
            assert_eq!(r.penalty_cents, expected_floor);
        }
    }

    #[test]
    fn cap_amounts_truth_table_2x2() {
        for (taxpayer, transaction, expected_cap) in [
            (
                TaxpayerType::NaturalPerson,
                TransactionType::Listed,
                10_000_000_i64,
            ),
            (
                TaxpayerType::Entity,
                TransactionType::Listed,
                20_000_000_i64,
            ),
            (
                TaxpayerType::NaturalPerson,
                TransactionType::OtherReportable,
                1_000_000_i64,
            ),
            (
                TaxpayerType::Entity,
                TransactionType::OtherReportable,
                5_000_000_i64,
            ),
        ] {
            let mut i = np_listed_base();
            i.taxpayer_type = taxpayer;
            i.transaction_type = transaction;
            i.decrease_in_tax_cents = 1_000_000_000;
            let r = check(&i);
            assert_eq!(r.cap_cents, expected_cap);
            assert_eq!(r.penalty_cents, expected_cap);
            assert!(r.cap_engaged);
        }
    }

    #[test]
    fn listed_cap_higher_than_other_cap_invariant() {
        let mut i_listed = entity_listed_base();
        i_listed.decrease_in_tax_cents = 1_000_000_000;
        let r_listed = check(&i_listed);

        let mut i_other = entity_other_base();
        i_other.decrease_in_tax_cents = 1_000_000_000;
        let r_other = check(&i_other);

        assert!(r_listed.cap_cents > r_other.cap_cents);
        assert_eq!(r_listed.cap_cents, 20_000_000);
        assert_eq!(r_other.cap_cents, 5_000_000);
    }

    #[test]
    fn entity_floor_higher_than_natural_person_invariant() {
        let mut i_entity = entity_listed_base();
        i_entity.decrease_in_tax_cents = 0;
        let r_entity = check(&i_entity);

        let mut i_np = np_listed_base();
        i_np.decrease_in_tax_cents = 0;
        let r_np = check(&i_np);

        assert!(r_entity.floor_cents > r_np.floor_cents);
        assert_eq!(r_entity.floor_cents, 1_000_000);
        assert_eq!(r_np.floor_cents, 500_000);
    }

    #[test]
    fn floor_or_cap_engagement_3x_regime_sweep() {
        let r_below_floor = check(&np_listed_base());
        let _ = r_below_floor;

        for (decrease, expect_floor, expect_cap) in [
            (100_000_i64, true, false),
            (4_000_000_i64, false, false),
            (100_000_000_i64, false, true),
        ] {
            let mut i = np_listed_base();
            i.decrease_in_tax_cents = decrease;
            let r = check(&i);
            assert_eq!(r.floor_engaged, expect_floor);
            assert_eq!(r.cap_engaged, expect_cap);
        }
    }

    #[test]
    fn listed_natural_person_cap_100k() {
        let mut i = np_listed_base();
        i.decrease_in_tax_cents = 1_000_000_000;
        let r = check(&i);
        assert_eq!(r.penalty_cents, 10_000_000);
    }

    #[test]
    fn other_reportable_natural_person_cap_10k() {
        let mut i = np_other_base();
        i.decrease_in_tax_cents = 1_000_000_000;
        let r = check(&i);
        assert_eq!(r.penalty_cents, 1_000_000);
    }

    #[test]
    fn entity_other_reportable_cap_50k_uniquely_different_from_listed() {
        let mut i_listed = entity_listed_base();
        i_listed.decrease_in_tax_cents = 1_000_000_000;
        let r_listed = check(&i_listed);

        let mut i_other = entity_other_base();
        i_other.decrease_in_tax_cents = 1_000_000_000;
        let r_other = check(&i_other);

        assert_eq!(r_listed.penalty_cents, 20_000_000);
        assert_eq!(r_other.penalty_cents, 5_000_000);
        assert_eq!(r_listed.penalty_cents, 4 * r_other.penalty_cents);
    }

    #[test]
    fn zero_decrease_engages_floor_for_all_4_combinations() {
        for taxpayer in [TaxpayerType::NaturalPerson, TaxpayerType::Entity] {
            for transaction in [TransactionType::Listed, TransactionType::OtherReportable] {
                let i = Section6707AInput {
                    taxpayer_type: taxpayer,
                    transaction_type: transaction,
                    decrease_in_tax_cents: 0,
                };
                let r = check(&i);
                assert!(r.floor_engaged);
                assert_eq!(r.base_75_percent_cents, 0);
                let expected_floor = match taxpayer {
                    TaxpayerType::NaturalPerson => 500_000,
                    TaxpayerType::Entity => 1_000_000,
                };
                assert_eq!(r.penalty_cents, expected_floor);
            }
        }
    }

    #[test]
    fn base_75_percent_precision_at_decrease_133() {
        let mut i = np_listed_base();
        i.decrease_in_tax_cents = 133;
        let r = check(&i);
        assert_eq!(r.base_75_percent_cents, 99);
    }

    #[test]
    fn base_75_percent_precision_at_decrease_400() {
        let mut i = np_listed_base();
        i.decrease_in_tax_cents = 400;
        let r = check(&i);
        assert_eq!(r.base_75_percent_cents, 300);
    }

    #[test]
    fn np_listed_pinned_at_5000_floor_when_decrease_5000() {
        let mut i = np_listed_base();
        i.decrease_in_tax_cents = 500_000;
        let r = check(&i);
        assert_eq!(r.base_75_percent_cents, 375_000);
        assert!(r.floor_engaged);
        assert_eq!(r.penalty_cents, 500_000);
    }
}
