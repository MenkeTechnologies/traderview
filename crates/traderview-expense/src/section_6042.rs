//! IRC § 6042 — Returns regarding payments of dividends
//! and corporate earnings and profits (1099-DIV framework).
//! Trader-relevant information-return module because
//! traders receive dividend distributions from common stock,
//! preferred stock, ETFs, mutual funds (RIC), REITs, BDCs,
//! and pass-through master limited partnerships (MLPs).
//! § 6042 governs the 1099-DIV reporting framework.
//! Companion to section_6041 general info returns and
//! section_6045 broker reporting and section_6049 interest
//! payments and section_6050W 1099-K payment card and
//! § 3406 backup withholding and § 1(h)(11) qualified
//! dividends and § 199A REIT dividends and § 1411 NIIT and
//! § 1202 QSBS and § 988 foreign currency.
//!
//! **§ 6042(a)(1) Requirement of reporting** — every person:
//! 1. Who **makes payments of dividends** aggregating **$10
//!    or more** to any person during calendar year; OR
//! 2. Who **receives payments of dividends as a NOMINEE**
//!    aggregating $10 or more during calendar year and who
//!    makes payment to any other person;
//!
//! SHALL make a return setting forth aggregate amount of
//! such payments and name + address of payee.
//!
//! **§ 6042(b) Dividend defined** — for purposes of § 6042
//! (a)(1)(A) the term "dividend" means:
//! 1. Any **distribution by a corporation** which is a
//!    dividend (as defined in § 316) — out of current or
//!    accumulated earnings and profits (E&P);
//! 2. Any payment made by a **stockbroker** to any person
//!    as a substitute for a dividend;
//! 3. **Capital gain distributions** by RIC (mutual funds)
//!    or REIT under § 852/§ 857;
//! 4. **Excluded**: payments subject to backup withholding
//!    under § 3406; exempt-interest dividends from
//!    qualified RIC under § 852(b)(5).
//!
//! **§ 6042(c) Statements to recipient** — every person
//! required to make a return under § 6042(a) shall furnish
//! to each person whose name is required to be set forth
//! on such return a **written statement** showing name +
//! address of payor + aggregate amount of dividend
//! payments. January 31 deadline.
//!
//! **§ 6042(d) Special rules**:
//! 1. **Substitute dividend payments** — payments by
//!    stockbroker for dividends on short sales treated as
//!    dividends for § 6042 purposes;
//! 2. **Uncertain payments rule** — if person unable to
//!    determine portion of payment that is dividend, must
//!    treat ENTIRE AMOUNT as dividend for § 6042(a)
//!    reporting purposes.
//!
//! **Form 1099-DIV box breakdown**:
//! - **Box 1a** Total ordinary dividends
//! - **Box 1b** Qualified dividends per § 1(h)(11) — taxed
//!   at LTCG rates if § 1(h)(11)(B)(iii) **60-day holding
//!   period** satisfied (120 days for preferred stock with
//!   dividend more than 366 days)
//! - **Box 2a** Total capital gain distributions
//! - **Box 2b** Unrecaptured § 1250 gain (depreciation
//!   recapture on real estate at 25% rate)
//! - **Box 2c** § 1202 QSBS gain (50%/75%/100% exclusion)
//! - **Box 2d** Collectibles 28% rate gain
//! - **Box 3** Nondividend distributions (return of
//!   capital — reduces basis)
//! - **Box 4** Federal income tax withheld (§ 3406 backup)
//! - **Box 5** § 199A dividends — REIT/PTP qualified
//!   business income 20% deduction
//! - **Box 6** Section 897 ordinary dividends
//! - **Box 7** Section 897 capital gain
//! - **Box 8** Foreign tax paid (eligible for § 901 foreign
//!   tax credit)
//! - **Box 9** Foreign country/US possession
//! - **Box 10** Cash liquidation distributions
//! - **Box 11** Noncash liquidation distributions
//! - **Box 12** Exempt-interest dividends from RIC
//!   qualified tax-exempt (federal tax-exempt under §
//!   852(b)(5))
//! - **Box 13** Specified private activity bond interest
//!   dividends (AMT preference item)
//!
//! **Trader-critical 1099-DIV provisions**:
//! - **Qualified dividends** (Box 1b) taxed at LTCG rates
//!   (0%/15%/20%) per § 1(h)(11) only if 60-day holding
//!   period satisfied;
//! - **§ 1411 NIIT** 3.8% applies to dividend income for
//!   MAGI over $200K single / $250K MFJ;
//! - **§ 199A 20% deduction** for qualified REIT dividends
//!   (Box 5) — no W-2 wage limit applies to REIT/PTP;
//! - **§ 1202 QSBS** gain pass-through via RIC (Box 2c);
//! - **§ 988 foreign currency** gain/loss on ADR
//!   conversions reported separately;
//! - **§ 901 foreign tax credit** on Box 7-8 foreign tax;
//! - **AMT** preference item for Box 13 specified private
//!   activity bond interest.
//!
//! Citations: 26 USC § 6042(a)-(d); 26 CFR § 1.6042-2 + §
//! 1.6042-3 + § 1.6042-4; Form 1099-DIV (2024 instructions
//! Rev. January 2024); § 316 dividend definition; § 852
//! RIC qualified dividends; § 857 REIT capital gain
//! distributions; § 1(h)(11) qualified dividends; § 199A
//! REIT/PTP dividends; § 1411 NIIT; § 1202 QSBS; § 988
//! foreign currency; § 901 foreign tax credit; § 6041 +
//! § 6045 + § 6049 + § 6050W info return cluster.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DividendType {
    /// Box 1a — total ordinary dividends.
    OrdinaryDividend,
    /// Box 1b — qualified dividends per § 1(h)(11) (LTCG
    /// rates if 60-day holding period satisfied).
    QualifiedDividend,
    /// Box 2a — total capital gain distributions (RIC + REIT
    /// long-term capital gain pass-through).
    CapitalGainDistribution,
    /// Box 2c — § 1202 QSBS gain pass-through.
    Section1202QsbsGain,
    /// Box 3 — nondividend distribution (return of capital).
    NondividendDistribution,
    /// Box 5 — § 199A REIT/PTP dividends (20% QBI deduction
    /// eligible).
    Section199AReitDividend,
    /// Box 12 — exempt-interest dividend from RIC (federal
    /// tax-exempt under § 852(b)(5)).
    ExemptInterestDividend,
    /// Substitute dividend payment by stockbroker on short
    /// sale.
    SubstituteDividendBroker,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6042Input {
    pub dividend_type: DividendType,
    /// Aggregate dividend payments to payee in cents during
    /// calendar year.
    pub aggregate_dividend_cents: u64,
    /// Whether payor is broker or nominee under § 6042(d)(1).
    pub payor_is_nominee: bool,
    /// Whether § 3406 backup withholding was applied.
    pub backup_withholding_applied: bool,
    /// Whether 1099-DIV return was filed with IRS.
    pub return_filed: bool,
    /// Whether written statement furnished to recipient (§
    /// 6042(c) January 31 deadline).
    pub written_statement_furnished: bool,
    /// Whether payor is unable to determine portion of
    /// payment that is dividend (§ 6042(d)(2) uncertain
    /// payments rule).
    pub uncertain_payment_portion: bool,
    /// Whether holding period 60+ days satisfied for
    /// qualified dividend treatment (§ 1(h)(11)(B)(iii)).
    pub sixty_day_holding_period_satisfied: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6042Result {
    pub reporting_required: bool,
    pub ten_dollar_threshold_met: bool,
    pub return_filing_compliant: bool,
    pub statement_to_recipient_compliant: bool,
    pub uncertain_payments_rule_engaged: bool,
    pub qualified_dividend_ltcg_eligible: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6042Input) -> Section6042Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    const TEN_DOLLAR_CENTS: u64 = 1_000;
    let threshold_met = input.aggregate_dividend_cents >= TEN_DOLLAR_CENTS;

    let exempt_interest = matches!(input.dividend_type, DividendType::ExemptInterestDividend);

    let required = (threshold_met || input.backup_withholding_applied) && !exempt_interest;

    if required && !input.return_filed {
        failure_reasons.push(
            "26 USC § 6042(a)(1) + 26 CFR § 1.6042-2 — Form 1099-DIV return required when aggregate dividend payments of $10 or more (or any amount with § 3406 backup withholding) to recipient".to_string(),
        );
    }

    if required && !input.written_statement_furnished {
        failure_reasons.push(
            "26 USC § 6042(c) — written statement showing aggregate dividend amount must be furnished to recipient by January 31 of year following payment".to_string(),
        );
    }

    let uncertain_engaged = input.uncertain_payment_portion;

    let qualified_eligible = matches!(input.dividend_type, DividendType::QualifiedDividend)
        && input.sixty_day_holding_period_satisfied;

    let notes: Vec<String> = vec![
        "26 USC § 6042(a)(1) — every person who makes dividend payments aggregating $10 or more during calendar year, or who receives dividends as nominee aggregating $10+ and pays to another, SHALL make a return setting forth aggregate amount + name and address of recipient".to_string(),
        "26 USC § 6042(b) — dividend defined as (1) corporate distribution under § 316 (out of current or accumulated E&P); (2) stockbroker substitute dividend payment; (3) capital gain distribution by RIC § 852 / REIT § 857; EXCLUDES exempt-interest dividends from RIC § 852(b)(5) and amounts subject to § 3406 backup withholding".to_string(),
        "26 USC § 6042(c) — written statement to recipient required showing name + address of payor + aggregate dividend amount; January 31 deadline of year following payment".to_string(),
        "26 USC § 6042(d)(1) — substitute dividend payments by stockbroker on short sales treated as dividends for § 6042 reporting purposes".to_string(),
        "26 USC § 6042(d)(2) — uncertain payments rule: if person unable to determine portion of payment that is dividend, MUST TREAT ENTIRE AMOUNT as dividend for § 6042(a) reporting".to_string(),
        "Form 1099-DIV Box 1a ordinary dividends; Box 1b qualified dividends (§ 1(h)(11) LTCG rates if 60-day holding period); Box 2a total capital gain distributions; Box 2b unrecaptured § 1250 gain (25%); Box 2c § 1202 QSBS gain (50/75/100% exclusion); Box 2d collectibles 28%; Box 3 return of capital (reduces basis); Box 4 federal tax withheld (§ 3406)".to_string(),
        "Form 1099-DIV Box 5 § 199A REIT/PTP qualified business income dividends (20% deduction eligible); Box 6+7 § 897 ordinary/capital gain (FIRPTA); Box 8+9 foreign tax + country (§ 901 foreign tax credit eligible); Box 10+11 liquidation distributions; Box 12 exempt-interest dividends (§ 852(b)(5) federal tax-exempt); Box 13 specified private activity bond interest (AMT preference)".to_string(),
        "Trader-critical: qualified dividends (Box 1b) taxed at LTCG rates 0%/15%/20% per § 1(h)(11) only if 60-day holding period satisfied (120 days for preferred stock with >366-day dividend); § 1411 NIIT 3.8% applies to dividend income over MAGI thresholds ($200K single/$250K MFJ); § 199A 20% deduction for REIT dividends (Box 5)".to_string(),
        "Trader-critical: § 1202 QSBS gain pass-through via RIC (Box 2c); § 988 foreign currency gain/loss on ADR conversions reported separately; § 901 foreign tax credit on Box 8 foreign tax; AMT preference item for Box 13 specified private activity bond interest".to_string(),
        "Cross-references: § 6041 general info returns; § 6045 broker reporting; § 6049 interest payments (1099-INT/OID); § 6050W 1099-K payment card; § 3406 backup withholding; § 316 dividend; § 852/§ 857 RIC/REIT distributions; § 1(h)(11) qualified dividends; § 199A QBI; § 1411 NIIT; § 1202 QSBS; § 988 foreign currency; § 901 FTC".to_string(),
    ];

    Section6042Result {
        reporting_required: required,
        ten_dollar_threshold_met: threshold_met,
        return_filing_compliant: !required || input.return_filed,
        statement_to_recipient_compliant: !required || input.written_statement_furnished,
        uncertain_payments_rule_engaged: uncertain_engaged,
        qualified_dividend_ltcg_eligible: qualified_eligible,
        failure_reasons,
        citation: "26 USC § 6042(a)-(d); 26 CFR § 1.6042-2 + § 1.6042-3 + § 1.6042-4; Form 1099-DIV (Rev. January 2024); § 316; § 852; § 857; § 1(h)(11); § 199A; § 1411; § 1202; § 988; § 901; § 6041; § 6045; § 6049; § 6050W; § 3406",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_base() -> Section6042Input {
        Section6042Input {
            dividend_type: DividendType::OrdinaryDividend,
            aggregate_dividend_cents: 5_000,
            payor_is_nominee: false,
            backup_withholding_applied: false,
            return_filed: true,
            written_statement_furnished: true,
            uncertain_payment_portion: false,
            sixty_day_holding_period_satisfied: false,
        }
    }

    #[test]
    fn fifty_dollar_ordinary_dividend_filed_compliant() {
        let r = check(&valid_base());
        assert!(r.reporting_required);
        assert!(r.ten_dollar_threshold_met);
    }

    #[test]
    fn ten_dollar_boundary_compliant() {
        let mut i = valid_base();
        i.aggregate_dividend_cents = 1_000;
        let r = check(&i);
        assert!(r.ten_dollar_threshold_met);
        assert!(r.reporting_required);
    }

    #[test]
    fn nine_dollar_below_threshold_no_reporting() {
        let mut i = valid_base();
        i.aggregate_dividend_cents = 999;
        let r = check(&i);
        assert!(!r.ten_dollar_threshold_met);
        assert!(!r.reporting_required);
    }

    #[test]
    fn backup_withholding_overrides_threshold() {
        let mut i = valid_base();
        i.aggregate_dividend_cents = 100;
        i.backup_withholding_applied = true;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn exempt_interest_dividend_no_reporting() {
        let mut i = valid_base();
        i.dividend_type = DividendType::ExemptInterestDividend;
        let r = check(&i);
        assert!(!r.reporting_required);
    }

    #[test]
    fn not_filed_violation() {
        let mut i = valid_base();
        i.return_filed = false;
        let r = check(&i);
        assert!(!r.return_filing_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6042(a)(1)") && f.contains("1099-DIV")));
    }

    #[test]
    fn no_statement_to_recipient_violation() {
        let mut i = valid_base();
        i.written_statement_furnished = false;
        let r = check(&i);
        assert!(!r.statement_to_recipient_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6042(c)") && f.contains("January 31")));
    }

    #[test]
    fn uncertain_payments_rule_engaged() {
        let mut i = valid_base();
        i.uncertain_payment_portion = true;
        let r = check(&i);
        assert!(r.uncertain_payments_rule_engaged);
    }

    #[test]
    fn qualified_dividend_with_60_day_holding_ltcg_eligible() {
        let mut i = valid_base();
        i.dividend_type = DividendType::QualifiedDividend;
        i.sixty_day_holding_period_satisfied = true;
        let r = check(&i);
        assert!(r.qualified_dividend_ltcg_eligible);
    }

    #[test]
    fn qualified_dividend_without_60_day_holding_not_ltcg() {
        let mut i = valid_base();
        i.dividend_type = DividendType::QualifiedDividend;
        i.sixty_day_holding_period_satisfied = false;
        let r = check(&i);
        assert!(!r.qualified_dividend_ltcg_eligible);
    }

    #[test]
    fn capital_gain_distribution_in_scope() {
        let mut i = valid_base();
        i.dividend_type = DividendType::CapitalGainDistribution;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn section_1202_qsbs_gain_in_scope() {
        let mut i = valid_base();
        i.dividend_type = DividendType::Section1202QsbsGain;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn nondividend_distribution_in_scope() {
        let mut i = valid_base();
        i.dividend_type = DividendType::NondividendDistribution;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn section_199a_reit_dividend_in_scope() {
        let mut i = valid_base();
        i.dividend_type = DividendType::Section199AReitDividend;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn substitute_dividend_broker_in_scope() {
        let mut i = valid_base();
        i.dividend_type = DividendType::SubstituteDividendBroker;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 6042(a)-(d)"));
        assert!(r.citation.contains("§ 1.6042-2"));
        assert!(r.citation.contains("Form 1099-DIV"));
        assert!(r.citation.contains("§ 316"));
        assert!(r.citation.contains("§ 852"));
        assert!(r.citation.contains("§ 857"));
        assert!(r.citation.contains("§ 1(h)(11)"));
        assert!(r.citation.contains("§ 199A"));
        assert!(r.citation.contains("§ 1411"));
        assert!(r.citation.contains("§ 1202"));
        assert!(r.citation.contains("§ 988"));
        assert!(r.citation.contains("§ 901"));
    }

    #[test]
    fn note_pins_subsection_a_10_dollar_minimum() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6042(a)(1)")
            && n.contains("$10 or more")
            && n.contains("name and address")));
    }

    #[test]
    fn note_pins_subsection_b_dividend_definition() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6042(b)")
            && n.contains("§ 316")
            && n.contains("RIC")
            && n.contains("REIT")
            && n.contains("EXCLUDES exempt-interest")));
    }

    #[test]
    fn note_pins_subsection_c_january_31_statement() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6042(c)") && n.contains("January 31")));
    }

    #[test]
    fn note_pins_subsection_d1_substitute_dividend() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6042(d)(1)")
            && n.contains("substitute dividend")
            && n.contains("short sales")));
    }

    #[test]
    fn note_pins_subsection_d2_uncertain_payments_entire_amount() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6042(d)(2)") && n.contains("ENTIRE AMOUNT")));
    }

    #[test]
    fn note_pins_1099_div_first_box_set() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Box 1a")
            && n.contains("Box 1b")
            && n.contains("§ 1(h)(11)")
            && n.contains("60-day holding period")
            && n.contains("Box 2a")
            && n.contains("Box 2c")
            && n.contains("§ 1202 QSBS")));
    }

    #[test]
    fn note_pins_1099_div_second_box_set() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Box 5")
            && n.contains("§ 199A")
            && n.contains("§ 897")
            && n.contains("§ 901")
            && n.contains("Box 12")
            && n.contains("§ 852(b)(5)")
            && n.contains("Box 13")
            && n.contains("AMT")));
    }

    #[test]
    fn note_pins_trader_critical_qualified_dividend_60_day() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("qualified dividends")
            && n.contains("60-day holding period")
            && n.contains("§ 1411 NIIT")
            && n.contains("3.8%")
            && n.contains("§ 199A")));
    }

    #[test]
    fn note_pins_trader_critical_qsbs_currency_ftc() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 1202 QSBS")
            && n.contains("§ 988")
            && n.contains("§ 901 foreign tax credit")
            && n.contains("AMT preference")));
    }

    #[test]
    fn note_pins_cross_references() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6041")
            && n.contains("§ 6045")
            && n.contains("§ 6049")
            && n.contains("§ 6050W")
            && n.contains("§ 3406")));
    }

    #[test]
    fn dividend_type_truth_table_eight_cells() {
        for (dividend_type, exp_reportable) in [
            (DividendType::OrdinaryDividend, true),
            (DividendType::QualifiedDividend, true),
            (DividendType::CapitalGainDistribution, true),
            (DividendType::Section1202QsbsGain, true),
            (DividendType::NondividendDistribution, true),
            (DividendType::Section199AReitDividend, true),
            (DividendType::ExemptInterestDividend, false),
            (DividendType::SubstituteDividendBroker, true),
        ] {
            let mut i = valid_base();
            i.dividend_type = dividend_type;
            let r = check(&i);
            assert_eq!(
                r.reporting_required, exp_reportable,
                "dividend_type={:?} expected reportable={}",
                dividend_type, exp_reportable
            );
        }
    }

    #[test]
    fn ten_dollar_threshold_boundary_invariant() {
        let mut i_at = valid_base();
        i_at.aggregate_dividend_cents = 1_000;
        let r_at = check(&i_at);
        assert!(r_at.ten_dollar_threshold_met);

        let mut i_under = valid_base();
        i_under.aggregate_dividend_cents = 999;
        let r_under = check(&i_under);
        assert!(!r_under.ten_dollar_threshold_met);
    }

    #[test]
    fn backup_withholding_overrides_threshold_invariant() {
        let mut i_below = valid_base();
        i_below.aggregate_dividend_cents = 500;
        i_below.backup_withholding_applied = false;
        let r_below = check(&i_below);
        assert!(!r_below.reporting_required);

        let mut i_backup = valid_base();
        i_backup.aggregate_dividend_cents = 500;
        i_backup.backup_withholding_applied = true;
        let r_backup = check(&i_backup);
        assert!(r_backup.reporting_required);
    }

    #[test]
    fn exempt_interest_dividend_uniquely_excluded_invariant() {
        for dividend_type in [
            DividendType::OrdinaryDividend,
            DividendType::QualifiedDividend,
            DividendType::CapitalGainDistribution,
            DividendType::Section1202QsbsGain,
            DividendType::NondividendDistribution,
            DividendType::Section199AReitDividend,
            DividendType::SubstituteDividendBroker,
        ] {
            let mut i = valid_base();
            i.dividend_type = dividend_type;
            let r = check(&i);
            assert!(r.reporting_required);
        }

        let mut i_exempt = valid_base();
        i_exempt.dividend_type = DividendType::ExemptInterestDividend;
        let r_exempt = check(&i_exempt);
        assert!(!r_exempt.reporting_required);
    }

    #[test]
    fn qualified_dividend_ltcg_requires_60_day_holding_invariant() {
        let mut i_satisfied = valid_base();
        i_satisfied.dividend_type = DividendType::QualifiedDividend;
        i_satisfied.sixty_day_holding_period_satisfied = true;
        let r_satisfied = check(&i_satisfied);
        assert!(r_satisfied.qualified_dividend_ltcg_eligible);

        let mut i_not_satisfied = valid_base();
        i_not_satisfied.dividend_type = DividendType::QualifiedDividend;
        i_not_satisfied.sixty_day_holding_period_satisfied = false;
        let r_not_satisfied = check(&i_not_satisfied);
        assert!(!r_not_satisfied.qualified_dividend_ltcg_eligible);
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = valid_base();
        i.return_filed = false;
        i.written_statement_furnished = false;
        let r = check(&i);
        assert_eq!(r.failure_reasons.len(), 2);
    }

    #[test]
    fn defensive_zero_dividend_no_reporting() {
        let mut i = valid_base();
        i.aggregate_dividend_cents = 0;
        let r = check(&i);
        assert!(!r.reporting_required);
    }
}
