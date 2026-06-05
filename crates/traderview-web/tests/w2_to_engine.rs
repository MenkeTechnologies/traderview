//! End-to-end: W-2 OCR text → tax_forms::extract → W2 struct →
//! TaxReturn::w2s[] → compute() → expected refund.
//!
//! This integration sits between the two layers that both have unit
//! tests in isolation but no contract-pinning between them. The
//! conversion logic (extract.payload → W2) is duplicated in:
//!   * Frontend wizard JS (`file_taxes.js` upload handler)
//!   * tax_filing_routes::upload_form
//!
//! Both must agree on box-key → W2 field mapping. This test pins the
//! conversion to ground truth.

use rust_decimal::Decimal;
use traderview_ocr::tax_forms::{self, TaxFormKind};
use traderview_tax::{
    compute,
    engine::{ScheduleC, W2},
    FilingStatus, TaxReturn,
};

/// Convert a W-2 TaxFormExtract into a populated `W2` struct. Mirrors
/// the wizard JS conversion at `file_taxes.js` and the backend
/// upload_form path.
fn w2_from_extract(extract: &tax_forms::TaxFormExtract) -> W2 {
    assert_eq!(extract.kind, TaxFormKind::W2, "expected W-2 extract");
    let g = |k: &str| {
        extract.payload.get(k).copied().unwrap_or(Decimal::ZERO)
    };
    W2 {
        employer_name: extract.party_name.clone().unwrap_or_default(),
        box_1_wages: g("box_1"),
        box_2_federal_income_tax_withheld: g("box_2"),
        box_3_ss_wages: g("box_3"),
        box_4_ss_tax_withheld: g("box_4"),
        box_5_medicare_wages: g("box_5"),
        box_6_medicare_tax_withheld: g("box_6"),
        box_17_state_income_tax: g("box_17"),
    }
}

#[test]
fn w2_ocr_text_flows_through_to_correct_refund() {
    // A realistic, slightly noisy W-2 OCR output. Tesseract typically
    // emits label + value on the same line or consecutive lines —
    // both shapes are present below.
    let ocr_text = "\
Form W-2 Wage and Tax Statement
Employer's name, address, and ZIP
ACME Industries Inc.
123 Industrial Pkwy
Wages, tips, other compensation 65000.00
Federal income tax withheld 7800.00
Social security wages 65000.00
Social security tax withheld 4030.00
Medicare wages and tips 65000.00
Medicare tax withheld 942.50
";

    let extract = tax_forms::extract(ocr_text).expect("must detect W-2");
    let w2 = w2_from_extract(&extract);

    // Sanity — conversion preserved the values.
    assert_eq!(w2.box_1_wages, Decimal::from(65_000));
    assert_eq!(w2.box_2_federal_income_tax_withheld, Decimal::from(7_800));
    assert_eq!(w2.employer_name, "ACME Industries Inc.");

    // Plug into a single-filer TaxReturn — should produce a specific refund.
    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Single,
        w2s: vec![w2],
        ..Default::default()
    };
    let res = compute(&r);

    // Hand math:
    //   AGI = 65,000
    //   std deduction = 15,000
    //   taxable = 50,000
    //   bracket tax:
    //     11,925 @ 10% =  1,192.50
    //     36,550 @ 12% =  4,386.00
    //      1,525 @ 22% =    335.50
    //                  =  5,914.00
    //   withholding = 7,800 → refund 1,886.00
    assert_eq!(res.agi, Decimal::from(65_000));
    assert_eq!(res.taxable_income, Decimal::from(50_000));
    assert_eq!(res.ordinary_tax, "5914".parse::<Decimal>().unwrap());
    assert_eq!(res.refund_due, Decimal::from(1_886));
    assert_eq!(res.tax_owed, Decimal::ZERO);
}

#[test]
fn multiple_w2s_from_ocr_aggregate_correctly() {
    // Person works two jobs. Both W-2s OCR'd, both add to total wages
    // and withholding. SS-cap interaction kicks in across the
    // combined wages.
    let w2a_text = "\
Form W-2 Wage and Tax Statement
Employer's name
Job One Corp
Wages, tips, other compensation 50000.00
Federal income tax withheld 5500.00
Social security wages 50000.00
";
    let w2b_text = "\
Form W-2 Wage and Tax Statement
Employer's name
Job Two LLC
Wages, tips, other compensation 40000.00
Federal income tax withheld 4400.00
Social security wages 40000.00
";

    let ea = tax_forms::extract(w2a_text).expect("w2a");
    let eb = tax_forms::extract(w2b_text).expect("w2b");

    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Single,
        w2s: vec![w2_from_extract(&ea), w2_from_extract(&eb)],
        ..Default::default()
    };
    let res = compute(&r);

    // Total wages = 90,000. Withhold = 9,900. AGI = 90,000.
    //   Taxable = 75,000.
    //   bracket tax:
    //     11,925 @ 10% =  1,192.50
    //     36,550 @ 12% =  4,386.00  (48,475 - 11,925)
    //     26,525 @ 22% =  5,835.50  (75,000 - 48,475)
    //                  = 11,414.00
    //   withhold 9,900 < 11,414 → owe 1,514.
    assert_eq!(res.agi, Decimal::from(90_000));
    assert_eq!(res.taxable_income, Decimal::from(75_000));
    assert_eq!(res.ordinary_tax, Decimal::from(11_414));
    assert_eq!(res.tax_owed, Decimal::from(1_514));
}

#[test]
fn w2_plus_1099_nec_from_ocr_feeds_schedule_c() {
    // Person has a W-2 day job + side gig with a 1099-NEC. The
    // wizard's logic adds 1099-NEC box 1 to Schedule C gross
    // receipts. Verify the full chain produces correct SE tax.
    let w2_text = "\
Form W-2 Wage and Tax Statement
Employer's name
Day Job Corp
Wages, tips, other compensation 60000.00
Federal income tax withheld 7000.00
Social security wages 60000.00
Medicare wages and tips 60000.00
";
    let nec_text = "\
Form 1099-NEC Nonemployee Compensation
PAYER'S name
Side Gig Inc
Nonemployee compensation 15000.00
";
    let we = tax_forms::extract(w2_text).expect("w2");
    let ne = tax_forms::extract(nec_text).expect("nec");
    assert_eq!(ne.kind, TaxFormKind::Form1099Nec);

    let nec_box1 = ne
        .payload
        .get("box_1")
        .copied()
        .unwrap_or(Decimal::ZERO);

    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Single,
        w2s: vec![w2_from_extract(&we)],
        schedule_c: ScheduleC {
            gross_receipts: nec_box1,
            total_expenses: Decimal::ZERO,
            net_profit: nec_box1,
        },
        ..Default::default()
    };
    let res = compute(&r);

    // SE tax kicks in on the $15k Schedule C net.
    //   base = 15,000 × 0.9235 = 13,852.50
    //   SS = 13,852.50 × 0.124 = 1,717.71
    //   Medicare = 13,852.50 × 0.029 = 401.72 (round 401.7225 → 401.72)
    //   total = 2,119.43
    assert!(res.se_tax.total > Decimal::ZERO,
        "Schedule C net should produce SE tax, got {:?}", res.se_tax);
    assert_eq!(res.se_tax.se_base, "13852.50".parse::<Decimal>().unwrap());
}
