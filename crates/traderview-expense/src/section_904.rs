//! IRC § 904 — Limitation on Foreign Tax Credit (FTC).
//!
//! § 904 caps the foreign tax credit at the US tax that would otherwise be
//! imposed on the foreign-source taxable income. This prevents the FTC from
//! offsetting US tax on US-source income — only US tax attributable to
//! foreign-source income may be reduced by the FTC.
//!
//! § 904(a) FTC limitation formula: FTC ≤ US tax × (foreign-source TI /
//! total worldwide TI).
//!
//! § 904(d) SEPARATE BASKET RULE: limitation computed SEPARATELY for each
//! of the categories ("baskets") of income. Post-TCJA (effective for taxable
//! years beginning after December 31, 2017) the baskets are:
//!
//! - § 904(d)(1)(A) PASSIVE category — dividends, interest, royalties,
//!   rents, annuities (with high-tax kick-out and CFC look-through
//!   exceptions).
//! - § 904(d)(1)(B) GILTI / NCTI category (post-OBBBA renamed) — § 951A
//!   inclusions and associated § 960(d) deemed-paid FTC.
//! - § 904(d)(1)(C) FOREIGN BRANCH category — income attributable to one
//!   or more qualified business units of US person.
//! - § 904(d)(1)(D) GENERAL category — everything else (active business,
//!   wages, financial services, etc.).
//! - § 904(d)(6) treaty-resourced income — income re-sourced under treaty
//!   tie-breaker, separate basket.
//! - § 901(j) income from sanctioned countries — separate basket, deemed
//!   non-creditable for some sanctioned regimes.
//! - Lump-sum distributions from foreign pension plans — separate basket.
//!
//! § 904(c) CARRYOVER RULES: excess FTC in non-GILTI baskets carries back 1
//! year and forward 10 years within the SAME basket. § 904(c) carryback /
//! carryforward EXPLICITLY UNAVAILABLE for GILTI / NCTI category § 904(c)(1)
//! flush language — § 951A excess credits expire annually.
//!
//! § 904(f) OVERALL FOREIGN LOSS (OFL) recapture: when foreign-source losses
//! in one basket exceed foreign-source income in that basket, the loss is
//! allocated against US-source income; in later years foreign-source income
//! recharacterized as US-source until OFL recaptured.
//!
//! § 904(g) OVERALL DOMESTIC LOSS (ODL) recapture: parallel for domestic
//! losses against foreign-source income.
//!
//! OBBBA (Pub. L. 119-21, signed July 4, 2025, effective for taxable years
//! beginning after December 31, 2025) changes to § 904:
//! - Interest and R&D expense NO LONGER allocated to § 951A NCTI basket
//!   per § 864(e) clarification, preserving more foreign-source income for
//!   FTC purposes.
//! - § 960(d) deemed-paid FTC rises from 80% to 90% (10% haircut down from
//!   20%).
//!
//! Coordination: § 904 baskets carry through PTEP groups under `section_959`
//! sixteen-basket framework — § 959 PTEP groups are maintained within EACH
//! § 904(d) basket category.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BasketCategory {
    Passive,
    GiltiOrNcti,
    ForeignBranch,
    General,
    TreatyResourced,
    Section901jSanctioned,
    LumpSumDistribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    FullyCreditedNoExcess,
    PartiallyCreditedExcessCarriedForward,
    GiltiNctiExcessExpired,
    OverallForeignLossRecaptureTriggered,
    OverallDomesticLossRecaptureTriggered,
    Section901jSanctionedNonCreditable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section904Input {
    pub basket_category: BasketCategory,
    pub taxable_year: i32,
    /// Foreign-source taxable income in this basket in cents.
    pub foreign_source_ti_in_basket_cents: u64,
    /// Total worldwide taxable income in cents.
    pub total_worldwide_ti_cents: u64,
    /// US tax liability (regular tax pre-FTC) in cents.
    pub us_tax_liability_cents: u64,
    /// Foreign tax paid / accrued in this basket in cents.
    pub foreign_tax_paid_in_basket_cents: u64,
    /// Prior-year excess credit carryforward in this basket in cents.
    pub prior_year_excess_carryforward_cents: u64,
    /// Whether overall foreign loss (OFL) in this basket triggers § 904(f)
    /// recapture.
    pub ofl_recapture_triggered: bool,
    /// OFL recapture amount in cents.
    pub ofl_recapture_amount_cents: u64,
    /// Whether overall domestic loss (ODL) triggers § 904(g) recapture.
    pub odl_recapture_triggered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section904Result {
    pub severity: Severity,
    pub ftc_limitation_in_basket_cents: u64,
    pub total_foreign_tax_creditable_cents: u64,
    pub ftc_actually_used_cents: u64,
    pub excess_ftc_in_basket_cents: u64,
    pub excess_carries_back_1_year_cents: u64,
    pub excess_carries_forward_10_years_cents: u64,
    pub excess_expires_no_carryover_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const FTC_CARRYBACK_YEARS: u32 = 1;
pub const FTC_CARRYFORWARD_YEARS: u32 = 10;
pub const TCJA_POST_2017_BASKETS_COUNT: u32 = 7;
pub const OBBBA_EFFECTIVE_YEAR: i32 = 2026;

pub fn check(input: &Section904Input) -> Section904Result {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if matches!(input.basket_category, BasketCategory::Section901jSanctioned) {
        notes.push(
            "§ 901(j) income from sanctioned countries (Iran, North Korea, Syria, Cuba, \
             Sudan partial) — separate basket, foreign tax PAID generally NOT creditable \
             per § 901(j)(1) unless taxpayer demonstrates qualifying status under § \
             901(j)(2) or § 901(j)(5). Treaty resourcing under § 904(d)(6) may permit \
             partial creditability."
                .to_string(),
        );
        return Section904Result {
            severity: Severity::Section901jSanctionedNonCreditable,
            ftc_limitation_in_basket_cents: 0,
            total_foreign_tax_creditable_cents: 0,
            ftc_actually_used_cents: 0,
            excess_ftc_in_basket_cents: input.foreign_tax_paid_in_basket_cents,
            excess_carries_back_1_year_cents: 0,
            excess_carries_forward_10_years_cents: 0,
            excess_expires_no_carryover_cents: input.foreign_tax_paid_in_basket_cents,
            recommended_actions: actions,
            citation: "26 U.S.C. § 904(d)(1); § 901(j)",
            notes,
        };
    }

    if input.total_worldwide_ti_cents == 0 {
        return empty_result(
            Severity::NotApplicable,
            input,
            actions,
            notes,
            "26 U.S.C. § 904(a); no worldwide TI",
        );
    }

    if input.odl_recapture_triggered {
        actions.push(
            "Overall Domestic Loss (ODL) under § 904(g) — domestic losses in prior years \
             allocated against foreign-source income, recharacterizing some foreign-source \
             income as US-source in current year up to recapture amount. This INCREASES \
             FTC limitation in current year per § 904(g)(3) ODL recapture mechanism."
                .to_string(),
        );
    }

    let raw_limitation: u128 = u128::from(input.us_tax_liability_cents)
        * u128::from(input.foreign_source_ti_in_basket_cents)
        / u128::from(input.total_worldwide_ti_cents).max(1);
    let limitation = raw_limitation.min(u128::from(u64::MAX)) as u64;

    let total_creditable = input
        .foreign_tax_paid_in_basket_cents
        .saturating_add(input.prior_year_excess_carryforward_cents);

    let ftc_actually_used = total_creditable.min(limitation);
    let excess = total_creditable.saturating_sub(ftc_actually_used);

    let (carryback, carryforward, expires) = match input.basket_category {
        BasketCategory::GiltiOrNcti => (0u64, 0u64, excess),
        _ => (excess.saturating_div(2), excess.saturating_div(2), 0u64),
    };

    let severity = if input.ofl_recapture_triggered {
        Severity::OverallForeignLossRecaptureTriggered
    } else if input.odl_recapture_triggered {
        Severity::OverallDomesticLossRecaptureTriggered
    } else if excess == 0 {
        Severity::FullyCreditedNoExcess
    } else if matches!(input.basket_category, BasketCategory::GiltiOrNcti) {
        Severity::GiltiNctiExcessExpired
    } else {
        Severity::PartiallyCreditedExcessCarriedForward
    };

    actions.push(format!(
        "§ 904(a) FTC limitation in {:?} basket: ${} US tax × ({}/{}) foreign-source TI / \
         worldwide TI ratio = {} cents limitation. Total creditable = current-year foreign \
         tax {} plus prior-year carryforward {} = {} cents. FTC actually used = min(creditable, \
         limitation) = {} cents. Excess of {} cents {}. Report on Form 1116 (individual) or \
         Form 1118 (corporate); maintain separate computation per § 904(d) basket.",
        input.basket_category,
        input.us_tax_liability_cents,
        input.foreign_source_ti_in_basket_cents,
        input.total_worldwide_ti_cents,
        limitation,
        input.foreign_tax_paid_in_basket_cents,
        input.prior_year_excess_carryforward_cents,
        total_creditable,
        ftc_actually_used,
        excess,
        match input.basket_category {
            BasketCategory::GiltiOrNcti => {
                "EXPIRES (§ 951A excess credits ineligible for carryback or carryforward per \
                 § 904(c)(1) flush language)"
            }
            _ => {
                "carries back 1 year then forward 10 years within same basket per § 904(c)"
            }
        }
    ));

    if input.ofl_recapture_triggered {
        actions.push(format!(
            "§ 904(f) Overall Foreign Loss (OFL) recapture triggered: prior-year foreign-\
             source losses in this basket of {} cents were allocated against US-source \
             income. Current year foreign-source income recharacterized as US-source up to \
             recapture amount, REDUCING the FTC limitation in this basket until OFL is fully \
             recaptured. Track OFL account per Form 1118 Schedule J.",
            input.ofl_recapture_amount_cents
        ));
    }

    notes.push(format!(
        "Post-TCJA (effective taxable years beginning after December 31, 2017) {} separate \
         § 904(d) baskets: Passive, GILTI/NCTI (post-OBBBA renamed), Foreign Branch, \
         General, Treaty-Resourced, § 901(j) Sanctioned, Lump-Sum Distribution. OBBBA \
         (Pub. L. 119-21, effective for taxable years beginning after December 31, {}) \
         removes interest plus R&D expense allocation to § 951A basket per § 864(e) \
         clarification, preserving more foreign-source income for FTC; § 960(d) deemed-paid \
         FTC rate rises from 80%% to 90%% (10%% haircut down from 20%%).",
        TCJA_POST_2017_BASKETS_COUNT,
        OBBBA_EFFECTIVE_YEAR - 1
    ));

    notes.push(
        "Coordination with [[section_901]] (general FTC), [[section_960]] (deemed-paid FTC \
         for CFC inclusions — pre-OBBBA 80% / post-OBBBA 90%), [[section_951a]] (GILTI / \
         NCTI inclusion — iter 500 — feeds GILTI basket), [[section_956]] (CFC US property \
         investment — iter 504), [[section_959]] (PTEP — iter 512 sixteen-basket framework \
         — PTEP groups maintained WITHIN each § 904(d) basket), [[section_962]] (individual \
         corporate-rate election — iter 510 — unlocks § 960 FTC for individuals), \
         [[section_245a]] (foreign-source DRD — iter 502 — § 245A(d) DISALLOWS FTC for \
         foreign tax on DRD-eligible dividend), [[section_965]] (transition tax — iter 514 \
         — § 965(g) FTC denial percentage 55.7% / 77.1%), [[section_59a]] (BEAT separate \
         regime), [[section_864e]] (expense allocation rules)."
            .to_string(),
    );

    Section904Result {
        severity,
        ftc_limitation_in_basket_cents: limitation,
        total_foreign_tax_creditable_cents: total_creditable,
        ftc_actually_used_cents: ftc_actually_used,
        excess_ftc_in_basket_cents: excess,
        excess_carries_back_1_year_cents: carryback,
        excess_carries_forward_10_years_cents: carryforward,
        excess_expires_no_carryover_cents: expires,
        recommended_actions: actions,
        citation: "26 U.S.C. § 904(a)-(j); § 901; § 960; Pub. L. 115-97; Pub. L. 119-21",
        notes,
    }
}

fn empty_result(
    severity: Severity,
    input: &Section904Input,
    recommended_actions: Vec<String>,
    mut notes: Vec<String>,
    citation: &'static str,
) -> Section904Result {
    notes.push(
        "Coordination with [[section_901]] (FTC), [[section_960]] (deemed-paid), \
         [[section_951a]] (GILTI/NCTI), [[section_956]] (CFC US property), [[section_959]] \
         (PTEP), [[section_962]] (election), [[section_245a]] (DRD), [[section_965]] \
         (transition tax)."
            .to_string(),
    );
    let _ = input;
    Section904Result {
        severity,
        ftc_limitation_in_basket_cents: 0,
        total_foreign_tax_creditable_cents: 0,
        ftc_actually_used_cents: 0,
        excess_ftc_in_basket_cents: 0,
        excess_carries_back_1_year_cents: 0,
        excess_carries_forward_10_years_cents: 0,
        excess_expires_no_carryover_cents: 0,
        recommended_actions,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section904Input {
        Section904Input {
            basket_category: BasketCategory::General,
            taxable_year: 2024,
            foreign_source_ti_in_basket_cents: 30_000_000_00,
            total_worldwide_ti_cents: 100_000_000_00,
            us_tax_liability_cents: 21_000_000_00,
            foreign_tax_paid_in_basket_cents: 5_000_000_00,
            prior_year_excess_carryforward_cents: 0,
            ofl_recapture_triggered: false,
            ofl_recapture_amount_cents: 0,
            odl_recapture_triggered: false,
        }
    }

    #[test]
    fn ftc_limitation_correctly_computed() {
        let i = baseline();
        let r = check(&i);
        let expected = 21_000_000_00u64 * 30_000_000_00 / 100_000_000_00;
        assert_eq!(r.ftc_limitation_in_basket_cents, expected);
    }

    #[test]
    fn full_creditability_when_under_limit() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullyCreditedNoExcess));
        assert_eq!(r.excess_ftc_in_basket_cents, 0);
    }

    #[test]
    fn excess_ftc_when_over_limit_general_basket_carries() {
        let mut i = baseline();
        i.foreign_tax_paid_in_basket_cents = 10_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::PartiallyCreditedExcessCarriedForward
        ));
        let limit = 21_000_000_00u64 * 30_000_000_00 / 100_000_000_00;
        let expected_excess = 10_000_000_00u64 - limit;
        assert_eq!(r.excess_ftc_in_basket_cents, expected_excess);
        assert_eq!(
            r.excess_carries_back_1_year_cents + r.excess_carries_forward_10_years_cents,
            expected_excess
        );
    }

    #[test]
    fn gilti_ncti_excess_expires_no_carryover() {
        let mut i = baseline();
        i.basket_category = BasketCategory::GiltiOrNcti;
        i.foreign_tax_paid_in_basket_cents = 10_000_000_00;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::GiltiNctiExcessExpired));
        assert_eq!(r.excess_carries_back_1_year_cents, 0);
        assert_eq!(r.excess_carries_forward_10_years_cents, 0);
        assert!(r.excess_expires_no_carryover_cents > 0);
    }

    #[test]
    fn section_901j_sanctioned_non_creditable() {
        let mut i = baseline();
        i.basket_category = BasketCategory::Section901jSanctioned;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::Section901jSanctionedNonCreditable));
        assert_eq!(r.ftc_actually_used_cents, 0);
        assert!(r.notes.iter().any(|n| n.contains("Iran")));
        assert!(r.notes.iter().any(|n| n.contains("§ 901(j)")));
    }

    #[test]
    fn ofl_recapture_triggers_severity() {
        let mut i = baseline();
        i.ofl_recapture_triggered = true;
        i.ofl_recapture_amount_cents = 5_000_000_00;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::OverallForeignLossRecaptureTriggered));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 904(f)")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 1118 Schedule J")));
    }

    #[test]
    fn odl_recapture_triggers_severity() {
        let mut i = baseline();
        i.odl_recapture_triggered = true;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::OverallDomesticLossRecaptureTriggered));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 904(g)")));
    }

    #[test]
    fn prior_year_carryforward_increases_creditable_pool() {
        let mut i = baseline();
        i.prior_year_excess_carryforward_cents = 2_000_000_00;
        let r = check(&i);
        assert_eq!(
            r.total_foreign_tax_creditable_cents,
            i.foreign_tax_paid_in_basket_cents + 2_000_000_00
        );
    }

    #[test]
    fn zero_worldwide_ti_not_applicable() {
        let mut i = baseline();
        i.total_worldwide_ti_cents = 0;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
    }

    #[test]
    fn passive_basket_compliant_with_carryover() {
        let mut i = baseline();
        i.basket_category = BasketCategory::Passive;
        i.foreign_tax_paid_in_basket_cents = 10_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::PartiallyCreditedExcessCarriedForward
        ));
        assert!(r.excess_carries_back_1_year_cents > 0);
    }

    #[test]
    fn foreign_branch_basket_compliant_with_carryover() {
        let mut i = baseline();
        i.basket_category = BasketCategory::ForeignBranch;
        i.foreign_tax_paid_in_basket_cents = 10_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::PartiallyCreditedExcessCarriedForward
        ));
    }

    #[test]
    fn treaty_resourced_basket_recognized() {
        let mut i = baseline();
        i.basket_category = BasketCategory::TreatyResourced;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullyCreditedNoExcess));
    }

    #[test]
    fn lump_sum_distribution_basket_recognized() {
        let mut i = baseline();
        i.basket_category = BasketCategory::LumpSumDistribution;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullyCreditedNoExcess));
    }

    #[test]
    fn ftc_carryback_years_pins_1() {
        assert_eq!(FTC_CARRYBACK_YEARS, 1);
    }

    #[test]
    fn ftc_carryforward_years_pins_10() {
        assert_eq!(FTC_CARRYFORWARD_YEARS, 10);
    }

    #[test]
    fn tcja_post_2017_baskets_count_pins_7() {
        assert_eq!(TCJA_POST_2017_BASKETS_COUNT, 7);
    }

    #[test]
    fn obbba_effective_year_pins_2026() {
        assert_eq!(OBBBA_EFFECTIVE_YEAR, 2026);
    }

    #[test]
    fn action_references_form_1116_and_1118() {
        let i = baseline();
        let r = check(&i);
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 1116")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 1118")));
    }

    #[test]
    fn note_pins_seven_baskets_and_obbba_864e() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("7 separate § 904(d) baskets")));
        assert!(r.notes.iter().any(|n| n.contains("OBBBA")));
        assert!(r.notes.iter().any(|n| n.contains("§ 864(e)")));
        assert!(r.notes.iter().any(|n| n.contains("80")));
        assert!(r.notes.iter().any(|n| n.contains("90")));
    }

    #[test]
    fn coordination_note_references_901_960_951a_956_959_962_245a_965() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_901")));
        assert!(r.notes.iter().any(|n| n.contains("section_960")));
        assert!(r.notes.iter().any(|n| n.contains("section_951a")));
        assert!(r.notes.iter().any(|n| n.contains("section_956")));
        assert!(r.notes.iter().any(|n| n.contains("section_959")));
        assert!(r.notes.iter().any(|n| n.contains("section_962")));
        assert!(r.notes.iter().any(|n| n.contains("section_245a")));
        assert!(r.notes.iter().any(|n| n.contains("section_965")));
    }

    #[test]
    fn citation_pins_904_901_960_pub_l_115_97_and_119_21() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 904(a)-(j)"));
        assert!(r.citation.contains("§ 901"));
        assert!(r.citation.contains("§ 960"));
        assert!(r.citation.contains("Pub. L. 115-97"));
        assert!(r.citation.contains("Pub. L. 119-21"));
    }

    #[test]
    fn realistic_corp_with_passive_basket_carryforward() {
        let mut i = baseline();
        i.basket_category = BasketCategory::Passive;
        i.foreign_source_ti_in_basket_cents = 5_000_000_00;
        i.total_worldwide_ti_cents = 50_000_000_00;
        i.us_tax_liability_cents = 10_500_000_00;
        i.foreign_tax_paid_in_basket_cents = 2_000_000_00;
        let r = check(&i);
        let limit = 10_500_000_00u64 * 5_000_000_00 / 50_000_000_00;
        assert_eq!(r.ftc_limitation_in_basket_cents, limit);
        let expected_excess = 2_000_000_00u64.saturating_sub(limit);
        assert_eq!(r.excess_ftc_in_basket_cents, expected_excess);
    }

    #[test]
    fn extreme_value_does_not_overflow() {
        let mut i = baseline();
        i.us_tax_liability_cents = u64::MAX / 100;
        i.foreign_source_ti_in_basket_cents = u64::MAX / 100;
        i.total_worldwide_ti_cents = u64::MAX / 100;
        let r = check(&i);
        let _ = r.ftc_limitation_in_basket_cents;
    }

    #[test]
    fn zero_foreign_tax_zero_credit_used() {
        let mut i = baseline();
        i.foreign_tax_paid_in_basket_cents = 0;
        let r = check(&i);
        assert_eq!(r.ftc_actually_used_cents, 0);
    }

    #[test]
    fn ftc_capped_at_limitation_not_at_total_creditable() {
        let mut i = baseline();
        i.foreign_tax_paid_in_basket_cents = 100_000_000_00;
        let r = check(&i);
        assert!(r.ftc_actually_used_cents <= r.ftc_limitation_in_basket_cents);
    }
}
