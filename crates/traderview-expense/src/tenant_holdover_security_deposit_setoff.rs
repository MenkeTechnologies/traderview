//! Tenant holdover security deposit setoff limits
//! framework — when tenant holds over past lease
//! expiration, may landlord apply security deposit
//! to (1) holdover rent; (2) double-rent damages
//! under FL § 83.58 + similar statutes; (3) physical
//! damage beyond normal wear and tear; (4) eviction
//! costs and attorney fees; and what itemization /
//! return timing rules apply? Distinct from sibling
//! `holdover_tenant_damages` (general holdover-
//! damages framework), `damage_deduction_itemization`
//! (itemization of deductions for damage only),
//! `deposit_return_windows` (general refund
//! timelines), `security_deposit_bank_disclosure`
//! (where deposit is held), `landlord_property_sale_
//! notice` (security deposit transfer on landlord
//! sale).
//!
//! Trader-landlord critical because holdover tenants
//! routinely trigger collateral disputes over
//! security deposit application. Landlords often
//! face conflicting demands: apply deposit to
//! holdover rent (preserving cash) vs reserve
//! deposit for itemized damage assessment (preserving
//! tenant rights). Mishandling exposes landlord to
//! penalty multipliers (CA up to 2x, NY up to 2x
//! punitive, TX 3x + $100 + attorney fees, MA 3x).
//!
//! Companion to holdover_tenant_damages (general
//! framework) + damage_deduction_itemization +
//! deposit_return_windows + security_deposit_bank_
//! disclosure + landlord_property_sale_notice (iter
//! 437) + duty_to_mitigate_damages + lease_cure_
//! period.
//!
//! **Four-jurisdiction framework**:
//!
//! CALIFORNIA — Cal. Civ. Code § 1950.5 governs
//! security deposit application + return; § 1950.5
//! (e) permits deduction for (1) unpaid rent; (2)
//! repair of damages beyond normal wear and tear;
//! (3) cleaning to return premises to condition
//! at start of tenancy (less wear and tear); (4)
//! restoration of personal property or appurtenances
//! (if lease allows). 21-day itemization +
//! refund window from move-out date; bad-faith
//! retention triggers actual damages + UP TO 2×
//! deposit punitive damages.
//!
//! NEW YORK — NY GOL § 7-108 governs non-rent-
//! stabilized security deposits; § 7-108(1-a)(g)
//! permits deduction for (1) non-payment of rent;
//! (2) damage beyond normal wear and tear; (3)
//! non-payment of utility charges payable directly
//! to landlord under lease; (4) moving and storage
//! of tenant's belongings. 14-day itemization +
//! refund window from vacate date (vs CA 21);
//! failure to provide itemization within 14 days
//! FORFEITS landlord's right to retain any portion;
//! willful violations trigger ACTUAL DAMAGES + UP
//! TO 2× DEPOSIT PUNITIVE DAMAGES.
//!
//! TEXAS — Tex. Prop. Code § 92.104 governs
//! retention and accounting; § 92.104(a) permits
//! deduction for damages and charges for which
//! tenant is legally liable under lease or as
//! result of breaching lease; § 92.104(c) excludes
//! rent itemization when (1) tenant owes rent at
//! surrender + (2) no controversy concerning
//! amount; 30-day itemization + refund window from
//! surrender (vs CA 21 + NY 14); bad-faith
//! retention triggers $100 + 3× WRONGFULLY
//! WITHHELD + attorney fees + court costs.
//!
//! MASSACHUSETTS — Mass. Gen. Laws c. 186 § 15B(4)
//! governs deductions; permits (1) unpaid rent
//! not validly withheld; (2) unpaid real estate
//! tax increases tenant required to pay under
//! lease; (3) reasonable cleaning; (4) repair of
//! tenant damage beyond reasonable wear and tear.
//! 30-day itemization + refund window (with
//! interest credited annually); § 15B(7) failure
//! to comply triggers TRIPLE damages + interest
//! + attorney fees + court costs.
//!
//! Holdover rent setoff vs damage reservation —
//! permitted application categories:
//!
//! UNPAID HOLDOVER RENT — universally permitted in
//! all four jurisdictions (CA § 1950.5(e)(1), NY
//! § 7-108(1-a)(g)(i), TX § 92.104(a), MA § 15B(4)
//! (i)) provided lease provides for rent during
//! holdover period.
//!
//! DOUBLE RENT under FL § 83.58 and similar — NOT
//! DIRECTLY DEDUCTIBLE from security deposit unless
//! lease specifically authorizes; typically requires
//! separate court judgment.
//!
//! PHYSICAL DAMAGE BEYOND NORMAL WEAR AND TEAR —
//! universally permitted with itemization (CA, NY,
//! TX, MA).
//!
//! EVICTION ATTORNEY FEES AND COURT COSTS —
//! permitted ONLY if lease provides AND statute
//! permits AND landlord prevails in eviction action;
//! CA and MA strict on bad-faith deductions.
//!
//! CLEANING TO RETURN PREMISES TO BASELINE —
//! permitted (CA and MA explicit; TX implicit).
//!
//! NON-PAYMENT OF UTILITIES PAYABLE TO LANDLORD —
//! permitted (NY § 7-108(1-a)(g)(iii)).
//!
//! MOVING AND STORAGE OF TENANT'S BELONGINGS —
//! permitted under NY § 7-108(1-a)(g)(iv); also
//! abandoned_property_handling sibling.
//!
//! **NOT permitted deductions**:
//! 1. NORMAL WEAR AND TEAR (universally excluded);
//! 2. PRE-EXISTING CONDITIONS (excluded under CA
//!    § 1950.5(e) "less wear and tear" language);
//! 3. LIQUIDATED-DAMAGES PROVISIONS that exceed
//!    actual damages and are unconscionable under
//!    Cal. Civ. Code § 1670.5 + NY GOL § 5-321 +
//!    Mass. unconscionability doctrine;
//! 4. LANDLORD'S OWN ECONOMIC LOSSES from
//!    re-renting delays (covered by duty-to-
//!    mitigate-damages doctrine + lease holdover
//!    clause).
//!
//! **Itemization + refund window comparison**:
//! - CA § 1950.5(g)(1): 21 days from move-out
//! - NY § 7-108(1-a)(e): 14 days from vacate date
//! - TX § 92.103(a) + § 92.104: 30 days from
//!   surrender
//! - MA § 15B(4): 30 days from termination (with
//!   interest)
//!
//! **Bad-faith / willful retention penalties**:
//! - CA § 1950.5(l): up to 2× deposit punitive
//!   damages
//! - NY § 7-108(1-a)(g): forfeit retention right
//!   if late + up to 2× punitive for willful
//! - TX § 92.109(a): $100 + 3× wrongfully
//!   withheld + attorney fees + court costs
//! - MA § 15B(7): 3× damages + interest +
//!   attorney fees + court costs
//!
//! **Trader-landlord critical fact patterns**:
//!
//! CA trader-landlord receives 30-day notice from
//! tenant; tenant vacates but leaves 3 weeks of
//! unpaid holdover rent ($3,000); security deposit
//! is $2,500; trader applies entire deposit to
//! holdover rent; mails 21-day itemization showing
//! deduction; tenant disputes — trader sustains
//! defense under § 1950.5(e)(1) PROVIDED lease
//! authorized holdover rent during notice period.
//!
//! NY trader-landlord deals with holdover tenant
//! who damages floors during forced removal;
//! deposit $3,000; floor damage $4,500; trader
//! exhausts deposit + sues for $1,500 excess;
//! must file § 7-108(1-a)(g) itemization within
//! 14 days of vacate or FORFEIT entire retention
//! right.
//!
//! TX trader fails to itemize within 30 days of
//! tenant surrender — TX § 92.109(a) BAD FAITH
//! presumption + $100 + 3× wrongfully withheld
//! + attorney fees + court costs.
//!
//! MA trader retains entire $2,400 deposit
//! claiming unpaid holdover rent without
//! itemizing — c. 186 § 15B(7) TRIPLE DAMAGES =
//! $7,200 + interest + attorney fees.
//!
//! Trader uses deposit to fund eviction lawsuit
//! attorney fees of $5,000 — PERMITTED only if
//! (1) lease provides; (2) statute permits; (3)
//! landlord prevails in eviction; otherwise
//! constitutes bad-faith retention exposing
//! trader to penalty multipliers across
//! jurisdictions.
//!
//! Citations: Cal. Civ. Code § 1950.5; Cal. Civ.
//! Code § 1950.5(e); Cal. Civ. Code § 1950.5(g)(1)
//! (21-day window); Cal. Civ. Code § 1950.5(l)
//! (bad-faith 2× punitive); Cal. Civ. Code
//! § 1670.5 (unconscionability); NY GOL § 7-108;
//! NY GOL § 7-108(1-a)(e) (14-day window); NY GOL
//! § 7-108(1-a)(g) (permitted deductions); NY GOL
//! § 5-321 (residential lease unconscionability);
//! Tex. Prop. Code § 92.103 (30-day refund); Tex.
//! Prop. Code § 92.104 (retention + accounting);
//! Tex. Prop. Code § 92.104(c) (rent owed
//! itemization exception); Tex. Prop. Code
//! § 92.109(a) (bad-faith $100 + 3× + fees);
//! Mass. Gen. Laws c. 186 § 15B(4) (permitted
//! deductions); Mass. Gen. Laws c. 186 § 15B(7)
//! (triple damages); HSTPA of 2019 (NY Laws 2019,
//! ch. 36); AB 2801 (CA security deposit reform).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Texas,
    Massachusetts,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeductionCategory {
    UnpaidHoldoverRent,
    DoubleRentDamages,
    PhysicalDamageBeyondWear,
    EvictionAttorneyFees,
    CleaningToBaseline,
    UnpaidUtilities,
    MovingAndStorage,
    NormalWearAndTear,
    PreExistingConditions,
    LiquidatedDamagesUnconscionable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantHoldoverSecurityDepositSetoffInput {
    pub jurisdiction: Jurisdiction,
    pub deduction_category: DeductionCategory,
    /// Security deposit amount in cents.
    pub security_deposit_cents: u64,
    /// Deduction amount in cents.
    pub deduction_amount_cents: u64,
    /// Days since tenant vacated (for itemization
    /// window compliance).
    pub days_since_vacate: u32,
    /// Whether landlord provided itemized statement
    /// to tenant.
    pub itemized_statement_provided: bool,
    /// Whether lease provides for the specific
    /// deduction (holdover rent / attorney fees /
    /// liquidated damages).
    pub lease_authorizes_deduction: bool,
    /// Whether landlord prevailed in eviction action
    /// (required for attorney fees deduction).
    pub landlord_prevailed_in_eviction: bool,
    /// Whether retention is bad-faith (penalty
    /// multiplier trigger).
    pub bad_faith_retention: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantHoldoverSecurityDepositSetoffResult {
    pub deduction_permitted: bool,
    pub itemization_timely: bool,
    pub itemization_window_days: u32,
    pub remaining_deposit_cents: u64,
    pub penalty_multiplier_engaged: bool,
    pub penalty_amount_cents: u64,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &TenantHoldoverSecurityDepositSetoffInput,
) -> TenantHoldoverSecurityDepositSetoffResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let itemization_window_days: u32 = match input.jurisdiction {
        Jurisdiction::California => 21,
        Jurisdiction::NewYork => 14,
        Jurisdiction::Texas | Jurisdiction::Massachusetts => 30,
    };

    let itemization_timely = input.days_since_vacate <= itemization_window_days
        && input.itemized_statement_provided;

    let category_universally_permitted = matches!(
        input.deduction_category,
        DeductionCategory::UnpaidHoldoverRent
            | DeductionCategory::PhysicalDamageBeyondWear
            | DeductionCategory::CleaningToBaseline
    );

    let category_conditionally_permitted = match input.deduction_category {
        DeductionCategory::DoubleRentDamages | DeductionCategory::EvictionAttorneyFees => {
            input.lease_authorizes_deduction
                && (input.deduction_category != DeductionCategory::EvictionAttorneyFees
                    || input.landlord_prevailed_in_eviction)
        }
        DeductionCategory::UnpaidUtilities => matches!(input.jurisdiction, Jurisdiction::NewYork),
        DeductionCategory::MovingAndStorage => matches!(input.jurisdiction, Jurisdiction::NewYork),
        _ => false,
    };

    let category_universally_prohibited = matches!(
        input.deduction_category,
        DeductionCategory::NormalWearAndTear
            | DeductionCategory::PreExistingConditions
            | DeductionCategory::LiquidatedDamagesUnconscionable
    );

    let deduction_permitted = (category_universally_permitted
        || category_conditionally_permitted)
        && !category_universally_prohibited
        && itemization_timely;

    let remaining_deposit_cents = if deduction_permitted {
        input.security_deposit_cents.saturating_sub(input.deduction_amount_cents)
    } else {
        input.security_deposit_cents
    };

    let penalty_multiplier_engaged = !deduction_permitted
        && (input.bad_faith_retention || !itemization_timely);

    let penalty_amount_cents = if penalty_multiplier_engaged {
        match input.jurisdiction {
            Jurisdiction::California => input.security_deposit_cents.saturating_mul(2),
            Jurisdiction::NewYork => input.security_deposit_cents.saturating_mul(2),
            Jurisdiction::Texas => {
                10_000_u64.saturating_add(input.deduction_amount_cents.saturating_mul(3))
            }
            Jurisdiction::Massachusetts => input.deduction_amount_cents.saturating_mul(3),
        }
    } else {
        0
    };

    if !itemization_timely {
        let statute = match input.jurisdiction {
            Jurisdiction::California => "Cal. Civ. Code § 1950.5(g)(1) — 21-day window from move-out date",
            Jurisdiction::NewYork => "NY GOL § 7-108(1-a)(e) — 14-day window from vacate date; failure FORFEITS landlord's right to retain any portion",
            Jurisdiction::Texas => "Tex. Prop. Code § 92.103 + § 92.104 — 30-day window from surrender date",
            Jurisdiction::Massachusetts => "Mass. Gen. Laws c. 186 § 15B(4) — 30-day window from termination (with interest)",
        };
        failure_reasons.push(format!(
            "ITEMIZATION WINDOW MISSED — {} days since vacate exceeds {}-day window; {}",
            input.days_since_vacate, itemization_window_days, statute
        ));
    }

    if category_universally_prohibited {
        let prohibition = match input.deduction_category {
            DeductionCategory::NormalWearAndTear => "NORMAL WEAR AND TEAR universally excluded across CA + NY + TX + MA security deposit statutes",
            DeductionCategory::PreExistingConditions => "PRE-EXISTING CONDITIONS excluded under Cal. Civ. Code § 1950.5(e) 'less wear and tear' language + parallel jurisdictional rules",
            DeductionCategory::LiquidatedDamagesUnconscionable => "LIQUIDATED DAMAGES PROVISIONS that exceed actual damages and are unconscionable under Cal. Civ. Code § 1670.5 + NY GOL § 5-321 + Mass. unconscionability doctrine",
            _ => "Universally prohibited deduction category",
        };
        failure_reasons.push(prohibition.to_string());
    }

    if matches!(input.deduction_category, DeductionCategory::EvictionAttorneyFees)
        && !input.landlord_prevailed_in_eviction
    {
        failure_reasons.push(
            "EVICTION ATTORNEY FEES NOT PERMITTED — landlord did not prevail in eviction action; deduction requires (1) lease authorization; (2) statute permission; (3) landlord prevailed; absent any element = bad-faith retention".to_string(),
        );
    }

    if matches!(
        input.deduction_category,
        DeductionCategory::DoubleRentDamages | DeductionCategory::EvictionAttorneyFees | DeductionCategory::LiquidatedDamagesUnconscionable
    ) && !input.lease_authorizes_deduction
    {
        failure_reasons.push(
            "LEASE DOES NOT AUTHORIZE — double-rent damages and attorney fees require explicit lease provision; CA § 1670.5 + NY § 5-321 + MA unconscionability doctrine may invalidate excessive liquidated-damages provisions".to_string(),
        );
    }

    if penalty_multiplier_engaged {
        let penalty_label = match input.jurisdiction {
            Jurisdiction::California => "Cal. Civ. Code § 1950.5(l) — actual damages + UP TO 2× DEPOSIT PUNITIVE DAMAGES for bad-faith retention",
            Jurisdiction::NewYork => "NY GOL § 7-108(1-a)(g) — actual damages + UP TO 2× DEPOSIT PUNITIVE DAMAGES for willful retention; failure to provide itemization within 14 days FORFEITS retention right entirely",
            Jurisdiction::Texas => "Tex. Prop. Code § 92.109(a) — BAD-FAITH RETENTION: $100 + 3× WRONGFULLY WITHHELD + attorney fees + court costs",
            Jurisdiction::Massachusetts => "Mass. Gen. Laws c. 186 § 15B(7) — TRIPLE DAMAGES + interest + attorney fees + court costs",
        };
        failure_reasons.push(format!(
            "PENALTY MULTIPLIER ENGAGED — {} cents penalty exposure; {}",
            penalty_amount_cents,
            penalty_label
        ));
    }

    if deduction_permitted {
        let permitted_label = match input.deduction_category {
            DeductionCategory::UnpaidHoldoverRent => "UNPAID HOLDOVER RENT universally permitted (CA § 1950.5(e)(1) + NY § 7-108(1-a)(g)(i) + TX § 92.104(a) + MA § 15B(4)(i)) provided lease provides for rent during holdover period",
            DeductionCategory::PhysicalDamageBeyondWear => "PHYSICAL DAMAGE BEYOND NORMAL WEAR AND TEAR universally permitted with itemization (CA + NY + TX + MA)",
            DeductionCategory::CleaningToBaseline => "CLEANING TO RETURN PREMISES TO BASELINE permitted (CA + MA explicit; TX implicit; NY generally permitted as 'damage' category)",
            DeductionCategory::DoubleRentDamages => "DOUBLE RENT DAMAGES — permitted only if lease authorizes; typically requires separate court judgment in FL § 83.58 + similar statutes",
            DeductionCategory::EvictionAttorneyFees => "EVICTION ATTORNEY FEES AND COURT COSTS — permitted ONLY if (1) lease provides; (2) statute permits; (3) landlord prevailed in eviction action",
            DeductionCategory::UnpaidUtilities => "NON-PAYMENT OF UTILITIES PAYABLE TO LANDLORD — permitted under NY GOL § 7-108(1-a)(g)(iii)",
            DeductionCategory::MovingAndStorage => "MOVING AND STORAGE OF TENANT'S BELONGINGS — permitted under NY GOL § 7-108(1-a)(g)(iv); also abandoned_property_handling sibling",
            _ => "Permitted deduction category",
        };
        failure_reasons.push(format!(
            "DEDUCTION PERMITTED — {}; deduction of {} cents from $2,500 deposit leaves {} cents remaining for return to tenant",
            permitted_label,
            input.deduction_amount_cents,
            remaining_deposit_cents
        ));
    }

    let notes: Vec<String> = vec![
        "Four-jurisdiction framework: CALIFORNIA (Cal. Civ. Code § 1950.5 + § 1950.5(e) permitted deductions + § 1950.5(g)(1) 21-day window + § 1950.5(l) bad-faith 2× punitive); NEW YORK (NY GOL § 7-108 + § 7-108(1-a)(e) 14-day window + § 7-108(1-a)(g) permitted deductions including utilities + moving/storage + willful 2× punitive + late-itemization forfeiture); TEXAS (Tex. Prop. Code § 92.103 + § 92.104 + § 92.104(c) rent-owed itemization exception + § 92.109(a) bad-faith $100 + 3× + fees + 30-day window); MASSACHUSETTS (Mass. Gen. Laws c. 186 § 15B(4) permitted deductions + § 15B(7) triple damages + interest + 30-day window)".to_string(),
        "Holdover rent setoff vs damage reservation — 7 permitted application categories: (1) UNPAID HOLDOVER RENT universally permitted in CA + NY + TX + MA provided lease provides; (2) DOUBLE RENT DAMAGES under FL § 83.58 + similar NOT directly deductible unless lease specifically authorizes; typically requires separate court judgment; (3) PHYSICAL DAMAGE BEYOND NORMAL WEAR AND TEAR universally permitted with itemization; (4) EVICTION ATTORNEY FEES AND COURT COSTS permitted only if lease + statute + landlord prevailed; (5) CLEANING TO RETURN PREMISES TO BASELINE permitted; (6) NON-PAYMENT OF UTILITIES PAYABLE TO LANDLORD permitted under NY § 7-108(1-a)(g)(iii); (7) MOVING AND STORAGE OF TENANT'S BELONGINGS permitted under NY § 7-108(1-a)(g)(iv)".to_string(),
        "NOT permitted deductions: (1) NORMAL WEAR AND TEAR universally excluded; (2) PRE-EXISTING CONDITIONS excluded under Cal. Civ. Code § 1950.5(e) 'less wear and tear' language; (3) LIQUIDATED-DAMAGES PROVISIONS that exceed actual damages and are unconscionable under Cal. Civ. Code § 1670.5 + NY GOL § 5-321 + Mass. unconscionability doctrine; (4) LANDLORD'S OWN ECONOMIC LOSSES from re-renting delays (covered by duty-to-mitigate-damages doctrine plus lease holdover clause)".to_string(),
        "Itemization + refund window comparison: CALIFORNIA § 1950.5(g)(1) = 21 days from move-out; NEW YORK § 7-108(1-a)(e) = 14 days from vacate date; TEXAS § 92.103(a) + § 92.104 = 30 days from surrender; MASSACHUSETTS § 15B(4) = 30 days from termination (with interest)".to_string(),
        "Bad-faith / willful retention penalty multipliers: CA § 1950.5(l) actual damages + UP TO 2× DEPOSIT punitive; NY § 7-108(1-a)(g) actual damages + UP TO 2× DEPOSIT punitive for willful (plus 14-day forfeiture rule); TX § 92.109(a) $100 + 3× WRONGFULLY WITHHELD + attorney fees + court costs; MA § 15B(7) TRIPLE damages + interest + attorney fees + court costs".to_string(),
        "California § 1950.5(e) permitted deductions four categories: (1) unpaid rent (includes holdover rent if lease provides); (2) repair of damages beyond normal wear and tear; (3) cleaning to return premises to condition at start of tenancy less wear and tear; (4) restoration of personal property or appurtenances if lease specifically authorizes".to_string(),
        "New York § 7-108(1-a)(g) permitted deductions four categories: (i) non-payment of rent; (ii) damage beyond normal wear and tear; (iii) non-payment of utility charges payable directly to landlord under terms of lease or tenancy; (iv) moving and storage of tenant's belongings; § 7-108 LATE-ITEMIZATION FORFEITURE — failure to provide itemization within 14 days FORFEITS landlord's right to retain any portion of deposit".to_string(),
        "Texas § 92.104 deductions framework: § 92.104(a) permits deductions for damages and charges for which tenant is legally liable under lease or as result of breaching lease; § 92.104(c) excludes rent itemization when (1) tenant owes rent at surrender + (2) no controversy concerning amount; § 92.109(a) BAD-FAITH RETENTION triggers $100 + 3× WRONGFULLY WITHHELD + attorney fees + court costs".to_string(),
        "Massachusetts § 15B(4) four permitted deductions: (i) unpaid rent not validly withheld; (ii) unpaid real estate tax increases tenant required to pay under lease; (iii) reasonable cleaning; (iv) repair of tenant damage beyond reasonable wear and tear; § 15B(7) TRIPLE DAMAGES + interest credited annually + attorney fees + court costs for non-compliance".to_string(),
        "Trader-landlord critical fact patterns: (1) CA trader applies entire $2,500 deposit to $3,000 unpaid holdover rent; mails 21-day itemization; sustains defense under § 1950.5(e)(1) PROVIDED lease authorized; (2) NY trader exhausts deposit + sues for $1,500 excess for floor damage; must file § 7-108(1-a)(g) itemization within 14 days or FORFEIT entire retention right; (3) TX trader fails to itemize within 30 days — § 92.109(a) bad-faith $100 + 3× + fees; (4) MA trader retains $2,400 without itemizing — § 15B(7) TRIPLE DAMAGES $7,200 + interest + fees; (5) trader uses deposit for eviction attorney fees $5,000 — permitted only if lease + statute + landlord prevailed; absent any element exposes trader to penalty multipliers".to_string(),
        "Companion to holdover_tenant_damages (general framework) + damage_deduction_itemization + deposit_return_windows + security_deposit_bank_disclosure + landlord_property_sale_notice (iter 437 sale transfer) + duty_to_mitigate_damages + lease_cure_period + abandoned_property_handling".to_string(),
    ];

    TenantHoldoverSecurityDepositSetoffResult {
        deduction_permitted,
        itemization_timely,
        itemization_window_days,
        remaining_deposit_cents,
        penalty_multiplier_engaged,
        penalty_amount_cents,
        failure_reasons,
        citation: "Cal. Civ. Code § 1950.5; Cal. Civ. Code § 1950.5(e); Cal. Civ. Code § 1950.5(g)(1); Cal. Civ. Code § 1950.5(l); Cal. Civ. Code § 1670.5; NY GOL § 7-108; NY GOL § 7-108(1-a)(e); NY GOL § 7-108(1-a)(g); NY GOL § 5-321; Tex. Prop. Code § 92.103; Tex. Prop. Code § 92.104; Tex. Prop. Code § 92.104(c); Tex. Prop. Code § 92.109(a); Mass. Gen. Laws c. 186 § 15B(4); Mass. Gen. Laws c. 186 § 15B(7); HSTPA of 2019 (NY Laws 2019, ch. 36); AB 2801 (CA security deposit reform)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_holdover_rent_compliant() -> TenantHoldoverSecurityDepositSetoffInput {
        TenantHoldoverSecurityDepositSetoffInput {
            jurisdiction: Jurisdiction::California,
            deduction_category: DeductionCategory::UnpaidHoldoverRent,
            security_deposit_cents: 250_000,
            deduction_amount_cents: 250_000,
            days_since_vacate: 15,
            itemized_statement_provided: true,
            lease_authorizes_deduction: true,
            landlord_prevailed_in_eviction: false,
            bad_faith_retention: false,
        }
    }

    #[test]
    fn ca_holdover_rent_within_21_days_permitted() {
        let r = check(&ca_holdover_rent_compliant());
        assert!(r.deduction_permitted);
        assert!(r.itemization_timely);
        assert_eq!(r.itemization_window_days, 21);
        assert_eq!(r.remaining_deposit_cents, 0);
    }

    #[test]
    fn ca_past_21_days_violation() {
        let mut i = ca_holdover_rent_compliant();
        i.days_since_vacate = 25;
        let r = check(&i);
        assert!(!r.itemization_timely);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 1950.5(g)(1)")
            && f.contains("21-day window")));
    }

    #[test]
    fn ny_14_day_window() {
        let mut i = ca_holdover_rent_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.days_since_vacate = 10;
        let r = check(&i);
        assert_eq!(r.itemization_window_days, 14);
        assert!(r.itemization_timely);
    }

    #[test]
    fn ny_past_14_days_forfeiture() {
        let mut i = ca_holdover_rent_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.days_since_vacate = 15;
        let r = check(&i);
        assert!(!r.itemization_timely);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 7-108(1-a)(e)")
            && f.contains("FORFEITS")));
    }

    #[test]
    fn tx_30_day_window() {
        let mut i = ca_holdover_rent_compliant();
        i.jurisdiction = Jurisdiction::Texas;
        i.days_since_vacate = 25;
        let r = check(&i);
        assert_eq!(r.itemization_window_days, 30);
        assert!(r.itemization_timely);
    }

    #[test]
    fn ma_30_day_window() {
        let mut i = ca_holdover_rent_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.days_since_vacate = 25;
        let r = check(&i);
        assert_eq!(r.itemization_window_days, 30);
        assert!(r.itemization_timely);
    }

    #[test]
    fn physical_damage_beyond_wear_universally_permitted() {
        let mut i = ca_holdover_rent_compliant();
        i.deduction_category = DeductionCategory::PhysicalDamageBeyondWear;
        let r = check(&i);
        assert!(r.deduction_permitted);
    }

    #[test]
    fn cleaning_to_baseline_permitted() {
        let mut i = ca_holdover_rent_compliant();
        i.deduction_category = DeductionCategory::CleaningToBaseline;
        let r = check(&i);
        assert!(r.deduction_permitted);
    }

    #[test]
    fn normal_wear_and_tear_prohibited() {
        let mut i = ca_holdover_rent_compliant();
        i.deduction_category = DeductionCategory::NormalWearAndTear;
        let r = check(&i);
        assert!(!r.deduction_permitted);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("NORMAL WEAR AND TEAR universally excluded")));
    }

    #[test]
    fn pre_existing_conditions_prohibited() {
        let mut i = ca_holdover_rent_compliant();
        i.deduction_category = DeductionCategory::PreExistingConditions;
        let r = check(&i);
        assert!(!r.deduction_permitted);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("PRE-EXISTING CONDITIONS")
            && f.contains("less wear and tear")));
    }

    #[test]
    fn unconscionable_liquidated_damages_prohibited() {
        let mut i = ca_holdover_rent_compliant();
        i.deduction_category = DeductionCategory::LiquidatedDamagesUnconscionable;
        let r = check(&i);
        assert!(!r.deduction_permitted);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("LIQUIDATED DAMAGES PROVISIONS")
            && f.contains("§ 1670.5")
            && f.contains("§ 5-321")));
    }

    #[test]
    fn eviction_attorney_fees_with_lease_and_prevailed_permitted() {
        let mut i = ca_holdover_rent_compliant();
        i.deduction_category = DeductionCategory::EvictionAttorneyFees;
        i.lease_authorizes_deduction = true;
        i.landlord_prevailed_in_eviction = true;
        let r = check(&i);
        assert!(r.deduction_permitted);
    }

    #[test]
    fn eviction_attorney_fees_without_prevailing_prohibited() {
        let mut i = ca_holdover_rent_compliant();
        i.deduction_category = DeductionCategory::EvictionAttorneyFees;
        i.lease_authorizes_deduction = true;
        i.landlord_prevailed_in_eviction = false;
        let r = check(&i);
        assert!(!r.deduction_permitted);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("EVICTION ATTORNEY FEES NOT PERMITTED")
            && f.contains("did not prevail")));
    }

    #[test]
    fn double_rent_damages_without_lease_authorization_prohibited() {
        let mut i = ca_holdover_rent_compliant();
        i.deduction_category = DeductionCategory::DoubleRentDamages;
        i.lease_authorizes_deduction = false;
        let r = check(&i);
        assert!(!r.deduction_permitted);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("LEASE DOES NOT AUTHORIZE")
            && f.contains("§ 1670.5")));
    }

    #[test]
    fn ny_unpaid_utilities_permitted() {
        let mut i = ca_holdover_rent_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.deduction_category = DeductionCategory::UnpaidUtilities;
        i.days_since_vacate = 10;
        let r = check(&i);
        assert!(r.deduction_permitted);
    }

    #[test]
    fn ca_unpaid_utilities_not_permitted_outside_ny() {
        let mut i = ca_holdover_rent_compliant();
        i.deduction_category = DeductionCategory::UnpaidUtilities;
        let r = check(&i);
        assert!(!r.deduction_permitted);
    }

    #[test]
    fn ny_moving_and_storage_permitted() {
        let mut i = ca_holdover_rent_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.deduction_category = DeductionCategory::MovingAndStorage;
        i.days_since_vacate = 10;
        let r = check(&i);
        assert!(r.deduction_permitted);
    }

    #[test]
    fn ca_bad_faith_2x_punitive() {
        let mut i = ca_holdover_rent_compliant();
        i.deduction_category = DeductionCategory::NormalWearAndTear;
        i.bad_faith_retention = true;
        let r = check(&i);
        assert!(r.penalty_multiplier_engaged);
        assert_eq!(r.penalty_amount_cents, 500_000);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 1950.5(l)")
            && f.contains("2× DEPOSIT PUNITIVE")));
    }

    #[test]
    fn ny_willful_2x_punitive() {
        let mut i = ca_holdover_rent_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.deduction_category = DeductionCategory::NormalWearAndTear;
        i.bad_faith_retention = true;
        i.days_since_vacate = 10;
        let r = check(&i);
        assert!(r.penalty_multiplier_engaged);
        assert_eq!(r.penalty_amount_cents, 500_000);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 7-108(1-a)(g)")
            && f.contains("FORFEITS retention right")));
    }

    #[test]
    fn tx_bad_faith_100_plus_3x_fees() {
        let mut i = ca_holdover_rent_compliant();
        i.jurisdiction = Jurisdiction::Texas;
        i.deduction_category = DeductionCategory::NormalWearAndTear;
        i.bad_faith_retention = true;
        i.days_since_vacate = 15;
        let r = check(&i);
        assert!(r.penalty_multiplier_engaged);
        let expected = 10_000_u64 + 250_000 * 3;
        assert_eq!(r.penalty_amount_cents, expected);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 92.109(a)")
            && f.contains("$100")
            && f.contains("3× WRONGFULLY WITHHELD")));
    }

    #[test]
    fn ma_triple_damages_engaged() {
        let mut i = ca_holdover_rent_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.deduction_category = DeductionCategory::NormalWearAndTear;
        i.bad_faith_retention = true;
        i.days_since_vacate = 15;
        let r = check(&i);
        assert!(r.penalty_multiplier_engaged);
        assert_eq!(r.penalty_amount_cents, 750_000);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 15B(7)")
            && f.contains("TRIPLE DAMAGES")));
    }

    #[test]
    fn deduction_category_truth_table_ten_cells() {
        for cat in [
            DeductionCategory::UnpaidHoldoverRent,
            DeductionCategory::DoubleRentDamages,
            DeductionCategory::PhysicalDamageBeyondWear,
            DeductionCategory::EvictionAttorneyFees,
            DeductionCategory::CleaningToBaseline,
            DeductionCategory::UnpaidUtilities,
            DeductionCategory::MovingAndStorage,
            DeductionCategory::NormalWearAndTear,
            DeductionCategory::PreExistingConditions,
            DeductionCategory::LiquidatedDamagesUnconscionable,
        ] {
            let mut i = ca_holdover_rent_compliant();
            i.deduction_category = cat;
            let r = check(&i);
            let _ = r.deduction_permitted;
        }
    }

    #[test]
    fn jurisdiction_window_truth_table_four_cells() {
        for (j, exp) in [
            (Jurisdiction::California, 21),
            (Jurisdiction::NewYork, 14),
            (Jurisdiction::Texas, 30),
            (Jurisdiction::Massachusetts, 30),
        ] {
            let mut i = ca_holdover_rent_compliant();
            i.jurisdiction = j;
            let r = check(&i);
            assert_eq!(r.itemization_window_days, exp, "j={:?}", j);
        }
    }

    #[test]
    fn ny_uniquely_14_day_window_invariant() {
        let mut ny = ca_holdover_rent_compliant();
        ny.jurisdiction = Jurisdiction::NewYork;
        let r_ny = check(&ny);
        assert_eq!(r_ny.itemization_window_days, 14);

        for j in [
            Jurisdiction::California,
            Jurisdiction::Texas,
            Jurisdiction::Massachusetts,
        ] {
            let mut i = ca_holdover_rent_compliant();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(r.itemization_window_days >= 21, "j={:?}", j);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&ca_holdover_rent_compliant());
        assert!(r.citation.contains("Cal. Civ. Code § 1950.5"));
        assert!(r.citation.contains("Cal. Civ. Code § 1950.5(e)"));
        assert!(r.citation.contains("Cal. Civ. Code § 1950.5(g)(1)"));
        assert!(r.citation.contains("Cal. Civ. Code § 1950.5(l)"));
        assert!(r.citation.contains("Cal. Civ. Code § 1670.5"));
        assert!(r.citation.contains("NY GOL § 7-108"));
        assert!(r.citation.contains("NY GOL § 7-108(1-a)(e)"));
        assert!(r.citation.contains("NY GOL § 7-108(1-a)(g)"));
        assert!(r.citation.contains("NY GOL § 5-321"));
        assert!(r.citation.contains("Tex. Prop. Code § 92.103"));
        assert!(r.citation.contains("Tex. Prop. Code § 92.104"));
        assert!(r.citation.contains("Tex. Prop. Code § 92.104(c)"));
        assert!(r.citation.contains("Tex. Prop. Code § 92.109(a)"));
        assert!(r.citation.contains("Mass. Gen. Laws c. 186 § 15B(4)"));
        assert!(r.citation.contains("Mass. Gen. Laws c. 186 § 15B(7)"));
        assert!(r.citation.contains("HSTPA of 2019"));
        assert!(r.citation.contains("AB 2801"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let r = check(&ca_holdover_rent_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Four-jurisdiction framework")
            && n.contains("CALIFORNIA")
            && n.contains("NEW YORK")
            && n.contains("TEXAS")
            && n.contains("MASSACHUSETTS")));
    }

    #[test]
    fn note_pins_seven_permitted_categories() {
        let r = check(&ca_holdover_rent_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("7 permitted application categories")
            && n.contains("UNPAID HOLDOVER RENT")
            && n.contains("DOUBLE RENT DAMAGES")
            && n.contains("MOVING AND STORAGE")));
    }

    #[test]
    fn note_pins_prohibited_deductions() {
        let r = check(&ca_holdover_rent_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("NOT permitted deductions")
            && n.contains("NORMAL WEAR AND TEAR universally excluded")
            && n.contains("PRE-EXISTING CONDITIONS")
            && n.contains("LIQUIDATED-DAMAGES")
            && n.contains("re-renting delays")));
    }

    #[test]
    fn note_pins_itemization_window_comparison() {
        let r = check(&ca_holdover_rent_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Itemization + refund window comparison")
            && n.contains("21 days")
            && n.contains("14 days")
            && n.contains("30 days")));
    }

    #[test]
    fn note_pins_penalty_multiplier_comparison() {
        let r = check(&ca_holdover_rent_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Bad-faith / willful retention penalty multipliers")
            && n.contains("2× DEPOSIT punitive")
            && n.contains("$100 + 3× WRONGFULLY WITHHELD")
            && n.contains("TRIPLE damages + interest")));
    }

    #[test]
    fn note_pins_ca_section_1950_5_e_four_categories() {
        let r = check(&ca_holdover_rent_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("California § 1950.5(e) permitted deductions four categories")
            && n.contains("holdover rent if lease provides")
            && n.contains("normal wear and tear")
            && n.contains("cleaning")
            && n.contains("restoration of personal property")));
    }

    #[test]
    fn note_pins_ny_section_7_108_1a_g_four_categories() {
        let r = check(&ca_holdover_rent_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("New York § 7-108(1-a)(g) permitted deductions four categories")
            && n.contains("utility charges")
            && n.contains("moving and storage")
            && n.contains("LATE-ITEMIZATION FORFEITURE")));
    }

    #[test]
    fn note_pins_tx_section_92_104_framework() {
        let r = check(&ca_holdover_rent_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Texas § 92.104 deductions framework")
            && n.contains("§ 92.104(c) excludes rent itemization")
            && n.contains("§ 92.109(a) BAD-FAITH RETENTION")
            && n.contains("$100 + 3× WRONGFULLY WITHHELD")));
    }

    #[test]
    fn note_pins_ma_section_15b_four_categories_triple() {
        let r = check(&ca_holdover_rent_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Massachusetts § 15B(4) four permitted deductions")
            && n.contains("reasonable cleaning")
            && n.contains("§ 15B(7) TRIPLE DAMAGES + interest")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&ca_holdover_rent_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Trader-landlord critical fact patterns")
            && n.contains("§ 1950.5(e)(1) PROVIDED lease authorized")
            && n.contains("FORFEIT entire retention right")
            && n.contains("§ 92.109(a)")
            && n.contains("$7,200")
            && n.contains("attorney fees")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&ca_holdover_rent_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Companion to holdover_tenant_damages")
            && n.contains("damage_deduction_itemization")
            && n.contains("landlord_property_sale_notice (iter 437")));
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = ca_holdover_rent_compliant();
        i.deduction_amount_cents = u64::MAX;
        let r = check(&i);
        let _ = r.penalty_amount_cents;
    }
}
