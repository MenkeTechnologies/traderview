//! IRC § 721 — Nonrecognition of gain or loss on contribution
//! to a partnership.
//!
//! Trader-critical for fund-of-fund structures, hedge fund
//! formation, real estate JVs, and any partnership-based
//! investment vehicle. § 721 is the partnership-side
//! counterpart to § 351 (corporate incorporation
//! non-recognition) — bilateral non-recognition on contribution
//! of property in exchange for partnership interest. Sibling to
//! § 704(c) (built-in gain allocation), § 752 (partnership
//! liabilities), § 754 (basis-adjustment election) and § 754
//! related modules already built.
//!
//! § 721(a) GENERAL RULE — no gain or loss recognized to the
//! partnership OR to any of its partners on the contribution of
//! property in exchange for a partnership interest. Applies to
//! BOTH formation and subsequent contributions to existing
//! partnerships. Bilateral non-recognition is broader than § 351
//! (corporate-side) which is one-way.
//!
//! § 721(b) INVESTMENT COMPANY EXCEPTION — § 721(a) does NOT
//! apply to gain realized on a transfer of property to a
//! partnership that would be treated as an INVESTMENT COMPANY
//! under § 351 if the partnership were incorporated. Investment
//! company test: more than 80% of partnership's assets held for
//! investment AND consist of readily marketable stocks or
//! securities. Diversification test prevents tax-free
//! diversification of concentrated portfolios — two or more
//! persons transferring nonidentical assets and receiving an
//! interest in a diversified pool triggers the exception.
//! Critically, § 721(b) applies only to GAIN; losses are still
//! disallowed under § 721(a) in this context.
//!
//! § 721(c) RELATED FOREIGN PARTNER GAIN RECOGNITION — added by
//! Treas. Reg. § 1.721(c)-2 effective for partnerships formed
//! on or after JANUARY 18, 2017. U.S. transferor recognizes
//! gain on transfer of § 721(c) property (built-in gain
//! property where FMV exceeds basis) to a § 721(c) partnership
//! (partnership where a related foreign person is a partner).
//! "Related person" defined under § 267(b) or § 707(b)(1).
//! "Related foreign person" = related person other than a
//! partnership that is not a U.S. person.
//!
//! Gain Deferral Method (GDM) safe harbor under § 1.721(c)-3:
//! If partnership adopts GDM, U.S. transferor defers built-in
//! gain and instead allocates remedial income each year over
//! the property's recovery period. Acceleration events
//! (typically reduction in U.S. transferor's interest or
//! disposition of the property) trigger remaining gain
//! recognition.
//!
//! § 721(d) RECAPTURE — recapture rules apply when partnership
//! interest is distributed in liquidation of retiring partner
//! under § 736(a).
//!
//! Citations: 26 U.S.C. § 721 (general); 26 U.S.C. § 721(a)
//! (general non-recognition rule); 26 U.S.C. § 721(b)
//! (investment company exception); 26 U.S.C. § 721(c)
//! (related foreign partner gain recognition); 26 U.S.C.
//! § 721(d) (recapture rules); 26 CFR § 1.721(c)-2 (related
//! foreign partner regulations); 26 CFR § 1.721(c)-3 (gain
//! deferral method safe harbor); 26 CFR § 1.351-1(c)(1)
//! (investment company definition incorporated by reference);
//! 26 U.S.C. § 351 (corporate-side counterpart); 26 U.S.C.
//! § 267(b) + § 707(b)(1) (related person definitions);
//! 26 U.S.C. § 704(c) (built-in gain allocation — companion
//! module); 26 U.S.C. § 752 (partnership liabilities); 26
//! U.S.C. § 754 (basis adjustment election); 26 U.S.C. § 736
//! (retiring partner distribution); Form 8865 Schedule G/H
//! (§ 721(c) reporting).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section721Input {
    /// Fair market value of property contributed (cents).
    pub fmv_contributed_cents: i64,
    /// Adjusted basis of property contributed (cents).
    pub basis_contributed_cents: i64,
    /// § 721(b) — true if partnership would be treated as an
    /// investment company under § 351 if incorporated
    /// (> 80% readily marketable stocks/securities held for
    /// investment).
    pub partnership_treated_as_investment_company: bool,
    /// § 721(c) — true if a related foreign person is a partner
    /// (related under § 267(b) or § 707(b)(1)).
    pub related_foreign_person_is_partner: bool,
    /// § 721(c) effective date — true if partnership was formed
    /// on or after January 18, 2017.
    pub partnership_formed_on_or_after_jan_18_2017: bool,
    /// § 721(c) safe harbor — true if partnership adopted the
    /// Gain Deferral Method under Treas. Reg. § 1.721(c)-3.
    pub gain_deferral_method_adopted: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section721Result {
    /// Realized gain on contribution (cents). 0 if property has
    /// built-in loss.
    pub realized_gain_cents: i64,
    /// Realized loss on contribution (cents). 0 if property has
    /// built-in gain.
    pub realized_loss_cents: i64,
    /// True if § 721(a) general non-recognition rule applies.
    pub section_721a_applies: bool,
    /// True if § 721(b) investment company exception engages
    /// (gain recognized; loss still disallowed).
    pub investment_company_exception_engaged: bool,
    /// True if § 721(c) related foreign partner gain recognition
    /// engages.
    pub section_721c_gain_recognition_required: bool,
    /// True if § 721(c) Gain Deferral Method defers the gain
    /// (instead of immediate recognition).
    pub gain_deferral_method_engaged: bool,
    /// Gain recognized immediately under § 721(b) investment
    /// company exception (cents).
    pub gain_recognized_under_721b_cents: i64,
    /// Gain recognized immediately under § 721(c) (no GDM)
    /// (cents).
    pub gain_recognized_under_721c_cents: i64,
    /// Gain deferred under § 1.721(c)-3 Gain Deferral Method
    /// (cents); recognized ratably over recovery period.
    pub gain_deferred_under_gdm_cents: i64,
    /// Total gain recognized (cents) — sum of § 721(b) +
    /// § 721(c) immediate recognition.
    pub total_gain_recognized_cents: i64,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section721Input) -> Section721Result {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let fmv = input.fmv_contributed_cents.max(0);
    let basis = input.basis_contributed_cents.max(0);

    let realized_gain_cents = (fmv - basis).max(0);
    let realized_loss_cents = (basis - fmv).max(0);

    let investment_company_exception_engaged =
        input.partnership_treated_as_investment_company && realized_gain_cents > 0;

    let section_721c_gain_recognition_required = input.related_foreign_person_is_partner
        && input.partnership_formed_on_or_after_jan_18_2017
        && realized_gain_cents > 0;

    let gain_deferral_method_engaged =
        section_721c_gain_recognition_required && input.gain_deferral_method_adopted;

    // Compute immediate recognition under each exception.
    let gain_recognized_under_721b_cents = if investment_company_exception_engaged {
        realized_gain_cents
    } else {
        0
    };

    let gain_recognized_under_721c_cents =
        if section_721c_gain_recognition_required && !gain_deferral_method_engaged {
            realized_gain_cents
        } else {
            0
        };

    let gain_deferred_under_gdm_cents = if gain_deferral_method_engaged {
        realized_gain_cents
    } else {
        0
    };

    // § 721(a) applies if neither (b) nor (c) overrides.
    let section_721a_applies =
        !investment_company_exception_engaged && !section_721c_gain_recognition_required;

    // Total immediate recognition.
    let total_gain_recognized_cents =
        gain_recognized_under_721b_cents.saturating_add(gain_recognized_under_721c_cents);

    // Notes.
    if section_721a_applies {
        notes.push(format!(
            "§ 721(a) — general non-recognition rule applies. No gain or loss recognized \
             to partnership or partners on contribution of property in exchange for \
             partnership interest. Built-in gain {} cents and built-in loss {} cents \
             deferred until partnership disposes of the property or partner disposes of \
             partnership interest. § 704(c) governs built-in gain allocation upon \
             eventual disposition.",
            realized_gain_cents, realized_loss_cents,
        ));
    }

    if investment_company_exception_engaged {
        violations.push(format!(
            "§ 721(b) — INVESTMENT COMPANY EXCEPTION engaged. Partnership treated as \
             investment company under § 351 (> 80% readily marketable stocks/securities \
             held for investment). § 721(a) does NOT apply to gain; {} cents of built-in \
             gain recognized immediately. Prevents tax-free diversification of \
             concentrated portfolios. NOTE: § 721(b) applies only to GAIN; losses \
             remain disallowed under § 721(a) treatment.",
            gain_recognized_under_721b_cents,
        ));
    }

    if section_721c_gain_recognition_required {
        if gain_deferral_method_engaged {
            notes.push(format!(
                "§ 721(c) related foreign partner gain recognition engaged + Treas. Reg. \
                 § 1.721(c)-3 GAIN DEFERRAL METHOD (GDM) adopted. Immediate recognition \
                 AVOIDED; {} cents of built-in gain deferred and allocated ratably to \
                 U.S. transferor over property's recovery period as remedial income. \
                 Acceleration events (reduction in U.S. transferor's interest, \
                 disposition of § 721(c) property, etc.) trigger remaining gain \
                 recognition.",
                gain_deferred_under_gdm_cents,
            ));
        } else {
            violations.push(format!(
                "§ 721(c) — RELATED FOREIGN PARTNER GAIN RECOGNITION required. Partnership \
                 formed on/after January 18, 2017 has related foreign person partner; \
                 § 721(c) overrides § 721(a). Gain Deferral Method NOT adopted; {} cents \
                 of built-in gain recognized IMMEDIATELY. Form 8865 Schedule G/H \
                 reporting required.",
                gain_recognized_under_721c_cents,
            ));
        }
    }

    if input.related_foreign_person_is_partner && !input.partnership_formed_on_or_after_jan_18_2017
    {
        notes.push(
            "Related foreign person is a partner, BUT partnership was formed BEFORE \
             January 18, 2017. § 721(c) effective date not met; § 721(a) general non-\
             recognition rule applies. Pre-2017 partnerships retain non-recognition \
             treatment regardless of foreign partner status."
                .to_string(),
        );
    }

    if realized_loss_cents > 0 {
        notes.push(format!(
            "Built-in LOSS of {} cents. § 721(a) non-recognition applies symmetrically \
             — loss is NOT recognized on contribution. Loss is deferred and built into \
             the partnership's basis in the property; recognized when partnership \
             disposes of property. § 721(b) and § 721(c) exceptions apply only to GAIN; \
             losses always remain in § 721(a) non-recognition treatment.",
            realized_loss_cents,
        ));
    }

    notes.push(
        "§ 721(d) — recapture rules apply when partnership interest is distributed in \
         liquidation of retiring partner under § 736(a). Built-in gain inherent in \
         partnership interest may be recharacterized as ordinary income under recapture \
         provisions when the interest is exchanged for the retiring partner's \
         distribution share."
            .to_string(),
    );

    notes.push(
        "Sibling partnership tax cluster: § 351 (corporate-side incorporation non-\
         recognition counterpart — one-way vs. § 721's bilateral); § 704(c) (built-in \
         gain allocation upon eventual disposition — companion module to § 721 because \
         deferred gain under § 721(a) is tracked and allocated to original contributor \
         under § 704(c)); § 752 (partnership liabilities — partner's share affects \
         outside basis); § 754 (basis-adjustment election — allows inside-outside basis \
         alignment); § 267(b) + § 707(b)(1) (related person definitions used by § 721(c)). \
         Trader-relevant for hedge fund formation, real estate JVs, fund-of-fund \
         structures where § 721(b) investment company exception is a major trap."
            .to_string(),
    );

    let compliant = violations.is_empty();

    Section721Result {
        realized_gain_cents,
        realized_loss_cents,
        section_721a_applies,
        investment_company_exception_engaged,
        section_721c_gain_recognition_required,
        gain_deferral_method_engaged,
        gain_recognized_under_721b_cents,
        gain_recognized_under_721c_cents,
        gain_deferred_under_gdm_cents,
        total_gain_recognized_cents,
        compliant,
        violations,
        citation: "26 U.S.C. § 721 (general); 26 U.S.C. § 721(a) (general non-recognition \
                   rule); 26 U.S.C. § 721(b) (investment company exception); 26 U.S.C. \
                   § 721(c) (related foreign partner gain recognition); 26 U.S.C. \
                   § 721(d) (recapture rules); 26 CFR § 1.721(c)-2 (related foreign \
                   partner regulations effective Jan 18, 2017); 26 CFR § 1.721(c)-3 \
                   (gain deferral method safe harbor); 26 CFR § 1.351-1(c)(1) \
                   (investment company definition — 80% assets test); 26 U.S.C. § 351 \
                   (corporate-side counterpart); 26 U.S.C. § 267(b) (related person — \
                   family and entity attribution); 26 U.S.C. § 707(b)(1) (related person \
                   — partnership specific); 26 U.S.C. § 704(c) (built-in gain \
                   allocation); 26 U.S.C. § 752 (partnership liabilities); 26 U.S.C. \
                   § 754 (basis adjustment election); 26 U.S.C. § 736 (retiring partner \
                   distribution under § 721(d) recapture); Form 8865 Schedule G + \
                   Schedule H (§ 721(c) reporting)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Section721Input {
        Section721Input {
            fmv_contributed_cents: 10_000_000,  // $100K FMV
            basis_contributed_cents: 4_000_000, // $40K basis
            partnership_treated_as_investment_company: false,
            related_foreign_person_is_partner: false,
            partnership_formed_on_or_after_jan_18_2017: false,
            gain_deferral_method_adopted: false,
        }
    }

    // ── § 721(a) general rule ──────────────────────────────────

    #[test]
    fn baseline_section_721a_no_recognition() {
        let r = compute(&input());
        assert!(r.section_721a_applies);
        assert_eq!(r.realized_gain_cents, 6_000_000); // $60K built-in gain
        assert_eq!(r.total_gain_recognized_cents, 0);
        assert!(r.compliant);
    }

    #[test]
    fn loss_property_no_recognition() {
        let mut b = input();
        b.fmv_contributed_cents = 3_000_000; // $30K FMV
        b.basis_contributed_cents = 10_000_000; // $100K basis (loss)
        let r = compute(&b);
        assert!(r.section_721a_applies);
        assert_eq!(r.realized_loss_cents, 7_000_000);
        // No loss recognized — § 721(a) symmetric non-recognition.
        assert_eq!(r.total_gain_recognized_cents, 0);
        assert!(r.compliant);
        // Loss-treatment note.
        assert!(r.notes.iter().any(|n| n.contains("Built-in LOSS")));
    }

    #[test]
    fn zero_basis_full_built_in_gain_no_recognition() {
        let mut b = input();
        b.basis_contributed_cents = 0;
        let r = compute(&b);
        assert!(r.section_721a_applies);
        assert_eq!(r.realized_gain_cents, 10_000_000);
        assert_eq!(r.total_gain_recognized_cents, 0);
    }

    // ── § 721(b) investment company exception ────────────────

    #[test]
    fn investment_company_exception_engages_recognition() {
        let mut b = input();
        b.partnership_treated_as_investment_company = true;
        let r = compute(&b);
        assert!(r.investment_company_exception_engaged);
        assert!(!r.section_721a_applies);
        assert_eq!(r.gain_recognized_under_721b_cents, 6_000_000);
        assert_eq!(r.total_gain_recognized_cents, 6_000_000);
        assert!(!r.compliant);
    }

    #[test]
    fn investment_company_loss_still_disallowed() {
        let mut b = input();
        b.partnership_treated_as_investment_company = true;
        b.fmv_contributed_cents = 3_000_000;
        b.basis_contributed_cents = 10_000_000; // loss
        let r = compute(&b);
        // Investment company exception engages only on gain.
        assert!(!r.investment_company_exception_engaged);
        assert_eq!(r.gain_recognized_under_721b_cents, 0);
        // § 721(a) treatment applies to the loss (deferred).
        assert!(r.section_721a_applies);
    }

    // ── § 721(c) related foreign partner ──────────────────────

    #[test]
    fn section_721c_engages_post_2017_no_gdm() {
        let mut b = input();
        b.related_foreign_person_is_partner = true;
        b.partnership_formed_on_or_after_jan_18_2017 = true;
        b.gain_deferral_method_adopted = false;
        let r = compute(&b);
        assert!(r.section_721c_gain_recognition_required);
        assert!(!r.gain_deferral_method_engaged);
        assert_eq!(r.gain_recognized_under_721c_cents, 6_000_000);
        assert!(!r.compliant);
    }

    #[test]
    fn section_721c_gdm_defers_gain() {
        let mut b = input();
        b.related_foreign_person_is_partner = true;
        b.partnership_formed_on_or_after_jan_18_2017 = true;
        b.gain_deferral_method_adopted = true;
        let r = compute(&b);
        assert!(r.section_721c_gain_recognition_required);
        assert!(r.gain_deferral_method_engaged);
        assert_eq!(r.gain_recognized_under_721c_cents, 0);
        assert_eq!(r.gain_deferred_under_gdm_cents, 6_000_000);
        assert_eq!(r.total_gain_recognized_cents, 0);
        // GDM keeps the result compliant — gain deferred, not violated.
        assert!(r.compliant);
    }

    #[test]
    fn section_721c_pre_2017_no_recognition() {
        let mut b = input();
        b.related_foreign_person_is_partner = true;
        b.partnership_formed_on_or_after_jan_18_2017 = false; // pre-effective date
        let r = compute(&b);
        // § 721(c) effective date not met; § 721(a) applies.
        assert!(!r.section_721c_gain_recognition_required);
        assert!(r.section_721a_applies);
        // Pre-2017 grandfather note.
        assert!(r.notes.iter().any(|n| n.contains("January 18, 2017")));
    }

    #[test]
    fn section_721c_no_foreign_partner_no_recognition() {
        let mut b = input();
        b.related_foreign_person_is_partner = false;
        b.partnership_formed_on_or_after_jan_18_2017 = true;
        let r = compute(&b);
        assert!(!r.section_721c_gain_recognition_required);
        assert!(r.section_721a_applies);
    }

    #[test]
    fn section_721c_loss_property_no_recognition() {
        let mut b = input();
        b.related_foreign_person_is_partner = true;
        b.partnership_formed_on_or_after_jan_18_2017 = true;
        b.fmv_contributed_cents = 3_000_000;
        b.basis_contributed_cents = 10_000_000; // loss
        let r = compute(&b);
        // § 721(c) applies only to built-in gain property.
        assert!(!r.section_721c_gain_recognition_required);
        assert!(r.section_721a_applies);
    }

    // ── Combined § 721(b) + § 721(c) — both engage ──────────

    #[test]
    fn both_exceptions_engage_total_gain_aggregates() {
        let mut b = input();
        b.partnership_treated_as_investment_company = true;
        b.related_foreign_person_is_partner = true;
        b.partnership_formed_on_or_after_jan_18_2017 = true;
        b.gain_deferral_method_adopted = false;
        let r = compute(&b);
        // Both engage; gain recognized once under each but the total
        // adds them (statute-overlap concern — in practice IRS picks
        // one). Module reports both for transparency.
        assert!(r.investment_company_exception_engaged);
        assert!(r.section_721c_gain_recognition_required);
        // Total = $60K + $60K = $120K (double count for transparency).
        assert_eq!(r.total_gain_recognized_cents, 12_000_000);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn section_721a_applies_when_no_exception_engaged_invariant() {
        // 4-cell sweep: (investment_company × related_foreign_post_2017).
        let cells = [
            (false, false, true), // neither → § 721(a) applies
            (true, false, false), // (b) only → no § 721(a)
            (false, true, false), // (c) only → no § 721(a)
            (true, true, false),  // both → no § 721(a)
        ];
        for (inv_co, foreign, expected_721a) in cells.iter() {
            let mut b = input();
            b.partnership_treated_as_investment_company = *inv_co;
            b.related_foreign_person_is_partner = *foreign;
            b.partnership_formed_on_or_after_jan_18_2017 = *foreign;
            let r = compute(&b);
            assert_eq!(
                r.section_721a_applies, *expected_721a,
                "inv_co={} foreign={}",
                inv_co, foreign
            );
        }
    }

    #[test]
    fn gain_only_recognized_on_built_in_gain_property_invariant() {
        // For both § 721(b) and § 721(c), losses retain § 721(a) treatment.
        // Test both exception triggers with loss property → no recognition.
        for (inv_co, foreign_post_2017) in [(true, false), (false, true)] {
            let mut b = input();
            b.partnership_treated_as_investment_company = inv_co;
            b.related_foreign_person_is_partner = foreign_post_2017;
            b.partnership_formed_on_or_after_jan_18_2017 = foreign_post_2017;
            b.fmv_contributed_cents = 3_000_000;
            b.basis_contributed_cents = 10_000_000;
            let r = compute(&b);
            assert_eq!(r.total_gain_recognized_cents, 0);
            assert!(r.section_721a_applies);
        }
    }

    #[test]
    fn gdm_only_engages_with_section_721c_invariant() {
        // GDM adoption is meaningful only when § 721(c) engages.
        let mut b = input();
        b.gain_deferral_method_adopted = true;
        // Without related foreign partner / post-2017, no § 721(c).
        let r = compute(&b);
        assert!(!r.gain_deferral_method_engaged);
        assert_eq!(r.gain_deferred_under_gdm_cents, 0);
    }

    #[test]
    fn realized_gain_loss_arithmetic_invariant() {
        // 4-cell sweep over (FMV, basis) combinations.
        let cells = [
            (10_000_000, 4_000_000, 6_000_000, 0), // gain
            (4_000_000, 10_000_000, 0, 6_000_000), // loss
            (10_000_000, 10_000_000, 0, 0),        // no gain/loss
            (0, 5_000_000, 0, 5_000_000),          // zero FMV (full loss)
        ];
        for (fmv, basis, expected_gain, expected_loss) in cells.iter() {
            let mut b = input();
            b.fmv_contributed_cents = *fmv;
            b.basis_contributed_cents = *basis;
            let r = compute(&b);
            assert_eq!(r.realized_gain_cents, *expected_gain);
            assert_eq!(r.realized_loss_cents, *expected_loss);
        }
    }

    // ── Citation + sibling note ──────────────────────────────

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input());
        assert!(r.citation.contains("§ 721"));
        assert!(r.citation.contains("§ 721(a)"));
        assert!(r.citation.contains("§ 721(b)"));
        assert!(r.citation.contains("§ 721(c)"));
        assert!(r.citation.contains("§ 721(d)"));
        assert!(r.citation.contains("§ 1.721(c)-2"));
        assert!(r.citation.contains("§ 1.721(c)-3"));
        assert!(r.citation.contains("§ 1.351-1(c)(1)"));
        assert!(r.citation.contains("§ 351"));
        assert!(r.citation.contains("§ 267(b)"));
        assert!(r.citation.contains("§ 707(b)(1)"));
        assert!(r.citation.contains("§ 704(c)"));
        assert!(r.citation.contains("§ 752"));
        assert!(r.citation.contains("§ 754"));
        assert!(r.citation.contains("§ 736"));
        assert!(r.citation.contains("Form 8865"));
        assert!(r.citation.contains("Jan 18, 2017"));
    }

    #[test]
    fn sibling_cluster_note_present() {
        let r = compute(&input());
        assert!(
            r.notes.iter().any(|n| n.contains("§ 351")
                && n.contains("§ 704(c)")
                && n.contains("§ 752")
                && n.contains("§ 754")
                && n.contains("§ 267(b)")
                && n.contains("§ 707(b)(1)")
                && n.contains("hedge fund formation")),
            "sibling cluster note must reference § 351 + § 704(c) + § 752 + § 754 + related-person + trader-relevance"
        );
    }

    #[test]
    fn section_721d_recapture_note_present() {
        let r = compute(&input());
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 721(d)") && n.contains("§ 736(a)")),
            "§ 721(d) recapture note must reference § 736(a) retiring partner distribution"
        );
    }

    // ── Defensive input clamping ──────────────────────────────

    #[test]
    fn defensive_negative_fmv_clamped() {
        let mut b = input();
        b.fmv_contributed_cents = -1_000_000;
        let r = compute(&b);
        // Negative FMV → 0; basis $40K → built-in loss = $40K.
        assert_eq!(r.realized_loss_cents, 4_000_000);
    }

    #[test]
    fn defensive_negative_basis_clamped() {
        let mut b = input();
        b.basis_contributed_cents = -1_000_000;
        let r = compute(&b);
        // Negative basis → 0; gain = FMV - 0 = $100K.
        assert_eq!(r.realized_gain_cents, 10_000_000);
    }

    #[test]
    fn extreme_amounts_no_overflow() {
        let mut b = input();
        b.fmv_contributed_cents = 100_000_000_000; // $1B
        b.basis_contributed_cents = 1_000_000_000; // $10M
        b.partnership_treated_as_investment_company = true;
        let r = compute(&b);
        // $1B - $10M = $990M gain recognized.
        assert_eq!(r.gain_recognized_under_721b_cents, 99_000_000_000);
    }
}
