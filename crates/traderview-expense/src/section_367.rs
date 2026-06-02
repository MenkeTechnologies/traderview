//! IRC § 367 — Foreign Corporations.
//!
//! § 367 overrides nonrecognition treatment that would otherwise apply under
//! § 332, § 351, § 354, § 355, § 356, § 361, or § 721 when a transfer
//! involves a foreign corporation. Prevents US shareholders from using
//! cross-border reorganizations to permanently avoid US tax on appreciated
//! property by moving it to a foreign corp outside the US tax net.
//!
//! § 367(a)(1) OUTBOUND TRANSFER GENERAL RULE: a US person transferring
//! property to a foreign corporation in a § 332, § 351, § 354, § 356, or
//! § 361 exchange is treated as transferring property in a TAXABLE exchange
//! at fair market value, recognizing built-in gain. Foreign corp NOT treated
//! as corporation for this transfer.
//!
//! § 367(a)(2) EXCEPTION for stock or securities of a foreign corp that is
//! party to the exchange (gain-recognition agreement under § 367(a)(8) may
//! defer).
//!
//! TCJA § 14102(e) AMENDMENT (effective for transfers after December 31,
//! 2017): REPEALED former § 367(a)(3) active trade or business exception
//! — outbound tangible property transfers now subject to FULL § 367(a)(1)
//! gain recognition with no active-business carve-out.
//!
//! § 367(b) INBOUND TRANSFER REGS: when foreign corp transfers property to
//! domestic corp in § 332 liquidation or § 368(a)(1) reorg, Treasury
//! regulations may require the domestic acquiring corp to include foreign
//! corp's all earnings and profits accumulated since 1986 as deemed
//! dividend. Treas. Reg. § 1.367(b)-3 implements.
//!
//! § 367(d) INTANGIBLE PROPERTY OUTBOUND TRANSFER: US person transferring
//! § 936(h)(3)(B) intangible property to foreign corporation in § 351 or §
//! 361 exchange is treated as TRANSFERRING the intangible in exchange for
//! deemed annual royalty payments over the useful life of the property
//! commensurate with the income attributable to the intangible.
//!
//! TCJA § 14221 INTANGIBLE DEFINITION EXPANSION (effective for transfers
//! after December 31, 2017): expanded § 936(h)(3)(B) intangible to include
//! goodwill + going-concern value + workforce-in-place + any other item the
//! value of which is not attributable to tangible property or services of
//! individuals.
//!
//! § 367(e) DISTRIBUTIONS to foreign corporate shareholders.
//!
//! Form 926 (Return by a U.S. Transferor of Property to a Foreign Corp)
//! filing required for § 367(a) transfers. Form 8865 for foreign
//! partnership transfers.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferDirection {
    /// US person → foreign corp (outbound) — § 367(a) or § 367(d).
    Outbound,
    /// Foreign corp → domestic corp (inbound) — § 367(b).
    Inbound,
    /// Foreign-to-foreign — § 367(b) regulations may apply if US
    /// shareholder.
    ForeignToForeignWithUsShareholder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    /// Tangible property (machinery, equipment, real estate).
    TangibleProperty,
    /// Stock or securities of foreign corporation party to exchange.
    StockOfForeignCorporationPartyToExchange,
    /// § 936(h)(3)(B) intangible (patents, trademarks, copyrights, post-
    /// TCJA goodwill + going concern + workforce-in-place).
    Section936IntangibleIncludingGoodwill,
    /// Inventory or trade receivables.
    InventoryOrReceivables,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    OutboundTangibleFullGainRecognition,
    OutboundStockGainRecognitionAgreementAvailable,
    OutboundIntangibleDeemedRoyalty,
    InboundEarningsAndProfitsInclusion,
    ForeignToForeignNoUsConsequence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section367Input {
    pub transfer_direction: TransferDirection,
    pub property_type: PropertyType,
    pub transfer_year: i32,
    /// Fair market value of transferred property in cents.
    pub property_fmv_cents: u64,
    /// US transferor's adjusted basis in property in cents.
    pub adjusted_basis_cents: u64,
    /// Foreign corp's post-1986 accumulated earnings and profits in cents
    /// (for § 367(b) inbound).
    pub foreign_corp_post_1986_ep_cents: u64,
    /// Useful life of intangible property in years (for § 367(d) deemed
    /// royalty computation).
    pub intangible_useful_life_years: u32,
    /// Whether gain-recognition agreement under § 367(a)(8) elected.
    pub gain_recognition_agreement_elected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section367Result {
    pub severity: Severity,
    pub recognized_gain_cents: u64,
    pub deemed_annual_royalty_cents: u64,
    pub deemed_dividend_inclusion_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const TCJA_14102_EFFECTIVE_YEAR: i32 = 2018;
pub const TCJA_14221_INTANGIBLE_EXPANSION_YEAR: i32 = 2018;
pub const ACTIVE_TRADE_BUSINESS_EXCEPTION_REPEAL_YEAR: i32 = 2018;

pub fn check(input: &Section367Input) -> Section367Result {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if matches!(
        input.transfer_direction,
        TransferDirection::ForeignToForeignWithUsShareholder
    ) {
        notes.push(
            "Foreign-to-foreign reorganization with US shareholder participation — § \
             367(b) regulations may impose US shareholder gross-income inclusion under § \
             1248 if E&P shifted across CFCs. Treas. Reg. § 1.367(b)-4 implements. \
             Confirm coordination with [[section_959]] PTEP framework (iter 512) and \
             [[section_960]] FTC (iter 520)."
                .to_string(),
        );
        return empty_result(
            Severity::ForeignToForeignNoUsConsequence,
            input,
            actions,
            notes,
            "26 U.S.C. § 367(b); Treas. Reg. § 1.367(b)-4",
        );
    }

    match (input.transfer_direction, input.property_type) {
        (TransferDirection::Outbound, PropertyType::TangibleProperty) => {
            let built_in_gain = input.property_fmv_cents.saturating_sub(input.adjusted_basis_cents);
            actions.push(format!(
                "§ 367(a)(1) outbound transfer of tangible property to foreign corporation: \
                 FULL GAIN RECOGNITION at fair market value. FMV = {} cents; adjusted basis \
                 = {} cents; built-in gain RECOGNIZED = {} cents. TCJA § 14102(e) REPEALED \
                 former § 367(a)(3) active trade or business exception effective for \
                 transfers after December 31, {} — no carve-out available for active-\
                 business assets. File Form 926 (Return by a U.S. Transferor of Property to \
                 a Foreign Corp) within 90 days after transfer; report gain on Form 1120 \
                 Schedule D or Form 1040 Schedule D.",
                input.property_fmv_cents,
                input.adjusted_basis_cents,
                built_in_gain,
                TCJA_14102_EFFECTIVE_YEAR - 1
            ));
            Section367Result {
                severity: Severity::OutboundTangibleFullGainRecognition,
                recognized_gain_cents: built_in_gain,
                deemed_annual_royalty_cents: 0,
                deemed_dividend_inclusion_cents: 0,
                recommended_actions: actions,
                citation: "26 U.S.C. § 367(a)(1); TCJA § 14102(e); Pub. L. 115-97",
                notes: build_coord_notes(notes),
            }
        }
        (
            TransferDirection::Outbound,
            PropertyType::StockOfForeignCorporationPartyToExchange,
        ) => {
            if input.gain_recognition_agreement_elected {
                actions.push(
                    "§ 367(a)(8) gain-recognition agreement (GRA) elected for outbound \
                     transfer of stock of foreign corporation party to exchange. Built-in \
                     gain DEFERRED for 5-year period; if triggering event occurs (sale, \
                     liquidation, deemed sale) within 5 years, US transferor includes \
                     entire deferred gain plus interest. File GRA statement on Form 926; \
                     maintain Treas. Reg. § 1.367(a)-8 compliance records for full 5-year \
                     deferral period plus statute of limitations extension."
                        .to_string(),
                );
                Section367Result {
                    severity: Severity::OutboundStockGainRecognitionAgreementAvailable,
                    recognized_gain_cents: 0,
                    deemed_annual_royalty_cents: 0,
                    deemed_dividend_inclusion_cents: 0,
                    recommended_actions: actions,
                    citation: "26 U.S.C. § 367(a)(8); Treas. Reg. § 1.367(a)-8",
                    notes: build_coord_notes(notes),
                }
            } else {
                let built_in_gain = input
                    .property_fmv_cents
                    .saturating_sub(input.adjusted_basis_cents);
                actions.push(format!(
                    "§ 367(a)(1) outbound transfer of stock without § 367(a)(8) GRA \
                     election: FULL GAIN RECOGNITION at FMV. Built-in gain = {} cents \
                     recognized. Consider GRA election to defer for 5 years if no \
                     triggering event reasonably foreseeable.",
                    built_in_gain
                ));
                Section367Result {
                    severity: Severity::OutboundTangibleFullGainRecognition,
                    recognized_gain_cents: built_in_gain,
                    deemed_annual_royalty_cents: 0,
                    deemed_dividend_inclusion_cents: 0,
                    recommended_actions: actions,
                    citation: "26 U.S.C. § 367(a)(1)-(8)",
                    notes: build_coord_notes(notes),
                }
            }
        }
        (
            TransferDirection::Outbound,
            PropertyType::Section936IntangibleIncludingGoodwill,
        ) => {
            let useful_life = input.intangible_useful_life_years.max(1);
            let annual_royalty = input.property_fmv_cents / u64::from(useful_life);
            actions.push(format!(
                "§ 367(d) outbound transfer of § 936(h)(3)(B) intangible property to \
                 foreign corp: treated as transfer in exchange for ANNUAL DEEMED ROYALTY \
                 over useful life of {} years; FMV {} cents / useful life = {} cents \
                 annual royalty deemed received by US transferor each year. TCJA § 14221 \
                 EXPANDED intangible definition effective {} to include goodwill + going-\
                 concern value + workforce-in-place + any item value not attributable to \
                 tangible property or individual services. Form 926 filing required.",
                useful_life,
                input.property_fmv_cents,
                annual_royalty,
                TCJA_14221_INTANGIBLE_EXPANSION_YEAR
            ));
            Section367Result {
                severity: Severity::OutboundIntangibleDeemedRoyalty,
                recognized_gain_cents: 0,
                deemed_annual_royalty_cents: annual_royalty,
                deemed_dividend_inclusion_cents: 0,
                recommended_actions: actions,
                citation: "26 U.S.C. § 367(d); § 936(h)(3)(B); TCJA § 14221",
                notes: build_coord_notes(notes),
            }
        }
        (TransferDirection::Inbound, _) => {
            actions.push(format!(
                "§ 367(b) inbound transfer of foreign corp property to domestic corp via § \
                 332 liquidation or § 368(a)(1) reorganization: domestic acquiring corp \
                 INCLUDES foreign corp's post-1986 accumulated E&P as deemed dividend = {} \
                 cents per Treas. Reg. § 1.367(b)-3. Coordinate with Notice 2024-16 § \
                 961(c) basis carryover rule (see [[section_961]] iter 522). Report on \
                 Form 5471 + Form 1120 dividend schedule.",
                input.foreign_corp_post_1986_ep_cents
            ));
            Section367Result {
                severity: Severity::InboundEarningsAndProfitsInclusion,
                recognized_gain_cents: 0,
                deemed_annual_royalty_cents: 0,
                deemed_dividend_inclusion_cents: input.foreign_corp_post_1986_ep_cents,
                recommended_actions: actions,
                citation: "26 U.S.C. § 367(b); Treas. Reg. § 1.367(b)-3; Notice 2024-16",
                notes: build_coord_notes(notes),
            }
        }
        (
            TransferDirection::Outbound,
            PropertyType::InventoryOrReceivables,
        ) => {
            actions.push(
                "Outbound transfer of inventory or receivables to foreign corp: § 367(a) \
                 general rule applies — full gain recognition. Inventory does not qualify \
                 for any § 367 exception. File Form 926 within 90 days; report gain on \
                 Form 1120 Schedule D."
                    .to_string(),
            );
            let built_in_gain = input
                .property_fmv_cents
                .saturating_sub(input.adjusted_basis_cents);
            Section367Result {
                severity: Severity::OutboundTangibleFullGainRecognition,
                recognized_gain_cents: built_in_gain,
                deemed_annual_royalty_cents: 0,
                deemed_dividend_inclusion_cents: 0,
                recommended_actions: actions,
                citation: "26 U.S.C. § 367(a)(1)",
                notes: build_coord_notes(notes),
            }
        }
        _ => empty_result(
            Severity::NotApplicable,
            input,
            actions,
            notes,
            "26 U.S.C. § 367",
        ),
    }
}

fn build_coord_notes(mut notes: Vec<String>) -> Vec<String> {
    notes.push(
        "Coordination with [[section_332]] (parent-subsidiary liquidation — nonrecognition \
         overridden by § 367), [[section_351]] (corporation transfer in exchange for stock \
         — overridden by § 367(a)), [[section_354]] (reorganization stock exchange — \
         overridden), [[section_355]] (corporate division — § 367(e) may apply), \
         [[section_361]] (corporate reorganization transfer — overridden by § 367), \
         [[section_368]] (corporate reorganization framework — Notice 2024-16 § 961(c) \
         carryover for inbound reorgs), [[section_721]] (partnership contribution — \
         coordinate with § 367(d) intangible regime), [[section_951a]] (GILTI / NCTI — \
         iter 500), [[section_956]] (CFC US property — iter 504), [[section_959]] (PTEP — \
         iter 512), [[section_960]] (deemed-paid FTC — iter 520), [[section_961]] (CFC \
         basis adjustments — iter 522 Notice 2024-16 carryover), [[section_245a]] (DRD — \
         iter 502 — coordinates with § 367(b) inbound E&P inclusion), [[section_1248]] \
         (gain on CFC stock sale recharacterized as dividend)."
            .to_string(),
    );
    notes
}

fn empty_result(
    severity: Severity,
    input: &Section367Input,
    recommended_actions: Vec<String>,
    notes: Vec<String>,
    citation: &'static str,
) -> Section367Result {
    let _ = input;
    Section367Result {
        severity,
        recognized_gain_cents: 0,
        deemed_annual_royalty_cents: 0,
        deemed_dividend_inclusion_cents: 0,
        recommended_actions,
        citation,
        notes: build_coord_notes(notes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section367Input {
        Section367Input {
            transfer_direction: TransferDirection::Outbound,
            property_type: PropertyType::TangibleProperty,
            transfer_year: 2024,
            property_fmv_cents: 100_000_000_00,
            adjusted_basis_cents: 30_000_000_00,
            foreign_corp_post_1986_ep_cents: 0,
            intangible_useful_life_years: 0,
            gain_recognition_agreement_elected: false,
        }
    }

    #[test]
    fn outbound_tangible_full_gain_recognition() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::OutboundTangibleFullGainRecognition
        ));
        assert_eq!(r.recognized_gain_cents, 70_000_000_00);
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 926")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("TCJA § 14102(e)")));
    }

    #[test]
    fn outbound_stock_with_gra_election_deferred() {
        let mut i = baseline();
        i.property_type = PropertyType::StockOfForeignCorporationPartyToExchange;
        i.gain_recognition_agreement_elected = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::OutboundStockGainRecognitionAgreementAvailable
        ));
        assert_eq!(r.recognized_gain_cents, 0);
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 367(a)(8)")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("5-year")));
    }

    #[test]
    fn outbound_stock_without_gra_full_recognition() {
        let mut i = baseline();
        i.property_type = PropertyType::StockOfForeignCorporationPartyToExchange;
        i.gain_recognition_agreement_elected = false;
        let r = check(&i);
        assert_eq!(r.recognized_gain_cents, 70_000_000_00);
    }

    #[test]
    fn outbound_intangible_deemed_royalty_over_useful_life() {
        let mut i = baseline();
        i.property_type = PropertyType::Section936IntangibleIncludingGoodwill;
        i.intangible_useful_life_years = 10;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::OutboundIntangibleDeemedRoyalty));
        let expected_annual = 100_000_000_00u64 / 10;
        assert_eq!(r.deemed_annual_royalty_cents, expected_annual);
        assert!(r.recommended_actions.iter().any(|a| a.contains("TCJA § 14221")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("goodwill")));
    }

    #[test]
    fn outbound_intangible_useful_life_zero_capped_at_one() {
        let mut i = baseline();
        i.property_type = PropertyType::Section936IntangibleIncludingGoodwill;
        i.intangible_useful_life_years = 0;
        let r = check(&i);
        assert_eq!(r.deemed_annual_royalty_cents, 100_000_000_00);
    }

    #[test]
    fn inbound_earnings_and_profits_inclusion() {
        let mut i = baseline();
        i.transfer_direction = TransferDirection::Inbound;
        i.foreign_corp_post_1986_ep_cents = 50_000_000_00;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::InboundEarningsAndProfitsInclusion));
        assert_eq!(r.deemed_dividend_inclusion_cents, 50_000_000_00);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Treas. Reg. § 1.367(b)-3")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Notice 2024-16")));
    }

    #[test]
    fn foreign_to_foreign_no_us_consequence_with_us_shareholder() {
        let mut i = baseline();
        i.transfer_direction = TransferDirection::ForeignToForeignWithUsShareholder;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ForeignToForeignNoUsConsequence));
        assert!(r.notes.iter().any(|n| n.contains("Treas. Reg. § 1.367(b)-4")));
        assert!(r.notes.iter().any(|n| n.contains("§ 1248")));
    }

    #[test]
    fn outbound_inventory_full_recognition() {
        let mut i = baseline();
        i.property_type = PropertyType::InventoryOrReceivables;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::OutboundTangibleFullGainRecognition
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Inventory")));
    }

    #[test]
    fn tcja_14102_effective_year_pins_2018() {
        assert_eq!(TCJA_14102_EFFECTIVE_YEAR, 2018);
    }

    #[test]
    fn tcja_14221_intangible_expansion_year_pins_2018() {
        assert_eq!(TCJA_14221_INTANGIBLE_EXPANSION_YEAR, 2018);
    }

    #[test]
    fn active_trade_business_exception_repeal_year_pins_2018() {
        assert_eq!(ACTIVE_TRADE_BUSINESS_EXCEPTION_REPEAL_YEAR, 2018);
    }

    #[test]
    fn action_references_form_926_and_form_1120_schedule_d() {
        let i = baseline();
        let r = check(&i);
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 926")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 1120")));
    }

    #[test]
    fn coordination_note_references_all_international_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_332")));
        assert!(r.notes.iter().any(|n| n.contains("section_351")));
        assert!(r.notes.iter().any(|n| n.contains("section_354")));
        assert!(r.notes.iter().any(|n| n.contains("section_355")));
        assert!(r.notes.iter().any(|n| n.contains("section_361")));
        assert!(r.notes.iter().any(|n| n.contains("section_368")));
        assert!(r.notes.iter().any(|n| n.contains("section_951a")));
        assert!(r.notes.iter().any(|n| n.contains("section_956")));
        assert!(r.notes.iter().any(|n| n.contains("section_959")));
        assert!(r.notes.iter().any(|n| n.contains("section_960")));
        assert!(r.notes.iter().any(|n| n.contains("section_961")));
        assert!(r.notes.iter().any(|n| n.contains("section_245a")));
        assert!(r.notes.iter().any(|n| n.contains("section_1248")));
    }

    #[test]
    fn citation_pins_367_a1_tcja_14102() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 367(a)(1)"));
        assert!(r.citation.contains("TCJA § 14102(e)"));
        assert!(r.citation.contains("Pub. L. 115-97"));
    }

    #[test]
    fn intangible_citation_pins_367_d_and_936_h_3_b() {
        let mut i = baseline();
        i.property_type = PropertyType::Section936IntangibleIncludingGoodwill;
        i.intangible_useful_life_years = 5;
        let r = check(&i);
        assert!(r.citation.contains("§ 367(d)"));
        assert!(r.citation.contains("§ 936(h)(3)(B)"));
        assert!(r.citation.contains("TCJA § 14221"));
    }

    #[test]
    fn gra_citation_pins_367_a8_and_treas_reg_1_367_a_8() {
        let mut i = baseline();
        i.property_type = PropertyType::StockOfForeignCorporationPartyToExchange;
        i.gain_recognition_agreement_elected = true;
        let r = check(&i);
        assert!(r.citation.contains("§ 367(a)(8)"));
        assert!(r.citation.contains("Treas. Reg. § 1.367(a)-8"));
    }

    #[test]
    fn inbound_citation_pins_367b_and_treas_reg_1_367_b_3_and_notice_2024_16() {
        let mut i = baseline();
        i.transfer_direction = TransferDirection::Inbound;
        let r = check(&i);
        assert!(r.citation.contains("§ 367(b)"));
        assert!(r.citation.contains("Treas. Reg. § 1.367(b)-3"));
        assert!(r.citation.contains("Notice 2024-16"));
    }

    #[test]
    fn zero_gain_when_basis_equals_fmv() {
        let mut i = baseline();
        i.adjusted_basis_cents = i.property_fmv_cents;
        let r = check(&i);
        assert_eq!(r.recognized_gain_cents, 0);
    }

    #[test]
    fn zero_gain_when_basis_exceeds_fmv() {
        let mut i = baseline();
        i.adjusted_basis_cents = 200_000_000_00;
        let r = check(&i);
        assert_eq!(r.recognized_gain_cents, 0);
    }

    #[test]
    fn extreme_value_does_not_overflow() {
        let mut i = baseline();
        i.property_fmv_cents = u64::MAX;
        i.adjusted_basis_cents = u64::MAX / 2;
        let r = check(&i);
        let _ = r.recognized_gain_cents;
    }

    #[test]
    fn realistic_corp_transfers_factory_to_ireland_subsidiary() {
        let mut i = baseline();
        i.property_fmv_cents = 50_000_000_000_00;
        i.adjusted_basis_cents = 10_000_000_000_00;
        let r = check(&i);
        assert_eq!(r.recognized_gain_cents, 40_000_000_000_00);
    }
}
