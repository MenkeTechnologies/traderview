//! Portable / reusable tenant screening report regulation — when
//! must a landlord accept a tenant-provided pre-pulled screening
//! report in lieu of conducting and charging for their own?
//!
//! Distinct from `application_fees` (caps on application fees
//! generally) and `adverse_action_notice` (FCRA notice when
//! application denied based on report). This module addresses
//! the narrow question of whether the landlord can REJECT a
//! tenant-prepared portable report and require a fresh fee-based
//! screening of their own.
//!
//! Colorado — Colo. Rev. Stat. § 38-12-902 / § 38-12-904 (HB23-
//! 1099, eff. 2023): MANDATORY ACCEPTANCE regime. If a
//! prospective tenant provides a portable tenant screening
//! report meeting four conditions (≤30 days old; obtained
//! directly from a consumer reporting agency or third-party
//! website complying with state and federal laws; available to
//! landlord at no cost; includes tenant's statement of no
//! material change), the landlord MUST accept it AND MAY NOT
//! charge a rental application fee or any fee to access/use the
//! report. § 38-12-904(4) violation: $2,500 plus court costs +
//! attorney fees; reduced to $50 if landlord cures within
//! 7 calendar days of receiving notice. Exception: landlord
//! accepting only one application at a time + refunding within
//! 20 days of denial is exempt.
//!
//! Washington — RCW 59.18.257: OPT-IN DISCLOSURE regime. The
//! landlord must DISCLOSE on the property's home page (if
//! advertising online) and in writing to the prospective tenant
//! whether the landlord will accept a Comprehensive Reusable
//! Tenant Screening Report (CRTSR). CRTSR defined as: paid for
//! by tenant, prepared within past 30 days, contains consumer
//! credit report + criminal history + eviction history +
//! employment verification + address and rental history. If
//! landlord opts in (declares acceptance), MUST accept and may
//! NOT charge tenant for screening. If landlord opts out
//! (declares non-acceptance), may conduct own screening at
//! actual cost capped at customary local screening-service rate.
//! § 59.18.257(2) violation: up to $100 + court costs + reasonable
//! attorney fees.
//!
//! Default — no statutory portability requirement. Landlord may
//! demand a fresh fee-based screening of their own under the
//! general application_fees regime + FCRA disclosure obligations.
//! Most states follow this default.
//!
//! Citations: Colo. Rev. Stat. § 38-12-902 (HB23-1099 portable
//! tenant screening report — definition); Colo. Rev. Stat.
//! § 38-12-904 (consideration of rental applications — mandatory
//! acceptance + fee prohibition); Colo. Rev. Stat. § 38-12-904(4)
//! ($2,500 violation penalty + $50 reduction-for-7-day cure);
//! RCW 59.18.257 (Washington screening of prospective tenants);
//! RCW 59.18.257(1) (opt-in disclosure requirement); RCW
//! 59.18.257(2) ($100 violation penalty + attorney fees);
//! Federal Fair Credit Reporting Act, 15 U.S.C. § 1681 et seq.
//! (consumer reporting agency framework underlying both state
//! regimes).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// Colo. Rev. Stat. § 38-12-902 / § 38-12-904 — mandatory
    /// acceptance + fee prohibition.
    Colorado,
    /// RCW 59.18.257 — opt-in disclosure regime.
    Washington,
    /// No statutory portability requirement; lease + landlord
    /// discretion controls.
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// True if the tenant has provided a portable tenant
    /// screening report to the landlord.
    pub screening_report_provided_by_tenant: bool,
    /// Age of the screening report in days at time of submission.
    pub report_age_days: i64,
    /// Colorado-specific — true if the tenant included the
    /// statutorily-required statement of no material change.
    pub no_material_change_statement_included: bool,
    /// Washington-specific CRTSR components (also Colorado uses
    /// similar completeness).
    pub report_includes_credit_history: bool,
    pub report_includes_criminal_history: bool,
    pub report_includes_eviction_history: bool,
    pub report_includes_employment_verification: bool,
    pub report_includes_rental_history: bool,
    /// Washington-specific — landlord's published disclosure
    /// stating whether they accept CRTSRs.
    pub landlord_website_disclosure_accepts_report: bool,
    /// True if landlord accepted the tenant-provided report.
    pub landlord_accepted_report: bool,
    /// True if landlord charged the tenant an application/
    /// screening fee.
    pub landlord_charged_application_fee: bool,
    /// Colorado-specific — true if landlord cured a violation
    /// within 7 calendar days of notice.
    pub landlord_cured_within_7_days: bool,
    /// Colorado-specific — true if landlord falls within the
    /// single-application-at-a-time + 20-day-refund exception.
    pub colorado_single_application_exception: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the regime requires landlord to accept a
    /// compliant portable report.
    pub portability_required: bool,
    /// True if the report meets the regime's completeness +
    /// validity requirements.
    pub report_meets_requirements: bool,
    /// True if the report is within the 30-day validity window.
    pub report_within_validity_window: bool,
    /// Penalty exposure if landlord refused compliant report
    /// and/or charged improper fee (cents).
    pub penalty_amount_cents: i64,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// 30-day report validity window (both CO + WA).
pub const REPORT_VALIDITY_WINDOW_DAYS: i64 = 30;
/// Colorado § 38-12-904(4) — full violation penalty (cents).
pub const COLORADO_FULL_PENALTY_CENTS: i64 = 250_000;
/// Colorado § 38-12-904(4) — cured-violation reduced penalty.
pub const COLORADO_CURED_PENALTY_CENTS: i64 = 5_000;
/// Colorado cure window (calendar days).
pub const COLORADO_CURE_WINDOW_DAYS: i64 = 7;
/// Washington § 59.18.257(2) — violation penalty cap (cents).
pub const WASHINGTON_VIOLATION_PENALTY_CENTS: i64 = 10_000;

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let report_within_validity_window =
        input.report_age_days >= 0 && input.report_age_days <= REPORT_VALIDITY_WINDOW_DAYS;

    // Completeness — all 5 CRTSR components required for WA;
    // CO requires similar baseline plus no-material-change statement.
    let crtsr_components_complete = input.report_includes_credit_history
        && input.report_includes_criminal_history
        && input.report_includes_eviction_history
        && input.report_includes_employment_verification
        && input.report_includes_rental_history;

    let report_meets_requirements = input.screening_report_provided_by_tenant
        && report_within_validity_window
        && crtsr_components_complete
        && (input.regime != Regime::Colorado || input.no_material_change_statement_included);

    let portability_required = match input.regime {
        Regime::Colorado => !input.colorado_single_application_exception,
        Regime::Washington => input.landlord_website_disclosure_accepts_report,
        Regime::Default => false,
    };

    let mut penalty_amount_cents: i64 = 0;

    match input.regime {
        Regime::Colorado => {
            if portability_required
                && report_meets_requirements
                && !input.landlord_accepted_report
            {
                penalty_amount_cents = if input.landlord_cured_within_7_days {
                    COLORADO_CURED_PENALTY_CENTS
                } else {
                    COLORADO_FULL_PENALTY_CENTS
                };
                violations.push(format!(
                    "Colo. Rev. Stat. § 38-12-904 — mandatory portable tenant screening \
                     report acceptance violated. Compliant report (≤30 days old, complete \
                     CRTSR components, no-material-change statement) was rejected. \
                     § 38-12-904(4) penalty: {} cents ({}).",
                    penalty_amount_cents,
                    if input.landlord_cured_within_7_days {
                        "$50 reduced — cured within 7 days of notice"
                    } else {
                        "$2,500 full — not cured"
                    },
                ));
            }
            if portability_required
                && report_meets_requirements
                && input.landlord_accepted_report
                && input.landlord_charged_application_fee
            {
                penalty_amount_cents = if input.landlord_cured_within_7_days {
                    COLORADO_CURED_PENALTY_CENTS
                } else {
                    COLORADO_FULL_PENALTY_CENTS
                };
                violations.push(format!(
                    "Colo. Rev. Stat. § 38-12-904 — fee charged despite accepting compliant \
                     portable report. § 38-12-904 prohibits BOTH application fee AND \
                     fee for access/use of report. Penalty: {} cents.",
                    penalty_amount_cents,
                ));
            }
        }
        Regime::Washington => {
            if input.landlord_website_disclosure_accepts_report
                && report_meets_requirements
                && !input.landlord_accepted_report
            {
                penalty_amount_cents = WASHINGTON_VIOLATION_PENALTY_CENTS;
                violations.push(
                    "RCW 59.18.257(1) — landlord declared CRTSR acceptance on property \
                     website but then refused the compliant report. § 59.18.257(2) \
                     penalty: up to $100 + court costs + reasonable attorney fees."
                        .to_string(),
                );
            }
            if input.landlord_website_disclosure_accepts_report
                && report_meets_requirements
                && input.landlord_accepted_report
                && input.landlord_charged_application_fee
            {
                penalty_amount_cents = WASHINGTON_VIOLATION_PENALTY_CENTS;
                violations.push(
                    "RCW 59.18.257(1) — landlord declared CRTSR acceptance but charged \
                     a screening fee. Once CRTSR is accepted, no separate screening fee \
                     may be charged. § 59.18.257(2) penalty: up to $100."
                        .to_string(),
                );
            }
        }
        Regime::Default => {
            // No statutory portability requirement.
        }
    }

    // Notes.
    match input.regime {
        Regime::Colorado => {
            if input.colorado_single_application_exception {
                notes.push(
                    "Colo. Rev. Stat. § 38-12-904 single-application exception engaged \
                     — landlord accepts only one application at a time AND refunds fee \
                     within 20 days of decline → not required to accept portable report."
                        .to_string(),
                );
            } else {
                notes.push(format!(
                    "Colo. Rev. Stat. § 38-12-902/904 — MANDATORY acceptance regime. Report \
                     must meet four conditions: (1) ≤{} days old; (2) prepared by consumer \
                     reporting agency or compliant third-party website; (3) available at \
                     no cost to landlord; (4) includes tenant statement of no material \
                     change. § 38-12-904(4) violation penalty: $2,500 (reduced to $50 if \
                     cured within {} days of notice).",
                    REPORT_VALIDITY_WINDOW_DAYS, COLORADO_CURE_WINDOW_DAYS,
                ));
            }
        }
        Regime::Washington => {
            notes.push(format!(
                "RCW 59.18.257 — OPT-IN DISCLOSURE regime. Landlord must publish on \
                 property website whether CRTSR will be accepted. Landlord disclosure \
                 status: {}. CRTSR components: credit + criminal + eviction + employment \
                 + rental history; ≤{} days old. § 59.18.257(2) violation penalty: up to \
                 $100 + attorney fees.",
                if input.landlord_website_disclosure_accepts_report {
                    "ACCEPTS — mandatory acceptance of compliant CRTSR"
                } else {
                    "DOES NOT ACCEPT — may conduct own screening at actual cost"
                },
                REPORT_VALIDITY_WINDOW_DAYS,
            ));
        }
        Regime::Default => {
            notes.push(
                "No statutory portability requirement — landlord may demand fresh \
                 fee-based screening under general application_fees regime + FCRA \
                 disclosure obligations. Most states follow this default; the portable-\
                 report mandate originated in Colorado (2023) and the opt-in disclosure \
                 model in Washington."
                    .to_string(),
            );
        }
    }

    if input.screening_report_provided_by_tenant && !report_within_validity_window {
        notes.push(format!(
            "Provided report is {} days old (>{} day validity window) — landlord may \
             reject without violating portability requirement.",
            input.report_age_days, REPORT_VALIDITY_WINDOW_DAYS,
        ));
    }

    if input.screening_report_provided_by_tenant && !crtsr_components_complete {
        let missing: Vec<&str> = [
            (input.report_includes_credit_history, "credit history"),
            (input.report_includes_criminal_history, "criminal history"),
            (input.report_includes_eviction_history, "eviction history"),
            (
                input.report_includes_employment_verification,
                "employment verification",
            ),
            (input.report_includes_rental_history, "rental history"),
        ]
        .iter()
        .filter_map(|(present, label)| if *present { None } else { Some(*label) })
        .collect();
        notes.push(format!(
            "Provided report missing CRTSR component(s): {}. Landlord may reject \
             incomplete report.",
            missing.join("; "),
        ));
    }

    notes.push(
        "Sibling distinction: this module covers REPORT PORTABILITY — when must a \
         landlord accept a tenant-prepared screening report? Application fee CAPS \
         (general limits on screening fees) are covered by `application_fees` sibling; \
         FCRA disclosure on application denial is covered by `adverse_action_notice` \
         sibling. Both Colorado and Washington base their portability regimes on the \
         federal Fair Credit Reporting Act (15 U.S.C. § 1681 et seq.) consumer reporting \
         agency framework."
            .to_string(),
    );

    CheckResult {
        portability_required,
        report_meets_requirements,
        report_within_validity_window,
        penalty_amount_cents,
        compliant: violations.is_empty(),
        violations,
        citation: "Colo. Rev. Stat. § 38-12-902 (HB23-1099 — portable tenant screening \
                   report definition); Colo. Rev. Stat. § 38-12-904 (mandatory \
                   acceptance + fee prohibition); Colo. Rev. Stat. § 38-12-904(4) \
                   ($2,500 violation penalty + $50 reduced-for-7-day cure); RCW \
                   59.18.257 (Washington screening of prospective tenants); RCW \
                   59.18.257(1) (opt-in disclosure requirement); RCW 59.18.257(2) \
                   ($100 violation penalty + attorney fees); Federal Fair Credit \
                   Reporting Act, 15 U.S.C. § 1681 et seq. (consumer reporting agency \
                   framework underlying both state regimes)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(regime: Regime) -> Input {
        Input {
            regime,
            screening_report_provided_by_tenant: true,
            report_age_days: 15,
            no_material_change_statement_included: true,
            report_includes_credit_history: true,
            report_includes_criminal_history: true,
            report_includes_eviction_history: true,
            report_includes_employment_verification: true,
            report_includes_rental_history: true,
            landlord_website_disclosure_accepts_report: true,
            landlord_accepted_report: true,
            landlord_charged_application_fee: false,
            landlord_cured_within_7_days: false,
            colorado_single_application_exception: false,
        }
    }

    // ── Colorado mandatory acceptance ──────────────────────────

    #[test]
    fn colorado_compliant_report_accepted_compliant() {
        let r = check(&input(Regime::Colorado));
        assert!(r.compliant);
        assert!(r.portability_required);
        assert!(r.report_meets_requirements);
        assert_eq!(r.penalty_amount_cents, 0);
    }

    #[test]
    fn colorado_compliant_report_refused_2500_penalty() {
        let mut b = input(Regime::Colorado);
        b.landlord_accepted_report = false;
        let r = check(&b);
        assert!(!r.compliant);
        assert_eq!(r.penalty_amount_cents, 250_000);
    }

    #[test]
    fn colorado_refused_cured_within_7_days_50_penalty() {
        let mut b = input(Regime::Colorado);
        b.landlord_accepted_report = false;
        b.landlord_cured_within_7_days = true;
        let r = check(&b);
        assert!(!r.compliant);
        assert_eq!(r.penalty_amount_cents, 5_000);
    }

    #[test]
    fn colorado_accepted_but_fee_charged_violation() {
        let mut b = input(Regime::Colorado);
        b.landlord_charged_application_fee = true;
        let r = check(&b);
        assert!(!r.compliant);
        assert_eq!(r.penalty_amount_cents, 250_000);
    }

    #[test]
    fn colorado_single_application_exception_no_portability_required() {
        let mut b = input(Regime::Colorado);
        b.colorado_single_application_exception = true;
        b.landlord_accepted_report = false;
        let r = check(&b);
        assert!(!r.portability_required);
        assert!(r.compliant);
    }

    #[test]
    fn colorado_report_31_days_old_outside_validity() {
        let mut b = input(Regime::Colorado);
        b.report_age_days = 31;
        b.landlord_accepted_report = false;
        let r = check(&b);
        assert!(!r.report_within_validity_window);
        assert!(!r.report_meets_requirements);
        // Landlord may reject without penalty.
        assert!(r.compliant);
    }

    #[test]
    fn colorado_missing_material_change_statement_not_compliant_report() {
        let mut b = input(Regime::Colorado);
        b.no_material_change_statement_included = false;
        b.landlord_accepted_report = false;
        let r = check(&b);
        assert!(!r.report_meets_requirements);
        assert!(r.compliant);
    }

    // ── Washington opt-in disclosure ──────────────────────────

    #[test]
    fn washington_landlord_accepts_compliant_report_compliant() {
        let r = check(&input(Regime::Washington));
        assert!(r.compliant);
        assert!(r.portability_required);
    }

    #[test]
    fn washington_landlord_declared_accepts_then_refused_100_penalty() {
        let mut b = input(Regime::Washington);
        b.landlord_accepted_report = false;
        let r = check(&b);
        assert!(!r.compliant);
        assert_eq!(r.penalty_amount_cents, 10_000);
    }

    #[test]
    fn washington_landlord_opt_out_no_portability_required() {
        let mut b = input(Regime::Washington);
        b.landlord_website_disclosure_accepts_report = false;
        b.landlord_accepted_report = false;
        let r = check(&b);
        assert!(!r.portability_required);
        assert!(r.compliant);
    }

    #[test]
    fn washington_accepts_then_charges_fee_violation() {
        let mut b = input(Regime::Washington);
        b.landlord_charged_application_fee = true;
        let r = check(&b);
        assert!(!r.compliant);
        assert_eq!(r.penalty_amount_cents, 10_000);
    }

    #[test]
    fn washington_missing_crtsr_component_landlord_may_reject() {
        let mut b = input(Regime::Washington);
        b.report_includes_employment_verification = false;
        b.landlord_accepted_report = false;
        let r = check(&b);
        assert!(!r.report_meets_requirements);
        assert!(r.compliant);
    }

    // ── Default ────────────────────────────────────────────────

    #[test]
    fn default_no_portability_no_violation() {
        let mut b = input(Regime::Default);
        b.landlord_accepted_report = false;
        b.landlord_charged_application_fee = true;
        let r = check(&b);
        assert!(!r.portability_required);
        assert!(r.compliant);
        assert_eq!(r.penalty_amount_cents, 0);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn colorado_penalty_higher_than_washington_invariant() {
        // CO full penalty $2,500 > WA penalty $100 cap.
        assert!(COLORADO_FULL_PENALTY_CENTS > WASHINGTON_VIOLATION_PENALTY_CENTS);
        // CO 25× WA penalty.
        assert_eq!(COLORADO_FULL_PENALTY_CENTS, 25 * WASHINGTON_VIOLATION_PENALTY_CENTS);
    }

    #[test]
    fn colorado_cure_reduces_penalty_50x_invariant() {
        // $50 cured / $2500 full = 1/50.
        assert_eq!(COLORADO_FULL_PENALTY_CENTS, 50 * COLORADO_CURED_PENALTY_CENTS);
    }

    #[test]
    fn validity_window_30_days_constant_invariant() {
        assert_eq!(REPORT_VALIDITY_WINDOW_DAYS, 30);
        assert_eq!(COLORADO_CURE_WINDOW_DAYS, 7);
    }

    #[test]
    fn only_colorado_has_mandatory_acceptance_invariant() {
        // CO portability_required = true by default; WA depends on disclosure; Default false.
        let co = check(&input(Regime::Colorado));
        let wa_opt_in = check(&input(Regime::Washington));
        let mut wa_opt_out = input(Regime::Washington);
        wa_opt_out.landlord_website_disclosure_accepts_report = false;
        let wa_opt_out_r = check(&wa_opt_out);
        let de = check(&input(Regime::Default));

        assert!(co.portability_required);
        assert!(wa_opt_in.portability_required); // because disclosed accept
        assert!(!wa_opt_out_r.portability_required); // because disclosed not-accept
        assert!(!de.portability_required);
    }

    #[test]
    fn citation_pins_all_regime_authorities() {
        let r = check(&input(Regime::Colorado));
        assert!(r.citation.contains("§ 38-12-902"));
        assert!(r.citation.contains("§ 38-12-904"));
        assert!(r.citation.contains("§ 38-12-904(4)"));
        assert!(r.citation.contains("RCW 59.18.257"));
        assert!(r.citation.contains("RCW 59.18.257(1)"));
        assert!(r.citation.contains("RCW 59.18.257(2)"));
        assert!(r.citation.contains("15 U.S.C. § 1681"));
        assert!(r.citation.contains("HB23-1099"));
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input(Regime::Colorado));
        assert!(
            r.notes.iter().any(|n| n.contains("application_fees")
                && n.contains("adverse_action_notice")
                && n.contains("Fair Credit Reporting Act")),
            "sibling distinction note must reference application_fees + adverse_action_notice + FCRA"
        );
    }

    #[test]
    fn defensive_negative_report_age_no_validity() {
        let mut b = input(Regime::Colorado);
        b.report_age_days = -5;
        let r = check(&b);
        assert!(!r.report_within_validity_window);
    }

    #[test]
    fn validity_boundary_30_days_exact_within_window() {
        let mut b = input(Regime::Colorado);
        b.report_age_days = 30;
        let r = check(&b);
        assert!(r.report_within_validity_window);
    }

    #[test]
    fn missing_crtsr_components_truth_table() {
        // Each missing component → report fails completeness.
        for missing_component in 0..5 {
            let mut b = input(Regime::Washington);
            match missing_component {
                0 => b.report_includes_credit_history = false,
                1 => b.report_includes_criminal_history = false,
                2 => b.report_includes_eviction_history = false,
                3 => b.report_includes_employment_verification = false,
                _ => b.report_includes_rental_history = false,
            }
            let r = check(&b);
            assert!(
                !r.report_meets_requirements,
                "missing component {} should fail",
                missing_component
            );
        }
    }
}
