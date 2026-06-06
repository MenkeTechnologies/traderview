//! IRC § 4940 — Excise tax based on investment income
//! of private foundations. Annual excise tax imposed on
//! every domestic tax-exempt private foundation (except
//! § 4940(d) exempt operating foundations) at a single
//! flat rate of 1.39% on net investment income, effective
//! for tax years beginning after December 20, 2019.
//!
//! Direct companion to section_4941 (PF self-dealing —
//! iter 468), section_4958 (intermediate sanctions —
//! iter 466), section_4960 (ATEO executive comp 21% —
//! iter 464), section_4973 (excess contribution excise —
//! iter 442), section_4974 (RMD excise — iter 436),
//! section_4975 (qualified plan prohibited transactions —
//! iter 434), section_4980 (employer reversion — iter
//! 460).
//!
//! TWO REGIMES:
//!
//! Pre-Dec-20-2019 regime: § 4940(a) imposed 2% on net
//! investment income with reduction to 1% under former
//! § 4940(e) if certain distribution-requirement tests
//! met (qualifying distributions >= average percentage
//! payout for 5-year base period plus 1% of NII).
//!
//! Post-Dec-20-2019 regime: Further Consolidated
//! Appropriations Act, 2020, Pub. L. 116-94 (signed
//! December 20, 2019) AMENDED § 4940(a) to a single flat
//! rate of 1.39% on net investment income for tax years
//! beginning AFTER December 20, 2019, and REPEALED former
//! § 4940(e) entirely. Calendar-year PFs effectively
//! transitioned January 1, 2020.
//!
//! Net investment income per § 4940(c):
//! 1. § 4940(c)(1) GROSS INVESTMENT INCOME: interest,
//!    dividends, rents, payments with respect to
//!    securities loans, royalties, and similar income
//!    from sources other than active conduct of an
//!    exempt purpose
//! 2. § 4940(c)(2) ALLOWABLE DEDUCTIONS: ordinary and
//!    necessary expenses PAID OR INCURRED for production
//!    or collection of gross investment income or
//!    management/conservation of property held for
//!    production of such income
//! 3. § 4940(c)(3) DEFINITIONS AND SPECIAL RULES: capital
//!    gains net of capital losses on property used for
//!    production of investment income
//! 4. § 4940(c)(4)(A) CAPITAL GAINS INCLUSION: net
//!    capital gain from sale or other disposition of
//!    property used for production of interest +
//!    dividends + rents + royalties included; gains on
//!    property used for exempt purpose EXCLUDED
//!
//! Exempt operating foundations per § 4940(d) (FOUR-PART
//! TEST):
//! 1. § 4940(d)(2)(A) — has been publicly supported (or
//!    operated as supporting organization) for at least
//!    10 prior taxable years before such year
//! 2. § 4940(d)(2)(B) — governing body broadly
//!    representative of general public, with not more
//!    than 25% of voting members being disqualified
//!    persons under § 4946
//! 3. § 4940(d)(2)(C) — operating-foundation status
//!    under § 4942(j)(3) for taxable year
//! 4. § 4940(d)(2)(D) — at no time during taxable year
//!    has an officer who is a disqualified individual
//!    appointed by disqualified persons
//!
//! Foreign private foundations operate under separate
//! regime per § 4948 at 4% rate on gross investment
//! income from US sources, not affected by 2019
//! amendment.
//!
//! Section 501(c)(3) PUBLIC CHARITIES are NOT subject to
//! § 4940 at all — the tax applies exclusively to PRIVATE
//! FOUNDATIONS.
//!
//! Trader-foundation critical because (1) every domestic
//! PF must compute § 4940 NII excise tax annually and
//! pay quarterly estimates per § 6655; (2) the 2019
//! simplification eliminated the strategic incentive to
//! qualify for 1% reduction by managing distribution
//! timing — under the prior regime, foundations
//! sometimes deliberately underdistributed to push gains
//! into later years; (3) capital gains on appreciated
//! securities donated to PF generate NII excise tax when
//! sold, but qualified dividends and tax-exempt municipal
//! bond interest reduce gross investment income; (4)
//! § 4940(d) exempt operating foundation status is
//! valuable but technically demanding — most family
//! foundations do NOT qualify; (5) PRG (program-related
//! investment) returns under § 4944(c) are EXCLUDED from
//! NII; (6) substitution of grant payments via DAFs
//! (donor-advised funds) does NOT shift § 4940 base
//! since the PF is still the foundation paying tax on
//! its own NII.
//!
//! Distinction from § 4941 (iter 468): § 4940 is the
//! ANNUAL EXCISE TAX on NII of every PF; § 4941 is the
//! per-act PUNITIVE EXCISE TAX on specific self-dealing
//! transactions. § 4940 is computed and paid annually
//! regardless of any wrongdoing; § 4941 is triggered
//! only by specific acts.
//!
//! Distinction from § 4958 (iter 466): § 4958 applies
//! to PUBLIC charities + § 501(c)(4) + § 501(c)(29);
//! PFs are EXCLUDED from § 4958 per § 4958(e) and instead
//! face § 4941 self-dealing rules PLUS § 4940 annual NII
//! excise.
//!
//! Authority: 26 U.S.C. § 4940; § 4940(a); § 4940(b);
//! § 4940(c)(1); § 4940(c)(2); § 4940(c)(3); § 4940(c)(4);
//! § 4940(c)(4)(A); § 4940(d); § 4940(d)(2)(A);
//! § 4940(d)(2)(B); § 4940(d)(2)(C); § 4940(d)(2)(D);
//! § 4942(j)(3); § 4944(c); § 4946; § 4948; § 6655;
//! 26 C.F.R. § 53.4940-1; 26 C.F.R. § 53.4940-2;
//! Tax Reform Act of 1969, Pub. L. 91-172 (Dec. 30,
//! 1969) — original § 4940 enactment; Further
//! Consolidated Appropriations Act, 2020, Pub. L. 116-94
//! (signed December 20, 2019) — 1.39% flat-rate
//! simplification and repeal of former § 4940(e).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationStatus {
    PrivateFoundation,
    ExemptOperatingFoundationSection4940d,
    ForeignPrivateFoundationSection4948,
    PublicCharity,
    NonExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxYearRegime {
    PreDec20_2019,
    PostDec20_2019,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub organization_status: OrganizationStatus,
    pub tax_year_regime: TaxYearRegime,
    pub gross_interest_dividends_cents: u64,
    pub gross_rents_royalties_cents: u64,
    pub gross_securities_loan_payments_cents: u64,
    pub net_capital_gain_investment_property_cents: u64,
    pub allowable_deductions_cents: u64,
    pub pre_2019_qualifies_for_one_percent_reduction: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    ExemptOperatingFoundation,
    ExciseTaxOwed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub gross_investment_income_cents: u64,
    pub net_investment_income_cents: u64,
    pub excise_tax_rate_basis_points: u64,
    pub excise_tax_cents: u64,
    pub notes: Vec<String>,
}

pub const FLAT_RATE_BASIS_POINTS: u64 = 139; // 1.39%
pub const PRE_2019_STANDARD_BASIS_POINTS: u64 = 200; // 2.00%
pub const PRE_2019_REDUCED_BASIS_POINTS: u64 = 100; // 1.00%
pub const FOREIGN_PF_BASIS_POINTS: u64 = 400; // 4.00%

pub type Section4940Input = Input;
pub type Section4940Result = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "§ 4940(a) imposes annual excise tax on net investment income of every domestic tax-exempt private foundation. Post-Dec-20-2019 regime: single flat rate of 1.39% per Further Consolidated Appropriations Act, 2020, Pub. L. 116-94 (signed December 20, 2019). Pre-Dec-20-2019 regime: 2% standard / 1% reduced under former § 4940(e) (REPEALED by Pub. L. 116-94).".to_string(),
        "Net investment income per § 4940(c): gross investment income (§ 4940(c)(1) — interest + dividends + rents + securities-loan payments + royalties) plus net capital gain from sale of investment property (§ 4940(c)(4)(A)) minus allowable deductions (§ 4940(c)(2) — ordinary and necessary expenses paid or incurred for production or collection of gross investment income or management/conservation of property held for production of such income).".to_string(),
        "Exempt operating foundations per § 4940(d) — four-part test: (A) publicly supported for at least 10 prior taxable years; (B) governing body broadly representative with not more than 25% disqualified persons under § 4946; (C) operating-foundation status under § 4942(j)(3); (D) no officer who is disqualified individual appointed by disqualified persons.".to_string(),
        "Foreign private foundations operate under § 4948 separate regime at 4% rate on gross investment income from US sources, not affected by 2019 simplification. § 501(c)(3) public charities are NOT subject to § 4940.".to_string(),
        "Capital gain inclusion per § 4940(c)(4)(A): net capital gain from sale or other disposition of property used for production of interest + dividends + rents + royalties INCLUDED in NII; gains on property used for exempt purpose EXCLUDED. Capital losses on investment property allowed under § 4940(c)(3) to offset gains.".to_string(),
        "Distinction from § 4941 (iter 468): § 4940 is ANNUAL excise on NII regardless of wrongdoing; § 4941 is per-act PUNITIVE excise on self-dealing transactions. Distinction from § 4958 (iter 466): § 4958 applies to PUBLIC charities + § 501(c)(4) + § 501(c)(29); PFs EXCLUDED from § 4958 per § 4958(e) and instead face § 4941 + § 4940.".to_string(),
        "Companion: section_4941 (iter 468), section_4958 (iter 466), section_4960 (iter 464), section_4973 (iter 442), section_4974 (iter 436), section_4975 (iter 434), section_4980 (iter 460); estimated tax payment due quarterly per § 6655.".to_string(),
    ];

    let rate_bps = match (input.organization_status, input.tax_year_regime) {
        (OrganizationStatus::PublicCharity, _) | (OrganizationStatus::NonExempt, _) => {
            let mut n = notes;
            n.push("Organization is not a private foundation — § 4940 does not apply.".to_string());
            return Output {
                severity: Severity::NotApplicable,
                gross_investment_income_cents: 0,
                net_investment_income_cents: 0,
                excise_tax_rate_basis_points: 0,
                excise_tax_cents: 0,
                notes: n,
            };
        }
        (OrganizationStatus::ExemptOperatingFoundationSection4940d, _) => {
            let mut n = notes;
            n.push("Exempt operating foundation under § 4940(d) — exempt from § 4940 annual NII excise tax.".to_string());
            return Output {
                severity: Severity::ExemptOperatingFoundation,
                gross_investment_income_cents: 0,
                net_investment_income_cents: 0,
                excise_tax_rate_basis_points: 0,
                excise_tax_cents: 0,
                notes: n,
            };
        }
        (OrganizationStatus::ForeignPrivateFoundationSection4948, _) => FOREIGN_PF_BASIS_POINTS,
        (OrganizationStatus::PrivateFoundation, TaxYearRegime::PostDec20_2019) => {
            FLAT_RATE_BASIS_POINTS
        }
        (OrganizationStatus::PrivateFoundation, TaxYearRegime::PreDec20_2019) => {
            if input.pre_2019_qualifies_for_one_percent_reduction {
                PRE_2019_REDUCED_BASIS_POINTS
            } else {
                PRE_2019_STANDARD_BASIS_POINTS
            }
        }
    };

    let gross_investment_income = input
        .gross_interest_dividends_cents
        .saturating_add(input.gross_rents_royalties_cents)
        .saturating_add(input.gross_securities_loan_payments_cents)
        .saturating_add(input.net_capital_gain_investment_property_cents);

    let net_investment_income =
        gross_investment_income.saturating_sub(input.allowable_deductions_cents);

    let excise_tax = net_investment_income
        .saturating_mul(rate_bps)
        .checked_div(10_000)
        .unwrap_or(0);

    let mut n = notes;
    n.push(format!(
        "Computed § 4940 excise tax: ${}.{:02} (rate {} bps × NII ${}.{:02}).",
        excise_tax / 100,
        excise_tax % 100,
        rate_bps,
        net_investment_income / 100,
        net_investment_income % 100
    ));

    Output {
        severity: Severity::ExciseTaxOwed,
        gross_investment_income_cents: gross_investment_income,
        net_investment_income_cents: net_investment_income,
        excise_tax_rate_basis_points: rate_bps,
        excise_tax_cents: excise_tax,
        notes: n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            organization_status: OrganizationStatus::PrivateFoundation,
            tax_year_regime: TaxYearRegime::PostDec20_2019,
            gross_interest_dividends_cents: 500_000_00,
            gross_rents_royalties_cents: 100_000_00,
            gross_securities_loan_payments_cents: 0,
            net_capital_gain_investment_property_cents: 400_000_00,
            allowable_deductions_cents: 100_000_00,
            pre_2019_qualifies_for_one_percent_reduction: false,
        }
    }

    #[test]
    fn public_charity_not_subject_to_4940() {
        let mut i = baseline();
        i.organization_status = OrganizationStatus::PublicCharity;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn non_exempt_not_applicable() {
        let mut i = baseline();
        i.organization_status = OrganizationStatus::NonExempt;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn exempt_operating_foundation_4940d() {
        let mut i = baseline();
        i.organization_status = OrganizationStatus::ExemptOperatingFoundationSection4940d;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptOperatingFoundation);
        assert_eq!(out.excise_tax_cents, 0);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4940(d)"));
    }

    #[test]
    fn post_2019_flat_rate_1_point_39_pct() {
        let i = baseline();
        // GII = $500K + $100K + $0 + $400K = $1M
        // NII = $1M - $100K = $900K
        // Tax = 1.39% × $900K = $12,510
        let out = check(&i);
        assert_eq!(out.gross_investment_income_cents, 1_000_000_00);
        assert_eq!(out.net_investment_income_cents, 900_000_00);
        assert_eq!(out.excise_tax_rate_basis_points, 139);
        assert_eq!(out.excise_tax_cents, 12_510_00);
    }

    #[test]
    fn pre_2019_standard_2_pct() {
        let mut i = baseline();
        i.tax_year_regime = TaxYearRegime::PreDec20_2019;
        // NII = $900K × 2% = $18K
        let out = check(&i);
        assert_eq!(out.excise_tax_rate_basis_points, 200);
        assert_eq!(out.excise_tax_cents, 18_000_00);
    }

    #[test]
    fn pre_2019_reduced_1_pct() {
        let mut i = baseline();
        i.tax_year_regime = TaxYearRegime::PreDec20_2019;
        i.pre_2019_qualifies_for_one_percent_reduction = true;
        // NII = $900K × 1% = $9K
        let out = check(&i);
        assert_eq!(out.excise_tax_rate_basis_points, 100);
        assert_eq!(out.excise_tax_cents, 9_000_00);
    }

    #[test]
    fn foreign_pf_4_pct_under_4948() {
        let mut i = baseline();
        i.organization_status = OrganizationStatus::ForeignPrivateFoundationSection4948;
        // NII = $900K × 4% = $36K
        let out = check(&i);
        assert_eq!(out.excise_tax_rate_basis_points, 400);
        assert_eq!(out.excise_tax_cents, 36_000_00);
    }

    #[test]
    fn allowable_deductions_reduce_nii() {
        let mut i = baseline();
        i.allowable_deductions_cents = 500_000_00; // larger deduction
                                                   // GII $1M − $500K = $500K NII × 1.39% = $6,950
        let out = check(&i);
        assert_eq!(out.net_investment_income_cents, 500_000_00);
        assert_eq!(out.excise_tax_cents, 6_950_00);
    }

    #[test]
    fn capital_gain_included_in_nii() {
        let mut i = baseline();
        i.net_capital_gain_investment_property_cents = 1_000_000_00; // $1M gain
                                                                     // GII = $500K + $100K + $0 + $1M = $1.6M
                                                                     // NII = $1.6M − $100K = $1.5M
                                                                     // Tax = $1.5M × 1.39% = $20,850
        let out = check(&i);
        assert_eq!(out.gross_investment_income_cents, 1_600_000_00);
        assert_eq!(out.net_investment_income_cents, 1_500_000_00);
        assert_eq!(out.excise_tax_cents, 20_850_00);
    }

    #[test]
    fn zero_investment_income_zero_tax() {
        let i = Input {
            gross_interest_dividends_cents: 0,
            gross_rents_royalties_cents: 0,
            gross_securities_loan_payments_cents: 0,
            net_capital_gain_investment_property_cents: 0,
            allowable_deductions_cents: 0,
            ..baseline()
        };
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        assert_eq!(out.excise_tax_cents, 0);
    }

    #[test]
    fn deductions_exceeding_gii_clamp_to_zero() {
        let mut i = baseline();
        i.allowable_deductions_cents = 5_000_000_00; // huge
        let out = check(&i);
        assert_eq!(out.net_investment_income_cents, 0);
        assert_eq!(out.excise_tax_cents, 0);
    }

    #[test]
    fn securities_loan_payments_included() {
        let mut i = baseline();
        i.gross_securities_loan_payments_cents = 50_000_00;
        // GII now $1.05M; NII $950K × 1.39% = $13,205
        let out = check(&i);
        assert_eq!(out.gross_investment_income_cents, 1_050_000_00);
        assert_eq!(out.excise_tax_cents, 13_205_00);
    }

    #[test]
    fn boundary_rate_change_post_vs_pre_2019() {
        let mut i = baseline();
        // Same financials, different regimes
        i.tax_year_regime = TaxYearRegime::PostDec20_2019;
        let post = check(&i).excise_tax_cents;
        i.tax_year_regime = TaxYearRegime::PreDec20_2019;
        let pre_standard = check(&i).excise_tax_cents;
        i.pre_2019_qualifies_for_one_percent_reduction = true;
        let pre_reduced = check(&i).excise_tax_cents;
        // Pre-2019 standard 2% > Post-2019 1.39% > Pre-2019 reduced 1%
        assert!(pre_standard > post);
        assert!(post > pre_reduced);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4940(a)"));
        assert!(joined.contains("§ 4940(c)"));
        assert!(joined.contains("§ 4940(c)(1)"));
        assert!(joined.contains("§ 4940(c)(2)"));
        assert!(joined.contains("§ 4940(c)(4)(A)"));
        assert!(joined.contains("§ 4940(d)"));
        assert!(joined.contains("§ 4942(j)(3)"));
        assert!(joined.contains("§ 4946"));
        assert!(joined.contains("§ 4948"));
        assert!(joined.contains("§ 4940(e)"));
        assert!(joined.contains("Pub. L. 116-94"));
        assert!(joined.contains("December 20, 2019"));
        assert!(joined.contains("§ 6655"));
    }

    #[test]
    fn note_pins_two_regimes() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("1.39%"));
        assert!(joined.contains("2%"));
        assert!(joined.contains("1%"));
        assert!(joined.contains("REPEALED"));
    }

    #[test]
    fn note_pins_four_part_operating_foundation_test() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("publicly supported for at least 10"));
        assert!(joined.contains("25% disqualified"));
        assert!(joined.contains("§ 4942(j)(3)"));
        assert!(joined.contains("officer who is disqualified"));
    }

    #[test]
    fn note_pins_foreign_pf_4948_4_percent() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("4%"));
        assert!(joined.contains("§ 4948"));
        assert!(joined.contains("US sources"));
    }

    #[test]
    fn note_pins_capital_gain_inclusion_rule() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("net capital gain"));
        assert!(joined.contains("exempt purpose EXCLUDED"));
    }

    #[test]
    fn note_pins_4941_4958_distinctions() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4941 (iter 468)"));
        assert!(joined.contains("§ 4958 (iter 466)"));
        assert!(joined.contains("ANNUAL"));
        assert!(joined.contains("per-act"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("section_4941"));
        assert!(joined.contains("section_4958"));
        assert!(joined.contains("section_4960"));
        assert!(joined.contains("§ 6655"));
    }

    #[test]
    fn truth_table_five_cells() {
        // Cell 1: Domestic PF post-2019 = 1.39%
        let c1 = check(&Input {
            organization_status: OrganizationStatus::PrivateFoundation,
            tax_year_regime: TaxYearRegime::PostDec20_2019,
            ..baseline()
        });
        assert_eq!(c1.excise_tax_rate_basis_points, 139);

        // Cell 2: Domestic PF pre-2019 standard = 2%
        let c2 = check(&Input {
            organization_status: OrganizationStatus::PrivateFoundation,
            tax_year_regime: TaxYearRegime::PreDec20_2019,
            pre_2019_qualifies_for_one_percent_reduction: false,
            ..baseline()
        });
        assert_eq!(c2.excise_tax_rate_basis_points, 200);

        // Cell 3: Domestic PF pre-2019 reduced = 1%
        let c3 = check(&Input {
            organization_status: OrganizationStatus::PrivateFoundation,
            tax_year_regime: TaxYearRegime::PreDec20_2019,
            pre_2019_qualifies_for_one_percent_reduction: true,
            ..baseline()
        });
        assert_eq!(c3.excise_tax_rate_basis_points, 100);

        // Cell 4: Foreign PF = 4%
        let c4 = check(&Input {
            organization_status: OrganizationStatus::ForeignPrivateFoundationSection4948,
            ..baseline()
        });
        assert_eq!(c4.excise_tax_rate_basis_points, 400);

        // Cell 5: Exempt operating foundation = 0%
        let c5 = check(&Input {
            organization_status: OrganizationStatus::ExemptOperatingFoundationSection4940d,
            ..baseline()
        });
        assert_eq!(c5.severity, Severity::ExemptOperatingFoundation);
    }

    #[test]
    fn post_2019_uniquely_repeals_e_invariant() {
        let mut i = baseline();
        i.tax_year_regime = TaxYearRegime::PostDec20_2019;
        // Post-2019: pre_2019_qualifies flag IGNORED — rate always 1.39%
        i.pre_2019_qualifies_for_one_percent_reduction = true;
        let post = check(&i);
        assert_eq!(post.excise_tax_rate_basis_points, 139);
        i.pre_2019_qualifies_for_one_percent_reduction = false;
        let post2 = check(&i);
        assert_eq!(post2.excise_tax_rate_basis_points, 139);
    }

    #[test]
    fn pre_2019_qualifies_flag_drives_rate_choice() {
        let mut i = baseline();
        i.tax_year_regime = TaxYearRegime::PreDec20_2019;
        i.pre_2019_qualifies_for_one_percent_reduction = false;
        assert_eq!(check(&i).excise_tax_rate_basis_points, 200);
        i.pre_2019_qualifies_for_one_percent_reduction = true;
        assert_eq!(check(&i).excise_tax_rate_basis_points, 100);
    }

    #[test]
    fn saturating_arithmetic_overflow_defense() {
        let i = Input {
            gross_interest_dividends_cents: u64::MAX,
            gross_rents_royalties_cents: u64::MAX,
            gross_securities_loan_payments_cents: u64::MAX,
            net_capital_gain_investment_property_cents: u64::MAX,
            allowable_deductions_cents: 0,
            ..baseline()
        };
        let out = check(&i);
        // No panic
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
    }

    #[test]
    fn boundary_one_cent_nii() {
        let i = Input {
            gross_interest_dividends_cents: 1,
            gross_rents_royalties_cents: 0,
            gross_securities_loan_payments_cents: 0,
            net_capital_gain_investment_property_cents: 0,
            allowable_deductions_cents: 0,
            ..baseline()
        };
        let out = check(&i);
        // 1 cent × 139 bps / 10000 = 0 (integer division)
        assert_eq!(out.excise_tax_cents, 0);
    }

    #[test]
    fn realistic_family_foundation_post_2019_calculation() {
        // $5M endowment, ~3% yield + ~2% appreciation, $50K expenses
        // GII = $150K interest/dividends + $0 rents + $100K capital gain = $250K
        // NII = $250K - $50K = $200K
        // Tax = $200K × 1.39% = $2,780
        let i = Input {
            organization_status: OrganizationStatus::PrivateFoundation,
            tax_year_regime: TaxYearRegime::PostDec20_2019,
            gross_interest_dividends_cents: 150_000_00,
            gross_rents_royalties_cents: 0,
            gross_securities_loan_payments_cents: 0,
            net_capital_gain_investment_property_cents: 100_000_00,
            allowable_deductions_cents: 50_000_00,
            pre_2019_qualifies_for_one_percent_reduction: false,
        };
        let out = check(&i);
        assert_eq!(out.net_investment_income_cents, 200_000_00);
        assert_eq!(out.excise_tax_cents, 2_780_00);
    }

    #[test]
    fn note_pins_excise_tax_computed_amount() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Computed § 4940 excise tax"));
        assert!(joined.contains("139 bps"));
    }
}
