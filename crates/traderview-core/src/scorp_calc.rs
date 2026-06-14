//! S-corp election calculator — LLC sole-prop vs S-corp tax burden at a given
//! net business income. A sole proprietor pays self-employment tax (15.3%, with
//! the Social Security portion capped) on 92.35% of net. An S-corp owner instead
//! takes "reasonable compensation" as W-2 wages (subject to FICA, split
//! employee/employer) and the rest as distributions free of SE/FICA tax — so the
//! gross saving is the SE tax minus total FICA, net of payroll-service and extra
//! filing overhead. Reports both burdens, the net saving, and an elect/marginal/
//! skip recommendation. Faithful port of the former client-side calculator.
//! Pure compute, not advice.

use serde::{Deserialize, Serialize};

const SS_BASE: f64 = 168_600.0;
const SS_RATE: f64 = 0.124;
const SS_RATE_EMPLOYEE: f64 = 0.062;
const SS_RATE_EMPLOYER: f64 = 0.062;
const MEDICARE_RATE: f64 = 0.029;
const MEDICARE_RATE_HALF: f64 = 0.0145;
const SE_DEDUCTION: f64 = 0.9235;

#[derive(Debug, Clone, Deserialize)]
pub struct ScorpInput {
    pub net_income_usd: f64,
    /// Reasonable compensation as a fraction of net (e.g. 0.40 = 40%).
    pub reasonable_comp_fraction: f64,
    pub payroll_cost_usd: f64,
    pub extra_filing_cost_usd: f64,
    /// Collected by the form for context; does not affect the SE/FICA comparison.
    #[serde(default)]
    pub marginal_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct ScorpReport {
    pub se_base_usd: f64,
    pub se_tax_usd: f64,
    pub w2_wages_usd: f64,
    pub distributions_usd: f64,
    pub fica_employee_usd: f64,
    pub fica_employer_usd: f64,
    pub total_fica_usd: f64,
    pub gross_savings_usd: f64,
    pub total_overhead_usd: f64,
    pub net_savings_usd: f64,
    /// "elect", "marginal", or "skip".
    pub recommendation: String,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn generate(i: &ScorpInput) -> ScorpReport {
    if i.net_income_usd <= 0.0 {
        return ScorpReport::default();
    }
    let net = i.net_income_usd;
    // Sole prop: SE tax on 92.35% of net (SS capped, Medicare uncapped).
    let se_base = net * SE_DEDUCTION;
    let ss_part = se_base.min(SS_BASE) * SS_RATE;
    let medicare_part = se_base * MEDICARE_RATE;
    let se_tax = ss_part + medicare_part;

    // S-corp: W-2 reasonable comp drives FICA; distributions are FICA-free.
    let w2_wages = net * i.reasonable_comp_fraction;
    let distributions = net - w2_wages;
    let ss_wages = w2_wages.min(SS_BASE);
    let fica_employee = ss_wages * SS_RATE_EMPLOYEE + w2_wages * MEDICARE_RATE_HALF;
    let fica_employer = ss_wages * SS_RATE_EMPLOYER + w2_wages * MEDICARE_RATE_HALF;
    let total_fica = fica_employee + fica_employer;

    let total_overhead = i.payroll_cost_usd + i.extra_filing_cost_usd;
    let gross_savings = se_tax - total_fica;
    let net_savings = gross_savings - total_overhead;
    let recommendation = if net_savings > 1500.0 {
        "elect"
    } else if net_savings > 0.0 {
        "marginal"
    } else {
        "skip"
    };

    ScorpReport {
        se_base_usd: round2(se_base),
        se_tax_usd: round2(se_tax),
        w2_wages_usd: round2(w2_wages),
        distributions_usd: round2(distributions),
        fica_employee_usd: round2(fica_employee),
        fica_employer_usd: round2(fica_employer),
        total_fica_usd: round2(total_fica),
        gross_savings_usd: round2(gross_savings),
        total_overhead_usd: round2(total_overhead),
        net_savings_usd: round2(net_savings),
        recommendation: recommendation.to_string(),
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> ScorpInput {
        ScorpInput {
            net_income_usd: 200_000.0,
            reasonable_comp_fraction: 0.40,
            payroll_cost_usd: 600.0,
            extra_filing_cost_usd: 1_200.0,
            marginal_rate_pct: 32.0,
        }
    }

    // Pins cross-checked against the JS compute() in Python.
    #[test]
    fn default_elects() {
        let d = generate(&base());
        assert!(d.valid);
        assert!(close(d.se_base_usd, 184_700.0));
        assert!(close(d.se_tax_usd, 26_262.7));
        assert!(close(d.w2_wages_usd, 80_000.0));
        assert!(close(d.distributions_usd, 120_000.0));
        assert!(close(d.fica_employee_usd, 6_120.0));
        assert!(close(d.fica_employer_usd, 6_120.0));
        assert!(close(d.total_fica_usd, 12_240.0));
        assert!(close(d.gross_savings_usd, 14_022.7));
        assert!(close(d.total_overhead_usd, 1_800.0));
        assert!(close(d.net_savings_usd, 12_222.7));
        assert_eq!(d.recommendation, "elect");
    }

    #[test]
    fn low_income_skips() {
        let d = generate(&ScorpInput { net_income_usd: 20_000.0, ..base() });
        assert!(close(d.net_savings_usd, -198.09)); // overhead exceeds SE saving
        assert_eq!(d.recommendation, "skip");
    }

    #[test]
    fn ss_cap_limits_sole_prop_tax() {
        // Above the SS wage base, only Medicare keeps scaling on the excess.
        let d = generate(&ScorpInput { net_income_usd: 500_000.0, ..base() });
        let se_base = 500_000.0 * SE_DEDUCTION;
        let expected = SS_BASE * SS_RATE + se_base * MEDICARE_RATE;
        assert!(close(d.se_tax_usd, round2(expected)));
    }

    #[test]
    fn invalid_when_income_zero() {
        let d = generate(&ScorpInput { net_income_usd: 0.0, ..base() });
        assert!(!d.valid);
    }
}
