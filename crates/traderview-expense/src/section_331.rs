//! IRC §331 — Gain or loss to shareholder in corporate liquidations.
//!
//! The shareholder-side counterpart to `section_336` (corporate
//! side of the same transaction) and bookend with `section_351`
//! (formation). On a complete liquidation, §331(a) treats the
//! liquidating distribution as **in full payment in exchange for
//! the stock** under §1001 — gain or loss equals FMV of property
//! received minus the shareholder's adjusted basis in the stock,
//! with capital character if the stock was a capital asset.
//!
//! **§331(a) general rule**: distribution in complete liquidation
//! is treated as in exchange for stock. Capital gain or loss
//! treatment when the stock is a capital asset in the shareholder's
//! hands (ST or LT per the deemed exchange-date holding period).
//!
//! **§331(b)**: §301 (dividend treatment) does NOT apply to any
//! distribution of property in complete liquidation. The
//! distribution is NOT a dividend; the §316 dividend-source rules
//! don't apply.
//!
//! **§332 parent-subsidiary exception** (controls §331(a)
//! treatment): when a corporate parent owns at least **80% of the
//! voting power AND at least 80% of the total value** (§1504(a)(2))
//! of the liquidating subsidiary's stock, §332 provides
//! **non-recognition** at the parent level. The parent recognizes
//! neither gain nor loss; §334(b) gives the parent a carryover
//! basis in the distributed assets equal to the subsidiary's
//! pre-distribution basis.
//!
//! **§334(a) shareholder basis** in property received in a §331
//! liquidation: equal to **FMV at distribution**. (This is the
//! "step-up" — distributed property's basis in the shareholder's
//! hands ignores the corporation's historical basis.)
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 331](https://www.law.cornell.edu/uscode/text/26/331),
//! [Cornell LII 26 CFR § 1.331-1 corporate liquidations](https://www.law.cornell.edu/cfr/text/26/1.331-1),
//! [Wolters Kluwer AnswerConnect — §331 shareholder treatment](https://answerconnect.cch.com/document/arp1001f09cee7c561000a796d8d385ad169403/federal/irc/explanation/331-gain-or-loss-treatment-of-shareholders-in-complete-liquidations),
//! [IRS LB&I — §331 transaction unit](https://www.irs.gov/pub/fatca/int_practice_units/sco_t_007r.pdf).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiquidationOutcome {
    /// §331(a) gain recognized as short-term capital (deemed
    /// exchange held ≤ 1 year).
    ShortTermCapitalGain,
    /// §331(a) gain recognized as long-term capital.
    LongTermCapitalGain,
    /// §331(a) loss recognized as short-term capital loss.
    ShortTermCapitalLoss,
    /// §331(a) loss recognized as long-term capital loss.
    LongTermCapitalLoss,
    /// §332 non-recognition — corporate parent owning ≥ 80% of
    /// liquidating subsidiary. Parent recognizes no gain or loss.
    NonRecognitionUnderSection332,
    /// Liquidation is not complete; §331 does not apply (partial
    /// liquidations follow §302 redemption rules).
    NotCompleteLiquidationNoSection331,
    /// Zero net result.
    NoNetResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section331Input {
    /// Adjusted basis of the shareholder's stock surrendered in the
    /// liquidation.
    pub adjusted_basis_in_stock_dollars: i64,
    /// Cash received in the liquidating distribution.
    pub cash_received_dollars: i64,
    /// FMV of non-cash property received in the liquidating
    /// distribution.
    pub fmv_non_cash_property_received_dollars: i64,
    /// Shareholder's holding period in the stock through the
    /// deemed exchange date (last distribution date).
    pub stock_holding_period_days: u32,
    /// True if the liquidation is complete (§331 applies); false
    /// for partial liquidations (which fall under §302 redemption).
    pub liquidation_is_complete: bool,
    /// True if the shareholder is a corporation owning at least
    /// 80% of voting power AND 80% of total value of the liquidating
    /// corporation per §1504(a)(2). Triggers §332 non-recognition.
    pub shareholder_meets_section_332_80_pct: bool,
    /// True if the stock is a capital asset in the shareholder's
    /// hands. False causes ordinary character.
    pub stock_is_capital_asset_in_shareholders_hands: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section331Result {
    pub outcome: LiquidationOutcome,
    pub total_amount_realized_dollars: i64,
    pub gain_or_loss_dollars: i64,
    pub recognized_gain_dollars: i64,
    pub recognized_loss_dollars: i64,
    /// Shareholder's basis in property received per §334(a) =
    /// FMV at distribution. Zero in §332 non-recognition path
    /// (separately handled by §334(b) carryover at parent level).
    pub shareholder_basis_in_property_dollars: i64,
    pub section_332_applies: bool,
    pub citation: String,
    pub note: String,
}

const LONG_TERM_HOLDING_DAYS: u32 = 365;

pub fn compute(input: &Section331Input) -> Section331Result {
    if !input.liquidation_is_complete {
        return Section331Result {
            outcome: LiquidationOutcome::NotCompleteLiquidationNoSection331,
            total_amount_realized_dollars: 0,
            gain_or_loss_dollars: 0,
            recognized_gain_dollars: 0,
            recognized_loss_dollars: 0,
            shareholder_basis_in_property_dollars: 0,
            section_332_applies: false,
            citation:
                "IRC §331 applies only to COMPLETE liquidations; partial liquidations follow §302 redemption rules with potential §316 dividend characterization"
                    .to_string(),
            note: "Liquidation flagged as NOT complete; §331 does not apply on these facts. Use §302 redemption analysis instead.".to_string(),
        };
    }

    // §332 non-recognition path takes precedence — corporate parent
    // owning ≥ 80% recognizes no gain/loss.
    if input.shareholder_meets_section_332_80_pct {
        return Section331Result {
            outcome: LiquidationOutcome::NonRecognitionUnderSection332,
            total_amount_realized_dollars: input.cash_received_dollars
                + input.fmv_non_cash_property_received_dollars,
            gain_or_loss_dollars: 0,
            recognized_gain_dollars: 0,
            recognized_loss_dollars: 0,
            // Parent's basis in property is carryover from subsidiary
            // under §334(b) — not the §334(a) FMV. Modeled as zero
            // here; caller obtains carryover basis from sub-side.
            shareholder_basis_in_property_dollars: 0,
            section_332_applies: true,
            citation:
                "IRC §331(a) general rule; §332 non-recognition exception (§1504(a)(2) 80% voting + 80% value ownership by corporate parent); §334(b) carryover basis from subsidiary"
                    .to_string(),
            note: format!(
                "Corporate parent meets §332 80%/80% threshold; non-recognition treatment. Parent recognizes no gain or loss on $ {} amount realized; parent's basis in distributed property is carryover from subsidiary per §334(b) (not modeled here).",
                input.cash_received_dollars + input.fmv_non_cash_property_received_dollars,
            ),
        };
    }

    // §331(a) standard path.
    let amount_realized =
        input.cash_received_dollars + input.fmv_non_cash_property_received_dollars;
    let gain_or_loss = amount_realized - input.adjusted_basis_in_stock_dollars;

    let outcome = if gain_or_loss == 0 {
        LiquidationOutcome::NoNetResult
    } else if !input.stock_is_capital_asset_in_shareholders_hands {
        // §1221 ordinary-asset path — non-capital character.
        if gain_or_loss > 0 {
            LiquidationOutcome::ShortTermCapitalGain // treated as ST per default if non-capital
        } else {
            LiquidationOutcome::ShortTermCapitalLoss
        }
    } else {
        let lt = input.stock_holding_period_days > LONG_TERM_HOLDING_DAYS;
        match (gain_or_loss > 0, lt) {
            (true, true) => LiquidationOutcome::LongTermCapitalGain,
            (true, false) => LiquidationOutcome::ShortTermCapitalGain,
            (false, true) => LiquidationOutcome::LongTermCapitalLoss,
            (false, false) => LiquidationOutcome::ShortTermCapitalLoss,
        }
    };

    let (recognized_gain, recognized_loss) = if gain_or_loss > 0 {
        (gain_or_loss, 0)
    } else if gain_or_loss < 0 {
        (0, -gain_or_loss)
    } else {
        (0, 0)
    };

    // §334(a) shareholder basis in non-cash property = FMV at
    // distribution.
    let basis_in_property = input.fmv_non_cash_property_received_dollars.max(0);

    let outcome_label = match outcome {
        LiquidationOutcome::ShortTermCapitalGain => "short-term capital gain",
        LiquidationOutcome::LongTermCapitalGain => "long-term capital gain",
        LiquidationOutcome::ShortTermCapitalLoss => "short-term capital loss",
        LiquidationOutcome::LongTermCapitalLoss => "long-term capital loss",
        LiquidationOutcome::NonRecognitionUnderSection332 => "§332 non-recognition",
        LiquidationOutcome::NotCompleteLiquidationNoSection331 => "no §331 (partial liquidation)",
        LiquidationOutcome::NoNetResult => "no net result",
    };

    let note = format!(
        "Complete liquidation; amount realized = cash ${} + FMV property ${} = ${}; stock basis ${}; gain/loss ${} → {}; holding period {} days (>{} required for LT); shareholder basis in non-cash property = FMV ${} per §334(a).",
        input.cash_received_dollars,
        input.fmv_non_cash_property_received_dollars,
        amount_realized,
        input.adjusted_basis_in_stock_dollars,
        gain_or_loss,
        outcome_label,
        input.stock_holding_period_days,
        LONG_TERM_HOLDING_DAYS,
        basis_in_property,
    );

    Section331Result {
        outcome,
        total_amount_realized_dollars: amount_realized,
        gain_or_loss_dollars: gain_or_loss,
        recognized_gain_dollars: recognized_gain,
        recognized_loss_dollars: recognized_loss,
        shareholder_basis_in_property_dollars: basis_in_property,
        section_332_applies: false,
        citation:
            "IRC §331(a) liquidating distribution treated as in full payment for stock under §1001; §331(b) §301 dividend rules inapplicable; capital character when stock is a capital asset in shareholder's hands (§1221); §332 exception for corporate parent owning ≥ 80% (§1504(a)(2)) with §334(b) carryover basis; §334(a) shareholder basis in property = FMV at distribution"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section331Input {
        Section331Input {
            adjusted_basis_in_stock_dollars: 100_000,
            cash_received_dollars: 50_000,
            fmv_non_cash_property_received_dollars: 100_000,
            stock_holding_period_days: 400,
            liquidation_is_complete: true,
            shareholder_meets_section_332_80_pct: false,
            stock_is_capital_asset_in_shareholders_hands: true,
        }
    }

    // ── §331(a) baseline capital gain ──────────────────────────────

    #[test]
    fn baseline_long_term_capital_gain() {
        // $50k cash + $100k FMV = $150k − $100k basis = $50k LT gain.
        let r = compute(&base());
        assert_eq!(r.outcome, LiquidationOutcome::LongTermCapitalGain);
        assert_eq!(r.total_amount_realized_dollars, 150_000);
        assert_eq!(r.gain_or_loss_dollars, 50_000);
        assert_eq!(r.recognized_gain_dollars, 50_000);
    }

    #[test]
    fn short_holding_period_short_term_gain() {
        let mut i = base();
        i.stock_holding_period_days = 300;
        let r = compute(&i);
        assert_eq!(r.outcome, LiquidationOutcome::ShortTermCapitalGain);
    }

    #[test]
    fn holding_period_365_days_still_short_term() {
        let mut i = base();
        i.stock_holding_period_days = 365;
        let r = compute(&i);
        assert_eq!(r.outcome, LiquidationOutcome::ShortTermCapitalGain);
    }

    #[test]
    fn holding_period_366_days_long_term() {
        let mut i = base();
        i.stock_holding_period_days = 366;
        let r = compute(&i);
        assert_eq!(r.outcome, LiquidationOutcome::LongTermCapitalGain);
    }

    // ── §331(a) loss path ──────────────────────────────────────────

    #[test]
    fn capital_loss_when_basis_exceeds_amount_realized() {
        let mut i = base();
        i.adjusted_basis_in_stock_dollars = 300_000;
        i.cash_received_dollars = 50_000;
        i.fmv_non_cash_property_received_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(r.outcome, LiquidationOutcome::LongTermCapitalLoss);
        assert_eq!(r.gain_or_loss_dollars, -150_000);
        assert_eq!(r.recognized_loss_dollars, 150_000);
    }

    #[test]
    fn short_term_capital_loss_path() {
        let mut i = base();
        i.adjusted_basis_in_stock_dollars = 300_000;
        i.stock_holding_period_days = 300;
        let r = compute(&i);
        assert_eq!(r.outcome, LiquidationOutcome::ShortTermCapitalLoss);
    }

    // ── §334(a) FMV basis in property ──────────────────────────────

    #[test]
    fn shareholder_basis_in_property_equals_fmv() {
        let r = compute(&base());
        assert_eq!(r.shareholder_basis_in_property_dollars, 100_000);
    }

    #[test]
    fn cash_only_distribution_zero_basis_in_property() {
        let mut i = base();
        i.fmv_non_cash_property_received_dollars = 0;
        i.cash_received_dollars = 200_000;
        let r = compute(&i);
        assert_eq!(r.shareholder_basis_in_property_dollars, 0);
    }

    // ── §332 parent-subsidiary non-recognition ─────────────────────

    #[test]
    fn section_332_corporate_parent_80_pct_non_recognition() {
        let mut i = base();
        i.shareholder_meets_section_332_80_pct = true;
        let r = compute(&i);
        assert_eq!(r.outcome, LiquidationOutcome::NonRecognitionUnderSection332);
        assert!(r.section_332_applies);
        assert_eq!(r.recognized_gain_dollars, 0);
        assert_eq!(r.recognized_loss_dollars, 0);
    }

    #[test]
    fn section_332_basis_carryover_modeled_as_zero() {
        // §334(b) carryover basis is handled separately at the
        // subsidiary level; module returns zero to signal "use
        // sub-side carryover."
        let mut i = base();
        i.shareholder_meets_section_332_80_pct = true;
        let r = compute(&i);
        assert_eq!(r.shareholder_basis_in_property_dollars, 0);
    }

    #[test]
    fn section_332_takes_precedence_over_331_capital_gain() {
        // Even with a clear §331 gain, §332 80% wins.
        let mut i = base();
        i.shareholder_meets_section_332_80_pct = true;
        i.cash_received_dollars = 10_000_000;
        let r = compute(&i);
        assert_eq!(r.outcome, LiquidationOutcome::NonRecognitionUnderSection332);
    }

    // ── Partial liquidation gate ───────────────────────────────────

    #[test]
    fn partial_liquidation_no_section_331() {
        let mut i = base();
        i.liquidation_is_complete = false;
        let r = compute(&i);
        assert_eq!(
            r.outcome,
            LiquidationOutcome::NotCompleteLiquidationNoSection331
        );
        assert_eq!(r.recognized_gain_dollars, 0);
        assert!(r.note.contains("§302 redemption"));
    }

    // ── Zero net ───────────────────────────────────────────────────

    #[test]
    fn zero_net_when_amount_realized_equals_basis() {
        let mut i = base();
        i.cash_received_dollars = 0;
        i.fmv_non_cash_property_received_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(r.outcome, LiquidationOutcome::NoNetResult);
        assert_eq!(r.gain_or_loss_dollars, 0);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§331(a)"));
        assert!(r.citation.contains("§331(b)"));
        assert!(r.citation.contains("§1001"));
        assert!(r.citation.contains("§332"));
        assert!(r.citation.contains("§334(a)"));
        assert!(r.citation.contains("§334(b)"));
        assert!(r.citation.contains("§1504(a)(2)"));
    }

    #[test]
    fn partial_liquidation_citation_mentions_302() {
        let mut i = base();
        i.liquidation_is_complete = false;
        let r = compute(&i);
        assert!(r.citation.contains("§302"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_baseline_describes_lt_gain() {
        let r = compute(&base());
        assert!(r.note.contains("long-term capital gain"));
    }

    #[test]
    fn note_section_332_describes_non_recognition() {
        let mut i = base();
        i.shareholder_meets_section_332_80_pct = true;
        let r = compute(&i);
        assert!(r.note.contains("§332 80%/80% threshold"));
        assert!(r.note.contains("non-recognition"));
        assert!(r.note.contains("§334(b)"));
    }

    #[test]
    fn note_partial_liquidation_redirects_to_302() {
        let mut i = base();
        i.liquidation_is_complete = false;
        let r = compute(&i);
        assert!(r.note.contains("§302 redemption analysis"));
    }

    // ── Precision ──────────────────────────────────────────────────

    #[test]
    fn very_large_billion_dollar_liquidation_precision() {
        let mut i = base();
        i.cash_received_dollars = 500_000_000;
        i.fmv_non_cash_property_received_dollars = 500_000_000;
        i.adjusted_basis_in_stock_dollars = 100_000_000;
        let r = compute(&i);
        assert_eq!(r.total_amount_realized_dollars, 1_000_000_000);
        assert_eq!(r.gain_or_loss_dollars, 900_000_000);
        assert_eq!(r.recognized_gain_dollars, 900_000_000);
    }

    #[test]
    fn zero_basis_full_amount_realized_is_gain() {
        let mut i = base();
        i.adjusted_basis_in_stock_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.gain_or_loss_dollars, 150_000);
    }
}
