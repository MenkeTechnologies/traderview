//! IRC §481(a) — Accounting-method change cumulative adjustment.
//!
//! A trader who elects §475(f) **mark-to-market** status (see
//! `mtm_475f`) must mark all open positions to market as of the first
//! day of the election year. The cumulative difference between the
//! prior-method realized basis and the new-method MTM value is a
//! §481(a) adjustment.
//!
//! Recognition timing per Rev. Proc. 2015-13:
//!
//!   * **Positive §481(a) adjustment (net unrealized GAIN)** — spread
//!     ratably over **4 tax years** beginning with the year of change.
//!     25% per year. Lets the trader avoid a one-time tax cliff on
//!     the cumulative pre-election appreciation.
//!   * **Negative §481(a) adjustment (net unrealized LOSS)** —
//!     recognized **entirely in the year of change**. No spread on
//!     losses. The trader gets the deduction immediately, matching
//!     §475(f)'s general ordinary-loss character.
//!
//! For the trader, the §481(a) adjustment is **ordinary income or
//! loss** (consistent with §475(f) treatment of MTM gains/losses).
//! Not capital, not LTCG-eligible. The 4-year spread is purely a
//! timing relief — it doesn't change character.
//!
//! Caller submits a list of open positions (each with cost basis and
//! FMV at start of election year); we compute per-position deltas,
//! the total §481(a), the spread schedule, and the recognition for
//! any specific tax year in the 4-year window.
//!
//! Pure compute. No tracking of the spread across years is persisted
//! here — caller is responsible for stamping each year's recognized
//! portion onto whatever ledger they keep.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPosition {
    pub symbol: String,
    pub cost_basis: Decimal,
    pub fmv_at_election_start: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section481Input {
    /// First tax year of the §475(f) election. The §481(a) adjustment
    /// is computed as of January 1 of this year.
    pub election_year: i32,
    /// The tax year the caller is asking about. Returns the spread
    /// portion recognized for THIS year.
    pub current_tax_year: i32,
    pub open_positions: Vec<OpenPosition>,
    /// Override the 4-year spread per Rev. Proc. 2015-13. Used only if
    /// the IRS publishes a different default for a particular regime.
    pub spread_years_override: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section481Result {
    pub election_year: i32,
    pub current_tax_year: i32,
    pub per_position_deltas: Vec<PositionDelta>,
    pub total_section_481_adjustment: Decimal,
    /// True for positive (gain) adjustments that get the 4-year spread.
    /// False for negative (loss) adjustments recognized immediately.
    pub spread_applies: bool,
    pub spread_years: u32,
    /// The amount recognized in each year of the spread (or just
    /// `[loss]` if it's a negative single-year recognition).
    pub annual_recognition_schedule: Vec<AnnualRecognition>,
    pub current_year_recognition: Decimal,
    pub cumulative_recognized_through_current_year: Decimal,
    pub remaining_to_recognize_after_current_year: Decimal,
    pub note: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositionDelta {
    pub symbol: String,
    pub cost_basis: Decimal,
    pub fmv_at_election_start: Decimal,
    pub delta: Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnnualRecognition {
    pub tax_year: i32,
    pub amount: Decimal,
}

fn quarter() -> Decimal {
    Decimal::from_str("0.25").unwrap()
}

pub fn compute(input: &Section481Input) -> Section481Result {
    let mut r = Section481Result {
        election_year: input.election_year,
        current_tax_year: input.current_tax_year,
        ..Section481Result::default()
    };

    for pos in &input.open_positions {
        let delta = pos.fmv_at_election_start - pos.cost_basis;
        r.total_section_481_adjustment += delta;
        r.per_position_deltas.push(PositionDelta {
            symbol: pos.symbol.clone(),
            cost_basis: pos.cost_basis,
            fmv_at_election_start: pos.fmv_at_election_start,
            delta,
        });
    }

    let spread_years = input.spread_years_override.unwrap_or(4);
    r.spread_years = spread_years;
    r.spread_applies = r.total_section_481_adjustment > Decimal::ZERO;

    if r.total_section_481_adjustment == Decimal::ZERO {
        r.note = "no §481(a) adjustment — pre-election cost basis equals MTM value".into();
        return r;
    }

    if !r.spread_applies {
        // Negative — full loss recognized in election year only.
        r.annual_recognition_schedule.push(AnnualRecognition {
            tax_year: input.election_year,
            amount: r.total_section_481_adjustment,
        });
        if input.current_tax_year == input.election_year {
            r.current_year_recognition = r.total_section_481_adjustment;
            r.cumulative_recognized_through_current_year = r.total_section_481_adjustment;
            r.remaining_to_recognize_after_current_year = Decimal::ZERO;
        } else if input.current_tax_year > input.election_year {
            // Past the election year — already fully recognized.
            r.cumulative_recognized_through_current_year = r.total_section_481_adjustment;
            r.remaining_to_recognize_after_current_year = Decimal::ZERO;
        } else {
            // Before election year — nothing yet.
        }
        r.note = format!(
            "§481(a) loss adjustment ${} recognized in full in election year {} (no spread per Rev. Proc. 2015-13)",
            r.total_section_481_adjustment, input.election_year
        );
        return r;
    }

    // Positive adjustment — 4-year ratable spread.
    let years = Decimal::from(spread_years);
    let per_year = if spread_years == 4 {
        (r.total_section_481_adjustment * quarter()).round_dp(2)
    } else {
        (r.total_section_481_adjustment / years).round_dp(2)
    };

    // Build per-year schedule, with the final year absorbing any
    // rounding residual so the cumulative ties out exactly to the
    // total adjustment.
    for n in 0..spread_years {
        let year = input.election_year + n as i32;
        let amount = if n == spread_years - 1 {
            r.total_section_481_adjustment - per_year * Decimal::from(spread_years - 1)
        } else {
            per_year
        };
        r.annual_recognition_schedule.push(AnnualRecognition {
            tax_year: year,
            amount,
        });
    }

    // Current-year + cumulative + remaining.
    let mut cumulative = Decimal::ZERO;
    let mut current_year_amt = Decimal::ZERO;
    for entry in &r.annual_recognition_schedule {
        if entry.tax_year <= input.current_tax_year {
            cumulative += entry.amount;
        }
        if entry.tax_year == input.current_tax_year {
            current_year_amt = entry.amount;
        }
    }
    r.current_year_recognition = current_year_amt;
    r.cumulative_recognized_through_current_year = cumulative;
    r.remaining_to_recognize_after_current_year =
        (r.total_section_481_adjustment - cumulative).max(Decimal::ZERO);

    r.note = format!(
        "§481(a) gain adjustment ${} ratably recognized over {} years from {} ({}/year). Current year {} recognition: ${}; cumulative through current: ${}; remaining: ${}",
        r.total_section_481_adjustment,
        spread_years,
        input.election_year,
        per_year,
        input.current_tax_year,
        r.current_year_recognition,
        r.cumulative_recognized_through_current_year,
        r.remaining_to_recognize_after_current_year,
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn pos(symbol: &str, basis: Decimal, fmv: Decimal) -> OpenPosition {
        OpenPosition {
            symbol: symbol.into(),
            cost_basis: basis,
            fmv_at_election_start: fmv,
        }
    }

    fn base_input(positions: Vec<OpenPosition>, current_year: i32) -> Section481Input {
        Section481Input {
            election_year: 2024,
            current_tax_year: current_year,
            open_positions: positions,
            spread_years_override: None,
        }
    }

    #[test]
    fn positive_adjustment_25pct_year_1() {
        // Single position $100k basis → $200k FMV = $100k gain.
        // Year 1 (2024) recognizes $25k.
        let r = compute(&base_input(
            vec![pos("AAPL", dec!(100000), dec!(200000))],
            2024,
        ));
        assert_eq!(r.total_section_481_adjustment, dec!(100000));
        assert!(r.spread_applies);
        assert_eq!(r.current_year_recognition, dec!(25000));
        assert_eq!(r.cumulative_recognized_through_current_year, dec!(25000));
        assert_eq!(r.remaining_to_recognize_after_current_year, dec!(75000));
    }

    #[test]
    fn positive_adjustment_year_4_full_recognition() {
        let r = compute(&base_input(
            vec![pos("AAPL", dec!(100000), dec!(200000))],
            2027,
        ));
        assert_eq!(r.cumulative_recognized_through_current_year, dec!(100000));
        assert_eq!(r.remaining_to_recognize_after_current_year, Decimal::ZERO);
    }

    #[test]
    fn negative_adjustment_recognized_immediately_no_spread() {
        // $200k basis → $150k FMV = $50k loss. Full recognition in 2024.
        let r = compute(&base_input(
            vec![pos("TSLA", dec!(200000), dec!(150000))],
            2024,
        ));
        assert_eq!(r.total_section_481_adjustment, dec!(-50000));
        assert!(!r.spread_applies);
        assert_eq!(r.current_year_recognition, dec!(-50000));
        assert_eq!(r.annual_recognition_schedule.len(), 1);
    }

    #[test]
    fn negative_adjustment_year_after_election_no_further_recognition() {
        // Already fully recognized in 2024.
        let r = compute(&base_input(
            vec![pos("TSLA", dec!(200000), dec!(150000))],
            2025,
        ));
        assert_eq!(r.current_year_recognition, Decimal::ZERO);
        assert_eq!(r.cumulative_recognized_through_current_year, dec!(-50000));
        assert_eq!(r.remaining_to_recognize_after_current_year, Decimal::ZERO);
    }

    #[test]
    fn aggregates_across_multiple_positions() {
        // 3 positions: +$50k, +$30k, -$20k = net +$60k.
        let r = compute(&base_input(
            vec![
                pos("AAPL", dec!(100000), dec!(150000)),
                pos("MSFT", dec!(50000), dec!(80000)),
                pos("META", dec!(60000), dec!(40000)),
            ],
            2024,
        ));
        assert_eq!(r.total_section_481_adjustment, dec!(60000));
        assert!(r.spread_applies);
        assert_eq!(r.current_year_recognition, dec!(15000));
        assert_eq!(r.per_position_deltas.len(), 3);
    }

    #[test]
    fn zero_adjustment_no_op() {
        let r = compute(&base_input(
            vec![pos("AAPL", dec!(100000), dec!(100000))],
            2024,
        ));
        assert_eq!(r.total_section_481_adjustment, Decimal::ZERO);
        assert!(r.note.contains("no §481(a) adjustment"));
    }

    #[test]
    fn multi_year_cumulative_grows_predictably() {
        let position = vec![pos("AAPL", dec!(100000), dec!(200000))];
        let y1 = compute(&base_input(position.clone(), 2024));
        let y2 = compute(&base_input(position.clone(), 2025));
        let y3 = compute(&base_input(position.clone(), 2026));
        let y4 = compute(&base_input(position, 2027));
        assert_eq!(y1.cumulative_recognized_through_current_year, dec!(25000));
        assert_eq!(y2.cumulative_recognized_through_current_year, dec!(50000));
        assert_eq!(y3.cumulative_recognized_through_current_year, dec!(75000));
        assert_eq!(y4.cumulative_recognized_through_current_year, dec!(100000));
    }

    #[test]
    fn recognition_before_election_year_zero() {
        let r = compute(&base_input(
            vec![pos("AAPL", dec!(100000), dec!(200000))],
            2023,
        ));
        // Caller asking about year before the election — nothing recognized.
        assert_eq!(r.current_year_recognition, Decimal::ZERO);
        assert_eq!(r.cumulative_recognized_through_current_year, Decimal::ZERO);
    }

    #[test]
    fn schedule_has_four_entries_for_positive_adjustment() {
        let r = compute(&base_input(
            vec![pos("AAPL", dec!(100000), dec!(200000))],
            2024,
        ));
        assert_eq!(r.annual_recognition_schedule.len(), 4);
        assert_eq!(r.annual_recognition_schedule[0].tax_year, 2024);
        assert_eq!(r.annual_recognition_schedule[3].tax_year, 2027);
    }

    #[test]
    fn final_year_absorbs_rounding_residual() {
        // $100k / 4 = $25k exactly — no rounding needed.
        // $100,001 / 4 = $25,000.25 each. Final year = $100,001 - 3 ×
        // $25,000.25 = $25,000.25 too. Verify schedule sums to total.
        let r = compute(&base_input(vec![pos("AAPL", dec!(0), dec!(100001))], 2024));
        let sum: Decimal = r.annual_recognition_schedule.iter().map(|x| x.amount).sum();
        assert_eq!(sum, dec!(100001));
    }

    #[test]
    fn final_year_absorbs_rounding_residual_odd_total() {
        // $100,003 / 4 = $25,000.75 each. Final year absorbs the
        // residual so the sum ties out.
        let r = compute(&base_input(vec![pos("AAPL", dec!(0), dec!(100003))], 2024));
        let sum: Decimal = r.annual_recognition_schedule.iter().map(|x| x.amount).sum();
        assert_eq!(sum, dec!(100003));
    }

    #[test]
    fn schedule_year_2_is_25k_for_100k_gain() {
        let r = compute(&base_input(
            vec![pos("AAPL", dec!(100000), dec!(200000))],
            2025,
        ));
        let y2 = &r.annual_recognition_schedule[1];
        assert_eq!(y2.tax_year, 2025);
        assert_eq!(y2.amount, dec!(25000));
        assert_eq!(r.current_year_recognition, dec!(25000));
    }

    #[test]
    fn override_spread_years_to_2_distributes_50_pct_each() {
        let mut i = base_input(vec![pos("AAPL", dec!(100000), dec!(200000))], 2024);
        i.spread_years_override = Some(2);
        let r = compute(&i);
        assert_eq!(r.spread_years, 2);
        assert_eq!(r.current_year_recognition, dec!(50000));
        assert_eq!(r.annual_recognition_schedule.len(), 2);
    }

    #[test]
    fn position_delta_breakdown_matches_per_position() {
        let r = compute(&base_input(
            vec![
                pos("AAPL", dec!(100), dec!(150)),
                pos("MSFT", dec!(200), dec!(180)),
            ],
            2024,
        ));
        assert_eq!(r.per_position_deltas[0].delta, dec!(50));
        assert_eq!(r.per_position_deltas[1].delta, dec!(-20));
    }

    #[test]
    fn empty_positions_zero_adjustment_zero_recognition() {
        let r = compute(&base_input(vec![], 2024));
        assert_eq!(r.total_section_481_adjustment, Decimal::ZERO);
        assert_eq!(r.annual_recognition_schedule.len(), 0);
        assert!(r.note.contains("no §481(a)"));
    }

    #[test]
    fn negative_adjustment_with_current_year_before_election_zero_recognition() {
        // -$50k loss, but caller asking about a year BEFORE election.
        let mut i = base_input(vec![pos("TSLA", dec!(200000), dec!(150000))], 2023);
        i.election_year = 2024;
        let r = compute(&i);
        assert_eq!(r.current_year_recognition, Decimal::ZERO);
        assert_eq!(r.cumulative_recognized_through_current_year, Decimal::ZERO);
    }

    #[test]
    fn mixed_winners_and_losers_net_to_negative_loss_recognized_immediately() {
        // 3 positions: +$10k, +$5k, -$49.9k = net -$34.9k loss.
        let r = compute(&base_input(
            vec![
                pos("AAPL", dec!(100), dec!(10100)),    // +$10,000
                pos("MSFT", dec!(100), dec!(5100)),     // +$5,000
                pos("TSLA", dec!(100000), dec!(50100)), // -$49,900
            ],
            2024,
        ));
        assert_eq!(r.total_section_481_adjustment, dec!(-34900));
        assert!(!r.spread_applies);
        assert_eq!(r.current_year_recognition, dec!(-34900));
    }
}
