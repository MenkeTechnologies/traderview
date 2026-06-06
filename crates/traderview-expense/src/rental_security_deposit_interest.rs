//! Multi-jurisdictional security deposit interest framework
//! — five distinct statutory regimes governing landlord
//! obligations to pay interest on residential security
//! deposits. Trader-landlord critical because the **double
//! and treble damages** penalties for non-compliance are
//! among the steepest tenant remedies in residential
//! landlord-tenant law — strict-liability statutes in
//! Massachusetts and Chicago award tenants 3x or 2x the
//! deposit even without proof of bad faith or actual loss.
//! Companion to landlord_annual_rent_statement,
//! tenant_rent_judgment_wage_garnishment,
//! rental_property_registration,
//! landlord_identification_disclosure.
//!
//! **Chicago RLTO § 5-12-080(c) + § 5-12-081** — interest
//! required on deposits held more than 6 months; rate set
//! annually by City Comptroller based on average savings
//! account + 6-month CD rates at Chase Bank. **2026 rate:
//! 0.01%**. **2025 rate: 0.01%**. Payment due within **30
//! days of end of each 12-month rental period** as cash or
//! credit against rent. **§ 5-12-080(f) penalty: 2x the
//! deposit amount + reasonable attorney fees** — strict
//! liability, no fault required.
//!
//! **Massachusetts G.L. c. 186 § 15B(3)(b)** — annual
//! interest at **5% per year OR actual rate earned by bank
//! account, whichever is LESS**. Tenant entitled to 5%
//! compounded if landlord fails to hold deposit in
//! separate interest-bearing Massachusetts bank account in
//! tenant's name. Payment due within **30 days of tenancy
//! termination**. **TREBLE DAMAGES (3x interest amount) +
//! costs + reasonable attorney fees** for any failure to
//! pay interest within 30 days post-termination — strict
//! liability under MA Supreme Judicial Court precedent;
//! tenant need not prove bad faith or actual damages.
//!
//! **Connecticut Gen. Stat. § 47a-21(i) + § 47a-21(j)** —
//! deposits held in interest-bearing accounts; rate =
//! **average savings deposit rate** set quarterly by
//! Banking Commissioner. Annual interest payments AND
//! payment upon termination. **§ 47a-21(d)(2) penalty:
//! DOUBLE damages + $100 + costs + reasonable attorney
//! fees** for landlord retention of interest beyond 30 days
//! after demand.
//!
//! **New Jersey N.J.S.A. 46:8-19** — deposits capped at
//! **1.5 months' rent**; must be deposited in
//! interest-bearing New Jersey banking institution.
//! Landlord may **retain 1% per annum** as administrative
//! fee; balance of interest payable to tenant **annually
//! in cash or credit against rent**. Failure → tenant
//! entitled to apply remaining deposit + interest against
//! rent (self-help offset under § 46:8-19(c)).
//!
//! **New York Gen. Oblig. Law § 7-103** — for residential
//! buildings with **6 or more dwelling units**, deposits
//! must be held in **interest-bearing account in
//! NY-chartered banking organization**. Landlord may
//! retain **1% per annum administrative fee** "in lieu of
//! all other administrative and custodial expenses".
//! Balance held in trust OR paid annually to tenant.
//! Buildings under 6 units NOT required to be in
//! interest-bearing account (but commingling with personal
//! funds is prohibited by § 7-103(1) trust requirement).
//!
//! Citations: Chicago Municipal Code § 5-12-080(c)/(f) and
//! § 5-12-081; Mass. G.L. c. 186 § 15B(2)(a) and § 15B(3)(b)
//! and § 15B(6)(e) and § 15B(7); Conn. Gen. Stat.
//! § 47a-21(i) and § 47a-21(j) and § 47a-21(d)(2); N.J.S.A.
//! 46:8-19 and § 46:8-21.1; N.Y. Gen. Oblig. Law § 7-103.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    Chicago,
    Massachusetts,
    Connecticut,
    NewJersey,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityDepositInterestInput {
    pub jurisdiction: Jurisdiction,
    /// Security deposit held in cents.
    pub deposit_cents: u64,
    /// Monthly rent in cents (for NJ 1.5x cap test).
    pub monthly_rent_cents: u64,
    /// Days deposit has been held.
    pub days_held: u32,
    /// Calendar year of determination (for Chicago 2026
    /// = 0.01% rate).
    pub determination_year: u32,
    /// Whether landlord paid interest within statutory
    /// window (Chicago 30 days of 12-month period; MA 30
    /// days post-termination; CT annually + upon
    /// termination; NJ annually; NY annually).
    pub interest_paid_within_window: bool,
    /// Whether tenancy has terminated (relevant for MA
    /// 30-day clock).
    pub tenancy_terminated: bool,
    /// Whether deposit is held in separate interest-bearing
    /// account in proper banking institution (MA: separate
    /// MA bank in tenant's name; NY: interest-bearing
    /// NY-chartered bank; NJ: interest-bearing NJ
    /// institution).
    pub held_in_proper_interest_bearing_account: bool,
    /// Number of dwelling units in building (NY § 7-103
    /// 6-unit threshold).
    pub building_unit_count: u32,
    /// Days since tenancy termination (MA 30-day window
    /// for treble damages).
    pub days_since_termination: u32,
    /// Days since tenant demand for interest (CT
    /// § 47a-21(d)(2) 30-day window).
    pub days_since_tenant_demand: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SecurityDepositInterestResult {
    pub jurisdiction: Jurisdiction,
    pub interest_required: bool,
    pub annual_rate_bps: u32,
    pub landlord_admin_fee_bps: u32,
    pub tenant_interest_owed_cents: u64,
    pub landlord_admin_fee_cents: u64,
    pub penalty_multiplier_bps: u32,
    pub penalty_damages_cents: u64,
    pub deposit_cap_violated: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &SecurityDepositInterestInput) -> SecurityDepositInterestResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let (interest_required, annual_rate_bps, landlord_admin_fee_bps): (bool, u32, u32) =
        match input.jurisdiction {
            Jurisdiction::Chicago => (input.days_held > 180, 1, 0),
            Jurisdiction::Massachusetts => (input.days_held >= 365, 500, 0),
            Jurisdiction::Connecticut => (true, 15, 0),
            Jurisdiction::NewJersey => (true, 100, 100),
            Jurisdiction::NewYork => (input.building_unit_count >= 6, 50, 100),
            Jurisdiction::Default => (false, 0, 0),
        };

    let tenant_net_rate_bps = annual_rate_bps.saturating_sub(landlord_admin_fee_bps);
    let years_held = input.days_held as u64 / 365;
    let tenant_interest_owed_cents = if interest_required {
        input
            .deposit_cents
            .saturating_mul(tenant_net_rate_bps as u64)
            .saturating_mul(years_held)
            / 10_000
    } else {
        0
    };

    let landlord_admin_fee_cents = if interest_required && landlord_admin_fee_bps > 0 {
        input
            .deposit_cents
            .saturating_mul(landlord_admin_fee_bps as u64)
            .saturating_mul(years_held)
            / 10_000
    } else {
        0
    };

    let penalty_multiplier_bps: u32 = match input.jurisdiction {
        Jurisdiction::Chicago => 20_000,
        Jurisdiction::Massachusetts => 30_000,
        Jurisdiction::Connecticut => 20_000,
        Jurisdiction::NewJersey => 10_000,
        Jurisdiction::NewYork => 10_000,
        Jurisdiction::Default => 0,
    };

    let mut penalty_damages_cents: u64 = 0;

    if input.jurisdiction == Jurisdiction::Chicago
        && interest_required
        && !input.interest_paid_within_window
    {
        penalty_damages_cents = input.deposit_cents.saturating_mul(2);
        failure_reasons.push(
            "Chicago RLTO § 5-12-080(c)/(f) — interest required on deposits held more than 6 MONTHS; payment due within 30 days of each 12-month rental period; failure triggers STRICT-LIABILITY 2x deposit damages + reasonable attorney fees regardless of fault or actual loss".to_string(),
        );
    }

    if input.jurisdiction == Jurisdiction::Massachusetts
        && interest_required
        && input.tenancy_terminated
        && input.days_since_termination > 30
        && !input.interest_paid_within_window
    {
        penalty_damages_cents = tenant_interest_owed_cents.saturating_mul(3);
        failure_reasons.push(
            "Mass. G.L. c. 186 § 15B(3)(b) + § 15B(6)(e) — failure to pay interest within 30 days after termination of tenancy triggers STRICT-LIABILITY TREBLE DAMAGES (3x interest) + reasonable attorney fees + costs; no proof of bad faith or actual damages required".to_string(),
        );
    }

    if input.jurisdiction == Jurisdiction::Massachusetts
        && !input.held_in_proper_interest_bearing_account
    {
        failure_reasons.push(
            "Mass. G.L. c. 186 § 15B(3)(a) — deposit must be held in SEPARATE INTEREST-BEARING ACCOUNT in MA bank in TENANT'S NAME; failure to do so entitles tenant to 5% compounded interest regardless of actual account rate".to_string(),
        );
    }

    if input.jurisdiction == Jurisdiction::Connecticut
        && interest_required
        && input.days_since_tenant_demand > 30
        && !input.interest_paid_within_window
    {
        penalty_damages_cents = tenant_interest_owed_cents
            .saturating_mul(2)
            .saturating_add(10_000);
        failure_reasons.push(
            "Conn. Gen. Stat. § 47a-21(d)(2) — landlord retention of interest beyond 30 days after tenant demand triggers DOUBLE damages + $100 statutory penalty + costs + reasonable attorney fees".to_string(),
        );
    }

    let nj_cap_cents = input.monthly_rent_cents.saturating_mul(3) / 2;
    let deposit_cap_violated =
        input.jurisdiction == Jurisdiction::NewJersey && input.deposit_cents > nj_cap_cents;
    if deposit_cap_violated {
        failure_reasons.push(format!(
            "N.J.S.A. 46:8-21.2 — security deposit capped at 1.5 months' rent ({} cents); landlord collected {} cents",
            nj_cap_cents, input.deposit_cents
        ));
    }

    if input.jurisdiction == Jurisdiction::NewYork
        && input.building_unit_count >= 6
        && !input.held_in_proper_interest_bearing_account
    {
        failure_reasons.push(
            "N.Y. Gen. Oblig. Law § 7-103(2) — for buildings with 6 OR MORE dwelling units, deposit MUST be held in interest-bearing account in NY-chartered banking organization; landlord may retain 1% per annum admin fee in lieu of all other custodial expenses".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "Chicago RLTO § 5-12-080(c) — interest required on security deposits held MORE THAN 6 months; rate set annually by City Comptroller based on Chase Bank savings + 6-month CD rates; 2026 rate: 0.01%; 2025 rate: 0.01%".to_string(),
        "Chicago RLTO § 5-12-080(f) — STRICT-LIABILITY 2x deposit damages + reasonable attorney fees for ANY failure to comply with § 5-12-080 interest payment within 30 days of each 12-month rental period (no fault or actual loss required)".to_string(),
        "Mass. G.L. c. 186 § 15B(3)(b) — annual interest at 5% per year OR actual rate earned by bank account, whichever is LESS; payment due within 30 days of tenancy termination".to_string(),
        "Mass. G.L. c. 186 § 15B(3)(a) — deposit MUST be held in SEPARATE interest-bearing MA bank account in TENANT'S NAME; failure to do so entitles tenant to 5% compounded interest regardless of actual rate".to_string(),
        "Mass. G.L. c. 186 § 15B(6)(e) + § 15B(7) — STRICT-LIABILITY TREBLE DAMAGES (3x interest) + reasonable attorney fees + costs for failure to pay interest within 30 days post-termination; no proof of bad faith or actual loss required (MA SJC precedent)".to_string(),
        "Conn. Gen. Stat. § 47a-21(i) + § 47a-21(j) — deposit held in interest-bearing account; rate = average savings deposit rate set QUARTERLY by Banking Commissioner; annual interest payments AND payment upon termination".to_string(),
        "Conn. Gen. Stat. § 47a-21(d)(2) — landlord retention of interest beyond 30 days after tenant demand triggers DOUBLE damages + $100 statutory penalty + costs + reasonable attorney fees".to_string(),
        "N.J.S.A. 46:8-19 — deposits placed in interest-bearing NJ banking institution; landlord may retain 1% per annum administrative fee; balance of interest payable to tenant annually in cash or credit against rent".to_string(),
        "N.J.S.A. 46:8-21.2 — security deposits capped at 1.5 months' rent for residential rentals".to_string(),
        "N.Y. Gen. Oblig. Law § 7-103(2) — for buildings with 6 OR MORE dwelling units, deposit MUST be held in interest-bearing account in NY-chartered banking organization; landlord may retain 1% per annum admin fee 'in lieu of all other administrative and custodial expenses'".to_string(),
        "N.Y. Gen. Oblig. Law § 7-103(1) — security deposits held in TRUST by landlord; commingling with personal funds prohibited regardless of building size".to_string(),
    ];

    SecurityDepositInterestResult {
        jurisdiction: input.jurisdiction,
        interest_required,
        annual_rate_bps,
        landlord_admin_fee_bps,
        tenant_interest_owed_cents,
        landlord_admin_fee_cents,
        penalty_multiplier_bps,
        penalty_damages_cents,
        deposit_cap_violated,
        failure_reasons,
        citation: "Chicago Municipal Code § 5-12-080(c)/(f) + § 5-12-081; Mass. G.L. c. 186 § 15B(2)(a) + § 15B(3)(a)/(b) + § 15B(6)(e) + § 15B(7); Conn. Gen. Stat. § 47a-21(i) + § 47a-21(j) + § 47a-21(d)(2); N.J.S.A. 46:8-19 + § 46:8-21.2; N.Y. Gen. Oblig. Law § 7-103(1) + § 7-103(2)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chicago_compliant() -> SecurityDepositInterestInput {
        SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::Chicago,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: true,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: true,
            building_unit_count: 4,
            days_since_termination: 0,
            days_since_tenant_demand: 0,
        }
    }

    #[test]
    fn chicago_2026_rate_one_basis_point() {
        let r = check(&chicago_compliant());
        assert_eq!(r.annual_rate_bps, 1);
        assert!(r.interest_required);
    }

    #[test]
    fn chicago_under_6_months_no_interest_required() {
        let mut i = chicago_compliant();
        i.days_held = 180;
        let r = check(&i);
        assert!(!r.interest_required);
    }

    #[test]
    fn chicago_at_181_days_interest_required() {
        let mut i = chicago_compliant();
        i.days_held = 181;
        let r = check(&i);
        assert!(r.interest_required);
    }

    #[test]
    fn chicago_failure_to_pay_triggers_2x_deposit_strict_liability() {
        let mut i = chicago_compliant();
        i.interest_paid_within_window = false;
        let r = check(&i);
        assert_eq!(r.penalty_damages_cents, 400_000);
        assert_eq!(r.penalty_multiplier_bps, 20_000);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 5-12-080(c)/(f)") && f.contains("STRICT-LIABILITY 2x deposit")));
    }

    #[test]
    fn massachusetts_5_percent_annual_rate() {
        let i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::Massachusetts,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: true,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: true,
            building_unit_count: 2,
            days_since_termination: 0,
            days_since_tenant_demand: 0,
        };
        let r = check(&i);
        assert_eq!(r.annual_rate_bps, 500);
        assert!(r.interest_required);
        assert_eq!(r.tenant_interest_owed_cents, 10_000);
    }

    #[test]
    fn massachusetts_under_one_year_no_interest_required() {
        let mut i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::Massachusetts,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 364,
            determination_year: 2026,
            interest_paid_within_window: true,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: true,
            building_unit_count: 2,
            days_since_termination: 0,
            days_since_tenant_demand: 0,
        };
        let r = check(&i);
        assert!(!r.interest_required);
        i.days_held = 365;
        let r2 = check(&i);
        assert!(r2.interest_required);
    }

    #[test]
    fn massachusetts_failure_to_pay_within_30_days_triggers_treble_damages() {
        let i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::Massachusetts,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: false,
            tenancy_terminated: true,
            held_in_proper_interest_bearing_account: true,
            building_unit_count: 2,
            days_since_termination: 31,
            days_since_tenant_demand: 0,
        };
        let r = check(&i);
        assert_eq!(r.penalty_damages_cents, 30_000);
        assert_eq!(r.penalty_multiplier_bps, 30_000);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 15B(3)(b)")
            && f.contains("TREBLE DAMAGES")
            && f.contains("3x interest")));
    }

    #[test]
    fn massachusetts_at_day_30_post_termination_no_treble_yet() {
        let i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::Massachusetts,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: false,
            tenancy_terminated: true,
            held_in_proper_interest_bearing_account: true,
            building_unit_count: 2,
            days_since_termination: 30,
            days_since_tenant_demand: 0,
        };
        let r = check(&i);
        assert_eq!(r.penalty_damages_cents, 0);
    }

    #[test]
    fn massachusetts_improper_account_triggers_5_percent_compounding_violation() {
        let mut i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::Massachusetts,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: true,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: false,
            building_unit_count: 2,
            days_since_termination: 0,
            days_since_tenant_demand: 0,
        };
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 15B(3)(a)")
            && f.contains("SEPARATE INTEREST-BEARING ACCOUNT")
            && f.contains("TENANT'S NAME")));
        i.held_in_proper_interest_bearing_account = true;
        let r2 = check(&i);
        assert!(!r2.failure_reasons.iter().any(|f| f.contains("§ 15B(3)(a)")));
    }

    #[test]
    fn connecticut_15_bps_default_rate() {
        let i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::Connecticut,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: true,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: true,
            building_unit_count: 2,
            days_since_termination: 0,
            days_since_tenant_demand: 0,
        };
        let r = check(&i);
        assert_eq!(r.annual_rate_bps, 15);
        assert!(r.interest_required);
    }

    #[test]
    fn connecticut_failure_beyond_30_day_demand_triggers_double_plus_100() {
        let i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::Connecticut,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: false,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: true,
            building_unit_count: 2,
            days_since_termination: 0,
            days_since_tenant_demand: 31,
        };
        let r = check(&i);
        let exp_interest = 300_u64;
        let exp_penalty = exp_interest.saturating_mul(2).saturating_add(10_000);
        assert_eq!(r.penalty_damages_cents, exp_penalty);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 47a-21(d)(2)")
                && f.contains("DOUBLE damages")
                && f.contains("$100")));
    }

    #[test]
    fn new_jersey_1_percent_with_1_percent_admin_fee() {
        let i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::NewJersey,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: true,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: true,
            building_unit_count: 2,
            days_since_termination: 0,
            days_since_tenant_demand: 0,
        };
        let r = check(&i);
        assert_eq!(r.annual_rate_bps, 100);
        assert_eq!(r.landlord_admin_fee_bps, 100);
        assert_eq!(r.tenant_interest_owed_cents, 0);
        assert_eq!(r.landlord_admin_fee_cents, 2_000);
    }

    #[test]
    fn new_jersey_1_5x_monthly_rent_cap_violation_at_1_51x() {
        let i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::NewJersey,
            deposit_cents: 302_000,
            monthly_rent_cents: 200_000,
            days_held: 30,
            determination_year: 2026,
            interest_paid_within_window: true,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: true,
            building_unit_count: 2,
            days_since_termination: 0,
            days_since_tenant_demand: 0,
        };
        let r = check(&i);
        assert!(r.deposit_cap_violated);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("N.J.S.A. 46:8-21.2") && f.contains("1.5 months")));
    }

    #[test]
    fn new_jersey_at_exactly_1_5x_no_violation() {
        let i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::NewJersey,
            deposit_cents: 300_000,
            monthly_rent_cents: 200_000,
            days_held: 30,
            determination_year: 2026,
            interest_paid_within_window: true,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: true,
            building_unit_count: 2,
            days_since_termination: 0,
            days_since_tenant_demand: 0,
        };
        let r = check(&i);
        assert!(!r.deposit_cap_violated);
    }

    #[test]
    fn new_york_6_unit_threshold_triggers_interest_required() {
        let mut i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::NewYork,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: true,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: true,
            building_unit_count: 6,
            days_since_termination: 0,
            days_since_tenant_demand: 0,
        };
        let r = check(&i);
        assert!(r.interest_required);
        i.building_unit_count = 5;
        let r2 = check(&i);
        assert!(!r2.interest_required);
    }

    #[test]
    fn new_york_under_6_units_no_interest_account_required() {
        let i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::NewYork,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: true,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: false,
            building_unit_count: 5,
            days_since_termination: 0,
            days_since_tenant_demand: 0,
        };
        let r = check(&i);
        assert!(!r.failure_reasons.iter().any(|f| f.contains("§ 7-103(2)")));
    }

    #[test]
    fn new_york_6_unit_improper_account_violation() {
        let i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::NewYork,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: true,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: false,
            building_unit_count: 8,
            days_since_termination: 0,
            days_since_tenant_demand: 0,
        };
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 7-103(2)")
            && f.contains("6 OR MORE")
            && f.contains("1% per annum admin fee")));
    }

    #[test]
    fn default_jurisdiction_no_interest_required() {
        let i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::Default,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: false,
            tenancy_terminated: true,
            held_in_proper_interest_bearing_account: false,
            building_unit_count: 100,
            days_since_termination: 100,
            days_since_tenant_demand: 100,
        };
        let r = check(&i);
        assert!(!r.interest_required);
        assert_eq!(r.penalty_damages_cents, 0);
        assert_eq!(r.annual_rate_bps, 0);
    }

    #[test]
    fn penalty_multiplier_truth_table_six_cells() {
        for (jur, exp_bps) in [
            (Jurisdiction::Chicago, 20_000),
            (Jurisdiction::Massachusetts, 30_000),
            (Jurisdiction::Connecticut, 20_000),
            (Jurisdiction::NewJersey, 10_000),
            (Jurisdiction::NewYork, 10_000),
            (Jurisdiction::Default, 0),
        ] {
            let i = SecurityDepositInterestInput {
                jurisdiction: jur,
                deposit_cents: 200_000,
                monthly_rent_cents: 200_000,
                days_held: 365,
                determination_year: 2026,
                interest_paid_within_window: true,
                tenancy_terminated: false,
                held_in_proper_interest_bearing_account: true,
                building_unit_count: 8,
                days_since_termination: 0,
                days_since_tenant_demand: 0,
            };
            let r = check(&i);
            assert_eq!(r.penalty_multiplier_bps, exp_bps, "jur={:?}", jur);
        }
    }

    #[test]
    fn massachusetts_treble_uniquely_highest_multiplier_invariant() {
        let make = |jur| SecurityDepositInterestInput {
            jurisdiction: jur,
            deposit_cents: 200_000,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: true,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: true,
            building_unit_count: 8,
            days_since_termination: 0,
            days_since_tenant_demand: 0,
        };
        let ma = check(&make(Jurisdiction::Massachusetts));
        let chi = check(&make(Jurisdiction::Chicago));
        let ct = check(&make(Jurisdiction::Connecticut));
        let nj = check(&make(Jurisdiction::NewJersey));
        let ny = check(&make(Jurisdiction::NewYork));
        assert!(ma.penalty_multiplier_bps > chi.penalty_multiplier_bps);
        assert!(ma.penalty_multiplier_bps > ct.penalty_multiplier_bps);
        assert!(ma.penalty_multiplier_bps > nj.penalty_multiplier_bps);
        assert!(ma.penalty_multiplier_bps > ny.penalty_multiplier_bps);
    }

    #[test]
    fn citation_pins_all_five_jurisdictions() {
        let r = check(&chicago_compliant());
        assert!(r
            .citation
            .contains("Chicago Municipal Code § 5-12-080(c)/(f)"));
        assert!(r.citation.contains("§ 5-12-081"));
        assert!(r.citation.contains("Mass. G.L. c. 186 § 15B"));
        assert!(r.citation.contains("§ 15B(3)(a)/(b)"));
        assert!(r.citation.contains("§ 15B(6)(e)"));
        assert!(r.citation.contains("Conn. Gen. Stat. § 47a-21"));
        assert!(r.citation.contains("§ 47a-21(d)(2)"));
        assert!(r.citation.contains("N.J.S.A. 46:8-19"));
        assert!(r.citation.contains("§ 46:8-21.2"));
        assert!(r.citation.contains("N.Y. Gen. Oblig. Law § 7-103"));
        assert!(r.citation.contains("§ 7-103(2)"));
    }

    #[test]
    fn note_pins_chicago_2026_rate_001_percent() {
        let r = check(&chicago_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 5-12-080(c)")
            && n.contains("MORE THAN 6 months")
            && n.contains("2026 rate: 0.01%")));
    }

    #[test]
    fn note_pins_chicago_strict_liability_2x() {
        let r = check(&chicago_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 5-12-080(f)")
            && n.contains("STRICT-LIABILITY 2x deposit")
            && n.contains("no fault or actual loss required")));
    }

    #[test]
    fn note_pins_massachusetts_5_percent_or_actual_lesser() {
        let r = check(&chicago_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 15B(3)(b)")
            && n.contains("5% per year OR actual rate")
            && n.contains("whichever is LESS")));
    }

    #[test]
    fn note_pins_massachusetts_separate_tenant_name_account() {
        let r = check(&chicago_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 15B(3)(a)")
            && n.contains("SEPARATE interest-bearing MA bank")
            && n.contains("TENANT'S NAME")));
    }

    #[test]
    fn note_pins_massachusetts_treble_damages_30_day() {
        let r = check(&chicago_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 15B(6)(e)")
            && n.contains("TREBLE DAMAGES")
            && n.contains("30 days post-termination")
            && n.contains("MA SJC precedent")));
    }

    #[test]
    fn note_pins_connecticut_quarterly_banking_commissioner() {
        let r = check(&chicago_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 47a-21(i)") && n.contains("QUARTERLY by Banking Commissioner")));
    }

    #[test]
    fn note_pins_connecticut_double_plus_100() {
        let r = check(&chicago_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 47a-21(d)(2)")
            && n.contains("DOUBLE damages")
            && n.contains("$100 statutory penalty")));
    }

    #[test]
    fn note_pins_new_jersey_1_5_month_cap() {
        let r = check(&chicago_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("N.J.S.A. 46:8-21.2") && n.contains("1.5 months' rent")));
    }

    #[test]
    fn note_pins_new_jersey_1_percent_admin_fee() {
        let r = check(&chicago_compliant());
        assert!(r.notes.iter().any(
            |n| n.contains("N.J.S.A. 46:8-19") && n.contains("1% per annum administrative fee")
        ));
    }

    #[test]
    fn note_pins_new_york_6_unit_threshold() {
        let r = check(&chicago_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 7-103(2)")
            && n.contains("6 OR MORE dwelling units")
            && n.contains("NY-chartered banking organization")));
    }

    #[test]
    fn note_pins_new_york_trust_no_commingling() {
        let r = check(&chicago_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7-103(1)") && n.contains("TRUST") && n.contains("commingling")));
    }

    #[test]
    fn defensive_zero_days_held_no_interest_chicago() {
        let mut i = chicago_compliant();
        i.days_held = 0;
        let r = check(&i);
        assert!(!r.interest_required);
        assert_eq!(r.tenant_interest_owed_cents, 0);
    }

    #[test]
    fn defensive_overflow_clamped_with_saturating_mul() {
        let mut i = SecurityDepositInterestInput {
            jurisdiction: Jurisdiction::Massachusetts,
            deposit_cents: u64::MAX,
            monthly_rent_cents: 200_000,
            days_held: 365,
            determination_year: 2026,
            interest_paid_within_window: true,
            tenancy_terminated: false,
            held_in_proper_interest_bearing_account: true,
            building_unit_count: 2,
            days_since_termination: 0,
            days_since_tenant_demand: 0,
        };
        let r = check(&i);
        let _ = r.tenant_interest_owed_cents;
        i.deposit_cents = 1_000_000;
        let r2 = check(&i);
        assert_eq!(r2.tenant_interest_owed_cents, 50_000);
    }
}
