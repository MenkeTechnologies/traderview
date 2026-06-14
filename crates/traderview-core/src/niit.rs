//! Net Investment Income Tax (NIIT) — the ACA 3.8% surtax on investment income
//! for high earners. It applies to the LESSER of net investment income or MAGI
//! over a filing-status threshold (frozen since 2013: $200k single/HoH, $250k
//! MFJ, $125k MFS). Net investment income = interest + dividends + net capital
//! gains + rents + royalties/passive business income − allocable deductions.
//! Faithful port of the former client-side calculator. Pure compute, not advice.

use serde::{Deserialize, Serialize};

const NIIT_RATE: f64 = 0.038;

fn threshold(status: &str) -> f64 {
    match status {
        "mfj" => 250_000.0,
        "mfs" => 125_000.0,
        // single and hoh share $200k.
        _ => 200_000.0,
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NiitInput {
    pub filing_status: String,
    pub magi_usd: f64,
    #[serde(default)]
    pub interest_usd: f64,
    #[serde(default)]
    pub dividends_usd: f64,
    #[serde(default)]
    pub net_capital_gains_usd: f64,
    #[serde(default)]
    pub rental_net_income_usd: f64,
    #[serde(default)]
    pub royalties_passive_usd: f64,
    #[serde(default)]
    pub allocable_deductions_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct NiitReport {
    pub threshold_usd: f64,
    pub magi_excess_usd: f64,
    pub net_investment_income_usd: f64,
    pub subject_to_niit_usd: f64,
    pub niit_tax_usd: f64,
    pub effective_on_investment_pct: f64,
    /// True when no NIIT is owed (below threshold or no net investment income).
    pub not_subject: bool,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

pub fn generate(i: &NiitInput) -> NiitReport {
    if i.magi_usd < 0.0 {
        return NiitReport::default();
    }
    let gross_investment = i.interest_usd
        + i.dividends_usd
        + i.net_capital_gains_usd
        + i.rental_net_income_usd
        + i.royalties_passive_usd;
    let net_investment = (gross_investment - i.allocable_deductions_usd).max(0.0);
    let thresh = threshold(&i.filing_status);
    let magi_excess = (i.magi_usd - thresh).max(0.0);
    let subject = net_investment.min(magi_excess);
    let niit_tax = subject * NIIT_RATE;
    let eff = if net_investment > 0.0 { niit_tax / net_investment } else { 0.0 };

    NiitReport {
        threshold_usd: thresh,
        magi_excess_usd: round2(magi_excess),
        net_investment_income_usd: round2(net_investment),
        subject_to_niit_usd: round2(subject),
        niit_tax_usd: round2(niit_tax),
        effective_on_investment_pct: round4(eff * 100.0),
        not_subject: subject <= 0.0,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> NiitInput {
        NiitInput {
            filing_status: "mfj".into(),
            magi_usd: 320_000.0,
            interest_usd: 5_000.0,
            dividends_usd: 12_000.0,
            net_capital_gains_usd: 40_000.0,
            rental_net_income_usd: 8_000.0,
            royalties_passive_usd: 0.0,
            allocable_deductions_usd: 0.0,
        }
    }

    // Pins cross-checked against the original JS compute().
    #[test]
    fn default_mfj() {
        let d = generate(&base());
        assert!(close(d.threshold_usd, 250_000.0));
        assert!(close(d.magi_excess_usd, 70_000.0));
        assert!(close(d.net_investment_income_usd, 65_000.0));
        assert!(close(d.subject_to_niit_usd, 65_000.0));
        assert!(close(d.niit_tax_usd, 2_470.0));
        assert!(close(d.effective_on_investment_pct, 3.8));
        assert!(!d.not_subject);
    }

    #[test]
    fn below_threshold_no_tax() {
        let d = generate(&NiitInput { magi_usd: 200_000.0, ..base() });
        assert!(close(d.magi_excess_usd, 0.0));
        assert!(close(d.niit_tax_usd, 0.0));
        assert!(d.not_subject);
    }

    #[test]
    fn magi_excess_caps_subject() {
        // MAGI only $10k over threshold, but $65k net inv → subject capped at $10k.
        let d = generate(&NiitInput { magi_usd: 260_000.0, ..base() });
        assert!(close(d.subject_to_niit_usd, 10_000.0));
        assert!(close(d.niit_tax_usd, 380.0));
    }

    #[test]
    fn single_threshold() {
        let d = generate(&NiitInput { filing_status: "single".into(), ..base() });
        assert!(close(d.threshold_usd, 200_000.0));
    }

    #[test]
    fn deductions_reduce_net() {
        let d = generate(&NiitInput { allocable_deductions_usd: 5_000.0, ..base() });
        assert!(close(d.net_investment_income_usd, 60_000.0));
    }
}
