//! Form 1099-NEC contractor-threshold tracker for landlords.
//!
//! IRC §6041 + §6041A + Reg. §1.6041-1: a landlord who pays a
//! non-corporate person or unincorporated entity **$600 or more in a
//! tax year** for services rendered must issue **Form 1099-NEC** to
//! that vendor by **January 31** of the following year and file
//! copies with the IRS by **January 31** as well. Missing the filing
//! costs **$310 per form** ($630 with intentional disregard per
//! §6721(e)). The tracker aggregates the existing `rental_expenses`
//! ledger to flag vendors at risk of triggering the requirement.
//!
//! Carve-outs:
//!
//!   * **Payments to corporations** are exempted per
//!     Reg. §1.6041-3(p): C-corps and S-corps generally don't need
//!     1099-NEC. The exception within the exception: **attorneys**
//!     get 1099-NEC even when incorporated (gross proceeds payments
//!     under §6045(f)). Caller asserts corp status; we have no way
//!     to look up Form W-9 responses.
//!
//!   * **Credit-card payments** are excluded per
//!     Reg. §1.6041-1(a)(1)(iv). The card processor (Stripe, Square,
//!     PayPal Business) files **Form 1099-K** instead, and double-
//!     issuance would create duplicate reporting. Card payments
//!     show as `method = "card"` on the `rental_expenses` table.
//!
//!   * **Materials-only purchases** are also excluded — 1099-NEC
//!     reports payments for **services**. A $5,000 lumber purchase
//!     from a non-corporate sawmill does not trigger the requirement
//!     because no labor is involved. Caller asserts this via the
//!     `services_payment` flag on each entry; defaults to true.
//!
//! Pure compute. Caller passes a list of expense entries; we
//! aggregate by `vendor_normalized`, apply exclusions, check the
//! $600 threshold, and return a per-vendor report flagging which
//! vendors require 1099-NEC issuance.

use chrono::{DateTime, Datelike, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseEntry {
    pub vendor_normalized: String,
    pub amount: Decimal,
    pub posted_at: DateTime<Utc>,
    /// Payment method: 'ach' / 'check' / 'card' / 'cash' / etc. Only
    /// card payments are excluded from 1099-NEC issuance.
    pub method: String,
    /// True when this payment was for services (the default §6041
    /// trigger). False for materials-only purchases.
    pub services_payment: bool,
    /// True when the vendor is a C-corp or S-corp. Caller fills from
    /// their Form W-9 records. Defaults to false (i.e. assume
    /// 1099-eligible) for safety.
    pub vendor_is_corporation: bool,
    /// True when the vendor is an **attorney** (incorporated or not).
    /// Per §6045(f), attorneys ALWAYS get 1099-NEC regardless of
    /// corporate status.
    pub vendor_is_attorney: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contractor1099Input {
    pub tax_year: i32,
    pub entries: Vec<ExpenseEntry>,
    /// Override the §6041 $600 threshold. Used only if Congress
    /// raises it (proposed increases to $1,000 occasionally).
    pub threshold_override: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorReport {
    pub vendor_normalized: String,
    pub total_paid_in_year: Decimal,
    pub total_paid_qualifying: Decimal,
    pub payment_count_in_year: u32,
    pub latest_payment: Option<DateTime<Utc>>,
    pub paid_by_card_only: bool,
    pub vendor_is_corporation: bool,
    pub vendor_is_attorney: bool,
    pub all_payments_materials: bool,
    pub requires_1099_nec: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Contractor1099Report {
    pub tax_year: i32,
    pub threshold: Decimal,
    pub vendors: Vec<VendorReport>,
    pub total_vendors_requiring_1099: u32,
    pub total_qualifying_payments: Decimal,
    pub note: String,
}

fn default_threshold() -> Decimal {
    Decimal::from_str("600").unwrap()
}

pub fn compute(input: &Contractor1099Input) -> Contractor1099Report {
    let mut r = Contractor1099Report {
        tax_year: input.tax_year,
        threshold: input.threshold_override.unwrap_or_else(default_threshold),
        ..Contractor1099Report::default()
    };

    // Group by vendor_normalized. BTreeMap keeps deterministic output
    // order for testable behavior; a HashMap would be faster but
    // non-deterministic.
    let mut by_vendor: BTreeMap<String, Vec<&ExpenseEntry>> = BTreeMap::new();
    for entry in &input.entries {
        if entry.posted_at.year() != input.tax_year {
            continue;
        }
        by_vendor
            .entry(entry.vendor_normalized.clone())
            .or_default()
            .push(entry);
    }

    for (vendor, payments) in by_vendor {
        let total_paid: Decimal = payments.iter().map(|e| e.amount).sum();
        let count = payments.len() as u32;
        let latest = payments.iter().map(|e| e.posted_at).max();
        let all_card = payments
            .iter()
            .all(|e| e.method.eq_ignore_ascii_case("card"));
        let any_non_corp = !payments.iter().all(|e| e.vendor_is_corporation);
        let any_non_corp_services_non_card: Vec<&&ExpenseEntry> = payments
            .iter()
            .filter(|e| {
                !e.method.eq_ignore_ascii_case("card")
                    && e.services_payment
                    && (!e.vendor_is_corporation || e.vendor_is_attorney)
            })
            .collect();
        let qualifying_total: Decimal = any_non_corp_services_non_card
            .iter()
            .map(|e| e.amount)
            .sum();
        let all_materials = payments.iter().all(|e| !e.services_payment);
        let attorney = payments.iter().any(|e| e.vendor_is_attorney);
        let is_corp = !any_non_corp;

        // Trigger logic — the 1099-NEC requirement applies when
        // qualifying-payment total reaches the threshold.
        let triggered = qualifying_total >= r.threshold;

        let reason = if triggered {
            format!(
                "qualifying payments ${} ≥ ${} threshold — issue Form 1099-NEC",
                qualifying_total, r.threshold
            )
        } else if all_card {
            "all payments via card — Form 1099-K handled by processor, no 1099-NEC".into()
        } else if all_materials {
            "all payments materials-only — §6041 services trigger not met".into()
        } else if is_corp && !attorney {
            "vendor is a corporation (non-attorney) — Reg. §1.6041-3(p) exemption".into()
        } else if qualifying_total < r.threshold {
            format!(
                "qualifying payments ${} below ${} threshold — no 1099-NEC required",
                qualifying_total, r.threshold
            )
        } else {
            "no 1099-NEC required".into()
        };

        if triggered {
            r.total_vendors_requiring_1099 += 1;
            r.total_qualifying_payments += qualifying_total;
        }

        r.vendors.push(VendorReport {
            vendor_normalized: vendor,
            total_paid_in_year: total_paid,
            total_paid_qualifying: qualifying_total,
            payment_count_in_year: count,
            latest_payment: latest,
            paid_by_card_only: all_card,
            vendor_is_corporation: is_corp,
            vendor_is_attorney: attorney,
            all_payments_materials: all_materials,
            requires_1099_nec: triggered,
            reason,
        });
    }

    r.note = format!(
        "{} of {} vendor(s) require 1099-NEC for {} (${} threshold)",
        r.total_vendors_requiring_1099,
        r.vendors.len(),
        r.tax_year,
        r.threshold
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    fn dt(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
    }

    fn entry(vendor: &str, amount: Decimal, year: i32) -> ExpenseEntry {
        ExpenseEntry {
            vendor_normalized: vendor.into(),
            amount,
            posted_at: dt(year, 6, 1),
            method: "check".into(),
            services_payment: true,
            vendor_is_corporation: false,
            vendor_is_attorney: false,
        }
    }

    fn input(entries: Vec<ExpenseEntry>) -> Contractor1099Input {
        Contractor1099Input {
            tax_year: 2024,
            entries,
            threshold_override: None,
        }
    }

    #[test]
    fn single_vendor_under_600_no_1099() {
        let r = compute(&input(vec![entry("HANDYMAN_BOB", dec!(500), 2024)]));
        assert_eq!(r.vendors.len(), 1);
        assert!(!r.vendors[0].requires_1099_nec);
        assert_eq!(r.total_vendors_requiring_1099, 0);
    }

    #[test]
    fn single_vendor_exactly_600_triggers_1099() {
        // §6041 threshold is ≥ $600, not > $600.
        let r = compute(&input(vec![entry("HANDYMAN_BOB", dec!(600), 2024)]));
        assert!(r.vendors[0].requires_1099_nec);
    }

    #[test]
    fn single_vendor_one_dollar_under_600_no_1099() {
        let r = compute(&input(vec![entry("HANDYMAN_BOB", dec!(599.99), 2024)]));
        assert!(!r.vendors[0].requires_1099_nec);
    }

    #[test]
    fn multiple_payments_aggregate_to_threshold() {
        // Three $250 payments = $750 total, crosses the threshold.
        let r = compute(&input(vec![
            entry("ELECTRIC_COMPANY", dec!(250), 2024),
            entry("ELECTRIC_COMPANY", dec!(250), 2024),
            entry("ELECTRIC_COMPANY", dec!(250), 2024),
        ]));
        assert_eq!(r.vendors[0].total_paid_qualifying, dec!(750));
        assert_eq!(r.vendors[0].payment_count_in_year, 3);
        assert!(r.vendors[0].requires_1099_nec);
    }

    #[test]
    fn all_card_payments_excluded_no_1099() {
        let mut e1 = entry("LANDSCAPER", dec!(1500), 2024);
        e1.method = "card".into();
        let r = compute(&input(vec![e1]));
        assert!(r.vendors[0].paid_by_card_only);
        assert!(!r.vendors[0].requires_1099_nec);
        assert!(r.vendors[0].reason.contains("1099-K"));
    }

    #[test]
    fn mixed_card_and_check_still_triggers_on_non_card_portion() {
        // $400 card + $400 check = $800 total. Only the $400 check
        // counts toward the §6041 qualifying threshold. Under $600.
        let mut e1 = entry("LANDSCAPER", dec!(400), 2024);
        e1.method = "card".into();
        let e2 = entry("LANDSCAPER", dec!(400), 2024);
        let r = compute(&input(vec![e1, e2]));
        assert_eq!(r.vendors[0].total_paid_in_year, dec!(800));
        assert_eq!(r.vendors[0].total_paid_qualifying, dec!(400));
        assert!(!r.vendors[0].requires_1099_nec);
    }

    #[test]
    fn mixed_card_and_check_over_threshold_triggers() {
        let mut e1 = entry("LANDSCAPER", dec!(400), 2024);
        e1.method = "card".into();
        let e2 = entry("LANDSCAPER", dec!(700), 2024);
        let r = compute(&input(vec![e1, e2]));
        // Qualifying = $700 (non-card portion) ≥ $600.
        assert!(r.vendors[0].requires_1099_nec);
        assert_eq!(r.vendors[0].total_paid_qualifying, dec!(700));
    }

    #[test]
    fn corporation_vendor_excluded_unless_attorney() {
        let mut e = entry("PLUMBING_INC", dec!(2000), 2024);
        e.vendor_is_corporation = true;
        let r = compute(&input(vec![e]));
        assert!(r.vendors[0].vendor_is_corporation);
        assert!(!r.vendors[0].requires_1099_nec);
        assert!(r.vendors[0].reason.contains("Reg. §1.6041-3(p)"));
    }

    #[test]
    fn attorney_corporation_still_triggers_1099_nec() {
        // §6045(f) requires 1099-NEC for attorney payments even when
        // the attorney is incorporated.
        let mut e = entry("LAW_FIRM_LLP", dec!(5000), 2024);
        e.vendor_is_corporation = true;
        e.vendor_is_attorney = true;
        let r = compute(&input(vec![e]));
        assert!(r.vendors[0].vendor_is_attorney);
        assert!(r.vendors[0].requires_1099_nec);
    }

    #[test]
    fn materials_only_vendor_no_1099() {
        let mut e = entry("LUMBER_YARD", dec!(5000), 2024);
        e.services_payment = false;
        let r = compute(&input(vec![e]));
        assert!(r.vendors[0].all_payments_materials);
        assert!(!r.vendors[0].requires_1099_nec);
        assert!(r.vendors[0].reason.contains("materials"));
    }

    #[test]
    fn mixed_materials_and_services_triggers_on_services_portion() {
        // $400 materials + $300 labor (services). Qualifying = $300
        // below threshold.
        let mut e1 = entry("CONTRACTOR_BOB", dec!(400), 2024);
        e1.services_payment = false;
        let e2 = entry("CONTRACTOR_BOB", dec!(300), 2024);
        let r = compute(&input(vec![e1, e2]));
        assert_eq!(r.vendors[0].total_paid_in_year, dec!(700));
        assert_eq!(r.vendors[0].total_paid_qualifying, dec!(300));
        assert!(!r.vendors[0].requires_1099_nec);
    }

    #[test]
    fn year_filter_excludes_other_years() {
        // 2023 + 2024 entries; report for 2024 should only see 2024.
        let r = compute(&input(vec![
            entry("HVAC_PRO", dec!(700), 2023),
            entry("HVAC_PRO", dec!(500), 2024),
        ]));
        assert_eq!(r.vendors[0].total_paid_in_year, dec!(500));
        assert!(!r.vendors[0].requires_1099_nec);
    }

    #[test]
    fn empty_input_no_vendors_no_op() {
        let r = compute(&input(vec![]));
        assert_eq!(r.vendors.len(), 0);
        assert_eq!(r.total_vendors_requiring_1099, 0);
    }

    #[test]
    fn multiple_vendors_aggregated_separately() {
        let r = compute(&input(vec![
            entry("BOB", dec!(700), 2024),
            entry("ALICE", dec!(800), 2024),
            entry("CAROL", dec!(500), 2024),
        ]));
        assert_eq!(r.vendors.len(), 3);
        assert_eq!(r.total_vendors_requiring_1099, 2);
    }

    #[test]
    fn threshold_override_replaces_600_default() {
        let mut i = input(vec![entry("BOB", dec!(700), 2024)]);
        i.threshold_override = Some(dec!(1000));
        let r = compute(&i);
        assert_eq!(r.threshold, dec!(1000));
        // $700 < $1000 → no 1099 required.
        assert!(!r.vendors[0].requires_1099_nec);
    }

    #[test]
    fn case_insensitive_method_card_match() {
        let mut e = entry("VENDOR", dec!(700), 2024);
        e.method = "CARD".into();
        let r = compute(&input(vec![e]));
        assert!(r.vendors[0].paid_by_card_only);
        assert!(!r.vendors[0].requires_1099_nec);
    }

    #[test]
    fn latest_payment_date_reflects_max() {
        let mut e1 = entry("HVAC_PRO", dec!(300), 2024);
        e1.posted_at = dt(2024, 1, 15);
        let mut e2 = entry("HVAC_PRO", dec!(400), 2024);
        e2.posted_at = dt(2024, 11, 30);
        let r = compute(&input(vec![e1, e2]));
        assert_eq!(r.vendors[0].latest_payment.unwrap().day(), 30);
        assert_eq!(r.vendors[0].latest_payment.unwrap().month(), 11);
    }

    #[test]
    fn total_qualifying_payments_aggregates_across_vendors() {
        let r = compute(&input(vec![
            entry("BOB", dec!(700), 2024),
            entry("ALICE", dec!(800), 2024),
            entry("CAROL", dec!(500), 2024), // under threshold
        ]));
        assert_eq!(r.total_qualifying_payments, dec!(1500));
    }
}
