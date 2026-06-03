//! Tenant-abandoned personal property handling compliance framework.
//!
//! When a tenant vacates and leaves personal property behind, the landlord must follow
//! state-specific procedures to lawfully dispose of, sell, or store the property.
//! Procedures vary sharply by jurisdiction with mandatory notice periods, storage
//! windows, sale procedures, and conversion-liability exposure for premature disposal.
//! Conversion of tenant property is a strict-liability tort independent of any breach
//! of contract.
//!
//! Jurisdictional grid:
//!
//! - CA Civ. Code §§ 1983-1991: written notice + minimum 15-day claim window (personal
//!   delivery) or 18-day claim window (mailed); $700 sale-vs-disposal threshold;
//!   itemized inventory + reasonable storage charges; auction sale if value > $700,
//!   landlord discretion if value ≤ $700.
//! - WA RCW 59.18.310: 45-day storage window from date notice mailed or personally
//!   delivered; landlord may sell or dispose after 45 days; deceased-tenant exception.
//! - TX Prop. Code § 54.045 + § 92.014: landlord lien procedure; 30-day notice before
//!   sale required (both first-class AND certified mail); proceeds applied first to
//!   delinquent rents + lawful packing/moving/storage/sale costs.
//! - FL Stat. ch. 715 (§§ 715.10-715.111): OPTIONAL procedure; minimum 10-day claim
//!   window (personal delivery) or 15-day (mailed); $500 sale-vs-disposal threshold;
//!   if property estimated < $500, landlord may retain or dispose at discretion.
//! - IL: no specific landlord-tenant abandoned-property statute statewide; common-law
//!   "reasonable time" standard + 765 ILCS 1026 Revised Uniform Unclaimed Property Act
//!   applies to bona-fide unclaimed property after escheat period.
//! - MA Gen. L. ch. 239 § 4 + ch. 105A: court-supervised storage procedure required.
//! - CO Rev. Stat. § 38-20-116: 30-day notice required before sale.
//! - DEFAULT: common-law reasonable-time standard; conservative 30-day notice + storage
//!   default.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.justia.com/codes/california/2005/civ/1980-1991.html
//! - app.leg.wa.gov/rcw/default.aspx?cite=59.18.310
//! - codes.findlaw.com/tx/property-code/prop-sect-54-045.html
//! - leg.state.fl.us/Statutes/index.cfm?App_mode=Display_Statute&URL=0700-0799/0715/0715ContentsIndex.html

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    Washington,
    Texas,
    Florida,
    IllinoisCommonLawOnly,
    Massachusetts,
    Colorado,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryMethod {
    PersonalDelivery,
    FirstClassMail,
    CertifiedMail,
    BothFirstClassAndCertifiedRequiredTexas,
    NoNoticeGiven,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisposalAction {
    LandlordRetainsForPersonalUseUnderValueThreshold,
    PublicAuctionSaleWithReasonableNotice,
    DisposedAsTrashAfterStorageWindow,
    PrematureDisposalBeforeNoticeWindow,
    StoredPendingTenantClaim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantStoredPendingTenantClaim,
    CompliantBelowSaleValueThresholdLandlordRetention,
    CompliantPublicAuctionSaleAfterNoticeWindow,
    PrematureDisposalConversionTortLiability,
    NoticeNotGivenStrictLiabilityConversion,
    NoticeDeliveryMethodNonCompliantPerJurisdiction,
    TexasCertifiedAndFirstClassMailDualRequirementViolated,
    CommonLawReasonableTimeUnverified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub delivery_method: DeliveryMethod,
    pub disposal_action: DisposalAction,
    pub days_after_notice_disposed: u32,
    pub estimated_property_value_cents: u64,
    pub tenant_actual_damages_cents: u64,
}

pub type RentalTenantAbandonedPersonalPropertyInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub statutory_notice_window_days: u32,
    pub statutory_value_threshold_cents: u64,
    pub estimated_landlord_exposure_cents: u64,
    pub note: String,
}

pub type RentalTenantAbandonedPersonalPropertyOutput = Output;
pub type RentalTenantAbandonedPersonalPropertyResult = Output;

const CA_PERSONAL_DELIVERY_NOTICE_DAYS: u32 = 15;
const CA_MAILED_NOTICE_DAYS: u32 = 18;
const CA_SALE_VALUE_THRESHOLD_CENTS: u64 = 70_000;
const WA_STORAGE_WINDOW_DAYS: u32 = 45;
const TX_PRE_SALE_NOTICE_DAYS: u32 = 30;
const FL_PERSONAL_DELIVERY_NOTICE_DAYS: u32 = 10;
const FL_MAILED_NOTICE_DAYS: u32 = 15;
const FL_SALE_VALUE_THRESHOLD_CENTS: u64 = 50_000;
const CO_PRE_SALE_NOTICE_DAYS: u32 = 30;
const DEFAULT_NOTICE_DAYS: u32 = 30;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(input.disposal_action, DisposalAction::StoredPendingTenantClaim) {
        return Output {
            severity: Severity::CompliantStoredPendingTenantClaim,
            statutory_notice_window_days: jurisdiction_notice_window(
                input.jurisdiction,
                input.delivery_method,
            ),
            statutory_value_threshold_cents: jurisdiction_value_threshold(input.jurisdiction),
            estimated_landlord_exposure_cents: 0,
            note: "Property currently stored pending tenant claim — compliant. Landlord must \
                   continue to provide reasonable safekeeping during the statutory storage \
                   window. Document the storage location + access controls + condition log to \
                   defend against conversion / damage claims if tenant ultimately reclaims."
                .to_string(),
        };
    }

    if matches!(input.delivery_method, DeliveryMethod::NoNoticeGiven) {
        let exposure = input
            .estimated_property_value_cents
            .saturating_add(input.tenant_actual_damages_cents);
        return Output {
            severity: Severity::NoticeNotGivenStrictLiabilityConversion,
            statutory_notice_window_days: jurisdiction_notice_window(
                input.jurisdiction,
                input.delivery_method,
            ),
            statutory_value_threshold_cents: jurisdiction_value_threshold(input.jurisdiction),
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "STRICT-LIABILITY CONVERSION: landlord disposed of tenant property WITHOUT \
                 giving the statutorily-required notice. Estimated exposure ${} = property \
                 value (${}) + tenant actual damages (${}). Conversion is a strict-liability \
                 tort independent of any breach-of-contract claim; punitive damages available \
                 in many jurisdictions where conversion is willful or in bad faith.",
                exposure / 100,
                input.estimated_property_value_cents / 100,
                input.tenant_actual_damages_cents / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::Texas)
        && !matches!(
            input.delivery_method,
            DeliveryMethod::BothFirstClassAndCertifiedRequiredTexas
        )
    {
        let exposure = input
            .estimated_property_value_cents
            .saturating_add(input.tenant_actual_damages_cents);
        return Output {
            severity: Severity::TexasCertifiedAndFirstClassMailDualRequirementViolated,
            statutory_notice_window_days: TX_PRE_SALE_NOTICE_DAYS,
            statutory_value_threshold_cents: 0,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "Texas Prop. Code § 54.045 dual-mail requirement VIOLATED. Notice must be \
                 sent to tenant by BOTH first-class mail AND certified mail return receipt \
                 requested AT LEAST 30 days before sale. Single-channel notice is non- \
                 compliant. Estimated exposure ${} = property value (${}) + tenant actual \
                 damages (${}) + court costs + attorney fees under § 54.046.",
                exposure / 100,
                input.estimated_property_value_cents / 100,
                input.tenant_actual_damages_cents / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::IllinoisCommonLawOnly) {
        return Output {
            severity: Severity::CommonLawReasonableTimeUnverified,
            statutory_notice_window_days: DEFAULT_NOTICE_DAYS,
            statutory_value_threshold_cents: 0,
            estimated_landlord_exposure_cents: 0,
            note: "Illinois has NO landlord-tenant-specific abandoned-property statute \
                   statewide. Common-law reasonable-time standard applies; courts typically \
                   approve 30-day storage + notice procedure as reasonable, but exposure \
                   depends on facts. 765 ILCS 1026 Revised Uniform Unclaimed Property Act \
                   applies to bona-fide unclaimed property after escheat period (multi-year \
                   timeline). Cook County / Chicago RLTO § 5-12-160 does NOT address \
                   abandoned property directly."
                .to_string(),
        };
    }

    let notice_window = jurisdiction_notice_window(input.jurisdiction, input.delivery_method);
    let value_threshold = jurisdiction_value_threshold(input.jurisdiction);

    if input.days_after_notice_disposed < notice_window {
        let exposure = input
            .estimated_property_value_cents
            .saturating_add(input.tenant_actual_damages_cents);
        return Output {
            severity: Severity::PrematureDisposalConversionTortLiability,
            statutory_notice_window_days: notice_window,
            statutory_value_threshold_cents: value_threshold,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "PREMATURE DISPOSAL: property disposed of {} days after notice; statutory \
                 window requires {} days. Conversion-tort liability triggers. Estimated \
                 exposure ${} = property value (${}) + tenant actual damages (${}). \
                 Conversion is strict-liability tort independent of contract; punitive \
                 damages available where disposal was willful or in bad faith.",
                input.days_after_notice_disposed,
                notice_window,
                exposure / 100,
                input.estimated_property_value_cents / 100,
                input.tenant_actual_damages_cents / 100
            ),
        };
    }

    match input.disposal_action {
        DisposalAction::LandlordRetainsForPersonalUseUnderValueThreshold => {
            if input.estimated_property_value_cents > value_threshold && value_threshold > 0 {
                let exposure = input
                    .estimated_property_value_cents
                    .saturating_add(input.tenant_actual_damages_cents);
                return Output {
                    severity: Severity::PrematureDisposalConversionTortLiability,
                    statutory_notice_window_days: notice_window,
                    statutory_value_threshold_cents: value_threshold,
                    estimated_landlord_exposure_cents: exposure,
                    note: format!(
                        "Landlord retention NOT permitted: estimated property value (${}) \
                         EXCEEDS jurisdictional sale-vs-retention threshold (${}). CA Civ. \
                         Code § 1988 requires public auction sale if property value > $700. \
                         FL Stat. § 715.109 + § 715.107 require public sale if value > $500. \
                         Estimated exposure ${} = property value + tenant actual damages \
                         (${}).",
                        input.estimated_property_value_cents / 100,
                        value_threshold / 100,
                        exposure / 100,
                        input.tenant_actual_damages_cents / 100
                    ),
                };
            }
            Output {
                severity: Severity::CompliantBelowSaleValueThresholdLandlordRetention,
                statutory_notice_window_days: notice_window,
                statutory_value_threshold_cents: value_threshold,
                estimated_landlord_exposure_cents: 0,
                note: format!(
                    "Compliant: landlord retention permitted because estimated property value \
                     (${}) is at or below the jurisdictional sale-vs-retention threshold \
                     (${}). Notice procedure satisfied; storage window elapsed.",
                    input.estimated_property_value_cents / 100,
                    value_threshold / 100
                ),
            }
        }
        DisposalAction::PublicAuctionSaleWithReasonableNotice
        | DisposalAction::DisposedAsTrashAfterStorageWindow => Output {
            severity: Severity::CompliantPublicAuctionSaleAfterNoticeWindow,
            statutory_notice_window_days: notice_window,
            statutory_value_threshold_cents: value_threshold,
            estimated_landlord_exposure_cents: 0,
            note: format!(
                "Compliant: sale or disposal occurred {} days after notice, satisfying the \
                 statutory {}-day notice window. Sale proceeds applied first to landlord's \
                 reasonable storage / sale costs and unpaid rent per jurisdiction-specific \
                 rules; surplus held for tenant + state unclaimed-property escheat per \
                 765 ILCS 1026 / equivalent UUPA jurisdictions.",
                input.days_after_notice_disposed, notice_window
            ),
        },
        DisposalAction::PrematureDisposalBeforeNoticeWindow => {
            let exposure = input
                .estimated_property_value_cents
                .saturating_add(input.tenant_actual_damages_cents);
            Output {
                severity: Severity::PrematureDisposalConversionTortLiability,
                statutory_notice_window_days: notice_window,
                statutory_value_threshold_cents: value_threshold,
                estimated_landlord_exposure_cents: exposure,
                note: format!(
                    "PREMATURE DISPOSAL: explicit early-disposal flag set. Conversion-tort \
                     liability. Estimated exposure ${}.",
                    exposure / 100
                ),
            }
        }
        DisposalAction::StoredPendingTenantClaim => unreachable!(),
    }
}

fn jurisdiction_notice_window(
    jurisdiction: Jurisdiction,
    delivery_method: DeliveryMethod,
) -> u32 {
    match jurisdiction {
        Jurisdiction::California => match delivery_method {
            DeliveryMethod::PersonalDelivery => CA_PERSONAL_DELIVERY_NOTICE_DAYS,
            _ => CA_MAILED_NOTICE_DAYS,
        },
        Jurisdiction::Washington => WA_STORAGE_WINDOW_DAYS,
        Jurisdiction::Texas => TX_PRE_SALE_NOTICE_DAYS,
        Jurisdiction::Florida => match delivery_method {
            DeliveryMethod::PersonalDelivery => FL_PERSONAL_DELIVERY_NOTICE_DAYS,
            _ => FL_MAILED_NOTICE_DAYS,
        },
        Jurisdiction::Colorado => CO_PRE_SALE_NOTICE_DAYS,
        Jurisdiction::Massachusetts => DEFAULT_NOTICE_DAYS,
        Jurisdiction::IllinoisCommonLawOnly | Jurisdiction::Default => DEFAULT_NOTICE_DAYS,
    }
}

fn jurisdiction_value_threshold(jurisdiction: Jurisdiction) -> u64 {
    match jurisdiction {
        Jurisdiction::California => CA_SALE_VALUE_THRESHOLD_CENTS,
        Jurisdiction::Florida => FL_SALE_VALUE_THRESHOLD_CENTS,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ca() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            delivery_method: DeliveryMethod::FirstClassMail,
            disposal_action: DisposalAction::PublicAuctionSaleWithReasonableNotice,
            days_after_notice_disposed: 20,
            estimated_property_value_cents: 100_000,
            tenant_actual_damages_cents: 50_000,
        }
    }

    #[test]
    fn stored_pending_claim_compliant() {
        let mut input = base_ca();
        input.disposal_action = DisposalAction::StoredPendingTenantClaim;
        let output = check(&input);
        assert_eq!(output.severity, Severity::CompliantStoredPendingTenantClaim);
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
    }

    #[test]
    fn no_notice_given_strict_liability_conversion() {
        let mut input = base_ca();
        input.delivery_method = DeliveryMethod::NoNoticeGiven;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoticeNotGivenStrictLiabilityConversion
        );
        // $1,000 property + $500 damages = $1,500
        assert_eq!(output.estimated_landlord_exposure_cents, 150_000);
        assert!(output.note.contains("STRICT-LIABILITY"));
    }

    #[test]
    fn california_mailed_notice_18_day_compliant() {
        let input = base_ca();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantPublicAuctionSaleAfterNoticeWindow
        );
        assert_eq!(output.statutory_notice_window_days, 18);
    }

    #[test]
    fn california_personal_delivery_15_day_compliant() {
        let mut input = base_ca();
        input.delivery_method = DeliveryMethod::PersonalDelivery;
        input.days_after_notice_disposed = 15;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantPublicAuctionSaleAfterNoticeWindow
        );
        assert_eq!(output.statutory_notice_window_days, 15);
    }

    #[test]
    fn california_premature_disposal_before_18_days_conversion() {
        let mut input = base_ca();
        input.days_after_notice_disposed = 17;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::PrematureDisposalConversionTortLiability
        );
        assert_eq!(output.estimated_landlord_exposure_cents, 150_000);
    }

    #[test]
    fn california_high_value_landlord_retention_not_permitted() {
        let mut input = base_ca();
        input.disposal_action =
            DisposalAction::LandlordRetainsForPersonalUseUnderValueThreshold;
        input.estimated_property_value_cents = 150_000;
        let output = check(&input);
        // $1,500 > $700 threshold → retention not permitted
        assert_eq!(
            output.severity,
            Severity::PrematureDisposalConversionTortLiability
        );
        assert!(output.note.contains("§ 1988"));
        assert!(output.note.contains("$700"));
    }

    #[test]
    fn california_low_value_landlord_retention_permitted() {
        let mut input = base_ca();
        input.disposal_action =
            DisposalAction::LandlordRetainsForPersonalUseUnderValueThreshold;
        input.estimated_property_value_cents = 50_000;
        let output = check(&input);
        // $500 ≤ $700 threshold → retention permitted
        assert_eq!(
            output.severity,
            Severity::CompliantBelowSaleValueThresholdLandlordRetention
        );
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
    }

    #[test]
    fn washington_45_day_storage_window_compliant() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Washington;
        input.days_after_notice_disposed = 45;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantPublicAuctionSaleAfterNoticeWindow
        );
        assert_eq!(output.statutory_notice_window_days, 45);
    }

    #[test]
    fn washington_44_days_premature_conversion() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Washington;
        input.days_after_notice_disposed = 44;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::PrematureDisposalConversionTortLiability
        );
    }

    #[test]
    fn texas_dual_mail_requirement_satisfied() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Texas;
        input.delivery_method =
            DeliveryMethod::BothFirstClassAndCertifiedRequiredTexas;
        input.days_after_notice_disposed = 30;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantPublicAuctionSaleAfterNoticeWindow
        );
    }

    #[test]
    fn texas_only_certified_mail_violates_dual_requirement() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Texas;
        input.delivery_method = DeliveryMethod::CertifiedMail;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::TexasCertifiedAndFirstClassMailDualRequirementViolated
        );
        assert!(output.note.contains("§ 54.045"));
        assert!(output.note.contains("§ 54.046"));
    }

    #[test]
    fn florida_personal_delivery_10_day_compliant() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Florida;
        input.delivery_method = DeliveryMethod::PersonalDelivery;
        input.days_after_notice_disposed = 10;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantPublicAuctionSaleAfterNoticeWindow
        );
        assert_eq!(output.statutory_notice_window_days, 10);
    }

    #[test]
    fn florida_mailed_15_day_compliant() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Florida;
        input.days_after_notice_disposed = 15;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantPublicAuctionSaleAfterNoticeWindow
        );
        assert_eq!(output.statutory_notice_window_days, 15);
    }

    #[test]
    fn florida_high_value_landlord_retention_not_permitted() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Florida;
        input.days_after_notice_disposed = 15;
        input.disposal_action =
            DisposalAction::LandlordRetainsForPersonalUseUnderValueThreshold;
        input.estimated_property_value_cents = 80_000;
        let output = check(&input);
        // $800 > $500 FL threshold → retention not permitted
        assert_eq!(
            output.severity,
            Severity::PrematureDisposalConversionTortLiability
        );
        assert!(output.note.contains("$500"));
    }

    #[test]
    fn florida_under_500_landlord_retention_permitted() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Florida;
        input.days_after_notice_disposed = 15;
        input.disposal_action =
            DisposalAction::LandlordRetainsForPersonalUseUnderValueThreshold;
        input.estimated_property_value_cents = 30_000;
        let output = check(&input);
        // $300 ≤ $500 → retention permitted
        assert_eq!(
            output.severity,
            Severity::CompliantBelowSaleValueThresholdLandlordRetention
        );
    }

    #[test]
    fn illinois_common_law_only_returns_unverified() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::IllinoisCommonLawOnly;
        let output = check(&input);
        assert_eq!(output.severity, Severity::CommonLawReasonableTimeUnverified);
        assert!(output.note.contains("765 ILCS 1026"));
    }

    #[test]
    fn explicit_premature_disposal_flag_triggers_conversion() {
        let mut input = base_ca();
        input.disposal_action = DisposalAction::PrematureDisposalBeforeNoticeWindow;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::PrematureDisposalConversionTortLiability
        );
    }

    #[test]
    fn ca_personal_delivery_notice_constant_pins_15_days() {
        assert_eq!(CA_PERSONAL_DELIVERY_NOTICE_DAYS, 15);
    }

    #[test]
    fn ca_mailed_notice_constant_pins_18_days() {
        assert_eq!(CA_MAILED_NOTICE_DAYS, 18);
    }

    #[test]
    fn ca_sale_value_threshold_constant_pins_700() {
        assert_eq!(CA_SALE_VALUE_THRESHOLD_CENTS, 70_000);
    }

    #[test]
    fn wa_storage_window_constant_pins_45_days() {
        assert_eq!(WA_STORAGE_WINDOW_DAYS, 45);
    }

    #[test]
    fn tx_pre_sale_notice_constant_pins_30_days() {
        assert_eq!(TX_PRE_SALE_NOTICE_DAYS, 30);
    }

    #[test]
    fn fl_personal_delivery_notice_constant_pins_10_days() {
        assert_eq!(FL_PERSONAL_DELIVERY_NOTICE_DAYS, 10);
    }

    #[test]
    fn fl_mailed_notice_constant_pins_15_days() {
        assert_eq!(FL_MAILED_NOTICE_DAYS, 15);
    }

    #[test]
    fn fl_sale_value_threshold_constant_pins_500() {
        assert_eq!(FL_SALE_VALUE_THRESHOLD_CENTS, 50_000);
    }

    #[test]
    fn co_pre_sale_notice_constant_pins_30_days() {
        assert_eq!(CO_PRE_SALE_NOTICE_DAYS, 30);
    }

    #[test]
    fn default_notice_days_constant_pins_30_days() {
        assert_eq!(DEFAULT_NOTICE_DAYS, 30);
    }

    #[test]
    fn very_large_property_value_no_overflow() {
        let mut input = base_ca();
        input.delivery_method = DeliveryMethod::NoNoticeGiven;
        input.estimated_property_value_cents = u64::MAX;
        let output = check(&input);
        assert_eq!(output.estimated_landlord_exposure_cents, u64::MAX);
    }

    #[test]
    fn zero_value_no_panic() {
        let mut input = base_ca();
        input.delivery_method = DeliveryMethod::NoNoticeGiven;
        input.estimated_property_value_cents = 0;
        input.tenant_actual_damages_cents = 0;
        let output = check(&input);
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
    }
}
