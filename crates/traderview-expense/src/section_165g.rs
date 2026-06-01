//! IRC §165(g) — Worthless securities.
//!
//! When a stock, bond, debenture, or other security held as a
//! capital asset becomes **wholly worthless** during the taxable
//! year, §165(g)(1) treats the loss as if the security were sold or
//! exchanged on the **last day of the taxable year**. The character
//! is generally capital (long-term or short-term depending on the
//! holding period through that deemed sale date) — with a critical
//! exception under §165(g)(3) for stock of a domestic affiliated
//! corporation that converts the loss to **ordinary**.
//!
//! **§165(g)(1) general rule**: capital-asset security wholly
//! worthless → deemed sold on last day of taxable year → capital
//! loss (ST or LT per holding period through Dec 31).
//!
//! **§165(g)(2) definition of "security"**:
//! - share of stock in a corporation
//! - right to subscribe for / receive a share of stock
//! - bond, debenture, note, certificate, or other evidence of
//!   indebtedness issued by a corporation or government with
//!   interest coupons or in registered form
//!
//! **§165(g)(3) affiliated-corporation ordinary loss exception**:
//! Stock of a domestic corporation affiliated with the taxpayer
//! (within §1504(a)(2) — 80% of vote AND value owned directly by
//! taxpayer) is treated as **not a capital asset**, converting any
//! worthlessness loss to **ordinary**. Two additional gates:
//!
//! - Domestic-corporation requirement (foreign subsidiaries are
//!   capital loss only).
//! - More than **90% of aggregate gross receipts** of the
//!   corporation for ALL taxable years must be from sources OTHER
//!   THAN royalties, rents, dividends, interest, annuities, and
//!   gains from sales or exchanges of stocks and securities (i.e.,
//!   primarily operating not passive).
//!
//! **Wholly-worthless requirement**: §165(g) is unforgiving on
//! partial worthlessness — a security that has merely declined in
//! value but retains some economic value does NOT qualify. The
//! taxpayer must wait to dispose of the security under §1001 or
//! until total worthlessness occurs.
//!
//! **Interaction with §1244 small business stock**: when a
//! qualifying §1244 stock becomes worthless, §1244's ordinary-loss
//! rules (up to $50k single / $100k MFJ per year) take precedence
//! over the §165(g)(1) capital-loss treatment. §165(g)(3) ordinary
//! loss similarly takes precedence over §165(g)(1) capital
//! treatment for affiliated stock. Both ordinary paths bypass the
//! capital-loss net-on-net annual limitation under §1211.
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 165](https://www.law.cornell.edu/uscode/text/26/165),
//! [Cornell LII 26 CFR § 1.165-5 worthless securities](https://www.law.cornell.edu/cfr/text/26/1.165-5),
//! [The Tax Adviser — Determining gross receipts under §165(g)(3)](https://www.thetaxadviser.com/issues/2023/feb/determining-gross-receipts-under-sec-165-g-3/),
//! [WLRK — Another Look Through the Worthless Stock Deduction: §165(g)(3)](https://www.wlrk.com/webdocs/wlrknew/AttorneyPubs/WLRK.22329.12.pdf).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterOfLoss {
    /// §165(g)(1) capital loss with short-term holding period.
    ShortTermCapital,
    /// §165(g)(1) capital loss with long-term holding period.
    LongTermCapital,
    /// §165(g)(3) affiliated-domestic-corporation ordinary loss.
    OrdinaryUnder165g3Affiliated,
    /// §1244 small business stock ordinary loss (up to $50k single
    /// / $100k MFJ — actual cap handled by `section_1244` module;
    /// this character pin signals which rule path was selected).
    OrdinaryUnderSection1244,
    /// Security not yet wholly worthless — no §165(g) deduction
    /// available this year.
    NoDeductionPartialWorthlessness,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section165gInput {
    pub tax_year: i32,
    /// Adjusted basis in the security at the time of worthlessness.
    pub security_adjusted_basis_dollars: i64,
    /// Whether the security has become WHOLLY worthless during the
    /// tax year. Partial decline in value does not qualify.
    pub security_wholly_worthless: bool,
    /// Holding period in days as of the deemed sale date
    /// (December 31 of the tax year). > 365 = long-term.
    pub holding_period_days_through_year_end: u32,
    /// True if the security qualifies under §1244 small business
    /// stock — ordinary loss treatment takes precedence over the
    /// §165(g)(1) capital-loss path.
    pub qualifies_under_section_1244: bool,
    /// True if the issuing corporation is domestic (required for
    /// §165(g)(3) ordinary-loss path).
    pub corporation_is_domestic: bool,
    /// True if the taxpayer directly owns ≥ 80% of total voting
    /// power AND ≥ 80% of total value of the corporation's stock
    /// (§1504(a)(2) affiliation test).
    pub taxpayer_meets_80_pct_affiliation_test: bool,
    /// Percentage of corporation's aggregate gross receipts (across
    /// all tax years) from non-passive sources (operating receipts
    /// — NOT royalties / rents / dividends / interest / annuities /
    /// securities-sale gains). In basis points (9000 = 90.00%).
    pub non_passive_gross_receipts_pct_bp: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section165gResult {
    pub deduction_available: bool,
    pub character_of_loss: CharacterOfLoss,
    pub loss_amount_dollars: i64,
    /// True if the §165(g)(3) affiliated ordinary-loss path applies.
    pub section_165g3_ordinary_path_applies: bool,
    /// True if §1244 small business stock ordinary-loss path applies
    /// (handled by the `section_1244` module for the $50k/$100k
    /// annual cap math).
    pub section_1244_ordinary_path_applies: bool,
    /// Deemed sale date — always last day of the tax year under
    /// §165(g)(1) when applicable.
    pub deemed_sale_date_iso: String,
    pub citation: String,
    pub note: String,
}

const LONG_TERM_HOLDING_DAYS: u32 = 365;
const SECTION_165G3_GROSS_RECEIPTS_THRESHOLD_BP: u32 = 9000; // > 90%

pub fn compute(input: &Section165gInput) -> Section165gResult {
    if !input.security_wholly_worthless {
        return Section165gResult {
            deduction_available: false,
            character_of_loss: CharacterOfLoss::NoDeductionPartialWorthlessness,
            loss_amount_dollars: 0,
            section_165g3_ordinary_path_applies: false,
            section_1244_ordinary_path_applies: false,
            deemed_sale_date_iso: String::new(),
            citation:
                "IRC §165(g)(1) — wholly worthless requirement not met; partial decline in value does not give rise to a §165(g) deduction; taxpayer must wait for total worthlessness or dispose under §1001"
                    .to_string(),
            note: format!(
                "Tax year {}; security adjusted basis ${} but NOT wholly worthless — no §165(g) deduction available this year; partial loss must be deferred until sale or total worthlessness.",
                input.tax_year, input.security_adjusted_basis_dollars,
            ),
        };
    }

    let loss = input.security_adjusted_basis_dollars.max(0);

    // §165(g)(3) gates: domestic + §1504(a)(2) 80% + > 90%
    // non-passive gross receipts.
    let s165g3_applies = input.corporation_is_domestic
        && input.taxpayer_meets_80_pct_affiliation_test
        && input.non_passive_gross_receipts_pct_bp > SECTION_165G3_GROSS_RECEIPTS_THRESHOLD_BP;

    let s1244_applies = input.qualifies_under_section_1244;

    // Priority: §1244 first (ordinary up to cap, see section_1244),
    // then §165(g)(3) ordinary (no cap), then §165(g)(1) capital.
    let character = if s1244_applies {
        CharacterOfLoss::OrdinaryUnderSection1244
    } else if s165g3_applies {
        CharacterOfLoss::OrdinaryUnder165g3Affiliated
    } else if input.holding_period_days_through_year_end > LONG_TERM_HOLDING_DAYS {
        CharacterOfLoss::LongTermCapital
    } else {
        CharacterOfLoss::ShortTermCapital
    };

    let deemed_date = format!("{}-12-31", input.tax_year);

    let character_label = match character {
        CharacterOfLoss::ShortTermCapital => "short-term capital loss",
        CharacterOfLoss::LongTermCapital => "long-term capital loss",
        CharacterOfLoss::OrdinaryUnder165g3Affiliated => {
            "ordinary loss under §165(g)(3) affiliated domestic corporation exception"
        }
        CharacterOfLoss::OrdinaryUnderSection1244 => {
            "ordinary loss under §1244 small business stock — see section_1244 module for $50k single / $100k MFJ cap"
        }
        CharacterOfLoss::NoDeductionPartialWorthlessness => "no deduction",
    };

    let note = format!(
        "Tax year {}; security adjusted basis ${} wholly worthless; deemed sold {} per §165(g)(1) — ${} {}; holding period {} days through year-end (>{} required for LT); §165(g)(3) gates: domestic={}, 80% affiliation={}, non-passive gross receipts {}.{}% (>{}.0% required) → §165(g)(3) {}.",
        input.tax_year,
        input.security_adjusted_basis_dollars,
        deemed_date,
        loss,
        character_label,
        input.holding_period_days_through_year_end,
        LONG_TERM_HOLDING_DAYS,
        input.corporation_is_domestic,
        input.taxpayer_meets_80_pct_affiliation_test,
        input.non_passive_gross_receipts_pct_bp / 100,
        input.non_passive_gross_receipts_pct_bp % 100,
        SECTION_165G3_GROSS_RECEIPTS_THRESHOLD_BP / 100,
        if s165g3_applies { "APPLIES" } else { "does NOT apply" },
    );

    Section165gResult {
        deduction_available: true,
        character_of_loss: character,
        loss_amount_dollars: loss,
        section_165g3_ordinary_path_applies: s165g3_applies,
        section_1244_ordinary_path_applies: s1244_applies,
        deemed_sale_date_iso: deemed_date,
        citation:
            "IRC §165(g)(1) wholly worthless security deemed sold on last day of taxable year; §165(g)(2) security definition (stock / stock rights / bond / debenture / registered indebtedness); §165(g)(3) affiliated-domestic-corporation ordinary loss exception (§1504(a)(2) 80%/80% ownership + > 90% non-passive gross receipts test); §1244 small business stock ordinary loss priority (annual cap handled by section_1244); 26 CFR § 1.165-5 worthless securities regulations"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section165gInput {
        Section165gInput {
            tax_year: 2026,
            security_adjusted_basis_dollars: 100_000,
            security_wholly_worthless: true,
            holding_period_days_through_year_end: 400,
            qualifies_under_section_1244: false,
            corporation_is_domestic: false,
            taxpayer_meets_80_pct_affiliation_test: false,
            non_passive_gross_receipts_pct_bp: 0,
        }
    }

    // ── §165(g)(1) baseline capital loss ───────────────────────────

    #[test]
    fn baseline_long_term_capital_loss() {
        // 400-day holding period → LT capital loss; non-affiliated.
        let r = compute(&base());
        assert!(r.deduction_available);
        assert_eq!(r.character_of_loss, CharacterOfLoss::LongTermCapital);
        assert_eq!(r.loss_amount_dollars, 100_000);
        assert_eq!(r.deemed_sale_date_iso, "2026-12-31");
    }

    #[test]
    fn short_holding_period_short_term_capital() {
        let mut i = base();
        i.holding_period_days_through_year_end = 300;
        let r = compute(&i);
        assert_eq!(r.character_of_loss, CharacterOfLoss::ShortTermCapital);
    }

    #[test]
    fn holding_period_boundary_365_days_is_short_term() {
        // §1222 LT requires holding period of MORE THAN one year.
        let mut i = base();
        i.holding_period_days_through_year_end = 365;
        let r = compute(&i);
        assert_eq!(r.character_of_loss, CharacterOfLoss::ShortTermCapital);
    }

    #[test]
    fn holding_period_boundary_366_days_is_long_term() {
        let mut i = base();
        i.holding_period_days_through_year_end = 366;
        let r = compute(&i);
        assert_eq!(r.character_of_loss, CharacterOfLoss::LongTermCapital);
    }

    // ── §165(g)(1) deemed sale date ─────────────────────────────────

    #[test]
    fn deemed_sale_date_is_dec_31_of_year() {
        let r = compute(&base());
        assert_eq!(r.deemed_sale_date_iso, "2026-12-31");
    }

    #[test]
    fn deemed_sale_date_tracks_year_input() {
        let mut i = base();
        i.tax_year = 2030;
        let r = compute(&i);
        assert_eq!(r.deemed_sale_date_iso, "2030-12-31");
    }

    // ── Wholly-worthless requirement ───────────────────────────────

    #[test]
    fn partial_worthlessness_no_deduction() {
        let mut i = base();
        i.security_wholly_worthless = false;
        let r = compute(&i);
        assert!(!r.deduction_available);
        assert_eq!(
            r.character_of_loss,
            CharacterOfLoss::NoDeductionPartialWorthlessness
        );
        assert_eq!(r.loss_amount_dollars, 0);
    }

    #[test]
    fn partial_worthlessness_note_explains_deferral() {
        let mut i = base();
        i.security_wholly_worthless = false;
        let r = compute(&i);
        assert!(r.note.contains("NOT wholly worthless"));
    }

    // ── §165(g)(3) affiliated ordinary-loss path ──────────────────

    #[test]
    fn affiliated_domestic_80_pct_90_pct_non_passive_ordinary() {
        let mut i = base();
        i.corporation_is_domestic = true;
        i.taxpayer_meets_80_pct_affiliation_test = true;
        i.non_passive_gross_receipts_pct_bp = 9500; // 95%
        let r = compute(&i);
        assert!(r.section_165g3_ordinary_path_applies);
        assert_eq!(
            r.character_of_loss,
            CharacterOfLoss::OrdinaryUnder165g3Affiliated
        );
    }

    #[test]
    fn foreign_subsidiary_does_not_qualify_for_165g3() {
        let mut i = base();
        i.corporation_is_domestic = false;
        i.taxpayer_meets_80_pct_affiliation_test = true;
        i.non_passive_gross_receipts_pct_bp = 9500;
        let r = compute(&i);
        assert!(!r.section_165g3_ordinary_path_applies);
        assert_eq!(r.character_of_loss, CharacterOfLoss::LongTermCapital);
    }

    #[test]
    fn under_80_pct_ownership_does_not_qualify_for_165g3() {
        let mut i = base();
        i.corporation_is_domestic = true;
        i.taxpayer_meets_80_pct_affiliation_test = false;
        i.non_passive_gross_receipts_pct_bp = 9500;
        let r = compute(&i);
        assert!(!r.section_165g3_ordinary_path_applies);
    }

    #[test]
    fn at_90_pct_non_passive_does_not_qualify_strict_greater() {
        // §165(g)(3) requires "more than" 90% — exactly 90% fails.
        let mut i = base();
        i.corporation_is_domestic = true;
        i.taxpayer_meets_80_pct_affiliation_test = true;
        i.non_passive_gross_receipts_pct_bp = 9000;
        let r = compute(&i);
        assert!(
            !r.section_165g3_ordinary_path_applies,
            "exactly 90% fails the strict-greater test"
        );
    }

    #[test]
    fn at_90_01_pct_non_passive_qualifies() {
        let mut i = base();
        i.corporation_is_domestic = true;
        i.taxpayer_meets_80_pct_affiliation_test = true;
        i.non_passive_gross_receipts_pct_bp = 9001;
        let r = compute(&i);
        assert!(r.section_165g3_ordinary_path_applies);
    }

    // ── §1244 priority over §165(g)(1) and §165(g)(3) ─────────────

    #[test]
    fn section_1244_takes_precedence_over_165g1_capital() {
        let mut i = base();
        i.qualifies_under_section_1244 = true;
        let r = compute(&i);
        assert_eq!(
            r.character_of_loss,
            CharacterOfLoss::OrdinaryUnderSection1244
        );
        assert!(r.section_1244_ordinary_path_applies);
    }

    #[test]
    fn section_1244_takes_precedence_over_165g3_affiliated() {
        // Both §1244 AND §165(g)(3) gates met → §1244 wins.
        let mut i = base();
        i.qualifies_under_section_1244 = true;
        i.corporation_is_domestic = true;
        i.taxpayer_meets_80_pct_affiliation_test = true;
        i.non_passive_gross_receipts_pct_bp = 9500;
        let r = compute(&i);
        assert_eq!(
            r.character_of_loss,
            CharacterOfLoss::OrdinaryUnderSection1244
        );
    }

    // ── Loss amount ────────────────────────────────────────────────

    #[test]
    fn loss_equals_adjusted_basis() {
        let r = compute(&base());
        assert_eq!(r.loss_amount_dollars, 100_000);
    }

    #[test]
    fn large_basis_no_precision_loss() {
        let mut i = base();
        i.security_adjusted_basis_dollars = 50_000_000;
        let r = compute(&i);
        assert_eq!(r.loss_amount_dollars, 50_000_000);
    }

    #[test]
    fn negative_basis_clamps_to_zero_loss() {
        let mut i = base();
        i.security_adjusted_basis_dollars = -1_000;
        let r = compute(&i);
        assert_eq!(r.loss_amount_dollars, 0);
    }

    // ── Citation ────────────────────────────────────────────────────

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§165(g)(1)"));
        assert!(r.citation.contains("§165(g)(2)"));
        assert!(r.citation.contains("§165(g)(3)"));
        assert!(r.citation.contains("§1504(a)(2)"));
        assert!(r.citation.contains("§1244"));
        assert!(r.citation.contains("26 CFR § 1.165-5"));
    }

    #[test]
    fn partial_worthlessness_citation_mentions_deferred() {
        let mut i = base();
        i.security_wholly_worthless = false;
        let r = compute(&i);
        assert!(r.citation.contains("wholly worthless"));
        assert!(r.citation.contains("§1001"));
    }

    // ── Notes ───────────────────────────────────────────────────────

    #[test]
    fn note_baseline_describes_lt_capital() {
        let r = compute(&base());
        assert!(r.note.contains("long-term capital loss"));
    }

    #[test]
    fn note_165g3_path_says_applies() {
        let mut i = base();
        i.corporation_is_domestic = true;
        i.taxpayer_meets_80_pct_affiliation_test = true;
        i.non_passive_gross_receipts_pct_bp = 9500;
        let r = compute(&i);
        assert!(r.note.contains("§165(g)(3) APPLIES"));
    }

    #[test]
    fn note_section_1244_path_mentions_section_1244_module() {
        let mut i = base();
        i.qualifies_under_section_1244 = true;
        let r = compute(&i);
        assert!(r.note.contains("section_1244 module"));
    }

    #[test]
    fn note_includes_holding_period_threshold_explanation() {
        let r = compute(&base());
        assert!(r.note.contains("365 required for LT"));
    }
}
