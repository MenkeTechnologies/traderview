//! IRC §475(c)(2) dealer-in-securities classification + §475(f)
//! trader mark-to-market election eligibility.
//!
//! Foundational rule for any active trader: are they a DEALER (§475(a)
//! MANDATORY mark-to-market, ordinary income), a TRADER (eligible to
//! ELECT §475(f) MTM, capital character default), or an INVESTOR
//! (capital character only, no MTM election available)?
//!
//! The four-way classification controls **every** downstream tax
//! treatment in the system — wash-sale applicability, $3k capital loss
//! cap, ordinary-vs-capital character, deductibility of trading
//! expenses, applicability of §163(d) investment-interest limit vs
//! §163(j) business-interest limit, etc.
//!
//! **§475(c)(2) dealer-in-securities definition** — taxpayer is a
//! dealer if EITHER prong is satisfied:
//!
//! - **Customer prong (§475(c)(1)(A))**: regularly purchases securities
//!   from OR sells securities to customers in the ordinary course of a
//!   trade or business.
//! - **Inventory prong (§475(c)(1)(B))**: regularly offers to enter
//!   into, assume, offset, assign, or otherwise terminate positions
//!   in securities with customers in the ordinary course of a trade
//!   or business.
//!
//! **Negligible-sales exception** (Treas. Reg. § 1.475(c)-1(c)(1)):
//! a taxpayer who regularly PURCHASES from customers but makes only
//! negligible sales of the securities so acquired is NOT a dealer —
//! unless (a) the taxpayer elects dealer treatment OR (b) accounts
//! for any §475(c)(2) security as inventory under § 471.
//!
//! **Trader-vs-investor distinction** (no statutory definition;
//! determined by case law summarized in IRS Topic 429):
//!
//! A taxpayer engaged in a trade or business as a TRADER must satisfy
//! ALL of:
//! 1. Seek to profit from DAILY market movements (short-term swings),
//!    not from dividends, interest, or long-term capital appreciation.
//! 2. Substantial activity (frequent trades, large volume).
//! 3. Continuous and regular activity — not occasional.
//!
//! Failing any prong → INVESTOR (capital character; cannot elect
//! §475(f)).
//!
//! **§475(f) trader MTM election**: a trader (not investor, not
//! dealer) may elect to mark-to-market all securities held in the
//! trade or business at the close of each taxable year. Effects:
//! - Gains and losses treated as ORDINARY (not capital)
//! - Wash sale rules (§1091) do NOT apply
//! - $3k capital loss limit (§1211) does NOT apply
//! - Trader expenses deductible above the line on Schedule C
//! - Election made by attaching a statement to a timely-filed return
//!   for the PRIOR year (e.g., 2024 election made on 2023 return)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    Dealer,
    TraderWithMtmElection,
    TraderWithoutMtmElection,
    Investor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryIncomeSource {
    ShortTermPriceSwings,
    DividendsAndInterest,
    LongTermAppreciation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section475c2Input {
    // Dealer prongs.
    pub has_customers: bool,
    pub regularly_purchases_from_customers: bool,
    pub regularly_sells_to_customers: bool,
    pub regularly_offers_derivative_positions_to_customers: bool,
    /// True if purchases are regular but sales of the purchased
    /// securities are negligible — invokes Treas. Reg. § 1.475(c)-1(c)
    /// exception.
    pub negligible_sales_of_purchased_securities: bool,
    pub accounts_as_inventory_under_section_471: bool,
    pub elected_dealer_treatment: bool,

    // Trader-vs-investor prongs.
    pub trades_for_own_account: bool,
    pub primary_income_source: PrimaryIncomeSource,
    pub trades_per_year: u32,
    pub trading_days_per_year: u32,
    pub average_holding_period_days: u32,

    // §475(f) election.
    pub has_section_475f_election: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section475c2Result {
    pub classification: Classification,
    pub dealer_customer_prong_satisfied: bool,
    pub dealer_inventory_prong_satisfied: bool,
    pub negligible_sales_exception_applies: bool,
    pub trader_short_term_profit_motive: bool,
    pub trader_substantial_activity: bool,
    pub trader_continuous_and_regular: bool,
    pub mtm_required: bool,
    pub mtm_elected: bool,
    pub character_is_ordinary: bool,
    pub wash_sale_rules_apply: bool,
    pub three_k_capital_loss_cap_applies: bool,
    pub citation: String,
    pub note: String,
}

const SUBSTANTIAL_ACTIVITY_TRADES_FLOOR: u32 = 720; // ~3 trades per business day
const CONTINUOUS_REGULAR_TRADING_DAYS_FLOOR: u32 = 175; // 70% of ~250 trading days
const SHORT_TERM_HOLDING_PERIOD_CEILING: u32 = 31; // typical case-law threshold

pub fn compute(input: &Section475c2Input) -> Section475c2Result {
    // §475(c)(2) two-prong dealer test (both prongs gated on
    // having customers).
    let customer_prong_raw = input.has_customers
        && (input.regularly_purchases_from_customers || input.regularly_sells_to_customers);
    let inventory_prong =
        input.has_customers && input.regularly_offers_derivative_positions_to_customers;

    // Negligible-sales exception: regularly purchases BUT negligible
    // sales → not a dealer unless inventory or election.
    let negligible_sales_exception = input.has_customers
        && input.regularly_purchases_from_customers
        && !input.regularly_sells_to_customers
        && input.negligible_sales_of_purchased_securities
        && !input.accounts_as_inventory_under_section_471
        && !input.elected_dealer_treatment;

    let customer_prong_satisfied = customer_prong_raw && !negligible_sales_exception;

    let is_dealer = customer_prong_satisfied
        || inventory_prong
        || input.elected_dealer_treatment
        || input.accounts_as_inventory_under_section_471;

    // Trader prongs (only meaningful if not a dealer).
    let short_term_motive = input.primary_income_source
        == PrimaryIncomeSource::ShortTermPriceSwings
        && input.average_holding_period_days <= SHORT_TERM_HOLDING_PERIOD_CEILING;
    let substantial_activity = input.trades_per_year >= SUBSTANTIAL_ACTIVITY_TRADES_FLOOR;
    let continuous_regular = input.trading_days_per_year >= CONTINUOUS_REGULAR_TRADING_DAYS_FLOOR;

    let qualifies_as_trader = !is_dealer
        && input.trades_for_own_account
        && short_term_motive
        && substantial_activity
        && continuous_regular;

    let classification = if is_dealer {
        Classification::Dealer
    } else if qualifies_as_trader && input.has_section_475f_election {
        Classification::TraderWithMtmElection
    } else if qualifies_as_trader {
        Classification::TraderWithoutMtmElection
    } else {
        Classification::Investor
    };

    let mtm_required = matches!(classification, Classification::Dealer);
    let mtm_elected = matches!(classification, Classification::TraderWithMtmElection);
    let character_is_ordinary = mtm_required || mtm_elected;
    let wash_sale_rules_apply = !character_is_ordinary;
    let three_k_cap_applies = !character_is_ordinary;

    let note = match classification {
        Classification::Dealer => format!(
            "DEALER under §475(c)(2): {}. §475(a) MTM MANDATORY; all gains/losses ORDINARY; no wash-sale rules; no $3k capital loss cap; inventory accounting under § 471.",
            if input.elected_dealer_treatment {
                "elected dealer treatment"
            } else if input.accounts_as_inventory_under_section_471 {
                "accounts as § 471 inventory"
            } else if customer_prong_satisfied && inventory_prong {
                "both customer + inventory prongs satisfied"
            } else if customer_prong_satisfied {
                "customer prong satisfied (regular sales to/purchases from customers)"
            } else {
                "inventory prong satisfied (regularly offers positions to customers)"
            },
        ),
        Classification::TraderWithMtmElection => {
            "TRADER WITH §475(f) MTM ELECTION: gains/losses ORDINARY; wash-sale rules DO NOT apply; $3k capital loss cap DOES NOT apply; expenses deductible above-the-line on Schedule C. Election filed by attaching statement to timely-filed prior-year return.".to_string()
        }
        Classification::TraderWithoutMtmElection => {
            "TRADER WITHOUT §475(f) MTM election: gains/losses CAPITAL (STCG / LTCG); wash-sale rules apply; $3k capital loss cap applies. Trading expenses deductible above-the-line on Schedule C as trade or business expenses.".to_string()
        }
        Classification::Investor => format!(
            "INVESTOR (default classification): failed {} trader prong{}. Gains/losses CAPITAL; wash-sale rules apply; $3k capital loss cap applies; investment expenses miscellaneous itemized deduction (TCJA suspended 2018-2025). Cannot elect §475(f) — not in a trade or business as a trader.",
            {
                let mut failures: Vec<&str> = Vec::new();
                if !input.trades_for_own_account {
                    failures.push("own-account");
                }
                if !short_term_motive {
                    failures.push("short-term-profit-motive");
                }
                if !substantial_activity {
                    failures.push("substantial-activity");
                }
                if !continuous_regular {
                    failures.push("continuous-and-regular");
                }
                if failures.is_empty() {
                    "no".to_string()
                } else {
                    failures.join(" + ")
                }
            },
            if !short_term_motive
                || !substantial_activity
                || !continuous_regular
            {
                "s"
            } else {
                ""
            },
        ),
    };

    Section475c2Result {
        classification,
        dealer_customer_prong_satisfied: customer_prong_satisfied,
        dealer_inventory_prong_satisfied: inventory_prong,
        negligible_sales_exception_applies: negligible_sales_exception,
        trader_short_term_profit_motive: short_term_motive,
        trader_substantial_activity: substantial_activity,
        trader_continuous_and_regular: continuous_regular,
        mtm_required,
        mtm_elected,
        character_is_ordinary,
        wash_sale_rules_apply,
        three_k_capital_loss_cap_applies: three_k_cap_applies,
        citation:
            "IRC §475(c)(2) dealer-in-securities definition; Treas. Reg. §1.475(c)-1 negligible-sales exception; IRC §475(a) dealer MTM mandatory; IRC §475(f)(1) trader MTM election; IRS Topic 429 trader-vs-investor case-law criteria"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn investor_base() -> Section475c2Input {
        Section475c2Input {
            has_customers: false,
            regularly_purchases_from_customers: false,
            regularly_sells_to_customers: false,
            regularly_offers_derivative_positions_to_customers: false,
            negligible_sales_of_purchased_securities: false,
            accounts_as_inventory_under_section_471: false,
            elected_dealer_treatment: false,
            trades_for_own_account: true,
            primary_income_source: PrimaryIncomeSource::LongTermAppreciation,
            trades_per_year: 50,
            trading_days_per_year: 20,
            average_holding_period_days: 365,
            has_section_475f_election: false,
        }
    }

    fn trader_base() -> Section475c2Input {
        Section475c2Input {
            has_customers: false,
            regularly_purchases_from_customers: false,
            regularly_sells_to_customers: false,
            regularly_offers_derivative_positions_to_customers: false,
            negligible_sales_of_purchased_securities: false,
            accounts_as_inventory_under_section_471: false,
            elected_dealer_treatment: false,
            trades_for_own_account: true,
            primary_income_source: PrimaryIncomeSource::ShortTermPriceSwings,
            trades_per_year: 1_000,
            trading_days_per_year: 200,
            average_holding_period_days: 5,
            has_section_475f_election: false,
        }
    }

    fn dealer_base() -> Section475c2Input {
        Section475c2Input {
            has_customers: true,
            regularly_purchases_from_customers: true,
            regularly_sells_to_customers: true,
            regularly_offers_derivative_positions_to_customers: false,
            negligible_sales_of_purchased_securities: false,
            accounts_as_inventory_under_section_471: true,
            elected_dealer_treatment: false,
            trades_for_own_account: false,
            primary_income_source: PrimaryIncomeSource::ShortTermPriceSwings,
            trades_per_year: 10_000,
            trading_days_per_year: 250,
            average_holding_period_days: 1,
            has_section_475f_election: false,
        }
    }

    // Dealer classification.

    #[test]
    fn dealer_customer_prong_satisfied_purchases_and_sells() {
        let r = compute(&dealer_base());
        assert_eq!(r.classification, Classification::Dealer);
        assert!(r.dealer_customer_prong_satisfied);
        assert!(r.mtm_required);
        assert!(r.character_is_ordinary);
        assert!(!r.wash_sale_rules_apply);
        assert!(!r.three_k_capital_loss_cap_applies);
    }

    #[test]
    fn dealer_inventory_prong_alone_suffices() {
        let mut i = dealer_base();
        i.regularly_purchases_from_customers = false;
        i.regularly_sells_to_customers = false;
        i.accounts_as_inventory_under_section_471 = false;
        i.regularly_offers_derivative_positions_to_customers = true;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Dealer);
        assert!(r.dealer_inventory_prong_satisfied);
        assert!(!r.dealer_customer_prong_satisfied);
    }

    #[test]
    fn dealer_election_alone_suffices() {
        let mut i = trader_base();
        i.has_customers = true;
        i.elected_dealer_treatment = true;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Dealer);
    }

    #[test]
    fn dealer_section_471_inventory_alone_suffices() {
        let mut i = trader_base();
        i.has_customers = true;
        i.accounts_as_inventory_under_section_471 = true;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Dealer);
    }

    #[test]
    fn negligible_sales_exception_blocks_dealer_classification() {
        // Regularly purchases from customers but only negligible sales,
        // not on inventory, not elected → NOT a dealer.
        let mut i = dealer_base();
        i.regularly_sells_to_customers = false;
        i.regularly_purchases_from_customers = true;
        i.negligible_sales_of_purchased_securities = true;
        i.accounts_as_inventory_under_section_471 = false;
        i.elected_dealer_treatment = false;
        let r = compute(&i);
        assert!(r.negligible_sales_exception_applies);
        assert_ne!(r.classification, Classification::Dealer);
    }

    #[test]
    fn negligible_sales_exception_yields_to_election() {
        // Negligible-sales pattern + dealer election → STILL dealer.
        let mut i = dealer_base();
        i.regularly_sells_to_customers = false;
        i.negligible_sales_of_purchased_securities = true;
        i.accounts_as_inventory_under_section_471 = false;
        i.elected_dealer_treatment = true;
        let r = compute(&i);
        assert!(!r.negligible_sales_exception_applies);
        assert_eq!(r.classification, Classification::Dealer);
    }

    #[test]
    fn negligible_sales_exception_yields_to_inventory_treatment() {
        let mut i = dealer_base();
        i.regularly_sells_to_customers = false;
        i.negligible_sales_of_purchased_securities = true;
        i.accounts_as_inventory_under_section_471 = true;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Dealer);
    }

    // Trader classification.

    #[test]
    fn trader_with_475f_election_baseline() {
        let mut i = trader_base();
        i.has_section_475f_election = true;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::TraderWithMtmElection);
        assert!(r.mtm_elected);
        assert!(r.character_is_ordinary);
        assert!(!r.wash_sale_rules_apply);
        assert!(!r.three_k_capital_loss_cap_applies);
    }

    #[test]
    fn trader_without_election_keeps_capital_character() {
        let r = compute(&trader_base());
        assert_eq!(r.classification, Classification::TraderWithoutMtmElection);
        assert!(!r.mtm_elected);
        assert!(!r.character_is_ordinary);
        assert!(r.wash_sale_rules_apply);
        assert!(r.three_k_capital_loss_cap_applies);
    }

    #[test]
    fn trader_long_holding_period_demoted_to_investor() {
        // Trader otherwise but holding 60 days → fails short-term motive.
        let mut i = trader_base();
        i.average_holding_period_days = 60;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Investor);
        assert!(!r.trader_short_term_profit_motive);
    }

    #[test]
    fn trader_insufficient_trades_demoted_to_investor() {
        let mut i = trader_base();
        i.trades_per_year = 100;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Investor);
        assert!(!r.trader_substantial_activity);
    }

    #[test]
    fn trader_insufficient_trading_days_demoted_to_investor() {
        let mut i = trader_base();
        i.trading_days_per_year = 50;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Investor);
        assert!(!r.trader_continuous_and_regular);
    }

    #[test]
    fn trader_dividend_motive_demoted_to_investor() {
        let mut i = trader_base();
        i.primary_income_source = PrimaryIncomeSource::DividendsAndInterest;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Investor);
    }

    #[test]
    fn trader_at_exact_floors_qualifies() {
        // 720 trades, 175 days, 31 days holding — all at exact floors.
        let mut i = trader_base();
        i.trades_per_year = 720;
        i.trading_days_per_year = 175;
        i.average_holding_period_days = 31;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::TraderWithoutMtmElection);
    }

    #[test]
    fn trader_just_below_substantial_activity_floor_fails() {
        let mut i = trader_base();
        i.trades_per_year = 719;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Investor);
        assert!(!r.trader_substantial_activity);
    }

    #[test]
    fn trader_just_below_continuous_regular_floor_fails() {
        let mut i = trader_base();
        i.trading_days_per_year = 174;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Investor);
        assert!(!r.trader_continuous_and_regular);
    }

    #[test]
    fn trader_holding_32_days_fails_short_term_motive() {
        let mut i = trader_base();
        i.average_holding_period_days = 32;
        let r = compute(&i);
        assert!(!r.trader_short_term_profit_motive);
    }

    // Investor classification.

    #[test]
    fn investor_baseline() {
        let r = compute(&investor_base());
        assert_eq!(r.classification, Classification::Investor);
        assert!(r.wash_sale_rules_apply);
        assert!(r.three_k_capital_loss_cap_applies);
        assert!(!r.character_is_ordinary);
    }

    #[test]
    fn investor_election_flag_ignored_no_eligibility() {
        // Investor with §475(f) election flag set → still investor;
        // module does not promote to trader without underlying prongs.
        let mut i = investor_base();
        i.has_section_475f_election = true;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Investor);
        assert!(!r.mtm_elected);
    }

    #[test]
    fn investor_with_high_trades_but_dividend_motive_stays_investor() {
        let mut i = investor_base();
        i.trades_per_year = 5_000;
        i.trading_days_per_year = 250;
        i.average_holding_period_days = 1;
        i.primary_income_source = PrimaryIncomeSource::DividendsAndInterest;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Investor);
    }

    // Notes & citations.

    #[test]
    fn dealer_note_describes_basis() {
        let r = compute(&dealer_base());
        assert!(r.note.contains("DEALER under §475(c)(2)"));
        assert!(r.note.contains("§475(a) MTM MANDATORY"));
    }

    #[test]
    fn trader_with_election_note_describes_consequences() {
        let mut i = trader_base();
        i.has_section_475f_election = true;
        let r = compute(&i);
        assert!(r.note.contains("§475(f) MTM ELECTION"));
        assert!(r.note.contains("wash-sale rules DO NOT apply"));
        assert!(r.note.contains("$3k capital loss cap DOES NOT apply"));
    }

    #[test]
    fn trader_without_election_note_describes_consequences() {
        let r = compute(&trader_base());
        assert!(r.note.contains("TRADER WITHOUT §475(f)"));
        assert!(r.note.contains("STCG / LTCG"));
        assert!(r.note.contains("Schedule C"));
    }

    #[test]
    fn investor_note_lists_failed_prongs() {
        let mut i = trader_base();
        i.trades_per_year = 100; // fails substantial-activity
        i.trading_days_per_year = 50; // fails continuous-regular
        let r = compute(&i);
        assert!(r.note.contains("substantial-activity"));
        assert!(r.note.contains("continuous-and-regular"));
    }

    #[test]
    fn investor_note_mentions_tcja_suspension() {
        let r = compute(&investor_base());
        assert!(r.note.contains("TCJA suspended"));
    }

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&trader_base());
        assert!(r.citation.contains("§475(c)(2)"));
        assert!(r.citation.contains("§1.475(c)-1"));
        assert!(r.citation.contains("§475(a)"));
        assert!(r.citation.contains("§475(f)"));
        assert!(r.citation.contains("Topic 429"));
    }
}
