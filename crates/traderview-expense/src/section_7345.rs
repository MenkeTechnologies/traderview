//! IRC § 7345 — Revocation or denial of passport in case of
//! certain tax delinquencies (FAST Act 2015).
//!
//! Trader-critical for high-net-worth individuals with
//! international travel needs and unresolved IRS tax issues.
//! Added by Section 32101 of the Fixing America's Surface
//! Transportation (FAST) Act, Pub. L. 114-94 (December 4, 2015).
//! IRS certifies "seriously delinquent tax debt" to the State
//! Department, which then DENIES passport applications, REVOKES
//! existing passports, or LIMITS passport use.
//!
//! § 7345(b)(1) SERIOUSLY DELINQUENT TAX DEBT — defined as
//! unpaid, legally enforceable federal tax liability of an
//! individual (including assessed penalties and interest) that
//! exceeds the inflation-adjusted threshold ($66,000 for 2025;
//! originally $50,000 in 2015) AND:
//!   (A) a notice of lien has been filed pursuant to § 6323
//!       AND administrative rights under § 6320 have been
//!       exhausted or have lapsed; OR
//!   (B) a levy has been issued under § 6331.
//!
//! § 7345(b)(2) EXCLUSIONS — debt is NOT seriously delinquent
//! when:
//!   (A) debt is being timely paid pursuant to § 6159
//!       installment agreement;
//!   (B) debt is being timely paid pursuant to § 7122 offer
//!       in compromise accepted by the IRS;
//!   (C) collection suspended due to § 6015 innocent spouse
//!       relief request (subsections (b), (c), or (f));
//!   (D) collection due process hearing requested under § 6320
//!       or § 6330;
//!   (E) taxpayer in pending bankruptcy proceeding;
//!   (F) taxpayer identified by IRS as victim of identity theft;
//!   (G) taxpayer in federally declared disaster area;
//!   (H) taxpayer in IRS "currently not collectible" status due
//!       to financial hardship.
//!
//! § 7345(c) REVERSAL — IRS notifies State Department within
//! 30 days of debt resolution (full payment, settlement,
//! certification found erroneous, or exclusion engaged).
//!
//! § 7345(d) NOTIFICATION — IRS provides contemporaneous
//! notification (CP508C) when certifying debt to State
//! Department; CP508R when reversing certification.
//!
//! Citations: 26 U.S.C. § 7345 (general); 26 U.S.C. § 7345(b)(1)
//! (seriously delinquent definition); 26 U.S.C. § 7345(b)(1)(A)
//! (lien + exhausted remedies prong); 26 U.S.C. § 7345(b)(1)(B)
//! (levy prong); 26 U.S.C. § 7345(b)(2) (exclusions); 26 U.S.C.
//! § 7345(c) (reversal); 26 U.S.C. § 7345(d) (notification);
//! 26 U.S.C. § 7345(e) (judicial review); FAST Act § 32101
//! (Pub. L. 114-94, December 4, 2015 enactment); 26 U.S.C.
//! § 6159 (installment agreement); 26 U.S.C. § 7122 (offer in
//! compromise); 26 U.S.C. § 6015 (innocent spouse relief);
//! 26 U.S.C. § 6320 (lien CDP hearing); 26 U.S.C. § 6330 (levy
//! CDP hearing); 26 U.S.C. § 6323 (notice of federal tax lien);
//! 26 U.S.C. § 6331 (levy authority); IRS Notice CP508C
//! (certification); IRS Notice CP508R (reversal).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section7345Input {
    /// Total assessed federal tax liability including penalties
    /// and interest (cents).
    pub assessed_tax_debt_cents: i64,
    /// Inflation-adjusted threshold for the relevant year
    /// (cents). $66,000 = 6,600,000 cents (2025).
    pub annual_threshold_cents: i64,
    /// § 7345(b)(1)(A) — notice of federal tax lien filed under
    /// § 6323.
    pub notice_of_federal_tax_lien_filed: bool,
    /// § 7345(b)(1)(A) — administrative rights under § 6320
    /// CDP hearing have been exhausted or have lapsed.
    pub administrative_remedies_exhausted: bool,
    /// § 7345(b)(1)(B) — levy has been issued under § 6331.
    pub levy_issued: bool,
    /// § 7345(b)(2)(A) — installment agreement under § 6159
    /// being timely paid.
    pub installment_agreement_active: bool,
    /// § 7345(b)(2)(B) — offer in compromise under § 7122
    /// accepted by IRS.
    pub offer_in_compromise_accepted: bool,
    /// § 7345(b)(2)(C) — innocent spouse relief under § 6015
    /// pending.
    pub innocent_spouse_claim_pending: bool,
    /// § 7345(b)(2)(D) — § 6320 or § 6330 CDP hearing pending.
    pub collection_due_process_pending: bool,
    /// § 7345(b)(2)(E) — pending bankruptcy proceeding.
    pub bankruptcy_pending: bool,
    /// § 7345(b)(2)(F) — identified by IRS as identity theft
    /// victim.
    pub identity_theft_victim: bool,
    /// § 7345(b)(2)(G) — taxpayer in federally declared
    /// disaster area.
    pub in_federally_declared_disaster_area: bool,
    /// § 7345(b)(2)(H) — IRS "currently not collectible"
    /// status due to financial hardship.
    pub currently_not_collectible_status: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7345Result {
    /// True if debt qualifies as "seriously delinquent" under
    /// § 7345(b)(1).
    pub seriously_delinquent_tax_debt: bool,
    /// True if any § 7345(b)(2) exclusion is engaged.
    pub exclusion_engaged: bool,
    /// List of active exclusion descriptions.
    pub active_exclusions: Vec<String>,
    /// True if collection action prong (A) or (B) of
    /// § 7345(b)(1) is satisfied.
    pub collection_action_initiated: bool,
    /// True if assessed debt exceeds the threshold.
    pub debt_exceeds_threshold: bool,
    /// Amount by which debt exceeds threshold (cents); negative
    /// if below.
    pub amount_above_threshold_cents: i64,
    /// True if IRS certification of debt to State Department is
    /// engaged (CP508C notice).
    pub passport_certification_engaged: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// 2025 inflation-adjusted threshold (cents).
pub const THRESHOLD_2025_CENTS: i64 = 6_600_000;
/// Original 2015 statutory threshold (cents) — for historical
/// reference.
pub const THRESHOLD_2015_CENTS: i64 = 5_000_000;
/// § 7345(c) reversal notification window (days).
pub const REVERSAL_NOTIFICATION_DAYS: i64 = 30;

pub fn compute(input: &Section7345Input) -> Section7345Result {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();
    let mut active_exclusions: Vec<String> = Vec::new();

    let debt = input.assessed_tax_debt_cents.max(0);
    let threshold = input.annual_threshold_cents.max(0);

    // § 7345(b)(1) — debt-threshold determination.
    let debt_exceeds_threshold = debt > threshold;
    let amount_above_threshold = debt - threshold;

    // § 7345(b)(1)(A)/(B) — collection action prong.
    let lien_and_exhausted =
        input.notice_of_federal_tax_lien_filed && input.administrative_remedies_exhausted;
    let collection_action_initiated = lien_and_exhausted || input.levy_issued;

    // § 7345(b)(2) exclusions.
    let exclusion_checks = [
        (
            input.installment_agreement_active,
            "§ 7345(b)(2)(A) — installment agreement under § 6159 being timely paid",
        ),
        (
            input.offer_in_compromise_accepted,
            "§ 7345(b)(2)(B) — offer in compromise under § 7122 accepted by IRS",
        ),
        (
            input.innocent_spouse_claim_pending,
            "§ 7345(b)(2)(C) — innocent spouse relief request under § 6015 pending",
        ),
        (
            input.collection_due_process_pending,
            "§ 7345(b)(2)(D) — § 6320 or § 6330 collection due process hearing pending",
        ),
        (
            input.bankruptcy_pending,
            "§ 7345(b)(2)(E) — pending bankruptcy proceeding",
        ),
        (
            input.identity_theft_victim,
            "§ 7345(b)(2)(F) — IRS-identified identity theft victim",
        ),
        (
            input.in_federally_declared_disaster_area,
            "§ 7345(b)(2)(G) — taxpayer in federally declared disaster area",
        ),
        (
            input.currently_not_collectible_status,
            "§ 7345(b)(2)(H) — IRS 'currently not collectible' status due to financial \
             hardship",
        ),
    ];
    for (active, label) in exclusion_checks.iter() {
        if *active {
            active_exclusions.push((*label).to_string());
        }
    }
    let exclusion_engaged = !active_exclusions.is_empty();

    // § 7345(b)(1) seriously delinquent — all three conditions:
    // (i) debt > threshold; (ii) collection action initiated;
    // (iii) no exclusion engaged.
    let seriously_delinquent_tax_debt =
        debt_exceeds_threshold && collection_action_initiated && !exclusion_engaged;
    let passport_certification_engaged = seriously_delinquent_tax_debt;

    if passport_certification_engaged {
        violations.push(format!(
            "§ 7345(b)(1) — debt of {} cents exceeds threshold {} cents by {} cents AND \
             collection action initiated ({}); no § 7345(b)(2) exclusion engaged. IRS \
             will certify (Notice CP508C) to State Department; passport may be denied, \
             revoked, or limited.",
            debt,
            threshold,
            amount_above_threshold,
            if lien_and_exhausted && input.levy_issued {
                "lien + administrative remedies exhausted AND levy issued"
            } else if lien_and_exhausted {
                "lien + administrative remedies exhausted"
            } else {
                "levy issued"
            },
        ));
    }

    // Threshold note.
    if debt_exceeds_threshold {
        notes.push(format!(
            "Debt {} cents EXCEEDS § 7345(b)(1) threshold {} cents by {} cents.",
            debt, threshold, amount_above_threshold,
        ));
    } else {
        notes.push(format!(
            "Debt {} cents does NOT exceed § 7345(b)(1) threshold {} cents (gap of {} \
             cents). § 7345 certification not engaged regardless of collection action or \
             exclusion status.",
            debt,
            threshold,
            (threshold - debt).max(0),
        ));
    }

    // Collection action note.
    if collection_action_initiated {
        notes.push(format!(
            "§ 7345(b)(1) collection action prong satisfied: {}{}",
            if lien_and_exhausted {
                "notice of federal tax lien under § 6323 filed + § 6320 administrative \
                 remedies exhausted (prong (A))"
            } else if input.notice_of_federal_tax_lien_filed {
                "notice of federal tax lien filed BUT § 6320 remedies not yet exhausted \
                 (prong (A) not fully satisfied)"
            } else {
                ""
            },
            if input.levy_issued {
                "; § 6331 levy issued (prong (B))"
            } else {
                ""
            },
        ));
    } else if input.notice_of_federal_tax_lien_filed && !input.administrative_remedies_exhausted {
        notes.push(
            "§ 7345(b)(1)(A) — notice of federal tax lien filed but § 6320 administrative \
             remedies have not yet been exhausted. Collection action prong not yet \
             satisfied. Levy alternative under § 7345(b)(1)(B) also not invoked."
                .to_string(),
        );
    } else {
        notes.push(
            "§ 7345(b)(1) collection action prong NOT satisfied — neither lien-with-\
             exhausted-remedies (A) nor levy (B) has occurred. § 7345 certification \
             cannot engage."
                .to_string(),
        );
    }

    // Exclusion notes.
    if exclusion_engaged {
        notes.push(format!(
            "§ 7345(b)(2) EXCLUSION(S) engaged — debt is NOT 'seriously delinquent' \
             regardless of amount + collection action:\n  - {}",
            active_exclusions.join("\n  - "),
        ));
        notes.push(
            "§ 7345(c) — IRS must notify State Department within 30 days when exclusion \
             engages (or any subsequent event that resolves the debt). Notice CP508R \
             issued to taxpayer."
                .to_string(),
        );
    }

    notes.push(
        "Sibling modules: § 6011 (taxpayer disclosure of reportable transactions — \
         disclosure regime), § 6651 (failure-to-file/pay penalties — source of much of \
         the assessed amount), § 6654 (failure-to-pay-estimated-tax), § 6662 (accuracy \
         penalty), § 6707A (taxpayer disclosure penalty). § 7345 is the COLLECTION \
         pressure layer — operates after underlying tax liability has been assessed and \
         collection action has begun. Original FAST Act § 32101 (Pub. L. 114-94, \
         December 4, 2015) implementation. § 7345(e) provides judicial review in U.S. \
         Tax Court OR U.S. District Court for the district where the taxpayer resides."
            .to_string(),
    );

    let compliant = violations.is_empty();

    Section7345Result {
        seriously_delinquent_tax_debt,
        exclusion_engaged,
        active_exclusions,
        collection_action_initiated,
        debt_exceeds_threshold,
        amount_above_threshold_cents: amount_above_threshold,
        passport_certification_engaged,
        compliant,
        violations,
        citation: "26 U.S.C. § 7345 (general); 26 U.S.C. § 7345(b)(1) (seriously \
                   delinquent definition); 26 U.S.C. § 7345(b)(1)(A) (lien + exhausted \
                   remedies prong); 26 U.S.C. § 7345(b)(1)(B) (levy prong); 26 U.S.C. \
                   § 7345(b)(2) (exclusions A-H); 26 U.S.C. § 7345(c) (30-day reversal \
                   notification); 26 U.S.C. § 7345(d) (CP508C/R contemporaneous \
                   notification); 26 U.S.C. § 7345(e) (judicial review — Tax Court OR \
                   District Court); FAST Act § 32101 (Pub. L. 114-94, December 4, 2015); \
                   26 U.S.C. § 6159 (installment agreement); 26 U.S.C. § 7122 (offer in \
                   compromise); 26 U.S.C. § 6015 (innocent spouse relief); 26 U.S.C. \
                   § 6320 (lien CDP hearing); 26 U.S.C. § 6330 (levy CDP hearing); \
                   26 U.S.C. § 6323 (notice of federal tax lien); 26 U.S.C. § 6331 \
                   (levy authority); IRS Notice CP508C (certification); IRS Notice \
                   CP508R (reversal)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Section7345Input {
        Section7345Input {
            assessed_tax_debt_cents: 10_000_000, // $100K — above threshold
            annual_threshold_cents: THRESHOLD_2025_CENTS,
            notice_of_federal_tax_lien_filed: true,
            administrative_remedies_exhausted: true,
            levy_issued: false,
            installment_agreement_active: false,
            offer_in_compromise_accepted: false,
            innocent_spouse_claim_pending: false,
            collection_due_process_pending: false,
            bankruptcy_pending: false,
            identity_theft_victim: false,
            in_federally_declared_disaster_area: false,
            currently_not_collectible_status: false,
        }
    }

    // ── Engagement triggers ────────────────────────────────────

    #[test]
    fn baseline_seriously_delinquent_certification_engaged() {
        let r = compute(&input());
        assert!(r.seriously_delinquent_tax_debt);
        assert!(r.passport_certification_engaged);
        assert!(!r.compliant);
    }

    #[test]
    fn debt_below_threshold_no_certification() {
        let mut b = input();
        b.assessed_tax_debt_cents = 5_000_000; // $50K — below $66K
        let r = compute(&b);
        assert!(!r.debt_exceeds_threshold);
        assert!(!r.seriously_delinquent_tax_debt);
        assert!(!r.passport_certification_engaged);
    }

    #[test]
    fn debt_at_threshold_exactly_not_exceeding() {
        let mut b = input();
        b.assessed_tax_debt_cents = THRESHOLD_2025_CENTS;
        let r = compute(&b);
        // Statute reads "exceeds" — not >=. Exact match does not engage.
        assert!(!r.debt_exceeds_threshold);
        assert!(!r.passport_certification_engaged);
    }

    #[test]
    fn debt_one_cent_above_threshold_engages() {
        let mut b = input();
        b.assessed_tax_debt_cents = THRESHOLD_2025_CENTS + 1;
        let r = compute(&b);
        assert!(r.debt_exceeds_threshold);
        assert!(r.passport_certification_engaged);
    }

    // ── Collection action prong ────────────────────────────────

    #[test]
    fn lien_without_exhausted_remedies_no_engagement() {
        let mut b = input();
        b.administrative_remedies_exhausted = false;
        let r = compute(&b);
        assert!(!r.collection_action_initiated);
        assert!(!r.seriously_delinquent_tax_debt);
    }

    #[test]
    fn levy_alone_satisfies_collection_action() {
        let mut b = input();
        b.notice_of_federal_tax_lien_filed = false;
        b.administrative_remedies_exhausted = false;
        b.levy_issued = true;
        let r = compute(&b);
        assert!(r.collection_action_initiated);
        assert!(r.seriously_delinquent_tax_debt);
    }

    #[test]
    fn no_lien_no_levy_no_engagement() {
        let mut b = input();
        b.notice_of_federal_tax_lien_filed = false;
        b.administrative_remedies_exhausted = false;
        b.levy_issued = false;
        let r = compute(&b);
        assert!(!r.collection_action_initiated);
        assert!(!r.seriously_delinquent_tax_debt);
    }

    // ── § 7345(b)(2) exclusions — each disqualifies certification ─

    #[test]
    fn installment_agreement_exclusion_blocks_certification() {
        let mut b = input();
        b.installment_agreement_active = true;
        let r = compute(&b);
        assert!(r.exclusion_engaged);
        assert!(!r.seriously_delinquent_tax_debt);
        assert!(r.active_exclusions.iter().any(|e| e.contains("§ 6159")));
    }

    #[test]
    fn offer_in_compromise_exclusion_blocks() {
        let mut b = input();
        b.offer_in_compromise_accepted = true;
        let r = compute(&b);
        assert!(r.exclusion_engaged);
        assert!(!r.seriously_delinquent_tax_debt);
        assert!(r.active_exclusions.iter().any(|e| e.contains("§ 7122")));
    }

    #[test]
    fn innocent_spouse_exclusion_blocks() {
        let mut b = input();
        b.innocent_spouse_claim_pending = true;
        let r = compute(&b);
        assert!(r.exclusion_engaged);
        assert!(r.active_exclusions.iter().any(|e| e.contains("§ 6015")));
    }

    #[test]
    fn cdp_hearing_exclusion_blocks() {
        let mut b = input();
        b.collection_due_process_pending = true;
        let r = compute(&b);
        assert!(r.exclusion_engaged);
        assert!(r
            .active_exclusions
            .iter()
            .any(|e| e.contains("§ 6320 or § 6330")));
    }

    #[test]
    fn bankruptcy_exclusion_blocks() {
        let mut b = input();
        b.bankruptcy_pending = true;
        let r = compute(&b);
        assert!(r.exclusion_engaged);
        assert!(!r.seriously_delinquent_tax_debt);
    }

    #[test]
    fn identity_theft_exclusion_blocks() {
        let mut b = input();
        b.identity_theft_victim = true;
        let r = compute(&b);
        assert!(r.exclusion_engaged);
        assert!(!r.seriously_delinquent_tax_debt);
    }

    #[test]
    fn disaster_area_exclusion_blocks() {
        let mut b = input();
        b.in_federally_declared_disaster_area = true;
        let r = compute(&b);
        assert!(r.exclusion_engaged);
        assert!(!r.seriously_delinquent_tax_debt);
    }

    #[test]
    fn currently_not_collectible_exclusion_blocks() {
        let mut b = input();
        b.currently_not_collectible_status = true;
        let r = compute(&b);
        assert!(r.exclusion_engaged);
        assert!(!r.seriously_delinquent_tax_debt);
    }

    #[test]
    fn multiple_exclusions_listed_individually() {
        let mut b = input();
        b.installment_agreement_active = true;
        b.bankruptcy_pending = true;
        b.identity_theft_victim = true;
        let r = compute(&b);
        assert_eq!(r.active_exclusions.len(), 3);
        assert!(!r.seriously_delinquent_tax_debt);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn all_three_engagement_conditions_required_truth_table() {
        // 8-cell truth table: debt > threshold × collection action × no exclusion.
        let cells = [
            (true, true, false, true),    // all three → engaged
            (false, true, false, false),  // debt below threshold
            (true, false, false, false),  // no collection action
            (true, true, true, false),    // exclusion engaged
            (false, false, false, false), // nothing
            (false, true, true, false),   // exclusion blocks even without debt above
            (true, false, true, false),   // multiple disqualifiers
            (false, false, true, false),  // everything against
        ];
        for (above_threshold, collection, exclusion, expected_engaged) in cells.iter() {
            let mut b = input();
            b.assessed_tax_debt_cents = if *above_threshold {
                THRESHOLD_2025_CENTS + 1_000_000
            } else {
                1_000_000
            };
            b.notice_of_federal_tax_lien_filed = *collection;
            b.administrative_remedies_exhausted = *collection;
            b.installment_agreement_active = *exclusion;
            let r = compute(&b);
            assert_eq!(
                r.passport_certification_engaged, *expected_engaged,
                "above_threshold={} collection={} exclusion={}",
                above_threshold, collection, exclusion
            );
        }
    }

    #[test]
    fn each_exclusion_independently_blocks_certification_invariant() {
        // 8-prong sweep — each § 7345(b)(2) exclusion independently
        // disqualifies certification.
        let setters: Vec<Box<dyn Fn(&mut Section7345Input)>> = vec![
            Box::new(|b| b.installment_agreement_active = true),
            Box::new(|b| b.offer_in_compromise_accepted = true),
            Box::new(|b| b.innocent_spouse_claim_pending = true),
            Box::new(|b| b.collection_due_process_pending = true),
            Box::new(|b| b.bankruptcy_pending = true),
            Box::new(|b| b.identity_theft_victim = true),
            Box::new(|b| b.in_federally_declared_disaster_area = true),
            Box::new(|b| b.currently_not_collectible_status = true),
        ];
        for setter in &setters {
            let mut b = input();
            setter(&mut b);
            let r = compute(&b);
            assert!(r.exclusion_engaged);
            assert!(!r.passport_certification_engaged);
        }
    }

    #[test]
    fn threshold_constants_invariant() {
        // $66K 2025 = 6,600,000 cents; $50K 2015 = 5,000,000 cents.
        assert_eq!(THRESHOLD_2025_CENTS, 6_600_000);
        assert_eq!(THRESHOLD_2015_CENTS, 5_000_000);
        assert!(THRESHOLD_2025_CENTS > THRESHOLD_2015_CENTS);
        assert_eq!(REVERSAL_NOTIFICATION_DAYS, 30);
    }

    #[test]
    fn parameterized_threshold_2024_value() {
        // 2024 threshold was $62K.
        let mut b = input();
        b.annual_threshold_cents = 6_200_000;
        b.assessed_tax_debt_cents = 6_300_000; // $63K — above 2024 but below 2025
        let r = compute(&b);
        assert!(r.debt_exceeds_threshold);
        assert!(r.seriously_delinquent_tax_debt);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input());
        assert!(r.citation.contains("§ 7345"));
        assert!(r.citation.contains("§ 7345(b)(1)"));
        assert!(r.citation.contains("§ 7345(b)(1)(A)"));
        assert!(r.citation.contains("§ 7345(b)(1)(B)"));
        assert!(r.citation.contains("§ 7345(b)(2)"));
        assert!(r.citation.contains("§ 7345(c)"));
        assert!(r.citation.contains("§ 7345(d)"));
        assert!(r.citation.contains("§ 7345(e)"));
        assert!(r.citation.contains("FAST Act § 32101"));
        assert!(r.citation.contains("December 4, 2015"));
        assert!(r.citation.contains("§ 6159"));
        assert!(r.citation.contains("§ 7122"));
        assert!(r.citation.contains("§ 6015"));
        assert!(r.citation.contains("§ 6320"));
        assert!(r.citation.contains("§ 6330"));
        assert!(r.citation.contains("§ 6323"));
        assert!(r.citation.contains("§ 6331"));
        assert!(r.citation.contains("CP508C"));
        assert!(r.citation.contains("CP508R"));
    }

    #[test]
    fn sibling_modules_note_present() {
        let r = compute(&input());
        assert!(
            r.notes.iter().any(|n| n.contains("§ 6011")
                && n.contains("§ 6651")
                && n.contains("§ 6654")
                && n.contains("§ 6662")
                && n.contains("§ 6707A")
                && n.contains("§ 32101")),
            "sibling cluster note must reference disclosure + penalty siblings + FAST Act origin"
        );
    }

    // ── Defensive input clamping ───────────────────────────────

    #[test]
    fn defensive_negative_debt_no_engagement() {
        let mut b = input();
        b.assessed_tax_debt_cents = -10_000_000;
        let r = compute(&b);
        // Negative → 0 → below threshold.
        assert!(!r.debt_exceeds_threshold);
        assert!(!r.seriously_delinquent_tax_debt);
    }

    #[test]
    fn defensive_negative_threshold_clamped_to_zero() {
        let mut b = input();
        b.annual_threshold_cents = -1_000_000;
        b.assessed_tax_debt_cents = 100;
        let r = compute(&b);
        // Threshold 0; debt $1 → above threshold.
        assert!(r.debt_exceeds_threshold);
    }

    #[test]
    fn zero_debt_no_engagement() {
        let mut b = input();
        b.assessed_tax_debt_cents = 0;
        let r = compute(&b);
        assert!(!r.passport_certification_engaged);
    }
}
