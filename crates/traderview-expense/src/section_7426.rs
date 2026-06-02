//! IRC § 7426 — Civil actions by persons other than
//! taxpayers (third-party wrongful levy + surplus proceeds +
//! substituted sales proceeds). Trader-relevant when IRS
//! levies on property that BELONGS to someone other than the
//! taxpayer assessed (joint accounts + nominee accounts +
//! community-property third-party interests + trader's
//! co-owner / lender / lien-holder rights in seized rental
//! property). Procedural-companion to § 7421 (Anti-
//! Injunction Act — § 7426 is a statutory exception under §
//! 7421(a)), § 7433 (civil damages for unauthorized
//! collection), § 7430 (litigation costs), and § 6334
//! (property exempt from levy).
//!
//! **§ 7426(a)(1) wrongful levy action** — any person (other
//! than the person against whom the tax was assessed) who
//! claims an interest in or lien on property and that such
//! property was wrongfully levied upon may bring civil action
//! against the United States in district court.
//!
//! **§ 7426(a)(2) surplus proceeds action** — if surplus
//! proceeds have been realized from sale of property pursuant
//! to a levy, any person (other than the taxpayer) who claims
//! interest in or lien upon such property JUNIOR to lien or
//! interest of the United States, and claims to be legally
//! entitled to all or part of such surplus proceeds, may
//! bring action against the United States in district court.
//!
//! **§ 7426(a)(3) substituted sales proceeds action** — if
//! court determines that a party has interest in or lien
//! upon amount held as fund pursuant to agreement providing
//! for proceeds of sale of property to be substituted for
//! the property, court may grant judgment in amount not in
//! excess of substituted sale proceeds.
//!
//! **§ 7426(c) SOL — 2 years for wrongful levy** (extended
//! from 9 months by Pub. L. 115-97 § 11071, eff. December
//! 22, 2017; pre-amendment 9-month rule still applies to
//! levies before that date).
//!
//! **§ 7426(h) civil damages for unauthorized collection** —
//! IRS officer / employee recklessly + intentionally + by
//! reason of negligence disregards any IRC provision in
//! connection with collection: lesser of $1,000,000 / $100,000
//! (negligence) OR actual damages + costs. Mirrors § 7433
//! framework.
//!
//! **§ 7421(a) Anti-Injunction Act exception** — § 7426(a)
//! and (b)(1) actions are statutorily excepted from AIA bar
//! on suits to restrain assessment or collection.
//!
//! Citations: 26 USC § 7426(a)(1), (a)(2), (a)(3), (b)(1)+
//! (b)(2), (c), (g), (h); Pub. L. 115-97 § 11071 (TCJA SOL
//! extension); 26 CFR § 301.7426-1; IRM 34.5.3; IRS Pub. 4528
//! (Making an Administrative Wrongful Levy Claim).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// § 7426(a)(1) — wrongful levy on third-party property.
    WrongfulLevy,
    /// § 7426(a)(2) — surplus proceeds entitlement.
    SurplusProceeds,
    /// § 7426(a)(3) — substituted sales proceeds.
    SubstitutedSalesProceeds,
    /// § 7426(h) — civil damages for unauthorized IRS
    /// collection action.
    UnauthorizedCollectionDamages,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Culpability {
    /// § 7426(h) — $100,000 cap for negligence.
    Negligence,
    /// § 7426(h) — $1,000,000 cap for reckless or intentional.
    RecklessOrIntentional,
    /// Not applicable for actions other than (h).
    NotApplicable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7426Input {
    pub action_type: ActionType,
    /// Whether the claimant is the taxpayer assessed (§ 7426
    /// limits actions to persons OTHER than the taxpayer).
    pub claimant_is_assessed_taxpayer: bool,
    /// Whether the claimant holds an interest or lien on the
    /// property (§ 7426(a)(1) gating clause).
    pub claimant_has_interest_or_lien: bool,
    /// Days since the levy (for § 7426(c) 2-year = 730-day SOL).
    pub days_since_levy: u32,
    /// Whether the levy occurred on or after December 22,
    /// 2017 (TCJA SOL extension date).
    pub levy_on_or_after_2017_tcja: bool,
    /// For § 7426(a)(2) surplus proceeds — whether claimant's
    /// interest is junior to United States.
    pub junior_to_united_states_interest: bool,
    /// For § 7426(h) civil damages — IRS culpability tier.
    pub culpability: Culpability,
    /// For § 7426(h) civil damages — actual damages sustained
    /// (excluding costs).
    pub actual_damages_cents: i64,
    /// For § 7426(h) civil damages — costs of action.
    pub costs_of_action_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7426Result {
    pub action_maintainable: bool,
    pub sol_window_days: u32,
    pub sol_satisfied: bool,
    pub damages_cap_cents: i64,
    pub damages_cents: i64,
    pub aia_exception_engaged: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7426Input) -> Section7426Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    if input.claimant_is_assessed_taxpayer {
        failure_reasons.push(
            "26 USC § 7426(a) — claimant is the person against whom the tax was assessed; § 7426 actions limited to persons OTHER than the taxpayer".to_string(),
        );
    }

    let sol_window = if input.levy_on_or_after_2017_tcja {
        730
    } else {
        274
    };

    let sol_satisfied = input.days_since_levy <= sol_window;

    if matches!(input.action_type, ActionType::WrongfulLevy) && !sol_satisfied {
        failure_reasons.push(format!(
            "26 USC § 7426(c) — {} days elapsed since levy exceeds {}-day SOL ({})",
            input.days_since_levy,
            sol_window,
            if input.levy_on_or_after_2017_tcja {
                "2-year TCJA-amended"
            } else {
                "pre-2017 9-month"
            }
        ));
    }

    match input.action_type {
        ActionType::WrongfulLevy => {
            if !input.claimant_has_interest_or_lien {
                failure_reasons.push(
                    "26 USC § 7426(a)(1) — wrongful levy action requires claimant interest in or lien on property".to_string(),
                );
            }
        }
        ActionType::SurplusProceeds => {
            if !input.claimant_has_interest_or_lien {
                failure_reasons.push(
                    "26 USC § 7426(a)(2) — surplus proceeds action requires claimant interest in or lien upon property".to_string(),
                );
            }
            if !input.junior_to_united_states_interest {
                failure_reasons.push(
                    "26 USC § 7426(a)(2) — surplus proceeds action requires claimant interest JUNIOR to lien or interest of United States".to_string(),
                );
            }
        }
        ActionType::SubstitutedSalesProceeds => {
            if !input.claimant_has_interest_or_lien {
                failure_reasons.push(
                    "26 USC § 7426(a)(3) — substituted sales proceeds action requires claimant interest in or lien upon fund pursuant to substitution agreement".to_string(),
                );
            }
        }
        ActionType::UnauthorizedCollectionDamages => {
            if matches!(input.culpability, Culpability::NotApplicable) {
                failure_reasons.push(
                    "26 USC § 7426(h) — unauthorized collection damages action requires reckless / intentional / negligent culpability finding".to_string(),
                );
            }
        }
    }

    let damages_cap: i64 = match (input.action_type, input.culpability) {
        (ActionType::UnauthorizedCollectionDamages, Culpability::Negligence) => 10_000_000_000,
        (ActionType::UnauthorizedCollectionDamages, Culpability::RecklessOrIntentional) => {
            100_000_000_000
        }
        _ => 0,
    };

    let damages = if matches!(input.action_type, ActionType::UnauthorizedCollectionDamages)
        && failure_reasons.is_empty()
    {
        let actual = input.actual_damages_cents.max(0);
        let costs = input.costs_of_action_cents.max(0);
        actual.saturating_add(costs).min(damages_cap)
    } else {
        0
    };

    let aia_exception = matches!(
        input.action_type,
        ActionType::WrongfulLevy | ActionType::SurplusProceeds | ActionType::SubstitutedSalesProceeds
    );

    let notes: Vec<String> = vec![
        "26 USC § 7426(a) — three civil actions for persons other than taxpayers: (1) wrongful levy; (2) surplus proceeds; (3) substituted sales proceeds; all in district court"
            .to_string(),
        "26 USC § 7426(c) — 2-year SOL for wrongful levy (extended from 9 months by Pub. L. 115-97 § 11071 TCJA, eff. December 22, 2017); pre-amendment levies retain 9-month SOL"
            .to_string(),
        "26 USC § 7426(h) — civil damages for IRS unauthorized collection: lesser of $1,000,000 (reckless/intentional) / $100,000 (negligence) OR actual damages + costs; mirrors § 7433"
            .to_string(),
        "26 USC § 7421(a) Anti-Injunction Act exception — § 7426(a) + (b)(1) actions statutorily excepted; pair with IRS Pub. 4528 (Making an Administrative Wrongful Levy Claim)"
            .to_string(),
    ];

    Section7426Result {
        action_maintainable: failure_reasons.is_empty(),
        sol_window_days: sol_window,
        sol_satisfied,
        damages_cap_cents: damages_cap,
        damages_cents: damages,
        aia_exception_engaged: aia_exception,
        failure_reasons,
        citation: "26 USC § 7426(a)(1), (a)(2), (a)(3), (b)(1)+(b)(2), (c), (g), (h); Pub. L. 115-97 § 11071; 26 CFR § 301.7426-1; IRM 34.5.3; IRS Pub. 4528",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrongful_levy_base() -> Section7426Input {
        Section7426Input {
            action_type: ActionType::WrongfulLevy,
            claimant_is_assessed_taxpayer: false,
            claimant_has_interest_or_lien: true,
            days_since_levy: 365,
            levy_on_or_after_2017_tcja: true,
            junior_to_united_states_interest: false,
            culpability: Culpability::NotApplicable,
            actual_damages_cents: 0,
            costs_of_action_cents: 0,
        }
    }

    fn surplus_base() -> Section7426Input {
        let mut i = wrongful_levy_base();
        i.action_type = ActionType::SurplusProceeds;
        i.junior_to_united_states_interest = true;
        i
    }

    fn substituted_base() -> Section7426Input {
        let mut i = wrongful_levy_base();
        i.action_type = ActionType::SubstitutedSalesProceeds;
        i
    }

    fn damages_base() -> Section7426Input {
        Section7426Input {
            action_type: ActionType::UnauthorizedCollectionDamages,
            claimant_is_assessed_taxpayer: false,
            claimant_has_interest_or_lien: false,
            days_since_levy: 0,
            levy_on_or_after_2017_tcja: true,
            junior_to_united_states_interest: false,
            culpability: Culpability::RecklessOrIntentional,
            actual_damages_cents: 5_000_000,
            costs_of_action_cents: 500_000,
        }
    }

    #[test]
    fn wrongful_levy_within_2_year_sol_maintainable() {
        let r = check(&wrongful_levy_base());
        assert!(r.action_maintainable);
        assert_eq!(r.sol_window_days, 730);
        assert!(r.sol_satisfied);
        assert!(r.aia_exception_engaged);
    }

    #[test]
    fn wrongful_levy_at_730_day_boundary_compliant() {
        let mut i = wrongful_levy_base();
        i.days_since_levy = 730;
        let r = check(&i);
        assert!(r.action_maintainable);
        assert!(r.sol_satisfied);
    }

    #[test]
    fn wrongful_levy_731_days_violates_sol() {
        let mut i = wrongful_levy_base();
        i.days_since_levy = 731;
        let r = check(&i);
        assert!(!r.action_maintainable);
        assert!(!r.sol_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7426(c)") && f.contains("731 days") && f.contains("2-year TCJA")));
    }

    #[test]
    fn pre_2017_tcja_9_month_sol() {
        let mut i = wrongful_levy_base();
        i.levy_on_or_after_2017_tcja = false;
        i.days_since_levy = 300;
        let r = check(&i);
        assert_eq!(r.sol_window_days, 274);
        assert!(!r.sol_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("pre-2017 9-month")));
    }

    #[test]
    fn pre_2017_tcja_at_274_day_boundary_compliant() {
        let mut i = wrongful_levy_base();
        i.levy_on_or_after_2017_tcja = false;
        i.days_since_levy = 274;
        let r = check(&i);
        assert!(r.sol_satisfied);
        assert!(r.action_maintainable);
    }

    #[test]
    fn taxpayer_assessed_blocked_from_action() {
        let mut i = wrongful_levy_base();
        i.claimant_is_assessed_taxpayer = true;
        let r = check(&i);
        assert!(!r.action_maintainable);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7426(a)") && f.contains("OTHER than the taxpayer")));
    }

    #[test]
    fn no_interest_or_lien_blocks_wrongful_levy() {
        let mut i = wrongful_levy_base();
        i.claimant_has_interest_or_lien = false;
        let r = check(&i);
        assert!(!r.action_maintainable);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7426(a)(1)") && f.contains("requires claimant interest")));
    }

    #[test]
    fn surplus_proceeds_compliant() {
        let r = check(&surplus_base());
        assert!(r.action_maintainable);
    }

    #[test]
    fn surplus_proceeds_no_junior_interest_violates() {
        let mut i = surplus_base();
        i.junior_to_united_states_interest = false;
        let r = check(&i);
        assert!(!r.action_maintainable);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7426(a)(2)") && f.contains("JUNIOR")));
    }

    #[test]
    fn substituted_sales_proceeds_compliant() {
        let r = check(&substituted_base());
        assert!(r.action_maintainable);
    }

    #[test]
    fn unauthorized_collection_damages_compliant() {
        let r = check(&damages_base());
        assert!(r.action_maintainable);
        assert_eq!(r.damages_cap_cents, 100_000_000_000);
        assert_eq!(r.damages_cents, 5_500_000);
    }

    #[test]
    fn unauthorized_collection_damages_negligence_cap_100k() {
        let mut i = damages_base();
        i.culpability = Culpability::Negligence;
        i.actual_damages_cents = 200_000_000_000;
        let r = check(&i);
        assert_eq!(r.damages_cap_cents, 10_000_000_000);
        assert_eq!(r.damages_cents, 10_000_000_000);
    }

    #[test]
    fn unauthorized_collection_damages_reckless_cap_1m() {
        let mut i = damages_base();
        i.culpability = Culpability::RecklessOrIntentional;
        i.actual_damages_cents = 200_000_000_000;
        let r = check(&i);
        assert_eq!(r.damages_cap_cents, 100_000_000_000);
        assert_eq!(r.damages_cents, 100_000_000_000);
    }

    #[test]
    fn unauthorized_collection_damages_not_applicable_violates() {
        let mut i = damages_base();
        i.culpability = Culpability::NotApplicable;
        let r = check(&i);
        assert!(!r.action_maintainable);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7426(h)") && f.contains("culpability")));
    }

    #[test]
    fn aia_exception_engaged_for_three_property_actions() {
        for action in [
            ActionType::WrongfulLevy,
            ActionType::SurplusProceeds,
            ActionType::SubstitutedSalesProceeds,
        ] {
            let mut i = wrongful_levy_base();
            i.action_type = action;
            i.junior_to_united_states_interest = true;
            let r = check(&i);
            assert!(r.aia_exception_engaged);
        }
    }

    #[test]
    fn aia_exception_not_engaged_for_unauthorized_collection_damages() {
        let r = check(&damages_base());
        assert!(!r.aia_exception_engaged);
    }

    #[test]
    fn sol_truth_table_2017_vs_pre_2017() {
        let mut i_post = wrongful_levy_base();
        i_post.levy_on_or_after_2017_tcja = true;
        let r_post = check(&i_post);
        assert_eq!(r_post.sol_window_days, 730);

        let mut i_pre = wrongful_levy_base();
        i_pre.levy_on_or_after_2017_tcja = false;
        let r_pre = check(&i_pre);
        assert_eq!(r_pre.sol_window_days, 274);
    }

    #[test]
    fn sol_2_year_uniquely_for_post_2017_invariant() {
        let mut i_post = wrongful_levy_base();
        i_post.levy_on_or_after_2017_tcja = true;
        i_post.days_since_levy = 500;
        let r_post = check(&i_post);
        assert!(r_post.sol_satisfied);

        let mut i_pre = wrongful_levy_base();
        i_pre.levy_on_or_after_2017_tcja = false;
        i_pre.days_since_levy = 500;
        let r_pre = check(&i_pre);
        assert!(!r_pre.sol_satisfied);
    }

    #[test]
    fn sol_only_applies_to_wrongful_levy_not_other_actions() {
        let mut i = surplus_base();
        i.days_since_levy = 1000;
        let r = check(&i);
        assert!(r.action_maintainable);
    }

    #[test]
    fn action_type_truth_table() {
        let mut i_wrongful = wrongful_levy_base();
        i_wrongful.action_type = ActionType::WrongfulLevy;
        let r_wrongful = check(&i_wrongful);
        assert!(r_wrongful.action_maintainable);
        assert!(r_wrongful.aia_exception_engaged);

        let mut i_surplus = wrongful_levy_base();
        i_surplus.action_type = ActionType::SurplusProceeds;
        i_surplus.junior_to_united_states_interest = true;
        let r_surplus = check(&i_surplus);
        assert!(r_surplus.action_maintainable);

        let mut i_sub = wrongful_levy_base();
        i_sub.action_type = ActionType::SubstitutedSalesProceeds;
        let r_sub = check(&i_sub);
        assert!(r_sub.action_maintainable);

        let r_damages = check(&damages_base());
        assert!(r_damages.action_maintainable);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&wrongful_levy_base());
        assert!(r.citation.contains("§ 7426(a)(1)"));
        assert!(r.citation.contains("(a)(2)"));
        assert!(r.citation.contains("(a)(3)"));
        assert!(r.citation.contains("(b)(1)+(b)(2)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("(g)"));
        assert!(r.citation.contains("(h)"));
        assert!(r.citation.contains("Pub. L. 115-97 § 11071"));
        assert!(r.citation.contains("Pub. 4528"));
    }

    #[test]
    fn note_pins_three_actions() {
        let r = check(&wrongful_levy_base());
        assert!(r.notes.iter().any(|n| n.contains("wrongful levy")
            && n.contains("surplus proceeds")
            && n.contains("substituted sales proceeds")));
    }

    #[test]
    fn note_pins_tcja_sol_extension() {
        let r = check(&wrongful_levy_base());
        assert!(r.notes.iter().any(|n| n.contains("2-year SOL")
            && n.contains("9 months")
            && n.contains("Pub. L. 115-97")
            && n.contains("December 22, 2017")));
    }

    #[test]
    fn note_pins_section_7426h_civil_damages_caps() {
        let r = check(&wrongful_levy_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7426(h)")
            && n.contains("$1,000,000")
            && n.contains("$100,000")
            && n.contains("§ 7433")));
    }

    #[test]
    fn note_pins_aia_exception_and_pub_4528() {
        let r = check(&wrongful_levy_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7421(a)")
            && n.contains("Anti-Injunction Act exception")
            && n.contains("Pub. 4528")));
    }

    #[test]
    fn defensive_negative_damages_clamped() {
        let mut i = damages_base();
        i.actual_damages_cents = -1_000_000;
        let r = check(&i);
        assert_eq!(r.damages_cents, 500_000);
    }

    #[test]
    fn defensive_negative_costs_clamped() {
        let mut i = damages_base();
        i.costs_of_action_cents = -500_000;
        let r = check(&i);
        assert_eq!(r.damages_cents, 5_000_000);
    }

    #[test]
    fn defensive_overflow_damages_saturating() {
        let mut i = damages_base();
        i.actual_damages_cents = i64::MAX - 1_000;
        i.costs_of_action_cents = i64::MAX - 1_000;
        let r = check(&i);
        assert_eq!(r.damages_cents, 100_000_000_000);
    }

    #[test]
    fn three_failures_stack_for_wrongful_levy() {
        let i = Section7426Input {
            action_type: ActionType::WrongfulLevy,
            claimant_is_assessed_taxpayer: true,
            claimant_has_interest_or_lien: false,
            days_since_levy: 1000,
            levy_on_or_after_2017_tcja: true,
            junior_to_united_states_interest: false,
            culpability: Culpability::NotApplicable,
            actual_damages_cents: 0,
            costs_of_action_cents: 0,
        };
        let r = check(&i);
        assert!(!r.action_maintainable);
        assert_eq!(r.failure_reasons.len(), 3);
    }

    #[test]
    fn surplus_proceeds_uniquely_requires_junior_interest_invariant() {
        let mut i_surplus = surplus_base();
        i_surplus.junior_to_united_states_interest = false;
        let r_surplus = check(&i_surplus);
        assert!(!r_surplus.action_maintainable);

        let mut i_wrongful = wrongful_levy_base();
        i_wrongful.junior_to_united_states_interest = false;
        let r_wrongful = check(&i_wrongful);
        assert!(r_wrongful.action_maintainable);
    }

    #[test]
    fn wrongful_levy_pre_2017_275_days_violates() {
        let mut i = wrongful_levy_base();
        i.levy_on_or_after_2017_tcja = false;
        i.days_since_levy = 275;
        let r = check(&i);
        assert!(!r.sol_satisfied);
    }

    #[test]
    fn unauthorized_collection_damages_costs_only_at_zero_actual() {
        let mut i = damages_base();
        i.actual_damages_cents = 0;
        i.costs_of_action_cents = 1_000_000;
        let r = check(&i);
        assert_eq!(r.damages_cents, 1_000_000);
    }

    #[test]
    fn damages_cap_negligence_uniquely_lower_invariant() {
        let mut i_neg = damages_base();
        i_neg.culpability = Culpability::Negligence;
        i_neg.actual_damages_cents = 50_000_000_000;
        let r_neg = check(&i_neg);

        let mut i_reck = damages_base();
        i_reck.culpability = Culpability::RecklessOrIntentional;
        i_reck.actual_damages_cents = 50_000_000_000;
        let r_reck = check(&i_reck);

        assert!(r_neg.damages_cap_cents < r_reck.damages_cap_cents);
        assert_eq!(r_neg.damages_cap_cents * 10, r_reck.damages_cap_cents);
    }
}
