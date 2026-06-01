//! IRC §1245 / §1250 — depreciation recapture on disposition of
//! depreciable property.
//!
//! Trader-relevant for any real estate trader, rental landlord, or
//! business owner who sells previously depreciated property. The
//! "recapture" rules recharacterize gain attributable to prior
//! depreciation as either ordinary income (§1245 personal
//! property; §1250 pre-1986 real-property excess-depreciation) or
//! as "unrecaptured §1250 gain" subject to a 25% maximum capital
//! rate (post-1986 MACRS real property). The residual gain after
//! recapture flows through to `section_1231` for quasi-capital
//! characterization.
//!
//! **§1245 mechanics** — applies to depreciable PERSONAL property
//! (machinery, equipment, vehicles, fixtures, certain intangibles
//! amortized under §197):
//!
//!   Ordinary_Recapture = min(realized_gain, accumulated_depreciation)
//!   Residual_Section_1231_Gain = realized_gain − Ordinary_Recapture
//!
//! All accumulated depreciation is "recaptured first" — there is
//! no straight-line carve-out for §1245 property.
//!
//! **§1250 mechanics** — applies to depreciable REAL property
//! (buildings, structures, structural components):
//!
//! Pre-1986 / pre-MACRS path: ordinary recapture = lesser of
//! (gain) or (additional depreciation taken after 1975), where
//! "additional depreciation" = depreciation in excess of
//! straight-line.
//!
//! Post-1986 MACRS path: real property is depreciated using
//! straight-line, so there is essentially NO additional
//! depreciation. §1250 ordinary recapture is therefore typically
//! ZERO for modern real estate. Instead, gain attributable to
//! prior depreciation is "unrecaptured §1250 gain" under §1(h)(7)
//! and taxed at a **25% maximum rate** rather than the standard
//! 20% LTCG rate.
//!
//! **§1(h)(7) unrecaptured §1250 gain** — for individual
//! taxpayers, the portion of long-term capital gain attributable
//! to prior depreciation that is NOT recaptured as ordinary income
//! is capped at a maximum 25% rate (vs. the 20% top LTCG rate).
//!
//! **Interaction with §1231**: after §1245 / §1250 recapture is
//! computed, any residual gain flows to §1231 for net-gain-vs-loss
//! characterization. Residual gain → potential LTCG; residual loss
//! → ordinary loss under §1231(a)(2).
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 1245](https://www.law.cornell.edu/uscode/text/26/1245),
//! [Cornell LII 26 U.S.C. § 1250](https://www.law.cornell.edu/uscode/text/26/1250),
//! [IRS Pub. 544 — Sales and Other Dispositions of Assets](https://www.irs.gov/publications/p544),
//! [CPA Exams Mastery — §1245 vs §1250 Depreciation Recapture](https://cpaexamsmastery.com/tcp/gain-and-loss-character/depreciation-recapture/),
//! [Bloomberg Tax — Depreciation Recapture §§ 1245 and 1250 Portfolio 563](https://pro.bloombergtax.com/portfolios/depreciation-recapture-sections-1245-and-1250-portfolio-563/).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    /// §1245 depreciable personal property (equipment, vehicles,
    /// machinery, fixtures, §197 intangibles).
    Section1245Personal,
    /// §1250 depreciable real property placed in service POST-1986
    /// using MACRS straight-line. Ordinary recapture is typically
    /// zero; gain attributable to depreciation is taxed at 25%
    /// maximum under §1(h)(7).
    Section1250RealPostMacrsStraightLine,
    /// §1250 depreciable real property placed in service PRE-1986
    /// or using accelerated depreciation. Additional-depreciation
    /// recapture as ordinary income applies.
    Section1250RealPreMacrsOrAccelerated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1245_1250Input {
    pub property_type: PropertyType,
    /// Realized gain on the disposition.
    pub realized_gain_dollars: i64,
    /// Total accumulated depreciation taken on the property.
    pub accumulated_depreciation_dollars: i64,
    /// "Additional depreciation" — depreciation in excess of
    /// straight-line. Relevant only for the pre-MACRS / accelerated
    /// §1250 path.
    pub additional_depreciation_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1245_1250Result {
    /// §1245(a)(1) ordinary recapture amount (personal property).
    pub section_1245_ordinary_recapture_dollars: i64,
    /// §1250 ordinary recapture amount (pre-1986 / accelerated
    /// path).
    pub section_1250_ordinary_recapture_dollars: i64,
    /// §1(h)(7) unrecaptured §1250 gain — gain attributable to
    /// prior depreciation NOT recaptured as ordinary, taxed at
    /// 25% maximum.
    pub unrecaptured_section_1250_gain_dollars: i64,
    /// Residual gain flowing to §1231 (potential LTCG).
    pub residual_section_1231_gain_dollars: i64,
    /// Total ordinary income from recapture (§1245 + §1250
    /// ordinary).
    pub total_ordinary_recapture_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section1245_1250Input) -> Section1245_1250Result {
    let gain = input.realized_gain_dollars.max(0);
    let accum_dep = input.accumulated_depreciation_dollars.max(0);
    let additional_dep = input.additional_depreciation_dollars.max(0);

    let (s1245_ordinary, s1250_ordinary, unrecaptured_1250, residual_1231) =
        match input.property_type {
            PropertyType::Section1245Personal => {
                // §1245(a)(1): ordinary = min(gain, accum dep).
                let ordinary = gain.min(accum_dep);
                let residual = gain - ordinary;
                (ordinary, 0, 0, residual)
            }
            PropertyType::Section1250RealPostMacrsStraightLine => {
                // Post-1986 MACRS straight-line: §1250 ordinary
                // recapture is zero; gain attributable to
                // depreciation → unrecaptured §1250 gain at 25%
                // max rate.
                let unrecaptured = gain.min(accum_dep);
                let residual = gain - unrecaptured;
                (0, 0, unrecaptured, residual)
            }
            PropertyType::Section1250RealPreMacrsOrAccelerated => {
                // Pre-1986 / accelerated: §1250 ordinary recapture
                // = min(gain, additional depreciation). Remaining
                // gain attributable to straight-line depreciation
                // → unrecaptured §1250 gain. Balance → §1231.
                let s1250 = gain.min(additional_dep);
                let remaining_after_s1250 = gain - s1250;
                // Unrecaptured §1250 = gain attributable to
                // straight-line depreciation = min(remaining, accum
                // dep − additional dep).
                let straight_line_dep = (accum_dep - additional_dep).max(0);
                let unrecaptured = remaining_after_s1250.min(straight_line_dep);
                let residual = remaining_after_s1250 - unrecaptured;
                (0, s1250, unrecaptured, residual)
            }
        };

    let total_ordinary = s1245_ordinary + s1250_ordinary;

    let property_label = match input.property_type {
        PropertyType::Section1245Personal => "§1245 personal property",
        PropertyType::Section1250RealPostMacrsStraightLine => {
            "§1250 real property post-1986 MACRS straight-line"
        }
        PropertyType::Section1250RealPreMacrsOrAccelerated => {
            "§1250 real property pre-1986 / accelerated"
        }
    };

    let note = format!(
        "Property type: {}; realized gain ${}; accumulated depreciation ${}; additional depreciation ${}; §1245 ordinary recapture ${}; §1250 ordinary recapture ${}; unrecaptured §1250 gain ${} (taxed at 25% max under §1(h)(7)); residual §1231 gain ${} (potential LTCG); total ordinary income from recapture ${}.",
        property_label,
        gain,
        accum_dep,
        additional_dep,
        s1245_ordinary,
        s1250_ordinary,
        unrecaptured_1250,
        residual_1231,
        total_ordinary,
    );

    Section1245_1250Result {
        section_1245_ordinary_recapture_dollars: s1245_ordinary,
        section_1250_ordinary_recapture_dollars: s1250_ordinary,
        unrecaptured_section_1250_gain_dollars: unrecaptured_1250,
        residual_section_1231_gain_dollars: residual_1231,
        total_ordinary_recapture_dollars: total_ordinary,
        citation:
            "IRC §1245(a)(1) personal-property depreciation recapture as ordinary income = min(realized gain, accumulated depreciation); §1250 real-property recapture: pre-1986 / accelerated path recaptures additional depreciation (excess over straight-line) as ordinary; post-1986 MACRS straight-line yields zero §1250 ordinary recapture (§1250(b) excess depreciation = 0); §1(h)(7) unrecaptured §1250 gain taxed at 25% maximum rate for individuals (vs. 20% top LTCG rate); residual gain flows to §1231 for net-gain-vs-loss characterization; reported on Form 4797"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_1245() -> Section1245_1250Input {
        Section1245_1250Input {
            property_type: PropertyType::Section1245Personal,
            realized_gain_dollars: 100_000,
            accumulated_depreciation_dollars: 60_000,
            additional_depreciation_dollars: 0,
        }
    }

    fn base_1250_post_macrs() -> Section1245_1250Input {
        Section1245_1250Input {
            property_type: PropertyType::Section1250RealPostMacrsStraightLine,
            realized_gain_dollars: 200_000,
            accumulated_depreciation_dollars: 80_000,
            additional_depreciation_dollars: 0,
        }
    }

    fn base_1250_pre_macrs() -> Section1245_1250Input {
        Section1245_1250Input {
            property_type: PropertyType::Section1250RealPreMacrsOrAccelerated,
            realized_gain_dollars: 200_000,
            accumulated_depreciation_dollars: 100_000,
            additional_depreciation_dollars: 30_000,
        }
    }

    // ── §1245 personal property ────────────────────────────────────

    #[test]
    fn s1245_gain_exceeds_accum_dep_full_recapture_plus_residual() {
        // Gain $100k > accum dep $60k → $60k ordinary + $40k §1231.
        let r = compute(&base_1245());
        assert_eq!(r.section_1245_ordinary_recapture_dollars, 60_000);
        assert_eq!(r.residual_section_1231_gain_dollars, 40_000);
        assert_eq!(r.total_ordinary_recapture_dollars, 60_000);
    }

    #[test]
    fn s1245_gain_below_accum_dep_full_gain_recaptured() {
        // Gain $40k < accum dep $60k → all $40k ordinary, zero §1231.
        let mut i = base_1245();
        i.realized_gain_dollars = 40_000;
        let r = compute(&i);
        assert_eq!(r.section_1245_ordinary_recapture_dollars, 40_000);
        assert_eq!(r.residual_section_1231_gain_dollars, 0);
    }

    #[test]
    fn s1245_exact_accum_dep_match_no_residual() {
        let mut i = base_1245();
        i.realized_gain_dollars = 60_000;
        let r = compute(&i);
        assert_eq!(r.section_1245_ordinary_recapture_dollars, 60_000);
        assert_eq!(r.residual_section_1231_gain_dollars, 0);
    }

    #[test]
    fn s1245_no_unrecaptured_1250_gain() {
        let r = compute(&base_1245());
        assert_eq!(r.unrecaptured_section_1250_gain_dollars, 0);
    }

    // ── §1250 post-1986 MACRS straight-line ─────────────────────────

    #[test]
    fn s1250_post_macrs_no_ordinary_recapture() {
        // Post-1986 MACRS straight-line: zero ordinary recapture.
        let r = compute(&base_1250_post_macrs());
        assert_eq!(r.section_1250_ordinary_recapture_dollars, 0);
        assert_eq!(r.total_ordinary_recapture_dollars, 0);
    }

    #[test]
    fn s1250_post_macrs_gain_below_accum_dep_all_unrecaptured() {
        // Gain $80k = accum dep $80k → all unrecaptured §1250.
        let mut i = base_1250_post_macrs();
        i.realized_gain_dollars = 80_000;
        let r = compute(&i);
        assert_eq!(r.unrecaptured_section_1250_gain_dollars, 80_000);
        assert_eq!(r.residual_section_1231_gain_dollars, 0);
    }

    #[test]
    fn s1250_post_macrs_gain_exceeds_accum_dep_residual_1231() {
        // Gain $200k − $80k unrecaptured = $120k §1231 residual.
        let r = compute(&base_1250_post_macrs());
        assert_eq!(r.unrecaptured_section_1250_gain_dollars, 80_000);
        assert_eq!(r.residual_section_1231_gain_dollars, 120_000);
    }

    // ── §1250 pre-MACRS / accelerated ──────────────────────────────

    #[test]
    fn s1250_pre_macrs_ordinary_recapture_additional_dep() {
        // Additional dep $30k → $30k §1250 ordinary recapture.
        let r = compute(&base_1250_pre_macrs());
        assert_eq!(r.section_1250_ordinary_recapture_dollars, 30_000);
    }

    #[test]
    fn s1250_pre_macrs_unrecaptured_equals_straight_line_dep() {
        // Straight-line dep = $100k accum − $30k additional = $70k.
        // After $30k §1250 ordinary, $170k gain remains. $70k of
        // that is unrecaptured §1250.
        let r = compute(&base_1250_pre_macrs());
        assert_eq!(r.unrecaptured_section_1250_gain_dollars, 70_000);
    }

    #[test]
    fn s1250_pre_macrs_residual_1231_balance() {
        // $200k − $30k §1250 ordinary − $70k unrecaptured = $100k.
        let r = compute(&base_1250_pre_macrs());
        assert_eq!(r.residual_section_1231_gain_dollars, 100_000);
    }

    #[test]
    fn s1250_pre_macrs_gain_below_additional_dep_caps_at_gain() {
        // Gain $20k < additional dep $30k → §1250 ordinary capped at gain.
        let mut i = base_1250_pre_macrs();
        i.realized_gain_dollars = 20_000;
        let r = compute(&i);
        assert_eq!(r.section_1250_ordinary_recapture_dollars, 20_000);
        assert_eq!(r.residual_section_1231_gain_dollars, 0);
    }

    // ── No-cross paths ──────────────────────────────────────────────

    #[test]
    fn s1245_path_produces_no_s1250_amounts() {
        let r = compute(&base_1245());
        assert_eq!(r.section_1250_ordinary_recapture_dollars, 0);
        assert_eq!(r.unrecaptured_section_1250_gain_dollars, 0);
    }

    #[test]
    fn s1250_path_produces_no_s1245_amount() {
        let r = compute(&base_1250_post_macrs());
        assert_eq!(r.section_1245_ordinary_recapture_dollars, 0);
    }

    // ── Defensive ──────────────────────────────────────────────────

    #[test]
    fn zero_gain_zero_recapture() {
        let mut i = base_1245();
        i.realized_gain_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.section_1245_ordinary_recapture_dollars, 0);
        assert_eq!(r.residual_section_1231_gain_dollars, 0);
    }

    #[test]
    fn negative_gain_clamped_to_zero() {
        let mut i = base_1245();
        i.realized_gain_dollars = -50_000;
        let r = compute(&i);
        assert_eq!(r.section_1245_ordinary_recapture_dollars, 0);
        assert_eq!(r.residual_section_1231_gain_dollars, 0);
    }

    #[test]
    fn zero_accumulated_depreciation_no_recapture() {
        // No prior depreciation → no recapture; full gain to §1231.
        let mut i = base_1245();
        i.accumulated_depreciation_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.section_1245_ordinary_recapture_dollars, 0);
        assert_eq!(r.residual_section_1231_gain_dollars, 100_000);
    }

    // ── Citation ───────────────────────────────────────────────────

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base_1245());
        assert!(r.citation.contains("§1245(a)(1)"));
        assert!(r.citation.contains("§1250"));
        assert!(r.citation.contains("§1(h)(7)"));
        assert!(r.citation.contains("25%"));
        assert!(r.citation.contains("Form 4797"));
        assert!(r.citation.contains("§1231"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_describes_property_type_path() {
        let r = compute(&base_1245());
        assert!(r.note.contains("§1245 personal property"));
    }

    #[test]
    fn note_mentions_25_pct_max_rate() {
        let r = compute(&base_1250_post_macrs());
        assert!(r.note.contains("25% max under §1(h)(7)"));
    }

    // ── Precision ──────────────────────────────────────────────────

    #[test]
    fn very_large_billion_dollar_recapture_precision() {
        let mut i = base_1245();
        i.realized_gain_dollars = 1_000_000_000;
        i.accumulated_depreciation_dollars = 400_000_000;
        let r = compute(&i);
        assert_eq!(r.section_1245_ordinary_recapture_dollars, 400_000_000);
        assert_eq!(r.residual_section_1231_gain_dollars, 600_000_000);
    }
}
