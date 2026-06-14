//! Federal income-tax estimate for TY2025 (IRS Rev. Proc. 2024-40 brackets).
//! Ordinary income (wages + short-term gains − pre-tax deferrals − deduction)
//! is taxed through the progressive brackets; long-term gains + qualified
//! dividends are taxed through the 0/15/20% preferential brackets STACKED on top
//! of ordinary taxable income; FICA (Social Security to the wage base, Medicare,
//! and the 0.9% Additional Medicare over the threshold) applies to wages only.
//! Excludes state tax, AMT, NIIT, SE tax, and credits. Faithful port of the
//! former client-side calculator. Pure compute, not advice.

use serde::{Deserialize, Serialize};

// [floor, rate] ordinary brackets by filing status.
fn ordinary_brackets(status: &str) -> &'static [(f64, f64)] {
    match status {
        "mfj" => &[(0.0, 0.10), (23_850.0, 0.12), (96_950.0, 0.22), (206_700.0, 0.24), (394_600.0, 0.32), (501_050.0, 0.35), (751_600.0, 0.37)],
        "mfs" => &[(0.0, 0.10), (11_925.0, 0.12), (48_475.0, 0.22), (103_350.0, 0.24), (197_300.0, 0.32), (250_525.0, 0.35), (375_800.0, 0.37)],
        "hoh" => &[(0.0, 0.10), (17_000.0, 0.12), (64_850.0, 0.22), (103_350.0, 0.24), (197_300.0, 0.32), (250_500.0, 0.35), (626_350.0, 0.37)],
        _ => &[(0.0, 0.10), (11_925.0, 0.12), (48_475.0, 0.22), (103_350.0, 0.24), (197_300.0, 0.32), (250_525.0, 0.35), (626_350.0, 0.37)],
    }
}

fn ltcg_brackets(status: &str) -> &'static [(f64, f64)] {
    match status {
        "mfj" => &[(0.0, 0.00), (96_700.0, 0.15), (600_050.0, 0.20)],
        "mfs" => &[(0.0, 0.00), (48_350.0, 0.15), (300_000.0, 0.20)],
        "hoh" => &[(0.0, 0.00), (64_750.0, 0.15), (566_700.0, 0.20)],
        _ => &[(0.0, 0.00), (48_350.0, 0.15), (533_400.0, 0.20)],
    }
}

fn std_deduction(status: &str) -> f64 {
    match status {
        "mfj" => 30_000.0,
        "hoh" => 22_500.0,
        _ => 15_000.0, // single, mfs
    }
}

fn addl_medi_threshold(status: &str) -> f64 {
    match status {
        "mfj" => 250_000.0,
        "mfs" => 125_000.0,
        _ => 200_000.0,
    }
}

const SS_WAGE_BASE: f64 = 176_100.0;
const SS_RATE: f64 = 0.062;
const MEDI_RATE: f64 = 0.0145;
const ADDL_MEDI_RATE: f64 = 0.009;

#[derive(Debug, Clone, Deserialize)]
pub struct IncomeTaxInput {
    pub filing_status: String,
    #[serde(default)]
    pub wages_usd: f64,
    #[serde(default)]
    pub short_term_gains_usd: f64,
    #[serde(default)]
    pub long_term_gains_usd: f64,
    #[serde(default)]
    pub qualified_dividends_usd: f64,
    /// 0 = use the standard deduction.
    #[serde(default)]
    pub itemized_override_usd: f64,
    #[serde(default)]
    pub pretax_401k_hsa_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BracketRow {
    pub rate_pct: f64,
    pub from_usd: f64,
    /// None for the top (open-ended) bracket.
    pub to_usd: Option<f64>,
    pub amount_usd: f64,
    pub tax_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct IncomeTaxReport {
    pub total_income_usd: f64,
    pub ordinary_taxable_usd: f64,
    pub deduction_usd: f64,
    pub ordinary_tax_usd: f64,
    pub marginal_rate_pct: f64,
    pub preferential_tax_usd: f64,
    pub social_security_tax_usd: f64,
    pub medicare_tax_usd: f64,
    pub fica_total_usd: f64,
    pub total_tax_usd: f64,
    pub effective_rate_pct: f64,
    pub take_home_usd: f64,
    pub ordinary_breakdown: Vec<BracketRow>,
    pub ltcg_breakdown: Vec<BracketRow>,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

// Progressive ordinary brackets: returns (tax, marginal_rate, breakdown).
fn apply_brackets(brackets: &[(f64, f64)], taxable: f64) -> (f64, f64, Vec<BracketRow>) {
    let mut tax = 0.0;
    let mut marginal = 0.0;
    let mut rows = Vec::with_capacity(brackets.len());
    for (idx, &(from, rate)) in brackets.iter().enumerate() {
        let to = brackets.get(idx + 1).map(|b| b.0);
        if taxable <= from {
            rows.push(BracketRow { rate_pct: round4(rate * 100.0), from_usd: from, to_usd: to, amount_usd: 0.0, tax_usd: 0.0 });
            continue;
        }
        let top = to.map_or(taxable, |t| taxable.min(t));
        let amount = (top - from).max(0.0);
        let seg = amount * rate;
        tax += seg;
        marginal = rate;
        rows.push(BracketRow { rate_pct: round4(rate * 100.0), from_usd: from, to_usd: to, amount_usd: round2(amount), tax_usd: round2(seg) });
    }
    (tax, marginal, rows)
}

// Preferential (LTCG/qualified) brackets stacked on top of `floor` taxable income.
fn apply_ltcg(brackets: &[(f64, f64)], floor: f64, pref: f64) -> (f64, Vec<BracketRow>) {
    let mut tax = 0.0;
    let mut rows = Vec::new();
    if pref <= 0.0 {
        return (tax, rows);
    }
    let ceiling = floor + pref;
    for (idx, &(from, rate)) in brackets.iter().enumerate() {
        let to = brackets.get(idx + 1).map(|b| b.0);
        if ceiling <= from {
            rows.push(BracketRow { rate_pct: round4(rate * 100.0), from_usd: from, to_usd: to, amount_usd: 0.0, tax_usd: 0.0 });
            continue;
        }
        let seg_lo = floor.max(from);
        let seg_hi = to.map_or(ceiling, |t| ceiling.min(t));
        let amount = (seg_hi - seg_lo).max(0.0);
        let seg = amount * rate;
        tax += seg;
        rows.push(BracketRow { rate_pct: round4(rate * 100.0), from_usd: from, to_usd: to, amount_usd: round2(amount), tax_usd: round2(seg) });
    }
    (tax, rows)
}

pub fn generate(i: &IncomeTaxInput) -> IncomeTaxReport {
    let status = i.filing_status.as_str();
    let ordinary_gross = i.wages_usd + i.short_term_gains_usd;
    let ordinary_after_pretax = (ordinary_gross - i.pretax_401k_hsa_usd).max(0.0);
    let deduction = if i.itemized_override_usd > 0.0 { i.itemized_override_usd } else { std_deduction(status) };
    let ordinary_taxable = (ordinary_after_pretax - deduction).max(0.0);
    let pref_taxable = i.long_term_gains_usd + i.qualified_dividends_usd;

    let (ord_tax, marginal, ord_break) = apply_brackets(ordinary_brackets(status), ordinary_taxable);
    let (pref_tax, pref_break) = apply_ltcg(ltcg_brackets(status), ordinary_taxable, pref_taxable);

    let ss_wages = i.wages_usd.min(SS_WAGE_BASE);
    let ss_tax = ss_wages * SS_RATE;
    let medi_tax = i.wages_usd * MEDI_RATE;
    let addl_medi = (i.wages_usd - addl_medi_threshold(status)).max(0.0) * ADDL_MEDI_RATE;
    let fica = ss_tax + medi_tax + addl_medi;

    let total_tax = ord_tax + pref_tax + fica;
    let total_income = i.wages_usd + i.short_term_gains_usd + i.long_term_gains_usd + i.qualified_dividends_usd;
    let eff = if total_income > 0.0 { total_tax / total_income } else { 0.0 };
    let take_home = total_income - i.pretax_401k_hsa_usd - total_tax;

    IncomeTaxReport {
        total_income_usd: round2(total_income),
        ordinary_taxable_usd: round2(ordinary_taxable),
        deduction_usd: round2(deduction),
        ordinary_tax_usd: round2(ord_tax),
        marginal_rate_pct: round4(marginal * 100.0),
        preferential_tax_usd: round2(pref_tax),
        social_security_tax_usd: round2(ss_tax),
        medicare_tax_usd: round2(medi_tax + addl_medi),
        fica_total_usd: round2(fica),
        total_tax_usd: round2(total_tax),
        effective_rate_pct: round4(eff * 100.0),
        take_home_usd: round2(take_home),
        ordinary_breakdown: ord_break,
        ltcg_breakdown: pref_break,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> IncomeTaxInput {
        IncomeTaxInput {
            filing_status: "single".into(),
            wages_usd: 180_000.0,
            short_term_gains_usd: 0.0,
            long_term_gains_usd: 0.0,
            qualified_dividends_usd: 0.0,
            itemized_override_usd: 0.0,
            pretax_401k_hsa_usd: 23_500.0,
        }
    }

    // Pins cross-checked against the original JS compute() in Python.
    #[test]
    fn default_single() {
        let d = generate(&base());
        assert!(close(d.ordinary_taxable_usd, 141_500.0));
        assert!(close(d.ordinary_tax_usd, 26_807.0));
        assert!(close(d.marginal_rate_pct, 24.0));
        assert!(close(d.social_security_tax_usd, 10_918.2));
        assert!(close(d.fica_total_usd, 13_528.2));
        assert!(close(d.total_tax_usd, 40_335.2));
        assert!(close(d.effective_rate_pct, 22.408));
        assert!(d.ltcg_breakdown.is_empty());
    }

    #[test]
    fn ltcg_stacks_above_ordinary() {
        // $40k LTCG on top of $141,500 ordinary taxable (single) → all in the 15% band.
        let d = generate(&IncomeTaxInput { long_term_gains_usd: 40_000.0, ..base() });
        assert!(close(d.preferential_tax_usd, 6_000.0)); // 40k * 15%
        assert!(!d.ltcg_breakdown.is_empty());
    }

    #[test]
    fn social_security_caps_at_wage_base() {
        // Wages far above the SS base → SS tax is capped at base * 6.2%.
        let d = generate(&IncomeTaxInput { wages_usd: 500_000.0, ..base() });
        assert!(close(d.social_security_tax_usd, SS_WAGE_BASE * SS_RATE));
    }

    #[test]
    fn mfj_uses_wider_brackets_and_deduction() {
        let d = generate(&IncomeTaxInput { filing_status: "mfj".into(), ..base() });
        assert!(close(d.deduction_usd, 30_000.0));
        // Same income taxed less under MFJ's wider brackets than single.
        assert!(d.ordinary_tax_usd < 26_807.0);
    }

    #[test]
    fn low_income_no_tax() {
        let d = generate(&IncomeTaxInput { wages_usd: 10_000.0, pretax_401k_hsa_usd: 0.0, ..base() });
        // Below the standard deduction → no ordinary income tax.
        assert!(close(d.ordinary_tax_usd, 0.0));
    }
}
