//! Standard vs itemized deduction — whether a taxpayer is better off taking the
//! standard deduction or itemizing on Schedule A.
//!
//! Itemized total = SALT (state/local income or sales tax + property tax, capped)
//! + home mortgage interest + charitable contributions + medical expenses above
//! the 7.5%-of-AGI floor. The taxpayer takes the larger of that and the standard
//! deduction.
//!
//! 2026 defaults (web-verified, OBBBA, overridable): standard deduction $16,100
//! single / $32,200 MFJ; SALT cap $40,400, reduced 30% of MAGI over $505,000 and
//! floored at $10,000 (the cap reverts to $10,000 in 2030); medical floor 7.5%.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    MarriedJoint,
}

fn d_salt_cap() -> f64 {
    40_400.0
}
fn d_salt_threshold() -> f64 {
    505_000.0
}
fn d_salt_floor() -> f64 {
    10_000.0
}
fn d_medical_floor_pct() -> f64 {
    7.5
}

#[derive(Debug, Clone, Deserialize)]
pub struct StdVsItemizedInput {
    /// Adjusted gross income (used as MAGI for the SALT phase-out and the
    /// medical floor).
    pub agi_usd: f64,
    pub filing_status: FilingStatus,
    /// State/local income or sales tax paid.
    #[serde(default)]
    pub state_local_tax_usd: f64,
    #[serde(default)]
    pub property_tax_usd: f64,
    #[serde(default)]
    pub mortgage_interest_usd: f64,
    #[serde(default)]
    pub charitable_usd: f64,
    #[serde(default)]
    pub medical_usd: f64,
    /// Base SALT cap before the high-income phase-out.
    #[serde(default = "d_salt_cap")]
    pub salt_cap_base_usd: f64,
    /// MAGI where the SALT cap begins to phase down.
    #[serde(default = "d_salt_threshold")]
    pub salt_phaseout_threshold_usd: f64,
    /// SALT cap floor the phase-out cannot drop below.
    #[serde(default = "d_salt_floor")]
    pub salt_floor_usd: f64,
    /// Medical expense floor, percent of AGI.
    #[serde(default = "d_medical_floor_pct")]
    pub medical_floor_pct: f64,
    /// Standard deduction. None → filing-status default.
    #[serde(default)]
    pub std_deduction_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StdVsItemizedResult {
    pub standard_deduction_usd: f64,
    pub salt_paid_usd: f64,
    /// SALT cap after the high-income phase-out.
    pub salt_cap_usd: f64,
    pub salt_deductible_usd: f64,
    /// SALT paid above the cap (lost).
    pub salt_lost_usd: f64,
    pub medical_floor_usd: f64,
    pub medical_deductible_usd: f64,
    pub itemized_total_usd: f64,
    pub deduction_taken_usd: f64,
    /// Itemized − standard. Positive means itemizing wins.
    pub itemizing_advantage_usd: f64,
    pub should_itemize: bool,
}

pub fn analyze(input: &StdVsItemizedInput) -> StdVsItemizedResult {
    let standard = input.std_deduction_usd.unwrap_or(match input.filing_status {
        FilingStatus::Single => 16_100.0,
        FilingStatus::MarriedJoint => 32_200.0,
    });

    let salt_paid = input.state_local_tax_usd + input.property_tax_usd;
    let over = (input.agi_usd - input.salt_phaseout_threshold_usd).max(0.0);
    let salt_cap = (input.salt_cap_base_usd - 0.30 * over).max(input.salt_floor_usd);
    let salt_deductible = salt_paid.min(salt_cap);
    let salt_lost = (salt_paid - salt_deductible).max(0.0);

    let medical_floor = input.medical_floor_pct / 100.0 * input.agi_usd;
    let medical_deductible = (input.medical_usd - medical_floor).max(0.0);

    let itemized = salt_deductible
        + input.mortgage_interest_usd
        + input.charitable_usd
        + medical_deductible;

    let should_itemize = itemized > standard;

    StdVsItemizedResult {
        standard_deduction_usd: standard,
        salt_paid_usd: salt_paid,
        salt_cap_usd: salt_cap,
        salt_deductible_usd: salt_deductible,
        salt_lost_usd: salt_lost,
        medical_floor_usd: medical_floor,
        medical_deductible_usd: medical_deductible,
        itemized_total_usd: itemized,
        deduction_taken_usd: itemized.max(standard),
        itemizing_advantage_usd: itemized - standard,
        should_itemize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn base() -> StdVsItemizedInput {
        StdVsItemizedInput {
            agi_usd: 100_000.0,
            filing_status: FilingStatus::Single,
            state_local_tax_usd: 0.0,
            property_tax_usd: 0.0,
            mortgage_interest_usd: 0.0,
            charitable_usd: 0.0,
            medical_usd: 0.0,
            salt_cap_base_usd: 40_400.0,
            salt_phaseout_threshold_usd: 505_000.0,
            salt_floor_usd: 10_000.0,
            medical_floor_pct: 7.5,
            std_deduction_usd: None,
        }
    }

    #[test]
    fn standard_wins_when_itemized_low() {
        let r = analyze(&StdVsItemizedInput {
            state_local_tax_usd: 5_000.0,
            mortgage_interest_usd: 3_000.0,
            charitable_usd: 1_000.0,
            ..base()
        });
        // itemized 9,000 < standard 16,100.
        assert!(close(r.itemized_total_usd, 9_000.0));
        assert!(close(r.standard_deduction_usd, 16_100.0));
        assert!(close(r.deduction_taken_usd, 16_100.0));
        assert!(!r.should_itemize);
        assert!(close(r.itemizing_advantage_usd, -7_100.0));
    }

    #[test]
    fn itemize_wins_for_mfj() {
        let r = analyze(&StdVsItemizedInput {
            agi_usd: 200_000.0,
            filing_status: FilingStatus::MarriedJoint,
            state_local_tax_usd: 30_000.0,
            mortgage_interest_usd: 18_000.0,
            charitable_usd: 5_000.0,
            ..base()
        });
        // itemized 53,000 > standard 32,200.
        assert!(close(r.itemized_total_usd, 53_000.0));
        assert!(close(r.itemizing_advantage_usd, 20_800.0));
        assert!(r.should_itemize);
    }

    #[test]
    fn salt_cap_binds() {
        let r = analyze(&StdVsItemizedInput {
            agi_usd: 300_000.0,
            filing_status: FilingStatus::MarriedJoint,
            state_local_tax_usd: 25_000.0,
            property_tax_usd: 20_000.0,
            ..base()
        });
        // paid 45,000; cap 40,400 (no phase-out); lost 4,600.
        assert!(close(r.salt_paid_usd, 45_000.0));
        assert!(close(r.salt_cap_usd, 40_400.0));
        assert!(close(r.salt_deductible_usd, 40_400.0));
        assert!(close(r.salt_lost_usd, 4_600.0));
    }

    #[test]
    fn salt_phaseout_partial() {
        let r = analyze(&StdVsItemizedInput {
            agi_usd: 605_000.0,
            filing_status: FilingStatus::MarriedJoint,
            state_local_tax_usd: 50_000.0,
            ..base()
        });
        // 40,400 − 0.30 × (605,000 − 505,000) = 40,400 − 30,000 = 10,400.
        assert!(close(r.salt_cap_usd, 10_400.0));
        assert!(close(r.salt_deductible_usd, 10_400.0));
    }

    #[test]
    fn salt_phaseout_hits_floor() {
        let r = analyze(&StdVsItemizedInput {
            agi_usd: 1_000_000.0,
            filing_status: FilingStatus::MarriedJoint,
            state_local_tax_usd: 50_000.0,
            ..base()
        });
        // 40,400 − 0.30 × 495,000 is negative → floored at 10,000.
        assert!(close(r.salt_cap_usd, 10_000.0));
        assert!(close(r.salt_deductible_usd, 10_000.0));
    }

    #[test]
    fn medical_above_floor() {
        let r = analyze(&StdVsItemizedInput {
            agi_usd: 80_000.0,
            medical_usd: 10_000.0,
            ..base()
        });
        // floor 7.5% × 80,000 = 6,000; deductible 4,000.
        assert!(close(r.medical_floor_usd, 6_000.0));
        assert!(close(r.medical_deductible_usd, 4_000.0));
    }

    #[test]
    fn medical_below_floor_zero() {
        let r = analyze(&StdVsItemizedInput {
            agi_usd: 100_000.0,
            medical_usd: 5_000.0,
            ..base()
        });
        // floor 7,500 > 5,000 → no deduction.
        assert!(close(r.medical_deductible_usd, 0.0));
    }

    #[test]
    fn standard_deduction_override() {
        let r = analyze(&StdVsItemizedInput {
            charitable_usd: 5_000.0,
            std_deduction_usd: Some(30_000.0),
            ..base()
        });
        assert!(close(r.standard_deduction_usd, 30_000.0));
        assert!(close(r.deduction_taken_usd, 30_000.0));
        assert!(!r.should_itemize);
    }
}
