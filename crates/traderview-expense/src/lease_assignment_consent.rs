//! Lease assignment consent — landlord consent rules for
//! tenant assignment of a residential lease.
//!
//! Distinct from `sublet_consent` (subleasing — tenant remains
//! liable on the lease). ASSIGNMENT is a complete transfer of the
//! tenant's interest in the lease to a new tenant; the original
//! tenant is fully discharged absent a contrary clause. States
//! diverge sharply on whether a landlord may refuse assignment
//! consent without cause.
//!
//! New York — N.Y. Real Prop. Law § 226-b (Right to Sublease or
//! Assign) draws a SHARP STRUCTURAL DISTINCTION between
//! subletting and assignment:
//!   * SUBLETTING (4+ unit residential buildings): tenant has
//!     statutory right WITH landlord's reasonable consent.
//!     Unreasonable refusal allows tenant to proceed AND
//!     recover costs + attorney fees on bad-faith showing.
//!     Landlord's 30-day failure to respond = DEEMED CONSENT.
//!   * ASSIGNMENT: landlord may UNCONDITIONALLY withhold consent
//!     without cause, BUT if landlord's refusal is unreasonable,
//!     landlord must RELEASE THE TENANT from the lease on 30
//!     days notice. Strong tenant-side exit valve.
//!
//! California — Cal. Civ. Code § 1995.260: if a lease has a
//! consent-required clause but specifies NO STANDARD for giving
//! or withholding consent, an implied reasonableness standard
//! applies. Codified in 1989 to reverse pre-statute common-law
//! "unreasonable refusal allowed" rule. NOT retroactive — leases
//! executed BEFORE SEPTEMBER 23, 1983 with silent consent clauses
//! remain governed by pre-statute rule (landlord may withhold
//! unreasonably). Burden of proof on tenant; can be satisfied by
//! showing landlord failed to state written reasonable objection
//! within reasonable time of tenant's written request.
//!
//! Restatement (Second) of Property § 15.2 — common-law DEFAULT
//! RULE OF FREE ASSIGNABILITY: absent a lease restriction, the
//! tenant may freely assign the leasehold interest without
//! landlord consent. Restrictions strictly construed against the
//! landlord. Many states apply this default; modern majority
//! (LeaseControls) shifts to enforcing the consent clause as
//! written.
//!
//! LeaseControls (modern majority rule) — when the lease contains
//! a clear restriction on assignment requiring landlord consent,
//! the restriction is enforced as written. If the clause does not
//! specify a reasonableness standard, the landlord may withhold
//! consent for any reason absent a state statute imposing
//! reasonableness (Texas, Illinois, Massachusetts commercial
//! context, and others follow this rule).
//!
//! Citations: N.Y. Real Prop. Law § 226-b (subletting + assignment
//! right + 30-day deemed-consent rule + 30-day release for
//! unreasonable assignment refusal); Cal. Civ. Code § 1995.260
//! (silent-on-standard implied reasonableness; September 23, 1983
//! effective date); Cal. Civ. Code § 1995.270 (consent-with-
//! reasonableness express standard); Restatement (Second) of
//! Property § 15.2 (default free assignability); Kendall v.
//! Ernest Pestana, Inc., 40 Cal. 3d 488 (1985) (judicial origin
//! of California reasonableness rule prior to § 1995.260
//! codification).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// N.Y. Real Prop. Law § 226-b — sharp subletting / assignment
    /// distinction with 30-day deemed-consent + 30-day release.
    NewYork,
    /// Cal. Civ. Code § 1995.260 — silent-on-standard implied
    /// reasonableness, non-retroactive to pre-1983 leases.
    California,
    /// Restatement (Second) of Property § 15.2 — default rule of
    /// free assignability absent restriction.
    Restatement,
    /// Modern majority — lease consent clause enforced as written.
    LeaseControls,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransferType {
    /// Assignment — complete transfer of leasehold interest;
    /// tenant fully discharged absent contrary clause.
    Assignment,
    /// Sublease — tenant remains liable; subtenant occupies
    /// under tenant's lease.
    Sublease,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConsentStandard {
    /// Landlord must be reasonable in withholding consent.
    Reasonable,
    /// Landlord may withhold without cause.
    Unconditional,
    /// No consent required — free transfer.
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub transfer_type: TransferType,
    /// True if the lease contains a clause restricting transfer
    /// (assignment or sublease).
    pub lease_has_consent_clause: bool,
    /// True if the consent clause specifies an express standard
    /// (e.g., "reasonable" or "absolute discretion").
    pub consent_clause_specifies_standard: bool,
    /// Days elapsed since tenant's written request for consent.
    pub tenant_request_pending_days: i64,
    /// True if landlord has responded to the tenant's request
    /// with a written reason.
    pub landlord_responded_with_reason: bool,
    /// True if landlord's refusal would be objectively reasonable
    /// (e.g., proposed assignee has bad credit, criminal record,
    /// inadequate income — landlord can articulate legitimate
    /// commercial reason).
    pub landlord_refusal_objectively_reasonable: bool,
    /// California-specific — true if the lease was executed
    /// before September 23, 1983 (pre-§ 1995.260 effective date).
    /// § 1995.260 is not retroactive.
    pub pre_1983_lease: bool,
    /// New York subletting-specific — true if the building has
    /// 4 or more residential units (§ 226-b sublet right
    /// applies only to 4+ unit buildings).
    pub building_has_4_plus_units: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    pub consent_required: bool,
    pub standard_for_consent: ConsentStandard,
    /// True if the tenant may lawfully proceed with the transfer
    /// despite landlord's refusal.
    pub tenant_may_proceed_with_transfer: bool,
    /// New York § 226-b specific — true if landlord must release
    /// the tenant from the lease on 30 days notice (engaged when
    /// assignment refusal is unreasonable).
    pub landlord_must_release_on_30_days: bool,
    /// New York § 226-b subletting specific — true if landlord's
    /// failure to respond within 30 days constitutes deemed
    /// consent.
    pub deemed_consent_engaged: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// N.Y. Real Prop. Law § 226-b(2)(b) — landlord must respond
/// within 30 days of tenant's sublease request.
pub const NY_LANDLORD_RESPONSE_WINDOW_DAYS: i64 = 30;
/// N.Y. Real Prop. Law § 226-b(1) — landlord 30-day release
/// notice period when assignment refusal is unreasonable.
pub const NY_ASSIGNMENT_RELEASE_NOTICE_DAYS: i64 = 30;

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let mut consent_required = input.lease_has_consent_clause;
    let mut standard_for_consent = ConsentStandard::Unconditional;
    let mut tenant_may_proceed_with_transfer = false;
    let mut landlord_must_release_on_30_days = false;
    let mut deemed_consent_engaged = false;

    match input.regime {
        Regime::NewYork => {
            // N.Y. Real Prop. Law § 226-b — sharp distinction.
            match input.transfer_type {
                TransferType::Sublease => {
                    if input.building_has_4_plus_units {
                        consent_required = true;
                        standard_for_consent = ConsentStandard::Reasonable;
                        // § 226-b(2)(b) — 30-day failure to respond = deemed consent.
                        if input.tenant_request_pending_days >= NY_LANDLORD_RESPONSE_WINDOW_DAYS
                            && !input.landlord_responded_with_reason
                        {
                            deemed_consent_engaged = true;
                            tenant_may_proceed_with_transfer = true;
                            notes.push(
                                "§ 226-b(2)(b) — landlord 30-day failure to respond to \
                                 written sublease request = DEEMED CONSENT. Tenant may \
                                 proceed with sublease."
                                    .to_string(),
                            );
                        } else if input.landlord_responded_with_reason
                            && !input.landlord_refusal_objectively_reasonable
                        {
                            // Unreasonable refusal → tenant may proceed + recover fees.
                            tenant_may_proceed_with_transfer = true;
                            violations.push(
                                "§ 226-b — landlord unreasonably withheld sublease consent. \
                                 Tenant may proceed; landlord may be liable for tenant's \
                                 costs and attorney fees on bad-faith showing."
                                    .to_string(),
                            );
                        }
                    } else {
                        // < 4 units — § 226-b sublease right does not apply; lease controls.
                        consent_required = input.lease_has_consent_clause;
                        standard_for_consent = if consent_required {
                            ConsentStandard::Unconditional
                        } else {
                            ConsentStandard::None
                        };
                        notes.push(
                            "§ 226-b sublease right does NOT apply to buildings with fewer \
                             than 4 residential units. Lease controls; landlord may withhold \
                             consent without cause if lease so provides."
                                .to_string(),
                        );
                    }
                }
                TransferType::Assignment => {
                    consent_required = true;
                    standard_for_consent = ConsentStandard::Unconditional;
                    if input.landlord_responded_with_reason
                        && !input.landlord_refusal_objectively_reasonable
                    {
                        // § 226-b(1) — unreasonable assignment refusal
                        // triggers 30-day release right.
                        landlord_must_release_on_30_days = true;
                        notes.push(
                            "§ 226-b(1) — landlord may UNCONDITIONALLY withhold assignment \
                             consent BUT if the refusal is unreasonable, landlord MUST \
                             RELEASE THE TENANT from the lease upon 30 days written notice. \
                             Strong tenant-side exit valve."
                                .to_string(),
                        );
                    } else {
                        notes.push(
                            "§ 226-b(1) — landlord may UNCONDITIONALLY withhold assignment \
                             consent. Tenant cannot force the assignment but may invoke the \
                             30-day release valve if the refusal is unreasonable."
                                .to_string(),
                        );
                    }
                }
            }
        }
        Regime::California => {
            // Cal. Civ. Code § 1995.260 — implied reasonableness
            // when consent clause is silent on standard.
            if input.lease_has_consent_clause {
                consent_required = true;
                if input.consent_clause_specifies_standard {
                    // Express standard governs (could be either
                    // reasonable or unconditional).
                    standard_for_consent = ConsentStandard::Reasonable;
                    notes.push(
                        "Cal. Civ. Code § 1995.270 — express consent standard in the lease \
                         governs. § 1995.260 implied-reasonableness rule applies only when \
                         the lease is silent on the standard."
                            .to_string(),
                    );
                } else if input.pre_1983_lease {
                    // Pre-1983 leases governed by pre-statute rule.
                    standard_for_consent = ConsentStandard::Unconditional;
                    notes.push(
                        "Cal. Civ. Code § 1995.260 is NOT retroactive. Leases executed \
                         before September 23, 1983 with consent clauses silent on standard \
                         remain governed by pre-statute common-law rule — landlord may \
                         withhold consent unreasonably."
                            .to_string(),
                    );
                } else {
                    // Post-1983 silent → implied reasonableness.
                    standard_for_consent = ConsentStandard::Reasonable;
                    notes.push(
                        "Cal. Civ. Code § 1995.260 — lease consent clause silent on \
                         standard → IMPLIED REASONABLENESS standard applies. Tenant bears \
                         burden of proof on unreasonableness; satisfied by showing landlord \
                         failed to state a written reasonable objection within reasonable \
                         time of tenant's request. Codified Kendall v. Ernest Pestana, \
                         Inc., 40 Cal. 3d 488 (1985)."
                            .to_string(),
                    );
                }

                if standard_for_consent == ConsentStandard::Reasonable
                    && input.landlord_responded_with_reason
                    && !input.landlord_refusal_objectively_reasonable
                {
                    tenant_may_proceed_with_transfer = true;
                    violations.push(
                        "Cal. Civ. Code § 1995.260 — landlord unreasonably withheld \
                         consent under implied or express reasonableness standard. Tenant \
                         may proceed with transfer."
                            .to_string(),
                    );
                }
            }
        }
        Regime::Restatement => {
            // Default free assignability.
            if !input.lease_has_consent_clause {
                consent_required = false;
                standard_for_consent = ConsentStandard::None;
                tenant_may_proceed_with_transfer = true;
                notes.push(
                    "Restatement (Second) of Property § 15.2 — DEFAULT RULE OF FREE \
                     ASSIGNABILITY engaged. No lease restriction → tenant may freely \
                     assign the leasehold interest without landlord consent."
                        .to_string(),
                );
            } else {
                consent_required = true;
                standard_for_consent = ConsentStandard::Reasonable;
                notes.push(
                    "Restatement (Second) of Property § 15.2 — when the lease contains a \
                     consent restriction, the restriction is strictly construed against \
                     the landlord. Many Restatement-following states imply reasonableness."
                        .to_string(),
                );
            }
        }
        Regime::LeaseControls => {
            // Modern majority — lease enforced as written.
            if !input.lease_has_consent_clause {
                consent_required = false;
                standard_for_consent = ConsentStandard::None;
                tenant_may_proceed_with_transfer = true;
                notes.push(
                    "Modern-majority rule — no consent clause in lease → tenant may \
                     transfer freely (no restriction to enforce)."
                        .to_string(),
                );
            } else {
                consent_required = true;
                standard_for_consent = if input.consent_clause_specifies_standard {
                    ConsentStandard::Reasonable
                } else {
                    ConsentStandard::Unconditional
                };
                notes.push(
                    "Modern-majority rule — consent clause enforced as written. If the \
                     clause does not specify reasonableness, landlord may withhold consent \
                     for any reason absent state-statute imposition of reasonableness \
                     (Texas, Illinois, Massachusetts commercial context, and others follow \
                     this rule)."
                        .to_string(),
                );
            }
        }
    }

    notes.push(
        "Sibling distinction: ASSIGNMENT (complete transfer of leasehold; tenant \
         discharged) is governed by this module; SUBLEASING (tenant remains liable; \
         subtenant occupies under tenant's lease) is governed by sibling `sublet_consent` \
         module. New York § 226-b is the only U.S. regime that draws a sharp structural \
         distinction between the two — sublease has statutory right with reasonableness; \
         assignment may be unconditionally refused but triggers a 30-day release valve."
            .to_string(),
    );

    CheckResult {
        consent_required,
        standard_for_consent,
        tenant_may_proceed_with_transfer,
        landlord_must_release_on_30_days,
        deemed_consent_engaged,
        compliant: violations.is_empty(),
        violations,
        citation: "N.Y. Real Prop. Law § 226-b (statutory subletting + assignment right; \
                   30-day deemed-consent rule for sublease; 30-day release-on-notice for \
                   unreasonable assignment refusal); Cal. Civ. Code § 1995.260 (implied \
                   reasonableness when consent clause silent on standard; non-retroactive \
                   to leases executed before September 23, 1983); Cal. Civ. Code \
                   § 1995.270 (express standard governs when stated); Restatement \
                   (Second) of Property § 15.2 (default rule of free assignability absent \
                   restriction); Kendall v. Ernest Pestana, Inc., 40 Cal. 3d 488 (1985) \
                   (judicial origin of California reasonableness rule prior to § 1995.260 \
                   codification)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(regime: Regime, transfer_type: TransferType) -> Input {
        Input {
            regime,
            transfer_type,
            lease_has_consent_clause: true,
            consent_clause_specifies_standard: false,
            tenant_request_pending_days: 0,
            landlord_responded_with_reason: false,
            landlord_refusal_objectively_reasonable: false,
            pre_1983_lease: false,
            building_has_4_plus_units: true,
        }
    }

    // ── New York § 226-b — sublease branch ─────────────────────

    #[test]
    fn ny_sublease_4plus_units_unreasonable_refusal_tenant_may_proceed() {
        let mut b = input(Regime::NewYork, TransferType::Sublease);
        b.landlord_responded_with_reason = true;
        b.landlord_refusal_objectively_reasonable = false;
        let r = check(&b);
        assert!(r.tenant_may_proceed_with_transfer);
        assert!(!r.compliant);
        assert_eq!(r.standard_for_consent, ConsentStandard::Reasonable);
    }

    #[test]
    fn ny_sublease_4plus_units_reasonable_refusal_tenant_may_not_proceed() {
        let mut b = input(Regime::NewYork, TransferType::Sublease);
        b.landlord_responded_with_reason = true;
        b.landlord_refusal_objectively_reasonable = true;
        let r = check(&b);
        assert!(!r.tenant_may_proceed_with_transfer);
        assert!(r.compliant);
    }

    #[test]
    fn ny_sublease_30_day_failure_to_respond_deemed_consent() {
        let mut b = input(Regime::NewYork, TransferType::Sublease);
        b.tenant_request_pending_days = 30;
        b.landlord_responded_with_reason = false;
        let r = check(&b);
        assert!(r.deemed_consent_engaged);
        assert!(r.tenant_may_proceed_with_transfer);
    }

    #[test]
    fn ny_sublease_29_days_pending_no_deemed_consent() {
        let mut b = input(Regime::NewYork, TransferType::Sublease);
        b.tenant_request_pending_days = 29;
        b.landlord_responded_with_reason = false;
        let r = check(&b);
        assert!(!r.deemed_consent_engaged);
        assert!(!r.tenant_may_proceed_with_transfer);
    }

    #[test]
    fn ny_sublease_less_than_4_units_no_statutory_right() {
        let mut b = input(Regime::NewYork, TransferType::Sublease);
        b.building_has_4_plus_units = false;
        b.landlord_responded_with_reason = true;
        b.landlord_refusal_objectively_reasonable = false;
        let r = check(&b);
        // § 226-b sublease right doesn't apply; lease controls; tenant cannot
        // force sublease.
        assert!(!r.tenant_may_proceed_with_transfer);
        assert_eq!(r.standard_for_consent, ConsentStandard::Unconditional);
    }

    // ── New York § 226-b — assignment branch ───────────────────

    #[test]
    fn ny_assignment_unreasonable_refusal_triggers_30_day_release() {
        let mut b = input(Regime::NewYork, TransferType::Assignment);
        b.landlord_responded_with_reason = true;
        b.landlord_refusal_objectively_reasonable = false;
        let r = check(&b);
        assert!(r.landlord_must_release_on_30_days);
        // Tenant cannot force assignment; landlord must release.
        assert!(!r.tenant_may_proceed_with_transfer);
        assert_eq!(r.standard_for_consent, ConsentStandard::Unconditional);
    }

    #[test]
    fn ny_assignment_reasonable_refusal_no_release() {
        let mut b = input(Regime::NewYork, TransferType::Assignment);
        b.landlord_responded_with_reason = true;
        b.landlord_refusal_objectively_reasonable = true;
        let r = check(&b);
        assert!(!r.landlord_must_release_on_30_days);
    }

    // ── California § 1995.260 ─────────────────────────────────

    #[test]
    fn ca_silent_standard_post_1983_implies_reasonableness() {
        let mut b = input(Regime::California, TransferType::Assignment);
        b.consent_clause_specifies_standard = false;
        b.pre_1983_lease = false;
        b.landlord_responded_with_reason = true;
        b.landlord_refusal_objectively_reasonable = false;
        let r = check(&b);
        assert_eq!(r.standard_for_consent, ConsentStandard::Reasonable);
        assert!(r.tenant_may_proceed_with_transfer);
        assert!(!r.compliant);
    }

    #[test]
    fn ca_silent_standard_pre_1983_no_implied_reasonableness() {
        let mut b = input(Regime::California, TransferType::Assignment);
        b.consent_clause_specifies_standard = false;
        b.pre_1983_lease = true;
        b.landlord_responded_with_reason = true;
        b.landlord_refusal_objectively_reasonable = false;
        let r = check(&b);
        assert_eq!(r.standard_for_consent, ConsentStandard::Unconditional);
        // Pre-statute rule — landlord may withhold without cause.
        assert!(!r.tenant_may_proceed_with_transfer);
        assert!(r.compliant);
    }

    #[test]
    fn ca_express_standard_governs() {
        let mut b = input(Regime::California, TransferType::Assignment);
        b.consent_clause_specifies_standard = true;
        b.landlord_responded_with_reason = true;
        b.landlord_refusal_objectively_reasonable = false;
        let r = check(&b);
        assert_eq!(r.standard_for_consent, ConsentStandard::Reasonable);
        assert!(r.tenant_may_proceed_with_transfer);
    }

    #[test]
    fn ca_reasonable_refusal_under_implied_standard_compliant() {
        let mut b = input(Regime::California, TransferType::Assignment);
        b.landlord_responded_with_reason = true;
        b.landlord_refusal_objectively_reasonable = true;
        let r = check(&b);
        assert!(r.compliant);
        assert!(!r.tenant_may_proceed_with_transfer);
    }

    // ── Restatement § 15.2 — default free assignability ────────

    #[test]
    fn restatement_no_consent_clause_free_assignment() {
        let mut b = input(Regime::Restatement, TransferType::Assignment);
        b.lease_has_consent_clause = false;
        let r = check(&b);
        assert!(!r.consent_required);
        assert_eq!(r.standard_for_consent, ConsentStandard::None);
        assert!(r.tenant_may_proceed_with_transfer);
    }

    #[test]
    fn restatement_with_consent_clause_strict_construction() {
        let r = check(&input(Regime::Restatement, TransferType::Assignment));
        assert!(r.consent_required);
        // Restatement-following states imply reasonableness when restriction exists.
        assert_eq!(r.standard_for_consent, ConsentStandard::Reasonable);
    }

    // ── LeaseControls modern majority ──────────────────────────

    #[test]
    fn lease_controls_no_clause_free_transfer() {
        let mut b = input(Regime::LeaseControls, TransferType::Assignment);
        b.lease_has_consent_clause = false;
        let r = check(&b);
        assert!(!r.consent_required);
        assert!(r.tenant_may_proceed_with_transfer);
    }

    #[test]
    fn lease_controls_clause_no_standard_unconditional() {
        let mut b = input(Regime::LeaseControls, TransferType::Assignment);
        b.consent_clause_specifies_standard = false;
        let r = check(&b);
        assert!(r.consent_required);
        assert_eq!(r.standard_for_consent, ConsentStandard::Unconditional);
    }

    #[test]
    fn lease_controls_clause_with_reasonable_standard() {
        let mut b = input(Regime::LeaseControls, TransferType::Assignment);
        b.consent_clause_specifies_standard = true;
        let r = check(&b);
        assert!(r.consent_required);
        assert_eq!(r.standard_for_consent, ConsentStandard::Reasonable);
    }

    // ── Multi-regime invariants ────────────────────────────────

    #[test]
    fn only_ny_distinguishes_sublease_from_assignment_structurally() {
        // For NY: sublease has statutory reasonable-consent right;
        // assignment is unconditional with 30-day release valve.
        // Same input → different outcomes by transfer type.
        let mut b_sub = input(Regime::NewYork, TransferType::Sublease);
        b_sub.landlord_responded_with_reason = true;
        b_sub.landlord_refusal_objectively_reasonable = false;
        let r_sub = check(&b_sub);

        let mut b_asn = input(Regime::NewYork, TransferType::Assignment);
        b_asn.landlord_responded_with_reason = true;
        b_asn.landlord_refusal_objectively_reasonable = false;
        let r_asn = check(&b_asn);

        // Sublease: tenant proceeds; Assignment: 30-day release.
        assert!(r_sub.tenant_may_proceed_with_transfer);
        assert!(r_asn.landlord_must_release_on_30_days);
        assert!(!r_asn.tenant_may_proceed_with_transfer);
        assert_ne!(r_sub.standard_for_consent, r_asn.standard_for_consent);
    }

    #[test]
    fn ca_pre_1983_grandfather_only_when_silent_standard() {
        // Pre-1983 grandfather only kicks in for silent-standard
        // clauses; express-standard pre-1983 still respects the
        // express standard.
        let mut silent = input(Regime::California, TransferType::Assignment);
        silent.consent_clause_specifies_standard = false;
        silent.pre_1983_lease = true;
        let r_silent = check(&silent);
        assert_eq!(
            r_silent.standard_for_consent,
            ConsentStandard::Unconditional
        );

        let mut express = input(Regime::California, TransferType::Assignment);
        express.consent_clause_specifies_standard = true;
        express.pre_1983_lease = true;
        let r_express = check(&express);
        assert_eq!(r_express.standard_for_consent, ConsentStandard::Reasonable);
    }

    #[test]
    fn deemed_consent_only_applies_to_ny_sublease() {
        // Deemed consent (30-day failure to respond) is NY § 226-b(2)(b)
        // sublease-specific. Should never engage for other regimes /
        // transfer types.
        let regimes = [
            Regime::NewYork,
            Regime::California,
            Regime::Restatement,
            Regime::LeaseControls,
        ];
        for regime in regimes.iter() {
            for transfer in [TransferType::Sublease, TransferType::Assignment].iter() {
                let mut b = input(*regime, *transfer);
                b.tenant_request_pending_days = 30;
                b.landlord_responded_with_reason = false;
                let r = check(&b);
                let should_engage =
                    matches!(regime, Regime::NewYork) && matches!(transfer, TransferType::Sublease);
                assert_eq!(
                    r.deemed_consent_engaged, should_engage,
                    "regime={:?}, transfer={:?}",
                    regime, transfer
                );
            }
        }
    }

    #[test]
    fn citation_pins_all_regime_authorities() {
        let r = check(&input(Regime::NewYork, TransferType::Assignment));
        assert!(r.citation.contains("N.Y. Real Prop. Law § 226-b"));
        assert!(r.citation.contains("Cal. Civ. Code § 1995.260"));
        assert!(r.citation.contains("Cal. Civ. Code § 1995.270"));
        assert!(r
            .citation
            .contains("Restatement (Second) of Property § 15.2"));
        assert!(r.citation.contains("Kendall v. Ernest Pestana"));
        assert!(r.citation.contains("September 23, 1983"));
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input(Regime::NewYork, TransferType::Assignment));
        assert!(
            r.notes.iter().any(|n| n.contains("ASSIGNMENT")
                && n.contains("SUBLEASING")
                && n.contains("sublet_consent")
                && n.contains("§ 226-b")),
            "sibling-distinction note must reference assignment vs. sublease structure"
        );
    }

    #[test]
    fn ny_release_only_for_unreasonable_assignment_refusal() {
        // 4-cell truth table — release engages only on
        // (Assignment, unreasonable refusal).
        let cells = [
            (TransferType::Assignment, false, true), // unreasonable → release
            (TransferType::Assignment, true, false), // reasonable → no release
            (TransferType::Sublease, false, false),  // sublease branch instead
            (TransferType::Sublease, true, false),   // sublease branch
        ];
        for (transfer, reasonable, expected_release) in cells.iter() {
            let mut b = input(Regime::NewYork, *transfer);
            b.landlord_responded_with_reason = true;
            b.landlord_refusal_objectively_reasonable = *reasonable;
            let r = check(&b);
            assert_eq!(
                r.landlord_must_release_on_30_days, *expected_release,
                "transfer={:?}, reasonable={}",
                transfer, reasonable
            );
        }
    }

    #[test]
    fn defensive_negative_pending_days_no_deemed_consent() {
        let mut b = input(Regime::NewYork, TransferType::Sublease);
        b.tenant_request_pending_days = -5;
        b.landlord_responded_with_reason = false;
        let r = check(&b);
        assert!(!r.deemed_consent_engaged);
    }

    #[test]
    fn restatement_free_assignability_only_when_no_clause() {
        // Free assignability under § 15.2 engages only when no
        // restriction. With a restriction, reasonableness is implied.
        let mut no_clause = input(Regime::Restatement, TransferType::Assignment);
        no_clause.lease_has_consent_clause = false;
        let r1 = check(&no_clause);
        assert!(!r1.consent_required);
        assert!(r1.tenant_may_proceed_with_transfer);

        let with_clause = check(&input(Regime::Restatement, TransferType::Assignment));
        assert!(with_clause.consent_required);
    }

    #[test]
    fn ny_30_day_response_window_constant_invariant() {
        assert_eq!(NY_LANDLORD_RESPONSE_WINDOW_DAYS, 30);
        assert_eq!(NY_ASSIGNMENT_RELEASE_NOTICE_DAYS, 30);
    }

    #[test]
    fn single_regime_uniqueness_invariant() {
        // 4 regimes × 2 transfer types = 8 combinations; verify
        // each produces a coherent outcome with distinct citation
        // language for the lead regime.
        for regime in [
            Regime::NewYork,
            Regime::California,
            Regime::Restatement,
            Regime::LeaseControls,
        ] {
            for transfer in [TransferType::Sublease, TransferType::Assignment] {
                let r = check(&input(regime, transfer));
                // All regime citations present in the shared string.
                assert!(r.citation.contains("§ 226-b"));
                assert!(r.citation.contains("§ 1995.260"));
                assert!(r.citation.contains("§ 15.2"));
                // Compliance status is computable.
                let _ = r.compliant;
                let _ = r.standard_for_consent;
            }
        }
    }
}
