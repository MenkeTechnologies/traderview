//! IRC § 6425 — Adjustment of Overpayment of Estimated
//! Income Tax by Corporation / Form 4466 Quick Refund
//! Module.
//!
//! Pure-compute check for IRC § 6425 corporate quick-refund
//! procedure. § 6425 is the procedural companion to § 6655
//! (corporate estimated tax underpayment penalty — built
//! iter 676) and § 6621 (interest rate determination —
//! built iter 674). It allows a C corporation that has
//! overpaid its quarterly estimated taxes to obtain an
//! ACCELERATED REFUND (within 45 days) rather than waiting
//! for the standard tax-return refund cycle. Form 4466 is
//! the operative IRS form; the application must be filed
//! BEFORE the corporation files its tax return and on or
//! before the 15th day of the 4th month after the close of
//! the taxable year. § 6425 adjustments are NOT considered
//! claims for credit or refund under § 6511 — they follow a
//! distinct procedural track.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 6425(a) Right to Adjustment**: a corporation may file an application for an adjustment of an overpayment of estimated income tax for any taxable year. The application must be VERIFIED, and made under penalties of perjury, on **FORM 4466 ("Corporation Application for Quick Refund of Overpayment of Estimated Tax")** ([Cornell LII 26 USC § 6425](https://www.law.cornell.edu/uscode/text/26/6425); [Bloomberg Tax Sec. 6425](https://irc.bloombergtax.com/public/uscode/doc/irc/section_6425); [IRS Instructions for Form 4466 (12/2025)](https://www.irs.gov/instructions/i4466); [IRS Form 4466 (Rev. December 2025)](https://www.irs.gov/pub/irs-access/f4466_accessible.pdf); [26 CFR § 1.6425-1 Adjustment of Overpayment of Estimated Income Tax by Corporation](https://www.law.cornell.edu/cfr/text/26/1.6425-1); [26 CFR § 1.6425-2 Computation of Adjustment of Overpayment of Estimated Tax](https://www.law.cornell.edu/cfr/text/26/1.6425-2); [26 CFR § 1.6425-3 Allowance of Adjustments](https://www.law.cornell.edu/cfr/text/26/1.6425-3); [Accountably — Form 4466 Quick Refund Guide 2026 for Corporations](https://accountably.com/irs-forms/f4466/)).
//! - **IRC § 6425(b) Filing Deadline**: the application must be filed on or before the **15th DAY of the 4th MONTH after the close of the taxable year** AND **BEFORE the corporation files its income tax return** for such taxable year (whichever occurs earlier). For calendar-year corporations the deadline is April 15; for fiscal-year corporations the deadline is the 15th day of the 4th month after the end of the fiscal year. Form 4466 is NOT eligible for an extension of time to file.
//! - **IRC § 6425(b)(2) and Treas. Reg. § 1.6425-2 Computation of Adjustment**: the amount of the adjustment under § 6425 is the EXCESS of the estimated income tax PAID by the corporation during the taxable year OVER the amount which, at the time of filing Form 4466, the corporation ESTIMATES as its income tax liability for the taxable year (i.e., overpayment = paid − estimated liability).
//! - **IRC § 6425(b)(3) Minimum Threshold**: no application is allowed unless the amount of the adjustment EQUALS OR EXCEEDS BOTH (a) **10 PERCENT of the amount estimated by the corporation on its application as its income tax liability for the taxable year** AND (b) **$500**. The "10% AND $500" double-test is conjunctive — applications below either threshold are disallowed.
//! - **IRC § 6425(c) IRS Processing Window**: within a period of **45 DAYS** from the date on which an application for an adjustment is filed, the Secretary shall (1) make a limited examination of the application to discover omissions and errors; (2) determine the amount of the adjustment; (3) credit the amount of the adjustment against any liability in respect of an internal revenue tax on the part of the corporation; and (4) refund any remainder to the corporation.
//! - **NOT a Refund Claim under § 6511**: an application under § 6425 SHALL NOT constitute a claim for credit or refund under § 6511 (which establishes the standard 3-year-from-return-filing or 2-year-from-payment refund-claim limitations period). The § 6425 quick-refund process is distinct from the standard § 6511 refund-claim procedure.
//! - **§ 6655(h) Excessive Adjustment Interest Charge**: if the IRS makes a § 6425 quick refund and the refund proves EXCESSIVE (i.e., the corporation's actual final tax liability exceeds the post-adjustment estimated liability), interest at the **§ 6621 UNDERPAYMENT RATE** accrues from the refund date through the 15th day of the 4th month following year-end. This § 6655(h) interest charge is the corporate counterpart to the § 6601 underpayment-interest mechanism and serves as the price for over-claiming the quick refund.
//! - **Tax-Year Application**: § 6425 applies only to C corporations subject to the corporate income tax under § 11 (and certain other corporate entities subject to corporate-style estimated tax under § 6655). S corporations, partnerships, individuals, trusts, and estates are NOT eligible to file Form 4466.
//! - **Form 4466 Filing Mechanics**: Form 4466 must be filed by paper (electronic filing is not currently available); duplicate not required; filed at the IRS Service Center where the corporation files its income tax return.
//! - **Penalty Considerations**: an application under § 6425 with a substantial understatement of estimated tax liability may trigger § 6662 accuracy-related penalties on the underpayment; the willful filing of a fraudulent Form 4466 may trigger § 7206 criminal penalties for fraud and false statements.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_6425_FILING_DEADLINE_MONTH: u32 = 4;
pub const IRC_6425_FILING_DEADLINE_DAY: u32 = 15;
pub const IRC_6425_MINIMUM_ADJUSTMENT_PERCENTAGE_BASIS_POINTS: u64 = 1_000;
pub const IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS: u64 = 500;
pub const IRC_6425_IRS_PROCESSING_WINDOW_DAYS: u32 = 45;
pub const IRC_6425_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorporationType {
    CCorporationEligibleForSection6425,
    OtherEntityNotEligibleForSection6425,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationStatus {
    Form4466ApplicationFiledWithinDeadlineAndBeforeReturn,
    Form4466ApplicationFiledAfter15thDayOf4thMonth,
    Form4466ApplicationFiledAfterTaxReturnFiled,
    NoApplicationFiledOrIncorrectForm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    QuickRefundEligibilityAndProcedureCheck,
    AdjustmentMinimumThresholdCheck,
    IrsProcessingWindowCheck,
    ExcessiveAdjustmentInterestChargeUnderSection6655H,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExcessiveAdjustmentStatus {
    QuickRefundNotExcessiveNoSection6655HInterest,
    QuickRefundExcessiveSection6655HInterestApplies,
    NotApplicableNoQuickRefundProcessed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section6425Mode {
    NotApplicableNotACCorporation,
    NotApplicableNoOverpaymentOfEstimatedTax,
    NotApplicableApplicationFiledAfterTaxReturnAlreadyFiled,
    CompliantQuickRefundApplicationFiledTimelyOnForm4466,
    CompliantAdjustmentMeets10PctAnd500DollarMinimumThreshold,
    CompliantIrsProcessedWithin45Days,
    CompliantQuickRefundNotExcessiveNoSection6655HInterestCharge,
    CompliantNotConstrutedAsRefundClaimUnderSection6511,
    ViolationApplicationFiledAfter15thDayOf4thMonth,
    ViolationApplicationFiledAfterTaxReturn,
    ViolationAdjustmentBelow10PercentMinimumOfEstimatedTaxLiability,
    ViolationAdjustmentBelow500DollarMinimum,
    ViolationIrsProcessingExceeded45Days,
    ViolationExcessiveQuickRefundSection6655HInterestChargeApplies,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub corporation_type: CorporationType,
    pub application_status: ApplicationStatus,
    pub compliance_aspect: ComplianceAspect,
    pub estimated_tax_paid_dollars: u64,
    pub estimated_income_tax_liability_dollars: u64,
    pub adjustment_amount_dollars: u64,
    pub irs_processing_days: u32,
    pub excessive_adjustment_status: ExcessiveAdjustmentStatus,
    pub federal_short_term_rate_basis_points: u64,
    pub days_excessive_period_for_section_6655h: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section6425Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub statutory_minimum_adjustment_percentage_dollars: u64,
    pub statutory_minimum_adjustment_absolute_dollars: u64,
    pub estimated_section_6655h_interest_charge_dollars: u64,
}

pub type Section6425Input = Input;
pub type Section6425Output = Output;
pub type Section6425Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 6425(a) Right to Adjustment — corporation may file application for adjustment of overpayment of estimated income tax for any taxable year; application must be VERIFIED, made under penalties of perjury, on FORM 4466 (Corporation Application for Quick Refund of Overpayment of Estimated Tax)".to_string(),
        "IRC § 6425(b) Filing Deadline — application must be filed on or before 15th DAY of 4th MONTH after close of taxable year AND BEFORE corporation files income tax return; for calendar-year corporations deadline is April 15; Form 4466 NOT eligible for extension".to_string(),
        "IRC § 6425(b)(2) + Treas. Reg. § 1.6425-2 Computation of Adjustment — adjustment amount = EXCESS of estimated income tax PAID by corporation during taxable year OVER amount which, at time of filing Form 4466, corporation ESTIMATES as its income tax liability for the taxable year".to_string(),
        "IRC § 6425(b)(3) Minimum Threshold — no application allowed unless adjustment EQUALS OR EXCEEDS BOTH (a) 10 PERCENT of corporation's estimated income tax liability AND (b) $500 (conjunctive double-test)".to_string(),
        "IRC § 6425(c) IRS Processing Window — within 45 DAYS from date application filed, Secretary shall (1) make limited examination for omissions and errors; (2) determine adjustment amount; (3) credit against any internal revenue tax liability of corporation; (4) refund remainder to corporation".to_string(),
        "Not a Refund Claim under § 6511 — application under § 6425 SHALL NOT constitute a claim for credit or refund under § 6511 (which establishes 3-year-from-return-filing or 2-year-from-payment refund-claim limitations); § 6425 quick-refund is distinct procedural track".to_string(),
        "§ 6655(h) Excessive Adjustment Interest Charge — if IRS makes § 6425 quick refund and refund proves EXCESSIVE (corporation's actual final tax liability exceeds post-adjustment estimated liability), interest at § 6621 UNDERPAYMENT RATE accrues from refund date through 15th day of 4th month following year-end; corporate counterpart to § 6601 underpayment-interest".to_string(),
        "Tax-Year Application — § 6425 applies only to C corporations subject to corporate income tax under § 11 (and certain other corporate entities subject to corporate-style estimated tax under § 6655); S corporations, partnerships, individuals, trusts, estates NOT eligible".to_string(),
        "Form 4466 Filing Mechanics — Form 4466 must be filed by paper (electronic filing not currently available); filed at IRS Service Center where corporation files income tax return".to_string(),
        "Penalty Considerations — application under § 6425 with substantial understatement may trigger § 6662 accuracy-related penalties; willful filing of fraudulent Form 4466 may trigger § 7206 criminal penalties for fraud and false statements".to_string(),
        "Companion Provisions — § 6621 (built iter 674; underpayment rate that § 6655(h) excessive-adjustment interest cites); § 6601 (general underpayment interest); § 6611 (overpayment interest); § 6655 (built iter 676; corporate estimated tax underpayment penalty); § 6511 (refund-claim limitations period — DISTINCT from § 6425); § 6662 (accuracy-related penalty); § 7206 (fraud and false statements)".to_string(),
        "Cornell LII 26 USC § 6425 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 6425 — comprehensive code commentary".to_string(),
        "IRS Instructions for Form 4466 (12/2025) — official filing instructions".to_string(),
        "IRS Form 4466 (Rev. December 2025) — operative quick-refund application form".to_string(),
        "26 CFR § 1.6425-1 — Adjustment of Overpayment of Estimated Income Tax by Corporation regulation".to_string(),
        "26 CFR § 1.6425-2 — Computation of Adjustment of Overpayment of Estimated Tax regulation".to_string(),
        "26 CFR § 1.6425-3 — Allowance of Adjustments regulation".to_string(),
        "Accountably — Form 4466 Quick Refund Guide 2026 for Corporations practitioner reference".to_string(),
    ];

    if input.corporation_type == CorporationType::OtherEntityNotEligibleForSection6425 {
        return Output {
            mode: Section6425Mode::NotApplicableNotACCorporation,
            statutory_basis: "IRC § 6425 — applies only to C corporations subject to corporate income tax under § 11 and corporate-style estimated tax under § 6655".to_string(),
            notes: "NOT APPLICABLE: taxpayer is not a C corporation eligible for § 6425 quick-refund procedure; S corporations, partnerships, individuals, trusts, and estates are not eligible to file Form 4466.".to_string(),
            citations,
            statutory_minimum_adjustment_percentage_dollars: 0,
            statutory_minimum_adjustment_absolute_dollars: IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS,
            estimated_section_6655h_interest_charge_dollars: 0,
        };
    }

    if input.estimated_tax_paid_dollars <= input.estimated_income_tax_liability_dollars {
        return Output {
            mode: Section6425Mode::NotApplicableNoOverpaymentOfEstimatedTax,
            statutory_basis: "IRC § 6425(a) + (b)(2) — adjustment available only when estimated tax paid exceeds estimated income tax liability".to_string(),
            notes: format!(
                "NOT APPLICABLE: estimated tax paid (${}) does not exceed estimated income tax liability (${}); no overpayment to adjust under § 6425; quick-refund procedure not available.",
                input.estimated_tax_paid_dollars, input.estimated_income_tax_liability_dollars
            ),
            citations,
            statutory_minimum_adjustment_percentage_dollars: 0,
            statutory_minimum_adjustment_absolute_dollars: IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS,
            estimated_section_6655h_interest_charge_dollars: 0,
        };
    }

    let minimum_adjustment_from_percentage = u128::from(input.estimated_income_tax_liability_dollars)
        .saturating_mul(u128::from(IRC_6425_MINIMUM_ADJUSTMENT_PERCENTAGE_BASIS_POINTS))
        .checked_div(u128::from(IRC_6425_BASIS_POINT_DENOMINATOR))
        .unwrap_or(0)
        .min(u128::from(u64::MAX)) as u64;

    match input.compliance_aspect {
        ComplianceAspect::QuickRefundEligibilityAndProcedureCheck => match input.application_status {
            ApplicationStatus::Form4466ApplicationFiledWithinDeadlineAndBeforeReturn => Output {
                mode: Section6425Mode::CompliantQuickRefundApplicationFiledTimelyOnForm4466,
                statutory_basis: "IRC § 6425(a) + (b) — Form 4466 application filed on or before 15th day of 4th month after close of taxable year and before tax return filed".to_string(),
                notes: format!(
                    "COMPLIANT: Form 4466 quick-refund application properly filed within the § 6425(b) deadline (15th day of 4th month after close of taxable year AND before tax return filed); adjustment amount ${} (= estimated tax paid ${} − estimated income tax liability ${}); § 6425 application is NOT a refund claim under § 6511.",
                    input.adjustment_amount_dollars,
                    input.estimated_tax_paid_dollars,
                    input.estimated_income_tax_liability_dollars
                ),
                citations,
                statutory_minimum_adjustment_percentage_dollars: minimum_adjustment_from_percentage,
                statutory_minimum_adjustment_absolute_dollars: IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS,
                estimated_section_6655h_interest_charge_dollars: 0,
            },
            ApplicationStatus::Form4466ApplicationFiledAfter15thDayOf4thMonth => Output {
                mode: Section6425Mode::ViolationApplicationFiledAfter15thDayOf4thMonth,
                statutory_basis: "IRC § 6425(b) — application must be filed on or before 15th day of 4th month after close of taxable year".to_string(),
                notes: "VIOLATION: Form 4466 application filed after the § 6425(b) statutory deadline of 15th day of 4th month after close of taxable year; quick-refund unavailable; corporation must wait for standard refund through tax-return filing.".to_string(),
                citations,
                statutory_minimum_adjustment_percentage_dollars: minimum_adjustment_from_percentage,
                statutory_minimum_adjustment_absolute_dollars: IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS,
                estimated_section_6655h_interest_charge_dollars: 0,
            },
            ApplicationStatus::Form4466ApplicationFiledAfterTaxReturnFiled => Output {
                mode: Section6425Mode::ViolationApplicationFiledAfterTaxReturn,
                statutory_basis: "IRC § 6425(b) — application must be filed BEFORE corporation files its income tax return".to_string(),
                notes: "VIOLATION: Form 4466 application filed AFTER the corporation already filed its income tax return for the taxable year; § 6425(b) requires application before tax return filing; quick-refund unavailable; corporation must claim refund through standard § 6402 / § 6511 mechanism.".to_string(),
                citations,
                statutory_minimum_adjustment_percentage_dollars: minimum_adjustment_from_percentage,
                statutory_minimum_adjustment_absolute_dollars: IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS,
                estimated_section_6655h_interest_charge_dollars: 0,
            },
            ApplicationStatus::NoApplicationFiledOrIncorrectForm => Output {
                mode: Section6425Mode::NotApplicableApplicationFiledAfterTaxReturnAlreadyFiled,
                statutory_basis: "IRC § 6425(a) — application required on Form 4466".to_string(),
                notes: "NOT APPLICABLE: no Form 4466 application filed or incorrect form used; § 6425 quick-refund procedure not invoked; corporation must use standard refund procedures.".to_string(),
                citations,
                statutory_minimum_adjustment_percentage_dollars: minimum_adjustment_from_percentage,
                statutory_minimum_adjustment_absolute_dollars: IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS,
                estimated_section_6655h_interest_charge_dollars: 0,
            },
        },
        ComplianceAspect::AdjustmentMinimumThresholdCheck => {
            if input.adjustment_amount_dollars < minimum_adjustment_from_percentage {
                Output {
                    mode: Section6425Mode::ViolationAdjustmentBelow10PercentMinimumOfEstimatedTaxLiability,
                    statutory_basis: "IRC § 6425(b)(3)(A) — adjustment must equal or exceed 10 percent of estimated income tax liability".to_string(),
                    notes: format!(
                        "VIOLATION: adjustment amount ${} is below the 10 % minimum of estimated income tax liability (${} = 10 % of ${}); § 6425(b)(3) requires adjustment to meet BOTH the 10 % AND $500 minimum thresholds.",
                        input.adjustment_amount_dollars,
                        minimum_adjustment_from_percentage,
                        input.estimated_income_tax_liability_dollars
                    ),
                    citations,
                    statutory_minimum_adjustment_percentage_dollars: minimum_adjustment_from_percentage,
                    statutory_minimum_adjustment_absolute_dollars: IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS,
                    estimated_section_6655h_interest_charge_dollars: 0,
                }
            } else if input.adjustment_amount_dollars < IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS {
                Output {
                    mode: Section6425Mode::ViolationAdjustmentBelow500DollarMinimum,
                    statutory_basis: "IRC § 6425(b)(3)(B) — adjustment must equal or exceed $500".to_string(),
                    notes: format!(
                        "VIOLATION: adjustment amount ${} is below the $500 absolute minimum under § 6425(b)(3)(B); even though 10 % threshold met, the $500 floor disallows the application.",
                        input.adjustment_amount_dollars
                    ),
                    citations,
                    statutory_minimum_adjustment_percentage_dollars: minimum_adjustment_from_percentage,
                    statutory_minimum_adjustment_absolute_dollars: IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS,
                    estimated_section_6655h_interest_charge_dollars: 0,
                }
            } else {
                Output {
                    mode: Section6425Mode::CompliantAdjustmentMeets10PctAnd500DollarMinimumThreshold,
                    statutory_basis: "IRC § 6425(b)(3) — adjustment meets both 10 % of estimated income tax liability AND $500 minimum".to_string(),
                    notes: format!(
                        "COMPLIANT: adjustment amount ${} satisfies BOTH minimum thresholds — (a) 10 % of estimated income tax liability = ${} AND (b) $500 absolute minimum.",
                        input.adjustment_amount_dollars,
                        minimum_adjustment_from_percentage
                    ),
                    citations,
                    statutory_minimum_adjustment_percentage_dollars: minimum_adjustment_from_percentage,
                    statutory_minimum_adjustment_absolute_dollars: IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS,
                    estimated_section_6655h_interest_charge_dollars: 0,
                }
            }
        }
        ComplianceAspect::IrsProcessingWindowCheck => {
            if input.irs_processing_days > IRC_6425_IRS_PROCESSING_WINDOW_DAYS {
                Output {
                    mode: Section6425Mode::ViolationIrsProcessingExceeded45Days,
                    statutory_basis: "IRC § 6425(c) — IRS must process application within 45 days".to_string(),
                    notes: format!(
                        "VIOLATION OF AGENCY DEADLINE: IRS processing took {} days; § 6425(c) requires examination, adjustment determination, credit / refund within 45 days; corporation may pursue administrative remedies through Taxpayer Advocate Service or formal complaint.",
                        input.irs_processing_days
                    ),
                    citations,
                    statutory_minimum_adjustment_percentage_dollars: minimum_adjustment_from_percentage,
                    statutory_minimum_adjustment_absolute_dollars: IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS,
                    estimated_section_6655h_interest_charge_dollars: 0,
                }
            } else {
                Output {
                    mode: Section6425Mode::CompliantIrsProcessedWithin45Days,
                    statutory_basis: "IRC § 6425(c) — IRS processed application within 45 days".to_string(),
                    notes: format!(
                        "COMPLIANT: IRS processed Form 4466 application within {} days (≤ 45-day statutory window); examination, adjustment determination, credit, and refund completed within § 6425(c) timeframe.",
                        input.irs_processing_days
                    ),
                    citations,
                    statutory_minimum_adjustment_percentage_dollars: minimum_adjustment_from_percentage,
                    statutory_minimum_adjustment_absolute_dollars: IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS,
                    estimated_section_6655h_interest_charge_dollars: 0,
                }
            }
        }
        ComplianceAspect::ExcessiveAdjustmentInterestChargeUnderSection6655H => match input.excessive_adjustment_status {
            ExcessiveAdjustmentStatus::QuickRefundExcessiveSection6655HInterestApplies => {
                let underpayment_rate_bps = input.federal_short_term_rate_basis_points.saturating_add(300);
                let interest_charge = u128::from(input.adjustment_amount_dollars)
                    .saturating_mul(u128::from(underpayment_rate_bps))
                    .saturating_mul(u128::from(input.days_excessive_period_for_section_6655h))
                    .checked_div(u128::from(IRC_6425_BASIS_POINT_DENOMINATOR).saturating_mul(365))
                    .unwrap_or(0)
                    .min(u128::from(u64::MAX)) as u64;
                Output {
                    mode: Section6425Mode::ViolationExcessiveQuickRefundSection6655HInterestChargeApplies,
                    statutory_basis: "§ 6655(h) — excessive § 6425 quick refund triggers interest at § 6621 underpayment rate from refund date through 15th day of 4th month following year-end".to_string(),
                    notes: format!(
                        "VIOLATION OF SAFE-REFUND TREATMENT: § 6425 quick refund of ${} proved excessive; § 6655(h) imposes interest at § 6621 underpayment rate (FSTR {} bps + 3 pp = {} bps) over {} excessive-period days; estimated linear interest charge ${} (statutory § 6622 daily compounding so this is an approximation).",
                        input.adjustment_amount_dollars,
                        input.federal_short_term_rate_basis_points,
                        underpayment_rate_bps,
                        input.days_excessive_period_for_section_6655h,
                        interest_charge
                    ),
                    citations,
                    statutory_minimum_adjustment_percentage_dollars: minimum_adjustment_from_percentage,
                    statutory_minimum_adjustment_absolute_dollars: IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS,
                    estimated_section_6655h_interest_charge_dollars: interest_charge,
                }
            }
            ExcessiveAdjustmentStatus::QuickRefundNotExcessiveNoSection6655HInterest
            | ExcessiveAdjustmentStatus::NotApplicableNoQuickRefundProcessed => Output {
                mode: Section6425Mode::CompliantQuickRefundNotExcessiveNoSection6655HInterestCharge,
                statutory_basis: "§ 6655(h) — § 6425 quick refund not excessive; no interest charge".to_string(),
                notes: "COMPLIANT: § 6425 quick refund did not prove excessive; § 6655(h) interest-charge mechanism not triggered; final corporate income tax liability matches or exceeds post-adjustment estimated liability.".to_string(),
                citations,
                statutory_minimum_adjustment_percentage_dollars: minimum_adjustment_from_percentage,
                statutory_minimum_adjustment_absolute_dollars: IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS,
                estimated_section_6655h_interest_charge_dollars: 0,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            corporation_type: CorporationType::CCorporationEligibleForSection6425,
            application_status: ApplicationStatus::Form4466ApplicationFiledWithinDeadlineAndBeforeReturn,
            compliance_aspect: ComplianceAspect::QuickRefundEligibilityAndProcedureCheck,
            estimated_tax_paid_dollars: 1_000_000,
            estimated_income_tax_liability_dollars: 800_000,
            adjustment_amount_dollars: 200_000,
            irs_processing_days: 30,
            excessive_adjustment_status:
                ExcessiveAdjustmentStatus::QuickRefundNotExcessiveNoSection6655HInterest,
            federal_short_term_rate_basis_points: 500,
            days_excessive_period_for_section_6655h: 0,
        }
    }

    #[test]
    fn not_a_c_corporation_not_applicable() {
        let mut input = baseline_input();
        input.corporation_type = CorporationType::OtherEntityNotEligibleForSection6425;
        let output = check(&input);
        assert_eq!(output.mode, Section6425Mode::NotApplicableNotACCorporation);
    }

    #[test]
    fn no_overpayment_not_applicable() {
        let mut input = baseline_input();
        input.estimated_tax_paid_dollars = 800_000;
        input.estimated_income_tax_liability_dollars = 800_000;
        let output = check(&input);
        assert_eq!(output.mode, Section6425Mode::NotApplicableNoOverpaymentOfEstimatedTax);
    }

    #[test]
    fn form_4466_filed_within_deadline_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section6425Mode::CompliantQuickRefundApplicationFiledTimelyOnForm4466
        );
        // 10 % of $800,000 = $80,000
        assert_eq!(output.statutory_minimum_adjustment_percentage_dollars, 80_000);
    }

    #[test]
    fn form_4466_filed_after_15th_day_of_4th_month_violation() {
        let mut input = baseline_input();
        input.application_status = ApplicationStatus::Form4466ApplicationFiledAfter15thDayOf4thMonth;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6425Mode::ViolationApplicationFiledAfter15thDayOf4thMonth
        );
    }

    #[test]
    fn form_4466_filed_after_tax_return_violation() {
        let mut input = baseline_input();
        input.application_status = ApplicationStatus::Form4466ApplicationFiledAfterTaxReturnFiled;
        let output = check(&input);
        assert_eq!(output.mode, Section6425Mode::ViolationApplicationFiledAfterTaxReturn);
    }

    #[test]
    fn adjustment_meets_10pct_and_500_minimum_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AdjustmentMinimumThresholdCheck;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6425Mode::CompliantAdjustmentMeets10PctAnd500DollarMinimumThreshold
        );
    }

    #[test]
    fn adjustment_below_10pct_minimum_violation() {
        // 10 % of $800K = $80K; adjustment $50K < $80K
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AdjustmentMinimumThresholdCheck;
        input.adjustment_amount_dollars = 50_000;
        input.estimated_tax_paid_dollars = 850_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6425Mode::ViolationAdjustmentBelow10PercentMinimumOfEstimatedTaxLiability
        );
    }

    #[test]
    fn adjustment_below_500_dollar_minimum_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::AdjustmentMinimumThresholdCheck;
        input.adjustment_amount_dollars = 400;
        input.estimated_tax_paid_dollars = 4_400;
        input.estimated_income_tax_liability_dollars = 4_000;
        // 10 % of $4,000 = $400; adjustment $400 >= $400 (10 % passes) BUT $400 < $500 absolute
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6425Mode::ViolationAdjustmentBelow500DollarMinimum
        );
    }

    #[test]
    fn irs_processed_within_45_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::IrsProcessingWindowCheck;
        let output = check(&input);
        assert_eq!(output.mode, Section6425Mode::CompliantIrsProcessedWithin45Days);
    }

    #[test]
    fn irs_processed_at_exactly_45_days_compliant_boundary() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::IrsProcessingWindowCheck;
        input.irs_processing_days = 45;
        let output = check(&input);
        assert_eq!(output.mode, Section6425Mode::CompliantIrsProcessedWithin45Days);
    }

    #[test]
    fn irs_processing_exceeds_45_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::IrsProcessingWindowCheck;
        input.irs_processing_days = 60;
        let output = check(&input);
        assert_eq!(output.mode, Section6425Mode::ViolationIrsProcessingExceeded45Days);
    }

    #[test]
    fn excessive_quick_refund_section_6655h_interest_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ExcessiveAdjustmentInterestChargeUnderSection6655H;
        input.excessive_adjustment_status =
            ExcessiveAdjustmentStatus::QuickRefundExcessiveSection6655HInterestApplies;
        input.days_excessive_period_for_section_6655h = 90;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6425Mode::ViolationExcessiveQuickRefundSection6655HInterestChargeApplies
        );
        // $200,000 × 800 bps × 90 days / (10,000 × 365) ≈ $3,945
        assert_eq!(output.estimated_section_6655h_interest_charge_dollars, 3_945);
    }

    #[test]
    fn not_excessive_quick_refund_no_section_6655h_interest_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ExcessiveAdjustmentInterestChargeUnderSection6655H;
        input.excessive_adjustment_status =
            ExcessiveAdjustmentStatus::QuickRefundNotExcessiveNoSection6655HInterest;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section6425Mode::CompliantQuickRefundNotExcessiveNoSection6655HInterestCharge
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_6425_FILING_DEADLINE_MONTH, 4);
        assert_eq!(IRC_6425_FILING_DEADLINE_DAY, 15);
        assert_eq!(IRC_6425_MINIMUM_ADJUSTMENT_PERCENTAGE_BASIS_POINTS, 1_000);
        assert_eq!(IRC_6425_MINIMUM_ADJUSTMENT_DOLLARS, 500);
        assert_eq!(IRC_6425_IRS_PROCESSING_WINDOW_DAYS, 45);
        assert_eq!(IRC_6425_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("§ 6425(a)"));
        assert!(joined.contains("§ 6425(b)"));
        assert!(joined.contains("§ 6425(c)"));
        assert!(joined.contains("§ 6425(b)(3)"));
        assert!(joined.contains("FORM 4466"));
        assert!(joined.contains("15th DAY of 4th MONTH"));
        assert!(joined.contains("10 PERCENT"));
        assert!(joined.contains("$500"));
        assert!(joined.contains("45 DAYS"));
        assert!(joined.contains("§ 6655(h)"));
        assert!(joined.contains("§ 6621"));
        assert!(joined.contains("§ 6511"));
        assert!(joined.contains("§ 6662"));
        assert!(joined.contains("§ 7206"));
        assert!(joined.contains("1.6425-1"));
    }

    #[test]
    fn saturating_overflow_defense() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ExcessiveAdjustmentInterestChargeUnderSection6655H;
        input.excessive_adjustment_status =
            ExcessiveAdjustmentStatus::QuickRefundExcessiveSection6655HInterestApplies;
        input.adjustment_amount_dollars = u64::MAX;
        input.federal_short_term_rate_basis_points = u64::MAX;
        input.days_excessive_period_for_section_6655h = u32::MAX;
        let output = check(&input);
        let _ = output.mode;
    }
}
