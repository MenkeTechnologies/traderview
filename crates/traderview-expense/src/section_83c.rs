//! IRC §83(c) — "Substantial risk of forfeiture" timing rules for
//! property transferred in connection with performance of services.
//!
//! Completes the §83 stock-compensation triplet alongside the
//! existing `section_83b` (election to recognize at grant) and
//! `section_83i` (qualified equity grants 5-year deferral) modules.
//! §83(c) defines WHEN property is "substantially vested" — the
//! point at which §83(a) requires income recognition. The two
//! prongs that delay vesting are (1) "substantial risk of
//! forfeiture" under §83(c)(1) and (2) non-transferability under
//! §83(c)(2). §83(c)(3) carves out a special rule for executives
//! whose sale would create § 16(b) Securities Exchange Act
//! liability.
//!
//! **§83(a) timing rule** (the rule §83(c) gates): property
//! transferred to a service provider in connection with the
//! performance of services is taxable when the property becomes
//! **either** (a) transferable, **or** (b) not subject to
//! substantial risk of forfeiture — whichever is the **earlier**.
//!
//! **§83(c)(1) substantial risk of forfeiture** — exists when
//! rights in the property are conditioned (directly or indirectly)
//! on:
//! - The **future performance** of SUBSTANTIAL services by any
//!   person; OR
//! - The **occurrence of a condition** related to a purpose of
//!   the transfer (e.g., performance metric, change-of-control).
//!
//! AND the possibility of forfeiture is **substantial**. An hour
//! a week of services is NOT substantial; longer durations or
//! material job functions are.
//!
//! **§83(c)(2) transferability**: property is "transferable" only
//! when the transferee is not subject to a substantial risk of
//! forfeiture. Restricted stock that the holder cannot sell except
//! to other employees who would themselves be subject to the same
//! restrictions is NOT transferable.
//!
//! **§83(c)(3) section 16(b) restriction**: property held by a
//! § 16(b) "insider" (officer, director, 10%+ shareholder of a
//! § 12 SEA-registered company) is treated as subject to a
//! substantial risk of forfeiture AND non-transferable until the
//! earlier of: (a) the 6-month § 16(b) short-swing-profit period
//! expires, OR (b) sale at a profit would no longer trigger
//! § 16(b) suit. Other insider-trading restrictions (Rule 10b-5,
//! lock-up agreements) do NOT delay §83 recognition.
//!
//! **Treas. Reg. § 1.83-3(c)** elaboration: substantial risk of
//! forfeiture may be established ONLY through a service condition
//! or a transfer-purpose condition. The likelihood that the
//! forfeiture condition would occur AND that it would be enforced
//! both matter.
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 83](https://www.law.cornell.edu/uscode/text/26/83),
//! [Treas. Reg. § 1.83-3(c)](https://www.govinfo.gov/content/pkg/CFR-2013-title26-vol2/pdf/CFR-2013-title26-vol2-sec1-83-3.pdf),
//! [Venable LLP — Substantial Risk of Forfeiture under the IRC](https://www.venable.com/-/media/files/publications/2024/01/substantial-risk-of-forfeiture-under-the-irc.pdf),
//! [Proskauer — IRS Proposed Regs §83 Substantial Risk of Forfeiture](https://www.proskauer.com/alert/irs-issuesproposed-regulations-underinternal-revenue-code-section-83-regarding-substantial-risk-of-forfeiture-analysis),
//! [Tax Adviser — Sec. 83 Substantial Risk of Forfeiture Clarified](https://www.thetaxadviser.com/issues/2012/aug/newsnotes-aug2012-story-06/).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForfeitureConditionType {
    /// §83(c)(1) future-performance-of-services condition.
    FutureServicePerformance,
    /// §83(c)(1) condition related to a purpose of the transfer
    /// (performance metric, change-of-control, hurdle event).
    TransferPurposeCondition,
    /// §83(c)(3) §16(b) Securities Exchange Act 6-month
    /// short-swing-profit restriction (insider).
    Section16bShortSwingPeriod,
    /// No qualifying §83(c) condition.
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VestingStatus {
    /// Property is substantially vested under §83(a) — income
    /// recognition required.
    SubstantiallyVested,
    /// Property is unvested (substantial risk of forfeiture + not
    /// transferable) — no income recognition under §83(a) until
    /// vesting.
    SubstantialRiskOfForfeiture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section83cInput {
    pub condition_type: ForfeitureConditionType,
    /// True if the property is transferable under §83(c)(2) (i.e.,
    /// transferee not subject to substantial risk of forfeiture).
    pub property_transferable: bool,
    /// True if the future-service condition requires substantial
    /// services (not just nominal). An hour a week is NOT
    /// substantial; full-time job duties are.
    pub service_condition_is_substantial: bool,
    /// True if the possibility of forfeiture is substantial (not
    /// merely theoretical).
    pub forfeiture_possibility_is_substantial: bool,
    /// True if the transfer-purpose condition is reasonably likely
    /// to occur (e.g., performance metric is achievable / hurdle
    /// event is foreseeable).
    pub purpose_condition_likely_to_occur: bool,
    /// True if the forfeiture condition is reasonably likely to be
    /// enforced by the employer / transferor.
    pub forfeiture_condition_likely_enforced: bool,
    /// §83(c)(3): days remaining in the §16(b) 6-month
    /// short-swing-profit period. Zero if the period has expired
    /// or the holder is not a §16(b) insider.
    pub days_remaining_in_section_16b_period: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section83cResult {
    pub vesting_status: VestingStatus,
    /// True if a substantial risk of forfeiture exists under
    /// §83(c)(1).
    pub substantial_risk_of_forfeiture_exists: bool,
    /// True if the property is "transferable" under §83(c)(2).
    pub property_transferable: bool,
    /// True if §83(c)(3) §16(b) restriction is currently delaying
    /// vesting.
    pub section_16b_restriction_active: bool,
    /// True if §83(a) currently requires income recognition.
    pub income_recognition_required: bool,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section83cInput) -> Section83cResult {
    // §83(c)(3) §16(b) restriction — applies regardless of other
    // facts when an insider is within the 6-month window.
    let section_16b_active = matches!(
        input.condition_type,
        ForfeitureConditionType::Section16bShortSwingPeriod
    ) && input.days_remaining_in_section_16b_period > 0;

    // §83(c)(1) substantial risk of forfeiture analysis.
    let srf_exists = match input.condition_type {
        ForfeitureConditionType::FutureServicePerformance => {
            input.service_condition_is_substantial
                && input.forfeiture_possibility_is_substantial
                && input.forfeiture_condition_likely_enforced
        }
        ForfeitureConditionType::TransferPurposeCondition => {
            input.purpose_condition_likely_to_occur
                && input.forfeiture_possibility_is_substantial
                && input.forfeiture_condition_likely_enforced
        }
        ForfeitureConditionType::Section16bShortSwingPeriod => section_16b_active,
        ForfeitureConditionType::None => false,
    };

    // §83(c)(2) transferability — overridden by §83(c)(3) to false
    // during 16(b) period.
    let transferable = if section_16b_active {
        false
    } else {
        input.property_transferable
    };

    // §83(a) timing rule: substantially vested when EITHER
    // transferable OR no SRF.
    let substantially_vested = transferable || !srf_exists;

    let vesting_status = if substantially_vested {
        VestingStatus::SubstantiallyVested
    } else {
        VestingStatus::SubstantialRiskOfForfeiture
    };

    let income_recognition_required = substantially_vested;

    let condition_label = match input.condition_type {
        ForfeitureConditionType::FutureServicePerformance => {
            "§83(c)(1) future-performance-of-substantial-services condition"
        }
        ForfeitureConditionType::TransferPurposeCondition => {
            "§83(c)(1) condition related to transfer purpose"
        }
        ForfeitureConditionType::Section16bShortSwingPeriod => {
            "§83(c)(3) § 16(b) 6-month short-swing-profit restriction"
        }
        ForfeitureConditionType::None => "no §83(c) condition",
    };

    let note = format!(
        "Condition: {}; substantial risk of forfeiture: {}; transferable under §83(c)(2): {}; §16(b) restriction active: {} ({} days remaining); vesting status: {:?}; §83(a) income recognition required: {}.",
        condition_label,
        srf_exists,
        transferable,
        section_16b_active,
        input.days_remaining_in_section_16b_period,
        vesting_status,
        income_recognition_required,
    );

    Section83cResult {
        vesting_status,
        substantial_risk_of_forfeiture_exists: srf_exists,
        property_transferable: transferable,
        section_16b_restriction_active: section_16b_active,
        income_recognition_required,
        citation:
            "IRC §83(a) timing rule (recognize on the EARLIER of transferable or no-SRF); §83(c)(1) substantial risk of forfeiture defined by future-performance-of-substantial-services OR transfer-purpose-condition + substantial possibility of forfeiture + likelihood of enforcement; §83(c)(2) transferability requires transferee not subject to SRF; §83(c)(3) § 16(b) Securities Exchange Act 6-month short-swing-profit restriction treats property as subject to SRF AND non-transferable until earlier of 6-month expiry or no-§16(b)-suit-on-profit-sale; Treas. Reg. § 1.83-3(c) — only service condition or transfer-purpose condition + likelihood of occurrence AND enforcement"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_service() -> Section83cInput {
        Section83cInput {
            condition_type: ForfeitureConditionType::FutureServicePerformance,
            property_transferable: false,
            service_condition_is_substantial: true,
            forfeiture_possibility_is_substantial: true,
            purpose_condition_likely_to_occur: false,
            forfeiture_condition_likely_enforced: true,
            days_remaining_in_section_16b_period: 0,
        }
    }

    // ── §83(c)(1) future-service condition ─────────────────────────

    #[test]
    fn substantial_future_service_unvested() {
        let r = compute(&base_service());
        assert!(r.substantial_risk_of_forfeiture_exists);
        assert!(!r.property_transferable);
        assert_eq!(r.vesting_status, VestingStatus::SubstantialRiskOfForfeiture);
        assert!(!r.income_recognition_required);
    }

    #[test]
    fn one_hour_per_week_not_substantial_vested() {
        let mut i = base_service();
        i.service_condition_is_substantial = false;
        let r = compute(&i);
        assert!(!r.substantial_risk_of_forfeiture_exists);
        assert_eq!(r.vesting_status, VestingStatus::SubstantiallyVested);
    }

    #[test]
    fn non_substantial_forfeiture_possibility_vested() {
        let mut i = base_service();
        i.forfeiture_possibility_is_substantial = false;
        let r = compute(&i);
        assert!(!r.substantial_risk_of_forfeiture_exists);
    }

    #[test]
    fn employer_unlikely_to_enforce_vested() {
        let mut i = base_service();
        i.forfeiture_condition_likely_enforced = false;
        let r = compute(&i);
        assert!(!r.substantial_risk_of_forfeiture_exists);
    }

    // ── §83(c)(1) transfer-purpose condition ────────────────────────

    #[test]
    fn purpose_condition_likely_to_occur_unvested() {
        let mut i = base_service();
        i.condition_type = ForfeitureConditionType::TransferPurposeCondition;
        i.purpose_condition_likely_to_occur = true;
        let r = compute(&i);
        assert!(r.substantial_risk_of_forfeiture_exists);
    }

    #[test]
    fn purpose_condition_unlikely_vested() {
        let mut i = base_service();
        i.condition_type = ForfeitureConditionType::TransferPurposeCondition;
        i.purpose_condition_likely_to_occur = false;
        let r = compute(&i);
        assert!(!r.substantial_risk_of_forfeiture_exists);
    }

    // ── §83(c)(2) transferability override ─────────────────────────

    #[test]
    fn transferable_property_vested_even_with_srf() {
        // Per §83(a) — taxable when EITHER transferable OR no SRF.
        let mut i = base_service();
        i.property_transferable = true;
        let r = compute(&i);
        assert!(r.substantial_risk_of_forfeiture_exists);
        assert!(r.property_transferable);
        assert_eq!(r.vesting_status, VestingStatus::SubstantiallyVested);
    }

    // ── §83(c)(3) §16(b) restriction ───────────────────────────────

    #[test]
    fn section_16b_period_active_unvested() {
        let mut i = base_service();
        i.condition_type = ForfeitureConditionType::Section16bShortSwingPeriod;
        i.days_remaining_in_section_16b_period = 30;
        i.property_transferable = true; // would otherwise vest
        let r = compute(&i);
        assert!(r.section_16b_restriction_active);
        assert!(r.substantial_risk_of_forfeiture_exists);
        assert!(
            !r.property_transferable,
            "§83(c)(3) overrides transferable to false during 16(b) period"
        );
        assert_eq!(r.vesting_status, VestingStatus::SubstantialRiskOfForfeiture);
    }

    #[test]
    fn section_16b_period_expired_vested() {
        let mut i = base_service();
        i.condition_type = ForfeitureConditionType::Section16bShortSwingPeriod;
        i.days_remaining_in_section_16b_period = 0;
        i.property_transferable = true;
        let r = compute(&i);
        assert!(!r.section_16b_restriction_active);
        assert!(!r.substantial_risk_of_forfeiture_exists);
        assert!(r.property_transferable);
    }

    #[test]
    fn rule_10b_5_not_section_16b_no_srf() {
        // Rule 10b-5 insider trading restrictions are NOT §83(c)(3) —
        // only §16(b). With condition_type None, no SRF.
        let mut i = base_service();
        i.condition_type = ForfeitureConditionType::None;
        i.property_transferable = true;
        let r = compute(&i);
        assert!(!r.substantial_risk_of_forfeiture_exists);
        assert_eq!(r.vesting_status, VestingStatus::SubstantiallyVested);
    }

    // ── No-condition path ──────────────────────────────────────────

    #[test]
    fn no_condition_immediately_vested() {
        let mut i = base_service();
        i.condition_type = ForfeitureConditionType::None;
        i.property_transferable = true;
        let r = compute(&i);
        assert_eq!(r.vesting_status, VestingStatus::SubstantiallyVested);
        assert!(r.income_recognition_required);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base_service());
        assert!(r.citation.contains("§83(a)"));
        assert!(r.citation.contains("§83(c)(1)"));
        assert!(r.citation.contains("§83(c)(2)"));
        assert!(r.citation.contains("§83(c)(3)"));
        assert!(r.citation.contains("§ 16(b)"));
        assert!(r.citation.contains("§ 1.83-3(c)"));
        assert!(r.citation.contains("EARLIER"));
        assert!(r.citation.contains("6-month"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_describes_service_condition_path() {
        let r = compute(&base_service());
        assert!(r
            .note
            .contains("§83(c)(1) future-performance-of-substantial-services"));
    }

    #[test]
    fn note_describes_section_16b_path() {
        let mut i = base_service();
        i.condition_type = ForfeitureConditionType::Section16bShortSwingPeriod;
        i.days_remaining_in_section_16b_period = 30;
        let r = compute(&i);
        assert!(r.note.contains("§83(c)(3) § 16(b)"));
        assert!(r.note.contains("30 days remaining"));
    }

    #[test]
    fn note_describes_purpose_condition_path() {
        let mut i = base_service();
        i.condition_type = ForfeitureConditionType::TransferPurposeCondition;
        let r = compute(&i);
        assert!(r
            .note
            .contains("§83(c)(1) condition related to transfer purpose"));
    }
}
