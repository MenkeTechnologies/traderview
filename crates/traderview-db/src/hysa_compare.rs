//! High-Yield Savings Account comparison.
//!
//! Compares N HYSA offers on net effective APY (after monthly fees)
//! and interest earned over a given holding period. Penalises accounts
//! whose minimum balance exceeds your deposit (effectively un-usable).
//!
//! Inputs: list of banks with name / APY / monthly_fee / min_balance,
//! plus your deposit amount and projection months.
//!
//! Compute returns per-bank effective APY, total interest earned (with
//! monthly compounding), total fees paid, net gain, min-balance flag,
//! and the winning bank (highest net gain).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BankInput {
    pub name: String,
    pub apy_pct: f64,
    #[serde(default)]
    pub monthly_fee_usd: f64,
    #[serde(default)]
    pub min_balance_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HysaCompareInput {
    pub deposit_usd: f64,
    pub months: u32,
    #[serde(default)]
    pub banks: Vec<BankInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BankResult {
    pub name: String,
    pub apy_pct: f64,
    pub effective_apy_pct: f64,
    pub interest_earned_usd: f64,
    pub total_fees_usd: f64,
    pub net_gain_usd: f64,
    pub final_balance_usd: f64,
    pub min_balance_met: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct HysaCompareReport {
    pub deposit_usd: f64,
    pub months: u32,
    pub banks: Vec<BankResult>,
    pub winner_name: String,
    pub winner_net_gain_usd: f64,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn compute(input: &HysaCompareInput) -> HysaCompareReport {
    let mut banks: Vec<BankResult> = Vec::with_capacity(input.banks.len());
    for b in &input.banks {
        let met = input.deposit_usd >= b.min_balance_usd;
        let monthly_rate = b.apy_pct / 100.0 / 12.0;
        let total_months = input.months as f64;
        let mut bal = input.deposit_usd;
        let mut interest_acc = 0.0_f64;
        let mut fees_acc = 0.0_f64;
        if met {
            for _ in 0..input.months {
                let interest = bal * monthly_rate;
                interest_acc += interest;
                bal += interest;
                bal -= b.monthly_fee_usd;
                fees_acc += b.monthly_fee_usd;
                if bal < 0.0 { bal = 0.0; }
            }
        }
        let net = interest_acc - fees_acc;
        // Effective APY = (1 + monthly_rate)^12 − 1, minus annual fee drag.
        let eff_apy_raw = ((1.0 + monthly_rate).powi(12) - 1.0) * 100.0;
        let annual_fee_drag = if input.deposit_usd > 0.0 {
            b.monthly_fee_usd * 12.0 / input.deposit_usd * 100.0
        } else { 0.0 };
        let effective_apy = if met { eff_apy_raw - annual_fee_drag } else { 0.0 };
        let _ = total_months;
        banks.push(BankResult {
            name: b.name.clone(),
            apy_pct: b.apy_pct,
            effective_apy_pct: effective_apy,
            interest_earned_usd: interest_acc,
            total_fees_usd: fees_acc,
            net_gain_usd: net,
            final_balance_usd: bal,
            min_balance_met: met,
        });
    }
    // Find winner by highest net gain (and met min balance).
    let winner = banks
        .iter()
        .filter(|b| b.min_balance_met)
        .max_by(|a, b| a.net_gain_usd.partial_cmp(&b.net_gain_usd).unwrap_or(std::cmp::Ordering::Equal));
    let (winner_name, winner_net) = match winner {
        Some(b) => (b.name.clone(), b.net_gain_usd),
        None => (String::new(), 0.0),
    };
    HysaCompareReport {
        deposit_usd: input.deposit_usd,
        months: input.months,
        banks,
        winner_name,
        winner_net_gain_usd: winner_net,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bk(name: &str, apy: f64, fee: f64, min: f64) -> BankInput {
        BankInput {
            name: name.into(), apy_pct: apy, monthly_fee_usd: fee, min_balance_usd: min,
        }
    }

    #[test]
    fn compute_empty_banks() {
        let r = compute(&HysaCompareInput {
            deposit_usd: 10_000.0, months: 12, banks: vec![],
        });
        assert_eq!(r.winner_name, "");
        assert_eq!(r.banks.len(), 0);
    }

    #[test]
    fn compute_single_bank_basic() {
        let r = compute(&HysaCompareInput {
            deposit_usd: 10_000.0, months: 12,
            banks: vec![bk("Ally", 4.5, 0.0, 0.0)],
        });
        assert_eq!(r.banks.len(), 1);
        // 10k at 4.5% monthly compound over 12mo ≈ $459.36 interest
        let b = &r.banks[0];
        assert!((b.interest_earned_usd - 459.36).abs() < 1.0);
        assert_eq!(b.total_fees_usd, 0.0);
        assert!((b.effective_apy_pct - 4.594).abs() < 0.05, "eff_apy got {}", b.effective_apy_pct);
    }

    #[test]
    fn compute_min_balance_not_met() {
        let r = compute(&HysaCompareInput {
            deposit_usd: 1_000.0, months: 12,
            banks: vec![bk("PremiumOnly", 6.0, 0.0, 25_000.0)],
        });
        let b = &r.banks[0];
        assert!(!b.min_balance_met);
        assert_eq!(b.interest_earned_usd, 0.0);
        assert_eq!(b.effective_apy_pct, 0.0);
    }

    #[test]
    fn compute_fees_reduce_net_gain() {
        let r = compute(&HysaCompareInput {
            deposit_usd: 10_000.0, months: 12,
            banks: vec![bk("Fee Trap", 5.0, 10.0, 0.0)],
        });
        let b = &r.banks[0];
        assert_eq!(b.total_fees_usd, 120.0);
        assert!(b.net_gain_usd < b.interest_earned_usd);
    }

    #[test]
    fn compute_picks_winner_by_net_gain() {
        let r = compute(&HysaCompareInput {
            deposit_usd: 10_000.0, months: 12,
            banks: vec![
                bk("LowFee",  4.5, 0.0, 0.0),
                bk("HighFee", 5.5, 50.0, 0.0),  // higher APY but $600 fees > $100 extra interest
                bk("Locked",  6.0, 0.0, 25_000.0),
            ],
        });
        assert_eq!(r.winner_name, "LowFee");
    }

    #[test]
    fn compute_no_eligible_winner_empty_name() {
        let r = compute(&HysaCompareInput {
            deposit_usd: 1_000.0, months: 12,
            banks: vec![bk("HighMin", 6.0, 0.0, 50_000.0)],
        });
        assert_eq!(r.winner_name, "");
        assert_eq!(r.winner_net_gain_usd, 0.0);
    }

    #[test]
    fn compute_final_balance_principal_plus_interest_minus_fees() {
        let r = compute(&HysaCompareInput {
            deposit_usd: 10_000.0, months: 12,
            banks: vec![bk("Bank", 5.0, 10.0, 0.0)],
        });
        let b = &r.banks[0];
        let expected = 10_000.0 + b.interest_earned_usd - b.total_fees_usd;
        assert!((b.final_balance_usd - expected).abs() < 0.01);
    }

    #[test]
    fn compute_zero_apy_zero_interest() {
        let r = compute(&HysaCompareInput {
            deposit_usd: 10_000.0, months: 12,
            banks: vec![bk("Mattress", 0.0, 0.0, 0.0)],
        });
        let b = &r.banks[0];
        assert_eq!(b.interest_earned_usd, 0.0);
        assert!(b.effective_apy_pct.abs() < 1e-6);
    }

    #[test]
    fn compute_zero_months_no_interest() {
        let r = compute(&HysaCompareInput {
            deposit_usd: 10_000.0, months: 0,
            banks: vec![bk("Ally", 4.5, 0.0, 0.0)],
        });
        let b = &r.banks[0];
        assert_eq!(b.interest_earned_usd, 0.0);
    }

    #[test]
    fn compute_effective_apy_accounts_for_compounding() {
        let r = compute(&HysaCompareInput {
            deposit_usd: 10_000.0, months: 12,
            banks: vec![bk("Bank", 5.0, 0.0, 0.0)],
        });
        let b = &r.banks[0];
        // (1 + 0.05/12)^12 − 1 = 5.116% effective
        assert!((b.effective_apy_pct - 5.116).abs() < 0.05);
    }
}
