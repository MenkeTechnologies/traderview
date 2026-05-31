//! Required lease disclosures per state — sixth state-data module
//! after `deposit_interest`, `late_fee_caps`, `eviction_notices`,
//! `contractor_1099`, and `deposit_return_windows`.
//!
//! Two layers of mandatory disclosures:
//!
//!   * **Federal Title X** — 42 USC §4852d + Reg. 24 CFR 35 / 40 CFR
//!     745: any **target housing** (residential built before 1978) must
//!     disclose known lead-based paint hazards + provide the EPA
//!     pamphlet "Protect Your Family from Lead in Your Home". Applies
//!     in EVERY state. Civil penalty up to $19,507 per violation
//!     (2024 inflation-adjusted).
//!
//!   * **State-specific disclosures** — mold, bedbug, sex offender /
//!     Megan's Law, radon, asbestos, methamphetamine contamination,
//!     truth-in-renting handbook, foreclosure proceedings, military
//!     ordnance, demolition permits, fire safety. Vary widely.
//!
//! Pure data + compute. Caller passes the state code + property facts
//! (year_built, landlord_in_foreclosure, known_lead_hazard, etc.); we
//! return the list of required disclosures with statute citations and
//! penalty exposure.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureType {
    /// Federal Title X — required if property built before 1978.
    LeadPaint,
    /// State-specific mold disclosure.
    Mold,
    /// Bedbug history disclosure (NYC, AZ, ME, CA).
    Bedbug,
    /// Megan's Law / sex-offender database notification.
    SexOffenderRegistry,
    /// State-specific radon disclosure.
    Radon,
    /// Asbestos for buildings built before specific year (varies by state).
    Asbestos,
    /// Methamphetamine contamination history.
    Methamphetamine,
    /// State-mandated truth-in-renting handbook or tenant rights pamphlet.
    TenantRightsHandbook,
    /// Foreclosure proceedings — required when landlord is in active
    /// foreclosure.
    ForeclosureProceedings,
    /// Military ordnance (former military bases).
    MilitaryOrdnance,
    /// Demolition permits filed.
    DemolitionPermit,
    /// Smoking policy disclosure.
    SmokingPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub statute: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosureRule {
    pub state: &'static str,
    pub disclosure: DisclosureType,
    /// Year built must be at or before this for the disclosure to
    /// apply (e.g. 1977 for federal lead paint = "built before 1978").
    /// `None` = no year-built gate.
    pub applies_if_built_before: Option<u32>,
    /// True when disclosure is conditional on a separate fact
    /// (foreclosure status, known hazard, etc.). The compute fn
    /// reads these from the caller's input.
    pub conditional: bool,
    /// Civil penalty exposure for non-compliance (US dollars). Zero
    /// when statute doesn't set a specific number.
    pub max_civil_penalty_usd: u32,
    pub notes: &'static str,
    pub citation: Citation,
}

fn rules() -> &'static [DisclosureRule] {
    static R: once_cell::sync::Lazy<Vec<DisclosureRule>> =
        once_cell::sync::Lazy::new(|| {
            vec![
                // ─── Federal Title X — applies in every state ──────────
                DisclosureRule {
                    state: "*",
                    disclosure: DisclosureType::LeadPaint,
                    applies_if_built_before: Some(1978),
                    conditional: false,
                    max_civil_penalty_usd: 19507,
                    notes: "Federal Title X — 42 USC §4852d + 24 CFR 35 / 40 CFR 745. Required for any target housing (built before 1978). Penalty up to $19,507/violation (2024 adjusted).",
                    citation: Citation {
                        statute: "42 U.S.C. §4852d + 24 CFR 35.92",
                        source: "https://www.epa.gov/lead/real-estate-disclosure",
                    },
                },
                // ─── CA — multiple state-specific disclosures ─────────
                DisclosureRule {
                    state: "CA",
                    disclosure: DisclosureType::Mold,
                    applies_if_built_before: None,
                    conditional: true,
                    max_civil_penalty_usd: 0,
                    notes: "California Health & Safety Code §26147: landlord must disclose visible mold or known mold contamination before lease.",
                    citation: Citation {
                        statute: "Cal. Health & Safety Code §26147",
                        source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=26147.&lawCode=HSC",
                    },
                },
                DisclosureRule {
                    state: "CA",
                    disclosure: DisclosureType::Bedbug,
                    applies_if_built_before: None,
                    conditional: true,
                    max_civil_penalty_usd: 0,
                    notes: "California Civil Code §1954.603: pre-lease bedbug disclosure required, including any prior infestations within the past year.",
                    citation: Citation {
                        statute: "Cal. Civ. Code §1954.603",
                        source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1954.603.&lawCode=CIV",
                    },
                },
                DisclosureRule {
                    state: "CA",
                    disclosure: DisclosureType::SexOffenderRegistry,
                    applies_if_built_before: None,
                    conditional: false,
                    max_civil_penalty_usd: 0,
                    notes: "California Civil Code §2079.10a: lease must reference Megan's Law database (www.meganslaw.ca.gov).",
                    citation: Citation {
                        statute: "Cal. Civ. Code §2079.10a",
                        source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=2079.10a.&lawCode=CIV",
                    },
                },
                DisclosureRule {
                    state: "CA",
                    disclosure: DisclosureType::Asbestos,
                    applies_if_built_before: Some(1981),
                    conditional: false,
                    max_civil_penalty_usd: 0,
                    notes: "California Civil Code §1102.6e: residential buildings built before 1981 must disclose presence or knowledge of asbestos-containing materials.",
                    citation: Citation {
                        statute: "Cal. Civ. Code §1102.6e",
                        source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1102.6.&lawCode=CIV",
                    },
                },
                DisclosureRule {
                    state: "CA",
                    disclosure: DisclosureType::DemolitionPermit,
                    applies_if_built_before: None,
                    conditional: true,
                    max_civil_penalty_usd: 0,
                    notes: "California Civil Code §1940.6: if landlord has applied for demolition permit on the unit, that fact must be disclosed before signing.",
                    citation: Citation {
                        statute: "Cal. Civ. Code §1940.6",
                        source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1940.6.&lawCode=CIV",
                    },
                },
                // ─── NY — bedbug + lead-paint-window-guard ─────────────
                DisclosureRule {
                    state: "NY",
                    disclosure: DisclosureType::Bedbug,
                    applies_if_built_before: None,
                    conditional: true,
                    max_civil_penalty_usd: 0,
                    notes: "NYC Local Law 69 (2017): NYC landlords must provide one-year bedbug history disclosure. State-wide rule pending.",
                    citation: Citation {
                        statute: "NYC Admin. Code §27-2018.1",
                        source: "https://www1.nyc.gov/site/hpd/services-and-information/bed-bug.page",
                    },
                },
                // ─── FL — radon + military ordnance ────────────────────
                DisclosureRule {
                    state: "FL",
                    disclosure: DisclosureType::Radon,
                    applies_if_built_before: None,
                    conditional: false,
                    max_civil_penalty_usd: 0,
                    notes: "Fla. Stat. §404.056(5): radon disclosure must appear in all real estate contracts including residential leases.",
                    citation: Citation {
                        statute: "Fla. Stat. §404.056",
                        source: "https://www.flsenate.gov/Laws/Statutes/2024/0404.056",
                    },
                },
                // ─── NJ — truth in renting + window guards ─────────────
                DisclosureRule {
                    state: "NJ",
                    disclosure: DisclosureType::TenantRightsHandbook,
                    applies_if_built_before: None,
                    conditional: false,
                    max_civil_penalty_usd: 500,
                    notes: "NJSA 46:8-44 to 50: landlord must provide 'Truth in Renting' statement to tenants in buildings of 3+ units. $500 penalty for non-compliance.",
                    citation: Citation {
                        statute: "N.J. Stat. §46:8-46",
                        source: "https://www.nj.gov/dca/codes/publications/truthinrent.html",
                    },
                },
                DisclosureRule {
                    state: "NJ",
                    disclosure: DisclosureType::Radon,
                    applies_if_built_before: None,
                    conditional: true,
                    max_civil_penalty_usd: 0,
                    notes: "NJSA 26:2D-73: landlord must disclose any known radon test results.",
                    citation: Citation {
                        statute: "N.J. Stat. §26:2D-73",
                        source: "https://lis.njleg.state.nj.us/nxt/gateway.dll?f=templates&fn=default.htm",
                    },
                },
                // ─── WA — mold + fire safety ──────────────────────────
                DisclosureRule {
                    state: "WA",
                    disclosure: DisclosureType::Mold,
                    applies_if_built_before: None,
                    conditional: false,
                    max_civil_penalty_usd: 0,
                    notes: "RCW 59.18.060(13): landlord must provide written information from DOH on mold + mold hazard prevention.",
                    citation: Citation {
                        statute: "RCW 59.18.060",
                        source: "https://app.leg.wa.gov/rcw/default.aspx?cite=59.18.060",
                    },
                },
                // ─── VA — mold + flood zone + meth ────────────────────
                DisclosureRule {
                    state: "VA",
                    disclosure: DisclosureType::Mold,
                    applies_if_built_before: None,
                    conditional: false,
                    max_civil_penalty_usd: 0,
                    notes: "Va. Code §55.1-1215: landlord must disclose results of any mold inspection conducted within 5 years.",
                    citation: Citation {
                        statute: "Va. Code §55.1-1215",
                        source: "https://law.lis.virginia.gov/vacode/title55.1/chapter12/section55.1-1215/",
                    },
                },
                DisclosureRule {
                    state: "VA",
                    disclosure: DisclosureType::Methamphetamine,
                    applies_if_built_before: None,
                    conditional: true,
                    max_civil_penalty_usd: 0,
                    notes: "Va. Code §55.1-1217: landlord must disclose known methamphetamine contamination history.",
                    citation: Citation {
                        statute: "Va. Code §55.1-1217",
                        source: "https://law.lis.virginia.gov/vacode/title55.1/chapter12/section55.1-1217/",
                    },
                },
                // ─── MI — Truth in Renting Act ────────────────────────
                DisclosureRule {
                    state: "MI",
                    disclosure: DisclosureType::TenantRightsHandbook,
                    applies_if_built_before: None,
                    conditional: false,
                    max_civil_penalty_usd: 0,
                    notes: "MCL 554.634: Truth in Renting Act — written lease must include statement of tenant rights and prohibited clauses.",
                    citation: Citation {
                        statute: "MCL 554.634",
                        source: "https://www.legislature.mi.gov/Laws/MCL?objectName=mcl-554-634",
                    },
                },
                // ─── OR — smoking policy ──────────────────────────────
                DisclosureRule {
                    state: "OR",
                    disclosure: DisclosureType::SmokingPolicy,
                    applies_if_built_before: None,
                    conditional: false,
                    max_civil_penalty_usd: 0,
                    notes: "ORS 90.220(1)(a): rental agreement must include landlord's smoking policy (no-smoking, smoking allowed in designated areas, etc.).",
                    citation: Citation {
                        statute: "ORS 90.220",
                        source: "https://oregon.public.law/statutes/ors_90.220",
                    },
                },
                // ─── ME — bedbug + radon ──────────────────────────────
                DisclosureRule {
                    state: "ME",
                    disclosure: DisclosureType::Bedbug,
                    applies_if_built_before: None,
                    conditional: true,
                    max_civil_penalty_usd: 0,
                    notes: "14 M.R.S. §6021-A: bedbug disclosure required prior to lease, including any infestation within the prior 12 months.",
                    citation: Citation {
                        statute: "14 M.R.S. §6021-A",
                        source: "https://legislature.maine.gov/statutes/14/title14sec6021-A.html",
                    },
                },
                DisclosureRule {
                    state: "ME",
                    disclosure: DisclosureType::Radon,
                    applies_if_built_before: None,
                    conditional: false,
                    max_civil_penalty_usd: 0,
                    notes: "14 M.R.S. §6030-D: landlords must test for radon every 10 years and disclose results.",
                    citation: Citation {
                        statute: "14 M.R.S. §6030-D",
                        source: "https://legislature.maine.gov/statutes/14/title14sec6030-D.html",
                    },
                },
                // ─── AZ — bedbug ──────────────────────────────────────
                DisclosureRule {
                    state: "AZ",
                    disclosure: DisclosureType::Bedbug,
                    applies_if_built_before: None,
                    conditional: false,
                    max_civil_penalty_usd: 0,
                    notes: "A.R.S. §33-1319: educational information about bedbugs must be provided to new tenants.",
                    citation: Citation {
                        statute: "A.R.S. §33-1319",
                        source: "https://www.azleg.gov/ars/33/01319.htm",
                    },
                },
                // ─── MD — lead paint inspection certificate ──────────
                DisclosureRule {
                    state: "MD",
                    disclosure: DisclosureType::LeadPaint,
                    applies_if_built_before: Some(1978),
                    conditional: false,
                    max_civil_penalty_usd: 0,
                    notes: "Md. Real Prop. §8-208.2 + MD Reduction of Lead Risk Act: pre-1978 rentals must have current lead-paint inspection certificate in addition to federal Title X disclosure.",
                    citation: Citation {
                        statute: "Md. Code Real Prop. §8-208.2 + §6-801",
                        source: "https://mgaleg.maryland.gov/mgawebsite/Laws/StatuteText?article=gen&section=6-801",
                    },
                },
                // ─── CA / OR / NV — foreclosure ───────────────────────
                DisclosureRule {
                    state: "CA",
                    disclosure: DisclosureType::ForeclosureProceedings,
                    applies_if_built_before: None,
                    conditional: true,
                    max_civil_penalty_usd: 2000,
                    notes: "California Civil Code §2924.85: tenant must be notified of recorded notice of default. $2000 + actual damages for non-compliance.",
                    citation: Citation {
                        statute: "Cal. Civ. Code §2924.85",
                        source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=2924.85.&lawCode=CIV",
                    },
                },
                DisclosureRule {
                    state: "OR",
                    disclosure: DisclosureType::ForeclosureProceedings,
                    applies_if_built_before: None,
                    conditional: true,
                    max_civil_penalty_usd: 0,
                    notes: "ORS 86.785: landlord in foreclosure must provide written notice to tenant within 60 days of trustee's notice of default.",
                    citation: Citation {
                        statute: "ORS 86.785",
                        source: "https://oregon.public.law/statutes/ors_86.785",
                    },
                },
            ]
        });
    &R
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyFacts {
    pub year_built: Option<u32>,
    pub landlord_in_foreclosure: bool,
    pub known_lead_hazard: bool,
    pub known_mold_history: bool,
    pub known_bedbug_history_12mo: bool,
    pub known_meth_contamination: bool,
    pub demolition_permit_pending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosuresRequiredInput {
    pub state: String,
    pub facts: PropertyFacts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosureRequirement {
    pub disclosure: DisclosureType,
    pub state_or_federal: String,
    pub max_civil_penalty_usd: u32,
    pub statute: String,
    pub source: String,
    pub notes: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisclosuresRequiredReport {
    pub state_recognized: bool,
    pub required_disclosures: Vec<DisclosureRequirement>,
    pub total_max_civil_penalty_usd: u64,
    pub notes: String,
}

fn applies(rule: &DisclosureRule, facts: &PropertyFacts) -> bool {
    if let Some(threshold) = rule.applies_if_built_before {
        match facts.year_built {
            // Property built BEFORE the threshold year qualifies.
            // "built before 1978" = year_built <= 1977.
            Some(y) if y < threshold => {}
            _ => return false,
        }
    }

    if !rule.conditional {
        return true;
    }

    // Conditional rules — check the facts that gate them.
    match rule.disclosure {
        DisclosureType::LeadPaint => facts.known_lead_hazard,
        DisclosureType::Mold => facts.known_mold_history,
        DisclosureType::Bedbug => facts.known_bedbug_history_12mo,
        DisclosureType::Methamphetamine => facts.known_meth_contamination,
        DisclosureType::ForeclosureProceedings => facts.landlord_in_foreclosure,
        DisclosureType::DemolitionPermit => facts.demolition_permit_pending,
        // Sex offender / radon / asbestos / tenant rights / smoking
        // policy / military ordnance unconditional when listed.
        _ => true,
    }
}

pub fn required_for(input: &DisclosuresRequiredInput) -> DisclosuresRequiredReport {
    let upper = input.state.to_uppercase();
    let mut r = DisclosuresRequiredReport {
        state_recognized: rules()
            .iter()
            .any(|rule| rule.state == upper.as_str()),
        ..DisclosuresRequiredReport::default()
    };

    for rule in rules() {
        let federal = rule.state == "*";
        let matches_state = federal || rule.state.eq_ignore_ascii_case(&upper);
        if !matches_state {
            continue;
        }
        if !applies(rule, &input.facts) {
            continue;
        }
        r.required_disclosures.push(DisclosureRequirement {
            disclosure: rule.disclosure,
            state_or_federal: if federal { "federal".into() } else { rule.state.into() },
            max_civil_penalty_usd: rule.max_civil_penalty_usd,
            statute: rule.citation.statute.into(),
            source: rule.citation.source.into(),
            notes: rule.notes.into(),
        });
        r.total_max_civil_penalty_usd += rule.max_civil_penalty_usd as u64;
    }

    r.notes = if r.required_disclosures.is_empty() {
        format!(
            "no disclosures required for {} given the property facts (post-1978 build, no known hazards, not in foreclosure)",
            upper
        )
    } else {
        format!(
            "{} disclosure(s) required for {}; total max civil penalty exposure ${}",
            r.required_disclosures.len(),
            upper,
            r.total_max_civil_penalty_usd
        )
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts_clean() -> PropertyFacts {
        PropertyFacts {
            year_built: Some(2000),
            landlord_in_foreclosure: false,
            known_lead_hazard: false,
            known_mold_history: false,
            known_bedbug_history_12mo: false,
            known_meth_contamination: false,
            demolition_permit_pending: false,
        }
    }

    #[test]
    fn pre_1978_property_in_any_state_requires_federal_lead_paint() {
        let mut facts = facts_clean();
        facts.year_built = Some(1970);
        let r = required_for(&DisclosuresRequiredInput {
            state: "AL".into(), // state we don't model — federal still applies
            facts,
        });
        assert!(r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::LeadPaint));
        // Federal Title X penalty $19,507.
        assert!(r.total_max_civil_penalty_usd >= 19507);
    }

    #[test]
    fn exactly_1977_built_pre_1978_lead_paint_required() {
        let mut facts = facts_clean();
        facts.year_built = Some(1977);
        let r = required_for(&DisclosuresRequiredInput {
            state: "CA".into(),
            facts,
        });
        assert!(r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::LeadPaint));
    }

    #[test]
    fn exactly_1978_built_post_threshold_no_lead_paint() {
        let mut facts = facts_clean();
        facts.year_built = Some(1978);
        let r = required_for(&DisclosuresRequiredInput {
            state: "CA".into(),
            facts,
        });
        assert!(!r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::LeadPaint));
    }

    #[test]
    fn post_1978_property_with_no_hazards_in_unmodeled_state_returns_no_disclosures() {
        let r = required_for(&DisclosuresRequiredInput {
            state: "AL".into(),
            facts: facts_clean(),
        });
        assert!(r.required_disclosures.is_empty());
        assert!(r.notes.contains("no disclosures required"));
    }

    #[test]
    fn ca_post_1978_no_hazards_still_has_sex_offender_disclosure() {
        // CA Megan's Law disclosure is unconditional.
        let r = required_for(&DisclosuresRequiredInput {
            state: "CA".into(),
            facts: facts_clean(),
        });
        assert!(r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::SexOffenderRegistry));
    }

    #[test]
    fn ca_with_mold_history_adds_mold_disclosure() {
        let mut facts = facts_clean();
        facts.known_mold_history = true;
        let r = required_for(&DisclosuresRequiredInput {
            state: "CA".into(),
            facts,
        });
        assert!(r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::Mold));
    }

    #[test]
    fn ca_without_mold_history_skips_mold_disclosure() {
        let r = required_for(&DisclosuresRequiredInput {
            state: "CA".into(),
            facts: facts_clean(),
        });
        assert!(!r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::Mold));
    }

    #[test]
    fn wa_mold_disclosure_unconditional() {
        // WA: must always provide DOH mold information — not conditional
        // on known history.
        let r = required_for(&DisclosuresRequiredInput {
            state: "WA".into(),
            facts: facts_clean(),
        });
        assert!(r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::Mold));
    }

    #[test]
    fn fl_radon_unconditional() {
        let r = required_for(&DisclosuresRequiredInput {
            state: "FL".into(),
            facts: facts_clean(),
        });
        assert!(r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::Radon));
    }

    #[test]
    fn ca_asbestos_required_for_pre_1981_construction() {
        let mut facts = facts_clean();
        facts.year_built = Some(1970);
        let r = required_for(&DisclosuresRequiredInput {
            state: "CA".into(),
            facts,
        });
        assert!(r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::Asbestos));
    }

    #[test]
    fn ca_asbestos_not_required_for_post_1981_construction() {
        let mut facts = facts_clean();
        facts.year_built = Some(1985);
        let r = required_for(&DisclosuresRequiredInput {
            state: "CA".into(),
            facts,
        });
        assert!(!r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::Asbestos));
    }

    #[test]
    fn ca_foreclosure_disclosure_when_landlord_in_foreclosure() {
        let mut facts = facts_clean();
        facts.landlord_in_foreclosure = true;
        let r = required_for(&DisclosuresRequiredInput {
            state: "CA".into(),
            facts,
        });
        let foreclosure_rule = r
            .required_disclosures
            .iter()
            .find(|d| d.disclosure == DisclosureType::ForeclosureProceedings)
            .expect("should include foreclosure");
        assert_eq!(foreclosure_rule.max_civil_penalty_usd, 2000);
    }

    #[test]
    fn nj_truth_in_renting_unconditional_with_500_penalty() {
        let r = required_for(&DisclosuresRequiredInput {
            state: "NJ".into(),
            facts: facts_clean(),
        });
        let rule = r
            .required_disclosures
            .iter()
            .find(|d| d.disclosure == DisclosureType::TenantRightsHandbook)
            .expect("should include truth in renting");
        assert_eq!(rule.max_civil_penalty_usd, 500);
    }

    #[test]
    fn case_insensitive_state_lookup() {
        let r = required_for(&DisclosuresRequiredInput {
            state: "ca".into(),
            facts: facts_clean(),
        });
        // CA Megan's Law unconditional should appear.
        assert!(r.state_recognized);
        assert!(r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::SexOffenderRegistry));
    }

    #[test]
    fn md_pre_1978_adds_state_lead_paint_atop_federal() {
        // MD has its OWN lead paint statute on top of federal Title X.
        // Both should fire for a pre-1978 MD property.
        let mut facts = facts_clean();
        facts.year_built = Some(1970);
        let r = required_for(&DisclosuresRequiredInput {
            state: "MD".into(),
            facts,
        });
        let lead_paint_count = r
            .required_disclosures
            .iter()
            .filter(|d| d.disclosure == DisclosureType::LeadPaint)
            .count();
        // Federal + MD state-specific = 2 LeadPaint entries.
        assert_eq!(lead_paint_count, 2);
    }

    #[test]
    fn conditional_bedbug_disclosure_only_when_history_known() {
        // CA, NY, ME require bedbug disclosure conditional on history.
        let mut facts = facts_clean();
        facts.known_bedbug_history_12mo = true;
        let with_history = required_for(&DisclosuresRequiredInput {
            state: "CA".into(),
            facts,
        });
        assert!(with_history
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::Bedbug));

        let without_history = required_for(&DisclosuresRequiredInput {
            state: "CA".into(),
            facts: facts_clean(),
        });
        assert!(!without_history
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::Bedbug));
    }

    #[test]
    fn az_bedbug_unconditional() {
        // AZ: bedbug educational info required regardless of history.
        let r = required_for(&DisclosuresRequiredInput {
            state: "AZ".into(),
            facts: facts_clean(),
        });
        assert!(r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::Bedbug));
    }

    #[test]
    fn or_smoking_policy_unconditional() {
        let r = required_for(&DisclosuresRequiredInput {
            state: "OR".into(),
            facts: facts_clean(),
        });
        assert!(r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::SmokingPolicy));
    }

    #[test]
    fn or_foreclosure_disclosure_only_when_landlord_in_foreclosure() {
        let mut facts = facts_clean();
        facts.landlord_in_foreclosure = true;
        let r = required_for(&DisclosuresRequiredInput {
            state: "OR".into(),
            facts,
        });
        assert!(r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::ForeclosureProceedings));
    }

    #[test]
    fn ca_demolition_permit_only_when_pending() {
        let mut facts = facts_clean();
        facts.demolition_permit_pending = true;
        let r = required_for(&DisclosuresRequiredInput {
            state: "CA".into(),
            facts,
        });
        assert!(r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::DemolitionPermit));
    }

    #[test]
    fn unknown_year_built_skips_year_gated_disclosures() {
        let mut facts = facts_clean();
        facts.year_built = None;
        let r = required_for(&DisclosuresRequiredInput {
            state: "CA".into(),
            facts,
        });
        // Federal lead paint requires built-before-1978 — without
        // year_built we err to NOT requiring it (caller should
        // assert if uncertain).
        assert!(!r
            .required_disclosures
            .iter()
            .any(|d| d.disclosure == DisclosureType::LeadPaint));
    }
}
