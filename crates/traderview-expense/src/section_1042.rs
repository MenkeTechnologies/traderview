//! IRC § 1042 — Sales of stock to employee stock
//! ownership plans (ESOPs) or certain cooperatives.
//! Long-term capital gain deferral on sale of qualified
//! securities of domestic C corporation to ESOP if
//! seller reinvests proceeds in qualified replacement
//! property (QRP) within replacement period. Direct
//! trader-business-owner companion to section_4940 (PF
//! NII excise — iter 470), section_4941 (PF self-dealing
//! — iter 468), section_4942 (PF minimum distribution —
//! iter 472), section_4943 (PF excess business holdings
//! — iter 474), section_4944 (PF jeopardizing
//! investments — iter 476), section_4945 (PF taxable
//! expenditures — iter 478), section_4958 (intermediate
//! sanctions for public charities — iter 466),
//! section_4960 (ATEO executive comp 21% — iter 464),
//! section_422 (ISO), section_423 (ESPP),
//! section_409a (NQDC). § 1042 originally enacted by Tax
//! Reform Act of 1984, Pub. L. 98-369.
//!
//! § 1042(a) GENERAL RULE: long-term capital gain on
//! sale of qualified securities is RECOGNIZED ONLY TO
//! THE EXTENT that the amount realized exceeds the
//! cost of qualified replacement property purchased
//! during replacement period.
//!
//! Five eligibility requirements per § 1042(b):
//! 1. § 1042(b)(1) THREE-YEAR HOLDING PERIOD — seller
//!    must have held the qualified securities for at
//!    least 3 years before sale
//! 2. § 1042(b)(2) 30% ESOP OWNERSHIP — ESOP or eligible
//!    worker-owned cooperative must own at least 30% of
//!    each class of outstanding stock of the corporation
//!    (or of the value of all outstanding stock other
//!    than nonvoting nonconvertible preferred stock)
//!    IMMEDIATELY AFTER THE SALE
//! 3. § 1042(b)(3) WRITTEN CONSENT TO § 4978 RECAPTURE —
//!    employer must consent in writing to the
//!    application of § 4978 recapture tax (10% excise
//!    if ESOP disposes of stock within 3 years)
//! 4. § 1042(b)(4) CORPORATION IS DOMESTIC C CORP — S
//!    corporations are NOT eligible (must close
//!    transaction as C corporation)
//! 5. § 1042(c)(1)(B) QUALIFIED SECURITIES — employer
//!    securities issued by domestic corporation; not
//!    readily tradable on established securities market;
//!    not received by seller in transaction described
//!    in § 83 (compensation), § 422 (ISO exercise), or
//!    § 423 (ESPP exercise)
//!
//! § 1042(c)(3) QUALIFIED REPLACEMENT PROPERTY (QRP)
//! permitted categories:
//! - Common stock with voting and dividend rights of
//!   domestic OPERATING corporations
//! - Preferred stock of domestic operating corporations
//! - Bonds of domestic operating corporations
//! - Convertible floating-rate notes
//! - Convertible bonds of operating companies
//!
//! § 1042(c)(3) QRP EXCLUDED categories:
//! - Securities issued by US government entities
//! - Securities issued by non-US entities
//! - Domestic subsidiaries of non-US parents
//! - FDIC certificates of deposit
//! - Mutual funds and money-market funds
//! - Securities of the corporation that issued the
//!   stock sold to the ESOP
//!
//! § 1042(c)(6) REPLACEMENT PERIOD: 15-month window
//! beginning 3 MONTHS BEFORE the sale and extending 12
//! MONTHS AFTER the sale. Seller must purchase QRP
//! within this window to defer gain.
//!
//! § 1042(d) BASIS ADJUSTMENT: basis in QRP = cost of
//! QRP REDUCED BY the non-recognized gain on § 1042 sale.
//! This means gain is DEFERRED, not eliminated — on
//! eventual sale of QRP, the deferred gain is recognized.
//! However, basis is STEPPED UP at seller's death,
//! permanently eliminating deferred gain.
//!
//! § 1042(e) RECAPTURE OF GAIN ON DISPOSITION OF QRP:
//! seller's eventual disposition of QRP triggers
//! recognition of deferred gain (with exceptions for
//! § 1041 spouse transfer + § 351 corporate formation +
//! gift to operating company under specific rules).
//!
//! § 4978 SEPARATE EXCISE TAX on the EMPLOYER if the
//! ESOP disposes of any qualified securities within
//! 3-year period after acquisition: 10% of amount
//! realized on disposition. Forces ESOP to hold the
//! purchased stock for at least 3 years.
//!
//! Trader-business-owner critical because (1) § 1042
//! is the single largest tax-deferral opportunity
//! available to founders of closely-held businesses
//! contemplating succession via ESOP; (2) basis step-up
//! at death under § 1014 PERMANENTLY ELIMINATES deferred
//! gain — making § 1042 + estate-planning combination
//! among the most powerful long-term wealth-transfer
//! strategies; (3) QRP excluded categories (no mutual
//! funds, no foreign, no government, no issuer-stock)
//! significantly constrain investment options; (4) S
//! corporations must convert to C corporation before
//! § 1042 sale, requiring multi-year planning; (5) 30%
//! post-sale ESOP ownership threshold means partial
//! sales are common but threshold cliff drives
//! transaction sizing; (6) trader-founders frequently
//! pair § 1042 with structured ESOP loan + § 162(k)
//! employer-deductible interest + § 401(a)(22) ESOP
//! requirements.
//!
//! Distinction from § 1031 (like-kind exchange): § 1031
//! covers real property only post-TCJA 2017; § 1042
//! covers C-corp stock-to-ESOP transaction only. Both
//! defer gain via reinvestment.
//!
//! Distinction from § 1045 (small business stock
//! rollover, iter shipped): § 1045 covers § 1202 QSBS
//! rollover into other QSBS within 60 days; § 1042
//! covers ESOP sale rollover into QRP within 15-month
//! window. Different statutory regimes.
//!
//! Authority: 26 U.S.C. § 1042; § 1042(a); § 1042(b)(1);
//! § 1042(b)(2); § 1042(b)(3); § 1042(b)(4);
//! § 1042(c)(1)(B); § 1042(c)(3); § 1042(c)(4);
//! § 1042(c)(6); § 1042(d); § 1042(e); § 4978; § 4979A
//! (additional ESOP excise); § 1014 (stepped-up basis);
//! § 1041 (spouse transfer exception); § 351 (corporate
//! formation exception); § 401(a)(22) ESOP requirements;
//! § 162(k) employer-deductible interest; Rev. Rul.
//! 2000-18; Tax Reform Act of 1984, Pub. L. 98-369 —
//! original § 1042 enactment.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorporationType {
    DomesticCCorporation,
    DomesticSCorporation,
    Partnership,
    LimitedLiabilityCompany,
    NonUsEntity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QrpType {
    DomesticOperatingCorpCommonStock,
    DomesticOperatingCorpPreferredStock,
    DomesticOperatingCorpBonds,
    ConvertibleFloatingRateNote,
    UsGovernmentSecurity,
    NonUsSecurity,
    DomesticSubsidiaryOfNonUsParent,
    FdicCertificateOfDeposit,
    MutualFundOrMoneyMarket,
    SecuritiesOfEsopCorporation,
    NotApplicable,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub corporation_type: CorporationType,
    pub seller_holding_period_years: u32,
    pub esop_post_sale_ownership_basis_points: u32,
    pub written_consent_to_section_4978_recapture: bool,
    pub securities_received_in_compensation_iso_or_espp: bool,
    pub securities_readily_tradable_on_established_market: bool,
    pub qrp_type: QrpType,
    pub qrp_purchase_within_15_month_window: bool,
    pub sale_proceeds_cents: u64,
    pub qrp_cost_cents: u64,
    pub long_term_capital_gain_cents: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotEligibleCorporationType,
    NotEligibleHoldingPeriod,
    NotEligibleEsopOwnership,
    NotEligibleConsentMissing,
    NotEligibleSecuritiesType,
    QrpInvalid,
    QrpPurchaseOutsideWindow,
    PartialDeferral,
    FullDeferral,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub deferred_gain_cents: u64,
    pub recognized_gain_cents: u64,
    pub qrp_basis_cents: u64,
    pub notes: Vec<String>,
}

pub const MIN_HOLDING_PERIOD_YEARS: u32 = 3;
pub const MIN_ESOP_OWNERSHIP_BPS: u32 = 3000; // 30%

pub type Section1042Input = Input;
pub type Section1042Result = Output;

fn is_valid_qrp(t: QrpType) -> bool {
    matches!(
        t,
        QrpType::DomesticOperatingCorpCommonStock
            | QrpType::DomesticOperatingCorpPreferredStock
            | QrpType::DomesticOperatingCorpBonds
            | QrpType::ConvertibleFloatingRateNote
    )
}

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "§ 1042(a) GENERAL RULE: long-term capital gain on sale of qualified securities is RECOGNIZED ONLY TO THE EXTENT amount realized exceeds cost of qualified replacement property (QRP) purchased during 15-month replacement period (3 months before sale + 12 months after).".to_string(),
        "Five eligibility requirements per § 1042(b): (1) § 1042(b)(1) THREE-YEAR HOLDING PERIOD; (2) § 1042(b)(2) 30% ESOP OWNERSHIP IMMEDIATELY AFTER SALE; (3) § 1042(b)(3) WRITTEN CONSENT to § 4978 recapture; (4) § 1042(b)(4) CORPORATION IS DOMESTIC C CORP — S corporations NOT eligible; (5) § 1042(c)(1)(B) QUALIFIED SECURITIES not received in § 83 compensation, § 422 ISO exercise, or § 423 ESPP exercise.".to_string(),
        "§ 1042(c)(3) qualified replacement property categories PERMITTED: common stock with voting and dividend rights of domestic operating corporations; preferred stock; bonds; convertible floating-rate notes; convertible bonds of operating companies. EXCLUDED: US government securities, non-US securities, domestic subsidiaries of non-US parents, FDIC CDs, mutual funds + money-market funds, securities of the corporation that issued the stock sold to the ESOP.".to_string(),
        "§ 1042(c)(6) REPLACEMENT PERIOD: 15-month window beginning 3 months BEFORE the sale and extending 12 months AFTER. § 1042(d) basis adjustment: QRP basis = QRP cost reduced by non-recognized gain. § 1042(e) disposition recapture: eventual sale of QRP triggers recognition of deferred gain (exceptions for § 1041 spouse transfer + § 351 corporate formation).".to_string(),
        "§ 4978 SEPARATE EXCISE TAX on EMPLOYER if ESOP disposes of qualified securities within 3-year period after acquisition: 10% of amount realized. Forces ESOP to hold purchased stock for at least 3 years.".to_string(),
        "§ 1014 BASIS STEP-UP at death PERMANENTLY ELIMINATES deferred § 1042 gain — making § 1042 + estate planning combination among the most powerful long-term wealth-transfer strategies for trader-founders of closely-held businesses.".to_string(),
        "Distinction from § 1031 (like-kind exchange): § 1031 covers real property only post-TCJA 2017; § 1042 covers C-corp stock-to-ESOP transactions only. Both defer gain via reinvestment. Distinction from § 1045 (small business stock rollover): § 1045 covers § 1202 QSBS rollover into other QSBS within 60 days; § 1042 covers 15-month QRP rollover.".to_string(),
        "Companion: section_4940 (iter 470), section_4941 (iter 468), section_4942 (iter 472), section_4943 (iter 474), section_4944 (iter 476), section_4945 (iter 478), section_4958 (iter 466), section_4960 (iter 464), section_422 (ISO), section_423 (ESPP), section_409a (NQDC).".to_string(),
    ];

    // Eligibility checks
    if !matches!(input.corporation_type, CorporationType::DomesticCCorporation) {
        let mut n = notes;
        n.push(format!(
            "Corporation type {:?} not eligible for § 1042 — § 1042(b)(4) requires DOMESTIC C CORPORATION. S corporations must convert before § 1042 sale.",
            input.corporation_type
        ));
        return Output {
            severity: Severity::NotEligibleCorporationType,
            deferred_gain_cents: 0,
            recognized_gain_cents: input.long_term_capital_gain_cents,
            qrp_basis_cents: 0,
            notes: n,
        };
    }

    if input.seller_holding_period_years < MIN_HOLDING_PERIOD_YEARS {
        let mut n = notes;
        n.push(format!(
            "Seller holding period {} years less than § 1042(b)(1) 3-year minimum — not eligible.",
            input.seller_holding_period_years
        ));
        return Output {
            severity: Severity::NotEligibleHoldingPeriod,
            deferred_gain_cents: 0,
            recognized_gain_cents: input.long_term_capital_gain_cents,
            qrp_basis_cents: 0,
            notes: n,
        };
    }

    if input.esop_post_sale_ownership_basis_points < MIN_ESOP_OWNERSHIP_BPS {
        let mut n = notes;
        n.push(format!(
            "ESOP post-sale ownership {} bps less than § 1042(b)(2) 30% (3000 bps) minimum — not eligible.",
            input.esop_post_sale_ownership_basis_points
        ));
        return Output {
            severity: Severity::NotEligibleEsopOwnership,
            deferred_gain_cents: 0,
            recognized_gain_cents: input.long_term_capital_gain_cents,
            qrp_basis_cents: 0,
            notes: n,
        };
    }

    if !input.written_consent_to_section_4978_recapture {
        let mut n = notes;
        n.push("Written consent to § 4978 recapture not given — § 1042(b)(3) requirement not satisfied.".to_string());
        return Output {
            severity: Severity::NotEligibleConsentMissing,
            deferred_gain_cents: 0,
            recognized_gain_cents: input.long_term_capital_gain_cents,
            qrp_basis_cents: 0,
            notes: n,
        };
    }

    if input.securities_received_in_compensation_iso_or_espp
        || input.securities_readily_tradable_on_established_market
    {
        let mut n = notes;
        n.push("Securities not 'qualified securities' under § 1042(c)(1)(B) — either received via § 83 compensation / § 422 ISO / § 423 ESPP exercise, or readily tradable on established securities market.".to_string());
        return Output {
            severity: Severity::NotEligibleSecuritiesType,
            deferred_gain_cents: 0,
            recognized_gain_cents: input.long_term_capital_gain_cents,
            qrp_basis_cents: 0,
            notes: n,
        };
    }

    if !is_valid_qrp(input.qrp_type) {
        let mut n = notes;
        n.push(format!(
            "QRP type {:?} not qualified replacement property under § 1042(c)(3) — excluded categories include US government securities, non-US securities, domestic subsidiaries of non-US parents, FDIC CDs, mutual funds, and securities of the ESOP corporation.",
            input.qrp_type
        ));
        return Output {
            severity: Severity::QrpInvalid,
            deferred_gain_cents: 0,
            recognized_gain_cents: input.long_term_capital_gain_cents,
            qrp_basis_cents: 0,
            notes: n,
        };
    }

    if !input.qrp_purchase_within_15_month_window {
        let mut n = notes;
        n.push("QRP not purchased within § 1042(c)(6) 15-month replacement period (3 months before sale + 12 months after) — no deferral available.".to_string());
        return Output {
            severity: Severity::QrpPurchaseOutsideWindow,
            deferred_gain_cents: 0,
            recognized_gain_cents: input.long_term_capital_gain_cents,
            qrp_basis_cents: 0,
            notes: n,
        };
    }

    // Compute deferral
    let deferred_gain = input
        .long_term_capital_gain_cents
        .min(input.qrp_cost_cents);
    let recognized_gain = input
        .long_term_capital_gain_cents
        .saturating_sub(deferred_gain);
    let qrp_basis = input.qrp_cost_cents.saturating_sub(deferred_gain);

    let severity = if recognized_gain == 0 {
        Severity::FullDeferral
    } else {
        Severity::PartialDeferral
    };

    let mut n = notes;
    n.push(format!(
        "§ 1042 deferral computed: deferred gain ${}.{:02}; recognized gain ${}.{:02}; QRP basis ${}.{:02} (basis = cost ${}.{:02} reduced by deferred gain).",
        deferred_gain / 100,
        deferred_gain % 100,
        recognized_gain / 100,
        recognized_gain % 100,
        qrp_basis / 100,
        qrp_basis % 100,
        input.qrp_cost_cents / 100,
        input.qrp_cost_cents % 100
    ));

    Output {
        severity,
        deferred_gain_cents: deferred_gain,
        recognized_gain_cents: recognized_gain,
        qrp_basis_cents: qrp_basis,
        notes: n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            corporation_type: CorporationType::DomesticCCorporation,
            seller_holding_period_years: 5,
            esop_post_sale_ownership_basis_points: 3500, // 35%
            written_consent_to_section_4978_recapture: true,
            securities_received_in_compensation_iso_or_espp: false,
            securities_readily_tradable_on_established_market: false,
            qrp_type: QrpType::DomesticOperatingCorpCommonStock,
            qrp_purchase_within_15_month_window: true,
            sale_proceeds_cents: 10_000_000_00,    // $10M
            qrp_cost_cents: 10_000_000_00,         // full reinvestment
            long_term_capital_gain_cents: 8_000_000_00, // $8M gain
        }
    }

    #[test]
    fn s_corporation_not_eligible() {
        let mut i = baseline();
        i.corporation_type = CorporationType::DomesticSCorporation;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotEligibleCorporationType);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1042(b)(4)"));
        assert!(joined.contains("DOMESTIC C CORPORATION"));
    }

    #[test]
    fn partnership_not_eligible() {
        let mut i = baseline();
        i.corporation_type = CorporationType::Partnership;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotEligibleCorporationType);
    }

    #[test]
    fn llc_not_eligible() {
        let mut i = baseline();
        i.corporation_type = CorporationType::LimitedLiabilityCompany;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotEligibleCorporationType);
    }

    #[test]
    fn non_us_entity_not_eligible() {
        let mut i = baseline();
        i.corporation_type = CorporationType::NonUsEntity;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotEligibleCorporationType);
    }

    #[test]
    fn holding_period_under_3_years_not_eligible() {
        let mut i = baseline();
        i.seller_holding_period_years = 2;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotEligibleHoldingPeriod);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1042(b)(1) 3-year"));
    }

    #[test]
    fn holding_period_exactly_3_years_eligible() {
        let mut i = baseline();
        i.seller_holding_period_years = 3;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FullDeferral);
    }

    #[test]
    fn esop_under_30_percent_not_eligible() {
        let mut i = baseline();
        i.esop_post_sale_ownership_basis_points = 2999; // 29.99%
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotEligibleEsopOwnership);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1042(b)(2)"));
        assert!(joined.contains("30%"));
    }

    #[test]
    fn esop_exactly_30_percent_eligible() {
        let mut i = baseline();
        i.esop_post_sale_ownership_basis_points = 3000;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FullDeferral);
    }

    #[test]
    fn written_consent_missing_not_eligible() {
        let mut i = baseline();
        i.written_consent_to_section_4978_recapture = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotEligibleConsentMissing);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4978"));
        assert!(joined.contains("§ 1042(b)(3)"));
    }

    #[test]
    fn securities_received_in_iso_compensation_not_eligible() {
        let mut i = baseline();
        i.securities_received_in_compensation_iso_or_espp = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotEligibleSecuritiesType);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1042(c)(1)(B)"));
        assert!(joined.contains("§ 422 ISO"));
    }

    #[test]
    fn securities_readily_tradable_not_eligible() {
        let mut i = baseline();
        i.securities_readily_tradable_on_established_market = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotEligibleSecuritiesType);
    }

    #[test]
    fn qrp_us_government_security_invalid() {
        let mut i = baseline();
        i.qrp_type = QrpType::UsGovernmentSecurity;
        let out = check(&i);
        assert_eq!(out.severity, Severity::QrpInvalid);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1042(c)(3)"));
        assert!(joined.contains("US government"));
    }

    #[test]
    fn qrp_non_us_security_invalid() {
        let mut i = baseline();
        i.qrp_type = QrpType::NonUsSecurity;
        let out = check(&i);
        assert_eq!(out.severity, Severity::QrpInvalid);
    }

    #[test]
    fn qrp_domestic_subsidiary_of_non_us_parent_invalid() {
        let mut i = baseline();
        i.qrp_type = QrpType::DomesticSubsidiaryOfNonUsParent;
        let out = check(&i);
        assert_eq!(out.severity, Severity::QrpInvalid);
    }

    #[test]
    fn qrp_fdic_cd_invalid() {
        let mut i = baseline();
        i.qrp_type = QrpType::FdicCertificateOfDeposit;
        let out = check(&i);
        assert_eq!(out.severity, Severity::QrpInvalid);
    }

    #[test]
    fn qrp_mutual_fund_invalid() {
        let mut i = baseline();
        i.qrp_type = QrpType::MutualFundOrMoneyMarket;
        let out = check(&i);
        assert_eq!(out.severity, Severity::QrpInvalid);
    }

    #[test]
    fn qrp_securities_of_esop_corporation_invalid() {
        let mut i = baseline();
        i.qrp_type = QrpType::SecuritiesOfEsopCorporation;
        let out = check(&i);
        assert_eq!(out.severity, Severity::QrpInvalid);
    }

    #[test]
    fn qrp_domestic_operating_common_stock_valid() {
        let mut i = baseline();
        i.qrp_type = QrpType::DomesticOperatingCorpCommonStock;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FullDeferral);
    }

    #[test]
    fn qrp_domestic_operating_preferred_valid() {
        let mut i = baseline();
        i.qrp_type = QrpType::DomesticOperatingCorpPreferredStock;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FullDeferral);
    }

    #[test]
    fn qrp_domestic_operating_bonds_valid() {
        let mut i = baseline();
        i.qrp_type = QrpType::DomesticOperatingCorpBonds;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FullDeferral);
    }

    #[test]
    fn qrp_convertible_floating_rate_note_valid() {
        let mut i = baseline();
        i.qrp_type = QrpType::ConvertibleFloatingRateNote;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FullDeferral);
    }

    #[test]
    fn qrp_purchased_outside_15_month_window_no_deferral() {
        let mut i = baseline();
        i.qrp_purchase_within_15_month_window = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::QrpPurchaseOutsideWindow);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1042(c)(6) 15-month"));
    }

    #[test]
    fn full_reinvestment_full_deferral() {
        let i = baseline(); // $8M gain, $10M QRP, full reinvestment
        let out = check(&i);
        assert_eq!(out.severity, Severity::FullDeferral);
        assert_eq!(out.deferred_gain_cents, 8_000_000_00);
        assert_eq!(out.recognized_gain_cents, 0);
        // Basis = $10M - $8M = $2M
        assert_eq!(out.qrp_basis_cents, 2_000_000_00);
    }

    #[test]
    fn partial_reinvestment_partial_deferral() {
        let mut i = baseline();
        i.qrp_cost_cents = 5_000_000_00; // only $5M reinvested
        // $8M gain capped at $5M deferral; $3M recognized
        let out = check(&i);
        assert_eq!(out.severity, Severity::PartialDeferral);
        assert_eq!(out.deferred_gain_cents, 5_000_000_00);
        assert_eq!(out.recognized_gain_cents, 3_000_000_00);
        // Basis = $5M - $5M = $0
        assert_eq!(out.qrp_basis_cents, 0);
    }

    #[test]
    fn over_reinvestment_full_deferral_with_full_basis() {
        let mut i = baseline();
        i.qrp_cost_cents = 12_000_000_00; // $12M QRP, $8M gain
        let out = check(&i);
        assert_eq!(out.severity, Severity::FullDeferral);
        // Deferred = min($8M, $12M) = $8M
        assert_eq!(out.deferred_gain_cents, 8_000_000_00);
        // Basis = $12M - $8M = $4M
        assert_eq!(out.qrp_basis_cents, 4_000_000_00);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1042(a)"));
        assert!(joined.contains("§ 1042(b)(1)"));
        assert!(joined.contains("§ 1042(b)(2)"));
        assert!(joined.contains("§ 1042(b)(3)"));
        assert!(joined.contains("§ 1042(b)(4)"));
        assert!(joined.contains("§ 1042(c)(1)(B)"));
        assert!(joined.contains("§ 1042(c)(3)"));
        assert!(joined.contains("§ 1042(c)(6)"));
        assert!(joined.contains("§ 1042(d)"));
        assert!(joined.contains("§ 1042(e)"));
        assert!(joined.contains("§ 4978"));
        assert!(joined.contains("§ 1014"));
        assert!(joined.contains("§ 1041"));
        assert!(joined.contains("§ 351"));
        assert!(joined.contains("§ 83"));
        assert!(joined.contains("§ 422"));
        assert!(joined.contains("§ 423"));
        assert!(joined.contains("§ 1031"));
        assert!(joined.contains("§ 1045"));
        assert!(joined.contains("§ 1202"));
    }

    #[test]
    fn note_pins_general_rule_15_month_window() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("15-month"));
        assert!(joined.contains("3 months before"));
        assert!(joined.contains("12 months after"));
    }

    #[test]
    fn note_pins_five_eligibility_requirements() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("THREE-YEAR HOLDING PERIOD"));
        assert!(joined.contains("30% ESOP OWNERSHIP"));
        assert!(joined.contains("WRITTEN CONSENT"));
        assert!(joined.contains("DOMESTIC C CORP"));
        assert!(joined.contains("QUALIFIED SECURITIES"));
    }

    #[test]
    fn note_pins_qrp_categories() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("common stock"));
        assert!(joined.contains("preferred stock"));
        assert!(joined.contains("bonds"));
        assert!(joined.contains("convertible floating-rate notes"));
        assert!(joined.contains("US government securities"));
        assert!(joined.contains("mutual funds"));
        assert!(joined.contains("securities of the corporation"));
    }

    #[test]
    fn note_pins_4978_employer_recapture() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4978 SEPARATE EXCISE TAX"));
        assert!(joined.contains("10%"));
        assert!(joined.contains("3-year period"));
    }

    #[test]
    fn note_pins_1014_basis_step_up_strategy() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1014 BASIS STEP-UP"));
        assert!(joined.contains("PERMANENTLY ELIMINATES"));
    }

    #[test]
    fn note_pins_1031_1045_distinctions() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1031 (like-kind exchange)"));
        assert!(joined.contains("§ 1045"));
        assert!(joined.contains("§ 1202 QSBS"));
        assert!(joined.contains("60 days"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("section_4940"));
        assert!(joined.contains("section_4941"));
        assert!(joined.contains("section_4942"));
        assert!(joined.contains("section_4945"));
        assert!(joined.contains("section_422"));
        assert!(joined.contains("section_423"));
        assert!(joined.contains("section_409a"));
    }

    #[test]
    fn truth_table_nine_severity_cells() {
        // 1: S corp → NotEligibleCorporationType
        let c1 = check(&Input {
            corporation_type: CorporationType::DomesticSCorporation,
            ..baseline()
        });
        assert_eq!(c1.severity, Severity::NotEligibleCorporationType);

        // 2: < 3-year holding → NotEligibleHoldingPeriod
        let c2 = check(&Input {
            seller_holding_period_years: 2,
            ..baseline()
        });
        assert_eq!(c2.severity, Severity::NotEligibleHoldingPeriod);

        // 3: < 30% ESOP → NotEligibleEsopOwnership
        let c3 = check(&Input {
            esop_post_sale_ownership_basis_points: 2500,
            ..baseline()
        });
        assert_eq!(c3.severity, Severity::NotEligibleEsopOwnership);

        // 4: No written consent → NotEligibleConsentMissing
        let c4 = check(&Input {
            written_consent_to_section_4978_recapture: false,
            ..baseline()
        });
        assert_eq!(c4.severity, Severity::NotEligibleConsentMissing);

        // 5: Securities received in ISO → NotEligibleSecuritiesType
        let c5 = check(&Input {
            securities_received_in_compensation_iso_or_espp: true,
            ..baseline()
        });
        assert_eq!(c5.severity, Severity::NotEligibleSecuritiesType);

        // 6: QRP invalid type → QrpInvalid
        let c6 = check(&Input {
            qrp_type: QrpType::MutualFundOrMoneyMarket,
            ..baseline()
        });
        assert_eq!(c6.severity, Severity::QrpInvalid);

        // 7: QRP outside window → QrpPurchaseOutsideWindow
        let c7 = check(&Input {
            qrp_purchase_within_15_month_window: false,
            ..baseline()
        });
        assert_eq!(c7.severity, Severity::QrpPurchaseOutsideWindow);

        // 8: Partial reinvestment → PartialDeferral
        let c8 = check(&Input {
            qrp_cost_cents: 4_000_000_00,
            ..baseline()
        });
        assert_eq!(c8.severity, Severity::PartialDeferral);

        // 9: Full reinvestment → FullDeferral
        let c9 = check(&baseline());
        assert_eq!(c9.severity, Severity::FullDeferral);
    }

    #[test]
    fn saturating_arithmetic_overflow_defense() {
        let i = Input {
            sale_proceeds_cents: u64::MAX,
            qrp_cost_cents: u64::MAX,
            long_term_capital_gain_cents: u64::MAX,
            ..baseline()
        };
        let out = check(&i);
        // No panic; deferred = min(MAX, MAX) = MAX
        assert_eq!(out.severity, Severity::FullDeferral);
    }

    #[test]
    fn boundary_zero_gain_zero_deferral() {
        let mut i = baseline();
        i.long_term_capital_gain_cents = 0;
        let out = check(&i);
        assert_eq!(out.deferred_gain_cents, 0);
        assert_eq!(out.recognized_gain_cents, 0);
    }

    #[test]
    fn realistic_50m_founder_sale_full_deferral() {
        // Founder sells $50M of qualified securities; $40M gain; reinvests $50M in QRP
        let i = Input {
            corporation_type: CorporationType::DomesticCCorporation,
            seller_holding_period_years: 25, // founder, long-term holder
            esop_post_sale_ownership_basis_points: 5000, // 50%
            written_consent_to_section_4978_recapture: true,
            securities_received_in_compensation_iso_or_espp: false,
            securities_readily_tradable_on_established_market: false,
            qrp_type: QrpType::DomesticOperatingCorpCommonStock,
            qrp_purchase_within_15_month_window: true,
            sale_proceeds_cents: 50_000_000_00, // $50M
            qrp_cost_cents: 50_000_000_00,      // full reinvest
            long_term_capital_gain_cents: 40_000_000_00, // $40M gain
        };
        let out = check(&i);
        assert_eq!(out.severity, Severity::FullDeferral);
        assert_eq!(out.deferred_gain_cents, 40_000_000_00);
        assert_eq!(out.recognized_gain_cents, 0);
        // Basis = $50M - $40M = $10M
        assert_eq!(out.qrp_basis_cents, 10_000_000_00);
    }
}
