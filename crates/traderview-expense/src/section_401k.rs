//! IRC § 401(k) — Cash or Deferred Arrangements
//! (employee elective deferral retirement plans).
//! Direct trader companion to section_408 (traditional
//! IRA — iter 432), section_408a (Roth IRA — iter
//! 430), section_4973 (excess contributions — iter
//! 442), section_4974 (RMD excise — iter 436),
//! section_4975 (prohibited transactions — iter 434),
//! section_72t (10% early-withdrawal penalty),
//! section_162m (executive comp deduction limit —
//! iter 446).
//!
//! § 401(k)(1) allows qualified profit-sharing plan
//! to include a CASH OR DEFERRED ARRANGEMENT under
//! which a covered employee may elect to have
//! employer contribute a portion of compensation to
//! plan TRUST in lieu of receiving as cash;
//! contributions excluded from employee's current
//! gross income; growth tax-deferred; distributions
//! taxed as ordinary income at receipt; Roth § 401(k)
//! contributions made post-tax with qualified
//! distributions tax-free.
//!
//! Trader-critical because traders at companies
//! offering 401(k) plans encounter complex
//! contribution + catch-up + Roth designation + § 415
//! annual addition + § 401(a)(17) compensation-limit
//! interactions:
//! Employee elective deferral ($24,500 in 2026)
//! excluded from current gross income.
//!
//! § 414(v)(1) catch-up ($8,000 in 2026) for employees
//! age 50+.
//!
//! § 414(v)(2)(E) SECURE 2.0 enhanced catch-up
//! ($11,250 in 2026) for ages 60-63 (effective 2025).
//!
//! § 415(c)(1)(A) annual addition limit ($72,000 in
//! 2026) total employer plus employee plus forfeiture
//! allocations to defined contribution plan (does
//! NOT include catch-up).
//!
//! § 401(a)(17) compensation limit ($360,000 in 2026)
//! cap on compensation considered for plan
//! contributions.
//!
//! § 414(v)(7) MANDATORY ROTH CATCH-UP for high
//! earners: beginning 2026, employees with prior-year
//! Social Security wages exceeding $150,000 MUST make
//! catch-up contributions as ROTH (after-tax) rather
//! than pretax.
//!
//! § 402(g)(1) elective deferral limit: employee can
//! defer combined § 401(k) plus § 403(b) plus § 457(b)
//! up to single limit ($24,500 in 2026, apart from
//! § 457(b) which has separate $24,500).
//!
//! § 401(k)(2) qualified status requires ADP/ACP
//! nondiscrimination tests OR SAFE HARBOR designation.
//!
//! **2026 contribution limit framework (IRS Notice
//! 2025-67, IRC § 415(d) cost-of-living
//! adjustments)**:
//!
//! | Limit | 2026 Amount |
//! |-------|-------------|
//! | Employee elective deferral § 402(g)(1) | $24,500 |
//! | Catch-up age 50+ § 414(v)(1) | $8,000 |
//! | Enhanced catch-up ages 60-63 § 414(v)(2)(E) | $11,250 |
//! | Annual addition § 415(c)(1)(A) | $72,000 |
//! | Compensation limit § 401(a)(17) | $360,000 |
//! | HCE (highly compensated employee) threshold | $160,000 |
//! | Roth catch-up wage threshold § 414(v)(7) | $150,000 |
//!
//! **§ 401(k)(3) ADP TEST (Actual Deferral Percentage)**
//! — nondiscrimination test ensures HCEs do not defer
//! disproportionately more than non-HCEs;
//! HCE ADP cannot exceed greater of (a) non-HCE ADP
//! × 1.25; OR (b) non-HCE ADP + 2% (capped at non-
//! HCE × 2).
//!
//! **§ 401(k)(12) SAFE HARBOR ALTERNATIVE** — plan
//! may avoid ADP/ACP testing by providing either:
//! 1. NON-ELECTIVE 3% employer contribution; OR
//! 2. MATCHING contribution (basic) — 100% match on
//!    first 3% of comp + 50% match on next 2%; OR
//! 3. ENHANCED matching — any formula that produces
//!    a match equal to or greater than basic safe
//!    harbor for all eligible employees.
//!
//! **§ 401(k)(2) QUALIFIED CASH OR DEFERRED
//! ARRANGEMENT REQUIREMENTS**:
//! 1. Employee may elect to defer cash compensation;
//! 2. Election available at least annually;
//! 3. Contributions vest IMMEDIATELY (no vesting
//!    schedule for employee deferrals);
//! 4. Distributions only on (a) separation from
//!    service; (b) age 59½; (c) hardship; (d) death;
//!    (e) disability; (f) plan termination;
//! 5. § 401(a)(9) RMD rules apply (with § 4974
//!    25%/10% excise tax on shortfall);
//! 6. Anti-conditioning rule — participation in plan
//!    not conditioned on benefits.
//!
//! **§ 402A(c)(1) DESIGNATED ROTH § 401(k)
//! CONTRIBUTIONS**:
//! 1. Post-tax employee contributions to designated
//!    Roth account within § 401(k) plan;
//! 2. Same § 402(g)(1) elective deferral limit
//!    ($24,500 + $8,000/$11,250 catch-up);
//! 3. Qualified distributions tax-free if 5-year
//!    holding period plus age 59½ / disability /
//!    death / first home (no $10K cap in plan);
//! 4. SECURE 2.0 § 325 — Roth § 401(k) accounts
//!    NOT SUBJECT to § 401(a)(9) lifetime RMDs
//!    starting 2024;
//! 5. SECURE 2.0 § 604 — EMPLOYER MATCH may be
//!    designated as Roth at participant election
//!    (subject to W-2 inclusion as ordinary income).
//!
//! **MEGA BACKDOOR ROTH** — after-tax contributions
//! up to § 415(c)(1)(A) $72,000 annual addition
//! limit (less employee deferral and employer match)
//! converted to Roth via in-plan rollover under
//! § 408A(d)(3); requires plan design supporting
//! after-tax contributions plus in-service
//! distributions.
//!
//! Trader-critical fact patterns:
//!
//! Trader age 35 maxes out 2026: $24,500 elective +
//! $7,500 employer match + remaining $40,000 after-tax
//! to mega backdoor Roth = $72,000 § 415(c) annual
//! addition; converts after-tax to Roth in-plan.
//!
//! Trader age 50-59 maxes out 2026: $24,500 + $8,000
//! catch-up = $32,500 employee deferral; catch-up
//! MUST be Roth if prior-year SS wages > $150K under
//! § 414(v)(7).
//!
//! Trader age 60-63 maxes out 2026: $24,500 + $11,250
//! enhanced catch-up (SECURE 2.0 § 109) = $35,750
//! employee deferral.
//!
//! Trader at small startup: plan uses § 401(k)(12)
//! safe harbor (3% non-elective OR basic match) to
//! avoid ADP/ACP testing.
//!
//! Multi-employer trader: combined § 402(g)(1) limit
//! applies across all § 401(k) + § 403(b) plans;
//! § 457(b) plans have separate $24,500 limit (DOUBLE
//! DEFERRAL strategy for governmental + nonprofit
//! employees).
//!
//! Citations: 26 USC § 401(k)(1)-(13); 26 USC
//! § 401(a)(17); 26 USC § 402(g)(1); 26 USC § 402A
//! (designated Roth contributions); 26 USC § 414(v)
//! (catch-up contributions); 26 USC § 414(v)(2)(E)
//! (SECURE 2.0 enhanced catch-up ages 60-63); 26
//! USC § 414(v)(7) (mandatory Roth catch-up for
//! high earners); 26 USC § 415(c)(1)(A); 26 USC
//! § 415(d) (cost-of-living adjustments); 26 USC
//! § 408A(d)(3) (in-plan Roth rollover); SECURE Act
//! 2.0 of 2022 § 109 (enhanced catch-up); SECURE
//! Act 2.0 of 2022 § 325 (no lifetime RMD on Roth
//! § 401(k)); SECURE Act 2.0 of 2022 § 603
//! (mandatory Roth catch-up); SECURE Act 2.0 of
//! 2022 § 604 (Roth employer match); Pub. L. 117-
//! 328 (Consolidated Appropriations Act, 2023 —
//! SECURE 2.0); IRS Notice 2025-67 (2026 dollar
//! limitations).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgeBracket {
    /// Under age 50.
    UnderFifty,
    /// Age 50 to 59 (standard catch-up).
    FiftyToFiftyNine,
    /// Ages 60-63 (SECURE 2.0 § 109 enhanced
    /// catch-up).
    SixtyToSixtyThree,
    /// Age 64+ (returns to standard catch-up).
    SixtyFourPlus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section401kInput {
    pub age_bracket: AgeBracket,
    /// Employee elective deferral amount in cents.
    pub elective_deferral_cents: u64,
    /// Catch-up contribution amount in cents.
    pub catch_up_contribution_cents: u64,
    /// Employer matching contribution amount in cents.
    pub employer_match_cents: u64,
    /// After-tax contribution amount in cents (mega
    /// backdoor Roth).
    pub after_tax_contribution_cents: u64,
    /// Plan compensation in cents (subject to § 401
    /// (a)(17) cap).
    pub plan_compensation_cents: u64,
    /// Prior year Social Security wages in cents
    /// (§ 414(v)(7) Roth catch-up threshold check).
    pub prior_year_ss_wages_cents: u64,
    /// Whether catch-up designated as Roth.
    pub catch_up_designated_roth: bool,
    /// Whether plan uses § 401(k)(12) safe harbor
    /// designation.
    pub safe_harbor_designated: bool,
    /// ADP for HCEs (highly compensated employees).
    pub hce_adp_percent: u32,
    /// ADP for non-HCEs.
    pub non_hce_adp_percent: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section401kResult {
    pub elective_deferral_limit_cents: u64,
    pub catch_up_limit_cents: u64,
    pub annual_addition_limit_cents: u64,
    pub compensation_limit_cents: u64,
    pub roth_catch_up_threshold_cents: u64,
    pub considered_compensation_cents: u64,
    pub total_annual_addition_cents: u64,
    pub elective_deferral_compliant: bool,
    pub catch_up_compliant: bool,
    pub annual_addition_compliant: bool,
    pub mandatory_roth_catch_up_engaged: bool,
    pub roth_catch_up_violation: bool,
    pub adp_test_passed: bool,
    pub safe_harbor_exempts_adp_test: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section401kInput) -> Section401kResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    // 2026 limits in cents
    let elective_deferral_limit_cents: u64 = 2_450_000;
    let catch_up_limit_cents: u64 = match input.age_bracket {
        AgeBracket::UnderFifty => 0,
        AgeBracket::FiftyToFiftyNine | AgeBracket::SixtyFourPlus => 800_000,
        AgeBracket::SixtyToSixtyThree => 1_125_000,
    };
    let annual_addition_limit_cents: u64 = 7_200_000;
    let compensation_limit_cents: u64 = 36_000_000;
    let roth_catch_up_threshold_cents: u64 = 15_000_000;

    let considered_compensation_cents = input
        .plan_compensation_cents
        .min(compensation_limit_cents);

    let total_annual_addition_cents = input
        .elective_deferral_cents
        .saturating_add(input.employer_match_cents)
        .saturating_add(input.after_tax_contribution_cents);

    let elective_deferral_compliant = input.elective_deferral_cents <= elective_deferral_limit_cents;

    let catch_up_compliant = input.catch_up_contribution_cents <= catch_up_limit_cents;

    let annual_addition_compliant = total_annual_addition_cents <= annual_addition_limit_cents;

    let mandatory_roth_catch_up_engaged = input.prior_year_ss_wages_cents > roth_catch_up_threshold_cents
        && input.catch_up_contribution_cents > 0;

    let roth_catch_up_violation = mandatory_roth_catch_up_engaged && !input.catch_up_designated_roth;

    let adp_safe_harbor_threshold_a = input
        .non_hce_adp_percent
        .saturating_mul(125)
        / 100;
    let adp_safe_harbor_threshold_b = input.non_hce_adp_percent.saturating_add(2);
    let adp_test_passed = input.hce_adp_percent <= adp_safe_harbor_threshold_a.max(adp_safe_harbor_threshold_b);
    let safe_harbor_exempts_adp_test = input.safe_harbor_designated;

    if !elective_deferral_compliant {
        failure_reasons.push(format!(
            "26 USC § 402(g)(1) ELECTIVE DEFERRAL LIMIT EXCEEDED — employee elective deferral of {} cents exceeds 2026 limit of {} cents ($24,500); excess subject to § 4979 excise tax + must be distributed by April 15 of following year to avoid double taxation",
            input.elective_deferral_cents,
            elective_deferral_limit_cents
        ));
    }

    if !catch_up_compliant {
        failure_reasons.push(format!(
            "26 USC § 414(v) CATCH-UP LIMIT EXCEEDED — catch-up contribution of {} cents exceeds 2026 limit of {} cents for {} bracket; excess subject to § 4979 excise tax",
            input.catch_up_contribution_cents,
            catch_up_limit_cents,
            match input.age_bracket {
                AgeBracket::UnderFifty => "under-50 (no catch-up available)",
                AgeBracket::FiftyToFiftyNine => "age 50-59 ($8,000 standard catch-up)",
                AgeBracket::SixtyToSixtyThree => "age 60-63 ($11,250 SECURE 2.0 § 109 enhanced catch-up)",
                AgeBracket::SixtyFourPlus => "age 64+ ($8,000 standard catch-up, enhanced catch-up no longer available)",
            }
        ));
    }

    if !annual_addition_compliant {
        failure_reasons.push(format!(
            "26 USC § 415(c)(1)(A) ANNUAL ADDITION LIMIT EXCEEDED — total annual addition of {} cents (elective deferral + employer match + after-tax) exceeds 2026 § 415(c) limit of {} cents ($72,000); excess subject to correction or disqualification under § 415(c)(6)",
            total_annual_addition_cents,
            annual_addition_limit_cents
        ));
    }

    if input.plan_compensation_cents > compensation_limit_cents {
        failure_reasons.push(format!(
            "26 USC § 401(a)(17) COMPENSATION LIMIT — plan compensation of {} cents EXCEEDS 2026 cap of {} cents ($360,000); only first $360,000 considered for plan contributions; excess compensation cannot be used for matching, profit-sharing, or vesting purposes",
            input.plan_compensation_cents,
            compensation_limit_cents
        ));
    }

    if mandatory_roth_catch_up_engaged && !input.catch_up_designated_roth {
        failure_reasons.push(format!(
            "26 USC § 414(v)(7) MANDATORY ROTH CATCH-UP for high earners (SECURE Act 2.0 § 603 effective 2026) — prior-year Social Security wages of {} cents EXCEED $150,000 threshold; catch-up contribution MUST be designated ROTH (after-tax); pre-tax catch-up election INVALID; plan must convert or refund",
            input.prior_year_ss_wages_cents
        ));
    }

    if !safe_harbor_exempts_adp_test && !adp_test_passed {
        failure_reasons.push(format!(
            "26 USC § 401(k)(3) ADP TEST FAILED — HCE ADP of {}% exceeds permitted threshold of greater of (a) non-HCE ADP × 1.25 = {}% OR (b) non-HCE ADP + 2 = {}% (capped at non-HCE × 2); failure requires (i) corrective distributions to HCEs OR (ii) qualified non-elective contributions to non-HCEs; alternative: switch to § 401(k)(12) safe harbor",
            input.hce_adp_percent,
            adp_safe_harbor_threshold_a,
            adp_safe_harbor_threshold_b
        ));
    }

    if safe_harbor_exempts_adp_test {
        failure_reasons.push(
            "26 USC § 401(k)(12) SAFE HARBOR — plan exempt from ADP/ACP testing via (1) NON-ELECTIVE 3% employer contribution; OR (2) MATCHING contribution (basic) = 100% on first 3% of comp + 50% on next 2%; OR (3) ENHANCED matching equal to or greater than basic safe harbor for all eligible employees".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 401(k)(1) — qualified profit-sharing plan may include CASH OR DEFERRED ARRANGEMENT under which covered employee may elect to defer compensation to plan trust in lieu of receiving as cash; contributions excluded from current gross income; growth tax-deferred; distributions taxed as ordinary income at receipt".to_string(),
        "2026 contribution limits (IRS Notice 2025-67, IRC § 415(d) cost-of-living adjustments): elective deferral § 402(g)(1) $24,500; catch-up age 50+ § 414(v)(1) $8,000; enhanced catch-up ages 60-63 § 414(v)(2)(E) $11,250; annual addition § 415(c)(1)(A) $72,000; compensation limit § 401(a)(17) $360,000; HCE threshold $160,000; Roth catch-up wage threshold § 414(v)(7) $150,000".to_string(),
        "26 USC § 414(v)(2)(E) SECURE 2.0 ENHANCED CATCH-UP for ages 60-63 (effective 2025) — higher catch-up contribution limit applies for employees who turn 60, 61, 62, and 63 in calendar year; 2026 amount $11,250 (vs $8,000 standard); SECURE Act 2.0 of 2022 § 109 (Pub. L. 117-328); reverts to $8,000 standard catch-up at age 64+".to_string(),
        "26 USC § 414(v)(7) MANDATORY ROTH CATCH-UP for high earners (SECURE Act 2.0 § 603 effective 2026) — employees with prior-year Social Security wages exceeding $150,000 MUST make catch-up contributions as Roth (after-tax) rather than pretax; plan must convert or refund non-compliant pretax catch-ups; designed to limit pretax catch-up benefit for high earners".to_string(),
        "26 USC § 401(k)(3) ADP TEST (Actual Deferral Percentage) — HCE ADP cannot exceed greater of (a) non-HCE ADP × 1.25; OR (b) non-HCE ADP + 2% (capped at non-HCE × 2); failure requires corrective distributions to HCEs OR qualified non-elective contributions to non-HCEs; § 401(k)(12) SAFE HARBOR alternative".to_string(),
        "26 USC § 401(k)(12) SAFE HARBOR — plan may avoid ADP/ACP testing via (1) NON-ELECTIVE 3% employer contribution to all eligible NHCEs; OR (2) MATCHING contribution (basic) — 100% match on first 3% of comp + 50% match on next 2%; OR (3) ENHANCED matching equal to or greater than basic safe harbor; all safe-harbor contributions must be 100% vested at all times".to_string(),
        "26 USC § 401(k)(2) QUALIFIED CASH OR DEFERRED ARRANGEMENT REQUIREMENTS: (1) employee may elect to defer cash compensation; (2) election available at least annually; (3) contributions vest IMMEDIATELY (no vesting schedule for employee deferrals); (4) distributions only on separation from service / age 59½ / hardship / death / disability / plan termination; (5) § 401(a)(9) RMD rules apply; (6) anti-conditioning rule".to_string(),
        "26 USC § 402A DESIGNATED ROTH § 401(k) CONTRIBUTIONS — (1) post-tax employee contributions to designated Roth account within § 401(k) plan; (2) same § 402(g)(1) elective deferral limit ($24,500 + catch-up); (3) qualified distributions tax-free with 5-year holding plus age 59½/disability/death; (4) SECURE 2.0 § 325 — Roth § 401(k) NOT SUBJECT to § 401(a)(9) lifetime RMDs starting 2024; (5) SECURE 2.0 § 604 — EMPLOYER MATCH may be designated as Roth at participant election (W-2 inclusion as ordinary income)".to_string(),
        "MEGA BACKDOOR ROTH strategy — after-tax employee contributions up to § 415(c)(1)(A) $72,000 annual addition limit (less employee deferral and employer match) converted to Roth via in-plan rollover under § 408A(d)(3); requires plan design supporting both after-tax contributions and in-service distributions; pro-rata rule limits effectiveness if traditional pretax balance exists".to_string(),
        "Multi-employer / multi-plan considerations: § 402(g)(1) combined elective deferral limit applies across all § 401(k) + § 403(b) plans (NOT split between plans); § 457(b) governmental plans have SEPARATE $24,500 limit (allows DOUBLE DEFERRAL strategy for government + nonprofit employees with eligible § 457(b)); § 415(c) annual addition computed per employer (separate limits at unaffiliated employers)".to_string(),
        "Trader-critical fact patterns: (1) trader age 35 maxes 2026 — $24,500 elective + $7,500 employer match + $40,000 after-tax = $72,000 § 415(c); converts after-tax via mega backdoor Roth; (2) age 50-59 maxes — $24,500 + $8,000 catch-up; catch-up MUST BE ROTH if prior SS wages > $150K (§ 414(v)(7)); (3) age 60-63 maxes — $24,500 + $11,250 enhanced catch-up (SECURE 2.0 § 109); (4) small startup safe harbor (3% non-elective or basic match) avoids ADP/ACP testing; (5) multi-employer trader combined § 402(g) limit + separate § 457(b) $24,500 DOUBLE DEFERRAL".to_string(),
        "Companion to section_408 (traditional IRA) + section_408a (Roth IRA) + section_4973 (excess contribution excise tax) + section_4974 (RMD excise tax) + section_4975 (prohibited transactions) + section_72t (10% early-withdrawal penalty) + section_162m ($1M public-company executive comp deduction limit)".to_string(),
    ];

    Section401kResult {
        elective_deferral_limit_cents,
        catch_up_limit_cents,
        annual_addition_limit_cents,
        compensation_limit_cents,
        roth_catch_up_threshold_cents,
        considered_compensation_cents,
        total_annual_addition_cents,
        elective_deferral_compliant,
        catch_up_compliant,
        annual_addition_compliant,
        mandatory_roth_catch_up_engaged,
        roth_catch_up_violation,
        adp_test_passed,
        safe_harbor_exempts_adp_test,
        failure_reasons,
        citation: "26 USC § 401(k)(1)-(13); 26 USC § 401(a)(17); 26 USC § 402(g)(1); 26 USC § 402A (designated Roth contributions); 26 USC § 414(v) (catch-up contributions); 26 USC § 414(v)(2)(E) (SECURE 2.0 enhanced catch-up ages 60-63); 26 USC § 414(v)(7) (mandatory Roth catch-up for high earners); 26 USC § 415(c)(1)(A); 26 USC § 415(d) (cost-of-living adjustments); 26 USC § 408A(d)(3) (in-plan Roth rollover); SECURE Act 2.0 of 2022 § 109; SECURE Act 2.0 of 2022 § 325; SECURE Act 2.0 of 2022 § 603; SECURE Act 2.0 of 2022 § 604; Pub. L. 117-328 (Consolidated Appropriations Act, 2023); IRS Notice 2025-67 (2026 dollar limitations)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn under_50_max() -> Section401kInput {
        Section401kInput {
            age_bracket: AgeBracket::UnderFifty,
            elective_deferral_cents: 2_450_000,
            catch_up_contribution_cents: 0,
            employer_match_cents: 750_000,
            after_tax_contribution_cents: 4_000_000,
            plan_compensation_cents: 30_000_000,
            prior_year_ss_wages_cents: 15_000_000,
            catch_up_designated_roth: false,
            safe_harbor_designated: true,
            hce_adp_percent: 6,
            non_hce_adp_percent: 4,
        }
    }

    #[test]
    fn under_50_max_all_compliant() {
        let r = check(&under_50_max());
        assert!(r.elective_deferral_compliant);
        assert!(r.catch_up_compliant);
        assert!(r.annual_addition_compliant);
        assert_eq!(r.elective_deferral_limit_cents, 2_450_000);
        assert_eq!(r.annual_addition_limit_cents, 7_200_000);
    }

    #[test]
    fn under_50_no_catch_up_limit() {
        let r = check(&under_50_max());
        assert_eq!(r.catch_up_limit_cents, 0);
    }

    #[test]
    fn fifty_to_fifty_nine_catch_up_8000() {
        let mut i = under_50_max();
        i.age_bracket = AgeBracket::FiftyToFiftyNine;
        i.catch_up_contribution_cents = 800_000;
        let r = check(&i);
        assert_eq!(r.catch_up_limit_cents, 800_000);
        assert!(r.catch_up_compliant);
    }

    #[test]
    fn sixty_to_sixty_three_enhanced_catch_up_11250() {
        let mut i = under_50_max();
        i.age_bracket = AgeBracket::SixtyToSixtyThree;
        i.catch_up_contribution_cents = 1_125_000;
        let r = check(&i);
        assert_eq!(r.catch_up_limit_cents, 1_125_000);
        assert!(r.catch_up_compliant);
    }

    #[test]
    fn sixty_four_plus_reverts_to_standard_catch_up() {
        let mut i = under_50_max();
        i.age_bracket = AgeBracket::SixtyFourPlus;
        i.catch_up_contribution_cents = 800_000;
        let r = check(&i);
        assert_eq!(r.catch_up_limit_cents, 800_000);
        assert!(r.catch_up_compliant);
    }

    #[test]
    fn elective_deferral_over_limit_violation() {
        let mut i = under_50_max();
        i.elective_deferral_cents = 2_500_000;
        let r = check(&i);
        assert!(!r.elective_deferral_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 402(g)(1)")
            && f.contains("$24,500")
            && f.contains("§ 4979")));
    }

    #[test]
    fn catch_up_over_limit_violation() {
        let mut i = under_50_max();
        i.age_bracket = AgeBracket::FiftyToFiftyNine;
        i.catch_up_contribution_cents = 1_000_000;
        let r = check(&i);
        assert!(!r.catch_up_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 414(v)")
            && f.contains("EXCEEDED")));
    }

    #[test]
    fn annual_addition_over_72k_violation() {
        let mut i = under_50_max();
        i.elective_deferral_cents = 2_450_000;
        i.employer_match_cents = 2_000_000;
        i.after_tax_contribution_cents = 5_000_000;
        let r = check(&i);
        assert!(!r.annual_addition_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 415(c)(1)(A)")
            && f.contains("$72,000")
            && f.contains("§ 415(c)(6)")));
    }

    #[test]
    fn compensation_over_360k_capped() {
        let mut i = under_50_max();
        i.plan_compensation_cents = 50_000_000;
        let r = check(&i);
        assert_eq!(r.considered_compensation_cents, 36_000_000);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 401(a)(17)")
            && f.contains("$360,000")));
    }

    #[test]
    fn mandatory_roth_catch_up_high_earner_engaged() {
        let mut i = under_50_max();
        i.age_bracket = AgeBracket::FiftyToFiftyNine;
        i.catch_up_contribution_cents = 800_000;
        i.prior_year_ss_wages_cents = 20_000_000;
        i.catch_up_designated_roth = false;
        let r = check(&i);
        assert!(r.mandatory_roth_catch_up_engaged);
        assert!(r.roth_catch_up_violation);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 414(v)(7)")
            && f.contains("SECURE Act 2.0 § 603")
            && f.contains("$150,000 threshold")));
    }

    #[test]
    fn mandatory_roth_catch_up_low_earner_not_engaged() {
        let mut i = under_50_max();
        i.age_bracket = AgeBracket::FiftyToFiftyNine;
        i.catch_up_contribution_cents = 800_000;
        i.prior_year_ss_wages_cents = 10_000_000;
        i.catch_up_designated_roth = false;
        let r = check(&i);
        assert!(!r.mandatory_roth_catch_up_engaged);
        assert!(!r.roth_catch_up_violation);
    }

    #[test]
    fn mandatory_roth_catch_up_when_designated_roth_compliant() {
        let mut i = under_50_max();
        i.age_bracket = AgeBracket::FiftyToFiftyNine;
        i.catch_up_contribution_cents = 800_000;
        i.prior_year_ss_wages_cents = 20_000_000;
        i.catch_up_designated_roth = true;
        let r = check(&i);
        assert!(r.mandatory_roth_catch_up_engaged);
        assert!(!r.roth_catch_up_violation);
    }

    #[test]
    fn adp_test_passes_within_125_percent() {
        let mut i = under_50_max();
        i.safe_harbor_designated = false;
        i.hce_adp_percent = 5;
        i.non_hce_adp_percent = 4;
        let r = check(&i);
        assert!(r.adp_test_passed);
    }

    #[test]
    fn adp_test_fails_exceeds_thresholds() {
        let mut i = under_50_max();
        i.safe_harbor_designated = false;
        i.hce_adp_percent = 8;
        i.non_hce_adp_percent = 4;
        let r = check(&i);
        assert!(!r.adp_test_passed);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 401(k)(3) ADP TEST FAILED")
            && f.contains("corrective distributions")
            && f.contains("§ 401(k)(12) safe harbor")));
    }

    #[test]
    fn safe_harbor_exempts_adp_test() {
        let mut i = under_50_max();
        i.safe_harbor_designated = true;
        i.hce_adp_percent = 10;
        i.non_hce_adp_percent = 2;
        let r = check(&i);
        assert!(r.safe_harbor_exempts_adp_test);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 401(k)(12) SAFE HARBOR")
            && f.contains("NON-ELECTIVE 3%")
            && f.contains("100% on first 3%")));
    }

    #[test]
    fn age_bracket_truth_table_four_cells() {
        for (bracket, expected_catch_up_cents) in [
            (AgeBracket::UnderFifty, 0u64),
            (AgeBracket::FiftyToFiftyNine, 800_000u64),
            (AgeBracket::SixtyToSixtyThree, 1_125_000u64),
            (AgeBracket::SixtyFourPlus, 800_000u64),
        ] {
            let mut i = under_50_max();
            i.age_bracket = bracket;
            let r = check(&i);
            assert_eq!(r.catch_up_limit_cents, expected_catch_up_cents, "bracket={:?}", bracket);
        }
    }

    #[test]
    fn sixty_to_sixty_three_uniquely_enhanced_invariant() {
        let mut enhanced = under_50_max();
        enhanced.age_bracket = AgeBracket::SixtyToSixtyThree;
        let r_enhanced = check(&enhanced);
        assert_eq!(r_enhanced.catch_up_limit_cents, 1_125_000);

        for bracket in [
            AgeBracket::UnderFifty,
            AgeBracket::FiftyToFiftyNine,
            AgeBracket::SixtyFourPlus,
        ] {
            let mut i = under_50_max();
            i.age_bracket = bracket;
            let r = check(&i);
            assert!(r.catch_up_limit_cents < 1_125_000, "bracket={:?}", bracket);
        }
    }

    #[test]
    fn limit_cents_pinned_2026() {
        let r = check(&under_50_max());
        assert_eq!(r.elective_deferral_limit_cents, 2_450_000);
        assert_eq!(r.annual_addition_limit_cents, 7_200_000);
        assert_eq!(r.compensation_limit_cents, 36_000_000);
        assert_eq!(r.roth_catch_up_threshold_cents, 15_000_000);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&under_50_max());
        assert!(r.citation.contains("§ 401(k)(1)-(13)"));
        assert!(r.citation.contains("§ 401(a)(17)"));
        assert!(r.citation.contains("§ 402(g)(1)"));
        assert!(r.citation.contains("§ 402A"));
        assert!(r.citation.contains("§ 414(v)"));
        assert!(r.citation.contains("§ 414(v)(2)(E)"));
        assert!(r.citation.contains("§ 414(v)(7)"));
        assert!(r.citation.contains("§ 415(c)(1)(A)"));
        assert!(r.citation.contains("§ 415(d)"));
        assert!(r.citation.contains("§ 408A(d)(3)"));
        assert!(r.citation.contains("SECURE Act 2.0 of 2022 § 109"));
        assert!(r.citation.contains("SECURE Act 2.0 of 2022 § 325"));
        assert!(r.citation.contains("SECURE Act 2.0 of 2022 § 603"));
        assert!(r.citation.contains("SECURE Act 2.0 of 2022 § 604"));
        assert!(r.citation.contains("Pub. L. 117-328"));
        assert!(r.citation.contains("IRS Notice 2025-67"));
    }

    #[test]
    fn note_pins_subsection_k1_cash_or_deferred_arrangement() {
        let r = check(&under_50_max());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 401(k)(1)")
            && n.contains("CASH OR DEFERRED ARRANGEMENT")
            && n.contains("growth tax-deferred")));
    }

    #[test]
    fn note_pins_2026_seven_limits() {
        let r = check(&under_50_max());
        assert!(r.notes.iter().any(|n|
            n.contains("2026 contribution limits")
            && n.contains("$24,500")
            && n.contains("$8,000")
            && n.contains("$11,250")
            && n.contains("$72,000")
            && n.contains("$360,000")
            && n.contains("$160,000")
            && n.contains("$150,000")));
    }

    #[test]
    fn note_pins_secure_2_section_109_enhanced_catch_up() {
        let r = check(&under_50_max());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 414(v)(2)(E)")
            && n.contains("SECURE 2.0 ENHANCED CATCH-UP")
            && n.contains("ages 60-63")
            && n.contains("$11,250 (vs $8,000")
            && n.contains("SECURE Act 2.0 of 2022 § 109")));
    }

    #[test]
    fn note_pins_secure_2_section_603_mandatory_roth() {
        let r = check(&under_50_max());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 414(v)(7)")
            && n.contains("MANDATORY ROTH CATCH-UP")
            && n.contains("SECURE Act 2.0 § 603")
            && n.contains("$150,000")));
    }

    #[test]
    fn note_pins_section_k3_adp_test() {
        let r = check(&under_50_max());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 401(k)(3) ADP TEST")
            && n.contains("× 1.25")
            && n.contains("+ 2%")
            && n.contains("non-HCE × 2")));
    }

    #[test]
    fn note_pins_section_k12_safe_harbor_three_alternatives() {
        let r = check(&under_50_max());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 401(k)(12) SAFE HARBOR")
            && n.contains("NON-ELECTIVE 3%")
            && n.contains("100% match on first 3%")
            && n.contains("50% match on next 2%")
            && n.contains("ENHANCED matching")));
    }

    #[test]
    fn note_pins_section_k2_six_requirements() {
        let r = check(&under_50_max());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 401(k)(2) QUALIFIED CASH OR DEFERRED ARRANGEMENT REQUIREMENTS")
            && n.contains("vest IMMEDIATELY")
            && n.contains("anti-conditioning rule")));
    }

    #[test]
    fn note_pins_section_402a_roth_401k() {
        let r = check(&under_50_max());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 402A DESIGNATED ROTH § 401(k) CONTRIBUTIONS")
            && n.contains("SECURE 2.0 § 325")
            && n.contains("NOT SUBJECT to § 401(a)(9) lifetime RMDs")
            && n.contains("SECURE 2.0 § 604")
            && n.contains("EMPLOYER MATCH may be designated as Roth")));
    }

    #[test]
    fn note_pins_mega_backdoor_roth() {
        let r = check(&under_50_max());
        assert!(r.notes.iter().any(|n|
            n.contains("MEGA BACKDOOR ROTH")
            && n.contains("§ 415(c)(1)(A) $72,000")
            && n.contains("§ 408A(d)(3)")
            && n.contains("pro-rata rule")));
    }

    #[test]
    fn note_pins_multi_employer_457b_double_deferral() {
        let r = check(&under_50_max());
        assert!(r.notes.iter().any(|n|
            n.contains("Multi-employer / multi-plan considerations")
            && n.contains("§ 457(b) governmental plans have SEPARATE $24,500 limit")
            && n.contains("DOUBLE DEFERRAL strategy")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&under_50_max());
        assert!(r.notes.iter().any(|n|
            n.contains("Trader-critical fact patterns")
            && n.contains("$72,000 § 415(c)")
            && n.contains("enhanced catch-up (SECURE 2.0 § 109)")
            && n.contains("DOUBLE DEFERRAL")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&under_50_max());
        assert!(r.notes.iter().any(|n|
            n.contains("Companion to section_408")
            && n.contains("section_408a")
            && n.contains("section_4973")
            && n.contains("section_162m")));
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = under_50_max();
        i.elective_deferral_cents = u64::MAX;
        i.employer_match_cents = u64::MAX;
        let r = check(&i);
        let _ = r.total_annual_addition_cents;
    }
}
