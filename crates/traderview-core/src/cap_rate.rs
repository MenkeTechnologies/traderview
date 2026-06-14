//! Rental-property underwriting suite — cap rate, cash-on-cash, the 1% rule,
//! GRM, DSCR, and the full NOI/cash-flow P&L. Cap rate is the unlevered yield
//! (NOI ÷ price); cash-on-cash is the levered return on cash invested; the 1%
//! rule flags whether monthly rent ≥ 1% of price; GRM is price ÷ annual gross
//! rent. NOI excludes debt service and depreciation. This is a faithful port of
//! the former client-side calculator so the numbers match exactly. Pure compute,
//! not financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CapRateInput {
    pub price_usd: f64,
    pub monthly_rent_usd: f64,
    /// Down payment as a fraction (0.25 = 25%).
    #[serde(default)]
    pub down_payment_frac: f64,
    /// Mortgage rate as a fraction (0.07 = 7%).
    #[serde(default)]
    pub mortgage_rate_frac: f64,
    #[serde(default = "default_term")]
    pub loan_term_years: u32,
    #[serde(default)]
    pub closing_costs_usd: f64,
    /// Vacancy as a fraction of gross rent.
    #[serde(default)]
    pub vacancy_frac: f64,
    #[serde(default)]
    pub property_tax_annual_usd: f64,
    #[serde(default)]
    pub insurance_annual_usd: f64,
    /// Maintenance/CapEx as a fraction of gross rent.
    #[serde(default)]
    pub maintenance_frac: f64,
    /// Property management as a fraction of gross rent.
    #[serde(default)]
    pub management_frac: f64,
    #[serde(default)]
    pub hoa_monthly_usd: f64,
}

fn default_term() -> u32 {
    30
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct CapRateReport {
    pub cap_rate_pct: f64,
    pub cash_on_cash_pct: f64,
    pub annual_cash_flow_usd: f64,
    pub monthly_cash_flow_usd: f64,
    /// None when there is no debt service (DSCR is infinite).
    pub dscr: Option<f64>,
    pub one_pct_rule_pct: f64,
    pub one_pct_ok: bool,
    pub grm: f64,
    pub verdict: String,
    // P&L lines (annual).
    pub gross_annual_rent_usd: f64,
    pub vacancy_loss_usd: f64,
    pub effective_gross_income_usd: f64,
    pub maintenance_usd: f64,
    pub management_usd: f64,
    pub hoa_annual_usd: f64,
    pub total_operating_expenses_usd: f64,
    pub noi_usd: f64,
    pub monthly_pi_usd: f64,
    pub annual_debt_service_usd: f64,
    pub cash_invested_usd: f64,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

pub fn generate(i: &CapRateInput) -> CapRateReport {
    if i.price_usd <= 0.0 || i.monthly_rent_usd <= 0.0 {
        return CapRateReport::default();
    }
    let gross_annual_rent = i.monthly_rent_usd * 12.0;
    let vacancy = gross_annual_rent * i.vacancy_frac;
    let maintenance = gross_annual_rent * i.maintenance_frac;
    let management = gross_annual_rent * i.management_frac;
    let hoa_annual = i.hoa_monthly_usd * 12.0;
    let egi = gross_annual_rent - vacancy;
    let opex = i.property_tax_annual_usd + i.insurance_annual_usd + maintenance + management + hoa_annual;
    let noi = egi - opex;
    let cap_rate = noi / i.price_usd;

    // Mortgage payment (closed-form annuity; r=0 falls back to straight-line).
    let loan = i.price_usd * (1.0 - i.down_payment_frac);
    let r_m = i.mortgage_rate_frac / 12.0;
    let n = (i.loan_term_years * 12) as f64;
    let pi = if r_m == 0.0 {
        if n > 0.0 { loan / n } else { 0.0 }
    } else {
        loan * r_m * (1.0 + r_m).powf(n) / ((1.0 + r_m).powf(n) - 1.0)
    };
    let annual_debt_service = pi * 12.0;
    let cash_flow = noi - annual_debt_service;
    let cash_invested = i.price_usd * i.down_payment_frac + i.closing_costs_usd;
    let cash_on_cash = if cash_invested > 0.0 { cash_flow / cash_invested } else { 0.0 };
    let dscr = if annual_debt_service > 0.0 { Some(round4(noi / annual_debt_service)) } else { None };

    let one_pct_rule = (i.monthly_rent_usd / i.price_usd) * 100.0;
    let grm = i.price_usd / gross_annual_rent;

    let verdict = if cap_rate >= 0.08 {
        "STRONG"
    } else if cap_rate >= 0.06 {
        "OK"
    } else if cap_rate >= 0.04 {
        "WEAK"
    } else {
        "PASS"
    };

    CapRateReport {
        cap_rate_pct: round4(cap_rate * 100.0),
        cash_on_cash_pct: round4(cash_on_cash * 100.0),
        annual_cash_flow_usd: round2(cash_flow),
        monthly_cash_flow_usd: round2(cash_flow / 12.0),
        dscr,
        one_pct_rule_pct: round4(one_pct_rule),
        one_pct_ok: one_pct_rule >= 1.0,
        grm: round4(grm),
        verdict: verdict.to_string(),
        gross_annual_rent_usd: round2(gross_annual_rent),
        vacancy_loss_usd: round2(vacancy),
        effective_gross_income_usd: round2(egi),
        maintenance_usd: round2(maintenance),
        management_usd: round2(management),
        hoa_annual_usd: round2(hoa_annual),
        total_operating_expenses_usd: round2(opex),
        noi_usd: round2(noi),
        monthly_pi_usd: round2(pi),
        annual_debt_service_usd: round2(annual_debt_service),
        cash_invested_usd: round2(cash_invested),
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> CapRateInput {
        CapRateInput {
            price_usd: 350_000.0,
            monthly_rent_usd: 2_800.0,
            down_payment_frac: 0.25,
            mortgage_rate_frac: 0.07,
            loan_term_years: 30,
            closing_costs_usd: 9_000.0,
            vacancy_frac: 0.05,
            property_tax_annual_usd: 3_800.0,
            insurance_annual_usd: 1_400.0,
            maintenance_frac: 0.10,
            management_frac: 0.08,
            hoa_monthly_usd: 0.0,
        }
    }

    // Pins cross-checked against the original JS compute() in Python.
    #[test]
    fn default_underwriting() {
        let d = generate(&base());
        assert!(close(d.noi_usd, 20_672.0));
        assert!(close(d.cap_rate_pct, 5.9063));
        assert!(close(d.monthly_pi_usd, 1_746.42));
        assert!(close(d.annual_cash_flow_usd, -285.03));
        assert!(close(d.cash_on_cash_pct, -0.2954));
        assert!(close(d.dscr.unwrap(), 0.9864));
        assert!(close(d.one_pct_rule_pct, 0.8));
        assert!(!d.one_pct_ok);
        assert!(close(d.grm, 10.4167));
        assert!(close(d.effective_gross_income_usd, 31_920.0));
        assert!(close(d.total_operating_expenses_usd, 11_248.0));
        assert_eq!(d.verdict, "WEAK");
    }

    #[test]
    fn all_cash_has_no_dscr() {
        let d = generate(&CapRateInput { down_payment_frac: 1.0, ..base() });
        assert!(d.dscr.is_none());
        // No debt service → cash flow equals NOI.
        assert!(close(d.annual_cash_flow_usd, d.noi_usd));
    }

    #[test]
    fn one_pct_rule_passes_when_rent_high() {
        // Monthly rent at 1% of a low price.
        let d = generate(&CapRateInput { price_usd: 200_000.0, monthly_rent_usd: 2_000.0, ..base() });
        assert!(close(d.one_pct_rule_pct, 1.0));
        assert!(d.one_pct_ok);
    }

    #[test]
    fn strong_verdict_at_high_cap() {
        // Cheap price + strong rent + no financing/opex drag → high cap rate.
        let d = generate(&CapRateInput {
            price_usd: 150_000.0,
            monthly_rent_usd: 2_500.0,
            vacancy_frac: 0.0,
            property_tax_annual_usd: 0.0,
            insurance_annual_usd: 0.0,
            maintenance_frac: 0.0,
            management_frac: 0.0,
            ..base()
        });
        assert!(d.cap_rate_pct >= 8.0);
        assert_eq!(d.verdict, "STRONG");
    }

    #[test]
    fn zero_price_invalid() {
        let d = generate(&CapRateInput { price_usd: 0.0, ..base() });
        assert!(!d.valid);
    }
}
