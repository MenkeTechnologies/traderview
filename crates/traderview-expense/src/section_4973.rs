//! IRC § 4973 — Tax on excess contributions to
//! certain tax-favored accounts and annuities. Direct
//! trader companion to section_408 (traditional IRA),
//! section_408a (Roth IRA — iter 430), section_4974
//! (RMD excise — iter 436), section_4975 (prohibited
//! transactions — iter 434), section_72t (10% early-
//! withdrawal penalty).
//!
//! § 4973(a) imposes a 6% EXCISE TAX on excess
//! contributions to:
//! 1. § 408(a) traditional IRAs;
//! 2. § 408A Roth IRAs;
//! 3. § 408(b) individual retirement annuities;
//! 4. § 408(p) SIMPLE IRAs;
//! 5. § 530 Coverdell Education Savings Accounts;
//! 6. § 220 Archer MSAs;
//! 7. § 223 Health Savings Accounts (HSAs).
//!
//! The 6% tax is an ANNUAL, NON-DEDUCTIBLE levy that
//! COMPOUNDS each year an uncorrected excess remains
//! in the account.
//!
//! Trader-critical because traders routinely make
//! contributions across multiple accounts and may
//! exceed limits when:
//! - Earnings increase mid-year above MAGI phase-out
//!   threshold (Roth IRA — § 408A(c)(3));
//! - Multiple employers each provide partial 401(k)
//!   contributions exceeding § 415(c) limit;
//! - Inherited IRA RMD missed creates excess in
//!   beneficiary's own IRA (§ 401(a)(9)(B));
//! - Spousal IRA contribution claimed without
//!   eligible compensation (§ 219(c));
//! - SEP/SIMPLE plan contributions exceed § 408(p)/
//!   § 408(k) limits.
//!
//! **§ 4973(b) excess contribution defined** —
//! contributions in excess of:
//! 1. § 408(a) traditional IRA: 2026 limit $7,500 +
//!    $1,100 catch-up (age 50+) under § 219(b);
//! 2. § 408A Roth IRA: 2026 limit $7,500 + $1,100
//!    catch-up subject to § 408A(c)(3) MAGI phase-out;
//! 3. § 408(p) SIMPLE IRA: 2026 limit $17,000
//!    (under age 50) / $20,750 (age 50-59 + 64+) /
//!    $22,750 (age 60-63 SECURE 2.0 enhanced
//!    catch-up);
//! 4. § 223 HSA (2026): self-only $4,400; family
//!    $8,800; plus $1,000 catch-up at age 55+;
//! 5. § 530 Coverdell ESA: $2,000 per beneficiary
//!    per year (NOT inflation-adjusted).
//!
//! **§ 4973(b) traditional IRA excess** = (contributions
//! made to traditional IRAs) − (deductible limit under
//! § 219) − (allowable contributions to Roth IRAs to
//! the extent shared limit accommodates).
//!
//! **§ 4973(f) Roth IRA excess** = contributions to
//! Roth IRAs (other than § 408A(e) qualified rollover
//! contributions) over the maximum allowable Roth
//! contribution.
//!
//! **§ 4973(c) correction window** — excess contributions
//! may be REMOVED without 6% excise tax if WITHDRAWN
//! PLUS NET INCOME ATTRIBUTABLE (NIA) before the LATER
//! of:
//! 1. Tax return DUE DATE (April 15 of year after
//!    contribution); OR
//! 2. Due date PLUS EXTENSIONS (October 15).
//!
//! NIA computed under Treas. Reg. § 1.408-11(b)
//! (deemed earnings calculation).
//!
//! **SECURE Act 2.0 § 333 (Pub. L. 117-328, 2022)**
//! eliminated the additional § 72(t) 10% early-
//! withdrawal penalty on the EARNINGS portion when
//! corrective distribution made by due date plus
//! extensions.
//!
//! **SECURE Act 2.0 § 313 — STATUTE OF LIMITATIONS** —
//! § 4973 6% excise tax now has a **6-YEAR STATUTE OF
//! LIMITATIONS** starting on the deadline (excluding
//! extensions) for the account owner's income tax
//! return for the year the excess contribution
//! occurred. Previously NO statute of limitations
//! because Form 5329 was treated as a separate
//! return requiring its own filing.
//!
//! **§ 4973(g) carryover absorption** — uncorrected
//! excess contribution is treated as a contribution
//! in the SUBSEQUENT YEAR (counts against next year's
//! limit) until absorbed. Compounding 6% tax continues
//! while excess remains.
//!
//! Reported on **Form 5329 Part III** (traditional/
//! Roth IRAs), **Part IV** (Coverdell ESAs), **Part V**
//! (Archer MSAs), **Part VI** (HSAs), **Part VII**
//! (ABLE accounts under § 529A — added by SECURE Act
//! 2.0 § 102).
//!
//! **Trader-critical fact patterns**:
//! 1. Trader contributes $7,500 to Roth IRA in 2026
//!    but MAGI exceeds $169,000 (single) → fully
//!    phased out under § 408A(c)(3) → $7,500 excess
//!    → $450/year compounding excise tax;
//! 2. Trader withdraws excess + NIA by October 15
//!    extended deadline → § 4973(c) avoids excise +
//!    § 72(t) eliminated by SECURE 2.0 § 333;
//! 3. Trader fails to correct → 6% × $7,500 = $450
//!    annual excise compounds (and counts toward next
//!    year's limit absorbing it);
//! 4. Trader misses deadline by one day → § 4973(c)
//!    not available → 6% annual excise until removed
//!    (no NIA included; just principal);
//! 5. SECURE 2.0 § 313 statute of limitations —
//!    6-year period from contribution-year return
//!    due date (without extensions) limits IRS
//!    assessment authority.
//!
//! Citations: 26 USC § 4973(a)-(g); 26 USC § 408(a);
//! 26 USC § 408A(c)(3); 26 USC § 219; 26 USC § 408(p);
//! 26 USC § 408(k); 26 USC § 530; 26 USC § 220; 26 USC
//! § 223; 26 USC § 72(t); SECURE Act 2.0 of 2022 § 313
//! (statute of limitations); SECURE Act 2.0 of 2022
//! § 333 (corrective distribution penalty
//! elimination); Pub. L. 117-328 (Consolidated
//! Appropriations Act, 2023); Treas. Reg. § 1.408-11
//! (NIA computation); Treas. Reg. § 54.4973-1; Form
//! 5329 (Additional Taxes on Qualified Plans).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    TraditionalIra408a,
    RothIra408A,
    SimpleIra408p,
    SepIra408k,
    ArcherMsa220,
    Hsa223,
    CoverdellEsa530,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section4973Input {
    pub account_type: AccountType,
    /// Total contributions made for the year in cents.
    pub contributions_cents: u64,
    /// Statutory maximum contribution for the account
    /// type for the year in cents (after any phase-
    /// out).
    pub statutory_max_cents: u64,
    /// Whether excess corrected within § 4973(c)
    /// window (return due date plus extensions).
    pub corrected_within_window: bool,
    /// Net Income Attributable (NIA) on the excess
    /// withdrawn during correction (Treas. Reg.
    /// § 1.408-11(b)).
    pub nia_withdrawn_cents: u64,
    /// Years uncorrected excess has remained in
    /// account.
    pub years_uncorrected: u32,
    /// Whether SECURE 2.0 § 313 6-year statute of
    /// limitations bars IRS assessment.
    pub statute_of_limitations_expired: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section4973Result {
    pub excess_contribution_cents: u64,
    pub annual_excise_tax_cents: u64,
    pub cumulative_excise_tax_cents: u64,
    pub correction_qualifies: bool,
    pub statute_of_limitations_bars_assessment: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section4973Input) -> Section4973Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let excess_contribution_cents = input
        .contributions_cents
        .saturating_sub(input.statutory_max_cents);

    let correction_qualifies = input.corrected_within_window
        && input.nia_withdrawn_cents > 0
        && excess_contribution_cents > 0;

    let statute_of_limitations_bars_assessment = input.statute_of_limitations_expired;

    let annual_excise_tax_cents = if correction_qualifies
        || statute_of_limitations_bars_assessment
        || excess_contribution_cents == 0
    {
        0
    } else {
        excess_contribution_cents.saturating_mul(6) / 100
    };

    let cumulative_excise_tax_cents = annual_excise_tax_cents
        .saturating_mul(input.years_uncorrected.max(1) as u64);

    if excess_contribution_cents == 0 {
        failure_reasons.push(
            "No excess contribution; total contributions within statutory limit; § 4973 inapplicable".to_string(),
        );
    } else if correction_qualifies {
        failure_reasons.push(format!(
            "26 USC § 4973(c) — CORRECTIVE DISTRIBUTION made within window (return due date PLUS EXTENSIONS = October 15 of year after contribution); excess of {} cents plus NIA of {} cents withdrawn; 6% excise tax AVOIDED; SECURE Act 2.0 § 333 (Pub. L. 117-328) eliminated additional § 72(t) 10% penalty on NIA earnings portion",
            excess_contribution_cents, input.nia_withdrawn_cents
        ));
    } else if statute_of_limitations_bars_assessment {
        failure_reasons.push(format!(
            "26 USC § 6501 + SECURE Act 2.0 § 313 — 6-YEAR STATUTE OF LIMITATIONS for § 4973 excise tax has EXPIRED; IRS cannot assess additional tax on excess of {} cents; period runs from deadline (excluding extensions) for account owner's income tax return for contribution year",
            excess_contribution_cents
        ));
    } else {
        failure_reasons.push(format!(
            "26 USC § 4973(a) — 6% EXCISE TAX of {} cents on excess contribution of {} cents; annual non-deductible levy; compounds each year uncorrected; cumulative tax {} cents over {} years",
            annual_excise_tax_cents,
            excess_contribution_cents,
            cumulative_excise_tax_cents,
            input.years_uncorrected.max(1)
        ));
        failure_reasons.push(
            "26 USC § 4973(g) — uncorrected excess treated as CONTRIBUTION IN SUBSEQUENT YEAR (counts against next year's limit) until absorbed; compounding 6% tax continues while excess remains".to_string(),
        );
    }

    let account_label = match input.account_type {
        AccountType::TraditionalIra408a => "§ 408(a) traditional IRA — 2026 limit $7,500 + $1,100 catch-up (age 50+) under § 219(b)(5)",
        AccountType::RothIra408A => "§ 408A Roth IRA — 2026 limit $7,500 + $1,100 catch-up subject to § 408A(c)(3) MAGI phase-out (single $153K-$168K, MFJ $242K-$252K)",
        AccountType::SimpleIra408p => "§ 408(p) SIMPLE IRA — 2026 limit $17,000 (under 50) / $20,750 (50-59 + 64+) / $22,750 (60-63 SECURE 2.0 § 109 enhanced catch-up)",
        AccountType::SepIra408k => "§ 408(k) SEP-IRA — 2026 limit lesser of 25% of compensation or $70,000",
        AccountType::ArcherMsa220 => "§ 220 Archer MSA — generally closed to new contributions post-2007",
        AccountType::Hsa223 => "§ 223 HSA — 2026 self-only $4,400; family $8,800; + $1,000 catch-up at age 55+",
        AccountType::CoverdellEsa530 => "§ 530 Coverdell ESA — $2,000 per beneficiary per year (NOT inflation-adjusted)",
    };
    failure_reasons.push(format!(
        "Account type: {}",
        account_label
    ));

    let notes: Vec<String> = vec![
        "26 USC § 4973(a) — 6% EXCISE TAX on excess contributions to (1) § 408(a) traditional IRA; (2) § 408A Roth IRA; (3) § 408(b) IRA annuity; (4) § 408(p) SIMPLE IRA; (5) § 530 Coverdell ESA; (6) § 220 Archer MSA; (7) § 223 HSA; annual, non-deductible, COMPOUNDS each year uncorrected".to_string(),
        "26 USC § 4973(b) traditional IRA EXCESS = (contributions to traditional IRAs) − (deductible limit under § 219) − (allowable Roth contributions to extent shared limit accommodates)".to_string(),
        "26 USC § 4973(f) Roth IRA EXCESS = contributions to Roth IRAs (other than § 408A(e) qualified rollover contributions) over the maximum allowable Roth contribution".to_string(),
        "26 USC § 4973(c) CORRECTION WINDOW — excess removed without 6% excise tax if WITHDRAWN PLUS NET INCOME ATTRIBUTABLE (NIA) before LATER of (1) tax return DUE DATE (April 15) OR (2) due date PLUS EXTENSIONS (October 15); NIA computed under Treas. Reg. § 1.408-11(b)".to_string(),
        "SECURE Act 2.0 § 333 (Pub. L. 117-328, 2022) — corrective distribution within window AVOIDS § 72(t) 10% early-withdrawal penalty on NIA earnings portion; previously the 10% penalty applied to NIA even on timely correction".to_string(),
        "SECURE Act 2.0 § 313 (Pub. L. 117-328, 2022) — § 4973 6% excise tax now has 6-YEAR STATUTE OF LIMITATIONS starting on deadline (excluding extensions) for account owner's income tax return for contribution year; previously NO statute of limitations (Form 5329 treated as separate return)".to_string(),
        "26 USC § 4973(g) CARRYOVER ABSORPTION — uncorrected excess treated as contribution in SUBSEQUENT YEAR (counts against next year's limit); compounding 6% tax continues while excess remains in account".to_string(),
        "Form 5329 reporting: Part III (traditional/Roth IRAs); Part IV (Coverdell ESAs); Part V (Archer MSAs); Part VI (HSAs); Part VII (ABLE accounts under § 529A — added by SECURE Act 2.0 § 102)".to_string(),
        "Trader-critical fact patterns: (1) $7,500 Roth contribution + MAGI exceeds phase-out → $7,500 excess + $450/year; (2) corrective distribution + NIA by October 15 = § 4973(c) avoids excise + SECURE 2.0 § 333 eliminates § 72(t) 10%; (3) uncorrected compounds + counts toward next year via § 4973(g); (4) miss correction deadline by one day = § 4973(c) not available; (5) 6-year statute under SECURE 2.0 § 313 limits IRS assessment".to_string(),
        "2026 contribution limits (post-SECURE 2.0): (a) Traditional + Roth IRA combined under § 219(b)(5) $7,500 + $1,100 catch-up (50+); (b) SIMPLE IRA § 408(p) $17,000 + $3,750 standard catch-up (50-59 + 64+) + $5,750 SECURE 2.0 § 109 enhanced catch-up (60-63); (c) SEP-IRA § 408(k) lesser of 25% of comp or $70,000; (d) HSA § 223 self-only $4,400 / family $8,800 + $1,000 catch-up at 55+; (e) Coverdell ESA § 530 $2,000 (not indexed)".to_string(),
        "Companion to section_408 (traditional IRA) + section_408a (Roth IRA) + section_4974 (RMD excise tax — iter 436) + section_4975 (prohibited transactions — iter 434) + section_72t (10% early-withdrawal penalty) + section_219 (deduction limit)".to_string(),
    ];

    Section4973Result {
        excess_contribution_cents,
        annual_excise_tax_cents,
        cumulative_excise_tax_cents,
        correction_qualifies,
        statute_of_limitations_bars_assessment,
        failure_reasons,
        citation: "26 USC § 4973(a)-(g); 26 USC § 408(a); 26 USC § 408A(c)(3); 26 USC § 219; 26 USC § 408(p); 26 USC § 408(k); 26 USC § 530; 26 USC § 220; 26 USC § 223; 26 USC § 72(t); 26 USC § 6501 (statute of limitations); SECURE Act 2.0 of 2022 § 313; SECURE Act 2.0 of 2022 § 333; Pub. L. 117-328 (Consolidated Appropriations Act, 2023); Treas. Reg. § 1.408-11 (NIA computation); Treas. Reg. § 54.4973-1; Form 5329 (Additional Taxes on Qualified Plans)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roth_excess_750() -> Section4973Input {
        Section4973Input {
            account_type: AccountType::RothIra408A,
            contributions_cents: 750_000,
            statutory_max_cents: 0,
            corrected_within_window: false,
            nia_withdrawn_cents: 0,
            years_uncorrected: 1,
            statute_of_limitations_expired: false,
        }
    }

    #[test]
    fn roth_full_phase_out_excess_at_six_percent() {
        let r = check(&roth_excess_750());
        assert_eq!(r.excess_contribution_cents, 750_000);
        assert_eq!(r.annual_excise_tax_cents, 45_000);
    }

    #[test]
    fn cumulative_compounds_across_years() {
        let mut i = roth_excess_750();
        i.years_uncorrected = 5;
        let r = check(&i);
        assert_eq!(r.cumulative_excise_tax_cents, 45_000 * 5);
    }

    #[test]
    fn no_excess_zero_tax() {
        let mut i = roth_excess_750();
        i.contributions_cents = 750_000;
        i.statutory_max_cents = 750_000;
        let r = check(&i);
        assert_eq!(r.excess_contribution_cents, 0);
        assert_eq!(r.annual_excise_tax_cents, 0);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("No excess contribution")));
    }

    #[test]
    fn correction_within_window_zeros_excise() {
        let mut i = roth_excess_750();
        i.corrected_within_window = true;
        i.nia_withdrawn_cents = 50_000;
        let r = check(&i);
        assert!(r.correction_qualifies);
        assert_eq!(r.annual_excise_tax_cents, 0);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 4973(c)")
            && f.contains("PLUS EXTENSIONS")
            && f.contains("SECURE Act 2.0 § 333")));
    }

    #[test]
    fn correction_without_nia_still_subject_to_excise() {
        let mut i = roth_excess_750();
        i.corrected_within_window = true;
        i.nia_withdrawn_cents = 0;
        let r = check(&i);
        assert!(!r.correction_qualifies);
        assert_eq!(r.annual_excise_tax_cents, 45_000);
    }

    #[test]
    fn statute_of_limitations_bars_assessment() {
        let mut i = roth_excess_750();
        i.statute_of_limitations_expired = true;
        let r = check(&i);
        assert!(r.statute_of_limitations_bars_assessment);
        assert_eq!(r.annual_excise_tax_cents, 0);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("SECURE Act 2.0 § 313")
            && f.contains("6-YEAR STATUTE OF LIMITATIONS")));
    }

    #[test]
    fn account_type_label_traditional_ira() {
        let mut i = roth_excess_750();
        i.account_type = AccountType::TraditionalIra408a;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("Account type:")
            && f.contains("§ 408(a) traditional IRA")
            && f.contains("§ 219(b)(5)")));
    }

    #[test]
    fn account_type_label_simple_ira_secure_2_enhanced_catch_up() {
        let mut i = roth_excess_750();
        i.account_type = AccountType::SimpleIra408p;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 408(p) SIMPLE IRA")
            && f.contains("SECURE 2.0 § 109")));
    }

    #[test]
    fn account_type_label_hsa_2026_limits() {
        let mut i = roth_excess_750();
        i.account_type = AccountType::Hsa223;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 223 HSA")
            && f.contains("$4,400")
            && f.contains("$8,800")
            && f.contains("age 55+")));
    }

    #[test]
    fn account_type_label_coverdell_2000() {
        let mut i = roth_excess_750();
        i.account_type = AccountType::CoverdellEsa530;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 530 Coverdell ESA")
            && f.contains("$2,000")
            && f.contains("NOT inflation-adjusted")));
    }

    #[test]
    fn account_type_label_sep_ira_70k() {
        let mut i = roth_excess_750();
        i.account_type = AccountType::SepIra408k;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 408(k) SEP-IRA")
            && f.contains("$70,000")));
    }

    #[test]
    fn account_type_label_archer_msa() {
        let mut i = roth_excess_750();
        i.account_type = AccountType::ArcherMsa220;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 220 Archer MSA")
            && f.contains("closed")));
    }

    #[test]
    fn account_type_truth_table_seven_cells() {
        for at in [
            AccountType::TraditionalIra408a,
            AccountType::RothIra408A,
            AccountType::SimpleIra408p,
            AccountType::SepIra408k,
            AccountType::ArcherMsa220,
            AccountType::Hsa223,
            AccountType::CoverdellEsa530,
        ] {
            let mut i = roth_excess_750();
            i.account_type = at;
            let r = check(&i);
            assert_eq!(r.excess_contribution_cents, 750_000, "at={:?}", at);
        }
    }

    #[test]
    fn carryover_absorption_note_present_when_uncorrected() {
        let r = check(&roth_excess_750());
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 4973(g)")
            && f.contains("CONTRIBUTION IN SUBSEQUENT YEAR")));
    }

    #[test]
    fn correction_uniquely_zeros_excise_invariant() {
        let mut not_corrected = roth_excess_750();
        not_corrected.corrected_within_window = false;
        let r_not = check(&not_corrected);
        assert!(r_not.annual_excise_tax_cents > 0);

        let mut corrected = roth_excess_750();
        corrected.corrected_within_window = true;
        corrected.nia_withdrawn_cents = 50_000;
        let r_corr = check(&corrected);
        assert_eq!(r_corr.annual_excise_tax_cents, 0);
    }

    #[test]
    fn statute_uniquely_bars_invariant() {
        let mut barred = roth_excess_750();
        barred.statute_of_limitations_expired = true;
        let r_barred = check(&barred);
        assert_eq!(r_barred.annual_excise_tax_cents, 0);

        let mut not_barred = roth_excess_750();
        not_barred.statute_of_limitations_expired = false;
        let r_not = check(&not_barred);
        assert!(r_not.annual_excise_tax_cents > 0);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&roth_excess_750());
        assert!(r.citation.contains("§ 4973(a)-(g)"));
        assert!(r.citation.contains("§ 408A(c)(3)"));
        assert!(r.citation.contains("§ 408(p)"));
        assert!(r.citation.contains("§ 408(k)"));
        assert!(r.citation.contains("§ 530"));
        assert!(r.citation.contains("§ 220"));
        assert!(r.citation.contains("§ 223"));
        assert!(r.citation.contains("§ 72(t)"));
        assert!(r.citation.contains("§ 6501"));
        assert!(r.citation.contains("SECURE Act 2.0 of 2022 § 313"));
        assert!(r.citation.contains("SECURE Act 2.0 of 2022 § 333"));
        assert!(r.citation.contains("Pub. L. 117-328"));
        assert!(r.citation.contains("Treas. Reg. § 1.408-11"));
        assert!(r.citation.contains("Treas. Reg. § 54.4973-1"));
        assert!(r.citation.contains("Form 5329"));
    }

    #[test]
    fn note_pins_subsection_a_six_percent() {
        let r = check(&roth_excess_750());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 4973(a)")
            && n.contains("6% EXCISE TAX")
            && n.contains("seven")
            || (n.contains("§ 4973(a)")
                && n.contains("6% EXCISE TAX")
                && n.contains("COMPOUNDS"))));
    }

    #[test]
    fn note_pins_subsection_b_traditional_excess() {
        let r = check(&roth_excess_750());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 4973(b) traditional IRA EXCESS")
            && n.contains("§ 219")));
    }

    #[test]
    fn note_pins_subsection_f_roth_excess() {
        let r = check(&roth_excess_750());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 4973(f) Roth IRA EXCESS")
            && n.contains("§ 408A(e) qualified rollover")));
    }

    #[test]
    fn note_pins_subsection_c_correction_window() {
        let r = check(&roth_excess_750());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 4973(c) CORRECTION WINDOW")
            && n.contains("April 15")
            && n.contains("October 15")
            && n.contains("Treas. Reg. § 1.408-11(b)")));
    }

    #[test]
    fn note_pins_secure_2_section_333() {
        let r = check(&roth_excess_750());
        assert!(r.notes.iter().any(|n|
            n.contains("SECURE Act 2.0 § 333")
            && n.contains("Pub. L. 117-328")
            && n.contains("§ 72(t) 10%")));
    }

    #[test]
    fn note_pins_secure_2_section_313_statute_six_years() {
        let r = check(&roth_excess_750());
        assert!(r.notes.iter().any(|n|
            n.contains("SECURE Act 2.0 § 313")
            && n.contains("6-YEAR STATUTE OF LIMITATIONS")
            && n.contains("excluding extensions")));
    }

    #[test]
    fn note_pins_subsection_g_carryover() {
        let r = check(&roth_excess_750());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 4973(g) CARRYOVER ABSORPTION")
            && n.contains("SUBSEQUENT YEAR")));
    }

    #[test]
    fn note_pins_form_5329_seven_parts() {
        let r = check(&roth_excess_750());
        assert!(r.notes.iter().any(|n|
            n.contains("Form 5329 reporting")
            && n.contains("Part III")
            && n.contains("Part IV")
            && n.contains("Part VII")
            && n.contains("§ 529A")
            && n.contains("SECURE Act 2.0 § 102")));
    }

    #[test]
    fn note_pins_2026_limits() {
        let r = check(&roth_excess_750());
        assert!(r.notes.iter().any(|n|
            n.contains("2026 contribution limits")
            && n.contains("$7,500 + $1,100 catch-up")
            && n.contains("SECURE 2.0 § 109 enhanced catch-up")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&roth_excess_750());
        assert!(r.notes.iter().any(|n|
            n.contains("Trader-critical fact patterns")
            && n.contains("$7,500 Roth")
            && n.contains("$450/year")
            && n.contains("6-year statute")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&roth_excess_750());
        assert!(r.notes.iter().any(|n|
            n.contains("Companion to section_408")
            && n.contains("section_4974")
            && n.contains("section_4975")
            && n.contains("section_72t")
            && n.contains("section_219")));
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = roth_excess_750();
        i.contributions_cents = u64::MAX;
        i.years_uncorrected = u32::MAX;
        let r = check(&i);
        let _ = r.cumulative_excise_tax_cents;
    }
}
