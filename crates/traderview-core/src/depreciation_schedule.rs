//! Depreciation schedule — the period-by-period book-value table for a fixed asset
//! under straight-line, double-declining-balance, or sum-of-years-digits
//! depreciation. Straight-line spreads (cost − salvage) evenly over the life;
//! double-declining-balance applies twice the straight-line rate to the declining
//! book value, floored so the book value never falls below salvage;
//! sum-of-years-digits weights the depreciable base by remaining life — year k of
//! an N-year life takes (N−k+1)/[N(N+1)/2] of the base, an accelerated method
//! gentler than DDB; units-of-production charges each period its share of the base
//! equal to that period's units over total estimated units, so depreciation tracks
//! actual usage rather than time. Distinct from depreciation recapture, which
//! computes the tax clawback on sale; this is the book schedule. Drafting aid, not
//! accounting/tax advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct DepreciationScheduleInput {
    pub company_name: String,
    pub asset_label: String,
    /// Depreciable cost (capitalized basis).
    pub cost_usd: f64,
    /// Estimated salvage (residual) value at end of life.
    #[serde(default)]
    pub salvage_usd: f64,
    /// Useful life in years.
    pub life_years: u32,
    /// "straight_line", "ddb" (double-declining-balance), "syd"
    /// (sum-of-years-digits), or "uop" (units-of-production).
    #[serde(default)]
    pub method: String,
    /// Units-of-production only: total units expected over the asset's life.
    #[serde(default)]
    pub total_estimated_units: f64,
    /// Units-of-production only: units produced each period, one entry per year.
    #[serde(default)]
    pub units_per_period: Vec<f64>,
    /// Year the asset was placed in service (for labelling).
    #[serde(default)]
    pub start_year: i32,
    pub date: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ScheduleRow {
    pub year: u32,
    pub depreciation_usd: f64,
    pub accumulated_usd: f64,
    pub book_value_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DepreciationSchedule {
    pub title: String,
    pub method_label: String,
    pub depreciable_base_usd: f64,
    /// Total depreciation over the life (cost − salvage).
    pub total_depreciation_usd: f64,
    pub schedule: Vec<ScheduleRow>,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &DepreciationScheduleInput) -> DepreciationSchedule {
    let method = i.method.trim().to_ascii_lowercase();
    let is_ddb = method == "ddb";
    let is_syd = method == "syd";
    let is_uop = method == "uop";
    let base = (i.cost_usd - i.salvage_usd).max(0.0);

    let mut schedule = Vec::with_capacity(i.life_years as usize);
    let mut book = i.cost_usd;
    let mut accumulated = 0.0;

    if is_uop {
        // Usage-based: each period takes the share of the base equal to that
        // period's units over total estimated units, floored at salvage. Periods
        // come from the units list, not life_years.
        if i.total_estimated_units > 0.0 {
            for (idx, &units) in i.units_per_period.iter().enumerate() {
                let raw = base * units / i.total_estimated_units;
                let dep = cents(raw.min((book - i.salvage_usd).max(0.0)));
                book = cents(book - dep);
                accumulated = cents(accumulated + dep);
                schedule.push(ScheduleRow {
                    year: (idx + 1) as u32,
                    depreciation_usd: dep,
                    accumulated_usd: accumulated,
                    book_value_usd: book,
                });
            }
        }
    } else if i.life_years > 0 {
        let sl_annual = base / i.life_years as f64;
        let ddb_rate = 2.0 / i.life_years as f64;
        // Sum-of-years-digits denominator: N(N+1)/2.
        let syd_sum = (i.life_years as f64 * (i.life_years as f64 + 1.0)) / 2.0;
        for y in 1..=i.life_years {
            let dep = if is_ddb {
                // Twice the SL rate on the declining book value, but never depreciate
                // below salvage.
                (book * ddb_rate).min((book - i.salvage_usd).max(0.0))
            } else if is_syd {
                // Year k takes (N−k+1)/Σ of the base; last year absorbs rounding.
                if y == i.life_years {
                    (book - i.salvage_usd).max(0.0)
                } else {
                    let remaining = (i.life_years - y + 1) as f64;
                    base * remaining / syd_sum
                }
            } else {
                // Last year absorbs any rounding so book lands exactly on salvage.
                if y == i.life_years {
                    (book - i.salvage_usd).max(0.0)
                } else {
                    sl_annual
                }
            };
            let dep = cents(dep);
            book = cents(book - dep);
            accumulated = cents(accumulated + dep);
            schedule.push(ScheduleRow {
                year: y,
                depreciation_usd: dep,
                accumulated_usd: accumulated,
                book_value_usd: book,
            });
        }
    }

    let total = cents(accumulated);
    let method_label = if is_ddb {
        "Double-declining-balance"
    } else if is_syd {
        "Sum-of-years-digits"
    } else if is_uop {
        "Units-of-production"
    } else {
        "Straight-line"
    };

    let detail = if schedule.is_empty() {
        "No schedule (life is zero).".to_string()
    } else {
        schedule
            .iter()
            .map(|r| {
                let yr = if i.start_year > 0 {
                    format!("{}", i.start_year + r.year as i32 - 1)
                } else {
                    format!("Year {}", r.year)
                };
                format!(
                    "  • {}: depreciation {}, accumulated {}, book value {}",
                    yr, money(r.depreciation_usd), money(r.accumulated_usd), money(r.book_value_usd)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let mut clauses = vec![
        DocClause {
            heading: "Asset".into(),
            body: format!(
                "Company: {}\nAsset: {}\nCost: {}\nSalvage: {}\nLife: {} years\nMethod: {}\nDate: {}",
                i.company_name,
                i.asset_label,
                money(i.cost_usd),
                money(i.salvage_usd),
                i.life_years,
                method_label,
                i.date
            ),
        },
        DocClause {
            heading: "Depreciable Base".into(),
            body: format!(
                "The depreciable base is {} (cost {} less salvage {}), fully depreciated over {} years.",
                money(base), money(i.cost_usd), money(i.salvage_usd), i.life_years
            ),
        },
        DocClause { heading: "Schedule".into(), body: detail },
    ];

    let note = i.note.trim();
    if !note.is_empty() {
        clauses.push(DocClause { heading: "Note".into(), body: note.to_string() });
    }

    DepreciationSchedule {
        title: "Depreciation Schedule".into(),
        method_label: method_label.to_string(),
        depreciable_base_usd: cents(base),
        total_depreciation_usd: total,
        schedule,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> DepreciationScheduleInput {
        DepreciationScheduleInput {
            company_name: "Acme Co".into(),
            asset_label: "Delivery van".into(),
            cost_usd: 10_000.0,
            salvage_usd: 1_000.0,
            life_years: 5,
            method: "straight_line".into(),
            total_estimated_units: 0.0,
            units_per_period: Vec::new(),
            start_year: 2026,
            date: "2026-07-01".into(),
            note: String::new(),
        }
    }

    #[test]
    fn straight_line_schedule() {
        let d = generate(&base());
        assert_eq!(d.schedule.len(), 5);
        assert!(close(d.schedule[0].depreciation_usd, 1_800.0));
        assert!(close(d.schedule[0].book_value_usd, 8_200.0));
        assert!(close(d.schedule[4].book_value_usd, 1_000.0));
        assert!(close(d.total_depreciation_usd, 9_000.0));
    }

    #[test]
    fn ddb_schedule_tapers_to_salvage() {
        let d = generate(&DepreciationScheduleInput { method: "ddb".into(), ..base() });
        assert!(close(d.schedule[0].depreciation_usd, 4_000.0));
        assert!(close(d.schedule[1].depreciation_usd, 2_400.0));
        // Final year tapers so book lands on salvage.
        assert!(close(d.schedule[4].depreciation_usd, 296.0));
        assert!(close(d.schedule[4].book_value_usd, 1_000.0));
        assert!(close(d.total_depreciation_usd, 9_000.0));
    }

    #[test]
    fn syd_schedule_weights_by_remaining_life() {
        // base 9000, life 5 → Σ = 15: 5/15, 4/15, 3/15, 2/15, 1/15 of 9000.
        let d = generate(&DepreciationScheduleInput { method: "syd".into(), ..base() });
        assert_eq!(d.schedule.len(), 5);
        assert!(close(d.schedule[0].depreciation_usd, 3_000.0));
        assert!(close(d.schedule[1].depreciation_usd, 2_400.0));
        assert!(close(d.schedule[2].depreciation_usd, 1_800.0));
        assert!(close(d.schedule[3].depreciation_usd, 1_200.0));
        assert!(close(d.schedule[4].depreciation_usd, 600.0));
        assert!(close(d.schedule[4].book_value_usd, 1_000.0));
        assert!(close(d.total_depreciation_usd, 9_000.0));
        assert_eq!(d.method_label, "Sum-of-years-digits");
    }

    #[test]
    fn uop_charges_by_usage_share() {
        // base 9000, total 20000 units, usage [8000,6000,4000,2000] (=1.0 of total):
        // 0.4/0.3/0.2/0.1 of base → 3600/2700/1800/900.
        let d = generate(&DepreciationScheduleInput {
            method: "uop".into(),
            total_estimated_units: 20_000.0,
            units_per_period: vec![8_000.0, 6_000.0, 4_000.0, 2_000.0],
            ..base()
        });
        assert_eq!(d.schedule.len(), 4);
        assert!(close(d.schedule[0].depreciation_usd, 3_600.0));
        assert!(close(d.schedule[1].depreciation_usd, 2_700.0));
        assert!(close(d.schedule[2].depreciation_usd, 1_800.0));
        assert!(close(d.schedule[3].depreciation_usd, 900.0));
        assert!(close(d.schedule[3].book_value_usd, 1_000.0));
        assert!(close(d.total_depreciation_usd, 9_000.0));
        assert_eq!(d.method_label, "Units-of-production");
    }

    #[test]
    fn uop_floors_at_salvage_when_usage_exceeds_estimate() {
        // Over-usage beyond the estimate can't depreciate past salvage.
        let d = generate(&DepreciationScheduleInput {
            method: "uop".into(),
            total_estimated_units: 10_000.0,
            units_per_period: vec![6_000.0, 6_000.0, 6_000.0],
            ..base()
        });
        for r in &d.schedule {
            assert!(r.book_value_usd >= 1_000.0 - 0.01);
        }
        assert!(close(d.total_depreciation_usd, 9_000.0));
    }

    #[test]
    fn uop_empty_without_total_units() {
        let d = generate(&DepreciationScheduleInput { method: "uop".into(), ..base() });
        assert!(d.schedule.is_empty());
    }

    #[test]
    fn never_below_salvage() {
        let d = generate(&DepreciationScheduleInput { method: "ddb".into(), ..base() });
        for r in &d.schedule {
            assert!(r.book_value_usd >= 1_000.0 - 0.01);
        }
    }

    #[test]
    fn depreciable_base_excludes_salvage() {
        let d = generate(&base());
        assert!(close(d.depreciable_base_usd, 9_000.0));
    }

    #[test]
    fn zero_salvage_full_cost_depreciated() {
        let d = generate(&DepreciationScheduleInput { salvage_usd: 0.0, ..base() });
        assert!(close(d.total_depreciation_usd, 10_000.0));
        assert!(close(d.schedule[4].book_value_usd, 0.0));
    }

    #[test]
    fn note_appended_when_present() {
        let d = generate(&DepreciationScheduleInput { note: "Fleet asset #12.".into(), ..base() });
        assert!(d.clauses.iter().any(|c| c.heading == "Note" && c.body.contains("Fleet")));
    }
}
