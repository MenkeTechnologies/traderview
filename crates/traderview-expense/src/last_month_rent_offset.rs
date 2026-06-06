//! Tenant's right to apply security deposit as last month's
//! rent — when may a tenant unilaterally withhold last month's
//! rent payment on grounds that the security deposit serves as
//! security for unpaid rent?
//!
//! Distinct from sibling modules `security_deposit_caps`
//! (statutory caps on deposit amount), `deposit_return_windows`
//! (refund timelines), `damage_deduction_itemization`
//! (deductions from deposit), and `security_deposit_bank_
//! disclosure` (bank location disclosure). This module focuses
//! on the TENANT-SIDE OFFSET RIGHT — when the tenant may treat
//! the deposit as last month's rent.
//!
//! Texas — Tex. Prop. Code § 92.108: STRICT PROHIBITION.
//! Tenant may NOT withhold last month's rent on grounds that
//! the security deposit serves as security for unpaid rent.
//! Bad-faith violation triggers TREBLE damages (3× the rent
//! wrongfully withheld) plus reasonable attorney's fees in a
//! suit to recover the rent. Strongest tenant penalty in the
//! U.S. Single statutory exception: § 92.056 health/safety
//! repair failure permits tenant to deduct deposit from rent
//! owed.
//!
//! California — Cal. Civ. Code § 1950.5: LABEL-DEPENDENT
//! TREATMENT. Any advance "last month's rent" payment is
//! treated as security under § 1950.5 unless the lease labels
//! it differently. If the lease labels the payment as "last
//! month's rent," tenant is RELIEVED of paying that month's
//! rent (effectively pre-paid). If labeled as "security for
//! last month's rent" or just "security deposit," tenant must
//! still pay last month's rent separately. AB 12 (2024)
//! capped total security deposit at 1 month's rent for most
//! residential rentals — eliminated the traditional "first +
//! last + security" combo for landlords.
//!
//! New York — N.Y. Gen. Oblig. Law § 7-103: TRUST FUND
//! PRINCIPLE. Deposit must be held in trust; not commingled
//! with landlord's funds. Landlord may apply deposit to last
//! month's rent at end of lease, but tenant may NOT
//! unilaterally apply. Companion to § 7-103(2) 6+ unit
//! interest-bearing requirement.
//!
//! Default — common-law separation. Security deposit and rent
//! are separate obligations; tenant cannot unilaterally offset
//! absent express lease provision authorizing it. Most states
//! follow this default.
//!
//! Citations: Tex. Prop. Code § 92.108 (general — strict
//! prohibition); Tex. Prop. Code § 92.108(b) (treble damages
//! plus attorney fees on bad-faith violation); Tex. Prop. Code
//! § 92.056 (health/safety repair exception); Cal. Civ. Code
//! § 1950.5 (security deposit definition + treatment);
//! Cal. Civ. Code § 1950.5(b) (general restrictions on use);
//! Cal. Civ. Code § 1950.5(c) (deposit cap under AB 12
//! effective July 1, 2024); N.Y. Gen. Oblig. Law § 7-103
//! (trust-fund principle); N.Y. Gen. Oblig. Law § 7-103(1)
//! (no commingling); N.Y. Gen. Oblig. Law § 7-103(2) (6+
//! unit interest requirement — cross-reference to security_
//! deposit_bank_disclosure module).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// Tex. Prop. Code § 92.108 — strict prohibition with
    /// treble damages + attorney fees on bad-faith violation.
    Texas,
    /// Cal. Civ. Code § 1950.5 — label-dependent treatment.
    California,
    /// N.Y. Gen. Oblig. Law § 7-103 — trust-fund principle;
    /// landlord may apply at end; tenant may not unilaterally.
    NewYork,
    /// Common-law separation — deposit and rent are separate
    /// obligations.
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// True if tenant attempted to withhold last month's rent
    /// against the security deposit.
    pub tenant_attempted_offset: bool,
    /// Monthly rent (cents).
    pub monthly_rent_cents: i64,
    /// California-specific — true if the lease expressly labels
    /// the advance payment as "last month's rent" (per
    /// § 1950.5 label-dependent treatment). When true, tenant
    /// is relieved of paying last month's rent; when false, the
    /// payment is treated as security and last month's rent is
    /// still owed.
    pub ca_lease_labels_as_last_month_rent: bool,
    /// Texas-specific — true if tenant has properly terminated
    /// lease under § 92.056 (landlord failure to repair health/
    /// safety condition). Permits offset.
    pub texas_repair_exception_engaged: bool,
    /// Texas-specific — true if tenant acted in bad faith
    /// (§ 92.108(b) treble damages presumption engages).
    /// Statute presumes bad faith on any § 92.108 violation.
    pub tenant_bad_faith: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if tenant may lawfully apply deposit to last month's
    /// rent under the applicable regime.
    pub offset_permitted: bool,
    /// True if Texas § 92.108(b) treble-damages presumption
    /// engages (bad-faith violation).
    pub texas_treble_damages_engaged: bool,
    /// Texas-specific treble-damages exposure (3× monthly rent
    /// wrongfully withheld) (cents).
    pub texas_treble_damages_cents: i64,
    /// True if landlord recovers attorney's fees as statutory
    /// remedy.
    pub attorney_fees_recoverable: bool,
    /// California-specific — true if lease label permits offset.
    pub ca_label_permits_relief: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// Texas § 92.108(b) treble multiplier.
pub const TEXAS_TREBLE_MULTIPLIER: i64 = 3;

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let monthly_rent = input.monthly_rent_cents.max(0);

    let offset_permitted;
    let mut texas_treble_damages_engaged = false;
    let mut texas_treble_damages_cents: i64 = 0;
    let mut attorney_fees_recoverable = false;
    let mut ca_label_permits_relief = false;

    match input.regime {
        Regime::Texas => {
            // § 92.056 repair exception — permits offset.
            if input.texas_repair_exception_engaged {
                offset_permitted = true;
                notes.push(
                    "Tex. Prop. Code § 92.056 health/safety repair exception engaged — \
                     tenant has properly terminated lease for landlord's failure to \
                     repair condition affecting health or safety. § 92.108 prohibition \
                     does NOT apply; tenant may deduct deposit from rent owed."
                        .to_string(),
                );
            } else {
                // Strict prohibition — § 92.108.
                offset_permitted = false;
                if input.tenant_attempted_offset {
                    if input.tenant_bad_faith {
                        texas_treble_damages_engaged = true;
                        texas_treble_damages_cents =
                            monthly_rent.saturating_mul(TEXAS_TREBLE_MULTIPLIER);
                        attorney_fees_recoverable = true;
                        violations.push(format!(
                            "Tex. Prop. Code § 92.108 — STRICT PROHIBITION violated. \
                             Tenant withheld last month's rent against security deposit. \
                             § 92.108(b) bad-faith finding engages TREBLE DAMAGES: 3 × {} \
                             cents = {} cents + landlord's reasonable attorney's fees. \
                             Strongest tenant penalty in the U.S. for deposit-offset \
                             violations.",
                            monthly_rent, texas_treble_damages_cents,
                        ));
                    } else {
                        violations.push(
                            "Tex. Prop. Code § 92.108 — prohibition violated; tenant \
                             withheld last month's rent against security deposit. Bad-\
                             faith presumption not established; only actual damages \
                             recoverable. Note: § 92.108 PRESUMES bad faith on any \
                             violation — landlord typically establishes via prima facie \
                             showing."
                                .to_string(),
                        );
                    }
                }
            }
        }
        Regime::California => {
            // § 1950.5 label-dependent treatment.
            if input.ca_lease_labels_as_last_month_rent {
                ca_label_permits_relief = true;
                offset_permitted = true;
                notes.push(
                    "Cal. Civ. Code § 1950.5 — lease expressly labels advance payment as \
                     'LAST MONTH'S RENT.' Tenant is RELIEVED of paying last month's rent \
                     (effectively pre-paid). Label-dependent treatment — if lease had \
                     said 'security for last month's rent' or just 'security deposit,' \
                     tenant would still owe last month's rent separately."
                        .to_string(),
                );
            } else {
                offset_permitted = false;
                if input.tenant_attempted_offset {
                    violations.push(
                        "Cal. Civ. Code § 1950.5 — lease does NOT expressly label payment \
                         as 'last month's rent.' Advance payment treated as SECURITY; \
                         tenant must pay last month's rent separately. Label-dependent \
                         treatment — only when lease clearly designates payment as last \
                         month's rent does tenant get relief."
                            .to_string(),
                    );
                }
            }
            notes.push(
                "Cal. Civ. Code § 1950.5(c) — AB 12 (effective July 1, 2024) capped \
                 security deposit at 1 month's rent for most residential rentals. \
                 Eliminated the traditional 'first + last + security' combo for \
                 landlords — first month + last month + security would exceed cap."
                    .to_string(),
            );
        }
        Regime::NewYork => {
            // GOL § 7-103 — trust-fund principle.
            offset_permitted = false;
            if input.tenant_attempted_offset {
                violations.push(
                    "N.Y. Gen. Oblig. Law § 7-103 — TRUST-FUND PRINCIPLE: deposit held \
                     in trust by landlord for tenant. Landlord may apply deposit to last \
                     month's rent at end of lease, but tenant may NOT unilaterally \
                     withhold last month's rent against the deposit. Companion to \
                     § 7-103(2) 6+ unit interest-bearing requirement (covered in \
                     security_deposit_bank_disclosure module)."
                        .to_string(),
                );
            }
        }
        Regime::Default => {
            // Common-law separation.
            offset_permitted = false;
            if input.tenant_attempted_offset {
                violations.push(
                    "Common-law separation rule — security deposit and rent are SEPARATE \
                     OBLIGATIONS. Tenant cannot unilaterally offset absent express lease \
                     provision authorizing such application. Most states follow this \
                     default rule."
                        .to_string(),
                );
            }
        }
    }

    notes.push(
        "Sibling distinction: this module covers TENANT-SIDE OFFSET RIGHT (may tenant \
         apply deposit as last month's rent?). Related modules: `security_deposit_caps` \
         (statutory caps on deposit amount), `deposit_return_windows` (refund \
         timelines), `damage_deduction_itemization` (deductions from deposit), \
         `security_deposit_bank_disclosure` (bank location disclosure including NY \
         § 7-103(2) 6+ unit interest-bearing requirement). Texas § 92.108 has the \
         strongest tenant penalty (treble damages + attorney fees); California § 1950.5 \
         label-dependence is uniquely permissive when lease drafted correctly; NY \
         § 7-103 trust-fund principle preserves deposit for end-of-lease landlord \
         application."
            .to_string(),
    );

    let compliant = violations.is_empty();

    CheckResult {
        offset_permitted,
        texas_treble_damages_engaged,
        texas_treble_damages_cents,
        attorney_fees_recoverable,
        ca_label_permits_relief,
        compliant,
        violations,
        citation: "Tex. Prop. Code § 92.108 (general — strict prohibition); Tex. Prop. \
                   Code § 92.108(b) (treble damages + attorney fees on bad-faith \
                   violation); Tex. Prop. Code § 92.056 (health/safety repair \
                   exception); Cal. Civ. Code § 1950.5 (security deposit definition + \
                   treatment); Cal. Civ. Code § 1950.5(b) (general restrictions on \
                   use); Cal. Civ. Code § 1950.5(c) (deposit cap under AB 12 effective \
                   July 1, 2024); N.Y. Gen. Oblig. Law § 7-103 (trust-fund principle); \
                   N.Y. Gen. Oblig. Law § 7-103(1) (no commingling); N.Y. Gen. Oblig. \
                   Law § 7-103(2) (6+ unit interest requirement)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(regime: Regime) -> Input {
        Input {
            regime,
            tenant_attempted_offset: false,
            monthly_rent_cents: 200_000, // $2,000
            ca_lease_labels_as_last_month_rent: false,
            texas_repair_exception_engaged: false,
            tenant_bad_faith: false,
        }
    }

    // ── Texas § 92.108 strict prohibition ─────────────────────

    #[test]
    fn texas_attempted_offset_violation() {
        let mut b = input(Regime::Texas);
        b.tenant_attempted_offset = true;
        let r = check(&b);
        assert!(!r.offset_permitted);
        assert!(!r.compliant);
    }

    #[test]
    fn texas_bad_faith_treble_damages_engages() {
        let mut b = input(Regime::Texas);
        b.tenant_attempted_offset = true;
        b.tenant_bad_faith = true;
        let r = check(&b);
        assert!(r.texas_treble_damages_engaged);
        // 3 × $2K = $6K.
        assert_eq!(r.texas_treble_damages_cents, 600_000);
        assert!(r.attorney_fees_recoverable);
    }

    #[test]
    fn texas_no_bad_faith_no_treble() {
        let mut b = input(Regime::Texas);
        b.tenant_attempted_offset = true;
        b.tenant_bad_faith = false;
        let r = check(&b);
        assert!(!r.texas_treble_damages_engaged);
        assert_eq!(r.texas_treble_damages_cents, 0);
        // Still a violation but no treble.
        assert!(!r.compliant);
    }

    #[test]
    fn texas_92_056_repair_exception_permits_offset() {
        let mut b = input(Regime::Texas);
        b.tenant_attempted_offset = true;
        b.texas_repair_exception_engaged = true;
        let r = check(&b);
        assert!(r.offset_permitted);
        // Exception engaged — no violation.
        assert!(r.compliant);
    }

    #[test]
    fn texas_no_attempted_offset_no_violation() {
        let r = check(&input(Regime::Texas));
        assert!(r.compliant);
        assert!(!r.texas_treble_damages_engaged);
    }

    // ── California § 1950.5 label-dependent treatment ─────────

    #[test]
    fn ca_lease_labels_as_last_month_permits_relief() {
        let mut b = input(Regime::California);
        b.ca_lease_labels_as_last_month_rent = true;
        let r = check(&b);
        assert!(r.ca_label_permits_relief);
        assert!(r.offset_permitted);
        assert!(r.compliant);
    }

    #[test]
    fn ca_lease_labels_as_security_no_relief() {
        let mut b = input(Regime::California);
        b.ca_lease_labels_as_last_month_rent = false;
        b.tenant_attempted_offset = true;
        let r = check(&b);
        assert!(!r.ca_label_permits_relief);
        assert!(!r.offset_permitted);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 1950.5")));
    }

    #[test]
    fn ca_ab12_note_present() {
        let r = check(&input(Regime::California));
        assert!(r.notes.iter().any(|n| n.contains("AB 12")));
        assert!(r.notes.iter().any(|n| n.contains("July 1, 2024")));
    }

    // ── New York GOL § 7-103 trust-fund principle ────────────

    #[test]
    fn ny_attempted_offset_violation() {
        let mut b = input(Regime::NewYork);
        b.tenant_attempted_offset = true;
        let r = check(&b);
        assert!(!r.offset_permitted);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("TRUST-FUND")));
    }

    #[test]
    fn ny_no_offset_no_violation() {
        let r = check(&input(Regime::NewYork));
        assert!(r.compliant);
    }

    // ── Default common-law separation ────────────────────────

    #[test]
    fn default_attempted_offset_violation() {
        let mut b = input(Regime::Default);
        b.tenant_attempted_offset = true;
        let r = check(&b);
        assert!(!r.offset_permitted);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Common-law separation")));
    }

    #[test]
    fn default_no_offset_no_violation() {
        let r = check(&input(Regime::Default));
        assert!(r.compliant);
    }

    // ── Multi-regime invariants ──────────────────────────────

    #[test]
    fn only_texas_engages_treble_damages_invariant() {
        for regime in [
            Regime::Texas,
            Regime::California,
            Regime::NewYork,
            Regime::Default,
        ] {
            let mut b = input(regime);
            b.tenant_attempted_offset = true;
            b.tenant_bad_faith = true;
            let r = check(&b);
            let expected = matches!(regime, Regime::Texas);
            assert_eq!(r.texas_treble_damages_engaged, expected, "{:?}", regime);
        }
    }

    #[test]
    fn only_ca_label_dependent_invariant() {
        for regime in [
            Regime::Texas,
            Regime::California,
            Regime::NewYork,
            Regime::Default,
        ] {
            let mut b = input(regime);
            b.ca_lease_labels_as_last_month_rent = true;
            let r = check(&b);
            let expected = matches!(regime, Regime::California);
            assert_eq!(r.ca_label_permits_relief, expected, "{:?}", regime);
        }
    }

    #[test]
    fn texas_treble_math_invariant() {
        // 3× monthly rent across multiple rent levels.
        let cells = [
            (100_000, 300_000),     // $1K → $3K
            (200_000, 600_000),     // $2K → $6K
            (500_000, 1_500_000),   // $5K → $15K
            (1_000_000, 3_000_000), // $10K → $30K
        ];
        for (rent, expected_treble) in cells.iter() {
            let mut b = input(Regime::Texas);
            b.tenant_attempted_offset = true;
            b.tenant_bad_faith = true;
            b.monthly_rent_cents = *rent;
            let r = check(&b);
            assert_eq!(
                r.texas_treble_damages_cents, *expected_treble,
                "rent={}",
                rent
            );
        }
    }

    #[test]
    fn texas_treble_multiplier_invariant() {
        assert_eq!(TEXAS_TREBLE_MULTIPLIER, 3);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&input(Regime::Texas));
        assert!(r.citation.contains("§ 92.108"));
        assert!(r.citation.contains("§ 92.108(b)"));
        assert!(r.citation.contains("§ 92.056"));
        assert!(r.citation.contains("§ 1950.5"));
        assert!(r.citation.contains("§ 1950.5(b)"));
        assert!(r.citation.contains("§ 1950.5(c)"));
        assert!(r.citation.contains("§ 7-103"));
        assert!(r.citation.contains("§ 7-103(1)"));
        assert!(r.citation.contains("§ 7-103(2)"));
        assert!(r.citation.contains("AB 12"));
        assert!(r.citation.contains("July 1, 2024"));
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input(Regime::Texas));
        assert!(
            r.notes.iter().any(|n| n.contains("security_deposit_caps")
                && n.contains("deposit_return_windows")
                && n.contains("damage_deduction_itemization")
                && n.contains("security_deposit_bank_disclosure")
                && n.contains("TENANT-SIDE OFFSET RIGHT")),
            "sibling distinction note must reference related modules + offset-right focus"
        );
    }

    // ── Defensive input clamping ──────────────────────────────

    #[test]
    fn defensive_negative_rent_clamped() {
        let mut b = input(Regime::Texas);
        b.tenant_attempted_offset = true;
        b.tenant_bad_faith = true;
        b.monthly_rent_cents = -100_000;
        let r = check(&b);
        // Negative rent → 0; treble = 0.
        assert_eq!(r.texas_treble_damages_cents, 0);
    }

    #[test]
    fn texas_extreme_rent_no_overflow() {
        let mut b = input(Regime::Texas);
        b.tenant_attempted_offset = true;
        b.tenant_bad_faith = true;
        b.monthly_rent_cents = 100_000_000_000; // $1B rent (absurd but safe)
        let r = check(&b);
        // 3 × $1B = $3B.
        assert_eq!(r.texas_treble_damages_cents, 300_000_000_000);
    }
}
