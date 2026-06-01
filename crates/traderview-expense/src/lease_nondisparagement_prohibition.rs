//! Lease non-disparagement clause prohibition under the federal
//! Consumer Review Fairness Act of 2016 (15 U.S.C. § 45b) plus
//! state attorney-general enforcement.
//!
//! Distinct from sibling module `lease_waiver_enforceability`
//! (general lease-waiver provisions) and `landlord_retaliation_
//! damages` (retaliation against tenant exercising rights). This
//! module focuses on a NARROW PROHIBITION — landlord-drafted
//! clauses in form (adhesion) lease contracts that gag the
//! tenant from posting honest reviews of the landlord/property
//! on Yelp, Google, Apartments.com, Reddit, etc. The CRFA
//! creates a federal floor; states may concurrently enforce.
//!
//! Trader-critical for landlord-investors using template lease
//! agreements — broad non-disparagement language is VOID FROM
//! INCEPTION and offering a lease containing such provisions is
//! itself an FTC violation actionable as an unfair or deceptive
//! act under § 5 of the FTC Act.
//!
//! 15 U.S.C. § 45b(a) DEFINITIONS:
//!   - "Covered communication" = written, oral, or pictorial
//!     review, performance assessment, or other similar analysis
//!     (including by electronic means) of the goods, services,
//!     or conduct of a person by an individual who is a party to
//!     a form contract.
//!   - "Form contract" = contract with standardized terms used
//!     in the course of selling or leasing goods or services and
//!     imposed on an individual without meaningful opportunity
//!     to negotiate the standardized terms (typical adhesion
//!     contract — covers most residential leases).
//!
//! 15 U.S.C. § 45b(b) VOID PROVISIONS — any form contract
//! provision is VOID FROM INCEPTION if it:
//!   (1) Prohibits or restricts the individual's ability to
//!       engage in a covered communication;
//!   (2) Imposes a penalty or fee against an individual for
//!       engaging in a covered communication; OR
//!   (3) Transfers or requires transfer of intellectual property
//!       rights in review or feedback content.
//!
//! 15 U.S.C. § 45b(c) EXCEPTIONS — § 45b(b) does NOT apply to
//! provisions that restrict:
//!   (1) Legally actionable content (defamation, harassment,
//!       knowingly false statements);
//!   (2) Trade secrets / personal information under existing
//!       law;
//!   (3) Content related to medical records (HIPAA);
//!   (4) Content unlawful under existing law (e.g., child
//!       pornography, true threats).
//!
//! 15 U.S.C. § 45b(d) PROHIBITION — unlawful to OFFER a form
//! contract containing a void provision (i.e., the offering
//! itself is unlawful, not just enforcement).
//!
//! 15 U.S.C. § 45b(e) ENFORCEMENT:
//!   - FTC enforces under § 5 of FTC Act (unfair or deceptive
//!     act);
//!   - State Attorneys General may bring civil action in
//!     federal district court for injunctive relief, damages,
//!     restitution, civil penalties, or other equitable relief.
//!
//! Citations: 15 U.S.C. § 45b (Consumer Review Fairness Act of
//! 2016 — Pub. L. 114-258, December 14, 2016); 15 U.S.C.
//! § 45b(a) (definitions of covered communication + form
//! contract); 15 U.S.C. § 45b(b)(1) (prohibition-or-restriction
//! void); 15 U.S.C. § 45b(b)(2) (penalty-or-fee void); 15 U.S.C.
//! § 45b(b)(3) (IP-transfer void); 15 U.S.C. § 45b(c) (four
//! exceptions — legally actionable, trade secret, medical,
//! unlawful); 15 U.S.C. § 45b(d) (offering void provision is
//! itself unlawful); 15 U.S.C. § 45b(e) (FTC + state AG
//! enforcement); 15 U.S.C. § 45 (FTC Act § 5 — unfair or
//! deceptive acts); FTC Bulletin 2022-05 (Unfair and Deceptive
//! Acts or Practices That Impede Consumer Reviews); Pub. L.
//! 114-258 (CRFA enactment).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    /// True if the lease is a form/adhesion contract (no
    /// meaningful opportunity to negotiate standardized terms).
    /// Required for CRFA to apply.
    pub lease_is_form_contract: bool,
    /// True if lease contains a clause prohibiting or
    /// restricting tenant from publishing reviews (Yelp, Google,
    /// Apartments.com, etc.).
    pub non_disparagement_clause_present: bool,
    /// True if lease imposes a penalty or fee on tenant for
    /// publishing a review.
    pub penalty_or_fee_for_review_clause_present: bool,
    /// True if lease transfers (or requires transfer of)
    /// intellectual property rights in the tenant's review or
    /// feedback content to landlord.
    pub ip_transfer_clause_present: bool,
    /// § 45b(c)(1) — true if challenged content would be
    /// legally actionable (defamation, harassment, knowingly
    /// false statements) and the clause restricts only such
    /// content.
    pub exception_legally_actionable_content: bool,
    /// § 45b(c)(2) — true if clause restricts trade secrets or
    /// personal information protected under existing law.
    pub exception_trade_secret_or_personal_info: bool,
    /// § 45b(c)(3) — true if clause restricts medical-record
    /// content (HIPAA).
    pub exception_medical_record_content: bool,
    /// § 45b(c)(4) — true if clause restricts content otherwise
    /// unlawful (child pornography, true threats, etc.).
    pub exception_unlawful_content: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if § 45b(b)(1) non-disparagement clause is void
    /// (form contract + clause present + no exception).
    pub non_disparagement_clause_void: bool,
    /// True if § 45b(b)(2) penalty/fee clause is void.
    pub penalty_clause_void: bool,
    /// True if § 45b(b)(3) IP-transfer clause is void.
    pub ip_transfer_clause_void: bool,
    /// True if any § 45b(c) exception engages (preserves the
    /// challenged clause from CRFA voiding).
    pub exception_engaged: bool,
    /// True if § 45b(d) prohibition engages — offering the
    /// lease containing the void provision is itself unlawful.
    pub offering_lease_unlawful: bool,
    /// True if FTC enforcement is available under § 45b(e).
    pub ftc_enforcement_available: bool,
    /// True if state AG enforcement is available under § 45b(e).
    pub state_ag_enforcement_available: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    // CRFA applies only to form (adhesion) contracts. Negotiated
    // contracts with meaningful opportunity to bargain over
    // standardized terms are outside the statute's scope.
    if !input.lease_is_form_contract {
        notes.push(
            "15 U.S.C. § 45b(a) — CRFA applies only to FORM CONTRACTS (adhesion contracts \
             with standardized terms imposed without meaningful opportunity to \
             negotiate). This lease was identified as negotiated; CRFA prohibition does \
             NOT apply. Non-disparagement clauses in genuinely negotiated leases remain \
             subject to common-law contract enforcement and lease_waiver_enforceability \
             sibling analysis."
                .to_string(),
        );
        return CheckResult {
            non_disparagement_clause_void: false,
            penalty_clause_void: false,
            ip_transfer_clause_void: false,
            exception_engaged: false,
            offering_lease_unlawful: false,
            ftc_enforcement_available: false,
            state_ag_enforcement_available: false,
            compliant: true,
            violations,
            citation: citation_text(),
            notes,
        };
    }

    // § 45b(c) exception evaluation — any exception preserves
    // the challenged clause from voiding.
    let exception_engaged = input.exception_legally_actionable_content
        || input.exception_trade_secret_or_personal_info
        || input.exception_medical_record_content
        || input.exception_unlawful_content;

    let non_disparagement_clause_void =
        input.non_disparagement_clause_present && !exception_engaged;
    let penalty_clause_void =
        input.penalty_or_fee_for_review_clause_present && !exception_engaged;
    let ip_transfer_clause_void =
        input.ip_transfer_clause_present && !exception_engaged;

    let any_void_clause = non_disparagement_clause_void
        || penalty_clause_void
        || ip_transfer_clause_void;

    let offering_lease_unlawful = any_void_clause;
    let ftc_enforcement_available = any_void_clause;
    let state_ag_enforcement_available = any_void_clause;

    if non_disparagement_clause_void {
        violations.push(
            "15 U.S.C. § 45b(b)(1) — non-disparagement clause is VOID FROM INCEPTION. \
             Form-contract provision prohibiting/restricting tenant's ability to engage \
             in covered communications (online reviews, performance assessments) about \
             landlord/property is void as a matter of federal law. No state-law \
             enforcement of this provision is possible."
                .to_string(),
        );
    }

    if penalty_clause_void {
        violations.push(
            "15 U.S.C. § 45b(b)(2) — penalty-or-fee clause for tenant review is VOID FROM \
             INCEPTION. Any provision imposing a financial penalty on tenant for posting \
             a review is unenforceable; landlord may not seek to collect such fees."
                .to_string(),
        );
    }

    if ip_transfer_clause_void {
        violations.push(
            "15 U.S.C. § 45b(b)(3) — intellectual property transfer clause is VOID FROM \
             INCEPTION. Provision transferring (or requiring transfer of) IP rights in \
             tenant's review or feedback content is unenforceable. Tenant retains all IP \
             rights in their own reviews."
                .to_string(),
        );
    }

    if offering_lease_unlawful {
        violations.push(
            "15 U.S.C. § 45b(d) — UNLAWFUL TO OFFER a form contract containing a void \
             provision. The offering itself violates federal law, separate from any \
             attempt to enforce. Landlord exposed to FTC enforcement under § 5 of FTC \
             Act (unfair or deceptive acts) and state attorney general civil action \
             under § 45b(e)."
                .to_string(),
        );
    }

    // § 45b(c) exception notes.
    if exception_engaged
        && (input.non_disparagement_clause_present
            || input.penalty_or_fee_for_review_clause_present
            || input.ip_transfer_clause_present)
    {
        let mut engaged_exceptions: Vec<&str> = Vec::new();
        if input.exception_legally_actionable_content {
            engaged_exceptions.push("§ 45b(c)(1) legally actionable content (defamation, harassment, knowingly false)");
        }
        if input.exception_trade_secret_or_personal_info {
            engaged_exceptions.push("§ 45b(c)(2) trade secret / personal information");
        }
        if input.exception_medical_record_content {
            engaged_exceptions.push("§ 45b(c)(3) medical-record content (HIPAA)");
        }
        if input.exception_unlawful_content {
            engaged_exceptions.push("§ 45b(c)(4) otherwise unlawful content");
        }
        notes.push(format!(
            "§ 45b(c) EXCEPTION engaged — clause restricts only carved-out content \
             categories; not voided by CRFA. Exception(s): {}. Note: exceptions are \
             NARROWLY construed; a broad non-disparagement clause that purports to \
             restrict only actionable content but in practice chills lawful reviews \
             will likely be voided as too broad.",
            engaged_exceptions.join("; "),
        ));
    }

    if !input.non_disparagement_clause_present
        && !input.penalty_or_fee_for_review_clause_present
        && !input.ip_transfer_clause_present
    {
        notes.push(
            "Lease contains no provision in any of the three § 45b(b) categories \
             (non-disparagement, penalty/fee, IP transfer). CRFA review compliance \
             baseline satisfied."
                .to_string(),
        );
    }

    notes.push(
        "FTC enforces CRFA under § 5 of the FTC Act (15 U.S.C. § 45) as an unfair or \
         deceptive act. State attorneys general have CONCURRENT enforcement authority \
         under § 45b(e) and may bring federal-district-court civil action for injunctive \
         relief, damages, restitution, civil penalties, or other equitable relief. FTC \
         Bulletin 2022-05 (March 2022) provides interpretive guidance on unfair and \
         deceptive practices impeding consumer reviews."
            .to_string(),
    );

    notes.push(
        "Sibling distinction: this module covers FEDERAL CRFA non-disparagement \
         prohibition. Related modules: `lease_waiver_enforceability` (general lease \
         waiver provisions — N.Y. GOL § 5-321 + Cal. Civ. Code § 1953 + common law); \
         `landlord_retaliation_damages` (retaliation against tenant for exercising \
         rights — different cause of action, may overlap when landlord retaliates for \
         tenant review); `landlord_harassment` (affirmative-harassment civil penalty \
         statutes). CRFA creates a federal FLOOR — state-law remedies remain available \
         and may provide additional damages."
            .to_string(),
    );

    let compliant = violations.is_empty();

    CheckResult {
        non_disparagement_clause_void,
        penalty_clause_void,
        ip_transfer_clause_void,
        exception_engaged,
        offering_lease_unlawful,
        ftc_enforcement_available,
        state_ag_enforcement_available,
        compliant,
        violations,
        citation: citation_text(),
        notes,
    }
}

fn citation_text() -> &'static str {
    "15 U.S.C. § 45b (Consumer Review Fairness Act of 2016 — Pub. L. 114-258, Dec 14, \
     2016); 15 U.S.C. § 45b(a) (definitions — covered communication + form contract); \
     15 U.S.C. § 45b(b)(1) (prohibition-or-restriction void); 15 U.S.C. § 45b(b)(2) \
     (penalty-or-fee void); 15 U.S.C. § 45b(b)(3) (IP-transfer void); 15 U.S.C. § 45b(c) \
     (four exceptions — legally actionable, trade secret, medical, unlawful); 15 U.S.C. \
     § 45b(d) (offering void provision is itself unlawful); 15 U.S.C. § 45b(e) (FTC + \
     state AG enforcement); 15 U.S.C. § 45 (FTC Act § 5 — unfair or deceptive acts); \
     FTC Bulletin 2022-05 (March 2022 — Unfair and Deceptive Acts or Practices That \
     Impede Consumer Reviews); Pub. L. 114-258 (CRFA enactment, December 14, 2016)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Input {
        Input {
            lease_is_form_contract: true,
            non_disparagement_clause_present: false,
            penalty_or_fee_for_review_clause_present: false,
            ip_transfer_clause_present: false,
            exception_legally_actionable_content: false,
            exception_trade_secret_or_personal_info: false,
            exception_medical_record_content: false,
            exception_unlawful_content: false,
        }
    }

    // ── Form contract gating ─────────────────────────────────

    #[test]
    fn non_form_contract_crfa_does_not_apply() {
        let mut b = input();
        b.lease_is_form_contract = false;
        b.non_disparagement_clause_present = true;
        b.penalty_or_fee_for_review_clause_present = true;
        b.ip_transfer_clause_present = true;
        let r = check(&b);
        assert!(r.compliant);
        assert!(!r.non_disparagement_clause_void);
        assert!(!r.penalty_clause_void);
        assert!(!r.ip_transfer_clause_void);
        assert!(!r.ftc_enforcement_available);
        assert!(r.notes.iter().any(|n| n.contains("FORM CONTRACTS")));
    }

    #[test]
    fn form_contract_no_voiding_clauses_compliant() {
        let r = check(&input());
        assert!(r.compliant);
        assert!(!r.offering_lease_unlawful);
        assert!(r.notes.iter().any(|n| n.contains("baseline satisfied")));
    }

    // ── § 45b(b)(1) non-disparagement clause ─────────────────

    #[test]
    fn non_disparagement_clause_void_no_exception() {
        let mut b = input();
        b.non_disparagement_clause_present = true;
        let r = check(&b);
        assert!(r.non_disparagement_clause_void);
        assert!(r.offering_lease_unlawful);
        assert!(r.ftc_enforcement_available);
        assert!(r.state_ag_enforcement_available);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 45b(b)(1)")));
    }

    // ── § 45b(b)(2) penalty/fee clause ───────────────────────

    #[test]
    fn penalty_clause_void_no_exception() {
        let mut b = input();
        b.penalty_or_fee_for_review_clause_present = true;
        let r = check(&b);
        assert!(r.penalty_clause_void);
        assert!(r.offering_lease_unlawful);
        assert!(r.violations.iter().any(|v| v.contains("§ 45b(b)(2)")));
    }

    // ── § 45b(b)(3) IP-transfer clause ───────────────────────

    #[test]
    fn ip_transfer_clause_void_no_exception() {
        let mut b = input();
        b.ip_transfer_clause_present = true;
        let r = check(&b);
        assert!(r.ip_transfer_clause_void);
        assert!(r.offering_lease_unlawful);
        assert!(r.violations.iter().any(|v| v.contains("§ 45b(b)(3)")));
    }

    // ── § 45b(c) exceptions ───────────────────────────────────

    #[test]
    fn legally_actionable_content_exception_preserves_clause() {
        let mut b = input();
        b.non_disparagement_clause_present = true;
        b.exception_legally_actionable_content = true;
        let r = check(&b);
        assert!(r.exception_engaged);
        assert!(!r.non_disparagement_clause_void);
        assert!(r.compliant);
    }

    #[test]
    fn trade_secret_exception_preserves_clause() {
        let mut b = input();
        b.non_disparagement_clause_present = true;
        b.exception_trade_secret_or_personal_info = true;
        let r = check(&b);
        assert!(r.exception_engaged);
        assert!(!r.non_disparagement_clause_void);
    }

    #[test]
    fn medical_record_exception_preserves_clause() {
        let mut b = input();
        b.non_disparagement_clause_present = true;
        b.exception_medical_record_content = true;
        let r = check(&b);
        assert!(r.exception_engaged);
        assert!(!r.non_disparagement_clause_void);
    }

    #[test]
    fn unlawful_content_exception_preserves_clause() {
        let mut b = input();
        b.non_disparagement_clause_present = true;
        b.exception_unlawful_content = true;
        let r = check(&b);
        assert!(r.exception_engaged);
        assert!(!r.non_disparagement_clause_void);
    }

    #[test]
    fn exception_blocks_all_three_void_categories() {
        let mut b = input();
        b.non_disparagement_clause_present = true;
        b.penalty_or_fee_for_review_clause_present = true;
        b.ip_transfer_clause_present = true;
        b.exception_legally_actionable_content = true;
        let r = check(&b);
        assert!(r.exception_engaged);
        assert!(!r.non_disparagement_clause_void);
        assert!(!r.penalty_clause_void);
        assert!(!r.ip_transfer_clause_void);
        assert!(r.compliant);
    }

    // ── Multiple void clauses aggregate ──────────────────────

    #[test]
    fn all_three_clauses_void_simultaneously() {
        let mut b = input();
        b.non_disparagement_clause_present = true;
        b.penalty_or_fee_for_review_clause_present = true;
        b.ip_transfer_clause_present = true;
        let r = check(&b);
        assert!(r.non_disparagement_clause_void);
        assert!(r.penalty_clause_void);
        assert!(r.ip_transfer_clause_void);
        assert!(r.offering_lease_unlawful);
    }

    // ── § 45b(d) offering prohibition ────────────────────────

    #[test]
    fn offering_violation_independent_of_enforcement_attempt() {
        let mut b = input();
        b.non_disparagement_clause_present = true;
        let r = check(&b);
        // Offering the lease is itself unlawful.
        assert!(r.offering_lease_unlawful);
        assert!(r.violations.iter().any(|v| v.contains("§ 45b(d)")));
    }

    #[test]
    fn no_void_clause_no_offering_violation() {
        let r = check(&input());
        assert!(!r.offering_lease_unlawful);
    }

    // ── § 45b(e) enforcement availability ────────────────────

    #[test]
    fn ftc_and_state_ag_enforcement_available_on_voiding() {
        let mut b = input();
        b.non_disparagement_clause_present = true;
        let r = check(&b);
        assert!(r.ftc_enforcement_available);
        assert!(r.state_ag_enforcement_available);
    }

    #[test]
    fn no_enforcement_available_on_compliant_lease() {
        let r = check(&input());
        assert!(!r.ftc_enforcement_available);
        assert!(!r.state_ag_enforcement_available);
    }

    // ── Multi-clause truth table ─────────────────────────────

    #[test]
    fn three_clause_truth_table() {
        // 8-cell sweep: non_disparagement × penalty × IP_transfer.
        let cells = [
            (false, false, false, 0),
            (true, false, false, 1),
            (false, true, false, 1),
            (false, false, true, 1),
            (true, true, false, 2),
            (true, false, true, 2),
            (false, true, true, 2),
            (true, true, true, 3),
        ];
        for (nd, pen, ip, expected_void_count) in cells.iter() {
            let mut b = input();
            b.non_disparagement_clause_present = *nd;
            b.penalty_or_fee_for_review_clause_present = *pen;
            b.ip_transfer_clause_present = *ip;
            let r = check(&b);
            let void_count = (r.non_disparagement_clause_void as usize)
                + (r.penalty_clause_void as usize)
                + (r.ip_transfer_clause_void as usize);
            assert_eq!(void_count, *expected_void_count, "nd={} pen={} ip={}", nd, pen, ip);
        }
    }

    #[test]
    fn exception_truth_table_for_4_carve_outs() {
        // 16-cell sweep: 4 boolean exceptions. Any single true → engaged.
        for legally_actionable in [false, true] {
            for trade_secret in [false, true] {
                for medical in [false, true] {
                    for unlawful in [false, true] {
                        let mut b = input();
                        b.non_disparagement_clause_present = true;
                        b.exception_legally_actionable_content = legally_actionable;
                        b.exception_trade_secret_or_personal_info = trade_secret;
                        b.exception_medical_record_content = medical;
                        b.exception_unlawful_content = unlawful;
                        let r = check(&b);
                        let any_exception = legally_actionable
                            || trade_secret
                            || medical
                            || unlawful;
                        assert_eq!(
                            r.exception_engaged,
                            any_exception,
                            "exceptions: la={} ts={} med={} unl={}",
                            legally_actionable,
                            trade_secret,
                            medical,
                            unlawful
                        );
                    }
                }
            }
        }
    }

    // ── Citation + sibling note ──────────────────────────────

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&input());
        assert!(r.citation.contains("15 U.S.C. § 45b"));
        assert!(r.citation.contains("Pub. L. 114-258"));
        assert!(r.citation.contains("Dec 14, 2016"));
        assert!(r.citation.contains("§ 45b(a)"));
        assert!(r.citation.contains("§ 45b(b)(1)"));
        assert!(r.citation.contains("§ 45b(b)(2)"));
        assert!(r.citation.contains("§ 45b(b)(3)"));
        assert!(r.citation.contains("§ 45b(c)"));
        assert!(r.citation.contains("§ 45b(d)"));
        assert!(r.citation.contains("§ 45b(e)"));
        assert!(r.citation.contains("15 U.S.C. § 45"));
        assert!(r.citation.contains("FTC Bulletin 2022-05"));
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input());
        assert!(
            r.notes.iter().any(|n| n.contains("lease_waiver_enforceability")
                && n.contains("landlord_retaliation_damages")
                && n.contains("landlord_harassment")
                && n.contains("federal FLOOR")),
            "sibling distinction note must reference related modules + federal-floor concept"
        );
    }

    #[test]
    fn ftc_state_ag_enforcement_note_present() {
        let r = check(&input());
        assert!(
            r.notes.iter().any(|n| n.contains("FTC")
                && n.contains("State attorneys general")
                && n.contains("FTC Bulletin 2022-05")),
            "enforcement note must reference FTC + State AG + Bulletin 2022-05"
        );
    }

    // ── Compliance baseline ─────────────────────────────────

    #[test]
    fn lease_with_no_offending_provisions_compliant() {
        let r = check(&input());
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("baseline satisfied")));
    }

    #[test]
    fn lease_with_exception_only_carve_out_compliant() {
        let mut b = input();
        b.non_disparagement_clause_present = true;
        b.exception_legally_actionable_content = true;
        let r = check(&b);
        assert!(r.compliant);
        assert!(r.exception_engaged);
    }
}
