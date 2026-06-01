//! IRC § 6045A — Information required in connection with transfers
//! of covered securities to brokers.
//!
//! Trader-critical for anyone changing brokers (transferring stock,
//! options, debt, or digital-asset positions). § 6045A requires the
//! TRANSFERRING broker (or other "applicable person") to furnish a
//! written information statement to the RECEIVING broker within 15
//! days of the transfer. The receiving broker uses this statement
//! to populate Form 1099-B basis reporting under § 6045 on the
//! eventual sale.
//!
//! Direct sibling to:
//!   - `section_6045` (broker information reporting on Form 1099-B —
//!     receiving broker's downstream filing obligation).
//!   - `section_6045b` (issuer Form 8937 organizational-action
//!     basis reporting — separate upstream feeder).
//!
//! Four operative subsections:
//!
//!   § 6045A(a) — GENERAL RULE: Every applicable person which
//!     transfers to a broker custody of a covered security in a
//!     transaction shall furnish to the broker a written statement
//!     as prescribed by the Secretary by regulations.
//!
//!   § 6045A(b) — APPLICABLE PERSON definition:
//!     (1) any broker (as defined in § 6045(c)(1)); and
//!     (2) any other person as provided by the Secretary in
//!         regulations.
//!
//!   § 6045A(c) — TIMING: Any statement required by subsection (a)
//!     shall be furnished not later than 15 DAYS after the date of
//!     the transfer.
//!
//!   § 6045A(d) — DIGITAL ASSET TRANSFERS (added by Infrastructure
//!     Investment and Jobs Act of 2021, Pub. L. 117-58 § 80603,
//!     effective for returns required to be filed and statements
//!     required to be furnished after 2025-12-31): Any broker with
//!     respect to any transfer of a covered security that is a
//!     digital asset from an account maintained by such broker to
//!     an account which is not maintained by a person that the
//!     broker knows or has reason to know is also a broker shall
//!     make a return showing the required transfer information.
//!
//! Statement content (Treas. Reg. § 1.6045A-1) includes:
//!   - Transferring broker name / address / TIN
//!   - Receiving broker name / address / TIN
//!   - Customer name / address / TIN
//!   - Security CUSIP / identifier
//!   - Quantity / share count
//!   - Adjusted basis (for covered securities)
//!   - Original acquisition date
//!   - Type of security
//!   - Whether security was acquired in a wash-sale-related transaction
//!
//! Citations: 26 U.S.C. § 6045A(a) (general transfer-statement rule);
//! § 6045A(b)(1) (broker definition via § 6045(c)(1)); § 6045A(b)(2)
//! (other person per Secretary); § 6045A(c) (15-day deadline);
//! § 6045A(d) (digital asset transfer return, effective post-2025-
//! 12-31 per Pub. L. 117-58 § 80603); Treas. Reg. § 1.6045A-1
//! (statement content + format).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityType {
    /// Covered traditional security — equity (post-2011), mutual
    /// funds (post-2012), debt + options (post-2014).
    CoveredSecurity,
    /// Covered digital asset — crypto/NFT/etc. § 6045A(d) digital-
    /// asset transfer return regime (post-2025-12-31 effective).
    DigitalAsset,
    /// Non-covered security — no basis reporting required.
    NonCovered,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransferDirection {
    /// § 6045A(a) — broker-to-broker custody transfer.
    BrokerToBroker,
    /// § 6045A(d) — broker-to-non-broker-account digital-asset
    /// transfer (post-2025-12-31).
    BrokerToNonBrokerAccount,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6045AInput {
    pub security_type: SecurityType,
    pub transfer_direction: TransferDirection,
    /// Days elapsed since the date of the transfer.
    pub days_since_transfer: u32,
    /// Whether the transferring person furnished the information
    /// statement.
    pub statement_furnished: bool,
    /// Whether the statement includes adjusted-basis information.
    pub statement_includes_basis: bool,
    /// Whether the statement includes the original acquisition
    /// date.
    pub statement_includes_acquisition_date: bool,
    /// Whether the statement identifies wash-sale-related acquisition.
    pub statement_includes_wash_sale_flag: bool,
    /// Whether the receiving party is known by the broker to also
    /// be a broker (relevant for § 6045A(d) digital-asset return
    /// trigger).
    pub receiving_party_known_as_broker: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6045AResult {
    /// True if § 6045A requires a statement / return for this
    /// transfer.
    pub statement_required: bool,
    /// Statutory deadline in days (§ 6045A(c) = 15 days).
    pub deadline_days: u32,
    /// True if the days_since_transfer exceeds the deadline.
    pub deadline_passed: bool,
    /// True for the § 6045A(d) digital-asset-transfer return path
    /// (post-2025-12-31 effective).
    pub digital_asset_return_required: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// § 6045A(c) — 15-day deadline for transferring broker to furnish
/// information statement to receiving broker.
pub const SECTION_6045A_DEADLINE_DAYS: u32 = 15;

pub fn check(input: &Section6045AInput) -> Section6045AResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    // Non-covered security — no § 6045A statement required.
    if matches!(input.security_type, SecurityType::NonCovered) {
        notes.push(
            "Non-covered security — broker is not required to report basis under § 6045 + \
             § 6045A. Transfer statement not required for non-covered securities."
                .to_string(),
        );
        return Section6045AResult {
            statement_required: false,
            deadline_days: SECTION_6045A_DEADLINE_DAYS,
            deadline_passed: false,
            digital_asset_return_required: false,
            compliant: true,
            violations,
            citation: "26 U.S.C. § 6045A applies to covered securities only; non-covered \
                       securities (acquired before 2011 for equity, before 2012 for mutual \
                       funds, before 2014 for debt and options) outside scope",
            notes,
        };
    }

    // § 6045A(d) digital-asset-to-non-broker-account return path.
    let digital_asset_return_required = matches!(input.security_type, SecurityType::DigitalAsset)
        && matches!(input.transfer_direction, TransferDirection::BrokerToNonBrokerAccount)
        && !input.receiving_party_known_as_broker;

    if digital_asset_return_required {
        notes.push(
            "§ 6045A(d) — digital-asset transfer from broker-maintained account to non-broker \
             account requires the broker to make a return showing the transfer information \
             (Pub. L. 117-58 § 80603, effective for returns required to be filed and \
             statements required to be furnished after 2025-12-31)."
                .to_string(),
        );
    }

    // § 6045A(c) 15-day deadline.
    let deadline_passed = input.days_since_transfer > SECTION_6045A_DEADLINE_DAYS;

    if !input.statement_furnished {
        violations.push(
            "§ 6045A(a) — transferring broker (or other applicable person) did not furnish the \
             required information statement to the receiving broker."
                .to_string(),
        );
    } else if deadline_passed {
        violations.push(format!(
            "§ 6045A(c) — statement furnished {} days after transfer; statutory deadline is \
             {} days.",
            input.days_since_transfer, SECTION_6045A_DEADLINE_DAYS,
        ));
    }

    // Statement content — basis + acquisition date required for
    // covered securities.
    if matches!(input.security_type, SecurityType::CoveredSecurity)
        && input.statement_furnished
    {
        if !input.statement_includes_basis {
            violations.push(
                "Treas. Reg. § 1.6045A-1 — transfer statement for covered security must include \
                 adjusted basis information."
                    .to_string(),
            );
        }
        if !input.statement_includes_acquisition_date {
            violations.push(
                "Treas. Reg. § 1.6045A-1 — transfer statement for covered security must include \
                 original acquisition date."
                    .to_string(),
            );
        }
        if !input.statement_includes_wash_sale_flag {
            violations.push(
                "Treas. Reg. § 1.6045A-1 — transfer statement for covered security must \
                 identify whether the security was acquired in a wash-sale-related transaction \
                 (per § 1091)."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Companion to section_6045 (broker Form 1099-B downstream basis reporting that consumes \
         the § 6045A transfer statement) and section_6045b (issuer Form 8937 organizational-\
         action basis reporting — separate upstream feeder)."
            .to_string(),
    );

    let citation = match input.security_type {
        SecurityType::DigitalAsset => {
            "26 U.S.C. § 6045A(a) (general transfer-statement rule); § 6045A(c) (15-day \
             deadline); § 6045A(d) (digital-asset transfer return — Pub. L. 117-58 § 80603 \
             effective post-2025-12-31); Treas. Reg. § 1.6045A-1 (content)"
        }
        SecurityType::CoveredSecurity => {
            "26 U.S.C. § 6045A(a) (general transfer-statement rule); § 6045A(b)(1) (broker \
             definition via § 6045(c)(1)); § 6045A(c) (15-day deadline); Treas. Reg. \
             § 1.6045A-1 (statement content — basis + acquisition date + wash-sale flag)"
        }
        SecurityType::NonCovered => unreachable!(),
    };

    Section6045AResult {
        statement_required: true,
        deadline_days: SECTION_6045A_DEADLINE_DAYS,
        deadline_passed,
        digital_asset_return_required,
        compliant: violations.is_empty(),
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(
        security: SecurityType,
        direction: TransferDirection,
    ) -> Section6045AInput {
        Section6045AInput {
            security_type: security,
            transfer_direction: direction,
            days_since_transfer: 10,
            statement_furnished: true,
            statement_includes_basis: true,
            statement_includes_acquisition_date: true,
            statement_includes_wash_sale_flag: true,
            receiving_party_known_as_broker: true,
        }
    }

    // ── § 6045A(a) general rule + § 6045A(c) deadline ──────────

    #[test]
    fn covered_security_broker_to_broker_within_15_days_compliant() {
        let r = check(&base(
            SecurityType::CoveredSecurity,
            TransferDirection::BrokerToBroker,
        ));
        assert!(r.compliant);
        assert!(r.statement_required);
        assert_eq!(r.deadline_days, 15);
        assert!(!r.deadline_passed);
        assert!(r.citation.contains("§ 6045A(a)"));
        assert!(r.citation.contains("§ 6045A(c)"));
    }

    #[test]
    fn covered_security_at_15_day_boundary_compliant() {
        let mut i = base(
            SecurityType::CoveredSecurity,
            TransferDirection::BrokerToBroker,
        );
        i.days_since_transfer = 15;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.deadline_passed);
    }

    #[test]
    fn covered_security_past_15_day_deadline_violation() {
        let mut i = base(
            SecurityType::CoveredSecurity,
            TransferDirection::BrokerToBroker,
        );
        i.days_since_transfer = 16;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.deadline_passed);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 6045A(c)") && v.contains("16 days"))
        );
    }

    #[test]
    fn statement_not_furnished_violation() {
        let mut i = base(
            SecurityType::CoveredSecurity,
            TransferDirection::BrokerToBroker,
        );
        i.statement_furnished = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 6045A(a)") && v.contains("did not furnish"))
        );
    }

    // ── Statement content requirements (Treas. Reg. § 1.6045A-1) ─

    #[test]
    fn missing_basis_violation() {
        let mut i = base(
            SecurityType::CoveredSecurity,
            TransferDirection::BrokerToBroker,
        );
        i.statement_includes_basis = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 1.6045A-1") && v.contains("adjusted basis"))
        );
    }

    #[test]
    fn missing_acquisition_date_violation() {
        let mut i = base(
            SecurityType::CoveredSecurity,
            TransferDirection::BrokerToBroker,
        );
        i.statement_includes_acquisition_date = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 1.6045A-1") && v.contains("acquisition date"))
        );
    }

    #[test]
    fn missing_wash_sale_flag_violation() {
        let mut i = base(
            SecurityType::CoveredSecurity,
            TransferDirection::BrokerToBroker,
        );
        i.statement_includes_wash_sale_flag = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 1.6045A-1")
                    && v.contains("wash-sale")
                    && v.contains("§ 1091"))
        );
    }

    // ── Non-covered security — outside § 6045A scope ────────────

    #[test]
    fn non_covered_security_outside_scope_compliant() {
        let r = check(&base(
            SecurityType::NonCovered,
            TransferDirection::BrokerToBroker,
        ));
        assert!(r.compliant);
        assert!(!r.statement_required);
        assert!(r.citation.contains("non-covered securities"));
    }

    #[test]
    fn non_covered_security_late_furnish_still_compliant() {
        // Even with no statement and 100 days elapsed, non-covered
        // is outside § 6045A scope.
        let mut i = base(
            SecurityType::NonCovered,
            TransferDirection::BrokerToBroker,
        );
        i.statement_furnished = false;
        i.days_since_transfer = 100;
        let r = check(&i);
        assert!(r.compliant);
    }

    // ── § 6045A(d) digital-asset transfer return ────────────────

    #[test]
    fn digital_asset_broker_to_non_broker_return_required() {
        let mut i = base(
            SecurityType::DigitalAsset,
            TransferDirection::BrokerToNonBrokerAccount,
        );
        i.receiving_party_known_as_broker = false;
        let r = check(&i);
        assert!(r.digital_asset_return_required);
        assert!(r.citation.contains("§ 6045A(d)"));
        assert!(r.citation.contains("Pub. L. 117-58"));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 6045A(d)") && n.contains("2025-12-31"))
        );
    }

    #[test]
    fn digital_asset_broker_to_known_broker_no_d_return() {
        let mut i = base(
            SecurityType::DigitalAsset,
            TransferDirection::BrokerToNonBrokerAccount,
        );
        i.receiving_party_known_as_broker = true;
        let r = check(&i);
        // Receiving party known as broker → § 6045A(a) applies but
        // not § 6045A(d) return.
        assert!(!r.digital_asset_return_required);
    }

    #[test]
    fn digital_asset_broker_to_broker_no_d_return() {
        let mut i = base(
            SecurityType::DigitalAsset,
            TransferDirection::BrokerToBroker,
        );
        i.receiving_party_known_as_broker = true;
        let r = check(&i);
        // Standard broker-to-broker → § 6045A(a) statement; not
        // § 6045A(d) return.
        assert!(!r.digital_asset_return_required);
    }

    // ── Edge cases ─────────────────────────────────────────────

    #[test]
    fn zero_days_since_transfer_compliant() {
        let mut i = base(
            SecurityType::CoveredSecurity,
            TransferDirection::BrokerToBroker,
        );
        i.days_since_transfer = 0;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn day_14_compliant_day_16_violation_boundary() {
        for (days, expected_passed) in [(0_u32, false), (14, false), (15, false), (16, true), (30, true)] {
            let mut i = base(
                SecurityType::CoveredSecurity,
                TransferDirection::BrokerToBroker,
            );
            i.days_since_transfer = days;
            let r = check(&i);
            assert_eq!(
                r.deadline_passed, expected_passed,
                "day {} expected_passed={}",
                days, expected_passed,
            );
        }
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn only_covered_or_digital_asset_requires_statement_invariant() {
        for security in [
            SecurityType::CoveredSecurity,
            SecurityType::DigitalAsset,
        ] {
            let r = check(&base(security, TransferDirection::BrokerToBroker));
            assert!(
                r.statement_required,
                "{:?}: must require statement",
                security,
            );
        }
        let r = check(&base(
            SecurityType::NonCovered,
            TransferDirection::BrokerToBroker,
        ));
        assert!(!r.statement_required);
    }

    #[test]
    fn digital_asset_return_only_triggered_by_specific_combo_invariant() {
        // § 6045A(d) return triggers ONLY when:
        // (1) security is DigitalAsset, (2) direction is
        // BrokerToNonBrokerAccount, AND (3) receiving party is NOT
        // known as broker.
        let mut both = base(
            SecurityType::DigitalAsset,
            TransferDirection::BrokerToNonBrokerAccount,
        );
        both.receiving_party_known_as_broker = false;
        assert!(check(&both).digital_asset_return_required);

        // Any single condition flipped → no § 6045A(d) trigger.
        for security in [SecurityType::CoveredSecurity, SecurityType::NonCovered] {
            let mut i = base(security, TransferDirection::BrokerToNonBrokerAccount);
            i.receiving_party_known_as_broker = false;
            assert!(
                !check(&i).digital_asset_return_required,
                "{:?}: must not trigger § 6045A(d)",
                security,
            );
        }
        let mut wrong_direction = base(
            SecurityType::DigitalAsset,
            TransferDirection::BrokerToBroker,
        );
        wrong_direction.receiving_party_known_as_broker = false;
        assert!(!check(&wrong_direction).digital_asset_return_required);

        let mut known_broker = base(
            SecurityType::DigitalAsset,
            TransferDirection::BrokerToNonBrokerAccount,
        );
        known_broker.receiving_party_known_as_broker = true;
        assert!(!check(&known_broker).digital_asset_return_required);
    }

    #[test]
    fn deadline_constant_15_days_invariant() {
        assert_eq!(SECTION_6045A_DEADLINE_DAYS, 15);
        let r = check(&base(
            SecurityType::CoveredSecurity,
            TransferDirection::BrokerToBroker,
        ));
        assert_eq!(r.deadline_days, 15);
    }

    #[test]
    fn citation_pins_subsections_per_security_type() {
        let covered = check(&base(
            SecurityType::CoveredSecurity,
            TransferDirection::BrokerToBroker,
        ));
        let digital = check(&base(
            SecurityType::DigitalAsset,
            TransferDirection::BrokerToBroker,
        ));
        let non_covered = check(&base(
            SecurityType::NonCovered,
            TransferDirection::BrokerToBroker,
        ));

        assert!(covered.citation.contains("§ 6045A(a)"));
        assert!(covered.citation.contains("§ 6045A(b)(1)"));
        assert!(covered.citation.contains("§ 6045A(c)"));
        assert!(covered.citation.contains("§ 1.6045A-1"));

        assert!(digital.citation.contains("§ 6045A(d)"));
        assert!(digital.citation.contains("Pub. L. 117-58"));

        assert!(non_covered.citation.contains("non-covered securities"));
    }

    #[test]
    fn content_requirements_only_apply_to_covered_securities_invariant() {
        // Digital asset path doesn't trigger same content-check
        // violations as covered-security path (different reg cite).
        let mut digital = base(
            SecurityType::DigitalAsset,
            TransferDirection::BrokerToBroker,
        );
        digital.statement_includes_basis = false;
        digital.statement_includes_acquisition_date = false;
        digital.statement_includes_wash_sale_flag = false;
        let r = check(&digital);
        // Digital-asset transfers don't fail § 1.6045A-1 covered-
        // security content checks.
        assert!(
            !r.violations
                .iter()
                .any(|v| v.contains("§ 1.6045A-1") && v.contains("adjusted basis"))
        );
    }

    #[test]
    fn sibling_module_note_present_across_all_security_types() {
        for security in [
            SecurityType::CoveredSecurity,
            SecurityType::DigitalAsset,
        ] {
            let r = check(&base(security, TransferDirection::BrokerToBroker));
            assert!(
                r.notes.iter().any(|n| n.contains("section_6045")
                    && n.contains("section_6045b")
                    && n.contains("Form 8937")),
                "{:?}: sibling-module note must be present",
                security,
            );
        }
    }
}
