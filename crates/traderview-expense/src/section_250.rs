//! IRC §250 — GILTI / FDII deduction (post-OBBBA: NCTI / FDDEI).
//!
//! Added by TCJA § 14202 (P.L. 115-97) and substantially modified by
//! the One Big Beautiful Bill Act (OBBBA) of 2025 effective for tax
//! years beginning after 2025-12-31. Provides a deduction for US C
//! corporations on two specific categories of foreign-related income:
//!
//! - **Pre-OBBBA**: GILTI (Global Intangible Low-Taxed Income) and
//!   FDII (Foreign-Derived Intangible Income)
//! - **Post-OBBBA (2026+)**: NCTI (Net CFC Tested Income) and FDDEI
//!   (Foreign-Derived Deduction Eligible Income) — renamed
//!
//! **Deduction percentages and effective rates** (assumes § 11 21%
//! corporate rate):
//!
//! | Period          | Income type | Deduction % | Effective rate |
//! |-----------------|-------------|-------------|----------------|
//! | 2018-2025       | GILTI       | 50%         | 10.5%          |
//! | 2018-2025       | FDII        | 37.5%       | 13.125%        |
//! | 2026+ (OBBBA)   | NCTI        | 40%         | 12.6%          |
//! | 2026+ (OBBBA)   | FDDEI       | 33.34%      | 14.0%          |
//!
//! **OBBBA structural changes** (eff. 2026-01-01):
//!
//! - Renaming: GILTI → NCTI; FDII → FDDEI
//! - DTIR (Deemed Tangible Income Return) and NDTIR (Net DTIR) — the
//!   10% return on Qualified Business Asset Investment (QBAI) that
//!   reduced GILTI inclusion — are ELIMINATED
//! - Interest expense and R&E expenditures EXCLUDED from allocable
//!   expenses in computing Deduction Eligible Income (DEI)
//! - Foreign Tax Credit (FTC) for deemed-paid taxes under NCTI
//!   raised from 80% to 90%
//!
//! Sources: [Cornell LII 26 U.S.C. § 250](https://www.law.cornell.edu/uscode/text/26/250),
//! [EY — OBBBA International Tax Reforms](https://www.ey.com/en_gl/technical/tax-alerts/united-states-changes-to-gilti-fdii-and-beat-among-others-included-in-final-reconciliation-legislation-but-not-previously-proposed-remedy-for-unfair-foreign-taxes),
//! [Steptoe — International Tax Changes in OBBBA](https://www.steptoe.com/en/news-publications/international-tax-changes-in-the-one-big-beautiful-bill-act.html).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section250Input {
    pub tax_year: i32,
    pub gilti_ncti_income_dollars: i64,
    pub fdii_fddei_income_dollars: i64,
    /// Qualified Business Asset Investment for pre-2026 DTIR
    /// computation. Ignored post-OBBBA (DTIR eliminated).
    pub qbai_dollars: i64,
    pub corporate_tax_rate_bp: u32,
    /// Deemed-paid foreign taxes on GILTI/NCTI for foreign tax credit
    /// computation. 80% allowed pre-OBBBA, 90% post-OBBBA.
    pub deemed_paid_foreign_taxes_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section250Result {
    pub post_obbba: bool,
    pub gilti_ncti_deduction_pct_bp: u32,
    pub fdii_fddei_deduction_pct_bp: u32,
    /// 10% × QBAI deduction reducing GILTI inclusion. Zero post-OBBBA.
    pub dtir_deduction_dollars: i64,
    pub net_gilti_ncti_inclusion_dollars: i64,
    pub gilti_ncti_section_250_deduction_dollars: i64,
    pub fdii_fddei_section_250_deduction_dollars: i64,
    pub effective_tax_rate_on_gilti_ncti_bp: u32,
    pub effective_tax_rate_on_fdii_fddei_bp: u32,
    pub foreign_tax_credit_pct_bp: u32,
    pub foreign_tax_credit_amount_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section250Input) -> Section250Result {
    let post_obbba = input.tax_year >= 2026;

    // Deduction percentages by era.
    let (gilti_pct_bp, fdii_pct_bp, ftc_pct_bp) = if post_obbba {
        (4000, 3334, 9000) // 40% / 33.34% / 90%
    } else {
        (5000, 3750, 8000) // 50% / 37.5% / 80%
    };

    // Pre-OBBBA: 10% of QBAI reduces GILTI inclusion via DTIR.
    let dtir = if post_obbba {
        0
    } else {
        input.qbai_dollars / 10
    };
    let net_gilti = (input.gilti_ncti_income_dollars - dtir).max(0);

    // § 250 deductions.
    let gilti_deduction = ((net_gilti as i128) * (gilti_pct_bp as i128) / 10_000) as i64;
    let fdii_deduction =
        ((input.fdii_fddei_income_dollars as i128) * (fdii_pct_bp as i128) / 10_000) as i64;

    // Effective rate = corp rate × (1 − deduction%).
    let effective_gilti_rate_bp =
        (input.corporate_tax_rate_bp as u64 * (10_000 - gilti_pct_bp) as u64 / 10_000) as u32;
    let effective_fdii_rate_bp =
        (input.corporate_tax_rate_bp as u64 * (10_000 - fdii_pct_bp) as u64 / 10_000) as u32;

    // Foreign tax credit.
    let ftc_amount =
        ((input.deemed_paid_foreign_taxes_dollars as i128) * (ftc_pct_bp as i128) / 10_000) as i64;

    let income_label_gilti = if post_obbba { "NCTI" } else { "GILTI" };
    let income_label_fdii = if post_obbba { "FDDEI" } else { "FDII" };
    let note = format!(
        "{} (tax year {}): {} ${} × {}.{}% deduction = ${} ({}.{}% effective rate); {} ${} × {}.{}% deduction = ${} ({}.{}% effective rate). {} deemed-paid FTC at {}.{}% = ${}.{}",
        if post_obbba { "Post-OBBBA NCTI/FDDEI" } else { "Pre-OBBBA GILTI/FDII" },
        input.tax_year,
        income_label_gilti,
        input.gilti_ncti_income_dollars,
        gilti_pct_bp / 100,
        gilti_pct_bp % 100,
        gilti_deduction,
        effective_gilti_rate_bp / 100,
        effective_gilti_rate_bp % 100,
        income_label_fdii,
        input.fdii_fddei_income_dollars,
        fdii_pct_bp / 100,
        fdii_pct_bp % 100,
        fdii_deduction,
        effective_fdii_rate_bp / 100,
        effective_fdii_rate_bp % 100,
        income_label_gilti,
        ftc_pct_bp / 100,
        ftc_pct_bp % 100,
        ftc_amount,
        if post_obbba {
            " DTIR/NDTIR eliminated; full GILTI/NCTI subject to § 250 directly."
        } else if dtir > 0 {
            ""
        } else {
            " (No QBAI / DTIR impact.)"
        },
    );

    Section250Result {
        post_obbba,
        gilti_ncti_deduction_pct_bp: gilti_pct_bp,
        fdii_fddei_deduction_pct_bp: fdii_pct_bp,
        dtir_deduction_dollars: dtir,
        net_gilti_ncti_inclusion_dollars: net_gilti,
        gilti_ncti_section_250_deduction_dollars: gilti_deduction,
        fdii_fddei_section_250_deduction_dollars: fdii_deduction,
        effective_tax_rate_on_gilti_ncti_bp: effective_gilti_rate_bp,
        effective_tax_rate_on_fdii_fddei_bp: effective_fdii_rate_bp,
        foreign_tax_credit_pct_bp: ftc_pct_bp,
        foreign_tax_credit_amount_dollars: ftc_amount,
        citation:
            "IRC §250 GILTI / FDII deduction (TCJA P.L. 115-97 §14202, eff. 2018); One Big Beautiful Bill Act of 2025 §250 amendments (eff. tax years beginning after 2025-12-31) renaming GILTI → NCTI / FDII → FDDEI, reducing deductions to 40%/33.34%, eliminating DTIR/NDTIR, raising FTC % to 90%"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section250Input {
        Section250Input {
            tax_year: 2025,
            gilti_ncti_income_dollars: 1_000_000,
            fdii_fddei_income_dollars: 500_000,
            qbai_dollars: 0,
            corporate_tax_rate_bp: 2100,
            deemed_paid_foreign_taxes_dollars: 0,
        }
    }

    // Pre-OBBBA (2025 and earlier).

    #[test]
    fn pre_obbba_gilti_50_pct_deduction() {
        let r = compute(&base());
        assert!(!r.post_obbba);
        assert_eq!(r.gilti_ncti_deduction_pct_bp, 5000);
        // $1M × 50% = $500k deduction.
        assert_eq!(r.gilti_ncti_section_250_deduction_dollars, 500_000);
    }

    #[test]
    fn pre_obbba_fdii_37_5_pct_deduction() {
        let r = compute(&base());
        assert_eq!(r.fdii_fddei_deduction_pct_bp, 3750);
        // $500k × 37.5% = $187,500.
        assert_eq!(r.fdii_fddei_section_250_deduction_dollars, 187_500);
    }

    #[test]
    fn pre_obbba_effective_gilti_rate_10_5_pct() {
        // 21% × (1 − 50%) = 10.5%.
        let r = compute(&base());
        assert_eq!(r.effective_tax_rate_on_gilti_ncti_bp, 1050);
    }

    #[test]
    fn pre_obbba_effective_fdii_rate_13_125_pct() {
        // 21% × (1 − 37.5%) = 13.125% → 1312 bp (truncated).
        let r = compute(&base());
        assert_eq!(r.effective_tax_rate_on_fdii_fddei_bp, 1312);
    }

    #[test]
    fn pre_obbba_dtir_10_pct_of_qbai() {
        let mut i = base();
        i.qbai_dollars = 2_000_000;
        let r = compute(&i);
        assert_eq!(r.dtir_deduction_dollars, 200_000);
        assert_eq!(r.net_gilti_ncti_inclusion_dollars, 800_000);
        // $800k × 50% = $400k deduction (not $500k).
        assert_eq!(r.gilti_ncti_section_250_deduction_dollars, 400_000);
    }

    #[test]
    fn pre_obbba_ftc_80_pct() {
        let mut i = base();
        i.deemed_paid_foreign_taxes_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(r.foreign_tax_credit_pct_bp, 8000);
        assert_eq!(r.foreign_tax_credit_amount_dollars, 80_000);
    }

    // Post-OBBBA (2026+).

    #[test]
    fn post_obbba_ncti_40_pct_deduction() {
        let mut i = base();
        i.tax_year = 2026;
        let r = compute(&i);
        assert!(r.post_obbba);
        assert_eq!(r.gilti_ncti_deduction_pct_bp, 4000);
        // $1M × 40% = $400k deduction.
        assert_eq!(r.gilti_ncti_section_250_deduction_dollars, 400_000);
    }

    #[test]
    fn post_obbba_fddei_33_34_pct_deduction() {
        let mut i = base();
        i.tax_year = 2026;
        let r = compute(&i);
        assert_eq!(r.fdii_fddei_deduction_pct_bp, 3334);
        // $500k × 33.34% = $166,700.
        assert_eq!(r.fdii_fddei_section_250_deduction_dollars, 166_700);
    }

    #[test]
    fn post_obbba_effective_ncti_rate_12_6_pct() {
        // 21% × (1 − 40%) = 12.6%.
        let mut i = base();
        i.tax_year = 2026;
        let r = compute(&i);
        assert_eq!(r.effective_tax_rate_on_gilti_ncti_bp, 1260);
    }

    #[test]
    fn post_obbba_effective_fddei_rate_14_pct() {
        // 21% × (1 − 33.34%) = 21% × 66.66% = 13.9986% ≈ 1399 bp.
        let mut i = base();
        i.tax_year = 2026;
        let r = compute(&i);
        assert_eq!(r.effective_tax_rate_on_fdii_fddei_bp, 1399);
    }

    #[test]
    fn post_obbba_dtir_eliminated() {
        // QBAI no longer reduces NCTI under OBBBA.
        let mut i = base();
        i.tax_year = 2026;
        i.qbai_dollars = 2_000_000;
        let r = compute(&i);
        assert_eq!(r.dtir_deduction_dollars, 0);
        assert_eq!(r.net_gilti_ncti_inclusion_dollars, 1_000_000);
    }

    #[test]
    fn post_obbba_ftc_90_pct() {
        let mut i = base();
        i.tax_year = 2026;
        i.deemed_paid_foreign_taxes_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(r.foreign_tax_credit_pct_bp, 9000);
        assert_eq!(r.foreign_tax_credit_amount_dollars, 90_000);
    }

    // Year boundary.

    #[test]
    fn year_2025_pre_obbba() {
        let mut i = base();
        i.tax_year = 2025;
        let r = compute(&i);
        assert!(!r.post_obbba);
    }

    #[test]
    fn year_2026_post_obbba() {
        let mut i = base();
        i.tax_year = 2026;
        let r = compute(&i);
        assert!(r.post_obbba);
    }

    // Notes / citations.

    #[test]
    fn pre_obbba_note_uses_gilti_fdii_labels() {
        let r = compute(&base());
        assert!(r.note.contains("Pre-OBBBA GILTI/FDII"));
        assert!(r.note.contains("GILTI"));
        assert!(r.note.contains("FDII"));
    }

    #[test]
    fn post_obbba_note_uses_ncti_fddei_labels() {
        let mut i = base();
        i.tax_year = 2026;
        let r = compute(&i);
        assert!(r.note.contains("Post-OBBBA NCTI/FDDEI"));
        assert!(r.note.contains("NCTI"));
        assert!(r.note.contains("FDDEI"));
    }

    #[test]
    fn post_obbba_note_mentions_dtir_elimination() {
        let mut i = base();
        i.tax_year = 2026;
        let r = compute(&i);
        assert!(r.note.contains("DTIR/NDTIR eliminated"));
    }

    #[test]
    fn citation_mentions_tcja_and_obbba() {
        let r = compute(&base());
        assert!(r.citation.contains("TCJA"));
        assert!(r.citation.contains("§14202"));
        assert!(r.citation.contains("One Big Beautiful Bill Act"));
        assert!(r.citation.contains("NCTI"));
        assert!(r.citation.contains("FDDEI"));
    }

    // Precision.

    #[test]
    fn very_large_gilti_precision() {
        let mut i = base();
        i.gilti_ncti_income_dollars = 1_000_000_000;
        let r = compute(&i);
        // $1B × 50% = $500M.
        assert_eq!(r.gilti_ncti_section_250_deduction_dollars, 500_000_000);
    }

    #[test]
    fn zero_income_zero_deduction() {
        let mut i = base();
        i.gilti_ncti_income_dollars = 0;
        i.fdii_fddei_income_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.gilti_ncti_section_250_deduction_dollars, 0);
        assert_eq!(r.fdii_fddei_section_250_deduction_dollars, 0);
    }
}
