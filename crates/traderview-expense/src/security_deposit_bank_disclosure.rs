//! Security deposit bank disclosure — when must a landlord
//! disclose to the tenant where the security deposit is held?
//!
//! Distinct from sibling modules `security_deposit_caps`
//! (caps on the amount of the deposit), `deposit_interest`
//! (interest-payment obligation on the deposit), `deposit_return_
//! windows` (timelines for refunding the deposit), and
//! `damage_deduction_itemization` (deductions from the deposit).
//! This module focuses on the LOCATION-DISCLOSURE obligation —
//! NY, NJ, MA impose detailed written notice of bank name +
//! address + amount within statutory windows; most other
//! states have no bank-disclosure requirement.
//!
//! New York — N.Y. Gen. Oblig. Law § 7-103: Landlord must
//! notify the tenant in writing of (1) the name and address of
//! the banking organization where the security money is
//! deposited and (2) the amount of the deposit. For buildings
//! with SIX or MORE family dwelling units, the deposit must be
//! held in an INTEREST-BEARING account earning the prevailing
//! rate for similar deposits in the area; landlord may retain
//! 1% per annum as administration expense (in lieu of all other
//! administrative + custodial expenses); remaining interest
//! belongs to tenant, payable annually or applied to rent.
//! Trust-fund principle: deposit may not be commingled with
//! landlord's personal funds.
//!
//! New Jersey — N.J.S.A. 46:8-19: Landlord must notify tenant
//! in writing WITHIN 30 DAYS of receipt of (1) name and address
//! of investment company, state/federally chartered bank,
//! savings bank, or savings and loan association; (2) type of
//! account; (3) current interest rate; (4) amount of deposit.
//! Re-notification required within 30 days of (a) transfer of
//! deposit to new landlord or (b) move of deposit to another
//! account/bank. Annual interest payable to tenant in cash,
//! credited toward rent on lease anniversary, or paid on
//! January 31 if landlord has provided written election notice.
//!
//! Massachusetts — Mass. Gen. Laws c. 186 § 15B(3)(a): Landlord
//! must provide tenant with a RECEIPT within 30 days of
//! receiving the deposit. Receipt must include (1) name and
//! location of bank; (2) amount; (3) account number. Annual
//! statement required at end of each tenancy year showing
//! bank, amount, account number, and interest payable. Failure
//! to comply entitles tenant to IMMEDIATE RETURN of the
//! security deposit — Massachusetts' strict-compliance remedy
//! is the harshest in the country.
//!
//! Default — most states impose no bank-location disclosure
//! requirement. California Civ. Code § 1950.5 governs amount +
//! return but not bank location. Texas Prop. Code § 92.103 +
//! § 92.108 govern return procedures only.
//!
//! Citations: N.Y. Gen. Oblig. Law § 7-103 (general trust + bank
//! disclosure); N.Y. Gen. Oblig. Law § 7-103(2) (6+ unit
//! interest-bearing requirement + 1% admin retention); N.Y.
//! Gen. Oblig. Law § 7-108 (related disclosure for 6+ unit
//! buildings — 2019 HSTPA); N.J.S.A. 46:8-19 (NJ general
//! disclosure requirement); N.J.S.A. 46:8-19(a) (30-day initial
//! notice); N.J.S.A. 46:8-19(b) (re-notification on bank/
//! landlord change); Mass. Gen. Laws c. 186 § 15B(3)(a) (MA
//! receipt requirement); Mass. Gen. Laws c. 186 § 15B(6) (MA
//! immediate-return remedy); Cal. Civ. Code § 1950.5 (CA
//! amount + return — no bank disclosure); Tex. Prop. Code
//! § 92.103, § 92.108 (TX return procedures).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// N.Y. Gen. Oblig. Law § 7-103 — bank name + address +
    /// amount; 6+ unit interest-bearing requirement.
    NewYork,
    /// N.J.S.A. 46:8-19 — 30-day notice of bank + account type +
    /// interest rate + amount; re-notification on changes.
    NewJersey,
    /// Mass. Gen. Laws c. 186 § 15B — 30-day receipt; immediate
    /// return remedy for non-compliance.
    Massachusetts,
    /// No statutory bank-disclosure requirement.
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// Number of residential dwelling units in the building —
    /// NY 6+ trigger for interest-bearing requirement.
    pub units_in_building: i64,
    pub days_since_deposit_received: i64,
    pub bank_name_disclosed: bool,
    pub bank_address_disclosed: bool,
    /// MA-specific receipt component under § 15B(3)(a).
    pub account_number_disclosed: bool,
    /// NJ-specific notice component under § 46:8-19.
    pub account_type_disclosed: bool,
    /// NJ-specific notice component.
    pub interest_rate_disclosed: bool,
    pub amount_disclosed: bool,
    /// NY-specific — true if deposit held in interest-bearing
    /// account (required for 6+ unit buildings under § 7-103(2)).
    pub deposit_in_interest_bearing_account: bool,
    /// MA-specific — annual statement provided at end of each
    /// tenancy year.
    pub ma_annual_statement_provided: bool,
    /// NJ-specific — annual interest paid in cash, credited
    /// to rent, or paid on January 31 election.
    pub nj_annual_interest_paid_or_credited: bool,
    /// True if deposit was transferred to new landlord or moved
    /// to a different bank/account during the tenancy
    /// (NJ-specific re-notification trigger).
    pub deposit_transferred_or_moved: bool,
    /// Days since the transfer/move event.
    pub days_since_transfer_event: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the regime imposes any bank-disclosure requirement.
    pub disclosure_required: bool,
    /// True if any required disclosure element is missing.
    pub bank_disclosure_violation: bool,
    /// New York-specific — true if building has 6+ units
    /// (triggers interest-bearing account requirement).
    pub ny_interest_bearing_required: bool,
    /// New York-specific — true if 6+ unit building has deposit
    /// in interest-bearing account.
    pub ny_interest_bearing_compliant: bool,
    /// Massachusetts-specific — true if tenant entitled to
    /// IMMEDIATE return of deposit under § 15B(6).
    pub ma_immediate_return_remedy_engaged: bool,
    /// New Jersey-specific — true if 30-day notice window
    /// applies and is being measured.
    pub nj_30_day_window_engaged: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// New York § 7-103(2) trigger threshold for interest-bearing
/// account requirement.
pub const NY_INTEREST_BEARING_UNIT_THRESHOLD: i64 = 6;
/// New Jersey § 46:8-19 notice window.
pub const NJ_NOTICE_WINDOW_DAYS: i64 = 30;
/// Massachusetts § 15B(3)(a) receipt window.
pub const MA_RECEIPT_WINDOW_DAYS: i64 = 30;
/// New York § 7-103(2) — landlord's 1% admin retention from
/// interest (basis points).
pub const NY_ADMIN_RETENTION_BPS: i64 = 100;

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let disclosure_required = matches!(
        input.regime,
        Regime::NewYork | Regime::NewJersey | Regime::Massachusetts
    );

    // Default cleared values.
    let mut bank_disclosure_violation = false;
    let mut ny_interest_bearing_required = false;
    let mut ny_interest_bearing_compliant = true;
    let mut ma_immediate_return_remedy_engaged = false;
    let mut nj_30_day_window_engaged = false;

    match input.regime {
        Regime::NewYork => {
            // § 7-103 — bank name + address + amount.
            if !input.bank_name_disclosed {
                violations.push(
                    "N.Y. Gen. Oblig. Law § 7-103 — bank name was not disclosed to \
                     tenant. Landlord must notify tenant in writing of name and address \
                     of banking organization where security money is deposited."
                        .to_string(),
                );
                bank_disclosure_violation = true;
            }
            if !input.bank_address_disclosed {
                violations.push(
                    "N.Y. Gen. Oblig. Law § 7-103 — bank address was not disclosed."
                        .to_string(),
                );
                bank_disclosure_violation = true;
            }
            if !input.amount_disclosed {
                violations.push(
                    "N.Y. Gen. Oblig. Law § 7-103 — deposit amount was not disclosed."
                        .to_string(),
                );
                bank_disclosure_violation = true;
            }

            // § 7-103(2) — 6+ unit interest-bearing requirement.
            if input.units_in_building >= NY_INTEREST_BEARING_UNIT_THRESHOLD {
                ny_interest_bearing_required = true;
                if !input.deposit_in_interest_bearing_account {
                    violations.push(format!(
                        "N.Y. Gen. Oblig. Law § 7-103(2) — building has {} units (≥ 6); \
                         deposit MUST be held in interest-bearing account at the \
                         prevailing rate. Landlord may retain 1% per annum as \
                         administration expense; remaining interest belongs to tenant.",
                        input.units_in_building,
                    ));
                    ny_interest_bearing_compliant = false;
                }
            }

            notes.push(format!(
                "N.Y. Gen. Oblig. Law § 7-103 — trust-fund principle: deposit may NOT \
                 be commingled with landlord's personal funds. Bank disclosure required. \
                 Building has {} unit{}; interest-bearing requirement {}.",
                input.units_in_building,
                if input.units_in_building == 1 { "" } else { "s" },
                if ny_interest_bearing_required {
                    "ENGAGED (≥ 6 unit threshold)"
                } else {
                    "not engaged (< 6 units)"
                },
            ));
        }
        Regime::NewJersey => {
            nj_30_day_window_engaged = true;

            // § 46:8-19 — bank name + address + account type +
            // interest rate + amount within 30 days.
            let mut missing: Vec<&str> = Vec::new();
            if !input.bank_name_disclosed {
                missing.push("bank name");
            }
            if !input.bank_address_disclosed {
                missing.push("bank address");
            }
            if !input.account_type_disclosed {
                missing.push("account type");
            }
            if !input.interest_rate_disclosed {
                missing.push("current interest rate");
            }
            if !input.amount_disclosed {
                missing.push("deposit amount");
            }

            if !missing.is_empty() {
                violations.push(format!(
                    "N.J.S.A. 46:8-19 — bank disclosure incomplete. Missing element(s): \
                     {}. All five components required within 30 days of deposit receipt.",
                    missing.join("; "),
                ));
                bank_disclosure_violation = true;
            }

            if input.days_since_deposit_received > NJ_NOTICE_WINDOW_DAYS
                && (!missing.is_empty() || !input.bank_name_disclosed)
            {
                violations.push(format!(
                    "N.J.S.A. 46:8-19(a) — 30-day notice window EXPIRED ({} days since \
                     deposit receipt). Tenant may treat undisclosed deposit as full \
                     rent payment.",
                    input.days_since_deposit_received,
                ));
            }

            // § 46:8-19(b) — re-notification on bank/landlord change.
            if input.deposit_transferred_or_moved
                && input.days_since_transfer_event > NJ_NOTICE_WINDOW_DAYS
            {
                violations.push(format!(
                    "N.J.S.A. 46:8-19(b) — 30-day re-notification window EXPIRED ({} \
                     days since transfer/move event). Landlord must re-notify tenant of \
                     bank/account changes within 30 days.",
                    input.days_since_transfer_event,
                ));
            }

            if !input.nj_annual_interest_paid_or_credited {
                violations.push(
                    "N.J.S.A. 46:8-19 — annual interest must be paid to tenant in cash, \
                     credited toward rent on lease anniversary, or paid on January 31 \
                     pursuant to written election."
                        .to_string(),
                );
            }
        }
        Regime::Massachusetts => {
            // § 15B(3)(a) — receipt within 30 days; bank + amount +
            // account number. Failure → § 15B(6) immediate return.
            let mut missing_components: Vec<&str> = Vec::new();
            if !input.bank_name_disclosed {
                missing_components.push("bank name and location");
            }
            if !input.amount_disclosed {
                missing_components.push("amount");
            }
            if !input.account_number_disclosed {
                missing_components.push("account number");
            }

            let receipt_overdue =
                input.days_since_deposit_received > MA_RECEIPT_WINDOW_DAYS;

            if !missing_components.is_empty() || receipt_overdue {
                violations.push(format!(
                    "Mass. Gen. Laws c. 186 § 15B(3)(a) — receipt requirement violated. \
                     Missing components: {}; receipt overdue: {} ({} days, 30-day \
                     limit).",
                    if missing_components.is_empty() {
                        "none".to_string()
                    } else {
                        missing_components.join("; ")
                    },
                    receipt_overdue,
                    input.days_since_deposit_received,
                ));
                bank_disclosure_violation = true;
                ma_immediate_return_remedy_engaged = true;
                violations.push(
                    "Mass. Gen. Laws c. 186 § 15B(6) — STRICT COMPLIANCE REMEDY engaged: \
                     tenant entitled to IMMEDIATE RETURN of security deposit. \
                     Massachusetts imposes the harshest non-compliance remedy in the \
                     country; minor procedural defects trigger full forfeiture."
                        .to_string(),
                );
            }

            if !input.ma_annual_statement_provided {
                violations.push(
                    "Mass. Gen. Laws c. 186 § 15B(3)(a) — annual statement required at \
                     end of each tenancy year showing bank name + address, amount, \
                     account number, and interest payable."
                        .to_string(),
                );
            }
        }
        Regime::Default => {
            notes.push(
                "No statutory bank-disclosure requirement in this regime. Most states \
                 (CA Civ. Code § 1950.5, TX Prop. Code § 92.103 + § 92.108, FL Stat. \
                 § 83.49, etc.) regulate deposit AMOUNT + RETURN procedures but impose \
                 no LOCATION-DISCLOSURE obligation on the landlord. Lease provision may \
                 still require disclosure as matter of contract."
                    .to_string(),
            );
        }
    }

    // Sibling distinction note.
    notes.push(
        "Sibling distinction: this module covers LOCATION DISCLOSURE — where is the \
         deposit held? Sibling modules: `security_deposit_caps` (statutory caps on \
         amount), `deposit_interest` (interest-payment obligation), `deposit_return_\
         windows` (refund timelines), `damage_deduction_itemization` (deductions). \
         New York requires bank + address + amount with 6+ unit interest-bearing \
         escalation; New Jersey adds account type + interest rate within 30 days + \
         re-notification on changes; Massachusetts adds account number with the \
         country's harshest remedy (immediate return for non-compliance under § 15B(6)). \
         Most other states impose no statutory disclosure requirement."
            .to_string(),
    );

    let compliant = violations.is_empty();

    CheckResult {
        disclosure_required,
        bank_disclosure_violation,
        ny_interest_bearing_required,
        ny_interest_bearing_compliant,
        ma_immediate_return_remedy_engaged,
        nj_30_day_window_engaged,
        compliant,
        violations,
        citation: "N.Y. Gen. Oblig. Law § 7-103 (general trust + bank disclosure); \
                   N.Y. Gen. Oblig. Law § 7-103(2) (6+ unit interest-bearing + 1% admin \
                   retention); N.Y. Gen. Oblig. Law § 7-108 (related 2019 HSTPA \
                   disclosure for 6+ unit buildings); N.J.S.A. 46:8-19 (NJ general \
                   disclosure); N.J.S.A. 46:8-19(a) (30-day initial notice); N.J.S.A. \
                   46:8-19(b) (re-notification on bank/landlord change); Mass. Gen. Laws \
                   c. 186 § 15B(3)(a) (MA receipt + annual statement requirement); \
                   Mass. Gen. Laws c. 186 § 15B(6) (MA immediate-return remedy for \
                   non-compliance); Cal. Civ. Code § 1950.5 (CA amount + return — no \
                   bank disclosure); Tex. Prop. Code § 92.103 + § 92.108 (TX return \
                   procedures — no bank disclosure)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(regime: Regime) -> Input {
        Input {
            regime,
            units_in_building: 4,
            days_since_deposit_received: 15,
            bank_name_disclosed: true,
            bank_address_disclosed: true,
            account_number_disclosed: true,
            account_type_disclosed: true,
            interest_rate_disclosed: true,
            amount_disclosed: true,
            deposit_in_interest_bearing_account: true,
            ma_annual_statement_provided: true,
            nj_annual_interest_paid_or_credited: true,
            deposit_transferred_or_moved: false,
            days_since_transfer_event: 0,
        }
    }

    // ── New York § 7-103 ──────────────────────────────────────

    #[test]
    fn ny_baseline_compliant() {
        let r = check(&input(Regime::NewYork));
        assert!(r.compliant);
        assert!(r.disclosure_required);
    }

    #[test]
    fn ny_missing_bank_name_violation() {
        let mut b = input(Regime::NewYork);
        b.bank_name_disclosed = false;
        let r = check(&b);
        assert!(!r.compliant);
        assert!(r.bank_disclosure_violation);
        assert!(r.violations.iter().any(|v| v.contains("§ 7-103")));
    }

    #[test]
    fn ny_5_units_no_interest_bearing_requirement() {
        let mut b = input(Regime::NewYork);
        b.units_in_building = 5;
        b.deposit_in_interest_bearing_account = false;
        let r = check(&b);
        assert!(!r.ny_interest_bearing_required);
        // No interest-bearing violation since under threshold.
        assert!(r.compliant);
    }

    #[test]
    fn ny_6_units_interest_bearing_required() {
        let mut b = input(Regime::NewYork);
        b.units_in_building = 6;
        let r = check(&b);
        assert!(r.ny_interest_bearing_required);
    }

    #[test]
    fn ny_6_units_no_interest_bearing_account_violation() {
        let mut b = input(Regime::NewYork);
        b.units_in_building = 6;
        b.deposit_in_interest_bearing_account = false;
        let r = check(&b);
        assert!(r.ny_interest_bearing_required);
        assert!(!r.ny_interest_bearing_compliant);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 7-103(2)")));
    }

    #[test]
    fn ny_100_units_interest_bearing_required() {
        let mut b = input(Regime::NewYork);
        b.units_in_building = 100;
        let r = check(&b);
        assert!(r.ny_interest_bearing_required);
    }

    // ── New Jersey § 46:8-19 ──────────────────────────────────

    #[test]
    fn nj_baseline_compliant() {
        let r = check(&input(Regime::NewJersey));
        assert!(r.compliant);
        assert!(r.nj_30_day_window_engaged);
    }

    #[test]
    fn nj_missing_account_type_violation() {
        let mut b = input(Regime::NewJersey);
        b.account_type_disclosed = false;
        let r = check(&b);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("account type")));
    }

    #[test]
    fn nj_missing_interest_rate_violation() {
        let mut b = input(Regime::NewJersey);
        b.interest_rate_disclosed = false;
        let r = check(&b);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("current interest rate")));
    }

    #[test]
    fn nj_30_day_window_expired_violation() {
        let mut b = input(Regime::NewJersey);
        b.bank_name_disclosed = false;
        b.days_since_deposit_received = 31;
        let r = check(&b);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("30-day notice window EXPIRED")));
    }

    #[test]
    fn nj_transfer_event_30_day_expired_violation() {
        let mut b = input(Regime::NewJersey);
        b.deposit_transferred_or_moved = true;
        b.days_since_transfer_event = 31;
        let r = check(&b);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("46:8-19(b)")));
    }

    #[test]
    fn nj_annual_interest_not_paid_violation() {
        let mut b = input(Regime::NewJersey);
        b.nj_annual_interest_paid_or_credited = false;
        let r = check(&b);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("annual interest")));
    }

    // ── Massachusetts § 15B ───────────────────────────────────

    #[test]
    fn ma_baseline_compliant() {
        let r = check(&input(Regime::Massachusetts));
        assert!(r.compliant);
        assert!(!r.ma_immediate_return_remedy_engaged);
    }

    #[test]
    fn ma_missing_account_number_engages_immediate_return() {
        let mut b = input(Regime::Massachusetts);
        b.account_number_disclosed = false;
        let r = check(&b);
        assert!(!r.compliant);
        assert!(r.ma_immediate_return_remedy_engaged);
        assert!(r.violations.iter().any(|v| v.contains("§ 15B(6)")));
        assert!(r.violations.iter().any(|v| v.contains("IMMEDIATE RETURN")));
    }

    #[test]
    fn ma_receipt_overdue_engages_immediate_return() {
        let mut b = input(Regime::Massachusetts);
        b.days_since_deposit_received = 31;
        let r = check(&b);
        assert!(r.ma_immediate_return_remedy_engaged);
    }

    #[test]
    fn ma_30_day_boundary_exactly_compliant() {
        let mut b = input(Regime::Massachusetts);
        b.days_since_deposit_received = 30;
        let r = check(&b);
        // Statute reads "within 30 days" → day 30 = on time.
        assert!(r.compliant);
    }

    #[test]
    fn ma_missing_annual_statement_violation() {
        let mut b = input(Regime::Massachusetts);
        b.ma_annual_statement_provided = false;
        let r = check(&b);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("annual statement")));
    }

    // ── Default — no statutory requirement ────────────────────

    #[test]
    fn default_no_disclosure_required() {
        let r = check(&input(Regime::Default));
        assert!(!r.disclosure_required);
        assert!(r.compliant);
    }

    #[test]
    fn default_no_violation_even_with_all_disclosure_missing() {
        let mut b = input(Regime::Default);
        b.bank_name_disclosed = false;
        b.bank_address_disclosed = false;
        b.amount_disclosed = false;
        b.account_number_disclosed = false;
        b.account_type_disclosed = false;
        b.interest_rate_disclosed = false;
        let r = check(&b);
        assert!(r.compliant);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn only_ma_engages_immediate_return_invariant() {
        for regime in [
            Regime::NewYork,
            Regime::NewJersey,
            Regime::Massachusetts,
            Regime::Default,
        ] {
            let mut b = input(regime);
            b.account_number_disclosed = false;
            b.days_since_deposit_received = 60;
            let r = check(&b);
            let expected = matches!(regime, Regime::Massachusetts);
            assert_eq!(r.ma_immediate_return_remedy_engaged, expected, "{:?}", regime);
        }
    }

    #[test]
    fn only_ny_has_unit_threshold_invariant() {
        for regime in [
            Regime::NewYork,
            Regime::NewJersey,
            Regime::Massachusetts,
            Regime::Default,
        ] {
            let mut b = input(regime);
            b.units_in_building = 10;
            b.deposit_in_interest_bearing_account = false;
            let r = check(&b);
            let expected = matches!(regime, Regime::NewYork);
            assert_eq!(r.ny_interest_bearing_required, expected, "{:?}", regime);
        }
    }

    #[test]
    fn only_nj_engages_30_day_window_invariant() {
        for regime in [
            Regime::NewYork,
            Regime::NewJersey,
            Regime::Massachusetts,
            Regime::Default,
        ] {
            let r = check(&input(regime));
            let expected = matches!(regime, Regime::NewJersey);
            assert_eq!(r.nj_30_day_window_engaged, expected, "{:?}", regime);
        }
    }

    #[test]
    fn ny_6_unit_threshold_boundary_truth_table() {
        // 4-cell sweep: 5/6/7/100 units.
        let cells = [(5, false), (6, true), (7, true), (100, true)];
        for (units, expected_required) in cells.iter() {
            let mut b = input(Regime::NewYork);
            b.units_in_building = *units;
            let r = check(&b);
            assert_eq!(
                r.ny_interest_bearing_required,
                *expected_required,
                "units={}",
                units
            );
        }
    }

    #[test]
    fn citation_pins_all_regime_authorities() {
        let r = check(&input(Regime::NewYork));
        assert!(r.citation.contains("§ 7-103"));
        assert!(r.citation.contains("§ 7-103(2)"));
        assert!(r.citation.contains("§ 7-108"));
        assert!(r.citation.contains("46:8-19"));
        assert!(r.citation.contains("46:8-19(a)"));
        assert!(r.citation.contains("46:8-19(b)"));
        assert!(r.citation.contains("§ 15B(3)(a)"));
        assert!(r.citation.contains("§ 15B(6)"));
        assert!(r.citation.contains("§ 1950.5"));
        assert!(r.citation.contains("§ 92.103"));
        assert!(r.citation.contains("§ 92.108"));
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input(Regime::NewYork));
        assert!(
            r.notes.iter().any(|n| n.contains("security_deposit_caps")
                && n.contains("deposit_interest")
                && n.contains("deposit_return_windows")
                && n.contains("damage_deduction_itemization")
                && n.contains("LOCATION DISCLOSURE")),
            "sibling distinction note must reference 4 deposit-related sibling modules + LOCATION DISCLOSURE focus"
        );
    }

    #[test]
    fn ny_admin_retention_constant() {
        // 1% per annum admin retention.
        assert_eq!(NY_ADMIN_RETENTION_BPS, 100);
        assert_eq!(NY_INTEREST_BEARING_UNIT_THRESHOLD, 6);
        assert_eq!(NJ_NOTICE_WINDOW_DAYS, 30);
        assert_eq!(MA_RECEIPT_WINDOW_DAYS, 30);
    }

    // ── Defensive input clamping ──────────────────────────────

    #[test]
    fn defensive_negative_units_no_interest_bearing_required() {
        let mut b = input(Regime::NewYork);
        b.units_in_building = -5;
        let r = check(&b);
        assert!(!r.ny_interest_bearing_required);
    }

    #[test]
    fn defensive_negative_days_no_premature_violation() {
        let mut b = input(Regime::Massachusetts);
        b.days_since_deposit_received = -10;
        let r = check(&b);
        // Negative day count = deposit not yet received in terms of clock.
        assert!(r.compliant);
        assert!(!r.ma_immediate_return_remedy_engaged);
    }

    #[test]
    fn boundary_nj_at_30_days_compliant() {
        let mut b = input(Regime::NewJersey);
        b.days_since_deposit_received = 30;
        let r = check(&b);
        // Statute reads "within 30 days" → day 30 = on time.
        assert!(r.compliant);
    }

    #[test]
    fn boundary_nj_at_31_days_expired() {
        let mut b = input(Regime::NewJersey);
        b.bank_name_disclosed = false;
        b.days_since_deposit_received = 31;
        let r = check(&b);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("30-day notice window EXPIRED")));
    }
}
