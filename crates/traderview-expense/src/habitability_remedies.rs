//! State habitability warranty + tenant-remedy table — eighth
//! state-data module after `deposit_interest`, `late_fee_caps`,
//! `eviction_notices`, `contractor_1099`, `deposit_return_windows`,
//! `lease_disclosures`, and `rent_control`.
//!
//! When a landlord fails to maintain habitable conditions (broken
//! heat in winter, leaking roof, code violations, vermin), most
//! states grant tenants one or more remedies under the **implied
//! warranty of habitability** + state landlord-tenant code. The
//! available remedies vary dramatically:
//!
//!   * **Repair-and-deduct** — tenant pays for the repair, deducts
//!     from next rent payment. Most state caps: one month's rent
//!     (CA) or a fixed dollar amount.
//!
//!   * **Rent withholding into escrow** — tenant continues to pay
//!     rent into a court-supervised escrow account until landlord
//!     cures. Common in MA, WA, NJ, VA. Withholding rent OUTRIGHT
//!     (rather than into escrow) is illegal in most states and
//!     can be grounds for nonpayment eviction.
//!
//!   * **Lease termination** — tenant breaks lease without penalty.
//!     Available in most URLTA states.
//!
//!   * **Damages action** — tenant sues for actual + statutory
//!     damages. CA awards up to $5,000 + attorney's fees per
//!     CC §1942.4.
//!
//!   * **Eviction defense** — tenant raises habitability as
//!     affirmative defense to nonpayment eviction. Universally
//!     available in URLTA states + jurisdictions adopting the
//!     Pugh v. Holmes / Lemle v. Breeden line.
//!
//! Notice requirements + cure periods vary: TX requires 7 days
//! after written notice; CA requires "reasonable time"; WA
//! requires specific written demand + 30 days for major repairs.
//!
//! Pure data + compute. Caller passes the state + habitability
//! defect facts + cure history; we return the available remedies
//! with statute citations.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HabitabilityRemedy {
    /// Tenant fixes + deducts cost from rent up to statutory cap.
    RepairAndDeduct,
    /// Tenant pays rent into court-supervised escrow until cure.
    RentWithholdingEscrow,
    /// Tenant terminates lease without penalty after notice + cure.
    LeaseTermination,
    /// Tenant sues for actual + statutory damages + attorney fees.
    DamagesAction,
    /// Tenant raises habitability as affirmative defense to eviction.
    EvictionAffirmativeDefense,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub statute: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemedyRule {
    pub state: &'static str,
    pub remedy: HabitabilityRemedy,
    /// Days of notice required before tenant may invoke remedy. Zero
    /// when statute permits without notice (emergency conditions).
    pub notice_days_required: u32,
    /// Cure period landlord has after notice before tenant may act.
    pub cure_period_days: u32,
    /// Cap on repair-and-deduct amount. Zero when not applicable.
    pub repair_deduct_cap_usd: Decimal,
    /// Cap expressed as months of rent (e.g. 1 = one month's rent).
    /// Zero when only a dollar cap or no cap applies.
    pub repair_deduct_cap_months_rent: u32,
    /// Statutory damages multiplier on actual damages for damages-
    /// action remedy. 1.0 = no multiplier; CA = up to $5k + fees;
    /// FL = punitive on retaliation.
    pub damages_multiplier: Decimal,
    pub attorney_fees_to_prevailing_tenant: bool,
    pub effective_year: i32,
    pub notes: &'static str,
    pub citation: Citation,
}

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

use std::str::FromStr;

fn rules() -> &'static [RemedyRule] {
    static R: once_cell::sync::Lazy<Vec<RemedyRule>> = once_cell::sync::Lazy::new(|| {
        vec![
                // ─── California ──────────────────────────────────
                RemedyRule {
                    state: "CA",
                    remedy: HabitabilityRemedy::RepairAndDeduct,
                    notice_days_required: 0, // reasonable time
                    cure_period_days: 30, // case law "reasonable" ≈ 30 days
                    repair_deduct_cap_usd: Decimal::ZERO,
                    repair_deduct_cap_months_rent: 1,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: false,
                    effective_year: 2024,
                    notes: "Cal. Civ. Code §1942: tenant can repair-and-deduct up to one month's rent after 30 days' reasonable time, twice per 12 months. Habitability standard per Civ. §1941.1.",
                    citation: Citation {
                        statute: "Cal. Civ. Code §1942",
                        source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1942.&lawCode=CIV",
                    },
                },
                RemedyRule {
                    state: "CA",
                    remedy: HabitabilityRemedy::DamagesAction,
                    notice_days_required: 30,
                    cure_period_days: 0,
                    repair_deduct_cap_usd: d("5000"),
                    repair_deduct_cap_months_rent: 0,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "Cal. Civ. Code §1942.4: actual damages + statutory damages $1,000-$5,000 + attorney's fees when landlord demands rent on a substandard property.",
                    citation: Citation {
                        statute: "Cal. Civ. Code §1942.4",
                        source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1942.4.&lawCode=CIV",
                    },
                },
                RemedyRule {
                    state: "CA",
                    remedy: HabitabilityRemedy::EvictionAffirmativeDefense,
                    notice_days_required: 0,
                    cure_period_days: 0,
                    repair_deduct_cap_usd: Decimal::ZERO,
                    repair_deduct_cap_months_rent: 0,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: false,
                    effective_year: 2024,
                    notes: "Green v. Superior Court (1974): habitability is affirmative defense to unlawful detainer for nonpayment.",
                    citation: Citation {
                        statute: "Green v. Superior Court (10 Cal. 3d 616)",
                        source: "https://law.justia.com/cases/california/supreme-court/3d/10/616.html",
                    },
                },
                // ─── Texas ───────────────────────────────────────
                RemedyRule {
                    state: "TX",
                    remedy: HabitabilityRemedy::RepairAndDeduct,
                    notice_days_required: 7,
                    cure_period_days: 7,
                    repair_deduct_cap_usd: Decimal::ZERO,
                    repair_deduct_cap_months_rent: 1,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "Tex. Prop. Code §92.0561: tenant can repair-and-deduct up to one month's rent after 7-day written notice + 7-day cure. Specific repairs only (broken hot water heater, sewage backup, etc.).",
                    citation: Citation {
                        statute: "Tex. Prop. Code §92.0561",
                        source: "https://statutes.capitol.texas.gov/Docs/PR/htm/PR.92.htm",
                    },
                },
                RemedyRule {
                    state: "TX",
                    remedy: HabitabilityRemedy::LeaseTermination,
                    notice_days_required: 7,
                    cure_period_days: 7,
                    repair_deduct_cap_usd: Decimal::ZERO,
                    repair_deduct_cap_months_rent: 0,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "Tex. Prop. Code §92.056(f): tenant may terminate after written notice + reasonable time for landlord to cure.",
                    citation: Citation {
                        statute: "Tex. Prop. Code §92.056",
                        source: "https://statutes.capitol.texas.gov/Docs/PR/htm/PR.92.htm",
                    },
                },
                // ─── New York ────────────────────────────────────
                RemedyRule {
                    state: "NY",
                    remedy: HabitabilityRemedy::RentWithholdingEscrow,
                    notice_days_required: 0,
                    cure_period_days: 0,
                    repair_deduct_cap_usd: Decimal::ZERO,
                    repair_deduct_cap_months_rent: 0,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: false,
                    effective_year: 2024,
                    notes: "RPL §235-b implied warranty of habitability — tenant can withhold rent and raise as defense to nonpayment proceeding; HP action permits court-ordered repairs.",
                    citation: Citation {
                        statute: "N.Y. Real Property Law §235-b",
                        source: "https://www.nysenate.gov/legislation/laws/RPP/235-B",
                    },
                },
                RemedyRule {
                    state: "NY",
                    remedy: HabitabilityRemedy::EvictionAffirmativeDefense,
                    notice_days_required: 0,
                    cure_period_days: 0,
                    repair_deduct_cap_usd: Decimal::ZERO,
                    repair_deduct_cap_months_rent: 0,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: false,
                    effective_year: 2024,
                    notes: "Park West Mgmt. v. Mitchell (1979): habitability is per-se defense to summary nonpayment proceeding.",
                    citation: Citation {
                        statute: "Park West Mgmt. v. Mitchell, 47 NY2d 316",
                        source: "https://casetext.com/case/park-west-management-corp-v-mitchell",
                    },
                },
                // ─── Illinois ────────────────────────────────────
                RemedyRule {
                    state: "IL",
                    remedy: HabitabilityRemedy::RepairAndDeduct,
                    notice_days_required: 14,
                    cure_period_days: 14,
                    repair_deduct_cap_usd: d("500"),
                    repair_deduct_cap_months_rent: 0,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "Chicago RLTO §5-12-110(c): tenant may repair-and-deduct lesser of $500 or half a month's rent after written notice + 14 days. State-wide (Jack Spring v. Little, 1972) implied warranty separately.",
                    citation: Citation {
                        statute: "Chicago Mun. Code §5-12-110",
                        source: "https://codelibrary.amlegal.com/codes/chicago/latest/chicago_il/0-0-0-2632135",
                    },
                },
                // ─── Washington ──────────────────────────────────
                RemedyRule {
                    state: "WA",
                    remedy: HabitabilityRemedy::RentWithholdingEscrow,
                    notice_days_required: 0,
                    cure_period_days: 30,
                    repair_deduct_cap_usd: Decimal::ZERO,
                    repair_deduct_cap_months_rent: 0,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "RCW 59.18.115: tenant may deposit rent into escrow + sue for breach of warranty after specific written notice + 30 days for major repairs.",
                    citation: Citation {
                        statute: "RCW 59.18.115",
                        source: "https://app.leg.wa.gov/rcw/default.aspx?cite=59.18.115",
                    },
                },
                RemedyRule {
                    state: "WA",
                    remedy: HabitabilityRemedy::RepairAndDeduct,
                    notice_days_required: 0,
                    cure_period_days: 10,
                    repair_deduct_cap_usd: Decimal::ZERO,
                    repair_deduct_cap_months_rent: 1,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "RCW 59.18.100: repair-and-deduct up to one month's rent after 10-day notice for repairs costing < 2 months' rent.",
                    citation: Citation {
                        statute: "RCW 59.18.100",
                        source: "https://app.leg.wa.gov/rcw/default.aspx?cite=59.18.100",
                    },
                },
                // ─── Florida ─────────────────────────────────────
                RemedyRule {
                    state: "FL",
                    remedy: HabitabilityRemedy::RentWithholdingEscrow,
                    notice_days_required: 7,
                    cure_period_days: 7,
                    repair_deduct_cap_usd: Decimal::ZERO,
                    repair_deduct_cap_months_rent: 0,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "Fla. Stat. §83.60: tenant can deposit rent into court registry after 7-day written notice + 7-day cure if landlord fails to maintain.",
                    citation: Citation {
                        statute: "Fla. Stat. §83.60",
                        source: "https://www.flsenate.gov/Laws/Statutes/2024/0083.60",
                    },
                },
                // ─── Massachusetts ───────────────────────────────
                RemedyRule {
                    state: "MA",
                    remedy: HabitabilityRemedy::RentWithholdingEscrow,
                    notice_days_required: 0,
                    cure_period_days: 0,
                    repair_deduct_cap_usd: Decimal::ZERO,
                    repair_deduct_cap_months_rent: 0,
                    damages_multiplier: d("3.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "G.L. c. 111 §127L + c.239 §8A: tenant may withhold rent for State Sanitary Code violations; consumer-protection claim (c.93A) yields treble damages + fees.",
                    citation: Citation {
                        statute: "Mass. Gen. Laws ch. 111 §127L + ch. 93A",
                        source: "https://malegislature.gov/Laws/GeneralLaws/PartI/TitleXVI/Chapter111/Section127L",
                    },
                },
                // ─── New Jersey ──────────────────────────────────
                RemedyRule {
                    state: "NJ",
                    remedy: HabitabilityRemedy::RentWithholdingEscrow,
                    notice_days_required: 0,
                    cure_period_days: 0,
                    repair_deduct_cap_usd: Decimal::ZERO,
                    repair_deduct_cap_months_rent: 0,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: false,
                    effective_year: 2024,
                    notes: "Marini v. Ireland (1970) + N.J.S.A. 2A:42-85: tenant may pay rent into court for breach of habitability; rent abatement available.",
                    citation: Citation {
                        statute: "Marini v. Ireland, 56 N.J. 130 + N.J.S.A. 2A:42-85",
                        source: "https://lis.njleg.state.nj.us/nxt/gateway.dll?f=templates&fn=default.htm",
                    },
                },
                // ─── Virginia ────────────────────────────────────
                RemedyRule {
                    state: "VA",
                    remedy: HabitabilityRemedy::RentWithholdingEscrow,
                    notice_days_required: 0,
                    cure_period_days: 21,
                    repair_deduct_cap_usd: d("1500"),
                    repair_deduct_cap_months_rent: 1,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "Va. Code §55.1-1244 (Tenant Assertion + Complaint): tenant may pay rent into court registry. §55.1-1245 repair-and-deduct up to greater of $1,500 or one month's rent.",
                    citation: Citation {
                        statute: "Va. Code §55.1-1244 + §55.1-1245",
                        source: "https://law.lis.virginia.gov/vacode/55.1-1244/",
                    },
                },
                // ─── Oregon ──────────────────────────────────────
                RemedyRule {
                    state: "OR",
                    remedy: HabitabilityRemedy::RepairAndDeduct,
                    notice_days_required: 0,
                    cure_period_days: 30,
                    repair_deduct_cap_usd: d("300"),
                    repair_deduct_cap_months_rent: 0,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "ORS 90.368 (Essential Services): tenant can pay for emergency repair (cost capped $300) + deduct after written notice + reasonable time. ORS 90.360 (Damages action) separately.",
                    citation: Citation {
                        statute: "ORS 90.368 + ORS 90.360",
                        source: "https://oregon.public.law/statutes/ors_90.368",
                    },
                },
                RemedyRule {
                    state: "OR",
                    remedy: HabitabilityRemedy::LeaseTermination,
                    notice_days_required: 30,
                    cure_period_days: 30,
                    repair_deduct_cap_usd: Decimal::ZERO,
                    repair_deduct_cap_months_rent: 0,
                    damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "ORS 90.360(1): tenant may terminate after 30 days written notice + 30-day cure if landlord materially breaches.",
                    citation: Citation {
                        statute: "ORS 90.360",
                        source: "https://oregon.public.law/statutes/ors_90.360",
                    },
                },
            ]
    });
    &R
}

pub fn remedies_for_state(state: &str) -> Vec<&'static RemedyRule> {
    let upper = state.to_uppercase();
    rules()
        .iter()
        .filter(|r| r.state.eq_ignore_ascii_case(&upper))
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitabilityRemediesInput {
    pub state: String,
    /// Current monthly rent — used to compute the repair-and-deduct
    /// cap when statute uses a months-of-rent expression.
    pub monthly_rent: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableRemedy {
    pub remedy: HabitabilityRemedy,
    pub notice_days_required: u32,
    pub cure_period_days: u32,
    pub repair_deduct_cap_dollars: Decimal,
    pub damages_multiplier: Decimal,
    pub attorney_fees_to_prevailing_tenant: bool,
    pub statute: String,
    pub source: String,
    pub notes: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HabitabilityRemediesReport {
    pub state_recognized: bool,
    pub remedies: Vec<AvailableRemedy>,
    pub notes: String,
}

pub fn remedies(input: &HabitabilityRemediesInput) -> HabitabilityRemediesReport {
    let mut r = HabitabilityRemediesReport::default();
    let upper = input.state.to_uppercase();
    let state_rules = remedies_for_state(&upper);

    if state_rules.is_empty() {
        r.notes = format!(
            "no habitability-remedy statute on file for {} — caller should consult state code directly",
            upper
        );
        return r;
    }
    r.state_recognized = true;

    let monthly_rent = input.monthly_rent.max(Decimal::ZERO);
    for rule in state_rules {
        let cap = if rule.repair_deduct_cap_months_rent > 0
            && rule.repair_deduct_cap_usd > Decimal::ZERO
        {
            // "Greater of" interpretation: VA §55.1-1245 uses greater of
            // $1,500 or one month's rent.
            let months_cap = monthly_rent * Decimal::from(rule.repair_deduct_cap_months_rent);
            months_cap.max(rule.repair_deduct_cap_usd)
        } else if rule.repair_deduct_cap_months_rent > 0 {
            monthly_rent * Decimal::from(rule.repair_deduct_cap_months_rent)
        } else {
            rule.repair_deduct_cap_usd
        };
        r.remedies.push(AvailableRemedy {
            remedy: rule.remedy,
            notice_days_required: rule.notice_days_required,
            cure_period_days: rule.cure_period_days,
            repair_deduct_cap_dollars: cap,
            damages_multiplier: rule.damages_multiplier,
            attorney_fees_to_prevailing_tenant: rule.attorney_fees_to_prevailing_tenant,
            statute: rule.citation.statute.into(),
            source: rule.citation.source.into(),
            notes: rule.notes.into(),
        });
    }
    r.notes = format!("{} remedy(ies) available for {}", r.remedies.len(), upper);
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn ca_has_three_remedies_modeled() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "CA".into(),
            monthly_rent: dec!(2000),
        });
        assert!(r.state_recognized);
        assert_eq!(r.remedies.len(), 3);
        assert!(r
            .remedies
            .iter()
            .any(|x| x.remedy == HabitabilityRemedy::RepairAndDeduct));
        assert!(r
            .remedies
            .iter()
            .any(|x| x.remedy == HabitabilityRemedy::DamagesAction));
        assert!(r
            .remedies
            .iter()
            .any(|x| x.remedy == HabitabilityRemedy::EvictionAffirmativeDefense));
    }

    #[test]
    fn ca_repair_deduct_cap_one_month_rent() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "CA".into(),
            monthly_rent: dec!(2500),
        });
        let rd = r
            .remedies
            .iter()
            .find(|x| x.remedy == HabitabilityRemedy::RepairAndDeduct)
            .unwrap();
        assert_eq!(rd.repair_deduct_cap_dollars, dec!(2500));
    }

    #[test]
    fn tx_repair_deduct_one_month_rent_with_7_day_notice() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "TX".into(),
            monthly_rent: dec!(1800),
        });
        let rd = r
            .remedies
            .iter()
            .find(|x| x.remedy == HabitabilityRemedy::RepairAndDeduct)
            .unwrap();
        assert_eq!(rd.notice_days_required, 7);
        assert_eq!(rd.cure_period_days, 7);
        assert_eq!(rd.repair_deduct_cap_dollars, dec!(1800));
    }

    #[test]
    fn il_repair_deduct_capped_at_500_dollars() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "IL".into(),
            monthly_rent: dec!(3000),
        });
        let rd = r
            .remedies
            .iter()
            .find(|x| x.remedy == HabitabilityRemedy::RepairAndDeduct)
            .unwrap();
        // Chicago RLTO: $500 fixed cap (we don't model the "half month's rent" alternative here).
        assert_eq!(rd.repair_deduct_cap_dollars, dec!(500));
    }

    #[test]
    fn va_repair_deduct_uses_greater_of_1500_or_month_rent() {
        // $1000 monthly rent → cap = max($1k, $1500) = $1500.
        let r = remedies(&HabitabilityRemediesInput {
            state: "VA".into(),
            monthly_rent: dec!(1000),
        });
        let escrow = r
            .remedies
            .iter()
            .find(|x| x.remedy == HabitabilityRemedy::RentWithholdingEscrow)
            .unwrap();
        assert_eq!(escrow.repair_deduct_cap_dollars, dec!(1500));

        // $2000 monthly rent → cap = max($2k, $1500) = $2000.
        let r2 = remedies(&HabitabilityRemediesInput {
            state: "VA".into(),
            monthly_rent: dec!(2000),
        });
        let escrow2 = r2
            .remedies
            .iter()
            .find(|x| x.remedy == HabitabilityRemedy::RentWithholdingEscrow)
            .unwrap();
        assert_eq!(escrow2.repair_deduct_cap_dollars, dec!(2000));
    }

    #[test]
    fn ma_treble_damages_multiplier_for_93a_action() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "MA".into(),
            monthly_rent: dec!(2500),
        });
        let withhold = r
            .remedies
            .iter()
            .find(|x| x.remedy == HabitabilityRemedy::RentWithholdingEscrow)
            .unwrap();
        assert_eq!(withhold.damages_multiplier, dec!(3));
        assert!(withhold.attorney_fees_to_prevailing_tenant);
    }

    #[test]
    fn ny_has_withholding_and_eviction_defense_no_repair_deduct() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "NY".into(),
            monthly_rent: dec!(3000),
        });
        assert!(r
            .remedies
            .iter()
            .any(|x| x.remedy == HabitabilityRemedy::RentWithholdingEscrow));
        assert!(r
            .remedies
            .iter()
            .any(|x| x.remedy == HabitabilityRemedy::EvictionAffirmativeDefense));
        // NY doesn't have a state-wide repair-and-deduct statute.
        assert!(!r
            .remedies
            .iter()
            .any(|x| x.remedy == HabitabilityRemedy::RepairAndDeduct));
    }

    #[test]
    fn fl_rent_into_court_registry_after_7_day_notice() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "FL".into(),
            monthly_rent: dec!(1500),
        });
        let withhold = r
            .remedies
            .iter()
            .find(|x| x.remedy == HabitabilityRemedy::RentWithholdingEscrow)
            .unwrap();
        assert_eq!(withhold.notice_days_required, 7);
        assert_eq!(withhold.cure_period_days, 7);
    }

    #[test]
    fn or_has_repair_deduct_and_termination_with_attorney_fees() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "OR".into(),
            monthly_rent: dec!(2000),
        });
        let rd = r
            .remedies
            .iter()
            .find(|x| x.remedy == HabitabilityRemedy::RepairAndDeduct)
            .unwrap();
        assert_eq!(rd.repair_deduct_cap_dollars, dec!(300));
        assert!(rd.attorney_fees_to_prevailing_tenant);
        assert!(r
            .remedies
            .iter()
            .any(|x| x.remedy == HabitabilityRemedy::LeaseTermination));
    }

    #[test]
    fn unknown_state_returns_not_recognized() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "XX".into(),
            monthly_rent: dec!(1000),
        });
        assert!(!r.state_recognized);
        assert!(r.notes.contains("no habitability-remedy statute"));
    }

    #[test]
    fn case_insensitive_state_lookup() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "ca".into(),
            monthly_rent: dec!(2000),
        });
        assert!(r.state_recognized);
        assert_eq!(r.remedies.len(), 3);
    }

    #[test]
    fn wa_repair_deduct_one_month_rent_at_10_day_cure() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "WA".into(),
            monthly_rent: dec!(2200),
        });
        let rd = r
            .remedies
            .iter()
            .find(|x| x.remedy == HabitabilityRemedy::RepairAndDeduct)
            .unwrap();
        assert_eq!(rd.cure_period_days, 10);
        assert_eq!(rd.repair_deduct_cap_dollars, dec!(2200));
    }

    #[test]
    fn wa_rent_withholding_escrow_30_day_cure() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "WA".into(),
            monthly_rent: dec!(2200),
        });
        let escrow = r
            .remedies
            .iter()
            .find(|x| x.remedy == HabitabilityRemedy::RentWithholdingEscrow)
            .unwrap();
        assert_eq!(escrow.cure_period_days, 30);
    }

    #[test]
    fn ca_damages_action_5000_cap_with_attorney_fees() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "CA".into(),
            monthly_rent: dec!(2500),
        });
        let damages = r
            .remedies
            .iter()
            .find(|x| x.remedy == HabitabilityRemedy::DamagesAction)
            .unwrap();
        assert_eq!(damages.repair_deduct_cap_dollars, dec!(5000));
        assert!(damages.attorney_fees_to_prevailing_tenant);
    }

    #[test]
    fn states_with_attorney_fees_flagged_correctly() {
        // For repair-and-deduct remedies specifically.
        for state in ["TX", "IL", "OR", "WA"] {
            let r = remedies(&HabitabilityRemediesInput {
                state: state.into(),
                monthly_rent: dec!(2000),
            });
            let rd = r
                .remedies
                .iter()
                .find(|x| x.remedy == HabitabilityRemedy::RepairAndDeduct)
                .unwrap();
            assert!(
                rd.attorney_fees_to_prevailing_tenant,
                "{} should award attorney fees",
                state
            );
        }
    }

    #[test]
    fn ca_eviction_defense_no_notice_required() {
        // Habitability as affirmative defense doesn't require pre-notice.
        let r = remedies(&HabitabilityRemediesInput {
            state: "CA".into(),
            monthly_rent: dec!(2000),
        });
        let defense = r
            .remedies
            .iter()
            .find(|x| x.remedy == HabitabilityRemedy::EvictionAffirmativeDefense)
            .unwrap();
        assert_eq!(defense.notice_days_required, 0);
        assert_eq!(defense.cure_period_days, 0);
    }

    #[test]
    fn report_total_count_matches_remedies_list_length() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "WA".into(),
            monthly_rent: dec!(2000),
        });
        assert!(r.notes.contains(&format!("{} remedy", r.remedies.len())));
    }

    #[test]
    fn remedies_for_state_helper_returns_all_matching_rules() {
        let ca = remedies_for_state("CA");
        assert_eq!(ca.len(), 3);
        let tx = remedies_for_state("TX");
        assert_eq!(tx.len(), 2);
    }

    #[test]
    fn citation_present_for_known_states() {
        let r = remedies(&HabitabilityRemediesInput {
            state: "MA".into(),
            monthly_rent: dec!(2000),
        });
        assert!(r.remedies[0].statute.contains("ch. 111"));
        let r2 = remedies(&HabitabilityRemediesInput {
            state: "TX".into(),
            monthly_rent: dec!(2000),
        });
        assert!(r2.remedies[0].statute.contains("92.0561"));
    }
}
