//! IRC §444 — Election of taxable year other than required taxable
//! year (fiscal year election).
//!
//! Allows a partnership, S corporation, or personal service
//! corporation (PSC) to elect a non-calendar fiscal year, subject to
//! a strict 3-month deferral cap and a §7519 "required payment"
//! mechanism that approximates the tax benefit of the deferral.
//! Useful for businesses with strongly seasonal income patterns
//! (e.g., agricultural partnerships, retail S-corps with December
//! peak) that benefit from matching tax year to natural business
//! cycle ([Cornell LII 26 U.S.C. § 444](https://www.law.cornell.edu/uscode/text/26/444),
//! [Cornell LII 26 CFR § 1.444-1T](https://www.law.cornell.edu/cfr/text/26/1.444-1T)).
//!
//! **§444(a) election availability** — three eligible entity types:
//!
//! - **Partnerships** (the required tax year is typically the
//!   majority-partner tax year, often calendar)
//! - **S corporations** (required tax year = calendar)
//! - **Personal service corporations (PSCs)** (required tax year =
//!   calendar)
//!
//! **§444(b) deferral period limit — 3 months maximum**: the
//! deferral period is the number of months between the end of the
//! elected fiscal year and the end of the required tax year. For
//! entities whose required year is the calendar year (ending
//! December 31), the only fiscal year-ends that satisfy the 3-month
//! cap are:
//!
//! | Elected fiscal year end | Deferral months | Eligible? |
//! |-------------------------|-----------------|-----------|
//! | September 30            | 3               | YES       |
//! | October 31              | 2               | YES       |
//! | November 30             | 1               | YES       |
//! | August 31               | 4               | NO        |
//! | December 31             | 0 (= required)  | No election needed |
//!
//! **§7519 required payment** (partnerships and S-corps): an annual
//! "deposit" approximating the deferral benefit; due May 15 of the
//! calendar year following the election year. Filed on Form 8752.
//! Formula approximates: `applicable_election_year_income ×
//! deferral_ratio × (highest §1 rate + 1%)`. Where deferral ratio =
//! deferral_months / 12.
//!
//! **§280H deduction limitations** (PSCs only): instead of a §7519
//! required payment, a PSC is subject to limitations on deductions
//! for amounts paid to employee-owners during the deferral period
//! that exceed an applicable percentage of the prior-year base.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Partnership,
    SCorporation,
    PersonalServiceCorporation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section444Input {
    pub entity_type: EntityType,
    /// Required tax year end month — typically 12 (December calendar
    /// year) for partnerships, S-corps, and PSCs.
    pub required_tax_year_end_month: u32,
    /// Proposed fiscal year end month (1 = January, 9 = September,
    /// etc.).
    pub proposed_fiscal_year_end_month: u32,
    /// For § 7519 required payment computation (partnerships and
    /// S-corps): the entity's net income for the election year.
    pub net_income_for_election_year_dollars: i64,
    /// Highest § 1 individual rate in basis points (3700 = 37% for
    /// 2018+). § 7519 uses this + 1%.
    pub highest_section_1_rate_bp: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section444Result {
    pub deferral_period_months: u32,
    pub deferral_period_within_3_month_limit: bool,
    pub election_available: bool,
    pub required_payment_under_7519_dollars: i64,
    pub subject_to_280h_deduction_limits: bool,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section444Input) -> Section444Result {
    // Deferral period = months between elected fiscal year end and
    // required tax year end.
    let required_end = input.required_tax_year_end_month;
    let proposed_end = input.proposed_fiscal_year_end_month;
    let deferral = if proposed_end == required_end {
        0
    } else if proposed_end < required_end {
        required_end - proposed_end
    } else {
        // Wraps around: fiscal year end after required year end
        // (e.g., required Dec / proposed Jan = 11 months deferral).
        12 - (proposed_end - required_end)
    };

    let within_3_months = deferral > 0 && deferral <= 3;
    let available = within_3_months;

    // §7519 required payment for partnerships and S-corps.
    let required_payment = match input.entity_type {
        EntityType::Partnership | EntityType::SCorporation if available => {
            // Simplified: net income × (deferral / 12) × (rate + 1%).
            let rate_plus_1 = input.highest_section_1_rate_bp.saturating_add(100);
            let payment = (input.net_income_for_election_year_dollars as i128)
                * (deferral as i128)
                * (rate_plus_1 as i128)
                / (12i128 * 10_000);
            payment as i64
        }
        _ => 0,
    };

    let subject_to_280h = matches!(input.entity_type, EntityType::PersonalServiceCorporation)
        && available;

    let note = if !within_3_months {
        if deferral == 0 {
            format!(
                "Proposed fiscal year (month {}) equals required tax year — no §444 election needed. Election not applicable.",
                proposed_end,
            )
        } else {
            format!(
                "§444(b) DEFERRAL LIMIT EXCEEDED: proposed fiscal year (month {}) yields {} months deferral; only 3-month maximum permitted. Election NOT available — entity must use required tax year (month {}).",
                proposed_end, deferral, required_end,
            )
        }
    } else {
        let mech = match input.entity_type {
            EntityType::Partnership | EntityType::SCorporation => format!(
                "§7519 required payment ${} due May 15 following election year (Form 8752); approximates {} months × ({}.{}% rate + 1%) deferral benefit",
                required_payment,
                deferral,
                input.highest_section_1_rate_bp / 100,
                input.highest_section_1_rate_bp % 100,
            ),
            EntityType::PersonalServiceCorporation =>
                "§280H deduction limitations apply: PSC payments to employee-owners during deferral period subject to applicable-percentage limit".to_string(),
        };
        format!(
            "§444 election AVAILABLE: {:?} elects fiscal year ending month {} ({} months deferral); {}.",
            input.entity_type, proposed_end, deferral, mech,
        )
    };

    Section444Result {
        deferral_period_months: deferral,
        deferral_period_within_3_month_limit: within_3_months,
        election_available: available,
        required_payment_under_7519_dollars: required_payment,
        subject_to_280h_deduction_limits: subject_to_280h,
        citation:
            "IRC §444(a) fiscal year election for partnerships / S-corps / PSCs; §444(b)(2) 3-month deferral cap; §7519 required payment for partnerships and S-corps (Form 8752, due May 15); §280H deduction limitations for PSCs; Treas. Reg. §1.444-1T"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn partnership_base() -> Section444Input {
        Section444Input {
            entity_type: EntityType::Partnership,
            required_tax_year_end_month: 12,
            proposed_fiscal_year_end_month: 9,
            net_income_for_election_year_dollars: 1_000_000,
            highest_section_1_rate_bp: 3700, // 37%
        }
    }

    // Deferral period boundary.

    #[test]
    fn sept_30_election_3_month_deferral_eligible() {
        let r = compute(&partnership_base());
        assert_eq!(r.deferral_period_months, 3);
        assert!(r.deferral_period_within_3_month_limit);
        assert!(r.election_available);
    }

    #[test]
    fn oct_31_election_2_month_deferral_eligible() {
        let mut i = partnership_base();
        i.proposed_fiscal_year_end_month = 10;
        let r = compute(&i);
        assert_eq!(r.deferral_period_months, 2);
        assert!(r.election_available);
    }

    #[test]
    fn nov_30_election_1_month_deferral_eligible() {
        let mut i = partnership_base();
        i.proposed_fiscal_year_end_month = 11;
        let r = compute(&i);
        assert_eq!(r.deferral_period_months, 1);
        assert!(r.election_available);
    }

    #[test]
    fn aug_31_election_4_month_deferral_violates() {
        let mut i = partnership_base();
        i.proposed_fiscal_year_end_month = 8;
        let r = compute(&i);
        assert_eq!(r.deferral_period_months, 4);
        assert!(!r.deferral_period_within_3_month_limit);
        assert!(!r.election_available);
        assert!(r.note.contains("DEFERRAL LIMIT EXCEEDED"));
    }

    #[test]
    fn dec_31_proposed_equals_required_no_election_needed() {
        let mut i = partnership_base();
        i.proposed_fiscal_year_end_month = 12;
        let r = compute(&i);
        assert_eq!(r.deferral_period_months, 0);
        assert!(!r.election_available);
        assert!(r.note.contains("no §444 election needed"));
    }

    #[test]
    fn jan_31_wraps_to_11_month_deferral_violates() {
        let mut i = partnership_base();
        i.proposed_fiscal_year_end_month = 1;
        let r = compute(&i);
        assert_eq!(r.deferral_period_months, 11);
        assert!(!r.election_available);
    }

    // § 7519 required payment.

    #[test]
    fn partnership_7519_payment_calculation() {
        // $1M net income × 3/12 × 38% (37%+1%) = $95,000.
        let r = compute(&partnership_base());
        assert_eq!(r.required_payment_under_7519_dollars, 95_000);
    }

    #[test]
    fn s_corp_subject_to_7519_payment_same_as_partnership() {
        let mut i = partnership_base();
        i.entity_type = EntityType::SCorporation;
        let r = compute(&i);
        assert_eq!(r.required_payment_under_7519_dollars, 95_000);
    }

    #[test]
    fn psc_no_7519_payment_but_280h_applies() {
        let mut i = partnership_base();
        i.entity_type = EntityType::PersonalServiceCorporation;
        let r = compute(&i);
        assert_eq!(r.required_payment_under_7519_dollars, 0);
        assert!(r.subject_to_280h_deduction_limits);
        assert!(r.note.contains("§280H"));
    }

    #[test]
    fn no_7519_payment_when_election_not_available() {
        let mut i = partnership_base();
        i.proposed_fiscal_year_end_month = 8; // > 3-month deferral
        let r = compute(&i);
        assert_eq!(r.required_payment_under_7519_dollars, 0);
        assert!(!r.subject_to_280h_deduction_limits);
    }

    // Smaller deferral path.

    #[test]
    fn nov_election_1_month_yields_smaller_7519_payment() {
        // $1M × 1/12 × 38% ≈ $31,666.
        let mut i = partnership_base();
        i.proposed_fiscal_year_end_month = 11;
        let r = compute(&i);
        assert_eq!(r.required_payment_under_7519_dollars, 31_666);
    }

    // Rate variations.

    #[test]
    fn higher_rate_yields_higher_7519_payment() {
        // Use 40% individual rate → 40% + 1% = 41% effective; $1M × 3/12 × 41% = $102,500.
        let mut i = partnership_base();
        i.highest_section_1_rate_bp = 4000;
        let r = compute(&i);
        assert_eq!(r.required_payment_under_7519_dollars, 102_500);
    }

    // Notes / citations.

    #[test]
    fn note_describes_election_available_with_mechanism() {
        let r = compute(&partnership_base());
        assert!(r.note.contains("§444 election AVAILABLE"));
        assert!(r.note.contains("§7519 required payment"));
    }

    #[test]
    fn psc_note_describes_280h() {
        let mut i = partnership_base();
        i.entity_type = EntityType::PersonalServiceCorporation;
        let r = compute(&i);
        assert!(r.note.contains("§280H"));
        assert!(r.note.contains("employee-owners"));
    }

    #[test]
    fn note_describes_deferral_limit_exceeded() {
        let mut i = partnership_base();
        i.proposed_fiscal_year_end_month = 6;
        let r = compute(&i);
        assert!(r.note.contains("DEFERRAL LIMIT EXCEEDED"));
        assert!(r.note.contains("3-month maximum"));
    }

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&partnership_base());
        assert!(r.citation.contains("§444(a)"));
        assert!(r.citation.contains("§444(b)(2)"));
        assert!(r.citation.contains("§7519"));
        assert!(r.citation.contains("§280H"));
        assert!(r.citation.contains("§1.444-1T"));
        assert!(r.citation.contains("Form 8752"));
        assert!(r.citation.contains("May 15"));
    }

    // Edge cases.

    #[test]
    fn zero_income_zero_7519_payment() {
        let mut i = partnership_base();
        i.net_income_for_election_year_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.required_payment_under_7519_dollars, 0);
    }

    #[test]
    fn very_large_income_precision_path() {
        let mut i = partnership_base();
        i.net_income_for_election_year_dollars = 1_000_000_000;
        let r = compute(&i);
        // $1B × 3/12 × 38% = $95M.
        assert_eq!(r.required_payment_under_7519_dollars, 95_000_000);
    }
}
