//! IRC §83(b) — Election to include in gross income at the time of grant.
//!
//! Every founder and early employee receiving restricted stock or
//! restricted stock units needs to decide within **30 calendar days of
//! grant** whether to file a §83(b) election. The election is one of
//! the most consequential tax-position choices in the code:
//!
//! - **§83(a)** (default): ordinary income is recognized at VESTING,
//!   measured by `FMV_vesting - amount_paid`. LTCG holding period
//!   begins at vesting.
//! - **§83(b)** (election): ordinary income is recognized at GRANT,
//!   measured by `FMV_grant - amount_paid`. LTCG holding period begins
//!   at grant.
//!
//! When stock appreciates significantly between grant and vesting (the
//! canonical founder case — grant FMV ≈ $0.001/share, vesting FMV
//! $10+/share), the election converts what would have been ordinary
//! income at vesting into long-term capital gain at sale. For a founder
//! with a $10M post-vesting appreciation, that's the difference between
//! ~37% federal ordinary + state + FICA-Medicare and 20% federal LTCG
//! + 3.8% NIIT + state — easily a 20%+ savings on the appreciation.
//!
//! **The 30-day deadline is bright-line and unforgivable.** No extension,
//! no equitable exception, no judicial relief. Carta + Cooley + the IRS
//! agree: file by day 30 or the option is gone.
//!
//! **§83(b)(2) forfeiture trap.** If the property is later forfeited
//! (e.g., the employee leaves before vesting completes), the §83(b)
//! election cannot be undone. The taxpayer paid ordinary income tax at
//! grant on property that was never received. Per §83(b)(2), there is
//! NO refund and NO deduction is allowed for the previously included
//! amount. The only loss recognized is the amount paid for the
//! property (the cash purchase price, if any) — usually zero for
//! pure stock grants. This is the downside risk that callers must
//! model and surface.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section83bInput {
    pub grant_date: NaiveDate,
    pub vesting_date: NaiveDate,
    /// Date the §83(b) election was filed (postmarked). `None` if not
    /// filed at all.
    pub election_filed_date: Option<NaiveDate>,
    pub fmv_at_grant: Decimal,
    /// Cash paid for the restricted property at grant. Zero for pure
    /// RSU/RSA grants.
    pub amount_paid_at_grant: Decimal,
    pub fmv_at_vesting: Decimal,
    /// Date of eventual sale. `None` if not yet sold (compute returns
    /// no capital-gain results).
    pub sale_date: Option<NaiveDate>,
    pub sale_price_per_share: Option<Decimal>,
    /// True if the property is forfeited before vesting. §83(b)(2) — no
    /// refund of grant-day tax even if election was filed.
    pub forfeited_before_vesting: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section83bRule {
    /// §83(b) election filed within 30 days; ordinary income at grant
    /// and LTCG clock from grant.
    ElectionTimelyValid,
    /// §83(b) election filed after 30-day deadline. Election invalid;
    /// reverts to §83(a) treatment at vesting.
    ElectionLateInvalid,
    /// No §83(b) election made. §83(a) default applies.
    NoElection,
    /// §83(b) election valid but property forfeited before vesting.
    /// §83(b)(2) — no refund of taxes paid at grant.
    ElectionWithForfeiture,
    /// No §83(b) election and property forfeited before vesting.
    /// §83(a) never triggered — no ordinary income recognized.
    NoElectionForfeiture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapitalGainCharacter {
    ShortTermCapital,
    LongTermCapital,
    NotYetSold,
    NoGainRecognized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section83bResult {
    pub rule_path: Section83bRule,
    /// True if the election is timely (filed within 30 days of grant).
    pub election_timely: bool,
    /// Days from grant to election filing, if the election was filed.
    pub days_grant_to_election: Option<i64>,
    /// Ordinary income recognized under the applicable rule.
    pub ordinary_income: Decimal,
    /// LTCG holding-period start date — grant for ElectionTimelyValid,
    /// vesting otherwise.
    pub holding_period_start: NaiveDate,
    /// Cost basis after the relevant ordinary inclusion.
    pub basis_after_ordinary_inclusion: Decimal,
    /// Capital gain on sale (positive = gain, negative = loss).
    pub capital_gain_at_sale: Decimal,
    pub capital_gain_character: CapitalGainCharacter,
    /// Days from holding_period_start to sale_date.
    pub holding_period_days: Option<i64>,
    /// §83(b)(2) loss when forfeited after valid election: limited to
    /// amount paid for the property (NOT the previously included ordinary
    /// amount). Zero unless forfeited.
    pub forfeiture_loss_amount_paid_only: Decimal,
    /// Difference: (ordinary_income_under_election) - (ordinary_income_
    /// without_election). Negative when the election was the right
    /// choice (it reduced ordinary income).
    pub election_ordinary_savings: Decimal,
    pub note: String,
}

/// §83(b) election must be filed within 30 days of grant per the statute
/// and the regulations. The deadline is bright-line.
const ELECTION_WINDOW_DAYS: i64 = 30;

/// §1222 holding-period boundary: > 365 days = long-term capital.
const ONE_YEAR_DAYS: i64 = 365;

pub fn compute(input: &Section83bInput) -> Section83bResult {
    // Step 1: Determine election validity.
    let days_to_election = input
        .election_filed_date
        .map(|f| (f - input.grant_date).num_days());
    let election_timely = match days_to_election {
        Some(d) => (0..=ELECTION_WINDOW_DAYS).contains(&d),
        None => false,
    };

    // Step 2: Forfeiture path.
    if input.forfeited_before_vesting {
        if election_timely {
            // §83(b)(2) — election valid, ordinary income at grant was
            // included; no refund; loss recognized = amount paid only.
            let grant_ordinary =
                (input.fmv_at_grant - input.amount_paid_at_grant).max(Decimal::ZERO);
            return Section83bResult {
                rule_path: Section83bRule::ElectionWithForfeiture,
                election_timely: true,
                days_grant_to_election: days_to_election,
                ordinary_income: grant_ordinary,
                holding_period_start: input.grant_date,
                basis_after_ordinary_inclusion: input.fmv_at_grant,
                capital_gain_at_sale: Decimal::ZERO,
                capital_gain_character: CapitalGainCharacter::NoGainRecognized,
                holding_period_days: None,
                forfeiture_loss_amount_paid_only: input.amount_paid_at_grant,
                election_ordinary_savings: grant_ordinary
                    - Decimal::ZERO, /* vs vesting (no inclusion since forfeited) */
                note: format!(
                    "§83(b)(2) forfeiture trap — election was timely (day {}/30), grant-day ordinary income of ${} was already recognized; NO refund of tax paid; loss recognized = amount paid ${} only",
                    days_to_election.unwrap_or(0),
                    grant_ordinary.round_dp(2),
                    input.amount_paid_at_grant.round_dp(2)
                ),
            };
        } else {
            // No (or late) election; §83(a) never triggered because
            // vesting never occurred — clean outcome.
            return Section83bResult {
                rule_path: Section83bRule::NoElectionForfeiture,
                election_timely: false,
                days_grant_to_election: days_to_election,
                ordinary_income: Decimal::ZERO,
                holding_period_start: input.vesting_date,
                basis_after_ordinary_inclusion: input.amount_paid_at_grant,
                capital_gain_at_sale: Decimal::ZERO,
                capital_gain_character: CapitalGainCharacter::NoGainRecognized,
                holding_period_days: None,
                forfeiture_loss_amount_paid_only: input.amount_paid_at_grant,
                election_ordinary_savings: -(input.fmv_at_grant
                    - input.amount_paid_at_grant)
                    .max(Decimal::ZERO),
                note: format!(
                    "no valid §83(b) election + forfeiture — clean outcome; §83(a) never triggered; loss = amount paid ${}",
                    input.amount_paid_at_grant.round_dp(2)
                ),
            };
        }
    }

    // Step 3: Non-forfeiture paths.
    let (rule, ordinary, hp_start, basis) = if election_timely {
        let grant_ordinary = (input.fmv_at_grant - input.amount_paid_at_grant).max(Decimal::ZERO);
        (
            Section83bRule::ElectionTimelyValid,
            grant_ordinary,
            input.grant_date,
            input.fmv_at_grant,
        )
    } else if input.election_filed_date.is_some() {
        // Late election — invalid; revert to §83(a).
        let vest_ordinary = (input.fmv_at_vesting - input.amount_paid_at_grant).max(Decimal::ZERO);
        (
            Section83bRule::ElectionLateInvalid,
            vest_ordinary,
            input.vesting_date,
            input.fmv_at_vesting,
        )
    } else {
        let vest_ordinary = (input.fmv_at_vesting - input.amount_paid_at_grant).max(Decimal::ZERO);
        (
            Section83bRule::NoElection,
            vest_ordinary,
            input.vesting_date,
            input.fmv_at_vesting,
        )
    };

    // Step 4: Capital gain at sale, if sold.
    let (cap_gain, cap_char, hp_days) = match (input.sale_date, input.sale_price_per_share) {
        (Some(sd), Some(sp)) => {
            let gain = sp - basis;
            let days = (sd - hp_start).num_days();
            let character = if days > ONE_YEAR_DAYS {
                CapitalGainCharacter::LongTermCapital
            } else {
                CapitalGainCharacter::ShortTermCapital
            };
            (gain, character, Some(days))
        }
        _ => (Decimal::ZERO, CapitalGainCharacter::NotYetSold, None),
    };

    // Step 5: Election savings — comparison vs §83(a) baseline.
    let vest_ordinary_baseline =
        (input.fmv_at_vesting - input.amount_paid_at_grant).max(Decimal::ZERO);
    let savings = match rule {
        Section83bRule::ElectionTimelyValid => ordinary - vest_ordinary_baseline,
        _ => Decimal::ZERO,
    };

    let note = match rule {
        Section83bRule::ElectionTimelyValid => format!(
            "§83(b) election timely (day {}/30); ordinary income of ${} recognized at grant ({}); LTCG clock starts at grant; election saves ${} of ordinary income (negative = good)",
            days_to_election.unwrap_or(0),
            ordinary.round_dp(2),
            input.grant_date,
            savings.round_dp(2)
        ),
        Section83bRule::ElectionLateInvalid => format!(
            "§83(b) election LATE — filed day {}/30 — invalid per the bright-line 30-day deadline; reverts to §83(a) at vesting with ordinary income of ${}",
            days_to_election.unwrap_or(0),
            ordinary.round_dp(2)
        ),
        Section83bRule::NoElection => format!(
            "no §83(b) election — §83(a) default: ordinary income ${} at vesting ({}); LTCG clock starts at vesting",
            ordinary.round_dp(2),
            input.vesting_date
        ),
        _ => String::new(),
    };

    Section83bResult {
        rule_path: rule,
        election_timely,
        days_grant_to_election: days_to_election,
        ordinary_income: ordinary,
        holding_period_start: hp_start,
        basis_after_ordinary_inclusion: basis,
        capital_gain_at_sale: cap_gain,
        capital_gain_character: cap_char,
        holding_period_days: hp_days,
        forfeiture_loss_amount_paid_only: Decimal::ZERO,
        election_ordinary_savings: savings,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn base() -> Section83bInput {
        Section83bInput {
            grant_date: d(2026, 1, 1),
            vesting_date: d(2030, 1, 1), // 4-year vesting
            election_filed_date: Some(d(2026, 1, 15)),
            fmv_at_grant: dec!(0.01), // founder grant
            amount_paid_at_grant: dec!(0.01),
            fmv_at_vesting: dec!(10),
            sale_date: Some(d(2031, 6, 1)),
            sale_price_per_share: Some(dec!(50)),
            forfeited_before_vesting: false,
        }
    }

    #[test]
    fn timely_election_within_30_days_is_valid() {
        // Election filed 14 days after grant → valid; ordinary income
        // at grant = $0 (FMV = paid), LTCG clock starts at grant.
        let r = compute(&base());
        assert_eq!(r.rule_path, Section83bRule::ElectionTimelyValid);
        assert!(r.election_timely);
        assert_eq!(r.days_grant_to_election, Some(14));
        assert_eq!(r.ordinary_income, Decimal::ZERO);
        assert_eq!(r.holding_period_start, d(2026, 1, 1));
    }

    #[test]
    fn election_filed_day_zero_is_valid() {
        // Same-day filing — day 0 — is within the 30-day window.
        let mut i = base();
        i.election_filed_date = Some(d(2026, 1, 1));
        let r = compute(&i);
        assert_eq!(r.days_grant_to_election, Some(0));
        assert!(r.election_timely);
    }

    #[test]
    fn election_filed_day_30_exact_boundary_is_valid() {
        // Day 30 exact — bright-line boundary. Still valid.
        let mut i = base();
        i.election_filed_date = Some(d(2026, 1, 31));
        let r = compute(&i);
        assert_eq!(r.days_grant_to_election, Some(30));
        assert!(r.election_timely);
        assert_eq!(r.rule_path, Section83bRule::ElectionTimelyValid);
    }

    #[test]
    fn election_filed_day_31_invalid_per_bright_line() {
        // Day 31 — one day too late. Brightline 30-day rule = no
        // extension, no equitable relief. Reverts to §83(a).
        let mut i = base();
        i.election_filed_date = Some(d(2026, 2, 1));
        let r = compute(&i);
        assert_eq!(r.days_grant_to_election, Some(31));
        assert!(!r.election_timely);
        assert_eq!(r.rule_path, Section83bRule::ElectionLateInvalid);
        assert!(r.note.contains("LATE"));
    }

    #[test]
    fn no_election_filed_falls_back_to_section_83a() {
        // No election → §83(a): ordinary income at vesting.
        // FMV vesting $10, paid $0.01 → ordinary $9.99.
        let mut i = base();
        i.election_filed_date = None;
        let r = compute(&i);
        assert_eq!(r.rule_path, Section83bRule::NoElection);
        assert_eq!(r.ordinary_income, dec!(9.99));
        assert_eq!(r.holding_period_start, d(2030, 1, 1));
    }

    #[test]
    fn founder_grant_election_saves_ordinary_income() {
        // Classic founder case: paid $0.01 = FMV at grant.
        // With election: ordinary $0 at grant. Without: $9.99 at vesting.
        // Savings = $0 - $9.99 = -$9.99 (negative = good).
        let r = compute(&base());
        assert_eq!(r.election_ordinary_savings, dec!(-9.99));
    }

    #[test]
    fn election_with_appreciation_to_50_full_ltcg() {
        // FMV grant $0.01, sale price $50, sold 2031-06-01.
        // Election: basis = $0.01, gain at sale = $49.99.
        // Holding period from 2026-01-01 to 2031-06-01 = ~5 years → LTCG.
        let r = compute(&base());
        assert_eq!(r.capital_gain_at_sale, dec!(49.99));
        assert_eq!(
            r.capital_gain_character,
            CapitalGainCharacter::LongTermCapital
        );
        assert!(r.holding_period_days.unwrap() > ONE_YEAR_DAYS);
    }

    #[test]
    fn no_election_with_appreciation_to_50_partial_ltcg_partial_ordinary() {
        // Without election: ordinary $9.99 at vesting; basis steps up to
        // $10 (vesting FMV). Sale at $50 → cap gain = $40. Sold 17 months
        // after vesting → LTCG. The election captured an extra $9.99 of
        // appreciation as LTCG instead of ordinary.
        let mut i = base();
        i.election_filed_date = None;
        let r = compute(&i);
        assert_eq!(r.ordinary_income, dec!(9.99));
        assert_eq!(r.capital_gain_at_sale, dec!(40));
        assert_eq!(
            r.capital_gain_character,
            CapitalGainCharacter::LongTermCapital
        );
    }

    #[test]
    fn election_sale_within_one_year_of_grant_is_stcg() {
        // Election valid, sale 6 months after grant → STCG (≤ 365 days).
        let mut i = base();
        i.sale_date = Some(d(2026, 7, 1));
        i.sale_price_per_share = Some(dec!(5));
        let r = compute(&i);
        assert_eq!(
            r.capital_gain_character,
            CapitalGainCharacter::ShortTermCapital
        );
    }

    #[test]
    fn election_sale_at_366_days_is_long_term() {
        // Day 366 from grant → > 1 year → LTCG.
        let mut i = base();
        i.sale_date = Some(d(2027, 1, 2));
        i.sale_price_per_share = Some(dec!(5));
        let r = compute(&i);
        assert_eq!(r.holding_period_days, Some(366));
        assert_eq!(
            r.capital_gain_character,
            CapitalGainCharacter::LongTermCapital
        );
    }

    #[test]
    fn no_election_sale_one_year_after_vesting_is_ltcg() {
        // No election: clock starts at vesting (2030-01-01). Sale 17
        // months later (2031-06-01) → LTCG.
        let mut i = base();
        i.election_filed_date = None;
        let r = compute(&i);
        assert_eq!(r.holding_period_start, d(2030, 1, 1));
        assert_eq!(
            r.capital_gain_character,
            CapitalGainCharacter::LongTermCapital
        );
    }

    #[test]
    fn no_election_sale_within_one_year_of_vesting_is_stcg() {
        // No election + sale 6 months after vesting → STCG.
        let mut i = base();
        i.election_filed_date = None;
        i.sale_date = Some(d(2030, 7, 1));
        let r = compute(&i);
        assert_eq!(
            r.capital_gain_character,
            CapitalGainCharacter::ShortTermCapital
        );
    }

    #[test]
    fn forfeiture_with_valid_election_no_refund_per_83b2() {
        // §83(b)(2) trap. Election was valid; grant-day ordinary income
        // was recognized; property forfeited. NO refund. Loss limited
        // to amount paid for the property ($0.01).
        let mut i = base();
        i.forfeited_before_vesting = true;
        let r = compute(&i);
        assert_eq!(r.rule_path, Section83bRule::ElectionWithForfeiture);
        assert_eq!(r.forfeiture_loss_amount_paid_only, dec!(0.01));
        assert!(r.note.contains("§83(b)(2)"));
        assert!(r.note.contains("NO refund"));
    }

    #[test]
    fn forfeiture_without_election_clean_no_income() {
        // No election + forfeiture → §83(a) never triggered. No ordinary
        // income recognized. Loss = amount paid.
        let mut i = base();
        i.election_filed_date = None;
        i.forfeited_before_vesting = true;
        let r = compute(&i);
        assert_eq!(r.rule_path, Section83bRule::NoElectionForfeiture);
        assert_eq!(r.ordinary_income, Decimal::ZERO);
        assert_eq!(r.forfeiture_loss_amount_paid_only, dec!(0.01));
    }

    #[test]
    fn forfeiture_with_late_election_clean_no_income() {
        // Late election is invalid → §83(a) default. Same outcome as
        // no-election forfeiture path: clean, no income.
        let mut i = base();
        i.election_filed_date = Some(d(2026, 2, 5)); // day 35, late
        i.forfeited_before_vesting = true;
        let r = compute(&i);
        assert_eq!(r.rule_path, Section83bRule::NoElectionForfeiture);
        assert_eq!(r.ordinary_income, Decimal::ZERO);
    }

    #[test]
    fn fmv_grant_above_paid_election_creates_ordinary_at_grant() {
        // Paid $1 for stock worth $5 at grant. Election: ordinary $4
        // at grant. Without election: ordinary $9 at vesting (FMV $10).
        let mut i = base();
        i.fmv_at_grant = dec!(5);
        i.amount_paid_at_grant = dec!(1);
        let r = compute(&i);
        assert_eq!(r.ordinary_income, dec!(4));
        assert_eq!(r.basis_after_ordinary_inclusion, dec!(5));
    }

    #[test]
    fn election_wrong_when_fmv_drops_after_grant() {
        // Election was a mistake: paid ordinary tax on $5 grant FMV but
        // vesting FMV is only $3. Election locked in $4 ordinary; §83(a)
        // would have produced only $2. Election savings is +$2 (positive
        // = bad; election cost extra ordinary income).
        let mut i = base();
        i.fmv_at_grant = dec!(5);
        i.amount_paid_at_grant = dec!(1);
        i.fmv_at_vesting = dec!(3);
        let r = compute(&i);
        assert_eq!(r.ordinary_income, dec!(4));
        assert_eq!(r.election_ordinary_savings, dec!(2));
    }

    #[test]
    fn negative_grant_minus_paid_clamps_to_zero_ordinary() {
        // Pathological: paid $5 for stock worth $0.01 at grant. The
        // ordinary income clamps at zero (can't be negative — the loss
        // is recognized at sale, not as negative ordinary).
        let mut i = base();
        i.fmv_at_grant = dec!(0.01);
        i.amount_paid_at_grant = dec!(5);
        let r = compute(&i);
        assert_eq!(r.ordinary_income, Decimal::ZERO);
    }

    #[test]
    fn not_yet_sold_returns_no_gain() {
        let mut i = base();
        i.sale_date = None;
        i.sale_price_per_share = None;
        let r = compute(&i);
        assert_eq!(r.capital_gain_character, CapitalGainCharacter::NotYetSold);
        assert_eq!(r.capital_gain_at_sale, Decimal::ZERO);
        assert!(r.holding_period_days.is_none());
    }

    #[test]
    fn election_before_grant_date_invalid() {
        // Election filed BEFORE grant date — impossible in practice but
        // pathological input. days_to_election goes negative → election_
        // timely = false.
        let mut i = base();
        i.election_filed_date = Some(d(2025, 12, 31));
        let r = compute(&i);
        assert_eq!(r.days_grant_to_election, Some(-1));
        assert!(!r.election_timely);
    }

    #[test]
    fn very_large_grant_appreciation_no_precision_loss() {
        // FMV grant $0.0001/share, FMV vesting $1000, sale $5000.
        // Decimal must stay exact across the multiplication.
        let mut i = base();
        i.fmv_at_grant = dec!(0.0001);
        i.amount_paid_at_grant = dec!(0.0001);
        i.fmv_at_vesting = dec!(1000);
        i.sale_price_per_share = Some(dec!(5000));
        let r = compute(&i);
        assert_eq!(r.capital_gain_at_sale, dec!(4999.9999));
    }

    #[test]
    fn day_31_late_election_with_sale_proceeds_uses_vesting_basis() {
        // Late election → §83(a) basis at vesting FMV ($10), not grant
        // ($0.01). Sale at $50 → cap gain $40, not $49.99.
        let mut i = base();
        i.election_filed_date = Some(d(2026, 2, 1));
        let r = compute(&i);
        assert_eq!(r.basis_after_ordinary_inclusion, dec!(10));
        assert_eq!(r.capital_gain_at_sale, dec!(40));
    }

    #[test]
    fn note_describes_30_day_deadline_explicitly() {
        // The note must spell out the day/30 ratio so the UI can show
        // the user how close they were to missing the deadline.
        let r = compute(&base());
        assert!(r.note.contains("day 14/30"));
    }

    #[test]
    fn forfeiture_note_calls_out_section_83b2_trap() {
        let mut i = base();
        i.forfeited_before_vesting = true;
        let r = compute(&i);
        assert!(r.note.contains("§83(b)(2)"));
        assert!(r.note.contains("forfeiture trap"));
    }
}
