//! IRC §1235 — Sale or exchange of patents.
//!
//! Provides **automatic long-term capital gain treatment** to a
//! qualifying "holder" who transfers ALL SUBSTANTIAL RIGHTS in a
//! patent, **regardless of holding period**. Even royalty-style
//! periodic payments contingent on use, productivity, or disposition
//! qualify — converting what would otherwise be ordinary royalty
//! income into LTCG.
//!
//! **POST-TCJA importance**: TCJA (P.L. 115-97 § 13314) amended
//! §1221(a)(3) to add "patent, invention, model, design, secret
//! formula, or process held by the taxpayer who created the property"
//! to the list of property EXCLUDED from capital-asset treatment.
//! This means a patent inventor selling OUTSIDE §1235 now gets
//! ordinary income on the gain. §1235 became MORE important after
//! TCJA — it is now the primary path for an inventor or
//! pre-reduction-to-practice financial backer to obtain LTCG ([Tax
//! reform's impact on patents — McDonald Hopkins](https://mcdonaldhopkins.com/insights/news/Tax-reforms-impact-on-patents),
//! [Sunstein LLP — New Tax Bill and Inventors](https://www.sunsteinlaw.com/publications/the-new-tax-bill-and-its-effect-on-inventors-a-capital-result)).
//!
//! **§1235(b) "Holder" definition** — exactly two paths:
//!
//! - Path 1: **Inventor** — any individual whose efforts created the
//!   property.
//! - Path 2: **Financial backer** — any other individual who
//!   acquired interest in the property in exchange for consideration
//!   in money or money's worth paid to the creator BEFORE actual
//!   reduction to practice, PROVIDED that the acquirer is (a) NOT
//!   the employer of the inventor AND (b) NOT a related party to the
//!   inventor under §267(b) modified (see related-party rules below).
//!
//! **§1235(d) related-party modification**: §1235(a) does not apply
//! to transfers between persons in any §267(b) or §707(b)
//! relationship, with **25% substituted for 50%** in the ownership
//! threshold. Importantly, **brothers and sisters are EXCLUDED from
//! related-party status** under the special §1235 modification
//! (unlike normal §267(b) where siblings are related).
//!
//! **"All substantial rights"** (Treas. Reg. § 1.1235-2(b)) — the
//! transfer must NOT be:
//!
//! - Limited geographically within the country of issuance
//! - Limited in duration to less than remaining patent life
//! - Limited in fields of use to less than all the rights
//!
//! **Payment types** (Treas. Reg. § 1.1235-1(b)) — all three qualify:
//!
//! - Lump sum
//! - Periodic payments
//! - Payments contingent on productivity, use, or disposition
//!   (royalties) — the key benefit, since royalties would otherwise
//!   be ordinary income

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HolderPath {
    Inventor,
    FinancialBackerBeforeReductionToPractice,
    NotAHolder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    LumpSum,
    PeriodicFixed,
    ContingentOnProductivityUseOrDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GainCharacter {
    LongTermCapitalGain,
    OrdinaryIncome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1235Input {
    pub transferor_is_inventor: bool,
    /// If not inventor: did they acquire the interest from the inventor
    /// for consideration BEFORE reduction to practice?
    pub acquired_from_inventor_before_reduction_to_practice: bool,
    pub paid_consideration_in_money_or_money_worth: bool,
    pub transferor_is_employer_of_inventor: bool,
    /// True if transferor is a related party to the inventor under
    /// §267(b) modified (25% threshold, brothers/sisters EXCLUDED).
    pub related_party_under_267b_modified: bool,

    // All substantial rights — must be all FALSE to qualify.
    pub geographically_limited_within_country: bool,
    pub duration_less_than_patent_life: bool,
    pub field_of_use_limitation_retained: bool,

    pub payment_type: PaymentType,
    pub gain_amount_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1235Result {
    pub holder_path: HolderPath,
    pub all_substantial_rights_transferred: bool,
    pub related_party_disqualification: bool,
    pub qualifies_for_section_1235: bool,
    pub gain_character: GainCharacter,
    pub auto_ltcg_treatment_applies: bool,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section1235Input) -> Section1235Result {
    // §1235(b) holder determination.
    let holder = if input.transferor_is_inventor {
        HolderPath::Inventor
    } else if input.acquired_from_inventor_before_reduction_to_practice
        && input.paid_consideration_in_money_or_money_worth
        && !input.transferor_is_employer_of_inventor
    {
        HolderPath::FinancialBackerBeforeReductionToPractice
    } else {
        HolderPath::NotAHolder
    };

    // Treas. Reg. § 1.1235-2(b) all substantial rights — none of the
    // three limitations can be present.
    let all_substantial = !input.geographically_limited_within_country
        && !input.duration_less_than_patent_life
        && !input.field_of_use_limitation_retained;

    // §1235(d) related-party disqualification.
    let related_party_blocked = input.related_party_under_267b_modified;

    let qualifies = holder != HolderPath::NotAHolder && all_substantial && !related_party_blocked;

    let (character, ltcg) = if qualifies {
        (GainCharacter::LongTermCapitalGain, true)
    } else {
        (GainCharacter::OrdinaryIncome, false)
    };

    let mut failure_reasons: Vec<String> = Vec::new();
    if holder == HolderPath::NotAHolder && !input.transferor_is_inventor {
        if !input.acquired_from_inventor_before_reduction_to_practice {
            failure_reasons.push(
                "transferor not inventor AND interest not acquired before reduction to practice"
                    .to_string(),
            );
        } else if input.transferor_is_employer_of_inventor {
            failure_reasons.push("transferor is employer of inventor".to_string());
        } else if !input.paid_consideration_in_money_or_money_worth {
            failure_reasons.push("no consideration in money or money worth paid".to_string());
        }
    }
    if !all_substantial {
        let mut limits: Vec<&str> = Vec::new();
        if input.geographically_limited_within_country {
            limits.push("geographic limitation within country");
        }
        if input.duration_less_than_patent_life {
            limits.push("duration less than patent life");
        }
        if input.field_of_use_limitation_retained {
            limits.push("field-of-use limitation retained");
        }
        failure_reasons.push(format!(
            "not all substantial rights ({})",
            limits.join("; ")
        ));
    }
    if related_party_blocked {
        failure_reasons.push("§1235(d) related-party (267(b) modified 25% threshold)".to_string());
    }

    let note = if qualifies {
        format!(
            "§1235 QUALIFIES: holder path {:?}; all substantial rights transferred; not a related-party transfer; payment type {:?} treated as LTCG regardless of holding period. Gain ${} is LTCG.",
            holder, input.payment_type, input.gain_amount_dollars,
        )
    } else {
        format!(
            "§1235 DOES NOT APPLY: gain ${} is ORDINARY income. Failure reason(s): {}. Post-TCJA (P.L. 115-97 §13314) inventors no longer get capital-asset treatment under §1221(a)(3); §1235 is the only LTCG path.",
            input.gain_amount_dollars,
            failure_reasons.join("; "),
        )
    };

    Section1235Result {
        holder_path: holder,
        all_substantial_rights_transferred: all_substantial,
        related_party_disqualification: related_party_blocked,
        qualifies_for_section_1235: qualifies,
        gain_character: character,
        auto_ltcg_treatment_applies: ltcg,
        citation:
            "IRC §1235(a) automatic LTCG on transfer of all substantial rights in patent; §1235(b) holder definition (inventor or pre-reduction-to-practice financial backer); §1235(d) related-party modification (§267(b) modified, 25% threshold, siblings excluded); Treas. Reg. §1.1235-2(b) all-substantial-rights definition; §1221(a)(3) post-TCJA exclusion of inventor's patent from capital-asset treatment"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inventor_base() -> Section1235Input {
        Section1235Input {
            transferor_is_inventor: true,
            acquired_from_inventor_before_reduction_to_practice: false,
            paid_consideration_in_money_or_money_worth: false,
            transferor_is_employer_of_inventor: false,
            related_party_under_267b_modified: false,
            geographically_limited_within_country: false,
            duration_less_than_patent_life: false,
            field_of_use_limitation_retained: false,
            payment_type: PaymentType::LumpSum,
            gain_amount_dollars: 5_000_000,
        }
    }

    fn backer_base() -> Section1235Input {
        Section1235Input {
            transferor_is_inventor: false,
            acquired_from_inventor_before_reduction_to_practice: true,
            paid_consideration_in_money_or_money_worth: true,
            transferor_is_employer_of_inventor: false,
            related_party_under_267b_modified: false,
            geographically_limited_within_country: false,
            duration_less_than_patent_life: false,
            field_of_use_limitation_retained: false,
            payment_type: PaymentType::ContingentOnProductivityUseOrDisposition,
            gain_amount_dollars: 1_000_000,
        }
    }

    // Inventor holder path.

    #[test]
    fn inventor_baseline_qualifies_for_ltcg() {
        let r = compute(&inventor_base());
        assert_eq!(r.holder_path, HolderPath::Inventor);
        assert!(r.qualifies_for_section_1235);
        assert_eq!(r.gain_character, GainCharacter::LongTermCapitalGain);
        assert!(r.auto_ltcg_treatment_applies);
    }

    #[test]
    fn inventor_with_contingent_royalty_payments_still_ltcg() {
        // The key §1235 benefit: royalty-style payments treated as LTCG.
        let mut i = inventor_base();
        i.payment_type = PaymentType::ContingentOnProductivityUseOrDisposition;
        let r = compute(&i);
        assert!(r.qualifies_for_section_1235);
        assert_eq!(r.gain_character, GainCharacter::LongTermCapitalGain);
    }

    // Financial backer path.

    #[test]
    fn backer_acquired_before_reduction_qualifies() {
        let r = compute(&backer_base());
        assert_eq!(
            r.holder_path,
            HolderPath::FinancialBackerBeforeReductionToPractice
        );
        assert!(r.qualifies_for_section_1235);
    }

    #[test]
    fn backer_acquired_after_reduction_to_practice_disqualified() {
        let mut i = backer_base();
        i.acquired_from_inventor_before_reduction_to_practice = false;
        let r = compute(&i);
        assert_eq!(r.holder_path, HolderPath::NotAHolder);
        assert!(!r.qualifies_for_section_1235);
        assert_eq!(r.gain_character, GainCharacter::OrdinaryIncome);
    }

    #[test]
    fn backer_who_is_employer_disqualified() {
        let mut i = backer_base();
        i.transferor_is_employer_of_inventor = true;
        let r = compute(&i);
        assert_eq!(r.holder_path, HolderPath::NotAHolder);
    }

    #[test]
    fn backer_without_consideration_disqualified() {
        let mut i = backer_base();
        i.paid_consideration_in_money_or_money_worth = false;
        let r = compute(&i);
        assert_eq!(r.holder_path, HolderPath::NotAHolder);
    }

    // All substantial rights tests.

    #[test]
    fn geographic_limitation_disqualifies() {
        let mut i = inventor_base();
        i.geographically_limited_within_country = true;
        let r = compute(&i);
        assert!(!r.all_substantial_rights_transferred);
        assert!(!r.qualifies_for_section_1235);
        assert!(r.note.contains("geographic limitation"));
    }

    #[test]
    fn duration_limitation_disqualifies() {
        let mut i = inventor_base();
        i.duration_less_than_patent_life = true;
        let r = compute(&i);
        assert!(!r.all_substantial_rights_transferred);
        assert!(r.note.contains("duration less than patent life"));
    }

    #[test]
    fn field_of_use_limitation_disqualifies() {
        let mut i = inventor_base();
        i.field_of_use_limitation_retained = true;
        let r = compute(&i);
        assert!(!r.all_substantial_rights_transferred);
        assert!(r.note.contains("field-of-use limitation"));
    }

    #[test]
    fn multiple_rights_limitations_all_listed() {
        let mut i = inventor_base();
        i.geographically_limited_within_country = true;
        i.duration_less_than_patent_life = true;
        i.field_of_use_limitation_retained = true;
        let r = compute(&i);
        assert!(!r.qualifies_for_section_1235);
        assert!(r.note.contains("geographic"));
        assert!(r.note.contains("duration"));
        assert!(r.note.contains("field-of-use"));
    }

    // Related-party tests.

    #[test]
    fn related_party_under_267b_modified_disqualifies() {
        let mut i = inventor_base();
        i.related_party_under_267b_modified = true;
        let r = compute(&i);
        assert!(r.related_party_disqualification);
        assert!(!r.qualifies_for_section_1235);
        assert!(r.note.contains("§1235(d) related-party"));
    }

    #[test]
    fn unrelated_party_qualifies_when_holder_and_substantial() {
        let i = inventor_base();
        let r = compute(&i);
        assert!(!r.related_party_disqualification);
        assert!(r.qualifies_for_section_1235);
    }

    // Payment type variants.

    #[test]
    fn lump_sum_payment_qualifies() {
        let mut i = inventor_base();
        i.payment_type = PaymentType::LumpSum;
        let r = compute(&i);
        assert!(r.qualifies_for_section_1235);
    }

    #[test]
    fn periodic_fixed_payment_qualifies() {
        let mut i = inventor_base();
        i.payment_type = PaymentType::PeriodicFixed;
        let r = compute(&i);
        assert!(r.qualifies_for_section_1235);
    }

    #[test]
    fn contingent_payment_qualifies_load_bearing() {
        // The single most important §1235 result: royalty payments
        // (otherwise ordinary income) become LTCG.
        let mut i = inventor_base();
        i.payment_type = PaymentType::ContingentOnProductivityUseOrDisposition;
        let r = compute(&i);
        assert!(r.qualifies_for_section_1235);
        assert_eq!(r.gain_character, GainCharacter::LongTermCapitalGain);
    }

    // Combined failure scenarios.

    #[test]
    fn not_holder_plus_no_substantial_rights_plus_related_party_all_listed() {
        let mut i = backer_base();
        i.acquired_from_inventor_before_reduction_to_practice = false;
        i.geographically_limited_within_country = true;
        i.related_party_under_267b_modified = true;
        let r = compute(&i);
        assert!(!r.qualifies_for_section_1235);
        assert!(r.note.contains("not inventor"));
        assert!(r.note.contains("not all substantial rights"));
        assert!(r.note.contains("§1235(d) related-party"));
    }

    // TCJA / §1221(a)(3) note.

    #[test]
    fn disqualified_note_mentions_tcja_1221_a_3_exclusion() {
        let mut i = inventor_base();
        i.geographically_limited_within_country = true;
        let r = compute(&i);
        assert!(r.note.contains("§13314"));
        assert!(r.note.contains("§1221(a)(3)"));
    }

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&inventor_base());
        assert!(r.citation.contains("§1235(a)"));
        assert!(r.citation.contains("§1235(b)"));
        assert!(r.citation.contains("§1235(d)"));
        assert!(r.citation.contains("§1.1235-2"));
        assert!(r.citation.contains("§1221(a)(3)"));
    }

    // Precision / boundary cases.

    #[test]
    fn very_large_gain_qualifies_full_amount_ltcg() {
        let mut i = inventor_base();
        i.gain_amount_dollars = 100_000_000_000; // $100B
        let r = compute(&i);
        assert!(r.qualifies_for_section_1235);
        assert_eq!(r.gain_character, GainCharacter::LongTermCapitalGain);
    }

    #[test]
    fn zero_gain_qualifies_but_no_gain_treatment() {
        let mut i = inventor_base();
        i.gain_amount_dollars = 0;
        let r = compute(&i);
        assert!(r.qualifies_for_section_1235);
    }

    // Note text.

    #[test]
    fn qualified_note_describes_holder_path() {
        let r = compute(&inventor_base());
        assert!(r.note.contains("§1235 QUALIFIES"));
        assert!(r.note.contains("Inventor"));
        assert!(r.note.contains("LTCG regardless of holding period"));
    }

    #[test]
    fn qualified_backer_note_describes_path() {
        let r = compute(&backer_base());
        assert!(r.note.contains("FinancialBackerBeforeReductionToPractice"));
    }

    #[test]
    fn disqualified_note_says_does_not_apply() {
        let mut i = inventor_base();
        i.related_party_under_267b_modified = true;
        let r = compute(&i);
        assert!(r.note.contains("§1235 DOES NOT APPLY"));
        assert!(r.note.contains("ORDINARY income"));
    }
}
