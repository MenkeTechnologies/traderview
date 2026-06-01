//! Tenant relocation assistance — landlord-paid relocation payments owed when
//! a no-fault eviction or qualifying displacement triggers a statutory transfer
//! from landlord to tenant.
//!
//! Three jurisdictions ship concrete dollar formulas; most other states leave
//! the question to local ordinance or to the lease itself. Modeling the three
//! that have legislated amounts gives the calculator real cash-flow numbers
//! while keeping the default branch honest (no obligation).
//!
//! Regimes:
//!
//! California (AB 1482, Civ. Code § 1946.2(d)(3)) — for any of the four
//! no-fault grounds (owner move-in, withdrawal from market, demolition or
//! substantial remodel, government order), the landlord owes ONE MONTH of
//! the rent in effect at notice service. Paid as direct payment within 15
//! calendar days of notice OR waived from the last month of rent. Strict
//! compliance is required — failure voids the termination notice.
//!
//! Portland, OR (PCC 30.01.085) — graduated by unit size: studio/SRO $2,900;
//! 1-bedroom $3,300; 2-bedroom $4,200; 3-bedroom+ $4,500. Triggered by
//! no-cause termination, qualifying landlord-cause termination, rent
//! increase ≥10% within a rolling 12-month window, or refusal to renew a
//! fixed-term lease that converts to month-to-month. Notice required at
//! least 90 days before effective date. Non-compliance penalty: up to 3×
//! monthly rent plus actual damages, attorney fees.
//!
//! Seattle TRAO (Tenant Relocation Assistance Ordinance, SMC 22.210) —
//! demolition / substantial rehab / change of use / removal of rent or
//! income restriction. Only LOW-INCOME households (≤50% Seattle AMI)
//! qualify. Total payment $5,552 (landlord pays half = $2,776, City pays
//! half = $2,776).
//!
//! Seattle EDRA (Economic Displacement Relocation Assistance, SMC 22.212,
//! effective July 1, 2022) — rent increase ≥10% in any rolling 12-month
//! period AND tenant household ≤80% AMI AND tenant gives notice to vacate.
//! Payment 3× current monthly housing cost, advanced by City and reimbursed
//! by landlord.
//!
//! Default — no statutory obligation. Lease terms or local ordinance may
//! still require payment; the compute return makes that ignorance explicit.
//!
//! Citations: Cal. Civ. Code § 1946.2(d)(3); Portland City Code § 30.01.085;
//! Seattle Municipal Code Ch. 22.210 (TRAO); Seattle Municipal Code Ch.
//! 22.212 (EDRA, eff. July 1, 2022).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplacementTrigger {
    OwnerMoveIn,
    WithdrawalFromMarket,
    DemolitionOrSubstantialRemodel,
    GovernmentOrder,
    RentIncreaseTenPercentPlus,
    NoCauseTermination,
    LandlordCauseTermination,
    SubstantialRehabOrChangeOfUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BedroomCount {
    StudioOrSro,
    OneBedroom,
    TwoBedroom,
    ThreeBedroomOrLarger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    CaliforniaAb1482,
    PortlandOr,
    SeattleTrao,
    SeattleEdra,
    Default,
}

impl Regime {
    pub fn for_jurisdiction(state: &str, city: &str) -> Self {
        let st = state.trim().to_ascii_uppercase();
        let ct = city.trim().to_ascii_lowercase();
        match (st.as_str(), ct.as_str()) {
            ("OR", "portland") => Self::PortlandOr,
            // Seattle has two ordinances; default to TRAO for displacement
            // triggers tied to building changes; caller should pass
            // SeattleEdra explicitly for rent-increase economic-displacement.
            ("WA", "seattle") => Self::SeattleTrao,
            ("CA", _) => Self::CaliforniaAb1482,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RelocationInput {
    pub regime: Regime,
    pub trigger: DisplacementTrigger,
    pub monthly_rent_cents: i64,
    pub bedrooms: BedroomCount,
    /// Tenant household area median income percent (e.g. 75 means 75% AMI).
    /// Only used by Seattle TRAO (≤50%) and Seattle EDRA (≤80%).
    pub household_ami_percent: u32,
    /// Whether the tenant agreed in writing to the termination — affects
    /// only the CA owner-move-in path for post-July-1-2020 leases. Out of
    /// scope of the dollar calc; modeled here so the note can flag it.
    pub tenant_agreed_in_writing: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RelocationResult {
    pub regime: Regime,
    pub trigger: DisplacementTrigger,
    pub amount_owed_cents: i64,
    /// For Seattle TRAO only — landlord pays $2,776, City pays $2,776.
    /// Returned as the landlord's portion in `amount_owed_cents`; this field
    /// holds the City's portion. Zero for every other regime.
    pub city_contribution_cents: i64,
    pub eligible: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &RelocationInput) -> RelocationResult {
    match input.regime {
        Regime::CaliforniaAb1482 => ca_compute(input),
        Regime::PortlandOr => portland_compute(input),
        Regime::SeattleTrao => seattle_trao_compute(input),
        Regime::SeattleEdra => seattle_edra_compute(input),
        Regime::Default => default_compute(input),
    }
}

fn ca_compute(input: &RelocationInput) -> RelocationResult {
    let qualifying = matches!(
        input.trigger,
        DisplacementTrigger::OwnerMoveIn
            | DisplacementTrigger::WithdrawalFromMarket
            | DisplacementTrigger::DemolitionOrSubstantialRemodel
            | DisplacementTrigger::GovernmentOrder
    );
    if !qualifying {
        return RelocationResult {
            regime: Regime::CaliforniaAb1482,
            trigger: input.trigger,
            amount_owed_cents: 0,
            city_contribution_cents: 0,
            eligible: false,
            citation:
                "Cal. Civ. Code § 1946.2(d)(3) — relocation owed only for the four no-fault grounds",
            note: format!(
                "Trigger {:?} is not a § 1946.2 no-fault ground; no relocation under AB 1482.",
                input.trigger
            ),
        };
    }
    let owner_move_in_caveat = if input.trigger == DisplacementTrigger::OwnerMoveIn
        && !input.tenant_agreed_in_writing
    {
        " For leases on or after July 1, 2020, owner move-in requires written tenant agreement or a lease clause permitting unilateral owner occupancy — verify the lease."
    } else {
        ""
    };
    RelocationResult {
        regime: Regime::CaliforniaAb1482,
        trigger: input.trigger,
        amount_owed_cents: input.monthly_rent_cents,
        city_contribution_cents: 0,
        eligible: true,
        citation:
            "Cal. Civ. Code § 1946.2(d)(3) — one month rent within 15 days of notice or waiver",
        note: format!(
            "AB 1482 owes ONE MONTH rent ({} cents) within 15 calendar days of the termination notice, or the landlord may waive the last month's rent. Strict compliance — failure voids the notice.{}",
            input.monthly_rent_cents, owner_move_in_caveat
        ),
    }
}

fn portland_compute(input: &RelocationInput) -> RelocationResult {
    let qualifying = matches!(
        input.trigger,
        DisplacementTrigger::NoCauseTermination
            | DisplacementTrigger::LandlordCauseTermination
            | DisplacementTrigger::RentIncreaseTenPercentPlus
            | DisplacementTrigger::OwnerMoveIn
            | DisplacementTrigger::DemolitionOrSubstantialRemodel
    );
    if !qualifying {
        return RelocationResult {
            regime: Regime::PortlandOr,
            trigger: input.trigger,
            amount_owed_cents: 0,
            city_contribution_cents: 0,
            eligible: false,
            citation: "Portland City Code § 30.01.085",
            note: format!(
                "Trigger {:?} is not a PCC 30.01.085 qualifying event.",
                input.trigger
            ),
        };
    }
    let amount_cents = match input.bedrooms {
        BedroomCount::StudioOrSro => 290000,
        BedroomCount::OneBedroom => 330000,
        BedroomCount::TwoBedroom => 420000,
        BedroomCount::ThreeBedroomOrLarger => 450000,
    };
    RelocationResult {
        regime: Regime::PortlandOr,
        trigger: input.trigger,
        amount_owed_cents: amount_cents,
        city_contribution_cents: 0,
        eligible: true,
        citation:
            "Portland City Code § 30.01.085 — graduated by bedroom count, 90-day notice required",
        note: format!(
            "Portland PCC 30.01.085 owes {} cents for {:?}. Notice required at least 90 days before effective date. Non-compliance: up to 3x monthly rent plus actual damages and attorney fees.",
            amount_cents, input.bedrooms
        ),
    }
}

fn seattle_trao_compute(input: &RelocationInput) -> RelocationResult {
    let qualifying_trigger = matches!(
        input.trigger,
        DisplacementTrigger::DemolitionOrSubstantialRemodel
            | DisplacementTrigger::SubstantialRehabOrChangeOfUse
    );
    let income_eligible = input.household_ami_percent <= 50;
    if !qualifying_trigger {
        return RelocationResult {
            regime: Regime::SeattleTrao,
            trigger: input.trigger,
            amount_owed_cents: 0,
            city_contribution_cents: 0,
            eligible: false,
            citation: "Seattle Municipal Code Ch. 22.210 (TRAO)",
            note: format!(
                "Trigger {:?} is not a TRAO qualifying event (demo/substantial-rehab/change-of-use only).",
                input.trigger
            ),
        };
    }
    if !income_eligible {
        return RelocationResult {
            regime: Regime::SeattleTrao,
            trigger: input.trigger,
            amount_owed_cents: 0,
            city_contribution_cents: 0,
            eligible: false,
            citation: "Seattle Municipal Code Ch. 22.210 (TRAO) — low-income (<=50% AMI) only",
            note: format!(
                "Household income {}% AMI exceeds the 50% threshold; no TRAO relocation owed.",
                input.household_ami_percent
            ),
        };
    }
    RelocationResult {
        regime: Regime::SeattleTrao,
        trigger: input.trigger,
        amount_owed_cents: 277600,
        city_contribution_cents: 277600,
        eligible: true,
        citation: "Seattle Municipal Code Ch. 22.210 — total $5,552 (landlord $2,776 + City $2,776)",
        note:
            "Seattle TRAO owes landlord half ($2,776) and City half ($2,776) of the $5,552 statutory amount. Only low-income (<=50% AMI) tenants qualify."
                .to_string(),
    }
}

fn seattle_edra_compute(input: &RelocationInput) -> RelocationResult {
    if input.trigger != DisplacementTrigger::RentIncreaseTenPercentPlus {
        return RelocationResult {
            regime: Regime::SeattleEdra,
            trigger: input.trigger,
            amount_owed_cents: 0,
            city_contribution_cents: 0,
            eligible: false,
            citation: "Seattle Municipal Code Ch. 22.212 (EDRA)",
            note: format!(
                "Trigger {:?} is not an EDRA event (rent-increase >=10% in rolling 12-month window only).",
                input.trigger
            ),
        };
    }
    if input.household_ami_percent > 80 {
        return RelocationResult {
            regime: Regime::SeattleEdra,
            trigger: input.trigger,
            amount_owed_cents: 0,
            city_contribution_cents: 0,
            eligible: false,
            citation: "Seattle Municipal Code Ch. 22.212 (EDRA) — <=80% AMI households only",
            note: format!(
                "Household income {}% AMI exceeds the 80% threshold; no EDRA assistance.",
                input.household_ami_percent
            ),
        };
    }
    let amount = input.monthly_rent_cents.saturating_mul(3);
    RelocationResult {
        regime: Regime::SeattleEdra,
        trigger: input.trigger,
        amount_owed_cents: amount,
        city_contribution_cents: 0,
        eligible: true,
        citation:
            "Seattle Municipal Code Ch. 22.212 (EDRA, eff. July 1, 2022) — 3x monthly housing cost",
        note: format!(
            "Seattle EDRA owes 3x monthly housing cost = {} cents. City advances payment to tenant; landlord reimburses City.",
            amount
        ),
    }
}

fn default_compute(input: &RelocationInput) -> RelocationResult {
    RelocationResult {
        regime: Regime::Default,
        trigger: input.trigger,
        amount_owed_cents: 0,
        city_contribution_cents: 0,
        eligible: false,
        citation: "No statewide relocation-assistance statute identified — check local ordinance",
        note:
            "Default regime: no statutory landlord-paid relocation assistance. Lease terms or local ordinances may still apply."
                .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        trigger: DisplacementTrigger,
        rent_cents: i64,
        bedrooms: BedroomCount,
        ami: u32,
    ) -> RelocationInput {
        RelocationInput {
            regime,
            trigger,
            monthly_rent_cents: rent_cents,
            bedrooms,
            household_ami_percent: ami,
            tenant_agreed_in_writing: false,
        }
    }

    #[test]
    fn ca_owner_move_in_owes_one_month_rent() {
        let r = compute(&input(
            Regime::CaliforniaAb1482,
            DisplacementTrigger::OwnerMoveIn,
            350000,
            BedroomCount::OneBedroom,
            120,
        ));
        assert!(r.eligible);
        assert_eq!(r.amount_owed_cents, 350000);
        assert!(r.note.contains("ONE MONTH"));
        assert!(r.note.contains("15 calendar days"));
        assert!(r.note.contains("July 1, 2020"));
    }

    #[test]
    fn ca_owner_move_in_with_written_consent_omits_caveat() {
        let mut i = input(
            Regime::CaliforniaAb1482,
            DisplacementTrigger::OwnerMoveIn,
            200000,
            BedroomCount::StudioOrSro,
            100,
        );
        i.tenant_agreed_in_writing = true;
        let r = compute(&i);
        assert!(r.eligible);
        assert!(!r.note.contains("July 1, 2020"));
    }

    #[test]
    fn ca_withdrawal_from_market_qualifies() {
        let r = compute(&input(
            Regime::CaliforniaAb1482,
            DisplacementTrigger::WithdrawalFromMarket,
            275000,
            BedroomCount::TwoBedroom,
            85,
        ));
        assert!(r.eligible);
        assert_eq!(r.amount_owed_cents, 275000);
    }

    #[test]
    fn ca_rent_increase_does_not_qualify() {
        let r = compute(&input(
            Regime::CaliforniaAb1482,
            DisplacementTrigger::RentIncreaseTenPercentPlus,
            200000,
            BedroomCount::OneBedroom,
            70,
        ));
        assert!(!r.eligible);
        assert_eq!(r.amount_owed_cents, 0);
        assert!(r.note.contains("not a § 1946.2 no-fault ground"));
    }

    #[test]
    fn portland_studio_owes_2900() {
        let r = compute(&input(
            Regime::PortlandOr,
            DisplacementTrigger::NoCauseTermination,
            150000,
            BedroomCount::StudioOrSro,
            100,
        ));
        assert!(r.eligible);
        assert_eq!(r.amount_owed_cents, 290000);
    }

    #[test]
    fn portland_one_bedroom_owes_3300() {
        let r = compute(&input(
            Regime::PortlandOr,
            DisplacementTrigger::NoCauseTermination,
            180000,
            BedroomCount::OneBedroom,
            100,
        ));
        assert_eq!(r.amount_owed_cents, 330000);
    }

    #[test]
    fn portland_two_bedroom_owes_4200() {
        let r = compute(&input(
            Regime::PortlandOr,
            DisplacementTrigger::NoCauseTermination,
            240000,
            BedroomCount::TwoBedroom,
            100,
        ));
        assert_eq!(r.amount_owed_cents, 420000);
    }

    #[test]
    fn portland_three_plus_bedroom_owes_4500() {
        let r = compute(&input(
            Regime::PortlandOr,
            DisplacementTrigger::NoCauseTermination,
            320000,
            BedroomCount::ThreeBedroomOrLarger,
            100,
        ));
        assert_eq!(r.amount_owed_cents, 450000);
    }

    #[test]
    fn portland_ten_percent_rent_increase_triggers() {
        let r = compute(&input(
            Regime::PortlandOr,
            DisplacementTrigger::RentIncreaseTenPercentPlus,
            200000,
            BedroomCount::OneBedroom,
            100,
        ));
        assert!(r.eligible);
        assert_eq!(r.amount_owed_cents, 330000);
        assert!(r.note.contains("90 days"));
        assert!(r.note.contains("3x monthly rent"));
    }

    #[test]
    fn portland_government_order_not_qualifying() {
        let r = compute(&input(
            Regime::PortlandOr,
            DisplacementTrigger::GovernmentOrder,
            200000,
            BedroomCount::OneBedroom,
            100,
        ));
        assert!(!r.eligible);
        assert!(r.note.contains("not a PCC 30.01.085 qualifying event"));
    }

    #[test]
    fn seattle_trao_low_income_demo_pays_split() {
        let r = compute(&input(
            Regime::SeattleTrao,
            DisplacementTrigger::DemolitionOrSubstantialRemodel,
            150000,
            BedroomCount::OneBedroom,
            45,
        ));
        assert!(r.eligible);
        assert_eq!(r.amount_owed_cents, 277600);
        assert_eq!(r.city_contribution_cents, 277600);
        assert!(r.note.contains("$5,552"));
        assert!(r.note.contains("<=50% AMI"));
    }

    #[test]
    fn seattle_trao_over_income_ineligible() {
        let r = compute(&input(
            Regime::SeattleTrao,
            DisplacementTrigger::DemolitionOrSubstantialRemodel,
            150000,
            BedroomCount::OneBedroom,
            65,
        ));
        assert!(!r.eligible);
        assert_eq!(r.amount_owed_cents, 0);
        assert!(r.note.contains("65% AMI exceeds the 50% threshold"));
    }

    #[test]
    fn seattle_trao_at_50_ami_boundary_qualifies() {
        let r = compute(&input(
            Regime::SeattleTrao,
            DisplacementTrigger::SubstantialRehabOrChangeOfUse,
            150000,
            BedroomCount::StudioOrSro,
            50,
        ));
        assert!(r.eligible);
        assert_eq!(r.amount_owed_cents, 277600);
    }

    #[test]
    fn seattle_trao_owner_move_in_not_qualifying() {
        let r = compute(&input(
            Regime::SeattleTrao,
            DisplacementTrigger::OwnerMoveIn,
            150000,
            BedroomCount::OneBedroom,
            45,
        ));
        assert!(!r.eligible);
        assert!(r.note.contains("not a TRAO qualifying event"));
    }

    #[test]
    fn seattle_edra_3x_rent_under_80_ami() {
        let r = compute(&input(
            Regime::SeattleEdra,
            DisplacementTrigger::RentIncreaseTenPercentPlus,
            210000,
            BedroomCount::OneBedroom,
            75,
        ));
        assert!(r.eligible);
        assert_eq!(r.amount_owed_cents, 630000);
        assert!(r.note.contains("3x monthly housing cost"));
        assert!(r.note.contains("City advances"));
    }

    #[test]
    fn seattle_edra_at_80_ami_boundary_qualifies() {
        let r = compute(&input(
            Regime::SeattleEdra,
            DisplacementTrigger::RentIncreaseTenPercentPlus,
            150000,
            BedroomCount::StudioOrSro,
            80,
        ));
        assert!(r.eligible);
        assert_eq!(r.amount_owed_cents, 450000);
    }

    #[test]
    fn seattle_edra_above_80_ami_ineligible() {
        let r = compute(&input(
            Regime::SeattleEdra,
            DisplacementTrigger::RentIncreaseTenPercentPlus,
            200000,
            BedroomCount::OneBedroom,
            81,
        ));
        assert!(!r.eligible);
        assert_eq!(r.amount_owed_cents, 0);
        assert!(r.note.contains("81% AMI exceeds the 80% threshold"));
    }

    #[test]
    fn seattle_edra_no_cause_termination_not_qualifying() {
        let r = compute(&input(
            Regime::SeattleEdra,
            DisplacementTrigger::NoCauseTermination,
            150000,
            BedroomCount::OneBedroom,
            50,
        ));
        assert!(!r.eligible);
        assert!(r.note.contains("not an EDRA event"));
    }

    #[test]
    fn default_regime_returns_zero() {
        let r = compute(&input(
            Regime::Default,
            DisplacementTrigger::OwnerMoveIn,
            250000,
            BedroomCount::OneBedroom,
            100,
        ));
        assert!(!r.eligible);
        assert_eq!(r.amount_owed_cents, 0);
        assert!(r.note.contains("no statutory"));
    }

    #[test]
    fn jurisdiction_lookup_routes_portland_seattle_california() {
        assert_eq!(
            Regime::for_jurisdiction("OR", "Portland"),
            Regime::PortlandOr
        );
        assert_eq!(
            Regime::for_jurisdiction("WA", "Seattle"),
            Regime::SeattleTrao
        );
        assert_eq!(
            Regime::for_jurisdiction("CA", "Anywhere"),
            Regime::CaliforniaAb1482
        );
        assert_eq!(Regime::for_jurisdiction("TX", "Austin"), Regime::Default);
    }

    #[test]
    fn jurisdiction_lookup_case_insensitive() {
        assert_eq!(
            Regime::for_jurisdiction("or", "PORTLAND"),
            Regime::PortlandOr
        );
        assert_eq!(
            Regime::for_jurisdiction("Wa", "seattle"),
            Regime::SeattleTrao
        );
    }

    #[test]
    fn ca_demolition_owes_one_month_rent() {
        let r = compute(&input(
            Regime::CaliforniaAb1482,
            DisplacementTrigger::DemolitionOrSubstantialRemodel,
            420000,
            BedroomCount::TwoBedroom,
            100,
        ));
        assert!(r.eligible);
        assert_eq!(r.amount_owed_cents, 420000);
    }

    #[test]
    fn ca_government_order_owes_one_month_rent() {
        let r = compute(&input(
            Regime::CaliforniaAb1482,
            DisplacementTrigger::GovernmentOrder,
            195000,
            BedroomCount::StudioOrSro,
            100,
        ));
        assert!(r.eligible);
        assert_eq!(r.amount_owed_cents, 195000);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let r_ca = compute(&input(
            Regime::CaliforniaAb1482,
            DisplacementTrigger::OwnerMoveIn,
            200000,
            BedroomCount::OneBedroom,
            100,
        ));
        assert!(r_ca.citation.contains("§ 1946.2(d)(3)"));

        let r_pdx = compute(&input(
            Regime::PortlandOr,
            DisplacementTrigger::NoCauseTermination,
            200000,
            BedroomCount::OneBedroom,
            100,
        ));
        assert!(r_pdx.citation.contains("30.01.085"));

        let r_trao = compute(&input(
            Regime::SeattleTrao,
            DisplacementTrigger::DemolitionOrSubstantialRemodel,
            200000,
            BedroomCount::OneBedroom,
            40,
        ));
        assert!(r_trao.citation.contains("22.210"));

        let r_edra = compute(&input(
            Regime::SeattleEdra,
            DisplacementTrigger::RentIncreaseTenPercentPlus,
            200000,
            BedroomCount::OneBedroom,
            60,
        ));
        assert!(r_edra.citation.contains("22.212"));
        assert!(r_edra.citation.contains("July 1, 2022"));
    }

    #[test]
    fn city_contribution_only_for_trao() {
        let r_ca = compute(&input(
            Regime::CaliforniaAb1482,
            DisplacementTrigger::OwnerMoveIn,
            200000,
            BedroomCount::OneBedroom,
            100,
        ));
        assert_eq!(r_ca.city_contribution_cents, 0);

        let r_pdx = compute(&input(
            Regime::PortlandOr,
            DisplacementTrigger::NoCauseTermination,
            200000,
            BedroomCount::OneBedroom,
            100,
        ));
        assert_eq!(r_pdx.city_contribution_cents, 0);

        let r_edra = compute(&input(
            Regime::SeattleEdra,
            DisplacementTrigger::RentIncreaseTenPercentPlus,
            200000,
            BedroomCount::OneBedroom,
            60,
        ));
        assert_eq!(r_edra.city_contribution_cents, 0);

        let r_trao = compute(&input(
            Regime::SeattleTrao,
            DisplacementTrigger::DemolitionOrSubstantialRemodel,
            200000,
            BedroomCount::OneBedroom,
            40,
        ));
        assert_eq!(r_trao.city_contribution_cents, 277600);
    }
}
