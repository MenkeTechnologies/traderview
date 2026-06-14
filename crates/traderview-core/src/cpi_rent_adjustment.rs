//! CPI rent adjustment notice — the index-based escalation used in commercial
//! leases. Rent is reset by the ratio of a current consumer price index to a
//! base-period index, bounded by an optional collar (a floor and/or ceiling on
//! the percentage increase). This differs from the fixed-percentage compounding
//! in `rent_escalation`: the increase is driven by published CPI, not a constant
//! rate, and the collar clamps it. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CpiRentInput {
    pub landlord_name: String,
    pub tenant_name: String,
    #[serde(default)]
    pub property_label: String,
    /// Current annual rent before adjustment.
    pub base_rent_usd: f64,
    /// CPI level for the base period (lease commencement or last adjustment).
    pub cpi_base: f64,
    /// CPI level for the current period.
    pub cpi_current: f64,
    /// Floor on the percentage increase (0 = none). Rent rises by at least this.
    #[serde(default)]
    pub min_increase_pct: f64,
    /// Ceiling on the percentage increase (0 = uncapped).
    #[serde(default)]
    pub max_increase_pct: f64,
    /// Index name, for the notice (e.g. "CPI-U, U.S. city average").
    #[serde(default)]
    pub index_label: String,
    pub effective_date: String,
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
pub struct CpiRentAdjustment {
    pub title: String,
    /// cpi_current ÷ cpi_base.
    pub index_ratio: f64,
    /// Raw percentage change implied by the index ratio.
    pub raw_increase_pct: f64,
    /// Percentage increase after applying the collar.
    pub applied_increase_pct: f64,
    /// True when the ceiling reduced the increase.
    pub ceiling_applied: bool,
    /// True when the floor raised the increase.
    pub floor_applied: bool,
    pub new_rent_usd: f64,
    pub dollar_increase_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &CpiRentInput) -> CpiRentAdjustment {
    let ratio = if i.cpi_base > 0.0 {
        i.cpi_current / i.cpi_base
    } else {
        1.0
    };
    let raw_pct = (ratio - 1.0) * 100.0;

    // Apply the ceiling first, then the floor, so a collar like 2%–5% pins the
    // increase into [min, max] regardless of which bound binds.
    let mut applied = raw_pct;
    let mut ceiling_applied = false;
    let mut floor_applied = false;
    if i.max_increase_pct > 0.0 && applied > i.max_increase_pct {
        applied = i.max_increase_pct;
        ceiling_applied = true;
    }
    if applied < i.min_increase_pct {
        applied = i.min_increase_pct;
        floor_applied = true;
    }

    let new_rent = cents(i.base_rent_usd * (1.0 + applied / 100.0));
    let increase = cents(new_rent - i.base_rent_usd);

    let property = if i.property_label.trim().is_empty() {
        "the Premises".to_string()
    } else {
        i.property_label.trim().to_string()
    };
    let index = if i.index_label.trim().is_empty() {
        "the Consumer Price Index".to_string()
    } else {
        i.index_label.trim().to_string()
    };

    let collar_desc = match (i.min_increase_pct > 0.0, i.max_increase_pct > 0.0) {
        (true, true) => format!(" The increase is collared between {:.2}% and {:.2}%.", i.min_increase_pct, i.max_increase_pct),
        (true, false) => format!(" The increase is floored at {:.2}%.", i.min_increase_pct),
        (false, true) => format!(" The increase is capped at {:.2}%.", i.max_increase_pct),
        (false, false) => String::new(),
    };

    let bound_note = if ceiling_applied {
        " (reduced to the ceiling)"
    } else if floor_applied {
        " (raised to the floor)"
    } else {
        ""
    };

    let calc_body = format!(
        "The index ratio is {} ÷ {} = {:.4}, an implied change of {:.4}%.{} The applied increase is {:.4}%{}. Base rent of {} becomes {}, an increase of {}.",
        i.cpi_current,
        i.cpi_base,
        round4(ratio),
        round4(raw_pct),
        collar_desc,
        round4(applied),
        bound_note,
        money(i.base_rent_usd),
        money(new_rent),
        money(increase)
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("This adjustment is made under the lease and the laws of the State of {}.", i.state)
    } else {
        format!("This adjustment is made under the lease and the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Landlord: {}\nTenant: {}\nPremises: {}\nEffective date: {}",
                i.landlord_name, i.tenant_name, property, i.effective_date
            ),
        },
        DocClause {
            heading: "1. Index Adjustment".into(),
            body: format!(
                "Rent is adjusted by reference to {}. The base-period index is {} and the current index is {}.{}",
                index, i.cpi_base, i.cpi_current, collar_desc
            ),
        },
        DocClause { heading: "2. Calculation".into(), body: calc_body },
        DocClause {
            heading: "3. New Rent".into(),
            body: format!(
                "Effective {}, the annual rent is {}, payable in equal monthly installments. The adjustment does not waive any other lease term.",
                i.effective_date, money(new_rent)
            ),
        },
        DocClause { heading: "4. Governing Law".into(), body: governing },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Landlord: ____________________  Date: __________\n{}",
                i.landlord_name
            ),
        },
    ];

    CpiRentAdjustment {
        title: "CPI Rent Adjustment Notice".into(),
        index_ratio: round4(ratio),
        raw_increase_pct: round4(raw_pct),
        applied_increase_pct: round4(applied),
        ceiling_applied,
        floor_applied,
        new_rent_usd: new_rent,
        dollar_increase_usd: increase,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> CpiRentInput {
        CpiRentInput {
            landlord_name: "Tower Owners LP".into(),
            tenant_name: "Office Tenant LLC".into(),
            property_label: "Floor 7".into(),
            base_rent_usd: 50_000.0,
            cpi_base: 280.0,
            cpi_current: 295.4,
            min_increase_pct: 2.0,
            max_increase_pct: 5.0,
            index_label: "CPI-U".into(),
            effective_date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn ceiling_caps_large_increase() {
        let d = generate(&base());
        assert!(close(d.index_ratio, 1.055));
        assert!(close(d.raw_increase_pct, 5.5));
        assert!(close(d.applied_increase_pct, 5.0));
        assert!(d.ceiling_applied);
        assert!(!d.floor_applied);
        assert!(close(d.new_rent_usd, 52_500.0));
        assert!(close(d.dollar_increase_usd, 2_500.0));
    }

    #[test]
    fn no_collar_uses_raw_change() {
        let d = generate(&CpiRentInput {
            cpi_current: 287.0,
            min_increase_pct: 0.0,
            max_increase_pct: 0.0,
            ..base()
        });
        assert!(close(d.raw_increase_pct, 2.5));
        assert!(close(d.applied_increase_pct, 2.5));
        assert!(close(d.new_rent_usd, 51_250.0));
    }

    #[test]
    fn floor_lifts_when_cpi_falls() {
        let d = generate(&CpiRentInput { cpi_current: 275.0, ..base() });
        // Raw is negative; floor of 2% applies.
        assert!(d.raw_increase_pct < 0.0);
        assert!(close(d.applied_increase_pct, 2.0));
        assert!(d.floor_applied);
        assert!(!d.ceiling_applied);
        assert!(close(d.new_rent_usd, 51_000.0));
    }

    #[test]
    fn within_collar_unbounded() {
        let d = generate(&CpiRentInput { cpi_current: 290.0, ..base() });
        // Ratio ~1.0357 → ~3.57%, inside 2–5%.
        assert!(!d.ceiling_applied);
        assert!(!d.floor_applied);
        assert!(close(d.applied_increase_pct, d.raw_increase_pct));
    }

    #[test]
    fn zero_base_index_no_change() {
        let d = generate(&CpiRentInput { cpi_base: 0.0, min_increase_pct: 0.0, max_increase_pct: 0.0, ..base() });
        assert!(close(d.index_ratio, 1.0));
        assert!(close(d.new_rent_usd, 50_000.0));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&CpiRentInput { statute_citation: "lease § 5.1".into(), ..base() });
        assert_eq!(d.statutory_citation, "lease § 5.1");
        assert!(d.clauses.iter().any(|c| c.body.contains("lease § 5.1")));
    }
}
