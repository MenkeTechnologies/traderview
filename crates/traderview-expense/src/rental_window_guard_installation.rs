//! Multi-jurisdictional residential rental window guard
//! installation requirement framework. Trader-landlord
//! critical because child window-fall injuries are among
//! the highest-stakes premises liability claims —
//! wrongful death awards routinely exceed $5M, and many
//! jurisdictions impose STRICT LIABILITY when window
//! guards are required by statute but absent. NYC alone
//! mandates window guards in every apartment of a
//! building with 3+ units where a child age 10 or younger
//! resides; failure exposes landlord to NYC Department of
//! Health civil penalties + tort liability + tenant
//! private rights of action.
//!
//! Companion to rental_bedroom_egress_window,
//! rental_carbon_monoxide_detector,
//! rental_swimming_pool_drain_safety, landlord_security_
//! device_obligations.
//!
//! **NYC Health Code Article 131 § 131.15 + NYC Admin
//! Code § 27-2043.1** — landlord of a multiple dwelling
//! with **3 OR MORE APARTMENTS** must install approved
//! window guards on:
//! 1. Every window in any apartment where **A CHILD AGE
//!    10 OR YOUNGER RESIDES** (with carveout for windows
//!    leading to fire escapes that would be made
//!    inoperable);
//! 2. Every window in public hallways of such buildings
//!    where a child age 10 or younger resides.
//!
//! NYC notice obligations:
//! 1. **30-day lease notice** — landlord must provide
//!    window guard notice form within first 30 days of
//!    occupancy.
//! 2. **Annual notice between January 1 and January 16** —
//!    building owners must send annual notice regarding
//!    window guards each year.
//!
//! NYC enforcement — Department of Health and Mental
//! Hygiene Class C violation; civil penalties up to
//! $1,000 per violation per day under NYC Health Code
//! § 3.11; NYC Administrative Code § 27-2115 ECB
//! penalties for housing maintenance code violations.
//!
//! **Chicago Building Code § 13-196-550** — operable
//! window guards required limiting window opening to **4
//! INCHES OR LESS** in dwelling units. Screen
//! requirements: every door opening and every window must
//! have screens in place from **April 15 through November
//! 15** each year. Burglar bars require landlord written
//! consent before tenant installation; once installed,
//! burglar bars become landlord property (permanent
//! fixture).
//!
//! **Massachusetts G.L. + State Sanitary Code 105 CMR
//! 410** — landlord must install window guards at
//! TENANT'S REQUEST when a child under age 10 resides;
//! applicable to **"applicable windows"** — windows that:
//! 1. Are GREATER THAN 6 FEET ABOVE GRADE; AND
//! 2. Are capable of opening sufficiently to allow a
//!    **5-INCH DIAMETER BALL** to pass through; AND
//! 3. Are NOT connected to a fire escape.
//!
//! Annual MA notice obligation: at beginning of tenancy
//! AND at least annually thereafter, landlord must
//! provide tenant with notice stating: "Parents with
//! children under the age of 10 have the right, at no
//! additional charge, to have window guards installed
//! within the rented apartment and the common areas of
//! the building."
//!
//! **Montgomery County Maryland DHCA Code § 29-23** —
//! window guard lease addendum required for tenants with
//! children under age 6; landlord must install guards
//! at tenant request.
//!
//! **Default** — no statewide window guard mandate;
//! general premises liability + common-law negligence
//! apply; ASTM F2090-23 voluntary standard for window
//! fall prevention devices may inform negligence
//! analysis.
//!
//! Citations: NYC Health Code Article 131 § 131.15; NYC
//! Admin Code § 27-2043.1; NYC Health Code § 3.11; NYC
//! Admin Code § 27-2115; Chicago Building Code
//! § 13-196-550; Mass. G.L. + 105 CMR 410; Montgomery
//! County MD DHCA Code § 29-23; ASTM F2090-23 (voluntary
//! standard for window fall prevention devices).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewYorkCity,
    Chicago,
    Massachusetts,
    MontgomeryCountyMd,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalWindowGuardInstallationInput {
    pub jurisdiction: Jurisdiction,
    /// Number of dwelling units in building (NYC: 3+
    /// threshold).
    pub dwelling_unit_count: u32,
    /// Whether a child age 10 or younger resides in the
    /// apartment (NYC + Boston trigger).
    pub child_age_10_or_younger_resides: bool,
    /// Whether a child age 6 or younger resides
    /// (Montgomery County MD trigger).
    pub child_age_6_or_younger_resides: bool,
    /// Whether tenant requested window guard installation
    /// (MA + Montgomery County trigger).
    pub tenant_requested_installation: bool,
    /// Whether window guards are installed in every
    /// non-fire-escape window of the apartment.
    pub window_guards_installed: bool,
    /// Whether 30-day lease notice was provided (NYC
    /// requirement).
    pub thirty_day_lease_notice_provided: bool,
    /// Whether annual notice between January 1 and
    /// January 16 was provided (NYC requirement).
    pub annual_notice_provided_jan_1_to_16: bool,
    /// Whether annual notice was provided (MA generic
    /// requirement).
    pub ma_annual_notice_provided: bool,
    /// Whether Chicago window opening limited to 4 inches
    /// or less.
    pub chicago_window_opening_4_inch_or_less: bool,
    /// Whether window is greater than 6 feet above grade
    /// (MA applicable-window threshold).
    pub window_above_6_feet_grade: bool,
    /// Whether window opens sufficiently for 5-inch
    /// diameter ball to pass (MA applicable-window
    /// threshold).
    pub window_opens_for_5_inch_ball: bool,
    /// Whether window connects to fire escape (carveout).
    pub window_connects_to_fire_escape: bool,
    /// Number of days violation has continued (NYC daily
    /// penalty multiplier).
    pub days_violation_continues: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalWindowGuardInstallationResult {
    pub jurisdiction: Jurisdiction,
    pub installation_obligation_triggered: bool,
    pub installation_compliant: bool,
    pub notice_compliant: bool,
    pub nyc_daily_penalty_cents: u64,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalWindowGuardInstallationInput) -> RentalWindowGuardInstallationResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let (
        installation_obligation_triggered,
        installation_compliant,
        notice_compliant,
        nyc_daily_penalty_cents,
    ) = match input.jurisdiction {
        Jurisdiction::NewYorkCity => {
            let obligation =
                input.dwelling_unit_count >= 3 && input.child_age_10_or_younger_resides;
            let installed = !obligation
                || input.window_guards_installed
                || input.window_connects_to_fire_escape;
            let notices = !obligation
                || (input.thirty_day_lease_notice_provided
                    && input.annual_notice_provided_jan_1_to_16);
            let penalty_cents = if obligation && !installed {
                (input.days_violation_continues as u64).saturating_mul(100_000)
            } else {
                0
            };

            if obligation && !installed {
                failure_reasons.push(
                        "NYC Health Code Article 131 § 131.15 + NYC Admin Code § 27-2043.1 — landlord of multiple dwelling with 3+ apartments MUST install approved window guards on every window in apartment where child age 10 or younger resides (carveout: windows leading to fire escapes); NYC Health Code § 3.11 civil penalties up to $1,000 per violation per day".to_string(),
                    );
            }
            if obligation && !input.thirty_day_lease_notice_provided {
                failure_reasons.push(
                        "NYC Health Code Article 131 § 131.15 — landlord must provide WINDOW GUARD NOTICE FORM within first 30 DAYS of occupancy".to_string(),
                    );
            }
            if obligation && !input.annual_notice_provided_jan_1_to_16 {
                failure_reasons.push(
                        "NYC Health Code Article 131 § 131.15 — building owner must send ANNUAL NOTICE regarding window guards between JANUARY 1 AND JANUARY 16 each year".to_string(),
                    );
            }

            (obligation, installed, notices, penalty_cents)
        }
        Jurisdiction::Chicago => {
            let obligation = true;
            let installed = input.chicago_window_opening_4_inch_or_less;
            let notices = true;
            if !installed {
                failure_reasons.push(
                        "Chicago Building Code § 13-196-550 — operable window guards required limiting window opening to 4 INCHES OR LESS in dwelling units; screens required April 15 through November 15 each year".to_string(),
                    );
            }
            (obligation, installed, notices, 0)
        }
        Jurisdiction::Massachusetts => {
            let applicable_window = input.window_above_6_feet_grade
                && input.window_opens_for_5_inch_ball
                && !input.window_connects_to_fire_escape;
            let obligation = applicable_window
                && input.child_age_10_or_younger_resides
                && input.tenant_requested_installation;
            let installed = !obligation || input.window_guards_installed;
            let notices = !applicable_window || input.ma_annual_notice_provided;
            if obligation && !installed {
                failure_reasons.push(
                        "Mass. G.L. + 105 CMR 410 (State Sanitary Code) — landlord MUST install window guards at TENANT'S REQUEST when child under age 10 resides; applies to 'applicable windows' (> 6 feet above grade + opens for 5-inch diameter ball + not fire escape)".to_string(),
                    );
            }
            if applicable_window && !input.ma_annual_notice_provided {
                failure_reasons.push(
                        "Mass. G.L. + 105 CMR 410 — landlord must provide annual notice stating 'Parents with children under the age of 10 have the right, at no additional charge, to have window guards installed within the rented apartment and the common areas of the building'".to_string(),
                    );
            }
            (obligation, installed, notices, 0)
        }
        Jurisdiction::MontgomeryCountyMd => {
            let obligation =
                input.child_age_6_or_younger_resides && input.tenant_requested_installation;
            let installed = !obligation || input.window_guards_installed;
            let notices = true;
            if obligation && !installed {
                failure_reasons.push(
                        "Montgomery County MD DHCA Code § 29-23 — landlord must install window guards at TENANT'S REQUEST when child under age 6 resides; lease addendum disclosure required at lease execution".to_string(),
                    );
            }
            (obligation, installed, notices, 0)
        }
        Jurisdiction::Default => {
            let obligation = false;
            let installed = true;
            let notices = true;
            (obligation, installed, notices, 0)
        }
    };

    let notes: Vec<String> = vec![
        "NYC Health Code Article 131 § 131.15 + NYC Admin Code § 27-2043.1 — landlord of multiple dwelling with 3 OR MORE APARTMENTS must install approved window guards on every window in apartment where child age 10 OR YOUNGER resides (carveout: windows leading to fire escapes); applies to public hallway windows in such buildings".to_string(),
        "NYC Health Code Article 131 § 131.15 notice obligations — (1) 30-DAY LEASE NOTICE within first 30 days of occupancy; (2) ANNUAL NOTICE between JANUARY 1 AND JANUARY 16 each year".to_string(),
        "NYC enforcement — Department of Health and Mental Hygiene Class C violation; civil penalties UP TO $1,000 PER VIOLATION PER DAY under NYC Health Code § 3.11; NYC Admin Code § 27-2115 ECB penalties for housing maintenance code violations".to_string(),
        "Chicago Building Code § 13-196-550 — operable window guards required limiting window opening to 4 INCHES OR LESS in dwelling units; screens required April 15 through November 15 each year; burglar bars require landlord written consent before tenant installation".to_string(),
        "Mass. G.L. + 105 CMR 410 (State Sanitary Code) — landlord must install window guards at TENANT'S REQUEST when child under age 10 resides; applies to 'applicable windows' meeting THREE PRONGS: (1) > 6 feet above grade AND (2) capable of opening sufficiently for 5-inch diameter ball to pass through AND (3) NOT connected to fire escape".to_string(),
        "Mass. G.L. + 105 CMR 410 annual notice — landlord must provide tenant at beginning of tenancy AND at least annually thereafter: 'Parents with children under the age of 10 have the right, at no additional charge, to have window guards installed within the rented apartment and the common areas of the building'".to_string(),
        "Montgomery County Maryland DHCA Code § 29-23 — window guard lease addendum required for tenants with children under age 6; landlord must install guards at tenant request".to_string(),
        "Default — no statewide window guard mandate; general premises liability + common-law negligence apply; ASTM F2090-23 voluntary standard for window fall prevention devices may inform negligence analysis".to_string(),
        "Trader-landlord critical — child window-fall injuries are among highest-stakes premises liability claims; wrongful death awards routinely exceed $5M; many jurisdictions impose STRICT LIABILITY when window guards are required by statute but absent".to_string(),
        "ASTM F2090-23 — voluntary standard for window fall prevention devices including emergency escape (egress) release mechanisms; compliance reduces negligence exposure but does not satisfy strict-liability statutes".to_string(),
        "Cross-jurisdictional architecture: NYC uses BUILDING-SIZE + CHILD-AGE TRIGGER + LANDLORD-INSTALLED MANDATE; Chicago uses BUILDING-WIDE 4-INCH OPENING MANDATE; Massachusetts uses TENANT-REQUEST TRIGGER + APPLICABLE-WINDOW THREE-PRONG TEST; Montgomery County MD uses TENANT-REQUEST + AGE-6 TRIGGER; Default uses common-law negligence".to_string(),
    ];

    RentalWindowGuardInstallationResult {
        jurisdiction: input.jurisdiction,
        installation_obligation_triggered,
        installation_compliant,
        notice_compliant,
        nyc_daily_penalty_cents,
        failure_reasons,
        citation: "NYC Health Code Article 131 § 131.15; NYC Admin Code § 27-2043.1; NYC Health Code § 3.11; NYC Admin Code § 27-2115; Chicago Building Code § 13-196-550; Mass. G.L. + 105 CMR 410 (State Sanitary Code); Montgomery County MD DHCA Code § 29-23; ASTM F2090-23",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nyc_compliant() -> RentalWindowGuardInstallationInput {
        RentalWindowGuardInstallationInput {
            jurisdiction: Jurisdiction::NewYorkCity,
            dwelling_unit_count: 10,
            child_age_10_or_younger_resides: true,
            child_age_6_or_younger_resides: false,
            tenant_requested_installation: false,
            window_guards_installed: true,
            thirty_day_lease_notice_provided: true,
            annual_notice_provided_jan_1_to_16: true,
            ma_annual_notice_provided: true,
            chicago_window_opening_4_inch_or_less: true,
            window_above_6_feet_grade: true,
            window_opens_for_5_inch_ball: true,
            window_connects_to_fire_escape: false,
            days_violation_continues: 0,
        }
    }

    #[test]
    fn nyc_3_plus_units_child_under_11_obligation_engaged() {
        let r = check(&nyc_compliant());
        assert!(r.installation_obligation_triggered);
        assert!(r.installation_compliant);
        assert!(r.notice_compliant);
    }

    #[test]
    fn nyc_2_unit_building_no_obligation() {
        let mut i = nyc_compliant();
        i.dwelling_unit_count = 2;
        let r = check(&i);
        assert!(!r.installation_obligation_triggered);
    }

    #[test]
    fn nyc_3_unit_building_boundary_engages() {
        let mut i = nyc_compliant();
        i.dwelling_unit_count = 3;
        let r = check(&i);
        assert!(r.installation_obligation_triggered);
    }

    #[test]
    fn nyc_no_child_no_obligation() {
        let mut i = nyc_compliant();
        i.child_age_10_or_younger_resides = false;
        let r = check(&i);
        assert!(!r.installation_obligation_triggered);
    }

    #[test]
    fn nyc_no_window_guards_violation() {
        let mut i = nyc_compliant();
        i.window_guards_installed = false;
        let r = check(&i);
        assert!(!r.installation_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("Article 131 § 131.15")
                && f.contains("§ 27-2043.1")
                && f.contains("$1,000 per violation per day")));
    }

    #[test]
    fn nyc_daily_penalty_accrues_at_1000_per_day() {
        let mut i = nyc_compliant();
        i.window_guards_installed = false;
        i.days_violation_continues = 30;
        let r = check(&i);
        assert_eq!(r.nyc_daily_penalty_cents, 30 * 100_000);
    }

    #[test]
    fn nyc_no_30_day_notice_violation() {
        let mut i = nyc_compliant();
        i.thirty_day_lease_notice_provided = false;
        let r = check(&i);
        assert!(!r.notice_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 131.15") && f.contains("30 DAYS")));
    }

    #[test]
    fn nyc_no_annual_notice_violation() {
        let mut i = nyc_compliant();
        i.annual_notice_provided_jan_1_to_16 = false;
        let r = check(&i);
        assert!(!r.notice_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 131.15")
            && f.contains("ANNUAL NOTICE")
            && f.contains("JANUARY 1 AND JANUARY 16")));
    }

    #[test]
    fn nyc_fire_escape_window_carveout() {
        let mut i = nyc_compliant();
        i.window_guards_installed = false;
        i.window_connects_to_fire_escape = true;
        let r = check(&i);
        assert!(r.installation_compliant);
    }

    #[test]
    fn chicago_4_inch_opening_compliant() {
        let mut i = nyc_compliant();
        i.jurisdiction = Jurisdiction::Chicago;
        i.chicago_window_opening_4_inch_or_less = true;
        let r = check(&i);
        assert!(r.installation_compliant);
    }

    #[test]
    fn chicago_above_4_inch_opening_violation() {
        let mut i = nyc_compliant();
        i.jurisdiction = Jurisdiction::Chicago;
        i.chicago_window_opening_4_inch_or_less = false;
        let r = check(&i);
        assert!(!r.installation_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 13-196-550") && f.contains("4 INCHES OR LESS")));
    }

    #[test]
    fn massachusetts_three_prong_applicable_window_with_child_request() {
        let mut i = nyc_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.window_above_6_feet_grade = true;
        i.window_opens_for_5_inch_ball = true;
        i.window_connects_to_fire_escape = false;
        i.child_age_10_or_younger_resides = true;
        i.tenant_requested_installation = true;
        let r = check(&i);
        assert!(r.installation_obligation_triggered);
    }

    #[test]
    fn massachusetts_no_tenant_request_no_obligation() {
        let mut i = nyc_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.tenant_requested_installation = false;
        let r = check(&i);
        assert!(!r.installation_obligation_triggered);
    }

    #[test]
    fn massachusetts_window_under_6_feet_not_applicable() {
        let mut i = nyc_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.window_above_6_feet_grade = false;
        i.tenant_requested_installation = true;
        let r = check(&i);
        assert!(!r.installation_obligation_triggered);
    }

    #[test]
    fn massachusetts_window_blocks_5_inch_ball_not_applicable() {
        let mut i = nyc_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.window_opens_for_5_inch_ball = false;
        i.tenant_requested_installation = true;
        let r = check(&i);
        assert!(!r.installation_obligation_triggered);
    }

    #[test]
    fn massachusetts_fire_escape_window_not_applicable() {
        let mut i = nyc_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.window_connects_to_fire_escape = true;
        i.tenant_requested_installation = true;
        let r = check(&i);
        assert!(!r.installation_obligation_triggered);
    }

    #[test]
    fn massachusetts_no_annual_notice_violation() {
        let mut i = nyc_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.ma_annual_notice_provided = false;
        let r = check(&i);
        assert!(!r.notice_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("105 CMR 410")
            && f.contains("annual notice")
            && f.contains("no additional charge")));
    }

    #[test]
    fn montgomery_md_child_under_6_with_request() {
        let mut i = nyc_compliant();
        i.jurisdiction = Jurisdiction::MontgomeryCountyMd;
        i.child_age_6_or_younger_resides = true;
        i.tenant_requested_installation = true;
        let r = check(&i);
        assert!(r.installation_obligation_triggered);
    }

    #[test]
    fn montgomery_md_no_child_under_6_no_obligation() {
        let mut i = nyc_compliant();
        i.jurisdiction = Jurisdiction::MontgomeryCountyMd;
        i.child_age_6_or_younger_resides = false;
        i.tenant_requested_installation = true;
        let r = check(&i);
        assert!(!r.installation_obligation_triggered);
    }

    #[test]
    fn montgomery_md_installation_failure() {
        let mut i = nyc_compliant();
        i.jurisdiction = Jurisdiction::MontgomeryCountyMd;
        i.child_age_6_or_younger_resides = true;
        i.tenant_requested_installation = true;
        i.window_guards_installed = false;
        let r = check(&i);
        assert!(!r.installation_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("DHCA Code § 29-23") && f.contains("child under age 6")));
    }

    #[test]
    fn default_no_obligation() {
        let mut i = nyc_compliant();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(!r.installation_obligation_triggered);
    }

    #[test]
    fn jurisdiction_truth_table_five_cells() {
        for jur in [
            Jurisdiction::NewYorkCity,
            Jurisdiction::Chicago,
            Jurisdiction::Massachusetts,
            Jurisdiction::MontgomeryCountyMd,
            Jurisdiction::Default,
        ] {
            let mut i = nyc_compliant();
            i.jurisdiction = jur;
            let r = check(&i);
            assert_eq!(r.jurisdiction, jur);
        }
    }

    #[test]
    fn nyc_uniquely_engages_daily_penalty_invariant() {
        let mut nyc = nyc_compliant();
        nyc.window_guards_installed = false;
        nyc.days_violation_continues = 50;
        let r_nyc = check(&nyc);
        assert!(r_nyc.nyc_daily_penalty_cents > 0);

        for jur in [
            Jurisdiction::Chicago,
            Jurisdiction::Massachusetts,
            Jurisdiction::MontgomeryCountyMd,
            Jurisdiction::Default,
        ] {
            let mut i = nyc_compliant();
            i.jurisdiction = jur;
            i.days_violation_continues = 50;
            let r = check(&i);
            assert_eq!(r.nyc_daily_penalty_cents, 0, "jur={:?}", jur);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&nyc_compliant());
        assert!(r.citation.contains("NYC Health Code Article 131 § 131.15"));
        assert!(r.citation.contains("NYC Admin Code § 27-2043.1"));
        assert!(r.citation.contains("NYC Health Code § 3.11"));
        assert!(r.citation.contains("NYC Admin Code § 27-2115"));
        assert!(r.citation.contains("Chicago Building Code § 13-196-550"));
        assert!(r.citation.contains("Mass. G.L. + 105 CMR 410"));
        assert!(r
            .citation
            .contains("Montgomery County MD DHCA Code § 29-23"));
        assert!(r.citation.contains("ASTM F2090-23"));
    }

    #[test]
    fn note_pins_nyc_three_plus_units_threshold() {
        let r = check(&nyc_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Article 131 § 131.15")
            && n.contains("3 OR MORE APARTMENTS")
            && n.contains("10 OR YOUNGER")
            && n.contains("fire escapes")));
    }

    #[test]
    fn note_pins_nyc_notice_dual_requirements() {
        let r = check(&nyc_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 131.15")
            && n.contains("30-DAY LEASE NOTICE")
            && n.contains("JANUARY 1 AND JANUARY 16")));
    }

    #[test]
    fn note_pins_nyc_daily_penalty_dhmh() {
        let r = check(&nyc_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Department of Health and Mental Hygiene")
                && n.contains("$1,000 PER VIOLATION PER DAY")
                && n.contains("§ 3.11")));
    }

    #[test]
    fn note_pins_chicago_4_inch_opening_seasonal_screens() {
        let r = check(&nyc_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 13-196-550")
            && n.contains("4 INCHES OR LESS")
            && n.contains("April 15 through November 15")));
    }

    #[test]
    fn note_pins_ma_three_prong_test() {
        let r = check(&nyc_compliant());
        assert!(r.notes.iter().any(|n| n.contains("105 CMR 410")
            && n.contains("TENANT'S REQUEST")
            && n.contains("> 6 feet above grade")
            && n.contains("5-inch diameter ball")
            && n.contains("NOT connected to fire escape")));
    }

    #[test]
    fn note_pins_ma_annual_notice_quote() {
        let r = check(&nyc_compliant());
        assert!(r.notes.iter().any(|n| n.contains("105 CMR 410")
            && n.contains("Parents with children under the age of 10")
            && n.contains("no additional charge")
            && n.contains("common areas of the building")));
    }

    #[test]
    fn note_pins_montgomery_md_dhca_29_23() {
        let r = check(&nyc_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("DHCA Code § 29-23") && n.contains("children under age 6")));
    }

    #[test]
    fn note_pins_default_astm_f2090_23() {
        let r = check(&nyc_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Default")
            && n.contains("ASTM F2090-23")
            && n.contains("common-law negligence")));
    }

    #[test]
    fn note_pins_trader_landlord_5m_award() {
        let r = check(&nyc_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-landlord critical")
                && n.contains("$5M")
                && n.contains("STRICT LIABILITY")));
    }

    #[test]
    fn note_pins_astm_voluntary_standard() {
        let r = check(&nyc_compliant());
        assert!(r.notes.iter().any(
            |n| n.contains("ASTM F2090-23") && n.contains("emergency escape (egress) release")
        ));
    }

    #[test]
    fn note_pins_cross_jurisdictional_architecture() {
        let r = check(&nyc_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Cross-jurisdictional architecture")
                && n.contains("BUILDING-SIZE + CHILD-AGE TRIGGER")
                && n.contains("4-INCH OPENING MANDATE")
                && n.contains("APPLICABLE-WINDOW THREE-PRONG TEST")));
    }

    #[test]
    fn multiple_nyc_failures_stack() {
        let mut i = nyc_compliant();
        i.window_guards_installed = false;
        i.thirty_day_lease_notice_provided = false;
        i.annual_notice_provided_jan_1_to_16 = false;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 3);
    }
}
