//! Tax-loss harvest suggester.
//!
//! End of year: find unrealized losers in open positions that, if sold
//! before Dec 31, would generate ordinary or capital losses to offset
//! gains. Optionally flag the §1091 wash-sale risk if the user has
//! bought (or might re-buy) the same name within 30 days.
//!
//! Pure compute. Inputs: open positions w/ current FMV + recent buys
//! (for wash-sale lookback). Output: ranked harvest candidates with
//! the loss size + caveats.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenLoser {
    pub symbol: String,
    pub qty: Decimal,
    pub avg_cost: Decimal,
    pub current_price: Decimal,
}

impl OpenLoser {
    pub fn unrealized_loss(&self) -> Decimal {
        // Loss is positive when current < cost (the user is in the red).
        (self.avg_cost - self.current_price) * self.qty
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentBuy {
    pub symbol: String,
    pub executed_at: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestCandidate {
    pub symbol: String,
    pub qty: Decimal,
    pub unrealized_loss: Decimal,
    /// True if the user bought this symbol within the last 30 days —
    /// selling now triggers §1091 wash-sale disallowance.
    pub wash_sale_risk: bool,
    /// True if selling would push the year-end realized losses past the
    /// $3k capital-loss cap (without MTM election). Caller passes the
    /// realized-loss-so-far context.
    pub exceeds_3k_cap: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HarvestReport {
    pub candidates: Vec<HarvestCandidate>,
    /// Sum of unrealized loss across all candidates (ignoring caps).
    pub total_available_loss: Decimal,
    /// Sum AFTER filtering out wash-sale-blocked candidates.
    pub safe_harvest_loss: Decimal,
}

/// Build the harvest report. `realized_loss_ytd` is positive when the
/// user has lost money YTD — used to detect when harvesting would push
/// past the $3k ordinary-income offset cap.
pub fn suggest(
    losers: &[OpenLoser],
    recent_buys: &[RecentBuy],
    today: NaiveDate,
    realized_loss_ytd: Decimal,
    mtm_elected: bool,
) -> HarvestReport {
    let mut candidates = Vec::new();
    let mut running_loss = realized_loss_ytd;
    let three_k = Decimal::from(3_000);

    // Sort losers by loss size (largest first) so the biggest harvest
    // shows at the top of the report.
    let mut sorted: Vec<&OpenLoser> = losers
        .iter()
        .filter(|l| l.current_price < l.avg_cost) // genuinely losing
        .collect();
    sorted.sort_by_key(|a| std::cmp::Reverse(a.unrealized_loss()));

    for l in sorted {
        let loss = l.unrealized_loss();
        let wash_risk = recent_buys
            .iter()
            .any(|b| b.symbol == l.symbol && (today - b.executed_at).num_days().abs() <= 30);
        running_loss += loss;
        let exceeds_cap = !mtm_elected && running_loss > three_k;

        let note = if wash_risk {
            "WASH SALE: bought within 30 days — loss would be disallowed".to_string()
        } else if exceeds_cap {
            format!(
                "loss pushes YTD past $3k capital-loss cap (current ${running_loss}); \
                     surplus carries forward to next year"
            )
        } else {
            "safe to harvest".to_string()
        };

        candidates.push(HarvestCandidate {
            symbol: l.symbol.clone(),
            qty: l.qty,
            unrealized_loss: loss,
            wash_sale_risk: wash_risk,
            exceeds_3k_cap: exceeds_cap,
            note,
        });
    }
    let total: Decimal = candidates.iter().map(|c| c.unrealized_loss).sum();
    let safe: Decimal = candidates
        .iter()
        .filter(|c| !c.wash_sale_risk)
        .map(|c| c.unrealized_loss)
        .sum();
    HarvestReport {
        candidates,
        total_available_loss: total,
        safe_harvest_loss: safe,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }
    fn day(y: i32, m: u32, d_: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d_).unwrap()
    }
    fn loser(symbol: &str, qty: &str, cost: &str, price: &str) -> OpenLoser {
        OpenLoser {
            symbol: symbol.into(),
            qty: d(qty),
            avg_cost: d(cost),
            current_price: d(price),
        }
    }

    #[test]
    fn winners_excluded_from_candidates() {
        // Current price ABOVE cost — not a loser.
        let r = suggest(
            &[loser("AAPL", "100", "150", "160")],
            &[],
            day(2026, 12, 15),
            Decimal::ZERO,
            false,
        );
        assert!(r.candidates.is_empty());
    }

    #[test]
    fn unrealized_loss_calc_is_positive_when_current_below_cost() {
        let l = loser("X", "100", "50", "40");
        assert_eq!(l.unrealized_loss(), d("1000"));
    }

    #[test]
    fn wash_sale_risk_flags_recent_buy() {
        // Bought 10 days before today.
        let r = suggest(
            &[loser("AAPL", "100", "150", "140")],
            &[RecentBuy {
                symbol: "AAPL".into(),
                executed_at: day(2026, 12, 5),
            }],
            day(2026, 12, 15),
            Decimal::ZERO,
            false,
        );
        let c = &r.candidates[0];
        assert!(c.wash_sale_risk);
        assert!(c.note.contains("WASH SALE"));
    }

    #[test]
    fn old_buy_outside_30_day_window_does_not_flag() {
        let r = suggest(
            &[loser("AAPL", "100", "150", "140")],
            &[RecentBuy {
                symbol: "AAPL".into(),
                executed_at: day(2026, 1, 1),
            }],
            day(2026, 12, 15),
            Decimal::ZERO,
            false,
        );
        assert!(!r.candidates[0].wash_sale_risk);
    }

    #[test]
    fn three_k_cap_flagged_when_not_mtm_elected() {
        // Already lost $2.5k YTD; harvesting another $1k pushes past $3k.
        let r = suggest(
            &[loser("AAPL", "100", "150", "140")], // $1k loss
            &[],
            day(2026, 12, 15),
            d("2500"),
            false,
        );
        assert!(r.candidates[0].exceeds_3k_cap);
        assert!(r.candidates[0].note.contains("$3k"));
    }

    #[test]
    fn mtm_election_skips_3k_cap_warning() {
        // Same scenario but MTM elected — no $3k cap applies.
        let r = suggest(
            &[loser("AAPL", "100", "150", "140")],
            &[],
            day(2026, 12, 15),
            d("10000"),
            true,
        );
        assert!(!r.candidates[0].exceeds_3k_cap);
        assert!(!r.candidates[0].note.contains("$3k"));
    }

    #[test]
    fn candidates_sorted_by_loss_size_descending() {
        let r = suggest(
            &[
                loser("TINY", "10", "50", "48"),  // $20 loss
                loser("BIG", "1000", "50", "30"), // $20,000 loss
                loser("MID", "100", "50", "40"),  // $1,000 loss
            ],
            &[],
            day(2026, 12, 15),
            Decimal::ZERO,
            false,
        );
        assert_eq!(r.candidates[0].symbol, "BIG");
        assert_eq!(r.candidates[1].symbol, "MID");
        assert_eq!(r.candidates[2].symbol, "TINY");
    }

    #[test]
    fn totals_correctly_split_safe_vs_total() {
        let r = suggest(
            &[
                loser("AAPL", "100", "150", "140"), // $1k loss, wash flagged
                loser("TSLA", "10", "300", "250"),  // $500 loss, safe
            ],
            &[RecentBuy {
                symbol: "AAPL".into(),
                executed_at: day(2026, 12, 1),
            }],
            day(2026, 12, 15),
            Decimal::ZERO,
            false,
        );
        assert_eq!(r.total_available_loss, d("1500"));
        assert_eq!(r.safe_harvest_loss, d("500")); // AAPL excluded
    }

    #[test]
    fn empty_input_returns_empty_report() {
        let r = suggest(&[], &[], day(2026, 12, 15), Decimal::ZERO, false);
        assert!(r.candidates.is_empty());
        assert_eq!(r.total_available_loss, Decimal::ZERO);
    }
}
