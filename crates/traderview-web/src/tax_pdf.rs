//! Filled-PDF export for the tax wizard — Form 1040 + Schedule C +
//! Schedule SE + Schedule E.
//!
//! v1 is a clean **data-dump PDF**, not a pixel-perfect IRS replica.
//! That's the same approach Cash App Taxes uses when they export a
//! "transcript" for paper filing: every value is labeled and grouped
//! so the user can transcribe to the IRS Free File Fillable Forms
//! online filing tool, or print/sign/mail as Form 1040 attachments.
//!
//! Pixel-perfect form overlays would need:
//!   * The official IRS PDF templates (~10 MB each, updated annually).
//!   * `pdf-form-overlay` style anchor positioning (we'd need to
//!     re-anchor every year as the IRS revises layouts).
//!   * Per-state forms — separate project.
//!
//! What we DO output:
//!   * Cover page: filer info, tax year, refund / owed summary.
//!   * Form 1040 page: every line the engine populates.
//!   * Schedule C page: gross receipts, expense detail, net profit.
//!   * Schedule SE page: SE base, SS, Medicare, additional Medicare.
//!   * Schedule E page: per-property + rollup.
//!
//! Output: a single multi-page PDF, served as `application/pdf` with
//! filename `traderview-tax-return-<year>.pdf`.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::Response;
use printpdf::{Mm, PdfDocument, BuiltinFont};
use rust_decimal::Decimal;
use traderview_tax::{compute as compute_tax, TaxReturn, TaxResult};

pub async fn generate_pdf(
    State(s): State<AppState>,
    user: AuthUser,
    Path(year): Path<i32>,
) -> Result<Response, ApiError> {
    let row: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT data FROM tax_returns WHERE user_id = $1 AND tax_year = $2",
    )
    .bind(user.id)
    .bind(year)
    .fetch_optional(&s.pool)
    .await?;
    let draft: TaxReturn = row
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    let result = compute_tax(&draft);

    let pdf_bytes = render_pdf(year, &draft, &result)
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("pdf: {e}")))?;

    let disposition = format!(
        "attachment; filename=\"traderview-tax-return-{year}.pdf\""
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_DISPOSITION, disposition)
        .body(Body::from(pdf_bytes))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("response build: {e}")))
}

fn render_pdf(year: i32, draft: &TaxReturn, result: &TaxResult) -> Result<Vec<u8>, String> {
    let (doc, page1, layer1) = PdfDocument::new(
        format!("Traderview Tax Return — {year}"),
        Mm(216.0), Mm(279.0), // US Letter portrait
        "Cover",
    );
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).map_err(|e| e.to_string())?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).map_err(|e| e.to_string())?;

    // ── Cover page ────────────────────────────────────────────────────
    let layer = doc.get_page(page1).get_layer(layer1);
    let mut y = 260.0;
    layer.use_text(
        format!("Tax Return — {year}"),
        18.0, Mm(20.0), Mm(y), &font_bold,
    );
    y -= 10.0;
    layer.use_text(
        "Computed by traderview · for review before paper filing or IRS Free File transcription",
        9.0, Mm(20.0), Mm(y), &font,
    );
    y -= 14.0;
    layer.use_text("Summary", 14.0, Mm(20.0), Mm(y), &font_bold);
    y -= 8.0;

    let lines = vec![
        ("Filing status",                 format!("{:?}", draft.status)),
        ("Total income",                  fmt_money(result.total_income)),
        ("Adjustments to income",         fmt_money(result.adjustments_total)),
        ("Adjusted gross income (AGI)",   fmt_money(result.agi)),
        ("Deduction used",                format!("{} — {}", result.deduction_label, fmt_money(result.deduction_used))),
        ("QBI § 199A deduction",          fmt_money(result.qbi_deduction)),
        ("Taxable income",                fmt_money(result.taxable_income)),
        ("Income tax (brackets)",         fmt_money(result.ordinary_tax)),
        ("Self-employment tax",           fmt_money(result.se_tax.total)),
        ("Child Tax Credit + ODC",        fmt_money(result.ctc.total)),
        ("Total tax (after credits)",     fmt_money(result.tax_after_credits)),
        ("Total payments + withholding",  fmt_money(result.total_payments)),
        ("",                              String::new()),
        ("REFUND DUE",                    fmt_money(result.refund_due)),
        ("TAX OWED",                      fmt_money(result.tax_owed)),
    ];
    for (label, value) in lines {
        let f = if label == "REFUND DUE" || label == "TAX OWED" { &font_bold } else { &font };
        layer.use_text(label.to_string(), 10.0, Mm(20.0), Mm(y), f);
        layer.use_text(value, 10.0, Mm(120.0), Mm(y), f);
        y -= 6.0;
    }
    if result.qbi_needs_manual_review {
        y -= 4.0;
        layer.use_text(
            "⚠ QBI: you may be above the SSTB threshold. Verify the W-2 wage / UBIA limits manually.",
            9.0, Mm(20.0), Mm(y), &font,
        );
    }

    // ── Form 1040 detail page ─────────────────────────────────────────
    let (p2, l2) = doc.add_page(Mm(216.0), Mm(279.0), "Form 1040");
    let layer = doc.get_page(p2).get_layer(l2);
    let mut y = 260.0;
    layer.use_text(format!("Form 1040 — {year}"), 16.0, Mm(20.0), Mm(y), &font_bold);
    y -= 10.0;

    let w2_total: Decimal = draft.w2s.iter().map(|w| w.box_1_wages).sum();
    let lines_1040 = vec![
        ("Line 1a — W-2 wages",                            fmt_money(w2_total)),
        ("Line 2b — Taxable interest",                     fmt_money(draft.interest_income)),
        ("Line 3a — Qualified dividends",                  fmt_money(draft.qualified_dividends)),
        ("Line 3b — Ordinary dividends",                   fmt_money(draft.ordinary_dividends)),
        ("Line 7 — Capital gain (LTCG)",                   fmt_money(draft.net_long_term_capital_gain)),
        ("Line 8 — Schedule 1 (Schedule C net profit)",    fmt_money(draft.schedule_c.net_profit)),
        ("Schedule E net income",                          fmt_money(draft.schedule_e.net_income)),
        ("Line 9 — Total income",                          fmt_money(result.total_income)),
        ("Line 10 — Adjustments (Sch 1)",                  fmt_money(result.adjustments_total)),
        ("Line 11 — AGI",                                  fmt_money(result.agi)),
        ("Line 12 — Deduction (std or itemized)",          fmt_money(result.deduction_used)),
        ("Line 13 — QBI § 199A",                           fmt_money(result.qbi_deduction)),
        ("Line 15 — Taxable income",                       fmt_money(result.taxable_income)),
        ("Line 16 — Tax (bracket table)",                  fmt_money(result.ordinary_tax)),
        ("Line 19 — Child Tax Credit",                     fmt_money(result.ctc.ctc)),
        ("Line 23 — Other taxes (Schedule 2 → SE tax)",    fmt_money(result.se_tax.total)),
        ("Line 24 — Total tax",                            fmt_money(result.tax_after_credits)),
        ("Line 25 — Withholding (W-2 box 2)",              fmt_money(draft.w2s.iter().map(|w| w.box_2_federal_income_tax_withheld).sum::<Decimal>())),
        ("Line 26 — Estimated tax payments",               fmt_money(draft.estimated_tax_payments)),
        ("Line 27 — EITC",                                 fmt_money(draft.eitc_claim)),
        ("Line 28 — Additional CTC (refundable)",          fmt_money(result.ctc.refundable_portion)),
        ("Line 33 — Total payments",                       fmt_money(result.total_payments)),
        ("Line 34 — Refund (if payments > tax)",           fmt_money(result.refund_due)),
        ("Line 37 — Amount you owe",                       fmt_money(result.tax_owed)),
    ];
    for (label, value) in lines_1040 {
        layer.use_text(label.to_string(), 9.0, Mm(20.0), Mm(y), &font);
        layer.use_text(value, 9.0, Mm(150.0), Mm(y), &font);
        y -= 5.5;
    }

    // ── Schedule C page ───────────────────────────────────────────────
    if draft.schedule_c.gross_receipts > Decimal::ZERO || draft.schedule_c.total_expenses > Decimal::ZERO {
        let (p3, l3) = doc.add_page(Mm(216.0), Mm(279.0), "Schedule C");
        let layer = doc.get_page(p3).get_layer(l3);
        let mut y = 260.0;
        layer.use_text("Schedule C — Profit or Loss from Business", 16.0, Mm(20.0), Mm(y), &font_bold);
        y -= 10.0;
        let lines_c = vec![
            ("Line 1 — Gross receipts or sales",  fmt_money(draft.schedule_c.gross_receipts)),
            ("Line 28 — Total expenses",          fmt_money(draft.schedule_c.total_expenses)),
            ("Line 31 — Net profit (or loss)",    fmt_money(draft.schedule_c.net_profit)),
        ];
        for (label, value) in lines_c {
            layer.use_text(label.to_string(), 10.0, Mm(20.0), Mm(y), &font);
            layer.use_text(value, 10.0, Mm(150.0), Mm(y), &font);
            y -= 6.0;
        }
    }

    // ── Schedule SE page ──────────────────────────────────────────────
    if result.se_tax.total > Decimal::ZERO {
        let (p4, l4) = doc.add_page(Mm(216.0), Mm(279.0), "Schedule SE");
        let layer = doc.get_page(p4).get_layer(l4);
        let mut y = 260.0;
        layer.use_text("Schedule SE — Self-Employment Tax", 16.0, Mm(20.0), Mm(y), &font_bold);
        y -= 10.0;
        let lines_se = vec![
            ("Line 2 — Net earnings from self-employment",     fmt_money(draft.schedule_c.net_profit)),
            ("Line 4a — × 92.35% (SE base)",                   fmt_money(result.se_tax.se_base)),
            ("Line 10 — SS portion (12.4%)",                   fmt_money(result.se_tax.ss_tax)),
            ("Line 11 — Medicare portion (2.9%)",              fmt_money(result.se_tax.medicare_tax)),
            ("Form 8959 — Additional Medicare (0.9%)",         fmt_money(result.se_tax.additional_medicare_tax)),
            ("Line 12 — Total SE tax",                         fmt_money(result.se_tax.total)),
            ("Line 13 — Half of SE tax (Schedule 1 line 15)",  fmt_money(result.se_tax.above_line_deduction)),
        ];
        for (label, value) in lines_se {
            layer.use_text(label.to_string(), 10.0, Mm(20.0), Mm(y), &font);
            layer.use_text(value, 10.0, Mm(150.0), Mm(y), &font);
            y -= 6.0;
        }
    }

    // ── Schedule E page ───────────────────────────────────────────────
    if draft.schedule_e.gross_rents > Decimal::ZERO || draft.schedule_e.total_expenses > Decimal::ZERO {
        let (p5, l5) = doc.add_page(Mm(216.0), Mm(279.0), "Schedule E");
        let layer = doc.get_page(p5).get_layer(l5);
        let mut y = 260.0;
        layer.use_text("Schedule E — Supplemental Income (Rental Real Estate)", 16.0, Mm(20.0), Mm(y), &font_bold);
        y -= 10.0;
        let lines_e = vec![
            ("Line 3 — Total rents received",  fmt_money(draft.schedule_e.gross_rents)),
            ("Line 20 — Total expenses",       fmt_money(draft.schedule_e.total_expenses)),
            ("Line 26 — Total rental income / (loss)", fmt_money(draft.schedule_e.net_income)),
        ];
        for (label, value) in lines_e {
            layer.use_text(label.to_string(), 10.0, Mm(20.0), Mm(y), &font);
            layer.use_text(value, 10.0, Mm(150.0), Mm(y), &font);
            y -= 6.0;
        }
    }

    let buf = doc.save_to_bytes().map_err(|e| e.to_string())?;
    Ok(buf)
}

fn fmt_money(d: Decimal) -> String {
    if d.is_sign_negative() {
        format!("({})", fmt_money(-d))
    } else {
        format!("${:.2}", d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use traderview_tax::engine::{ScheduleC, ScheduleE, W2};

    /// Every PDF starts with the literal bytes `%PDF-` and ends with
    /// `%%EOF`. A failed render that returns garbage would miss
    /// either anchor — this is the cheapest possible "did printpdf
    /// produce a valid file?" check.
    fn assert_valid_pdf(bytes: &[u8]) {
        assert!(bytes.starts_with(b"%PDF-"),
            "PDF must start with %PDF-, got: {:?}", &bytes[..bytes.len().min(20)]);
        // %%EOF can be followed by a trailing newline — search the tail.
        let tail = &bytes[bytes.len().saturating_sub(32)..];
        assert!(
            tail.windows(5).any(|w| w == b"%%EOF"),
            "PDF tail must contain %%EOF, got: {:?}",
            std::str::from_utf8(tail).unwrap_or("<invalid utf8>")
        );
        // Sanity floor — a useful PDF page is hundreds of bytes at
        // minimum.
        assert!(bytes.len() > 400, "PDF suspiciously small: {} bytes", bytes.len());
    }

    #[test]
    fn render_pdf_works_for_empty_return() {
        // Empty draft — covers the conditional-page paths (Sched C / SE /
        // E are all skipped because their inputs are zero). Cover page
        // and 1040 detail page still render.
        let draft = TaxReturn::default();
        let result = compute_tax(&draft);
        let bytes = render_pdf(2025, &draft, &result).expect("render");
        assert_valid_pdf(&bytes);
    }

    #[test]
    fn render_pdf_works_for_populated_return() {
        // Hits every conditional page: Sched C (gross > 0), Sched SE
        // (se_tax > 0), Sched E (gross_rents > 0). Plus the QBI
        // manual-review warning triggers since net SE crosses SSTB.
        let draft = TaxReturn {
            tax_year: 2025,
            status: traderview_tax::FilingStatus::Single,
            w2s: vec![W2 {
                employer_name: "ACME".into(),
                box_1_wages: Decimal::from(80_000),
                box_2_federal_income_tax_withheld: Decimal::from(10_000),
                ..Default::default()
            }],
            interest_income: Decimal::from(500),
            schedule_c: ScheduleC {
                gross_receipts: Decimal::from(30_000),
                total_expenses: Decimal::from(5_000),
                net_profit: Decimal::from(25_000),
            },
            schedule_e: ScheduleE {
                gross_rents: Decimal::from(24_000),
                total_expenses: Decimal::from(8_000),
                net_income: Decimal::from(16_000),
            },
            qualifying_children_under_17: 1,
            ..Default::default()
        };
        let result = compute_tax(&draft);
        let bytes = render_pdf(2025, &draft, &result).expect("render");
        assert_valid_pdf(&bytes);
        // Populated PDF is meaningfully bigger than the empty one — a
        // regression that silently dropped all the dynamic content
        // would shrink it.
        assert!(bytes.len() > 2_000, "populated PDF too small: {} bytes", bytes.len());
    }

    #[test]
    fn render_pdf_handles_negative_values_without_crashing() {
        // Schedule C loss → negative net profit. PDF must not panic on
        // formatting a negative Decimal.
        let draft = TaxReturn {
            tax_year: 2025,
            status: traderview_tax::FilingStatus::Single,
            schedule_c: ScheduleC {
                gross_receipts: Decimal::from(5_000),
                total_expenses: Decimal::from(20_000),
                net_profit: Decimal::from(-15_000),  // loss
            },
            ..Default::default()
        };
        let result = compute_tax(&draft);
        let bytes = render_pdf(2025, &draft, &result).expect("render");
        assert_valid_pdf(&bytes);
    }

    #[test]
    fn render_pdf_handles_many_w2s() {
        // Filer with 10 W-2s (gig economy, multi-employer year). PDF
        // generation must not fail or produce a corrupt file when
        // total_w2.sum() is computed over a Vec of 10 entries.
        let mut w2s = Vec::new();
        for i in 0..10 {
            w2s.push(W2 {
                employer_name: format!("Employer #{i}"),
                box_1_wages: Decimal::from(8_000 + i * 100),
                box_2_federal_income_tax_withheld: Decimal::from(900),
                ..Default::default()
            });
        }
        let draft = TaxReturn {
            tax_year: 2025,
            status: traderview_tax::FilingStatus::Single,
            w2s,
            ..Default::default()
        };
        let result = compute_tax(&draft);
        let bytes = render_pdf(2025, &draft, &result).expect("render");
        assert_valid_pdf(&bytes);
    }

    #[test]
    fn render_pdf_handles_large_dollar_amounts() {
        // Million-dollar wages — formatting must not overflow the layout
        // bounds (current `fmt_money` uses `${:.2}` which auto-widens).
        let draft = TaxReturn {
            tax_year: 2025,
            status: traderview_tax::FilingStatus::Mfj,
            w2s: vec![W2 {
                employer_name: "Mega Corp".into(),
                box_1_wages: Decimal::from(1_500_000),
                box_2_federal_income_tax_withheld: Decimal::from(450_000),
                ..Default::default()
            }],
            ..Default::default()
        };
        let result = compute_tax(&draft);
        let bytes = render_pdf(2025, &draft, &result).expect("render");
        assert_valid_pdf(&bytes);
    }

    #[test]
    fn render_pdf_handles_long_employer_names() {
        // Real-world employer names sometimes exceed 60 chars. Printpdf
        // doesn't wrap automatically — but it must NOT panic.
        let very_long = "The Extremely Long Name of an Enterprise Corporation \
                         International Holdings, LLC (a Subsidiary of Mega Holdco Inc.)";
        let draft = TaxReturn {
            tax_year: 2025,
            status: traderview_tax::FilingStatus::Single,
            w2s: vec![W2 {
                employer_name: very_long.into(),
                box_1_wages: Decimal::from(50_000),
                box_2_federal_income_tax_withheld: Decimal::from(5_000),
                ..Default::default()
            }],
            ..Default::default()
        };
        let result = compute_tax(&draft);
        let bytes = render_pdf(2025, &draft, &result).expect("render");
        assert_valid_pdf(&bytes);
    }

    #[test]
    fn render_pdf_full_combo_w2_schedule_c_schedule_e() {
        // Maximally populated draft — every conditional page renders.
        // Also flags QBI manual-review warning at high income.
        let draft = TaxReturn {
            tax_year: 2025,
            status: traderview_tax::FilingStatus::Mfj,
            w2s: vec![
                W2 {
                    employer_name: "Day Job A".into(),
                    box_1_wages: Decimal::from(120_000),
                    box_2_federal_income_tax_withheld: Decimal::from(18_000),
                    ..Default::default()
                },
                W2 {
                    employer_name: "Day Job B".into(),
                    box_1_wages: Decimal::from(95_000),
                    box_2_federal_income_tax_withheld: Decimal::from(13_000),
                    ..Default::default()
                },
            ],
            interest_income: Decimal::from(800),
            ordinary_dividends: Decimal::from(2_000),
            qualified_dividends: Decimal::from(1_500),
            net_long_term_capital_gain: Decimal::from(5_000),
            schedule_c: ScheduleC {
                gross_receipts: Decimal::from(60_000),
                total_expenses: Decimal::from(10_000),
                net_profit: Decimal::from(50_000),
            },
            schedule_e: ScheduleE {
                gross_rents: Decimal::from(36_000),
                total_expenses: Decimal::from(12_000),
                net_income: Decimal::from(24_000),
            },
            qualifying_children_under_17: 2,
            other_dependents: 1,
            estimated_tax_payments: Decimal::from(8_000),
            ..Default::default()
        };
        let result = compute_tax(&draft);
        let bytes = render_pdf(2025, &draft, &result).expect("render");
        assert_valid_pdf(&bytes);
        // Must be substantially larger than the empty case.
        assert!(bytes.len() > 3_000,
            "full-combo PDF should be ~3+ kb, got {} bytes", bytes.len());
    }

    #[test]
    fn fmt_money_renders_parens_for_negative() {
        // Locks the IRS-style accounting format: ($1,234.56) for
        // negatives, $1,234.56 for positives. Tax forms show losses
        // in parens.
        assert_eq!(fmt_money(Decimal::from(1234)), "$1234.00");
        assert_eq!(fmt_money(Decimal::from(-1234)), "($1234.00)");
        assert_eq!(fmt_money(Decimal::ZERO), "$0.00");
    }
}
