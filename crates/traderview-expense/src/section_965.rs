//! IRC § 965 — Transition Tax / Mandatory Repatriation Tax (MRT).
//!
//! Enacted by Tax Cuts and Jobs Act of 2017 (Pub. L. 115-97 § 14103) as a
//! ONE-TIME deemed-repatriation tax on accumulated post-1986 deferred
//! foreign earnings of CFCs and other 10%-or-greater foreign corporations
//! ("specified foreign corporations" — SFCs). The transition tax applies
//! in the LAST taxable year of the SFC beginning before January 1, 2018.
//!
//! § 965(a): inclusion in US shareholder's gross income for the inclusion
//! year of the accumulated post-1986 deferred foreign income of each SFC
//! determined as of November 2, 2017 OR December 31, 2017 (whichever is
//! GREATER per § 965(a)(2)).
//!
//! § 965(c): rate differentiation via a deduction equal to (1) "8% deduction
//! percentage" applied to the cash-position portion of inclusion, yielding
//! a 15.5% effective rate, plus (2) "non-cash deduction percentage" applied
//! to the non-cash portion, yielding an 8% effective rate.
//!
//! § 965(b): a US shareholder's aggregate inclusion is REDUCED by the
//! shareholder's aggregate E&P DEFICIT of other SFCs — netting across the
//! shareholder's CFC portfolio.
//!
//! § 965(c)(3) cash position: defined as cash plus net accounts receivable
//! plus actively traded property held for investment plus short-term
//! obligations; determined at three measurement dates (November 2, 2017,
//! December 31, 2017, and end of inclusion year) — the GREATEST of (1) the
//! average of the two pre-inclusion-year measurements or (2) the inclusion-
//! year measurement.
//!
//! § 965(h) 8-YEAR INSTALLMENT ELECTION: US shareholder may elect to pay
//! the net tax liability under § 965 in installments over 8 years on the
//! following schedule: 8% in years 1-5, 15% in year 6, 20% in year 7, 25%
//! in year 8. Default is single-year payment.
//!
//! § 965(i): S corporation shareholder election to DEFER liability until
//! triggering event (S corp ceases to be S corp, sale of substantially
//! all assets, liquidation, transfer of S corp stock).
//!
//! § 965(m): REIT election to spread inclusion ratably over 8-year period
//! at the REIT level.
//!
//! § 965(d): accumulated post-1986 deferred foreign income — earnings and
//! profits accumulated since 1986 that have not previously been (1)
//! distributed, (2) included in income under § 951 Subpart F, or (3)
//! effectively connected to a US trade or business.
//!
//! § 965(o) cash position rules and currency translation.
//!
//! Moore v. United States, 602 U.S. ___ (June 20, 2024): Supreme Court 7-2
//! decision authored by Justice Kavanaugh UPHELD constitutionality of §
//! 965 mandatory repatriation tax under the Sixteenth Amendment. Holding
//! NARROW: limited to entities treated as pass-throughs; does not decide
//! whether realization is a constitutional requirement; does not address
//! wealth taxes, mark-to-market taxes, or taxes on appreciation.
//!
//! Inclusion creates § 959(c)(2) PTEP — distributions of § 965-included
//! E&P qualify for § 959(a)(1) exclusion (see `section_959` iter 512
//! sixteen-basket framework — § 965 PTEP is one of the seven § 959(c)(2)
//! groups).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShareholderType {
    /// Domestic C corporation.
    DomesticCCorporation,
    /// US individual / estate / trust.
    USIndividualEstateTrust,
    /// S corporation (may elect § 965(i) deferral).
    SCorporationElectingDeferral,
    /// REIT (may elect § 965(m) 8-year spread).
    RealEstateInvestmentTrust,
    /// Less-than-10% shareholder — § 965 inapplicable.
    LessThan10PctNotUsShareholder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentElection {
    /// Default — single-year payment of full liability.
    SingleYearFull,
    /// § 965(h) 8-year installment election.
    EightYearInstallment,
    /// § 965(i) S corp deferral until triggering event.
    SCorpDeferralUntilTriggeringEvent,
    /// § 965(m) REIT 8-year ratable inclusion.
    ReitRatableInclusionEightYear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NotUsShareholderNoInclusion,
    SingleYearPaymentFull,
    EightYearInstallmentScheduleAdopted,
    SCorpDeferralActive,
    ReitRatableSpreadActive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section965Input {
    pub shareholder_type: ShareholderType,
    pub payment_election: PaymentElection,
    /// Pro-rata share of accumulated post-1986 deferred foreign income
    /// (greater of November 2, 2017 or December 31, 2017 measurement) in
    /// cents per § 965(a)(2).
    pub aggregate_inclusion_cents: u64,
    /// US shareholder's pro-rata share of cash-position portion of
    /// inclusion (taxed at 15.5%) in cents.
    pub cash_position_portion_cents: u64,
    /// Aggregate E&P deficit of other SFCs of the US shareholder available
    /// to offset inclusion under § 965(b) in cents.
    pub aggregate_ep_deficit_cents: u64,
    /// Installment year (1 through 8) for installment-schedule computation;
    /// 0 for non-installment.
    pub installment_year: u32,
    /// Whether S corp triggering event has occurred (§ 965(i) deferral
    /// terminates).
    pub s_corp_triggering_event_occurred: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section965Result {
    pub severity: Severity,
    pub net_inclusion_after_ep_deficit_cents: u64,
    pub cash_portion_at_15_5_pct_cents: u64,
    pub non_cash_portion_at_8_pct_cents: u64,
    pub total_transition_tax_cents: u64,
    pub current_installment_amount_cents: u64,
    pub current_installment_pct_bps: u32,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const TCJA_PUB_L: &str = "Pub. L. 115-97";
pub const SECTION_965_LAST_INCLUSION_YEAR_BEFORE: i32 = 2018;
pub const CASH_RATE_BPS: u32 = 1_550;
pub const NON_CASH_RATE_BPS: u32 = 800;
pub const INSTALLMENT_YEARS: u32 = 8;
pub const MOORE_V_UNITED_STATES_DATE: &str = "2024-06-20";
pub const SECTION_965_INCLUSION_MEASUREMENT_DATE_1: &str = "2017-11-02";
pub const SECTION_965_INCLUSION_MEASUREMENT_DATE_2: &str = "2017-12-31";

pub fn check(input: &Section965Input) -> Section965Result {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if matches!(
        input.shareholder_type,
        ShareholderType::LessThan10PctNotUsShareholder
    ) {
        notes.push(
            "Less-than-10% shareholder is NOT a US shareholder under § 951(b); § 965 \
             transition tax does not apply. § 965 reaches only 10%-or-greater US shareholders \
             of specified foreign corporations (SFCs)."
                .to_string(),
        );
        return empty_result(
            Severity::NotUsShareholderNoInclusion,
            input,
            actions,
            notes,
            "26 U.S.C. § 951(b); § 965(a); § 965(e)",
        );
    }

    let net_inclusion = input
        .aggregate_inclusion_cents
        .saturating_sub(input.aggregate_ep_deficit_cents);

    let cash_portion = input
        .cash_position_portion_cents
        .min(net_inclusion);
    let non_cash_portion = net_inclusion.saturating_sub(cash_portion);

    let cash_tax: u64 =
        (u128::from(cash_portion) * u128::from(CASH_RATE_BPS) / 10_000) as u64;
    let non_cash_tax: u64 =
        (u128::from(non_cash_portion) * u128::from(NON_CASH_RATE_BPS) / 10_000) as u64;
    let total_tax = cash_tax.saturating_add(non_cash_tax);

    let (installment_pct_bps, current_installment) = match input.payment_election {
        PaymentElection::EightYearInstallment => match input.installment_year {
            1..=5 => (800, total_tax * 800 / 10_000),
            6 => (1_500, total_tax * 1_500 / 10_000),
            7 => (2_000, total_tax * 2_000 / 10_000),
            8 => (2_500, total_tax * 2_500 / 10_000),
            _ => (0, 0),
        },
        _ => (10_000, total_tax),
    };

    let severity = match input.payment_election {
        PaymentElection::SingleYearFull => Severity::SingleYearPaymentFull,
        PaymentElection::EightYearInstallment => {
            Severity::EightYearInstallmentScheduleAdopted
        }
        PaymentElection::SCorpDeferralUntilTriggeringEvent => {
            if input.s_corp_triggering_event_occurred {
                Severity::SingleYearPaymentFull
            } else {
                Severity::SCorpDeferralActive
            }
        }
        PaymentElection::ReitRatableInclusionEightYear => Severity::ReitRatableSpreadActive,
    };

    actions.push(format!(
        "§ 965 transition tax computation: aggregate inclusion of {} cents (greater of \
         November 2, 2017 or December 31, 2017 measurement under § 965(a)(2)); reduced by \
         § 965(b) aggregate E&P deficit of {} cents; net inclusion = {} cents. Cash-position \
         portion of {} cents taxed at 15.5% via § 965(c) deduction = {} cents; non-cash \
         portion of {} cents taxed at 8% = {} cents. Total transition tax = {} cents. \
         Report on Form 5471 plus Form 965 (Inclusion of Deferred Foreign Income) plus \
         IRC § 965 Transition Tax Statement attached to original return.",
        input.aggregate_inclusion_cents,
        input.aggregate_ep_deficit_cents,
        net_inclusion,
        cash_portion,
        cash_tax,
        non_cash_portion,
        non_cash_tax,
        total_tax
    ));

    if matches!(
        input.payment_election,
        PaymentElection::EightYearInstallment
    ) {
        actions.push(format!(
            "§ 965(h) 8-year installment election adopted: schedule is 8% years 1-5, 15% \
             year 6, 20% year 7, 25% year 8. Current installment year {} payment = {} cents \
             ({} bps of total). Annual interest computed at § 6601 underpayment rate; \
             acceleration on disposition of substantially all assets, liquidation, or \
             cessation of US shareholder status per § 965(h)(3) acceleration event.",
            input.installment_year, current_installment, installment_pct_bps
        ));
    } else if matches!(
        input.payment_election,
        PaymentElection::SCorpDeferralUntilTriggeringEvent
    ) {
        actions.push(
            "§ 965(i) S corporation deferral election: liability deferred until triggering \
             event (S corp ceases to be S corp, transfer of substantially all assets, \
             liquidation, or transfer of S corp stock by electing shareholder). If \
             triggering event occurs, S corp shareholder may further elect § 965(h) 8-year \
             installment from triggering event date."
                .to_string(),
        );
    } else if matches!(
        input.payment_election,
        PaymentElection::ReitRatableInclusionEightYear
    ) {
        actions.push(
            "§ 965(m) REIT election: inclusion spread ratably over 8-year period beginning \
             with the inclusion year. Each year's REIT-level inclusion = 1/8 of total \
             aggregate inclusion. Coordination with § 857 REIT distribution requirement — \
             ratable spread permits REIT to satisfy 90% distribution requirement gradually \
             rather than in one year."
                .to_string(),
        );
    }

    notes.push(format!(
        "Moore v. United States, 602 U.S. ___ ({}): Supreme Court 7-2 decision authored by \
         Justice Kavanaugh UPHELD constitutionality of § 965 mandatory repatriation tax \
         under Sixteenth Amendment. Holding NARROW: limited to entities treated as pass-\
         throughs; does not decide whether realization is constitutional requirement; does \
         not address wealth taxes, mark-to-market taxes, or taxes on appreciation.",
        MOORE_V_UNITED_STATES_DATE
    ));

    notes.push(format!(
        "§ 965 enacted by {} § 14103 — applied in last taxable year of SFC beginning before \
         January 1, {}. Inclusion measurement at later of two dates per § 965(a)(2): {} \
         (TCJA passage) or {} (end of pre-TCJA tax year). Inclusion creates § 959(c)(2) \
         PTEP — distributions of § 965-included E&P qualify for § 959(a)(1) exclusion (see \
         [[section_959]] iter 512 sixteen-basket framework; § 965 PTEP is one of the seven \
         § 959(c)(2) groups). Coordination with [[section_951]] (Subpart F), [[section_951a]] \
         (GILTI/NCTI — iter 500), [[section_956]] (US property investment — iter 504), \
         [[section_959]] (PTEP — iter 512), [[section_962]] (individual corporate-rate \
         election — iter 510), [[section_245a]] (foreign-source DRD pathway — iter 502 — \
         note that § 245A inapplicable to § 965 inclusion since pre-TCJA E&P), [[section_904]] \
         (FTC limitation baskets — § 965 inclusion has separate § 904(d) basket), \
         [[section_960]] (deemed-paid FTC — only 55.7% / 77.1% of FTC creditable against § \
         965 tax via § 965(g) FTC denial percentage).",
        TCJA_PUB_L,
        SECTION_965_LAST_INCLUSION_YEAR_BEFORE,
        SECTION_965_INCLUSION_MEASUREMENT_DATE_1,
        SECTION_965_INCLUSION_MEASUREMENT_DATE_2
    ));

    Section965Result {
        severity,
        net_inclusion_after_ep_deficit_cents: net_inclusion,
        cash_portion_at_15_5_pct_cents: cash_tax,
        non_cash_portion_at_8_pct_cents: non_cash_tax,
        total_transition_tax_cents: total_tax,
        current_installment_amount_cents: current_installment,
        current_installment_pct_bps: installment_pct_bps,
        recommended_actions: actions,
        citation: "26 U.S.C. § 965(a)-(o); Pub. L. 115-97 § 14103; Moore v. United States, 602 U.S. ___ (2024)",
        notes,
    }
}

fn empty_result(
    severity: Severity,
    input: &Section965Input,
    recommended_actions: Vec<String>,
    mut notes: Vec<String>,
    citation: &'static str,
) -> Section965Result {
    notes.push(
        "Coordination with [[section_951]] (Subpart F), [[section_951a]] (GILTI / NCTI), \
         [[section_956]] (US property), [[section_959]] (PTEP — iter 512 sixteen-basket), \
         [[section_962]] (election), [[section_245a]] (DRD pathway), [[section_904]] \
         (FTC baskets), [[section_960]] (deemed-paid FTC)."
            .to_string(),
    );
    let _ = input;
    Section965Result {
        severity,
        net_inclusion_after_ep_deficit_cents: 0,
        cash_portion_at_15_5_pct_cents: 0,
        non_cash_portion_at_8_pct_cents: 0,
        total_transition_tax_cents: 0,
        current_installment_amount_cents: 0,
        current_installment_pct_bps: 0,
        recommended_actions,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section965Input {
        Section965Input {
            shareholder_type: ShareholderType::DomesticCCorporation,
            payment_election: PaymentElection::SingleYearFull,
            aggregate_inclusion_cents: 100_000_000_00,
            cash_position_portion_cents: 60_000_000_00,
            aggregate_ep_deficit_cents: 0,
            installment_year: 0,
            s_corp_triggering_event_occurred: false,
        }
    }

    #[test]
    fn less_than_10_pct_not_us_shareholder() {
        let mut i = baseline();
        i.shareholder_type = ShareholderType::LessThan10PctNotUsShareholder;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotUsShareholderNoInclusion));
        assert_eq!(r.total_transition_tax_cents, 0);
    }

    #[test]
    fn single_year_payment_full_path() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::SingleYearPaymentFull));
        assert_eq!(r.current_installment_pct_bps, 10_000);
    }

    #[test]
    fn cash_position_taxed_at_15_5_pct() {
        let i = baseline();
        let r = check(&i);
        let expected_cash_tax = 60_000_000_00u64 * 1_550 / 10_000;
        assert_eq!(r.cash_portion_at_15_5_pct_cents, expected_cash_tax);
    }

    #[test]
    fn non_cash_portion_taxed_at_8_pct() {
        let i = baseline();
        let r = check(&i);
        let expected_non_cash_tax = 40_000_000_00u64 * 800 / 10_000;
        assert_eq!(r.non_cash_portion_at_8_pct_cents, expected_non_cash_tax);
    }

    #[test]
    fn total_transition_tax_correctly_summed() {
        let i = baseline();
        let r = check(&i);
        assert_eq!(
            r.total_transition_tax_cents,
            r.cash_portion_at_15_5_pct_cents + r.non_cash_portion_at_8_pct_cents
        );
    }

    #[test]
    fn ep_deficit_reduces_aggregate_inclusion() {
        let mut i = baseline();
        i.aggregate_ep_deficit_cents = 30_000_000_00;
        let r = check(&i);
        assert_eq!(r.net_inclusion_after_ep_deficit_cents, 70_000_000_00);
    }

    #[test]
    fn ep_deficit_exceeding_inclusion_saturates_at_zero() {
        let mut i = baseline();
        i.aggregate_ep_deficit_cents = 200_000_000_00;
        let r = check(&i);
        assert_eq!(r.net_inclusion_after_ep_deficit_cents, 0);
        assert_eq!(r.total_transition_tax_cents, 0);
    }

    #[test]
    fn installment_year_1_pins_8_pct() {
        let mut i = baseline();
        i.payment_election = PaymentElection::EightYearInstallment;
        i.installment_year = 1;
        let r = check(&i);
        assert_eq!(r.current_installment_pct_bps, 800);
    }

    #[test]
    fn installment_year_5_pins_8_pct() {
        let mut i = baseline();
        i.payment_election = PaymentElection::EightYearInstallment;
        i.installment_year = 5;
        let r = check(&i);
        assert_eq!(r.current_installment_pct_bps, 800);
    }

    #[test]
    fn installment_year_6_pins_15_pct() {
        let mut i = baseline();
        i.payment_election = PaymentElection::EightYearInstallment;
        i.installment_year = 6;
        let r = check(&i);
        assert_eq!(r.current_installment_pct_bps, 1_500);
    }

    #[test]
    fn installment_year_7_pins_20_pct() {
        let mut i = baseline();
        i.payment_election = PaymentElection::EightYearInstallment;
        i.installment_year = 7;
        let r = check(&i);
        assert_eq!(r.current_installment_pct_bps, 2_000);
    }

    #[test]
    fn installment_year_8_pins_25_pct() {
        let mut i = baseline();
        i.payment_election = PaymentElection::EightYearInstallment;
        i.installment_year = 8;
        let r = check(&i);
        assert_eq!(r.current_installment_pct_bps, 2_500);
    }

    #[test]
    fn installment_schedule_sums_to_100_pct() {
        let total: u32 = 5 * 800 + 1_500 + 2_000 + 2_500;
        assert_eq!(total, 10_000);
    }

    #[test]
    fn s_corp_deferral_no_triggering_event_active() {
        let mut i = baseline();
        i.shareholder_type = ShareholderType::SCorporationElectingDeferral;
        i.payment_election = PaymentElection::SCorpDeferralUntilTriggeringEvent;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::SCorpDeferralActive));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("§ 965(i)")));
    }

    #[test]
    fn s_corp_deferral_with_triggering_event_full_payment() {
        let mut i = baseline();
        i.shareholder_type = ShareholderType::SCorporationElectingDeferral;
        i.payment_election = PaymentElection::SCorpDeferralUntilTriggeringEvent;
        i.s_corp_triggering_event_occurred = true;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::SingleYearPaymentFull));
    }

    #[test]
    fn reit_8_year_ratable_spread_active() {
        let mut i = baseline();
        i.shareholder_type = ShareholderType::RealEstateInvestmentTrust;
        i.payment_election = PaymentElection::ReitRatableInclusionEightYear;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ReitRatableSpreadActive));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 965(m)")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 857")));
    }

    #[test]
    fn cash_rate_pins_15_5_pct() {
        assert_eq!(CASH_RATE_BPS, 1_550);
    }

    #[test]
    fn non_cash_rate_pins_8_pct() {
        assert_eq!(NON_CASH_RATE_BPS, 800);
    }

    #[test]
    fn installment_years_pins_8() {
        assert_eq!(INSTALLMENT_YEARS, 8);
    }

    #[test]
    fn moore_decision_date_pins_2024_06_20() {
        assert_eq!(MOORE_V_UNITED_STATES_DATE, "2024-06-20");
    }

    #[test]
    fn measurement_date_1_pins_2017_11_02() {
        assert_eq!(SECTION_965_INCLUSION_MEASUREMENT_DATE_1, "2017-11-02");
    }

    #[test]
    fn measurement_date_2_pins_2017_12_31() {
        assert_eq!(SECTION_965_INCLUSION_MEASUREMENT_DATE_2, "2017-12-31");
    }

    #[test]
    fn last_inclusion_year_before_pins_2018() {
        assert_eq!(SECTION_965_LAST_INCLUSION_YEAR_BEFORE, 2018);
    }

    #[test]
    fn tcja_pub_l_pins_115_97() {
        assert_eq!(TCJA_PUB_L, "Pub. L. 115-97");
    }

    #[test]
    fn note_pins_moore_v_united_states_decision() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Moore v. United States")));
        assert!(r.notes.iter().any(|n| n.contains("2024-06-20")));
        assert!(r.notes.iter().any(|n| n.contains("Kavanaugh")));
        assert!(r.notes.iter().any(|n| n.contains("Sixteenth Amendment")));
    }

    #[test]
    fn note_pins_section_959_ptep_creation() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_959")));
        assert!(r.notes.iter().any(|n| n.contains("§ 959(c)(2) PTEP")));
        assert!(r.notes.iter().any(|n| n.contains("section_951a")));
        assert!(r.notes.iter().any(|n| n.contains("section_960")));
    }

    #[test]
    fn citation_pins_pub_l_115_97_14103_and_moore() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 965(a)-(o)"));
        assert!(r.citation.contains("Pub. L. 115-97 § 14103"));
        assert!(r.citation.contains("Moore v. United States"));
    }

    #[test]
    fn action_references_form_5471_and_form_965() {
        let i = baseline();
        let r = check(&i);
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 5471")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 965")));
    }

    #[test]
    fn cash_position_portion_capped_at_net_inclusion() {
        let mut i = baseline();
        i.cash_position_portion_cents = 200_000_000_00;
        i.aggregate_inclusion_cents = 100_000_000_00;
        let r = check(&i);
        let expected_cash_tax = 100_000_000_00u64 * 1_550 / 10_000;
        assert_eq!(r.cash_portion_at_15_5_pct_cents, expected_cash_tax);
        assert_eq!(r.non_cash_portion_at_8_pct_cents, 0);
    }

    #[test]
    fn extreme_inclusion_does_not_overflow() {
        let mut i = baseline();
        i.aggregate_inclusion_cents = u64::MAX / 100;
        i.cash_position_portion_cents = u64::MAX / 100;
        let r = check(&i);
        let _ = r.total_transition_tax_cents;
    }

    #[test]
    fn zero_inclusion_zero_tax() {
        let mut i = baseline();
        i.aggregate_inclusion_cents = 0;
        i.cash_position_portion_cents = 0;
        let r = check(&i);
        assert_eq!(r.total_transition_tax_cents, 0);
    }
}
