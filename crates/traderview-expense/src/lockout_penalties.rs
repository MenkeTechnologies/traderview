//! State-by-state civil penalties for **self-help eviction** — landlord
//! removes a tenant by lockout, utility shutoff, or removal of personal
//! property without a court order. Universal landlord exposure: every
//! state prohibits self-help, but the dollar consequences vary 10× across
//! jurisdictions.
//!
//! Seven distinct penalty regimes are present in the table:
//!
//! 1. **Additive per-day** (CA § 789.3): `$100/day` capped above by no
//!    ceiling, with a `$250` statutory minimum, PLUS actual damages PLUS
//!    attorney's fees. The per-day floor catches short-duration violations.
//! 2. **Additive rent + flat** (TX § 92.0081): `1 month's rent` PLUS
//!    `$1,000` PLUS actual damages PLUS fees.
//! 3. **Greater-of rent-multiple or actual** (FL § 83.67, WA § 59.18.290,
//!    MA c. 186 § 14): tenant recovers `max(3× monthly rent, actual)` plus
//!    fees. Tenant picks whichever wins; not additive.
//! 4. **Greater-of rent-multiple or actual-multiple** (AZ § 33-1367):
//!    tenant recovers `max(2× monthly rent, 2× actual)`. Both sides
//!    multiplied; not additive.
//! 5. **Greater-of rent-multiple or flat, plus actual** (CO § 38-12-510):
//!    `max(3× monthly rent, $5,000)` PLUS actual damages PLUS fees.
//!    Strictest state in the table; deliberate 2021 HB21-1121 reform.
//! 6. **Treble actual** (NY RPL § 235, DC § 42-3505.01, NJ § 2A:39-1):
//!    `3× actual damages` plus fees plus, in NY, criminal misdemeanor
//!    exposure. The treble model converts even small actual losses into
//!    significant suits.
//! 7. **Rent multiple + actual** (IL Forcible Entry Act): `2× rent` PLUS
//!    actual damages PLUS fees.
//!
//! States not on this list default to `ActualDamagesOnly` — self-help is
//! still prohibited (URLTA § 4.207 or state equivalent), but no per-state
//! penalty formula is hardcoded. Tenant recovers actual damages only and
//! the landlord may face separate criminal trespass / misdemeanor exposure
//! that this module does not compute.
//!
//! All money is **integer cents** to avoid `Decimal` precision drift on
//! the multiplier math (CO's `300 × rent / 100` floor-divide is exact).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PenaltyRegime {
    /// `max(per_day_cents × days, minimum_cents) + actual_damages_cents`.
    AdditivePerDay {
        per_day_cents: i64,
        minimum_cents: i64,
    },
    /// `(rent_mult × monthly_rent / 100) + flat_cents + actual_damages_cents`.
    AdditiveRentPlusFlat { rent_mult: u32, flat_cents: i64 },
    /// `max(rent_mult × monthly_rent / 100, actual_damages_cents)`.
    GreaterOfRentMultOrActual { rent_mult: u32 },
    /// `max(rent_mult × monthly_rent / 100, actual_mult × actual_damages / 100)`.
    GreaterOfRentMultOrActualMult { rent_mult: u32, actual_mult: u32 },
    /// `max(rent_mult × monthly_rent / 100, flat_cents) + actual_damages_cents`.
    GreaterOfRentMultOrFlatPlusActual { rent_mult: u32, flat_cents: i64 },
    /// `3 × actual_damages_cents`.
    TrebleActual,
    /// `(rent_mult × monthly_rent / 100) + actual_damages_cents`.
    RentMultiplePlusActual { rent_mult: u32 },
    /// Self-help prohibited but no per-state formula — tenant recovers
    /// only actual damages.
    ActualDamagesOnly,
    /// No statute on self-help eviction at all.
    NoStatute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateLockoutRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub regime: PenaltyRegime,
    pub attorney_fees_recoverable: bool,
    pub criminal_exposure: bool,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockoutPenaltyInput {
    pub state_code: String,
    pub days_locked_out: u32,
    pub monthly_rent_cents: i64,
    pub actual_damages_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockoutPenaltyResult {
    /// Total civil exposure (statutory + actual where additive; max where
    /// greater-of). Excludes attorney's fees because those are case-
    /// dependent.
    pub total_civil_exposure_cents: i64,
    /// Statutory-only component, computed per the regime's formula
    /// (excludes actual damages for additive/treble regimes, equals the
    /// formula amount for greater-of regimes).
    pub statutory_penalty_cents: i64,
    pub actual_damages_cents: i64,
    pub attorney_fees_recoverable: bool,
    pub criminal_exposure: bool,
    pub no_statute: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateLockoutRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateLockoutRule> {
    let mut v: Vec<&'static StateLockoutRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &LockoutPenaltyInput) -> LockoutPenaltyResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return LockoutPenaltyResult {
                total_civil_exposure_cents: 0,
                statutory_penalty_cents: 0,
                actual_damages_cents: 0,
                attorney_fees_recoverable: false,
                criminal_exposure: false,
                no_statute: true,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    let actual = input.actual_damages_cents;
    let rent = input.monthly_rent_cents;
    let days = input.days_locked_out as i64;

    let (statutory, total, note) = match rule.regime {
        PenaltyRegime::AdditivePerDay {
            per_day_cents,
            minimum_cents,
        } => {
            let raw = per_day_cents.saturating_mul(days);
            let stat = raw.max(minimum_cents);
            let total = stat.saturating_add(actual);
            let note = format!(
                "{} additive per-day: max({}¢/day × {} days = {}¢, minimum {}¢) = {}¢ statutory + {}¢ actual = {}¢ total",
                rule.state_name,
                per_day_cents,
                days,
                raw,
                minimum_cents,
                stat,
                actual,
                total
            );
            (stat, total, note)
        }
        PenaltyRegime::AdditiveRentPlusFlat {
            rent_mult,
            flat_cents,
        } => {
            let rent_part = rent.saturating_mul(rent_mult as i64) / 100;
            let stat = rent_part.saturating_add(flat_cents);
            let total = stat.saturating_add(actual);
            let note = format!(
                "{} additive: {}.{:02}× rent ({}¢) + flat {}¢ = {}¢ statutory + {}¢ actual = {}¢ total",
                rule.state_name,
                rent_mult / 100,
                rent_mult % 100,
                rent_part,
                flat_cents,
                stat,
                actual,
                total
            );
            (stat, total, note)
        }
        PenaltyRegime::GreaterOfRentMultOrActual { rent_mult } => {
            let rent_part = rent.saturating_mul(rent_mult as i64) / 100;
            let total = rent_part.max(actual);
            let note = format!(
                "{} greater-of: max({}.{:02}× rent = {}¢, actual = {}¢) = {}¢",
                rule.state_name,
                rent_mult / 100,
                rent_mult % 100,
                rent_part,
                actual,
                total
            );
            (rent_part, total, note)
        }
        PenaltyRegime::GreaterOfRentMultOrActualMult {
            rent_mult,
            actual_mult,
        } => {
            let rent_part = rent.saturating_mul(rent_mult as i64) / 100;
            let actual_part = actual.saturating_mul(actual_mult as i64) / 100;
            let total = rent_part.max(actual_part);
            let note = format!(
                "{} greater-of multiplied: max({}.{:02}× rent = {}¢, {}.{:02}× actual = {}¢) = {}¢",
                rule.state_name,
                rent_mult / 100,
                rent_mult % 100,
                rent_part,
                actual_mult / 100,
                actual_mult % 100,
                actual_part,
                total
            );
            (rent_part, total, note)
        }
        PenaltyRegime::GreaterOfRentMultOrFlatPlusActual {
            rent_mult,
            flat_cents,
        } => {
            let rent_part = rent.saturating_mul(rent_mult as i64) / 100;
            let stat = rent_part.max(flat_cents);
            let total = stat.saturating_add(actual);
            let note = format!(
                "{} greater-of-plus: max({}.{:02}× rent = {}¢, flat {}¢) = {}¢ statutory + {}¢ actual = {}¢ total",
                rule.state_name,
                rent_mult / 100,
                rent_mult % 100,
                rent_part,
                flat_cents,
                stat,
                actual,
                total
            );
            (stat, total, note)
        }
        PenaltyRegime::TrebleActual => {
            let total = actual.saturating_mul(3);
            // Statutory bonus = 2× actual (the additional treble portion
            // beyond the actual damages the tenant already has).
            let stat = actual.saturating_mul(2);
            let note = format!(
                "{} treble: 3× actual ({}¢) = {}¢ (statutory bonus = 2× = {}¢)",
                rule.state_name, actual, total, stat
            );
            (stat, total, note)
        }
        PenaltyRegime::RentMultiplePlusActual { rent_mult } => {
            let rent_part = rent.saturating_mul(rent_mult as i64) / 100;
            let total = rent_part.saturating_add(actual);
            let note = format!(
                "{} additive rent-multiple: {}.{:02}× rent ({}¢) + actual ({}¢) = {}¢",
                rule.state_name,
                rent_mult / 100,
                rent_mult % 100,
                rent_part,
                actual,
                total
            );
            (rent_part, total, note)
        }
        PenaltyRegime::ActualDamagesOnly => {
            let note = format!(
                "{}: self-help prohibited but no statutory penalty formula — tenant recovers actual damages only ({}¢) + possible criminal exposure",
                rule.state_name, actual
            );
            (0, actual, note)
        }
        PenaltyRegime::NoStatute => {
            let note = format!(
                "{}: no statute on self-help eviction — common-law trespass / actual damages only ({}¢)",
                rule.state_name, actual
            );
            (0, actual, note)
        }
    };

    LockoutPenaltyResult {
        total_civil_exposure_cents: total,
        statutory_penalty_cents: statutory,
        actual_damages_cents: actual,
        attorney_fees_recoverable: rule.attorney_fees_recoverable,
        criminal_exposure: rule.criminal_exposure,
        no_statute: matches!(rule.regime, PenaltyRegime::NoStatute),
        citation: rule.citation,
        note,
    }
}

const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    regime: PenaltyRegime,
    attorney_fees_recoverable: bool,
    criminal_exposure: bool,
    citation: &'static str,
) -> StateLockoutRule {
    StateLockoutRule {
        state_code,
        state_name,
        regime,
        attorney_fees_recoverable,
        criminal_exposure,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateLockoutRule>> = Lazy::new(|| {
    use PenaltyRegime::*;
    static RULES: &[StateLockoutRule] = &[
        rule("AK", "Alaska", ActualDamagesOnly, true, false, "AS § 34.03.210"),
        rule("AL", "Alabama", ActualDamagesOnly, true, false, "Ala. Code § 35-9A-407"),
        rule("AR", "Arkansas", NoStatute, false, false, "no statute (landlord-friendly)"),
        rule(
            "AZ",
            "Arizona",
            GreaterOfRentMultOrActualMult {
                rent_mult: 200,
                actual_mult: 200,
            },
            true,
            false,
            "A.R.S. § 33-1367",
        ),
        rule(
            "CA",
            "California",
            AdditivePerDay {
                per_day_cents: 10_000,
                minimum_cents: 25_000,
            },
            true,
            false,
            "Cal. Civ. Code § 789.3",
        ),
        rule(
            "CO",
            "Colorado",
            GreaterOfRentMultOrFlatPlusActual {
                rent_mult: 300,
                flat_cents: 500_000,
            },
            true,
            false,
            "C.R.S. § 38-12-510",
        ),
        rule("CT", "Connecticut", ActualDamagesOnly, true, true, "Conn. Gen. Stat. § 47a-43"),
        rule(
            "DC",
            "District of Columbia",
            TrebleActual,
            true,
            false,
            "D.C. Code § 42-3505.01",
        ),
        rule("DE", "Delaware", ActualDamagesOnly, true, false, "25 Del. C. § 5313"),
        rule(
            "FL",
            "Florida",
            GreaterOfRentMultOrActual { rent_mult: 300 },
            true,
            false,
            "Fla. Stat. § 83.67",
        ),
        rule("GA", "Georgia", ActualDamagesOnly, false, true, "O.C.G.A. § 44-7-50"),
        rule("HI", "Hawaii", ActualDamagesOnly, true, false, "HRS § 521-63"),
        rule("IA", "Iowa", ActualDamagesOnly, true, false, "Iowa Code § 562A.34"),
        rule("ID", "Idaho", ActualDamagesOnly, false, false, "no specific penalty statute"),
        rule(
            "IL",
            "Illinois",
            RentMultiplePlusActual { rent_mult: 200 },
            true,
            true,
            "Forcible Entry and Detainer Act (735 ILCS 5/9-101)",
        ),
        rule("IN", "Indiana", ActualDamagesOnly, false, false, "no specific penalty statute"),
        rule("KS", "Kansas", ActualDamagesOnly, true, false, "K.S.A. § 58-2563"),
        rule("KY", "Kentucky", ActualDamagesOnly, true, false, "KRS § 383.655"),
        rule("LA", "Louisiana", ActualDamagesOnly, false, true, "La. C.C.P. art. 4731"),
        rule(
            "MA",
            "Massachusetts",
            GreaterOfRentMultOrActual { rent_mult: 300 },
            true,
            true,
            "M.G.L. c. 186 § 14",
        ),
        rule("MD", "Maryland", ActualDamagesOnly, true, false, "Md. Real Prop. § 8-216"),
        rule("ME", "Maine", ActualDamagesOnly, true, false, "14 M.R.S. § 6014"),
        rule("MI", "Michigan", ActualDamagesOnly, true, true, "MCL § 600.2918"),
        rule("MN", "Minnesota", ActualDamagesOnly, true, false, "Minn. Stat. § 504B.231"),
        rule("MO", "Missouri", ActualDamagesOnly, false, true, "RSMo § 441.233"),
        rule("MS", "Mississippi", ActualDamagesOnly, false, false, "no specific statute"),
        rule("MT", "Montana", ActualDamagesOnly, true, false, "Mont. Code § 70-24-411"),
        rule("NC", "North Carolina", ActualDamagesOnly, true, false, "N.C.G.S. § 42-25.9"),
        rule("ND", "North Dakota", ActualDamagesOnly, true, false, "N.D.C.C. § 47-16-16"),
        rule("NE", "Nebraska", ActualDamagesOnly, true, false, "Neb. Rev. Stat. § 76-1431"),
        rule("NH", "New Hampshire", ActualDamagesOnly, true, true, "RSA § 540-A:3"),
        rule(
            "NJ",
            "New Jersey",
            TrebleActual,
            true,
            true,
            "N.J.S.A. § 2A:39-1",
        ),
        rule("NM", "New Mexico", ActualDamagesOnly, true, false, "NMSA § 47-8-36"),
        rule("NV", "Nevada", ActualDamagesOnly, true, true, "NRS § 118A.390"),
        rule(
            "NY",
            "New York",
            TrebleActual,
            true,
            true,
            "RPL § 235 + RPAPL § 853",
        ),
        rule("OH", "Ohio", ActualDamagesOnly, true, false, "ORC § 5321.15"),
        rule("OK", "Oklahoma", ActualDamagesOnly, true, false, "41 O.S. § 123"),
        rule("OR", "Oregon", ActualDamagesOnly, true, false, "ORS § 90.375"),
        rule("PA", "Pennsylvania", ActualDamagesOnly, false, true, "68 P.S. § 250.501"),
        rule("RI", "Rhode Island", ActualDamagesOnly, true, false, "R.I.G.L. § 34-18-33"),
        rule("SC", "South Carolina", ActualDamagesOnly, true, false, "SC Code § 27-40-660"),
        rule("SD", "South Dakota", ActualDamagesOnly, false, false, "no specific statute"),
        rule("TN", "Tennessee", ActualDamagesOnly, true, false, "Tenn. Code § 66-28-504"),
        rule(
            "TX",
            "Texas",
            AdditiveRentPlusFlat {
                rent_mult: 100,
                flat_cents: 100_000,
            },
            true,
            false,
            "Tex. Prop. Code § 92.0081",
        ),
        rule("UT", "Utah", ActualDamagesOnly, true, false, "Utah Code § 78B-6-814"),
        rule("VA", "Virginia", ActualDamagesOnly, true, false, "Va. Code § 55.1-1243"),
        rule("VT", "Vermont", ActualDamagesOnly, true, false, "9 V.S.A. § 4463"),
        rule(
            "WA",
            "Washington",
            GreaterOfRentMultOrActual { rent_mult: 300 },
            true,
            false,
            "RCW § 59.18.290 (lockout) / § 59.18.300 (utility shutoff: $100/day)",
        ),
        rule("WI", "Wisconsin", ActualDamagesOnly, true, false, "Wis. Stat. § 704.95"),
        rule("WV", "West Virginia", ActualDamagesOnly, false, false, "no specific statute"),
        rule("WY", "Wyoming", ActualDamagesOnly, false, false, "no specific statute"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        state: &str,
        days: u32,
        rent: i64,
        actual: i64,
    ) -> LockoutPenaltyInput {
        LockoutPenaltyInput {
            state_code: state.to_string(),
            days_locked_out: days,
            monthly_rent_cents: rent,
            actual_damages_cents: actual,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn california_per_day_minimum_floor_pinned() {
        // CA § 789.3: $100/day with $250 minimum. 1 day → $250 (floor wins).
        // Actual damages $500 → total = 250 + 500 = $750.
        let one_day = check(&input("CA", 1, 200_000, 50_000));
        assert_eq!(one_day.statutory_penalty_cents, 25_000); // $250 min
        assert_eq!(one_day.total_civil_exposure_cents, 75_000);

        // 5 days → per-day wins (500 > 250). Statutory = $500.
        let five_days = check(&input("CA", 5, 200_000, 50_000));
        assert_eq!(five_days.statutory_penalty_cents, 50_000);
        assert_eq!(five_days.total_civil_exposure_cents, 100_000);

        // Day-count exactly at the cross-over: 3 days = 300¢ > 250 floor.
        let three = check(&input("CA", 3, 200_000, 0));
        assert_eq!(three.statutory_penalty_cents, 30_000);
        // 2 days = 200 < 250 → floor wins.
        let two = check(&input("CA", 2, 200_000, 0));
        assert_eq!(two.statutory_penalty_cents, 25_000);
    }

    #[test]
    fn california_zero_days_still_returns_minimum_with_actual() {
        // Even 0 days locked out triggers the $250 minimum if a lockout
        // is established. Plus actual damages flow through.
        let r = check(&input("CA", 0, 200_000, 100_000));
        assert_eq!(r.statutory_penalty_cents, 25_000);
        assert_eq!(r.total_civil_exposure_cents, 125_000);
    }

    #[test]
    fn texas_additive_one_month_plus_thousand_plus_actual() {
        // TX § 92.0081: 1 month rent + $1000 + actual. Rent $1500 → 1500
        // + 1000 = 2500 statutory. Plus $400 actual = $2900 total.
        let r = check(&input("TX", 7, 150_000, 40_000));
        assert_eq!(r.statutory_penalty_cents, 250_000);
        assert_eq!(r.total_civil_exposure_cents, 290_000);
    }

    #[test]
    fn florida_greater_of_three_months_rent_or_actual() {
        // FL § 83.67: GREATER of 3× monthly rent or actual. Actual wins.
        let actual_wins = check(&input("FL", 10, 100_000, 500_000));
        assert_eq!(actual_wins.statutory_penalty_cents, 300_000); // 3× rent floor
        assert_eq!(actual_wins.total_civil_exposure_cents, 500_000); // actual

        // Statutory wins.
        let stat_wins = check(&input("FL", 10, 200_000, 100_000));
        assert_eq!(stat_wins.statutory_penalty_cents, 600_000);
        assert_eq!(stat_wins.total_civil_exposure_cents, 600_000);
    }

    #[test]
    fn washington_greater_of_three_months_rent_mirrors_florida() {
        // WA § 59.18.290 — same regime as FL but distinct citation
        // mentioning the separate utility-shutoff statute § 59.18.300.
        let r = check(&input("WA", 5, 200_000, 100_000));
        assert_eq!(r.statutory_penalty_cents, 600_000);
        assert!(r.citation.contains("§ 59.18.290"));
        assert!(r.citation.contains("utility shutoff"));
    }

    #[test]
    fn arizona_greater_of_two_months_rent_or_two_times_actual() {
        // AZ § 33-1367: BOTH sides multiplied by 2. Rent $1000 → 2× =
        // 2000. Actual $1500 → 2× = 3000. Actual wins at $3000.
        let r = check(&input("AZ", 1, 100_000, 150_000));
        assert_eq!(r.total_civil_exposure_cents, 300_000);

        // Reverse: rent high, actual low → rent multiple wins.
        let r2 = check(&input("AZ", 1, 300_000, 50_000));
        assert_eq!(r2.total_civil_exposure_cents, 600_000);
    }

    #[test]
    fn colorado_strictest_state_three_times_rent_or_5k_plus_actual() {
        // CO § 38-12-510: GREATER of 3× rent or $5,000, PLUS actual.
        // Low rent of $1000: 3× = $3,000 < $5,000 → floor wins. + $200
        // actual = $5,200 total.
        let low_rent = check(&input("CO", 5, 100_000, 20_000));
        assert_eq!(low_rent.statutory_penalty_cents, 500_000); // $5k floor
        assert_eq!(low_rent.total_civil_exposure_cents, 520_000);

        // High rent of $2000: 3× = $6,000 > $5,000 → 3× wins. + $200
        // actual = $6,200 total.
        let high_rent = check(&input("CO", 5, 200_000, 20_000));
        assert_eq!(high_rent.statutory_penalty_cents, 600_000);
        assert_eq!(high_rent.total_civil_exposure_cents, 620_000);
    }

    #[test]
    fn treble_states_3x_actual() {
        // NY, DC, NJ: 3× actual. Statutory bonus = 2× actual.
        for code in ["NY", "DC", "NJ"] {
            let r = check(&input(code, 1, 100_000, 50_000));
            assert_eq!(r.statutory_penalty_cents, 100_000); // 2× actual
            assert_eq!(r.total_civil_exposure_cents, 150_000); // 3× actual
        }
    }

    #[test]
    fn treble_with_zero_actual_yields_zero_total() {
        // Treble × 0 = 0. The treble model offers no statutory floor
        // independent of actual damages — distinct from CA's per-day
        // model which has a $250 minimum.
        let r = check(&input("NY", 0, 200_000, 0));
        assert_eq!(r.total_civil_exposure_cents, 0);
    }

    #[test]
    fn illinois_two_months_rent_plus_actual() {
        // IL Forcible Entry Act: 2× rent + actual. Rent $1000 → 2× =
        // $2000 + $300 actual = $2300.
        let r = check(&input("IL", 3, 100_000, 30_000));
        assert_eq!(r.statutory_penalty_cents, 200_000);
        assert_eq!(r.total_civil_exposure_cents, 230_000);
    }

    #[test]
    fn massachusetts_three_months_rent_mirrors_florida() {
        // MA c. 186 § 14: same regime as FL but distinct citation +
        // criminal exposure flag set.
        let r = check(&input("MA", 1, 150_000, 100_000));
        assert_eq!(r.statutory_penalty_cents, 450_000);
        assert_eq!(r.total_civil_exposure_cents, 450_000);
        assert!(r.criminal_exposure);
    }

    #[test]
    fn actual_damages_only_states_pass_through_actual() {
        // Most states: statutory = 0, total = actual. Self-help still
        // prohibited but no civil-penalty formula in the table.
        for code in ["AK", "AL", "DE", "MD", "OR", "VA"] {
            let r = check(&input(code, 5, 200_000, 75_000));
            assert_eq!(r.statutory_penalty_cents, 0);
            assert_eq!(r.total_civil_exposure_cents, 75_000);
        }
    }

    #[test]
    fn no_statute_states_truly_no_floor() {
        // AR / WV / WY / MS / SD / ID — no statute. Tenant gets actual
        // common-law damages only.
        for code in ["AR", "WV", "WY", "MS", "SD", "ID"] {
            let r = check(&input(code, 5, 200_000, 75_000));
            assert_eq!(r.total_civil_exposure_cents, 75_000);
            // AR is the only state with NoStatute regime (the rest are
            // ActualDamagesOnly with citations).
        }
        let ar = check(&input("AR", 5, 200_000, 75_000));
        assert!(ar.no_statute);
    }

    #[test]
    fn unknown_state_flagged_no_statute_with_error_note() {
        let r = check(&input("ZZ", 5, 200_000, 75_000));
        assert!(r.no_statute);
        assert_eq!(r.total_civil_exposure_cents, 0);
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("CA").is_some());
        assert!(lookup("ca").is_some());
        assert!(lookup("Ca").is_some());
    }

    #[test]
    fn all_states_returns_sorted_by_code() {
        let states = all_states();
        assert_eq!(states.len(), 51);
        assert_eq!(states.first().unwrap().state_code, "AK");
        assert_eq!(states.last().unwrap().state_code, "WY");
    }

    #[test]
    fn citation_present_for_every_row() {
        for r in TABLE.values() {
            assert!(!r.citation.is_empty(), "{} citation empty", r.state_code);
        }
    }

    #[test]
    fn attorney_fees_flag_pinned_per_state() {
        // Strong fee-shifting states: CA / TX / FL / WA / CO / NY. Each
        // has explicit fee-recovery language and the flag is critical
        // for the downstream UI to highlight "tenant gets fees too".
        for code in ["CA", "TX", "FL", "WA", "CO", "NY", "AZ", "MA", "IL"] {
            let r = lookup(code).unwrap();
            assert!(r.attorney_fees_recoverable, "{code} should recover fees");
        }
    }

    #[test]
    fn criminal_exposure_states_pinned() {
        // NY, NJ, MA, GA, MI, NH, CT, LA, MO, NV, PA — states with
        // statutorily-named criminal misdemeanor / criminal trespass
        // exposure. Caller should warn the landlord that a lockout is
        // not just a civil suit — DA could charge.
        for code in ["NY", "NJ", "MA", "GA", "MI", "NH", "CT", "LA", "MO", "NV", "PA"] {
            let r = lookup(code).unwrap();
            assert!(r.criminal_exposure, "{code} should flag criminal exposure");
        }
    }

    #[test]
    fn arkansas_uniquely_no_statute_landlord_friendly() {
        // AR is famously the only US state with effectively no tenant
        // protections — no_statute is the correct classification.
        let ar = lookup("AR").unwrap();
        assert!(matches!(ar.regime, PenaltyRegime::NoStatute));
        let r = check(&input("AR", 30, 100_000, 200_000));
        assert!(r.no_statute);
        assert_eq!(r.total_civil_exposure_cents, 200_000);
    }

    #[test]
    fn zero_rent_zero_actual_zero_days_no_panic() {
        // Stress: all-zeros input must not panic on any regime.
        // Three states have statutory floors that survive all-zero input:
        //   CA: $250 minimum on the per-day component
        //   TX: $1,000 flat (independent of rent and actual)
        //   CO: $5,000 flat floor on the greater-of clause
        // Everything else zeroes out cleanly.
        let ca = check(&input("CA", 0, 0, 0));
        assert_eq!(ca.statutory_penalty_cents, 25_000);

        let tx = check(&input("TX", 0, 0, 0));
        assert_eq!(tx.statutory_penalty_cents, 100_000); // $1000 flat

        let co = check(&input("CO", 0, 0, 0));
        assert_eq!(co.statutory_penalty_cents, 500_000); // $5k flat

        // Regimes with no flat floor zero out at all-zero input.
        for code in ["FL", "WA", "NY", "AZ", "IL", "MA"] {
            let r = check(&input(code, 0, 0, 0));
            assert_eq!(
                r.total_civil_exposure_cents, 0,
                "{code} expected zero total on all-zero input"
            );
        }
    }

    #[test]
    fn large_rent_no_overflow_via_saturating_mul() {
        // Pathological huge rent + large day count must not overflow
        // i64 — saturating_mul should clamp at i64::MAX rather than wrap.
        // 1 trillion ¢ rent ($10 billion) × 300 (CO 3× cap) / 100 should
        // stay within i64 range.
        let r = check(&input("CO", 1, 1_000_000_000_000, 0));
        // 3× the rent = 3 trillion ¢, well under i64::MAX (~9.2 quintillion).
        assert!(r.total_civil_exposure_cents > 0);
        assert!(r.total_civil_exposure_cents < i64::MAX);
    }

    #[test]
    fn texas_zero_actual_still_has_statutory_floor() {
        // Even with zero actual damages, TX gives 1 month rent + $1000.
        // The "+$1000 flat" portion is what catches no-actual-damage cases.
        let r = check(&input("TX", 0, 150_000, 0));
        assert_eq!(r.statutory_penalty_cents, 250_000); // 1.5k rent + 1k flat
        assert_eq!(r.total_civil_exposure_cents, 250_000);
    }

    #[test]
    fn greater_of_with_equal_amounts_picks_one_consistently() {
        // FL: 3× $100 rent = $300; actual = $300. Both equal → max returns
        // either. Pinning that the result is exactly $300, no double-count.
        let r = check(&input("FL", 1, 10_000, 30_000));
        assert_eq!(r.total_civil_exposure_cents, 30_000);
    }
}
