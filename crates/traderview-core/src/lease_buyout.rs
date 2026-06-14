//! Lease buyout / early-termination settlement — the financial settlement a
//! tenant pays to be released from a lease early. Distinct from the
//! lease-termination notice (which is the letter): this computes the settlement
//! figure as the present value of the remaining base-rent stream (an annuity
//! discounted at a monthly rate), plus unamortized concessions (tenant
//! improvements and leasing commissions not yet earned back), plus a termination
//! fee, less the landlord's expected reletting recovery. No existing generator
//! assembles this multi-component buyout. Drafting aid, not legal/financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LeaseBuyoutInput {
    pub landlord_name: String,
    pub tenant_name: String,
    #[serde(default)]
    pub property_label: String,
    /// Monthly base rent remaining on the lease.
    pub monthly_rent_usd: f64,
    /// Months remaining on the term.
    pub remaining_months: u32,
    /// Annual discount rate for present-valuing the remaining rent, percent.
    #[serde(default)]
    pub annual_discount_pct: f64,
    /// Unamortized concessions (TI + leasing commissions) the landlord recovers.
    #[serde(default)]
    pub unamortized_concessions_usd: f64,
    /// Termination fee expressed in months of rent.
    #[serde(default)]
    pub termination_fee_months: f64,
    /// Expected recovery from re-letting the space, which offsets the settlement.
    #[serde(default)]
    pub reletting_recovery_usd: f64,
    pub date: String,
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LeaseBuyout {
    pub title: String,
    /// Nominal remaining rent (monthly × months), undiscounted.
    pub nominal_remaining_rent_usd: f64,
    /// Present value of the remaining rent stream.
    pub pv_remaining_rent_usd: f64,
    /// Termination fee (months × monthly rent).
    pub termination_fee_usd: f64,
    pub unamortized_concessions_usd: f64,
    pub reletting_recovery_usd: f64,
    /// PV rent + concessions + fee − reletting recovery, floored at 0.
    pub settlement_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

/// Present-value annuity factor for `n` monthly payments at monthly rate `r`.
/// With r = 0 the factor is just n (no discounting).
fn annuity_factor(r: f64, n: u32) -> f64 {
    if r > 0.0 {
        (1.0 - (1.0 + r).powi(-(n as i32))) / r
    } else {
        n as f64
    }
}

pub fn generate(i: &LeaseBuyoutInput) -> LeaseBuyout {
    let nominal = cents(i.monthly_rent_usd * i.remaining_months as f64);
    let r = i.annual_discount_pct / 100.0 / 12.0;
    let pv_rent = cents(i.monthly_rent_usd * annuity_factor(r, i.remaining_months));
    let fee = cents(i.termination_fee_months * i.monthly_rent_usd);
    let settlement = cents(
        (pv_rent + i.unamortized_concessions_usd + fee - i.reletting_recovery_usd).max(0.0),
    );

    let property = if i.property_label.trim().is_empty() {
        "the Premises".to_string()
    } else {
        i.property_label.trim().to_string()
    };

    let disc_desc = if i.annual_discount_pct > 0.0 {
        format!("present-valued at {:.2}% per year", i.annual_discount_pct)
    } else {
        "undiscounted".to_string()
    };

    let calc_body = format!(
        "Remaining rent of {} ({} months × {}) {} is {}. Adding unamortized concessions of {} and a termination fee of {} ({:.1} months' rent), and subtracting expected reletting recovery of {}, the buyout settlement is {}.",
        money(nominal),
        i.remaining_months,
        money(i.monthly_rent_usd),
        disc_desc,
        money(pv_rent),
        money(i.unamortized_concessions_usd),
        money(fee),
        i.termination_fee_months,
        money(i.reletting_recovery_usd),
        money(settlement)
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("This settlement is governed by the lease and the laws of the State of {}.", i.state)
    } else {
        format!("This settlement is governed by the lease and the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Landlord: {}\nTenant: {}\nPremises: {}\nDate: {}",
                i.landlord_name, i.tenant_name, property, i.date
            ),
        },
        DocClause {
            heading: "1. Early Termination".into(),
            body: format!(
                "The Tenant requests to terminate the lease early, with {} months remaining at a monthly base rent of {}. In consideration of release, the Tenant shall pay the buyout settlement computed below.",
                i.remaining_months, money(i.monthly_rent_usd)
            ),
        },
        DocClause { heading: "2. Settlement Calculation".into(), body: calc_body },
        DocClause {
            heading: "3. Payment & Release".into(),
            body: format!(
                "Upon payment of {} and surrender of the Premises in the required condition, the Landlord releases the Tenant from further rent and obligations accruing after the termination date.",
                money(settlement)
            ),
        },
        DocClause {
            heading: "4. Mitigation".into(),
            body: "The reletting recovery reflects the Landlord's expected mitigation. If the Landlord relets sooner or at higher rent than assumed, the parties have nonetheless agreed to the fixed settlement above in full satisfaction.".into(),
        },
        DocClause { heading: "5. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Landlord: ____________________  Date: __________\n{}\n\nTenant: ____________________  Date: __________\n{}",
                i.landlord_name, i.tenant_name
            ),
        },
    ];

    LeaseBuyout {
        title: "Lease Buyout / Early Termination Settlement".into(),
        nominal_remaining_rent_usd: nominal,
        pv_remaining_rent_usd: pv_rent,
        termination_fee_usd: fee,
        unamortized_concessions_usd: cents(i.unamortized_concessions_usd),
        reletting_recovery_usd: cents(i.reletting_recovery_usd),
        settlement_usd: settlement,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.02
    }

    fn base() -> LeaseBuyoutInput {
        LeaseBuyoutInput {
            landlord_name: "Tower Owners LP".into(),
            tenant_name: "Departing Tenant LLC".into(),
            property_label: "Suite 900".into(),
            monthly_rent_usd: 10_000.0,
            remaining_months: 24,
            annual_discount_pct: 6.0,
            unamortized_concessions_usd: 20_000.0,
            termination_fee_months: 3.0,
            reletting_recovery_usd: 40_000.0,
            date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn pv_and_settlement() {
        let d = generate(&base());
        assert!(close(d.nominal_remaining_rent_usd, 240_000.0));
        // PV of 24 × 10,000 at 0.5%/mo ≈ 225,628.66.
        assert!(close(d.pv_remaining_rent_usd, 225_628.66));
        assert!(close(d.termination_fee_usd, 30_000.0));
        // 225,628.66 + 20,000 + 30,000 − 40,000 = 235,628.66.
        assert!(close(d.settlement_usd, 235_628.66));
    }

    #[test]
    fn zero_discount_uses_nominal() {
        let d = generate(&LeaseBuyoutInput { annual_discount_pct: 0.0, ..base() });
        assert!(close(d.pv_remaining_rent_usd, 240_000.0));
        // 240,000 + 20,000 + 30,000 − 40,000 = 250,000.
        assert!(close(d.settlement_usd, 250_000.0));
    }

    #[test]
    fn pv_less_than_nominal_when_discounted() {
        let d = generate(&base());
        assert!(d.pv_remaining_rent_usd < d.nominal_remaining_rent_usd);
    }

    #[test]
    fn settlement_floored_at_zero() {
        // Huge reletting recovery would drive the settlement negative; floor at 0.
        let d = generate(&LeaseBuyoutInput { reletting_recovery_usd: 1_000_000.0, ..base() });
        assert!(close(d.settlement_usd, 0.0));
    }

    #[test]
    fn no_fee_no_concessions() {
        let d = generate(&LeaseBuyoutInput {
            termination_fee_months: 0.0,
            unamortized_concessions_usd: 0.0,
            reletting_recovery_usd: 0.0,
            ..base()
        });
        assert!(close(d.termination_fee_usd, 0.0));
        assert!(close(d.settlement_usd, d.pv_remaining_rent_usd));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&LeaseBuyoutInput { statute_citation: "lease § 21".into(), ..base() });
        assert_eq!(d.statutory_citation, "lease § 21");
        assert!(d.clauses.iter().any(|c| c.body.contains("lease § 21")));
    }
}
