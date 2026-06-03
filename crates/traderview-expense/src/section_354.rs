//! IRC § 354 — Exchanges of Stock and Securities in Certain Reorganizations.
//!
//! § 354 provides general nonrecognition treatment for the EXCHANGE of stock
//! and securities pursuant to a § 368(a) reorganization. Shareholders +
//! security holders of a party to a reorganization may exchange their stock
//! and securities for stock and securities of another party to the
//! reorganization without recognizing gain or loss — preserving tax basis
//! and deferring recognition until ultimate disposition.
//!
//! § 354(a)(1) GENERAL RULE: no gain or loss recognized if stock or
//! securities in a corporation that is a party to a reorganization are, in
//! pursuance of the plan of reorganization, exchanged SOLELY for stock or
//! securities in such corporation OR in another corporation a party to the
//! reorganization.
//!
//! § 354(a)(2)(A) SECURITIES PRINCIPAL-AMOUNT BOOT: if (i) principal amount
//! of securities RECEIVED exceeds principal amount of securities
//! SURRENDERED, OR (ii) securities are received and NO securities are
//! surrendered, then the FAIR MARKET VALUE of the excess principal amount
//! is treated as money received (boot) — taxable up to gain realized per §
//! 356(a).
//!
//! § 354(a)(2)(B) "NONQUALIFIED PREFERRED STOCK" treated as boot per §
//! 351(g)(2) definition (mandatory redemption + dividend-yield-only
//! preferred stock features).
//!
//! § 354(b) ADDITIONAL REQUIREMENTS for § 368(a)(1)(D) D-type reorganization
//! (divisive D split): (1) acquiring corp must acquire substantially all
//! assets of transferor; (2) transferor must distribute everything received
//! in the exchange PLUS retained assets to its shareholders. § 354(b)
//! prevents D-type reorgs from avoiding taxation when transferor retains
//! assets.
//!
//! § 354(c) RIGHTS / WARRANTS: rights to acquire stock are treated as
//! "securities" for purposes of § 354 per Treas. Reg. § 1.354-1(e).
//!
//! Coordination: § 354 nonrecognition OVERRIDDEN by § 367 (foreign corp
//! exchanges — see `section_367` iter 524). Basis preserved per § 358
//! substituted basis. Boot recognized via § 356. Holding period tacked per
//! § 1223(1).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReorganizationType {
    /// § 368(a)(1)(A) statutory merger.
    AStatutoryMerger,
    /// § 368(a)(1)(B) stock-for-stock acquisition.
    BStockForStock,
    /// § 368(a)(1)(C) asset acquisition for voting stock.
    CAssetAcquisitionForVotingStock,
    /// § 368(a)(1)(D) divisive split-up / split-off / spin-off — § 354(b)
    /// additional requirements apply.
    DDivisiveSplitWithSection354b,
    /// § 368(a)(1)(E) recapitalization.
    ERecapitalization,
    /// § 368(a)(1)(F) reorganization (mere change in identity, form, or
    /// place of organization).
    FMereChangeIdentityForm,
    /// § 368(a)(1)(G) bankruptcy reorganization.
    GBankruptcyReorg,
    /// Not a § 368(a) reorganization.
    NotASection368Reorganization,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    FullNonrecognitionUnderSection354a1,
    PartialBootSection354a2aSecuritiesExcess,
    NonqualifiedPreferredStockBootSection354a2b,
    Section354bAdditionalRequirementsNotSatisfied,
    NotASection368ReorganizationFullRecognition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section354Input {
    pub reorganization_type: ReorganizationType,
    /// Principal amount of securities surrendered in cents.
    pub principal_amount_securities_surrendered_cents: u64,
    /// Principal amount of securities received in cents.
    pub principal_amount_securities_received_cents: u64,
    /// Fair market value of nonqualified preferred stock received per §
    /// 354(a)(2)(B) treated as boot.
    pub nonqualified_preferred_stock_received_cents: u64,
    /// Realized gain on exchange in cents (= FMV received - basis
    /// surrendered).
    pub realized_gain_cents: u64,
    /// Whether acquiring corp acquired substantially all assets of
    /// transferor per § 354(b)(1)(A).
    pub substantially_all_assets_acquired: bool,
    /// Whether transferor distributed everything received plus retained
    /// assets per § 354(b)(1)(B).
    pub transferor_distributed_all_received_and_retained_assets: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section354Result {
    pub severity: Severity,
    pub boot_recognized_cents: u64,
    pub nonrecognized_gain_cents: u64,
    pub principal_amount_excess_boot_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const TREAS_REG_354_1_CITATION: &str = "Treas. Reg. § 1.354-1";

pub fn check(input: &Section354Input) -> Section354Result {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if matches!(
        input.reorganization_type,
        ReorganizationType::NotASection368Reorganization
    ) {
        notes.push(
            "Exchange is NOT pursuant to a § 368(a) reorganization plan — § 354 \
             nonrecognition unavailable; full gain recognition under § 1001 general realization \
             and recognition rules. Confirm reorganization-plan documentation including \
             § 368 statutory requirement satisfaction (continuity of interest + continuity \
             of business enterprise + valid business purpose + step-transaction analysis)."
                .to_string(),
        );
        return Section354Result {
            severity: Severity::NotASection368ReorganizationFullRecognition,
            boot_recognized_cents: input.realized_gain_cents,
            nonrecognized_gain_cents: 0,
            principal_amount_excess_boot_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 1001; § 354 (n/a); § 368(a)",
            notes: build_coord_notes(notes),
        };
    }

    if matches!(
        input.reorganization_type,
        ReorganizationType::DDivisiveSplitWithSection354b
    ) && (!input.substantially_all_assets_acquired
        || !input.transferor_distributed_all_received_and_retained_assets)
    {
        let mut missing: Vec<&str> = Vec::new();
        if !input.substantially_all_assets_acquired {
            missing.push(
                "§ 354(b)(1)(A) acquiring corp must acquire substantially all assets of \
                 transferor",
            );
        }
        if !input.transferor_distributed_all_received_and_retained_assets {
            missing.push(
                "§ 354(b)(1)(B) transferor must distribute everything received plus retained \
                 assets to its shareholders",
            );
        }
        actions.push(format!(
            "§ 354(b) ADDITIONAL REQUIREMENTS not satisfied for § 368(a)(1)(D) divisive \
             reorganization: {}. § 354 nonrecognition UNAVAILABLE; full gain recognition \
             under § 1001. Restructure transaction to satisfy § 354(b) plus reconfirm § \
             368(a)(1)(D) divisive-D classification before claiming nonrecognition.",
            missing.join(" PLUS ")
        ));
        return Section354Result {
            severity: Severity::Section354bAdditionalRequirementsNotSatisfied,
            boot_recognized_cents: input.realized_gain_cents,
            nonrecognized_gain_cents: 0,
            principal_amount_excess_boot_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 354(b)(1)(A)-(B); § 368(a)(1)(D)",
            notes: build_coord_notes(notes),
        };
    }

    let principal_excess: u64 = input
        .principal_amount_securities_received_cents
        .saturating_sub(input.principal_amount_securities_surrendered_cents);

    let boot_from_securities: u64 = principal_excess;
    let boot_from_nqps: u64 = input.nonqualified_preferred_stock_received_cents;
    let total_boot: u64 = boot_from_securities.saturating_add(boot_from_nqps);
    let recognized_boot: u64 = total_boot.min(input.realized_gain_cents);
    let nonrecognized: u64 = input.realized_gain_cents.saturating_sub(recognized_boot);

    let severity = if boot_from_nqps > 0 && total_boot > 0 {
        Severity::NonqualifiedPreferredStockBootSection354a2b
    } else if total_boot > 0 {
        Severity::PartialBootSection354a2aSecuritiesExcess
    } else {
        Severity::FullNonrecognitionUnderSection354a1
    };

    actions.push(format!(
        "§ 354 nonrecognition for {:?} reorganization: principal amount of securities \
         surrendered {} cents; principal amount of securities received {} cents; excess \
         principal amount = {} cents (treated as money per § 354(a)(2)(A)). NQPS received \
         under § 354(a)(2)(B) = {} cents (boot per § 351(g)(2) definition). Total boot \
         {} cents; gain recognized = min(total boot, realized gain {}) = {} cents; \
         nonrecognized gain = {} cents. Basis substituted per § 358; holding period tacked \
         per § 1223(1). Report on Form 1120 Schedule M-3 + Form 8865 (foreign partnership) + \
         Form 8865 Schedule O if applicable.",
        input.reorganization_type,
        input.principal_amount_securities_surrendered_cents,
        input.principal_amount_securities_received_cents,
        principal_excess,
        boot_from_nqps,
        total_boot,
        input.realized_gain_cents,
        recognized_boot,
        nonrecognized
    ));

    notes.push(
        "Coordination with [[section_368]] (corporate reorganization framework — § 354 \
         requires § 368(a) plan), [[section_356]] (boot computation — recognizes gain to \
         extent of boot received in § 354/§ 355 exchange), [[section_358]] (substituted \
         basis for property received in § 354/§ 355/§ 356 exchange), [[section_355]] \
         (parallel divisive nonrecognition framework for spin-offs), [[section_361]] \
         (corporate transferor nonrecognition + Treasury Reg. § 1.361-1), [[section_362]] \
         (acquirer's basis in property received), [[section_367]] (foreign corp exchanges \
         override § 354 nonrecognition — iter 524), [[section_1223]] (holding period \
         tacking § 1223(1)), [[section_351]] (corporate formation parallel nonrecognition \
         framework), [[section_332]] (parent-subsidiary liquidation), [[section_1001]] \
         (general realization and recognition rules)."
            .to_string(),
    );

    Section354Result {
        severity,
        boot_recognized_cents: recognized_boot,
        nonrecognized_gain_cents: nonrecognized,
        principal_amount_excess_boot_cents: principal_excess,
        recommended_actions: actions,
        citation: "26 U.S.C. § 354(a)(1)-(c); § 356; § 358; § 368(a); Treas. Reg. § 1.354-1",
        notes,
    }
}

fn build_coord_notes(mut notes: Vec<String>) -> Vec<String> {
    notes.push(
        "Coordination with [[section_368]] (reorganization framework), [[section_356]] \
         (boot computation), [[section_358]] (substituted basis), [[section_355]] \
         (parallel divisive), [[section_361]] (corporate transferor), [[section_362]] \
         (acquirer basis), [[section_367]] (foreign corp override — iter 524), \
         [[section_1223]] (holding period tacking), [[section_351]] (corp formation \
         parallel), [[section_1001]] (general realization)."
            .to_string(),
    );
    notes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section354Input {
        Section354Input {
            reorganization_type: ReorganizationType::AStatutoryMerger,
            principal_amount_securities_surrendered_cents: 100_000_000_00,
            principal_amount_securities_received_cents: 100_000_000_00,
            nonqualified_preferred_stock_received_cents: 0,
            realized_gain_cents: 50_000_000_00,
            substantially_all_assets_acquired: true,
            transferor_distributed_all_received_and_retained_assets: true,
        }
    }

    #[test]
    fn full_nonrecognition_section_354a1() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullNonrecognitionUnderSection354a1));
        assert_eq!(r.boot_recognized_cents, 0);
        assert_eq!(r.nonrecognized_gain_cents, 50_000_000_00);
        assert_eq!(r.principal_amount_excess_boot_cents, 0);
    }

    #[test]
    fn securities_principal_excess_treated_as_boot() {
        let mut i = baseline();
        i.principal_amount_securities_received_cents = 130_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::PartialBootSection354a2aSecuritiesExcess
        ));
        assert_eq!(r.principal_amount_excess_boot_cents, 30_000_000_00);
        assert_eq!(r.boot_recognized_cents, 30_000_000_00);
    }

    #[test]
    fn nonqualified_preferred_stock_treated_as_boot() {
        let mut i = baseline();
        i.nonqualified_preferred_stock_received_cents = 20_000_000_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::NonqualifiedPreferredStockBootSection354a2b
        ));
        assert_eq!(r.boot_recognized_cents, 20_000_000_00);
    }

    #[test]
    fn nqps_plus_principal_excess_combined_boot() {
        let mut i = baseline();
        i.principal_amount_securities_received_cents = 110_000_000_00;
        i.nonqualified_preferred_stock_received_cents = 5_000_000_00;
        let r = check(&i);
        assert_eq!(r.boot_recognized_cents, 15_000_000_00);
    }

    #[test]
    fn boot_capped_at_realized_gain() {
        let mut i = baseline();
        i.principal_amount_securities_received_cents = 1_000_000_000_00;
        i.realized_gain_cents = 10_000_000_00;
        let r = check(&i);
        assert_eq!(r.boot_recognized_cents, 10_000_000_00);
    }

    #[test]
    fn d_reorg_without_substantially_all_fails() {
        let mut i = baseline();
        i.reorganization_type = ReorganizationType::DDivisiveSplitWithSection354b;
        i.substantially_all_assets_acquired = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::Section354bAdditionalRequirementsNotSatisfied
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 354(b)(1)(A)")));
    }

    #[test]
    fn d_reorg_without_full_distribution_fails() {
        let mut i = baseline();
        i.reorganization_type = ReorganizationType::DDivisiveSplitWithSection354b;
        i.transferor_distributed_all_received_and_retained_assets = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::Section354bAdditionalRequirementsNotSatisfied
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 354(b)(1)(B)")));
    }

    #[test]
    fn d_reorg_with_all_requirements_satisfied_compliant() {
        let mut i = baseline();
        i.reorganization_type = ReorganizationType::DDivisiveSplitWithSection354b;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullNonrecognitionUnderSection354a1));
    }

    #[test]
    fn not_a_section_368_reorg_full_recognition() {
        let mut i = baseline();
        i.reorganization_type = ReorganizationType::NotASection368Reorganization;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::NotASection368ReorganizationFullRecognition
        ));
        assert_eq!(r.boot_recognized_cents, i.realized_gain_cents);
        assert!(r.notes.iter().any(|n| n.contains("§ 1001")));
        assert!(r.notes.iter().any(|n| n.contains("continuity of interest")));
    }

    #[test]
    fn b_reorg_full_nonrecognition() {
        let mut i = baseline();
        i.reorganization_type = ReorganizationType::BStockForStock;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullNonrecognitionUnderSection354a1));
    }

    #[test]
    fn c_reorg_full_nonrecognition() {
        let mut i = baseline();
        i.reorganization_type = ReorganizationType::CAssetAcquisitionForVotingStock;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullNonrecognitionUnderSection354a1));
    }

    #[test]
    fn e_recapitalization_full_nonrecognition() {
        let mut i = baseline();
        i.reorganization_type = ReorganizationType::ERecapitalization;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullNonrecognitionUnderSection354a1));
    }

    #[test]
    fn f_reorg_full_nonrecognition() {
        let mut i = baseline();
        i.reorganization_type = ReorganizationType::FMereChangeIdentityForm;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullNonrecognitionUnderSection354a1));
    }

    #[test]
    fn g_bankruptcy_reorg_full_nonrecognition() {
        let mut i = baseline();
        i.reorganization_type = ReorganizationType::GBankruptcyReorg;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FullNonrecognitionUnderSection354a1));
    }

    #[test]
    fn action_references_form_1120_schedule_m_3() {
        let i = baseline();
        let r = check(&i);
        assert!(r.recommended_actions.iter().any(|a| a.contains("Form 1120")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Schedule M-3")));
    }

    #[test]
    fn coordination_note_references_368_356_358_355_361_367() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_368")));
        assert!(r.notes.iter().any(|n| n.contains("section_356")));
        assert!(r.notes.iter().any(|n| n.contains("section_358")));
        assert!(r.notes.iter().any(|n| n.contains("section_355")));
        assert!(r.notes.iter().any(|n| n.contains("section_361")));
        assert!(r.notes.iter().any(|n| n.contains("section_362")));
        assert!(r.notes.iter().any(|n| n.contains("section_367")));
        assert!(r.notes.iter().any(|n| n.contains("section_1223")));
        assert!(r.notes.iter().any(|n| n.contains("section_351")));
    }

    #[test]
    fn citation_pins_354_356_358_368_treas_reg() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 354(a)(1)-(c)"));
        assert!(r.citation.contains("§ 356"));
        assert!(r.citation.contains("§ 358"));
        assert!(r.citation.contains("§ 368(a)"));
        assert!(r.citation.contains("Treas. Reg. § 1.354-1"));
    }

    #[test]
    fn treas_reg_citation_constant_pinned() {
        assert_eq!(TREAS_REG_354_1_CITATION, "Treas. Reg. § 1.354-1");
    }

    #[test]
    fn zero_securities_zero_boot() {
        let mut i = baseline();
        i.principal_amount_securities_surrendered_cents = 0;
        i.principal_amount_securities_received_cents = 0;
        let r = check(&i);
        assert_eq!(r.principal_amount_excess_boot_cents, 0);
        assert!(matches!(r.severity, Severity::FullNonrecognitionUnderSection354a1));
    }

    #[test]
    fn extreme_value_does_not_overflow() {
        let mut i = baseline();
        i.principal_amount_securities_received_cents = u64::MAX;
        i.principal_amount_securities_surrendered_cents = u64::MAX / 2;
        let r = check(&i);
        let _ = r.principal_amount_excess_boot_cents;
    }

    #[test]
    fn securities_received_less_than_surrendered_no_boot() {
        let mut i = baseline();
        i.principal_amount_securities_received_cents = 50_000_000_00;
        i.principal_amount_securities_surrendered_cents = 100_000_000_00;
        let r = check(&i);
        assert_eq!(r.principal_amount_excess_boot_cents, 0);
        assert!(matches!(r.severity, Severity::FullNonrecognitionUnderSection354a1));
    }
}
