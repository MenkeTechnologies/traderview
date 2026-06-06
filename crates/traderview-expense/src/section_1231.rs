//! IRC §1231 — Property used in the trade or business and
//! involuntary conversions.
//!
//! The "best of both worlds" tax provision for real estate traders,
//! business owners, and equipment-heavy operations. When a taxpayer
//! disposes of §1231 property (real property + depreciable personal
//! property used in trade or business held > 1 year), the entire
//! year's net result is recharacterized:
//!
//! - **§1231(a)(1)** — net gain → treated as **long-term capital
//!   gain** (preferential 0 / 15 / 20% LTCG rates).
//! - **§1231(a)(2)** — net loss → treated as **ordinary loss**
//!   (no §1211 capital-loss cap; full offset against ordinary
//!   income; can drive §172 NOL).
//!
//! Asymmetric upside/downside is the defining feature: capital-gain
//! treatment on the win, ordinary-loss treatment on the loss. To
//! prevent gaming the timing, **§1231(c)** imposes a 5-year lookback
//! recapture: any current-year net §1231 gain is treated as ORDINARY
//! income to the extent of unrecovered ("nonrecaptured") net §1231
//! losses from the preceding 5 tax years.
//!
//! **§1231(b) property definition**:
//! - Real property used in the trade or business (includes rental
//!   real estate).
//! - Depreciable personal property used in the trade or business.
//! - Property held > 1 year (long-term holding period).
//! - Plus specific carve-ins for timber, coal, livestock, unharvested
//!   crops, copyrights (§1231(b)(2)–(6)).
//! - **Excluded**: inventory, accounts receivable, copyrights /
//!   musical works in the hands of the creator (unless §1221(b)(3)
//!   election), personal-use property.
//!
//! **§1231(c) 5-year lookback recapture mechanism**:
//!
//!   Nonrecaptured_Loss = Σ(prior 5 years' net §1231 losses)
//!                       − Σ(amounts already recaptured in interim)
//!   Recapture_Ordinary = min(current_net_§1231_gain,
//!                            Nonrecaptured_Loss_balance)
//!   Remaining_LTCG = current_net_§1231_gain − Recapture_Ordinary
//!
//! After interaction with depreciation recapture under §1245 / §1250
//! (which converts gain on personal property / real property up to
//! prior depreciation back to ordinary), the §1231 character
//! determination applies to whatever §1231 character remains.
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 1231](https://www.law.cornell.edu/uscode/text/26/1231),
//! [IRS Pub. 544 — Sales and Other Dispositions of Assets](https://www.irs.gov/publications/p544),
//! [IRS Form 4797 instructions (2025)](https://www.irs.gov/instructions/i4797),
//! [Wolters Kluwer AnswerConnect — Recaptured §1231 Losses](https://answerconnect.cch.com/document/arp2808d0ed447db8100084c4000d3a8abb4e0a/federal/irc/explanation/recaptured-section-1231-losses).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetCharacterization {
    /// Net §1231 gain — full LTCG (no §1231(c) recapture exposure).
    NetGainPureLongTermCapital,
    /// Net §1231 gain partly recharacterized as ordinary under
    /// §1231(c) 5-year lookback recapture.
    NetGainWithSection1231cOrdinaryRecapture,
    /// Net §1231 loss — full ordinary loss treatment.
    NetLossOrdinary,
    /// Net result is zero — no characterization needed.
    NoNetResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1231Input {
    pub tax_year: i32,
    /// Sum of all §1231 gains in the current tax year.
    pub current_year_gains_dollars: i64,
    /// Sum of all §1231 losses in the current tax year.
    pub current_year_losses_dollars: i64,
    /// Nonrecaptured §1231 loss carryover balance — the sum of net
    /// §1231 losses from the preceding 5 tax years that have not
    /// yet been "recaptured" by a prior year's §1231 gain.
    pub nonrecaptured_section_1231_loss_carryover_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1231Result {
    pub characterization: NetCharacterization,
    /// Net §1231 gain or loss for the year (gains − losses; sign
    /// indicates direction).
    pub net_section_1231_dollars: i64,
    /// Portion of net gain treated as long-term capital gain.
    pub long_term_capital_gain_dollars: i64,
    /// Portion of net gain treated as ordinary income under
    /// §1231(c) recapture. Zero for net loss years.
    pub ordinary_recapture_dollars: i64,
    /// Net §1231 loss treated as ordinary loss. Zero for net gain
    /// years.
    pub ordinary_loss_dollars: i64,
    /// Nonrecaptured §1231 loss balance going into NEXT year:
    /// (a) net loss years → previous carryover + current loss;
    /// (b) net gain years → previous carryover − ordinary recapture.
    pub nonrecaptured_loss_carryover_to_next_year_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section1231Input) -> Section1231Result {
    let net = input.current_year_gains_dollars - input.current_year_losses_dollars;
    let carryover_in = input
        .nonrecaptured_section_1231_loss_carryover_dollars
        .max(0);

    let (characterization, ltcg, ordinary_recap, ordinary_loss, carryover_out) = if net == 0 {
        (
            NetCharacterization::NoNetResult,
            0i64,
            0i64,
            0i64,
            carryover_in,
        )
    } else if net < 0 {
        // §1231(a)(2): net loss → ordinary. Loss is added to the
        // nonrecaptured loss carryover for the next 5 years.
        let loss = (-net).max(0);
        (
            NetCharacterization::NetLossOrdinary,
            0,
            0,
            loss,
            carryover_in + loss,
        )
    } else {
        // §1231(a)(1) + §1231(c) recapture interaction.
        let recap = net.min(carryover_in);
        let ltcg_portion = net - recap;
        let characterization = if recap > 0 {
            NetCharacterization::NetGainWithSection1231cOrdinaryRecapture
        } else {
            NetCharacterization::NetGainPureLongTermCapital
        };
        (
            characterization,
            ltcg_portion,
            recap,
            0,
            (carryover_in - recap).max(0),
        )
    };

    let char_label = match characterization {
        NetCharacterization::NetGainPureLongTermCapital => {
            "§1231(a)(1) net gain → pure LTCG (no §1231(c) recapture)"
        }
        NetCharacterization::NetGainWithSection1231cOrdinaryRecapture => {
            "§1231(a)(1) net gain WITH §1231(c) 5-year lookback recapture"
        }
        NetCharacterization::NetLossOrdinary => "§1231(a)(2) net loss → ordinary",
        NetCharacterization::NoNetResult => "no net result",
    };

    let note = format!(
        "Tax year {}; gains ${} − losses ${} = net ${}; nonrecaptured §1231 loss carry-in ${}; characterization: {}; LTCG portion ${}; ordinary recapture portion ${}; ordinary loss portion ${}; carry-out to next year ${}.",
        input.tax_year,
        input.current_year_gains_dollars,
        input.current_year_losses_dollars,
        net,
        carryover_in,
        char_label,
        ltcg,
        ordinary_recap,
        ordinary_loss,
        carryover_out,
    );

    Section1231Result {
        characterization,
        net_section_1231_dollars: net,
        long_term_capital_gain_dollars: ltcg,
        ordinary_recapture_dollars: ordinary_recap,
        ordinary_loss_dollars: ordinary_loss,
        nonrecaptured_loss_carryover_to_next_year_dollars: carryover_out,
        citation:
            "IRC §1231 quasi-capital gain/loss for property used in trade or business + involuntary conversions; §1231(a)(1) net gain → LTCG; §1231(a)(2) net loss → ordinary; §1231(b) property definition (real or depreciable property used in trade/business held > 1 year); §1231(c) 5-year lookback recapture treats current-year net gain as ordinary to extent of nonrecaptured prior-5-year net §1231 losses; reported on Form 4797 Part I; interacts downstream with §1245 / §1250 depreciation recapture"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section1231Input {
        Section1231Input {
            tax_year: 2026,
            current_year_gains_dollars: 200_000,
            current_year_losses_dollars: 50_000,
            nonrecaptured_section_1231_loss_carryover_dollars: 0,
        }
    }

    // ── §1231(a)(1) net gain → LTCG ─────────────────────────────────

    #[test]
    fn net_gain_no_carryover_full_ltcg() {
        // $200k gains − $50k losses = $150k net; no carryover →
        // full LTCG.
        let r = compute(&base());
        assert_eq!(
            r.characterization,
            NetCharacterization::NetGainPureLongTermCapital
        );
        assert_eq!(r.net_section_1231_dollars, 150_000);
        assert_eq!(r.long_term_capital_gain_dollars, 150_000);
        assert_eq!(r.ordinary_recapture_dollars, 0);
        assert_eq!(r.ordinary_loss_dollars, 0);
    }

    #[test]
    fn carryover_to_next_year_is_zero_when_no_carry_in_and_net_gain() {
        let r = compute(&base());
        assert_eq!(r.nonrecaptured_loss_carryover_to_next_year_dollars, 0);
    }

    // ── §1231(c) recapture mechanics ────────────────────────────────

    #[test]
    fn current_gain_partial_recapture_against_carryover() {
        // $200k − $50k = $150k net; $100k carryover → $100k ordinary
        // recapture + $50k LTCG residual.
        let mut i = base();
        i.nonrecaptured_section_1231_loss_carryover_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(
            r.characterization,
            NetCharacterization::NetGainWithSection1231cOrdinaryRecapture
        );
        assert_eq!(r.ordinary_recapture_dollars, 100_000);
        assert_eq!(r.long_term_capital_gain_dollars, 50_000);
        assert_eq!(r.nonrecaptured_loss_carryover_to_next_year_dollars, 0);
    }

    #[test]
    fn carryover_exceeds_gain_full_recapture_no_ltcg() {
        // Net gain $150k; carryover $300k → full recapture $150k,
        // remaining carryover $150k.
        let mut i = base();
        i.nonrecaptured_section_1231_loss_carryover_dollars = 300_000;
        let r = compute(&i);
        assert_eq!(r.ordinary_recapture_dollars, 150_000);
        assert_eq!(r.long_term_capital_gain_dollars, 0);
        assert_eq!(r.nonrecaptured_loss_carryover_to_next_year_dollars, 150_000);
    }

    #[test]
    fn carryover_exact_match_to_net_gain_zero_ltcg() {
        let mut i = base();
        i.nonrecaptured_section_1231_loss_carryover_dollars = 150_000;
        let r = compute(&i);
        assert_eq!(r.ordinary_recapture_dollars, 150_000);
        assert_eq!(r.long_term_capital_gain_dollars, 0);
        assert_eq!(r.nonrecaptured_loss_carryover_to_next_year_dollars, 0);
    }

    // ── §1231(a)(2) net loss → ordinary ────────────────────────────

    #[test]
    fn net_loss_ordinary_treatment() {
        // $50k gains − $200k losses = −$150k net loss.
        let mut i = base();
        i.current_year_gains_dollars = 50_000;
        i.current_year_losses_dollars = 200_000;
        let r = compute(&i);
        assert_eq!(r.characterization, NetCharacterization::NetLossOrdinary);
        assert_eq!(r.net_section_1231_dollars, -150_000);
        assert_eq!(r.ordinary_loss_dollars, 150_000);
        assert_eq!(r.long_term_capital_gain_dollars, 0);
        assert_eq!(r.ordinary_recapture_dollars, 0);
    }

    #[test]
    fn net_loss_increases_carryover_for_5_year_window() {
        // Pre-existing carryover $50k + new $150k loss = $200k.
        let mut i = base();
        i.current_year_gains_dollars = 50_000;
        i.current_year_losses_dollars = 200_000;
        i.nonrecaptured_section_1231_loss_carryover_dollars = 50_000;
        let r = compute(&i);
        assert_eq!(r.nonrecaptured_loss_carryover_to_next_year_dollars, 200_000);
    }

    // ── Zero net result ────────────────────────────────────────────

    #[test]
    fn zero_net_no_characterization() {
        let mut i = base();
        i.current_year_gains_dollars = 100_000;
        i.current_year_losses_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(r.characterization, NetCharacterization::NoNetResult);
        assert_eq!(r.net_section_1231_dollars, 0);
        assert_eq!(r.long_term_capital_gain_dollars, 0);
        assert_eq!(r.ordinary_recapture_dollars, 0);
        assert_eq!(r.ordinary_loss_dollars, 0);
    }

    #[test]
    fn zero_net_preserves_carryover() {
        let mut i = base();
        i.current_year_gains_dollars = 100_000;
        i.current_year_losses_dollars = 100_000;
        i.nonrecaptured_section_1231_loss_carryover_dollars = 75_000;
        let r = compute(&i);
        assert_eq!(r.nonrecaptured_loss_carryover_to_next_year_dollars, 75_000);
    }

    // ── Multi-year chain consistency ───────────────────────────────

    #[test]
    fn multi_year_chain_recapture_then_pure_ltcg() {
        // Year 1: loss $100k → carryover $100k.
        // Year 2: gain $40k → recapture $40k, residual carryover $60k.
        // Year 3: gain $80k → recapture $60k + $20k LTCG, carryover 0.
        let year1 = compute(&Section1231Input {
            tax_year: 2024,
            current_year_gains_dollars: 0,
            current_year_losses_dollars: 100_000,
            nonrecaptured_section_1231_loss_carryover_dollars: 0,
        });
        assert_eq!(year1.ordinary_loss_dollars, 100_000);
        assert_eq!(
            year1.nonrecaptured_loss_carryover_to_next_year_dollars,
            100_000
        );

        let year2 = compute(&Section1231Input {
            tax_year: 2025,
            current_year_gains_dollars: 40_000,
            current_year_losses_dollars: 0,
            nonrecaptured_section_1231_loss_carryover_dollars: 100_000,
        });
        assert_eq!(year2.ordinary_recapture_dollars, 40_000);
        assert_eq!(year2.long_term_capital_gain_dollars, 0);
        assert_eq!(
            year2.nonrecaptured_loss_carryover_to_next_year_dollars,
            60_000
        );

        let year3 = compute(&Section1231Input {
            tax_year: 2026,
            current_year_gains_dollars: 80_000,
            current_year_losses_dollars: 0,
            nonrecaptured_section_1231_loss_carryover_dollars: 60_000,
        });
        assert_eq!(year3.ordinary_recapture_dollars, 60_000);
        assert_eq!(year3.long_term_capital_gain_dollars, 20_000);
        assert_eq!(year3.nonrecaptured_loss_carryover_to_next_year_dollars, 0);
    }

    // ── Defensive / precision ──────────────────────────────────────

    #[test]
    fn negative_carryover_input_clamped_to_zero() {
        let mut i = base();
        i.nonrecaptured_section_1231_loss_carryover_dollars = -50_000;
        let r = compute(&i);
        assert_eq!(r.ordinary_recapture_dollars, 0);
        assert_eq!(r.long_term_capital_gain_dollars, 150_000);
    }

    #[test]
    fn very_large_billion_dollar_real_estate_gain() {
        // $1B gain, no carryover → $1B LTCG.
        let mut i = base();
        i.current_year_gains_dollars = 1_000_000_000;
        i.current_year_losses_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.long_term_capital_gain_dollars, 1_000_000_000);
    }

    #[test]
    fn very_large_gain_partial_recapture_precision() {
        // $500M gain, $200M carryover → $200M ordinary + $300M LTCG.
        let mut i = base();
        i.current_year_gains_dollars = 500_000_000;
        i.current_year_losses_dollars = 0;
        i.nonrecaptured_section_1231_loss_carryover_dollars = 200_000_000;
        let r = compute(&i);
        assert_eq!(r.ordinary_recapture_dollars, 200_000_000);
        assert_eq!(r.long_term_capital_gain_dollars, 300_000_000);
        assert_eq!(r.nonrecaptured_loss_carryover_to_next_year_dollars, 0);
    }

    // ── Citation ───────────────────────────────────────────────────

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§1231(a)(1)"));
        assert!(r.citation.contains("§1231(a)(2)"));
        assert!(r.citation.contains("§1231(b)"));
        assert!(r.citation.contains("§1231(c)"));
        assert!(r.citation.contains("§1245"));
        assert!(r.citation.contains("§1250"));
        assert!(r.citation.contains("Form 4797"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_pure_ltcg_describes_path() {
        let r = compute(&base());
        assert!(r.note.contains("pure LTCG"));
    }

    #[test]
    fn note_recapture_path_describes_5_year_lookback() {
        let mut i = base();
        i.nonrecaptured_section_1231_loss_carryover_dollars = 100_000;
        let r = compute(&i);
        assert!(r.note.contains("5-year lookback recapture"));
    }

    #[test]
    fn note_ordinary_loss_describes_path() {
        let mut i = base();
        i.current_year_gains_dollars = 10_000;
        i.current_year_losses_dollars = 100_000;
        let r = compute(&i);
        assert!(r.note.contains("net loss → ordinary"));
    }

    #[test]
    fn note_zero_net_describes_no_result() {
        let mut i = base();
        i.current_year_gains_dollars = 50_000;
        i.current_year_losses_dollars = 50_000;
        let r = compute(&i);
        assert!(r.note.contains("no net result"));
    }
}
