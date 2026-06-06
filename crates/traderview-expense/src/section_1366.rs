//! IRC § 1366 — Pass-thru of items to shareholders. The
//! cornerstone S-corporation pass-through provision —
//! every item of income, loss, deduction, credit, and
//! tax-exempt income earned by the S corporation flows
//! through to shareholders in pro rata share and retains
//! its character at the shareholder level. Direct
//! trader-business-owner companion to section_1361 (S
//! corporation election), section_1367 (basis
//! adjustments), section_1368 (distributions), section_1374
//! (built-in gains), section_1375 (passive investment
//! income), section_1042 (ESOP rollover — iter 480),
//! section_4978 (ESOP recapture — iter 484), section_6166
//! (estate tax installment — iter 486).
//!
//! § 1366(a)(1) PRO RATA SHARE — every shareholder must
//! report on personal return their pro rata share of:
//! 1. § 1366(a)(1)(A) SEPARATELY-STATED ITEMS — items
//!    that could affect shareholder tax liability
//!    differently than if not separately stated. Common
//!    categories include:
//!    - Short-term capital gains and losses
//!    - Long-term capital gains and losses
//!    - § 1231 gains and losses
//!    - Charitable contributions
//!    - Dividend income (qualified vs ordinary)
//!    - Tax-exempt interest income
//!    - Foreign tax credit items
//!    - Investment interest expense
//!    - § 179 expense election
//!    - AMT preference items
//!    - § 199A qualified business income deduction
//!    - Section 1411 net investment income items
//! 2. § 1366(a)(1)(B) NON-SEPARATELY STATED INCOME OR
//!    LOSS — ordinary trade or business income or loss
//!    (lumped together)
//!
//! § 1366(b) CHARACTER FLOW-THROUGH — each pro rata share
//! item is treated by the shareholder AS IF the underlying
//! activity that generated the item had occurred at the
//! SHAREHOLDER level. Long-term capital gains stay long-
//! term capital gains; § 1231 stays § 1231; tax-exempt
//! interest stays tax-exempt.
//!
//! § 1366(c) GROSS INCOME — for purposes of computing
//! gross income of shareholder, gross income includes
//! shareholder's pro rata share of S corporation gross
//! income.
//!
//! § 1366(d)(1) LOSS LIMITATION — three-tier limitation:
//! 1. § 1366(d)(1)(A) BASIS LIMITATION — aggregate
//!    losses + deductions allowed to shareholder cannot
//!    exceed sum of (i) ADJUSTED BASIS IN S CORP STOCK
//!    plus (ii) ADJUSTED BASIS IN INDEBTEDNESS of S corp
//!    to shareholder
//! 2. § 465 AT-RISK LIMITATION — losses also limited to
//!    amount shareholder is at risk in S corp activity
//! 3. § 469 PASSIVE ACTIVITY LOSS LIMITATION — passive
//!    losses can offset only passive income
//!
//! § 1366(d)(2) CARRYOVER OF DISALLOWED LOSSES — any
//! loss or deduction disallowed by basis limitation is
//! treated as incurred by S corporation in succeeding
//! tax year with respect to that shareholder; carries
//! over INDEFINITELY until basis restored.
//!
//! § 1366(d)(3) POST-TERMINATION TRANSITION PERIOD — if
//! S election terminates, suspended losses become
//! deductible during PTTP (greater of 1-year window
//! after termination or 120 days after IRS notice of
//! termination determination) to the extent of basis
//! increase resulting from shareholder capital
//! contribution during PTTP.
//!
//! § 1366(e) FAMILY GROUP REASONABLE COMPENSATION — IRS
//! authorized to reallocate items among family group
//! members of S corporation if compensation paid to
//! family members performing services is unreasonable.
//! Forces family-shareholder S-corps to pay reasonable
//! W-2 wages.
//!
//! § 1366(f)(1) REDUCTION FOR CERTAIN BUILT-IN GAINS —
//! pro rata share adjusted for any § 1374 tax paid by S
//! corp on built-in gain (avoids double tax).
//!
//! § 1366(f)(2) PASSIVE INVESTMENT INCOME — pro rata
//! share adjusted for § 1375 tax paid by S corp on
//! passive investment income exceeding 25% of gross
//! receipts.
//!
//! Three-tier ordering of loss limitations per
//! 26 C.F.R. § 1.1366-2:
//!
//! 1. § 1366(d)(1) BASIS LIMITATION applied first
//! 2. § 465 AT-RISK LIMITATION applied second
//! 3. § 469 PASSIVE ACTIVITY LOSS LIMITATION applied
//!    third
//!
//! Losses suspended at one tier may flow through to next
//! tier only if next-tier test allows.
//!
//! Trader-business-owner critical because (1) S
//! corporation losses can be used to offset shareholder's
//! W-2 wages, capital gains, and other income — but
//! ONLY to extent of stock + debt basis under
//! § 1366(d)(1) basis cap; (2) at-risk and passive
//! activity loss limitations stack on top of basis
//! limitation; (3) charitable contributions flow
//! through SEPARATELY at character of contribution —
//! retain 30%/60% AGI limitation per § 170(b) at
//! shareholder level; (4) § 199A 20% QBI deduction
//! flows through SEPARATELY as a deduction in
//! determining shareholder taxable income; (5)
//! § 1366(e) family group reasonable compensation is
//! the primary IRS enforcement tool against family
//! S-corp wage-vs-distribution arbitrage; (6) suspended
//! losses under § 1366(d)(2) preserve indefinitely so
//! patient shareholders eventually capture economic
//! losses when basis restored.
//!
//! Distinction from § 702 (partnership pass-through):
//! both § 702 and § 1366 require separately-stated items
//! and pass character through, but partnership Schedule
//! K-1 items can include items not available to S corp
//! (foreign-currency gain/loss as separate item; partner-
//! specific basis tracking via § 704(b); various § 704
//! special allocations). S corporations are stricter on
//! single-class-of-stock rule (§ 1361(b)(1)(D)).
//!
//! Authority: 26 U.S.C. § 1366; § 1366(a)(1)(A);
//! § 1366(a)(1)(B); § 1366(a)(2); § 1366(b); § 1366(c);
//! § 1366(d)(1)(A); § 1366(d)(1)(B); § 1366(d)(2);
//! § 1366(d)(3); § 1366(e); § 1366(f)(1); § 1366(f)(2);
//! § 1361 (S corp election); § 1367 (basis adjustments);
//! § 1368 (distributions); § 1374 (built-in gains);
//! § 1375 (passive investment income); § 1377 (pro rata
//! share and tax year); § 465 (at-risk); § 469 (passive
//! activity loss); § 170(b) (charitable contribution
//! AGI cap); § 199A (QBI deduction); § 702 (partnership
//! pass-through analog); § 1411 (net investment income
//! tax); 26 C.F.R. § 1.1366-1; 26 C.F.R. § 1.1366-2;
//! 26 C.F.R. § 1.1366-3; 26 C.F.R. § 1.1366-4; Tax
//! Reform Act of 1982 Subchapter S Revision, Pub. L.
//! 97-354 — current § 1366 framework.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    SCorporation,
    CCorporation,
    Partnership,
    SoleProprietor,
    NotApplicable,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub entity_type: EntityType,
    pub shareholder_pro_rata_share_basis_points: u32,
    pub adjusted_basis_in_stock_cents: u64,
    pub adjusted_basis_in_indebtedness_cents: u64,
    pub at_risk_amount_cents: u64,
    pub passive_activity: bool,
    pub passive_income_available_to_offset_cents: u64,
    pub corporation_ordinary_business_loss_cents: u64,
    pub corporation_long_term_capital_loss_cents: u64,
    pub corporation_separately_stated_loss_cents: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotEligibleEntityType,
    Compliant,
    LossPartiallySuspendedBasis,
    LossPartiallySuspendedAtRisk,
    LossPartiallySuspendedPassive,
    LossFullySuspended,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub shareholder_pro_rata_total_loss_cents: u64,
    pub loss_allowed_after_basis_limit_cents: u64,
    pub loss_allowed_after_at_risk_limit_cents: u64,
    pub loss_allowed_after_passive_limit_cents: u64,
    pub loss_suspended_under_basis_limit_cents: u64,
    pub loss_suspended_under_at_risk_limit_cents: u64,
    pub loss_suspended_under_passive_limit_cents: u64,
    pub notes: Vec<String>,
}

pub type Section1366Input = Input;
pub type Section1366Result = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "§ 1366(a)(1) PRO RATA SHARE: every shareholder reports pro rata share of (A) SEPARATELY-STATED ITEMS that could affect tax liability differently (short/long-term capital gains/losses + § 1231 + charitable contributions + dividend income + tax-exempt interest + foreign tax credit + investment interest expense + § 179 election + AMT preferences + § 199A QBI deduction + § 1411 NII items) and (B) NON-SEPARATELY STATED ordinary trade or business income/loss.".to_string(),
        "§ 1366(b) CHARACTER FLOW-THROUGH: each pro rata share item treated by shareholder as if underlying activity had occurred at SHAREHOLDER level — long-term capital gains stay long-term, § 1231 stays § 1231, tax-exempt interest stays tax-exempt.".to_string(),
        "§ 1366(d)(1) LOSS LIMITATION three-tier: (1) § 1366(d)(1)(A) BASIS LIMITATION = (adjusted basis in S corp stock) + (adjusted basis in indebtedness of S corp to shareholder); (2) § 465 AT-RISK LIMITATION; (3) § 469 PASSIVE ACTIVITY LOSS LIMITATION — passive losses offset only passive income. Three-tier ordering per 26 C.F.R. § 1.1366-2: basis applied first, at-risk second, passive third.".to_string(),
        "§ 1366(d)(2) CARRYOVER OF DISALLOWED LOSSES: any loss or deduction disallowed by basis limitation treated as incurred by S corporation in succeeding tax year with respect to that shareholder; carries over INDEFINITELY until basis restored (basis adjustments per § 1367 + distributions per § 1368 + built-in gains tax per § 1374 + passive investment income tax per § 1375 + pro-rata-share computation per § 1377).".to_string(),
        "§ 1366(d)(3) POST-TERMINATION TRANSITION PERIOD: if S election terminates, suspended losses become deductible during PTTP (greater of 1-year window after termination or 120 days after IRS notice of termination determination) to extent of basis increase from shareholder capital contribution.".to_string(),
        "§ 1366(e) FAMILY GROUP REASONABLE COMPENSATION: IRS authorized to reallocate items among family group members if compensation paid to family members performing services is unreasonable. Forces family-shareholder S-corps to pay reasonable W-2 wages.".to_string(),
        "§ 1366(f) S-corporation-level tax adjustments: § 1366(f)(1) pro rata share reduced for § 1374 built-in gains tax paid by S corp (avoids double tax); § 1366(f)(2) pro rata share adjusted for § 1375 passive investment income tax when passive income exceeds 25% of gross receipts.".to_string(),
        "Distinction from § 702 partnership pass-through: both require separately-stated items + character flow-through but partnerships allow partner-specific § 704(b) basis tracking and § 704 special allocations; S corporations require single-class-of-stock under § 1361(b)(1)(D).".to_string(),
        "Companion: section_1361 (S corp election), section_1367 (basis adjustments), section_1368 (distributions), section_1374 (built-in gains), section_1375 (passive investment income), section_1042 (iter 480), section_4978 (iter 484), section_6166 (iter 486); also references § 465 + § 469 + § 199A + § 1411 + § 170(b).".to_string(),
    ];

    if !matches!(input.entity_type, EntityType::SCorporation) {
        let mut n = notes;
        n.push("Entity is not an S corporation — § 1366 pass-through does not apply.".to_string());
        return Output {
            severity: Severity::NotEligibleEntityType,
            shareholder_pro_rata_total_loss_cents: 0,
            loss_allowed_after_basis_limit_cents: 0,
            loss_allowed_after_at_risk_limit_cents: 0,
            loss_allowed_after_passive_limit_cents: 0,
            loss_suspended_under_basis_limit_cents: 0,
            loss_suspended_under_at_risk_limit_cents: 0,
            loss_suspended_under_passive_limit_cents: 0,
            notes: n,
        };
    }

    // Compute total corporate loss
    let total_corp_loss = input
        .corporation_ordinary_business_loss_cents
        .saturating_add(input.corporation_long_term_capital_loss_cents)
        .saturating_add(input.corporation_separately_stated_loss_cents);

    // Compute shareholder's pro rata share
    let shareholder_total_loss = (total_corp_loss as u128)
        .saturating_mul(input.shareholder_pro_rata_share_basis_points as u128)
        .checked_div(10_000)
        .unwrap_or(0) as u64;

    // § 1366(d)(1)(A) BASIS LIMITATION
    let basis_cap = input
        .adjusted_basis_in_stock_cents
        .saturating_add(input.adjusted_basis_in_indebtedness_cents);
    let loss_after_basis = shareholder_total_loss.min(basis_cap);
    let suspended_basis = shareholder_total_loss.saturating_sub(loss_after_basis);

    // § 465 AT-RISK LIMITATION
    let loss_after_at_risk = loss_after_basis.min(input.at_risk_amount_cents);
    let suspended_at_risk = loss_after_basis.saturating_sub(loss_after_at_risk);

    // § 469 PASSIVE ACTIVITY LOSS LIMITATION
    let (loss_after_passive, suspended_passive) = if input.passive_activity {
        let allowed = loss_after_at_risk.min(input.passive_income_available_to_offset_cents);
        let suspended = loss_after_at_risk.saturating_sub(allowed);
        (allowed, suspended)
    } else {
        (loss_after_at_risk, 0)
    };

    let mut n = notes;
    n.push(format!(
        "Shareholder pro rata share total loss: ${}.{:02}; § 1366(d)(1)(A) basis cap ${}.{:02} (stock ${}.{:02} + indebtedness ${}.{:02}); after basis limit ${}.{:02} allowed (${}.{:02} suspended INDEFINITELY per § 1366(d)(2)); after § 465 at-risk ${}.{:02} allowed (${}.{:02} suspended); after § 469 passive ${}.{:02} allowed (${}.{:02} suspended).",
        shareholder_total_loss / 100,
        shareholder_total_loss % 100,
        basis_cap / 100,
        basis_cap % 100,
        input.adjusted_basis_in_stock_cents / 100,
        input.adjusted_basis_in_stock_cents % 100,
        input.adjusted_basis_in_indebtedness_cents / 100,
        input.adjusted_basis_in_indebtedness_cents % 100,
        loss_after_basis / 100,
        loss_after_basis % 100,
        suspended_basis / 100,
        suspended_basis % 100,
        loss_after_at_risk / 100,
        loss_after_at_risk % 100,
        suspended_at_risk / 100,
        suspended_at_risk % 100,
        loss_after_passive / 100,
        loss_after_passive % 100,
        suspended_passive / 100,
        suspended_passive % 100
    ));

    let severity = if loss_after_passive == shareholder_total_loss {
        Severity::Compliant
    } else if loss_after_passive == 0 && shareholder_total_loss > 0 {
        Severity::LossFullySuspended
    } else if suspended_basis > 0 {
        Severity::LossPartiallySuspendedBasis
    } else if suspended_at_risk > 0 {
        Severity::LossPartiallySuspendedAtRisk
    } else {
        Severity::LossPartiallySuspendedPassive
    };

    Output {
        severity,
        shareholder_pro_rata_total_loss_cents: shareholder_total_loss,
        loss_allowed_after_basis_limit_cents: loss_after_basis,
        loss_allowed_after_at_risk_limit_cents: loss_after_at_risk,
        loss_allowed_after_passive_limit_cents: loss_after_passive,
        loss_suspended_under_basis_limit_cents: suspended_basis,
        loss_suspended_under_at_risk_limit_cents: suspended_at_risk,
        loss_suspended_under_passive_limit_cents: suspended_passive,
        notes: n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            entity_type: EntityType::SCorporation,
            shareholder_pro_rata_share_basis_points: 5000, // 50%
            adjusted_basis_in_stock_cents: 500_000_00,     // $500K
            adjusted_basis_in_indebtedness_cents: 100_000_00, // $100K
            at_risk_amount_cents: 600_000_00,              // $600K
            passive_activity: false,
            passive_income_available_to_offset_cents: 0,
            corporation_ordinary_business_loss_cents: 200_000_00, // $200K corp loss
            corporation_long_term_capital_loss_cents: 0,
            corporation_separately_stated_loss_cents: 0,
        }
    }

    #[test]
    fn c_corp_not_eligible() {
        let mut i = baseline();
        i.entity_type = EntityType::CCorporation;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotEligibleEntityType);
    }

    #[test]
    fn partnership_not_eligible() {
        let mut i = baseline();
        i.entity_type = EntityType::Partnership;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotEligibleEntityType);
    }

    #[test]
    fn sole_proprietor_not_eligible() {
        let mut i = baseline();
        i.entity_type = EntityType::SoleProprietor;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotEligibleEntityType);
    }

    #[test]
    fn losses_fully_within_basis_compliant() {
        let i = baseline(); // 50% × $200K = $100K loss; basis $600K
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
        assert_eq!(out.shareholder_pro_rata_total_loss_cents, 100_000_00);
        assert_eq!(out.loss_allowed_after_basis_limit_cents, 100_000_00);
        assert_eq!(out.loss_suspended_under_basis_limit_cents, 0);
    }

    #[test]
    fn losses_exceed_basis_partially_suspended() {
        let mut i = baseline();
        i.adjusted_basis_in_stock_cents = 50_000_00; // $50K
        i.adjusted_basis_in_indebtedness_cents = 20_000_00; // $20K
                                                            // basis cap = $70K; shareholder loss = $100K → $30K suspended
        let out = check(&i);
        assert_eq!(out.severity, Severity::LossPartiallySuspendedBasis);
        assert_eq!(out.loss_allowed_after_basis_limit_cents, 70_000_00);
        assert_eq!(out.loss_suspended_under_basis_limit_cents, 30_000_00);
    }

    #[test]
    fn losses_fully_suspended_zero_basis() {
        let mut i = baseline();
        i.adjusted_basis_in_stock_cents = 0;
        i.adjusted_basis_in_indebtedness_cents = 0;
        let out = check(&i);
        assert_eq!(out.severity, Severity::LossFullySuspended);
        assert_eq!(out.loss_allowed_after_basis_limit_cents, 0);
        assert_eq!(out.loss_suspended_under_basis_limit_cents, 100_000_00);
    }

    #[test]
    fn at_risk_limitation_below_basis() {
        let mut i = baseline();
        // basis $600K, but at-risk only $80K; loss $100K
        i.at_risk_amount_cents = 80_000_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::LossPartiallySuspendedAtRisk);
        assert_eq!(out.loss_allowed_after_basis_limit_cents, 100_000_00);
        assert_eq!(out.loss_allowed_after_at_risk_limit_cents, 80_000_00);
        assert_eq!(out.loss_suspended_under_at_risk_limit_cents, 20_000_00);
    }

    #[test]
    fn passive_activity_no_passive_income_fully_suspended_at_passive() {
        let mut i = baseline();
        i.passive_activity = true;
        i.passive_income_available_to_offset_cents = 0;
        let out = check(&i);
        assert_eq!(out.severity, Severity::LossFullySuspended);
        // basis OK, at-risk OK, but passive zero
        assert_eq!(out.loss_allowed_after_passive_limit_cents, 0);
        assert_eq!(out.loss_suspended_under_passive_limit_cents, 100_000_00);
    }

    #[test]
    fn passive_activity_partial_offset() {
        let mut i = baseline();
        i.passive_activity = true;
        i.passive_income_available_to_offset_cents = 60_000_00; // $60K passive offset
        let out = check(&i);
        assert_eq!(out.severity, Severity::LossPartiallySuspendedPassive);
        assert_eq!(out.loss_allowed_after_passive_limit_cents, 60_000_00);
        assert_eq!(out.loss_suspended_under_passive_limit_cents, 40_000_00);
    }

    #[test]
    fn non_passive_no_passive_offset_needed() {
        let i = baseline(); // passive_activity = false
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn pro_rata_share_25_percent() {
        let mut i = baseline();
        i.shareholder_pro_rata_share_basis_points = 2500; // 25%
                                                          // shareholder loss = 25% × $200K = $50K
        let out = check(&i);
        assert_eq!(out.shareholder_pro_rata_total_loss_cents, 50_000_00);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn pro_rata_share_100_percent() {
        let mut i = baseline();
        i.shareholder_pro_rata_share_basis_points = 10000; // 100%
                                                           // shareholder loss = $200K (full corp loss)
        let out = check(&i);
        assert_eq!(out.shareholder_pro_rata_total_loss_cents, 200_000_00);
    }

    #[test]
    fn long_term_capital_loss_separately_stated() {
        let mut i = baseline();
        i.corporation_ordinary_business_loss_cents = 0;
        i.corporation_long_term_capital_loss_cents = 100_000_00;
        // 50% × $100K = $50K LTC loss
        let out = check(&i);
        assert_eq!(out.shareholder_pro_rata_total_loss_cents, 50_000_00);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn separately_stated_loss_flows_through() {
        let mut i = baseline();
        i.corporation_ordinary_business_loss_cents = 100_000_00;
        i.corporation_long_term_capital_loss_cents = 50_000_00;
        i.corporation_separately_stated_loss_cents = 50_000_00;
        // Total corp loss = $200K; 50% = $100K shareholder
        let out = check(&i);
        assert_eq!(out.shareholder_pro_rata_total_loss_cents, 100_000_00);
    }

    #[test]
    fn basis_limit_at_risk_limit_passive_limit_stack() {
        let mut i = baseline();
        i.adjusted_basis_in_stock_cents = 60_000_00;
        i.adjusted_basis_in_indebtedness_cents = 0; // basis cap $60K
        i.at_risk_amount_cents = 40_000_00; // at-risk $40K
        i.passive_activity = true;
        i.passive_income_available_to_offset_cents = 20_000_00; // passive $20K
                                                                // shareholder loss $100K → basis $60K allowed → at-risk $40K allowed → passive $20K allowed
        let out = check(&i);
        assert_eq!(out.loss_allowed_after_basis_limit_cents, 60_000_00);
        assert_eq!(out.loss_allowed_after_at_risk_limit_cents, 40_000_00);
        assert_eq!(out.loss_allowed_after_passive_limit_cents, 20_000_00);
        assert_eq!(out.loss_suspended_under_basis_limit_cents, 40_000_00);
        assert_eq!(out.loss_suspended_under_at_risk_limit_cents, 20_000_00);
        assert_eq!(out.loss_suspended_under_passive_limit_cents, 20_000_00);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1366(a)(1)"));
        assert!(joined.contains("§ 1366(b)"));
        assert!(joined.contains("§ 1366(d)(1)"));
        assert!(joined.contains("§ 1366(d)(1)(A)"));
        assert!(joined.contains("§ 1366(d)(2)"));
        assert!(joined.contains("§ 1366(d)(3)"));
        assert!(joined.contains("§ 1366(e)"));
        assert!(joined.contains("§ 1366(f)"));
        assert!(joined.contains("§ 1366(f)(1)"));
        assert!(joined.contains("§ 1366(f)(2)"));
        assert!(joined.contains("§ 465"));
        assert!(joined.contains("§ 469"));
        assert!(joined.contains("§ 1361"));
        assert!(joined.contains("§ 1367"));
        assert!(joined.contains("§ 1368"));
        assert!(joined.contains("§ 1374"));
        assert!(joined.contains("§ 1375"));
        assert!(joined.contains("§ 1377"));
        assert!(joined.contains("§ 199A"));
        assert!(joined.contains("§ 1411"));
        assert!(joined.contains("§ 170(b)"));
        assert!(joined.contains("§ 702"));
        assert!(joined.contains("§ 704"));
        assert!(joined.contains("§ 1361(b)(1)(D)"));
        assert!(joined.contains("26 C.F.R. § 1.1366-2"));
    }

    #[test]
    fn note_pins_pro_rata_separately_stated_categories() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("SEPARATELY-STATED ITEMS"));
        assert!(joined.contains("short/long-term capital gains/losses"));
        assert!(joined.contains("§ 1231"));
        assert!(joined.contains("charitable contributions"));
        assert!(joined.contains("dividend income"));
        assert!(joined.contains("tax-exempt interest"));
        assert!(joined.contains("foreign tax credit"));
        assert!(joined.contains("§ 179"));
        assert!(joined.contains("§ 199A QBI"));
    }

    #[test]
    fn note_pins_character_flow_through() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("CHARACTER FLOW-THROUGH"));
        assert!(joined.contains("SHAREHOLDER level"));
        assert!(joined.contains("long-term capital gains stay long-term"));
    }

    #[test]
    fn note_pins_three_tier_loss_limitation() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("three-tier"));
        assert!(joined.contains("BASIS LIMITATION"));
        assert!(joined.contains("AT-RISK LIMITATION"));
        assert!(joined.contains("PASSIVE ACTIVITY LOSS LIMITATION"));
        assert!(joined.contains("basis applied first"));
    }

    #[test]
    fn note_pins_d2_indefinite_carryover() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1366(d)(2) CARRYOVER"));
        assert!(joined.contains("INDEFINITELY"));
    }

    #[test]
    fn note_pins_d3_post_termination_transition_period() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("POST-TERMINATION TRANSITION PERIOD"));
        assert!(joined.contains("1-year window"));
        assert!(joined.contains("120 days"));
    }

    #[test]
    fn note_pins_e_family_group_reasonable_compensation() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("FAMILY GROUP REASONABLE COMPENSATION"));
        assert!(joined.contains("reasonable W-2 wages"));
    }

    #[test]
    fn note_pins_partnership_distinction() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 702"));
        assert!(joined.contains("§ 704"));
        assert!(joined.contains("single-class-of-stock"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("section_1361"));
        assert!(joined.contains("section_1367"));
        assert!(joined.contains("section_1368"));
        assert!(joined.contains("section_1374"));
        assert!(joined.contains("section_1375"));
        assert!(joined.contains("section_1042"));
        assert!(joined.contains("section_6166"));
    }

    #[test]
    fn truth_table_six_severity_cells() {
        // Non-S corp
        let c1 = check(&Input {
            entity_type: EntityType::CCorporation,
            ..baseline()
        });
        assert_eq!(c1.severity, Severity::NotEligibleEntityType);

        // Compliant
        let c2 = check(&baseline());
        assert_eq!(c2.severity, Severity::Compliant);

        // Partial suspended at basis
        let c3 = check(&Input {
            adjusted_basis_in_stock_cents: 50_000_00,
            adjusted_basis_in_indebtedness_cents: 20_000_00,
            ..baseline()
        });
        assert_eq!(c3.severity, Severity::LossPartiallySuspendedBasis);

        // Partial suspended at at-risk
        let c4 = check(&Input {
            at_risk_amount_cents: 80_000_00,
            ..baseline()
        });
        assert_eq!(c4.severity, Severity::LossPartiallySuspendedAtRisk);

        // Partial suspended at passive
        let c5 = check(&Input {
            passive_activity: true,
            passive_income_available_to_offset_cents: 60_000_00,
            ..baseline()
        });
        assert_eq!(c5.severity, Severity::LossPartiallySuspendedPassive);

        // Fully suspended
        let c6 = check(&Input {
            adjusted_basis_in_stock_cents: 0,
            adjusted_basis_in_indebtedness_cents: 0,
            ..baseline()
        });
        assert_eq!(c6.severity, Severity::LossFullySuspended);
    }

    #[test]
    fn saturating_arithmetic_overflow_defense() {
        let i = Input {
            corporation_ordinary_business_loss_cents: u64::MAX,
            shareholder_pro_rata_share_basis_points: 10000,
            ..baseline()
        };
        let out = check(&i);
        // No panic
        assert!(matches!(
            out.severity,
            Severity::LossPartiallySuspendedBasis | Severity::LossFullySuspended
        ));
    }

    #[test]
    fn boundary_zero_loss_compliant() {
        let mut i = baseline();
        i.corporation_ordinary_business_loss_cents = 0;
        i.corporation_long_term_capital_loss_cents = 0;
        i.corporation_separately_stated_loss_cents = 0;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
        assert_eq!(out.shareholder_pro_rata_total_loss_cents, 0);
    }

    #[test]
    fn realistic_2m_trader_s_corp_loss() {
        // Trader-CEO of S corp; $2M corporate loss; 60% pro rata = $1.2M loss
        // Stock basis $800K + debt basis $200K = $1M basis cap
        // At-risk $1M; not passive; → $1M allowed, $200K suspended
        let i = Input {
            entity_type: EntityType::SCorporation,
            shareholder_pro_rata_share_basis_points: 6000,
            adjusted_basis_in_stock_cents: 800_000_00,
            adjusted_basis_in_indebtedness_cents: 200_000_00,
            at_risk_amount_cents: 1_000_000_00,
            passive_activity: false,
            passive_income_available_to_offset_cents: 0,
            corporation_ordinary_business_loss_cents: 2_000_000_00,
            corporation_long_term_capital_loss_cents: 0,
            corporation_separately_stated_loss_cents: 0,
        };
        let out = check(&i);
        assert_eq!(out.shareholder_pro_rata_total_loss_cents, 1_200_000_00);
        assert_eq!(out.loss_allowed_after_basis_limit_cents, 1_000_000_00);
        assert_eq!(out.loss_suspended_under_basis_limit_cents, 200_000_00);
        assert_eq!(out.severity, Severity::LossPartiallySuspendedBasis);
    }
}
