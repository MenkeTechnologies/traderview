//! IRC §170(e) — Charitable contributions of appreciated property.
//!
//! The single highest-frequency tax-planning move for successful traders:
//! donate winners to charity, deduct FMV (or basis under specific paths),
//! pay NO capital gain tax on the embedded appreciation. Pairs with
//! §1091 wash sale only by exclusion — gifts aren't sales, so no wash
//! sale issue.
//!
//! Six rule paths cover every combination of property kind × charity
//! type × basis-election flag:
//!
//! 1. **LTCG to public charity (no election)** — §170(b)(1)(C)(i):
//!    contribution = FMV (no reduction), 30% AGI limit. The canonical
//!    "donate appreciated stock" path.
//! 2. **LTCG to public charity (basis election)** — §170(b)(1)(C)(iii):
//!    contribution = basis, 50% AGI limit. Trades the FMV deduction for
//!    a higher AGI cap. Useful when basis is close to FMV or AGI is
//!    constrained.
//! 3. **LTCG to private foundation (qualified appreciated stock)** —
//!    §170(e)(5): publicly-traded stock not exceeding 10% of outstanding
//!    shares qualifies for FMV deduction (carve-out from the general
//!    private-foundation reduction rule), 20% AGI limit.
//! 4. **LTCG to private foundation (not QAS)** — §170(e)(1)(B)(ii):
//!    contribution = basis after reduction, 20% AGI limit. Real estate,
//!    closely-held stock, art, etc.
//! 5. **STCG or ordinary income property** — §170(e)(1)(A):
//!    contribution reduced by the entire would-be gain, so deduction =
//!    basis. 50% AGI limit (public) / 30% AGI (private). The "don't
//!    donate winners you've held < 1 year" trap.
//! 6. **Tangible personal property whose use is unrelated to the
//!    donee's exempt purpose** — §170(e)(1)(B)(i): contribution = basis
//!    even for LTCG. Hits the "donate art to a hospital" case but not
//!    "donate art to a museum that displays it".
//!
//! §170(d) provides a **5-year carryforward** for any contribution
//! amount that exceeds the AGI limit in the current year. Carryforward
//! retains its character category, so callers must track buckets
//! separately if they straddle paths.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyKind {
    /// Capital asset held > 1 year. The canonical "appreciated stock"
    /// category for this module.
    LongTermCapitalGain,
    /// Capital asset held ≤ 1 year.
    ShortTermCapitalGain,
    /// §1221(a)(1) inventory or §1221(a)(2) trade/business property —
    /// any property whose sale would produce ordinary income.
    OrdinaryIncome,
    /// Tangible personal property whose use is unrelated to the donee's
    /// exempt purpose (the donor's intended use, not the donee's actual
    /// use — that's a caller-side determination).
    TangiblePersonalUnrelatedUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharityType {
    /// §170(b)(1)(A) public charity — churches, schools, hospitals,
    /// 509(a)(1)/(a)(2)/(a)(3) supporting orgs, etc.
    PublicCharity,
    /// §509(a) private foundation — the family-foundation case.
    PrivateFoundation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section170eInput {
    pub property_kind: PropertyKind,
    pub fmv: Decimal,
    pub basis: Decimal,
    pub charity_type: CharityType,
    pub agi: Decimal,
    /// Unused carryforward from prior years (same character category).
    pub prior_year_carryover: Decimal,
    /// Other charitable contributions already deducted this year against
    /// the same AGI bucket.
    pub other_charity_contributions_this_year: Decimal,
    /// §170(b)(1)(C)(iii) election to deduct LTCG public-charity gifts
    /// at basis with the 50% AGI limit instead of FMV at 30%.
    pub elect_basis_for_higher_limit: bool,
    /// True if this is publicly-traded stock not exceeding 10% of the
    /// corporation's outstanding shares — §170(e)(5) qualified
    /// appreciated stock exception to the private-foundation reduction.
    pub is_qualified_appreciated_stock: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section170eRule {
    LtcgPublicFmv,
    LtcgPublicBasisElect,
    LtcgPrivateFoundationQas,
    LtcgPrivateFoundationBasis,
    StcgOrOrdinaryReduction,
    TangibleUnrelatedReduction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section170eResult {
    /// Contribution amount AFTER §170(e) reduction (FMV for LTCG-public-
    /// FMV and LTCG-private-QAS paths; basis otherwise).
    pub contribution_amount_after_170e_reduction: Decimal,
    /// AGI limit as basis points: 5000 = 50%, 3000 = 30%, 2000 = 20%.
    pub agi_limit_basis_points: u32,
    pub agi_limit_dollars: Decimal,
    /// AGI budget remaining after other-this-year contributions consume
    /// the bucket. Never negative (clamped at zero).
    pub remaining_agi_budget: Decimal,
    /// Deductible amount THIS year: min(contribution + prior carryover,
    /// remaining_agi_budget).
    pub deductible_this_year: Decimal,
    /// Excess that carries forward to next year under §170(d).
    pub carryforward_to_next_year: Decimal,
    /// Embedded capital gain that escapes taxation entirely (only on
    /// FMV-deduction paths; zero for reduction paths).
    pub capital_gain_eliminated_no_tax: Decimal,
    pub rule_path: Section170eRule,
    pub note: String,
}

pub fn compute(input: &Section170eInput) -> Section170eResult {
    // Step 1: §170(e) reduction — pick the rule path.
    let (contribution, rule) = match (
        input.property_kind,
        input.charity_type,
        input.elect_basis_for_higher_limit,
        input.is_qualified_appreciated_stock,
    ) {
        // §170(e)(1)(B)(i) tangible personal unrelated use → basis
        // regardless of holding period.
        (PropertyKind::TangiblePersonalUnrelatedUse, _, _, _) => {
            (input.basis, Section170eRule::TangibleUnrelatedReduction)
        }
        // §170(e)(1)(A) ordinary/STCG property → basis
        (PropertyKind::ShortTermCapitalGain, _, _, _)
        | (PropertyKind::OrdinaryIncome, _, _, _) => {
            (input.basis, Section170eRule::StcgOrOrdinaryReduction)
        }
        // §170(b)(1)(C)(iii) basis election → basis at 50% cap
        (PropertyKind::LongTermCapitalGain, CharityType::PublicCharity, true, _) => {
            (input.basis, Section170eRule::LtcgPublicBasisElect)
        }
        // §170(b)(1)(C)(i) LTCG to public charity → FMV
        (PropertyKind::LongTermCapitalGain, CharityType::PublicCharity, false, _) => {
            (input.fmv, Section170eRule::LtcgPublicFmv)
        }
        // §170(e)(5) QAS exception → FMV to private foundation
        (PropertyKind::LongTermCapitalGain, CharityType::PrivateFoundation, _, true) => {
            (input.fmv, Section170eRule::LtcgPrivateFoundationQas)
        }
        // §170(e)(1)(B)(ii) general private-foundation reduction → basis
        (PropertyKind::LongTermCapitalGain, CharityType::PrivateFoundation, _, false) => {
            (input.basis, Section170eRule::LtcgPrivateFoundationBasis)
        }
    };

    // Step 2: AGI limit basis points by rule × charity type.
    let agi_bp: u32 = match rule {
        Section170eRule::LtcgPublicFmv => 3000,
        Section170eRule::LtcgPublicBasisElect => 5000,
        Section170eRule::LtcgPrivateFoundationQas => 2000,
        Section170eRule::LtcgPrivateFoundationBasis => 2000,
        Section170eRule::StcgOrOrdinaryReduction | Section170eRule::TangibleUnrelatedReduction => {
            match input.charity_type {
                CharityType::PublicCharity => 5000,
                CharityType::PrivateFoundation => 3000,
            }
        }
    };

    let agi_limit_dollars =
        input.agi * Decimal::from(agi_bp) / Decimal::from(10_000);
    let remaining = (agi_limit_dollars - input.other_charity_contributions_this_year)
        .max(Decimal::ZERO);

    // Step 3: Deductible this year = min(contribution + prior_cf, remaining).
    let total_claimed = contribution + input.prior_year_carryover;
    let deductible = total_claimed.min(remaining).max(Decimal::ZERO);
    let carryforward = (total_claimed - deductible).max(Decimal::ZERO);

    // Step 4: Capital gain eliminated only on FMV paths and only when
    // FMV exceeds basis (no negative "gain eliminated" reporting).
    let gain_eliminated = match rule {
        Section170eRule::LtcgPublicFmv | Section170eRule::LtcgPrivateFoundationQas => {
            (input.fmv - input.basis).max(Decimal::ZERO)
        }
        _ => Decimal::ZERO,
    };

    let path_label = match rule {
        Section170eRule::LtcgPublicFmv => "§170(b)(1)(C)(i) LTCG → public charity at FMV",
        Section170eRule::LtcgPublicBasisElect => {
            "§170(b)(1)(C)(iii) basis election → 50% AGI cap"
        }
        Section170eRule::LtcgPrivateFoundationQas => {
            "§170(e)(5) qualified appreciated stock → private foundation at FMV"
        }
        Section170eRule::LtcgPrivateFoundationBasis => {
            "§170(e)(1)(B)(ii) private foundation reduction → basis"
        }
        Section170eRule::StcgOrOrdinaryReduction => {
            "§170(e)(1)(A) STCG/ordinary reduction → basis"
        }
        Section170eRule::TangibleUnrelatedReduction => {
            "§170(e)(1)(B)(i) tangible personal unrelated use → basis"
        }
    };

    let note = format!(
        "{} — contribution ${} after reduction; {}% AGI cap = ${}; ${} deductible this year; ${} carries forward under §170(d)",
        path_label,
        contribution.round_dp(2),
        agi_bp / 100,
        agi_limit_dollars.round_dp(2),
        deductible.round_dp(2),
        carryforward.round_dp(2),
    );

    Section170eResult {
        contribution_amount_after_170e_reduction: contribution,
        agi_limit_basis_points: agi_bp,
        agi_limit_dollars,
        remaining_agi_budget: remaining,
        deductible_this_year: deductible,
        carryforward_to_next_year: carryforward,
        capital_gain_eliminated_no_tax: gain_eliminated,
        rule_path: rule,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section170eInput {
        Section170eInput {
            property_kind: PropertyKind::LongTermCapitalGain,
            fmv: dec!(100_000),
            basis: dec!(10_000),
            charity_type: CharityType::PublicCharity,
            agi: dec!(200_000),
            prior_year_carryover: Decimal::ZERO,
            other_charity_contributions_this_year: Decimal::ZERO,
            elect_basis_for_higher_limit: false,
            is_qualified_appreciated_stock: false,
        }
    }

    #[test]
    fn ltcg_public_fmv_canonical_path() {
        // $100k FMV / $10k basis LTCG stock → public charity. No
        // election. Contribution = FMV ($100k). 30% AGI cap = $60k.
        // $60k deductible, $40k carries forward, $90k gain escapes tax.
        let r = compute(&base());
        assert_eq!(r.rule_path, Section170eRule::LtcgPublicFmv);
        assert_eq!(r.contribution_amount_after_170e_reduction, dec!(100_000));
        assert_eq!(r.agi_limit_basis_points, 3000);
        assert_eq!(r.agi_limit_dollars, dec!(60_000));
        assert_eq!(r.deductible_this_year, dec!(60_000));
        assert_eq!(r.carryforward_to_next_year, dec!(40_000));
        assert_eq!(r.capital_gain_eliminated_no_tax, dec!(90_000));
    }

    #[test]
    fn ltcg_public_basis_election_lower_deduction_higher_cap() {
        // Same property, §170(b)(1)(C)(iii) election. Contribution drops
        // to $10k basis but cap jumps to 50% × $200k = $100k. Full $10k
        // deductible, zero carryforward, no gain eliminated.
        let mut i = base();
        i.elect_basis_for_higher_limit = true;
        let r = compute(&i);
        assert_eq!(r.rule_path, Section170eRule::LtcgPublicBasisElect);
        assert_eq!(r.contribution_amount_after_170e_reduction, dec!(10_000));
        assert_eq!(r.agi_limit_basis_points, 5000);
        assert_eq!(r.agi_limit_dollars, dec!(100_000));
        assert_eq!(r.deductible_this_year, dec!(10_000));
        assert_eq!(r.carryforward_to_next_year, Decimal::ZERO);
        assert_eq!(r.capital_gain_eliminated_no_tax, Decimal::ZERO);
    }

    #[test]
    fn stcg_property_reduced_to_basis_under_170e1a() {
        // STCG stock (held ≤ 1 year): §170(e)(1)(A) reduces to basis.
        // 50% AGI cap to public charity.
        let mut i = base();
        i.property_kind = PropertyKind::ShortTermCapitalGain;
        i.fmv = dec!(50_000);
        i.basis = dec!(30_000);
        let r = compute(&i);
        assert_eq!(r.rule_path, Section170eRule::StcgOrOrdinaryReduction);
        assert_eq!(r.contribution_amount_after_170e_reduction, dec!(30_000));
        assert_eq!(r.agi_limit_basis_points, 5000);
        // 50% × $200k = $100k cap > $30k contribution → fully deductible.
        assert_eq!(r.deductible_this_year, dec!(30_000));
        assert_eq!(r.capital_gain_eliminated_no_tax, Decimal::ZERO);
    }

    #[test]
    fn ordinary_income_property_same_reduction_as_stcg() {
        // Ordinary income property (inventory, §1221(a)(1)): same §170(e)
        // (1)(A) reduction, contribution = basis.
        let mut i = base();
        i.property_kind = PropertyKind::OrdinaryIncome;
        let r = compute(&i);
        assert_eq!(r.rule_path, Section170eRule::StcgOrOrdinaryReduction);
        assert_eq!(r.contribution_amount_after_170e_reduction, dec!(10_000));
    }

    #[test]
    fn ltcg_private_foundation_qas_fmv_deduction_at_20pct() {
        // §170(e)(5) — publicly-traded stock ≤ 10% of outstanding shares
        // → FMV deduction even to private foundation. 20% AGI cap.
        let mut i = base();
        i.charity_type = CharityType::PrivateFoundation;
        i.is_qualified_appreciated_stock = true;
        let r = compute(&i);
        assert_eq!(r.rule_path, Section170eRule::LtcgPrivateFoundationQas);
        assert_eq!(r.contribution_amount_after_170e_reduction, dec!(100_000));
        assert_eq!(r.agi_limit_basis_points, 2000);
        assert_eq!(r.agi_limit_dollars, dec!(40_000));
        // $100k contribution capped at $40k → $40k deductible, $60k CF.
        assert_eq!(r.deductible_this_year, dec!(40_000));
        assert_eq!(r.carryforward_to_next_year, dec!(60_000));
        assert_eq!(r.capital_gain_eliminated_no_tax, dec!(90_000));
    }

    #[test]
    fn ltcg_private_foundation_not_qas_reduced_to_basis() {
        // Closely-held stock, real estate, art → §170(e)(1)(B)(ii)
        // reduction. Contribution = basis. 20% AGI cap.
        let mut i = base();
        i.charity_type = CharityType::PrivateFoundation;
        i.is_qualified_appreciated_stock = false;
        let r = compute(&i);
        assert_eq!(r.rule_path, Section170eRule::LtcgPrivateFoundationBasis);
        assert_eq!(r.contribution_amount_after_170e_reduction, dec!(10_000));
        assert_eq!(r.agi_limit_basis_points, 2000);
        assert_eq!(r.capital_gain_eliminated_no_tax, Decimal::ZERO);
    }

    #[test]
    fn tangible_personal_unrelated_use_reduces_to_basis() {
        // §170(e)(1)(B)(i) — even LTCG art/collectibles donated to a
        // hospital (unrelated use) deducts at basis only. 50% cap public.
        let mut i = base();
        i.property_kind = PropertyKind::TangiblePersonalUnrelatedUse;
        let r = compute(&i);
        assert_eq!(r.rule_path, Section170eRule::TangibleUnrelatedReduction);
        assert_eq!(r.contribution_amount_after_170e_reduction, dec!(10_000));
        assert_eq!(r.agi_limit_basis_points, 5000);
    }

    #[test]
    fn tangible_unrelated_use_to_private_foundation_30pct_cap() {
        // Tangible unrelated use to a private foundation: 30% cap
        // (general non-LTCG cap for private foundation contributions),
        // contribution = basis.
        let mut i = base();
        i.property_kind = PropertyKind::TangiblePersonalUnrelatedUse;
        i.charity_type = CharityType::PrivateFoundation;
        let r = compute(&i);
        assert_eq!(r.agi_limit_basis_points, 3000);
    }

    #[test]
    fn prior_carryover_added_to_current_contribution_for_cap_check() {
        // Carry $50k from last year into a $100k LTCG-public donation.
        // Total claimed = $150k. Cap = $60k. $60k deductible this year,
        // $90k rolls to next year.
        let mut i = base();
        i.prior_year_carryover = dec!(50_000);
        let r = compute(&i);
        assert_eq!(r.deductible_this_year, dec!(60_000));
        assert_eq!(r.carryforward_to_next_year, dec!(90_000));
    }

    #[test]
    fn other_year_contributions_eat_into_remaining_budget() {
        // Already deducted $40k of other LTCG public-charity contribs
        // this year. Cap was $60k; remaining = $20k. New $100k contrib
        // → $20k deductible, $80k carries forward.
        let mut i = base();
        i.other_charity_contributions_this_year = dec!(40_000);
        let r = compute(&i);
        assert_eq!(r.remaining_agi_budget, dec!(20_000));
        assert_eq!(r.deductible_this_year, dec!(20_000));
        assert_eq!(r.carryforward_to_next_year, dec!(80_000));
    }

    #[test]
    fn zero_agi_everything_carries_forward() {
        // No AGI to deduct against → 0 deductible, full contribution
        // carries forward (5-year clock starts running).
        let mut i = base();
        i.agi = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.deductible_this_year, Decimal::ZERO);
        assert_eq!(r.carryforward_to_next_year, dec!(100_000));
    }

    #[test]
    fn contribution_exactly_at_cap_zero_carryforward() {
        // FMV $60k exactly at the 30% × $200k = $60k cap → fully
        // deductible, zero carryforward, zero remainder.
        let mut i = base();
        i.fmv = dec!(60_000);
        let r = compute(&i);
        assert_eq!(r.deductible_this_year, dec!(60_000));
        assert_eq!(r.carryforward_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn other_contributions_exceeding_cap_zero_remaining_budget() {
        // Other contribs $80k > cap $60k → remaining = 0 (clamped).
        // New $100k contribution all carries forward.
        let mut i = base();
        i.other_charity_contributions_this_year = dec!(80_000);
        let r = compute(&i);
        assert_eq!(r.remaining_agi_budget, Decimal::ZERO);
        assert_eq!(r.deductible_this_year, Decimal::ZERO);
        assert_eq!(r.carryforward_to_next_year, dec!(100_000));
    }

    #[test]
    fn fmv_below_basis_no_gain_eliminated_reports_zero_not_negative() {
        // Underwater stock: FMV $5k < basis $10k. LTCG public no
        // election → contribution = FMV ($5k). Gain "eliminated" should
        // be 0 (not negative), because there's no embedded gain.
        let mut i = base();
        i.fmv = dec!(5_000);
        let r = compute(&i);
        assert_eq!(r.contribution_amount_after_170e_reduction, dec!(5_000));
        assert_eq!(r.capital_gain_eliminated_no_tax, Decimal::ZERO);
    }

    #[test]
    fn basis_election_does_not_apply_to_stcg_property() {
        // Election is for LTCG only (§170(b)(1)(C)(iii)). Setting the
        // flag on STCG property has no effect — the §170(e)(1)(A) path
        // already reduces to basis, and the AGI cap is already 50%.
        let mut i = base();
        i.property_kind = PropertyKind::ShortTermCapitalGain;
        i.elect_basis_for_higher_limit = true; // ignored
        let r = compute(&i);
        assert_eq!(r.rule_path, Section170eRule::StcgOrOrdinaryReduction);
        assert_eq!(r.agi_limit_basis_points, 5000);
    }

    #[test]
    fn qas_flag_ignored_for_public_charity_path() {
        // The §170(e)(5) QAS carve-out only matters for the private-
        // foundation reduction rule. Setting the flag on a public-
        // charity donation should not change the rule path.
        let mut i = base();
        i.is_qualified_appreciated_stock = true;
        let r = compute(&i);
        assert_eq!(r.rule_path, Section170eRule::LtcgPublicFmv);
    }

    #[test]
    fn qas_with_basis_election_election_wins() {
        // Both QAS and basis-election flags set on a public-charity LTCG
        // donation. Election takes the path (basis at 50% cap) because
        // QAS only matters for private foundations.
        let mut i = base();
        i.is_qualified_appreciated_stock = true;
        i.elect_basis_for_higher_limit = true;
        let r = compute(&i);
        assert_eq!(r.rule_path, Section170eRule::LtcgPublicBasisElect);
        assert_eq!(r.contribution_amount_after_170e_reduction, dec!(10_000));
    }

    #[test]
    fn note_describes_rule_path_and_cap_pct() {
        let r = compute(&base());
        assert!(r.note.contains("§170(b)(1)(C)(i)"));
        assert!(r.note.contains("30%"));
        assert!(r.note.contains("§170(d)"));
    }

    #[test]
    fn note_for_qas_path_mentions_subsection_e5() {
        let mut i = base();
        i.charity_type = CharityType::PrivateFoundation;
        i.is_qualified_appreciated_stock = true;
        let r = compute(&i);
        assert!(r.note.contains("§170(e)(5)"));
        assert!(r.note.contains("20%"));
    }

    #[test]
    fn very_large_donation_no_precision_loss() {
        // $9.876543B basis donated to public charity with $20B AGI.
        // Decimal arithmetic must stay exact across the basis-points
        // multiplication and the percent-of-AGI calculation.
        let mut i = base();
        i.fmv = dec!(9_876_543_210.99);
        i.basis = dec!(1_234_567_890.12);
        i.agi = dec!(20_000_000_000);
        let r = compute(&i);
        assert_eq!(r.agi_limit_dollars, dec!(6_000_000_000));
        assert_eq!(r.contribution_amount_after_170e_reduction, dec!(9_876_543_210.99));
        assert_eq!(r.deductible_this_year, dec!(6_000_000_000));
        assert_eq!(
            r.carryforward_to_next_year,
            dec!(3_876_543_210.99) // contribution - cap
        );
    }

    #[test]
    fn multi_year_roll_picks_up_prior_carryforward_against_current_cap() {
        // Year 1 created $40k CF (from the canonical test). Year 2: same
        // $200k AGI, no new contribution, but prior CF = $40k. Total
        // claimed = $40k, cap = $60k, fully deductible.
        let mut i = base();
        i.fmv = Decimal::ZERO; // no new contribution
        i.basis = Decimal::ZERO;
        i.prior_year_carryover = dec!(40_000);
        let r = compute(&i);
        assert_eq!(r.deductible_this_year, dec!(40_000));
        assert_eq!(r.carryforward_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn carryforward_never_negative_under_strange_inputs() {
        // Pathological: negative other_year_contributions input. The
        // remaining_agi_budget calculation should still clamp at zero
        // and carryforward at zero. The max() clamps protect this.
        let mut i = base();
        i.other_charity_contributions_this_year = dec!(-100_000_000);
        i.fmv = Decimal::ZERO;
        i.basis = Decimal::ZERO;
        let r = compute(&i);
        assert!(r.carryforward_to_next_year >= Decimal::ZERO);
        assert!(r.deductible_this_year >= Decimal::ZERO);
    }

    #[test]
    fn private_foundation_stcg_uses_30pct_cap() {
        // STCG to private foundation: §170(e)(1)(A) reduction +
        // private-foundation 30% cap (not the 20% LTCG cap).
        let mut i = base();
        i.property_kind = PropertyKind::ShortTermCapitalGain;
        i.charity_type = CharityType::PrivateFoundation;
        let r = compute(&i);
        assert_eq!(r.agi_limit_basis_points, 3000);
        assert_eq!(r.contribution_amount_after_170e_reduction, dec!(10_000));
    }
}
