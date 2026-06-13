//! Lease cost with annual rent escalations and free-rent concessions — the
//! total and present value of a lease, and the effective (average) monthly
//! rent tenants compare offers on.
//!
//! Rent steps up at each anniversary: month `m` (1-based) sits in year
//! `(m−1)/12`, so `rent = base × (1 + escalation)^year`. The first
//! `free_months` are concessions (rent waived). The effective monthly rent
//! spreads the total over the full term, including the free months.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RentEscalationInput {
    pub base_monthly_rent_usd: f64,
    /// Annual rent escalation, percent (applied each anniversary).
    pub annual_escalation_pct: f64,
    pub term_months: u32,
    /// Free-rent months at the start (concession).
    #[serde(default)]
    pub free_months: u32,
    /// Annual discount rate for the present value, percent (0 = nominal only).
    #[serde(default)]
    pub discount_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RentEscalationResult {
    /// Total nominal rent paid over the term.
    pub total_rent_usd: f64,
    /// Present value of the rent stream at the discount rate.
    pub npv_usd: f64,
    /// Total ÷ term — the effective monthly rent.
    pub effective_monthly_rent_usd: f64,
    /// Value of the waived free-rent months (at the base rate).
    pub concession_value_usd: f64,
    /// Rent in the final month (fully escalated).
    pub final_monthly_rent_usd: f64,
}

pub fn analyze(input: &RentEscalationInput) -> RentEscalationResult {
    let esc = input.annual_escalation_pct / 100.0;
    let rm = input.discount_rate_pct / 100.0 / 12.0;

    let mut total = 0.0;
    let mut npv = 0.0;
    for m in 1..=input.term_months {
        let year = (m - 1) / 12;
        let mut rent = input.base_monthly_rent_usd * (1.0 + esc).powi(year as i32);
        if m <= input.free_months {
            rent = 0.0;
        }
        total += rent;
        npv += rent / (1.0 + rm).powi(m as i32);
    }

    let effective = if input.term_months > 0 {
        total / input.term_months as f64
    } else {
        0.0
    };
    let final_rent = if input.term_months > 0 {
        let year = (input.term_months - 1) / 12;
        input.base_monthly_rent_usd * (1.0 + esc).powi(year as i32)
    } else {
        0.0
    };

    RentEscalationResult {
        total_rent_usd: total,
        npv_usd: npv,
        effective_monthly_rent_usd: effective,
        concession_value_usd: input.free_months as f64 * input.base_monthly_rent_usd,
        final_monthly_rent_usd: final_rent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn base() -> RentEscalationInput {
        RentEscalationInput {
            base_monthly_rent_usd: 2000.0,
            annual_escalation_pct: 3.0,
            term_months: 36,
            free_months: 2,
            discount_rate_pct: 6.0,
        }
    }

    #[test]
    fn total_rent() {
        assert!(close(analyze(&base()).total_rent_usd, 70181.60));
    }

    #[test]
    fn effective_monthly() {
        assert!(close(analyze(&base()).effective_monthly_rent_usd, 1949.4889));
    }

    #[test]
    fn npv_below_nominal() {
        let r = analyze(&base());
        assert!(close(r.npv_usd, 63684.0024));
        assert!(r.npv_usd < r.total_rent_usd);
    }

    #[test]
    fn final_month_fully_escalated() {
        // Year 2 (months 25-36): 2000 × 1.03² = 2121.80.
        assert!(close(analyze(&base()).final_monthly_rent_usd, 2121.80));
    }

    #[test]
    fn concession_value() {
        assert!(close(analyze(&base()).concession_value_usd, 4000.0));
    }

    #[test]
    fn no_escalation_is_flat() {
        let r = analyze(&RentEscalationInput {
            annual_escalation_pct: 0.0,
            free_months: 0,
            discount_rate_pct: 0.0,
            ..base()
        });
        // 36 × 2000 = 72,000; effective equals base.
        assert!(close(r.total_rent_usd, 72_000.0));
        assert!(close(r.effective_monthly_rent_usd, 2000.0));
    }

    #[test]
    fn free_months_lower_effective_rent() {
        let with_free = analyze(&base());
        let no_free = analyze(&RentEscalationInput {
            free_months: 0,
            ..base()
        });
        assert!(with_free.effective_monthly_rent_usd < no_free.effective_monthly_rent_usd);
    }

    #[test]
    fn zero_discount_npv_equals_total() {
        let r = analyze(&RentEscalationInput {
            discount_rate_pct: 0.0,
            ..base()
        });
        assert!(close(r.npv_usd, r.total_rent_usd));
    }
}
