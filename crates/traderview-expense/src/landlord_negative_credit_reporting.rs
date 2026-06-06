//! Landlord negative credit reporting framework —
//! when may a landlord report unpaid rent, collection
//! accounts, or eviction history to consumer reporting
//! agencies (Equifax/Experian/TransUnion + tenant-
//! screening CRAs)? Distinct from sibling
//! `rent_credit_reporting` (POSITIVE rent reporting
//! under Cal. Civ. Code § 1954.06 / AB 2747) which
//! addresses the LANDLORD'S OFFER to report positive
//! payment history. This module addresses NEGATIVE
//! reporting of delinquent rent / collection accounts.
//!
//! Federal floor: FCRA § 1681s-2 furnisher
//! requirements + FDCPA § 1692 collection rules + 7-
//! year statute of limitations on negative reporting
//! under FCRA § 1681c. Tenant remedies: § 1681n
//! willful (actual + punitive + attorney's fees) and
//! § 1681o negligent (actual + attorney's fees) civil
//! liability against furnisher.
//!
//! Trader-landlord critical because furnishing
//! inaccurate or unverifiable negative information to
//! a CRA exposes landlord to STATUTORY DAMAGES of
//! $100-$1,000 per violation plus actual and
//! attorney's fees plus PUNITIVE damages for willful
//! violations. Furnisher's investigation duty under
//! § 1681s-2(b) is MANDATORY AND NON-DISCRETIONARY
//! once CRA transmits dispute notice.
//!
//! Companion to rent_credit_reporting (positive
//! reporting), tenant_data_privacy (broader privacy
//! framework), adverse_action_notice (post-denial
//! notice), credit_check_authorization (pre-screening
//! disclosure), application_fees (screening fee cap),
//! tenant_rent_judgment_wage_garnishment (post-
//! judgment enforcement).
//!
//! **FCRA § 1681s-2(a) — accuracy and integrity of
//! information**:
//! 1. Furnisher CANNOT report information that
//!    furnisher knows or has reasonable cause to
//!    believe is INACCURATE;
//! 2. If furnisher determines information previously
//!    reported is not complete or accurate, MUST
//!    promptly provide complete and accurate
//!    information AND notify all CRAs that received
//!    the information of corrections;
//! 3. If reporting delinquent account placed for
//!    collection, must provide CRA with MONTH AND
//!    YEAR of commencement of delinquency within
//!    90 days after reporting (governs 7-year
//!    aging under § 1681c).
//!
//! **FCRA § 1681s-2(b) — INVESTIGATION DUTY upon
//! CRA dispute notice (mandatory, non-discretionary)**:
//! 1. § 1681s-2(b)(1)(A) — CONDUCT investigation
//!    with respect to disputed information;
//! 2. § 1681s-2(b)(1)(B) — REVIEW all relevant
//!    information provided by CRA;
//! 3. § 1681s-2(b)(1)(C) — REPORT results of
//!    investigation to CRA;
//! 4. § 1681s-2(b)(1)(D) — if found inaccurate, also
//!    REPORT corrections to other CRAs that received
//!    the information;
//! 5. § 1681s-2(b)(1)(E) — if information NOT
//!    VERIFIABLE, MAY NOT remain in consumer
//!    reporting system (categorical command per 4th
//!    Circuit jurisprudence).
//!
//! **FCRA § 1681c — 7-YEAR STATUTE OF LIMITATIONS
//! on negative information**:
//! 1. § 1681c(a)(4) — most negative items including
//!    accounts placed for collection or charged to
//!    profit and loss reportable for ONLY 7 YEARS
//!    from date of delinquency;
//! 2. § 1681c(a)(1) — bankruptcies under Chapter 7
//!    reportable for 10 YEARS;
//! 3. § 1681c(c)(1) — RUNNING OF THE 7-YEAR PERIOD
//!    begins on DATE OF DELINQUENCY (NOT date of
//!    collection placement or judgment);
//! 4. UNPAID RENT in CRA records typically governed
//!    by § 1681c(a)(4) 7-year limitation from date
//!    of original delinquency.
//!
//! **FCRA § 1681n WILLFUL NONCOMPLIANCE — CIVIL
//! LIABILITY**:
//! 1. Statutory damages $100-$1,000 per violation;
//!    OR actual damages, whichever is greater;
//! 2. PUNITIVE damages as court may allow;
//! 3. Reasonable attorney's fees + costs;
//! 4. Willful = knowing or reckless disregard of
//!    statutory requirements (Safeco Ins. Co. v.
//!    Burr, 551 U.S. 47 (2007)).
//!
//! **FCRA § 1681o NEGLIGENT NONCOMPLIANCE — CIVIL
//! LIABILITY**:
//! 1. Actual damages from negligent violation;
//! 2. Reasonable attorney's fees + costs;
//! 3. NO statutory damages, NO punitive damages.
//!
//! **FDCPA § 1692 INTERACTION** — when landlord
//! places debt with third-party collector or sells
//! to debt buyer:
//! 1. FDCPA § 1692e prohibits false or misleading
//!    representations including DISPUTED DEBT not
//!    flagged as disputed in CRA report;
//! 2. FDCPA § 1692f prohibits unfair or
//!    unconscionable collection means;
//: 3. FDCPA § 1692g(a) requires 5-day VALIDATION
//!    NOTICE with debt amount, original creditor,
//!    dispute rights;
//! 4. FDCPA § 1692g(b) — if tenant DISPUTES IN
//!    WRITING within 30 days, collector must
//!    VERIFY before continuing collection or
//!    reporting;
//! 5. CFPB Regulation F (12 CFR § 1006) requires
//!    debt collectors to provide ITEMIZATION DATE
//!    and validation information.
//!
//! **TENANT-SCREENING CRA SUBSET** — specialized
//! CRAs serving rental industry (CoreLogic SafeRent,
//! TransUnion SmartMove, RentBureau by Experian,
//! LexisNexis):
//! 1. Subject to all FCRA § 1681 requirements;
//! 2. § 1681e(b) accuracy requirement;
//! 3. § 1681i investigation of disputes (within 30
//!    days; 45 days if consumer provides additional
//!    information);
//! 4. § 1681m adverse action notice when landlord
//!    denies/raises rent/imposes condition based on
//!    CRA report;
//! 5. EVICTION RECORDS — controversial; many
//!    jurisdictions now seal or expunge eviction
//!    records under "Just Cause" laws making CRA
//!    reporting impermissible.
//!
//! **STATE LAW OVERLAY**:
//! 1. NEW YORK — NY GBL § 380 New York Fair Credit
//!    Reporting Act mirrors FCRA; § 380-d
//!    investigation requirement parallel to
//!    § 1681s-2(b); NY HSTPA 2019 created TENANT
//!    BLACKLIST DATABASE prohibition (CIVIL PENALTY
//!    $500-$1,000 per violation under NY RPL
//!    § 227-f);
//! 2. CALIFORNIA — Cal. Civ. Code § 1785 California
//!    Consumer Credit Reporting Agencies Act mirrors
//!    FCRA; Cal. Civ. Code § 1786 Investigative
//:    Consumer Reporting Agencies Act covers tenant-
//!    screening CRAs;
//! 3. OREGON SB 970 + WASHINGTON RCW 59.18.367
//!    eviction record sealing laws prohibit CRA
//!    reporting of sealed evictions;
//! 4. CONNECTICUT — Conn. Gen. Stat. § 47a-71
//!    prohibits landlord from reporting tenant to
//!    CRA without 30-day pre-reporting notice.
//!
//! **Trader-landlord critical fact patterns**:
//!
//! Trader-landlord reports $5,000 unpaid rent to
//! Experian; tenant disputes within 30 days; trader
//! fails to investigate within 30 days; FCRA
//! § 1681s-2(b) MANDATORY investigation violated;
//! statutory damages $100-$1,000 + actual damages +
//! attorney's fees.
//!
//! Trader-landlord reports tenant for unpaid rent 8
//! years after original delinquency; FCRA § 1681c(a)
//! (4) 7-YEAR LIMIT EXCEEDED; CRA must remove;
//! continued reporting = willful FCRA violation +
//! § 1681n punitive damages.
//!
//! NY trader-landlord reports tenant to specialized
//! "tenant blacklist" database after eviction
//! proceeding; NY HSTPA 2019 + NY RPL § 227-f
//! TENANT BLACKLIST PROHIBITION; $500-$1,000 civil
//! penalty per violation + tenant remedies.
//!
//! Trader-landlord places $3,000 unpaid rent debt
//! with collection agency that reports to CRA
//! without § 1692g(a) 5-day validation notice;
//! FDCPA violation; tenant remedies under § 1692k
//! statutory damages up to $1,000 + actual + fees.
//!
//! CA trader-landlord engages tenant-screening CRA
//! that reports SEALED EVICTION record; Cal. Civ.
//! Code § 1786 + FCRA § 1681e(b) accuracy violation;
//! tenant remedies against CRA AND furnishing
//! landlord.
//!
//! Citations: 15 USC § 1681s-2(a) (accuracy and
//! integrity); 15 USC § 1681s-2(b) (investigation
//! duty); 15 USC § 1681c (7-year limitation period);
//! 15 USC § 1681c(a)(4) (collection accounts); 15
//! USC § 1681n (willful noncompliance civil
//! liability); 15 USC § 1681o (negligent
//! noncompliance civil liability); 15 USC § 1681e(b)
//! (accuracy); 15 USC § 1681i (investigation of
//! disputes); 15 USC § 1681m (adverse action notice);
//! 15 USC § 1692e (false or misleading
//! representations); 15 USC § 1692f (unfair
//! collection); 15 USC § 1692g (validation notice);
//! 15 USC § 1692k (FDCPA civil liability); 12 CFR
//! § 1006 (CFPB Regulation F debt collection); NY
//! GBL § 380 (NY Fair Credit Reporting Act); NY
//! GBL § 380-d (investigation); NY RPL § 227-f
//! (tenant blacklist prohibition); Cal. Civ. Code
//! § 1785 (California Consumer Credit Reporting
//! Agencies Act); Cal. Civ. Code § 1786 (CA
//! Investigative Consumer Reporting Agencies Act);
//! Conn. Gen. Stat. § 47a-71 (30-day pre-reporting
//! notice); Oregon SB 970 (eviction record sealing);
//! Washington RCW 59.18.367 (eviction record
//! sealing); Safeco Ins. Co. v. Burr, 551 U.S. 47
//! (2007) (FCRA willfulness standard).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewYork,
    California,
    Connecticut,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LandlordNegativeCreditReportingInput {
    pub jurisdiction: Jurisdiction,
    /// Whether furnisher (landlord) is reporting
    /// information to a CRA.
    pub reporting_to_cra: bool,
    /// Whether reported information is inaccurate per
    /// FCRA § 1681s-2(a).
    pub information_inaccurate: bool,
    /// Whether tenant disputed the information through
    /// CRA per FCRA § 1681s-2(b) trigger.
    pub tenant_disputed_through_cra: bool,
    /// Whether furnisher conducted § 1681s-2(b)
    /// investigation within 30 days of CRA dispute
    /// notice.
    pub investigation_conducted_within_30_days: bool,
    /// Years since date of original delinquency
    /// (§ 1681c(a)(4) 7-year limit trigger).
    pub years_since_delinquency: u32,
    /// Whether debt placed with third-party collection
    /// agency (triggers FDCPA).
    pub debt_placed_with_collector: bool,
    /// Whether collector provided § 1692g(a) 5-day
    /// validation notice.
    pub fdcpa_validation_notice_provided: bool,
    /// Whether tenant disputed in writing within 30
    /// days of validation notice.
    pub tenant_disputed_in_writing_30_days: bool,
    /// Whether debt verified before continued
    /// collection (FDCPA § 1692g(b) requirement).
    pub debt_verified_after_dispute: bool,
    /// Whether NY landlord reporting to tenant
    /// blacklist database (NY RPL § 227-f trigger).
    pub ny_tenant_blacklist_reporting: bool,
    /// Whether CT landlord provided 30-day pre-
    /// reporting notice (Conn. Gen. Stat. § 47a-71).
    pub ct_30_day_pre_reporting_notice_given: bool,
    /// Whether eviction record reported was sealed/
    /// expunged under state law.
    pub sealed_eviction_record_reported: bool,
    /// Whether violation was willful for § 1681n
    /// punitive damages eligibility.
    pub willful_violation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LandlordNegativeCreditReportingResult {
    pub furnisher_accuracy_compliant: bool,
    pub investigation_duty_compliant: bool,
    pub seven_year_aging_compliant: bool,
    pub fdcpa_compliant: bool,
    pub ny_tenant_blacklist_violation: bool,
    pub ct_30_day_notice_compliant: bool,
    pub sealed_eviction_violation: bool,
    pub fcra_statutory_damages_min_cents: u64,
    pub fcra_statutory_damages_max_cents: u64,
    pub punitive_damages_available: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &LandlordNegativeCreditReportingInput,
) -> LandlordNegativeCreditReportingResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let furnisher_accuracy_compliant = !input.reporting_to_cra || !input.information_inaccurate;

    let investigation_duty_compliant =
        !input.tenant_disputed_through_cra || input.investigation_conducted_within_30_days;

    let seven_year_aging_compliant = !input.reporting_to_cra || input.years_since_delinquency <= 7;

    let fdcpa_compliant = !input.debt_placed_with_collector
        || (input.fdcpa_validation_notice_provided
            && (!input.tenant_disputed_in_writing_30_days || input.debt_verified_after_dispute));

    let ny_tenant_blacklist_violation =
        matches!(input.jurisdiction, Jurisdiction::NewYork) && input.ny_tenant_blacklist_reporting;

    let ct_30_day_notice_compliant = !matches!(input.jurisdiction, Jurisdiction::Connecticut)
        || input.ct_30_day_pre_reporting_notice_given;

    let sealed_eviction_violation = input.sealed_eviction_record_reported;

    let fcra_statutory_damages_min_cents: u64 = 10_000;
    let fcra_statutory_damages_max_cents: u64 = 100_000;

    let punitive_damages_available = input.willful_violation
        && (!furnisher_accuracy_compliant
            || !investigation_duty_compliant
            || !seven_year_aging_compliant
            || ny_tenant_blacklist_violation
            || sealed_eviction_violation);

    if !furnisher_accuracy_compliant {
        failure_reasons.push(
            "15 USC § 1681s-2(a) FURNISHER ACCURACY VIOLATION — landlord furnisher reported information to CRA that landlord knew or had reasonable cause to believe was INACCURATE; if previously reported, must promptly provide complete and accurate information AND notify all CRAs that received the information of corrections; tenant remedies under § 1681n willful or § 1681o negligent civil liability".to_string(),
        );
    }

    if !investigation_duty_compliant {
        failure_reasons.push(
            "15 USC § 1681s-2(b) INVESTIGATION DUTY VIOLATION — CRA transmitted dispute notice to furnisher; furnisher's obligations are MANDATORY AND NON-DISCRETIONARY (4th Circuit jurisprudence); must (1) CONDUCT investigation; (2) REVIEW all relevant information from CRA; (3) REPORT results to CRA within 30 days; (4) REPORT corrections to other CRAs that received the information; (5) DELETE if not verifiable per § 1681s-2(b)(1)(E)".to_string(),
        );
    }

    if !seven_year_aging_compliant {
        failure_reasons.push(format!(
            "15 USC § 1681c(a)(4) 7-YEAR LIMITATION EXCEEDED — {} years since date of original delinquency exceeds 7-year statutory cap on negative information reporting; § 1681c(c)(1) running begins on DATE OF DELINQUENCY (NOT date of collection placement or judgment); CRA must remove; continued reporting = willful FCRA violation + § 1681n punitive damages",
            input.years_since_delinquency
        ));
    }

    if !fdcpa_compliant {
        if !input.fdcpa_validation_notice_provided {
            failure_reasons.push(
                "15 USC § 1692g(a) FDCPA VALIDATION NOTICE VIOLATION — debt collector failed to provide 5-day validation notice with debt amount + name of original creditor + dispute rights; CFPB Regulation F (12 CFR § 1006) requires itemization date; tenant remedies under § 1692k statutory damages up to $1,000 + actual damages + attorney's fees + costs".to_string(),
            );
        }
        if input.tenant_disputed_in_writing_30_days && !input.debt_verified_after_dispute {
            failure_reasons.push(
                "15 USC § 1692g(b) FDCPA VERIFICATION VIOLATION — tenant disputed debt in writing within 30 days of validation notice; collector must VERIFY debt before continuing collection or reporting; failure violates FDCPA § 1692g(b)".to_string(),
            );
        }
    }

    if ny_tenant_blacklist_violation {
        failure_reasons.push(
            "NY HSTPA 2019 + NY RPL § 227-f TENANT BLACKLIST PROHIBITION — landlord prohibited from reporting tenant to specialized tenant blacklist database after eviction proceeding regardless of outcome; CIVIL PENALTY $500-$1,000 PER VIOLATION; tenant private cause of action with attorney's fees + injunctive relief".to_string(),
        );
    }

    if !ct_30_day_notice_compliant {
        failure_reasons.push(
            "Conn. Gen. Stat. § 47a-71 — Connecticut requires 30-DAY PRE-REPORTING NOTICE to tenant before landlord may report unpaid rent to CRA; tenant may dispute or pay during notice period; failure invalidates reporting".to_string(),
        );
    }

    if sealed_eviction_violation {
        failure_reasons.push(
            "SEALED EVICTION RECORD VIOLATION — Oregon SB 970 + Washington RCW 59.18.367 + similar Just Cause state eviction record sealing laws prohibit CRA reporting of sealed or expunged evictions; FCRA § 1681e(b) accuracy violation; tenant remedies against CRA AND furnishing landlord under § 1681n/§ 1681o".to_string(),
        );
    }

    if punitive_damages_available {
        failure_reasons.push(
            "15 USC § 1681n WILLFUL NONCOMPLIANCE — willful = knowing or reckless disregard of statutory requirements (Safeco Ins. Co. v. Burr, 551 U.S. 47 (2007)); statutory damages $100-$1,000 per violation OR actual damages whichever is greater; PUNITIVE damages as court may allow; reasonable attorney's fees + costs".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "Federal FCRA framework: § 1681s-2(a) accuracy and integrity of information + § 1681s-2(b) investigation duty upon CRA dispute notice (mandatory and non-discretionary, 4th Circuit jurisprudence) + § 1681c 7-year statute of limitations on negative reporting (date-of-delinquency trigger) + § 1681n willful civil liability (statutory $100-$1,000 + punitive + fees) + § 1681o negligent civil liability (actual + fees) + § 1681m adverse action notice + § 1681i CRA investigation of disputes (30/45 days)".to_string(),
        "FCRA § 1681s-2(a) furnisher accuracy: (1) cannot report inaccurate information; (2) must promptly correct and notify all CRAs; (3) collection accounts require month/year of delinquency commencement within 90 days (governs 7-year aging under § 1681c)".to_string(),
        "FCRA § 1681s-2(b) investigation duty 5-element framework: (1)(A) conduct investigation; (1)(B) review all relevant information from CRA; (1)(C) report results to CRA within 30 days; (1)(D) report corrections to other CRAs that received the information; (1)(E) DELETE information that is not verifiable (categorical command per 4th Circuit; cannot remain in consumer reporting system)".to_string(),
        "FCRA § 1681c 7-YEAR STATUTE OF LIMITATIONS on negative reporting: (1) § 1681c(a)(4) most negative items including accounts placed for collection or charged to profit and loss = 7 YEARS from date of delinquency; (2) § 1681c(a)(1) Chapter 7 bankruptcies = 10 YEARS; (3) § 1681c(c)(1) running begins on DATE OF DELINQUENCY not date of collection placement or judgment; (4) unpaid rent typically governed by § 1681c(a)(4) 7-year limitation".to_string(),
        "FCRA § 1681n willful noncompliance civil liability: statutory damages $100-$1,000 per violation OR actual damages whichever greater; PUNITIVE damages as court may allow; reasonable attorney's fees + costs; willful = knowing or reckless disregard of statutory requirements per Safeco Ins. Co. v. Burr, 551 U.S. 47 (2007)".to_string(),
        "FCRA § 1681o negligent noncompliance civil liability: actual damages from negligent violation; reasonable attorney's fees + costs; NO statutory damages NO punitive damages; lower standard of proof than § 1681n willful".to_string(),
        "FDCPA § 1692 interaction when landlord places debt with third-party collector: (1) § 1692e prohibits false or misleading representations including disputed debt not flagged as disputed in CRA report; (2) § 1692f prohibits unfair or unconscionable collection means; (3) § 1692g(a) requires 5-day VALIDATION NOTICE with debt amount + original creditor + dispute rights; (4) § 1692g(b) — tenant disputes in writing within 30 days collector must VERIFY before continuing collection or reporting; (5) CFPB Regulation F (12 CFR § 1006) requires itemization date; (6) § 1692k civil liability statutory damages up to $1,000 + actual + fees".to_string(),
        "Tenant-screening CRA subset (CoreLogic SafeRent + TransUnion SmartMove + RentBureau by Experian + LexisNexis): subject to all FCRA § 1681 requirements; § 1681e(b) accuracy; § 1681i investigation of disputes (30 days; 45 days if consumer provides additional information); § 1681m adverse action notice when landlord denies/raises rent/imposes condition based on CRA report; eviction records controversial — many jurisdictions seal or expunge under Just Cause laws making CRA reporting impermissible".to_string(),
        "State law overlay: (1) NEW YORK — NY GBL § 380 NY Fair Credit Reporting Act mirrors FCRA; § 380-d investigation requirement; HSTPA 2019 + NY RPL § 227-f TENANT BLACKLIST PROHIBITION with $500-$1,000 civil penalty per violation; (2) CALIFORNIA — Cal. Civ. Code § 1785 CA Consumer Credit Reporting Agencies Act + Cal. Civ. Code § 1786 Investigative Consumer Reporting Agencies Act covering tenant-screening CRAs; (3) Oregon SB 970 + Washington RCW 59.18.367 eviction record sealing prohibits CRA reporting of sealed evictions; (4) CONNECTICUT Conn. Gen. Stat. § 47a-71 requires 30-DAY PRE-REPORTING NOTICE before landlord may report to CRA".to_string(),
        "Trader-landlord critical fact patterns: (1) trader reports $5K unpaid rent + tenant disputes within 30 days + trader fails to investigate within 30 days — § 1681s-2(b) MANDATORY investigation violated + $100-$1,000 statutory damages + actual + fees; (2) trader reports 8 years after original delinquency — § 1681c(a)(4) 7-year limit exceeded + willful § 1681n punitive; (3) NY trader reports to tenant blacklist database — NY RPL § 227-f $500-$1,000 civil penalty; (4) collection agency without § 1692g(a) 5-day validation notice — FDCPA § 1692k $1K statutory + actual + fees; (5) CA trader engages CRA that reports SEALED EVICTION — § 1786 + § 1681e(b) accuracy violation against CRA AND furnishing landlord".to_string(),
        "Companion to rent_credit_reporting (positive reporting under Cal. Civ. Code § 1954.06 / AB 2747) + tenant_data_privacy (broader privacy framework) + adverse_action_notice (post-denial notice) + credit_check_authorization (pre-screening disclosure) + application_fees (screening fee cap) + tenant_rent_judgment_wage_garnishment (post-judgment enforcement)".to_string(),
    ];

    LandlordNegativeCreditReportingResult {
        furnisher_accuracy_compliant,
        investigation_duty_compliant,
        seven_year_aging_compliant,
        fdcpa_compliant,
        ny_tenant_blacklist_violation,
        ct_30_day_notice_compliant,
        sealed_eviction_violation,
        fcra_statutory_damages_min_cents,
        fcra_statutory_damages_max_cents,
        punitive_damages_available,
        failure_reasons,
        citation: "15 USC § 1681s-2(a) (accuracy and integrity); 15 USC § 1681s-2(b) (investigation duty); 15 USC § 1681c (7-year limitation); 15 USC § 1681c(a)(4) (collection accounts); 15 USC § 1681n (willful noncompliance civil liability); 15 USC § 1681o (negligent noncompliance civil liability); 15 USC § 1681e(b) (accuracy); 15 USC § 1681i (investigation of disputes); 15 USC § 1681m (adverse action notice); 15 USC § 1692e (false or misleading representations); 15 USC § 1692f (unfair collection); 15 USC § 1692g (validation notice); 15 USC § 1692k (FDCPA civil liability); 12 CFR § 1006 (CFPB Regulation F debt collection); NY GBL § 380; NY GBL § 380-d; NY RPL § 227-f (tenant blacklist prohibition); Cal. Civ. Code § 1785 (California Consumer Credit Reporting Agencies Act); Cal. Civ. Code § 1786 (Investigative Consumer Reporting Agencies Act); Conn. Gen. Stat. § 47a-71 (30-day pre-reporting notice); Oregon SB 970 (eviction record sealing); Washington RCW 59.18.367 (eviction record sealing); Safeco Ins. Co. v. Burr, 551 U.S. 47 (2007)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_compliant() -> LandlordNegativeCreditReportingInput {
        LandlordNegativeCreditReportingInput {
            jurisdiction: Jurisdiction::Default,
            reporting_to_cra: true,
            information_inaccurate: false,
            tenant_disputed_through_cra: false,
            investigation_conducted_within_30_days: true,
            years_since_delinquency: 3,
            debt_placed_with_collector: false,
            fdcpa_validation_notice_provided: false,
            tenant_disputed_in_writing_30_days: false,
            debt_verified_after_dispute: false,
            ny_tenant_blacklist_reporting: false,
            ct_30_day_pre_reporting_notice_given: false,
            sealed_eviction_record_reported: false,
            willful_violation: false,
        }
    }

    #[test]
    fn compliant_baseline_no_violations() {
        let r = check(&baseline_compliant());
        assert!(r.furnisher_accuracy_compliant);
        assert!(r.investigation_duty_compliant);
        assert!(r.seven_year_aging_compliant);
        assert!(r.fdcpa_compliant);
        assert!(!r.ny_tenant_blacklist_violation);
    }

    #[test]
    fn inaccurate_information_violation() {
        let mut i = baseline_compliant();
        i.information_inaccurate = true;
        let r = check(&i);
        assert!(!r.furnisher_accuracy_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1681s-2(a) FURNISHER ACCURACY VIOLATION")
                && f.contains("notify all CRAs")));
    }

    #[test]
    fn investigation_failure_violation() {
        let mut i = baseline_compliant();
        i.tenant_disputed_through_cra = true;
        i.investigation_conducted_within_30_days = false;
        let r = check(&i);
        assert!(!r.investigation_duty_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1681s-2(b) INVESTIGATION DUTY VIOLATION")
                && f.contains("MANDATORY AND NON-DISCRETIONARY")
                && f.contains("§ 1681s-2(b)(1)(E)")));
    }

    #[test]
    fn seven_year_aging_compliant_at_seven_years() {
        let mut i = baseline_compliant();
        i.years_since_delinquency = 7;
        let r = check(&i);
        assert!(r.seven_year_aging_compliant);
    }

    #[test]
    fn seven_year_aging_violation_at_eight_years() {
        let mut i = baseline_compliant();
        i.years_since_delinquency = 8;
        let r = check(&i);
        assert!(!r.seven_year_aging_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1681c(a)(4) 7-YEAR LIMITATION EXCEEDED")
                && f.contains("8 years")
                && f.contains("DATE OF DELINQUENCY")));
    }

    #[test]
    fn fdcpa_no_validation_notice_violation() {
        let mut i = baseline_compliant();
        i.debt_placed_with_collector = true;
        i.fdcpa_validation_notice_provided = false;
        let r = check(&i);
        assert!(!r.fdcpa_compliant);
        assert!(r.failure_reasons.iter().any(|f| f
            .contains("§ 1692g(a) FDCPA VALIDATION NOTICE VIOLATION")
            && f.contains("5-day validation")
            && f.contains("Regulation F")));
    }

    #[test]
    fn fdcpa_dispute_without_verification_violation() {
        let mut i = baseline_compliant();
        i.debt_placed_with_collector = true;
        i.fdcpa_validation_notice_provided = true;
        i.tenant_disputed_in_writing_30_days = true;
        i.debt_verified_after_dispute = false;
        let r = check(&i);
        assert!(!r.fdcpa_compliant);
        assert!(
            r.failure_reasons
                .iter()
                .any(|f| f.contains("§ 1692g(b) FDCPA VERIFICATION VIOLATION")
                    && f.contains("VERIFY"))
        );
    }

    #[test]
    fn ny_tenant_blacklist_violation() {
        let mut i = baseline_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.ny_tenant_blacklist_reporting = true;
        let r = check(&i);
        assert!(r.ny_tenant_blacklist_violation);
        assert!(r.failure_reasons.iter().any(|f| f.contains("NY HSTPA 2019")
            && f.contains("§ 227-f")
            && f.contains("$500-$1,000 PER VIOLATION")));
    }

    #[test]
    fn ny_tenant_blacklist_not_engaged_outside_ny() {
        let mut i = baseline_compliant();
        i.jurisdiction = Jurisdiction::California;
        i.ny_tenant_blacklist_reporting = true;
        let r = check(&i);
        assert!(!r.ny_tenant_blacklist_violation);
    }

    #[test]
    fn ct_30_day_notice_required_in_ct() {
        let mut i = baseline_compliant();
        i.jurisdiction = Jurisdiction::Connecticut;
        i.ct_30_day_pre_reporting_notice_given = false;
        let r = check(&i);
        assert!(!r.ct_30_day_notice_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 47a-71") && f.contains("30-DAY PRE-REPORTING NOTICE")));
    }

    #[test]
    fn ct_30_day_notice_with_notice_compliant() {
        let mut i = baseline_compliant();
        i.jurisdiction = Jurisdiction::Connecticut;
        i.ct_30_day_pre_reporting_notice_given = true;
        let r = check(&i);
        assert!(r.ct_30_day_notice_compliant);
    }

    #[test]
    fn ct_30_day_notice_not_required_outside_ct() {
        let mut i = baseline_compliant();
        i.jurisdiction = Jurisdiction::California;
        i.ct_30_day_pre_reporting_notice_given = false;
        let r = check(&i);
        assert!(r.ct_30_day_notice_compliant);
    }

    #[test]
    fn sealed_eviction_record_violation() {
        let mut i = baseline_compliant();
        i.sealed_eviction_record_reported = true;
        let r = check(&i);
        assert!(r.sealed_eviction_violation);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("SEALED EVICTION RECORD VIOLATION")
                && f.contains("Oregon SB 970")
                && f.contains("Washington RCW 59.18.367")));
    }

    #[test]
    fn willful_violation_punitive_damages_available() {
        let mut i = baseline_compliant();
        i.information_inaccurate = true;
        i.willful_violation = true;
        let r = check(&i);
        assert!(r.punitive_damages_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1681n WILLFUL NONCOMPLIANCE")
                && f.contains("Safeco Ins. Co. v. Burr")
                && f.contains("$100-$1,000")));
    }

    #[test]
    fn willful_without_underlying_violation_no_punitive() {
        let mut i = baseline_compliant();
        i.willful_violation = true;
        let r = check(&i);
        assert!(!r.punitive_damages_available);
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        for j in [
            Jurisdiction::NewYork,
            Jurisdiction::California,
            Jurisdiction::Connecticut,
            Jurisdiction::Default,
        ] {
            let mut i = baseline_compliant();
            i.jurisdiction = j;
            let r = check(&i);
            let _ = r.ny_tenant_blacklist_violation;
            let _ = r.ct_30_day_notice_compliant;
        }
    }

    #[test]
    fn ny_uniquely_engages_tenant_blacklist_invariant() {
        let mut ny = baseline_compliant();
        ny.jurisdiction = Jurisdiction::NewYork;
        ny.ny_tenant_blacklist_reporting = true;
        let r_ny = check(&ny);
        assert!(r_ny.ny_tenant_blacklist_violation);

        for j in [
            Jurisdiction::California,
            Jurisdiction::Connecticut,
            Jurisdiction::Default,
        ] {
            let mut i = baseline_compliant();
            i.jurisdiction = j;
            i.ny_tenant_blacklist_reporting = true;
            let r = check(&i);
            assert!(!r.ny_tenant_blacklist_violation, "j={:?}", j);
        }
    }

    #[test]
    fn ct_uniquely_requires_30_day_notice_invariant() {
        let mut ct = baseline_compliant();
        ct.jurisdiction = Jurisdiction::Connecticut;
        ct.ct_30_day_pre_reporting_notice_given = false;
        let r_ct = check(&ct);
        assert!(!r_ct.ct_30_day_notice_compliant);

        for j in [
            Jurisdiction::NewYork,
            Jurisdiction::California,
            Jurisdiction::Default,
        ] {
            let mut i = baseline_compliant();
            i.jurisdiction = j;
            i.ct_30_day_pre_reporting_notice_given = false;
            let r = check(&i);
            assert!(r.ct_30_day_notice_compliant, "j={:?}", j);
        }
    }

    #[test]
    fn statutory_damages_range_pinned() {
        let r = check(&baseline_compliant());
        assert_eq!(r.fcra_statutory_damages_min_cents, 10_000);
        assert_eq!(r.fcra_statutory_damages_max_cents, 100_000);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&baseline_compliant());
        assert!(r.citation.contains("§ 1681s-2(a)"));
        assert!(r.citation.contains("§ 1681s-2(b)"));
        assert!(r.citation.contains("§ 1681c"));
        assert!(r.citation.contains("§ 1681c(a)(4)"));
        assert!(r.citation.contains("§ 1681n"));
        assert!(r.citation.contains("§ 1681o"));
        assert!(r.citation.contains("§ 1681e(b)"));
        assert!(r.citation.contains("§ 1681i"));
        assert!(r.citation.contains("§ 1681m"));
        assert!(r.citation.contains("§ 1692e"));
        assert!(r.citation.contains("§ 1692f"));
        assert!(r.citation.contains("§ 1692g"));
        assert!(r.citation.contains("§ 1692k"));
        assert!(r.citation.contains("12 CFR § 1006"));
        assert!(r.citation.contains("NY GBL § 380"));
        assert!(r.citation.contains("NY GBL § 380-d"));
        assert!(r.citation.contains("NY RPL § 227-f"));
        assert!(r.citation.contains("Cal. Civ. Code § 1785"));
        assert!(r.citation.contains("Cal. Civ. Code § 1786"));
        assert!(r.citation.contains("Conn. Gen. Stat. § 47a-71"));
        assert!(r.citation.contains("Oregon SB 970"));
        assert!(r.citation.contains("Washington RCW 59.18.367"));
        assert!(r.citation.contains("Safeco Ins. Co. v. Burr, 551 U.S. 47"));
    }

    #[test]
    fn note_pins_federal_fcra_framework() {
        let r = check(&baseline_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Federal FCRA framework")
            && n.contains("§ 1681s-2(a)")
            && n.contains("§ 1681s-2(b)")
            && n.contains("§ 1681n willful civil liability")
            && n.contains("§ 1681o negligent civil liability")));
    }

    #[test]
    fn note_pins_subsection_s2_a_accuracy() {
        let r = check(&baseline_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1681s-2(a) furnisher accuracy") && n.contains("90 days")));
    }

    #[test]
    fn note_pins_subsection_s2_b_five_element_investigation() {
        let r = check(&baseline_compliant());
        assert!(r.notes.iter().any(|n| n
            .contains("§ 1681s-2(b) investigation duty 5-element framework")
            && n.contains("4th Circuit")
            && n.contains("not verifiable")));
    }

    #[test]
    fn note_pins_subsection_c_seven_year_limitation() {
        let r = check(&baseline_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1681c 7-YEAR STATUTE OF LIMITATIONS")
                && n.contains("10 YEARS")
                && n.contains("DATE OF DELINQUENCY")));
    }

    #[test]
    fn note_pins_section_1681n_willful_safeco() {
        let r = check(&baseline_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1681n willful noncompliance")
                && n.contains("PUNITIVE")
                && n.contains("Safeco Ins. Co. v. Burr, 551 U.S. 47 (2007)")));
    }

    #[test]
    fn note_pins_section_1681o_negligent() {
        let r = check(&baseline_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1681o negligent noncompliance")
                && n.contains("NO statutory damages NO punitive damages")));
    }

    #[test]
    fn note_pins_fdcpa_six_element_framework() {
        let r = check(&baseline_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("FDCPA § 1692 interaction")
                && n.contains("§ 1692e")
                && n.contains("§ 1692f")
                && n.contains("§ 1692g(a)")
                && n.contains("§ 1692g(b)")
                && n.contains("Regulation F")
                && n.contains("§ 1692k")));
    }

    #[test]
    fn note_pins_tenant_screening_cra_subset() {
        let r = check(&baseline_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Tenant-screening CRA subset")
                && n.contains("CoreLogic SafeRent")
                && n.contains("TransUnion SmartMove")
                && n.contains("RentBureau")
                && n.contains("Just Cause laws")));
    }

    #[test]
    fn note_pins_state_law_overlay_four_jurisdictions() {
        let r = check(&baseline_compliant());
        assert!(r.notes.iter().any(|n| n.contains("State law overlay")
            && n.contains("NY GBL § 380")
            && n.contains("NY RPL § 227-f")
            && n.contains("Cal. Civ. Code § 1785")
            && n.contains("Cal. Civ. Code § 1786")
            && n.contains("Oregon SB 970")
            && n.contains("Washington RCW 59.18.367")
            && n.contains("Conn. Gen. Stat. § 47a-71")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&baseline_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-landlord critical fact patterns")
                && n.contains("$5K unpaid rent")
                && n.contains("8 years after")
                && n.contains("NY RPL § 227-f")
                && n.contains("SEALED EVICTION")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&baseline_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Companion to rent_credit_reporting")
                && n.contains("AB 2747")
                && n.contains("adverse_action_notice")
                && n.contains("tenant_rent_judgment_wage_garnishment")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = baseline_compliant();
        i.information_inaccurate = true;
        i.tenant_disputed_through_cra = true;
        i.investigation_conducted_within_30_days = false;
        i.years_since_delinquency = 10;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 3);
    }
}
