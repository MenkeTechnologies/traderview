//! IRC § 7502 — Timely mailing treated as timely filing and paying
//! ("mailbox rule"). Critical procedural rule for ALL tax filings
//! (returns, claims for refund, Tax Court petitions, deficiency
//! responses) — the date a filing is mailed (not received) can
//! make the difference between timely and barred.
//!
//! Trader-relevant for every paper-filed Form 1040-X amended
//! return, Form 12153 CDP hearing request, Tax Court petition,
//! protective claim filing, and § 6213(a) deficiency response.
//! When paired with Hallmark Research Collective (jurisdictional
//! § 6213(a) deadline) and Boechler (§ 6330(d)(1) equitable
//! tolling for CDP petitions), § 7502 is often the difference
//! between Tax Court jurisdiction and complete loss.
//!
//! § 7502(a)(1) GENERAL RULE — if any return, claim, statement, or
//! other document required to be filed within a prescribed period
//! is, after such period, delivered by United States mail, the
//! date of the United States postmark stamped on the cover shall
//! be deemed to be the date of delivery. § 7502(a)(2) — applies
//! only if the postmark was within the prescribed period and the
//! envelope was properly addressed with sufficient postage.
//!
//! § 7502(b) POSTMARKS MADE OTHER THAN BY USPS — when postmark
//! made by postal employee but not on USPS postmark machine
//! (mostly metered mail), regs at 26 CFR § 301.7502-1 control.
//!
//! § 7502(c) REGISTERED AND CERTIFIED MAIL — § 7502(c)(1) — for
//! registered mail, registration becomes PRIMA FACIE EVIDENCE that
//! the document was delivered. § 7502(c)(2) — for certified mail,
//! the date of registration on the certified mail sender's
//! receipt = postmark date.
//!
//! § 7502(f) PRIVATE DELIVERY SERVICES — § 7502(f)(2) authorizes
//! the Secretary to designate private delivery services (PDS) that
//! qualify for the same timely-mailing treatment. Designated PDSs
//! per Notice 2016-30 (April 11, 2016):
//!   - FedEx: First Overnight, Priority Overnight, Standard
//!     Overnight, 2 Day, International Priority, International
//!     First, International Economy
//!   - UPS: Next Day Air Early AM, Next Day Air, Next Day Air
//!     Saver, 2nd Day Air, 2nd Day Air A.M., Worldwide Express
//!     Plus, Worldwide Express
//!   - DHL Express: Express 9:00, Express 10:30, Express 12:00,
//!     Express Worldwide, Express Envelope, Import Express 10:30,
//!     Import Express 12:00, Import Express Worldwide
//!
//! ONLY the enumerated services qualify — FedEx Ground, UPS
//! Ground, FedEx Home Delivery, and similar non-listed services
//! do NOT qualify under § 7502(f).
//!
//! ELECTRONIC FILING NOTE — § 7502 governs paper filings. Electronic
//! filings (e-file) have their own timely-receipt rules under
//! § 7502(c)(2) / 26 CFR § 301.7502-1(d). E-filed returns are
//! deemed filed on the date of the electronic acknowledgment.
//!
//! COMMON-LAW MAILBOX RULE PRE-EMPTED — Anderson v. United States
//! (5th Cir. 1992) and most circuits hold § 7502 displaces the
//! common-law mailbox rule (where any evidence of mailing creates
//! a presumption of receipt). Taxpayers must comply with § 7502's
//! specific proof requirements; ordinary first-class mail without
//! certified/registered tracking provides no presumption.
//!
//! Citations: IRC § 7502(a)(1) (general timely-mailing rule);
//! § 7502(a)(2) (proper addressing + sufficient postage);
//! § 7502(b) (postmarks other than USPS); § 7502(c)(1) (registered
//! mail prima facie evidence); § 7502(c)(2) (certified mail
//! registration date); § 7502(f) (designated PDS authorization);
//! 26 CFR § 301.7502-1 (regulations); Notice 2016-30 (April 11,
//! 2016 PDS list); Anderson v. United States, 966 F.2d 487
//! (9th Cir. 1992) (§ 7502 displaces common-law mailbox rule
//! in most circuits).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryMethod {
    /// Regular first-class USPS mail (no tracking).
    UspsRegular,
    /// USPS Certified Mail with sender's receipt.
    UspsCertified,
    /// USPS Registered Mail (highest-quality USPS tracking).
    UspsRegistered,
    /// Designated FedEx service per Notice 2016-30 (First Overnight,
    /// Priority Overnight, Standard Overnight, 2 Day, International
    /// Priority/First/Economy).
    FedExDesignated,
    /// Designated UPS service per Notice 2016-30 (Next Day Air
    /// variants, 2nd Day Air variants, Worldwide Express/Express
    /// Plus).
    UpsDesignated,
    /// Designated DHL Express service per Notice 2016-30 (Express
    /// 9:00/10:30/12:00, Worldwide, Envelope, Import Express
    /// variants).
    DhlExpressDesignated,
    /// Non-designated PDS — FedEx Ground, UPS Ground, FedEx Home
    /// Delivery, etc. Does NOT qualify under § 7502(f).
    NonDesignatedPrivateDeliveryService,
    /// Electronic / e-file submission.
    Electronic,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidentiaryStatus {
    /// § 7502(c)(1) prima facie evidence of timely filing
    /// (registered mail with proof of registration).
    PrimaFacieTimely,
    /// § 7502(a) ordinary timely filing — postmark within period
    /// plus proper addressing plus sufficient postage. Burden on
    /// taxpayer to prove postmark date.
    OrdinaryTimely,
    /// No § 7502 protection — non-designated PDS, regular USPS
    /// without proof, or postmark outside the prescribed period.
    NotProtected,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7502Input {
    pub delivery_method: DeliveryMethod,
    /// Whether the postmark / receipt date is within the prescribed
    /// filing period.
    pub postmark_within_prescribed_period: bool,
    /// Whether the envelope was properly addressed with sufficient
    /// postage (§ 7502(a)(2) prerequisite).
    pub properly_addressed_with_sufficient_postage: bool,
    /// Whether the filing arrived AFTER the deadline (and § 7502
    /// is needed to save it).
    pub delivered_after_deadline: bool,
    /// Whether the taxpayer retained the certified mail sender's
    /// receipt or registered mail proof (required for
    /// PrimaFacieTimely status under § 7502(c)).
    pub mail_proof_retained: bool,
    /// For electronic filings — whether the e-file acknowledgment
    /// timestamp is within the prescribed period.
    pub electronic_acknowledgment_within_period: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7502Result {
    pub treated_as_timely_filed: bool,
    pub evidentiary_status: EvidentiaryStatus,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section7502Input) -> Section7502Result {
    let mut notes: Vec<String> = Vec::new();

    if matches!(input.delivery_method, DeliveryMethod::Electronic) {
        let timely = input.electronic_acknowledgment_within_period;
        notes.push(
            "26 CFR § 301.7502-1(d) — e-filed returns deemed filed on date of electronic acknowledgment; § 7502 paper-mailing framework does not apply"
                .to_string(),
        );
        return Section7502Result {
            treated_as_timely_filed: timely,
            evidentiary_status: if timely {
                EvidentiaryStatus::OrdinaryTimely
            } else {
                EvidentiaryStatus::NotProtected
            },
            citation: citation(),
            notes,
        };
    }

    if !input.delivered_after_deadline {
        notes.push(
            "filing arrived ON OR BEFORE the deadline; § 7502 mailbox rule analysis is unnecessary (filing is timely by ordinary receipt)"
                .to_string(),
        );
        return Section7502Result {
            treated_as_timely_filed: true,
            evidentiary_status: EvidentiaryStatus::OrdinaryTimely,
            citation: citation(),
            notes,
        };
    }

    if !input.postmark_within_prescribed_period {
        notes.push(
            "§ 7502(a)(2) — postmark NOT within prescribed period; § 7502 mailbox rule does NOT apply; filing untimely"
                .to_string(),
        );
        return Section7502Result {
            treated_as_timely_filed: false,
            evidentiary_status: EvidentiaryStatus::NotProtected,
            citation: citation(),
            notes,
        };
    }

    if !input.properly_addressed_with_sufficient_postage {
        notes.push(
            "§ 7502(a)(2) — envelope NOT properly addressed or lacks sufficient postage; § 7502 mailbox rule does NOT apply; filing untimely"
                .to_string(),
        );
        return Section7502Result {
            treated_as_timely_filed: false,
            evidentiary_status: EvidentiaryStatus::NotProtected,
            citation: citation(),
            notes,
        };
    }

    match input.delivery_method {
        DeliveryMethod::UspsRegistered => {
            if input.mail_proof_retained {
                notes.push(
                    "§ 7502(c)(1) — registered mail with proof of registration constitutes PRIMA FACIE evidence of timely filing"
                        .to_string(),
                );
                Section7502Result {
                    treated_as_timely_filed: true,
                    evidentiary_status: EvidentiaryStatus::PrimaFacieTimely,
                    citation: citation(),
                    notes,
                }
            } else {
                notes.push(
                    "registered mail without retained registration proof — § 7502(c)(1) prima facie status unavailable; falls back to ordinary § 7502(a) treatment"
                        .to_string(),
                );
                Section7502Result {
                    treated_as_timely_filed: true,
                    evidentiary_status: EvidentiaryStatus::OrdinaryTimely,
                    citation: citation(),
                    notes,
                }
            }
        }
        DeliveryMethod::UspsCertified => {
            if input.mail_proof_retained {
                notes.push(
                    "§ 7502(c)(2) — certified mail sender's receipt postmark serves as filing date; PRIMA FACIE evidence of timely filing"
                        .to_string(),
                );
                Section7502Result {
                    treated_as_timely_filed: true,
                    evidentiary_status: EvidentiaryStatus::PrimaFacieTimely,
                    citation: citation(),
                    notes,
                }
            } else {
                notes.push(
                    "certified mail without retained sender's receipt — § 7502(c)(2) prima facie status unavailable; falls back to ordinary § 7502(a) treatment"
                        .to_string(),
                );
                Section7502Result {
                    treated_as_timely_filed: true,
                    evidentiary_status: EvidentiaryStatus::OrdinaryTimely,
                    citation: citation(),
                    notes,
                }
            }
        }
        DeliveryMethod::UspsRegular => {
            notes.push(
                "§ 7502(a) — first-class USPS mail with postmark within period qualifies; burden on taxpayer to prove postmark date (no prima facie status without certified/registered tracking)"
                    .to_string(),
            );
            notes.push(
                "Anderson v. United States (9th Cir. 1992) — § 7502 displaces common-law mailbox rule in most circuits; ordinary first-class evidence may be insufficient to prove postmark date if disputed"
                    .to_string(),
            );
            Section7502Result {
                treated_as_timely_filed: true,
                evidentiary_status: EvidentiaryStatus::OrdinaryTimely,
                citation: citation(),
                notes,
            }
        }
        DeliveryMethod::FedExDesignated => {
            notes.push(
                "§ 7502(f) + Notice 2016-30 — FedEx First Overnight, Priority Overnight, Standard Overnight, 2 Day, or International Priority/First/Economy qualifies; receipt date serves as postmark"
                    .to_string(),
            );
            Section7502Result {
                treated_as_timely_filed: true,
                evidentiary_status: EvidentiaryStatus::OrdinaryTimely,
                citation: citation(),
                notes,
            }
        }
        DeliveryMethod::UpsDesignated => {
            notes.push(
                "§ 7502(f) + Notice 2016-30 — UPS Next Day Air variants, 2nd Day Air variants, or Worldwide Express variants qualify; receipt date serves as postmark"
                    .to_string(),
            );
            Section7502Result {
                treated_as_timely_filed: true,
                evidentiary_status: EvidentiaryStatus::OrdinaryTimely,
                citation: citation(),
                notes,
            }
        }
        DeliveryMethod::DhlExpressDesignated => {
            notes.push(
                "§ 7502(f) + Notice 2016-30 — DHL Express 9:00/10:30/12:00, Worldwide, Envelope, or Import Express variants qualify; receipt date serves as postmark"
                    .to_string(),
            );
            Section7502Result {
                treated_as_timely_filed: true,
                evidentiary_status: EvidentiaryStatus::OrdinaryTimely,
                citation: citation(),
                notes,
            }
        }
        DeliveryMethod::NonDesignatedPrivateDeliveryService => {
            notes.push(
                "§ 7502(f) — non-designated PDS (FedEx Ground, UPS Ground, FedEx Home Delivery, etc.) does NOT qualify; only services enumerated in Notice 2016-30 qualify"
                    .to_string(),
            );
            Section7502Result {
                treated_as_timely_filed: false,
                evidentiary_status: EvidentiaryStatus::NotProtected,
                citation: citation(),
                notes,
            }
        }
        DeliveryMethod::Electronic => unreachable!("electronic path handled above"),
    }
}

fn citation() -> &'static str {
    "IRC § 7502(a)(1)/(a)(2)/(b)/(c)(1)/(c)(2)/(f); 26 CFR § 301.7502-1; Notice 2016-30 (April 11, 2016 PDS list); Anderson v. United States, 966 F.2d 487 (9th Cir. 1992)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(method: DeliveryMethod) -> Section7502Input {
        Section7502Input {
            delivery_method: method,
            postmark_within_prescribed_period: true,
            properly_addressed_with_sufficient_postage: true,
            delivered_after_deadline: true,
            mail_proof_retained: true,
            electronic_acknowledgment_within_period: true,
        }
    }

    #[test]
    fn usps_certified_with_receipt_prima_facie() {
        let r = compute(&base(DeliveryMethod::UspsCertified));
        assert!(r.treated_as_timely_filed);
        assert_eq!(r.evidentiary_status, EvidentiaryStatus::PrimaFacieTimely);
        assert!(r.notes.iter().any(|n| n.contains("§ 7502(c)(2)")));
    }

    #[test]
    fn usps_certified_without_receipt_falls_back_to_ordinary() {
        let mut i = base(DeliveryMethod::UspsCertified);
        i.mail_proof_retained = false;
        let r = compute(&i);
        assert!(r.treated_as_timely_filed);
        assert_eq!(r.evidentiary_status, EvidentiaryStatus::OrdinaryTimely);
    }

    #[test]
    fn usps_registered_with_proof_prima_facie() {
        let r = compute(&base(DeliveryMethod::UspsRegistered));
        assert!(r.treated_as_timely_filed);
        assert_eq!(r.evidentiary_status, EvidentiaryStatus::PrimaFacieTimely);
        assert!(r.notes.iter().any(|n| n.contains("§ 7502(c)(1)")));
    }

    #[test]
    fn usps_regular_first_class_ordinary_timely_with_anderson_caveat() {
        let r = compute(&base(DeliveryMethod::UspsRegular));
        assert!(r.treated_as_timely_filed);
        assert_eq!(r.evidentiary_status, EvidentiaryStatus::OrdinaryTimely);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Anderson v. United States") && n.contains("displaces common-law mailbox rule")));
    }

    #[test]
    fn fedex_designated_qualifies() {
        let r = compute(&base(DeliveryMethod::FedExDesignated));
        assert!(r.treated_as_timely_filed);
        assert!(r.notes.iter().any(|n| n.contains("FedEx First Overnight") && n.contains("Notice 2016-30")));
    }

    #[test]
    fn ups_designated_qualifies() {
        let r = compute(&base(DeliveryMethod::UpsDesignated));
        assert!(r.treated_as_timely_filed);
        assert!(r.notes.iter().any(|n| n.contains("UPS Next Day Air variants")));
    }

    #[test]
    fn dhl_express_designated_qualifies() {
        let r = compute(&base(DeliveryMethod::DhlExpressDesignated));
        assert!(r.treated_as_timely_filed);
        assert!(r.notes.iter().any(|n| n.contains("DHL Express 9:00/10:30/12:00")));
    }

    #[test]
    fn non_designated_pds_does_not_qualify() {
        let r = compute(&base(DeliveryMethod::NonDesignatedPrivateDeliveryService));
        assert!(!r.treated_as_timely_filed);
        assert_eq!(r.evidentiary_status, EvidentiaryStatus::NotProtected);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("FedEx Ground, UPS Ground, FedEx Home Delivery")));
    }

    #[test]
    fn postmark_outside_period_not_protected() {
        let mut i = base(DeliveryMethod::UspsCertified);
        i.postmark_within_prescribed_period = false;
        let r = compute(&i);
        assert!(!r.treated_as_timely_filed);
        assert_eq!(r.evidentiary_status, EvidentiaryStatus::NotProtected);
        assert!(r.notes.iter().any(|n| n.contains("postmark NOT within prescribed period")));
    }

    #[test]
    fn improperly_addressed_envelope_not_protected() {
        let mut i = base(DeliveryMethod::UspsCertified);
        i.properly_addressed_with_sufficient_postage = false;
        let r = compute(&i);
        assert!(!r.treated_as_timely_filed);
        assert_eq!(r.evidentiary_status, EvidentiaryStatus::NotProtected);
        assert!(r.notes.iter().any(|n| n.contains("NOT properly addressed")));
    }

    #[test]
    fn filing_arrived_before_deadline_no_7502_needed() {
        let mut i = base(DeliveryMethod::UspsRegular);
        i.delivered_after_deadline = false;
        let r = compute(&i);
        assert!(r.treated_as_timely_filed);
        assert!(r.notes.iter().any(|n| n.contains("ON OR BEFORE the deadline")));
    }

    #[test]
    fn electronic_filing_acknowledgment_within_period_timely() {
        let mut i = base(DeliveryMethod::Electronic);
        i.electronic_acknowledgment_within_period = true;
        let r = compute(&i);
        assert!(r.treated_as_timely_filed);
        assert!(r.notes.iter().any(|n| n.contains("§ 301.7502-1(d)") && n.contains("electronic acknowledgment")));
    }

    #[test]
    fn electronic_filing_acknowledgment_outside_period_untimely() {
        let mut i = base(DeliveryMethod::Electronic);
        i.electronic_acknowledgment_within_period = false;
        let r = compute(&i);
        assert!(!r.treated_as_timely_filed);
        assert_eq!(r.evidentiary_status, EvidentiaryStatus::NotProtected);
    }

    #[test]
    fn citation_pins_all_subsections_and_authorities() {
        let r = compute(&base(DeliveryMethod::UspsRegular));
        assert!(r.citation.contains("§ 7502(a)(1)"));
        assert!(r.citation.contains("(a)(2)"));
        assert!(r.citation.contains("(b)"));
        assert!(r.citation.contains("(c)(1)"));
        assert!(r.citation.contains("(c)(2)"));
        assert!(r.citation.contains("(f)"));
        assert!(r.citation.contains("§ 301.7502-1"));
        assert!(r.citation.contains("Notice 2016-30"));
        assert!(r.citation.contains("Anderson v. United States"));
    }

    #[test]
    fn registered_mail_without_proof_falls_back_to_ordinary() {
        let mut i = base(DeliveryMethod::UspsRegistered);
        i.mail_proof_retained = false;
        let r = compute(&i);
        assert!(r.treated_as_timely_filed);
        assert_eq!(r.evidentiary_status, EvidentiaryStatus::OrdinaryTimely);
    }

    #[test]
    fn fedex_designated_does_not_get_prima_facie_status() {
        let r = compute(&base(DeliveryMethod::FedExDesignated));
        assert_eq!(r.evidentiary_status, EvidentiaryStatus::OrdinaryTimely);
    }

    #[test]
    fn ups_designated_does_not_get_prima_facie_status() {
        let r = compute(&base(DeliveryMethod::UpsDesignated));
        assert_eq!(r.evidentiary_status, EvidentiaryStatus::OrdinaryTimely);
    }

    #[test]
    fn three_pds_invariant_all_qualify_when_postmark_within_period() {
        for method in [
            DeliveryMethod::FedExDesignated,
            DeliveryMethod::UpsDesignated,
            DeliveryMethod::DhlExpressDesignated,
        ] {
            let r = compute(&base(method));
            assert!(r.treated_as_timely_filed, "designated PDS {:?} should qualify", method);
        }
    }

    #[test]
    fn non_designated_pds_uniquely_loses_protection_invariant() {
        let methods_that_protect = [
            DeliveryMethod::UspsRegular,
            DeliveryMethod::UspsCertified,
            DeliveryMethod::UspsRegistered,
            DeliveryMethod::FedExDesignated,
            DeliveryMethod::UpsDesignated,
            DeliveryMethod::DhlExpressDesignated,
        ];
        for method in methods_that_protect {
            let r = compute(&base(method));
            assert!(r.treated_as_timely_filed, "method {:?} should be protected", method);
        }
        let r_none = compute(&base(DeliveryMethod::NonDesignatedPrivateDeliveryService));
        assert!(!r_none.treated_as_timely_filed);
    }

    #[test]
    fn certified_and_registered_uniquely_get_prima_facie_status_invariant() {
        let prima_facie_methods = [DeliveryMethod::UspsCertified, DeliveryMethod::UspsRegistered];
        let ordinary_methods = [
            DeliveryMethod::UspsRegular,
            DeliveryMethod::FedExDesignated,
            DeliveryMethod::UpsDesignated,
            DeliveryMethod::DhlExpressDesignated,
        ];
        for method in prima_facie_methods {
            let r = compute(&base(method));
            assert_eq!(r.evidentiary_status, EvidentiaryStatus::PrimaFacieTimely);
        }
        for method in ordinary_methods {
            let r = compute(&base(method));
            assert_eq!(r.evidentiary_status, EvidentiaryStatus::OrdinaryTimely);
        }
    }

    #[test]
    fn postmark_outside_period_overrides_method_protection() {
        for method in [
            DeliveryMethod::UspsRegistered,
            DeliveryMethod::FedExDesignated,
        ] {
            let mut i = base(method);
            i.postmark_within_prescribed_period = false;
            let r = compute(&i);
            assert!(!r.treated_as_timely_filed, "method {:?} cannot save outside-period postmark", method);
        }
    }

    #[test]
    fn improperly_addressed_overrides_method_protection() {
        for method in [
            DeliveryMethod::UspsRegistered,
            DeliveryMethod::FedExDesignated,
        ] {
            let mut i = base(method);
            i.properly_addressed_with_sufficient_postage = false;
            let r = compute(&i);
            assert!(!r.treated_as_timely_filed);
        }
    }

    #[test]
    fn anderson_caveat_appears_only_for_usps_regular() {
        let r_regular = compute(&base(DeliveryMethod::UspsRegular));
        assert!(r_regular.notes.iter().any(|n| n.contains("Anderson")));

        for method in [
            DeliveryMethod::UspsCertified,
            DeliveryMethod::UspsRegistered,
            DeliveryMethod::FedExDesignated,
        ] {
            let r = compute(&base(method));
            assert!(!r.notes.iter().any(|n| n.contains("Anderson")));
        }
    }

    #[test]
    fn before_deadline_path_dominates_method_check() {
        for method in [
            DeliveryMethod::UspsRegular,
            DeliveryMethod::NonDesignatedPrivateDeliveryService,
        ] {
            let mut i = base(method);
            i.delivered_after_deadline = false;
            let r = compute(&i);
            assert!(r.treated_as_timely_filed, "before-deadline filing always timely regardless of method");
        }
    }

    #[test]
    fn fedex_note_lists_designated_services() {
        let r = compute(&base(DeliveryMethod::FedExDesignated));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Priority Overnight") && n.contains("International Priority/First/Economy")));
    }
}
