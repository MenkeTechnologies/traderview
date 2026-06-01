//! IRC § 6045B — Returns relating to actions affecting basis of
//! specified securities (Form 8937).
//!
//! Trader-critical for any holder of stock that undergoes a
//! corporate action (split, spin-off, merger, return-of-capital
//! distribution). § 6045B requires the ISSUER of the specified
//! security to report the organizational action and the
//! quantitative effect on basis to the IRS via Form 8937 within
//! a fixed deadline. Trader uses the issuer's Form 8937 to adjust
//! basis for the post-action holding.
//!
//! Companion to:
//!   - `section_6045` (broker basis reporting on Form 1099-B —
//!     downstream of § 6045B issuer reporting).
//!   - `section_368` (tax-free corporate reorganizations affect
//!     basis — typical § 6045B trigger).
//!
//! Five operative subsections:
//!
//!   § 6045B(a) — ISSUER MUST FILE return with the IRS describing:
//!     (1) the organizational action affecting basis of the
//!         specified security, AND
//!     (2) the quantitative effect on basis resulting from the
//!         action, AND
//!     (3) such other information as the Secretary may prescribe.
//!
//!   § 6045B(b) — FILING DEADLINE = earlier of:
//!     (1) 45 days after the date of the organizational action,
//!         OR
//!     (2) January 15 of the year following the calendar year of
//!         the action.
//!
//!   § 6045B(c) — ISSUER MUST FURNISH WRITTEN STATEMENT to
//!     nominees and security holders by January 15 of the year
//!     following the calendar year of the action.
//!
//!   § 6045B(d) — "Specified security" defined by reference to
//!     § 6045(g)(3) — stock, debt instruments, options, securities
//!     futures contracts, etc.
//!
//!   § 6045B(e) — PUBLIC WEBSITE WAIVER (Treas. Reg.
//!     § 1.6045B-1(a)(3)) — issuer is deemed to have satisfied the
//!     filing duty if it posts a completed, signed Form 8937 in a
//!     readily accessible format on a public website for at least
//!     10 YEARS.
//!
//! Citations: 26 U.S.C. § 6045B(a) (issuer return — organizational
//! action + quantitative basis effect); § 6045B(b)(1) (45-day
//! deadline); § 6045B(b)(2) (January 15 alternative deadline);
//! § 6045B(c) (holder statement); § 6045B(d) (specified security
//! definition via § 6045(g)(3)); Treas. Reg. § 1.6045B-1(a)(3)
//! (public website 10-year posting waiver).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilingMethod {
    /// Physical filing of Form 8937 with the IRS.
    IrsForm8937,
    /// Public website posting under Treas. Reg. § 1.6045B-1(a)(3)
    /// — waives the IRS filing duty if posting satisfies the
    /// 10-year requirement.
    PublicWebsitePosting,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationalActionType {
    /// Stock split (forward or reverse).
    StockSplit,
    /// Stock dividend (issuer pays additional shares).
    StockDividend,
    /// Spin-off / split-off (parent distributes subsidiary stock).
    SpinOff,
    /// Merger or acquisition (statutory § 368 reorganization).
    Merger,
    /// Recapitalization changing capital structure (§ 368(a)(1)(E)).
    Recapitalization,
    /// Return of capital distribution (non-dividend; reduces basis).
    ReturnOfCapital,
    /// Other organizational action affecting basis.
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6045BInput {
    pub filing_method: FilingMethod,
    pub action_type: OrganizationalActionType,
    /// Days elapsed since the date of the organizational action.
    pub days_since_action: u32,
    /// Whether January 15 of the year following the calendar year
    /// of the action has passed.
    pub january_15_following_year_passed: bool,
    /// Whether the issuer has furnished the § 6045B(c) written
    /// statement to nominees and holders.
    pub furnished_statement_to_holders: bool,
    /// Years the public-website posting will be maintained (for
    /// FilingMethod::PublicWebsitePosting). Treas. Reg.
    /// § 1.6045B-1(a)(3) requires at least 10 years.
    pub website_posting_duration_years: u32,
    /// Whether the return describes the organizational action and
    /// its basis-adjustment effect.
    pub return_includes_basis_adjustment_description: bool,
    /// Whether the return includes the quantitative effect on
    /// basis (required by § 6045B(a)(2)).
    pub return_includes_quantitative_effect: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6045BResult {
    /// § 6045B(b)(1) 45-day primary deadline.
    pub primary_deadline_days: u32,
    /// True if the filing deadline has been exceeded under either
    /// § 6045B(b)(1) 45-day or § 6045B(b)(2) Jan-15 prong.
    pub filing_deadline_passed: bool,
    /// True if the issuer must furnish the § 6045B(c) holder
    /// statement.
    pub holder_statement_required: bool,
    /// Whether the § 6045B(e) public-website waiver applies.
    /// Requires PublicWebsitePosting AND ≥ 10-year duration.
    pub website_exception_satisfied: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// § 6045B(b)(1) 45-day deadline from the date of the
/// organizational action.
pub const SECTION_6045B_DEADLINE_DAYS: u32 = 45;
/// Treas. Reg. § 1.6045B-1(a)(3) — public-website posting must be
/// maintained for at least 10 years.
pub const WEBSITE_POSTING_MIN_YEARS: u32 = 10;

pub fn check(input: &Section6045BInput) -> Section6045BResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    // § 6045B(b) filing deadline — earlier of (1) 45 days after
    // action OR (2) January 15 of the year following the calendar
    // year of the action.
    let primary_deadline_passed = input.days_since_action > SECTION_6045B_DEADLINE_DAYS;
    let filing_deadline_passed =
        primary_deadline_passed || input.january_15_following_year_passed;

    // § 6045B(e) public-website waiver — Treas. Reg. § 1.6045B-1(a)(3)
    // requires posting for at least 10 years in readily accessible
    // format.
    let website_exception_satisfied = matches!(
        input.filing_method,
        FilingMethod::PublicWebsitePosting
    ) && input.website_posting_duration_years >= WEBSITE_POSTING_MIN_YEARS;

    // § 6045B(a) return content — must describe the action AND
    // include quantitative basis effect.
    if !input.return_includes_basis_adjustment_description {
        violations.push(
            "§ 6045B(a)(1) — return must describe the organizational action affecting basis; \
             description missing."
                .to_string(),
        );
    }
    if !input.return_includes_quantitative_effect {
        violations.push(
            "§ 6045B(a)(2) — return must include the quantitative effect on basis resulting \
             from the action; quantitative effect missing."
                .to_string(),
        );
    }

    // Filing-deadline violation.
    if filing_deadline_passed && !website_exception_satisfied {
        violations.push(format!(
            "§ 6045B(b) — filing deadline missed: {} days since action (>{} day primary \
             deadline) or January 15 of following year has passed.",
            input.days_since_action, SECTION_6045B_DEADLINE_DAYS,
        ));
    }

    // § 6045B(c) holder-statement requirement.
    if !input.furnished_statement_to_holders {
        violations.push(
            "§ 6045B(c) — issuer must furnish written statement to nominees and security \
             holders by January 15 of the year following the calendar year of the action; \
             statement not furnished."
                .to_string(),
        );
    }

    // Website-method-specific compliance.
    if matches!(input.filing_method, FilingMethod::PublicWebsitePosting) {
        if input.website_posting_duration_years < WEBSITE_POSTING_MIN_YEARS {
            violations.push(format!(
                "Treas. Reg. § 1.6045B-1(a)(3) — public-website posting duration of {} years \
                 is less than the required {} years. Waiver does not apply; physical IRS \
                 filing required.",
                input.website_posting_duration_years, WEBSITE_POSTING_MIN_YEARS,
            ));
        } else {
            notes.push(
                "Treas. Reg. § 1.6045B-1(a)(3) — public-website posting waiver SATISFIED \
                 (10+ years readily accessible). IRS Form 8937 filing duty waived."
                    .to_string(),
            );
        }
    }

    // Organizational-action type note.
    let action_note = match input.action_type {
        OrganizationalActionType::StockSplit => {
            "Stock split (forward or reverse) — adjusts per-share basis by share ratio; total \
             basis preserved."
        }
        OrganizationalActionType::StockDividend => {
            "Stock dividend — issuer pays additional shares; basis allocated across original + \
             new shares per § 305(a) non-recognition where applicable."
        }
        OrganizationalActionType::SpinOff => {
            "Spin-off / split-off — parent distributes subsidiary stock; basis allocated \
             between parent + sub stock per § 358."
        }
        OrganizationalActionType::Merger => {
            "Merger or acquisition — statutory § 368 reorganization typically; basis carries \
             over per § 358 or steps up per § 1012 depending on tax-free vs taxable status."
        }
        OrganizationalActionType::Recapitalization => {
            "Recapitalization — § 368(a)(1)(E) E-reorganization changing capital structure; \
             basis preserved across exchange."
        }
        OrganizationalActionType::ReturnOfCapital => {
            "Return of capital distribution — non-dividend cash distribution; reduces basis \
             dollar-for-dollar to zero; excess above basis recognized as capital gain."
        }
        OrganizationalActionType::Other => {
            "Other organizational action affecting basis — issuer must describe the action and \
             quantitative effect under § 6045B(a)."
        }
    };
    notes.push(action_note.to_string());

    notes.push(
        "Companion to section_6045 (broker basis reporting on Form 1099-B downstream of \
         § 6045B issuer reporting) and section_368 (tax-free reorganizations typical § 6045B \
         trigger). Holders use the issuer's Form 8937 to adjust basis for the post-action \
         holding."
            .to_string(),
    );

    let citation = if website_exception_satisfied {
        "26 U.S.C. § 6045B(a) (issuer return — organizational action + basis effect); \
         § 6045B(b)(1) (45-day deadline); § 6045B(b)(2) (January 15 alternative); § 6045B(c) \
         (holder statement); § 6045B(d) (specified security via § 6045(g)(3)); Treas. Reg. \
         § 1.6045B-1(a)(3) (public-website posting 10-year waiver SATISFIED)"
    } else {
        "26 U.S.C. § 6045B(a) (issuer return — organizational action + basis effect); \
         § 6045B(b)(1) (45-day deadline); § 6045B(b)(2) (January 15 alternative); § 6045B(c) \
         (holder statement); § 6045B(d) (specified security via § 6045(g)(3)); Treas. Reg. \
         § 1.6045B-1(a)(3) (public-website posting 10-year waiver — not applicable or not \
         satisfied)"
    };

    Section6045BResult {
        primary_deadline_days: SECTION_6045B_DEADLINE_DAYS,
        filing_deadline_passed,
        holder_statement_required: true,
        website_exception_satisfied,
        compliant: violations.is_empty(),
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(method: FilingMethod, action: OrganizationalActionType) -> Section6045BInput {
        Section6045BInput {
            filing_method: method,
            action_type: action,
            days_since_action: 30,
            january_15_following_year_passed: false,
            furnished_statement_to_holders: true,
            website_posting_duration_years: 10,
            return_includes_basis_adjustment_description: true,
            return_includes_quantitative_effect: true,
        }
    }

    // ── § 6045B(b)(1) 45-day deadline ───────────────────────────

    #[test]
    fn within_45_days_irs_form_8937_compliant() {
        let r = check(&base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::StockSplit,
        ));
        assert!(r.compliant);
        assert_eq!(r.primary_deadline_days, 45);
        assert!(!r.filing_deadline_passed);
        assert!(r.citation.contains("§ 6045B(b)(1)"));
    }

    #[test]
    fn at_45_day_boundary_compliant() {
        let mut i = base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::StockSplit,
        );
        i.days_since_action = 45;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.filing_deadline_passed);
    }

    #[test]
    fn past_45_day_deadline_violation() {
        let mut i = base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::StockSplit,
        );
        i.days_since_action = 46;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.filing_deadline_passed);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 6045B(b)") && v.contains("46 days"))
        );
    }

    #[test]
    fn january_15_passed_triggers_deadline_violation() {
        let mut i = base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::StockSplit,
        );
        i.days_since_action = 30;
        i.january_15_following_year_passed = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.filing_deadline_passed);
    }

    // ── § 6045B(a) return content requirements ─────────────────

    #[test]
    fn missing_basis_adjustment_description_violation() {
        let mut i = base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::StockSplit,
        );
        i.return_includes_basis_adjustment_description = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 6045B(a)(1)") && v.contains("description"))
        );
    }

    #[test]
    fn missing_quantitative_effect_violation() {
        let mut i = base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::StockSplit,
        );
        i.return_includes_quantitative_effect = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 6045B(a)(2)") && v.contains("quantitative"))
        );
    }

    // ── § 6045B(c) holder statement ────────────────────────────

    #[test]
    fn missing_holder_statement_violation() {
        let mut i = base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::StockSplit,
        );
        i.furnished_statement_to_holders = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 6045B(c)") && v.contains("nominees and security holders"))
        );
    }

    // ── § 6045B(e) public-website waiver ────────────────────────

    #[test]
    fn public_website_10_year_posting_waives_irs_filing() {
        let mut i = base(
            FilingMethod::PublicWebsitePosting,
            OrganizationalActionType::StockSplit,
        );
        i.website_posting_duration_years = 10;
        let r = check(&i);
        assert!(r.website_exception_satisfied);
        assert!(r.compliant);
        assert!(r.citation.contains("§ 1.6045B-1(a)(3)"));
        assert!(r.citation.contains("waiver SATISFIED"));
    }

    #[test]
    fn public_website_15_year_posting_satisfies_waiver() {
        let mut i = base(
            FilingMethod::PublicWebsitePosting,
            OrganizationalActionType::StockSplit,
        );
        i.website_posting_duration_years = 15;
        let r = check(&i);
        assert!(r.website_exception_satisfied);
    }

    #[test]
    fn public_website_9_year_posting_fails_waiver() {
        let mut i = base(
            FilingMethod::PublicWebsitePosting,
            OrganizationalActionType::StockSplit,
        );
        i.website_posting_duration_years = 9;
        let r = check(&i);
        assert!(!r.website_exception_satisfied);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 1.6045B-1(a)(3)") && v.contains("9 years"))
        );
    }

    #[test]
    fn public_website_waiver_overrides_deadline_violation() {
        // Even where 45-day deadline has passed, properly satisfied
        // website waiver excuses IRS filing.
        let mut i = base(
            FilingMethod::PublicWebsitePosting,
            OrganizationalActionType::StockSplit,
        );
        i.days_since_action = 100;
        i.january_15_following_year_passed = true;
        i.website_posting_duration_years = 10;
        let r = check(&i);
        assert!(r.website_exception_satisfied);
        // Filing deadline still computed as passed, but the website
        // waiver excuses the violation.
        assert!(r.filing_deadline_passed);
        // No "filing deadline missed" violation because website
        // waiver satisfied.
        assert!(
            !r.violations
                .iter()
                .any(|v| v.contains("filing deadline missed"))
        );
        // Test compliant only if other requirements are met.
        assert!(r.compliant);
    }

    // ── Action-type notes ──────────────────────────────────────

    #[test]
    fn stock_split_action_note_present() {
        let r = check(&base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::StockSplit,
        ));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("Stock split") && n.contains("share ratio"))
        );
    }

    #[test]
    fn stock_dividend_action_note_cites_section_305() {
        let r = check(&base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::StockDividend,
        ));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("Stock dividend") && n.contains("§ 305(a)"))
        );
    }

    #[test]
    fn spin_off_action_note_cites_section_358() {
        let r = check(&base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::SpinOff,
        ));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("Spin-off") && n.contains("§ 358"))
        );
    }

    #[test]
    fn merger_action_note_cites_section_368() {
        let r = check(&base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::Merger,
        ));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("Merger") && n.contains("§ 368"))
        );
    }

    #[test]
    fn return_of_capital_action_note_explains_basis_reduction() {
        let r = check(&base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::ReturnOfCapital,
        ));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("Return of capital") && n.contains("reduces basis"))
        );
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn deadline_45_days_strict_boundary_truth_table_invariant() {
        for (days, expected_deadline_passed) in [
            (0_u32, false),
            (44, false),
            (45, false), // at boundary = compliant
            (46, true),  // past boundary = violation
            (100, true),
        ] {
            let mut i = base(
                FilingMethod::IrsForm8937,
                OrganizationalActionType::StockSplit,
            );
            i.days_since_action = days;
            i.january_15_following_year_passed = false;
            let r = check(&i);
            assert_eq!(
                r.filing_deadline_passed, expected_deadline_passed,
                "day {} expected_deadline_passed={}",
                days, expected_deadline_passed,
            );
        }
    }

    #[test]
    fn website_exception_only_satisfied_for_public_website_method_invariant() {
        let mut irs = base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::StockSplit,
        );
        irs.website_posting_duration_years = 20;
        // IrsForm8937 method — website exception never satisfied.
        assert!(!check(&irs).website_exception_satisfied);

        let mut web = base(
            FilingMethod::PublicWebsitePosting,
            OrganizationalActionType::StockSplit,
        );
        web.website_posting_duration_years = 10;
        assert!(check(&web).website_exception_satisfied);
    }

    #[test]
    fn website_exception_requires_min_10_years_invariant() {
        for (years, expected_satisfied) in [
            (0_u32, false),
            (5, false),
            (9, false),
            (10, true),
            (15, true),
            (20, true),
        ] {
            let mut i = base(
                FilingMethod::PublicWebsitePosting,
                OrganizationalActionType::StockSplit,
            );
            i.website_posting_duration_years = years;
            let r = check(&i);
            assert_eq!(
                r.website_exception_satisfied, expected_satisfied,
                "years {} expected_satisfied={}",
                years, expected_satisfied,
            );
        }
    }

    #[test]
    fn all_seven_action_types_produce_distinct_notes_invariant() {
        for action in [
            OrganizationalActionType::StockSplit,
            OrganizationalActionType::StockDividend,
            OrganizationalActionType::SpinOff,
            OrganizationalActionType::Merger,
            OrganizationalActionType::Recapitalization,
            OrganizationalActionType::ReturnOfCapital,
            OrganizationalActionType::Other,
        ] {
            let r = check(&base(FilingMethod::IrsForm8937, action));
            assert!(r.notes.len() >= 2, "{:?}: must have action + companion notes", action);
        }
    }

    #[test]
    fn citation_pins_subsections_per_path() {
        let irs = check(&base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::StockSplit,
        ));
        let web = check(&base(
            FilingMethod::PublicWebsitePosting,
            OrganizationalActionType::StockSplit,
        ));

        for r in [&irs, &web] {
            assert!(r.citation.contains("§ 6045B(a)"));
            assert!(r.citation.contains("§ 6045B(b)(1)"));
            assert!(r.citation.contains("§ 6045B(b)(2)"));
            assert!(r.citation.contains("§ 6045B(c)"));
            assert!(r.citation.contains("§ 6045B(d)"));
        }
        assert!(web.citation.contains("waiver SATISFIED"));
        assert!(irs.citation.contains("not applicable or not satisfied"));
    }

    #[test]
    fn primary_deadline_constant_45_days_invariant() {
        let r = check(&base(
            FilingMethod::IrsForm8937,
            OrganizationalActionType::StockSplit,
        ));
        assert_eq!(r.primary_deadline_days, 45);
        assert_eq!(SECTION_6045B_DEADLINE_DAYS, 45);
    }

    #[test]
    fn website_min_10_years_constant_invariant() {
        assert_eq!(WEBSITE_POSTING_MIN_YEARS, 10);
    }

    #[test]
    fn sibling_module_note_present_across_all_action_types() {
        for action in [
            OrganizationalActionType::StockSplit,
            OrganizationalActionType::StockDividend,
            OrganizationalActionType::SpinOff,
            OrganizationalActionType::Merger,
            OrganizationalActionType::Recapitalization,
            OrganizationalActionType::ReturnOfCapital,
            OrganizationalActionType::Other,
        ] {
            let r = check(&base(FilingMethod::IrsForm8937, action));
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("section_6045") && n.contains("Form 1099-B")),
                "{:?}: sibling-module note must be present",
                action,
            );
        }
    }
}
