//! State tenant rent escrow / withholding-into-court compliance.
//!
//! Distinct from `repair_and_deduct` (which lets tenant deduct
//! repair cost FROM rent). Rent escrow lets the tenant PAY RENT
//! INTO COURT (or escrow) while landlord remedies habitability
//! defects — the landlord doesn't get the rent until repairs are
//! made. Four states have detailed statutory rent-escrow regimes;
//! the rest rely on common-law warranty of habitability or limit
//! tenant withholding to the repair-and-deduct mechanism.
//!
//! Five regimes:
//!
//! `MarylandRentEscrowAct`: MD only. Real Property § 8-211 (Rent
//! Escrow Act). Tenant may bring rent-escrow action when landlord
//! refuses to repair conditions materially affecting health and
//! safety. **Rebuttable presumption that > 30 days from notice
//! is unreasonable**. Court may order rent paid into escrow +
//! abate prospective rent in an amount the court determines.
//!
//! `MassachusettsCounterclaimDefense`: MA only. G.L. c. 239 § 8A
//! (rent withholding statute). Tenant raises warranty-of-
//! habitability counterclaim in eviction action; court may order
//! tenant to deposit fair use and occupation value into court
//! pending repairs. Funds used for repairs by landlord or
//! receiver; balance to landlord after work complete.
//!
//! `NewJerseyMariniHearingAdministrator`: NJ only. N.J.S.A.
//! 2A:42-85 et seq. + Marini v. Ireland (1970) common-law
//! doctrine. Tenant requests Marini hearing; **must deposit ALL
//! unpaid rent with court** at hearing. Statutory framework
//! authorizes court-appointed administrator to hold rent + use it
//! to remedy defective conditions in substandard dwellings.
//!
//! `ColoradoLimitedWithholding`: CO only. C.R.S. § 38-12-503 +
//! § 38-12-507 statutory warranty of habitability framework.
//! Rent withholding limited to repair-and-deduct OR court-ordered
//! escrow / abatement during litigation. **Withholding the
//! ENTIRE rent is NOT permitted** — tenant must continue paying
//! or seek formal court relief.
//!
//! `NoStatutoryRentEscrowFramework`: 46 other states + DC. Tenant
//! has common-law warranty-of-habitability remedies (constructive
//! eviction, rent abatement at trial) but no statutory escrow
//! framework. Many states allow tenant to pay rent into court as
//! a procedural posting during eviction litigation but lack the
//! detailed rent-escrow mechanisms of MD / MA / NJ / CO.
//!
//! Sources:
//! [Md. Real Prop. § 8-211 — Justia](https://law.justia.com/codes/maryland/real-property/title-8/subtitle-2/section-8-211/),
//! [Mass. G.L. c. 239 § 8A — Mass. General Laws](https://malegislature.gov/Laws/GeneralLaws/PartIII/TitleIII/Chapter239/Section8A),
//! [Mass. Legal Services — Eviction GL 239 8A](https://www.masslegalservices.org/library/directory/housing/summary-process-counterclaims-defenses/eviction-gl-239-8a),
//! [Marini v. Ireland, 56 N.J. 130 (1970) — Justia case law](https://law.justia.com/cases/new-jersey/supreme-court/),
//! [Whelan Law — What is a Marini Hearing?](https://www.whelan-law.com/what-is-a-marini-hearing/),
//! [CO C.R.S. § 38-12-503 — Robinson and Henry guide](https://www.robinsonandhenry.com/blog/real-estate/warranty-of-habitability/).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RentEscrowRegime {
    MarylandRentEscrowAct,
    MassachusettsCounterclaimDefense,
    NewJerseyMariniHearingAdministrator,
    ColoradoLimitedWithholding,
    NoStatutoryRentEscrowFramework,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: RentEscrowRegime,
    /// Statutory reasonableness presumption window (days). MD has
    /// 30-day rebuttable presumption.
    pub reasonableness_presumption_days: u32,
    /// True if statute requires tenant to deposit ALL accrued
    /// unpaid rent with court at hearing (NJ requirement).
    pub all_unpaid_rent_deposit_required: bool,
    /// True if regime explicitly prohibits withholding the entire
    /// rent without court order (CO).
    pub entire_rent_withholding_prohibited: bool,
    /// True if statute authorizes court-appointed administrator
    /// to hold rent + remedy defects (NJ).
    pub court_appointed_administrator_authorized: bool,
    /// True if regime arises through eviction-counterclaim
    /// mechanism (MA).
    pub counterclaim_defense_mechanism: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: RentEscrowRegime,
    reasonableness_presumption_days: u32,
    all_unpaid_rent_deposit_required: bool,
    entire_rent_withholding_prohibited: bool,
    court_appointed_administrator_authorized: bool,
    counterclaim_defense_mechanism: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        reasonableness_presumption_days,
        all_unpaid_rent_deposit_required,
        entire_rent_withholding_prohibited,
        court_appointed_administrator_authorized,
        counterclaim_defense_mechanism,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use RentEscrowRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "MD",
        rule(
            MarylandRentEscrowAct,
            30,
            false,
            false,
            false,
            false,
            "Md. Real Prop. § 8-211 (Rent Escrow Act) — tenant may bring rent-escrow action when landlord refuses to repair conditions materially affecting health and safety; rebuttable presumption that > 30 days from notice is unreasonable; court may order rent paid into escrow + abate prospective rent in amount court determines",
        ),
    );

    m.insert(
        "MA",
        rule(
            MassachusettsCounterclaimDefense,
            0,
            false,
            false,
            false,
            true,
            "Mass. G.L. c. 239 § 8A (rent withholding statute) — tenant raises warranty-of-habitability counterclaim in eviction action; court may order tenant to deposit fair use and occupation value into court pending repairs; funds used for repairs by landlord or receiver; balance to landlord after work complete",
        ),
    );

    m.insert(
        "NJ",
        rule(
            NewJerseyMariniHearingAdministrator,
            0,
            true,
            false,
            true,
            false,
            "N.J.S.A. 2A:42-85 et seq. + Marini v. Ireland (1970) — tenant must request Marini hearing AND deposit ALL unpaid rent with court at hearing; statutory framework authorizes court-appointed administrator to hold rent + use it to remedy defective conditions in substandard dwellings",
        ),
    );

    m.insert(
        "CO",
        rule(
            ColoradoLimitedWithholding,
            0,
            false,
            true,
            false,
            false,
            "C.R.S. §§ 38-12-503 + 38-12-507 statutory warranty of habitability framework — rent withholding limited to repair-and-deduct or court-ordered escrow / abatement during litigation; withholding the ENTIRE rent is NOT permitted; tenant must continue paying or seek formal court relief",
        ),
    );

    // NoStatutoryRentEscrowFramework default — 46 other states + DC.
    let default_states = [
        "AL", "AK", "AZ", "AR", "CA", "CT", "DC", "DE",
        "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
        "KY", "LA", "ME", "MI", "MN", "MS", "MO", "MT",
        "NE", "NV", "NH", "NM", "NY", "NC", "ND", "OH",
        "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX",
        "UT", "VT", "VA", "WA", "WV", "WI", "WY",
    ];
    for code in default_states {
        m.insert(
            code,
            rule(
                NoStatutoryRentEscrowFramework,
                0,
                false,
                false,
                false,
                false,
                "No statutory rent-escrow framework; tenant has common-law warranty-of-habitability remedies (constructive eviction, rent abatement at trial); some states allow procedural rent-into-court posting during eviction litigation but lack detailed escrow mechanism",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RentEscrowInput {
    pub state_code: String,
    /// True if the landlord has been notified of a habitability
    /// defect.
    pub landlord_notified_of_defect: bool,
    /// Days since notice was given (for MD 30-day presumption).
    pub days_since_notice: u32,
    /// True if the defect materially affects health and safety
    /// (MD § 8-211 threshold).
    pub defect_materially_affects_health_safety: bool,
    /// True if the tenant has deposited all accrued unpaid rent
    /// with the court (NJ Marini-hearing requirement).
    pub tenant_deposited_all_unpaid_rent: bool,
    /// True if the tenant is asserting an eviction counterclaim
    /// (MA mechanism).
    pub tenant_asserting_eviction_counterclaim: bool,
    /// True if the tenant is attempting to withhold the entire
    /// rent (CO bar).
    pub tenant_withholding_entire_rent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RentEscrowResult {
    pub regime: RentEscrowRegime,
    pub statutory_remedy_available: bool,
    /// True if landlord has had unreasonable time per the state's
    /// presumption.
    pub unreasonable_time_presumption_triggered: bool,
    /// True if the regime-specific procedural requirements are met.
    pub procedural_requirements_satisfied: bool,
    pub tenant_may_pursue_escrow: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &RentEscrowInput) -> RentEscrowResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: RentEscrowRegime::NoStatutoryRentEscrowFramework,
        reasonableness_presumption_days: 0,
        all_unpaid_rent_deposit_required: false,
        entire_rent_withholding_prohibited: false,
        court_appointed_administrator_authorized: false,
        counterclaim_defense_mechanism: false,
        citation: "Unknown state code; no statutory rent-escrow framework assumed",
    });

    // MD reasonableness presumption.
    let unreasonable_time = rule.reasonableness_presumption_days > 0
        && input.days_since_notice > rule.reasonableness_presumption_days;

    // Procedural requirements depend on regime.
    let procedural_ok = match rule.regime {
        RentEscrowRegime::MarylandRentEscrowAct => {
            input.landlord_notified_of_defect && input.defect_materially_affects_health_safety
        }
        RentEscrowRegime::MassachusettsCounterclaimDefense => {
            input.tenant_asserting_eviction_counterclaim
        }
        RentEscrowRegime::NewJerseyMariniHearingAdministrator => {
            input.tenant_deposited_all_unpaid_rent
        }
        RentEscrowRegime::ColoradoLimitedWithholding => {
            !input.tenant_withholding_entire_rent
                && input.landlord_notified_of_defect
        }
        RentEscrowRegime::NoStatutoryRentEscrowFramework => input.landlord_notified_of_defect,
    };

    let statutory_available = !matches!(
        rule.regime,
        RentEscrowRegime::NoStatutoryRentEscrowFramework
    );

    let may_pursue = statutory_available && procedural_ok;

    let regime_label = match rule.regime {
        RentEscrowRegime::MarylandRentEscrowAct => {
            "Maryland Rent Escrow Act § 8-211 (30-day reasonableness presumption)"
        }
        RentEscrowRegime::MassachusettsCounterclaimDefense => {
            "Massachusetts c. 239 § 8A counterclaim defense"
        }
        RentEscrowRegime::NewJerseyMariniHearingAdministrator => {
            "New Jersey Marini hearing + court-appointed administrator"
        }
        RentEscrowRegime::ColoradoLimitedWithholding => {
            "Colorado § 38-12-507 limited withholding (entire-rent bar)"
        }
        RentEscrowRegime::NoStatutoryRentEscrowFramework => "no statutory rent-escrow framework",
    };

    let note = if !statutory_available {
        format!(
            "State applies {} regime; only common-law warranty-of-habitability remedies available.",
            regime_label,
        )
    } else if may_pursue {
        format!(
            "State applies {} regime; tenant may pursue rent escrow on these facts.",
            regime_label,
        )
    } else {
        let mut reasons = vec![];
        match rule.regime {
            RentEscrowRegime::MarylandRentEscrowAct => {
                if !input.landlord_notified_of_defect {
                    reasons.push("landlord not notified");
                }
                if !input.defect_materially_affects_health_safety {
                    reasons.push("defect does not materially affect health and safety");
                }
            }
            RentEscrowRegime::MassachusettsCounterclaimDefense => {
                if !input.tenant_asserting_eviction_counterclaim {
                    reasons.push("tenant not asserting eviction counterclaim (MA mechanism requires it)");
                }
            }
            RentEscrowRegime::NewJerseyMariniHearingAdministrator => {
                if !input.tenant_deposited_all_unpaid_rent {
                    reasons.push("all unpaid rent not deposited with court (NJ Marini-hearing requirement)");
                }
            }
            RentEscrowRegime::ColoradoLimitedWithholding => {
                if input.tenant_withholding_entire_rent {
                    reasons.push("tenant withholding entire rent (CO prohibits)");
                }
                if !input.landlord_notified_of_defect {
                    reasons.push("landlord not notified");
                }
            }
            RentEscrowRegime::NoStatutoryRentEscrowFramework => {}
        }
        format!(
            "State applies {} regime; tenant MAY NOT pursue escrow on these facts: {}.",
            regime_label,
            reasons.join("; "),
        )
    };

    RentEscrowResult {
        regime: rule.regime,
        statutory_remedy_available: statutory_available,
        unreasonable_time_presumption_triggered: unreasonable_time,
        procedural_requirements_satisfied: procedural_ok,
        tenant_may_pursue_escrow: may_pursue,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> RentEscrowInput {
        RentEscrowInput {
            state_code: state.to_string(),
            landlord_notified_of_defect: true,
            days_since_notice: 45,
            defect_materially_affects_health_safety: true,
            tenant_deposited_all_unpaid_rent: true,
            tenant_asserting_eviction_counterclaim: true,
            tenant_withholding_entire_rent: false,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn md_rent_escrow_act_regime() {
        let r = check(&baseline("MD"));
        assert_eq!(r.regime, RentEscrowRegime::MarylandRentEscrowAct);
    }

    #[test]
    fn ma_counterclaim_defense_regime() {
        let r = check(&baseline("MA"));
        assert_eq!(
            r.regime,
            RentEscrowRegime::MassachusettsCounterclaimDefense
        );
    }

    #[test]
    fn nj_marini_hearing_administrator_regime() {
        let r = check(&baseline("NJ"));
        assert_eq!(
            r.regime,
            RentEscrowRegime::NewJerseyMariniHearingAdministrator
        );
    }

    #[test]
    fn co_limited_withholding_regime() {
        let r = check(&baseline("CO"));
        assert_eq!(r.regime, RentEscrowRegime::ColoradoLimitedWithholding);
    }

    #[test]
    fn default_state_no_framework_regime() {
        for s in ["AL", "CA", "FL", "NY", "TX", "WA", "DC", "WY"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                RentEscrowRegime::NoStatutoryRentEscrowFramework,
                "expected {s} no-framework regime"
            );
        }
    }

    // ── MD Rent Escrow Act ─────────────────────────────────────────

    #[test]
    fn md_baseline_tenant_may_pursue() {
        let r = check(&baseline("MD"));
        assert!(r.tenant_may_pursue_escrow);
    }

    #[test]
    fn md_30_day_presumption_not_triggered_at_30_days() {
        let mut i = baseline("MD");
        i.days_since_notice = 30;
        let r = check(&i);
        assert!(
            !r.unreasonable_time_presumption_triggered,
            "MD 30-day presumption requires > 30 (strict greater)"
        );
    }

    #[test]
    fn md_31_day_presumption_triggered() {
        let mut i = baseline("MD");
        i.days_since_notice = 31;
        let r = check(&i);
        assert!(r.unreasonable_time_presumption_triggered);
    }

    #[test]
    fn md_no_notice_cannot_pursue() {
        let mut i = baseline("MD");
        i.landlord_notified_of_defect = false;
        let r = check(&i);
        assert!(!r.tenant_may_pursue_escrow);
        assert!(r.note.contains("landlord not notified"));
    }

    #[test]
    fn md_non_material_defect_cannot_pursue() {
        let mut i = baseline("MD");
        i.defect_materially_affects_health_safety = false;
        let r = check(&i);
        assert!(!r.tenant_may_pursue_escrow);
        assert!(r.note.contains("does not materially affect"));
    }

    // ── MA Counterclaim Defense ────────────────────────────────────

    #[test]
    fn ma_counterclaim_asserted_may_pursue() {
        let r = check(&baseline("MA"));
        assert!(r.tenant_may_pursue_escrow);
    }

    #[test]
    fn ma_no_counterclaim_cannot_pursue() {
        let mut i = baseline("MA");
        i.tenant_asserting_eviction_counterclaim = false;
        let r = check(&i);
        assert!(!r.tenant_may_pursue_escrow);
        assert!(r.note.contains("not asserting eviction counterclaim"));
    }

    // ── NJ Marini hearing ──────────────────────────────────────────

    #[test]
    fn nj_all_unpaid_rent_deposited_may_pursue() {
        let r = check(&baseline("NJ"));
        assert!(r.tenant_may_pursue_escrow);
    }

    #[test]
    fn nj_no_deposit_cannot_pursue() {
        let mut i = baseline("NJ");
        i.tenant_deposited_all_unpaid_rent = false;
        let r = check(&i);
        assert!(!r.tenant_may_pursue_escrow);
        assert!(r.note.contains("all unpaid rent not deposited"));
    }

    // ── CO limited withholding ─────────────────────────────────────

    #[test]
    fn co_not_withholding_entire_rent_may_pursue() {
        let r = check(&baseline("CO"));
        assert!(r.tenant_may_pursue_escrow);
    }

    #[test]
    fn co_withholding_entire_rent_blocked() {
        let mut i = baseline("CO");
        i.tenant_withholding_entire_rent = true;
        let r = check(&i);
        assert!(!r.tenant_may_pursue_escrow);
        assert!(r.note.contains("CO prohibits"));
    }

    // ── Default state ──────────────────────────────────────────────

    #[test]
    fn default_state_no_statutory_remedy() {
        let r = check(&baseline("CA"));
        assert!(!r.statutory_remedy_available);
        assert!(!r.tenant_may_pursue_escrow);
        assert!(r.note.contains("common-law warranty-of-habitability remedies"));
    }

    // ── Citations ──────────────────────────────────────────────────

    #[test]
    fn md_citation_mentions_8_211_and_30_days() {
        let r = check(&baseline("MD"));
        assert!(r.citation.contains("§ 8-211"));
        assert!(r.citation.contains("30 days"));
        assert!(r.citation.contains("Rent Escrow Act"));
    }

    #[test]
    fn ma_citation_mentions_c_239_8a() {
        let r = check(&baseline("MA"));
        assert!(r.citation.contains("c. 239 § 8A"));
        assert!(r.citation.contains("rent withholding statute"));
    }

    #[test]
    fn nj_citation_mentions_marini_and_administrator() {
        let r = check(&baseline("NJ"));
        assert!(r.citation.contains("2A:42-85"));
        assert!(r.citation.contains("Marini v. Ireland"));
        assert!(r.citation.contains("court-appointed administrator"));
        assert!(r.citation.contains("ALL unpaid rent"));
    }

    #[test]
    fn co_citation_mentions_38_12_507_and_entire_rent_bar() {
        let r = check(&baseline("CO"));
        assert!(r.citation.contains("38-12-507"));
        assert!(r.citation.contains("ENTIRE rent is NOT permitted"));
    }

    // ── Coverage / single-state-uniqueness ─────────────────────────

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        assert_eq!(RULES.len(), 51);
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} empty citation");
        }
    }

    #[test]
    fn md_only_30_day_presumption_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| r.reasonableness_presumption_days > 0)
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ma_only_counterclaim_mechanism_state() {
        let count = RULES.iter().filter(|(_, r)| r.counterclaim_defense_mechanism).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn nj_only_court_appointed_administrator_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| r.court_appointed_administrator_authorized)
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn nj_only_all_unpaid_rent_deposit_state() {
        let count = RULES.iter().filter(|(_, r)| r.all_unpaid_rent_deposit_required).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn co_only_entire_rent_withholding_prohibited_state() {
        let count = RULES.iter().filter(|(_, r)| r.entire_rent_withholding_prohibited).count();
        assert_eq!(count, 1);
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn md_compliant_note_describes_regime() {
        let r = check(&baseline("MD"));
        assert!(r.note.contains("Maryland Rent Escrow Act"));
    }

    #[test]
    fn ma_non_compliant_note_describes_counterclaim_requirement() {
        let mut i = baseline("MA");
        i.tenant_asserting_eviction_counterclaim = false;
        let r = check(&i);
        assert!(r.note.contains("counterclaim"));
    }

    // ── Normalization ──────────────────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&baseline("md"));
        assert_eq!(r.regime, RentEscrowRegime::MarylandRentEscrowAct);
    }
}
