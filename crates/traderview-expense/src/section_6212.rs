//! IRC § 6212 — Statutory Notice of Deficiency (SNOD), also known
//! as the "90-day letter" or "ticket to Tax Court." Critical
//! procedural step BEFORE assessment can occur — the IRS must
//! generally issue § 6212 SNOD before assessing additional tax,
//! and that SNOD opens the taxpayer's Tax Court petition window
//! under § 6213. Natural sibling to `section_6213` (Tax Court
//! petition deadline + restrictions on assessment), `section_6501`
//! (ASED), and `section_6502` (CSED).
//!
//! **§ 6212(a) — In general**. The Secretary is authorized to
//! issue a SNOD by certified or registered mail when there is a
//! deficiency as defined in § 6211. The SNOD must state the
//! amount of deficiency and identify the tax year(s).
//!
//! **§ 6212(b) — Last known address rule** (LOAD-BEARING). The
//! SNOD must be mailed to the taxpayer's LAST KNOWN ADDRESS as
//! reflected in IRS records. If IRS uses a stale or wrong
//! address, the SNOD is INVALID — assessment based on an invalid
//! SNOD is itself invalid. Treas. Reg. § 301.6212-2 defines
//! "last known address" as the taxpayer's most recent address on
//! IRS records.
//!
//! **§ 6212(c) — One SNOD per taxable year** (with exceptions).
//! Generally only one SNOD per tax year; once issued, additional
//! SNODs for the same year are barred. Exceptions: (1) fraud,
//! (2) substantial omissions of items required to be shown,
//! (3) jeopardy assessment under § 6861, (4) bankruptcy.
//!
//! **§ 6212(d) — Rescission with taxpayer consent**. The
//! Secretary may rescind a SNOD WITH WRITTEN CONSENT of the
//! taxpayer. Once rescinded, the SNOD is treated as if never
//! issued; § 6212(c) one-per-year limit does not apply to
//! subsequent re-issued SNOD.
//!
//! **§ 6213(a) — Petition deadline**. Within 90 days (150 days
//! if taxpayer's address is outside the US) of SNOD mailing,
//! taxpayer may file a Tax Court petition for redetermination of
//! the deficiency. During the petition window AND while petition
//! is pending, the IRS is BARRED from assessment (§ 6213(a)
//! restraint on assessment).
//!
//! **Hopkins v. Commissioner (Tax Court 2024)** — taxpayer may
//! rely on the "last day to file petition" date stated in the
//! SNOD even when that stated date is incorrect (e.g., more than
//! 90 days after actual mailing). Equitable estoppel applies
//! against IRS where taxpayer relied to detriment.
//!
//! Trader-relevant: SNOD is the gateway between examination and
//! collection. Aggressive § 1256 mark-to-market positions,
//! § 988 currency reclassifications, § 1202 QSBS holding-period
//! audits all proceed via SNOD if examiner sustains deficiency.
//! Last-known-address defects are the most common SNOD
//! invalidation defense.
//!
//! Citations: IRC § 6212(a) (Secretary authority + certified /
//! registered mail); § 6212(b) (last known address rule);
//! § 6212(c) (one SNOD per year with fraud/omission/jeopardy
//! exceptions); § 6212(d) (rescission with taxpayer consent);
//! § 6213(a) (90-day petition deadline + 150-day extension for
//! outside-US addresses + restraint on assessment); Treas. Reg.
//! § 301.6212-2 (last known address definition); IRM 4.8.9
//! (statutory notices of deficiency procedural manual); IRM
//! 8.2.2 (SNOD case manual); Hopkins v. Commissioner (T.C. 2024)
//! (equitable reliance on stated last day to file).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryMethod {
    CertifiedMail,
    RegisteredMail,
    /// Any other delivery method — does NOT satisfy § 6212(a).
    OtherUnauthorized,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6212Input {
    /// Whether the IRS mailed a SNOD at all.
    pub snod_mailed: bool,
    /// Whether the SNOD was mailed to the taxpayer's last known
    /// address per IRS records (Treas. Reg. § 301.6212-2). If
    /// false, SNOD is INVALID.
    pub mailed_to_last_known_address: bool,
    /// Whether the taxpayer's address is outside the United
    /// States, triggering 150-day petition window under § 6213(a).
    pub taxpayer_address_outside_us: bool,
    /// Delivery method used — must be certified or registered
    /// mail per § 6212(a).
    pub delivery_method: DeliveryMethod,
    /// Whether the SNOD includes the "last day to file petition"
    /// statement (Hopkins v. Commissioner equitable reliance).
    pub snod_includes_last_day_statement: bool,
    /// Whether multiple SNODs were issued for the same taxable
    /// year (§ 6212(c) violation unless exception applies).
    pub multiple_snods_for_same_year: bool,
    /// Whether a § 6212(c) exception applies — fraud, substantial
    /// omission, jeopardy assessment under § 6861, or bankruptcy.
    pub multiple_snod_exception_applies: bool,
    /// Whether the SNOD was rescinded with taxpayer's WRITTEN
    /// consent under § 6212(d).
    pub rescinded_with_written_taxpayer_consent: bool,
    /// Number of days elapsed since SNOD mailing (for petition
    /// window analysis).
    pub days_since_snod_mailing: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6212Result {
    /// Whether the SNOD is valid for purposes of restraint on
    /// assessment and triggering the Tax Court petition window.
    pub snod_valid: bool,
    /// Petition deadline in days — 90 (default) or 150 (outside
    /// US per § 6213(a)).
    pub petition_deadline_days: u32,
    /// Whether the petition filing window is still open
    /// (days_since_snod_mailing < petition_deadline_days).
    pub petition_window_open: bool,
    /// Whether the IRS is barred from assessment during the
    /// petition window (§ 6213(a) restraint on assessment).
    pub assessment_barred_during_petition_window: bool,
    /// Whether the SNOD was validly rescinded under § 6212(d).
    pub rescission_authorized: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6212Input) -> Section6212Result {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let petition_deadline_days = if input.taxpayer_address_outside_us {
        150
    } else {
        90
    };

    if !input.snod_mailed {
        notes.push(
            "§ 6212(a) — no SNOD has been mailed; IRS may not assess deficiency until SNOD is issued (with limited exceptions: § 6213(b) summary assessment, § 6861 jeopardy assessment, math error)"
                .to_string(),
        );
        return Section6212Result {
            snod_valid: false,
            petition_deadline_days,
            petition_window_open: false,
            assessment_barred_during_petition_window: false,
            rescission_authorized: false,
            violations,
            citation: citation(),
            notes,
        };
    }

    if !input.mailed_to_last_known_address {
        violations.push(
            "§ 6212(b) + Treas. Reg. § 301.6212-2 — SNOD MUST be mailed to taxpayer's LAST KNOWN ADDRESS per IRS records; mailing to wrong address renders SNOD INVALID and any subsequent assessment also invalid"
                .to_string(),
        );
    }

    if !matches!(
        input.delivery_method,
        DeliveryMethod::CertifiedMail | DeliveryMethod::RegisteredMail
    ) {
        violations.push(
            "§ 6212(a) — SNOD must be sent by CERTIFIED or REGISTERED mail; other delivery methods do not satisfy statutory requirement"
                .to_string(),
        );
    }

    if input.multiple_snods_for_same_year && !input.multiple_snod_exception_applies {
        violations.push(
            "§ 6212(c) — generally only ONE SNOD per taxable year; subsequent SNODs barred absent fraud, substantial omission, § 6861 jeopardy assessment, or bankruptcy exception"
                .to_string(),
        );
    }

    let snod_valid = violations.is_empty();
    let petition_window_open = snod_valid && input.days_since_snod_mailing < petition_deadline_days;
    let assessment_barred = snod_valid && petition_window_open;

    let rescission_authorized = input.rescinded_with_written_taxpayer_consent;

    notes.push(
        "§ 6212(a) — Secretary authorized to issue SNOD when deficiency exists under § 6211; SNOD must state deficiency amount + tax year(s)"
            .to_string(),
    );

    if input.mailed_to_last_known_address {
        notes.push(
            "§ 6212(b) + Treas. Reg. § 301.6212-2 — SNOD mailed to last known address satisfies statutory requirement; 'last known address' is taxpayer's most recent address on IRS records"
                .to_string(),
        );
    }

    notes.push(format!(
        "§ 6213(a) — petition deadline is {} days from SNOD mailing ({})",
        petition_deadline_days,
        if input.taxpayer_address_outside_us {
            "150-day window applies because taxpayer's address is outside the United States"
        } else {
            "default 90-day window"
        }
    ));

    notes.push(
        "§ 6213(a) — during petition window and while Tax Court petition pending, IRS is BARRED from assessment (restraint on assessment)"
            .to_string(),
    );

    if input.multiple_snods_for_same_year && input.multiple_snod_exception_applies {
        notes.push(
            "§ 6212(c) exception engaged — multiple SNODs permitted under fraud / substantial omission / § 6861 jeopardy / bankruptcy exception"
                .to_string(),
        );
    }

    if rescission_authorized {
        notes.push(
            "§ 6212(d) — SNOD validly rescinded with taxpayer's WRITTEN consent; SNOD treated as if never issued; § 6212(c) one-per-year limit does NOT bar subsequent re-issued SNOD after rescission"
                .to_string(),
        );
    }

    if input.snod_includes_last_day_statement {
        notes.push(
            "Hopkins v. Commissioner (T.C. 2024) — taxpayer may equitably rely on 'last day to file petition' date stated in SNOD even when that stated date is incorrect"
                .to_string(),
        );
    }

    notes.push(
        "IRM 4.8.9 — statutory notices of deficiency procedural manual; IRM 8.2.2 — SNOD case manual"
            .to_string(),
    );

    Section6212Result {
        snod_valid,
        petition_deadline_days,
        petition_window_open,
        assessment_barred_during_petition_window: assessment_barred,
        rescission_authorized,
        violations,
        citation: citation(),
        notes,
    }
}

fn citation() -> &'static str {
    "IRC §§ 6212(a), 6212(b), 6212(c), 6212(d), 6213(a); Treas. Reg. § 301.6212-2; IRM 4.8.9; IRM 8.2.2; Hopkins v. Commissioner (T.C. 2024)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section6212Input {
        Section6212Input {
            snod_mailed: true,
            mailed_to_last_known_address: true,
            taxpayer_address_outside_us: false,
            delivery_method: DeliveryMethod::CertifiedMail,
            snod_includes_last_day_statement: true,
            multiple_snods_for_same_year: false,
            multiple_snod_exception_applies: false,
            rescinded_with_written_taxpayer_consent: false,
            days_since_snod_mailing: 30,
        }
    }

    #[test]
    fn clean_snod_is_valid() {
        let r = check(&base());
        assert!(r.snod_valid);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn default_petition_deadline_is_90_days() {
        let r = check(&base());
        assert_eq!(r.petition_deadline_days, 90);
    }

    #[test]
    fn outside_us_petition_deadline_is_150_days() {
        let mut i = base();
        i.taxpayer_address_outside_us = true;
        let r = check(&i);
        assert_eq!(r.petition_deadline_days, 150);
        assert!(r.notes.iter().any(|n| n.contains(
            "150-day window applies because taxpayer's address is outside the United States"
        )));
    }

    #[test]
    fn no_snod_mailed_invalid_with_pre_snod_note() {
        let mut i = base();
        i.snod_mailed = false;
        let r = check(&i);
        assert!(!r.snod_valid);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6212(a)") && n.contains("no SNOD has been mailed")));
    }

    #[test]
    fn wrong_address_invalidates_snod() {
        let mut i = base();
        i.mailed_to_last_known_address = false;
        let r = check(&i);
        assert!(!r.snod_valid);
        assert!(r.violations.iter().any(|v| v.contains("§ 6212(b)")
            && v.contains("LAST KNOWN ADDRESS")
            && v.contains("INVALID")));
    }

    #[test]
    fn unauthorized_delivery_method_invalidates_snod() {
        let mut i = base();
        i.delivery_method = DeliveryMethod::OtherUnauthorized;
        let r = check(&i);
        assert!(!r.snod_valid);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6212(a)") && v.contains("CERTIFIED or REGISTERED")));
    }

    #[test]
    fn registered_mail_satisfies_delivery_requirement() {
        let mut i = base();
        i.delivery_method = DeliveryMethod::RegisteredMail;
        let r = check(&i);
        assert!(r.snod_valid);
    }

    #[test]
    fn multiple_snods_without_exception_violates_subsection_c() {
        let mut i = base();
        i.multiple_snods_for_same_year = true;
        let r = check(&i);
        assert!(!r.snod_valid);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6212(c)") && v.contains("ONE SNOD per taxable year")));
    }

    #[test]
    fn multiple_snods_with_exception_compliant() {
        let mut i = base();
        i.multiple_snods_for_same_year = true;
        i.multiple_snod_exception_applies = true;
        let r = check(&i);
        assert!(r.snod_valid);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6212(c) exception") && n.contains("§ 6861 jeopardy")));
    }

    #[test]
    fn petition_window_open_within_default_90() {
        let mut i = base();
        i.days_since_snod_mailing = 30;
        let r = check(&i);
        assert!(r.petition_window_open);
        assert!(r.assessment_barred_during_petition_window);
    }

    #[test]
    fn petition_window_closed_after_90_default() {
        let mut i = base();
        i.days_since_snod_mailing = 91;
        let r = check(&i);
        assert!(!r.petition_window_open);
        assert!(!r.assessment_barred_during_petition_window);
    }

    #[test]
    fn petition_window_boundary_day_89_open() {
        let mut i = base();
        i.days_since_snod_mailing = 89;
        let r = check(&i);
        assert!(r.petition_window_open);
    }

    #[test]
    fn petition_window_boundary_day_90_closed() {
        let mut i = base();
        i.days_since_snod_mailing = 90;
        let r = check(&i);
        assert!(!r.petition_window_open);
    }

    #[test]
    fn petition_window_outside_us_boundary_149_open() {
        let mut i = base();
        i.taxpayer_address_outside_us = true;
        i.days_since_snod_mailing = 149;
        let r = check(&i);
        assert!(r.petition_window_open);
    }

    #[test]
    fn petition_window_outside_us_boundary_150_closed() {
        let mut i = base();
        i.taxpayer_address_outside_us = true;
        i.days_since_snod_mailing = 150;
        let r = check(&i);
        assert!(!r.petition_window_open);
    }

    #[test]
    fn invalid_snod_blocks_petition_window() {
        let mut i = base();
        i.mailed_to_last_known_address = false;
        i.days_since_snod_mailing = 10;
        let r = check(&i);
        assert!(!r.petition_window_open);
        assert!(!r.assessment_barred_during_petition_window);
    }

    #[test]
    fn rescission_with_written_consent_authorized() {
        let mut i = base();
        i.rescinded_with_written_taxpayer_consent = true;
        let r = check(&i);
        assert!(r.rescission_authorized);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6212(d)") && n.contains("WRITTEN consent")));
    }

    #[test]
    fn rescission_without_consent_not_authorized() {
        let r = check(&base());
        assert!(!r.rescission_authorized);
    }

    #[test]
    fn hopkins_reliance_note_when_last_day_statement_present() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Hopkins v. Commissioner")));
    }

    #[test]
    fn no_hopkins_note_when_last_day_statement_absent() {
        let mut i = base();
        i.snod_includes_last_day_statement = false;
        let r = check(&i);
        assert!(!r
            .notes
            .iter()
            .any(|n| n.contains("Hopkins v. Commissioner")));
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&base());
        assert!(r
            .citation
            .contains("§§ 6212(a), 6212(b), 6212(c), 6212(d), 6213(a)"));
    }

    #[test]
    fn citation_pins_treas_reg_and_irm_and_hopkins() {
        let r = check(&base());
        assert!(r.citation.contains("Treas. Reg. § 301.6212-2"));
        assert!(r.citation.contains("IRM 4.8.9"));
        assert!(r.citation.contains("IRM 8.2.2"));
        assert!(r.citation.contains("Hopkins v. Commissioner"));
    }

    #[test]
    fn restraint_on_assessment_note_present_when_window_open() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6213(a)") && n.contains("BARRED from assessment")));
    }

    #[test]
    fn petition_deadline_note_describes_days() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6213(a)") && n.contains("90 days from SNOD")));
    }

    #[test]
    fn multiple_violations_all_surface() {
        let mut i = base();
        i.mailed_to_last_known_address = false;
        i.delivery_method = DeliveryMethod::OtherUnauthorized;
        i.multiple_snods_for_same_year = true;
        let r = check(&i);
        assert_eq!(r.violations.len(), 3);
    }

    #[test]
    fn outside_us_unique_150_day_invariant() {
        let mut i_outside = base();
        i_outside.taxpayer_address_outside_us = true;
        assert_eq!(check(&i_outside).petition_deadline_days, 150);

        let mut i_us = base();
        i_us.taxpayer_address_outside_us = false;
        assert_eq!(check(&i_us).petition_deadline_days, 90);
    }

    #[test]
    fn snod_invalid_blocks_assessment_bar() {
        let mut i = base();
        i.mailed_to_last_known_address = false;
        let r = check(&i);
        assert!(!r.snod_valid);
        assert!(!r.assessment_barred_during_petition_window);
    }

    #[test]
    fn last_known_address_note_when_satisfied() {
        let r = check(&base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6212(b)")
            && n.contains("Treas. Reg. § 301.6212-2")
            && n.contains("last known address")));
    }

    #[test]
    fn irm_8_2_2_note_always_present_when_snod_mailed() {
        let r = check(&base());
        assert!(r.notes.iter().any(|n| n.contains("IRM 8.2.2")));
    }

    #[test]
    fn rescission_does_not_invalidate_snod_facts() {
        let mut i = base();
        i.rescinded_with_written_taxpayer_consent = true;
        let r = check(&i);
        assert!(r.snod_valid);
        assert!(r.rescission_authorized);
    }
}
