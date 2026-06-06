//! IRC §864(b)(2) — Trader / investor safe harbor for non-US persons
//! trading US securities and commodities.
//!
//! A non-US person's US-source income is generally taxed as either:
//!
//!   * **Effectively Connected Income** (ECI) under §871/§882 — taxed
//!     on a net basis at graduated rates, US tax return filing
//!     required, US trade or business present.
//!
//!   * **Fixed, Determinable, Annual, or Periodical** (FDAP) income
//!     under §871(a)/§881 — flat 30% withholding at source, no
//!     filing required if all income is FDAP.
//!
//! §864(b)(2) provides a critical **safe harbor**: trading stocks or
//! securities (or commodities) for one's own account is NOT a US
//! trade or business — gains are NEITHER ECI NOR FDAP, falling
//! entirely outside the US tax net for non-US persons. This is the
//! legal basis for:
//!
//!   * Foreign individuals trading US stocks through US brokers
//!     (e.g. Schwab International, Interactive Brokers).
//!   * Cayman / BVI / Dublin master-fund structures used by
//!     hedge funds with US PMs.
//!   * Foreign sovereign-wealth funds running internal trading desks.
//!
//! The **dealer exception** under §864(b)(2)(A) is the load-bearing
//! carve-out: a "dealer in stocks or securities" does NOT qualify
//! for the safe harbor — gains become ECI subject to net US tax. A
//! dealer is someone who buys and sells securities to customers
//! in the ordinary course of trade (Reg. §1.864-2(c)(2)). The key
//! indicator: maintenance of a US office where principal activities
//! (research, deal execution, customer contact) take place.
//!
//! §864(b)(2)(A) treats SECURITIES trading; §864(b)(2)(A)(i)
//! parallel treats COMMODITY trading for own account. Both have
//! the same dealer-exclusion structure.
//!
//! Pure compute. Caller asserts the four-factor test; we return the
//! classification (NotEffectivelyConnected vs EffectivelyConnected)
//! + a list of reasons + a citation to the controlling subsection.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentClass {
    /// §864(b)(2)(A) — stocks, bonds, options, securities.
    Securities,
    /// §864(b)(2)(A)(i) — commodities (futures, physical, derivatives).
    Commodities,
    /// Trader handles both. Safe harbor depends on dealer status
    /// against each class separately.
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GainClassification {
    /// Falls outside the US tax net — §864(b)(2) safe harbor applies.
    /// No US filing required; no 30% withholding (would only apply to
    /// US-source FDAP, which trader gains generally aren't).
    #[default]
    NotEffectivelyConnected,
    /// Subject to net US tax under §871/§882 — US trade or business
    /// established; US return filing required.
    EffectivelyConnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section864B2Input {
    /// Taxpayer is a non-US person (foreign individual, foreign
    /// corporation, foreign partnership). §864(b)(2) is irrelevant
    /// for US persons — they're already subject to net US tax.
    pub non_us_person: bool,
    /// Trading is for own account (not for a US customer base or
    /// foreign-investor base via a dealer-like middleman role).
    /// §864(b)(2)(A) requires own-account treatment.
    pub trades_for_own_account: bool,
    /// Acts as a dealer in securities — buys and sells to customers
    /// in the ordinary course of trade (Reg. §1.864-2(c)(2)).
    /// Negates the safe harbor for securities under §864(b)(2)(B).
    pub acts_as_securities_dealer: bool,
    /// Acts as a dealer in commodities (parallel exclusion).
    pub acts_as_commodities_dealer: bool,
    /// Maintains a US office or fixed place of business primarily
    /// devoted to the trading activity. A US-located PM, research
    /// desk, or execution facility often kicks the trader into ECI
    /// territory irrespective of the dealer test (per §865 / IRC
    /// §864(c)(5) US office rule).
    pub has_us_office_for_trading: bool,
    pub instrument_class: InstrumentClass,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section864B2Result {
    pub classification: GainClassification,
    pub safe_harbor_applies: bool,
    pub controlling_subsection: String,
    pub reasons: Vec<String>,
    pub note: String,
}

pub fn compute(input: &Section864B2Input) -> Section864B2Result {
    let mut r = Section864B2Result::default();

    if !input.non_us_person {
        r.classification = GainClassification::EffectivelyConnected;
        r.safe_harbor_applies = false;
        r.reasons
            .push("§864(b)(2) safe harbor is for non-US persons only".into());
        r.note = "US person — §864(b)(2) does not apply; gains taxed under regular US rules".into();
        return r;
    }

    if !input.trades_for_own_account {
        r.classification = GainClassification::EffectivelyConnected;
        r.safe_harbor_applies = false;
        r.reasons.push(
            "trading not for own account — §864(b)(2)(A) requires proprietary trading".into(),
        );
        r.note =
            "non-own-account activity — gains effectively connected to US trade or business".into();
        return r;
    }

    if input.has_us_office_for_trading {
        r.classification = GainClassification::EffectivelyConnected;
        r.safe_harbor_applies = false;
        r.reasons.push(
            "US office maintained for trading — §864(c)(5) US-office rule attributes income to US TB"
                .into(),
        );
        r.note =
            "US office presence triggers ECI under §864(c)(5) regardless of safe harbor".into();
        return r;
    }

    // Apply dealer exclusion against the relevant instrument class.
    let securities_disqualified = matches!(
        input.instrument_class,
        InstrumentClass::Securities | InstrumentClass::Both
    ) && input.acts_as_securities_dealer;
    let commodities_disqualified = matches!(
        input.instrument_class,
        InstrumentClass::Commodities | InstrumentClass::Both
    ) && input.acts_as_commodities_dealer;

    if securities_disqualified {
        r.reasons
            .push("securities dealer — §864(b)(2)(B) excludes dealers from the safe harbor".into());
    }
    if commodities_disqualified {
        r.reasons.push(
            "commodities dealer — §864(b)(2)(B) excludes commodity dealers from the safe harbor"
                .into(),
        );
    }

    if securities_disqualified || commodities_disqualified {
        r.classification = GainClassification::EffectivelyConnected;
        r.safe_harbor_applies = false;
        r.controlling_subsection = "§864(b)(2)(B)".into();
        r.note = format!(
            "§864(b)(2) safe harbor unavailable: {}",
            r.reasons.join("; ")
        );
        return r;
    }

    r.classification = GainClassification::NotEffectivelyConnected;
    r.safe_harbor_applies = true;
    r.controlling_subsection = match input.instrument_class {
        InstrumentClass::Securities => "§864(b)(2)(A)(ii)".into(),
        InstrumentClass::Commodities => "§864(b)(2)(A)(i)".into(),
        InstrumentClass::Both => "§864(b)(2)(A)".into(),
    };
    r.note = format!(
        "§864(b)(2) safe harbor applies under {}: own-account trading by non-US person, no dealer activity, no US office. Gains NOT effectively connected to US trade or business.",
        r.controlling_subsection
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section864B2Input {
        Section864B2Input {
            non_us_person: true,
            trades_for_own_account: true,
            acts_as_securities_dealer: false,
            acts_as_commodities_dealer: false,
            has_us_office_for_trading: false,
            instrument_class: InstrumentClass::Securities,
        }
    }

    #[test]
    fn non_us_individual_trading_securities_qualifies_for_safe_harbor() {
        let r = compute(&base());
        assert!(r.safe_harbor_applies);
        assert_eq!(
            r.classification,
            GainClassification::NotEffectivelyConnected
        );
        assert_eq!(r.controlling_subsection, "§864(b)(2)(A)(ii)");
    }

    #[test]
    fn us_person_does_not_qualify() {
        let mut i = base();
        i.non_us_person = false;
        let r = compute(&i);
        assert!(!r.safe_harbor_applies);
        assert_eq!(r.classification, GainClassification::EffectivelyConnected);
        assert!(r.reasons[0].contains("non-US"));
    }

    #[test]
    fn non_own_account_trading_does_not_qualify() {
        let mut i = base();
        i.trades_for_own_account = false;
        let r = compute(&i);
        assert!(!r.safe_harbor_applies);
        assert!(r.reasons[0].contains("own account"));
    }

    #[test]
    fn securities_dealer_excluded_under_864_b_2_b() {
        let mut i = base();
        i.acts_as_securities_dealer = true;
        let r = compute(&i);
        assert!(!r.safe_harbor_applies);
        assert_eq!(r.classification, GainClassification::EffectivelyConnected);
        assert_eq!(r.controlling_subsection, "§864(b)(2)(B)");
        assert!(r.reasons[0].contains("securities dealer"));
    }

    #[test]
    fn commodities_dealer_excluded_when_trading_commodities() {
        let mut i = base();
        i.instrument_class = InstrumentClass::Commodities;
        i.acts_as_commodities_dealer = true;
        let r = compute(&i);
        assert!(!r.safe_harbor_applies);
        assert!(r.reasons[0].contains("commodities dealer"));
    }

    #[test]
    fn securities_dealer_status_irrelevant_when_only_trading_commodities() {
        // A securities dealer who is NOT a commodities dealer can still
        // qualify under §864(b)(2)(A)(i) for commodities trading.
        let mut i = base();
        i.instrument_class = InstrumentClass::Commodities;
        i.acts_as_securities_dealer = true; // doesn't matter
        let r = compute(&i);
        assert!(r.safe_harbor_applies);
        assert_eq!(r.controlling_subsection, "§864(b)(2)(A)(i)");
    }

    #[test]
    fn commodities_dealer_status_irrelevant_when_only_trading_securities() {
        let mut i = base();
        i.instrument_class = InstrumentClass::Securities;
        i.acts_as_commodities_dealer = true; // doesn't matter
        let r = compute(&i);
        assert!(r.safe_harbor_applies);
    }

    #[test]
    fn both_classes_dealer_in_one_disqualifies() {
        let mut i = base();
        i.instrument_class = InstrumentClass::Both;
        i.acts_as_securities_dealer = true;
        let r = compute(&i);
        assert!(!r.safe_harbor_applies);
    }

    #[test]
    fn both_classes_no_dealer_qualifies() {
        let mut i = base();
        i.instrument_class = InstrumentClass::Both;
        let r = compute(&i);
        assert!(r.safe_harbor_applies);
        assert_eq!(r.controlling_subsection, "§864(b)(2)(A)");
    }

    #[test]
    fn us_office_kicks_out_under_864_c_5() {
        let mut i = base();
        i.has_us_office_for_trading = true;
        let r = compute(&i);
        assert!(!r.safe_harbor_applies);
        assert!(r.reasons[0].contains("US office"));
        assert!(r.note.contains("§864(c)(5)"));
    }

    #[test]
    fn us_office_overrides_otherwise_clean_profile() {
        let mut i = base();
        i.has_us_office_for_trading = true;
        // Everything else clean — US office still kicks out.
        let r = compute(&i);
        assert_eq!(r.classification, GainClassification::EffectivelyConnected);
    }

    #[test]
    fn note_text_distinguishes_safe_harbor_path_from_disqualification() {
        let safe = compute(&base());
        assert!(safe.note.contains("safe harbor applies"));

        let mut i = base();
        i.acts_as_securities_dealer = true;
        let disq = compute(&i);
        assert!(disq.note.contains("unavailable"));
    }

    #[test]
    fn both_classes_dual_dealer_lists_both_reasons() {
        let mut i = base();
        i.instrument_class = InstrumentClass::Both;
        i.acts_as_securities_dealer = true;
        i.acts_as_commodities_dealer = true;
        let r = compute(&i);
        assert_eq!(r.reasons.len(), 2);
        assert!(r.reasons.iter().any(|x| x.contains("securities dealer")));
        assert!(r.reasons.iter().any(|x| x.contains("commodities dealer")));
    }

    #[test]
    fn us_person_check_runs_first_overrides_other_factors() {
        // Even an obviously qualifying profile fails immediately when
        // taxpayer is a US person.
        let mut i = base();
        i.non_us_person = false;
        i.acts_as_securities_dealer = true; // also bad
        let r = compute(&i);
        assert!(!r.safe_harbor_applies);
        // The first reason should be the US-person check, not the
        // dealer check — short-circuit order.
        assert!(r.reasons[0].contains("non-US"));
    }

    #[test]
    fn commodities_only_trading_commodities_dealer_disqualified() {
        let mut i = base();
        i.instrument_class = InstrumentClass::Commodities;
        i.acts_as_commodities_dealer = true;
        let r = compute(&i);
        assert!(!r.safe_harbor_applies);
        assert_eq!(r.controlling_subsection, "§864(b)(2)(B)");
    }

    #[test]
    fn safe_harbor_note_cites_applicable_subsection() {
        let mut i = base();

        i.instrument_class = InstrumentClass::Securities;
        let s = compute(&i);
        assert!(s.note.contains("§864(b)(2)(A)(ii)"));

        i.instrument_class = InstrumentClass::Commodities;
        let c = compute(&i);
        assert!(c.note.contains("§864(b)(2)(A)(i)"));

        i.instrument_class = InstrumentClass::Both;
        let b = compute(&i);
        assert!(b.note.contains("§864(b)(2)(A)"));
    }

    #[test]
    fn short_circuit_own_account_runs_before_dealer_check() {
        let mut i = base();
        i.trades_for_own_account = false;
        i.acts_as_securities_dealer = true;
        let r = compute(&i);
        assert!(!r.safe_harbor_applies);
        assert!(r.reasons[0].contains("own account"));
    }
}
