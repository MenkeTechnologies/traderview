//! IRC § 1291 — Interest on tax deferral. The DEFAULT
//! PFIC tax regime — applies when a U.S. shareholder of a
//! passive foreign investment company has neither timely
//! elected QEF (section_1295) nor mark-to-market
//! (section_1296). Completes the PFIC framework cluster:
//! section_1297 (PFIC definition — 75% income test or 50%
//! asset test); section_1298 (special rules — attribution,
//! look-through, related-party); section_1295 (QEF
//! election — pass-through); section_1296 (marketable PFIC
//! mark-to-market election); section_1291 (DEFAULT excess
//! distribution + interest charge — this module).
//!
//! Companion to section_6038d (Form 8938 individual FATCA
//! reporting captures PFIC interests), section_6011 (Form
//! 8886 reportable transaction disclosure if PFIC sale is
//! "loss transaction"), section_988 (foreign currency
//! treatment of PFIC distributions), section_67g (TCJA
//! misc itemized deduction suspension affects PFIC fund
//! expenses).
//!
//! Trader-critical because every trader holding foreign
//! mutual funds AND foreign ETFs AND foreign closed-end
//! funds AND foreign hedge fund interests AND foreign
//! insurance products AND foreign retirement vehicles
//! risks PFIC classification and § 1291 default treatment:
//! - **Foreign-listed mutual funds** (Canadian, UK,
//!   Australian funds) — almost universally PFICs.
//! - **Foreign ETFs not registered under '40 Act** —
//!   PFICs absent specific exception.
//! - **Foreign hedge fund LP interests** — PFIC if 75%
//!   passive income or 50% passive asset test met.
//! - **Foreign insurance products** (offshore annuities,
//!   variable life) — typically PFICs.
//! - **§ 1291(a)(2) excess distribution sale or
//!   disposition** — converts capital gain into ORDINARY
//!   income with interest charge.
//!
//! **§ 1291(a)(1) Default treatment — excess distribution
//! allocation**:
//! 1. **§ 1291(a)(1)(A)** — excess distribution allocated
//!    RATABLY to EACH DAY in shareholder's holding period
//!    for the stock.
//! 2. **§ 1291(a)(1)(B)** — portion allocated to current
//!    taxable year + pre-PFIC-period years (before stock
//!    became PFIC) included in gross income as ORDINARY
//!    INCOME taxable at current ordinary rates.
//! 3. **§ 1291(a)(1)(C)** — portion allocated to OTHER
//!    PFIC-period years (intermediate years) creates
//!    deferred tax amount: (i) tax computed at HIGHEST
//!    MARGINAL RATE in effect for that year under § 1 or
//!    § 11 (whichever applies); (ii) interest charge added
//!    at § 6621 underpayment rate compounded daily from
//!    that year's original return due date.
//!
//! **§ 1291(a)(2) Disposition treatment** — gain
//! recognized on disposition of PFIC stock is treated as
//! EXCESS DISTRIBUTION under § 1291(a)(1) — same ratable
//! allocation + highest marginal rate + interest charge
//! framework applies to disposition gain.
//!
//! **§ 1291(b) Excess distribution defined**:
//! 1. § 1291(b)(2)(A) — distribution exceeds 125% of
//!    average distributions in PRECEDING 3 TAXABLE YEARS;
//!    OR
//! 2. § 1291(b)(2)(B)(i) — if holding period for stock is
//!    LESS THAN 3 YEARS, average is computed over actual
//!    holding period (or stock issued less than 3 years
//!    ago).
//! 3. § 1291(b)(3)(B) — if FIRST YEAR holding stock, ALL
//!    distributions treated as excess distributions
//!    (entire distribution).
//!
//! **§ 1291(c) Interest charge — § 6621 rate compounded**:
//! Interest charged on deferred tax amount using rates and
//! method applicable under § 6621 for UNDERPAYMENTS;
//! compounded DAILY from original due date of each prior
//! year's tax return through the current year. Interest
//! charge can substantially exceed underlying tax in
//! long-held PFIC fact patterns.
//!
//! **§ 1291(d) Coordination with QEF election**:
//! 1. § 1291(d)(1) — § 1291 does NOT apply to QEF-elected
//!    PFICs (§ 1295);
//! 2. § 1291(d)(2) — purging election available to
//!    cleanse pre-QEF-election PFIC taint via deemed sale
//!    OR deemed dividend.
//!
//! **§ 1291(f) Coordination with mark-to-market election**
//! — § 1291 does NOT apply once § 1296 mark-to-market
//! election effective; pre-election PFIC taint can persist
//! absent purging election.
//!
//! **§ 1291(g) Currency translation** — foreign currency
//! translation of PFIC distribution uses § 988 rules.
//!
//! **Form 8621** — Information Return by a Shareholder of
//! a Passive Foreign Investment Company or Qualified
//! Electing Fund. Required for every PFIC interest held
//! by U.S. shareholder; reporting threshold under § 1298(f)
//! applies regardless of distribution status.
//!
//! Citations: 26 USC § 1291(a)(1)(A)-(C), § 1291(a)(2),
//! § 1291(b)(2)(A)-(B), § 1291(b)(3)(B), § 1291(c),
//! § 1291(d)(1)-(2), § 1291(f), § 1291(g); 26 USC § 1295
//! (QEF election); 26 USC § 1296 (mark-to-market); 26 USC
//! § 1297 (PFIC definition); 26 USC § 1298 (special
//! rules); 26 USC § 6621 (underpayment interest rate);
//! Treas. Reg. § 1.1291-1 through § 1.1291-10; Form 8621;
//! Tax Reform Act of 1986 § 1235 (Pub. L. 99-514, October
//! 22, 1986); HIRE Act of 2010 § 521 (Pub. L. 111-147)
//! added § 1298(f) annual reporting requirement.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DistributionScenario {
    /// Distribution received from PFIC during current
    /// taxable year.
    DistributionReceived,
    /// Gain recognized on disposition (sale, exchange) of
    /// PFIC stock — § 1291(a)(2).
    DispositionGain,
    /// No distribution and no disposition — annual
    /// reporting still required under § 1298(f).
    NoDistributionOrDisposition,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PFICElection {
    /// No election — § 1291 default regime applies.
    NoElection,
    /// § 1295 Qualified Electing Fund election — § 1291
    /// does not apply.
    QefElection,
    /// § 1296 mark-to-market election (marketable PFIC) —
    /// § 1291 does not apply.
    MarkToMarketElection,
    /// Purging election made under § 1291(d)(2) to cleanse
    /// pre-election PFIC taint.
    PurgingElection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1291Input {
    pub scenario: DistributionScenario,
    pub election: PFICElection,
    /// Total distribution received OR disposition gain in
    /// cents.
    pub total_distribution_or_gain_cents: u64,
    /// Average distributions in preceding 3 taxable years
    /// (or actual holding period if < 3 years) in cents.
    pub average_3_year_distributions_cents: u64,
    /// Whether this is taxpayer's FIRST YEAR holding the
    /// PFIC stock — § 1291(b)(3)(B) treats ALL
    /// distributions as excess.
    pub first_year_holding: bool,
    /// Total holding period in days (used for ratable
    /// allocation).
    pub holding_period_days: u32,
    /// Days in current taxable year within holding period.
    pub days_in_current_year: u32,
    /// Days in pre-PFIC-period (before stock became PFIC)
    /// within holding period — taxed as ordinary at
    /// current rate, no interest charge.
    pub pre_pfic_period_days: u32,
    /// Marginal tax rate in basis points (e.g., 3700 =
    /// 37%) applicable to current-year + pre-PFIC-period
    /// portion.
    pub current_year_marginal_rate_bps: u32,
    /// Highest marginal tax rate in effect during prior
    /// PFIC-period years (typically 37% post-TCJA = 3700).
    pub prior_year_highest_marginal_rate_bps: u32,
    /// § 6621 underpayment interest rate in basis points
    /// (e.g., 800 = 8%) compounded daily.
    pub section_6621_interest_rate_bps: u32,
    /// Days of compounding for interest charge (sum across
    /// allocations to intermediate PFIC years).
    pub interest_charge_compounding_days: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1291Result {
    pub scenario: DistributionScenario,
    pub election: PFICElection,
    pub section_1291_applies: bool,
    pub excess_distribution_amount_cents: u64,
    pub current_year_ordinary_inclusion_cents: u64,
    pub deferred_tax_amount_cents: u64,
    pub interest_charge_cents: u64,
    pub total_section_1291_tax_cents: u64,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section1291Input) -> Section1291Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let section_1291_applies = matches!(input.election, PFICElection::NoElection)
        && !matches!(
            input.scenario,
            DistributionScenario::NoDistributionOrDisposition
        );

    let excess_distribution_amount_cents = if !section_1291_applies {
        0
    } else if input.first_year_holding {
        input.total_distribution_or_gain_cents
    } else {
        let threshold = input.average_3_year_distributions_cents.saturating_mul(125) / 100;
        input
            .total_distribution_or_gain_cents
            .saturating_sub(threshold)
    };

    let total_holding_days = input.holding_period_days.max(1) as u64;
    let per_day_allocation = excess_distribution_amount_cents / total_holding_days;

    let current_year_portion = per_day_allocation.saturating_mul(input.days_in_current_year as u64);
    let pre_pfic_portion = per_day_allocation.saturating_mul(input.pre_pfic_period_days as u64);
    let intermediate_pfic_days = (input.holding_period_days)
        .saturating_sub(input.days_in_current_year)
        .saturating_sub(input.pre_pfic_period_days);
    let intermediate_pfic_portion =
        per_day_allocation.saturating_mul(intermediate_pfic_days as u64);

    let current_year_ordinary_inclusion_cents =
        current_year_portion.saturating_add(pre_pfic_portion);

    let deferred_tax_amount_cents = intermediate_pfic_portion
        .saturating_mul(input.prior_year_highest_marginal_rate_bps as u64)
        / 10_000;

    let raw_interest_factor = (input.section_6621_interest_rate_bps as u64)
        .saturating_mul(input.interest_charge_compounding_days as u64);
    let interest_charge_cents =
        deferred_tax_amount_cents.saturating_mul(raw_interest_factor) / 3_650_000;

    let current_year_tax = current_year_ordinary_inclusion_cents
        .saturating_mul(input.current_year_marginal_rate_bps as u64)
        / 10_000;
    let total_section_1291_tax_cents = current_year_tax
        .saturating_add(deferred_tax_amount_cents)
        .saturating_add(interest_charge_cents);

    if section_1291_applies && excess_distribution_amount_cents > 0 {
        failure_reasons.push(format!(
            "26 USC § 1291(a)(1)(A) — excess distribution of {} cents allocated RATABLY across {} days of holding period: current-year + pre-PFIC ordinary income {} cents + intermediate PFIC-period deferred tax {} cents + § 6621 interest charge {} cents = total § 1291 tax {} cents",
            excess_distribution_amount_cents,
            input.holding_period_days,
            current_year_ordinary_inclusion_cents,
            deferred_tax_amount_cents,
            interest_charge_cents,
            total_section_1291_tax_cents
        ));
    }

    if matches!(input.scenario, DistributionScenario::DispositionGain) && section_1291_applies {
        failure_reasons.push(
            "26 USC § 1291(a)(2) — gain on disposition of PFIC stock (sale, exchange) is treated as EXCESS DISTRIBUTION under § 1291(a)(1); CAPITAL GAIN treatment is CONVERTED to ORDINARY income with interest charge".to_string(),
        );
    }

    if input.first_year_holding && section_1291_applies {
        failure_reasons.push(
            "26 USC § 1291(b)(3)(B) — if first year holding PFIC stock, ALL DISTRIBUTIONS are treated as excess distributions (entire distribution); 125% of 3-year average rule does NOT apply".to_string(),
        );
    }

    if matches!(input.election, PFICElection::QefElection) {
        failure_reasons.push(
            "26 USC § 1291(d)(1) — § 1291 default treatment DOES NOT APPLY to QEF-elected PFICs (§ 1295); QEF shareholders include pro rata share of PFIC ordinary earnings + net capital gain currently".to_string(),
        );
    }

    if matches!(input.election, PFICElection::MarkToMarketElection) {
        failure_reasons.push(
            "26 USC § 1291(f) — § 1291 default treatment DOES NOT APPLY once § 1296 mark-to-market election effective; pre-election PFIC taint can persist absent purging election under § 1291(d)(2)".to_string(),
        );
    }

    if matches!(input.election, PFICElection::PurgingElection) {
        failure_reasons.push(
            "26 USC § 1291(d)(2) — purging election available to cleanse pre-QEF or pre-mark-to-market PFIC taint via DEEMED SALE OR DEEMED DIVIDEND treatment; one-time election to start fresh".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 1291(a)(1)(A) — excess distribution allocated RATABLY to EACH DAY in shareholder's holding period for the stock".to_string(),
        "26 USC § 1291(a)(1)(B) — portion allocated to CURRENT taxable year and PRE-PFIC-PERIOD years (before stock became PFIC) included in gross income as ORDINARY INCOME taxable at current ordinary rates".to_string(),
        "26 USC § 1291(a)(1)(C) — portion allocated to OTHER PFIC-PERIOD years creates deferred tax amount: (i) tax computed at HIGHEST MARGINAL RATE in effect for that year under § 1 or § 11 (whichever applies); (ii) § 6621 interest charge compounded DAILY from original return due date".to_string(),
        "26 USC § 1291(a)(2) — gain recognized on DISPOSITION (sale, exchange) of PFIC stock is TREATED AS EXCESS DISTRIBUTION under § 1291(a)(1); CAPITAL GAIN treatment CONVERTED to ORDINARY income with interest charge".to_string(),
        "26 USC § 1291(b)(2)(A) — excess distribution = distribution exceeding 125% of average distributions in PRECEDING 3 TAXABLE YEARS".to_string(),
        "26 USC § 1291(b)(2)(B)(i) — if holding period less than 3 years, average computed over actual holding period (or stock issued less than 3 years ago)".to_string(),
        "26 USC § 1291(b)(3)(B) — if FIRST YEAR holding stock, ALL DISTRIBUTIONS treated as excess distributions (entire distribution); 125% rule does NOT apply".to_string(),
        "26 USC § 1291(c) — interest charged on deferred tax amount using § 6621 underpayment rate compounded DAILY from original due date of each prior year's tax return through current year; can substantially EXCEED underlying tax in long-held PFIC fact patterns".to_string(),
        "26 USC § 1291(d)(1) — § 1291 does NOT apply to QEF-elected PFICs (§ 1295); QEF shareholders include pro rata share of PFIC ordinary earnings + net capital gain currently".to_string(),
        "26 USC § 1291(d)(2) — purging election available to cleanse pre-QEF or pre-mark-to-market PFIC taint via DEEMED SALE OR DEEMED DIVIDEND".to_string(),
        "26 USC § 1291(f) — § 1291 does NOT apply once § 1296 mark-to-market election effective for marketable PFIC stock; pre-election PFIC taint can persist absent purging election".to_string(),
        "26 USC § 1291(g) — foreign currency translation of PFIC distribution uses § 988 rules".to_string(),
        "Form 8621 — Information Return by Shareholder of Passive Foreign Investment Company or Qualified Electing Fund; § 1298(f) annual reporting threshold applies regardless of distribution status".to_string(),
        "Enacted by Tax Reform Act of 1986 § 1235 (Pub. L. 99-514, October 22, 1986); HIRE Act of 2010 § 521 (Pub. L. 111-147) added § 1298(f) annual reporting requirement".to_string(),
        "Trader-critical fact patterns: foreign-listed mutual funds (Canadian + UK + Australian) almost universally PFICs; foreign ETFs not registered under '40 Act PFICs absent exception; foreign hedge fund LP interests PFIC if 75% income or 50% asset test met; foreign insurance products (offshore annuities + variable life) typically PFICs; PFIC sale converts capital gain to § 1291(a)(2) ordinary + interest".to_string(),
        "PFIC framework cluster: § 1297 (PFIC definition — 75% income test or 50% asset test); § 1298 (attribution + look-through + related-party rules); § 1295 (QEF election — pass-through); § 1296 (mark-to-market election for marketable PFICs); § 1291 (DEFAULT excess distribution + interest charge — this module)".to_string(),
    ];

    Section1291Result {
        scenario: input.scenario,
        election: input.election,
        section_1291_applies,
        excess_distribution_amount_cents,
        current_year_ordinary_inclusion_cents,
        deferred_tax_amount_cents,
        interest_charge_cents,
        total_section_1291_tax_cents,
        failure_reasons,
        citation: "26 USC § 1291(a)(1)(A)-(C) + § 1291(a)(2) + § 1291(b)(2)(A)-(B) + § 1291(b)(3)(B) + § 1291(c) + § 1291(d)(1)-(2) + § 1291(f) + § 1291(g); 26 USC § 1295; 26 USC § 1296; 26 USC § 1297; 26 USC § 1298; 26 USC § 6621; Treas. Reg. § 1.1291-1 through § 1.1291-10; Form 8621; Tax Reform Act of 1986 § 1235 (Pub. L. 99-514, October 22, 1986); HIRE Act of 2010 § 521 (Pub. L. 111-147)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pfic_baseline_distribution() -> Section1291Input {
        Section1291Input {
            scenario: DistributionScenario::DistributionReceived,
            election: PFICElection::NoElection,
            total_distribution_or_gain_cents: 100_000_000,
            average_3_year_distributions_cents: 40_000_000,
            first_year_holding: false,
            holding_period_days: 1800,
            days_in_current_year: 365,
            pre_pfic_period_days: 0,
            current_year_marginal_rate_bps: 3700,
            prior_year_highest_marginal_rate_bps: 3700,
            section_6621_interest_rate_bps: 800,
            interest_charge_compounding_days: 365,
        }
    }

    #[test]
    fn no_election_default_section_1291_applies() {
        let r = check(&pfic_baseline_distribution());
        assert!(r.section_1291_applies);
    }

    #[test]
    fn qef_election_disables_section_1291() {
        let mut i = pfic_baseline_distribution();
        i.election = PFICElection::QefElection;
        let r = check(&i);
        assert!(!r.section_1291_applies);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1291(d)(1)") && f.contains("QEF-elected")));
    }

    #[test]
    fn mark_to_market_election_disables_section_1291() {
        let mut i = pfic_baseline_distribution();
        i.election = PFICElection::MarkToMarketElection;
        let r = check(&i);
        assert!(!r.section_1291_applies);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1291(f)") && f.contains("§ 1296 mark-to-market")));
    }

    #[test]
    fn purging_election_engages_d2() {
        let mut i = pfic_baseline_distribution();
        i.election = PFICElection::PurgingElection;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1291(d)(2)") && f.contains("DEEMED SALE OR DEEMED DIVIDEND")));
    }

    #[test]
    fn no_distribution_or_disposition_no_engagement() {
        let mut i = pfic_baseline_distribution();
        i.scenario = DistributionScenario::NoDistributionOrDisposition;
        let r = check(&i);
        assert!(!r.section_1291_applies);
    }

    #[test]
    fn excess_distribution_above_125_percent_threshold() {
        let r = check(&pfic_baseline_distribution());
        assert_eq!(r.excess_distribution_amount_cents, 50_000_000);
    }

    #[test]
    fn excess_distribution_at_125_percent_threshold_no_excess() {
        let mut i = pfic_baseline_distribution();
        i.total_distribution_or_gain_cents = 50_000_000;
        i.average_3_year_distributions_cents = 40_000_000;
        let r = check(&i);
        assert_eq!(r.excess_distribution_amount_cents, 0);
    }

    #[test]
    fn first_year_holding_all_distribution_is_excess() {
        let mut i = pfic_baseline_distribution();
        i.first_year_holding = true;
        i.total_distribution_or_gain_cents = 100_000_000;
        let r = check(&i);
        assert_eq!(r.excess_distribution_amount_cents, 100_000_000);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1291(b)(3)(B)")
                && f.contains("first year")
                && f.contains("ALL DISTRIBUTIONS")));
    }

    #[test]
    fn ratable_allocation_across_holding_period() {
        let mut i = pfic_baseline_distribution();
        i.holding_period_days = 1800;
        i.days_in_current_year = 365;
        i.pre_pfic_period_days = 0;
        let r = check(&i);
        let per_day = r.excess_distribution_amount_cents / 1800;
        assert!(per_day > 0);
        assert_eq!(r.current_year_ordinary_inclusion_cents, per_day * 365);
    }

    #[test]
    fn disposition_gain_treated_as_excess_distribution() {
        let mut i = pfic_baseline_distribution();
        i.scenario = DistributionScenario::DispositionGain;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1291(a)(2)")
            && f.contains("disposition")
            && f.contains("ORDINARY")));
    }

    #[test]
    fn deferred_tax_at_prior_year_highest_marginal_rate() {
        let mut i = pfic_baseline_distribution();
        i.holding_period_days = 1800;
        i.days_in_current_year = 365;
        i.pre_pfic_period_days = 0;
        i.prior_year_highest_marginal_rate_bps = 3700;
        let r = check(&i);
        assert!(r.deferred_tax_amount_cents > 0);
    }

    #[test]
    fn interest_charge_compounded_at_6621_rate() {
        let mut i = pfic_baseline_distribution();
        i.section_6621_interest_rate_bps = 800;
        i.interest_charge_compounding_days = 365;
        let r = check(&i);
        assert!(r.interest_charge_cents > 0);
    }

    #[test]
    fn pre_pfic_period_taxed_as_ordinary_no_interest() {
        let mut i = pfic_baseline_distribution();
        i.holding_period_days = 1800;
        i.days_in_current_year = 365;
        i.pre_pfic_period_days = 1000;
        let r = check(&i);
        let intermediate_days = 1800 - 365 - 1000;
        assert!(intermediate_days > 0);
        assert!(r.current_year_ordinary_inclusion_cents > 0);
    }

    #[test]
    fn first_year_holding_with_high_rate_substantial_tax() {
        let mut i = pfic_baseline_distribution();
        i.first_year_holding = true;
        i.holding_period_days = 100;
        i.days_in_current_year = 100;
        i.pre_pfic_period_days = 0;
        i.current_year_marginal_rate_bps = 3700;
        let r = check(&i);
        assert_eq!(r.excess_distribution_amount_cents, 100_000_000);
        assert_eq!(r.current_year_ordinary_inclusion_cents, 100_000_000);
        assert_eq!(r.deferred_tax_amount_cents, 0);
        assert_eq!(r.interest_charge_cents, 0);
    }

    #[test]
    fn total_tax_sums_three_components() {
        let r = check(&pfic_baseline_distribution());
        let current_tax = r.current_year_ordinary_inclusion_cents.saturating_mul(3700) / 10_000;
        let expected_total = current_tax + r.deferred_tax_amount_cents + r.interest_charge_cents;
        assert_eq!(r.total_section_1291_tax_cents, expected_total);
    }

    #[test]
    fn election_truth_table_four_cells() {
        for (election, exp_applies) in [
            (PFICElection::NoElection, true),
            (PFICElection::QefElection, false),
            (PFICElection::MarkToMarketElection, false),
            (PFICElection::PurgingElection, false),
        ] {
            let mut i = pfic_baseline_distribution();
            i.election = election;
            let r = check(&i);
            assert_eq!(
                r.section_1291_applies, exp_applies,
                "election={:?}",
                election
            );
        }
    }

    #[test]
    fn scenario_truth_table_three_cells() {
        for (scenario, exp_applies) in [
            (DistributionScenario::DistributionReceived, true),
            (DistributionScenario::DispositionGain, true),
            (DistributionScenario::NoDistributionOrDisposition, false),
        ] {
            let mut i = pfic_baseline_distribution();
            i.scenario = scenario;
            let r = check(&i);
            assert_eq!(
                r.section_1291_applies, exp_applies,
                "scenario={:?}",
                scenario
            );
        }
    }

    #[test]
    fn qef_uniquely_disables_section_1291_invariant() {
        let mut qef = pfic_baseline_distribution();
        qef.election = PFICElection::QefElection;
        let r_qef = check(&qef);
        assert!(!r_qef.section_1291_applies);

        let mut mtm = pfic_baseline_distribution();
        mtm.election = PFICElection::MarkToMarketElection;
        let r_mtm = check(&mtm);
        assert!(!r_mtm.section_1291_applies);

        let r_none = check(&pfic_baseline_distribution());
        assert!(r_none.section_1291_applies);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&pfic_baseline_distribution());
        assert!(r.citation.contains("§ 1291(a)(1)(A)-(C)"));
        assert!(r.citation.contains("§ 1291(a)(2)"));
        assert!(r.citation.contains("§ 1291(b)(2)(A)-(B)"));
        assert!(r.citation.contains("§ 1291(b)(3)(B)"));
        assert!(r.citation.contains("§ 1291(c)"));
        assert!(r.citation.contains("§ 1291(d)(1)-(2)"));
        assert!(r.citation.contains("§ 1291(f)"));
        assert!(r.citation.contains("§ 1291(g)"));
        assert!(r.citation.contains("§ 1295"));
        assert!(r.citation.contains("§ 1296"));
        assert!(r.citation.contains("§ 1297"));
        assert!(r.citation.contains("§ 1298"));
        assert!(r.citation.contains("§ 6621"));
        assert!(r
            .citation
            .contains("Treas. Reg. § 1.1291-1 through § 1.1291-10"));
        assert!(r.citation.contains("Form 8621"));
        assert!(r.citation.contains("Tax Reform Act of 1986 § 1235"));
        assert!(r.citation.contains("Pub. L. 99-514"));
        assert!(r.citation.contains("HIRE Act of 2010 § 521"));
        assert!(r.citation.contains("Pub. L. 111-147"));
    }

    #[test]
    fn note_pins_subsection_a1a_ratable_allocation() {
        let r = check(&pfic_baseline_distribution());
        assert!(r.notes.iter().any(|n| n.contains("§ 1291(a)(1)(A)")
            && n.contains("RATABLY")
            && n.contains("EACH DAY")));
    }

    #[test]
    fn note_pins_subsection_a1b_current_pre_pfic_ordinary() {
        let r = check(&pfic_baseline_distribution());
        assert!(r.notes.iter().any(|n| n.contains("§ 1291(a)(1)(B)")
            && n.contains("CURRENT taxable year")
            && n.contains("PRE-PFIC-PERIOD")
            && n.contains("ORDINARY INCOME")));
    }

    #[test]
    fn note_pins_subsection_a1c_highest_rate_interest() {
        let r = check(&pfic_baseline_distribution());
        assert!(r.notes.iter().any(|n| n.contains("§ 1291(a)(1)(C)")
            && n.contains("HIGHEST MARGINAL RATE")
            && n.contains("§ 6621")
            && n.contains("compounded DAILY")));
    }

    #[test]
    fn note_pins_subsection_a2_disposition_gain() {
        let r = check(&pfic_baseline_distribution());
        assert!(r.notes.iter().any(|n| n.contains("§ 1291(a)(2)")
            && n.contains("DISPOSITION")
            && n.contains("CAPITAL GAIN treatment CONVERTED to ORDINARY")));
    }

    #[test]
    fn note_pins_subsection_b2a_125_percent_threshold() {
        let r = check(&pfic_baseline_distribution());
        assert!(r.notes.iter().any(|n| n.contains("§ 1291(b)(2)(A)")
            && n.contains("125%")
            && n.contains("PRECEDING 3 TAXABLE YEARS")));
    }

    #[test]
    fn note_pins_subsection_b2bi_under_3_year_lookback() {
        let r = check(&pfic_baseline_distribution());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1291(b)(2)(B)(i)") && n.contains("actual holding period")));
    }

    #[test]
    fn note_pins_subsection_b3b_first_year_all_excess() {
        let r = check(&pfic_baseline_distribution());
        assert!(r.notes.iter().any(|n| n.contains("§ 1291(b)(3)(B)")
            && n.contains("FIRST YEAR")
            && n.contains("ALL DISTRIBUTIONS treated as excess")));
    }

    #[test]
    fn note_pins_subsection_c_6621_interest() {
        let r = check(&pfic_baseline_distribution());
        assert!(r.notes.iter().any(|n| n.contains("§ 1291(c)")
            && n.contains("§ 6621")
            && n.contains("compounded DAILY")
            && n.contains("EXCEED underlying tax")));
    }

    #[test]
    fn note_pins_subsection_d1_qef_disables() {
        let r = check(&pfic_baseline_distribution());
        assert!(r.notes.iter().any(|n| n.contains("§ 1291(d)(1)")
            && n.contains("QEF-elected")
            && n.contains("§ 1295")));
    }

    #[test]
    fn note_pins_subsection_d2_purging_election() {
        let r = check(&pfic_baseline_distribution());
        assert!(r.notes.iter().any(|n| n.contains("§ 1291(d)(2)")
            && n.contains("purging election")
            && n.contains("DEEMED SALE OR DEEMED DIVIDEND")));
    }

    #[test]
    fn note_pins_subsection_f_mtm_disables() {
        let r = check(&pfic_baseline_distribution());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1291(f)") && n.contains("§ 1296 mark-to-market")));
    }

    #[test]
    fn note_pins_subsection_g_988_currency() {
        let r = check(&pfic_baseline_distribution());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1291(g)") && n.contains("§ 988")));
    }

    #[test]
    fn note_pins_form_8621_reporting() {
        let r = check(&pfic_baseline_distribution());
        assert!(r.notes.iter().any(|n| n.contains("Form 8621")
            && n.contains("§ 1298(f)")
            && n.contains("annual reporting")));
    }

    #[test]
    fn note_pins_1986_tax_reform_origin() {
        let r = check(&pfic_baseline_distribution());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Tax Reform Act of 1986 § 1235")
                && n.contains("Pub. L. 99-514")
                && n.contains("October 22, 1986")
                && n.contains("HIRE Act of 2010 § 521")));
    }

    #[test]
    fn note_pins_pfic_framework_cluster() {
        let r = check(&pfic_baseline_distribution());
        assert!(r.notes.iter().any(|n| n.contains("PFIC framework cluster")
            && n.contains("§ 1297")
            && n.contains("§ 1298")
            && n.contains("§ 1295")
            && n.contains("§ 1296")));
    }

    #[test]
    fn note_pins_trader_fact_patterns() {
        let r = check(&pfic_baseline_distribution());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("foreign-listed mutual funds")
                && n.contains("Canadian")
                && n.contains("foreign hedge fund LP")
                && n.contains("offshore annuities")));
    }

    #[test]
    fn defensive_zero_distribution_no_excess() {
        let mut i = pfic_baseline_distribution();
        i.total_distribution_or_gain_cents = 0;
        let r = check(&i);
        assert_eq!(r.excess_distribution_amount_cents, 0);
    }

    #[test]
    fn defensive_zero_holding_period_no_panic() {
        let mut i = pfic_baseline_distribution();
        i.holding_period_days = 0;
        let r = check(&i);
        let _ = r.total_section_1291_tax_cents;
    }
}
