//! IRC § 6038B — Notice of certain transfers to foreign
//! persons. Reporting obligation for US persons that
//! transfer property to foreign corporations (Form 926) or
//! foreign partnerships (Form 8865). Companion to
//! section_6038a (Form 5472 25%-foreign-owned domestic
//! corp / DRE), section_6038d (Form 8938 individual FATCA),
//! section_367 (transfers to foreign corp gain recognition
//! framework), section_721 (partnership contribution non-
//! recognition rules), section_6501 (assessment SOL —
//! § 6038B non-filing TOLLS § 6501(c)(8)).
//!
//! Trader-critical because trader fact patterns frequently
//! trigger § 6038B:
//! - **Cryptocurrency transfer to foreign exchange or
//!   foreign-incorporated wallet entity** (treated as
//!   property transfer under Notice 2014-21).
//! - **Intangible asset transfer to foreign hedge fund /
//!   family office** — trading algorithms, proprietary
//!   models, software, copyrights, patents, brand IP.
//!   Triggers § 367(d) deemed-sale treatment for
//!   intangibles transferred to foreign corp.
//! - **§ 351 contribution to foreign-incorporated trading
//!   entity** — initial capitalization of offshore trading
//!   structure with appreciated securities, real estate,
//!   or intangible assets.
//! - **§ 721 contribution to foreign partnership trading
//!   vehicle** — fund-of-funds, master/feeder, parallel
//!   structures with foreign GP/LP.
//! - **§ 721(c) related-party gain-deferral method** —
//!   Treas. Reg. § 1.721(c)-3 requires multi-year
//!   reporting + remedial allocations to avoid immediate
//!   gain recognition.
//!
//! **§ 6038B(a)(1)(A) Foreign corp transfer reporting
//! requirement** — US person who transfers property to a
//! foreign corporation in an exchange described in
//! § 332, § 351, § 354, § 355, § 356, or § 361 SHALL file
//! Form 926 reporting:
//! 1. Property transferred description and amount;
//! 2. Foreign corp identity (name, country, EIN if any);
//! 3. Consideration received;
//! 4. § 367(a) or § 367(d) gain recognition position taken.
//!
//! **§ 6038B(a)(1)(B) Foreign partnership transfer
//! reporting requirement** — US person who transfers
//! property to a foreign partnership in a § 721
//! contribution SHALL file Form 8865 reporting same
//! information categories plus § 721(c) gain-deferral
//! method elections.
//!
//! **§ 6038B(b)(1) Monetary penalty — 10% of FMV** — if
//! US person fails to comply with reporting requirement,
//! such person shall pay a penalty equal to **10 percent
//! of the fair market value of the property at the time
//! of the exchange or transfer**.
//!
//! **§ 6038B(b)(1)(A) $100,000 cap** — penalty LIMITED to
//! $100,000 except as provided in § 6038B(b)(1)(B).
//!
//! **§ 6038B(b)(1)(B) Intentional disregard exception —
//! UNCAPPED** — $100,000 cap shall NOT apply if failure
//! due to **INTENTIONAL DISREGARD** of reporting
//! requirement. Penalty = full 10% of FMV with no ceiling
//! when intentional disregard established.
//!
//! **§ 6038B(b)(2) Gain recognition consequence** — if
//! failure to comply with § 6038B reporting requirement,
//! § 367 (and applicable partnership rules) operate to
//! treat the property as if it had been **SOLD AT FAIR
//! MARKET VALUE** at the time of transfer. This forces
//! immediate recognition of all gain in addition to the
//! monetary penalty.
//!
//! **§ 6038B(c) Reasonable cause exception** — Treas.
//! Reg. § 1.6038B-1(f)(3) and § 1.6038B-2(j)(3) —
//! penalty may be abated upon showing of **reasonable
//! cause and not willful neglect**. Reasonable cause
//! requires affirmative showing of due diligence + good
//! faith effort to comply.
//!
//! **Treas. Reg. § 1.6038B-1 (Form 926 — foreign
//! corporations)** — implements § 6038B(a)(1)(A) for
//! transfers to foreign corporations. § 1.6038B-1(b)(3)
//! de minimis cash-only safe harbor: cash transfers of
//! $100,000 or less aggregate per year not required to
//! be reported (limited application).
//!
//! **Treas. Reg. § 1.6038B-2 (Form 8865 — foreign
//! partnerships)** — implements § 6038B(a)(1)(B) for
//! transfers to foreign partnerships. § 1.6038B-2(c)
//! reporting threshold: aggregate $100,000+ contribution
//! within 12-month period OR ANY contribution if US
//! transferor owns at least 10% of foreign partnership
//! immediately after transfer.
//!
//! **§ 6501(c)(8) SOL tolling** — assessment SOL does NOT
//! start running until required § 6038B information is
//! filed; non-filing keeps ASED OPEN INDEFINITELY for the
//! entire tax year.
//!
//! Citations: 26 USC § 6038B(a)-(c); 26 USC § 6501(c)(8);
//! 26 USC § 367(a) and § 367(d); 26 USC § 721 and § 721(c);
//! Treas. Reg. § 1.6038B-1 and § 1.6038B-2; Treas. Reg.
//! § 1.721(c)-3 (gain-deferral method); IRS Form 926;
//! IRS Form 8865; Notice 2014-21 (cryptocurrency
//! property classification); IRM 8.11.5; IRM 20.1.9.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransferType {
    /// § 6038B(a)(1)(A) — transfer to foreign corporation
    /// (Form 926 required); § 332, § 351, § 354, § 355,
    /// § 356, or § 361 exchange.
    ToForeignCorporation,
    /// § 6038B(a)(1)(B) — § 721 contribution to foreign
    /// partnership (Form 8865 required).
    ToForeignPartnership,
    /// Not a § 6038B reportable transfer (domestic
    /// transfer or excepted category).
    NotReportable,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PropertyCategory {
    /// Tangible property (equipment, real estate, inventory).
    Tangible,
    /// Cash only (subject to § 1.6038B-1(b)(3) safe harbor
    /// for foreign corp transfers).
    CashOnly,
    /// Intangible property (algorithms, software, IP,
    /// patents) — triggers § 367(d) deemed-sale treatment.
    Intangible,
    /// Securities or financial instruments.
    Securities,
    /// Cryptocurrency (treated as property per Notice
    /// 2014-21).
    Cryptocurrency,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6038bInput {
    pub transfer_type: TransferType,
    pub property_category: PropertyCategory,
    /// FMV of property transferred at time of exchange in
    /// cents.
    pub fmv_at_transfer_cents: u64,
    /// US transferor's tax basis in transferred property
    /// in cents (gain = FMV - basis).
    pub basis_cents: u64,
    /// Aggregate value of all transfers to same foreign
    /// partnership within 12-month period in cents (for
    /// § 1.6038B-2(c) reporting threshold).
    pub aggregate_12_month_transfers_cents: u64,
    /// US transferor's ownership percentage in foreign
    /// partnership immediately after transfer in basis
    /// points (e.g., 1000 = 10%).
    pub ownership_pct_after_transfer_bps: u32,
    /// Whether Form 926 or Form 8865 was filed for the
    /// transfer.
    pub form_filed: bool,
    /// Whether failure to file was due to intentional
    /// disregard (uncaps the $100K penalty ceiling under
    /// § 6038B(b)(1)(B)).
    pub intentional_disregard: bool,
    /// Whether reasonable cause defense applies under
    /// § 6038B(c) (NOT willful neglect).
    pub reasonable_cause_engaged: bool,
    /// Whether § 721(c) gain-deferral method elected per
    /// Treas. Reg. § 1.721(c)-3 (only applies to foreign
    /// partnership transfers).
    pub gain_deferral_method_elected: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6038bResult {
    pub transfer_type: TransferType,
    pub reporting_required: bool,
    pub monetary_penalty_cents: u64,
    pub penalty_uncapped_intentional_disregard: bool,
    pub gain_recognition_forced_cents: u64,
    pub section_367d_deemed_sale_engaged: bool,
    pub section_6501_c8_sol_tolled: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6038bInput) -> Section6038bResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let reporting_required = match input.transfer_type {
        TransferType::ToForeignCorporation => {
            !matches!(input.property_category, PropertyCategory::CashOnly)
                || input.fmv_at_transfer_cents > 10_000_000
        }
        TransferType::ToForeignPartnership => {
            input.aggregate_12_month_transfers_cents > 10_000_000
                || input.ownership_pct_after_transfer_bps >= 1000
        }
        TransferType::NotReportable => false,
    };

    let mut monetary_penalty_cents: u64 = 0;
    let mut penalty_uncapped_intentional_disregard = false;

    if reporting_required && !input.form_filed && !input.reasonable_cause_engaged {
        let ten_percent_fmv = input.fmv_at_transfer_cents / 10;
        if input.intentional_disregard {
            monetary_penalty_cents = ten_percent_fmv;
            penalty_uncapped_intentional_disregard = true;
            failure_reasons.push(format!(
                "26 USC § 6038B(b)(1)(B) — INTENTIONAL DISREGARD of § 6038B reporting requirement; $100,000 cap does NOT apply; penalty = 10% of FMV (${} cents) UNCAPPED",
                monetary_penalty_cents
            ));
        } else {
            monetary_penalty_cents = ten_percent_fmv.min(10_000_000);
            failure_reasons.push(format!(
                "26 USC § 6038B(b)(1) — failure to file Form 926 (foreign corp) or Form 8865 (foreign partnership) triggers 10% of FMV penalty (${} cents), CAPPED at $100,000 absent intentional disregard",
                monetary_penalty_cents
            ));
        }
    } else if reporting_required && !input.form_filed && input.reasonable_cause_engaged {
        failure_reasons.push(
            "26 USC § 6038B(c) + Treas. Reg. § 1.6038B-1(f)(3) / § 1.6038B-2(j)(3) — reasonable cause defense (NOT willful neglect) may abate penalty upon affirmative showing of due diligence and good faith effort".to_string(),
        );
    }

    let gain_recognition_forced_cents = if reporting_required && !input.form_filed {
        input.fmv_at_transfer_cents.saturating_sub(input.basis_cents)
    } else {
        0
    };

    if gain_recognition_forced_cents > 0 {
        failure_reasons.push(format!(
            "26 USC § 6038B(b)(2) + § 367(a) — failure to comply with § 6038B forces gain recognition AS IF property had been SOLD AT FAIR MARKET VALUE at time of transfer; forced gain = FMV (${}) - basis (${}) = ${} cents",
            input.fmv_at_transfer_cents, input.basis_cents, gain_recognition_forced_cents
        ));
    }

    let section_367d_deemed_sale_engaged = reporting_required
        && matches!(input.property_category, PropertyCategory::Intangible)
        && matches!(input.transfer_type, TransferType::ToForeignCorporation);

    if section_367d_deemed_sale_engaged {
        failure_reasons.push(
            "26 USC § 367(d) — intangible property (algorithms, software, IP, patents) transferred to foreign corp is subject to DEEMED-SALE treatment requiring annual commensurate-with-income inclusion under § 367(d)(2); failure to report on Form 926 + § 6038B exposes US transferor to BOTH § 6038B penalty AND § 367(d) deemed-sale gain recognition".to_string(),
        );
    }

    let section_6501_c8_sol_tolled = reporting_required && !input.form_filed;
    if section_6501_c8_sol_tolled {
        failure_reasons.push(
            "26 USC § 6501(c)(8) — § 6501 assessment SOL does NOT start running until required § 6038B information is filed; non-filing keeps § 6501 ASED OPEN INDEFINITELY for ENTIRE TAX YEAR".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 6038B(a)(1)(A) — US person who transfers property to foreign corporation in § 332/§ 351/§ 354/§ 355/§ 356/§ 361 exchange SHALL file Form 926 reporting transferred property + foreign corp identity + consideration received + § 367(a)/(d) position".to_string(),
        "26 USC § 6038B(a)(1)(B) — US person who transfers property to foreign partnership in § 721 contribution SHALL file Form 8865 reporting same information categories plus § 721(c) gain-deferral method elections".to_string(),
        "26 USC § 6038B(b)(1) — failure to comply with § 6038B reporting triggers MONETARY PENALTY equal to 10 PERCENT OF FAIR MARKET VALUE of the property at time of exchange or transfer".to_string(),
        "26 USC § 6038B(b)(1)(A) — monetary penalty LIMITED to $100,000".to_string(),
        "26 USC § 6038B(b)(1)(B) — $100,000 cap does NOT apply if failure due to INTENTIONAL DISREGARD of reporting requirement; penalty = full 10% of FMV UNCAPPED".to_string(),
        "26 USC § 6038B(b)(2) — failure to comply ALSO forces § 367 gain recognition AS IF property had been SOLD AT FAIR MARKET VALUE at time of transfer (in ADDITION to monetary penalty)".to_string(),
        "26 USC § 6038B(c) + Treas. Reg. § 1.6038B-1(f)(3) and § 1.6038B-2(j)(3) — REASONABLE CAUSE defense (NOT willful neglect) requires affirmative showing of due diligence and good faith effort".to_string(),
        "26 USC § 367(d) + § 367(d)(2) — intangible property (algorithms, software, IP, patents) transferred to foreign corp is subject to DEEMED-SALE treatment requiring annual commensurate-with-income inclusion".to_string(),
        "Treas. Reg. § 1.6038B-2(c) — Form 8865 reporting threshold: aggregate $100,000+ contribution within 12-month period OR ANY contribution if US transferor owns at least 10% of foreign partnership immediately after transfer".to_string(),
        "Treas. Reg. § 1.721(c)-3 — § 721(c) gain-deferral method for related-party foreign partnership transfers; multi-year reporting + remedial allocations required to avoid immediate gain recognition".to_string(),
        "26 USC § 6501(c)(8) — assessment SOL does NOT start running until required § 6038B information return is filed; non-filing keeps § 6501 ASED OPEN INDEFINITELY".to_string(),
        "Notice 2014-21 — cryptocurrency classified as PROPERTY for federal tax purposes; cryptocurrency transfers to foreign exchanges or foreign-incorporated wallet entities trigger § 6038B reporting".to_string(),
    ];

    Section6038bResult {
        transfer_type: input.transfer_type,
        reporting_required,
        monetary_penalty_cents,
        penalty_uncapped_intentional_disregard,
        gain_recognition_forced_cents,
        section_367d_deemed_sale_engaged,
        section_6501_c8_sol_tolled,
        failure_reasons,
        citation: "26 USC § 6038B(a)-(c); 26 USC § 6501(c)(8); 26 USC § 367(a) and § 367(d); 26 USC § 721 and § 721(c); Treas. Reg. § 1.6038B-1 and § 1.6038B-2; Treas. Reg. § 1.721(c)-3; IRS Form 926; IRS Form 8865; Notice 2014-21; IRM 8.11.5; IRM 20.1.9",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn foreign_corp_base() -> Section6038bInput {
        Section6038bInput {
            transfer_type: TransferType::ToForeignCorporation,
            property_category: PropertyCategory::Securities,
            fmv_at_transfer_cents: 50_000_000,
            basis_cents: 10_000_000,
            aggregate_12_month_transfers_cents: 0,
            ownership_pct_after_transfer_bps: 0,
            form_filed: true,
            intentional_disregard: false,
            reasonable_cause_engaged: false,
            gain_deferral_method_elected: false,
        }
    }

    #[test]
    fn foreign_corp_filed_no_penalty() {
        let r = check(&foreign_corp_base());
        assert!(r.reporting_required);
        assert_eq!(r.monetary_penalty_cents, 0);
        assert_eq!(r.gain_recognition_forced_cents, 0);
    }

    #[test]
    fn foreign_corp_failed_to_file_10_percent_penalty_capped() {
        let mut i = foreign_corp_base();
        i.form_filed = false;
        let r = check(&i);
        assert_eq!(r.monetary_penalty_cents, 5_000_000);
        assert!(!r.penalty_uncapped_intentional_disregard);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6038B(b)(1)")
            && f.contains("10% of FMV")));
    }

    #[test]
    fn foreign_corp_failed_to_file_above_100k_cap_clamped() {
        let mut i = foreign_corp_base();
        i.form_filed = false;
        i.fmv_at_transfer_cents = 500_000_000;
        let r = check(&i);
        assert_eq!(r.monetary_penalty_cents, 10_000_000);
    }

    #[test]
    fn foreign_corp_intentional_disregard_uncapped() {
        let mut i = foreign_corp_base();
        i.form_filed = false;
        i.fmv_at_transfer_cents = 500_000_000;
        i.intentional_disregard = true;
        let r = check(&i);
        assert_eq!(r.monetary_penalty_cents, 50_000_000);
        assert!(r.penalty_uncapped_intentional_disregard);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6038B(b)(1)(B)")
            && f.contains("INTENTIONAL DISREGARD")
            && f.contains("UNCAPPED")));
    }

    #[test]
    fn foreign_corp_reasonable_cause_zeros_penalty() {
        let mut i = foreign_corp_base();
        i.form_filed = false;
        i.reasonable_cause_engaged = true;
        let r = check(&i);
        assert_eq!(r.monetary_penalty_cents, 0);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6038B(c)")
            && f.contains("reasonable cause")));
    }

    #[test]
    fn foreign_corp_gain_recognition_forced_on_failure() {
        let mut i = foreign_corp_base();
        i.form_filed = false;
        let r = check(&i);
        assert_eq!(r.gain_recognition_forced_cents, 40_000_000);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6038B(b)(2)")
            && f.contains("AS IF")
            && f.contains("SOLD AT FAIR MARKET VALUE")));
    }

    #[test]
    fn foreign_corp_intangible_engages_section_367d() {
        let mut i = foreign_corp_base();
        i.form_filed = false;
        i.property_category = PropertyCategory::Intangible;
        let r = check(&i);
        assert!(r.section_367d_deemed_sale_engaged);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 367(d)")
            && f.contains("DEEMED-SALE")
            && f.contains("commensurate-with-income")));
    }

    #[test]
    fn foreign_corp_cash_only_under_100k_no_obligation() {
        let mut i = foreign_corp_base();
        i.property_category = PropertyCategory::CashOnly;
        i.fmv_at_transfer_cents = 5_000_000;
        let r = check(&i);
        assert!(!r.reporting_required);
    }

    #[test]
    fn foreign_corp_cash_only_above_100k_reporting_required() {
        let mut i = foreign_corp_base();
        i.property_category = PropertyCategory::CashOnly;
        i.fmv_at_transfer_cents = 15_000_000;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn foreign_corp_cryptocurrency_engages_reporting() {
        let mut i = foreign_corp_base();
        i.property_category = PropertyCategory::Cryptocurrency;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn foreign_partnership_above_100k_aggregate_reporting_required() {
        let mut i = foreign_corp_base();
        i.transfer_type = TransferType::ToForeignPartnership;
        i.aggregate_12_month_transfers_cents = 20_000_000;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn foreign_partnership_under_100k_no_ownership_no_reporting() {
        let mut i = foreign_corp_base();
        i.transfer_type = TransferType::ToForeignPartnership;
        i.aggregate_12_month_transfers_cents = 5_000_000;
        i.ownership_pct_after_transfer_bps = 999;
        let r = check(&i);
        assert!(!r.reporting_required);
    }

    #[test]
    fn foreign_partnership_10_percent_ownership_triggers_reporting() {
        let mut i = foreign_corp_base();
        i.transfer_type = TransferType::ToForeignPartnership;
        i.aggregate_12_month_transfers_cents = 1_000_000;
        i.ownership_pct_after_transfer_bps = 1000;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn foreign_partnership_exactly_100k_boundary() {
        let mut i = foreign_corp_base();
        i.transfer_type = TransferType::ToForeignPartnership;
        i.aggregate_12_month_transfers_cents = 10_000_000;
        i.ownership_pct_after_transfer_bps = 500;
        let r = check(&i);
        assert!(!r.reporting_required);
        i.aggregate_12_month_transfers_cents = 10_000_001;
        let r2 = check(&i);
        assert!(r2.reporting_required);
    }

    #[test]
    fn not_reportable_transfer_no_obligation() {
        let mut i = foreign_corp_base();
        i.transfer_type = TransferType::NotReportable;
        i.form_filed = false;
        let r = check(&i);
        assert!(!r.reporting_required);
        assert_eq!(r.monetary_penalty_cents, 0);
    }

    #[test]
    fn section_6501_c8_sol_tolled_on_non_filing() {
        let mut i = foreign_corp_base();
        i.form_filed = false;
        let r = check(&i);
        assert!(r.section_6501_c8_sol_tolled);
    }

    #[test]
    fn section_6501_c8_sol_not_tolled_when_filed() {
        let r = check(&foreign_corp_base());
        assert!(!r.section_6501_c8_sol_tolled);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&foreign_corp_base());
        assert!(r.citation.contains("§ 6038B(a)-(c)"));
        assert!(r.citation.contains("§ 6501(c)(8)"));
        assert!(r.citation.contains("§ 367(a) and § 367(d)"));
        assert!(r.citation.contains("§ 721 and § 721(c)"));
        assert!(r.citation.contains("Treas. Reg. § 1.6038B-1 and § 1.6038B-2"));
        assert!(r.citation.contains("Treas. Reg. § 1.721(c)-3"));
        assert!(r.citation.contains("Form 926"));
        assert!(r.citation.contains("Form 8865"));
        assert!(r.citation.contains("Notice 2014-21"));
        assert!(r.citation.contains("IRM 8.11.5"));
        assert!(r.citation.contains("IRM 20.1.9"));
    }

    #[test]
    fn note_pins_subsection_a1A_foreign_corp_form_926() {
        let r = check(&foreign_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038B(a)(1)(A)")
            && n.contains("§ 332")
            && n.contains("§ 351")
            && n.contains("§ 354")
            && n.contains("§ 355")
            && n.contains("§ 356")
            && n.contains("§ 361")
            && n.contains("Form 926")));
    }

    #[test]
    fn note_pins_subsection_a1B_foreign_partnership_form_8865() {
        let r = check(&foreign_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038B(a)(1)(B)")
            && n.contains("§ 721")
            && n.contains("Form 8865")
            && n.contains("§ 721(c) gain-deferral")));
    }

    #[test]
    fn note_pins_subsection_b1_10_percent_fmv_penalty() {
        let r = check(&foreign_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038B(b)(1)")
            && n.contains("10 PERCENT OF FAIR MARKET VALUE")));
    }

    #[test]
    fn note_pins_subsection_b1A_100k_cap() {
        let r = check(&foreign_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038B(b)(1)(A)")
            && n.contains("$100,000")));
    }

    #[test]
    fn note_pins_subsection_b1B_intentional_disregard_uncapped() {
        let r = check(&foreign_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038B(b)(1)(B)")
            && n.contains("INTENTIONAL DISREGARD")
            && n.contains("UNCAPPED")));
    }

    #[test]
    fn note_pins_subsection_b2_gain_recognition_forced() {
        let r = check(&foreign_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038B(b)(2)")
            && n.contains("§ 367")
            && n.contains("SOLD AT FAIR MARKET VALUE")));
    }

    #[test]
    fn note_pins_subsection_c_reasonable_cause() {
        let r = check(&foreign_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038B(c)")
            && n.contains("§ 1.6038B-1(f)(3)")
            && n.contains("§ 1.6038B-2(j)(3)")
            && n.contains("REASONABLE CAUSE")));
    }

    #[test]
    fn note_pins_section_367d_intangible_deemed_sale() {
        let r = check(&foreign_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 367(d)")
            && n.contains("DEEMED-SALE")
            && n.contains("commensurate-with-income")));
    }

    #[test]
    fn note_pins_form_8865_100k_or_10_percent_threshold() {
        let r = check(&foreign_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("Treas. Reg. § 1.6038B-2(c)")
            && n.contains("$100,000+ contribution")
            && n.contains("12-month period")
            && n.contains("10% of foreign partnership")));
    }

    #[test]
    fn note_pins_721c_gain_deferral_method() {
        let r = check(&foreign_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("Treas. Reg. § 1.721(c)-3")
            && n.contains("§ 721(c) gain-deferral method")
            && n.contains("multi-year reporting")));
    }

    #[test]
    fn note_pins_6501_c8_sol_tolling() {
        let r = check(&foreign_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6501(c)(8)")
            && n.contains("OPEN INDEFINITELY")));
    }

    #[test]
    fn note_pins_notice_2014_21_cryptocurrency() {
        let r = check(&foreign_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("Notice 2014-21")
            && n.contains("cryptocurrency")
            && n.contains("PROPERTY")));
    }

    #[test]
    fn property_category_truth_table_five_cells() {
        for cat in [
            PropertyCategory::Tangible,
            PropertyCategory::CashOnly,
            PropertyCategory::Intangible,
            PropertyCategory::Securities,
            PropertyCategory::Cryptocurrency,
        ] {
            let mut i = foreign_corp_base();
            i.property_category = cat;
            i.fmv_at_transfer_cents = 50_000_000;
            let r = check(&i);
            assert!(r.reporting_required, "cat={:?}", cat);
        }
    }

    #[test]
    fn intangible_uniquely_engages_367d_deemed_sale_invariant() {
        let mut i_intangible = foreign_corp_base();
        i_intangible.form_filed = false;
        i_intangible.property_category = PropertyCategory::Intangible;
        let r_intangible = check(&i_intangible);
        assert!(r_intangible.section_367d_deemed_sale_engaged);

        for cat in [
            PropertyCategory::Tangible,
            PropertyCategory::CashOnly,
            PropertyCategory::Securities,
            PropertyCategory::Cryptocurrency,
        ] {
            let mut i = foreign_corp_base();
            i.form_filed = false;
            i.property_category = cat;
            let r = check(&i);
            assert!(!r.section_367d_deemed_sale_engaged, "cat={:?}", cat);
        }
    }

    #[test]
    fn intentional_disregard_uniquely_uncaps_penalty_invariant() {
        let mut high_fmv = foreign_corp_base();
        high_fmv.form_filed = false;
        high_fmv.fmv_at_transfer_cents = 1_000_000_000;
        let r_capped = check(&high_fmv);
        assert_eq!(r_capped.monetary_penalty_cents, 10_000_000);
        assert!(!r_capped.penalty_uncapped_intentional_disregard);

        high_fmv.intentional_disregard = true;
        let r_uncapped = check(&high_fmv);
        assert_eq!(r_uncapped.monetary_penalty_cents, 100_000_000);
        assert!(r_uncapped.penalty_uncapped_intentional_disregard);
        assert!(r_uncapped.monetary_penalty_cents > r_capped.monetary_penalty_cents);
    }

    #[test]
    fn defensive_zero_basis_full_gain_recognition() {
        let mut i = foreign_corp_base();
        i.form_filed = false;
        i.basis_cents = 0;
        let r = check(&i);
        assert_eq!(r.gain_recognition_forced_cents, 50_000_000);
    }

    #[test]
    fn defensive_basis_exceeds_fmv_no_negative_gain() {
        let mut i = foreign_corp_base();
        i.form_filed = false;
        i.basis_cents = 100_000_000;
        let r = check(&i);
        assert_eq!(r.gain_recognition_forced_cents, 0);
    }
}
