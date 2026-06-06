//! IRC § 302 distributions in redemption of stock.
//!
//! § 302 determines whether a corporation's redemption of its own stock is treated as
//! a § 1001 SALE OR EXCHANGE (producing capital gain or loss) or as a § 301
//! DISTRIBUTION (producing ordinary dividend income to the extent of E&P + basis
//! recovery + capital gain on excess). § 302(a) treats a redemption as sale or
//! exchange if any of the four § 302(b) tests is satisfied:
//!
//! § 302(b)(1) NOT ESSENTIALLY EQUIVALENT TO A DIVIDEND (NEED-test): facts-and-
//! circumstances inquiry asking whether the redemption produces a "meaningful
//! reduction" in the shareholder's interest (United States v. Davis, 397 U.S. 301
//! (1970)). Highly subjective; minority redemptions sometimes qualify when board
//! control or veto rights are affected.
//!
//! § 302(b)(2) SUBSTANTIALLY DISPROPORTIONATE REDEMPTION (mechanical 50/80 test):
//!   - Post-redemption: shareholder owns LESS THAN 50% of total combined voting
//!     power; AND
//!   - Post-redemption common-voting interest is LESS THAN 80% of pre-redemption
//!     common-voting interest; AND
//!   - Post-redemption common-voting-AND-non-voting interest is LESS THAN 80% of
//!     pre-redemption common-voting-AND-non-voting interest.
//!
//! § 302(b)(3) COMPLETE TERMINATION: redemption terminates ALL of the shareholder's
//! interest in the corporation. Family attribution under § 318(a)(1) may be
//! WAIVED if (A) shareholder has no interest other than as creditor immediately
//! after distribution (including officer, director, employee), (B) shareholder
//! acquires no such interest within 10 years (other than by bequest or
//! inheritance), and (C) shareholder files agreement with IRS to notify of any
//! reacquisition and retain records (Form attached to return).
//!
//! § 302(b)(4) PARTIAL LIQUIDATION (non-corporate shareholder): redemption is part
//! of a partial liquidation defined in § 302(e) — corporation conducts active
//! trade or business for 5+ years and distributes proceeds of a discontinued
//! business segment.
//!
//! § 302(c)(1) ATTRIBUTION RULES: § 318(a) attribution rules apply to determine
//! ownership for § 302 purposes. Family + entity + option attribution may convert
//! what appears to be a complete-termination redemption into a § 301 dividend.
//!
//! § 302(d) DEFAULT § 301 TREATMENT: if redemption fails ALL four § 302(b) tests,
//! the redemption is treated as a § 301 distribution (dividend up to E&P + basis
//! recovery + capital gain on excess).
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/302
//! - law.cornell.edu/cfr/text/26/1.302-3
//! - irc.bloombergtax.com/public/uscode/doc/irc/section_302
//! - thetaxadviser.com/issues/2018/oct/s-corporation-redemptions-secs-302-301/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedemptionTestPath {
    /// § 302(b)(2) substantially disproportionate (mechanical 50/80 test).
    SubstantiallyDisproportionateSection302B2,
    /// § 302(b)(3) complete termination of shareholder's interest.
    CompleteTerminationSection302B3,
    /// § 302(b)(1) not essentially equivalent to a dividend (NEED).
    NotEssentiallyEquivalentSection302B1,
    /// § 302(b)(4) partial liquidation (non-corporate shareholder).
    PartialLiquidationSection302B4,
    /// No § 302(b) test asserted; default § 301 treatment.
    NoSection302BTestAssertedDefaultDistribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributionStatus {
    /// § 318(a) attribution applies and creates additional constructive ownership.
    AttributionAppliesIncreasesOwnership,
    /// § 318(a) attribution does not apply (no qualifying relationships).
    AttributionDoesNotApply,
    /// § 302(c)(2) family-attribution WAIVER filed for complete-termination
    /// redemption.
    Section302C2FamilyAttributionWaiverFiled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub redemption_test_path: RedemptionTestPath,
    pub attribution_status: AttributionStatus,
    /// Pre-redemption percentage interest in voting stock (basis points).
    pub pre_redemption_voting_pct_bps: u32,
    /// Post-redemption percentage interest in voting stock (basis points).
    pub post_redemption_voting_pct_bps: u32,
    pub property_received_cents: u64,
    pub shareholder_basis_in_redeemed_stock_cents: u64,
    pub acquiring_corp_eep_cents: u64,
}

pub type Section302RedemptionTestInput = Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Section302BTestSatisfiedSaleOrExchangeTreatment,
    Section302B2SubstantiallyDisproportionate50_80TestFailed,
    Section302B3CompleteTerminationWithAttributionWaiver,
    Section302B3CompleteTerminationAttributionDefeatsFailedWaiver,
    Section302DDefaultDistributionTreatmentSection301,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub treated_as_sale_or_exchange: bool,
    pub capital_gain_or_loss_cents: u64,
    pub dividend_treatment_cents: u64,
    pub note: String,
}

pub type Section302RedemptionTestOutput = Output;
pub type Section302RedemptionTestResult = Output;

const SECTION_302B2_POST_REDEMPTION_VOTING_CAP_BPS: u32 = 5_000;
const SECTION_302B2_PRE_TO_POST_DROP_FACTOR_BPS: u32 = 8_000;
const SECTION_302C2_TEN_YEAR_REACQUISITION_BAR_YEARS: u32 = 10;

#[must_use]
pub fn check(input: &Input) -> Output {
    match input.redemption_test_path {
        RedemptionTestPath::SubstantiallyDisproportionateSection302B2 => {
            let post_below_50 =
                input.post_redemption_voting_pct_bps < SECTION_302B2_POST_REDEMPTION_VOTING_CAP_BPS;
            let required_post_for_80 = u32::try_from(
                u128::from(input.pre_redemption_voting_pct_bps)
                    .saturating_mul(u128::from(SECTION_302B2_PRE_TO_POST_DROP_FACTOR_BPS))
                    .saturating_div(10_000),
            )
            .unwrap_or(u32::MAX);
            let dropped_below_80 = input.post_redemption_voting_pct_bps < required_post_for_80;
            if post_below_50 && dropped_below_80 {
                return sale_exchange_output(
                    input,
                    Severity::Section302BTestSatisfiedSaleOrExchangeTreatment,
                    "§ 302(b)(2) SUBSTANTIALLY DISPROPORTIONATE REDEMPTION test satisfied. \
                     Post-redemption voting interest below 50% AND post-redemption interest \
                     less than 80% of pre-redemption interest. Redemption treated as § 1001 \
                     sale or exchange producing capital gain or loss. Coordinates with § 318 \
                     constructive ownership (iter 552), § 1222 capital-gain holding period, \
                     § 1(h)(11) qualified-dividend rate where applicable for dividend-treated \
                     redemptions.",
                );
            }
            Output {
                severity: Severity::Section302B2SubstantiallyDisproportionate50_80TestFailed,
                treated_as_sale_or_exchange: false,
                capital_gain_or_loss_cents: 0,
                dividend_treatment_cents: input
                    .property_received_cents
                    .min(input.acquiring_corp_eep_cents),
                note: format!(
                    "§ 302(b)(2) substantially disproportionate test FAILED. Required: \
                     post-redemption voting < 50% (actual {} bps, threshold {} bps) AND \
                     post-redemption < 80% of pre-redemption (actual {} bps, required {} \
                     bps). Default § 301 distribution treatment applies — ordinary dividend \
                     income to extent of acquiring-corp E&P (${}), basis recovery, then \
                     capital gain on excess. Consider whether other § 302(b) tests apply \
                     (complete termination + § 302(c)(2) waiver; not-essentially-equivalent \
                     under § 302(b)(1) Davis meaningful-reduction analysis; partial \
                     liquidation under § 302(b)(4)).",
                    input.post_redemption_voting_pct_bps,
                    SECTION_302B2_POST_REDEMPTION_VOTING_CAP_BPS,
                    input.post_redemption_voting_pct_bps,
                    required_post_for_80,
                    input.acquiring_corp_eep_cents / 100
                ),
            }
        }
        RedemptionTestPath::CompleteTerminationSection302B3 => match input.attribution_status {
            AttributionStatus::Section302C2FamilyAttributionWaiverFiled => sale_exchange_output(
                input,
                Severity::Section302B3CompleteTerminationWithAttributionWaiver,
                "§ 302(b)(3) COMPLETE TERMINATION test satisfied with § 302(c)(2) family-\
                     attribution waiver. Shareholder: (A) has no interest other than as \
                     creditor immediately after distribution (no officer/director/employee \
                     status), (B) acquires no such interest within 10-year reacquisition bar \
                     (other than by bequest or inheritance), and (C) filed agreement with \
                     IRS to notify of any reacquisition and retain records. § 318(a)(1) \
                     family attribution waived; redemption treated as § 1001 sale or \
                     exchange producing capital gain or loss.",
            ),
            AttributionStatus::AttributionDoesNotApply => sale_exchange_output(
                input,
                Severity::Section302BTestSatisfiedSaleOrExchangeTreatment,
                "§ 302(b)(3) COMPLETE TERMINATION test satisfied: shareholder has no \
                     remaining direct OR constructive interest in the corporation. \
                     Redemption treated as § 1001 sale or exchange producing capital gain \
                     or loss.",
            ),
            AttributionStatus::AttributionAppliesIncreasesOwnership => Output {
                severity: Severity::Section302B3CompleteTerminationAttributionDefeatsFailedWaiver,
                treated_as_sale_or_exchange: false,
                capital_gain_or_loss_cents: 0,
                dividend_treatment_cents: input
                    .property_received_cents
                    .min(input.acquiring_corp_eep_cents),
                note: format!(
                    "§ 302(b)(3) complete termination DEFEATED by § 318(a) attribution. \
                             Shareholder still owns constructive interest through family + \
                             entity + option attribution. § 302(c)(2) waiver NOT filed (or \
                             waiver conditions not met). Required: (A) no interest other than \
                             as creditor immediately after distribution, (B) no reacquisition \
                             within {SECTION_302C2_TEN_YEAR_REACQUISITION_BAR_YEARS}-year \
                             window (other than by bequest or inheritance), (C) signed IRS \
                             notification agreement. Default § 301 distribution treatment \
                             applies — ordinary dividend up to acquiring-corp E&P (${}).",
                    input.acquiring_corp_eep_cents / 100
                ),
            },
        },
        RedemptionTestPath::NotEssentiallyEquivalentSection302B1 => sale_exchange_output(
            input,
            Severity::Section302BTestSatisfiedSaleOrExchangeTreatment,
            "§ 302(b)(1) NOT ESSENTIALLY EQUIVALENT TO A DIVIDEND test asserted. Facts-and-\
             circumstances inquiry under United States v. Davis, 397 U.S. 301 (1970) — \
             requires MEANINGFUL REDUCTION in shareholder's interest. Subjective standard; \
             minority redemptions occasionally qualify when board control or veto rights \
             are affected. Document the specific reduction (voting-percent decrease, board-\
             seat loss, dividend-right loss, liquidation-preference loss). § 302(b)(2) \
             mechanical test provides safer harbor when its 50/80 thresholds are met.",
        ),
        RedemptionTestPath::PartialLiquidationSection302B4 => sale_exchange_output(
            input,
            Severity::Section302BTestSatisfiedSaleOrExchangeTreatment,
            "§ 302(b)(4) PARTIAL LIQUIDATION test asserted (non-corporate shareholder). \
             Requires § 302(e) partial-liquidation definition: corporation conducted active \
             trade or business for 5+ years AND distributes proceeds of a discontinued \
             business segment. § 302(e)(1)(A) safe harbor + § 302(e)(2) active-business \
             requirement + § 302(e)(3) prohibition on substantially-all-net-investment-asset \
             redemptions. Redemption treated as § 1001 sale or exchange. Coordinates with \
             § 331 corporate liquidation distinction + § 332 parent-sub-liquidation regime.",
        ),
        RedemptionTestPath::NoSection302BTestAssertedDefaultDistribution => Output {
            severity: Severity::Section302DDefaultDistributionTreatmentSection301,
            treated_as_sale_or_exchange: false,
            capital_gain_or_loss_cents: 0,
            dividend_treatment_cents: input
                .property_received_cents
                .min(input.acquiring_corp_eep_cents),
            note: format!(
                "§ 302(d) DEFAULT § 301 distribution treatment. No § 302(b) test asserted or \
                 satisfied; redemption defaults to § 301 distribution character — ordinary \
                 dividend income to extent of acquiring-corp E&P (${}), basis recovery, then \
                 capital gain on excess. § 1(h)(11) qualified-dividend rate may apply if \
                 holding-period and other QDI requirements met. Consider rerunning analysis \
                 against § 302(b)(1) Davis NEED test, § 302(b)(2) 50/80 mechanical test, \
                 § 302(b)(3) complete-termination + § 302(c)(2) waiver path, or § 302(b)(4) \
                 partial-liquidation framework before defaulting to dividend treatment.",
                input.acquiring_corp_eep_cents / 100
            ),
        },
    }
}

fn sale_exchange_output(input: &Input, severity: Severity, note: &str) -> Output {
    let gain = input
        .property_received_cents
        .saturating_sub(input.shareholder_basis_in_redeemed_stock_cents);
    Output {
        severity,
        treated_as_sale_or_exchange: true,
        capital_gain_or_loss_cents: gain,
        dividend_treatment_cents: 0,
        note: format!(
            "{} Property received (${}) - shareholder basis (${}) = ${} capital gain or loss \
             under § 1001 + § 1222.",
            note,
            input.property_received_cents / 100,
            input.shareholder_basis_in_redeemed_stock_cents / 100,
            gain / 100
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_sub_disp() -> Input {
        Input {
            redemption_test_path: RedemptionTestPath::SubstantiallyDisproportionateSection302B2,
            attribution_status: AttributionStatus::AttributionDoesNotApply,
            pre_redemption_voting_pct_bps: 6_000,
            post_redemption_voting_pct_bps: 2_000,
            property_received_cents: 1_000_000_00,
            shareholder_basis_in_redeemed_stock_cents: 200_000_00,
            acquiring_corp_eep_cents: 500_000_00,
        }
    }

    #[test]
    fn substantially_disproportionate_50_80_test_satisfied() {
        let input = base_sub_disp();
        let output = check(&input);
        // Post 20% < 50% AND post 20% < pre 60% × 80% = 48% → both met
        assert_eq!(
            output.severity,
            Severity::Section302BTestSatisfiedSaleOrExchangeTreatment
        );
        assert!(output.treated_as_sale_or_exchange);
        // $1M - $200K = $800K capital gain
        assert_eq!(output.capital_gain_or_loss_cents, 800_000_00);
    }

    #[test]
    fn substantially_disproportionate_50_test_failed() {
        let mut input = base_sub_disp();
        input.post_redemption_voting_pct_bps = 5_500; // 55% post
        let output = check(&input);
        // Post 55% > 50% threshold → fails 50% test
        assert_eq!(
            output.severity,
            Severity::Section302B2SubstantiallyDisproportionate50_80TestFailed
        );
        assert!(!output.treated_as_sale_or_exchange);
    }

    #[test]
    fn substantially_disproportionate_80_test_failed() {
        let mut input = base_sub_disp();
        input.post_redemption_voting_pct_bps = 4_900; // 49% post; 49 / 60 = 81.6% → > 80%
        let output = check(&input);
        // 49% > 60% × 80% = 48% → fails 80% drop test
        assert_eq!(
            output.severity,
            Severity::Section302B2SubstantiallyDisproportionate50_80TestFailed
        );
    }

    #[test]
    fn complete_termination_with_attribution_waiver() {
        let mut input = base_sub_disp();
        input.redemption_test_path = RedemptionTestPath::CompleteTerminationSection302B3;
        input.attribution_status = AttributionStatus::Section302C2FamilyAttributionWaiverFiled;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section302B3CompleteTerminationWithAttributionWaiver
        );
        assert!(output.treated_as_sale_or_exchange);
        assert!(output.note.contains("§ 302(c)(2)"));
        assert!(output.note.contains("10-year"));
    }

    #[test]
    fn complete_termination_attribution_defeats_failed_waiver() {
        let mut input = base_sub_disp();
        input.redemption_test_path = RedemptionTestPath::CompleteTerminationSection302B3;
        input.attribution_status = AttributionStatus::AttributionAppliesIncreasesOwnership;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section302B3CompleteTerminationAttributionDefeatsFailedWaiver
        );
        assert!(!output.treated_as_sale_or_exchange);
    }

    #[test]
    fn complete_termination_no_attribution_required_sale_treatment() {
        let mut input = base_sub_disp();
        input.redemption_test_path = RedemptionTestPath::CompleteTerminationSection302B3;
        input.attribution_status = AttributionStatus::AttributionDoesNotApply;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section302BTestSatisfiedSaleOrExchangeTreatment
        );
    }

    #[test]
    fn not_essentially_equivalent_dividend_test_asserted() {
        let mut input = base_sub_disp();
        input.redemption_test_path = RedemptionTestPath::NotEssentiallyEquivalentSection302B1;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section302BTestSatisfiedSaleOrExchangeTreatment
        );
        assert!(output.note.contains("United States v. Davis"));
        assert!(output.note.contains("MEANINGFUL REDUCTION"));
    }

    #[test]
    fn partial_liquidation_test_asserted() {
        let mut input = base_sub_disp();
        input.redemption_test_path = RedemptionTestPath::PartialLiquidationSection302B4;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section302BTestSatisfiedSaleOrExchangeTreatment
        );
        assert!(output.note.contains("§ 302(b)(4)"));
        assert!(output.note.contains("§ 302(e)"));
        assert!(output.note.contains("5+ years"));
    }

    #[test]
    fn no_test_asserted_default_section_301_distribution() {
        let mut input = base_sub_disp();
        input.redemption_test_path =
            RedemptionTestPath::NoSection302BTestAssertedDefaultDistribution;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section302DDefaultDistributionTreatmentSection301
        );
        assert!(!output.treated_as_sale_or_exchange);
        assert!(output.note.contains("§ 1(h)(11)"));
    }

    #[test]
    fn section_302b2_50_pct_threshold_constant_pins_5000_bps() {
        assert_eq!(SECTION_302B2_POST_REDEMPTION_VOTING_CAP_BPS, 5_000);
    }

    #[test]
    fn section_302b2_80_pct_drop_factor_constant_pins_8000_bps() {
        assert_eq!(SECTION_302B2_PRE_TO_POST_DROP_FACTOR_BPS, 8_000);
    }

    #[test]
    fn section_302c2_ten_year_bar_constant_pins_10_years() {
        assert_eq!(SECTION_302C2_TEN_YEAR_REACQUISITION_BAR_YEARS, 10);
    }

    #[test]
    fn very_large_property_no_overflow() {
        let mut input = base_sub_disp();
        input.property_received_cents = u64::MAX;
        let output = check(&input);
        assert!(output.capital_gain_or_loss_cents > 0);
    }

    #[test]
    fn zero_property_zero_gain() {
        let mut input = base_sub_disp();
        input.property_received_cents = 0;
        let output = check(&input);
        assert_eq!(output.capital_gain_or_loss_cents, 0);
    }

    #[test]
    fn note_pins_section_318_attribution_companion() {
        let input = base_sub_disp();
        let output = check(&input);
        assert!(output.note.contains("§ 318"));
    }

    #[test]
    fn note_pins_section_1222_capital_gain_character() {
        let input = base_sub_disp();
        let output = check(&input);
        assert!(output.note.contains("§ 1222"));
    }

    #[test]
    fn note_pins_section_1001_sale_or_exchange() {
        let input = base_sub_disp();
        let output = check(&input);
        assert!(output.note.contains("§ 1001"));
    }

    #[test]
    fn note_pins_section_301_distribution_default_when_failed() {
        let mut input = base_sub_disp();
        input.post_redemption_voting_pct_bps = 5_500;
        let output = check(&input);
        assert!(output.note.contains("§ 301"));
    }

    #[test]
    fn boundary_post_50_pct_exactly_fails_test_must_be_less_than() {
        let mut input = base_sub_disp();
        input.post_redemption_voting_pct_bps = 5_000; // Exactly 50%
        let output = check(&input);
        // < 50% required; exactly 50% fails
        assert_eq!(
            output.severity,
            Severity::Section302B2SubstantiallyDisproportionate50_80TestFailed
        );
    }

    #[test]
    fn boundary_post_49_pct_satisfies_50_test_if_80_also_met() {
        let mut input = base_sub_disp();
        input.post_redemption_voting_pct_bps = 4_700; // 47%
                                                      // Required for 80%: 6000 × 0.8 = 4800; 4700 < 4800 → satisfies 80%
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section302BTestSatisfiedSaleOrExchangeTreatment
        );
    }
}
