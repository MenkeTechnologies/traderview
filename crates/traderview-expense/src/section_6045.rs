//! IRC § 6045 — Returns of brokers (Form 1099-B / Form 1099-DA).
//!
//! § 6045(a) requires any broker — defined as a person who, in the
//! ordinary course of a trade or business, stands ready to effect sales
//! to be made by others — to file information returns reporting customer
//! sales/exchanges. The two forms are Form 1099-B (securities and barter
//! exchange transactions) and Form 1099-DA (digital asset transactions,
//! new for 2025).
//!
//! § 6045(g) bifurcates securities into "covered" (broker REQUIRED to
//! report adjusted basis) and "non-covered" (broker reports gross
//! proceeds only). The acquisition-date cutoffs per Treas. Reg.
//! § 1.6045-1(a)(15) are:
//!
//! Stock acquired in account on or after 2011-01-01 — covered.
//!
//! Mutual funds and dividend-reinvestment plan stock acquired on or
//! after 2012-01-01 — covered.
//!
//! Less complex debt instruments acquired on or after 2014-01-01 —
//! covered.
//!
//! More complex debt instruments acquired on or after 2016-01-01 —
//! covered.
//!
//! Digital assets acquired on or after 2026-01-01 AND held continuously
//! in a broker's account — covered (NEW under IIJA § 80603 amending
//! § 6045 + final Treas. Reg. published 2024).
//!
//! Form 1099-DA is the new IRS form for digital asset broker reporting
//! beginning with transactions on or after 2025-01-01. For 2025 only,
//! brokers report gross proceeds (no basis) because no digital assets
//! qualify as "covered" until the 2026-01-01 cutoff is met. Beginning
//! 2026, basis reporting phases in for digital assets acquired and held
//! continuously after the cutoff.
//!
//! Citations: 26 U.S.C. § 6045; § 6045(a) (broker reporting); § 6045(g)
//! (covered vs non-covered securities + basis); IIJA § 80603 (digital
//! asset broker definition + amendments to § 6045); Treas. Reg.
//! § 1.6045-1(a)(15) (covered-security definition); Treas. Reg.
//! § 1.6045-1(d)(2) (digital asset reporting); IRS Notice 2025-33
//! (transitional relief through 2027 for certain digital asset broker
//! reporting); IRS FAQ on Form 1099-DA.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentType {
    Stock,
    MutualFundOrDrip,
    LessComplexDebt,
    MoreComplexDebt,
    DigitalAsset,
    /// Outside broker reporting (commodity, real estate, non-broker
    /// transaction). Modeled for completeness; § 6045 does not apply.
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6045Input {
    pub instrument_type: InstrumentType,
    pub acquisition_year: u32,
    pub acquisition_month: u32,
    pub acquisition_day: u32,
    pub transaction_year: u32,
    pub proceeds_cents: i64,
    /// Whether the digital asset has been held CONTINUOUSLY in the same
    /// broker's account since acquisition. Required for covered-security
    /// status under the 2026 digital-asset cutoff.
    pub digital_asset_held_continuously_in_broker_account: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FormRequired {
    None,
    Form1099B,
    Form1099Da,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6045Result {
    pub form_required: FormRequired,
    pub covered_security: bool,
    pub basis_reporting_required: bool,
    pub gross_proceeds_reportable_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section6045Input) -> Section6045Result {
    let proceeds = input.proceeds_cents.max(0);

    if input.instrument_type == InstrumentType::Other {
        return Section6045Result {
            form_required: FormRequired::None,
            covered_security: false,
            basis_reporting_required: false,
            gross_proceeds_reportable_cents: 0,
            citation: "26 U.S.C. § 6045 — broker reporting applies only to specified instruments (stock, mutual fund, debt, digital asset)",
            note: "Instrument type Other is outside § 6045 broker reporting scope.".to_string(),
        };
    }

    // Form selection: digital assets → 1099-DA (eff. 2025-01-01);
    // everything else → 1099-B.
    let form = if input.instrument_type == InstrumentType::DigitalAsset {
        if input.transaction_year >= 2025 {
            FormRequired::Form1099Da
        } else {
            // Pre-2025 digital asset transactions had no dedicated form;
            // reporting was inconsistent.
            FormRequired::None
        }
    } else {
        FormRequired::Form1099B
    };

    // Covered-security determination.
    let covered = is_covered(input);
    let basis_required = covered;

    let citation = match (input.instrument_type, covered) {
        (InstrumentType::DigitalAsset, true) => {
            "26 U.S.C. § 6045(g) + Treas. Reg. § 1.6045-1(a)(15)(i)(F) — digital asset acquired on/after 2026-01-01 + held continuously in broker account is COVERED; basis reportable on Form 1099-DA"
        }
        (InstrumentType::DigitalAsset, false) => {
            "26 U.S.C. § 6045(g) + Treas. Reg. § 1.6045-1(d)(2) — pre-2026 digital asset is NON-COVERED; broker reports gross proceeds only on Form 1099-DA"
        }
        (InstrumentType::Stock, true) => {
            "26 U.S.C. § 6045(g) + Treas. Reg. § 1.6045-1(a)(15)(i)(A) — stock acquired on/after 2011-01-01 is COVERED; basis reportable on Form 1099-B"
        }
        (InstrumentType::Stock, false) => {
            "26 U.S.C. § 6045(g) — pre-2011 stock is NON-COVERED; broker reports gross proceeds only on Form 1099-B"
        }
        (InstrumentType::MutualFundOrDrip, true) => {
            "26 U.S.C. § 6045(g) + Treas. Reg. § 1.6045-1(a)(15)(i)(B) — mutual fund / DRIP stock acquired on/after 2012-01-01 is COVERED"
        }
        (InstrumentType::MutualFundOrDrip, false) => {
            "26 U.S.C. § 6045(g) — pre-2012 mutual fund / DRIP stock is NON-COVERED"
        }
        (InstrumentType::LessComplexDebt, true) => {
            "26 U.S.C. § 6045(g) + Treas. Reg. § 1.6045-1(a)(15)(i)(C) — less complex debt instrument acquired on/after 2014-01-01 is COVERED"
        }
        (InstrumentType::LessComplexDebt, false) => {
            "26 U.S.C. § 6045(g) — pre-2014 less-complex debt is NON-COVERED"
        }
        (InstrumentType::MoreComplexDebt, true) => {
            "26 U.S.C. § 6045(g) + Treas. Reg. § 1.6045-1(a)(15)(i)(D) — more complex debt instrument acquired on/after 2016-01-01 is COVERED"
        }
        (InstrumentType::MoreComplexDebt, false) => {
            "26 U.S.C. § 6045(g) — pre-2016 more-complex debt is NON-COVERED"
        }
        (InstrumentType::Other, _) => unreachable!(),
    };

    let note = format!(
        "Instrument type {:?} acquired {}-{:02}-{:02} sold/exchanged in {}. {} reporting {}. Gross proceeds = {} cents.{}",
        input.instrument_type,
        input.acquisition_year,
        input.acquisition_month,
        input.acquisition_day,
        input.transaction_year,
        match form {
            FormRequired::Form1099B => "Form 1099-B",
            FormRequired::Form1099Da => "Form 1099-DA",
            FormRequired::None => "No § 6045 form",
        },
        if covered {
            "COVERED — basis required"
        } else {
            "NON-COVERED — broker reports gross proceeds only"
        },
        proceeds,
        if input.instrument_type == InstrumentType::DigitalAsset && covered && !input.digital_asset_held_continuously_in_broker_account {
            " (continuous-broker-account holding required for digital-asset covered status — flag set by caller)"
        } else {
            ""
        },
    );

    Section6045Result {
        form_required: form,
        covered_security: covered,
        basis_reporting_required: basis_required,
        gross_proceeds_reportable_cents: proceeds,
        citation,
        note,
    }
}

fn is_covered(input: &Section6045Input) -> bool {
    let acq_date = (
        input.acquisition_year,
        input.acquisition_month,
        input.acquisition_day,
    );
    match input.instrument_type {
        InstrumentType::Stock => after_or_on(acq_date, 2011, 1, 1),
        InstrumentType::MutualFundOrDrip => after_or_on(acq_date, 2012, 1, 1),
        InstrumentType::LessComplexDebt => after_or_on(acq_date, 2014, 1, 1),
        InstrumentType::MoreComplexDebt => after_or_on(acq_date, 2016, 1, 1),
        InstrumentType::DigitalAsset => {
            after_or_on(acq_date, 2026, 1, 1)
                && input.digital_asset_held_continuously_in_broker_account
        }
        InstrumentType::Other => false,
    }
}

fn after_or_on(date: (u32, u32, u32), cy: u32, cm: u32, cd: u32) -> bool {
    let (y, m, d) = date;
    match y.cmp(&cy) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => match m.cmp(&cm) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => d >= cd,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        instr: InstrumentType,
        acq_y: u32,
        acq_m: u32,
        acq_d: u32,
        txn_y: u32,
        proceeds: i64,
        continuous_broker: bool,
    ) -> Section6045Input {
        Section6045Input {
            instrument_type: instr,
            acquisition_year: acq_y,
            acquisition_month: acq_m,
            acquisition_day: acq_d,
            transaction_year: txn_y,
            proceeds_cents: proceeds,
            digital_asset_held_continuously_in_broker_account: continuous_broker,
        }
    }

    #[test]
    fn stock_acquired_2015_covered() {
        let r = compute(&input(InstrumentType::Stock, 2015, 6, 1, 2026, 1_000_00, false));
        assert!(r.covered_security);
        assert!(r.basis_reporting_required);
        assert_eq!(r.form_required, FormRequired::Form1099B);
    }

    #[test]
    fn stock_acquired_2010_non_covered() {
        let r = compute(&input(InstrumentType::Stock, 2010, 6, 1, 2026, 1_000_00, false));
        assert!(!r.covered_security);
        assert!(!r.basis_reporting_required);
        assert_eq!(r.form_required, FormRequired::Form1099B);
    }

    #[test]
    fn stock_at_2011_01_01_boundary_covered() {
        let r = compute(&input(InstrumentType::Stock, 2011, 1, 1, 2026, 1_000_00, false));
        assert!(r.covered_security);
    }

    #[test]
    fn stock_at_2010_12_31_boundary_non_covered() {
        let r = compute(&input(
            InstrumentType::Stock,
            2010,
            12,
            31,
            2026,
            1_000_00,
            false,
        ));
        assert!(!r.covered_security);
    }

    #[test]
    fn mutual_fund_2013_covered() {
        let r = compute(&input(
            InstrumentType::MutualFundOrDrip,
            2013,
            1,
            1,
            2026,
            1_000_00,
            false,
        ));
        assert!(r.covered_security);
    }

    #[test]
    fn mutual_fund_2011_non_covered_under_2012_cutoff() {
        let r = compute(&input(
            InstrumentType::MutualFundOrDrip,
            2011,
            12,
            31,
            2026,
            1_000_00,
            false,
        ));
        assert!(!r.covered_security);
    }

    #[test]
    fn less_complex_debt_2015_covered() {
        let r = compute(&input(
            InstrumentType::LessComplexDebt,
            2015,
            1,
            1,
            2026,
            1_000_00,
            false,
        ));
        assert!(r.covered_security);
    }

    #[test]
    fn less_complex_debt_2013_non_covered() {
        let r = compute(&input(
            InstrumentType::LessComplexDebt,
            2013,
            12,
            31,
            2026,
            1_000_00,
            false,
        ));
        assert!(!r.covered_security);
    }

    #[test]
    fn more_complex_debt_2017_covered() {
        let r = compute(&input(
            InstrumentType::MoreComplexDebt,
            2017,
            1,
            1,
            2026,
            1_000_00,
            false,
        ));
        assert!(r.covered_security);
    }

    #[test]
    fn more_complex_debt_2015_non_covered() {
        let r = compute(&input(
            InstrumentType::MoreComplexDebt,
            2015,
            12,
            31,
            2026,
            1_000_00,
            false,
        ));
        assert!(!r.covered_security);
    }

    #[test]
    fn digital_asset_2025_form_1099_da_non_covered() {
        // 2025 digital asset: Form 1099-DA filed but NOT covered (no
        // basis reporting in 2025).
        let r = compute(&input(
            InstrumentType::DigitalAsset,
            2025,
            6,
            1,
            2025,
            1_000_00,
            true,
        ));
        assert_eq!(r.form_required, FormRequired::Form1099Da);
        assert!(!r.covered_security);
    }

    #[test]
    fn digital_asset_2026_acquired_with_continuous_broker_covered() {
        let r = compute(&input(
            InstrumentType::DigitalAsset,
            2026,
            6,
            1,
            2026,
            1_000_00,
            true,
        ));
        assert!(r.covered_security);
        assert_eq!(r.form_required, FormRequired::Form1099Da);
    }

    #[test]
    fn digital_asset_2026_without_continuous_broker_non_covered() {
        // Held outside broker account before 2026 → not covered even if
        // acquisition date qualifies because continuous-broker requirement fails.
        let r = compute(&input(
            InstrumentType::DigitalAsset,
            2026,
            6,
            1,
            2026,
            1_000_00,
            false,
        ));
        assert!(!r.covered_security);
    }

    #[test]
    fn digital_asset_2024_pre_form_1099_da_no_form() {
        // Pre-2025 digital asset: no dedicated form; § 6045 didn't apply
        // for digital asset reporting before 2025.
        let r = compute(&input(
            InstrumentType::DigitalAsset,
            2023,
            6,
            1,
            2024,
            1_000_00,
            true,
        ));
        assert_eq!(r.form_required, FormRequired::None);
    }

    #[test]
    fn other_instrument_outside_scope() {
        let r = compute(&input(
            InstrumentType::Other,
            2024,
            1,
            1,
            2026,
            10_000_00,
            false,
        ));
        assert_eq!(r.form_required, FormRequired::None);
        assert!(!r.covered_security);
    }

    #[test]
    fn no_de_minimis_one_cent_covered_still_reported() {
        // Unlike most 1099 forms, § 6045 has NO de minimis threshold —
        // even one cent triggers reporting for covered securities.
        let r = compute(&input(InstrumentType::Stock, 2020, 1, 1, 2026, 1, false));
        assert_eq!(r.gross_proceeds_reportable_cents, 1);
        assert!(r.covered_security);
    }

    #[test]
    fn negative_proceeds_clamped() {
        let r = compute(&input(InstrumentType::Stock, 2020, 1, 1, 2026, -1, false));
        assert_eq!(r.gross_proceeds_reportable_cents, 0);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let r_stock = compute(&input(InstrumentType::Stock, 2015, 1, 1, 2026, 1000_00, false));
        assert!(r_stock.citation.contains("§ 6045(g)"));
        assert!(r_stock.citation.contains("(a)(15)(i)(A)"));

        let r_da_covered = compute(&input(
            InstrumentType::DigitalAsset,
            2026,
            6,
            1,
            2026,
            1000_00,
            true,
        ));
        assert!(r_da_covered.citation.contains("(a)(15)(i)(F)"));

        let r_da_non = compute(&input(
            InstrumentType::DigitalAsset,
            2024,
            6,
            1,
            2025,
            1000_00,
            true,
        ));
        assert!(r_da_non.citation.contains("(d)(2)"));
    }

    #[test]
    fn cutoff_chronology_invariant() {
        // Acquisition cutoffs in chronological order: 2011 stock < 2012
        // mutual fund < 2014 less complex < 2016 more complex < 2026
        // digital asset.
        // Verify each at-boundary is covered; one day before is not.
        let cases = [
            (InstrumentType::Stock, 2011, 1, 1),
            (InstrumentType::MutualFundOrDrip, 2012, 1, 1),
            (InstrumentType::LessComplexDebt, 2014, 1, 1),
            (InstrumentType::MoreComplexDebt, 2016, 1, 1),
        ];
        for (instr, y, m, d) in cases {
            let at_boundary = compute(&input(instr, y, m, d, 2026, 1000_00, true));
            let just_before = compute(&input(instr, y - 1, 12, 31, 2026, 1000_00, true));
            assert!(at_boundary.covered_security, "{:?} at {} cutoff should be covered", instr, y);
            assert!(!just_before.covered_security, "{:?} one day before {} cutoff should NOT be covered", instr, y);
        }
    }

    #[test]
    fn digital_asset_2026_boundary_and_one_day_before() {
        let at = compute(&input(
            InstrumentType::DigitalAsset,
            2026,
            1,
            1,
            2026,
            1000_00,
            true,
        ));
        let before = compute(&input(
            InstrumentType::DigitalAsset,
            2025,
            12,
            31,
            2026,
            1000_00,
            true,
        ));
        assert!(at.covered_security);
        assert!(!before.covered_security);
    }

    #[test]
    fn digital_asset_form_1099_da_vs_1099_b() {
        // Digital asset → 1099-DA; stock → 1099-B; never crossed.
        let da = compute(&input(
            InstrumentType::DigitalAsset,
            2026,
            6,
            1,
            2026,
            1000_00,
            true,
        ));
        let stock = compute(&input(InstrumentType::Stock, 2020, 1, 1, 2026, 1000_00, false));
        assert_eq!(da.form_required, FormRequired::Form1099Da);
        assert_eq!(stock.form_required, FormRequired::Form1099B);
    }

    #[test]
    fn pre_2025_digital_asset_no_form_required() {
        // Pre-2025 digital asset transactions: no § 6045 form yet (IIJA
        // implementation begins 2025-01-01).
        let r = compute(&input(
            InstrumentType::DigitalAsset,
            2022,
            6,
            1,
            2024,
            1000_00,
            true,
        ));
        assert_eq!(r.form_required, FormRequired::None);
        assert!(!r.covered_security);
    }

    #[test]
    fn year_2025_digital_asset_no_basis_reporting() {
        // 2025: digital asset form 1099-DA but gross proceeds only.
        let r = compute(&input(
            InstrumentType::DigitalAsset,
            2025,
            6,
            1,
            2025,
            1000_00,
            true,
        ));
        assert_eq!(r.form_required, FormRequired::Form1099Da);
        assert!(!r.covered_security);
        assert!(!r.basis_reporting_required);
    }
}
