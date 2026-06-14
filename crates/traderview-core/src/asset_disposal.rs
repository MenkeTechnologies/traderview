//! Fixed-asset disposal (gain/loss) — the gain or loss when a business sells or
//! scraps a depreciable asset, with the §1245 ordinary-income recapture split.
//! Net book value is cost less accumulated depreciation; gain or loss is proceeds
//! less net book value. For §1245 personal property, gain is ordinary income to
//! the extent of prior depreciation (recapture); any amount above the original
//! cost is §1231 (capital) gain; a sale below book value is an ordinary §1231
//! loss. Distinct from the rental §1250 recapture module, which recaptures only
//! depreciation above straight-line. Drafting aid, not tax advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AssetDisposalInput {
    pub company_name: String,
    pub asset_label: String,
    /// Original capitalized cost.
    pub cost_usd: f64,
    /// Accumulated depreciation taken to the disposal date.
    pub accumulated_depreciation_usd: f64,
    /// Sale proceeds (0 for a scrap/abandonment).
    #[serde(default)]
    pub proceeds_usd: f64,
    pub disposal_date: String,
    pub date: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AssetDisposal {
    pub title: String,
    /// Cost − accumulated depreciation.
    pub net_book_value_usd: f64,
    /// Proceeds − net book value (negative = loss).
    pub gain_loss_usd: f64,
    /// Ordinary income: §1245 depreciation recapture (or an ordinary §1231 loss).
    pub ordinary_usd: f64,
    /// §1231 (capital) gain — the portion of gain above original cost.
    pub section_1231_gain_usd: f64,
    /// True when the disposal produced a loss.
    pub is_loss: bool,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &AssetDisposalInput) -> AssetDisposal {
    let book = cents(i.cost_usd - i.accumulated_depreciation_usd);
    let gain_loss = cents(i.proceeds_usd - book);
    let is_loss = gain_loss < 0.0;

    let (ordinary, sec_1231) = if gain_loss > 0.0 {
        // §1231 capital gain only for proceeds above original cost; the rest of
        // the gain is ordinary §1245 recapture.
        let capital = cents((i.proceeds_usd - i.cost_usd).max(0.0));
        (cents(gain_loss - capital), capital)
    } else {
        // A loss (or zero) on §1231 property is an ordinary loss.
        (gain_loss, 0.0)
    };

    let outcome = if gain_loss > 0.0 {
        format!(
            "The disposal produces a gain of {}: {} is ordinary income (§1245 depreciation recapture) and {} is §1231 capital gain.",
            money(gain_loss), money(ordinary), money(sec_1231)
        )
    } else if is_loss {
        format!("The disposal produces an ordinary §1231 loss of {}.", money(gain_loss.abs()))
    } else {
        "The asset is disposed of at its net book value, with no gain or loss.".to_string()
    };

    let calc_body = format!(
        "Net book value is {} (cost {} less accumulated depreciation {}). Proceeds are {}, so the gain/loss is {}. {}",
        money(book),
        money(i.cost_usd),
        money(i.accumulated_depreciation_usd),
        money(i.proceeds_usd),
        money(gain_loss),
        outcome
    );

    let mut clauses = vec![
        DocClause {
            heading: "Asset".into(),
            body: format!(
                "Company: {}\nAsset: {}\nDisposal date: {}\nStatement date: {}",
                i.company_name, i.asset_label, i.disposal_date, i.date
            ),
        },
        DocClause { heading: "Disposal".into(), body: calc_body },
        DocClause {
            heading: "Journal Entry".into(),
            body: format!(
                "Remove the asset: credit the asset {} and debit accumulated depreciation {}. Debit cash {}. {}",
                money(i.cost_usd),
                money(i.accumulated_depreciation_usd),
                money(i.proceeds_usd),
                if gain_loss > 0.0 {
                    format!("Credit gain on disposal {}.", money(gain_loss))
                } else if is_loss {
                    format!("Debit loss on disposal {}.", money(gain_loss.abs()))
                } else {
                    "No gain or loss.".to_string()
                }
            ),
        },
    ];

    let note = i.note.trim();
    if !note.is_empty() {
        clauses.push(DocClause { heading: "Note".into(), body: note.to_string() });
    }

    AssetDisposal {
        title: "Fixed-Asset Disposal — Gain/Loss".into(),
        net_book_value_usd: book,
        gain_loss_usd: gain_loss,
        ordinary_usd: ordinary,
        section_1231_gain_usd: sec_1231,
        is_loss,
        statutory_citation: "IRC §1245 / §1231".to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> AssetDisposalInput {
        AssetDisposalInput {
            company_name: "Acme Co".into(),
            asset_label: "CNC machine".into(),
            cost_usd: 10_000.0,
            accumulated_depreciation_usd: 6_000.0,
            proceeds_usd: 7_000.0,
            disposal_date: "2026-06-15".into(),
            date: "2026-07-01".into(),
            note: String::new(),
        }
    }

    #[test]
    fn gain_below_cost_all_ordinary() {
        let d = generate(&base());
        assert!(close(d.net_book_value_usd, 4_000.0));
        assert!(close(d.gain_loss_usd, 3_000.0));
        assert!(close(d.ordinary_usd, 3_000.0));
        assert!(close(d.section_1231_gain_usd, 0.0));
        assert!(!d.is_loss);
    }

    #[test]
    fn gain_above_cost_splits_1231() {
        let d = generate(&AssetDisposalInput { proceeds_usd: 12_000.0, ..base() });
        assert!(close(d.gain_loss_usd, 8_000.0));
        // Recapture capped at accumulated depreciation; excess above cost is §1231.
        assert!(close(d.ordinary_usd, 6_000.0));
        assert!(close(d.section_1231_gain_usd, 2_000.0));
    }

    #[test]
    fn sold_below_book_is_loss() {
        let d = generate(&AssetDisposalInput { proceeds_usd: 3_000.0, ..base() });
        assert!(close(d.gain_loss_usd, -1_000.0));
        assert!(close(d.ordinary_usd, -1_000.0));
        assert!(d.is_loss);
    }

    #[test]
    fn at_book_no_gain_loss() {
        let d = generate(&AssetDisposalInput { proceeds_usd: 4_000.0, ..base() });
        assert!(close(d.gain_loss_usd, 0.0));
        assert!(!d.is_loss);
    }

    #[test]
    fn scrap_full_recapture_then_book_loss() {
        // Proceeds 0 on a fully-depreciated-down asset → loss equal to book value.
        let d = generate(&AssetDisposalInput { proceeds_usd: 0.0, ..base() });
        assert!(close(d.gain_loss_usd, -4_000.0));
        assert!(d.is_loss);
    }

    #[test]
    fn citation_present() {
        let d = generate(&base());
        assert_eq!(d.statutory_citation, "IRC §1245 / §1231");
    }
}
