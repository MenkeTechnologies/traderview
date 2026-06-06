//! Insider Buying Scanner — flags symbols where corporate insiders
//! have cluster-bought shares over a rolling window.
//!
//! Detection criteria (configurable):
//!   - distinct_insiders ≥ min_distinct_insiders
//!   - total_dollar_volume ≥ min_dollar_volume
//!   - net_share_change (buys − sells) > 0 OR (net_dollar_volume > 0)
//!   - average_transaction_dollar_size ≥ min_average_transaction
//!
//! Output sorted by net_dollar_volume descending so the strongest
//! cluster-buy signals surface first.
//!
//! Inputs are Form 4-style records (post-trade SEC filings). For each
//! transaction:
//!   - direction = "buy" / "sell" / "option_exercise" / "gift" / "other"
//!   - Only "buy" and "sell" are counted in the scan; option_exercise,
//!     gift, and other are filtered out (they don't represent
//!     conviction trades).
//!
//! Pure compute. Companion to `short_interest_scanner`, `momentum_12_1`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsiderTransaction {
    pub symbol: String,
    pub insider_name: String,
    pub insider_title: String,
    /// "buy", "sell", "option_exercise", "gift", "other"
    pub direction: String,
    pub shares: f64,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsiderClusterHit {
    pub symbol: String,
    pub distinct_insiders: usize,
    pub total_buy_dollar_volume: f64,
    pub total_sell_dollar_volume: f64,
    pub net_dollar_volume: f64,
    pub net_shares: f64,
    pub average_transaction_dollar_size: f64,
    pub insider_titles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub min_distinct_insiders: usize,
    pub min_dollar_volume: f64,
    pub min_average_transaction: f64,
    pub require_net_buy: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_distinct_insiders: 2,
            min_dollar_volume: 100_000.0,
            min_average_transaction: 25_000.0,
            require_net_buy: true,
        }
    }
}

pub fn scan(transactions: &[InsiderTransaction], cfg: &Config) -> Vec<InsiderClusterHit> {
    use std::collections::HashMap;
    if transactions.is_empty() {
        return Vec::new();
    }
    // Group by symbol, filtering to buy/sell only.
    let mut by_symbol: HashMap<String, Vec<&InsiderTransaction>> = HashMap::new();
    for tx in transactions {
        if !tx.shares.is_finite() || !tx.price.is_finite() || tx.shares <= 0.0 || tx.price <= 0.0 {
            continue;
        }
        if tx.direction != "buy" && tx.direction != "sell" {
            continue;
        }
        by_symbol.entry(tx.symbol.clone()).or_default().push(tx);
    }
    let mut hits: Vec<InsiderClusterHit> = by_symbol
        .into_iter()
        .filter_map(|(sym, txs)| {
            use std::collections::HashSet;
            let mut buy_dollar = 0.0_f64;
            let mut sell_dollar = 0.0_f64;
            let mut buy_shares = 0.0_f64;
            let mut sell_shares = 0.0_f64;
            let mut insiders: HashSet<String> = HashSet::new();
            let mut titles: HashSet<String> = HashSet::new();
            for tx in &txs {
                let dollars = tx.shares * tx.price;
                if tx.direction == "buy" {
                    buy_dollar += dollars;
                    buy_shares += tx.shares;
                } else {
                    sell_dollar += dollars;
                    sell_shares += tx.shares;
                }
                insiders.insert(tx.insider_name.clone());
                titles.insert(tx.insider_title.clone());
            }
            let n_distinct = insiders.len();
            let total_dollar = buy_dollar + sell_dollar;
            let avg_tx = total_dollar / txs.len() as f64;
            let net_dollar = buy_dollar - sell_dollar;
            let net_shares = buy_shares - sell_shares;
            if n_distinct < cfg.min_distinct_insiders
                || total_dollar < cfg.min_dollar_volume
                || avg_tx < cfg.min_average_transaction
                || (cfg.require_net_buy && net_dollar <= 0.0)
            {
                return None;
            }
            let mut title_list: Vec<String> = titles.into_iter().collect();
            title_list.sort();
            Some(InsiderClusterHit {
                symbol: sym,
                distinct_insiders: n_distinct,
                total_buy_dollar_volume: buy_dollar,
                total_sell_dollar_volume: sell_dollar,
                net_dollar_volume: net_dollar,
                net_shares,
                average_transaction_dollar_size: avg_tx,
                insider_titles: title_list,
            })
        })
        .collect();
    hits.sort_by(|a, b| {
        b.net_dollar_volume
            .partial_cmp(&a.net_dollar_volume)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(
        sym: &str,
        name: &str,
        title: &str,
        dir: &str,
        shares: f64,
        price: f64,
    ) -> InsiderTransaction {
        InsiderTransaction {
            symbol: sym.into(),
            insider_name: name.into(),
            insider_title: title.into(),
            direction: dir.into(),
            shares,
            price,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(scan(&[], &Config::default()).is_empty());
    }

    #[test]
    fn lone_insider_filtered_when_min_distinct_above_one() {
        let txs = vec![
            tx("AAA", "Smith", "CEO", "buy", 1000.0, 50.0),
            tx("AAA", "Smith", "CEO", "buy", 1000.0, 50.0),
        ];
        assert!(scan(&txs, &Config::default()).is_empty());
    }

    #[test]
    fn cluster_buy_emitted() {
        let txs = vec![
            tx("AAA", "Smith", "CEO", "buy", 2000.0, 50.0), // $100K
            tx("AAA", "Jones", "CFO", "buy", 2000.0, 50.0), // $100K
        ];
        let hits = scan(&txs, &Config::default());
        assert_eq!(hits.len(), 1);
        let h = &hits[0];
        assert_eq!(h.symbol, "AAA");
        assert_eq!(h.distinct_insiders, 2);
        assert!((h.total_buy_dollar_volume - 200_000.0).abs() < 1e-9);
        assert!((h.net_dollar_volume - 200_000.0).abs() < 1e-9);
    }

    #[test]
    fn net_seller_filtered_when_require_net_buy() {
        let txs = vec![
            tx("AAA", "Smith", "CEO", "buy", 1000.0, 50.0),
            tx("AAA", "Jones", "CFO", "sell", 3000.0, 50.0),
        ];
        let hits = scan(&txs, &Config::default());
        assert!(hits.is_empty());
    }

    #[test]
    fn option_exercise_excluded() {
        let txs = vec![
            tx("AAA", "Smith", "CEO", "option_exercise", 5000.0, 50.0),
            tx("AAA", "Jones", "CFO", "buy", 1000.0, 50.0),
        ];
        // Only one real buy; below min_distinct_insiders = 2 → filtered.
        let hits = scan(&txs, &Config::default());
        assert!(hits.is_empty());
    }

    #[test]
    fn small_transactions_filtered_by_avg_size() {
        let cfg = Config {
            min_average_transaction: 100_000.0,
            ..Default::default()
        };
        let txs = vec![
            tx("AAA", "Smith", "CEO", "buy", 100.0, 50.0), // $5K
            tx("AAA", "Jones", "CFO", "buy", 100.0, 50.0), // $5K
        ];
        assert!(scan(&txs, &cfg).is_empty());
    }

    #[test]
    fn sorted_by_net_dollar_volume_descending() {
        let txs = vec![
            tx("AAA", "Smith", "CEO", "buy", 1000.0, 50.0),
            tx("AAA", "Jones", "CFO", "buy", 1000.0, 50.0), // AAA net $100K
            tx("BBB", "Brown", "CEO", "buy", 5000.0, 50.0),
            tx("BBB", "White", "CFO", "buy", 5000.0, 50.0), // BBB net $500K
        ];
        let hits = scan(&txs, &Config::default());
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].symbol, "BBB");
        assert_eq!(hits[1].symbol, "AAA");
    }

    #[test]
    fn invalid_share_or_price_skipped() {
        let txs = vec![
            tx("AAA", "Smith", "CEO", "buy", f64::NAN, 50.0),
            tx("AAA", "Jones", "CFO", "buy", 1000.0, -1.0),
            tx("AAA", "Brown", "Director", "buy", 0.0, 50.0),
        ];
        assert!(scan(&txs, &Config::default()).is_empty());
    }

    #[test]
    fn titles_deduplicated_and_sorted() {
        let txs = vec![
            tx("AAA", "Smith", "CEO", "buy", 2000.0, 50.0),
            tx("AAA", "Jones", "CFO", "buy", 2000.0, 50.0),
            tx("AAA", "Brown", "CEO", "buy", 2000.0, 50.0), // duplicate title
        ];
        let hits = scan(&txs, &Config::default());
        assert_eq!(
            hits[0].insider_titles,
            vec!["CEO".to_string(), "CFO".to_string()]
        );
    }
}
