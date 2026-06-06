//! State-specific eviction-notice period table.
//!
//! Sibling to `late_fee_caps` and `deposit_interest` — per-state
//! landlord-tenant operations data. Each state's statute sets a
//! minimum notice period for each eviction ground:
//!
//!   * **Pay or Quit** — nonpayment of rent. Shortest period (3-14
//!     days typical), longest 30 in some jurisdictions.
//!   * **Cure or Quit** — curable lease violation (unauthorized pet,
//!     noise, smoking). Tenant has the right to fix the breach
//!     within the notice window and avoid eviction.
//!   * **Unconditional Quit** — non-curable (illegal activity, severe
//!     property damage, repeated violations). Tenant must vacate;
//!     no cure right.
//!   * **No-Cause Termination** — month-to-month tenancy ended with
//!     no specific ground. Several states (NJ, OR, WA, CA in part)
//!     have eliminated or sharply restricted this.
//!
//! Numbers are statutory references; rules change frequently as
//! legislatures amend. Prefer the published statute over this module
//! for legal advice. Many cities (NYC, Seattle, San Francisco) impose
//! stricter requirements than state law — caller's responsibility to
//! layer those on top.
//!
//! Pure data + compute. Caller passes the state code + ground +
//! tenancy length (for the no-cause sliding scale in CA/OR); we
//! return the required notice days + statute citation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub statute: &'static str,
    pub source: &'static str,
}

/// Required notice in days for each eviction ground in a state.
/// `None` for a ground that the state has effectively eliminated
/// (e.g. NJ no-cause for residential tenants under the Anti-Eviction
/// Act) or where statute requires court filing instead of a tenant
/// notice (some states' pay-or-quit).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateNoticeRules {
    pub state: &'static str,
    pub pay_or_quit_days: Option<u32>,
    pub cure_or_quit_days: Option<u32>,
    pub unconditional_quit_days: Option<u32>,
    /// Default no-cause notice for month-to-month tenancies.
    pub no_cause_termination_days: Option<u32>,
    /// Several states scale the no-cause period by tenancy length —
    /// CA: 60 days if tenancy ≥ 1 year (else 30); OR: 30/60/90 by
    /// length. When `true`, caller should use `long_tenancy_no_cause_days`
    /// for tenants ≥ 1 year.
    pub no_cause_scales_with_tenancy: bool,
    pub long_tenancy_no_cause_days: Option<u32>,
    pub just_cause_required: bool,
    pub effective_year: i32,
    pub notes: &'static str,
    pub citation: Citation,
}

fn rules() -> &'static [StateNoticeRules] {
    static R: once_cell::sync::Lazy<Vec<StateNoticeRules>> = once_cell::sync::Lazy::new(|| {
        vec![
            StateNoticeRules {
                state: "AL",
                pay_or_quit_days: Some(7),
                cure_or_quit_days: Some(7),
                unconditional_quit_days: Some(7),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "Alabama Uniform Residential Landlord and Tenant Act.",
                citation: Citation {
                    statute: "Ala. Code §35-9A-421",
                    source: "https://law.justia.com/codes/alabama/title-35/chapter-9a/article-4/section-35-9a-421/",
                },
            },
            StateNoticeRules {
                state: "AZ",
                pay_or_quit_days: Some(5),
                cure_or_quit_days: Some(10),
                unconditional_quit_days: Some(10),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "AZ requires 5-day pay-or-quit; 10-day cure for non-rent breaches.",
                citation: Citation {
                    statute: "A.R.S. §33-1368",
                    source: "https://www.azleg.gov/ars/33/01368.htm",
                },
            },
            StateNoticeRules {
                state: "CA",
                pay_or_quit_days: Some(3),
                cure_or_quit_days: Some(3),
                unconditional_quit_days: Some(3),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: true,
                long_tenancy_no_cause_days: Some(60),
                just_cause_required: true,
                effective_year: 2024,
                notes: "AB1482 (Tenant Protection Act) requires just cause for tenancies > 12 months in most multi-unit rentals. 60-day no-cause notice for tenancies ≥ 1 year. 3-day pay-or-quit (business days, not counting weekends/holidays).",
                citation: Citation {
                    statute: "Cal. Civ. Code §1946.2 + Cal. CCP §1161",
                    source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1946.2.&lawCode=CIV",
                },
            },
            StateNoticeRules {
                state: "CO",
                pay_or_quit_days: Some(10),
                cure_or_quit_days: Some(10),
                unconditional_quit_days: Some(3),
                no_cause_termination_days: Some(21),
                no_cause_scales_with_tenancy: true,
                long_tenancy_no_cause_days: Some(91),
                just_cause_required: false,
                effective_year: 2024,
                notes: "HB21-1121 raised pay-or-quit from 3 to 10 days. No-cause notice scales: 1 day (tenancy ≤ 1 week), 3 days (1 week–1 month), 7 days (1–6 months), 28 days (6 months–1 year), 91 days (≥ 1 year).",
                citation: Citation {
                    statute: "C.R.S. §13-40-104 + §13-40-107",
                    source: "https://leg.colorado.gov/sites/default/files/2021a_1121_signed.pdf",
                },
            },
            StateNoticeRules {
                state: "CT",
                pay_or_quit_days: Some(3),
                cure_or_quit_days: Some(15),
                unconditional_quit_days: Some(3),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "3-day notice to quit for pay-or-quit; 15-day cure for non-rent breaches.",
                citation: Citation {
                    statute: "Conn. Gen. Stat. §47a-23",
                    source: "https://www.cga.ct.gov/current/pub/chap_832.htm",
                },
            },
            StateNoticeRules {
                state: "DC",
                pay_or_quit_days: Some(30),
                cure_or_quit_days: Some(30),
                unconditional_quit_days: Some(30),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: true,
                effective_year: 2024,
                notes: "D.C. requires 30-day notice for all eviction grounds + just cause under Rental Housing Act.",
                citation: Citation {
                    statute: "D.C. Code §42-3505.01",
                    source: "https://code.dccouncil.us/dc/council/code/sections/42-3505.01.html",
                },
            },
            StateNoticeRules {
                state: "FL",
                pay_or_quit_days: Some(3),
                cure_or_quit_days: Some(7),
                unconditional_quit_days: Some(7),
                no_cause_termination_days: Some(15),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "Fla. Stat. §83.56(3) — 3-day notice excluding weekends + court holidays.",
                citation: Citation {
                    statute: "Fla. Stat. §83.56",
                    source: "https://www.flsenate.gov/Laws/Statutes/2024/0083.56",
                },
            },
            StateNoticeRules {
                state: "GA",
                pay_or_quit_days: None,
                cure_or_quit_days: None,
                unconditional_quit_days: None,
                no_cause_termination_days: Some(60),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "Georgia has no statutory notice-before-filing requirement for pay-or-quit — landlord can demand possession and file dispossessory affidavit immediately. 60-day no-cause notice required for landlord ending month-to-month.",
                citation: Citation {
                    statute: "O.C.G.A. §44-7-7",
                    source: "https://law.justia.com/codes/georgia/2022/title-44/chapter-7/article-1/section-44-7-7/",
                },
            },
            StateNoticeRules {
                state: "IL",
                pay_or_quit_days: Some(5),
                cure_or_quit_days: Some(10),
                unconditional_quit_days: Some(10),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "Chicago Residential Landlord and Tenant Ordinance + state Eviction Act. Chicago/Cook County stricter.",
                citation: Citation {
                    statute: "735 ILCS 5/9-209",
                    source: "https://www.ilga.gov/legislation/ilcs/ilcs4.asp?DocName=073500050HArt%2E+IX&ActID=2017",
                },
            },
            StateNoticeRules {
                state: "MA",
                pay_or_quit_days: Some(14),
                cure_or_quit_days: Some(7),
                unconditional_quit_days: Some(7),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "14-day notice to quit for nonpayment per M.G.L. c.186 §11; cure right within 10 days of receipt.",
                citation: Citation {
                    statute: "Mass. Gen. Laws ch.186 §11",
                    source: "https://malegislature.gov/Laws/GeneralLaws/PartII/TitleI/Chapter186/Section11",
                },
            },
            StateNoticeRules {
                state: "MD",
                pay_or_quit_days: Some(10),
                cure_or_quit_days: Some(30),
                unconditional_quit_days: Some(14),
                no_cause_termination_days: Some(60),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "RP §8-401 (nonpayment); 30-day cure for breach; 60-day no-cause for month-to-month per RP §8-402.",
                citation: Citation {
                    statute: "Md. Code Real Prop. §8-401 / §8-402",
                    source: "https://mgaleg.maryland.gov/mgawebsite/Laws/StatuteText?article=gre&section=8-401",
                },
            },
            StateNoticeRules {
                state: "MI",
                pay_or_quit_days: Some(7),
                cure_or_quit_days: Some(7),
                unconditional_quit_days: Some(7),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "7-day notice to quit for nonpayment under MCL 600.5714(1)(a).",
                citation: Citation {
                    statute: "MCL 600.5714",
                    source: "https://www.legislature.mi.gov/Laws/MCL?objectName=mcl-600-5714",
                },
            },
            StateNoticeRules {
                state: "MN",
                pay_or_quit_days: Some(14),
                cure_or_quit_days: Some(14),
                unconditional_quit_days: Some(14),
                no_cause_termination_days: Some(60),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "2024 amendments raised pay-or-quit to 14 days. 60-day no-cause for tenancies > 1 year per Minn. Stat. §504B.135.",
                citation: Citation {
                    statute: "Minn. Stat. §504B.291 + §504B.135",
                    source: "https://www.revisor.mn.gov/statutes/cite/504B",
                },
            },
            StateNoticeRules {
                state: "NC",
                pay_or_quit_days: Some(10),
                cure_or_quit_days: Some(10),
                unconditional_quit_days: Some(10),
                no_cause_termination_days: Some(7),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "NCGS §42-3 — 10 days; 7-day no-cause for week-to-week.",
                citation: Citation {
                    statute: "N.C.G.S. §42-3 + §42-14",
                    source: "https://www.ncleg.gov/EnactedLegislation/Statutes/HTML/ByChapter/Chapter_42.html",
                },
            },
            StateNoticeRules {
                state: "NJ",
                pay_or_quit_days: None,
                cure_or_quit_days: Some(30),
                unconditional_quit_days: Some(3),
                no_cause_termination_days: None,
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: true,
                effective_year: 2024,
                notes: "Anti-Eviction Act — NO no-cause evictions for most residential tenants. Landlord files complaint directly after 5 business days late on rent (no separate pay-or-quit notice required for filing).",
                citation: Citation {
                    statute: "N.J. Stat. §2A:18-61.1 et seq.",
                    source: "https://lis.njleg.state.nj.us/nxt/gateway.dll?f=templates&fn=default.htm",
                },
            },
            StateNoticeRules {
                state: "NY",
                pay_or_quit_days: Some(14),
                cure_or_quit_days: Some(10),
                unconditional_quit_days: Some(10),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: true,
                long_tenancy_no_cause_days: Some(90),
                just_cause_required: false,
                effective_year: 2024,
                notes: "HSTPA 2019: 14-day pay-or-quit (was 3). No-cause: 30 days (<1 year), 60 days (1-2 years), 90 days (≥ 2 years). NYC + ETPA jurisdictions add good-cause protections.",
                citation: Citation {
                    statute: "N.Y. Real Prop. Acts. §711(2) + N.Y. Real Prop. §226-c",
                    source: "https://www.nysenate.gov/legislation/laws/RPA/711",
                },
            },
            StateNoticeRules {
                state: "OH",
                pay_or_quit_days: Some(3),
                cure_or_quit_days: Some(30),
                unconditional_quit_days: Some(3),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "R.C. §1923.04 — 3-day notice to vacate before forcible entry/detainer action.",
                citation: Citation {
                    statute: "Ohio Rev. Code §1923.04",
                    source: "https://codes.ohio.gov/ohio-revised-code/section-1923.04",
                },
            },
            StateNoticeRules {
                state: "OR",
                pay_or_quit_days: Some(10),
                cure_or_quit_days: Some(30),
                unconditional_quit_days: Some(24),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: true,
                long_tenancy_no_cause_days: Some(90),
                just_cause_required: true,
                effective_year: 2024,
                notes: "SB608 (2019) restricts no-cause to first year of tenancy; after 1 year requires qualifying landlord reason + 90-day notice + relocation assistance for 4+ unit buildings. Pay-or-quit: 10 days after 8th day of month (or 144 hours / 6 days fast-track at landlord's election).",
                citation: Citation {
                    statute: "ORS 90.394 + 90.427",
                    source: "https://oregon.public.law/statutes/ors_90.394",
                },
            },
            StateNoticeRules {
                state: "PA",
                pay_or_quit_days: Some(10),
                cure_or_quit_days: Some(15),
                unconditional_quit_days: Some(15),
                no_cause_termination_days: Some(15),
                no_cause_scales_with_tenancy: true,
                long_tenancy_no_cause_days: Some(30),
                just_cause_required: false,
                effective_year: 2024,
                notes: "PA Landlord-Tenant Act: 10-day notice (nonpayment); 15-day breach notice; 15 days no-cause for tenancies ≤ 1 year, 30 days for > 1 year.",
                citation: Citation {
                    statute: "68 Pa. C.S. §250.501",
                    source: "https://www.legis.state.pa.us/cfdocs/legis/LI/uconsCheck.cfm?txtType=HTM&yr=1951&sessInd=0&smthLwInd=0&act=20",
                },
            },
            StateNoticeRules {
                state: "SC",
                pay_or_quit_days: Some(5),
                cure_or_quit_days: Some(14),
                unconditional_quit_days: Some(14),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "SC Code §27-40-710 — 5 days for nonpayment.",
                citation: Citation {
                    statute: "S.C. Code §27-40-710",
                    source: "https://www.scstatehouse.gov/code/t27c040.php",
                },
            },
            StateNoticeRules {
                state: "TN",
                pay_or_quit_days: Some(14),
                cure_or_quit_days: Some(14),
                unconditional_quit_days: Some(3),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "Tenn. Code §66-28-505 — URLTA pay-or-quit 14 days.",
                citation: Citation {
                    statute: "Tenn. Code Ann. §66-28-505",
                    source: "https://www.tn.gov/content/dam/tn/general-services/documents/cpoSurplus/RentalAgreement.pdf",
                },
            },
            StateNoticeRules {
                state: "TX",
                pay_or_quit_days: Some(3),
                cure_or_quit_days: Some(3),
                unconditional_quit_days: Some(3),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "Tex. Prop. Code §24.005 — 3-day notice to vacate.",
                citation: Citation {
                    statute: "Tex. Prop. Code §24.005",
                    source: "https://statutes.capitol.texas.gov/Docs/PR/htm/PR.24.htm",
                },
            },
            StateNoticeRules {
                state: "VA",
                pay_or_quit_days: Some(5),
                cure_or_quit_days: Some(21),
                unconditional_quit_days: Some(30),
                no_cause_termination_days: Some(30),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "Va. Code §55.1-1245 — 5 days for pay-or-quit; 21-day cure with 30-day vacate for non-rent breach.",
                citation: Citation {
                    statute: "Va. Code §55.1-1245",
                    source: "https://law.lis.virginia.gov/vacode/55.1-1245/",
                },
            },
            StateNoticeRules {
                state: "WA",
                pay_or_quit_days: Some(14),
                cure_or_quit_days: Some(10),
                unconditional_quit_days: Some(3),
                no_cause_termination_days: Some(20),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: true,
                effective_year: 2024,
                notes: "RCW 59.18 + 2021 Just Cause Act: 14-day pay-or-quit (raised from 3 in 2019). Just cause required statewide; no-cause largely eliminated except tenant-initiated 20-day termination.",
                citation: Citation {
                    statute: "RCW 59.18.030 + 59.12.030",
                    source: "https://app.leg.wa.gov/RCW/default.aspx?cite=59.18",
                },
            },
            StateNoticeRules {
                state: "WI",
                pay_or_quit_days: Some(5),
                cure_or_quit_days: Some(5),
                unconditional_quit_days: Some(14),
                no_cause_termination_days: Some(28),
                no_cause_scales_with_tenancy: false,
                long_tenancy_no_cause_days: None,
                just_cause_required: false,
                effective_year: 2024,
                notes: "Wis. Stat. §704.17 — 5-day pay-or-quit (or 14-day no-cure for repeat violators within 12 months).",
                citation: Citation {
                    statute: "Wis. Stat. §704.17",
                    source: "https://docs.legis.wisconsin.gov/statutes/statutes/704/17",
                },
            },
        ]
    });
    &R
}

pub fn rule_for(state: &str) -> Option<&'static StateNoticeRules> {
    let upper = state.to_uppercase();
    rules()
        .iter()
        .find(|r| r.state.eq_ignore_ascii_case(&upper))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvictionGround {
    PayOrQuit,
    CureOrQuit,
    UnconditionalQuit,
    NoCauseTermination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeCheckInput {
    pub state: String,
    pub ground: EvictionGround,
    /// Months of tenancy at the time notice is served. Drives the
    /// no-cause scale in CA / NY / OR / PA.
    pub tenancy_months: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeCheckResult {
    pub state_recognized: bool,
    pub ground: EvictionGround,
    /// Days the landlord must wait after serving the notice before
    /// filing for possession. `None` when statute doesn't require a
    /// pre-filing notice (NJ pay-or-quit; GA pay-or-quit).
    pub required_notice_days: Option<u32>,
    pub just_cause_required: bool,
    pub statute: String,
    pub source: String,
    pub notes: String,
}

pub fn check(input: &NoticeCheckInput) -> NoticeCheckResult {
    let rule = match rule_for(&input.state) {
        Some(r) => r,
        None => {
            return NoticeCheckResult {
                state_recognized: false,
                ground: input.ground,
                required_notice_days: None,
                just_cause_required: false,
                statute: String::new(),
                source: String::new(),
                notes: format!(
                    "no eviction-notice statute on file for {} — consult state landlord-tenant code directly",
                    input.state.to_uppercase()
                ),
            };
        }
    };

    let days = match input.ground {
        EvictionGround::PayOrQuit => rule.pay_or_quit_days,
        EvictionGround::CureOrQuit => rule.cure_or_quit_days,
        EvictionGround::UnconditionalQuit => rule.unconditional_quit_days,
        EvictionGround::NoCauseTermination => {
            // Apply the long-tenancy scale when configured. Threshold = 12 mo.
            if rule.no_cause_scales_with_tenancy && input.tenancy_months >= 12 {
                rule.long_tenancy_no_cause_days
                    .or(rule.no_cause_termination_days)
            } else {
                rule.no_cause_termination_days
            }
        }
    };

    NoticeCheckResult {
        state_recognized: true,
        ground: input.ground,
        required_notice_days: days,
        just_cause_required: rule.just_cause_required,
        statute: rule.citation.statute.into(),
        source: rule.citation.source.into(),
        notes: rule.notes.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_pay_or_quit_is_3_days() {
        let r = check(&NoticeCheckInput {
            state: "TX".into(),
            ground: EvictionGround::PayOrQuit,
            tenancy_months: 6,
        });
        assert_eq!(r.required_notice_days, Some(3));
        assert!(r.state_recognized);
        assert!(r.statute.contains("24.005"));
    }

    #[test]
    fn ny_pay_or_quit_is_14_days_post_hstpa() {
        let r = check(&NoticeCheckInput {
            state: "NY".into(),
            ground: EvictionGround::PayOrQuit,
            tenancy_months: 24,
        });
        assert_eq!(r.required_notice_days, Some(14));
    }

    #[test]
    fn ny_no_cause_scales_with_tenancy_to_90_at_2_years() {
        // Long-tenancy default for NY = 90 days (≥ 2 years).
        let r = check(&NoticeCheckInput {
            state: "NY".into(),
            ground: EvictionGround::NoCauseTermination,
            tenancy_months: 24,
        });
        assert_eq!(r.required_notice_days, Some(90));
    }

    #[test]
    fn ny_no_cause_at_short_tenancy_returns_30_default() {
        let r = check(&NoticeCheckInput {
            state: "NY".into(),
            ground: EvictionGround::NoCauseTermination,
            tenancy_months: 6,
        });
        assert_eq!(r.required_notice_days, Some(30));
    }

    #[test]
    fn ca_no_cause_scales_30_to_60_at_1_year() {
        let r_short = check(&NoticeCheckInput {
            state: "CA".into(),
            ground: EvictionGround::NoCauseTermination,
            tenancy_months: 6,
        });
        assert_eq!(r_short.required_notice_days, Some(30));

        let r_long = check(&NoticeCheckInput {
            state: "CA".into(),
            ground: EvictionGround::NoCauseTermination,
            tenancy_months: 24,
        });
        assert_eq!(r_long.required_notice_days, Some(60));
    }

    #[test]
    fn ca_pay_or_quit_3_days() {
        let r = check(&NoticeCheckInput {
            state: "CA".into(),
            ground: EvictionGround::PayOrQuit,
            tenancy_months: 6,
        });
        assert_eq!(r.required_notice_days, Some(3));
        assert!(r.just_cause_required);
    }

    #[test]
    fn co_post_2021_pay_or_quit_is_10_days() {
        let r = check(&NoticeCheckInput {
            state: "CO".into(),
            ground: EvictionGround::PayOrQuit,
            tenancy_months: 12,
        });
        assert_eq!(r.required_notice_days, Some(10));
    }

    #[test]
    fn wa_post_2019_pay_or_quit_is_14_days() {
        // Washington raised from 3 to 14 in 2019.
        let r = check(&NoticeCheckInput {
            state: "WA".into(),
            ground: EvictionGround::PayOrQuit,
            tenancy_months: 12,
        });
        assert_eq!(r.required_notice_days, Some(14));
        assert!(r.just_cause_required);
    }

    #[test]
    fn nj_no_pay_or_quit_notice_required() {
        let r = check(&NoticeCheckInput {
            state: "NJ".into(),
            ground: EvictionGround::PayOrQuit,
            tenancy_months: 12,
        });
        assert_eq!(r.required_notice_days, None);
        assert!(r.just_cause_required);
    }

    #[test]
    fn nj_no_cause_termination_unavailable() {
        let r = check(&NoticeCheckInput {
            state: "NJ".into(),
            ground: EvictionGround::NoCauseTermination,
            tenancy_months: 6,
        });
        // Anti-Eviction Act eliminates no-cause for most residential.
        assert_eq!(r.required_notice_days, None);
    }

    #[test]
    fn ga_no_pre_filing_pay_or_quit() {
        let r = check(&NoticeCheckInput {
            state: "GA".into(),
            ground: EvictionGround::PayOrQuit,
            tenancy_months: 3,
        });
        assert_eq!(r.required_notice_days, None);
    }

    #[test]
    fn dc_requires_30_days_for_all_grounds() {
        for ground in [
            EvictionGround::PayOrQuit,
            EvictionGround::CureOrQuit,
            EvictionGround::UnconditionalQuit,
            EvictionGround::NoCauseTermination,
        ] {
            let r = check(&NoticeCheckInput {
                state: "DC".into(),
                ground,
                tenancy_months: 6,
            });
            assert_eq!(r.required_notice_days, Some(30), "{:?}", ground);
        }
    }

    #[test]
    fn unknown_state_returns_not_recognized() {
        let r = check(&NoticeCheckInput {
            state: "XX".into(),
            ground: EvictionGround::PayOrQuit,
            tenancy_months: 6,
        });
        assert!(!r.state_recognized);
        assert_eq!(r.required_notice_days, None);
        assert!(r.notes.contains("no eviction-notice statute"));
    }

    #[test]
    fn case_insensitive_state_lookup() {
        let r = check(&NoticeCheckInput {
            state: "tx".into(),
            ground: EvictionGround::PayOrQuit,
            tenancy_months: 6,
        });
        assert_eq!(r.required_notice_days, Some(3));
    }

    #[test]
    fn just_cause_jurisdictions_flagged() {
        for state in ["CA", "DC", "NJ", "OR", "WA"] {
            let r = check(&NoticeCheckInput {
                state: state.into(),
                ground: EvictionGround::PayOrQuit,
                tenancy_months: 12,
            });
            assert!(r.just_cause_required, "{state} should require just cause");
        }
    }

    #[test]
    fn states_without_just_cause_not_flagged() {
        for state in ["TX", "FL", "AZ", "AL"] {
            let r = check(&NoticeCheckInput {
                state: state.into(),
                ground: EvictionGround::PayOrQuit,
                tenancy_months: 12,
            });
            assert!(
                !r.just_cause_required,
                "{state} should NOT require just cause"
            );
        }
    }

    #[test]
    fn rule_for_returns_citation_for_known_states() {
        let r = rule_for("CA").unwrap();
        assert!(r.citation.statute.contains("1946.2") || r.citation.statute.contains("CCP"));
        let r = rule_for("TX").unwrap();
        assert!(r.citation.statute.contains("24.005"));
        let r = rule_for("NY").unwrap();
        assert!(r.citation.statute.contains("711"));
    }

    #[test]
    fn or_no_cause_scales_to_90_after_1_year() {
        let r = check(&NoticeCheckInput {
            state: "OR".into(),
            ground: EvictionGround::NoCauseTermination,
            tenancy_months: 18,
        });
        assert_eq!(r.required_notice_days, Some(90));
    }
}
