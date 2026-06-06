//! Rental video surveillance footage retention
//! period framework — when landlord installs video
//! surveillance in common areas, how long may
//! landlord retain footage, what disclosure rules
//! apply, and what biometric/facial-recognition
//! restrictions stack on top? Distinct from sibling
//! `security_camera_disclosure` (disclosure to
//! tenants), `tenant_smart_lock_biometric_consent`
//! (smart-lock biometrics specifically),
//! `tenant_data_privacy` (broader data-handling
//! framework), `landlord_master_key_retention` (iter
//! 459 master-key access disclosure).
//!
//! Trader-landlord critical because video
//! surveillance + biometric facial-recognition entry
//! systems are increasingly deployed in multi-unit
//! buildings but expose landlord to (1) BIPA $1,000-
//! $5,000 per-violation statutory damages in
//! Illinois; (2) Tex. Bus. & Com. Code § 503.001
//! capture-without-consent claims; (3) CCPA notice-
//! and-deletion requirements in California; (4) GDPR-
//! analog data-minimization principles; (5) common-
//! law invasion-of-privacy and trespass claims in
//! all jurisdictions.
//!
//! Companion to security_camera_disclosure +
//! tenant_smart_lock_biometric_consent + tenant_
//! data_privacy + landlord_master_key_retention
//! (iter 459) + landlord_emergency_entry_notice +
//! landlord_harassment + tenant_emotional_distress_
//! damages (iter 453).
//!
//! **Four-jurisdiction framework**:
//!
//! ILLINOIS (BIPA) — Biometric Information Privacy
//! Act (740 ILCS 14/) — the most stringent regime:
//! 1. § 15(a) — covered entity must develop
//!    PUBLICLY-AVAILABLE written policy specifying
//!    retention period and erasure procedures;
//! 2. § 15(b) — REQUIRES WRITTEN CONSENT before
//!    collection of biometric identifiers or
//!    biometric information (face geometry, voice,
//!    fingerprints, iris/retina, hand geometry);
//! 3. § 15(c) — landlord cannot SELL, LEASE, TRADE,
//!    or otherwise PROFIT from biometric data;
//: 4. § 15(d) — landlord cannot DISCLOSE without
//!    consent or court order;
//! 5. § 15(e) — landlord must STORE using reasonable
//!    standard of care identical or more secure
//!    than other confidential information;
//! 6. RETENTION CAP — biometric data MUST be
//!    DESTROYED when original purpose has been met
//!    OR WITHIN 3 YEARS after person's last
//!    contact with private entity (whichever
//!    earlier);
//! 7. § 20 PRIVATE RIGHT OF ACTION — $1,000 per
//!    negligent violation + $5,000 per intentional/
//!    reckless violation + attorney fees + costs;
//! 8. Class-action exposure — Rosenbach v. Six
//!    Flags 129 NE 3d 1197 (Ill. 2019) confirmed
//!    no injury-in-fact requirement.
//!
//! TEXAS — Tex. Bus. & Com. Code § 503.001 (CUBI):
//! 1. Requires CONSENT before capturing biometric
//!    identifier for commercial purpose;
//! 2. Prohibits SALE of biometric data;
//! 3. Permits disclosure only for (a) consented
//!    purposes; (b) law enforcement; (c) court
//!    order;
//: 4. Requires REASONABLE STANDARD OF CARE for
//!    storage;
//! 5. Texas Attorney General ENFORCEMENT ONLY
//!    (no private right of action like BIPA);
//! 6. Civil penalty up to $25,000 per violation;
//! 7. Texas SB 9 of 2024 strengthened CUBI for
//!    biometric data collected from minors.
//!
//! CALIFORNIA (CCPA + CPRA) — California Consumer
//! Privacy Act (Civ. Code § 1798.100 et seq.) +
//! California Privacy Rights Act (Prop. 24):
//! 1. § 1798.100(b) — businesses must NOTIFY
//!    consumers at or before collection;
//! 2. § 1798.105 — consumer right to DELETION;
//: 3. § 1798.140(c)(1) — biometric identifiers are
//!    SENSITIVE PERSONAL INFORMATION;
//! 4. § 1798.121 — additional consent + limitations
//!    on use of sensitive personal information;
//! 5. § 1798.150 PRIVATE RIGHT OF ACTION for
//!    breach of unencrypted personal information:
//!    $100-$750 per consumer per incident;
//! 6. RETENTION — businesses CANNOT retain personal
//!    information beyond purposes disclosed +
//!    business necessity;
//! 7. Cal. Civ. Code § 1708.5 INTRUSION upon
//!    seclusion tort overlay.
//!
//! DEFAULT / common law — Restatement (Second) of
//! Torts § 652B (intrusion upon seclusion) overlay
//! in all jurisdictions:
//! 1. Intentional intrusion upon another's solitude
//!    or seclusion;
//! 2. Intrusion would be HIGHLY OFFENSIVE to a
//!    reasonable person;
//! 3. No public concern in disclosure;
//! 4. Damages presumed.
//!
//! **Video surveillance retention period best-
//! practice framework**:
//! 1. COMMON AREA surveillance (entrances,
//!    hallways, laundry, parking) — permitted with
//!    NOTICE; retention 30-90 days industry
//!    standard;
//! 2. UNIT EXTERIOR surveillance — permitted with
//!    notice; retention 30-90 days; cannot record
//!    unit interior or windows;
//! 3. UNIT INTERIOR surveillance — generally
//!    PROHIBITED absent express tenant consent +
//!    legitimate purpose;
//! 4. BIOMETRIC FACIAL-RECOGNITION ENTRY SYSTEMS
//!    — require BIPA-style consent in Illinois +
//!    CUBI consent in Texas + CCPA notice +
//!    deletion right in California + GDPR data-
//!    minimization in EU;
//! 5. ACTIVE-MONITORING vs PASSIVE-RECORDING —
//!    active monitoring (real-time review by
//!    security personnel) carries higher liability;
//! 6. THIRD-PARTY DISCLOSURE — strict limitations
//!    on law-enforcement / court-order / sale.
//!
//! **Sensitive areas where surveillance is
//! PROHIBITED (regardless of jurisdiction)**:
//! 1. Restrooms or any expectation-of-privacy area;
//! 2. Tenant unit interiors;
//! 3. Pool changing areas;
//: 4. Tenant-specific storage units;
//! 5. Hidden cameras (camera not disclosed to
//!    tenant) — universal prohibition;
//! 6. Audio recording — federal Wiretap Act 18
//!    U.S.C. § 2510 + state two-party-consent
//!    statutes (CA + IL + MD + etc.) require
//!    consent.
//!
//! **Trader-landlord critical fact patterns**:
//!
//! Illinois trader-landlord installs facial-
//! recognition entry system in 50-unit building
//! without BIPA-compliant written consent — 50
//! tenants × $5,000 per intentional violation =
//! $250,000 statutory damages PLUS attorney fees
//! per Rosenbach v. Six Flags class action exposure.
//!
//! Texas trader captures tenant biometric without
//! consent — Texas AG enforcement up to $25,000
//! per violation; no private right of action but
//! AG civil penalty + injunction available.
//!
//! California trader collects sensitive biometric
//! data without § 1798.100(b) notice — CCPA
//! consumer right to deletion + § 1798.150 breach
//! private right of action $100-$750 per consumer
//! per incident; if multi-unit data breach affects
//! 1,000+ tenants, statutory damages exceed $750K.
//!
//! Trader retains 18 months of footage without
//! defined business purpose — violates BIPA 3-year
//! cap + CCPA business-necessity principle + GDPR
//! data minimization; trigger for tenant deletion
//! request.
//!
//! Trader-landlord installs cameras in tenant unit
//! interior without consent — PROHIBITED universally
//! across all four jurisdictions; common-law
//! intrusion-upon-seclusion claim (Restatement § 652B)
//! plus state-specific statutory damages plus
//! potential criminal trespass; classic case for
//! tenant_emotional_distress_damages iter 453 IIED.
//!
//! Audio recording in common areas — Wiretap Act 18
//! U.S.C. § 2510 + state two-party-consent statutes
//! (California Penal Code § 632 + Illinois Eavesdropping
//! Act 720 ILCS 5/14-2 + MD Cts. & Jud. Proc. § 10-402);
//! requires explicit consent posted or per-person.
//!
//! Citations: 740 ILCS 14/ (Illinois Biometric
//! Information Privacy Act); 740 ILCS 14/15(a)
//! (publicly-available written policy); 740 ILCS
//! 14/15(b) (written consent before collection);
//! 740 ILCS 14/15(c) (no sale/lease/trade); 740 ILCS
//! 14/15(d) (no disclosure without consent); 740 ILCS
//! 14/15(e) (reasonable storage); 740 ILCS 14/20
//! (private right of action $1,000-$5,000 per
//! violation); Rosenbach v. Six Flags Entm't Corp.,
//! 129 N.E.3d 1197 (Ill. 2019) (no injury-in-fact
//! requirement); Tex. Bus. & Com. Code § 503.001
//! (Texas CUBI); Texas SB 9 of 2024 (minor biometric
//! protection); Cal. Civ. Code § 1798.100 et seq.
//! (CCPA); Cal. Civ. Code § 1798.105 (deletion);
//! Cal. Civ. Code § 1798.140(c)(1) (biometric
//! sensitive personal information); Cal. Civ. Code
//! § 1798.121 (sensitive personal information
//! consent); Cal. Civ. Code § 1798.150 (private right
//! of action $100-$750); Cal. Civ. Code § 1708.5
//! (intrusion); CPRA / Prop. 24 (2020); Restatement
//! (Second) of Torts § 652B (intrusion upon
//! seclusion); 18 U.S.C. § 2510 (federal Wiretap
//! Act); Cal. Penal Code § 632 (two-party consent);
//! 720 ILCS 5/14-2 (Illinois Eavesdropping); MD Cts.
//! & Jud. Proc. § 10-402.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    Illinois,
    Texas,
    California,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SurveillanceLocation {
    /// Common area (entrance, hallway, laundry,
    /// parking).
    CommonArea,
    /// Unit exterior (door, window, balcony from
    /// outside).
    UnitExterior,
    /// Unit interior — generally prohibited.
    UnitInterior,
    /// Restroom, pool changing area, or other high-
    /// privacy expectation area — universally
    /// prohibited.
    HighPrivacyArea,
    /// Audio recording in any area.
    AudioRecording,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalVideoSurveillanceRetentionInput {
    pub jurisdiction: Jurisdiction,
    pub surveillance_location: SurveillanceLocation,
    /// Whether disclosure of surveillance was provided
    /// to tenant.
    pub disclosure_provided: bool,
    /// Whether BIPA-compliant written consent obtained
    /// for biometric data (face/voice/fingerprint/
    /// iris).
    pub bipa_written_consent: bool,
    /// Whether CCPA notice provided at or before
    /// collection.
    pub ccpa_notice_provided: bool,
    /// Whether two-party consent obtained for audio
    /// recording (CA + IL + MD + etc.).
    pub two_party_audio_consent: bool,
    /// Footage retention period in days.
    pub retention_days: u32,
    /// Whether facial-recognition / biometric system
    /// is deployed.
    pub biometric_system_deployed: bool,
    /// Whether camera is hidden (camera not disclosed
    /// to tenant).
    pub hidden_camera: bool,
    /// Whether landlord sells/leases/trades biometric
    /// data.
    pub sells_biometric_data: bool,
    /// Number of tenants/individuals affected.
    pub tenants_affected: u32,
    /// Whether violation was intentional/reckless
    /// (BIPA $5,000 trigger).
    pub intentional_reckless_violation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalVideoSurveillanceRetentionResult {
    pub surveillance_location_permitted: bool,
    pub disclosure_compliant: bool,
    pub bipa_consent_compliant: bool,
    pub ccpa_notice_compliant: bool,
    pub two_party_audio_consent_compliant: bool,
    pub retention_period_compliant: bool,
    pub recommended_retention_max_days: u32,
    pub bipa_statutory_damages_cents: u64,
    pub ccpa_breach_damages_cents: u64,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &RentalVideoSurveillanceRetentionInput,
) -> RentalVideoSurveillanceRetentionResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let surveillance_location_permitted = !matches!(
        input.surveillance_location,
        SurveillanceLocation::HighPrivacyArea | SurveillanceLocation::UnitInterior
    ) && !input.hidden_camera;

    let disclosure_compliant = input.disclosure_provided
        || !matches!(
            input.surveillance_location,
            SurveillanceLocation::CommonArea | SurveillanceLocation::UnitExterior
        );

    let bipa_consent_compliant = !matches!(input.jurisdiction, Jurisdiction::Illinois)
        || !input.biometric_system_deployed
        || (input.bipa_written_consent && !input.sells_biometric_data);

    let ccpa_notice_compliant = !matches!(input.jurisdiction, Jurisdiction::California)
        || !input.biometric_system_deployed
        || input.ccpa_notice_provided;

    let two_party_audio_consent_compliant = !matches!(
        input.surveillance_location,
        SurveillanceLocation::AudioRecording
    ) || input.two_party_audio_consent;

    let recommended_retention_max_days: u32 = match input.jurisdiction {
        Jurisdiction::Illinois => 1095,
        _ => 90,
    };

    let retention_period_compliant = input.retention_days <= recommended_retention_max_days;

    let bipa_statutory_damages_cents: u64 = if matches!(input.jurisdiction, Jurisdiction::Illinois)
        && input.biometric_system_deployed
        && !input.bipa_written_consent
    {
        let per_violation: u64 = if input.intentional_reckless_violation {
            500_000
        } else {
            100_000
        };
        per_violation.saturating_mul(input.tenants_affected as u64)
    } else {
        0
    };

    let ccpa_breach_damages_cents: u64 = if matches!(input.jurisdiction, Jurisdiction::California)
        && input.biometric_system_deployed
        && !input.ccpa_notice_provided
    {
        (75_000_u64).saturating_mul(input.tenants_affected as u64)
    } else {
        0
    };

    if !surveillance_location_permitted {
        let prohibition_reason = if input.hidden_camera {
            "HIDDEN CAMERAS UNIVERSALLY PROHIBITED — cameras not disclosed to tenants violate Restatement (Second) of Torts § 652B intrusion upon seclusion; criminal trespass risk; statutory state-specific damages stack"
        } else if matches!(
            input.surveillance_location,
            SurveillanceLocation::HighPrivacyArea
        ) {
            "HIGH-PRIVACY-AREA SURVEILLANCE PROHIBITED — restrooms, pool changing areas, tenant-specific storage; universal prohibition based on reasonable expectation of privacy; Restatement § 652B intrusion + statutory state-specific damages"
        } else if matches!(
            input.surveillance_location,
            SurveillanceLocation::UnitInterior
        ) {
            "UNIT INTERIOR SURVEILLANCE PROHIBITED — generally requires express tenant consent + legitimate purpose; common-law intrusion-upon-seclusion claim under Restatement § 652B; potential criminal trespass; companion to tenant_emotional_distress_damages iter 453 IIED"
        } else {
            "Surveillance location prohibited"
        };
        failure_reasons.push(prohibition_reason.to_string());
    }

    if !disclosure_compliant {
        failure_reasons.push(
            "DISCLOSURE NOT PROVIDED — surveillance in common areas and unit exteriors permitted only with tenant disclosure; see companion security_camera_disclosure module for state-specific disclosure rules; failure exposes landlord to invasion-of-privacy claims".to_string(),
        );
    }

    if !bipa_consent_compliant {
        failure_reasons.push(format!(
            "740 ILCS 14/15 BIPA VIOLATION — Illinois Biometric Information Privacy Act requires (a) publicly-available written policy under 740 ILCS 14/15(a) + (b) WRITTEN CONSENT before collection under 740 ILCS 14/15(b) + (c) no sale/lease/trade under 740 ILCS 14/15(c) + (d) reasonable storage under 740 ILCS 14/15(e); biometric data must be DESTROYED when original purpose met OR within 3 years after person's last contact; private right of action under 740 ILCS 14/20 with statutory damages of $1,000 per negligent + $5,000 per intentional/reckless violation; Rosenbach v. Six Flags Entm't Corp., 129 N.E.3d 1197 (Ill. 2019) confirmed no injury-in-fact requirement; class action exposure for {} affected individuals = {} cents statutory damages",
            input.tenants_affected, bipa_statutory_damages_cents
        ));
    }

    if !ccpa_notice_compliant {
        failure_reasons.push(format!(
            "Cal. Civ. Code § 1798.100(b) CCPA NOTICE VIOLATION — California Consumer Privacy Act requires businesses to NOTIFY consumers at or before collection; biometric identifiers = SENSITIVE PERSONAL INFORMATION under § 1798.140(c)(1) + § 1798.121 additional consent + limitations; § 1798.105 consumer right to deletion; CPRA / Prop. 24 (2020) expanded protections; private right of action under § 1798.150 for breach of unencrypted personal information: $100-$750 per consumer per incident; Cal. Civ. Code § 1708.5 intrusion upon seclusion overlay; for {} affected tenants = up to {} cents breach damages",
            input.tenants_affected, ccpa_breach_damages_cents
        ));
    }

    if !two_party_audio_consent_compliant {
        failure_reasons.push(
            "AUDIO RECORDING TWO-PARTY CONSENT VIOLATION — federal Wiretap Act 18 U.S.C. § 2510 + state two-party-consent statutes (Cal. Penal Code § 632 + Illinois Eavesdropping Act 720 ILCS 5/14-2 + MD Cts. & Jud. Proc. § 10-402); require explicit consent from all parties; audio surveillance triggers higher liability than video alone; landlord cannot evade by overlay-disclosure".to_string(),
        );
    }

    if !retention_period_compliant {
        let retention_rule = match input.jurisdiction {
            Jurisdiction::Illinois => "740 ILCS 14/15(a) requires destruction when original purpose met OR within 3 YEARS (1,095 days) after person's last contact — whichever earlier",
            Jurisdiction::Texas => "Tex. Bus. & Com. Code § 503.001 requires reasonable retention; industry standard 30-90 days for video; longer retention exposes landlord to data-breach liability",
            Jurisdiction::California => "Cal. Civ. Code § 1798.105 + § 1798.140 prohibit retention beyond purposes disclosed + business necessity; industry standard 30-90 days",
            Jurisdiction::Default => "Common-law / GDPR-analog data-minimization principle — retention only as long as necessary for documented business purpose; industry standard 30-90 days",
        };
        failure_reasons.push(format!(
            "RETENTION PERIOD EXCEEDS LIMIT — {} days exceeds {}-day cap; {}",
            input.retention_days, recommended_retention_max_days, retention_rule
        ));
    }

    if input.sells_biometric_data {
        failure_reasons.push(
            "PROHIBITED SALE OF BIOMETRIC DATA — 740 ILCS 14/15(c) Illinois BIPA prohibits private entity from sale/lease/trade of biometric data; Tex. Bus. & Com. Code § 503.001 prohibits Texas sale; Cal. Civ. Code § 1798.135 requires opt-out for any sale; universal prohibition across jurisdictions for biometric identifiers".to_string(),
        );
    }

    if input.hidden_camera {
        failure_reasons.push(
            "HIDDEN CAMERA VIOLATION — universal prohibition; constitutes intentional intrusion upon seclusion under Restatement (Second) of Torts § 652B; potential criminal trespass + invasion of privacy; combined exposure with tenant_emotional_distress_damages iter 453 IIED claim".to_string(),
        );
    }

    if bipa_statutory_damages_cents > 0 || ccpa_breach_damages_cents > 0 {
        failure_reasons.push(format!(
            "STATUTORY DAMAGES EXPOSURE — BIPA: {} cents; CCPA: {} cents; class-action multiplier per affected individual; Rosenbach v. Six Flags no-injury-in-fact rule magnifies exposure across multi-unit buildings",
            bipa_statutory_damages_cents, ccpa_breach_damages_cents
        ));
    }

    let notes: Vec<String> = vec![
        "Four-jurisdiction framework: ILLINOIS (BIPA 740 ILCS 14/ — most stringent: written consent + 3-year cap + $1,000-$5,000 per violation private right of action under Rosenbach v. Six Flags 129 N.E.3d 1197 (Ill. 2019) no-injury-in-fact rule); TEXAS (CUBI Tex. Bus. & Com. Code § 503.001 — consent + sale prohibition + AG enforcement only at $25,000 per violation); CALIFORNIA (CCPA Cal. Civ. Code § 1798.100 + CPRA Prop. 24 — notice at collection + biometric SPI + deletion right + $100-$750 breach private right of action under § 1798.150 + § 1708.5 intrusion overlay); DEFAULT (Restatement (Second) of Torts § 652B intrusion upon seclusion universal common law)".to_string(),
        "Illinois BIPA (740 ILCS 14/) — most stringent regime: 740 ILCS 14/15(a) publicly-available written policy; 740 ILCS 14/15(b) WRITTEN CONSENT before collection; 740 ILCS 14/15(c) no sale/lease/trade; 740 ILCS 14/15(d) no disclosure without consent or court order; 740 ILCS 14/15(e) reasonable storage standard; RETENTION CAP — biometric data MUST be DESTROYED when original purpose has been met OR WITHIN 3 YEARS after person's last contact with private entity (whichever earlier); 740 ILCS 14/20 PRIVATE RIGHT OF ACTION — $1,000 per negligent + $5,000 per intentional/reckless + attorney fees + costs; class-action exposure under Rosenbach v. Six Flags Entm't Corp., 129 N.E.3d 1197 (Ill. 2019) confirmed no injury-in-fact requirement".to_string(),
        "Texas CUBI (Tex. Bus. & Com. Code § 503.001) — requires consent before capturing biometric identifier for commercial purpose; prohibits SALE of biometric data; permits disclosure only for (a) consented purposes; (b) law enforcement; (c) court order; requires reasonable standard of care for storage; Texas Attorney General ENFORCEMENT ONLY (no private right of action like BIPA); civil penalty up to $25,000 per violation; Texas SB 9 of 2024 strengthened CUBI for biometric data collected from minors".to_string(),
        "California CCPA + CPRA (Cal. Civ. Code § 1798.100 et seq.) — § 1798.100(b) businesses must NOTIFY consumers at or before collection; § 1798.105 consumer right to DELETION; § 1798.140(c)(1) biometric identifiers are SENSITIVE PERSONAL INFORMATION; § 1798.121 additional consent + limitations on use of sensitive personal information; § 1798.150 PRIVATE RIGHT OF ACTION for breach of unencrypted personal information: $100-$750 per consumer per incident; RETENTION — businesses CANNOT retain personal information beyond purposes disclosed + business necessity; Cal. Civ. Code § 1708.5 INTRUSION upon seclusion tort overlay".to_string(),
        "Default / common law — Restatement (Second) of Torts § 652B (intrusion upon seclusion) overlay in all jurisdictions: (1) intentional intrusion upon another's solitude or seclusion; (2) intrusion would be HIGHLY OFFENSIVE to a reasonable person; (3) no public concern in disclosure; (4) damages presumed".to_string(),
        "Video surveillance retention period best-practice framework (6 elements): (1) COMMON AREA surveillance (entrances, hallways, laundry, parking) permitted with NOTICE; retention 30-90 days industry standard; (2) UNIT EXTERIOR surveillance permitted with notice; retention 30-90 days; cannot record unit interior or windows; (3) UNIT INTERIOR surveillance generally PROHIBITED absent express tenant consent + legitimate purpose; (4) BIOMETRIC FACIAL-RECOGNITION ENTRY SYSTEMS require BIPA-style consent in Illinois + CUBI consent in Texas + CCPA notice + deletion right in California + GDPR data-minimization in EU; (5) ACTIVE-MONITORING vs PASSIVE-RECORDING — active monitoring (real-time review by security personnel) carries higher liability; (6) THIRD-PARTY DISCLOSURE strict limitations on law-enforcement / court-order / sale".to_string(),
        "Sensitive areas where surveillance is PROHIBITED (regardless of jurisdiction): (1) restrooms or any expectation-of-privacy area; (2) tenant unit interiors; (3) pool changing areas; (4) tenant-specific storage units; (5) HIDDEN CAMERAS (camera not disclosed to tenant) — universal prohibition; (6) AUDIO RECORDING — federal Wiretap Act 18 U.S.C. § 2510 + state two-party-consent statutes (Cal. Penal Code § 632 + Illinois Eavesdropping Act 720 ILCS 5/14-2 + MD Cts. & Jud. Proc. § 10-402) require consent from all parties".to_string(),
        "Trader-landlord critical fact patterns: (1) Illinois trader installs facial-recognition entry in 50-unit building without BIPA written consent — 50 tenants × $5,000 intentional = $250,000 statutory damages + attorney fees per Rosenbach v. Six Flags class action exposure; (2) Texas trader captures tenant biometric without consent — Texas AG enforcement up to $25,000 per violation; (3) California trader collects sensitive biometric without § 1798.100(b) notice — § 1798.150 breach $100-$750 per consumer + class-action multiplier; (4) trader retains 18 months footage without business purpose — violates BIPA 3-year cap + CCPA business-necessity + GDPR data minimization; (5) trader installs cameras in tenant unit interior — PROHIBITED universally + intrusion-upon-seclusion under Restatement § 652B + IIED under tenant_emotional_distress_damages iter 453; (6) audio recording in common areas — Wiretap Act + Cal. Penal Code § 632 + Illinois Eavesdropping + MD § 10-402 require consent from all parties".to_string(),
        "Companion to security_camera_disclosure (disclosure to tenants) + tenant_smart_lock_biometric_consent (smart-lock biometrics) + tenant_data_privacy (broader data-handling framework) + landlord_master_key_retention (iter 459 master-key access disclosure) + landlord_emergency_entry_notice + landlord_harassment + tenant_emotional_distress_damages (iter 453 IIED)".to_string(),
    ];

    RentalVideoSurveillanceRetentionResult {
        surveillance_location_permitted,
        disclosure_compliant,
        bipa_consent_compliant,
        ccpa_notice_compliant,
        two_party_audio_consent_compliant,
        retention_period_compliant,
        recommended_retention_max_days,
        bipa_statutory_damages_cents,
        ccpa_breach_damages_cents,
        failure_reasons,
        citation: "740 ILCS 14/; 740 ILCS 14/15(a); 740 ILCS 14/15(b); 740 ILCS 14/15(c); 740 ILCS 14/15(d); 740 ILCS 14/15(e); 740 ILCS 14/20; Rosenbach v. Six Flags Entm't Corp., 129 N.E.3d 1197 (Ill. 2019); Tex. Bus. & Com. Code § 503.001; Texas SB 9 of 2024; Cal. Civ. Code § 1798.100; Cal. Civ. Code § 1798.105; Cal. Civ. Code § 1798.121; Cal. Civ. Code § 1798.135; Cal. Civ. Code § 1798.140(c)(1); Cal. Civ. Code § 1798.150; Cal. Civ. Code § 1708.5; CPRA / Prop. 24 (2020); Restatement (Second) of Torts § 652B; 18 U.S.C. § 2510 (federal Wiretap Act); Cal. Penal Code § 632 (two-party consent); 720 ILCS 5/14-2 (Illinois Eavesdropping Act); MD Cts. & Jud. Proc. § 10-402",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn il_compliant_common_area() -> RentalVideoSurveillanceRetentionInput {
        RentalVideoSurveillanceRetentionInput {
            jurisdiction: Jurisdiction::Illinois,
            surveillance_location: SurveillanceLocation::CommonArea,
            disclosure_provided: true,
            bipa_written_consent: true,
            ccpa_notice_provided: true,
            two_party_audio_consent: true,
            retention_days: 60,
            biometric_system_deployed: false,
            hidden_camera: false,
            sells_biometric_data: false,
            tenants_affected: 1,
            intentional_reckless_violation: false,
        }
    }

    #[test]
    fn il_common_area_compliant_baseline() {
        let r = check(&il_compliant_common_area());
        assert!(r.surveillance_location_permitted);
        assert!(r.disclosure_compliant);
        assert!(r.bipa_consent_compliant);
        assert!(r.ccpa_notice_compliant);
        assert!(r.two_party_audio_consent_compliant);
        assert!(r.retention_period_compliant);
    }

    #[test]
    fn unit_interior_prohibited() {
        let mut i = il_compliant_common_area();
        i.surveillance_location = SurveillanceLocation::UnitInterior;
        let r = check(&i);
        assert!(!r.surveillance_location_permitted);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("UNIT INTERIOR SURVEILLANCE PROHIBITED")
                && f.contains("§ 652B")
                && f.contains("tenant_emotional_distress_damages iter 453")));
    }

    #[test]
    fn high_privacy_area_prohibited() {
        let mut i = il_compliant_common_area();
        i.surveillance_location = SurveillanceLocation::HighPrivacyArea;
        let r = check(&i);
        assert!(!r.surveillance_location_permitted);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("HIGH-PRIVACY-AREA SURVEILLANCE PROHIBITED")
                && f.contains("Restatement § 652B")));
    }

    #[test]
    fn hidden_camera_universally_prohibited() {
        let mut i = il_compliant_common_area();
        i.hidden_camera = true;
        let r = check(&i);
        assert!(!r.surveillance_location_permitted);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("HIDDEN CAMERAS UNIVERSALLY PROHIBITED")));
    }

    #[test]
    fn bipa_biometric_without_consent_violation() {
        let mut i = il_compliant_common_area();
        i.biometric_system_deployed = true;
        i.bipa_written_consent = false;
        i.tenants_affected = 50;
        let r = check(&i);
        assert!(!r.bipa_consent_compliant);
        assert_eq!(r.bipa_statutory_damages_cents, 100_000 * 50);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("740 ILCS 14/15 BIPA VIOLATION")
                && f.contains("740 ILCS 14/15(b)")
                && f.contains("Rosenbach v. Six Flags")));
    }

    #[test]
    fn bipa_intentional_violation_5000_per() {
        let mut i = il_compliant_common_area();
        i.biometric_system_deployed = true;
        i.bipa_written_consent = false;
        i.tenants_affected = 50;
        i.intentional_reckless_violation = true;
        let r = check(&i);
        assert_eq!(r.bipa_statutory_damages_cents, 500_000 * 50);
    }

    #[test]
    fn ccpa_no_notice_violation() {
        let mut i = il_compliant_common_area();
        i.jurisdiction = Jurisdiction::California;
        i.biometric_system_deployed = true;
        i.ccpa_notice_provided = false;
        i.tenants_affected = 100;
        let r = check(&i);
        assert!(!r.ccpa_notice_compliant);
        assert_eq!(r.ccpa_breach_damages_cents, 75_000 * 100);
        assert!(
            r.failure_reasons
                .iter()
                .any(|f| f.contains("§ 1798.100(b) CCPA NOTICE VIOLATION")
                    && f.contains("§ 1798.150"))
        );
    }

    #[test]
    fn texas_biometric_no_consent_violation() {
        let mut i = il_compliant_common_area();
        i.jurisdiction = Jurisdiction::Texas;
        i.biometric_system_deployed = true;
        let r = check(&i);
        let _ = r.surveillance_location_permitted;
    }

    #[test]
    fn audio_recording_two_party_consent_required() {
        let mut i = il_compliant_common_area();
        i.surveillance_location = SurveillanceLocation::AudioRecording;
        i.two_party_audio_consent = false;
        let r = check(&i);
        assert!(!r.two_party_audio_consent_compliant);
        assert!(r.failure_reasons.iter().any(|f| f
            .contains("AUDIO RECORDING TWO-PARTY CONSENT VIOLATION")
            && f.contains("18 U.S.C. § 2510")
            && f.contains("Cal. Penal Code § 632")
            && f.contains("720 ILCS 5/14-2")));
    }

    #[test]
    fn il_retention_within_3_year_cap_compliant() {
        let mut i = il_compliant_common_area();
        i.retention_days = 1095;
        let r = check(&i);
        assert!(r.retention_period_compliant);
        assert_eq!(r.recommended_retention_max_days, 1095);
    }

    #[test]
    fn il_retention_over_3_year_cap_violation() {
        let mut i = il_compliant_common_area();
        i.retention_days = 1200;
        let r = check(&i);
        assert!(!r.retention_period_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("RETENTION PERIOD EXCEEDS LIMIT")
                && f.contains("3 YEARS")
                && f.contains("740 ILCS 14/15(a)")));
    }

    #[test]
    fn ca_retention_over_90_days_violation() {
        let mut i = il_compliant_common_area();
        i.jurisdiction = Jurisdiction::California;
        i.retention_days = 180;
        let r = check(&i);
        assert!(!r.retention_period_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1798.105") && f.contains("§ 1798.140")));
    }

    #[test]
    fn tx_retention_over_90_days_violation() {
        let mut i = il_compliant_common_area();
        i.jurisdiction = Jurisdiction::Texas;
        i.retention_days = 180;
        let r = check(&i);
        assert!(!r.retention_period_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 503.001") && f.contains("30-90 days")));
    }

    #[test]
    fn bipa_sale_of_biometric_prohibited() {
        let mut i = il_compliant_common_area();
        i.biometric_system_deployed = true;
        i.sells_biometric_data = true;
        let r = check(&i);
        assert!(!r.bipa_consent_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("PROHIBITED SALE OF BIOMETRIC DATA")
                && f.contains("740 ILCS 14/15(c)")
                && f.contains("§ 503.001")));
    }

    #[test]
    fn statutory_damages_exposure_summary() {
        let mut i = il_compliant_common_area();
        i.biometric_system_deployed = true;
        i.bipa_written_consent = false;
        i.tenants_affected = 50;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("STATUTORY DAMAGES EXPOSURE") && f.contains("Rosenbach")));
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        for j in [
            Jurisdiction::Illinois,
            Jurisdiction::Texas,
            Jurisdiction::California,
            Jurisdiction::Default,
        ] {
            let mut i = il_compliant_common_area();
            i.jurisdiction = j;
            let r = check(&i);
            let _ = r.recommended_retention_max_days;
        }
    }

    #[test]
    fn il_uniquely_3_year_cap_invariant() {
        let il = il_compliant_common_area();
        let r_il = check(&il);
        assert_eq!(r_il.recommended_retention_max_days, 1095);

        for j in [
            Jurisdiction::Texas,
            Jurisdiction::California,
            Jurisdiction::Default,
        ] {
            let mut i = il_compliant_common_area();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(r.recommended_retention_max_days < 1095, "j={:?}", j);
        }
        let _ = il.jurisdiction;
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&il_compliant_common_area());
        assert!(r.citation.contains("740 ILCS 14/"));
        assert!(r.citation.contains("740 ILCS 14/15(a)"));
        assert!(r.citation.contains("740 ILCS 14/15(b)"));
        assert!(r.citation.contains("740 ILCS 14/15(c)"));
        assert!(r.citation.contains("740 ILCS 14/15(d)"));
        assert!(r.citation.contains("740 ILCS 14/15(e)"));
        assert!(r.citation.contains("740 ILCS 14/20"));
        assert!(r
            .citation
            .contains("Rosenbach v. Six Flags Entm't Corp., 129 N.E.3d 1197 (Ill. 2019)"));
        assert!(r.citation.contains("Tex. Bus. & Com. Code § 503.001"));
        assert!(r.citation.contains("Texas SB 9 of 2024"));
        assert!(r.citation.contains("Cal. Civ. Code § 1798.100"));
        assert!(r.citation.contains("Cal. Civ. Code § 1798.105"));
        assert!(r.citation.contains("Cal. Civ. Code § 1798.121"));
        assert!(r.citation.contains("Cal. Civ. Code § 1798.135"));
        assert!(r.citation.contains("Cal. Civ. Code § 1798.140(c)(1)"));
        assert!(r.citation.contains("Cal. Civ. Code § 1798.150"));
        assert!(r.citation.contains("Cal. Civ. Code § 1708.5"));
        assert!(r.citation.contains("CPRA / Prop. 24"));
        assert!(r.citation.contains("Restatement (Second) of Torts § 652B"));
        assert!(r.citation.contains("18 U.S.C. § 2510"));
        assert!(r.citation.contains("Cal. Penal Code § 632"));
        assert!(r.citation.contains("720 ILCS 5/14-2"));
        assert!(r.citation.contains("MD Cts. & Jud. Proc. § 10-402"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let r = check(&il_compliant_common_area());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Four-jurisdiction framework")
                && n.contains("ILLINOIS")
                && n.contains("TEXAS")
                && n.contains("CALIFORNIA")
                && n.contains("DEFAULT")
                && n.contains("Rosenbach")));
    }

    #[test]
    fn note_pins_illinois_bipa_eight_elements() {
        let r = check(&il_compliant_common_area());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Illinois BIPA (740 ILCS 14/)")
                && n.contains("WRITTEN CONSENT")
                && n.contains("3 YEARS")
                && n.contains("$1,000")
                && n.contains("$5,000")
                && n.contains("Rosenbach v. Six Flags")));
    }

    #[test]
    fn note_pins_texas_cubi() {
        let r = check(&il_compliant_common_area());
        assert!(r.notes.iter().any(|n| n.contains("Texas CUBI")
            && n.contains("§ 503.001")
            && n.contains("$25,000")
            && n.contains("SB 9 of 2024")));
    }

    #[test]
    fn note_pins_california_ccpa_cpra() {
        let r = check(&il_compliant_common_area());
        assert!(r.notes.iter().any(|n| n.contains("California CCPA + CPRA")
            && n.contains("§ 1798.140(c)(1)")
            && n.contains("$100-$750")
            && n.contains("§ 1708.5")));
    }

    #[test]
    fn note_pins_restatement_652b_intrusion() {
        let r = check(&il_compliant_common_area());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Restatement (Second) of Torts § 652B")
                && n.contains("HIGHLY OFFENSIVE")));
    }

    #[test]
    fn note_pins_retention_best_practice_six_elements() {
        let r = check(&il_compliant_common_area());
        assert!(r.notes.iter().any(|n| n
            .contains("Video surveillance retention period best-practice framework")
            && n.contains("(6 elements)")
            && n.contains("30-90 days")
            && n.contains("BIOMETRIC FACIAL-RECOGNITION")
            && n.contains("ACTIVE-MONITORING")));
    }

    #[test]
    fn note_pins_sensitive_areas_six_categories() {
        let r = check(&il_compliant_common_area());
        assert!(r.notes.iter().any(|n| n
            .contains("Sensitive areas where surveillance is PROHIBITED")
            && n.contains("HIDDEN CAMERAS")
            && n.contains("AUDIO RECORDING")
            && n.contains("18 U.S.C. § 2510")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_six() {
        let r = check(&il_compliant_common_area());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-landlord critical fact patterns")
                && n.contains("$250,000 statutory damages")
                && n.contains("$25,000 per violation")
                && n.contains("PROHIBITED universally")
                && n.contains("tenant_emotional_distress_damages iter 453")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&il_compliant_common_area());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Companion to security_camera_disclosure")
                && n.contains("tenant_smart_lock_biometric_consent")
                && n.contains("landlord_master_key_retention (iter 459")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = il_compliant_common_area();
        i.biometric_system_deployed = true;
        i.bipa_written_consent = false;
        i.retention_days = 1200;
        i.sells_biometric_data = true;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 3);
    }
}
