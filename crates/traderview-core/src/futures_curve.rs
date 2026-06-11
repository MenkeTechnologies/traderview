//! Futures term structure — contract-month symbol generation and
//! curve analytics.
//!
//! Symbols follow the Yahoo convention `{root}{month_code}{yy}.{exch}`
//! (CLQ26.NYM = Crude Aug-2026 on NYMEX), verified live against NYM /
//! CMX / CME / CBT. Month codes: F G H J K M N Q U V X Z.
//!
//! Curve analytics over (months_out, price) points:
//!   leg roll (annualized) = (p_next / p_prev)^(12/Δm) − 1
//! positive = contango (long rolls bleed), negative = backwardation
//! (long rolls earn). The roll yield IS the carry trade.
//!
//! Pure compute; the data wrapper quotes each month through the cached
//! quote path.

use serde::Serialize;

pub const MONTH_CODES: [char; 12] = ['F', 'G', 'H', 'J', 'K', 'M', 'N', 'Q', 'U', 'V', 'X', 'Z'];

/// Next `n` contract symbols from (year, month) inclusive.
/// Returns (symbol, "MMM-YY" label, months_out starting at 0).
pub fn next_contract_symbols(
    root: &str,
    exchange: &str,
    from_year: i32,
    from_month: u32,
    n: usize,
) -> Vec<(String, String, u32)> {
    const NAMES: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    if !(1..=12).contains(&from_month) || n == 0 || n > 36 {
        return Vec::new();
    }
    (0..n)
        .map(|k| {
            let total = from_year as u32 * 12 + (from_month - 1) + k as u32;
            let (y, m) = (total / 12, total % 12);
            (
                format!("{root}{}{:02}.{exchange}", MONTH_CODES[m as usize], y % 100),
                format!("{}-{:02}", NAMES[m as usize], y % 100),
                k as u32,
            )
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct CurveLegRow {
    pub label: String,
    pub months_out: u32,
    pub price: f64,
    /// Spread to the PREVIOUS listed month, %.
    pub spread_pct: Option<f64>,
    /// Annualized roll between this and the previous month, %.
    pub roll_annualized_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurveReport {
    pub rows: Vec<CurveLegRow>,
    /// "contango", "backwardation", or "mixed".
    pub shape: &'static str,
    /// Front-to-back annualized roll, %.
    pub overall_roll_annualized_pct: f64,
}

/// `points` = (label, months_out, price), ascending months_out with
/// at least two entries.
pub fn analyze_curve(points: &[(String, u32, f64)]) -> Option<CurveReport> {
    if points.len() < 2
        || points.iter().any(|(_, _, p)| !p.is_finite() || *p <= 0.0)
        || points.windows(2).any(|w| w[1].1 <= w[0].1)
    {
        return None;
    }
    let mut rows: Vec<CurveLegRow> = Vec::with_capacity(points.len());
    let mut ups = 0usize;
    let mut downs = 0usize;
    for (i, (label, m, p)) in points.iter().enumerate() {
        let (spread, roll) = if i == 0 {
            (None, None)
        } else {
            let (_, pm, pp) = &points[i - 1];
            let dm = (*m - *pm) as f64;
            let ratio = *p / *pp;
            if ratio > 1.0 {
                ups += 1;
            } else if ratio < 1.0 {
                downs += 1;
            }
            (
                Some((ratio - 1.0) * 100.0),
                Some((ratio.powf(12.0 / dm) - 1.0) * 100.0),
            )
        };
        rows.push(CurveLegRow {
            label: label.clone(),
            months_out: *m,
            price: *p,
            spread_pct: spread,
            roll_annualized_pct: roll,
        });
    }
    let shape = if downs == 0 && ups > 0 {
        "contango"
    } else if ups == 0 && downs > 0 {
        "backwardation"
    } else {
        "mixed"
    };
    let first = &points[0];
    let last = &points[points.len() - 1];
    let dm = (last.1 - first.1) as f64;
    let overall = ((last.2 / first.2).powf(12.0 / dm) - 1.0) * 100.0;
    Some(CurveReport {
        rows,
        shape,
        overall_roll_annualized_pct: overall,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbols_match_the_live_verified_convention() {
        // From June 2026: CLM26.NYM, CLN26 (Jul), CLQ26 (Aug, the
        // live-checked one), CLU26 (Sep).
        let s = next_contract_symbols("CL", "NYM", 2026, 6, 4);
        let syms: Vec<&str> = s.iter().map(|(x, _, _)| x.as_str()).collect();
        assert_eq!(syms, ["CLM26.NYM", "CLN26.NYM", "CLQ26.NYM", "CLU26.NYM"]);
        assert_eq!(s[2].1, "Aug-26");
    }

    #[test]
    fn year_rollover_dec_to_jan() {
        let s = next_contract_symbols("GC", "CMX", 2026, 11, 4);
        let syms: Vec<&str> = s.iter().map(|(x, _, _)| x.as_str()).collect();
        assert_eq!(syms, ["GCX26.CMX", "GCZ26.CMX", "GCF27.CMX", "GCG27.CMX"]);
    }

    #[test]
    fn backwardated_curve_hand_walk() {
        // 100 → 99 → 98 at one-month gaps: each leg rolls at
        // (0.99)^12 − 1 ≈ −11.36%; shape backwardation; overall
        // (98/100)^(12/2) − 1 ≈ −11.42%.
        let pts = vec![
            ("M1".to_string(), 0, 100.0),
            ("M2".to_string(), 1, 99.0),
            ("M3".to_string(), 2, 98.0),
        ];
        let r = analyze_curve(&pts).unwrap();
        assert_eq!(r.shape, "backwardation");
        let leg = r.rows[1].roll_annualized_pct.unwrap();
        assert!((leg - (0.99_f64.powi(12) - 1.0) * 100.0).abs() < 1e-9);
        let want_overall = ((98.0_f64 / 100.0).powf(6.0) - 1.0) * 100.0;
        assert!((r.overall_roll_annualized_pct - want_overall).abs() < 1e-9);
        assert_eq!(r.rows[0].spread_pct, None);
        assert!((r.rows[1].spread_pct.unwrap() + 1.0).abs() < 1e-12);
    }

    #[test]
    fn contango_and_mixed_shapes() {
        let up = vec![
            ("a".into(), 0, 100.0),
            ("b".into(), 1, 101.0),
            ("c".into(), 2, 102.0),
        ];
        assert_eq!(analyze_curve(&up).unwrap().shape, "contango");
        let mixed = vec![
            ("a".into(), 0, 100.0),
            ("b".into(), 1, 101.0),
            ("c".into(), 2, 100.5),
        ];
        assert_eq!(analyze_curve(&mixed).unwrap().shape, "mixed");
    }

    #[test]
    fn gaps_in_months_use_actual_spacing() {
        // 100 → 98 over a 2-month gap: annualized (0.98)^6 − 1.
        let pts = vec![("a".into(), 0, 100.0), ("b".into(), 2, 98.0)];
        let r = analyze_curve(&pts).unwrap();
        let want = (0.98_f64.powf(6.0) - 1.0) * 100.0;
        assert!((r.rows[1].roll_annualized_pct.unwrap() - want).abs() < 1e-9);
    }

    #[test]
    fn hostile_inputs_return_none_or_empty() {
        assert!(analyze_curve(&[("a".into(), 0, 100.0)]).is_none());
        assert!(analyze_curve(&[("a".into(), 0, 100.0), ("b".into(), 0, 99.0)]).is_none()); // non-ascending
        assert!(analyze_curve(&[("a".into(), 0, 0.0), ("b".into(), 1, 99.0)]).is_none());
        assert!(next_contract_symbols("CL", "NYM", 2026, 13, 4).is_empty());
        assert!(next_contract_symbols("CL", "NYM", 2026, 6, 0).is_empty());
    }
}
