//! IRC § 1293 — Current taxation of income from qualified
//! electing funds. The CURRENT-TAXATION MECHANIC that
//! engages when a U.S. shareholder of a PFIC has made a
//! valid § 1295 QEF election. Completes the PFIC framework
//! cluster: section_1291 (default excess distribution +
//! interest charge); section_1295 (QEF election
//! mechanism); section_1296 (mark-to-market alternative);
//! section_1297 (PFIC definition — 75% income or 50% asset
//! test); section_1298 (special rules — attribution,
//! look-through, related-party).
//!
//! Companion to section_6038d (Form 8938 FATCA captures
//! QEF interests), section_988 (foreign currency
//! translation), section_67g (TCJA misc itemized
//! suspension affects QEF expenses).
//!
//! Trader-critical because QEF election is the preferred
//! escape from § 1291 punitive regime — converts what
//! would be ORDINARY income + interest charge into
//! current-year pass-through inclusion that PRESERVES
//! CHARACTER (long-term capital gain stays LTCG; qualified
//! dividend treatment may apply per § 1(h)(11)).
//!
//! **§ 1293(a)(1) Pro rata inclusion** — shareholder of
//! qualified electing fund includes in gross income for
//! the taxable year:
//! 1. **§ 1293(a)(1)(A)** — as ORDINARY INCOME,
//!    shareholder's pro rata share of the fund's
//!    ORDINARY EARNINGS for the year; AND
//! 2. **§ 1293(a)(1)(B)** — as LONG-TERM CAPITAL GAIN,
//!    shareholder's pro rata share of the fund's NET
//!    CAPITAL GAIN for the year.
//!
//! Character preservation — LTCG character preserved
//! (capital gain remains capital gain regardless of
//! shareholder's holding period in the QEF stock).
//!
//! **§ 1293(b) Definitions**:
//! 1. § 1293(b)(1) — "ORDINARY EARNINGS" = EXCESS of fund's
//!    earnings and profits over net capital gain.
//! 2. § 1293(b)(2) — "NET CAPITAL GAIN" has meaning given
//!    by § 1222(11) (excess of net long-term capital gain
//!    over net short-term capital loss).
//!
//! **§ 1293(c) Pro rata share definition** — pro rata
//! share = amount that WOULD HAVE BEEN DISTRIBUTED with
//! respect to shareholder's stock if, on each day during
//! the taxable year of the fund, the fund had distributed
//! to each shareholder a pro rata share of that day's
//! ratable share of the fund's ordinary earnings and net
//! capital gain for such year.
//!
//! **§ 1293(d) Basis adjustments**:
//! 1. § 1293(d)(1) — Stock basis INCREASED by any amount
//!    included in shareholder's income under § 1293(a).
//! 2. § 1293(d)(2) — Stock basis DECREASED by any amount
//!    distributed with respect to stock which is NOT
//!    INCLUDIBLE in shareholder's income (previously
//!    taxed income PTI distribution).
//!
//! Basis-tracking mechanic prevents DOUBLE TAXATION of
//! the same earnings — PTI distribution reduces basis
//! rather than re-creating income.
//!
//! **§ 1293(e) Coordination with subpart F** — to the
//! extent earnings already taxed under § 951 (subpart F
//! controlled foreign corporation rules), § 1293 does
//! NOT impose additional inclusion; PFIC and CFC regimes
//! generally do not overlap due to § 1297(d) PFIC-CFC
//! overlap rule.
//!
//! **§ 1293(f) Coordination with § 1294 deferral** —
//! shareholder may elect under § 1294 to defer payment
//! of tax on QEF inclusion (with interest charge) when
//! fund has not made distribution sufficient to pay the
//! tax. § 1294 deferral is rarely used due to interest
//! charge accrual.
//!
//! **Form 8621** — Information Return by Shareholder of
//! a Passive Foreign Investment Company or Qualified
//! Electing Fund. QEF election + § 1293 inclusion
//! reported annually; required attachment to § 1298(f)
//! annual disclosure (added by HIRE Act of 2010 § 521).
//!
//! Character preservation note — § 1293(a)(1)(B) LTCG
//! treatment requires the QEF election + fund's
//! disclosure of ordinary-vs-capital-gain split via
//! PFIC Annual Information Statement (or alternative
//! information statement) per Treas. Reg. § 1.1295-1(g).
//!
//! Citations: 26 USC § 1293(a)(1)(A)-(B) and § 1293(b)(1)-(2)
//! and § 1293(c) and § 1293(d)(1)-(2) and § 1293(e) and
//! § 1293(f); 26 USC § 1222(11) and § 1294 and § 1295 and
//! § 1297(d) and § 951 and § 1(h)(11); Treas. Reg.
//! § 1.1293-1 through § 1.1293-3 and Treas. Reg.
//! § 1.1295-1(g); Form 8621; Tax Reform Act of 1986 § 1235
//! (Pub. L. 99-514, October 22, 1986); HIRE Act of 2010
//! § 521 (Pub. L. 111-147).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section1293Input {
    /// Fund's total ordinary earnings (E&P minus net
    /// capital gain) in cents.
    pub fund_ordinary_earnings_cents: u64,
    /// Fund's total net capital gain (§ 1222(11)
    /// definition — long-term net gain minus short-term
    /// net loss) in cents.
    pub fund_net_capital_gain_cents: u64,
    /// Shareholder's pro rata share percentage in basis
    /// points (e.g., 100 = 1%, 500 = 5%).
    pub pro_rata_share_bps: u32,
    /// Shareholder's basis in QEF stock at start of year
    /// in cents (for § 1293(d) adjustment tracking).
    pub starting_basis_cents: u64,
    /// Cash distributed by fund to shareholder during
    /// year in cents (used to determine PTI distribution
    /// vs ordinary distribution).
    pub distribution_received_cents: u64,
    /// Whether § 951 subpart F overlap engages (CFC
    /// shareholder of a CFC that is also PFIC; § 1297(d)
    /// generally prevents).
    pub subpart_f_overlap_engaged: bool,
    /// Whether § 1294 deferral election made (rare).
    pub section_1294_deferral_elected: bool,
    /// Whether PFIC Annual Information Statement (or
    /// alternative information statement) per Treas. Reg.
    /// § 1.1295-1(g) provided by fund (required for QEF
    /// election validity + ordinary/LTCG split).
    pub pfic_annual_information_statement_provided: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1293Result {
    pub ordinary_income_inclusion_cents: u64,
    pub long_term_capital_gain_inclusion_cents: u64,
    pub total_inclusion_cents: u64,
    pub basis_after_inclusion_cents: u64,
    pub pti_distribution_cents: u64,
    pub basis_after_pti_distribution_cents: u64,
    pub character_preserved: bool,
    pub subpart_f_overlap_engaged: bool,
    pub section_1294_deferral_engaged: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section1293Input) -> Section1293Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let pro_rata_factor = input.pro_rata_share_bps as u64;

    let ordinary_income_inclusion_cents =
        input.fund_ordinary_earnings_cents.saturating_mul(pro_rata_factor) / 10_000;
    let long_term_capital_gain_inclusion_cents =
        input.fund_net_capital_gain_cents.saturating_mul(pro_rata_factor) / 10_000;
    let total_inclusion_cents =
        ordinary_income_inclusion_cents.saturating_add(long_term_capital_gain_inclusion_cents);

    let basis_after_inclusion_cents = input
        .starting_basis_cents
        .saturating_add(total_inclusion_cents);

    let pti_distribution_cents = input
        .distribution_received_cents
        .min(total_inclusion_cents);

    let basis_after_pti_distribution_cents =
        basis_after_inclusion_cents.saturating_sub(pti_distribution_cents);

    let character_preserved = input.pfic_annual_information_statement_provided;

    if !input.pfic_annual_information_statement_provided {
        failure_reasons.push(
            "Treas. Reg. § 1.1295-1(g) — PFIC Annual Information Statement (or alternative information statement) is REQUIRED for QEF election validity + ordinary-earnings/net-capital-gain split documentation; absence defeats long-term capital gain character preservation under § 1293(a)(1)(B)".to_string(),
        );
    }

    if input.subpart_f_overlap_engaged {
        failure_reasons.push(
            "26 USC § 1293(e) + § 1297(d) — to extent earnings already taxed under § 951 subpart F controlled foreign corporation rules, § 1293 does NOT impose additional inclusion; PFIC and CFC regimes generally do not overlap per § 1297(d) PFIC-CFC overlap rule".to_string(),
        );
    }

    if input.section_1294_deferral_elected {
        failure_reasons.push(
            "26 USC § 1293(f) + § 1294 — shareholder may elect to defer payment of tax on QEF inclusion (with interest charge) when fund has not made distribution sufficient to pay the tax; § 1294 deferral rarely used due to interest charge accrual".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 1293(a)(1)(A) — shareholder of qualified electing fund includes in gross income as ORDINARY INCOME the shareholder's pro rata share of the fund's ORDINARY EARNINGS for the year".to_string(),
        "26 USC § 1293(a)(1)(B) — shareholder includes in gross income as LONG-TERM CAPITAL GAIN the shareholder's pro rata share of the fund's NET CAPITAL GAIN for the year; CHARACTER PRESERVED — LTCG stays LTCG regardless of shareholder's QEF stock holding period".to_string(),
        "26 USC § 1293(b)(1) — 'ORDINARY EARNINGS' means EXCESS of fund's earnings and profits over net capital gain for the taxable year".to_string(),
        "26 USC § 1293(b)(2) + § 1222(11) — 'NET CAPITAL GAIN' = excess of net long-term capital gain over net short-term capital loss for the taxable year".to_string(),
        "26 USC § 1293(c) — pro rata share = amount that WOULD HAVE BEEN DISTRIBUTED with respect to shareholder's stock if, on each day during the taxable year, the fund had distributed to each shareholder a pro rata share of that day's ratable share of ordinary earnings and net capital gain".to_string(),
        "26 USC § 1293(d)(1) — stock basis INCREASED by any amount included in shareholder's income under § 1293(a); basis-tracking mechanic prevents DOUBLE TAXATION of the same earnings".to_string(),
        "26 USC § 1293(d)(2) — stock basis DECREASED by any amount distributed with respect to stock which is NOT INCLUDIBLE in shareholder's income (previously taxed income PTI distribution)".to_string(),
        "26 USC § 1293(e) + § 1297(d) — to extent earnings already taxed under § 951 subpart F CFC rules, § 1293 does NOT impose additional inclusion; PFIC and CFC regimes do not overlap per § 1297(d) PFIC-CFC overlap rule".to_string(),
        "26 USC § 1293(f) + § 1294 — shareholder may elect to defer payment of tax on QEF inclusion (with interest charge) when fund has not made distribution sufficient to pay the tax; § 1294 deferral RARELY USED due to interest charge accrual".to_string(),
        "Treas. Reg. § 1.1295-1(g) — PFIC Annual Information Statement (or alternative information statement) REQUIRED for QEF election validity + ordinary-earnings/net-capital-gain split documentation; absence defeats § 1293(a)(1)(B) LTCG character preservation".to_string(),
        "26 USC § 1(h)(11) — qualified dividend treatment may apply to QEF ordinary inclusion if fund is qualified foreign corporation under § 1(h)(11)(C); typical QEF treatment, however, is ordinary income at marginal rates".to_string(),
        "Form 8621 — Information Return by Shareholder of PFIC or QEF; QEF election + § 1293 inclusion reported annually; required attachment to § 1298(f) annual disclosure (added by HIRE Act of 2010 § 521)".to_string(),
        "PFIC framework cluster: § 1291 (DEFAULT excess distribution + interest charge — punitive); § 1293 (QEF current-taxation MECHANIC — this module); § 1295 (QEF election mechanism); § 1296 (mark-to-market alternative); § 1297 (PFIC definition — 75% income or 50% asset test); § 1298 (special rules — attribution, look-through, related-party)".to_string(),
        "Enacted by Tax Reform Act of 1986 § 1235 (Pub. L. 99-514, October 22, 1986); HIRE Act of 2010 § 521 (Pub. L. 111-147) added § 1298(f) annual reporting requirement".to_string(),
    ];

    Section1293Result {
        ordinary_income_inclusion_cents,
        long_term_capital_gain_inclusion_cents,
        total_inclusion_cents,
        basis_after_inclusion_cents,
        pti_distribution_cents,
        basis_after_pti_distribution_cents,
        character_preserved,
        subpart_f_overlap_engaged: input.subpart_f_overlap_engaged,
        section_1294_deferral_engaged: input.section_1294_deferral_elected,
        failure_reasons,
        citation: "26 USC § 1293(a)(1)(A)-(B) + § 1293(b)(1)-(2) + § 1293(c) + § 1293(d)(1)-(2) + § 1293(e) + § 1293(f); 26 USC § 1222(11); 26 USC § 1294; 26 USC § 1295; 26 USC § 1296; 26 USC § 1297(d); 26 USC § 951; 26 USC § 1(h)(11); 26 USC § 1298(f); Treas. Reg. § 1.1293-1 through § 1.1293-3; Treas. Reg. § 1.1295-1(g); Form 8621; Tax Reform Act of 1986 § 1235 (Pub. L. 99-514, October 22, 1986); HIRE Act of 2010 § 521 (Pub. L. 111-147)",
        notes,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn baseline_qef() -> Section1293Input {
        Section1293Input {
            fund_ordinary_earnings_cents: 100_000_000,
            fund_net_capital_gain_cents: 50_000_000,
            pro_rata_share_bps: 100,
            starting_basis_cents: 200_000_000,
            distribution_received_cents: 0,
            subpart_f_overlap_engaged: false,
            section_1294_deferral_elected: false,
            pfic_annual_information_statement_provided: true,
        }
    }

    #[test]
    fn pro_rata_share_1_percent_ordinary_and_ltcg_split() {
        let r = check(&baseline_qef());
        assert_eq!(r.ordinary_income_inclusion_cents, 1_000_000);
        assert_eq!(r.long_term_capital_gain_inclusion_cents, 500_000);
        assert_eq!(r.total_inclusion_cents, 1_500_000);
    }

    #[test]
    fn character_preserved_when_statement_provided() {
        let r = check(&baseline_qef());
        assert!(r.character_preserved);
        assert!(r.failure_reasons.is_empty());
    }

    #[test]
    fn missing_information_statement_violation() {
        let mut i = baseline_qef();
        i.pfic_annual_information_statement_provided = false;
        let r = check(&i);
        assert!(!r.character_preserved);
        assert!(r.failure_reasons.iter().any(|f| f.contains("Treas. Reg. § 1.1295-1(g)")
            && f.contains("PFIC Annual Information Statement")
            && f.contains("§ 1293(a)(1)(B)")));
    }

    #[test]
    fn basis_increased_by_inclusion() {
        let r = check(&baseline_qef());
        assert_eq!(r.basis_after_inclusion_cents, 200_000_000 + 1_500_000);
    }

    #[test]
    fn pti_distribution_reduces_basis() {
        let mut i = baseline_qef();
        i.distribution_received_cents = 500_000;
        let r = check(&i);
        assert_eq!(r.pti_distribution_cents, 500_000);
        assert_eq!(
            r.basis_after_pti_distribution_cents,
            200_000_000 + 1_500_000 - 500_000
        );
    }

    #[test]
    fn pti_distribution_capped_at_inclusion() {
        let mut i = baseline_qef();
        i.distribution_received_cents = 3_000_000;
        let r = check(&i);
        assert_eq!(r.pti_distribution_cents, 1_500_000);
    }

    #[test]
    fn zero_ordinary_earnings_no_ordinary_inclusion() {
        let mut i = baseline_qef();
        i.fund_ordinary_earnings_cents = 0;
        let r = check(&i);
        assert_eq!(r.ordinary_income_inclusion_cents, 0);
        assert_eq!(r.long_term_capital_gain_inclusion_cents, 500_000);
    }

    #[test]
    fn zero_net_capital_gain_no_ltcg_inclusion() {
        let mut i = baseline_qef();
        i.fund_net_capital_gain_cents = 0;
        let r = check(&i);
        assert_eq!(r.ordinary_income_inclusion_cents, 1_000_000);
        assert_eq!(r.long_term_capital_gain_inclusion_cents, 0);
    }

    #[test]
    fn pro_rata_5_percent_5x_baseline() {
        let mut i = baseline_qef();
        i.pro_rata_share_bps = 500;
        let r = check(&i);
        assert_eq!(r.ordinary_income_inclusion_cents, 5_000_000);
        assert_eq!(r.long_term_capital_gain_inclusion_cents, 2_500_000);
    }

    #[test]
    fn subpart_f_overlap_note_engages() {
        let mut i = baseline_qef();
        i.subpart_f_overlap_engaged = true;
        let r = check(&i);
        assert!(r.subpart_f_overlap_engaged);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1293(e)")
            && f.contains("§ 1297(d)")
            && f.contains("§ 951 subpart F")));
    }

    #[test]
    fn section_1294_deferral_election_note_engages() {
        let mut i = baseline_qef();
        i.section_1294_deferral_elected = true;
        let r = check(&i);
        assert!(r.section_1294_deferral_engaged);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1293(f)")
            && f.contains("§ 1294")
            && f.contains("interest charge accrual")));
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&baseline_qef());
        assert!(r.citation.contains("§ 1293(a)(1)(A)-(B)"));
        assert!(r.citation.contains("§ 1293(b)(1)-(2)"));
        assert!(r.citation.contains("§ 1293(c)"));
        assert!(r.citation.contains("§ 1293(d)(1)-(2)"));
        assert!(r.citation.contains("§ 1293(e)"));
        assert!(r.citation.contains("§ 1293(f)"));
        assert!(r.citation.contains("§ 1222(11)"));
        assert!(r.citation.contains("§ 1294"));
        assert!(r.citation.contains("§ 1295"));
        assert!(r.citation.contains("§ 1296"));
        assert!(r.citation.contains("§ 1297(d)"));
        assert!(r.citation.contains("§ 951"));
        assert!(r.citation.contains("§ 1(h)(11)"));
        assert!(r.citation.contains("§ 1298(f)"));
        assert!(r.citation.contains("Treas. Reg. § 1.1293-1 through § 1.1293-3"));
        assert!(r.citation.contains("Treas. Reg. § 1.1295-1(g)"));
        assert!(r.citation.contains("Form 8621"));
        assert!(r.citation.contains("Tax Reform Act of 1986 § 1235"));
        assert!(r.citation.contains("Pub. L. 99-514"));
        assert!(r.citation.contains("HIRE Act of 2010 § 521"));
        assert!(r.citation.contains("Pub. L. 111-147"));
    }

    #[test]
    fn note_pins_subsection_a1A_ordinary_income() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("§ 1293(a)(1)(A)")
            && n.contains("ORDINARY INCOME")
            && n.contains("pro rata share")));
    }

    #[test]
    fn note_pins_subsection_a1B_ltcg_character_preserved() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("§ 1293(a)(1)(B)")
            && n.contains("LONG-TERM CAPITAL GAIN")
            && n.contains("CHARACTER PRESERVED")
            && n.contains("regardless of shareholder's QEF stock holding period")));
    }

    #[test]
    fn note_pins_subsection_b1_ordinary_earnings_definition() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("§ 1293(b)(1)")
            && n.contains("ORDINARY EARNINGS")
            && n.contains("EXCESS")
            && n.contains("earnings and profits")));
    }

    #[test]
    fn note_pins_subsection_b2_net_capital_gain_1222_11() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("§ 1293(b)(2)")
            && n.contains("§ 1222(11)")
            && n.contains("net long-term capital gain")
            && n.contains("net short-term capital loss")));
    }

    #[test]
    fn note_pins_subsection_c_pro_rata_share_daily_ratable() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("§ 1293(c)")
            && n.contains("WOULD HAVE BEEN DISTRIBUTED")
            && n.contains("each day during the taxable year")
            && n.contains("ratable share")));
    }

    #[test]
    fn note_pins_subsection_d1_basis_increase() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("§ 1293(d)(1)")
            && n.contains("INCREASED")
            && n.contains("DOUBLE TAXATION")));
    }

    #[test]
    fn note_pins_subsection_d2_basis_decrease_pti() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("§ 1293(d)(2)")
            && n.contains("DECREASED")
            && n.contains("previously taxed income PTI")));
    }

    #[test]
    fn note_pins_subsection_e_subpart_f_coordination() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("§ 1293(e)")
            && n.contains("§ 1297(d)")
            && n.contains("§ 951 subpart F")
            && n.contains("PFIC-CFC overlap")));
    }

    #[test]
    fn note_pins_subsection_f_1294_deferral() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("§ 1293(f)")
            && n.contains("§ 1294")
            && n.contains("interest charge accrual")
            && n.contains("RARELY USED")));
    }

    #[test]
    fn note_pins_1295_1g_information_statement() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("Treas. Reg. § 1.1295-1(g)")
            && n.contains("PFIC Annual Information Statement")
            && n.contains("§ 1293(a)(1)(B) LTCG character preservation")));
    }

    #[test]
    fn note_pins_qualified_dividend_treatment_1h11() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("§ 1(h)(11)")
            && n.contains("qualified dividend treatment")
            && n.contains("qualified foreign corporation")));
    }

    #[test]
    fn note_pins_form_8621_annual_reporting() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("Form 8621")
            && n.contains("§ 1293 inclusion reported annually")
            && n.contains("§ 1298(f)")));
    }

    #[test]
    fn note_pins_pfic_framework_cluster() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("PFIC framework cluster")
            && n.contains("§ 1291")
            && n.contains("§ 1295")
            && n.contains("§ 1296")
            && n.contains("§ 1297")
            && n.contains("§ 1298")));
    }

    #[test]
    fn note_pins_1986_tax_reform_origin() {
        let r = check(&baseline_qef());
        assert!(r.notes.iter().any(|n| n.contains("Tax Reform Act of 1986 § 1235")
            && n.contains("Pub. L. 99-514")
            && n.contains("October 22, 1986")
            && n.contains("HIRE Act of 2010 § 521")));
    }

    #[test]
    fn character_preservation_unique_to_information_statement_invariant() {
        let mut with_stmt = baseline_qef();
        with_stmt.pfic_annual_information_statement_provided = true;
        let r_with = check(&with_stmt);
        assert!(r_with.character_preserved);

        let mut without_stmt = baseline_qef();
        without_stmt.pfic_annual_information_statement_provided = false;
        let r_without = check(&without_stmt);
        assert!(!r_without.character_preserved);
    }

    #[test]
    fn basis_tracking_invariant_increase_then_decrease() {
        let mut i = baseline_qef();
        i.distribution_received_cents = 1_000_000;
        let r = check(&i);
        assert_eq!(
            r.basis_after_pti_distribution_cents,
            i.starting_basis_cents + r.total_inclusion_cents - r.pti_distribution_cents
        );
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = baseline_qef();
        i.fund_ordinary_earnings_cents = u64::MAX;
        i.pro_rata_share_bps = 10_000;
        let r = check(&i);
        let _ = r.total_inclusion_cents;
    }

    #[test]
    fn ltcg_character_preserved_regardless_of_holding_period() {
        let r = check(&baseline_qef());
        assert_eq!(r.long_term_capital_gain_inclusion_cents, 500_000);
        assert!(r.character_preserved);
    }
}
