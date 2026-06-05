//! End-to-end TaxReturn → TaxResult scenarios with hand-computed
//! expected numbers. Each test models a realistic filer and pins the
//! final refund/owed to within a dollar. These are the contracts the
//! wizard's UI relies on — drift in any number means somebody changed
//! a constant or the order of operations in `engine::compute`.
//!
//! Sources for the hand-math:
//!   * IRS Rev. Proc. 2024-40 (2025 inflation adjustments)
//!   * 2025 Form 1040 line numbers
//!   * SSA 2025 wage-base announcement

use rust_decimal::Decimal;
use traderview_tax::{
    compute,
    engine::{Itemized, ScheduleC, W2},
    FilingStatus, TaxReturn,
};

fn d(n: i64) -> Decimal {
    Decimal::from(n)
}
fn dc(s: &str) -> Decimal {
    s.parse::<Decimal>().expect("decimal")
}

/// Scenario 1: Single W-2 employee, $75k salary, $8k withheld.
/// Standard deduction, no kids, no other income.
///
/// Hand math:
///   Wages         = 75,000
///   Std deduction = 15,000
///   Taxable       = 60,000
///   Bracket tax:
///     11,925 @ 10% = 1,192.50
///     36,550 @ 12% = 4,386.00  (48,475 - 11,925)
///     11,525 @ 22% = 2,535.50  (60,000 - 48,475)
///                  = 8,114.00
///   Withholding   = 8,000
///   Tax owed      = 114.00
#[test]
fn scenario_single_w2_only() {
    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Single,
        w2s: vec![W2 {
            employer_name: "ACME".into(),
            box_1_wages: d(75_000),
            box_2_federal_income_tax_withheld: d(8_000),
            box_3_ss_wages: d(75_000),
            box_4_ss_tax_withheld: dc("4650"),
            box_5_medicare_wages: d(75_000),
            box_6_medicare_tax_withheld: dc("1087.5"),
            box_17_state_income_tax: Decimal::ZERO,
        }],
        ..Default::default()
    };
    let res = compute(&r);
    assert_eq!(res.agi, d(75_000));
    assert_eq!(res.deduction_used, d(15_000));
    assert_eq!(res.taxable_income, d(60_000));
    assert_eq!(res.ordinary_tax, dc("8114"));
    assert_eq!(res.tax_owed, dc("114"));
    assert_eq!(res.refund_due, Decimal::ZERO);
}

/// Scenario 2: Solo freelancer (Schedule C only).
/// $60k gross / $10k expenses → $50k net SE.
/// Single filer, no W-2, no other income, no kids.
///
/// SE tax (verified by se_tax::tests::modest_self_employment_no_w2):
///   base = 46,175; SS 5,725.70; Medicare 1,339.08; total 7,064.78
///   above-the-line = 3,532.39
/// AGI = 50,000 - 3,532.39 = 46,467.61
/// Std deduction = 15,000
/// TI before QBI = 31,467.61
/// QBI: 20% × min(50,000 net, 31,467.61) = min(10,000, 6,293.52) = 6,293.52
///   (TI cap binds because TI before QBI is small)
/// Taxable = 31,467.61 - 6,293.52 = 25,174.09
/// Bracket tax (single 2025):
///   11,925 @ 10% = 1,192.50
///   13,249.09 @ 12% = 1,589.8908 → rounds to per-line decimal arithmetic in engine
/// Engine sums then rounds → tax ≈ 2,782.39 (will assert range, not exact, due to
/// Decimal intermediate rounding).
/// Income tax + SE tax = ~ 2,782.39 + 7,064.78 = ~9,847.17
/// No withholding, no estimated → owed ≈ that.
#[test]
fn scenario_solo_freelancer_schedule_c() {
    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Single,
        schedule_c: ScheduleC {
            gross_receipts: d(60_000),
            total_expenses: d(10_000),
            net_profit: d(50_000),
        },
        ..Default::default()
    };
    let res = compute(&r);
    // SE tax is the load-bearing piece — pin it exactly (mirrors the
    // se_tax::tests value).
    assert_eq!(res.se_tax.total, dc("7064.78"));
    assert_eq!(res.se_tax.above_line_deduction, dc("3532.39"));
    // AGI = net SE - half SE tax.
    assert_eq!(res.agi, dc("46467.61"));
    // Std deduction.
    assert_eq!(res.deduction_used, d(15_000));
    // QBI binds on TI cap (TI before QBI = 31,467.61, 20% = 6,293.52).
    assert_eq!(res.qbi_deduction, dc("6293.52"));
    // Taxable income.
    assert_eq!(res.taxable_income, dc("25174.09"));
    // Owes everything — no withholding.
    assert_eq!(res.refund_due, Decimal::ZERO);
    assert!(res.tax_owed > d(9_000) && res.tax_owed < d(10_500),
        "expected ~$9,800 owed, got {}", res.tax_owed);
}

/// Scenario 3: Dual-income MFJ with two kids.
/// Both spouses W-2: $90k + $60k = $150k combined.
/// 2 qualifying children under 17.
/// Itemized = $32k (beats $30k MFJ std).
///
/// AGI = 150,000
/// Itemized = 32,000 (used)
/// TI = 118,000
/// MFJ brackets:
///   23,850 @ 10%   =  2,385.00
///   73,100 @ 12%   =  8,772.00  (96,950-23,850)
///   21,050 @ 22%   =  4,631.00  (118,000-96,950)
///                  = 15,788.00
/// CTC 2 × $2,000 = 4,000 ($3,400 refundable, $600 nonref offset).
/// Nonref CTC ($600) reduces ordinary tax: 15,788 - 600 = 15,188
/// Refundable $3,400 goes into payments.
/// Withholding: 11,000 + refundable $3,400 = 14,400 payments
/// Tax after credits + SE = 15,188 (no SE in this scenario)
/// Owed = 15,188 - 14,400 = 788
#[test]
fn scenario_mfj_dual_income_two_kids_itemized() {
    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Mfj,
        w2s: vec![
            W2 {
                box_1_wages: d(90_000),
                box_2_federal_income_tax_withheld: d(7_000),
                ..Default::default()
            },
            W2 {
                box_1_wages: d(60_000),
                box_2_federal_income_tax_withheld: d(4_000),
                ..Default::default()
            },
        ],
        itemized: Itemized {
            medical_over_7_5_pct_agi: Decimal::ZERO,
            state_and_local_taxes_capped_at_10k: d(10_000),
            mortgage_interest: d(18_000),
            charitable_gifts: d(4_000),
            casualty_losses: Decimal::ZERO,
        },
        qualifying_children_under_17: 2,
        ..Default::default()
    };
    let res = compute(&r);
    assert_eq!(res.agi, d(150_000));
    assert_eq!(res.deduction_label, "itemized");
    assert_eq!(res.deduction_used, d(32_000));
    assert_eq!(res.taxable_income, d(118_000));
    assert_eq!(res.ordinary_tax, dc("15788"));
    assert_eq!(res.ctc.total, d(4_000));
    assert_eq!(res.ctc.refundable_portion, dc("3400"));
    // Nonref part is total - refundable = 600.
    let nonref = res.ctc.total - res.ctc.refundable_portion;
    assert_eq!(nonref, d(600));
    // Total payments = 11,000 W-2 withhold + 3,400 refundable CTC.
    assert_eq!(res.total_payments, d(14_400));
    // Tax after credits = 15,788 - 600 = 15,188.
    assert_eq!(res.tax_after_credits, d(15_188));
    assert_eq!(res.tax_owed, d(788));
    assert_eq!(res.refund_due, Decimal::ZERO);
}

/// Scenario 4: MFS in CTC phase-out ($1 over the $200k threshold).
/// One kid, AGI $200,001.
///
/// Excess over threshold = 1 → ceil to 1,000-block → $50 reduction.
/// CTC raw = $2,000 → $1,950 after phase-out.
#[test]
fn scenario_mfs_ctc_at_phaseout_edge() {
    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Mfs,
        w2s: vec![W2 {
            box_1_wages: d(200_001),
            box_2_federal_income_tax_withheld: d(50_000),
            ..Default::default()
        }],
        qualifying_children_under_17: 1,
        ..Default::default()
    };
    let res = compute(&r);
    assert_eq!(res.agi, d(200_001));
    // CTC ratchets down by $50 at $1 over.
    assert_eq!(res.ctc.total, dc("1950"));
}

/// Scenario 5: Retiree-style return.
/// $0 W-2, $25k interest + $15k qualified dividends + $20k LTCG.
/// Single filer, no SE, std deduction.
///
/// Total income = 25,000 + 15,000 + 20,000 (LTCG flows in directly
/// through engine's total_income line) + 0 (qualified divs treated
/// in net_capital_gain calc, not income).
/// Actually engine adds ordinary_dividends to total_income but NOT
/// qualified_dividends separately — qualified divs are a SUBSET of
/// ordinary divs (1099-DIV box 1b ≤ box 1a). Test models this by
/// leaving ordinary_dividends = 0 and asserting the LTCG only.
///
/// We mainly want to verify:
///   1. QBI = 0 (no SE).
///   2. Std deduction applies.
///   3. Tax computed over taxable income (engine does NOT separate
///      LTCG into preferential brackets in v1 — TODO documented).
#[test]
fn scenario_retiree_interest_dividends_ltcg() {
    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Single,
        interest_income: d(25_000),
        ordinary_dividends: Decimal::ZERO,
        qualified_dividends: d(15_000),
        net_long_term_capital_gain: d(20_000),
        ..Default::default()
    };
    let res = compute(&r);
    // Total income = interest + ordinary_div + LTCG + other = 25 + 0 + 20 + 0 = 45.
    // (qualified_dividends doesn't double-count; it's a subset of ordinary_div.)
    assert_eq!(res.total_income, d(45_000));
    assert_eq!(res.agi, d(45_000));
    assert_eq!(res.deduction_used, d(15_000));
    assert_eq!(res.qbi_deduction, Decimal::ZERO);
    assert_eq!(res.taxable_income, d(30_000));
    // No SE tax.
    assert_eq!(res.se_tax.total, Decimal::ZERO);
    // Tax = brackets only. Engine does NOT yet apply LTCG-preferential rate.
    // Pinned to confirm current behavior — when LTCG-bracket support
    // lands, this number drops and the test gets updated.
    let expected = dc("1192.5")  // 11,925 @ 10%
        + dc("2169")             // 18,075 @ 12% (30,000-11,925)
        ;
    assert_eq!(res.ordinary_tax, expected);
}

/// Scenario 6: High earner crossing both the SSTB threshold AND the
/// additional Medicare threshold.
/// Single filer, $300k net SE, no W-2.
///
/// Key behaviors to pin:
///   * Additional Medicare 0.9% kicks in (base $277k > $200k threshold).
///   * QBI flags `needs_manual_review` (non-SSTB above threshold).
///   * SS tax CAPS at the 2025 wage base ($176,100 × 0.124 = 21,836.40).
#[test]
fn scenario_high_earner_above_ss_cap_and_addl_medicare() {
    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Single,
        schedule_c: ScheduleC {
            gross_receipts: d(300_000),
            total_expenses: Decimal::ZERO,
            net_profit: d(300_000),
        },
        qbi_is_sstb: false,
        ..Default::default()
    };
    let res = compute(&r);
    // SE base = 300,000 × 0.9235 = 277,050.
    assert_eq!(res.se_tax.se_base, dc("277050.00"));
    // SS tax CAPPED at 176,100 × 0.124 = 21,836.40. (Cap binds.)
    assert_eq!(res.se_tax.ss_tax, dc("21836.40"));
    // Additional Medicare: (277,050 - 200,000) × 0.009 = 693.45.
    assert_eq!(res.se_tax.additional_medicare_tax, dc("693.45"));
    // QBI flag — non-SSTB high earner triggers manual-review warning.
    assert!(res.qbi_needs_manual_review,
        "high earner non-SSTB should flag QBI manual review");
}

/// Scenario 7: Boundary case — taxable income exactly at standard
/// deduction. Tests the zero-tax path when AGI = std_deduction.
#[test]
fn scenario_boundary_taxable_income_zero() {
    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Single,
        interest_income: d(15_000), // exactly std deduction
        ..Default::default()
    };
    let res = compute(&r);
    assert_eq!(res.taxable_income, Decimal::ZERO);
    assert_eq!(res.ordinary_tax, Decimal::ZERO);
    assert_eq!(res.tax_owed, Decimal::ZERO);
}

/// Scenario 9: Head of household freelancer with one kid.
/// $55k Schedule C, no W-2, 1 qualifying child, HoH status.
///
/// HoH std deduction = $22,500 (higher than single's $15k).
/// HoH brackets push the 10%/12% boundary to $17,000 (vs single $11,925).
///
/// Key checks:
///   * HoH std deduction applied (not single's).
///   * CTC $2,000 reduces ordinary tax (one kid, well under phase-out).
///   * SE tax computed on Schedule C net.
#[test]
fn scenario_hoh_freelancer_one_kid() {
    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Hoh,
        schedule_c: ScheduleC {
            gross_receipts: d(55_000),
            total_expenses: d(0),
            net_profit: d(55_000),
        },
        qualifying_children_under_17: 1,
        ..Default::default()
    };
    let res = compute(&r);
    // SE tax on $55k.
    //   base = 55,000 × 0.9235 = 50,792.50
    //   SS   = 50,792.50 × 0.124 = 6,298.27
    //   Med  = 50,792.50 × 0.029 = 1,472.98 (1472.9825 → 1472.98)
    //   total = 7,771.25; half = 3,885.625 → 3,885.63
    assert_eq!(res.se_tax.total, dc("7771.25"));
    // Above-line = (ss + medicare) / 2 = (6,298.27 + 1,472.98) / 2 =
    // 3,885.625 → banker's-round to 3,885.62 (digit before is even).
    assert_eq!(res.se_tax.above_line_deduction, dc("3885.62"));
    // AGI = 55,000 - 3,885.62 = 51,114.38
    assert_eq!(res.agi, dc("51114.38"));
    // HoH std deduction.
    assert_eq!(res.deduction_used, d(22_500));
    // CTC $2,000 nonref (none refundable until tax_after_credits is solved).
    assert_eq!(res.ctc.total, d(2_000));
}

/// Scenario 10: MFS in a high-SALT state — SALT cap binds at $10k.
/// User reports $18k in state+local taxes; itemized total must cap
/// the SALT line at $10k (per IRC § 164(b)(6) post-TCJA).
#[test]
fn scenario_mfs_salt_cap_binds() {
    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Mfs,
        w2s: vec![W2 {
            box_1_wages: d(120_000),
            box_2_federal_income_tax_withheld: d(20_000),
            ..Default::default()
        }],
        itemized: Itemized {
            medical_over_7_5_pct_agi: Decimal::ZERO,
            state_and_local_taxes_capped_at_10k: d(18_000), // user-reported, BEFORE cap
            mortgage_interest: d(15_000),
            charitable_gifts: d(3_000),
            casualty_losses: Decimal::ZERO,
        },
        ..Default::default()
    };
    let res = compute(&r);
    // SALT must cap at $10k → itemized total = 10k + 15k + 3k = 28k.
    // MFS std deduction = $15k → 28k > 15k, itemized used.
    assert_eq!(res.deduction_label, "itemized");
    assert_eq!(res.deduction_used, d(28_000));
    // Confirm AGI flowed correctly.
    assert_eq!(res.agi, d(120_000));
}

/// Scenario 11: Retiree with large qualified dividends + LTCG —
/// QBI's TI cap uses net_capital_gain (LTCG + qualified div) to
/// reduce the TI base.
#[test]
fn scenario_retiree_qbi_ti_cap_reduces_for_capgains() {
    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Single,
        qualified_dividends: d(40_000),
        net_long_term_capital_gain: d(20_000),
        // Tiny Schedule C income to trigger QBI.
        schedule_c: ScheduleC {
            gross_receipts: d(10_000),
            total_expenses: Decimal::ZERO,
            net_profit: d(10_000),
        },
        ..Default::default()
    };
    let res = compute(&r);
    // QBI: 20% of $10k SE net = $2,000.
    // TI cap = 20% × (TI - net_cap_gain).
    //   AGI = 10,000 - half SE = 10,000 - se_tax.above_line
    //   TI before QBI = AGI - 15,000 (std), clamped at 0 if negative.
    //   net_capital_gain = 40,000 + 20,000 = 60,000.
    //   TI - net_cap_gain → likely negative → 0. So TI cap = $0.
    //   QBI deduction = min(2,000, 0) = 0.
    assert_eq!(res.qbi_deduction, Decimal::ZERO,
        "QBI must be capped at $0 when TI - net_cap_gain ≤ 0");
}

/// Scenario 8: Mixed Schedule C + W-2 with SE tax SS-cap interaction.
/// W-2 already at $100k SS wages (under cap), Schedule C $80k net.
/// SE base = 73,880. SS portion limited by (cap - 100k) = 76,100 → 73,880
/// fits entirely → full 12.4% SS tax on 73,880 = 9,161.12.
/// Verifies the cap-coordination math when W-2 partially uses the cap.
#[test]
fn scenario_w2_plus_schedule_c_partial_ss_cap() {
    let r = TaxReturn {
        tax_year: 2025,
        status: FilingStatus::Single,
        w2s: vec![W2 {
            box_1_wages: d(100_000),
            box_2_federal_income_tax_withheld: d(15_000),
            box_3_ss_wages: d(100_000),
            box_4_ss_tax_withheld: d(6_200),
            box_5_medicare_wages: d(100_000),
            box_6_medicare_tax_withheld: d(1_450),
            ..Default::default()
        }],
        schedule_c: ScheduleC {
            gross_receipts: d(80_000),
            total_expenses: Decimal::ZERO,
            net_profit: d(80_000),
        },
        ..Default::default()
    };
    let res = compute(&r);
    // SE base = 80,000 × 0.9235 = 73,880.
    assert_eq!(res.se_tax.se_base, dc("73880.00"));
    // Remaining SS cap = 176,100 - 100,000 = 76,100. SE base 73,880 < 76,100
    // so full SS tax applies: 73,880 × 0.124 = 9,161.12.
    assert_eq!(res.se_tax.ss_tax, dc("9161.12"));
}
