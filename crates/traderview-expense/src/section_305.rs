//! IRC §305 — Distributions of stock and stock rights.
//!
//! Trader-relevant for anyone holding corporate stock who receives
//! a stock dividend or stock right. §305(a) general rule excludes
//! stock-on-stock distributions from gross income (the
//! distribution simply changes the per-share basis under §307(a)).
//! §305(b) enumerates **5 exceptions** that make a stock
//! distribution taxable as a §301 distribution. §305(c) creates
//! deemed distributions from certain capital-structure events.
//!
//! **§305(a) general rule**: gross income does NOT include any
//! distribution of stock made by a corporation to its
//! shareholders with respect to its stock. The non-cash dividend
//! is treated as a pure capital re-organization at the
//! shareholder level.
//!
//! **§305(b) five taxable exceptions**:
//!
//! - **(b)(1) In lieu of money**: any shareholder may elect cash
//!   or property instead of stock. The mere existence of the
//!   election triggers taxability for ALL shareholders.
//! - **(b)(2) Disproportionate distributions**: some shareholders
//!   receive property while others' proportionate interest
//!   increases — the increased-interest shareholders are taxable.
//! - **(b)(3) Common-and-preferred**: some common shareholders
//!   receive preferred stock while others receive common.
//! - **(b)(4) Distributions on preferred stock**: any distribution
//!   on preferred stock (except conversion-ratio adjustments to
//!   account for splits/dividends on the underlying).
//! - **(b)(5) Convertible preferred**: distribution of convertible
//!   preferred stock unless shown not to result in a
//!   disproportionate distribution.
//!
//! **§305(c) deemed distributions**: certain capital-structure
//! events (conversion-ratio adjustments to maintain anti-dilution
//! protection, redemption premium accruals, etc.) are treated as
//! deemed distributions even though no actual stock is
//! distributed. Taxable only if the triggering event is taxable to
//! actual shareholders AND the convertible holder has an increased
//! proportional interest in E&P.
//!
//! **§307(a) basis allocation**: when §305(a) applies (non-taxable
//! stock dividend), the shareholder allocates the basis of the old
//! stock between old and new shares in proportion to FMV. Holding
//! period of old shares tacks to the new shares under §1223(5).
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 305](https://www.law.cornell.edu/uscode/text/26/305),
//! [IRS Bloomberg Tax — Sec. 305 Distributions of Stock and Stock Rights](https://irc.bloombergtax.com/public/uscode/doc/irc/section_305),
//! [Cornell LII 26 CFR § 1.305-3 — Disproportionate distributions](https://www.taxnotes.com/research/federal/cfr26/1.305-3),
//! [Cornell LII 26 CFR § 1.305-6 — Distributions of convertible preferred](https://www.law.cornell.edu/cfr/text/26/1.305-6),
//! [U.S. Bank — IRC § 305(c) Deemed Distributions](https://www.usbank.com/financialiq/plan-your-growth/trends-and-analysis/IRC-Section-305c-Deemed-distributions-and-related-regulations.html).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section305Exception {
    /// §305(b)(1) — any shareholder may elect cash instead of stock.
    InLieuOfMoney,
    /// §305(b)(2) — some get property; others' interest increases.
    DisproportionateDistribution,
    /// §305(b)(3) — common shareholders split between common and
    /// preferred receipts.
    CommonAndPreferred,
    /// §305(b)(4) — distribution on preferred stock.
    DistributionOnPreferredStock,
    /// §305(b)(5) — convertible preferred without anti-dilution
    /// safe harbor.
    ConvertiblePreferred,
    /// §305(c) — deemed distribution from capital-structure event.
    DeemedDistribution,
    /// No exception triggered — §305(a) non-taxable general rule.
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section305Input {
    /// FMV of the stock distributed.
    pub stock_distributed_fmv_dollars: i64,
    /// Shareholder's adjusted basis in the OLD shares (pre-
    /// distribution).
    pub old_share_adjusted_basis_dollars: i64,
    /// FMV of the OLD shares at the time of distribution.
    pub old_share_fmv_dollars: i64,
    /// True if any shareholder may elect to receive cash or other
    /// property instead of the stock distribution (§305(b)(1)).
    pub any_shareholder_may_elect_cash: bool,
    /// True if the distribution results in some shareholders
    /// receiving property and others' proportionate interest
    /// increasing (§305(b)(2)).
    pub distribution_is_disproportionate: bool,
    /// True if some common shareholders receive preferred stock
    /// while others receive common (§305(b)(3)).
    pub common_and_preferred_split: bool,
    /// True if the distribution is on preferred stock without the
    /// conversion-ratio safe-harbor exception (§305(b)(4)).
    pub distribution_on_preferred_stock: bool,
    /// True if distribution is convertible preferred stock without
    /// the anti-dilution safe harbor (§305(b)(5)).
    pub convertible_preferred_without_safe_harbor: bool,
    /// True if this is a §305(c) deemed distribution from a
    /// capital-structure event.
    pub is_section_305c_deemed_distribution: bool,
    /// Corporation's current and accumulated E&P (for §301
    /// dividend characterization when distribution is taxable).
    pub corporation_earnings_and_profits_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section305Result {
    pub triggered_exception: Section305Exception,
    pub distribution_is_taxable: bool,
    /// §301 dividend income to shareholder (limited by E&P).
    pub dividend_income_dollars: i64,
    /// Amount in excess of E&P treated as basis recovery (then
    /// capital gain).
    pub basis_recovery_or_capital_gain_dollars: i64,
    /// §307(a) allocated basis to NEW shares when §305(a) applies.
    pub new_share_basis_dollars: i64,
    /// §307(a) reduced basis remaining on OLD shares.
    pub old_share_basis_after_allocation_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section305Input) -> Section305Result {
    // Determine which §305(b) exception (if any) triggers.
    // Priority order: (b)(1) → (b)(2) → (b)(3) → (b)(4) → (b)(5)
    // → §305(c).
    let exception = if input.any_shareholder_may_elect_cash {
        Section305Exception::InLieuOfMoney
    } else if input.distribution_is_disproportionate {
        Section305Exception::DisproportionateDistribution
    } else if input.common_and_preferred_split {
        Section305Exception::CommonAndPreferred
    } else if input.distribution_on_preferred_stock {
        Section305Exception::DistributionOnPreferredStock
    } else if input.convertible_preferred_without_safe_harbor {
        Section305Exception::ConvertiblePreferred
    } else if input.is_section_305c_deemed_distribution {
        Section305Exception::DeemedDistribution
    } else {
        Section305Exception::None
    };

    let taxable = !matches!(exception, Section305Exception::None);

    let stock_fmv = input.stock_distributed_fmv_dollars.max(0);
    let old_basis = input.old_share_adjusted_basis_dollars.max(0);
    let old_fmv = input.old_share_fmv_dollars.max(0);
    let e_and_p = input.corporation_earnings_and_profits_dollars.max(0);

    let (dividend_income, basis_recovery, new_basis, old_basis_after) = if taxable {
        // §301 distribution treatment: dividend up to E&P, then
        // basis recovery, then capital gain. New shares take FMV
        // basis (§301(d)).
        let dividend = stock_fmv.min(e_and_p);
        let excess = stock_fmv - dividend;
        (dividend, excess, stock_fmv, old_basis)
    } else {
        // §305(a) non-taxable: §307(a) basis allocation between
        // old and new shares in proportion to FMV.
        let total_fmv = old_fmv + stock_fmv;
        let new_share_allocated = if total_fmv > 0 {
            ((old_basis as i128) * (stock_fmv as i128) / (total_fmv as i128)) as i64
        } else {
            0
        };
        let old_share_remaining = old_basis - new_share_allocated;
        (0, 0, new_share_allocated, old_share_remaining)
    };

    let exception_label = match exception {
        Section305Exception::None => "§305(a) general rule — non-taxable",
        Section305Exception::InLieuOfMoney => {
            "§305(b)(1) in lieu of money — taxable (any-shareholder cash election)"
        }
        Section305Exception::DisproportionateDistribution => {
            "§305(b)(2) disproportionate distribution — taxable"
        }
        Section305Exception::CommonAndPreferred => {
            "§305(b)(3) common-and-preferred split — taxable"
        }
        Section305Exception::DistributionOnPreferredStock => {
            "§305(b)(4) distribution on preferred stock — taxable"
        }
        Section305Exception::ConvertiblePreferred => {
            "§305(b)(5) convertible preferred without safe harbor — taxable"
        }
        Section305Exception::DeemedDistribution => {
            "§305(c) deemed distribution from capital-structure event — taxable"
        }
    };

    let note = format!(
        "Stock distributed FMV ${}; old share basis ${}; old share FMV ${}; E&P ${}; result: {}; {} ${} dividend income, ${} basis recovery/capital gain; new share basis ${}; old share basis after §307(a) allocation ${}.",
        stock_fmv,
        old_basis,
        old_fmv,
        e_and_p,
        exception_label,
        if taxable { "taxable as §301 distribution:" } else { "non-taxable; §307(a) basis allocation:" },
        dividend_income,
        basis_recovery,
        new_basis,
        old_basis_after,
    );

    Section305Result {
        triggered_exception: exception,
        distribution_is_taxable: taxable,
        dividend_income_dollars: dividend_income,
        basis_recovery_or_capital_gain_dollars: basis_recovery,
        new_share_basis_dollars: new_basis,
        old_share_basis_after_allocation_dollars: old_basis_after,
        citation:
            "IRC §305(a) general rule: stock-on-stock distributions excluded from gross income; §305(b) 5 taxable exceptions (in lieu of money, disproportionate, common-and-preferred split, distribution on preferred, convertible preferred without anti-dilution safe harbor); §305(c) deemed distributions from capital-structure events; §307(a) basis allocation between old and new shares in proportion to FMV when §305(a) applies; §1223(5) holding period tacks; §301 distribution treatment when taxable (dividend up to E&P, then basis recovery, then capital gain)"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section305Input {
        Section305Input {
            stock_distributed_fmv_dollars: 1_000,
            old_share_adjusted_basis_dollars: 10_000,
            old_share_fmv_dollars: 19_000,
            any_shareholder_may_elect_cash: false,
            distribution_is_disproportionate: false,
            common_and_preferred_split: false,
            distribution_on_preferred_stock: false,
            convertible_preferred_without_safe_harbor: false,
            is_section_305c_deemed_distribution: false,
            corporation_earnings_and_profits_dollars: 50_000,
        }
    }

    // ── §305(a) general rule — non-taxable ─────────────────────────

    #[test]
    fn no_exception_non_taxable() {
        let r = compute(&base());
        assert_eq!(r.triggered_exception, Section305Exception::None);
        assert!(!r.distribution_is_taxable);
        assert_eq!(r.dividend_income_dollars, 0);
    }

    #[test]
    fn section_307a_basis_allocation() {
        // Old basis $10k, old FMV $19k, new FMV $1k.
        // Total FMV = $20k; new share allocation = $10k × ($1k / $20k) = $500.
        // Old basis after = $10k − $500 = $9,500.
        let r = compute(&base());
        assert_eq!(r.new_share_basis_dollars, 500);
        assert_eq!(r.old_share_basis_after_allocation_dollars, 9_500);
    }

    // ── §305(b)(1) in lieu of money ────────────────────────────────

    #[test]
    fn in_lieu_of_money_triggers_taxability() {
        let mut i = base();
        i.any_shareholder_may_elect_cash = true;
        let r = compute(&i);
        assert_eq!(r.triggered_exception, Section305Exception::InLieuOfMoney);
        assert!(r.distribution_is_taxable);
        assert_eq!(r.dividend_income_dollars, 1_000);
    }

    // ── §305(b)(2) disproportionate ────────────────────────────────

    #[test]
    fn disproportionate_distribution_triggers_taxability() {
        let mut i = base();
        i.distribution_is_disproportionate = true;
        let r = compute(&i);
        assert_eq!(
            r.triggered_exception,
            Section305Exception::DisproportionateDistribution
        );
        assert!(r.distribution_is_taxable);
    }

    // ── §305(b)(3) common-and-preferred ────────────────────────────

    #[test]
    fn common_and_preferred_split_triggers_taxability() {
        let mut i = base();
        i.common_and_preferred_split = true;
        let r = compute(&i);
        assert_eq!(
            r.triggered_exception,
            Section305Exception::CommonAndPreferred
        );
        assert!(r.distribution_is_taxable);
    }

    // ── §305(b)(4) distribution on preferred ───────────────────────

    #[test]
    fn distribution_on_preferred_stock_triggers_taxability() {
        let mut i = base();
        i.distribution_on_preferred_stock = true;
        let r = compute(&i);
        assert_eq!(
            r.triggered_exception,
            Section305Exception::DistributionOnPreferredStock
        );
        assert!(r.distribution_is_taxable);
    }

    // ── §305(b)(5) convertible preferred ───────────────────────────

    #[test]
    fn convertible_preferred_without_safe_harbor_triggers_taxability() {
        let mut i = base();
        i.convertible_preferred_without_safe_harbor = true;
        let r = compute(&i);
        assert_eq!(
            r.triggered_exception,
            Section305Exception::ConvertiblePreferred
        );
    }

    // ── §305(c) deemed distribution ────────────────────────────────

    #[test]
    fn section_305c_deemed_distribution_triggers_taxability() {
        let mut i = base();
        i.is_section_305c_deemed_distribution = true;
        let r = compute(&i);
        assert_eq!(
            r.triggered_exception,
            Section305Exception::DeemedDistribution
        );
        assert!(r.distribution_is_taxable);
    }

    // ── Exception priority ordering ────────────────────────────────

    #[test]
    fn b1_in_lieu_short_circuits_other_exceptions() {
        // §305(b)(1) wins when both b(1) and b(2) trigger.
        let mut i = base();
        i.any_shareholder_may_elect_cash = true;
        i.distribution_is_disproportionate = true;
        let r = compute(&i);
        assert_eq!(r.triggered_exception, Section305Exception::InLieuOfMoney);
    }

    #[test]
    fn b2_disproportionate_short_circuits_lower_priority() {
        let mut i = base();
        i.distribution_is_disproportionate = true;
        i.distribution_on_preferred_stock = true;
        let r = compute(&i);
        assert_eq!(
            r.triggered_exception,
            Section305Exception::DisproportionateDistribution
        );
    }

    #[test]
    fn deemed_distribution_lowest_priority() {
        // §305(c) only fires when no §305(b) exception triggers.
        let mut i = base();
        i.distribution_is_disproportionate = true;
        i.is_section_305c_deemed_distribution = true;
        let r = compute(&i);
        assert_ne!(
            r.triggered_exception,
            Section305Exception::DeemedDistribution
        );
    }

    // ── §301 dividend treatment when taxable ───────────────────────

    #[test]
    fn taxable_distribution_dividend_up_to_e_and_p() {
        // $1k distribution × E&P $50k → $1k all dividend.
        let mut i = base();
        i.any_shareholder_may_elect_cash = true;
        let r = compute(&i);
        assert_eq!(r.dividend_income_dollars, 1_000);
        assert_eq!(r.basis_recovery_or_capital_gain_dollars, 0);
    }

    #[test]
    fn taxable_distribution_excess_over_e_and_p_basis_recovery() {
        // $1k distribution × E&P $200 → $200 dividend + $800 basis
        // recovery.
        let mut i = base();
        i.any_shareholder_may_elect_cash = true;
        i.corporation_earnings_and_profits_dollars = 200;
        let r = compute(&i);
        assert_eq!(r.dividend_income_dollars, 200);
        assert_eq!(r.basis_recovery_or_capital_gain_dollars, 800);
    }

    #[test]
    fn taxable_distribution_zero_e_and_p_all_basis_recovery() {
        let mut i = base();
        i.any_shareholder_may_elect_cash = true;
        i.corporation_earnings_and_profits_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.dividend_income_dollars, 0);
        assert_eq!(r.basis_recovery_or_capital_gain_dollars, 1_000);
    }

    #[test]
    fn taxable_distribution_new_basis_equals_fmv() {
        let mut i = base();
        i.any_shareholder_may_elect_cash = true;
        let r = compute(&i);
        // §301(d): taxable distribution gives new shares FMV basis.
        assert_eq!(r.new_share_basis_dollars, 1_000);
    }

    #[test]
    fn taxable_distribution_old_share_basis_unchanged() {
        let mut i = base();
        i.any_shareholder_may_elect_cash = true;
        let r = compute(&i);
        // Old shares retain pre-distribution basis when taxable.
        assert_eq!(r.old_share_basis_after_allocation_dollars, 10_000);
    }

    // ── §307(a) edge cases ─────────────────────────────────────────

    #[test]
    fn non_taxable_zero_fmv_old_share_no_allocation() {
        let mut i = base();
        i.old_share_fmv_dollars = 0;
        i.stock_distributed_fmv_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.new_share_basis_dollars, 0);
        assert_eq!(r.old_share_basis_after_allocation_dollars, 10_000);
    }

    #[test]
    fn non_taxable_proportional_basis_split() {
        // Old basis $10k, old FMV $10k, new FMV $10k.
        // Equal split: $5k each.
        let mut i = base();
        i.old_share_adjusted_basis_dollars = 10_000;
        i.old_share_fmv_dollars = 10_000;
        i.stock_distributed_fmv_dollars = 10_000;
        let r = compute(&i);
        assert_eq!(r.new_share_basis_dollars, 5_000);
        assert_eq!(r.old_share_basis_after_allocation_dollars, 5_000);
    }

    // ── Citation ───────────────────────────────────────────────────

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§305(a)"));
        assert!(r.citation.contains("§305(b)"));
        assert!(r.citation.contains("§305(c)"));
        assert!(r.citation.contains("§307(a)"));
        assert!(r.citation.contains("§1223(5)"));
        assert!(r.citation.contains("§301"));
        assert!(r.citation.contains("5 taxable exceptions"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_non_taxable_describes_general_rule() {
        let r = compute(&base());
        assert!(r.note.contains("§305(a) general rule — non-taxable"));
    }

    #[test]
    fn note_taxable_describes_exception() {
        let mut i = base();
        i.any_shareholder_may_elect_cash = true;
        let r = compute(&i);
        assert!(r.note.contains("§305(b)(1)"));
        assert!(r.note.contains("taxable as §301 distribution"));
    }

    // ── Precision ──────────────────────────────────────────────────

    #[test]
    fn very_large_distribution_precision() {
        let mut i = base();
        i.stock_distributed_fmv_dollars = 1_000_000_000;
        i.old_share_adjusted_basis_dollars = 5_000_000_000;
        i.old_share_fmv_dollars = 9_000_000_000;
        let r = compute(&i);
        // Total FMV = $10B; new share allocation = $5B × $1B / $10B = $500M.
        assert_eq!(r.new_share_basis_dollars, 500_000_000);
        assert_eq!(r.old_share_basis_after_allocation_dollars, 4_500_000_000);
    }
}
