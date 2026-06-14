//! Stateless risk / sizing / tax / fixed-income calculators.
//!
//! Every endpoint here is pure compute: it takes a JSON body, runs a single
//! function from `traderview-core`, and returns the result. No database
//! access, no auth-scoped data. Useful as building blocks for both the UI
//! sidebars and third-party integrations.

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::routing::post;
use axum::{Json, Router};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use traderview_core::{
    bond_duration, buying_power, carry_score, commission_optimizer, cost_basis, currency_exposure,
    dynamic_kelly, kelly, margin_call, margin_runway, monte_carlo, optimal_f, risk_on_off,
    risk_parity, tax_loss_harvest, var_estimator, vix_term_structure, wash_sale, yield_curve,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // ── Position sizing ───────────────────────────────────────────
        .route("/calc/kelly", post(kelly_route))
        .route("/calc/dynamic-kelly", post(dynamic_kelly_route))
        .route("/calc/optimal-f", post(optimal_f_route))
        // ── Risk / VaR ────────────────────────────────────────────────
        .route("/calc/var-historical", post(var_historical_route))
        .route("/calc/var-gaussian", post(var_gaussian_route))
        .route("/calc/monte-carlo", post(monte_carlo_route))
        .route("/calc/risk-parity", post(risk_parity_route))
        .route("/calc/efficient-frontier", post(efficient_frontier_route))
        .route("/calc/valuation-multiples", post(valuation_multiples_route))
        .route("/calc/dividend-discount-model", post(dividend_discount_model_route))
        .route("/calc/probability-of-profit", post(probability_of_profit_route))
        .route("/calc/straddle", post(straddle_route))
        .route("/calc/strangle", post(strangle_route))
        .route("/calc/collar", post(collar_route))
        .route("/calc/iron-butterfly", post(iron_butterfly_route))
        .route("/calc/butterfly-spread", post(butterfly_spread_route))
        .route("/calc/box-spread", post(box_spread_route))
        .route("/calc/crypto-liquidation", post(crypto_liquidation_route))
        .route("/calc/perp-funding", post(perp_funding_route))
        .route("/calc/span-margin", post(span_margin_route))
        .route("/calc/iv-surface", post(iv_surface_route))
        .route("/calc/merton-default", post(merton_default_route))
        .route("/calc/cape-valuation", post(cape_valuation_route))
        .route("/calc/decumulation-mc", post(decumulation_mc_route))
        .route("/calc/callable-oas", post(callable_oas_route))
        .route("/calc/iv-cone", post(iv_cone_route))
        .route("/calc/gamma-pin-zone", post(gamma_pin_zone_route))
        .route("/calc/calendar-spread", post(calendar_spread_route))
        .route("/calc/risk-on-off", post(risk_on_off_route))
        // ── Margin / buying power ─────────────────────────────────────
        .route("/calc/margin-call", post(margin_call_route))
        .route("/calc/margin-runway", post(margin_runway_route))
        .route("/calc/buying-power", post(buying_power_route))
        // ── Tax / fees ────────────────────────────────────────────────
        .route("/calc/tax-loss-harvest", post(tax_loss_harvest_route))
        .route("/calc/tax-aware-rebalance", post(tax_aware_rebalance_route))
        .route("/calc/savings-waterfall", post(savings_waterfall_route))
        .route("/calc/house-hacking", post(house_hacking_route))
        .route("/calc/brrrr", post(brrrr_route))
        .route("/calc/paycheck-401k", post(paycheck_401k_route))
        .route("/calc/guyton-klinger", post(guyton_klinger_route))
        .route("/calc/irmaa", post(irmaa_route))
        .route("/calc/break-even", post(break_even_route))
        .route("/calc/lease-generator", post(lease_generator_route))
        .route("/calc/invoice-generator", post(invoice_generator_route))
        .route("/calc/landlord-notice", post(landlord_notice_route))
        .route("/calc/security-deposit-itemization", post(security_deposit_itemization_route))
        .route("/calc/promissory-note", post(promissory_note_route))
        .route("/calc/rent-increase-notice", post(rent_increase_notice_route))
        .route("/calc/demand-for-payment", post(demand_for_payment_route))
        .route("/calc/lease-renewal", post(lease_renewal_route))
        .route("/calc/bill-of-sale", post(bill_of_sale_route))
        .route("/calc/rent-receipt", post(rent_receipt_route))
        .route("/calc/contractor-agreement", post(contractor_agreement_route))
        .route("/calc/notice-of-entry", post(notice_of_entry_route))
        .route("/calc/lease-termination", post(lease_termination_route))
        .route("/calc/nda", post(nda_route))
        .route("/calc/pet-addendum", post(pet_addendum_route))
        .route("/calc/inspection-checklist", post(inspection_checklist_route))
        .route("/calc/estimate", post(estimate_route))
        .route("/calc/purchase-order", post(purchase_order_route))
        .route("/calc/sublease", post(sublease_route))
        .route("/calc/roommate-agreement", post(roommate_agreement_route))
        .route("/calc/commercial-lease", post(commercial_lease_route))
        .route("/calc/guaranty", post(guaranty_route))
        .route("/calc/equipment-rental", post(equipment_rental_route))
        .route("/calc/llc-operating-agreement", post(llc_operating_agreement_route))
        .route("/calc/lead-paint-disclosure", post(lead_paint_disclosure_route))
        .route("/calc/offer-letter", post(offer_letter_route))
        .route("/calc/severance", post(severance_route))
        .route("/calc/commission-agreement", post(commission_agreement_route))
        .route("/calc/pto-policy", post(pto_policy_route))
        .route("/calc/expense-reimbursement", post(expense_reimbursement_route))
        .route("/calc/timesheet", post(timesheet_route))
        .route("/calc/pay-stub", post(pay_stub_route))
        .route("/calc/rental-application", post(rental_application_route))
        .route("/calc/cease-desist", post(cease_desist_route))
        .route("/calc/employee-writeup", post(employee_writeup_route))
        .route("/calc/purchase-agreement", post(purchase_agreement_route))
        .route("/calc/closing-statement", post(closing_statement_route))
        .route("/calc/lease-option", post(lease_option_route))
        .route("/calc/land-contract", post(land_contract_route))
        .route("/calc/lease-assignment", post(lease_assignment_route))
        .route("/calc/seller-disclosure", post(seller_disclosure_route))
        .route("/calc/earnest-money-receipt", post(earnest_money_receipt_route))
        .route("/calc/stock-subscription", post(stock_subscription_route))
        .route("/calc/convertible-note", post(convertible_note_route))
        .route("/calc/cap-table", post(cap_table_route))
        .route("/calc/board-resolution", post(board_resolution_route))
        .route("/calc/safe", post(safe_route))
        .route("/calc/option-grant", post(option_grant_route))
        .route("/calc/rsu-grant", post(rsu_grant_route))
        .route("/calc/statement-of-account", post(statement_of_account_route))
        .route("/calc/warrant", post(warrant_route))
        .route("/calc/earnout", post(earnout_route))
        .route("/calc/royalty", post(royalty_route))
        .route("/calc/cam-reconciliation", post(cam_reconciliation_route))
        .route("/calc/percentage-rent", post(percentage_rent_route))
        .route("/calc/cpi-rent-adjustment", post(cpi_rent_adjustment_route))
        .route("/calc/deposit-interest", post(deposit_interest_route))
        .route("/calc/lease-buyout", post(lease_buyout_route))
        .route("/calc/opex-escalation", post(opex_escalation_route))
        .route("/calc/leasing-commission", post(leasing_commission_route))
        .route("/calc/holdover-rent", post(holdover_rent_route))
        .route("/calc/prorated-rent", post(prorated_rent_route))
        .route("/calc/ti-allowance", post(ti_allowance_route))
        .route("/calc/contractor-1099", post(contractor_1099_route))
        .route("/calc/pto-balance", post(pto_balance_route))
        .route("/calc/wage-garnishment", post(wage_garnishment_route))
        .route("/calc/final-paycheck", post(final_paycheck_route))
        .route("/calc/break-premium", post(break_premium_route))
        .route("/calc/reporting-time-pay", post(reporting_time_pay_route))
        .route("/calc/split-shift-premium", post(split_shift_premium_route))
        .route("/calc/workers-comp-premium", post(workers_comp_premium_route))
        .route("/calc/allowance-doubtful", post(allowance_doubtful_route))
        .route("/calc/depreciation-schedule", post(depreciation_schedule_route))
        .route("/calc/asset-disposal", post(asset_disposal_route))
        .route("/calc/cash-flow-statement", post(cash_flow_statement_route))
        .route("/calc/income-statement", post(income_statement_route))
        .route("/calc/bank-reconciliation", post(bank_reconciliation_route))
        .route("/calc/trial-balance", post(trial_balance_route))
        .route("/calc/fix-and-flip", post(fix_and_flip_route))
        .route("/calc/cash-conversion-cycle", post(cash_conversion_cycle_route))
        .route("/calc/profit-first", post(profit_first_route))
        .route("/calc/markup-margin", post(markup_margin_route))
        .route("/calc/inventory-eoq", post(inventory_eoq_route))
        .route("/calc/rent-vs-sell", post(rent_vs_sell_route))
        .route("/calc/depreciation-recapture", post(depreciation_recapture_route))
        .route("/calc/like-kind-exchange", post(like_kind_exchange_route))
        .route("/calc/cost-of-hire", post(cost_of_hire_route))
        .route("/calc/invoice-factoring", post(invoice_factoring_route))
        .route("/calc/ltv-cac", post(ltv_cac_route))
        .route("/calc/burn-rate", post(burn_rate_route))
        .route("/calc/qlac", post(qlac_route))
        .route("/calc/spousal-ira", post(spousal_ira_route))
        .route("/calc/pension-survivor", post(pension_survivor_route))
        .route("/calc/ss-pia", post(ss_pia_route))
        .route("/calc/hsa-triple-tax", post(hsa_triple_tax_route))
        .route("/calc/age-allocation", post(age_allocation_route))
        .route("/calc/roth-bracket-fill", post(roth_bracket_fill_route))
        .route("/calc/mortgage-points", post(mortgage_points_route))
        .route("/calc/apr-apy", post(apr_apy_route))
        .route("/calc/blended-debt", post(blended_debt_route))
        .route("/calc/dividend-coverage", post(dividend_coverage_route))
        .route("/calc/spia", post(spia_route))
        .route("/calc/debt-yield", post(debt_yield_route))
        .route("/calc/price-to-rent", post(price_to_rent_route))
        .route("/calc/years-to-fi", post(years_to_fi_route))
        .route("/calc/grm", post(grm_route))
        .route("/calc/seller-financing", post(seller_financing_route))
        .route("/calc/expense-drag", post(expense_drag_route))
        .route("/calc/lease-payment", post(lease_payment_route))
        .route("/calc/real-return", post(real_return_route))
        .route("/calc/cd-penalty", post(cd_penalty_route))
        .route("/calc/yield-on-cost", post(yield_on_cost_route))
        .route("/calc/trade-expectancy", post(trade_expectancy_route))
        .route("/calc/wage-converter", post(wage_converter_route))
        .route("/calc/sales-tax", post(sales_tax_route))
        .route("/calc/accrued-interest", post(accrued_interest_route))
        .route("/calc/stock-split", post(stock_split_route))
        .route("/calc/tbill-yield", post(tbill_yield_route))
        .route("/calc/dscr", post(dscr_route))
        .route("/calc/graham-number", post(graham_number_route))
        .route("/calc/take-home-paycheck", post(take_home_paycheck_route))
        .route("/calc/ev-ebitda", post(ev_ebitda_route))
        .route("/calc/holding-period-return", post(holding_period_return_route))
        .route("/calc/altman-z-score", post(altman_z_score_route))
        .route("/calc/piotroski-f-score", post(piotroski_f_score_route))
        .route("/calc/gmroi", post(gmroi_route))
        .route("/calc/roth-contribution", post(roth_contribution_route))
        .route("/calc/interest-coverage", post(interest_coverage_route))
        .route("/calc/capital-gains-tax", post(capital_gains_tax_route))
        .route("/calc/traditional-ira-deduction", post(traditional_ira_deduction_route))
        .route("/calc/rule-of-40", post(rule_of_40_route))
        .route("/calc/wacc", post(wacc_route))
        .route("/calc/dupont-roe", post(dupont_roe_route))
        .route("/calc/ss-taxation", post(ss_taxation_route))
        .route("/calc/npv-irr", post(npv_irr_route))
        .route("/calc/leverage", post(leverage_route))
        .route("/calc/two-asset-portfolio", post(two_asset_portfolio_route))
        .route("/calc/mortgage-recast", post(mortgage_recast_route))
        .route("/calc/tax-equivalent-yield", post(tax_equivalent_yield_route))
        .route("/calc/pmi-removal", post(pmi_removal_route))
        .route("/calc/free-cash-flow", post(free_cash_flow_route))
        .route("/calc/credit-card-payoff", post(credit_card_payoff_route))
        .route("/calc/bond-pricing", post(bond_pricing_route))
        .route("/calc/cash-out-refinance", post(cash_out_refinance_route))
        .route("/calc/margin-analysis", post(margin_analysis_route))
        .route("/calc/bonus-grossup", post(bonus_grossup_route))
        .route("/calc/rent-escalation", post(rent_escalation_route))
        .route("/calc/loan-apr", post(loan_apr_route))
        .route("/calc/home-sale-exclusion", post(home_sale_exclusion_route))
        .route("/calc/life-insurance-needs", post(life_insurance_needs_route))
        .route("/calc/car-affordability", post(car_affordability_route))
        .route("/calc/disability-insurance-needs", post(disability_insurance_needs_route))
        .route("/calc/true-hourly-wage", post(true_hourly_wage_route))
        .route("/calc/property-tax", post(property_tax_route))
        .route("/calc/rental-noi", post(rental_noi_route))
        .route("/calc/mortgage-affordability", post(mortgage_affordability_route))
        .route("/calc/overtime-pay", post(overtime_pay_route))
        .route("/calc/solar-payback", post(solar_payback_route))
        .route("/calc/portfolio-longevity", post(portfolio_longevity_route))
        .route("/calc/second-income", post(second_income_route))
        .route("/calc/breakeven-occupancy", post(breakeven_occupancy_route))
        .route("/calc/rent-affordability", post(rent_affordability_route))
        .route("/calc/real-raise", post(real_raise_route))
        .route("/calc/sde-valuation", post(sde_valuation_route))
        .route("/calc/freelance-rate", post(freelance_rate_route))
        .route("/calc/preferred-stock", post(preferred_stock_route))
        .route("/calc/margin-interest", post(margin_interest_route))
        .route("/calc/qbi-deduction", post(qbi_deduction_route))
        .route("/calc/estate-tax", post(estate_tax_route))
        .route("/calc/marriage-penalty", post(marriage_penalty_route))
        .route("/calc/standard-vs-itemized", post(standard_vs_itemized_route))
        .route("/calc/capture-ratio", post(capture_ratio_route))
        .route("/calc/rental-total-return", post(rental_total_return_route))
        .route("/calc/economic-value-added", post(economic_value_added_route))
        .route("/calc/mirr", post(mirr_route))
        .route("/calc/equivalent-annual-cost", post(equivalent_annual_cost_route))
        .route("/calc/multi-product-breakeven", post(multi_product_breakeven_route))
        .route("/calc/wash-sale", post(wash_sale_route))
        .route("/calc/cost-basis", post(cost_basis_route))
        .route("/calc/section-1244", post(section_1244_route))
        .route("/calc/section-1248", post(section_1248_route))
        .route("/calc/section-1252", post(section_1252_route))
        .route("/calc/section-1254", post(section_1254_route))
        .route("/calc/section-1255", post(section_1255_route))
        .route("/calc/section-1245-1250", post(section_1245_1250_route))
        .route("/calc/section-1202", post(section_1202_route))
        .route("/calc/section-1045", post(section_1045_route))
        .route("/calc/section-121", post(section_121_route))
        .route("/calc/section-121d", post(section_121d_route))
        .route("/calc/section-132", post(section_132_route))
        .route("/calc/reps-qualification", post(reps_qualification_route))
        .route("/calc/section-163j", post(section_163j_route))
        .route("/calc/section-165d", post(section_165d_route))
        .route("/calc/section-165g", post(section_165g_route))
        .route("/calc/section-267", post(section_267_route))
        .route("/calc/section-269", post(section_269_route))
        .route("/calc/section-269a", post(section_269a_route))
        .route("/calc/section-274", post(section_274_route))
        .route("/calc/section-279", post(section_279_route))
        .route("/calc/section-988", post(section_988_route))
        .route("/calc/section-1296", post(section_1296_route))
        .route("/calc/section-1341", post(section_1341_route))
        .route("/calc/section-168", post(section_168_route))
        .route("/calc/section-168g", post(section_168g_route))
        .route("/calc/section-168k", post(section_168k_route))
        .route(
            "/calc/section-163j-tradeoff",
            post(section_163j_tradeoff_route),
        )
        .route("/calc/section-164", post(section_164_route))
        .route("/calc/section-165h", post(section_165h_route))
        .route("/calc/section-25c", post(section_25c_route))
        .route("/calc/section-25d", post(section_25d_route))
        .route("/calc/section-25e", post(section_25e_route))
        .route("/calc/section-30c", post(section_30c_route))
        .route("/calc/section-30d", post(section_30d_route))
        .route("/calc/mlp-ubti", post(mlp_ubti_route))
        .route("/calc/section-1258", post(section_1258_route))
        .route("/calc/section-1259", post(section_1259_route))
        .route("/calc/section-1260", post(section_1260_route))
        .route("/calc/section-1361", post(section_1361_route))
        .route("/calc/section-1366", post(section_1366_route))
        .route("/calc/section-1377", post(section_1377_route))
        .route("/calc/section-1367", post(section_1367_route))
        .route("/calc/section-1368", post(section_1368_route))
        .route("/calc/section-1374", post(section_1374_route))
        .route("/calc/section-1375", post(section_1375_route))
        .route("/calc/section-1400z-2", post(section_1400z_2_route))
        .route("/calc/section-1402", post(section_1402_route))
        .route("/calc/section-1411", post(section_1411_route))
        .route("/calc/section-1445", post(section_1445_route))
        .route("/calc/section-1446", post(section_1446_route))
        .route("/calc/section-1471", post(section_1471_route))
        .route("/calc/section-475c2", post(section_475c2_route))
        .route("/calc/section-475f", post(section_475f_route))
        .route("/calc/section-4701", post(section_4701_route))
        .route("/calc/section-213", post(section_213_route))
        .route("/calc/section-170", post(section_170_route))
        .route("/calc/section-219", post(section_219_route))
        .route("/calc/section-221", post(section_221_route))
        .route("/calc/section-223", post(section_223_route))
        .route("/calc/section-243", post(section_243_route))
        .route("/calc/section-245a", post(section_245a_route))
        .route("/calc/section-246", post(section_246_route))
        .route("/calc/section-246a", post(section_246a_route))
        .route("/calc/section-250", post(section_250_route))
        .route("/calc/section-56a", post(section_56a_route))
        .route("/calc/section-59a", post(section_59a_route))
        .route("/calc/section-641", post(section_641_route))
        .route("/calc/section-642", post(section_642_route))
        .route("/calc/section-643", post(section_643_route))
        .route("/calc/section-651", post(section_651_route))
        .route("/calc/section-661", post(section_661_route))
        .route("/calc/section-671", post(section_671_route))
        .route("/calc/section-673", post(section_673_route))
        .route("/calc/section-674", post(section_674_route))
        .route("/calc/section-675", post(section_675_route))
        .route("/calc/section-676", post(section_676_route))
        .route("/calc/section-677", post(section_677_route))
        .route("/calc/section-678", post(section_678_route))
        .route("/calc/section-679", post(section_679_route))
        .route("/calc/section-67g", post(section_67g_route))
        .route("/calc/section-6041", post(section_6041_route))
        .route("/calc/section-6109", post(section_6109_route))
        .route("/calc/section-6042", post(section_6042_route))
        .route("/calc/section-6045", post(section_6045_route))
        .route("/calc/section-6049", post(section_6049_route))
        .route("/calc/section-6050i", post(section_6050i_route))
        .route("/calc/section-6050w", post(section_6050w_route))
        .route("/calc/section-6212", post(section_6212_route))
        .route("/calc/section-6213", post(section_6213_route))
        .route("/calc/section-6201", post(section_6201_route))
        .route("/calc/section-6203", post(section_6203_route))
        .route("/calc/section-6303", post(section_6303_route))
        .route("/calc/section-6304", post(section_6304_route))
        .route("/calc/section-6306", post(section_6306_route))
        .route("/calc/section-6320", post(section_6320_route))
        .route("/calc/section-6321", post(section_6321_route))
        .route("/calc/section-6323", post(section_6323_route))
        .route("/calc/section-6325", post(section_6325_route))
        .route("/calc/section-6330", post(section_6330_route))
        .route("/calc/section-6331", post(section_6331_route))
        .route("/calc/section-6332", post(section_6332_route))
        .route("/calc/section-6334", post(section_6334_route))
        .route("/calc/section-6402", post(section_6402_route))
        .route("/calc/section-6404", post(section_6404_route))
        .route("/calc/section-6411", post(section_6411_route))
        .route("/calc/section-6417", post(section_6417_route))
        .route("/calc/section-6418", post(section_6418_route))
        .route("/calc/section-6425", post(section_6425_route))
        .route("/calc/section-7201", post(section_7201_route))
        .route("/calc/section-7202", post(section_7202_route))
        .route("/calc/section-7203", post(section_7203_route))
        .route("/calc/section-7212", post(section_7212_route))
        .route("/calc/section-7216", post(section_7216_route))
        .route("/calc/section-7206", post(section_7206_route))
        .route("/calc/section-7207", post(section_7207_route))
        .route("/calc/section-7421", post(section_7421_route))
        .route("/calc/section-7422", post(section_7422_route))
        .route("/calc/section-7426", post(section_7426_route))
        .route("/calc/section-7429", post(section_7429_route))
        .route("/calc/section-7430", post(section_7430_route))
        .route("/calc/section-7433", post(section_7433_route))
        .route("/calc/section-7434", post(section_7434_route))
        .route("/calc/section-7463", post(section_7463_route))
        .route("/calc/section-7491", post(section_7491_route))
        .route("/calc/section-162a", post(section_162a_route))
        .route("/calc/section-162e", post(section_162e_route))
        .route("/calc/section-162f", post(section_162f_route))
        .route("/calc/section-162l", post(section_162l_route))
        .route("/calc/section-162m", post(section_162m_route))
        .route("/calc/section-7502", post(section_7502_route))
        .route("/calc/section-7503", post(section_7503_route))
        .route("/calc/section-7508", post(section_7508_route))
        .route("/calc/section-7508a", post(section_7508a_route))
        .route("/calc/section-7521", post(section_7521_route))
        .route("/calc/section-7522", post(section_7522_route))
        .route("/calc/section-7525", post(section_7525_route))
        .route("/calc/section-7811", post(section_7811_route))
        .route("/calc/section-6501", post(section_6501_route))
        .route("/calc/section-6502", post(section_6502_route))
        .route("/calc/section-6531", post(section_6531_route))
        .route("/calc/section-6532", post(section_6532_route))
        .route("/calc/section-6511", post(section_6511_route))
        .route("/calc/section-6601", post(section_6601_route))
        .route("/calc/section-6611", post(section_6611_route))
        .route("/calc/section-6621", post(section_6621_route))
        .route("/calc/section-6651", post(section_6651_route))
        .route("/calc/section-6654", post(section_6654_route))
        .route("/calc/section-6655", post(section_6655_route))
        .route("/calc/section-6662", post(section_6662_route))
        .route("/calc/section-448", post(section_448_route))
        .route("/calc/section-446", post(section_446_route))
        .route("/calc/section-444", post(section_444_route))
        .route("/calc/section-3406", post(section_3406_route))
        .route("/calc/section-302", post(section_302_route))
        .route("/calc/section-304", post(section_304_route))
        .route("/calc/section-305", post(section_305_route))
        .route("/calc/section-311", post(section_311_route))
        .route("/calc/section-312", post(section_312_route))
        .route("/calc/section-318", post(section_318_route))
        .route("/calc/section-331", post(section_331_route))
        .route("/calc/section-332", post(section_332_route))
        .route("/calc/section-1234a", post(section_1234a_route))
        .route("/calc/section-1234b", post(section_1234b_route))
        .route("/calc/section-263g", post(section_263g_route))
        .route("/calc/section-264", post(section_264_route))
        .route("/calc/section-265", post(section_265_route))
        .route("/calc/section-1276", post(section_1276_route))
        .route("/calc/section-1277", post(section_1277_route))
        .route("/calc/section-1278", post(section_1278_route))
        .route("/calc/section-1271", post(section_1271_route))
        .route("/calc/section-1272", post(section_1272_route))
        .route("/calc/section-1273", post(section_1273_route))
        .route("/calc/section-1274", post(section_1274_route))
        .route("/calc/section-1275", post(section_1275_route))
        .route("/calc/section-1281", post(section_1281_route))
        .route("/calc/section-1283", post(section_1283_route))
        .route("/calc/section-1286", post(section_1286_route))
        .route("/calc/section-1287", post(section_1287_route))
        .route("/calc/section-1288", post(section_1288_route))
        .route("/calc/section-1282", post(section_1282_route))
        .route("/calc/section-7704", post(section_7704_route))
        .route("/calc/section-6045b", post(section_6045b_route))
        .route("/calc/section-6045a", post(section_6045a_route))
        .route("/calc/section-1297", post(section_1297_route))
        .route("/calc/section-1298", post(section_1298_route))
        .route("/calc/section-6020", post(section_6020_route))
        .route("/calc/section-6035", post(section_6035_route))
        .route("/calc/section-6038a", post(section_6038a_route))
        .route("/calc/section-6038b", post(section_6038b_route))
        .route("/calc/section-6038c", post(section_6038c_route))
        .route("/calc/section-6038d", post(section_6038d_route))
        .route("/calc/section-6039", post(section_6039_route))
        .route("/calc/section-6011", post(section_6011_route))
        .route("/calc/section-6111", post(section_6111_route))
        .route("/calc/section-6112", post(section_6112_route))
        .route("/calc/section-6662a", post(section_6662a_route))
        .route("/calc/section-6663", post(section_6663_route))
        .route("/calc/section-6664", post(section_6664_route))
        .route("/calc/section-6672", post(section_6672_route))
        .route("/calc/section-6694", post(section_6694_route))
        .route("/calc/section-6695", post(section_6695_route))
        .route("/calc/section-6695a", post(section_6695a_route))
        .route("/calc/section-6700", post(section_6700_route))
        .route("/calc/section-6701", post(section_6701_route))
        .route("/calc/section-6707", post(section_6707_route))
        .route("/calc/section-6707a", post(section_6707a_route))
        .route("/calc/section-6708", post(section_6708_route))
        .route("/calc/section-6713", post(section_6713_route))
        .route("/calc/section-6721", post(section_6721_route))
        .route("/calc/section-6722", post(section_6722_route))
        .route("/calc/section-6723", post(section_6723_route))
        .route("/calc/section-6724", post(section_6724_route))
        .route("/calc/section-6851", post(section_6851_route))
        .route("/calc/section-6861", post(section_6861_route))
        .route("/calc/section-6862", post(section_6862_route))
        .route("/calc/section-6863", post(section_6863_route))
        .route("/calc/section-336", post(section_336_route))
        .route("/calc/section-351", post(section_351_route))
        .route("/calc/section-354", post(section_354_route))
        .route("/calc/section-357", post(section_357_route))
        .route("/calc/section-358", post(section_358_route))
        .route("/calc/section-362", post(section_362_route))
        .route("/calc/section-367", post(section_367_route))
        .route("/calc/section-45l", post(section_45l_route))
        .route("/calc/section-45q", post(section_45q_route))
        .route("/calc/section-45u", post(section_45u_route))
        .route("/calc/section-45v", post(section_45v_route))
        .route("/calc/section-45w", post(section_45w_route))
        .route("/calc/section-45x", post(section_45x_route))
        .route("/calc/section-45y", post(section_45y_route))
        .route("/calc/section-45z", post(section_45z_route))
        .route("/calc/section-47", post(section_47_route))
        .route("/calc/section-48", post(section_48_route))
        .route("/calc/section-48c", post(section_48c_route))
        .route("/calc/section-48e", post(section_48e_route))
        .route("/calc/section-51", post(section_51_route))
        .route("/calc/section-451b", post(section_451b_route))
        .route("/calc/section-451c", post(section_451c_route))
        .route("/calc/section-1031", post(section_1031_route))
        .route("/calc/section-1031-f", post(section_1031_f_route))
        .route("/calc/section-1033", post(section_1033_route))
        .route("/calc/section-481", post(section_481_route))
        .route("/calc/section-482", post(section_482_route))
        .route("/calc/section-514", post(section_514_route))
        .route("/calc/section-530", post(section_530_route))
        .route("/calc/section-280f", post(section_280f_route))
        .route("/calc/section-280b", post(section_280b_route))
        .route("/calc/section-280c", post(section_280c_route))
        .route("/calc/section-280e", post(section_280e_route))
        .route("/calc/section-280g", post(section_280g_route))
        .route("/calc/section-280h", post(section_280h_route))
        .route("/calc/section-163d", post(section_163d_route))
        .route("/calc/section-163h", post(section_163h_route))
        .route("/calc/section-864b2", post(section_864b2_route))
        .route("/calc/section-72t", post(section_72t_route))
        .route("/calc/section-7345", post(section_7345_route))
        .route("/calc/section-7623", post(section_7623_route))
        .route("/calc/section-7405", post(section_7405_route))
        .route("/calc/section-7408", post(section_7408_route))
        .route("/calc/section-7701", post(section_7701_route))
        .route("/calc/section-7872", post(section_7872_route))
        .route("/calc/section-1291", post(section_1291_route))
        .route("/calc/section-1293", post(section_1293_route))
        .route("/calc/section-1294", post(section_1294_route))
        .route("/calc/section-1295", post(section_1295_route))
        .route("/calc/section-1058", post(section_1058_route))
        .route("/calc/section-1092", post(section_1092_route))
        .route("/calc/section-408", post(section_408_route))
        .route("/calc/section-401k", post(section_401k_route))
        .route("/calc/section-415", post(section_415_route))
        .route("/calc/section-408a", post(section_408a_route))
        .route("/calc/section-421", post(section_421_route))
        .route("/calc/section-422", post(section_422_route))
        .route("/calc/section-423", post(section_423_route))
        .route("/calc/section-4501", post(section_4501_route))
        .route("/calc/section-4940", post(section_4940_route))
        .route("/calc/section-4941", post(section_4941_route))
        .route("/calc/section-4942", post(section_4942_route))
        .route("/calc/section-4943", post(section_4943_route))
        .route("/calc/section-4944", post(section_4944_route))
        .route("/calc/section-4945", post(section_4945_route))
        .route("/calc/section-4958", post(section_4958_route))
        .route("/calc/section-4960", post(section_4960_route))
        .route("/calc/section-4972", post(section_4972_route))
        .route("/calc/section-4973", post(section_4973_route))
        .route("/calc/section-4974", post(section_4974_route))
        .route("/calc/section-4975", post(section_4975_route))
        .route("/calc/section-4978", post(section_4978_route))
        .route("/calc/section-6166", post(section_6166_route))
        .route("/calc/section-4980", post(section_4980_route))
        .route("/calc/section-4980h", post(section_4980h_route))
        .route("/calc/section-453", post(section_453_route))
        .route("/calc/section-453a", post(section_453a_route))
        .route("/calc/section-457a", post(section_457a_route))
        .route("/calc/section-457b", post(section_457b_route))
        .route("/calc/section-461g", post(section_461g_route))
        .route("/calc/section-461h", post(section_461h_route))
        .route("/calc/section-461l", post(section_461l_route))
        .route("/calc/section-465", post(section_465_route))
        .route("/calc/section-691", post(section_691_route))
        .route("/calc/section-704d", post(section_704d_route))
        .route("/calc/section-704c", post(section_704c_route))
        .route("/calc/section-706", post(section_706_route))
        .route("/calc/section-707", post(section_707_route))
        .route("/calc/section-721", post(section_721_route))
        .route("/calc/section-723", post(section_723_route))
        .route("/calc/section-731", post(section_731_route))
        .route("/calc/section-732", post(section_732_route))
        .route("/calc/section-734", post(section_734_route))
        .route("/calc/section-736", post(section_736_route))
        .route("/calc/section-737", post(section_737_route))
        .route("/calc/section-741", post(section_741_route))
        .route("/calc/section-743", post(section_743_route))
        .route("/calc/section-751", post(section_751_route))
        .route("/calc/section-752", post(section_752_route))
        .route("/calc/section-1235", post(section_1235_route))
        .route("/calc/section-1239", post(section_1239_route))
        .route("/calc/section-754", post(section_754_route))
        .route("/calc/section-755", post(section_755_route))
        .route("/calc/section-871m", post(section_871m_route))
        .route("/calc/section-901", post(section_901_route))
        .route("/calc/section-903", post(section_903_route))
        .route("/calc/section-904", post(section_904_route))
        .route("/calc/section-911", post(section_911_route))
        .route("/calc/section-951a", post(section_951a_route))
        .route("/calc/section-956", post(section_956_route))
        .route("/calc/section-959", post(section_959_route))
        .route("/calc/section-960", post(section_960_route))
        .route("/calc/section-961", post(section_961_route))
        .route("/calc/section-962", post(section_962_route))
        .route("/calc/section-965", post(section_965_route))
        .route("/calc/section-401a9", post(section_401a9_route))
        .route("/calc/section-409a", post(section_409a_route))
        .route("/calc/section-382", post(section_382_route))
        .route("/calc/section-383", post(section_383_route))
        .route("/calc/section-384", post(section_384_route))
        .route("/calc/section-83i", post(section_83i_route))
        .route("/calc/section-408-d3", post(section_408_d3_route))
        .route("/calc/section-408m", post(section_408m_route))
        .route("/calc/section-41", post(section_41_route))
        .route("/calc/section-38", post(section_38_route))
        .route("/calc/section-42", post(section_42_route))
        .route("/calc/section-44", post(section_44_route))
        .route("/calc/section-408a-d3", post(section_408A_d3_route))
        .route("/calc/section-174", post(section_174_route))
        .route("/calc/section-179", post(section_179_route))
        .route("/calc/section-179d", post(section_179d_route))
        .route("/calc/section-183", post(section_183_route))
        .route("/calc/section-263", post(section_263_route))
        .route("/calc/section-263a", post(section_263a_route))
        .route("/calc/section-168-e6", post(section_168_e6_route))
        .route("/calc/section-108", post(section_108_route))
        .route("/calc/section-104", post(section_104_route))
        .route("/calc/section-1012", post(section_1012_route))
        .route("/calc/section-1014", post(section_1014_route))
        .route("/calc/section-1014e", post(section_1014e_route))
        .route("/calc/section-1015", post(section_1015_route))
        .route("/calc/section-1041", post(section_1041_route))
        .route("/calc/section-1042", post(section_1042_route))
        .route("/calc/section-170e", post(section_170e_route))
        .route("/calc/section-172", post(section_172_route))
        .route("/calc/section-195", post(section_195_route))
        .route("/calc/section-248", post(section_248_route))
        .route("/calc/section-709", post(section_709_route))
        .route("/calc/section-197", post(section_197_route))
        .route("/calc/section-199a", post(section_199a_route))
        .route("/calc/section-83b", post(section_83b_route))
        .route("/calc/section-83c", post(section_83c_route))
        .route("/calc/section-1059", post(section_1059_route))
        .route("/calc/section-1060", post(section_1060_route))
        .route("/calc/section-1091", post(section_1091_route))
        .route("/calc/section-1231", post(section_1231_route))
        .route("/calc/section-1233", post(section_1233_route))
        .route("/calc/section-1234", post(section_1234_route))
        .route(
            "/calc/commission-optimizer",
            post(commission_optimizer_route),
        )
        // ── Fixed income / FX ─────────────────────────────────────────
        .route("/calc/yield-curve", post(yield_curve_route))
        .route("/calc/bond-duration", post(bond_duration_route))
        .route("/calc/carry-score", post(carry_score_route))
        .route("/calc/fx-carry", post(fx_carry_route))
        .route("/calc/currency-exposure", post(currency_exposure_route))
        .route("/calc/vix-term-structure", post(vix_term_structure_route))
}

// ─── Position sizing ────────────────────────────────────────────────────

async fn kelly_route(
    _u: AuthUser,
    Json(input): Json<kelly::KellyInput>,
) -> Json<kelly::KellyOutput> {
    Json(kelly::compute(&input))
}

#[derive(Deserialize)]
struct DynamicKellyBody {
    trade_pnls: Vec<f64>,
    window: usize,
}

async fn dynamic_kelly_route(
    _u: AuthUser,
    Json(b): Json<DynamicKellyBody>,
) -> Result<Json<Vec<dynamic_kelly::DynamicKellyPoint>>, ApiError> {
    if b.window == 0 {
        return Err(ApiError::BadRequest("window must be > 0".into()));
    }
    Ok(Json(dynamic_kelly::compute(&b.trade_pnls, b.window)))
}

#[derive(Deserialize)]
struct OptimalFBody {
    returns: Vec<f64>,
}

async fn optimal_f_route(
    _u: AuthUser,
    Json(b): Json<OptimalFBody>,
) -> Json<optimal_f::OptimalFReport> {
    Json(optimal_f::compute(&b.returns))
}

// ─── VaR ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct VarBody {
    daily_returns: Vec<f64>,
    position_value: f64,
    /// Confidence as a fraction in (0, 1). e.g. 0.95 for 95% VaR.
    confidence: f64,
}

async fn var_historical_route(
    _u: AuthUser,
    Json(b): Json<VarBody>,
) -> Result<Json<var_estimator::VarReport>, ApiError> {
    validate_confidence(b.confidence)?;
    Ok(Json(var_estimator::historical(
        &b.daily_returns,
        b.position_value,
        b.confidence,
    )))
}

async fn var_gaussian_route(
    _u: AuthUser,
    Json(b): Json<VarBody>,
) -> Result<Json<var_estimator::VarReport>, ApiError> {
    validate_confidence(b.confidence)?;
    Ok(Json(var_estimator::parametric_gaussian(
        &b.daily_returns,
        b.position_value,
        b.confidence,
    )))
}

fn validate_confidence(c: f64) -> Result<(), ApiError> {
    if !(c > 0.0 && c < 1.0) {
        return Err(ApiError::BadRequest(
            "confidence must be in (0, 1) exclusive".into(),
        ));
    }
    Ok(())
}

// ─── Monte Carlo ────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct MonteCarloBody {
    historical_r: Vec<f64>,
    config: monte_carlo::McConfig,
}

async fn monte_carlo_route(
    _u: AuthUser,
    Json(b): Json<MonteCarloBody>,
) -> Result<Json<monte_carlo::McReport>, ApiError> {
    monte_carlo::simulate(&b.historical_r, &b.config)
        .ok_or_else(|| ApiError::BadRequest(
            "monte carlo input invalid — historical_r non-empty, n_curves > 0, trades_per_curve > 0".into()
        ))
        .map(Json)
}

// ─── Risk parity / on-off ───────────────────────────────────────────────

#[derive(Deserialize)]
struct RiskParityBody {
    assets: Vec<risk_parity::AssetVol>,
}

async fn risk_parity_route(
    _u: AuthUser,
    Json(b): Json<RiskParityBody>,
) -> Json<risk_parity::RiskParityReport> {
    Json(risk_parity::allocate(&b.assets))
}

/// Efficient frontier: min-variance + max-Sharpe tangency portfolios + frontier curve.
async fn efficient_frontier_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::efficient_frontier::FrontierInput>,
) -> Json<traderview_core::efficient_frontier::FrontierReport> {
    Json(traderview_core::efficient_frontier::generate(&b))
}

/// Valuation multiples: P/E, P/B, P/S, PEG, and dividend/earnings/FCF yields.
async fn valuation_multiples_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::valuation_multiples::MultiplesInput>,
) -> Json<traderview_core::valuation_multiples::MultiplesReport> {
    Json(traderview_core::valuation_multiples::generate(&b))
}

/// Dividend discount model: Gordon growth or two-stage intrinsic value.
async fn dividend_discount_model_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::dividend_discount_model::DdmInput>,
) -> Json<traderview_core::dividend_discount_model::DdmReport> {
    Json(traderview_core::dividend_discount_model::generate(&b))
}

/// Probability of profit: lognormal P(profit) for a defined profit zone.
async fn probability_of_profit_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::probability_of_profit::PopInput>,
) -> Json<Option<traderview_core::probability_of_profit::PopReport>> {
    Json(traderview_core::probability_of_profit::compute(&b))
}

/// Straddle: max profit/loss, breakevens, profit-zone width.
async fn straddle_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::straddle::Straddle>,
) -> Json<Option<traderview_core::straddle::StraddleReport>> {
    Json(traderview_core::straddle::analyze(&b))
}

/// Strangle: max profit/loss, breakevens, profit-zone width.
async fn strangle_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::strangle::Strangle>,
) -> Json<Option<traderview_core::strangle::StrangleReport>> {
    Json(traderview_core::strangle::analyze(&b))
}

/// Collar: max profit/loss, upside cap, downside floor, breakeven.
async fn collar_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::collar::Collar>,
) -> Json<Option<traderview_core::collar::CollarReport>> {
    Json(traderview_core::collar::analyze(&b))
}

/// Iron butterfly: max profit/loss, breakevens, wing width.
async fn iron_butterfly_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::iron_butterfly::IronButterfly>,
) -> Json<Option<traderview_core::iron_butterfly::IronButterflyReport>> {
    Json(traderview_core::iron_butterfly::analyze(&b))
}

/// Butterfly spread: max profit/loss, breakevens, wing width, debit ratio.
async fn butterfly_spread_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::butterfly_spread::Butterfly>,
) -> Json<Option<traderview_core::butterfly_spread::ButterflyReport>> {
    Json(traderview_core::butterfly_spread::analyze(&b))
}

#[derive(serde::Deserialize)]
struct BoxSpreadBody {
    strike_low: f64,
    strike_high: f64,
    call_low_price: f64,
    call_high_price: f64,
    put_low_price: f64,
    put_high_price: f64,
    time_to_expiry_years: f64,
    #[serde(default)]
    market_risk_free_rate: f64,
    #[serde(default)]
    arbitrage_threshold_bps: f64,
}

/// Box spread: synthetic-loan implied rate and arbitrage check.
async fn box_spread_route(
    _u: AuthUser,
    Json(b): Json<BoxSpreadBody>,
) -> Json<Option<traderview_core::box_spread::BoxSpreadReport>> {
    Json(traderview_core::box_spread::compute(
        b.strike_low,
        b.strike_high,
        b.call_low_price,
        b.call_high_price,
        b.put_low_price,
        b.put_high_price,
        b.time_to_expiry_years,
        b.market_risk_free_rate,
        b.arbitrage_threshold_bps,
    ))
}

/// Crypto perpetual liquidation price (isolated margin).
async fn crypto_liquidation_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::crypto_liquidation::LiquidationInput>,
) -> Json<traderview_core::crypto_liquidation::LiquidationReport> {
    Json(traderview_core::crypto_liquidation::generate(&b))
}

/// Perpetual funding & basis: funding cost, annualized rate, premium to spot.
async fn perp_funding_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::perp_funding::PerpFundingInput>,
) -> Json<traderview_core::perp_funding::PerpFundingReport> {
    Json(traderview_core::perp_funding::generate(&b))
}

/// SPAN-style portfolio margin: worst-case loss across 16 risk scenarios.
async fn span_margin_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::span_margin::SpanInput>,
) -> Json<traderview_core::span_margin::SpanReport> {
    Json(traderview_core::span_margin::generate(&b))
}

/// IV smile surface: 2D implied-vol grid across moneyness and expiry.
async fn iv_surface_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::iv_surface::IvSurfaceInput>,
) -> Json<traderview_core::iv_surface::IvSurfaceReport> {
    Json(traderview_core::iv_surface::generate(&b))
}

/// Merton structural default: distance to default and probability of default.
async fn merton_default_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::merton_default::MertonInput>,
) -> Json<traderview_core::merton_default::MertonReport> {
    Json(traderview_core::merton_default::generate(&b))
}

/// CAPE valuation & CAPE-adjusted safe withdrawal rate.
async fn cape_valuation_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::cape_valuation::CapeInput>,
) -> Json<traderview_core::cape_valuation::CapeReport> {
    Json(traderview_core::cape_valuation::generate(&b))
}

/// Retirement decumulation Monte Carlo: success rate + percentile ending balances.
async fn decumulation_mc_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::decumulation_mc::DecumulationInput>,
) -> Json<traderview_core::decumulation_mc::DecumulationReport> {
    Json(traderview_core::decumulation_mc::generate(&b))
}

/// Callable-bond option-adjusted spread via a short-rate lattice.
async fn callable_oas_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::callable_oas::CallableOasInput>,
) -> Json<traderview_core::callable_oas::CallableOasReport> {
    Json(traderview_core::callable_oas::generate(&b))
}

#[derive(serde::Deserialize)]
struct IvConeBody {
    spot: f64,
    #[serde(default)]
    term: Vec<traderview_core::iv_cone::TermPoint>,
}

/// IV cone: expected 1σ/2σ move bands across the IV term structure.
async fn iv_cone_route(
    _u: AuthUser,
    Json(b): Json<IvConeBody>,
) -> Json<Option<Vec<traderview_core::iv_cone::ConeRow>>> {
    Json(traderview_core::iv_cone::compute(b.spot, &b.term))
}

#[derive(serde::Deserialize)]
struct GammaPinBody {
    #[serde(default)]
    strike_gex: Vec<traderview_core::gamma_pin_zone::StrikeGex>,
    spot: f64,
    #[serde(default = "default_pin_radius")]
    pin_radius_pct: f64,
}

fn default_pin_radius() -> f64 {
    2.0
}

/// Gamma pin zone: gamma-flip level and the pinning strike near spot.
async fn gamma_pin_zone_route(
    _u: AuthUser,
    Json(b): Json<GammaPinBody>,
) -> Json<Option<traderview_core::gamma_pin_zone::GammaPinReport>> {
    Json(traderview_core::gamma_pin_zone::compute(&b.strike_gex, b.spot, b.pin_radius_pct))
}

#[derive(serde::Deserialize)]
struct CalendarBody {
    spread: traderview_core::calendar_spread::CalendarSpread,
    config: traderview_core::calendar_spread::AnalyzerConfig,
}

/// Calendar spread: net debit, P&L grid, breakevens, max profit/loss.
async fn calendar_spread_route(
    _u: AuthUser,
    Json(b): Json<CalendarBody>,
) -> Json<Option<traderview_core::calendar_spread::CalendarReport>> {
    Json(traderview_core::calendar_spread::analyze(&b.spread, &b.config))
}

async fn risk_on_off_route(
    _u: AuthUser,
    Json(snap): Json<risk_on_off::CrossAssetSnapshot>,
) -> Json<risk_on_off::RiskReport> {
    Json(risk_on_off::evaluate(&snap))
}

// ─── Margin / buying power ──────────────────────────────────────────────

async fn margin_call_route(
    _u: AuthUser,
    Json(snap): Json<margin_call::AccountSnapshot>,
) -> Json<margin_call::MarginCallReport> {
    Json(margin_call::evaluate(&snap))
}

#[derive(Deserialize)]
struct MarginRunwayBody {
    account_equity: f64,
    position_value: f64,
    maintenance_req_pct: f64,
}

async fn margin_runway_route(
    _u: AuthUser,
    Json(b): Json<MarginRunwayBody>,
) -> Json<margin_runway::MarginRunwayReport> {
    Json(margin_runway::compute(
        b.account_equity,
        b.position_value,
        b.maintenance_req_pct,
    ))
}

async fn buying_power_route(
    _u: AuthUser,
    Json(input): Json<buying_power::BpInput>,
) -> Json<buying_power::BpReport> {
    Json(buying_power::compute(&input))
}

// ─── Tax / fees ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TaxLossHarvestBody {
    losers: Vec<tax_loss_harvest::OpenLoser>,
    recent_buys: Vec<tax_loss_harvest::RecentBuy>,
    today: NaiveDate,
    /// YTD realized loss in dollars (positive when net-losing) — used to
    /// flag harvests that push past the $3k ordinary-income offset cap.
    realized_loss_ytd: Decimal,
    /// Trader-status mark-to-market (§475(f)) election — when true, the
    /// $3k cap doesn't apply.
    #[serde(default)]
    mtm_elected: bool,
}

async fn tax_loss_harvest_route(
    _u: AuthUser,
    Json(b): Json<TaxLossHarvestBody>,
) -> Json<tax_loss_harvest::HarvestReport> {
    Json(tax_loss_harvest::suggest(
        &b.losers,
        &b.recent_buys,
        b.today,
        b.realized_loss_ytd,
        b.mtm_elected,
    ))
}

#[derive(Deserialize)]
struct WashSaleBody {
    closings: Vec<wash_sale::ClosingTrade>,
    openings: Vec<wash_sale::OpeningExecution>,
}

#[derive(Serialize)]
struct WashSaleResp {
    hits: Vec<wash_sale::WashHit>,
    total_disallowed: Decimal,
}

async fn wash_sale_route(_u: AuthUser, Json(b): Json<WashSaleBody>) -> Json<WashSaleResp> {
    let hits = wash_sale::detect_hits(&b.closings, &b.openings);
    let total_disallowed = wash_sale::total_disallowed(&hits);
    Json(WashSaleResp {
        hits,
        total_disallowed,
    })
}

#[derive(Deserialize)]
struct CostBasisBody {
    lots: Vec<cost_basis::CostLot>,
    qty_to_close: Decimal,
    price_per_share: Decimal,
    method: cost_basis::LotMethod,
}

async fn cost_basis_route(
    _u: AuthUser,
    Json(b): Json<CostBasisBody>,
) -> Json<cost_basis::CloseReport> {
    Json(cost_basis::close(
        &b.lots,
        b.qty_to_close,
        b.price_per_share,
        b.method,
    ))
}

#[derive(Deserialize)]
struct CommissionOptimizerBody {
    executions: Vec<commission_optimizer::Execution>,
    tiers: Vec<commission_optimizer::Tier>,
}

async fn commission_optimizer_route(
    _u: AuthUser,
    Json(b): Json<CommissionOptimizerBody>,
) -> Json<commission_optimizer::OptimizerReport> {
    Json(commission_optimizer::evaluate(&b.executions, &b.tiers))
}

// ─── Fixed income / FX ──────────────────────────────────────────────────

async fn yield_curve_route(
    _u: AuthUser,
    Json(c): Json<yield_curve::YieldCurve>,
) -> Json<yield_curve::CurveReport> {
    Json(yield_curve::classify(&c))
}

#[derive(Deserialize)]
struct BondDurationBody {
    cash_flows: Vec<bond_duration::CashFlow>,
    ytm: f64,
    compounding_per_year: usize,
}

async fn bond_duration_route(
    _u: AuthUser,
    Json(b): Json<BondDurationBody>,
) -> Json<bond_duration::DurationReport> {
    Json(bond_duration::compute(
        &b.cash_flows,
        b.ytm,
        b.compounding_per_year,
    ))
}

#[derive(Deserialize)]
struct CarryScoreBody {
    long_rate: f64,
    funding_rate: f64,
    annualized_vol: f64,
}

async fn carry_score_route(
    _u: AuthUser,
    Json(b): Json<CarryScoreBody>,
) -> Json<carry_score::CarryReport> {
    Json(carry_score::score(
        b.long_rate,
        b.funding_rate,
        b.annualized_vol,
    ))
}

/// FX carry + covered interest parity — pure compute.
async fn fx_carry_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::fx_carry::Input>,
) -> Result<Json<traderview_core::fx_carry::Report>, ApiError> {
    traderview_core::fx_carry::compute(&b)
        .map(Json)
        .ok_or_else(|| ApiError::BadRequest("invalid inputs: spot/notional/days must be positive, rates finite".into()))
}

#[derive(Deserialize)]
struct CurrencyExposureBody {
    positions: Vec<currency_exposure::ForeignPosition>,
    home_currency: String,
    /// Map of currency-code → spot rate to convert TO home (e.g. for a USD
    /// home, EUR → 1.08 means 1 EUR = 1.08 USD).
    fx_to_home: BTreeMap<String, f64>,
}

async fn currency_exposure_route(
    _u: AuthUser,
    Json(b): Json<CurrencyExposureBody>,
) -> Json<currency_exposure::CurrencyReport> {
    Json(currency_exposure::analyze(
        &b.positions,
        &b.home_currency,
        &b.fx_to_home,
    ))
}

async fn vix_term_structure_route(
    _u: AuthUser,
    Json(ts): Json<vix_term_structure::VixTermStructure>,
) -> Json<vix_term_structure::TermStructureReport> {
    Json(vix_term_structure::analyze(&ts))
}

// ── §1244 small business stock loss ────────────────────────────────────
// Mounted at /api/calc/section-1244. Pure compute; takes the full
// Section1244Input struct (loss + filing status + prior-claimed + the
// 5-test qualification checklist) and returns the ordinary/capital split.

async fn section_1244_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1244::Section1244Input>,
) -> Result<Json<traderview_expense::section_1244::Section1244Result>, ApiError> {
    if b.realized_loss < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "realized_loss must be >= 0 (pass loss as positive number)".into(),
        ));
    }
    if b.ordinary_loss_claimed_this_year_so_far < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "ordinary_loss_claimed_this_year_so_far must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1244::compute(&b)))
}

// ── §1245 / §1250 depreciation recapture ───────────────────────────
// Mounted at /api/calc/section-1245-1250. §1245(a)(1) personal-
// property recapture = min(gain, accumulated depreciation) ordinary;
// §1250 real-property: post-1986 MACRS straight-line → zero ordinary
// + §1(h)(7) unrecaptured §1250 gain taxed at 25% maximum rate for
// individuals (vs. 20% LTCG); pre-1986 / accelerated → recapture of
// additional depreciation as ordinary; residual gain flows to §1231.

async fn section_1245_1250_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1245_1250::Section1245_1250Input>,
) -> Result<Json<traderview_expense::section_1245_1250::Section1245_1250Result>, ApiError> {
    if b.accumulated_depreciation_dollars < 0 || b.additional_depreciation_dollars < 0 {
        return Err(ApiError::BadRequest(
            "non-negative depreciation inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1245_1250::compute(&b)))
}

// ── §1202 QSBS exclusion ──────────────────────────────────────────────
// Mounted at /api/calc/section-1202. Pure compute; up to $10M / 10× basis
// of gain on qualified small-business stock excluded at 50/75/100% depending
// on acquisition date.

async fn section_1202_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1202::Section1202Input>,
) -> Result<Json<traderview_expense::section_1202::Section1202Result>, ApiError> {
    if b.taxpayer_basis < Decimal::ZERO {
        return Err(ApiError::BadRequest("taxpayer_basis must be >= 0".into()));
    }
    if b.prior_exclusion_used_this_issuer < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "prior_exclusion_used_this_issuer must be >= 0".into(),
        ));
    }
    if b.disposition_date < b.acquisition_date {
        return Err(ApiError::BadRequest(
            "disposition_date must be >= acquisition_date".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1202::compute(&b)))
}

// ── §1045 QSBS rollover ───────────────────────────────────────────────
// Mounted at /api/calc/section-1045. Pure compute; rolls QSBS gain
// into replacement QSBS within 60 days, deferring up to the full
// gain. Holding period tacks for the §1202 5-year clock.

async fn section_1045_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1045::Section1045Input>,
) -> Result<Json<traderview_expense::section_1045::Section1045Result>, ApiError> {
    if b.sale_proceeds_net < Decimal::ZERO || b.replacement_cost < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "sale_proceeds_net and replacement_cost must be >= 0".into(),
        ));
    }
    if b.original_sale_date < b.original_acquisition_date {
        return Err(ApiError::BadRequest(
            "original_sale_date must be >= original_acquisition_date".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1045::compute(&b)))
}

// ── §163(j) business interest limitation ─────────────────────────────
// Mounted at /api/calc/section-163j. Pure compute; caps margin interest
// deduction at 30% × ATI + business interest income + floor plan
// financing for §475(f) traders. Indefinite carryforward; small-business
// exception ($30M gross receipts for 2024) bypasses the cap entirely.

async fn section_163j_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_163j::Section163jInput>,
) -> Result<Json<traderview_expense::section_163j::Section163jResult>, ApiError> {
    if b.business_interest_expense < Decimal::ZERO
        || b.business_interest_income < Decimal::ZERO
        || b.floor_plan_financing_interest < Decimal::ZERO
        || b.prior_year_carryforward < Decimal::ZERO
        || b.avg_3yr_gross_receipts < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_163j::compute(&b)))
}

// ── §165(d) wagering loss deduction ─────────────────────────────────
// Mounted at /api/calc/section-165d. Pre-OBBBA: 100% of losses up
// to winnings; post-OBBBA (P.L. 119-21 signed 2025-07-04 eff. 2026):
// 90% of losses + still capped at winnings; phantom-income emerges
// when 90% × losses < winnings ≤ losses; §162 trade-or-business
// expense carve-out preserved for professional gamblers; itemized
// Schedule A (Schedule C for professional).

async fn section_165d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_165d::Section165dInput>,
) -> Result<Json<traderview_expense::section_165d::Section165dResult>, ApiError> {
    Ok(Json(traderview_expense::section_165d::compute(&b)))
}

// ── §165(g) worthless securities deduction ──────────────────────────
// Mounted at /api/calc/section-165g. Wholly worthless capital-asset
// security deemed sold last day of taxable year (§165(g)(1));
// §165(g)(2) security definition (stock + bond + debenture + registered
// indebtedness); §165(g)(3) affiliated-domestic-corporation ordinary
// loss exception (§1504(a)(2) 80%/80% + > 90% non-passive gross
// receipts); §1244 small business stock ordinary loss priority.

async fn section_165g_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_165g::Section165gInput>,
) -> Result<Json<traderview_expense::section_165g::Section165gResult>, ApiError> {
    if b.non_passive_gross_receipts_pct_bp > 10_000 {
        return Err(ApiError::BadRequest(
            "non_passive_gross_receipts_pct_bp must be ≤ 100% (10,000bp)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_165g::compute(&b)))
}

// ── §408 traditional IRA + SEP + SIMPLE + collectibles + QCD ─────────
// Mounted at /api/calc/section-408. § 408(a) IRA defined as trust for
// exclusive benefit; six requirements (within § 219 limits + qualified
// trustee + no life insurance + nonforfeitable + not commingled +
// § 401(a)(9) RMD). § 408(b) individual retirement annuity. 2026
// contribution limits aggregate with § 408A Roth: $7,500 base + $1,100
// catch-up (50+) = $8,600. § 219(g) deduction phase-out for ACTIVE
// PARTICIPANT in employer-sponsored plan (Single/HOH $81K-$91K; MFJ
// covered $129K-$149K; MFJ spouse-covered $242K-$252K; MFS $0-$10K).
// § 408(d)(1) distributions ordinary income; § 408(d)(2) PRO-RATA RULE
// aggregate across ALL IRAs (CRITICAL for backdoor Roth); § 408(d)(3)
// 60-day rollover + ONE-ROLLOVER-PER-YEAR per Bobrow v. Commissioner
// T.C. Memo 2014-21 + IRS Announcement 2014-15; § 408(d)(6) RMD per
// SECURE Act 2.0 (age 73 / 75 born 1960+); § 408(d)(8) QCD age 70½+
// $111,000 (2026) + § 408(d)(8)(F) $50K split-interest entity.
// § 408(k) SEP IRA 25%/$70K. § 408(m) COLLECTIBLES PROHIBITION
// (artwork, antique, gem, stamp/coin, alcohol) + § 408(m)(3) EXCEPTION
// for gold/silver coins/bullion. § 408(p) SIMPLE IRA (≤ 100 employees;
// 2026 $17K + $4K catch-up). § 408(q) deemed IRA. Companion to § 408A
// + § 72(t) + § 67(g) + § 1411 + § 475 + § 4975 + § 4973. Created by
// ERISA 1974 (Pub. L. 93-406, September 2, 1974); modified by SECURE
// Act 2019 + SECURE Act 2.0 of 2022.

async fn section_408_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_408::Section408Input>,
) -> Result<Json<traderview_expense::section_408::Section408Result>, ApiError> {
    Ok(Json(traderview_expense::section_408::check(&b)))
}

// ── §408A Roth IRA contribution + phase-out + qualified distribution ─
// Mounted at /api/calc/section-408a. § 408A(c)(1) 2026 contribution
// limit aggregate with § 408 traditional IRA — $7,500 base + $1,100
// catch-up (age 50+); § 408A(c)(3) income-based phase-out (Single/HOH
// $153K-$168K; MFJ $242K-$252K; MFS $0-$10K NOT cost-of-living
// adjusted); § 408A(c)(3)(B) modified AGI DISREGARDS Roth conversion
// income; § 408A(d)(2) qualified distribution two-prong test (5-year
// holding + age 59½ OR disability OR death OR first-time home up to
// $10K lifetime); § 408A(d)(3) ordering rules (contributions then
// conversions then earnings); § 408A(d)(3)(A) separate 5-year per-
// conversion holding period; § 408A(e) backdoor Roth (non-deductible
// traditional + conversion; pro-rata rule under § 408(d)(2));
// § 408A(c)(5) NO RMD during owner's lifetime; Roth distributions
// EXEMPT from § 1411 NIIT 3.8% surtax. Created by Taxpayer Relief Act
// of 1997 § 302 (Pub. L. 105-34, August 5, 1997); modified by SECURE
// Act of 2019 + SECURE Act 2.0 of 2022. Trader-critical fact patterns:
// high-income trader backdoor Roth; active trader self-directed Roth
// IRA escaping § 1411 NIIT; mega backdoor Roth (after-tax 401(k));
// § 72(t) substantially-equal-periodic-payments. § 4973 6% excise tax
// on excess contributions.

async fn section_408a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_408a::Section408aInput>,
) -> Result<Json<traderview_expense::section_408a::Section408aResult>, ApiError> {
    Ok(Json(traderview_expense::section_408a::check(&b)))
}

// ── § 401(k) Cash or Deferred Arrangements ──────────────────────────
// Mounted at /api/calc/section-401k. Pure compute; 2026 limits per
// IRS Notice 2025-67: § 402(g)(1) elective deferral $24,500;
// § 414(v)(1) catch-up age 50+ $8,000; § 414(v)(2)(E) SECURE 2.0
// enhanced catch-up ages 60-63 $11,250; § 415(c)(1)(A) annual
// addition $72,000; § 401(a)(17) compensation limit $360,000;
// HCE threshold $160,000; § 414(v)(7) mandatory Roth catch-up
// threshold $150,000 (SECURE 2.0 § 603 effective 2026); § 401(k)(3)
// ADP test (HCE ADP ≤ greater of non-HCE × 1.25 OR non-HCE + 2%);
// § 401(k)(12) safe harbor (3% non-elective OR basic match);
// § 402A designated Roth § 401(k); SECURE 2.0 § 325 (no lifetime
// RMD on Roth § 401(k)); SECURE 2.0 § 604 (Roth employer match);
// mega backdoor Roth via § 408A(d)(3) in-plan rollover.

async fn section_401k_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_401k::Section401kInput>,
) -> Result<Json<traderview_expense::section_401k::Section401kResult>, ApiError> {
    Ok(Json(traderview_expense::section_401k::check(&b)))
}

// ── § 415 Limits on Benefits and Contributions (umbrella statute) ────
// Mounted at /api/calc/section-415. Pure compute; 2026 limits per
// IRS Notice 2025-67: § 415(b)(1)(A) DB annual benefit $290,000;
// § 415(c)(1)(A) DC annual addition $72,000; § 401(a)(17)
// compensation limit $360,000. § 415(a) disqualification cascade
// (denial of § 401(a) qualified status for entire plan if any
// participant exceeds). § 415(b) DB limit = lesser of dollar limit
// or 100% of average high-3-year compensation (NOT subject to
// § 401(a)(17)). § 415(c) DC annual addition = employer + employee
// pretax + Roth + forfeitures (EXCLUDES § 414(v) catch-up). § 415(d)
// CPI-U annual COLA adjustment. § 415(f) aggregation: all DC plans
// of single employer aggregated; all DB plans of single employer
// aggregated; DC and DB limits applied SEPARATELY (§ 415(f)(2));
// § 414(b)/(c)/(m)/(o) controlled group + affiliated service group
// treat related employers as single employer. § 415(g) anti-cutback;
// § 415(k) grandfathered old-limit benefits; § 415(n) USERRA make-up
// rights.

async fn section_415_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_415::Section415Input>,
) -> Result<Json<traderview_expense::section_415::Section415Result>, ApiError> {
    Ok(Json(traderview_expense::section_415::check(&b)))
}

// ── § 422 Incentive Stock Options (ISOs) ─────────────────────────────
// Mounted at /api/calc/section-422. Pure compute; § 422(b) 6-element
// statutory test (shareholder-approved plan + 10-year window + price
// ≥ FMV + 3-month employment trail + transferability + 10-year
// exercise period); § 422(d) $100K annual limit (excess auto NQSO);
// § 422(a) 2-year-from-grant + 1-year-from-exercise qualified-
// disposition holding periods; § 421(b) disqualifying-disposition
// ordinary-income lesser-of rule (FMV-exercise - strike OR sale -
// strike); § 56(b)(3) AMT preference on exercise spread; § 422(c)(2)
// same-year disqualifying-disposition AMT reversal; § 53 AMT credit
// recovery; § 1411 NIIT 3.8% on qualified-disposition LTCG.

async fn section_422_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_422::Section422Input>,
) -> Result<Json<traderview_expense::section_422::Section422Result>, ApiError> {
    Ok(Json(traderview_expense::section_422::check(&b)))
}

// ── § 421 General rules for statutory stock options ──────────────────
// Mounted at /api/calc/section-421. Pure compute; § 421(a) general
// rule — no income at grant or exercise of statutory option (§ 422
// ISO or § 423 ESPP) for regular income tax purposes + no § 162
// deduction to employer + only option price considered as received
// by corporation. § 421(b) disqualifying disposition — increase in
// FMV over option price at time of exercise treated as compensation
// (ordinary income) in year of disposition; § 162 deduction allowed
// to employer; additional gain treated as capital gain per § 1222
// holding period from exercise date. ISO qualifying-disposition
// holding requirements per § 422(a)(1): no disposition within 2
// years from grant date AND no disposition within 1 year from
// transfer (exercise) date; ESPP similar requirements per § 423(a)(1).
// Employment requirement per § 422(a)(2): individual must be employee
// of granting corporation (or related corp under § 424(e)/(f)) at all
// times from grant date through 3 months before exercise (death or
// disability extends). AMT preference per § 56(b)(3) on exercise
// spread creates 'phantom income' for trader-employees. Information
// reporting per § 6039 + Form 3921 (ISO) + Form 3922 (ESPP).
// Coordination with § 1042 (iter 480): ISO-exercised shares NOT
// 'qualified securities' for ESOP rollover. Coordination with § 83:
// § 421 OVERRIDES § 83 for qualifying ISO/ESPP exercises. Original
// framework Tax Reform Act of 1976 + Economic Recovery Tax Act of
// 1981 + American Jobs Creation Act of 2004.

async fn section_421_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_421::Section421Input>,
) -> Result<Json<traderview_expense::section_421::Section421Result>, ApiError> {
    Ok(Json(traderview_expense::section_421::check(&b)))
}

// ── § 423 Employee Stock Purchase Plans (ESPPs) ──────────────────────
// Mounted at /api/calc/section-423. Pure compute; § 423(b) 9-element
// statutory test (employees only + shareholder-approved + no 5%+
// owner + all employees eligible + same rights/privileges + price
// ≥ 85% of lower of offering/purchase FMV + 27-month/5-year outer
// limit + $25K annual accrual cap + non-transferable); § 423(b)(6)
// look-back provision (85% of LOWER of offering or purchase FMV);
// § 421(a) 2-year-from-offering + 1-year-from-purchase qualifying-
// disposition holding periods; § 423(c) qualifying-disposition
// ordinary-income lesser-of rule (discount-at-offering OR actual
// gain); § 421(b) disqualifying-disposition full-spread-at-purchase
// rule; § 162 employer deduction only on disqualifying; Notice
// 2002-47 + Rev. Rul. 71-52 FICA exemption on qualifying ordinary
// income; § 1411 NIIT 3.8% on qualifying LTCG; § 424(d)
// constructive-ownership rules; Form 3922 ESPP Transfer
// Information Statement.

async fn section_423_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_423::Section423Input>,
) -> Result<Json<traderview_expense::section_423::Section423Result>, ApiError> {
    Ok(Json(traderview_expense::section_423::check(&b)))
}

// ── § 4973 excise tax on excess IRA / Roth / HSA / Coverdell contrib ─
// Mounted at /api/calc/section-4973. Pure compute; 6% annual non-
// deductible excise tax on excess contributions to § 408(a)
// traditional IRA, § 408A Roth IRA, § 408(b) IRA annuity, § 408(p)
// SIMPLE IRA, § 530 Coverdell ESA, § 220 Archer MSA, § 223 HSA;
// § 4973(c) correction window (return due date plus extensions =
// October 15) with NIA computed under Treas. Reg. § 1.408-11(b);
// SECURE Act 2.0 § 333 eliminates additional § 72(t) 10% early-
// withdrawal penalty on NIA when corrective distribution made
// timely; SECURE Act 2.0 § 313 establishes 6-year statute of
// limitations (previously no SoL); § 4973(g) uncorrected excess
// carryover-absorbed into subsequent year limit; Form 5329 Parts
// III-VII reporting.

async fn section_4973_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4973::Section4973Input>,
) -> Result<Json<traderview_expense::section_4973::Section4973Result>, ApiError> {
    Ok(Json(traderview_expense::section_4973::check(&b)))
}

// ── § 4972 tax on nondeductible contributions to qualified plans ─────
// Mounted at /api/calc/section-4972. Pure compute; § 4972(a) imposes
// 10% annual excise tax on nondeductible contributions to qualified
// employer plans paid by EMPLOYER (not individual — § 4973 covers
// individual side); § 4972(c)(1)(A) nondeductible = current-year
// employer contributions plus unused prior-year carryforwards less
// § 404 deduction limit; § 4972(c)(2) ordering rule applies deduction
// first to carryforwards then to current contributions; § 4972(d)
// qualified employer plan = § 401(a) qualified plan + § 403(a)
// qualified annuity + § 408(k) SEP-IRA + § 408(p) SIMPLE IRA;
// § 4972(c)(6) exceptions: (A) SEP excess allocable to participant
// under § 415, (B) PSP deductibility from post-plan-year-end
// compensation increase. Carryforward compounds annually until
// consumed by future § 404 deduction headroom or returned under
// § 401(a)(2) reversion (§ 4980 iter 460 reversion excise also
// applies). Form 5330 filing deadline last day of 7th month after
// employer tax-year-end. Distinction from § 4973 (iter 442 individual
// IRA 6% excise) + § 4974 (iter 436 RMD 25% post-SECURE 2.0) +
// § 4975 (iter 434 prohibited transactions 15%/100%). Original
// enactment Deficit Reduction Act of 1984 Pub. L. 98-369 with current
// 10% rate from Omnibus Budget Reconciliation Act of 1989 Pub. L.
// 101-239.

async fn section_4972_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4972::Section4972Input>,
) -> Result<Json<traderview_expense::section_4972::Section4972Result>, ApiError> {
    Ok(Json(traderview_expense::section_4972::check(&b)))
}

// ── § 4974 excise tax on RMD failures (post-SECURE 2.0) ──────────────
// Mounted at /api/calc/section-4974. Pure compute; 25% standard +
// 10% reduced (within § 4974(e) 2-year correction window) excise
// tax on shortfall between RMD required and amount distributed;
// SECURE Act 2.0 § 302 reduced rate from 50% to 25%; SECURE Act
// 2.0 § 107 raised RMD age from 72 to 73 (born 1951-1959) and to
// 75 (born 1960+) effective January 1, 2033; § 408(d)(8) QCD up
// to $108K satisfies RMD without inclusion in gross income;
// § 408A(c)(5) exempts Roth IRA from lifetime RMD; § 401(a)(9)(B)
// post-death 5-category beneficiary stretch / 10-year rule;
// § 4974(d) reasonable-error waiver via Form 5329.

async fn section_4974_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4974::Section4974Input>,
) -> Result<Json<traderview_expense::section_4974::Section4974Result>, ApiError> {
    Ok(Json(traderview_expense::section_4974::check(&b)))
}

// ── § 4501 Repurchase of Corporate Stock Excise Tax ──────────────────
// Mounted at /api/calc/section-4501 (iter 496). Pure compute. IRA 2022
// Pub. L. 117-169 § 10201 1% excise tax on stock buybacks by covered
// corporations effective for repurchases after December 31, 2022. § 4501(b)
// covered-corporation definition: domestic corporation traded on
// established securities market per § 7704(b)(1) — NYSE, NASDAQ, national
// exchanges. SPAC sponsor / shareholder redemptions explicitly subject
// per Final Regs TD 10002 (July 3, 2024); IRS rejected SPAC carve-out.
// § 4501(c)(3) netting rule: FMV of repurchases reduced by FMV of statutory
// § 4501(e) excepted repurchases plus FMV of stock issuances (including
// compensatory RSU vest, ISO/NSO exercise, ESPP, equity grants) per
// Treas. Reg. § 1.4501-2(c). § 4501(e) six exceptions: § 368
// reorganization, ESOP/retirement-plan contribution, $1M de minimis,
// dealer ordinary course, RIC/REIT, § 301 dividend treatment. § 4501(d)
// specified-affiliate extension to foreign-parent anti-inversion. Excise
// tax NOT deductible per § 275(a)(6) — permanent book-tax difference.
// Form 7208 attached to Form 720 quarterly. Coordination with § 280G
// (golden parachute), § 421 (statutory stock option issuance offset),
// § 56A (corporate AMT 15% — separate IRA 2022 provision), § 4960
// (ATEO executive comp 21%), § 1042 (ESOP rollover for retirement-plan
// exception).

async fn section_4501_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4501::Section4501Input>,
) -> Result<Json<traderview_expense::section_4501::Section4501Result>, ApiError> {
    Ok(Json(traderview_expense::section_4501::check(&b)))
}

// ── § 4940 annual excise tax on private foundation NII ───────────────
// Mounted at /api/calc/section-4940. Pure compute; § 4940(a) imposes
// annual excise tax on net investment income of every domestic tax-
// exempt private foundation (except § 4940(d) exempt operating
// foundations). Post-Dec-20-2019 regime: single flat rate of 1.39%
// per Further Consolidated Appropriations Act, 2020, Pub. L. 116-94
// (signed December 20, 2019) which amended § 4940(a) and REPEALED
// former § 4940(e). Pre-Dec-20-2019 regime: 2% standard with 1%
// reduced under former § 4940(e) for foundations meeting
// distribution-requirement tests (qualifying distributions ≥ average
// 5-year payout + 1% of NII). Net investment income per § 4940(c):
// gross investment income (§ 4940(c)(1) — interest + dividends +
// rents + securities-loan payments + royalties) plus net capital
// gain from sale of investment property (§ 4940(c)(4)(A)) minus
// allowable deductions (§ 4940(c)(2) — ordinary and necessary
// expenses paid or incurred for production/collection of gross
// investment income). Exempt operating foundations per § 4940(d)
// four-part test: (A) publicly supported for at least 10 prior
// taxable years; (B) governing body broadly representative with not
// more than 25% disqualified persons under § 4946; (C) operating-
// foundation status under § 4942(j)(3); (D) no officer who is
// disqualified individual appointed by disqualified persons.
// Foreign private foundations under § 4948 separate regime at 4% on
// US-source gross investment income. § 501(c)(3) public charities NOT
// subject to § 4940. Quarterly estimated tax payments under § 6655.
// Original enactment Tax Reform Act of 1969, Pub. L. 91-172.

async fn section_4940_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4940::Section4940Input>,
) -> Result<Json<traderview_expense::section_4940::Section4940Result>, ApiError> {
    Ok(Json(traderview_expense::section_4940::check(&b)))
}

// ── § 4941 taxes on self-dealing (private foundation regime) ─────────
// Mounted at /api/calc/section-4941. Pure compute; four-tier excise
// tax structure on private-foundation self-dealing: § 4941(a)(1)
// Tier-1 disqualified person 10% of amount involved per year of
// taxable period; § 4941(a)(2) Tier-1 foundation manager 5% (knowing
// willful participant) capped at $20,000 per act per § 4941(c)(2);
// § 4941(b)(1) Tier-2 DP 200% (uncorrected within taxable period);
// § 4941(b)(2) Tier-2 manager 50% (refusing correction) capped at
// $20,000. Six self-dealing categories per § 4941(d)(1): (A) sale/
// exchange/lease; (B) lending of money or extension of credit; (C)
// furnishing goods/services/facilities; (D) compensation/expense
// reimbursement; (E) transfer/use of income or assets; (F) agreement
// to pay government official (§ 4946(c)). Four statutory exceptions
// per § 4941(d)(2): § 4941(d)(2)(B) interest-free loan from DP to PF
// for charitable purpose; § 4941(d)(2)(C) DP furnishes goods to PF
// without charge; § 4941(d)(2)(D) PF furnishes to DP on no more
// favorable basis than to general public; § 4941(d)(2)(E) reasonable
// compensation for personal services necessary to exempt purpose
// (NOT permitted to government official). Disqualified person per
// § 4946: substantial contributors (§ 507(d)(2) > $5K AND > 2%) +
// foundation managers (§ 4946(b)) + 20%-owners of contributor
// entities + family per § 4946(d) + 35%-controlled entities + other
// related PFs + government officials (§ 4946(c)). Amount involved
// per § 4941(e)(1): greater of money + FMV given vs received; loans
// per § 4941(e)(2). Taxable period per § 4941(e)(3) begins on
// transaction date, ends on earliest of statutory notice / tax
// assessment / correction. Correction per § 4941(e)(4): undo +
// place PF in position no worse than under highest fiduciary
// standards. Distinct from § 4958 (iter 466) intermediate sanctions:
// § 4941 applies to PRIVATE FOUNDATIONS only (excluded from § 4958
// per § 4958(e)), uses lower 10% Tier-1 rate but per-se rule (no
// excess-benefit comparison required); original enactment Tax
// Reform Act of 1969, Pub. L. 91-172.

async fn section_4941_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4941::Section4941Input>,
) -> Result<Json<traderview_expense::section_4941::Section4941Result>, ApiError> {
    Ok(Json(traderview_expense::section_4941::check(&b)))
}

// ── § 4942 taxes on PF failure to distribute income ──────────────────
// Mounted at /api/calc/section-4942. Pure compute; § 4942(a) 30%
// Tier-1 excise tax on undistributed income for each year/partial
// year deficiency remains uncorrected; § 4942(b) additional 100%
// Tier-2 if PF fails to make up deficient distribution within 90
// days of IRS notice. Distributable amount per § 4942(d) = minimum
// investment return (5% of non-charitable-use FMV under § 4942(e),
// reduced by acquisition indebtedness) reduced by § 4940 excise tax
// and UBI tax; must be paid as qualifying distributions by end of
// immediately following taxable year. Qualifying distributions per
// § 4942(g)(1)(A) = amount paid for § 170(c)(2)(B) religious/
// charitable/scientific/literary/educational/public purposes
// including reasonable and necessary administrative expenses, or
// § 4942(g)(1)(B) amount paid to acquire asset used directly in
// exempt purpose. § 4942(g)(2) set-asides for specific project
// payable within 60 months if suitability or cash distribution test
// satisfied. § 4942(h) treatment first out of prior-year UI then
// current year unless § 4942(h)(2) corpus election. § 4942(i) excess
// distributions carry forward FIVE years. Exceptions: § 4942(a)(2)(A)
// operating foundations § 4942(j)(3); § 4942(a)(2)(B) conduit
// foundations § 170(b)(1)(F)(ii); § 4942(j)(5) grandfathered PFs
// pre-May-27-1969. Distinct from § 4940 (iter 470) ANNUAL NII tax
// and § 4941 (iter 468) per-act self-dealing punitive — § 4942 is
// ANNUAL MINIMUM-DISTRIBUTION REQUIREMENT backed by 30% + 100%
// penalty. Original enactment Tax Reform Act of 1969 Pub. L. 91-172.

async fn section_4942_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4942::Section4942Input>,
) -> Result<Json<traderview_expense::section_4942::Section4942Result>, ApiError> {
    Ok(Json(traderview_expense::section_4942::check(&b)))
}

// ── § 4943 taxes on PF excess business holdings ──────────────────────
// Mounted at /api/calc/section-4943. Pure compute; § 4943(a)(1) 10%
// Tier-1 excise tax on value of excess business holdings of private
// foundation as of date of greatest excess during taxable year;
// § 4943(b) 200% Tier-2 if not corrected within taxable period.
// Combined holding limits under § 4943(c)(2): § 4943(c)(2)(A) default
// 20% combined PF + DP voting stock of corporation (or equivalent
// profits interest in partnership/joint venture/unincorporated
// enterprise); § 4943(c)(2)(B) raised to 35% if PF establishes
// effective control of business is in non-DPs; § 4943(c)(2)(C) 2%
// de minimis — PF alone may hold up to 2% regardless of DPs.
// § 4943(c)(3)(B) non-voting stock — PF may hold ALL non-voting
// stock if combined DP voting holdings under applicable limit.
// Business enterprise per § 4943(d)(3) EXCLUDES: (A) functionally-
// related business substantially related to PF exempt purpose; (B)
// 95% passive income test trade or business with ≥ 95% gross income
// from interest + dividends + rents + royalties + capital gains; and
// § 4944(c) program-related investments. § 4943(c)(6) FIVE-YEAR
// disposition period for holdings acquired by gift/bequest/devise;
// § 4943(c)(7) IRS may grant additional 5-year (10-year total)
// extension for complex/unusual estates. § 4943(g) FAMILY BUSINESS
// EXCEPTION (added by Tax Cuts and Jobs Act 2017, Pub. L. 115-97,
// Dec 22 2017) permits 100% PF ownership of philanthropic business
// holding if ALL THREE: PF owns ALL voting stock at all times; PF
// received voting stock OTHER THAN by purchase; all net operating
// income distributed annually + no DP serves as director/officer/
// employee. Original enactment Tax Reform Act of 1969 Pub. L. 91-172.

async fn section_4943_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4943::Section4943Input>,
) -> Result<Json<traderview_expense::section_4943::Section4943Result>, ApiError> {
    Ok(Json(traderview_expense::section_4943::check(&b)))
}

// ── § 4944 taxes on PF jeopardizing investments ──────────────────────
// Mounted at /api/calc/section-4944. Pure compute; § 4944(a)(1) 10%
// Tier-1 PF excise on amount of jeopardizing investment for each year
// or partial year in taxable period; § 4944(a)(2) 10% Tier-1 manager
// excise (knowing willful without reasonable cause) capped at $10,000
// per investment per § 4944(d)(2); § 4944(b)(1) 25% Tier-2 PF excise
// if not removed from jeopardy within taxable period; § 4944(b)(2) 5%
// Tier-2 manager excise (refuses correction) capped at $20,000.
// Jeopardizing investment standard = ordinary business care and
// prudence at TIME OF INVESTMENT (not hindsight) providing for long-
// term and short-term financial needs of PF to carry out exempt
// purposes; modern portfolio theory recognized. Categories typically
// scrutinized per 26 C.F.R. § 53.4944-1(a)(2): trading on margin +
// short sales + options/derivatives + futures/commodity + warrants +
// working interests oil/gas + land contracts + speculative private
// placements (NOT per se jeopardizing — facts and circumstances).
// § 4944(c) PROGRAM-RELATED INVESTMENT (PRI) EXCEPTION: NOT jeopardizing
// if ALL THREE — (1) primary purpose accomplishes § 170(c)(2)(B)
// charitable; (2) no significant income or appreciation purpose; (3)
// no political/lobbying purpose under § 4945(d)(1) or § 4945(d)(2).
// Distinction from § 4943 (iter 474): § 4943 limits concentration in
// single business enterprise; § 4944 evaluates prudence across
// portfolio. Original enactment Tax Reform Act of 1969 Pub. L. 91-172.

async fn section_4944_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4944::Section4944Input>,
) -> Result<Json<traderview_expense::section_4944::Section4944Result>, ApiError> {
    Ok(Json(traderview_expense::section_4944::check(&b)))
}

// ── § 4945 taxes on PF taxable expenditures ──────────────────────────
// Mounted at /api/calc/section-4945. Pure compute; § 4945(a)(1) 20%
// Tier-1 PF excise on amount of taxable expenditure; § 4945(a)(2) 5%
// Tier-1 manager excise (knowingly agreed) capped at $10,000 per
// expenditure per § 4945(c)(2); § 4945(b)(1) 100% Tier-2 PF excise if
// not corrected within taxable period; § 4945(b)(2) 50% Tier-2 manager
// excise (refused correction) capped at $20,000. Five categories of
// taxable expenditures per § 4945(d): (1) § 4945(d)(1) influencing
// legislation/lobbying with § 4945(e) exceptions (nonpartisan analysis
// + technical advice + self-defense + employee communications); (2)
// § 4945(d)(2) influencing elections / voter registration with
// § 4945(f) five-condition safe harbor (501(c)(3)/(509(a)(1)-(3)) +
// substantial-all income + 85%+ non-DP support + 5+ state nonpartisan
// + non-earmarked); (3) § 4945(d)(3) grants to individuals without
// § 4945(g) advance IRS approval; (4) § 4945(d)(4) grants to
// organizations not § 509(a)(1)/(2)/(3) or § 4942(j)(3) operating
// foundation without § 4945(h) expenditure responsibility four-prong
// (pre-grant inquiry + written grant agreement + grantee reports +
// IRS Form 990-PF reports); (5) § 4945(d)(5) non-charitable
// expenditures outside § 170(c)(2)(B) charitable purposes.
// Distinction from § 4944 (iter 476): § 4944 evaluates prudence of
// INVESTMENTS (asset side); § 4945 evaluates propriety of
// EXPENDITURES (program side). Distinction from § 4941 (iter 468):
// § 4941 punishes self-dealing TRANSACTIONS between PF and DP; § 4945
// punishes expenditures outside permitted charitable purposes
// regardless of recipient relationship. Original enactment Tax Reform
// Act of 1969 Pub. L. 91-172.

async fn section_4945_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4945::Section4945Input>,
) -> Result<Json<traderview_expense::section_4945::Section4945Result>, ApiError> {
    Ok(Json(traderview_expense::section_4945::check(&b)))
}

// ── § 4958 intermediate sanctions on excess benefit transactions ─────
// Mounted at /api/calc/section-4958. Pure compute; § 4958(a)(1)
// 25% excise tax on disqualified person who receives an excess
// benefit from a transaction with an applicable tax-exempt
// organization (ATEO); § 4958(b) additional 200% excise tax if
// not corrected within taxable period; § 4958(a)(2) 10% excise
// tax on knowing willful organization manager capped at $20K per
// transaction under § 4958(d)(2); § 4958(e) ATEO = § 501(c)(3)
// public charity (NOT private foundation — those use § 4941) +
// § 501(c)(4) social welfare + § 501(c)(29) qualified nonprofit
// health insurance issuer (added by ACA 2010) + 5-year look-back;
// § 4958(f)(1) disqualified person = substantial influence at
// any time during 5-year period including § 4958(f)(4) family +
// § 4958(f)(3) 35%-controlled entity; § 4958(f)(5) taxable
// period; § 4958(f)(6) correction via cash plus AFR interest;
// Treas. Reg. § 53.4958-6 rebuttable presumption of
// reasonableness three-prong safe harbor (advance approval by
// independent body + comparability data + contemporaneous
// documentation) shifts burden of proof to IRS; § 4961(b) 90-day
// post-assessment abatement for tier-2 200% tax; § 4958(c)(1)(A)
// excess benefit transaction = economic benefit exceeding
// consideration received including excessive compensation +
// bargain sale + above-market purchase + below-market loan +
// below-market rental + personal expense + automatic excess
// benefit per Treas. Reg. § 53.4958-4(c). Coordinate with § 4960
// (iter 464) ATEO 21% remuneration tax, not duplicative; original
// enactment Taxpayer Bill of Rights 2, Pub. L. 104-168 (July 30,
// 1996); PPA 2006 extended to donor-advised funds + supporting
// orgs.

async fn section_4958_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4958::Section4958Input>,
) -> Result<Json<traderview_expense::section_4958::Section4958Result>, ApiError> {
    Ok(Json(traderview_expense::section_4958::check(&b)))
}

// ── § 4960 excise tax on excess tax-exempt org executive comp ────────
// Mounted at /api/calc/section-4960. Pure compute; § 4960(a) imposes
// 21% excise tax on applicable tax-exempt organization (ATEO) for sum
// of (i) remuneration paid by ATEO to covered employee exceeding
// $1,000,000 in the taxable year, plus (ii) any excess parachute
// payment paid by ATEO to covered employee; § 4960(c)(1) ATEO
// definition (§ 501(a) exempt org + § 521(b)(1) farmers coop +
// § 115(1) state/political subdivision instrumentality + § 527(e)(1)
// political org); § 4960(c)(2) covered employee — PRE-OBBBA regime
// (2018-2025): five highest-compensated employees for taxable year
// OR any preceding year beginning after 12/31/2016 forever-covered
// rule; POST-OBBBA regime (after 12/31/2025 per One Big Beautiful
// Bill Act, Pub. L. 119-21, July 4, 2025): five-employee cap REMOVED,
// any current or former employee over $1M triggers tax; § 4960(c)(3)
// remuneration = § 3401(a) wages excluding designated Roth plus
// § 457(f) vested deferred comp, EXCLUDES medical services by
// licensed medical professional (doctor, nurse, veterinarian) per
// § 4960(c)(3)(B); § 4960(c)(5) excess parachute payment modeled on
// § 280G — aggregate payments contingent on SEPARATION FROM
// EMPLOYMENT with present value ≥ 3× base amount triggers tax on
// amount EXCEEDING 1× base amount; § 4960(c)(7) coordination with
// § 162(m) $1M deduction cap (publicly held corporations); Final
// Treasury Regulations 26 C.F.R. § 53.4960-0 through § 53.4960-6
// effective January 15, 2021; original enactment Tax Cuts and Jobs
// Act, Pub. L. 115-97 (Dec. 22, 2017).

async fn section_4960_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4960::Section4960Input>,
) -> Result<Json<traderview_expense::section_4960::Section4960Result>, ApiError> {
    Ok(Json(traderview_expense::section_4960::check(&b)))
}

// ── § 4975 prohibited transactions in IRA / qualified plans ──────────
// Mounted at /api/calc/section-4975. Pure compute; 15% standard +
// 100% non-correction excise tax on prohibited transactions between
// plan and disqualified person under 6 § 4975(c)(1) categories;
// § 4975(e)(2) 9-category disqualified person definition (including
// family § 4975(e)(6) = spouse + ancestor + lineal descendant +
// spouse of lineal descendant); § 408(e)(2) IRA disqualification
// triggers deemed distribution at FMV + § 72(t) 10% penalty if under
// 59½; § 4975(h) 90-day correction window for 100% tax abatement;
// DOL PTE 80-26 + 75-1 + 84-24 statutory exemptions.

async fn section_4975_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4975::Section4975Input>,
) -> Result<Json<traderview_expense::section_4975::Section4975Result>, ApiError> {
    Ok(Json(traderview_expense::section_4975::check(&b)))
}

// ── § 4978 tax on certain dispositions by ESOPs ──────────────────────
// Mounted at /api/calc/section-4978. Pure compute; § 4978(a) 10%
// excise tax on amount realized on disposition of qualified securities
// by ESOP or eligible worker-owned cooperative within 3 years after
// acquisition under § 1042 sale or § 664(g) qualified gratuitous
// transfer. Tax paid by EMPLOYER that maintains the plan. § 4978(b)
// triggering conditions (either): (1) § 4978(b)(1) share count test —
// total employer securities held by plan after disposition less than
// total held immediately after § 1042 sale; (2) § 4978(b)(2) 30%-value
// test — value of qualified securities held after disposition less
// than 30% of total employer securities value at disposition (60% for
// § 664(g) qualified gratuitous transfer). § 4978(c) exceptions:
// § 4978(c)(1) distribution on separation from service / death /
// retirement / disability / divorce; § 4978(c)(2) employee stock
// purchase; § 4978(c)(3) merger or reorganization under § 354 +
// § 355 + § 356 + § 368 with ESOP retaining successor securities;
// § 4978(c)(4) diversification rights under § 401(a)(28)(B).
// Companion to § 1042 (iter 480): § 1042 provides seller capital
// gain deferral; § 4978 is employer recapture if ESOP fails 3-year
// hold. § 1042(b)(3) written consent to § 4978 recapture is
// prerequisite to seller § 1042 election. Form 5330 filing deadline
// last day of 7th month after employer tax-year-end. Original
// enactment Deficit Reduction Act of 1984 Pub. L. 98-369.

async fn section_4978_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4978::Section4978Input>,
) -> Result<Json<traderview_expense::section_4978::Section4978Result>, ApiError> {
    Ok(Json(traderview_expense::section_4978::check(&b)))
}

// ── § 6166 estate tax installment for closely held business ──────────
// Mounted at /api/calc/section-6166. Pure compute; § 6166(a)(1)
// general rule allowing executor of estate where interest in closely
// held business exceeds 35% of adjusted gross estate to elect 14-year
// deferral (5 years interest-only + 10 years principal+interest);
// § 6166(a)(2) cross-references § 6166(b)(1) qualifying interests
// (sole proprietorship + partnership with 20%+ capital OR ≤45 partners
// + corporation with 20%+ voting stock OR ≤45 shareholders);
// § 6166(a)(3) election filing on timely Form 706 or amended return
// within 6 months of non-extended due date; § 6166(b)(6) adjusted
// gross estate = gross estate less § 2053 (debts/expenses/mortgages)
// and § 2054 (casualty losses) deductions; § 6166(f) 14-year deferral
// period; § 6601(j) subsidized 2% interest rate on first 2-percent
// portion ($1,830,000 multiplied by value of $1M ÷ applicable
// exclusion, 2024 indexed); § 6621 underpayment rate × 45% on excess;
// § 6166(g) acceleration events: § 6166(g)(1)(A) disposition of 50%+
// of decedent's interest + § 6166(g)(1)(B) withdrawal exceeding 50% +
// § 6166(g)(3) missed installment payment beyond 6-month grace +
// § 6166(g)(4) undistributed income must be applied to installments;
// § 303 stock redemption does NOT trigger acceleration. PV savings on
// $10M+ estate tax routinely exceed $2M-$3M. Distinct from § 6161
// general 12-month extension. Original enactment Tax Reform Act of
// 1976; amended Economic Growth and Tax Relief Reconciliation Act of
// 2001 Pub. L. 107-16.

async fn section_6166_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6166::Section6166Input>,
) -> Result<Json<traderview_expense::section_6166::Section6166Result>, ApiError> {
    Ok(Json(traderview_expense::section_6166::check(&b)))
}

// ── § 4980 Tax on Reversion of Qualified Plan Assets to Employer ─────
// Mounted at /api/calc/section-4980. Pure compute; § 4980(a) 20%
// base excise tax on amount of employer reversion from qualified
// retirement plan (defined benefit pension); § 4980(d)(1)
// increases rate to 50% unless employer satisfies § 4980(d)(2)
// qualified replacement plan (QRP) or § 4980(d)(3) pro rata
// benefit increase requirement; § 4980(d)(2) QRP three
// requirements (95% active participants + 25% direct transfer +
// 7-year ratable allocation); § 4980(d)(3) pro rata benefit
// increase 20%+ of maximum reversion with immediate effect on
// plan termination; § 4980(c) employer reversion = cash or FMV
// received as result of plan termination (excludes non-§ 404
// deductible contributions); § 4980(d)(4) qualified participant
// definition; stacks with corporate income tax for 70-75%
// effective combined rate at 50% rate or 45-50% at 20% rate; Rev.
// Rul. 2003-85 + PLR 9701036 confirm DB-to-DC transfer
// preferential treatment.

async fn section_4980_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4980::Section4980Input>,
) -> Result<Json<traderview_expense::section_4980::Section4980Result>, ApiError> {
    Ok(Json(traderview_expense::section_4980::check(&b)))
}

// ── § 4980H Employer Shared Responsibility Payment (ESRP / ACA) ──────
// Mounted at /api/calc/section-4980h. Pure compute; 2026 amounts:
// § 4980H(a) failure-to-offer-MEC penalty $3,340 per FT minus 30
// (monthly $278.33); § 4980H(b) unaffordable or non-MV penalty
// $5,010 per FT receiving PTC (monthly $417.50); § 4980H(c)(2)
// ALE = 50+ FT employees (30+ hours/week) including § 4980H(c)(2)
// (E) FTE-equivalents; § 4980H(c)(2)(D) seasonal worker 120-day-
// or-fewer exception; § 4980H(c)(4) affordability 9.96% of
// household income (2026, up from 9.02% for 2025); § 36B(c)(2)(C)
// (ii) minimum value 60% of expected healthcare costs; § 4980H(d)
// MEC definition under § 5000A(f); § 6056 Form 1094-C transmittal
// + Form 1095-C employee statement by January 31; § 6721/§ 6722
// civil penalties up to $310 per return; § 414(b)/(c)/(m)/(o)
// controlled group aggregation for ALE threshold determination
// + each entity separately assessed ESRP; Pub. L. 111-148 PPACA
// + Pub. L. 111-152 HCERA enacting authority.

async fn section_4980h_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4980h::Section4980hInput>,
) -> Result<Json<traderview_expense::section_4980h::Section4980hResult>, ApiError> {
    Ok(Json(traderview_expense::section_4980h::check(&b)))
}

// ── §453 installment sale gain deferral ──────────────────────────────
// Mounted at /api/calc/section-453. Pure compute; gross profit ratio
// applied to each year's principal payment with §453(k) marketable
// securities exclusion + §453(g) related-party 2-year resale anti-
// abuse + §453(d) opt-out election.

async fn section_453_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_453::Section453Input>,
) -> Result<Json<traderview_expense::section_453::Section453Result>, ApiError> {
    if b.sale_price < Decimal::ZERO
        || b.selling_costs < Decimal::ZERO
        || b.adjusted_basis < Decimal::ZERO
        || b.principal_received_this_year < Decimal::ZERO
        || b.interest_received_this_year < Decimal::ZERO
        || b.unrecognized_gain_remaining < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_453::compute(&b)))
}

// ── §453A nondealer installment interest charge ─────────────────────
// Mounted at /api/calc/section-453a. Pairs with §453: imposes a
// non-deductible interest charge on the deferred tax liability of
// large installment obligations exceeding $5M aggregate face at
// year-end. Per-sale floor $150k + non-dealer + non-personal-use +
// non-residential-lots/timeshares. Interest = applicable % ×
// deferred tax × §6621 underpayment rate (short-term AFR + 3 pp).

async fn section_453a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_453a::Section453aInput>,
) -> Result<Json<traderview_expense::section_453a::Section453aResult>, ApiError> {
    if b.sales_price_dollars < 0
        || b.aggregate_year_end_face_obligations_dollars < 0
        || b.maximum_applicable_tax_rate_bp > 10_000
        || b.underpayment_rate_bp > 10_000
    {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs and rates ≤ 100% (10,000bp) required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_453a::compute(&b)))
}

// ── § 461(g) Prepaid Interest Deduction Timing ───────────────────────
// Mounted at /api/calc/section-461g. Pure compute; § 461(g)(1)
// cash-basis taxpayer must treat prepaid interest like accrual-
// basis (interest allocable to period after close of taxable year
// CHARGED TO CAPITAL ACCOUNT and deducted in period properly
// allocable); § 461(g)(2) EXCEPTION for points on principal
// residence purchase or improvement (5 conditions: principal
// residence purchase/improvement + secured by residence +
// established practice in area + not excessive + percentage of
// principal); Rev. Rul. 87-22 refinancing exclusion (points
// amortized over loan life); Rev. Rul. 70-540 rental property
// straight-line amortization; Rev. Proc. 94-27 seller-paid
// points treated as buyer-paid; interaction with § 163(d)
// investment interest (margin loan) + § 163(j) business
// interest + § 163(h) qualified residence interest + § 475(f)
// trader mark-to-market reclassification + § 263A UNICAP
// capitalization for construction-period interest.

async fn section_461g_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_461g::Section461gInput>,
) -> Result<Json<traderview_expense::section_461g::Section461gResult>, ApiError> {
    Ok(Json(traderview_expense::section_461g::check(&b)))
}

// ── § 461(h) economic performance rule ──────────────────────────
// Mounted at /api/calc/section-461h. Enacted by DRA 1984 (Pub. L.
// 98-369). § 461(h)(1) economic performance general rule. § 461(h)
// (2) when economic performance occurs: (A) services/property
// provided to taxpayer = as provided; (B) services/property
// provided by taxpayer = as taxpayer provides; (C) workers comp/
// tort/breach/violation of law = as payments made; (D) other
// liabilities = at time per regulations. § 461(h)(3) recurring
// item exception: 8.5 months after close of taxable year + all-
// events test + recurring + immaterial-or-better-matching. § 461(h)
// (4) reserves for estimated expenses prohibited. Three-prong all-
// events test per Treas. Reg. § 1.461-1(a)(2): fact of liability +
// determinable amount + economic performance. Sibling cluster:
// § 446(c)(2) accrual method, § 461(g) prepaid interest timing,
// § 461(l) excess business loss limitation, § 162 ordinary and
// necessary business expense, § 165(d) wagering losses limit.

async fn section_461h_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_461h::Section461hInput>,
) -> Result<Json<traderview_expense::section_461h::Section461hOutput>, ApiError> {
    Ok(Json(traderview_expense::section_461h::check(&b)))
}

// ── § 457(b) Governmental and Tax-Exempt Deferred Compensation ───────
// Mounted at /api/calc/section-457b. Pure compute; 2026 limits:
// elective deferral § 457(b)(2) $24,500; age-50 catch-up § 414(v)
// $8,000 (GOVERNMENTAL ONLY); ages-60-63 SECURE 2.0 § 109 enhanced
// catch-up $11,250 (GOVERNMENTAL ONLY); § 457(b)(3) special 3-year
// pre-retirement catch-up = lesser of 2× annual limit ($49,000) or
// underutilized prior-year limitation (BOTH governmental + tax-
// exempt). Two plan types: GOVERNMENTAL (§ 457(g) trust, no § 72(t)
// 10% penalty, rollovers permitted) vs TAX-EXEMPT (unfunded top-
// hat, substantial credit risk in employer bankruptcy, § 72(t)
// applies, rollovers not permitted). § 402(g)(1) NON-AGGREGATION
// rule allows DOUBLE DEFERRAL with § 401(k)/§ 403(b) ($49,000 in
// 2026). § 457(b)(3) ANTI-STACKING with § 414(v) catch-up
// (participant must choose ONE catch-up mechanism per year).

async fn section_457b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_457b::Section457bInput>,
) -> Result<Json<traderview_expense::section_457b::Section457bResult>, ApiError> {
    Ok(Json(traderview_expense::section_457b::check(&b)))
}

// ── § 457A Nonqualified Deferred Compensation From Tax Indifferent Parties ──
// Mounted at /api/calc/section-457a (iter 494). Pure compute. EESA 2008
// anti-deferral provision targeting US hedge-fund / PE managers operating
// through Cayman / BVI master-feeder structures. Computes (1) nonqualified-
// entity classification per § 457A(b) (foreign corp w/o comprehensive
// foreign tax or ECI; partnership w/ substantially-all tax-indifferent
// allocations); (2) substantial-risk-of-forfeiture test under § 457A(d)(2)
// (future-performance-of-substantial-services standard, stricter than §
// 409A); (3) immediate-inclusion when SROF absent OR amount-not-determinable
// 20% additional tax + AFR + 1% interest under § 457A(c)(1)(B)(i)–(ii);
// (4) pre-2009 transition rule per § 457A(e) (must include by last tax year
// before 2017 per Notice 2009-8); (5) § 409A-exempt stock-right safe harbor
// per Notice 2009-8 Q&A 2 + Rev. Rul. 2014-18. Coordinates with § 409A
// (applies IN ADDITION per § 457A(d)(4)), § 457(b) (governmental NQDC), §
// 280G (golden parachutes on change in control).

async fn section_457a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_457a::Section457aInput>,
) -> Result<Json<traderview_expense::section_457a::Section457aResult>, ApiError> {
    Ok(Json(traderview_expense::section_457a::check(&b)))
}

// ── §168(e)(6) Qualified Improvement Property ────────────────────────
// Mounted at /api/calc/section-168-e6. Pure compute; interior
// improvements to nonresidential buildings qualify as 15-year QIP +
// §168(k) bonus eligible. Excluded types (enlargement, elevator,
// internal structural framework) fall to 39-year nonresidential.

async fn section_168_e6_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_168_e6::Section168E6Input>,
) -> Result<Json<traderview_expense::section_168_e6::Section168E6Result>, ApiError> {
    if b.improvement_cost < Decimal::ZERO {
        return Err(ApiError::BadRequest("improvement_cost must be >= 0".into()));
    }
    Ok(Json(traderview_expense::section_168_e6::compute(&b)))
}

// ── §263A UNICAP (trader vs dealer classifier) ───────────────────────
// Mounted at /api/calc/section-263a. Pure compute; dealers must
// capitalize direct + indirect inventory costs; traders + investors
// are exempt. §263A(b)(2)(B) small business exception per §448(c)
// threshold.

// ── § 263 capital expenditures general rule ─────────────────────
// Mounted at /api/calc/section-263. § 263(a)(1)(A) general
// capitalization rule for buildings + permanent improvements +
// betterments. § 263(a)(1)(B) restoration capitalization.
// Tangible Property Regulations (T.D. 9636, 2013) finalized BAR
// test: Betterment (§ 1.263(a)-3(j)) + Adaptation (§ 1.263(a)-3
// (l)) + Restoration (§ 1.263(a)-3(k)). Three safe harbors:
// § 1.263(a)-1(f) de minimis ($5,000 AFS / $2,500 no AFS per
// invoice); § 1.263(a)-3(h) small taxpayer (lesser of 2% basis
// or $10,000 aggregate); § 1.263(a)-3(i) routine maintenance
// (10 years buildings / 3 years non-buildings). Sibling cluster:
// § 263A (UNICAP inventory capitalization), § 263(g) (interest
// + carrying charges), § 162 (ordinary & necessary expense),
// § 168 (MACRS depreciation of capitalized amounts), § 461(h)
// (economic performance timing).

async fn section_263_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_263::Section263Input>,
) -> Result<Json<traderview_expense::section_263::Section263Output>, ApiError> {
    Ok(Json(traderview_expense::section_263::check(&b)))
}

async fn section_263a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_263a::Section263AInput>,
) -> Result<Json<traderview_expense::section_263a::Section263AResult>, ApiError> {
    if b.direct_costs < Decimal::ZERO
        || b.indirect_costs_allocable_to_inventory < Decimal::ZERO
        || b.avg_3yr_gross_receipts < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_263a::compute(&b)))
}

// ── §174 R&D capitalization (post-TCJA) ──────────────────────────────
// Mounted at /api/calc/section-174. Pure compute; capitalizes R&D
// expenditures and amortizes over 5y domestic / 15y foreign with
// half-year convention. Hit algorithmic traders writing internal
// trading software starting in 2022.

async fn section_174_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_174::Section174Input>,
) -> Result<Json<traderview_expense::section_174::Section174Result>, ApiError> {
    if b.r_and_d_amount < Decimal::ZERO {
        return Err(ApiError::BadRequest("r_and_d_amount must be >= 0".into()));
    }
    Ok(Json(traderview_expense::section_174::compute(&b)))
}

// ── §179 election to expense certain depreciable business assets ─────
// Mounted at /api/calc/section-179. §179(b)(1) dollar cap (2026 =
// $2,560,000); §179(b)(2) phaseout dollar-for-dollar above threshold
// (2026 = $4,090,000); §179(b)(3)(A) taxable-income limitation with
// §179(b)(3)(B) indefinite carryforward; §179(b)(5) heavy-SUV sublimit
// (GVWR 6,001-14,000 lb. — 2026 = $32,000) with excess flowing to §168(k)
// 100% bonus depreciation made permanent by OBBBA §70302 (eff. 2025-01-01).
// Out of scope: §179(d)(3) related-party purchase restriction; §179(d)(10)
// recapture on business-use percentage drop below 50%; §179(f) qualified
// real-property carve-in (roofs, HVAC, fire alarm, security systems).

async fn section_179_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_179::Section179Input>,
) -> Result<Json<traderview_expense::section_179::Section179Result>, ApiError> {
    if b.qualifying_property_cents < 0
        || b.suv_property_cents < 0
        || b.dollar_cap_cents < 0
        || b.phaseout_threshold_cents < 0
        || b.suv_sublimit_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents values required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_179::compute(&b)))
}

// ── §179D Energy Efficient Commercial Buildings Deduction ──────────
// Mounted at /api/calc/section-179d. Added by Section 1331 of the
// Energy Policy Act of 2005 (Public Law 109-58, 119 Stat. 594),
// signed by President George W. Bush on August 8, 2005, effective
// for property placed in service after December 31, 2005.
// Substantially amended by Section 13303 of the Inflation Reduction
// Act of 2022 (Public Law 117-169, 136 Stat. 1818), signed by
// President Biden on August 16, 2022. Energy reduction threshold
// reduced from 50 % to 25 %; base deduction $0.50-$1.00/sq ft;
// bonus deduction $2.50-$5.00/sq ft with prevailing wage +
// apprenticeship; three eligible building systems (interior
// lighting / HVAC and hot water / building envelope); ASHRAE 90.1
// reference standard; designer allocation from government / tax-
// exempt entity under § 179D(d)(2). Inflation-adjusted TY 2025 max
// = $5.81/sq ft. TERMINATED by One Big Beautiful Bill Act of 2025
// (Public Law 119-21, 139 Stat. 72, signed July 4, 2025) for
// buildings whose construction begins AFTER June 30, 2026.
async fn section_179d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_179d::Section179DInput>,
) -> Result<Json<traderview_expense::section_179d::Section179DResult>, ApiError> {
    Ok(Json(traderview_expense::section_179d::check(&b)))
}

// ── §183 hobby loss rules ────────────────────────────────────────────
// Mounted at /api/calc/section-183. §183(a) general rule; §183(b)(1)
// always-allowed deductions (taxes, interest); §183(b)(2) capped at
// gross income − (b)(1) — effectively ZERO post-TCJA via §67(g)
// misc-itemized-deduction suspension made permanent by OBBBA 2025;
// §183(d) profit-motive presumption (3-of-5 standard / 2-of-7 horse);
// §183(e) deferral election; Reg. § 1.183-2(b) 9-factor backup test.

async fn section_183_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_183::Section183Input>,
) -> Result<Json<traderview_expense::section_183::Section183Result>, ApiError> {
    if b.nine_factors_favoring_profit > 9
        || b.gross_income_from_activity_dollars < 0
        || b.section_183b1_deductions_dollars < 0
        || b.other_activity_deductions_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "nine_factors_favoring_profit must be ≤ 9 and dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_183::compute(&b)))
}

// ── §1234 options character + holding-period rules ──────────────────
// Mounted at /api/calc/section-1234. Pure compute; §1234(a) holder
// character mirrors underlying with option holding period driving
// ST/LT; §1234(b) writer is fixed short-term capital regardless of
// holding period; §1234(c) §1256 contract override; exercise +
// assignment are basis-adjustment events with no separate option
// gain/loss.

async fn section_1234_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1234::Section1234Input>,
) -> Result<Json<traderview_expense::section_1234::Section1234Result>, ApiError> {
    if b.option_close_date < b.option_open_date {
        return Err(ApiError::BadRequest(
            "option_close_date must be on or after option_open_date".into(),
        ));
    }
    if b.premium < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "premium must be >= 0 (sign is implicit from role: writer = received, holder = paid)"
                .into(),
        ));
    }
    if b.close_proceeds_or_cost < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "close_proceeds_or_cost must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1234::compute(&b)))
}

// ── §1231 quasi-capital gain/loss with §1231(c) recapture ──────────
// Mounted at /api/calc/section-1231. §1231(a)(1) net gain → LTCG;
// §1231(a)(2) net loss → ordinary; §1231(b) property = real /
// depreciable held > 1 year used in trade/business; §1231(c) 5-year
// lookback recaptures current net gain as ordinary up to prior-5-yr
// nonrecaptured net §1231 losses.

async fn section_1231_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1231::Section1231Input>,
) -> Result<Json<traderview_expense::section_1231::Section1231Result>, ApiError> {
    if b.current_year_gains_dollars < 0 || b.current_year_losses_dollars < 0 {
        return Err(ApiError::BadRequest(
            "current_year_gains_dollars and current_year_losses_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1231::compute(&b)))
}

// ── §1233 short-sale character + holding-period rules ───────────────
// Mounted at /api/calc/section-1233. Pure compute; §1233(b) gain
// short-term + holding-period reset for short-held or during-short
// substantially identical; §1233(d) loss long-term for long-held
// substantially identical at short open.

async fn section_1233_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1233::Section1233Input>,
) -> Result<Json<traderview_expense::section_1233::Section1233Result>, ApiError> {
    if b.short_shares < 0 {
        return Err(ApiError::BadRequest("short_shares must be >= 0".into()));
    }
    if b.short_close_date < b.short_sale_date {
        return Err(ApiError::BadRequest(
            "short_close_date must be on or after short_sale_date".into(),
        ));
    }
    for p in b
        .substantially_identical_held_at_open
        .iter()
        .chain(b.substantially_identical_acquired_during_short.iter())
    {
        if p.shares < 0 {
            return Err(ApiError::BadRequest(
                "long position shares must be >= 0".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_1233::compute(&b)))
}

// ── §83(b) restricted-stock election ─────────────────────────────────
// Mounted at /api/calc/section-83b. Validates 30-day deadline,
// computes ordinary income with vs without election, LTCG holding-
// period start (grant vs vesting), capital gain at sale,
// §83(b)(2) forfeiture trap with no refund.

async fn section_83b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_83b::Section83bInput>,
) -> Result<Json<traderview_expense::section_83b::Section83bResult>, ApiError> {
    if b.vesting_date < b.grant_date {
        return Err(ApiError::BadRequest(
            "vesting_date must be on or after grant_date".into(),
        ));
    }
    if b.fmv_at_grant < Decimal::ZERO
        || b.amount_paid_at_grant < Decimal::ZERO
        || b.fmv_at_vesting < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "fmv_at_grant, amount_paid_at_grant, and fmv_at_vesting must be >= 0".into(),
        ));
    }
    if let Some(sp) = b.sale_price_per_share {
        if sp < Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "sale_price_per_share must be >= 0".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_83b::compute(&b)))
}

// ── §83(c) substantial risk of forfeiture timing rules ───────────
// Mounted at /api/calc/section-83c. §83(a) recognize on EARLIER of
// transferable or no-SRF; §83(c)(1) SRF requires future-performance
// of substantial services OR transfer-purpose condition + substantial
// possibility of forfeiture + likelihood of enforcement; §83(c)(2)
// transferability = transferee not subject to SRF; §83(c)(3) § 16(b)
// 6-month short-swing-profit restriction (treats property as SRF AND
// non-transferable until 6-month expiry or no-§16(b)-suit-on-profit);
// Treas. Reg. § 1.83-3(c) elaboration.

async fn section_83c_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_83c::Section83cInput>,
) -> Result<Json<traderview_expense::section_83c::Section83cResult>, ApiError> {
    if b.days_remaining_in_section_16b_period > 366 {
        return Err(ApiError::BadRequest(
            "days_remaining_in_section_16b_period must be <= 366".into(),
        ));
    }
    Ok(Json(traderview_expense::section_83c::compute(&b)))
}

// ── §172 Net Operating Loss deduction ────────────────────────────────
// Mounted at /api/calc/section-172. Three regimes by NOL year:
// pre-2018 legacy (2yr carryback / 20yr carryforward / no 80% limit),
// CARES Act 2018-2020 (5yr carryback / 100% offset), permanent TCJA
// post-2020 (no carryback / indefinite carryforward / 80% limit).
// §172(b)(1)(B) farming + insurance 2-year carryback exception.

async fn section_172_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_172::Section172Input>,
) -> Result<Json<traderview_expense::section_172::Section172Result>, ApiError> {
    if b.current_year_nol < Decimal::ZERO
        || b.current_year_taxable_income_before_nol < Decimal::ZERO
        || b.prior_year_nol_carryforward < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_172::compute(&b)))
}

// ── §195 startup expenditures — election to deduct $5k first-year ───
// (phased out dollar-for-dollar above $50k startup costs, fully phased
// at $55k) plus 180-month amortization of the remainder beginning with
// the month active trade or business begins. § 195(c)(1) excludes
// amounts deductible under §§ 163(a), 164, 174. § 195(d) automatic
// election per T.D. 9542 (Sept. 8, 2011) — caller passes
// `affirmative_capitalization_election = true` to opt OUT of the
// default deduction treatment. Trader-relevant for new TTS-elected
// LLCs and prop-trading entities organizing pre-launch operations.
async fn section_195_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_195::Section195Input>,
) -> Result<Json<traderview_expense::section_195::Section195Result>, ApiError> {
    if b.total_startup_expenditures_cents < -10_000_000_000
        || b.total_startup_expenditures_cents > 10_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "total_startup_expenditures_cents out of plausible range".into(),
        ));
    }
    if b.months_active_in_first_year > 12 {
        return Err(ApiError::BadRequest(
            "months_active_in_first_year must be 0..=12".into(),
        ));
    }
    Ok(Json(traderview_expense::section_195::compute(&b)))
}

// ── §248 corporate organizational expenditures ──────────────────────
// Mounted at /api/calc/section-248. Parallel to § 195 startup
// expenditures with corporation-specific terminology. § 248(a)
// election yields lesser of $5,000 first-year deduction OR ($5,000 -
// max(0, org_costs - $50,000)) phase-out + 180-month amortization of
// remainder beginning month corporation begins business. § 248(b)
// organizational expenditure defined; Treas. Reg. § 1.248-1(b)
// excludes expenses for issuing/selling shares + § 351 transfer
// expenses + § 368 reorganization expenses. § 248(c) automatic
// election deemed per T.D. 9542 (Sept. 8, 2011). AJCA 2004 § 902
// harmonized § 248 with § 195 / § 709 (cross-reference modules).
// Trader-relevant when forming a C-corporation for trading
// operations.

async fn section_248_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_248::Section248Input>,
) -> Result<Json<traderview_expense::section_248::Section248Result>, ApiError> {
    if b.total_organizational_expenditures_cents < -10_000_000_000
        || b.total_organizational_expenditures_cents > 10_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "total_organizational_expenditures_cents out of plausible range".into(),
        ));
    }
    if b.months_active_in_first_year > 12 {
        return Err(ApiError::BadRequest(
            "months_active_in_first_year must be 0..=12".into(),
        ));
    }
    Ok(Json(traderview_expense::section_248::compute(&b)))
}

// ── §709 partnership organizational expenditures + syndication ──────
// Mounted at /api/calc/section-709. Parallel to § 195 + § 248 with
// partnership-specific terminology. § 709(b)(1) $5K first-year
// deduction + $50K phase-out floor + $55K ceiling + 180-month
// amortization. § 709(b)(2) organizational expense defined. § 709(b)
// (3) SYNDICATION EXPENSES (brokerage + registration + legal/
// accounting fees for prospectus + printing) PERMANENTLY CAPITALIZED
// to partner basis with NO amortization — DISTINCT from § 248 which
// only excludes share-issuance expenses. Treas. Reg. § 1.709-2(a)
// organizational definition; § 1.709-2(b) syndication definition.
// T.D. 9542 (Sept. 8, 2011) automatic election. AJCA 2004 § 902
// harmonization with § 195 / § 248.

// ── § 707 partner-partnership transactions ──────────────────────
// Mounted at /api/calc/section-707. Four operative paragraphs:
// § 707(a) — partner-partnership transactions treated as between
// non-partners when partner not acting in capacity as partner;
// § 707(a)(2)(A) — payments to partner for services may be
// recharacterized as guaranteed payment or distributive share;
// § 707(a)(2)(B) — DISGUISED SALES: contribution + related
// distribution recast as sale. Treas. Reg. § 1.707-3(c)(1) creates
// 2-year presumption that distributions within 24 months are sales
// unless facts and circumstances clearly establish otherwise;
// § 1.707-3(d) creates opposite presumption for transfers > 2 years
// apart (presumed NOT sales). Two-prong test: but-for + entrepreneurial-
// risk independence. § 707(b) — losses disallowed between partner
// owning > 50% capital or profits and partnership, or between two
// partnerships with > 50% common owner. § 707(c) — guaranteed payments:
// payments to partner for services or use of capital DETERMINED
// WITHOUT REGARD TO partnership income; ordinary income to recipient
// + § 162 deduction to partnership. § 1402(a)(13) limited-partner
// SECA exclusion does NOT apply to GP for services — GP for capital
// to limited partner IS excluded from SE tax. Sibling cluster:
// § 704(b) (allocations), § 704(c) (built-in gain), § 704(d) (basis
// limit), § 705 (basis), § 721 (contribution nonrecognition), § 723
// (partnership basis in contributed property), § 731 (distribution
// nonrecognition), § 751 (hot assets), § 752 (liabilities), § 754
// (basis adjustment election).

async fn section_707_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_707::Section707Input>,
) -> Result<Json<traderview_expense::section_707::Section707Output>, ApiError> {
    Ok(Json(traderview_expense::section_707::check(&b)))
}

// ── § 706 partnership taxable year + varying interests ───────────
// Mounted at /api/calc/section-706. § 706(a): partner includes
// distributive share in year containing partnership year-end.
// § 706(b) three-tier hierarchy for partnership-year selection:
// (1) Majority Interest Test § 706(b)(1)(B)(i) — partners holding
// > 50% profits and capital with same year; (2) Principal Partner
// Test § 706(b)(1)(B)(ii) — all ≥ 5% partners with same year;
// (3) Least Aggregate Deferral Test § 706(b)(1)(B)(iii) — fallback
// with de minimis < 0.5 month exception. § 706(c)(1): partnership
// year does NOT close on partial interest change. § 706(c)(2):
// partnership year DOES close with respect to partner whose entire
// interest terminates (death, complete liquidation, complete sale/
// exchange). § 706(d) varying-interest allocations: interim closing
// method (default) vs. proration method (must be elected in writing
// per Treas. Reg. § 1.706-4(f)). Sibling cluster: § 707
// (partner-partnership transactions), § 731 (distribution rules),
// § 736 (retiring/deceased partner payment), § 743 (transferee
// basis adjustment — triggered by entire-interest termination
// event under § 706(c)(2)), § 751 (hot assets — character analysis
// on entire-interest sale).

async fn section_706_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_706::Section706Input>,
) -> Result<Json<traderview_expense::section_706::Section706Output>, ApiError> {
    Ok(Json(traderview_expense::section_706::check(&b)))
}

async fn section_709_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_709::Section709Input>,
) -> Result<Json<traderview_expense::section_709::Section709Result>, ApiError> {
    if b.total_organizational_expenses_cents < -10_000_000_000
        || b.total_organizational_expenses_cents > 10_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "total_organizational_expenses_cents out of plausible range".into(),
        ));
    }
    if b.total_syndication_expenses_cents < -10_000_000_000
        || b.total_syndication_expenses_cents > 10_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "total_syndication_expenses_cents out of plausible range".into(),
        ));
    }
    if b.months_active_in_first_year > 12 {
        return Err(ApiError::BadRequest(
            "months_active_in_first_year must be 0..=12".into(),
        ));
    }
    Ok(Json(traderview_expense::section_709::compute(&b)))
}

// ── §197 amortization of goodwill and certain other intangibles ─────
// Mounted at /api/calc/section-197. § 197(a) 15-year (180-month)
// straight-line amortization beginning month acquired for any
// "amortizable section 197 intangible" — § 197(d) nine categories
// (goodwill, going concern value, workforce in place, books and
// records, patent/copyright/process, customer or supplier intangibles,
// government license, covenant not to compete, franchise/trademark/
// trade name). § 197(e) three exceptions covered (land, financial
// interest, lease of tangible property). § 197(c) requires post-
// August-10-1993 acquisition + trade-or-business use. § 197(f)(9)
// anti-churning bars amortization when intangible held during
// 7/25/1991-8/10/1993 transition by taxpayer or related (>20%) party,
// OR acquired from related party with continued use. § 197(b) bars
// any § 167 depreciation deduction. Trader-relevant when acquiring a
// trading business (customer list, workforce, goodwill, non-compete
// with seller).

async fn section_197_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_197::Section197Input>,
) -> Result<Json<traderview_expense::section_197::Section197Result>, ApiError> {
    if b.adjusted_basis_cents < -10_000_000_000 || b.adjusted_basis_cents > 10_000_000_000_000 {
        return Err(ApiError::BadRequest(
            "adjusted_basis_cents out of plausible range".into(),
        ));
    }
    if b.months_held_since_acquisition > 100_000 {
        return Err(ApiError::BadRequest(
            "months_held_since_acquisition looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_197::compute(&b)))
}

// ── §199A Qualified Business Income (QBI) deduction ─────────────────
// Mounted at /api/calc/section-199a. § 199A(a) basic 20% deduction =
// LESSER of (1) 20% × QBI (combined QBI amount) or (2) 20% × (Taxable
// Income − Net Capital Gain). § 199A(b)(2) W-2 wage / UBIA phase-in
// limitation applies when TI exceeds threshold: limits 20% × QBI to
// GREATER of (a) 50% × W-2 wages or (b) 25% × W-2 wages + 2.5% ×
// UBIA. § 199A(e)(2) 2026 thresholds: Single / HoH $201,750 phase-in
// begin / $276,750 phase-out complete; MFJ / QSS $403,500 / $553,500.
// § 199A(d)(2) SSTB phases out completely above upper threshold.
// OBBBA 2025 (Pub. L. 119-21) made § 199A PERMANENT, expanded phase-
// in window from $50K → $75K single / $100K → $150K joint, and added
// $400 minimum deduction when QBI ≥ $1,000 + material participation.
// Rev. Proc. 2019-38 — rental real estate safe harbor (250+ hours/yr
// + separate books + contemporaneous records) treats rental as trade
// or business for § 199A. Trader-critical for traders with § 475(f)
// MTM election + trader-landlords with rental real estate qualifying
// as trade or business. IRS Form 8995 / 8995-A.

async fn section_199a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_199a::Section199AInput>,
) -> Result<Json<traderview_expense::section_199a::Section199AResult>, ApiError> {
    Ok(Json(traderview_expense::section_199a::check(&b)))
}

// ── §170(e) charitable contribution of appreciated property ─────────
// Mounted at /api/calc/section-170e. Six rule paths cover LTCG-public
// FMV (30% AGI), basis election (50% AGI), QAS to private foundation
// (FMV, 20% AGI), private-foundation reduction (basis, 20% AGI),
// STCG/ordinary reduction (basis, 50% public / 30% private), and
// tangible unrelated use (basis). §170(d) 5-year carryforward.

async fn section_170e_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_170e::Section170eInput>,
) -> Result<Json<traderview_expense::section_170e::Section170eResult>, ApiError> {
    if b.fmv < Decimal::ZERO || b.basis < Decimal::ZERO || b.agi < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "fmv, basis, and agi must be >= 0".into(),
        ));
    }
    if b.prior_year_carryover < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "prior_year_carryover must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_170e::compute(&b)))
}

// ── §72(t) 10% additional tax on early retirement distributions ─────
// Mounted at /api/calc/section-72t. §72(t)(1) 10% additional tax on
// includible portion of pre-age-59½ distributions; §72(t)(2) ~14
// exceptions including age 59½, death, disability, SEPP, separation
// after 55, medical > 7.5% AGI, QDRO, higher education, first-time
// homebuyer $10k IRA-only, unemployed health, §72(t)(11) federally
// declared disaster $22k, birth/adoption $5k, SECURE 2.0 §326
// terminal illness, §115 emergency personal expense $1k (plan-
// optional), §314 domestic abuse $10k (plan-optional), §334 long-
// term care $2.5k eff. 2026 (plan-optional).

async fn section_72t_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_72t::Section72tInput>,
) -> Result<Json<traderview_expense::section_72t::Section72tResult>, ApiError> {
    if b.distribution_amount_dollars < 0 || b.includible_in_gross_income_dollars < 0 {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_72t::compute(&b)))
}

// ── § 7345 passport revocation for seriously delinquent tax debt ────
// Mounted at /api/calc/section-7345. FAST Act § 32101 (Pub. L.
// 114-94, December 4, 2015) authorizes IRS to certify "seriously
// delinquent tax debt" to State Department, which then denies,
// revokes, or limits passports. § 7345(b)(1) threshold: debt
// exceeding inflation-adjusted amount ($66,000 for 2025; $50K
// originally in 2015) including penalties + interest, with EITHER
// (A) lien filed + § 6320 administrative remedies exhausted OR
// (B) § 6331 levy issued. § 7345(b)(2) exclusions: installment
// agreement (§ 6159), offer in compromise (§ 7122), innocent
// spouse claim (§ 6015), CDP hearing pending (§ 6320/§ 6330),
// bankruptcy, identity theft, disaster area, currently-not-
// collectible status. § 7345(c) 30-day reversal notification.
// § 7345(e) judicial review in Tax Court OR District Court.
// Sibling cluster: § 6011 + § 6651 + § 6654 + § 6662 + § 6707A.

async fn section_7345_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7345::Section7345Input>,
) -> Result<Json<traderview_expense::section_7345::Section7345Result>, ApiError> {
    if b.assessed_tax_debt_cents < 0 || b.assessed_tax_debt_cents > 100_000_000_000 {
        return Err(ApiError::BadRequest(
            "assessed_tax_debt_cents out of range".into(),
        ));
    }
    if b.annual_threshold_cents < 0 || b.annual_threshold_cents > 100_000_000_000 {
        return Err(ApiError::BadRequest(
            "annual_threshold_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_7345::compute(&b)))
}

// ── §7623 IRS whistleblower awards (Tax Relief 2006 / BBA 2018 / TFA 2019) ─
// Mounted at /api/calc/section-7623. § 7623 framework spans 1867
// discretionary regime (§ 7623(a)) + 2006 Tax Relief and Health Care
// Act § 406 mandatory 15-30% regime (§ 7623(b)) + 2018 Bipartisan
// Budget Act § 41108 broadened "collected proceeds" definition
// (§ 7623(c)) + 2019 Taxpayer First Act § 1405 anti-retaliation
// protections (§ 7623(d)). Mandatory thresholds (§ 7623(b)(5)):
// amount in dispute > $2,000,000 AND if individual, gross income
// > $200,000. Public-information-based awards capped at 10%
// (§ 7623(b)(2)(A)); planned/initiated noncompliance reduces award
// (§ 7623(b)(2)(B)); criminal conviction arising from role denies
// award entirely (§ 7623(b)(3)). § 7623(b)(4) Tax Court appeal
// within 30 days. § 7623(c) "collected proceeds" includes criminal
// fines, civil forfeitures, and FBAR penalties under 31 USC § 5321.
// § 7623(d) remedies: reinstatement, DOUBLE back pay with interest,
// special damages, attorney fees. Trader-relevant: wealthy/
// sophisticated traders are precisely the IRS Whistleblower Office
// target taxpayer class — high gross income + complex tax positions
// makes them disproportionately exposed to whistleblower tips from
// disgruntled fund employees, ex-spouses, business partners, or
// accountants. Sibling cluster: § 6663 + § 7201 + § 7202 + § 7206 +
// § 6701 + § 7430 + § 6038D + § 6111 + § 6112.

async fn section_7623_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7623::Section7623Input>,
) -> Result<Json<traderview_expense::section_7623::Section7623Result>, ApiError> {
    if b.award_percentage_bps > 10_000 {
        return Err(ApiError::BadRequest(
            "award_percentage_bps must be ≤ 10000 (100%)".into(),
        ));
    }
    if b.days_to_tax_court_appeal > 36_500 {
        return Err(ApiError::BadRequest(
            "days_to_tax_court_appeal out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_7623::check(&b)))
}

// ── §7405 IRS action for recovery of erroneous refunds ─────────────
// Mounted at /api/calc/section-7405. § 7405 is the IRS-side reverse
// mechanism to § 7422 (taxpayer-initiated refund suit). § 7405(a)
// recovers refunds erroneous within meaning of § 6514; § 7405(b)
// reaches refunds outside § 6514 scope. § 7405(d) statute of
// limitations — 2 years (730 days) from making of refund standard;
// 5 years (1825 days) if refund induced by fraud or misrepresentation
// of material fact. IRS burden of proof per IRM 5.17.4 + case law:
// (1) refund was erroneous; (2) amount of refund; (3) taxpayer
// received or benefited. Jurisdiction: district court (concurrent
// with Court of Federal Claims under 28 USC § 1346(a)(1)). Trader-
// relevant when IRS issues refund (e.g., NOL carryback via § 475(f)
// MTM election) and later determines computation was erroneous.

async fn section_7405_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7405::Section7405Input>,
) -> Result<Json<traderview_expense::section_7405::Section7405Result>, ApiError> {
    Ok(Json(traderview_expense::section_7405::check(&b)))
}

// ── § 7408 injunction remedy for preparer/promoter conduct ──────────
// Mounted at /api/calc/section-7408. Completes the preparer +
// promoter enforcement cluster: § 6694 + § 6695 + § 6700 + § 6701
// + § 7408. § 7408 is the EQUITABLE INJUNCTION remedy IRS uses
// to STOP ongoing promoter/aider conduct (not just penalize past
// conduct). Two-prong test under § 7408(b): (1) person engaged
// in specified conduct (§ 6700/§ 6701/§ 6707/§ 6708/Circular 230)
// AND (2) injunction appropriate to prevent recurrence. Action
// commenced at request of Secretary. § 7408(d) venue: district
// court for person's residence, principal place of business, OR
// district where conduct occurred. § 7408(e) treats non-resident
// U.S. citizens/residents as residing in D.C. § 7402(a)
// jurisdiction independent of any other government action.

async fn section_7408_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7408::Section7408Input>,
) -> Result<Json<traderview_expense::section_7408::Section7408Result>, ApiError> {
    Ok(Json(traderview_expense::section_7408::compute(&b)))
}

// ── §7701 entity classification check-the-box ───────────────────────
// Mounted at /api/calc/section-7701. Treas. Reg. § 301.7701-2 default
// classifications (single-member → disregarded entity; multi-member →
// partnership; per-se corporation via federal/state statute or
// § 301.7701-2(b)(8) foreign list); § 301.7701-3 Form 8832 election;
// § 301.7701-3(c)(1)(iv) 60-month lockout after change (waived by
// > 50% ownership change). CTB regs effective 1997-01-01.

async fn section_7701_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7701::Section7701Input>,
) -> Result<Json<traderview_expense::section_7701::Section7701Result>, ApiError> {
    Ok(Json(traderview_expense::section_7701::compute(&b)))
}

// ── §7872 below-market loans ─────────────────────────────────────────
// Mounted at /api/calc/section-7872. Pure compute; AFR imputation
// for below-market loans; §7872(c)(2)(A) $10k de minimis (gift loans
// only, no income-producing assets); §7872(d)(1) $100k NII cap with
// $1k floor; full AFR imputation otherwise.

async fn section_7872_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7872::Section7872Input>,
) -> Result<Json<traderview_expense::section_7872::Section7872Result>, ApiError> {
    if b.loan_principal < Decimal::ZERO
        || b.loan_term_years < Decimal::ZERO
        || b.actual_interest_rate < Decimal::ZERO
        || b.applicable_federal_rate < Decimal::ZERO
        || b.aggregate_outstanding_between_parties < Decimal::ZERO
        || b.borrower_net_investment_income < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar/rate/term inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_7872::compute(&b)))
}

// ── §1041 transfers between spouses ──────────────────────────────────
// Mounted at /api/calc/section-1041. Pure compute; §1041(a) no
// gain/loss; §1041(b) carryover basis (no dual basis); §1041(c)
// timing rules (1-year automatic / 1-6 year with instrument / 6+
// year with instrument); §1041(d) NR alien disqualification;
// §1223(2) holding period tacking.

async fn section_1041_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1041::Section1041Input>,
) -> Result<Json<traderview_expense::section_1041::Section1041Result>, ApiError> {
    if b.transferor_adjusted_basis < Decimal::ZERO
        || b.fmv_at_transfer < Decimal::ZERO
        || b.sale_price < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "transferor_adjusted_basis, fmv_at_transfer, and sale_price must be >= 0".into(),
        ));
    }
    if b.sale_date < b.transfer_date {
        return Err(ApiError::BadRequest(
            "sale_date must be on or after transfer_date".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1041::compute(&b)))
}

// ── § 1042 sales of stock to ESOPs or certain cooperatives ───────────
// Mounted at /api/calc/section-1042. Pure compute; § 1042(a) long-term
// capital gain on sale of qualified securities of domestic C
// corporation to ESOP is RECOGNIZED ONLY TO THE EXTENT amount realized
// exceeds cost of qualified replacement property (QRP) purchased
// during 15-month replacement period (3 months before sale + 12 months
// after, per § 1042(c)(6)). Five eligibility requirements per
// § 1042(b): (1) § 1042(b)(1) 3-year seller holding period; (2)
// § 1042(b)(2) ESOP must own 30%+ of each class of outstanding stock
// immediately after sale; (3) § 1042(b)(3) written consent to § 4978
// recapture (10% excise on employer if ESOP disposes within 3 years);
// (4) § 1042(b)(4) corporation must be DOMESTIC C CORP (S corps NOT
// eligible); (5) § 1042(c)(1)(B) qualified securities — not received
// via § 83 compensation / § 422 ISO / § 423 ESPP exercise, not
// readily tradable on established securities market. § 1042(c)(3)
// QRP categories: common stock + preferred + bonds + convertible
// floating-rate notes of domestic operating corporations; EXCLUDED
// are US government securities, non-US securities, domestic
// subsidiaries of non-US parents, FDIC CDs, mutual funds + money-
// market funds, and securities of the ESOP corporation. § 1042(d)
// basis = QRP cost reduced by non-recognized gain. § 1042(e)
// disposition recapture. § 1014 basis step-up at death permanently
// eliminates deferred gain — making § 1042 + estate planning among
// the most powerful trader-founder wealth-transfer strategies.
// Distinction from § 1031 like-kind exchange (real property only) and
// § 1045 QSBS rollover (60-day window). Original enactment Tax Reform
// Act of 1984 Pub. L. 98-369.

async fn section_1042_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1042::Section1042Input>,
) -> Result<Json<traderview_expense::section_1042::Section1042Result>, ApiError> {
    Ok(Json(traderview_expense::section_1042::check(&b)))
}

// ── §1015 carryover basis on gifts ───────────────────────────────────
// Mounted at /api/calc/section-1015. Pure compute; §1015(a) general
// carryover; §1015(a) dual-basis rule for depreciated property with
// phantom zone; §1015(d) gift-tax basis increase with two ceilings
// (cap at net appreciation, cap at FMV); §1223(2) holding-period
// tacking on gain path; gift-date start on dual-basis loss path.

async fn section_1015_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1015::Section1015Input>,
) -> Result<Json<traderview_expense::section_1015::Section1015Result>, ApiError> {
    if b.donor_adjusted_basis < Decimal::ZERO
        || b.fmv_at_gift_date < Decimal::ZERO
        || b.gift_tax_paid < Decimal::ZERO
        || b.gift_amount_for_tax_purposes < Decimal::ZERO
        || b.sale_price < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    if b.sale_date < b.gift_date {
        return Err(ApiError::BadRequest(
            "sale_date must be on or after gift_date".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1015::compute(&b)))
}

// ── §108 cancellation of debt income ─────────────────────────────────
// Mounted at /api/calc/section-108. §61(a)(12) gross-income default;
// §108(a)(1)(A) bankruptcy full exclusion (priority 1); §108(a)(1)(E)
// QPRI for pre-2026 arrangements (priority 2 over insolvency unless
// elected otherwise); §108(a)(1)(B) insolvency under §108(d)(3) test;
// §108(a)(1)(C) qualified farm; §108(a)(1)(D) QRPBI for non-C-corp;
// §108(b) attribute reduction = excluded amount.

async fn section_108_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_108::Section108Input>,
) -> Result<Json<traderview_expense::section_108::Section108Result>, ApiError> {
    if b.canceled_debt_amount < Decimal::ZERO
        || b.debtor_assets_fmv < Decimal::ZERO
        || b.debtor_liabilities < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "canceled_debt_amount, debtor_assets_fmv, and debtor_liabilities must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_108::compute(&b)))
}

// ── §104 damages for personal injury / sickness ─────────────────────
// Mounted at /api/calc/section-104. §104(a)(2) exclusion for damages
// on account of personal physical injury / sickness (compensatory +
// pain & suffering + lost wages + physical-origin emotional distress
// all excluded); non-physical emotional distress included except
// medical care amount; punitive damages included except § 104(c)
// wrongful-death only-punitives state carveout; interest always
// included; § 104(a) flush prior-§213 tax benefit recapture.

async fn section_104_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_104::Section104Input>,
) -> Result<Json<traderview_expense::section_104::Section104Result>, ApiError> {
    if b.physical_injury_compensatory_dollars < 0
        || b.pain_suffering_physical_origin_dollars < 0
        || b.lost_wages_physical_origin_dollars < 0
        || b.emotional_distress_physical_origin_dollars < 0
        || b.emotional_distress_non_physical_dollars < 0
        || b.medical_care_for_emotional_distress_dollars < 0
        || b.punitive_damages_dollars < 0
        || b.interest_on_award_dollars < 0
        || b.previously_deducted_medical_with_tax_benefit_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_104::compute(&b)))
}

// ── §1012 cost basis general rule + identification methods ──────
// Mounted at /api/calc/section-1012. § 1012 is the foundational
// basis-tracking provision that every trader uses to compute gain
// or loss on disposition of stock, mutual fund shares, debt
// instruments, and options. § 1012(a) general rule: basis = cost
// of property, except as overridden by subchapter C (corporate
// distributions), K (partners and partnerships), or P (capital
// gains and losses), or by § 1014 stepped-up basis at death,
// § 1015 basis of gifts, § 1031 like-kind exchange, etc.
// § 1012(b) real property taxes treated as imposed on taxpayer
// under § 164(d) must be excluded from cost. § 1012(c)(1) account-
// by-account basis tracking for specified securities sold after
// applicable dates (multi-account aggregation prohibited).
// § 1012(c)(2) average cost method for RIC mutual fund shares
// (average cost single category ACSC = total cost / total shares).
// § 1012(c)(3) pre-2012 / post-2012 RIC stock treated as separate
// accounts (bifurcation prevents post-2012 basis-method election
// from affecting pre-2012 non-covered shares). § 1012(d) dividend
// reinvestment plan stock acquired after December 31, 2011 uses
// one of the RIC basis methods (typically average cost). Treas.
// Reg. § 1.1012-1(c) specific identification requires (1) at-time-
// of-sale designation to broker / transfer agent AND (2) written
// confirmation within reasonable time; failure causes FIFO default
// per Treas. Reg. § 1.1012-1(c)(1). LIFO NOT permitted for stock
// or securities under § 1012; LIFO allowed only for inventory under
// § 471 / § 472. Cost basis reporting reform (Energy Improvement
// and Extension Act of 2008; Public Law 110-343): phased-in broker
// 1099-B reporting — stock acquired on or after January 1, 2011;
// mutual fund / DRIP stock acquired on or after January 1, 2012;
// debt instruments and options acquired on or after January 1,
// 2014; pre-effective-date holdings are "non-covered" and taxpayer
// bears basis-tracking burden. Treas. Reg. § 1.1012-1(e) wash sale
// basis adjustment: § 1091(d) disallowed loss added to replacement
// security basis; holding period of replacement tacks onto
// original. 15-mode severity ladder × 9 property types × 5 basis
// method elections × 6 acquisition date statuses × 8 compliance
// aspects × variable specific-identification / written-confirmation
// / pre/post-2012 separation / wash-sale flags. Sibling cluster:
// section_1001 (gain or loss = amount realized − adjusted basis),
// section_1011 (adjusted basis for determining gain or loss),
// section_1014 (stepped-up basis at death), section_1015 (basis of
// gifts), section_1031 (like-kind exchange basis carryover),
// section_1091 (wash sale basis adjustment under § 1091(d)),
// section_1295 (PFIC qualified electing fund), section_164 (real
// property tax allocation under § 164(d)), section_471 / section_
// 472 (inventory methods including LIFO).

async fn section_1012_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1012::Section1012Input>,
) -> Result<Json<traderview_expense::section_1012::Section1012Result>, ApiError> {
    Ok(Json(traderview_expense::section_1012::compute(&b)))
}

// ── §1014 stepped-up basis at death ──────────────────────────────────
// Mounted at /api/calc/section-1014. Pure compute; §1014(a)(1) DOD
// step-up; §1014(a)(2) §2032 alternate-valuation-date election;
// §1014(c) IRD denies step-up; §1014(e) 1-year clawback for deathbed
// gifts returning to donor; §1014(f) Form 706 consistent-basis cap.

async fn section_1014_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1014::Section1014Input>,
) -> Result<Json<traderview_expense::section_1014::Section1014Result>, ApiError> {
    if b.decedents_adjusted_basis < Decimal::ZERO || b.fmv_at_dod < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "decedents_adjusted_basis and fmv_at_dod must be >= 0".into(),
        ));
    }
    if let Some(av) = b.fmv_at_alternate_valuation_date {
        if av < Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "fmv_at_alternate_valuation_date must be >= 0".into(),
            ));
        }
    }
    if let Some(f706) = b.fmv_reported_on_form_706 {
        if f706 < Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "fmv_reported_on_form_706 must be >= 0".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_1014::compute(&b)))
}

// ── §1014(e) appreciated-property-by-gift-within-1-year-of-death ────
// Mounted at /api/calc/section-1014e. Anti-abuse companion to § 1014
// general step-up. Triggers when (1) decedent acquired property by
// gift, (2) within 1 year of death, (3) property passes back to
// donor or donor's spouse. Result: basis = decedent's adjusted
// basis immediately before death (no FMV step-up). Credit-shelter-
// trust workaround per NAEPC Journal analysis.

async fn section_1014e_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1014e::Section1014eInput>,
) -> Result<Json<traderview_expense::section_1014e::Section1014eResult>, ApiError> {
    if b.donor_adjusted_basis_at_gift_dollars < 0
        || b.fmv_at_gift_dollars < 0
        || b.decedent_adjusted_basis_immediately_before_death_dollars < 0
        || b.fmv_at_death_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1014e::compute(&b)))
}

// ── §1091 wash sale loss disallowance ────────────────────────────────
// Mounted at /api/calc/section-1091. Pure compute; 61-day window
// (sale_date ±30 days inclusive), FIFO basis allocation to replacement
// lots under §1091(d), Rev. Rul. 2008-5 IRA permanent-loss carve-out,
// and §475(f)(1)(C) MTM elector exemption.

// ── § 1059 extraordinary dividend basis reduction ────────────────
// Mounted at /api/calc/section-1059. Anti-abuse for corporate
// shareholders claiming dividends-received deduction (§ 243 +
// § 245 + § 245A) on dividends exceeding 10% of common stock basis
// (5% preferred) within 2-year holding period. Basis reduced (not
// below zero) by nontaxed portion; excess recognized as gain from
// sale or exchange of stock. § 1059(e)(1) per se extraordinary
// dividends override threshold and holding period: non-pro-rata
// redemptions, partial liquidations, § 318(a)(4) options-attribution
// redemptions. § 1059(c)(3) aggregation: 85-day short window, 365-
// day long window. TCJA 2017 § 14101 added § 245A (100% DRD for
// foreign-source income from 10%-owned foreign corporations) —
// significantly expanded § 1059 cross-border scope. Sibling cluster:
// § 243 / § 245 / § 245A (DRD definitions populating nontaxed
// portion), § 301 (dividend distribution rules), § 318 (constructive
// ownership), § 1059(d)(6) (entire-existence-of-corporation exception
// inapplicable to § 1059(e)(1)).

async fn section_1059_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1059::Section1059Input>,
) -> Result<Json<traderview_expense::section_1059::Section1059Output>, ApiError> {
    Ok(Json(traderview_expense::section_1059::check(&b)))
}

// ── §1060 Special Allocation Rules for Certain Asset Acquisitions ──
// Mounted at /api/calc/section-1060. Enacted by Section 641 of the
// Tax Reform Act of 1986 (Public Law 99-514). Both buyer and seller
// in an applicable asset acquisition (transfer of a group of assets
// constituting a trade or business where goodwill or going concern
// attaches) must file Form 8594 and allocate consideration using the
// residual method per § 338(b)(5) and Treas. Reg. § 1.338-6 across
// SEVEN ASSET CLASSES (Class I cash through Class VII goodwill
// residual). § 1060(b) consistency requirement between buyer and
// seller; § 6662 + § 6721 + § 6722 penalty exposure for inconsistent
// allocation or failure to file Form 8594. Trader-business-purchase
// / M&A relevance for every taxpayer buying or selling a trade or
// business.
async fn section_1060_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1060::Section1060Input>,
) -> Result<Json<traderview_expense::section_1060::Section1060Result>, ApiError> {
    Ok(Json(traderview_expense::section_1060::check(&b)))
}

async fn section_1091_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1091::Section1091Input>,
) -> Result<Json<traderview_expense::section_1091::Section1091Result>, ApiError> {
    if b.sale_shares < 0
        || b.sale_price_per_share < Decimal::ZERO
        || b.basis_per_share < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "sale_shares, sale_price_per_share, basis_per_share must be >= 0".into(),
        ));
    }
    for p in &b.replacement_purchases {
        if p.shares < 0 || p.price_per_share < Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "replacement shares and price must be >= 0".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_1091::compute(&b)))
}

// ── §408A(d)(3)(F) Roth conversion 5-year rule ───────────────────────
// Mounted at /api/calc/section-408a-d3. Pure compute; §408A(d)(4)
// ordering rules (contributions FIFO first, conversions FIFO with
// separate 5-year clocks, earnings last), §408A(d)(3)(F) 5-year
// aging per conversion, age 59½ bypasses 5-year for §72(t).

#[allow(non_snake_case)]
async fn section_408A_d3_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_408A_d3::Section408AD3Input>,
) -> Result<Json<traderview_expense::section_408A_d3::Section408AD3Result>, ApiError> {
    if b.withdrawal_amount < Decimal::ZERO
        || b.total_contributions_basis < Decimal::ZERO
        || b.earnings_balance < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_408A_d3::compute(&b)))
}

// ── §461(l) excess business loss limitation ─────────────────────────
// Mounted at /api/calc/section-461l. Completes loss-limit cascade
// after §704(d) → §465 → §469. Noncorporate taxpayers only; 2021+
// effective (CARES suspended 2018-2020). 2026 thresholds re-indexed
// by OBBBA: $256k single / $512k MFJ. Excess becomes §172 NOL
// carryforward.

async fn section_461l_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_461l::Section461lInput>,
) -> Result<Json<traderview_expense::section_461l::Section461lResult>, ApiError> {
    if b.aggregate_business_deductions_after_prior_limits < Decimal::ZERO
        || b.aggregate_business_income < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "aggregate dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_461l::compute(&b)))
}

// ── §691 income in respect of decedent (IRD) ─────────────────────────
// Mounted at /api/calc/section-691. §691(a) IRD includible in heir's
// gross income (character preserved); §691(c) deduction = heir's
// pro-rata share of federal estate tax attributable to total IRD per
// Treas. Reg. § 1.691(c)-1(a)(2) two-step. Pairs with §1014(c) IRD
// exception (no step-up on IRD assets).

async fn section_691_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_691::Section691Input>,
) -> Result<Json<traderview_expense::section_691::Section691Result>, ApiError> {
    if b.ird_received_by_heir < Decimal::ZERO
        || b.total_ird_in_estate < Decimal::ZERO
        || b.federal_estate_tax_attributable_to_total_ird < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_691::compute(&b)))
}

// ── §704(d) partner basis limitation ─────────────────────────────────
// Mounted at /api/calc/section-704d. Outside basis = beginning + cap
// contributions + share of income + §752 liability increases -
// §752 liability decreases - distributions. Loss allowed ≤ basis;
// excess carries forward indefinitely. Sequential pre-§465/§469/
// §461(l) limitation.

async fn section_704d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_704d::Section704dInput>,
) -> Result<Json<traderview_expense::section_704d::Section704dResult>, ApiError> {
    if b.capital_contributions_this_year < Decimal::ZERO
        || b.share_of_partnership_income < Decimal::ZERO
        || b.share_of_recourse_liabilities_increase < Decimal::ZERO
        || b.share_of_nonrecourse_liabilities_increase < Decimal::ZERO
        || b.share_of_recourse_liabilities_decrease < Decimal::ZERO
        || b.share_of_nonrecourse_liabilities_decrease < Decimal::ZERO
        || b.distributions_received < Decimal::ZERO
        || b.allocated_partnership_loss < Decimal::ZERO
        || b.prior_year_suspended_loss < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs other than beginning basis must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_704d::compute(&b)))
}

// ── § 721 partnership contribution non-recognition ─────────────────
// Mounted at /api/calc/section-721. Partnership-side counterpart
// to § 351. § 721(a) general bilateral non-recognition rule.
// § 721(b) investment company exception (> 80% readily marketable
// stocks/securities triggers gain recognition; prevents tax-free
// diversification). § 721(c) related foreign partner gain
// recognition (effective Jan 18, 2017) with Gain Deferral Method
// safe harbor under § 1.721(c)-3 allowing remedial-income
// allocation over recovery period. § 721(d) recapture rules tie
// in with § 736(a) retiring partner distributions. Sibling
// modules: § 351 (corporate-side counterpart), § 704(c) (built-
// in gain allocation), § 752 (partnership liabilities), § 754
// (basis adjustment election). Trader-relevant for hedge funds,
// real estate JVs, fund-of-fund structures.

async fn section_721_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_721::Section721Input>,
) -> Result<Json<traderview_expense::section_721::Section721Result>, ApiError> {
    if b.fmv_contributed_cents > 1_000_000_000_000 || b.basis_contributed_cents > 1_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "fmv_contributed_cents or basis_contributed_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_721::compute(&b)))
}

// ── § 723 basis of property contributed to partnership ──────────
// Mounted at /api/calc/section-723. Foundational inside-basis
// provision: partnership takes contributing partner's adjusted
// basis (carryover) plus any § 721(b) investment-company gain
// recognized. § 722 paired outside-basis rule produces inside =
// outside basis at moment of contribution. § 704(c) requires pre-
// contribution gain/loss to be allocated back to contributing
// partner upon subsequent partnership disposition. § 1223(2)
// holding-period tacking rule. § 351(e)(1) investment-company
// definition (80%-of-assets threshold). Sibling cluster: § 721
// (contribution nonrecognition), § 722 (outside basis), § 732
// (distributee basis), § 743 + § 734 (basis adjustments under
// § 754), § 704(c) (built-in gain allocation), § 1223(2) (holding-
// period tacking).

async fn section_723_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_723::Section723Input>,
) -> Result<Json<traderview_expense::section_723::Section723Output>, ApiError> {
    Ok(Json(traderview_expense::section_723::check(&b)))
}

// ── § 731 partnership distribution gain/loss recognition ────────────
// Mounted at /api/calc/section-731. Direct sibling to § 721
// (contribution non-recognition, iter 264) — completes the
// partnership contribution/distribution cycle. § 731(a)(1) gain
// recognition only to extent MONEY distributed exceeds partner's
// outside basis (applies to current AND liquidating). § 731(a)(2)
// LOSS recognition only on LIQUIDATING distribution when partner
// receives only money + § 751 hot assets + inventory. § 731(b)
// partnership-level non-recognition. § 731(c) marketable
// securities treated as money for gain calculation; § 731(c)(3)
// exceptions for investment partnerships + contribution rollover
// + reduction-of-net-gain rule. Sibling cluster: § 721 + § 732 +
// § 733 + § 736 + § 751 + § 754 + § 707(c).

async fn section_731_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_731::Section731Input>,
) -> Result<Json<traderview_expense::section_731::Section731Result>, ApiError> {
    if b.money_distributed_cents > 1_000_000_000_000
        || b.marketable_securities_fmv_distributed_cents > 1_000_000_000_000
        || b.partner_outside_basis_cents > 1_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "money_distributed_cents, marketable_securities_fmv_distributed_cents, or partner_outside_basis_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_731::compute(&b)))
}

// ── § 752 partnership liabilities — outside basis allocation ────────
// Mounted at /api/calc/section-752. Completes partnership cluster
// (§ 721 + § 731 + § 752). § 752(a) liability share increase
// treated as money contribution (basis +); § 752(b) decrease
// treated as money distribution (basis -, potential § 731(a)(1)
// gain). § 752(c) property-subject-to-liability rule.
// Treas. Reg. § 1.752-1 netting rule for single-transaction
// gross changes. Treas. Reg. § 1.752-2 recourse allocation
// (economic risk of loss). Treas. Reg. § 1.752-3 nonrecourse
// THREE-TIER allocation: tier 1 § 704(b) minimum gain; tier 2
// § 704(c) hypothetical-disposition gain; tier 3 excess-
// nonrecourse profit share. TD 10014 (December 2, 2024) final
// recourse regulations. Sibling cluster: § 721 + § 731 + § 704(b)
// + § 704(c) + § 704(d) + § 705.

// ── § 751 hot assets recharacterization ──────────────────────────
// Mounted at /api/calc/section-751. § 751 overrides Subchapter K
// capital-character default in two scenarios: (1) § 751(a) sale or
// exchange of partnership interest — bifurcate amount realized into
// ordinary income (partner's share of unrealized receivables +
// inventory items) + § 741 capital remainder; ALL inventory is hot
// regardless of appreciation. (2) § 751(b) disproportionate
// distribution — recast as deemed sale/exchange between distributee
// and partnership when distribution alters partner's share of
// unrealized receivables or SUBSTANTIALLY APPRECIATED inventory items
// (FMV > 120% of adjusted basis per § 751(b)(3)(A)). Unrealized
// receivables (§ 751(c)) include § 1245/§ 1250/§ 1252/§ 1254
// recapture potential. Inventory items (§ 751(d)) include partnership
// inventory + property held primarily for sale to customers. Sibling
// cluster: § 741 (capital character default), § 731 (distribution
// nonrecognition), § 752 (liabilities), § 754 (basis adjustment
// election), § 743 (transferee basis adjustment under § 754),
// § 734 (transferor basis adjustment under § 754).

async fn section_751_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_751::Section751Input>,
) -> Result<Json<traderview_expense::section_751::Section751Output>, ApiError> {
    Ok(Json(traderview_expense::section_751::check(&b)))
}

// ── § 743 transferee basis adjustment ────────────────────────────
// Mounted at /api/calc/section-743. § 743(a) default rule: no basis
// adjustment on transfer. § 743(b) exception: adjustment REQUIRED if
// EITHER § 754 election is in effect OR partnership has substantial
// built-in loss immediately after transfer. § 743(b) math: increase
// inside basis by excess of transferee outside basis over their
// proportionate share of inside basis; decrease by the reverse.
// Partner-specific (transferee only). § 743(d) substantial built-in
// loss two-prong test: (1) partnership inside basis > FMV by more
// than $250K; or (2) (TCJA 2017, eff. transfers after Dec. 31, 2017)
// transferee would be allocated loss > $250K if all assets sold at
// FMV. Sibling cluster: § 754 (election mechanics), § 734
// (distributee basis adjustment under § 754), § 755 (allocation
// among partnership properties), § 751 (hot assets — interacts on
// transfer), § 1014 (estate basis step-up on death).

async fn section_743_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_743::Section743Input>,
) -> Result<Json<traderview_expense::section_743::Section743Output>, ApiError> {
    Ok(Json(traderview_expense::section_743::check(&b)))
}

async fn section_752_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_752::Section752Input>,
) -> Result<Json<traderview_expense::section_752::Section752Result>, ApiError> {
    if b.partner_share_liabilities_before_cents > 1_000_000_000_000
        || b.partner_share_liabilities_after_cents > 1_000_000_000_000
        || b.partner_outside_basis_before_cents > 1_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "partner_share_liabilities or partner_outside_basis_before_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_752::compute(&b)))
}

// ── §704(c) pre-contribution built-in gain/loss allocation ──────────
// Mounted at /api/calc/section-704c. §704(c)(1)(A) gain allocation on
// disposition; §704(c)(1)(B) 7-year anti-mixing-bowl (distribution to
// other partner); §737 reverse (contributor receives other property);
// §704(c)(1)(C) built-in loss restriction (AJCA 2004 §833(a)); three
// allocation methods (traditional / curative / remedial).

async fn section_704c_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_704c::Section704cInput>,
) -> Result<Json<traderview_expense::section_704c::Section704cResult>, ApiError> {
    if b.pre_contribution_built_in_gain < Decimal::ZERO
        || b.pre_contribution_built_in_loss < Decimal::ZERO
        || b.disposition_gain_realized < Decimal::ZERO
        || b.other_property_received_fmv < Decimal::ZERO
        || b.contributor_outside_basis < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_704c::compute(&b)))
}

// ── §1235 sale or exchange of patents ───────────────────────────────
// Mounted at /api/calc/section-1235. Automatic LTCG on transfer of
// all substantial rights in a patent by qualifying "holder"
// regardless of holding period. §1235(b) holder = inventor OR
// pre-reduction-to-practice financial backer who paid consideration
// and is not employer or related party. §1235(d) related-party
// disqualification (§267(b) modified, 25% threshold, siblings
// excluded). Treas. Reg. §1.1235-2(b) all-substantial-rights test
// (no geographic / duration / field-of-use limitations).
// Post-TCJA: §1221(a)(3) now excludes inventor's patent from
// capital-asset treatment, so §1235 is the only LTCG path.

async fn section_1235_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1235::Section1235Input>,
) -> Result<Json<traderview_expense::section_1235::Section1235Result>, ApiError> {
    if b.gain_amount_dollars < 0 {
        return Err(ApiError::BadRequest(
            "gain_amount_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1235::compute(&b)))
}

// ── §1239 ordinary-income recharacterization on related-party sales ──
// Mounted at /api/calc/section-1239. § 1239(a) all gain on sale or
// exchange of property between related persons treated as ORDINARY
// INCOME if property is depreciable under § 167 in the hands of the
// TRANSFEREE (not transferor). § 1239(b) related persons = (1)
// controlled entities under § 1239(c)(1) (corp > 50 % by value;
// partnership > 50 % capital or profits; § 267(b)(3)/(10)/(11)/(12)
// related); (2) taxpayer + trust where taxpayer/spouse is non-
// remote-contingent beneficiary; (3) executor + non-pecuniary-
// bequest beneficiary. § 1239(c)(2) constructive ownership under
// § 267(c) applies. § 1239(d) employer + owner-employee pension
// plan related. Treas. Reg. § 1.1239-1 (post-Oct-4-1976). Defeats
// tax arbitrage of repeatedly stepping up basis via related-party
// sales to re-depreciate at capital-gains cost.

async fn section_1239_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1239::Section1239Input>,
) -> Result<Json<traderview_expense::section_1239::Section1239Result>, ApiError> {
    Ok(Json(traderview_expense::section_1239::compute(&b)))
}

// ── §1248 CFC stock sale deemed-dividend recharacterization ──────────
// Mounted at /api/calc/section-1248. § 1248(a) 10 %+ US shareholder
// (§ 958(a)/(b)) at any time during 5-year period ending on sale
// date — gain recharacterized as deemed dividend to extent of CFC
// E&P attributable to stock + ownership period while CFC. § 1248(b)
// individual LTCG limitation: pro rata domestic-corp tax + LTCG tax
// on residual. § 1248(c)(2) lower-tier CFC E&P inclusion when upper
// CFC owns ≥ 50 %. § 1248(d)(1) PTI (§ 951 subpart F / § 951A
// GILTI) excluded. § 1248(d)(2) ECI excluded. § 1248(d)(3) pre-1963
// excluded. § 1248(e) US-holding-corp avoidance extension. § 245A
// TCJA 2017 100 % DRD effectively eliminates § 1248 for US C-corp
// sellers via foreign-source dividend participation exemption.
// Trader-critical for individuals selling foreign-corp stock and
// hedge fund / private equity LPs computing CFC-stock-sale tax.

async fn section_1248_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1248::Section1248Input>,
) -> Result<Json<traderview_expense::section_1248::Section1248Result>, ApiError> {
    Ok(Json(traderview_expense::section_1248::compute(&b)))
}

// ── §1252 farm land disposition soil/water conservation recapture ────
// Mounted at /api/calc/section-1252. § 1252(a)(1) gain on farm land
// held < 10 years recharacterized as ordinary income to extent of
// lesser of applicable percentage × § 175 soil and water
// conservation deductions OR gain recognized. Sliding scale:
// 100 % < 5 years; 80 % 6th; 60 % 7th; 40 % 8th; 20 % 9th; 0 %
// at 10+ years. § 1252(a)(2) formerly § 182 land clearing —
// REPEALED for taxable years beginning after Dec 31, 1985 by Tax
// Reform Act of 1986 (P.L. 99-514). § 1252(b) applies before
// § 1245. § 1252(c) farm land = any land with § 175 deductions
// allowed. Treas. Reg. § 1.1252-1 + § 1.1252-2. Trader-critical
// for farm/ranch LPs + MLPs, family-office farm portfolios,
// agricultural REIT exits, family-farm estate liquidations.

async fn section_1252_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1252::Section1252Input>,
) -> Result<Json<traderview_expense::section_1252::Section1252Result>, ApiError> {
    Ok(Json(traderview_expense::section_1252::compute(&b)))
}

// ── §1254 oil/gas/mineral property natural-resource recapture ────────
// Mounted at /api/calc/section-1254. § 1254(a)(1) gain on
// disposition of § 1254 property recharacterized as ordinary income
// to extent of lesser of aggregate § 1254 costs OR gain. § 1254(a)
// (1)(B) post-1986: aggregate IDC § 263(c) + mine development § 616
// + mine exploration § 617 + § 611 depletion reducing basis. Pre-
// Jan 1, 1987: IDC reduced by hypothetical capitalized depletion.
// § 1254(a)(3) partnership/trust election with Treas. Reg.
// § 1.1254-5 allocation. § 1254(b) carryover basis transferee
// (§ 351/§ 721) tacks transferor periods. § 1254(c) definition:
// § 1254 property = any § 614 property with § 611 depletion basis
// adjustments. § 1254(d) partial disposition: entire § 1254 costs
// allocable to disposed portion to extent of gain. Treas. Reg.
// § 1.1254-2(b) nonproductive-well exception NOT recapturable
// except limited-risk reimbursement situations. Trader-critical
// for oil/gas LPs + MLPs, mining MLPs, mineral rights, working
// interest, overriding royalty interest, net profits interest.
// Sibling to § 1245 (personal property), § 1250 (real property),
// § 1252 (farm land), § 1255 (§ 126 property) recapture.

async fn section_1254_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1254::Section1254Input>,
) -> Result<Json<traderview_expense::section_1254::Section1254Result>, ApiError> {
    Ok(Json(traderview_expense::section_1254::compute(&b)))
}

// ── §1255 § 126 property conservation cost-sharing recapture ─────────
// Mounted at /api/calc/section-1255. § 1255(a)(1) gain on
// disposition of § 126 property recharacterized as ordinary income
// to extent of lesser of applicable percentage × § 126 excluded
// payments OR gain recognized. Sliding scale: 100 % through year
// 10; -10 % per full year thereafter; 0 % at 20+ years. § 1255(b)
// special rules + cross-reference to § 126. § 126 excluded payments
// = USDA Agricultural Conservation Program / Conservation Reserve
// Program / Environmental Quality Incentives Program; Forest
// Service Forestry Incentives Program; designated state programs.
// Treas. Reg. § 16A.1255-1 + 26 CFR Part 16A temporary regs.
// Completes recapture quartet with § 1245 (personal property) +
// § 1250 (real property) + § 1252 (farm land — iter 634) + § 1254
// (oil/gas/mineral — iter 622). Anti-double-benefit policy
// prevents taxpayer from excluding § 126 payment AND getting
// capital gains on improved property.

async fn section_1255_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1255::Section1255Input>,
) -> Result<Json<traderview_expense::section_1255::Section1255Result>, ApiError> {
    Ok(Json(traderview_expense::section_1255::compute(&b)))
}

// ── §754 election + §743(b) inside basis adjustment ─────────────────
// Mounted at /api/calc/section-754. §743(b) inside basis adjustment
// for transferee partner = outside basis − share of inside basis;
// applies when §754 election in effect OR §743(d)(1)(A) partnership
// BIL > $250k OR §743(d)(1)(B) (TCJA addition) transferee
// hypothetical loss > $250k. Sale/exchange + death-of-partner
// transfer types covered (death takes §1014 FMV outside basis).

// ── § 734 distributee basis adjustment ───────────────────────────
// Mounted at /api/calc/section-734. § 734(a) default: no basis
// adjustment to remaining partnership property on distribution.
// § 734(b) exception: adjustment REQUIRED if § 754 election in
// effect OR substantial basis reduction. § 734(b)(1) increase:
// distributee gain under § 731(a)(1) + excess of partnership basis
// in distributed property over distributee § 732 basis. § 734(b)(2)
// decrease: distributee loss under § 731(a)(2) + excess of
// distributee § 732 basis in property over partnership basis.
// § 734(d) substantial basis reduction: sum of § 734(b)(2)(A)+(B)
// > $250K (AJCA 2004 § 833 added this prong, made mandatory even
// without § 754). Companion to § 743 (transfer-side basis adjustment
// — § 734 is distribution-side). Sibling: § 754 (election), § 743
// (transferee basis), § 755 (allocation), § 731 (distribution
// recognition), § 732 (distributee basis in distributed property).

async fn section_734_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_734::Section734Input>,
) -> Result<Json<traderview_expense::section_734::Section734Output>, ApiError> {
    Ok(Json(traderview_expense::section_734::check(&b)))
}

// ── § 732 distributee basis in distributed property ──────────────
// Mounted at /api/calc/section-732. § 732(a)(1) general carryover
// basis rule for current distributions; § 732(a)(2) limitation to
// outside basis minus money. § 732(b) liquidating-distribution
// substituted basis = outside basis minus money. § 732(c) basis
// allocation: first to § 751(c)/(d) hot assets, then to other
// property. § 732(d) special rule: distributee within 2 years of
// transfer where § 754 NOT in effect may elect to treat property as
// if § 743(b) adjustment were in effect. Mandatory application per
// Treas. Reg. § 1.732-1(d)(4) when FMV at transfer > 110% of basis
// + § 732(c) shift to depreciable + § 743(b) would change basis.
// § 732(f) corporate-partner distribution rule (post-2015). Sibling
// cluster: § 731 (distribution recognition rules — basis determined
// by § 732 sets up § 731 gain/loss), § 734 (iter 578 — distributee
// basis adjustment), § 736 (iter 582 — retiring partner payment
// characterization), § 737 (iter 580 — mixing-bowl), § 743 (iter
// 576 — transferee basis adjustment), § 751 (iter 572 — hot assets
// definition that § 732(c) cross-references), § 754 (election
// mechanics — § 732(d) operates when § 754 was NOT in effect at
// time of transfer), § 755 (iter 586 — § 755 allocation framework).

async fn section_732_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_732::Section732Input>,
) -> Result<Json<traderview_expense::section_732::Section732Output>, ApiError> {
    Ok(Json(traderview_expense::section_732::check(&b)))
}

// ── § 737 mixing-bowl recognition for contributing partner ───────
// Mounted at /api/calc/section-737. One half of the Subchapter K
// "anti-mixing-bowl" anti-abuse regime — contributing-partner-side
// recognition when a partner who contributed appreciated § 704(c)
// property receives a distribution of OTHER property (not money)
// within 7 years of contribution. Other half is § 704(c)(1)(B) —
// noncontributing-partner-side recognition. § 737(a)(1) lesser-of
// test: (1) net precontribution gain under § 737(b) (7-year
// lookback); or (2) excess of distributed property FMV over
// partner's outside basis. Money-only distributions carved out
// (§ 731(a)(1) governs). § 737(c) basis adjustments prevent double-
// counting. Original 5-year window enacted by Energy Policy Act of
// 1992 (Pub. L. 102-486); extended to 7 years by Taxpayer Relief
// Act of 1997 (Pub. L. 105-34). Sibling cluster: § 704(c)(1)(B)
// (paired recognition rule), § 704(c) (built-in gain allocation),
// § 731 (distribution gain/loss), § 732 (distributee basis),
// § 751 (hot assets — applies before § 737), § 707(a)(2)(B)
// (disguised sales — alternative anti-abuse path).

async fn section_737_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_737::Section737Input>,
) -> Result<Json<traderview_expense::section_737::Section737Output>, ApiError> {
    Ok(Json(traderview_expense::section_737::check(&b)))
}

// ── § 741 sale or exchange of partnership interest ───────────────
// Mounted at /api/calc/section-741. Capital character default for
// partnership-interest sales. Three-step computation: (1) amount
// realized = cash + FMV property + liability relief (§ 752(d)
// deemed distribution); (2) adjusted basis = outside basis under
// § 705; (3) gain/loss = amount realized − adjusted basis. Holding
// period > 1 year (366+ days) qualifies for § 1(h) preferred 0/15/
// 20% long-term rates; ≤ 1 year is short-term at ordinary rates up
// to 37%. § 751(a) override: portion attributable to unrealized
// receivables (§ 751(c)) + inventory items (§ 751(d)) is ordinary
// character; remainder is § 741 capital. Sibling cluster: § 705
// (outside basis), § 706(c)(2) (partnership year closes on entire-
// interest sale triggering § 741), § 736 (distinguish redemption
// from sale), § 743 (transferee basis adjustment downstream),
// § 751 (hot assets carve-out), § 752(d) (liability relief deemed
// distribution), § 1223 (holding period), § 1(h) (LTCG rates).

async fn section_741_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_741::Section741Input>,
) -> Result<Json<traderview_expense::section_741::Section741Output>, ApiError> {
    Ok(Json(traderview_expense::section_741::check(&b)))
}

// ── § 736 retiring/deceased partner payment characterization ─────
// Mounted at /api/calc/section-736. § 736(a) payments NOT in
// exchange for partnership property: subdivides into § 736(a)(1)
// distributive share (payment determined with regard to partnership
// income — ordinary, reduces other partners' shares) and
// § 736(a)(2) guaranteed payment under § 707(c) (payment determined
// without regard to income — ordinary, § 162 partnership deduction).
// § 736(b) payments in exchange for partnership PROPERTY: capital
// character, § 731 / § 732 distribution rules, no partnership
// deduction. § 736(b)(2) special rule for service partnerships
// (capital not material income-producing factor per § 736(b)(3))
// where general partner is retiring/dying: unrealized receivables
// (§ 751(c)) + goodwill (except as partnership agreement provides)
// fall back into § 736(a) ordinary treatment. DRA 1993 § 13262
// (Pub. L. 103-66) effective for partners retiring/dying on or
// after Jan. 5, 1993 limits the § 736(b)(2) special rule to
// service partnerships + general partners. Sibling: § 707(c)
// (guaranteed payment definition that § 736(a)(2) cross-references),
// § 731 (distribution recognition that § 736(b) invokes), § 732
// (distributee basis that § 736(b) invokes), § 751 (hot assets
// recharacterization for retiring partner sale-of-interest), § 1402
// (a)(13) (limited-partner SECA exclusion analysis).

async fn section_736_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_736::Section736Input>,
) -> Result<Json<traderview_expense::section_736::Section736Output>, ApiError> {
    Ok(Json(traderview_expense::section_736::check(&b)))
}

async fn section_754_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_754::Section754Input>,
) -> Result<Json<traderview_expense::section_754::Section754Result>, ApiError> {
    if b.transferee_outside_basis < Decimal::ZERO
        || b.transferee_share_of_inside_basis < Decimal::ZERO
        || b.partnership_total_inside_basis < Decimal::ZERO
        || b.partnership_total_fmv < Decimal::ZERO
        || b.transferee_hypothetical_loss_on_immediate_sale < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_754::compute(&b)))
}

// ── § 755 allocation of § 743(b) and § 734(b) basis adjustments ──
// Mounted at /api/calc/section-755. Downstream allocation regime
// that determines how § 743(b) transferee-side basis adjustments
// (iter 576) and § 734(b) distribution-side basis adjustments (iter
// 578) are spread across partnership properties. § 755(a) general
// rule: adjustments allocated to preserve partner economic position
// based on hypothetical-sale income/gain/loss. § 755(b) two-class
// rule: capital gain property class (§ 1221 capital assets + § 1231
// (b) depreciable trade-or-business property) vs ordinary income
// property class (§ 751 hot assets + § 1245 recapture potential +
// inventory). Within-class allocation per net unrealized appreciation
// or depreciation. § 743(b) class attribution: amount to ordinary
// income class = total ordinary income/gain/loss that would be
// allocated to transferee on hypothetical sale of all ordinary
// income property; remainder to capital gain class. § 734(b) class
// attribution: follows character of distributee gain/loss under
// § 731. TD 9059 (June 9, 2003) finalized coordination with § 1060
// residual method. Sibling cluster: § 754 (election that triggers
// § 755 allocation downstream), § 743 (iter 576 — transferee
// adjustment subject to § 755 allocation), § 734 (iter 578 —
// distributee adjustment subject to § 755 allocation), § 751
// (iter 572 — hot assets define ordinary income class composition),
// § 1221 + § 1231(b) (capital gain class composition).

async fn section_755_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_755::Section755Input>,
) -> Result<Json<traderview_expense::section_755::Section755Output>, ApiError> {
    Ok(Json(traderview_expense::section_755::check(&b)))
}

// ── §465 at-risk rules ───────────────────────────────────────────────
// Mounted at /api/calc/section-465. §465(a) loss limited to amount
// at risk; §465(b)(1) cash + basis + recourse; §465(b)(2) external
// pledged property; §465(b)(3) related-party reduces; §465(b)(4)
// general nonrecourse excluded; §465(b)(6) qualified nonrecourse for
// real property included; §465(d) suspended loss carryover; §465(e)
// negative-at-risk recapture.

async fn section_465_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_465::Section465Input>,
) -> Result<Json<traderview_expense::section_465::Section465Result>, ApiError> {
    if b.activity_loss_this_year < Decimal::ZERO
        || b.cash_and_basis_contributed < Decimal::ZERO
        || b.recourse_debt < Decimal::ZERO
        || b.external_pledged_property_fmv < Decimal::ZERO
        || b.qualified_nonrecourse_financing < Decimal::ZERO
        || b.other_nonrecourse_debt < Decimal::ZERO
        || b.related_party_borrowing < Decimal::ZERO
        || b.prior_year_suspended_loss < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_465::compute(&b)))
}

// ── §401(a)(9) Required Minimum Distributions (RMDs) ─────────────────
// Mounted at /api/calc/section-401a9. SECURE 2.0 age cohorts
// (1949-/1950/1951-1959/1960+), Roth IRA + Roth 401(k) post-2024
// exemptions, Uniform Lifetime Table factors (ages 72-100), §4974
// 25% / 10% correction-window penalty.

async fn section_401a9_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_401a9::Section401a9Input>,
) -> Result<Json<traderview_expense::section_401a9::Section401a9Result>, ApiError> {
    if b.prior_year_end_balance < Decimal::ZERO || b.actual_distribution_taken < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "prior_year_end_balance and actual_distribution_taken must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_401a9::compute(&b)))
}

// ── §409A nonqualified deferred compensation ─────────────────────────
// Mounted at /api/calc/section-409a. §409A(a)(1) three-tier penalty
// (immediate income inclusion + 20% additional tax + premium interest
// IRS rate + 1%); §409A(a)(2)(A) permitted distribution events check;
// §409A(a)(2)(B)(i) specified-employee 6-month delay for public
// companies; §409A(a)(3) anti-acceleration.

async fn section_409a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_409a::Section409aInput>,
) -> Result<Json<traderview_expense::section_409a::Section409aResult>, ApiError> {
    if b.deferred_amount_vested < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "deferred_amount_vested must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_409a::compute(&b)))
}

// ── §382 NOL limitation following ownership change ──────────────────
// Mounted at /api/calc/section-382. §382(b)(1) annual limitation =
// corp FMV × applicable LT tax-exempt rate; §382(g) ownership change
// (> 50% shift among 5%+ shareholders / 3-year testing period);
// §382(l)(5) bankruptcy exception waives the annual limit at the cost
// of a mandatory interest haircut; §382(h) NUBIG recognition can
// increase the limit during the 5-year recognition period. Pairs with
// /api/calc/section-172 for the underlying NOL deduction.

// ── § 354 Exchanges of Stock and Securities in Reorganizations ────
// Mounted at /api/calc/section-354 (iter 526). Pure compute. § 354
// provides general nonrecognition treatment for the EXCHANGE of stock
// and securities pursuant to a § 368(a) reorganization. § 354(a)(1)
// general rule: no gain/loss recognized if stock/securities of corp
// party to reorganization exchanged SOLELY for stock/securities of
// same or another party. § 354(a)(2)(A) securities principal-amount
// boot: if principal of securities received exceeds principal
// surrendered (or securities received with none surrendered), excess
// FMV treated as money (boot) — taxable up to gain realized per
// § 356(a). § 354(a)(2)(B) NQPS (nonqualified preferred stock per
// § 351(g)(2)) treated as boot. § 354(b) additional requirements for
// § 368(a)(1)(D) divisive D split: (1) acquiring corp must acquire
// substantially all assets; (2) transferor must distribute everything
// received plus retained assets. § 354(c) rights/warrants treated as
// securities per Treas. Reg. § 1.354-1(e). Eight reorganization types:
// AStatutoryMerger, BStockForStock, CAssetAcquisitionForVotingStock,
// DDivisiveSplitWithSection354b, ERecapitalization, FMereChange-
// IdentityForm, GBankruptcyReorg, NotASection368Reorganization. Six-
// mode severity ladder: NotApplicable, FullNonrecognitionUnder-
// Section354a1, PartialBootSection354a2aSecuritiesExcess, Nonqualified-
// PreferredStockBootSection354a2b, Section354bAdditionalRequirements-
// NotSatisfied, NotASection368ReorganizationFullRecognition. Basis
// substituted per § 358; holding period tacked per § 1223(1).
// Coordinates with § 368 (reorganization framework — § 354 requires
// § 368(a) plan), § 356 (boot computation), § 358 (substituted basis),
// § 355 (parallel divisive nonrecognition), § 361 (corporate transferor),
// § 362 (acquirer basis), § 367 (foreign corp exchanges override —
// iter 524), § 1223 (holding period tacking), § 351 (corp formation
// parallel), § 332 (parent-subsidiary liquidation), § 1001 (general
// realization).

async fn section_354_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_354::Section354Input>,
) -> Result<Json<traderview_expense::section_354::Section354Result>, ApiError> {
    Ok(Json(traderview_expense::section_354::check(&b)))
}

// ── § 357 Assumption of Liability in Tax-Free Transfers ─────────────
// Mounted at /api/calc/section-357 (iter 564). Pure compute. § 357
// governs how the assumption of liabilities by a transferee corporation
// affects gain recognition by the transferor in § 351 transfers and
// § 368 reorganizations. Companion to § 358 (iter 560 — shareholder
// basis) and § 362 (iter 562 — corporation basis).
//
// § 357(a) general rule: liability assumption does NOT cause boot
// treatment; preserves non-recognition.
//
// § 357(b) tax-avoidance exception: if principal purpose is tax
// avoidance OR no bona-fide business purpose, ALL liabilities assumed
// treated as money received (full boot). Taxpayer bears burden of
// proof by clear preponderance of evidence per Treas. Reg. § 1.357-1(c).
//
// § 357(c)(1) excess-liability gain: gain recognized to extent
// liabilities assumed exceed adjusted basis of property transferred,
// even with bona-fide business purpose. Treas. Reg. § 1.357-2 classic
// example: $20K basis + $30K mortgage → $10K gain.
//
// § 357(c)(2) exceptions: (A) liability whose discharge would give
// rise to deduction (accounts payable from cash-basis trade or
// business), (B) § 736(a) retiring-partner liability.
//
// § 357(d) determination of liability assumed: liability treated as
// assumed only to extent transferor is RELIEVED of it.
//
// Five-mode severity ladder: NotApplicable,
// Section357ANonRecognitionPreservedNoGain,
// Section357BTaxAvoidanceExceptionFullLiabilityTreatedAsBoot,
// Section357C1ExcessLiabilityGainRecognition,
// Section357C2ExceptionAppliesNoExcessLiabilityGain.
//
// Coordinates with § 351 (transfer parent regime), § 358 + § 362
// (basis-preservation companions), § 736(a) (retiring-partner
// payments), § 1245 + § 1250 (depreciation recapture as ordinary
// income on § 357(c)(1) excess gain), § 368(c) 80% control.

async fn section_357_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_357::Section357LiabilityAssumptionInput>,
) -> Result<Json<traderview_expense::section_357::Section357LiabilityAssumptionOutput>, ApiError> {
    Ok(Json(traderview_expense::section_357::check(&b)))
}

// ── § 358 Basis to Distributees (Shareholder Basis in Stock Received) ─
// Mounted at /api/calc/section-358 (iter 560 milestone). Pure compute.
// § 358 governs the basis a shareholder takes in stock + securities
// received in tax-free § 351 corporate formations and § 368 tax-free
// reorganizations. Combined with § 362 corporation-basis side, § 358
// implements double basis preservation that keeps gain or loss latent
// across formation/reorganization transactions.
//
// § 358(a)(1) general rule: shareholder's basis in stock received =
// adjusted basis of property transferred + gain recognized on transfer
// - money received (boot) - FMV of other property received (boot) -
// liabilities assumed (treated as boot under § 358(d)).
//
// § 358(a)(2) basis allocation: among multiple classes of property
// received, basis allocated to each class in proportion to FMV at
// receipt.
//
// § 358(c) reorganizations: tracing method per Federal Register 2006
// final regulations — basis traced from each surrendered share to each
// received share.
//
// § 358(d) liability-as-boot: liabilities assumed by transferee reduce
// shareholder basis; § 357(c)(1) triggers gain recognition if
// liabilities exceed adjusted basis of property transferred.
//
// § 358(h) anti-loss-importation: added Pub. L. 106-554 (2000),
// finalized in 2014 Treasury regs. Shareholder basis REDUCED to FMV
// if liability assumption is part of tax-avoidance scheme to import
// built-in loss into United States. Companion: § 362(e) corporate-level
// anti-loss-importation.
//
// Five-mode severity ladder: NotApplicable,
// Section358InapplicableNotTaxDeferred,
// Section358ABasisPreservedCarryoverWithAdjustments,
// Section358HAntiLossImportationBasisReducedToFmv,
// Section358ABasisReducedBelowZeroSection357C1GainRecognition.
//
// Coordinates with § 351 (transfer to controlled corp), § 354 (basis
// in reorganization stock), § 357 (liability rules + § 357(a)/(b)/(c)),
// § 362 (corporation basis in transferred property), § 368 (reorg
// definitions), § 368(c) 80% control requirement, § 1001 (general
// realization), § 1012 (cost basis general rule).

async fn section_358_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_358::Section358ShareholderBasisInput>,
) -> Result<Json<traderview_expense::section_358::Section358ShareholderBasisOutput>, ApiError> {
    Ok(Json(traderview_expense::section_358::check(&b)))
}

// ── § 362 Basis to Corporations on Tax-Free Transfers ────────────────
// Mounted at /api/calc/section-362 (iter 562). Pure compute. § 362
// governs the basis a TRANSFEREE CORPORATION takes in property received
// in tax-free § 351 transfers, § 368 reorganizations, and contributions
// to capital. Companion to § 358 (iter 560) shareholder basis side;
// together they implement double basis preservation through formation
// and reorganization transactions.
//
// § 362(a) general rule: corp's basis = transferor's adjusted basis +
// gain recognized by transferor under § 351(b).
// § 362(b) reorganizations: same carryover with § 356 gain.
// § 362(c) paid-in surplus / capital contribution by shareholder:
// carryover basis from contributor.
// § 362(d) contribution by non-shareholder: zero basis; TCJA (Pub. L.
// 115-97 § 13312) repealed § 118 exclusion for contributions after
// Dec 22, 2017.
//
// § 362(e)(1) anti-loss-importation rule (American Jobs Creation Act
// of 2004, finalized in 2016 regs effective March 28, 2016): if
// property would be subject to US tax in transferee's hands but was
// NOT subject in transferor's hands (foreign / tax-exempt) AND
// transferor's aggregate basis exceeds aggregate FMV, transferee's
// basis stepped down to FMV per § 1.362-3. Companion § 358(h) at
// shareholder level + § 334(b)(1)(B) parallel for § 332 liquidations.
//
// § 362(e)(2) anti-loss-duplication rule (2004): § 351 transfers of
// net-built-in-loss property — transferee's aggregate basis reduced
// to aggregate FMV; allocation per § 1.362-4 in proportion to
// built-in losses. § 362(e)(2)(C) joint election: transferor and
// transferee may instead reduce transferor's stock basis (preserving
// corporation's carryover basis); strategic for preserving tax
// attributes (NOLs, depreciation, capital-loss carryforward).
//
// Eight-mode severity ladder: NotApplicable,
// Section362ACarryoverBasisStandardSection351,
// Section362BCarryoverBasisReorganization,
// Section362CPaidInSurplusCarryover,
// Section362DContributionByNonShareholderZeroOrTcjaTreatment,
// Section362E1ImportationBasisSteppedDownToFmv,
// Section362E2DuplicationCorpBasisSteppedDownToFmv,
// Section362E2CJointElectionTransferorStockBasisReducedCorpKeepsCarryover.
//
// Coordinates with § 351 (transfer parent regime), § 357 (liability
// rules), § 358 (iter 560 — shareholder basis side), § 368 (reorg
// definitions + § 368(c) 80% control), § 1223 (holding-period
// tacking), § 1245 + § 1250 (depreciation recapture carryover),
// § 334(b)(1)(B) (§ 332 liquidation parallel).

async fn section_362_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_362::Section362CorporationBasisInput>,
) -> Result<Json<traderview_expense::section_362::Section362CorporationBasisOutput>, ApiError> {
    Ok(Json(traderview_expense::section_362::check(&b)))
}

// ── § 367 Foreign Corporations ────────────────────────────────────────
// Mounted at /api/calc/section-367 (iter 524). Pure compute. § 367
// overrides nonrecognition treatment that would otherwise apply under
// § 332 + § 351 + § 354 + § 355 + § 356 + § 361 + § 721 when a transfer
// involves a foreign corporation — prevents US shareholders from using
// cross-border reorganizations to permanently avoid US tax on appreciated
// property. § 367(a)(1) outbound transfer general rule: US person
// transferring property to foreign corp in § 332/351/354/356/361
// exchange treated as TAXABLE exchange at FMV. § 367(a)(2) exception
// for stock/securities of foreign corp party to exchange. § 367(a)(8)
// gain-recognition agreement (GRA) — Treas. Reg. § 1.367(a)-8 5-year
// deferral. TCJA § 14102(e) effective transfers after Dec 31 2017
// REPEALED former § 367(a)(3) active trade or business exception.
// § 367(b) inbound transfer regs (foreign corp → domestic corp in
// § 332 liquidation or § 368(a)(1) reorg) — domestic acquiring corp
// includes foreign corp's post-1986 accumulated E&P as deemed dividend
// per Treas. Reg. § 1.367(b)-3. Coordinates with Notice 2024-16
// § 961(c) basis carryover. § 367(d) outbound intangible transfer:
// treated as deemed annual royalty over useful life; TCJA § 14221
// effective Dec 31 2017 expanded § 936(h)(3)(B) intangible definition
// to include goodwill + going-concern value + workforce-in-place +
// any item value not attributable to tangible property or individual
// services. § 367(e) distributions to foreign corp shareholders. Form
// 926 filing required. Six-mode severity ladder: NotApplicable,
// OutboundTangibleFullGainRecognition, OutboundStockGainRecognition-
// AgreementAvailable, OutboundIntangibleDeemedRoyalty, InboundEarnings-
// AndProfitsInclusion, ForeignToForeignNoUsConsequence. Coordinates
// with § 332 + § 351 + § 354 + § 355 + § 361 + § 368 + § 721 + § 951A
// + § 956 + § 959 (PTEP) + § 960 (FTC) + § 961 (CFC basis Notice
// 2024-16 carryover — iter 522) + § 245A (DRD) + § 1248 (CFC stock
// gain recharacterization).

async fn section_367_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_367::Section367Input>,
) -> Result<Json<traderview_expense::section_367::Section367Result>, ApiError> {
    Ok(Json(traderview_expense::section_367::check(&b)))
}

async fn section_382_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_382::Section382Input>,
) -> Result<Json<traderview_expense::section_382::Section382Result>, ApiError> {
    if b.corporation_fmv_at_change < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "corporation_fmv_at_change must be >= 0".into(),
        ));
    }
    if b.pre_change_nol_carryover < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "pre_change_nol_carryover must be >= 0".into(),
        ));
    }
    if b.mandatory_interest_haircut_l5 < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "mandatory_interest_haircut_l5 must be >= 0".into(),
        ));
    }
    if b.recognized_built_in_gain_this_year < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "recognized_built_in_gain_this_year must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_382::compute(&b)))
}

// ── § 383 Special Limitations on Certain Excess Credits + Capital Loss ─
// Mounted at /api/calc/section-383 (iter 540). Pure compute. § 383
// extends the § 382 ownership-change annual limitation regime to four
// categories of corporate carryover attributes outside § 382: (1)
// general business credits under § 39, (2) minimum tax credits under
// § 53, (3) net capital loss carryovers under § 1212, (4) excess
// foreign tax credits under § 904(c). Mechanic: compute tax that
// WOULD have been due on taxable income equal to the § 382 limit,
// then cap pre-change credit/loss usage against that ceiling.
//
// § 383(a) general business credit + minimum tax credit regime:
// post-change use limited to tax attributable to income within § 382
// limit. § 383(b) net capital loss carryover: § 1212(a) carryover
// limited under regulations based on § 382 principles, with
// anti-stuffing rule (capital loss used in post-change year REDUCES
// the § 382 limit applied to pre-change NOLs in that same year).
// § 383(c) excess foreign tax credit: § 904(c) excess FTC carryover
// limited consistent with § 382 + § 383 principles. § 383(d) ordering:
// terms used in § 383 have same meaning as § 382 with adjustments.
//
// Six-mode severity ladder: NotApplicable,
// NoOwnershipChangeFullAttributeUsageAllowed,
// Section383CreditLimitationApplied,
// Section383BCapitalLossLimitationAppliedAntiStuffing,
// Section383CExcessFtcLimitationApplied,
// AttributeFullyUsedWithinSection383Limit.
//
// Coordinates with § 382 (NOL annual cap parent regime), § 384
// (iter 538 — preacquisition-loss built-in-gain disallowance), § 269
// (iter 536 — discretionary disallowance), § 1212 (capital loss
// carryover), § 39 (general business credit), § 53 (minimum tax
// credit), § 904(c) (FTC carryover).

async fn section_383_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_383::Section383ExcessCreditLimitationInput>,
) -> Result<Json<traderview_expense::section_383::Section383ExcessCreditLimitationOutput>, ApiError>
{
    Ok(Json(traderview_expense::section_383::check(&b)))
}

// ── § 384 Limitation on Preacquisition Losses to Offset Built-In Gain ─
// Mounted at /api/calc/section-384 (iter 538). Pure compute. § 384
// prevents a "loss corporation" from offsetting "recognized built-in
// gain" of an acquired profitable corporation (the "gain corporation")
// during a 5-year recognition period. The provision plugs the converse
// hole § 382 leaves open: § 382 stops a loss-corp from absorbing the
// profitable-corp's POST-acquisition income; § 384 stops the loss-corp
// from absorbing the gain-corp's PRE-acquisition built-in gain that
// happens to be recognized within the 5-year window.
//
// § 384(a) general rule: if a corporation acquires control of another
// corporation OR the assets of a corporation are acquired in a § 368
// reorganization AND either corporation is a "gain corporation," income
// attributable to RECOGNIZED BUILT-IN GAIN cannot be offset by any
// "preacquisition loss" — except the gain corp's OWN preacquisition
// loss (statutory carveout).
//
// § 384(a)(1) recognition period: 5 years beginning on the acquisition
// date. § 384(b)(2) control threshold: § 1504(a)(2) standard — at least
// 80% voting power AND 80% value. § 384(b)(3) common-control exception:
// inapplicable where loss corp and gain corp were in same controlled
// group (more than 50% per § 384(b)(3) modified § 1563(a)) for the
// 5-year period ending on the acquisition date.
//
// § 384(c)(8) preacquisition-loss definition: NOL carryforward to year
// of acquisition + NOL for year of acquisition allocable to
// pre-acquisition portion + capital loss carryover + general business
// credit carryforward + foreign tax credit carryover.
//
// Six-mode severity ladder: NotApplicable,
// NoQualifyingAcquisitionSection384Inapplicable,
// CommonControlExceptionAppliesSection384bThreeNoDisallowance,
// BuiltInGainRecognizedAfterFiveYearWindowNoDisallowance,
// NoPreacquisitionLossNoDisallowance,
// Section384APreacquisitionLossOffsetDisallowed.
//
// Coordinates with § 382 (annual NOL cap post-ownership-change), § 383
// (general-business-credit cap), § 269 (discretionary disallowance on
// principal-purpose-of-tax-avoidance), § 381 (carryover-attribute
// transferee rules), § 1504 (affiliated-group control standard).

async fn section_384_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_384::Section384PreacquisitionLossDisallowanceInput>,
) -> Result<
    Json<traderview_expense::section_384::Section384PreacquisitionLossDisallowanceOutput>,
    ApiError,
> {
    Ok(Json(traderview_expense::section_384::check(&b)))
}

// ── §83(i) qualified equity grant 5-year income-tax deferral ────────
// Mounted at /api/calc/section-83i. TCJA addition; defers federal
// income tax (NOT FICA) up to 5 years on NQSO exercise / RSU vesting
// for eligible employees of eligible private corporations. §83(i)(2)(C)
// eligible-corp test (no tradable stock + 80% broad-based written
// plan); §83(i)(3)(B) excluded-employee exclusions (1% owner, CEO/CFO,
// top-4 paid in current or any 10 prior years); §83(i)(1)(B) deferral
// end triggers (5y max, IPO/tradable, buyback, revocation, becoming
// excluded — earliest wins); §83(i)(4)(A) 30-day election window.

async fn section_83i_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_83i::Section83iInput>,
) -> Result<Json<traderview_expense::section_83i::Section83iResult>, ApiError> {
    if b.deferred_income_amount < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "deferred_income_amount must be >= 0".into(),
        ));
    }
    if b.fmv_at_vesting_for_fica < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "fmv_at_vesting_for_fica must be >= 0".into(),
        ));
    }
    if b.as_of_date < b.vesting_or_exercise_date {
        return Err(ApiError::BadRequest(
            "as_of_date must be on or after vesting_or_exercise_date".into(),
        ));
    }
    Ok(Json(traderview_expense::section_83i::compute(&b)))
}

// ── §408(m) collectibles in IRA ──────────────────────────────────────
// Mounted at /api/calc/section-408m. Pure compute; §408(m)(1)
// prohibited collectible = deemed distribution; §408(m)(3)(A)
// Eagle / state-issued coin exception; §408(m)(3)(B) bullion
// exception with purity threshold (.995 gold / .999 silver / .9995
// platinum / .9995 palladium) AND trustee custody.

async fn section_408m_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_408m::Section408mInput>,
) -> Result<Json<traderview_expense::section_408m::Section408mResult>, ApiError> {
    if b.purchase_price < Decimal::ZERO {
        return Err(ApiError::BadRequest("purchase_price must be >= 0".into()));
    }
    Ok(Json(traderview_expense::section_408m::compute(&b)))
}

// ── §41 R&D credit (Regular + Alternative Simplified + §280C(c)) ─────
// Mounted at /api/calc/section-41. Practical for algorithmic traders
// building custom trading systems + data pipelines + ML models that
// qualify as research under §41(d). Two computation methods:
// §41(a)(1) Regular Credit = 20% × (QRE − base amount) where base =
// max(fixed-base-% × 4-year avg gross receipts, 50% × current QRE);
// fixed-base-% capped at 16% under §41(c)(3); startup uses 3%.
// §41(c)(4) Alternative Simplified Credit (ASC) = 14% × (QRE − 50% ×
// prior 3-year avg QRE); §41(c)(4)(B) startup path 6% × current QRE
// when no QRE in any of 3 prior years. §280C(c)(2) reduced-credit
// election reduces credit by 21% in exchange for keeping full §174
// deduction (§280C(c)(3) election must be on original return).

async fn section_41_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_41::Section41Input>,
) -> Result<Json<traderview_expense::section_41::Section41Result>, ApiError> {
    if b.current_year_qre_cents < 0
        || b.prior_3_year_avg_qre_cents < 0
        || b.prior_4_year_avg_gross_receipts_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_41::compute(&b)))
}

async fn section_38_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_38::Section38Input>,
) -> Result<Json<traderview_expense::section_38::Section38Result>, ApiError> {
    Ok(Json(traderview_expense::section_38::check(&b)))
}

async fn section_42_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_42::Section42Input>,
) -> Result<Json<traderview_expense::section_42::Section42Result>, ApiError> {
    Ok(Json(traderview_expense::section_42::check(&b)))
}

async fn section_44_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_44::Section44Input>,
) -> Result<Json<traderview_expense::section_44::Section44Result>, ApiError> {
    Ok(Json(traderview_expense::section_44::check(&b)))
}

// ── §408(d)(3) IRA 60-day rollover rules ─────────────────────────────
// Mounted at /api/calc/section-408-d3. Pure compute; validates that
// an indirect IRA rollover satisfies (a) 60-day deposit window,
// (b) Bobrow once-per-12-months aggregated across all IRAs, with
// §408(d)(3)(I) hardship-waiver path and §72(t) 10% early withdrawal
// penalty calculation on failed rollovers.

async fn section_408_d3_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_408_d3::Section408D3Input>,
) -> Result<Json<traderview_expense::section_408_d3::Section408D3Result>, ApiError> {
    if b.distribution_amount < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "distribution_amount must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_408_d3::compute(&b)))
}

// ── §871(m) dividend-equivalent withholding for non-US persons ───────
// Mounted at /api/calc/section-871m. Pure compute; classifies a US-
// equity-linked derivative as a Specified Equity-Linked Instrument
// (SELI) based on delta + original term, applies statutory 30% or
// treaty-reduced rate.

async fn section_871m_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_871m::Section871MInput>,
) -> Result<Json<traderview_expense::section_871m::Section871MResult>, ApiError> {
    if b.dividend_equivalent_amount < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "dividend_equivalent_amount must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_871m::compute(&b)))
}

// ── §911 foreign earned income exclusion ─────────────────────────────
// Mounted at /api/calc/section-911. §911(a)(1) FEIE inflation-indexed
// (2025 $130k / 2026 $132,900 caller-supplied year-agnostic) + §911(a)(2)
// housing exclusion + §911(b)(1) foreign earned income definition (no
// US-gov / passive / pension) + §911(c)(2) housing cap 30% × FEIE +
// §911(d)(1)(A) bona fide residence test + §911(d)(1)(B) physical
// presence test ≥ 330 full days + §911(d)(7) base housing 16% × FEIE.

// ── § 901 Taxes of Foreign Countries / Foreign Tax Credit ───────────
// Mounted at /api/calc/section-901 (iter 518). Pure compute. § 901 is
// the operative FTC provision — allows credit for foreign income, war
// profits, and excess profits taxes paid or accrued to foreign
// governments or US possessions. § 901(a) general rule for domestic
// corp + US citizen / resident alien individuals; § 901(b)(1)
// creditable for 10pct-or-more foreign corp shareholder via § 960
// deemed-paid mechanism. § 901(j) sanctioned-country disallowance
// (Iran + North Korea + Syria + Cuba + Sudan partial per Secretary
// of State designation) plus § 901(j)(5) treaty resourcing carve-out
// plus separate § 904(d) basket. § 901(k) dividend holding-period
// requirement: common stock + preferred short-period = 16 days in
// 31-day window; preferred long-period (dividends > 366 days) = 46
// days in 91-day window. § 901(l) holding-period for other income.
// § 901(m) covered asset acquisition disallowance — Pub. L. 111-226
// (2010) — disqualified portion = (US basis step-up / total basis) ×
// foreign tax; Notice 2014-44 + Notice 2014-45 + Treas. Reg. § 1.901(m)-
// 1 through § 1.901(m)-8 (proposed) implement. § 901(b)(5) US-
// possessions analogous credit (Puerto Rico + USVI + Guam + American
// Samoa + Northern Mariana Islands). Six-mode severity ladder:
// NotApplicable, FullyCreditable, PartiallyCreditableCaaDisqualified-
// Portion, NonCreditableSanctionedCountryFull, NonCreditableHolding-
// PeriodFailed, NonCreditableNonIncomeTax (VAT + customs + penalty +
// interest — deductible under § 164(a)(3) instead). Coordinates with
// § 904 (FTC limitation — iter 516), § 960 (deemed-paid FTC), § 903
// (in-lieu-of tax extension), § 245A (DRD — iter 502 § 245A(d) FTC
// disallowance), § 951A (GILTI/NCTI — iter 500), § 956 (CFC US
// property — iter 504), § 959 (PTEP — iter 512), § 962 (individual
// election — iter 510), § 965 (transition tax § 965(g) FTC denial
// percentage — iter 514), § 164 (foreign tax deduction alternative).

async fn section_901_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_901::Section901Input>,
) -> Result<Json<traderview_expense::section_901::Section901Result>, ApiError> {
    Ok(Json(traderview_expense::section_901::check(&b)))
}

// ── § 903 In-Lieu-Of Tax Creditability ───────────────────────────────
// Mounted at /api/calc/section-903 (iter 528). Pure compute. § 903 extends
// § 901 FTC creditability to foreign taxes that are NOT a generally-imposed
// foreign income/war-profits/excess-profits tax but SUBSTITUTE for such a
// tax (the "in-lieu-of" branch). Classic application: foreign withholding
// tax on services/royalties where the foreign jurisdiction waives its
// generally-imposed net-income tax in favor of a gross-basis withholding
// levy. Treas. Reg. § 1.903-1(c)(2) SUBSTITUTION TEST: in-lieu-of tax must
// substitute for an income tax that would otherwise be imposed on the same
// taxpayer/income; additive levies that supplement (rather than replace)
// the underlying income tax fail substitution. Treas. Reg. § 1.903-1(c)
// (1)(iii) SOAK-UP RULE: foreign tax whose liability depends on availability
// of US foreign tax credit (only payable to extent FTC is allowed) is
// non-creditable regardless of substitution — disqualifying ahead of all
// other tests because the soak-up structure absorbs the credit it claims
// to enable. TD 9959 FINAL REGULATIONS (effective January 4, 2022) added
// "attribution requirement" to § 903: the generally-imposed income tax
// which the levy substitutes for must independently satisfy sourcing-nexus
// attribution rules. Notice 2023-55 (July 21, 2023) deferred attribution
// requirement for tax years ending on or before December 31, 2023.
// Notice 2025-23 (per practitioner reporting) extended deferral through
// tax years ending in 2024 and 2025; verify against IRS published guidance
// before relying. Six-mode severity ladder: NotApplicable, Creditable-
// UnderInLieuOfBranchAttributionDeferred, CreditableUnderInLieuOfBranch-
// AttributionMet, NonCreditableSoakUpTaxRule, NonCreditableFailsSubstitution,
// NonCreditableFailsAttributionPostDeferral. Disallowed foreign tax remains
// deductible under § 164(a)(3) in lieu of credit. Creditable amount remains
// subject to § 904(d) basket limitations and § 904(c) carryover rules.
// Coordinates with § 901 (net-basis branch) + § 904 (limitation) + § 164
// (deduction alternative) + § 960 (deemed-paid for § 951A inclusions where
// § 903 in-lieu-of withholding occurs at CFC level).

async fn section_903_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_903::Section903InLieuOfTaxCreditabilityInput>,
) -> Result<Json<traderview_expense::section_903::Section903InLieuOfTaxCreditabilityOutput>, ApiError>
{
    Ok(Json(traderview_expense::section_903::check(&b)))
}

// ── § 904 Limitation on Foreign Tax Credit (FTC) ─────────────────────
// Mounted at /api/calc/section-904 (iter 516). Pure compute. § 904 caps
// the foreign tax credit at the US tax that would otherwise be imposed
// on foreign-source taxable income. § 904(a) formula: FTC ≤ US tax ×
// (foreign-source TI / total worldwide TI). § 904(d) SEPARATE BASKET
// RULE — limitation computed SEPARATELY for each post-TCJA basket
// (effective taxable years beginning after December 31, 2017): (1)
// Passive (dividends + interest + royalties + rents + annuities — with
// high-tax kick-out and CFC look-through), (2) GILTI / NCTI (post-OBBBA
// renamed — § 951A inclusions), (3) Foreign Branch (qualified business
// unit income), (4) General (active business + wages + financial
// services), (5) Treaty-Resourced (§ 904(d)(6)), (6) § 901(j)
// sanctioned-country income (Iran + North Korea + Syria + Cuba + Sudan
// partial), (7) Lump-Sum Distribution from foreign pension. § 904(c)
// CARRYOVER: 1-year carryback + 10-year carryforward within same basket
// — but § 951A GILTI/NCTI basket EXCLUDED from carryover per § 904(c)(1)
// flush language (excess credits expire annually). § 904(f) OFL recapture:
// prior-year foreign-source losses allocated against US-source income;
// later years foreign-source income recharacterized as US-source until
// OFL fully recaptured. § 904(g) ODL recapture: parallel for domestic
// losses. OBBBA (Pub. L. 119-21, effective for taxable years beginning
// after December 31, 2025): interest + R&D expense no longer allocated
// to § 951A basket per § 864(e); § 960(d) deemed-paid FTC rate rises from
// 80% to 90% (10% haircut down from 20%). Seven-mode severity ladder:
// NotApplicable, FullyCreditedNoExcess, PartiallyCreditedExcessCarried-
// Forward, GiltiNctiExcessExpired, OverallForeignLossRecaptureTriggered,
// OverallDomesticLossRecaptureTriggered, Section901jSanctionedNon-
// Creditable. Form 1116 (individual) or Form 1118 (corporate) plus
// Schedule J OFL tracking. Coordinates with § 901 + § 960 + § 951A
// + § 956 + § 959 (PTEP — sixteen-basket framework maintained within
// each § 904(d) basket — iter 512) + § 962 + § 245A + § 965 + § 59A +
// § 864(e) (expense allocation).

async fn section_904_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_904::Section904Input>,
) -> Result<Json<traderview_expense::section_904::Section904Result>, ApiError> {
    Ok(Json(traderview_expense::section_904::check(&b)))
}

async fn section_911_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_911::Section911Input>,
) -> Result<Json<traderview_expense::section_911::Section911Result>, ApiError> {
    if b.feie_inflation_adjusted_cap_dollars < 0
        || b.foreign_earned_income_dollars < 0
        || b.housing_expenses_dollars < 0
        || b.physical_presence_days_in_12_month_period > 366
    {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs and days in 12-month period ≤ 366 required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_911::compute(&b)))
}

// ── § 951A GILTI / NCTI (Global Intangible Low-Taxed Income / Net CFC
// Tested Income) ─────────────────────────────────────────────────────
// Mounted at /api/calc/section-951a (iter 500 milestone). Pure compute.
// TCJA 2017 Pub. L. 115-97 § 14201 enacted GILTI effective for foreign
// corp taxable years beginning after December 31, 2017. OBBBA Pub. L.
// 119-21 signed July 4, 2025 effective for tax years beginning after
// December 31, 2025: (1) renames GILTI to Net CFC Tested Income (NCTI);
// (2) REPEALS QBAI 10% deduction (full tested income inclusion); (3)
// permanently sets § 250 deduction to 40% (was 50% pre-2026, scheduled
// to 37.5%); (4) reduces § 960(d) FTC haircut from 20% to 10% (90%
// FTCs allowed); (5) excludes interest + R&E expense from GILTI / NCTI
// basket allocation per § 864(e) clarification. Pre-OBBBA effective
// rate ~10.5%; post-OBBBA effective rate ~12.6% at 21% corporate rate.
// Six-mode severity ladder: NotApplicable, NotUsShareholderNoInclusion,
// NotACfcNoInclusion, Pre2026GiltiInclusionWithQbai, Post2026Ncti-
// InclusionNoQbai, TestedLossNoCurrentInclusion. Coordinates with §
// 250 (FDII / GILTI / NCTI deduction), § 59A (BEAT — separate base-
// erosion regime), § 56A (CAMT — CFC income flows through AFSI), §
// 988 (forex on CFC distributions), § 1297 (PFIC mutually exclusive),
// § 911 (foreign earned income exclusion — individual regime), § 962
// (election for individual / S-corp / partnership shareholder to be
// taxed at corporate rate plus claim § 250 + § 960(d) FTC). Form 5471
// + Form 8992 reporting.

async fn section_951a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_951a::Section951aInput>,
) -> Result<Json<traderview_expense::section_951a::Section951aResult>, ApiError> {
    Ok(Json(traderview_expense::section_951a::check(&b)))
}

// ── § 956 Investment of CFC Earnings in United States Property ──────
// Mounted at /api/calc/section-956 (iter 504). Pure compute. § 956
// anti-deferral rule treats CFC investment in US property (tangible
// property in US, domestic-corp stock, US-person obligations including
// pledges + guarantees per Rev. Rul. 90-112 / Notice 88-108, US-source
// intangible rights) as constructive distribution to US shareholders.
// § 956(a) caps inclusion at CFC E&P. § 956(c)(2) statutory exceptions:
// US bank deposits, export property, shipping property, insurance
// reserves, certain securities, aircraft/vessels in international
// commerce, related-US-person working capital. **§ 245A coordination
// rule** (Treas. Reg. § 1.956-1(a)(2)-(4), effective for CFC tax years
// beginning on or after July 22, 2019): CORPORATE US shareholder's §
// 956 inclusion REDUCED by hypothetical-distribution offset to extent
// § 245A 100% DRD would apply. Ordering: hypothetical distribution
// attributable first to § 959(c)(2) PTEP then § 959(c)(3) non-PTEP
// E&P. Non-corporate shareholders (individuals, RICs, REITs, S corps,
// partnerships) DO NOT benefit absent § 962 election. Form 5471
// Schedule I-1. Coordinates with § 951A (GILTI/NCTI residual), § 245A
// (DRD hypothetical basis), § 1297 (PFIC mutually exclusive), § 959
// (PTEP ordering), § 962 (election for individual shareholder), § 59A
// (BEAT separate regime).

async fn section_956_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_956::Section956Input>,
) -> Result<Json<traderview_expense::section_956::Section956Result>, ApiError> {
    Ok(Json(traderview_expense::section_956::check(&b)))
}

// ── § 959 Exclusion from Gross Income of Previously-Taxed Earnings
// and Profits (PTEP) ─────────────────────────────────────────────────
// Mounted at /api/calc/section-959 (iter 512). Pure compute. § 959
// prevents double US taxation of CFC earnings already included under
// § 951(a) Subpart F + § 951A GILTI/NCTI + § 956 US-property
// investment provisions. § 959(c) three-category distribution
// ordering: (c)(1) US property investment-related PTEP first, then
// (c)(2) Subpart F + GILTI/NCTI PTEP, finally (c)(3) untaxed E&P
// triggering taxable dividend. § 961 stock basis adjustment: § 961(a)
// basis increase on PTEP inclusion + § 961(b) basis decrease on PTEP
// distribution. Notice 2019-01 (December 14, 2018) sixteen-basket
// PTEP framework: 9 § 959(c)(1) groups + 7 § 959(c)(2) groups within
// each § 904 FTC category. Proposed Regs REG-105479-18 (published
// December 2, 2024 in Federal Register, signed November 29, 2024)
// implement Notice 2019-01 framework plus shareholder-level + CFC-
// level accounting. Notice 2024-16 additional § 961(c) basis in §
// 332 liquidation + § 368(a)(1) asset reorg regulations forthcoming.
// Form 5471 Schedule J + Schedule P PTEP reporting. Six-mode severity
// ladder: NotApplicable, NotUsShareholderNoExclusion, FullyAttributable-
// ToSection959C1PtepExcluded, FullyAttributableToSection959C2Ptep-
// Excluded, PartiallyAttributableNonPtepTaxableRemainder, FullyAttributable-
// ToNonPtepDividend. Coordinates with § 951 (Subpart F inclusion → PTEP
// (c)(2)), § 951A (GILTI/NCTI inclusion → PTEP (c)(2)), § 956 (US
// property → PTEP (c)(1) + reclassification of (c)(2) to (c)(1)), § 962
// (§ 962 E&P treated as PTEP per § 962(d)), § 245A (DRD pathway for
// non-PTEP foreign-source portion — iter 502), § 961 (basis), § 965
// (transition tax → § 959(c)(2) PTEP), § 904 (FTC limitation baskets).

async fn section_959_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_959::Section959Input>,
) -> Result<Json<traderview_expense::section_959::Section959Result>, ApiError> {
    Ok(Json(traderview_expense::section_959::check(&b)))
}

// ── § 962 Election by Individuals to Be Subject to Tax at Corporate
// Rates ─────────────────────────────────────────────────────────────
// Mounted at /api/calc/section-962 (iter 510). Pure compute. § 962
// permits individual US shareholder (plus estate plus US trust) of CFC
// to elect taxation at corporate § 11 rate (21%) on (1) Subpart F
// inclusions under § 951(a), (2) GILTI / NCTI inclusions under § 951A,
// (3) § 956 US property investment inclusions — instead of individual
// marginal rate (up to 37%). Election unlocks § 250 GILTI / NCTI
// deduction (50% pre-OBBBA / 40% post-OBBBA) plus § 960 deemed-paid
// FTC (80% pre-OBBBA / 90% post-OBBBA per Pub. L. 119-21 effective
// for taxable years beginning after December 31 2025). § 962(b)
// ceiling rule. § 962(d) ACTUAL DISTRIBUTION RULE — multi-year
// consideration: when CFC later makes actual distribution of § 962
// E&P, includible in gross income to extent EXCEEDS cumulative US tax
// paid under § 962 in prior years; SECOND layer of tax at qualified
// dividend / treaty preferential rate. § 245A 100% DRD NOT available
// to § 962 electors per Treas. Reg. § 1.245A-5(b)(2) — election
// forecloses participation-exemption pathway. Treas. Reg. § 1.962-
// 2(b)(1) annual election via Form 1040 statement; election applies
// to ALL CFCs of shareholder. Rev. Rul. 2019-10 permits partner /
// S corp shareholder to elect at individual level for flowed-through
// inclusions. Six-mode severity ladder: NotApplicable, NotEligible-
// ShareholderType (partnership entity-level / domestic C corp),
// ElectionNotMadeIndividualRateApplies, ElectionMadeCurrentYear-
// Benefit, ActualDistributionExceedsTaxPaidSecondLayerTriggered,
// ActualDistributionWithinTaxPaidNoSecondLayer. Coordinates with
// § 951 (Subpart F), § 951A (GILTI/NCTI), § 956 (US property),
// § 245A (DRD pathway alternative — mutually exclusive with § 962
// on actual distribution), § 250 (deduction), § 960 (FTC), § 59A
// (BEAT separate regime), § 911 (FEIE individual regime).

// ── § 960 Deemed-Paid Credit for Subpart F + GILTI/NCTI + PTEP ──────
// Mounted at /api/calc/section-960 (iter 520). Pure compute. § 960
// grants domestic corporate US shareholders deemed-paid FTC for foreign
// taxes paid by CFC attributable to amounts US shareholder includes in
// income. § 960(a) Subpart F deemed-paid (full creditability). § 960(b)
// PTEP distribution credit. § 960(c) limits mechanism to domestic C
// corp + § 962-electing individuals. § 960(d) GILTI/NCTI deemed-paid:
// pre-OBBBA 80% × inclusion-pct × tested-foreign-income-taxes ("GILTI
// haircut"); OBBBA Pub. L. 119-21 raises to 90% effective taxable years
// beginning after December 31 2025. § 960(d)(4) PTEP DISTRIBUTION
// HAIRCUT (OBBBA): disallows 10% of foreign taxes paid or accrued
// (incl. § 960(b)(1) deemed-paid) with respect to § 959(a) PTEP
// distribution where PTEP results from § 951A inclusion in US
// shareholder taxable year ending after June 28 2025. Notice 2025-77
// interim guidance pending proposed regulations; taxpayers may rely
// on Section 3 of Notice 2025-77. Treas. Reg. § 1.960-1 + § 1.960-2 +
// § 1.960-3 implement; Proposed Regs REG-105479-18 (Nov 29 2024 /
// pub Dec 2 2024) modify under Notice 2019-01 sixteen-basket PTEP
// framework. Seven-mode severity ladder: NotApplicable, NotEligible-
// NoSection962Election (individual/S corp/partnership without § 962),
// SubpartFFullDeemedPaidCredit, GiltiNcti80PctPreObbba, GiltiNcti90Pct-
// PostObbba, PtepDistributionFullCredit, PtepDistributionWithGilti-
// HaircutOpbba. Form 1118 Schedule C (corp) + Form 1116 (individual
// with § 962 election). Coordinates with § 901 (FTC operative — iter
// 518), § 904 (FTC limitation by basket — iter 516), § 951 (Subpart F),
// § 951A (GILTI/NCTI — iter 500), § 956 (CFC US property — iter 504),
// § 959 (PTEP — iter 512), § 962 (individual election — iter 510),
// § 245A (DRD — iter 502 § 245A(d) FTC disallowance), § 965 (transition
// tax — iter 514 § 965(g) FTC denial), § 902 (REPEALED by TCJA for
// foreign corp years beginning after Dec 31 2017).

async fn section_960_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_960::Section960Input>,
) -> Result<Json<traderview_expense::section_960::Section960Result>, ApiError> {
    Ok(Json(traderview_expense::section_960::check(&b)))
}

// ── § 961 Basis Adjustments for CFC Stock ──────────────────────────
// Mounted at /api/calc/section-961 (iter 522). Pure compute. § 961
// establishes stock-basis tracking that prevents double US taxation
// of CFC earnings: § 961(a) PTEP inclusion (§ 951(a) Subpart F + §
// 951A GILTI/NCTI + § 956 US property) INCREASES basis; § 961(b)(1)
// actual PTEP distribution under § 959(a) DECREASES basis; § 961(b)(2)
// distribution exceeding basis = § 301(c)(3) capital gain (basis cannot
// go negative). § 961(c) indirectly-owned CFC stock basis limited to
// § 951 inclusion determination only — not for gain/loss recognition
// on intermediate stock disposition. Notice 2024-16 (January 16,
// 2024) carryover rule: in qualifying inbound § 332 liquidation
// or § 368(a)(1) asset reorg where domestic acquiring corp receives
// CFC stock from another domestic corp, § 961(c) basis CARRIES OVER
// to acquiror — prevents trapped-PTEP gain on subsequent distribution.
// Proposed Regs REG-105479-18 (Nov 29 2024 / pub Dec 2 2024) implement
// § 959 sixteen-basket PTEP framework + § 961 basis tracking + § 962
// election + currency translation + S corp PTEP + consolidated group
// PTEP + anti-avoidance rules. Six basis adjustment events: PtepInclusion-
// SubpartF, PtepInclusionGiltiOrNcti, PtepInclusionSection956, Actual-
// PtepDistribution, Section332InboundLiquidation, Section368Asset-
// Reorganization. Six-mode severity ladder: NotApplicable, BasisIncrease-
// UnderSection961a, BasisDecreaseUnderSection961b, BasisFloorExcess-
// DistributionGainSection961b2, InboundNonrecognitionCarryoverUnder-
// Notice2024_16, IndirectlyOwnedSection961cLimitedToSection951Inclusion.
// Form 5471 Schedule J + Schedule P (PTEP) + Schedule D for § 961(b)(2)
// gain. Coordinates with § 959 (PTEP — iter 512), § 960 (deemed-paid
// FTC — iter 520), § 951 + § 951A + § 956 + § 962 (individual election
// — iter 510), § 245A (DRD — iter 502), § 965 (transition tax — iter
// 514), § 301 (corporate distribution framework — § 301(c)(3)), § 332
// + § 368 (Notice 2024-16 carryover).

async fn section_961_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_961::Section961Input>,
) -> Result<Json<traderview_expense::section_961::Section961Result>, ApiError> {
    Ok(Json(traderview_expense::section_961::check(&b)))
}

async fn section_962_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_962::Section962Input>,
) -> Result<Json<traderview_expense::section_962::Section962Result>, ApiError> {
    Ok(Json(traderview_expense::section_962::check(&b)))
}

// ── § 965 Transition Tax / Mandatory Repatriation Tax (MRT) ──────────
// Mounted at /api/calc/section-965 (iter 514). Pure compute. TCJA 2017
// Pub. L. 115-97 § 14103 ONE-TIME deemed-repatriation tax on accumulated
// post-1986 deferred foreign earnings of CFCs and 10pct-or-greater
// foreign corps (SFCs); applies in last taxable year of SFC beginning
// before January 1, 2018. § 965(a)(2) measurement at greater of
// November 2, 2017 (TCJA passage) or December 31, 2017 (end of pre-
// TCJA tax year). § 965(c) rate differentiation via deduction: 15.5pct
// effective on cash position + 8pct on non-cash. § 965(b) E&P deficits
// of other SFCs reduce aggregate inclusion. § 965(h) 8-year installment
// election: 8pct years 1-5, 15pct year 6, 20pct year 7, 25pct year 8 —
// sums to 100pct. § 965(i) S corp shareholder deferral until triggering
// event (S corp termination + asset sale + liquidation + stock transfer).
// § 965(m) REIT ratable 8-year inclusion coordinating with § 857 90pct
// distribution requirement. Moore v. United States 602 U.S. ___ (June
// 20, 2024) 7-2 decision authored by Justice Kavanaugh UPHELD constitutionality
// under Sixteenth Amendment; holding NARROW (does not decide whether
// realization is constitutional requirement). Inclusion creates §
// 959(c)(2) PTEP — distributions of § 965-included E&P qualify for §
// 959(a)(1) exclusion (one of seven § 959(c)(2) groups in Notice
// 2019-01 sixteen-basket framework). Form 5471 + Form 965 + Transition
// Tax Statement. Six-mode severity ladder: NotApplicable, NotUs-
// ShareholderNoInclusion, SingleYearPaymentFull, EightYearInstallment-
// ScheduleAdopted, SCorpDeferralActive, ReitRatableSpreadActive.
// Coordinates with § 951 (Subpart F), § 951A (GILTI/NCTI — iter 500),
// § 956 (US property — iter 504), § 959 (PTEP — iter 512), § 962
// (individual election — iter 510), § 245A (DRD pathway — iter 502;
// note INAPPLICABLE to § 965 since pre-TCJA E&P), § 904 (FTC limitation
// — § 965 separate basket), § 960 (deemed-paid FTC — only 55.7pct /
// 77.1pct creditable via § 965(g) FTC denial).

async fn section_965_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_965::Section965Input>,
) -> Result<Json<traderview_expense::section_965::Section965Result>, ApiError> {
    Ok(Json(traderview_expense::section_965::check(&b)))
}

// ── §1058 securities loan non-recognition ─────────────────────────────
// Mounted at /api/calc/section-1058. Pure compute; § 1058(a) provides
// non-recognition treatment for securities loans satisfying four-prong
// § 1058(b) qualification test: (1) return identical securities; (2)
// dividend-equivalent payments to transferor during loan period; (3)
// risk of loss / opportunity for gain preserved; (4) terminable on
// demand per Treas. Reg. § 1.1058-1 + Rev. Proc. 2008-63 (5 business
// days). § 1058(a)(2) holding period tacking — loan period adds to
// transferor's holding period in returned securities. § 1058(c)
// "securities" definition = § 1236(c). Anshutz v. Commissioner, 135
// T.C. No. 5 (2010) + Calloway v. Commissioner, 135 T.C. No. 3 (2010)
// — variable prepaid forward contract bundled with stock loan FAILS
// § 1058(b)(3). Failure consequence: TAXABLE SALE at FMV + basis reset
// + holding period restart + potential § 1259 constructive sale.
// Trader-critical for Interactive Brokers SYEP + Robinhood Securities
// Lending + Schwab SLFPS + TD Ameritrade Securities Lending + hedge
// fund prime brokerage stock-loan + short seller's borrow (lender
// side). Companion: § 1259 (constructive sales), § 1092 (straddles),
// § 1236(c), § 475.

async fn section_1058_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1058::Section1058Input>,
) -> Result<Json<traderview_expense::section_1058::Section1058Result>, ApiError> {
    Ok(Json(traderview_expense::section_1058::check(&b)))
}

// ── §1092 straddle loss deferral ──────────────────────────────────────
// Mounted at /api/calc/section-1092. Pure compute; defers loss on a
// closed straddle leg up to unrecognized gain on remaining legs;
// §1092(c)(4)(B) qualified covered call carve-out exempts qualifying
// covered call positions from straddle treatment.

async fn section_1092_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1092::Section1092Input>,
) -> Result<Json<traderview_expense::section_1092::Section1092Result>, ApiError> {
    if b.realized_loss_on_disposed_leg < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "realized_loss_on_disposed_leg must be >= 0 (pass loss as positive)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1092::compute(&b)))
}

// ── §1291 PFIC default excess distribution + interest charge ─────────
// Mounted at /api/calc/section-1291. § 1291(a)(1)(A) excess
// distribution allocated RATABLY to each day in shareholder's
// holding period; § 1291(a)(1)(B) current-year + pre-PFIC-period
// portion taxed as ordinary; § 1291(a)(1)(C) intermediate-PFIC-year
// portion creates deferred tax at HIGHEST MARGINAL RATE for that
// year + § 6621 interest charge compounded DAILY. § 1291(a)(2)
// disposition gain converted to ordinary + interest charge.
// § 1291(b)(2)(A) excess = > 125% of 3-year average distributions;
// § 1291(b)(3)(B) FIRST YEAR all distributions excess.
// § 1291(d)(1) disabled by QEF election; § 1291(d)(2) purging
// election available; § 1291(f) disabled by mark-to-market.
// § 1291(g) § 988 currency translation. Trader-critical for
// foreign mutual funds + ETFs + hedge fund LP interests +
// offshore insurance products. Companion: § 1297 (PFIC definition)
// + § 1298 (special rules) + § 1295 (QEF election) + § 1296
// (mark-to-market) + § 6621 (underpayment interest rate) + Form
// 8621. Enacted by Tax Reform Act of 1986 § 1235 (Pub. L. 99-514,
// October 22, 1986); HIRE Act of 2010 § 521 added § 1298(f) annual
// reporting requirement.

async fn section_1291_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1291::Section1291Input>,
) -> Result<Json<traderview_expense::section_1291::Section1291Result>, ApiError> {
    if b.current_year_marginal_rate_bps > 10_000 || b.prior_year_highest_marginal_rate_bps > 10_000
    {
        return Err(ApiError::BadRequest(
            "marginal rate bps must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1291::check(&b)))
}

// ── §1293 PFIC QEF current-taxation mechanic ─────────────────────────
// Mounted at /api/calc/section-1293. § 1293(a)(1)(A) pro-rata
// inclusion of ORDINARY EARNINGS as ordinary income; § 1293(a)(1)(B)
// pro-rata inclusion of NET CAPITAL GAIN as LONG-TERM capital gain
// (CHARACTER PRESERVED regardless of shareholder holding period).
// § 1293(b)(1) ordinary earnings = E&P minus net capital gain;
// § 1293(b)(2) net capital gain per § 1222(11) (LT gain - ST loss).
// § 1293(c) pro rata share = daily-ratable distribution. § 1293(d)(1)
// basis INCREASED by inclusion (prevents double tax); § 1293(d)(2)
// basis DECREASED by PTI distribution. § 1293(e) coordinates with
// § 951 subpart F via § 1297(d) PFIC-CFC overlap rule. § 1293(f) +
// § 1294 deferral election (rarely used due to interest charge).
// Treas. Reg. § 1.1295-1(g) PFIC Annual Information Statement
// required for QEF election validity + ordinary/LTCG split.
// § 1(h)(11) qualified dividend treatment may apply for qualified
// foreign corporation status. Form 8621 + § 1298(f) annual
// reporting. Trader-critical for foreign-fund holders who elected
// QEF to escape § 1291 punitive regime. Completes PFIC framework
// cluster: § 1291 + § 1293 + § 1295 + § 1296 + § 1297 + § 1298.
// Sibling cluster: § 1222(11) + § 1294 + § 1297(d) + § 951 +
// § 1(h)(11) + Form 8621.

async fn section_1293_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1293::Section1293Input>,
) -> Result<Json<traderview_expense::section_1293::Section1293Result>, ApiError> {
    if b.pro_rata_share_bps > 10_000 {
        return Err(ApiError::BadRequest(
            "pro_rata_share_bps must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1293::check(&b)))
}

// ── §1294 QEF election to extend time for payment of tax ─────────────
// Mounted at /api/calc/section-1294. § 1294(a)(1) U.S. shareholder of
// QEF MAY ELECT to extend time for payment of tax attributable to
// share of UNDISTRIBUTED EARNINGS. § 1294(b) undistributed earnings
// = § 1293(a) includible amount - distributions - disposed-stock
// portion. § 1294(c) § 6601 interest accrues at § 6621 quarterly
// underpayment rate, compounded DAILY per § 6622, must be paid on
// termination. § 1294(d)(1) election UNAVAILABLE if § 551 foreign
// personal holding company rules engaged; § 1294(d)(2) election
// UNAVAILABLE if § 951 subpart F CFC rules engaged (§ 1297(d)
// PFIC-CFC overlap rule resolves). § 1294(e) termination upon
// EARLIEST of (1) distribution reducing undistributed earnings;
// (2) QEF stock disposition; (3) affirmative termination; (4) death
// of individual shareholder; (5) QEF ceases to be QEF; (6)
// shareholder ceases to be U.S. person. Treas. Reg. § 1.1294-1T
// temporary regulations. Form 8621 annual election + reporting.
// RARELY USED in practice due to interest charge accrual + complex
// reporting. Completes PFIC framework cluster: § 1291 + § 1293 +
// § 1294 + § 1295 + § 1296 + § 1297 + § 1298. Sibling cluster:
// § 6601 + § 6621 + § 6622 + Form 8621 + § 551 + § 951 + § 1297(d).

async fn section_1294_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1294::Section1294Input>,
) -> Result<Json<traderview_expense::section_1294::Section1294Result>, ApiError> {
    if b.tax_rate_on_undistributed_bps > 10_000 || b.section_6601_interest_rate_bps > 10_000 {
        return Err(ApiError::BadRequest(
            "rate bps must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1294::check(&b)))
}

// ── §1295 PFIC Qualified Electing Fund election ──────────────────────
// Mounted at /api/calc/section-1295. Pure compute; pro-rata
// inclusion of PFIC ordinary earnings + net capital gain per §1293.
// Character preserved (LTCG stays LTCG). Basis + PTI account
// evolution; previously-taxed-income distribution excluded from gross.

async fn section_1295_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1295::Section1295Input>,
) -> Result<Json<traderview_expense::section_1295::Section1295Result>, ApiError> {
    if b.adjusted_basis_year_start < Decimal::ZERO
        || b.distributions_received < Decimal::ZERO
        || b.pti_account_year_start < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "adjusted_basis_year_start, distributions_received, pti_account_year_start must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1295::compute(&b)))
}

// ── §864(b)(2) non-US trader/investor safe harbor ────────────────────
// Mounted at /api/calc/section-864b2. Pure compute; classifies a
// non-US person's US securities/commodities trading as effectively
// connected or not, based on the four-factor test (non-US person /
// own-account / not a dealer / no US office).

async fn section_864b2_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_864b2::Section864B2Input>,
) -> Result<Json<traderview_expense::section_864b2::Section864B2Result>, ApiError> {
    Ok(Json(traderview_expense::section_864b2::compute(&b)))
}

// ── §163(d) investment interest expense limitation ───────────────────
// Mounted at /api/calc/section-163d. Pure compute; investment
// interest deductible only up to net investment income, indefinite
// carryforward. Models the §1(h)(11)(D)(i) QD election and
// §163(d)(4)(B)(iii) LTCG election that boost the limit but forfeit
// preferential rates.

async fn section_163d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_163d::Section163dInput>,
) -> Result<Json<traderview_expense::section_163d::Section163dResult>, ApiError> {
    if b.investment_interest_expense < Decimal::ZERO
        || b.interest_income < Decimal::ZERO
        || b.ordinary_dividends < Decimal::ZERO
        || b.qualified_dividends < Decimal::ZERO
        || b.other_investment_expenses < Decimal::ZERO
        || b.prior_year_carryforward < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_163d::compute(&b)))
}

// ── §163(h) home mortgage interest deduction ────────────────────────
// Mounted at /api/calc/section-163h. Universal qualified residence
// interest computation: $750k acquisition indebtedness cap (TCJA,
// made permanent by OBBBA 2025 § 70108); $1M grandfathered cap for
// pre-2017-12-16 mortgages; MFS half-caps ($375k / $500k); home
// equity interest permanently disallowed unless acquisition use;
// PMI premiums reinstated as deductible 2026+ per OBBBA; refinance
// blended-cap calculation under § 163(h)(3)(F)(iii).

async fn section_163h_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_163h::Section163hInput>,
) -> Result<Json<traderview_expense::section_163h::Section163hResult>, ApiError> {
    if b.acquisition_indebtedness_balance < Decimal::ZERO
        || b.non_acquisition_home_equity_balance < Decimal::ZERO
        || b.interest_paid_acquisition < Decimal::ZERO
        || b.interest_paid_non_acquisition_home_equity < Decimal::ZERO
        || b.mortgage_insurance_premiums_paid < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    if let Some(grand) = b.grandfathered_refinance_portion {
        if grand < Decimal::ZERO {
            return Err(ApiError::BadRequest(
                "grandfathered_refinance_portion must be >= 0 when set".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_163h::compute(&b)))
}

// ── §280F luxury auto depreciation cap ───────────────────────────────
// Mounted at /api/calc/section-280f. Pure compute; caps annual
// depreciation on passenger autos under §280F(a)(1). Year-by-year
// caps from Rev. Proc. tables 2020-2024 (caller_override for 2025+).
// §280F(d)(5) heavy-vehicle carve-out for > 6,000 lb GVWR exempts.

async fn section_280f_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_280f::Section280FInput>,
) -> Result<Json<traderview_expense::section_280f::Section280FResult>, ApiError> {
    if b.cost_basis < Decimal::ZERO {
        return Err(ApiError::BadRequest("cost_basis must be >= 0".into()));
    }
    Ok(Json(traderview_expense::section_280f::compute(&b)))
}

// ── §280B demolition of structures ──────────────────────────────────
// Mounted at /api/calc/section-280b. §280B(1) NO deduction for
// demolition costs or loss sustained; §280B(2) capitalized to land
// basis. IRS Notice 90-21 casualty exception allows separate § 165
// casualty loss when structure was casualty-damaged before
// demolition; § 168(i)(4) GAA election can permit abandonment loss
// via GAA termination under Treas. Reg. § 1.168(i)-1(e).

async fn section_280b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_280b::Section280BInput>,
) -> Result<Json<traderview_expense::section_280b::Section280BResult>, ApiError> {
    if b.demolition_costs_paid_dollars < 0
        || b.structure_remaining_adjusted_basis_dollars < 0
        || b.structure_salvage_value_dollars < 0
        || b.land_pre_demolition_basis_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_280b::compute(&b)))
}

// ── §280C credits-for-which-expenses-disallowed (anti-double-dip) ────
// Mounted at /api/calc/section-280c. § 280C(a) wage credits (§ 45A
// Indian Employment, § 45P Military Differential Wage, § 45S Paid
// Family Medical Leave, § 51 WOTC, § 1396 Empowerment Zone) require
// MANDATORY deduction disallowance equal to credit determined; no
// reduced-credit election available. § 280C(b) orphan drug § 45C
// permits § 280C(b)(3) reduced-credit election cross-referencing
// § 280C(c)(2)/(3) procedure. § 280C(c)(1) default rule for § 41
// research credit: § 174 deduction reduced by credit determined;
// § 280C(c)(2) election: credit × (1 − 21 %) = credit × 79 % and
// taxpayer keeps full § 174 deduction; § 280C(c)(3) election must
// be on ORIGINAL return (cannot be made on amended). Tax Court:
// § 280C applies to credit DETERMINED not credit ALLOWED — § 38
// general business credit limitation does NOT reduce § 280C
// disallowance.

async fn section_280c_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_280c::Section280cInput>,
) -> Result<Json<traderview_expense::section_280c::Section280cResult>, ApiError> {
    Ok(Json(traderview_expense::section_280c::compute(&b)))
}

// ── §280E controlled-substance trafficking deduction disallowance ────
// Mounted at /api/calc/section-280e. Disallows §162 deductions for
// trafficking in Schedule I/II controlled substances regardless of
// state legalization. COGS always allowed (Champ T.C. 2007); non-
// trafficking bifurcated activity expenses always allowed.
// EO 14370 (2025-12-18) directs DEA Schedule I → III rescheduling
// for marijuana; DOJ Final Order partially reschedules FDA-approved
// and state-licensed medical marijuana but leaves bulk / unlicensed
// crops in Schedule I.

async fn section_280e_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_280e::Section280eInput>,
) -> Result<Json<traderview_expense::section_280e::Section280eResult>, ApiError> {
    if b.gross_revenue_dollars < 0
        || b.cogs_dollars < 0
        || b.trafficking_business_expenses_dollars < 0
        || b.non_trafficking_bifurcated_expenses_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_280e::compute(&b)))
}

// ── § 280G Golden Parachute Payments + § 4999 20% recipient excise ───
// Mounted at /api/calc/section-280g. Pure compute; § 280G(a) denies
// employer compensation deduction for "excess parachute payment" to
// "disqualified individual" "contingent on change in ownership or
// control"; § 280G(b)(1) parachute = aggregate present value ≥ 3×
// base amount triggers CLIFF on entire excess over 1× base (not
// just over 3× portion); § 280G(b)(3) base amount = 5-year
// annualized includible compensation; § 280G(c) disqualified
// individual = (1) officer (max 50 employees regardless of title);
// (2) 1%+ shareholder; (3) highly compensated (top 1% or top 250);
// § 280G(b)(2)(A) change in control = > 50% ownership / 35% voting
// in 12 months / majority board in 12 months / 40%+ asset
// acquisition; § 280G(b)(5) small business exception = private
// corp + (S election or > 75% shareholder vote with adequate
// disclosure cleansing vote); § 280G(b)(4) reasonable compensation
// for post-change services exception with clear and convincing
// evidence burden; § 4999 20% recipient excise tax on excess
// parachute payment; gross-up vs modified-cutback structures.

async fn section_280g_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_280g::Section280gInput>,
) -> Result<Json<traderview_expense::section_280g::Section280gResult>, ApiError> {
    Ok(Json(traderview_expense::section_280g::check(&b)))
}

// ── § 280H PSC Fiscal-Year Minimum Distribution Requirement ─────────
// Mounted at /api/calc/section-280h (iter 546). Pure compute. § 280H
// disallows a portion of the PSC's employee-owner compensation
// deduction when the PSC has a § 444 fiscal-year election in effect
// AND fails the § 280H(c) minimum distribution requirement. Closes
// the income-deferral loophole that a non-calendar fiscal-year PSC
// would otherwise enjoy.
//
// § 280H(a) general rule: deduction otherwise allowed for "applicable
// amounts" paid or incurred to employee-owners is limited to the
// "maximum deductible amount" when PSC has § 444 election AND fails
// minimum distribution requirement.
//
// § 280H(c) minimum distribution: applicable amounts paid during
// deferral period must equal or exceed LESSER OF: (A) prior-year
// applicable amounts × (deferral months / preceding-year months),
// or (B) applicable percentage × adjusted taxable income for the
// deferral period.
//
// § 280H(d) applicable percentage: capped at 95%; computed from
// prior-3-year applicable amounts / prior-3-year adjusted taxable
// income.
//
// § 280H(f) carryover: nondeductible amounts treated as paid or
// incurred in the succeeding taxable year. § 280H(g) NOL carryback
// bar: no NOL carryback to/from any year of PSC with § 444 election.
//
// Five-mode severity ladder: NotApplicable,
// NotPersonalServiceCorporationSection280HInapplicable,
// CalendarYearNoSection280HApplied,
// MinimumDistributionRequirementSatisfiedNoDisallowance,
// Section280HDisallowanceAppliedCarryoverToNextYear.
//
// Coordinates with § 444 PSC fiscal-year election (Form 8716),
// § 269A (iter 544 — PSC tax-avoidance allocation), § 269 (iter 536),
// § 162 reasonable compensation, Schedule H (Form 1120) § 280H
// computation.

async fn section_280h_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_280h::Section280HPscMinimumDistributionInput>,
) -> Result<Json<traderview_expense::section_280h::Section280HPscMinimumDistributionOutput>, ApiError>
{
    Ok(Json(traderview_expense::section_280h::check(&b)))
}

// ── §481(a) accounting method change adjustment ──────────────────────
// Mounted at /api/calc/section-481. Pure compute; cumulative MTM
// adjustment for §475(f) trader-status election, 4-year ratable
// spread on positive (gain) per Rev. Proc. 2015-13, immediate
// recognition on negative (loss).

async fn section_481_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_481::Section481Input>,
) -> Result<Json<traderview_expense::section_481::Section481Result>, ApiError> {
    Ok(Json(traderview_expense::section_481::compute(&b)))
}

// ── § 482 Transfer Pricing Allocation Among Related Taxpayers ────────
// Mounted at /api/calc/section-482 (iter 566). Pure compute. § 482
// grants Treasury authority to allocate gross income, deductions,
// credits, and allowances between or among two or more organizations,
// trades, or businesses owned or controlled by same interests when
// necessary to prevent tax evasion or clearly reflect income.
// Cornerstone of US transfer-pricing enforcement.
//
// § 482 arm's-length standard (Treas. Reg. § 1.482-1): controlled
// transaction satisfies arm's-length if results consistent with
// uncontrolled taxpayers under same circumstances.
//
// § 482 best-method rule (Treas. Reg. § 1.482-1(c)): no strict
// hierarchy; most reliable method given facts and circumstances.
//
// Tangible-property methods (Treas. Reg. § 1.482-3): CUP + Resale
// Price + Cost Plus. Profit methods (Treas. Reg. § 1.482-5 + § 1.482-6):
// CPM + Profit Split. Services (Treas. Reg. § 1.482-9): CUSP + Gross
// Services Margin + Cost-of-Services Plus + CPM-for-services + Profit
// Split + Services Cost Method (SCM) safe harbor.
//
// Intangible property (Treas. Reg. § 1.482-4): Tax Reform Act of 1986
// commensurate-with-income standard requires periodic adjustments;
// Comparable Uncontrolled Transaction (CUT) preferred.
//
// Cost sharing arrangements (Treas. Reg. § 1.482-7): related parties
// share R&D costs proportional to expected benefits; Platform
// Contribution Transaction (PCT) payment required for pre-existing
// intangibles.
//
// § 6662(e) substantial-valuation-misstatement 20% penalty: § 482
// adjustment > $5M or 10% gross receipts. § 6662(h) gross-valuation-
// misstatement 40% penalty: 200% / 50% threshold. § 6664(c)
// reasonable-cause defense requires contemporaneous documentation per
// § 1.6662-6(d) (best-method + comparability + economic + financial
// analysis).
//
// Seven-mode severity ladder: NotApplicable,
// Section482WithinArmsLengthRangeNoAdjustment,
// Section482AdjustmentBelowPenaltyThreshold,
// Section6662ESubstantialValuationMisstatement20Pct,
// Section6662HGrossValuationMisstatement40Pct,
// CostSharingPctPlatformContributionTransactionRequired,
// CommensurateWithIncomeStandardIntangible.
//
// Coordinates with § 367(d) outbound intangible transfer, § 250 FDII,
// § 6662(e)/(h) accuracy-related penalties, § 6664(c) reasonable
// cause, Form 5471 + Form 5472 + Form 8975 CbCR reporting (≥ $850M
// global group revenue).

async fn section_482_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_482::Section482TransferPricingInput>,
) -> Result<Json<traderview_expense::section_482::Section482TransferPricingOutput>, ApiError> {
    Ok(Json(traderview_expense::section_482::check(&b)))
}

// ── § 514 Unrelated Debt-Financed Income (UBTI) ─────────────────────
// Mounted at /api/calc/section-514 (iter 506). Pure compute. Tax Reform
// Act of 1969 (Pub. L. 91-172) § 514 expands UBTI framework of § 511 to
// capture investment income that would otherwise be excluded under §
// 512(b)(1)-(5) when underlying property is debt-financed — prevents
// tax-exempt orgs (pension funds, university endowments, foundations,
// churches, IRAs) from using exempt status to guarantee leveraged
// investments without paying tax on the debt-financed portion. § 514(a)
// general rule: include in UBTI a debt/basis percentage of gross income
// from debt-financed property (avg acquisition indebtedness ÷ avg
// adjusted basis). § 514(b)(1) debt-financed property definition;
// § 514(b)(1)(A) substantially-exempt-function-use exclusion (university-
// owned dormitory + mortgaged hospital). § 514(c)(1) acquisition
// indebtedness definition (incurred-in-acquiring + pre-acquisition
// but-for + post-acquisition reasonably-foreseeable). § 514(c)(9)
// qualified real property exception (educational institution + qualified
// pension trust + § 501(c)(25) title-holding co + § 403(b)(9) account)
// with fractions rule under § 514(c)(9)(E). § 512(c) partnership look-
// through. Form 990-T under § 6012(a)(2); IRA UBTI above $1,000 annual
// exemption per § 512(b)(12). Tax at corporate rate per § 511(a) for
// § 501(c) orgs, trust rate per § 511(b) for § 401(a) qualified trusts
// and IRAs. Coordinates with § 511 (UBTI tax), § 512 (computation +
// partnership look-through + investment-income exclusions), § 513
// (unrelated trade/business definition), § 4940 (PF NII excise separate
// regime), § 408 (IRA rules), § 4944 (PF jeopardy investment).

async fn section_514_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_514::Section514Input>,
) -> Result<Json<traderview_expense::section_514::Section514Result>, ApiError> {
    Ok(Json(traderview_expense::section_514::check(&b)))
}

// ── §530 Coverdell Education Savings Accounts (ESA) ──────────────────
// Mounted at /api/calc/section-530. §530(b)(1)(A)(ii) $2,000 statutory
// annual contribution limit per beneficiary (unchanged since 2002; does
// NOT inflation-adjust); §530(b)(1)(A)(i) beneficiary must be under
// age 18 for contributions; §530(c) MAGI phaseout 95K-110K single +
// 190K-220K MFJ; §530(d)(7) special-needs beneficiary exception waives
// both age limits; §530(d)(8) age-30 distribution requirement (waived
// for special needs); §4973 6% excise tax on excess imposed on the
// BENEFICIARY (not contributor) annually. Sibling to section_223 (HSA)
// and section_219 (IRA) tax-favored savings vehicles.

async fn section_530_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_530::Section530Input>,
) -> Result<Json<traderview_expense::section_530::Section530Result>, ApiError> {
    if b.contributor_modified_agi_cents < 0 || b.aggregate_contributions_for_beneficiary_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1990..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest("year must be in [1990, 2100]".into()));
    }
    Ok(Json(traderview_expense::section_530::compute(&b)))
}

// ── §1031(f) related-party 2-year clawback ───────────────────────────
// Mounted at /api/calc/section-1031-f. Pure compute; evaluates whether
// a subsequent disposition of property received in a related-party
// §1031 exchange triggers retroactive gain recognition.

// ── § 1031 like-kind exchanges of real property ─────────────────
// Mounted at /api/calc/section-1031. § 1031(a)(1) general nonrecognition
// rule for real property held for productive use in trade/business or
// investment. § 1031(a)(2) post-TCJA exclusions: personal property,
// stocks/bonds/notes, partnership interests, certificates of trust,
// choses in action, inventory, foreign real property. § 1031(a)(3)
// deferred exchange — 45-day identification + 180-day exchange period.
// § 1031(b) boot gain recognition. § 1031(c) loss not recognized.
// § 1031(d) basis in replacement. § 1031(f) related party 2-year hold
// (separate iter 1031f module). Treas. Reg. § 1.1031(k)-1(c)(4)
// identification rules (3-property + 200% + 95% acquired). Treas. Reg.
// § 1.1031(k)-1(g)(4) qualified intermediary safe harbor. Treas. Reg.
// § 1.1031(k)-1(c)(5) 15% incidental personal property test. T.D.
// 9935 (Nov 2020) defines real property post-TCJA. TCJA 2017
// (Pub. L. 115-97 § 13303) limited § 1031 to real property only.

async fn section_1031_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1031::Section1031Input>,
) -> Result<Json<traderview_expense::section_1031::Section1031Output>, ApiError> {
    Ok(Json(traderview_expense::section_1031::check(&b)))
}

async fn section_1031_f_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1031_f::Section1031FInput>,
) -> Result<Json<traderview_expense::section_1031_f::Section1031FResult>, ApiError> {
    if b.deferred_gain_at_exchange < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "deferred_gain_at_exchange must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1031_f::compute(&b)))
}

// ── §1033 involuntary conversion gain-deferral ───────────────────────
// Mounted at /api/calc/section-1033. §1033(a)(2)(A) gain recognized to
// extent amount realized exceeds replacement cost (capped at realized
// gain); §1033(b)(2) basis in replacement = replacement cost − deferred
// gain; replacement windows: 2-year general (§1033(a)(2)(B)(i)) / 3-year
// condemnation real-property-trade-or-investment (§1033(g)(4)) / 4-year
// federally-declared-disaster principal residence (§1033(h)(1)(B)) / 5-year
// qualifying-disaster property (§1033(h)(2)(A)); similar-or-related-in-
// service-or-use test under Treas. Reg. § 1.1033(a)-2 (functional-use for
// owner-users, end-use for lessors of investment real estate); §1033(a)(2)
// election required for proceeds-into-property path (mandatory §1033(a)(1)
// when proceeds converted directly into property).

async fn section_1033_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1033::Section1033Input>,
) -> Result<Json<traderview_expense::section_1033::Section1033Result>, ApiError> {
    if b.amount_realized_cents < 0 || b.replacement_cost_cents < 0 {
        return Err(ApiError::BadRequest(
            "amount_realized_cents and replacement_cost_cents must be non-negative".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1033::compute(&b)))
}

// ── §1258 recharacterization of gain from conversion transactions ────
// Mounted at /api/calc/section-1258. § 1258(a) recharacterizes capital
// gain on disposition of position held as part of CONVERSION
// TRANSACTION as ORDINARY INCOME to extent of APPLICABLE IMPUTED
// INCOME AMOUNT (120 % of applicable rate × net investment for
// holding period). § 1258(c)(1) requires substantially all of
// expected return attributable to TIME VALUE of net investment.
// § 1258(c)(2) four enumerated categories: (A) applicable straddle
// within § 1092(c); (B) buy-and-forward-sale; (C) marketed/sold
// under marketing materials promising overall after-tax economic
// profit attributable to capital-gain conversion; (D) other
// transactions specified by regs. § 1258(d) applicable rate: § 1274(d)
// AFR compounded semiannually (standard term) or § 6621(b) federal
// short-term rate compounded daily (indefinite term). Treas. Reg.
// § 1.1258-1 netting rule. Enacted by OBRA 1993 (PL 103-66) § 13206
// in response to growing time-value-based capital-gain conversion
// structures. Trader/hedge-fund critical for marketed box spreads,
// buy-stock-plus-short-forward synthetic loans, calendar spreads
// with dominant time-decay leg, deep-in-the-money covered-call
// structures.

async fn section_1258_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1258::Section1258Input>,
) -> Result<Json<traderview_expense::section_1258::Section1258Result>, ApiError> {
    Ok(Json(traderview_expense::section_1258::compute(&b)))
}

// ── §1259 constructive sale of appreciated financial position ────────
// Mounted at /api/calc/section-1259. Pure compute; evaluates whether
// a hedge transaction triggers constructive sale of an appreciated
// long position, including the §1259(c)(3)(A) safe harbor.

async fn section_1259_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1259::Section1259Input>,
) -> Result<Json<traderview_expense::section_1259::Section1259Result>, ApiError> {
    Ok(Json(traderview_expense::section_1259::compute(&b)))
}

// ── §1260 constructive ownership transactions pass-thru loophole ──
// Mounted at /api/calc/section-1260. Added by Public Law 106-170
// § 534(a) on Dec 17 1999 (Ticket to Work Act) and effective for
// transactions entered after July 11 1999. § 1260 closes the hedge-
// fund synthetic-equity loophole that converted ordinary pass-thru
// income into LTCG via total-return swaps, forwards, or call+put
// collars referencing pass-thru entity interests (RIC, REIT, S-corp,
// partnership, trust, common trust fund, PFIC, REMIC — eight
// enumerated under § 1260(c)(2)). § 1260(a) recharacterizes any LTCG
// gain that EXCEEDS the "net underlying long-term capital gain"
// (NULTCG, defined by § 1260(e) — what taxpayer would have had on a
// direct buy-and-sell of the asset at open/close FMVs, established
// by clear and convincing evidence) as ORDINARY INCOME. § 1260(b)
// imposes an interest charge under § 6601 on the ordinary-income
// portion, computed as if the gain accrued at a CONSTANT RATE equal
// to the applicable Federal rate (AFR) in effect on the day the
// transaction closed, compounded semiannually; the interest cannot
// credit other tax. § 1260(c)(2) flush language M2M EXCEPTION:
// § 1260 does NOT apply if ALL positions in the transaction are
// marked to market under another IRC provision (covers § 1256
// contracts, dealer M2M, § 475(f) trader M2M election — the trader-
// tax-election bypass route). Four constructive-ownership transaction
// types under § 1260(d)(1): (A) long position under notional
// principal contract — taxpayer paid all/substantially-all yield
// including appreciation AND obligated to reimburse all/substantially-
// all decline; (B) forward or futures contract to acquire financial
// asset; (C) call holder + put grantor with substantially equal
// strike prices AND substantially contemporaneous maturity dates
// ("collar around zero" / synthetic forward construction); (D) other
// transactions with substantially the same effect (regulatory).
// Module computes simple linear AFR interest approximation
// (excess × afr_bps × years / 10_000) using u128 saturating
// arithmetic; statutory § 1260(b) uses semi-annual compounding
// under § 6601, so the module output is an approximation and notes
// that real-world final amount must be computed against the actual
// AFR table for each prior year of accrual. Seven-mode severity
// ladder × nine financial-asset types × five constructive-ownership
// transaction types × two M2M statuses × four gain statuses ×
// variable AFR / years inputs. Sibling cluster: section_1259 (built
// earlier — constructive sale of appreciated financial position
// targeting different anti-deferral vector — short-against-the-box
// closure), section_1258 (built iter 654 — conversion transactions
// closing related capital-vs-ordinary arbitrage), section_1092
// (straddle rules — partial wash sale + loss deferral coverage of
// offsetting positions), section_1256 (60/40 mark-to-market for
// listed contracts — provides the M2M-exception escape hatch for
// § 1256-listed underlying positions in § 1260 structures),
// section_475 / section_475f (dealer & trader mark-to-market
// elections — same M2M-exception bypass for traders who have
// elected ordinary M2M treatment), section_6601 (interest on
// underpayments — provides the rate engine cited by § 1260(b)
// interest charge mechanism).

async fn section_1260_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1260::Section1260Input>,
) -> Result<Json<traderview_expense::section_1260::Section1260Result>, ApiError> {
    Ok(Json(traderview_expense::section_1260::compute(&b)))
}

// ── §1361 S-corp eligibility 6-prong test ──────────────────────────
// Mounted at /api/calc/section-1361. §1361(b)(1) eligibility prongs:
// (A) domestic corporation + (B) not ineligible corp under
// §1361(b)(2) (financial institutions reserve method / insurance
// Subchapter L / FSC / DISC) + (C) ≤ 100 shareholders after
// §1361(c)(1) family attribution + (D) shareholders limited to
// individuals / qualifying estates / qualifying trusts (no
// partnerships / no non-S corps) + (E) no nonresident alien
// shareholders + (F) only one class of stock (voting-rights
// differences ARE permitted; economic differences NOT).

async fn section_1361_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1361::Section1361Input>,
) -> Result<Json<traderview_expense::section_1361::Section1361Result>, ApiError> {
    Ok(Json(traderview_expense::section_1361::compute(&b)))
}

// ── § 1366 S-corp pass-thru of items to shareholders ─────────────────
// Mounted at /api/calc/section-1366. Pure compute; cornerstone S-corp
// pass-through provision under § 1366(a)(1) — every shareholder reports
// pro rata share of (A) separately-stated items that could affect tax
// liability differently (capital gains/losses + § 1231 + charitable
// contributions + dividend income + tax-exempt interest + foreign tax
// credit + investment interest expense + § 179 expense + AMT
// preferences + § 199A QBI deduction + § 1411 NII items) and (B)
// non-separately-stated ordinary trade or business income/loss.
// § 1366(b) character flow-through: items treated by shareholder as
// if generated at shareholder level. § 1366(d)(1) three-tier loss
// limitation: § 1366(d)(1)(A) basis cap = adjusted basis in stock +
// adjusted basis in indebtedness; § 465 at-risk limitation; § 469
// passive activity loss limitation. § 1366(d)(2) suspended losses
// carry over indefinitely. § 1366(d)(3) post-termination transition
// period (1 year or 120 days after IRS notice). § 1366(e) family
// group reasonable compensation (IRS reallocation tool). § 1366(f)
// adjustment for § 1374 built-in gains tax and § 1375 passive
// investment income tax paid by S corp. Three-tier ordering per
// 26 C.F.R. § 1.1366-2: basis → at-risk → passive. Distinction from
// § 702 partnership pass-through (partners allow § 704(b) basis
// tracking and § 704 special allocations; S corp requires single-
// class-of-stock under § 1361(b)(1)(D)). Original framework Tax
// Reform Act of 1982 Subchapter S Revision Pub. L. 97-354.

async fn section_1366_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1366::Section1366Input>,
) -> Result<Json<traderview_expense::section_1366::Section1366Result>, ApiError> {
    Ok(Json(traderview_expense::section_1366::check(&b)))
}

// ── § 1377 S-corp pro rata + terminating election + PTTP ─────────────
// Mounted at /api/calc/section-1377. § 1377(a)(1) pro rata share
// general rule — DAILY ASSIGNMENT method: each shareholder's pro rata
// share determined by assigning equal portion of any S corp item to
// each day of taxable year, then dividing pro rata among shares
// outstanding on that day. Special-day rules per 26 C.F.R.
// § 1.1377-1(a)(2): disposing shareholder treated as shareholder for
// day of disposition; deceased shareholder for day of death.
// § 1377(a)(2) TERMINATING ELECTION ('closing of the books'): if
// shareholder's entire interest terminates AND all affected
// shareholders consent, corporation may elect to apply § 1377(a)(1)
// AS IF taxable year consisted of two separate years, first ending
// on termination date. Eligibility: § 1377(a)(2)(A) full disposition
// (sale + exchange + gift) OR § 1377(a)(2)(B) § 302 or § 303
// redemption + all-affected-shareholder consent + timely Form 1120-S
// attached statement. § 1377(b) POST-TERMINATION TRANSITION PERIOD
// (PTTP) definitions: § 1377(b)(1)(A) 1-year period after S corp
// ceases; § 1377(b)(1)(B) 120-day determination period; § 1377(b)(1)(C)
// 120-day E&P determination period. § 1377(b)(2) distribution during
// PTTP treated as reducing AAA under § 1368(c) first (tax-free), then
// E&P (dividend treatment). § 1377(b)(3) determination definitions
// (§ 1313(a)(1) IRS or court determination + Secretary determination
// + corporation-Secretary agreement). Distinction from § 706
// partnership: partnerships use varying interest rules under
// § 706(d) (interim closing or proration); S corps restricted to
// § 1377(a)(1) daily method unless § 1377(a)(2) election. Current
// framework Subchapter S Revision Act of 1982 Pub. L. 97-354.

async fn section_1377_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1377::Section1377Input>,
) -> Result<Json<traderview_expense::section_1377::Section1377Result>, ApiError> {
    Ok(Json(traderview_expense::section_1377::check(&b)))
}

// ── § 1367 S-corp shareholder stock basis adjustments ───────────────
// Mounted at /api/calc/section-1367. Core math for trader S-corp
// entity selection. § 1367(a)(1) increases (separately stated +
// nonseparately computed income + depletion excess); § 1367(a)(2)
// decreases (distributions + losses + nondeductibles + depletion);
// Treas. Reg. § 1.1367-1(f) standard ordering — increases →
// distributions → NONDEDUCTIBLES (lost if excess) → LOSSES
// (suspended if excess under § 1366(d)(2)); Treas. Reg.
// § 1.1367-1(g) election — losses-before-nondeductibles, with
// nondeductibles SUSPENDED instead of lost. § 1368(b)(2) excess
// distribution treated as capital gain. Sibling S-corp cluster:
// § 1361 (definition + eligibility) + § 1366 (pass-through) +
// § 1368 (distribution mechanics) + § 1374 (built-in gains tax).
// Form 7203 is the IRS basis-tracking schedule.

async fn section_1367_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1367::Section1367Input>,
) -> Result<Json<traderview_expense::section_1367::Section1367Result>, ApiError> {
    Ok(Json(traderview_expense::section_1367::compute(&b)))
}

// ── § 1368 S-corp distributions ─────────────────────────────────────
// Mounted at /api/calc/section-1368. Direct sibling to § 1367
// (basis adjustments). § 1368(b) governs S-corps without
// accumulated E&P: tax-free basis reduction then capital gain.
// § 1368(c) four-step ordering for S-corps with E&P: AAA tax-free
// → E&P dividend → remaining-basis tax-free → capital gain.
// § 1368(e)(1)(C) net-negative-adjustment rule excludes net
// negative from AAA-available calculation for distribution
// purposes. § 1368(e)(3) election with unanimous shareholder
// consent reverses (c)(1) and (c)(2) — distribute E&P first as
// dividends (used to purge E&P for § 1375 PII tax avoidance).
// Form 1120-S Schedule M-2 tracks AAA + E&P year over year.
// Sibling cluster: § 1361 + § 1366 + § 1367 + § 1374 + § 1375.

async fn section_1368_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1368::Section1368Input>,
) -> Result<Json<traderview_expense::section_1368::Section1368Result>, ApiError> {
    Ok(Json(traderview_expense::section_1368::compute(&b)))
}

// ── § 1375 S-corp passive investment income tax ─────────────────────
// Mounted at /api/calc/section-1375. Completes S-corp cluster
// (§ 1361 + § 1366 + § 1367 + § 1368 + § 1374 + § 1375). Tax
// engages when S-corp has accumulated E&P AND passive investment
// income exceeds 25% of gross receipts. § 1375(b)(1)(B) ENPI
// formula: NPI × (PII - 25% × GR) / PII. § 1375(b)(1)(A) caps
// ENPI at taxable income. Tax at highest § 11(b) corporate rate
// (21% post-TCJA). Companion: § 1362(d)(3) — three consecutive
// years of E&P + >25% PII terminates S election. § 1362(g) —
// 5-year re-election waiting period after termination.

async fn section_1375_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1375::Section1375Input>,
) -> Result<Json<traderview_expense::section_1375::Section1375Result>, ApiError> {
    if b.corporate_tax_rate_bps < 0 || b.corporate_tax_rate_bps > 10_000 {
        return Err(ApiError::BadRequest(
            "corporate_tax_rate_bps out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1375::compute(&b)))
}

// ── §1411 Net Investment Income Tax (NIIT) 3.8% surtax ──────────────
// Mounted at /api/calc/section-1411. § 1411(a)(1) tax = 3.8% × LESSER
// of net investment income (NII) or excess of MAGI over applicable
// threshold. § 1411(b) MAGI thresholds (NOT indexed; same since 2013
// ACA enactment): Single/HoH $200,000; MFJ/QSS $250,000; MFS $125,000.
// § 1411(c)(1) NII categories: interest + dividends + capital gains +
// passive rental income + royalties + non-qualified annuity income;
// § 1411(c)(1)(B) deductions for investment expenses + state tax;
// § 1411(c)(2) trade or business carve-outs for material participation;
// § 1411(c)(5) qualified retirement plan distributions EXCLUDED.
// § 469(c)(7) real estate professional carve-out — if taxpayer
// performs ≥ 750 hours per year in real property trades AND > 50% of
// personal services in real property, rental income may be treated as
// ACTIVE and excluded from NII. Pub. L. 119-21 OBBBA 2025 did NOT
// modify § 1411; 3.8% rate + thresholds + categories + retirement-
// plan exception remain identical to 2013 form. Trader-critical for
// any high-income trader. IRS Form 8960 (2025); IRS Topic 559.

async fn section_1411_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1411::Section1411Input>,
) -> Result<Json<traderview_expense::section_1411::Section1411Result>, ApiError> {
    Ok(Json(traderview_expense::section_1411::check(&b)))
}

// ── §1402 Self-Employment Income Definitions + SECA Tax ─────────────
// Mounted at /api/calc/section-1402. § 1402(a) net earnings from
// self-employment = gross trade-or-business income minus
// attributable deductions plus § 702(a)(8) partnership distributive
// share. § 1402(a)(1) rental real estate EXCLUDED unless real
// estate dealer. § 1402(a)(2) interest/dividends EXCLUDED unless
// securities dealer. § 1402(a)(3)(A) capital asset gain/loss
// EXCLUDED. § 1402(a)(3)(B) § 1231 property gain/loss EXCLUDED.
// § 1402(a)(13) limited partner distributive share EXCLUDED except
// § 707(c) guaranteed payments for services. § 1402(b) SE income
// definition. § 1401(a) OASDI 12.4 % up to wage base ($176,100 for
// 2025). § 1401(b)(1) Medicare 2.9 % uncapped. § 1401(b)(2)
// Additional Medicare 0.9 % on SE income > $200K single / $250K
// MFJ / $125K MFS. IRS Topic 429: § 475(f) MTM trader gains NOT
// SE income (sole proprietor TTS). Soroban Capital Partners (T.C.
// 2023) + Denham Capital (T.C. 2024) + Sirius Solutions (5th Cir.):
// active limited partner § 1402(a)(13) exclusion contested. Net
// earnings multiplier 0.9235 reflects employer-equivalent SE tax
// deduction.

async fn section_1402_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1402::Section1402Input>,
) -> Result<Json<traderview_expense::section_1402::Section1402Result>, ApiError> {
    Ok(Json(traderview_expense::section_1402::compute(&b)))
}

// ── §1400Z-2 Special Rules for Capital Gains Invested in QOZs ──────
// Mounted at /api/calc/section-1400z-2. Originally enacted by
// Section 13823 of the Tax Cuts and Jobs Act of 2017 (Public Law
// 115-97), signed December 22, 2017. Capital gain deferral via QOF
// investment within 180-day reinvestment window; original deferral
// until December 31, 2026; 15 % combined basis step-up (10 % at
// 5-year hold + 5 % at 7-year hold); permanent gain exclusion via
// FMV election at 10-year hold under § 1400Z-2(c). Comprehensively
// amended by One Big Beautiful Bill Act of 2025 (OBBBA) signed
// July 4, 2025: permanent program; rolling 5-year deferral period
// post-December 31, 2026; basis step-up reduced to 10 % regular
// OZ / 30 % rural QROF; 30-year gain exclusion limit; QROF
// substantial improvement requirement 50 % over 31 months;
// enhanced reporting + $50,000 large-fund penalty exposure; new
// QOZ designations effective January 1, 2027.
async fn section_1400z_2_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1400z_2::Section1400Z2Input>,
) -> Result<Json<traderview_expense::section_1400z_2::Section1400Z2Result>, ApiError> {
    Ok(Json(traderview_expense::section_1400z_2::check(&b)))
}

// ── §1445 FIRPTA withholding on USRPI dispositions by foreign person ──
// Mounted at /api/calc/section-1445. § 1445(a) default 15 % withholding
// on amount realized on USRPI disposition by foreign person; buyer
// (transferee) is statutory withholding agent and personally liable
// for unwithheld tax. § 1445(b)(2) non-foreign affidavit exception
// (transferor TIN + under penalty of perjury). § 1445(b)(4) publicly
// traded stock exception. § 1445(b)(5) domestically controlled REIT
// exception. § 1445(b)(6) buyer-residence exception: amount realized
// ≤ $300,000 + buyer 50 % residence affidavit = 0 % withholding.
// PATH Act of 2015 (P.L. 114-113) § 324: raised statutory rate from
// 10 % to 15 %; created tiered residence rate ($300K = 0 %, $300K-$1M
// + residence = 10 %, > $1M = 15 %). § 1445(c)(4) IRS withholding
// certificate (Form 8288-B) reduces/eliminates withholding. Form 8288
// + Form 8288-A due within 20 days of closing. Trader-landlord
// critical for any cross-border USRPI transaction; commercial
// brokers; trader entities with foreign-investor LP/LLC members.

async fn section_1445_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1445::Section1445Input>,
) -> Result<Json<traderview_expense::section_1445::Section1445Result>, ApiError> {
    Ok(Json(traderview_expense::section_1445::compute(&b)))
}

// ── §1446 partnership withholding on foreign-partner ECTI ────────────
// Mounted at /api/calc/section-1446. § 1446(a) partnership with
// effectively connected taxable income (ECTI) allocable to foreign
// partner must withhold; partnership is statutory withholding agent
// personally liable. § 1446(b)(1) highest § 1 rate (37 %) for
// noncorporate; highest § 11 rate (21 %) for corporate; reduced
// rates with documentation: 20 % LTCG, 25 % unrecaptured § 1250,
// 28 % collectibles. § 1446(c) foreign partner = non-US-person per
// § 7701(a). § 1446(e) quarterly installments due 15th of
// 4th/6th/9th/12th months on Form 8813. § 1446(f) added by TCJA
// 2017 (P.L. 115-97 § 13501): transferee withholds 10 % of amount
// realized on foreign-partner partnership interest transfer if
// portion of gain would be ECI; Treas. Reg. § 1.1446(f)-1 et seq.
// final regs effective Oct 7, 2020. Form 8804 annual return due
// 15th day of 3rd month after close. Form 8805 information
// statement per foreign partner. Form 8804-C partner-level
// certificate reducing withholding for partner-level deductions.
// Trader-critical for hedge funds, real estate LPs, S-corp
// converts with foreign LP/LLC members.

async fn section_1446_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1446::Section1446Input>,
) -> Result<Json<traderview_expense::section_1446::Section1446Result>, ApiError> {
    Ok(Json(traderview_expense::section_1446::compute(&b)))
}

// ── §1471 FATCA Chapter 4 withholding on payments to FFIs ────────────
// Mounted at /api/calc/section-1471. § 1471(a) 30 % withholding on
// withholdable payments to FFI not meeting § 1471(b) participating-
// FFI requirements (FFI agreement with IRS + Form 8966 reporting +
// recalcitrant-account withholding + due-diligence). § 1471(b)(2)
// deemed-compliant FFI via Treas. Reg. § 1.1471-5(f). § 1471(c)
// account-level information reporting (name + TIN + account number
// + balance + gross receipts). § 1471(d) FFI = depository,
// custodial, investment entity, certain insurance. § 1471(e) exempt
// beneficial owners (foreign governments, international
// organizations, foreign central banks). § 1471(f) pre-existing
// obligations grandfathered to January 1, 2014. § 1472 30 %
// withholding on NFFE unless certified no substantial US owners or
// identifies substantial US owners. § 1473 withholdable payment =
// US-source FDAP; gross proceeds rescinded by Treas. Reg.
// § 1.1473-1(a) + Notice 2014-33. § 1474 special rules. Forms:
// W-8BEN-E (foreign entity), W-8IMY (intermediary), W-8EXP (foreign
// govt), W-9 (US person); Form 1042 annual + Form 1042-S per-payee
// (due March 15) + Form 8966 FATCA report. GIIN identifies
// Chapter 4 status. Trader-critical for any cross-border payment.

async fn section_1471_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1471::Section1471Input>,
) -> Result<Json<traderview_expense::section_1471::Section1471Result>, ApiError> {
    Ok(Json(traderview_expense::section_1471::compute(&b)))
}

// ── §1374 S-corp built-in gains (BIG) tax ───────────────────────────
// Mounted at /api/calc/section-1374. Models the 5-year §1374(d)(7)
// recognition period (PATH Act 2015), the §1374(d)(2) lesser-of-three
// NRBIG computation (recognized BIG vs taxable-income limit vs NUBIG
// ceiling), §1374(b)(2) C-corp NOL deduction, §1374(b)(3) credit
// offset, and §1374(d)(2)(B) NRBIG carryforward when TI limit binds.
// 21% rate under §11(b) post-TCJA but rate is parameterized.

async fn section_1374_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1374::Section1374Input>,
) -> Result<Json<traderview_expense::section_1374::Section1374Result>, ApiError> {
    if b.nubig_at_conversion < Decimal::ZERO
        || b.recognized_big_this_year < Decimal::ZERO
        || b.recognized_bil_this_year < Decimal::ZERO
        || b.cumulative_prior_nrbig < Decimal::ZERO
        || b.c_corp_nol_carryforward < Decimal::ZERO
        || b.c_corp_credit_offset < Decimal::ZERO
        || b.nrbig_carryforward_from_prior_year < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1374::compute(&b)))
}

// ── §475(c)(2) dealer-in-securities classification ──────────────────
// Mounted at /api/calc/section-475c2. Returns one of Dealer /
// TraderWithMtmElection / TraderWithoutMtmElection / Investor based
// on the §475(c)(2) two-prong dealer test (customer + inventory
// prongs), Treas. Reg. §1.475(c)-1 negligible-sales exception, IRS
// Topic 429 trader case-law criteria (short-term profit motive,
// substantial activity, continuous & regular), and §475(f) election
// status. Drives downstream wash-sale, $3k capital loss cap, and
// ordinary-vs-capital character treatment across the system.

async fn section_475c2_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_475c2::Section475c2Input>,
) -> Result<Json<traderview_expense::section_475c2::Section475c2Result>, ApiError> {
    Ok(Json(traderview_expense::section_475c2::compute(&b)))
}

// ── §475(f) trader mark-to-market election mechanics ────────────────
// Mounted at /api/calc/section-475f. Companion to §475(c)(2) classifier:
// pins the ELECTION mechanics for trader-tax-status filers — April-15
// individual / March-15 entity election-statement deadline (unextended
// per IRS Topic 429), mandatory Form 3115 (Application for Change in
// Accounting Method) on year-of-election return under Rev. Proc.
// 2025-23 § 24.01, full §1091 wash-sale-rule exemption, removal of the
// $3,000 §1211(b) capital-loss cap with unlimited ordinary-loss
// passthrough, and the 5-year revocation lockout under Rev. Proc.
// 99-17 + Rev. Proc. 2025-23 § 24.02. Trader-tax-status floors
// (substantial + regular + continuous test per Endicott v. Commissioner
// T.C. Memo. 2013-199 + IRS Pub. 550): ~720 trades/year and ~4 hours
// per trading day. Severity ladder: optimal (timely election + Form
// 3115), missed-deadline-fatal, election-without-3115, TTS-floor-miss,
// prior-revocation-5-year-lock, newly-formed-entity-internal-books
// 2.5-month window.

async fn section_475f_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_475f::Section475fInput>,
) -> Result<Json<traderview_expense::section_475f::Section475fOutput>, ApiError> {
    Ok(Json(traderview_expense::section_475f::check(&b)))
}

// ── §4701 tax on issuer of registration-required obligation ──────────
// Mounted at /api/calc/section-4701. § 4701 issuer-side excise tax
// companion to § 1287 holder-side ordinary income recharacterization
// (built iter 666). Together § 1287 and § 4701 form the TEFRA anti-
// bearer-bond duo enacted by Public Law 97-248 § 310 effective for
// obligations issued after December 31, 1982. § 4701(a) imposes 1
// PERCENT of principal amount × number of calendar years (or
// portions thereof) during the period from issue date through
// maturity. § 4701(b) registration-required obligation definition
// matches § 163(f)(2): excepts (1) individual issuers; (2) non-
// public obligations; (3) short-term ≤ 1 year; (4) foreign-targeted
// Eurobond / TEFRA D obligations under Treas. Reg. § 1.163-5(c)(2)
// (i)(D). HIRE Act of 2010 (Pub. L. 111-147) substantially narrowed
// the Eurobond / TEFRA D exception for obligations issued after
// March 18, 2012 — most foreign-targeted bearer obligations now
// subject to § 4701 except narrow class of qualifying obligations.
// TEFRA § 310(d)(3) exception for warrants/convertibles offered
// outside US without 1933 Act registration before August 10, 1982.
// Treas. Reg. § 46.4701-1 implementing regulation. § 1287(a)
// parenthetical exception applies if § 4701 tax paid by issuer
// (avoids double penalty on holder).

async fn section_4701_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_4701::Section4701Input>,
) -> Result<Json<traderview_expense::section_4701::Section4701Result>, ApiError> {
    Ok(Json(traderview_expense::section_4701::compute(&b)))
}

// ── §213 medical expense deduction ──────────────────────────────────
// Mounted at /api/calc/section-213. §213(a) 7.5% AGI floor (CAA 2020
// § 103 made permanent); §213(d) qualified medical care; §213(d)(10)
// age-tiered LTC premium caps from IRS Rev. Proc. 2024-40 (2025) and
// Rev. Proc. 2025-32 (2026); HSA/FSA/HRA reimbursement
// double-deduction prevention. Requires Schedule A itemization.

async fn section_213_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_213::Section213Input>,
) -> Result<Json<traderview_expense::section_213::Section213Result>, ApiError> {
    if b.adjusted_gross_income < Decimal::ZERO
        || b.qualified_medical_expenses_other_than_ltc_premiums < Decimal::ZERO
        || b.ltc_premiums_paid < Decimal::ZERO
        || b.hsa_fsa_hra_reimbursements < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_213::compute(&b)))
}

// ── §170 charitable contribution deduction (post-OBBBA 2026 changes) ──
// Mounted at /api/calc/section-170. §170(a) general deduction;
// §170(b)(1) per-category AGI ceilings (60% cash to public charity made
// permanent by OBBBA + 50% non-cash to public + 30% capital-gain
// property or cash to 30%-limit orgs + 20% capital-gain property to
// private foundations); §170(b)(1)(I) OBBBA §70425 0.5% AGI FLOOR for
// itemizers eff. tax years after 2025-12-31 (amounts below floor carry
// forward 5 years); §170(p) OBBBA non-itemizer above-the-line deduction
// $1,000 single / $2,000 MFJ for cash to public charity only eff. 2026;
// §170(d)(1) 5-year carryforward for ceiling-blocked + floor-blocked.
// Sibling to section_170e (built-in-gain ordinary-income reduction).

async fn section_170_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_170::Section170Input>,
) -> Result<Json<traderview_expense::section_170::Section170Result>, ApiError> {
    if b.agi_cents < 0
        || b.cash_to_public_charity_cents < 0
        || b.capital_gain_property_to_public_charity_cents < 0
        || b.cash_to_private_foundation_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1990..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest("year must be in [1990, 2100]".into()));
    }
    Ok(Json(traderview_expense::section_170::compute(&b)))
}

// ── §219 IRA contribution deduction + Roth phaseout ──────────────────
// Mounted at /api/calc/section-219. §219(a) above-the-line Traditional
// IRA deduction; §219(b)(5)(A) 2026 $7,500 base contribution limit (was
// $7,000 for 2024/2025); §219(b)(5)(B) age-50+ catch-up — SECURE 2.0
// indexed starting 2024: 2026 = $1,100 (was statutory $1,000 pre-SECURE
// -2.0 still applies for 2024). §219(g) Traditional deduction phaseout
// when taxpayer OR spouse covered by workplace retirement plan: 2026
// Single $81K-$91K + MFJ taxpayer-covered $129K-$149K + MFJ spouse-only
// covered $242K-$252K (§219(g)(7) widened range) + MFS $0-$10K.
// §408A(c)(3) Roth contribution phaseout: 2026 Single $153K-$168K + MFJ
// $242K-$252K + MFS $0-$10K. §4973 6% excise on excess. Earned income
// caps contribution under §219(b)(1).

async fn section_219_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_219::Section219Input>,
) -> Result<Json<traderview_expense::section_219::Section219Result>, ApiError> {
    if b.contributions_cents < 0 {
        return Err(ApiError::BadRequest(
            "contributions_cents must be non-negative".into(),
        ));
    }
    if !(1990..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest("year must be in [1990, 2100]".into()));
    }
    Ok(Json(traderview_expense::section_219::compute(&b)))
}

// ── §221 student loan interest deduction (above-the-line) ────────────
// Mounted at /api/calc/section-221. §221(a) above-the-line deduction
// up to $2,500 for interest paid on qualified education loans;
// §221(b)(1) STATUTORY $2,500 cap does NOT inflation-adjust;
// §221(b)(2) MAGI phaseout — 2026 single/HoH $85K-$100K + MFJ
// $175K-$205K + 2025 single $80K-$95K + MFJ $165K-$195K; §221(e)(2)
// EXCLUDES Married Filing Separately filers entirely. Above-the-line
// = available even without itemizing.

async fn section_221_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_221::Section221Input>,
) -> Result<Json<traderview_expense::section_221::Section221Result>, ApiError> {
    if b.interest_paid_cents < 0 || b.modified_agi_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1990..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest("year must be in [1990, 2100]".into()));
    }
    Ok(Json(traderview_expense::section_221::compute(&b)))
}

// ── §223 Health Savings Accounts (HSAs) — triple-tax-advantaged ──────
// Mounted at /api/calc/section-223. §223(a) above-the-line deduction;
// §223(b)(2) contribution limits (2026 self-only $4,400 + family
// $8,750; 2025 $4,300 + $8,550); §223(b)(3) age-55+ catch-up $1,000
// STATUTORY not inflation-adjusted; §223(c)(2) HDHP definition (2026
// self-only min deductible $1,700 max OOP $8,500; family $3,400 / $17K;
// 2025 self-only $1,650 / $8,300; family $3,300 / $16,600). §4973 6%
// excise tax on excess contributions modeled.

async fn section_223_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_223::Section223Input>,
) -> Result<Json<traderview_expense::section_223::Section223Result>, ApiError> {
    if b.contributions_cents < 0
        || b.hdhp_deductible_cents < 0
        || b.hdhp_out_of_pocket_max_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1990..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest("year must be in [1990, 2100]".into()));
    }
    Ok(Json(traderview_expense::section_223::compute(&b)))
}

// ── §243 / §246 Dividends Received Deduction (DRD) ──────────────────
// Mounted at /api/calc/section-243. C-corp DRD with §243(a)(1) 50%
// baseline tier (<20% owned), §243(c) 65% (20-79%), §243(b) 100%
// (≥80% qualifying group); §246(c) holding-period (>45 days in
// 91-day window for common / short-preferred, >90 days in 181-day
// window for long-preferred — failure = full disallowance); §246A
// debt-financed portfolio stock reduction (DRD% × (100% −
// indebtedness%), not applicable to 80%+ tier).

async fn section_243_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_243::Section243Input>,
) -> Result<Json<traderview_expense::section_243::Section243Result>, ApiError> {
    if b.dividend_received_dollars < 0 {
        return Err(ApiError::BadRequest(
            "dividend_received_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_243::compute(&b)))
}

// ── § 245A Deduction for Foreign-Source-Portion of Dividends (TCJA
// participation exemption "territorial" system) ──────────────────────
// Mounted at /api/calc/section-245a (iter 502). Pure compute. TCJA 2017
// Pub. L. 115-97 § 14101 enacted § 245A effective for distributions made
// after December 31, 2017. 100% dividends-received deduction for foreign-
// source-portion of dividend received by domestic C corporation from
// Specified 10-Percent Owned Foreign Corporation (SFC) per § 245A(b)
// (any foreign corp other than PFIC-not-CFC with respect to which any
// domestic corp is a § 951(b) 10%-or-more US shareholder). § 246(c)(1)
// holding period requirement: domestic corp must hold SFC stock for more
// than 365 days in 731-day window beginning 365 days before ex-dividend
// date. § 245A(d) coordination: NO FTC or § 164(a) deduction allowed for
// foreign tax (including withholding) on DRD-eligible dividend per
// § 275(a)(4) — permanent book-tax difference. § 245A(e) hybrid dividend
// rule: dividend where foreign corp received foreign tax deduction or
// similar tax benefit is recharacterized as Subpart F inclusion, no DRD,
// no FTC; hybrid deduction account reduced per Treas. Reg. § 1.245A(e)-
// 1(d). Treas. Reg. § 1.245A-5 anti-abuse for extraordinary disposition
// / extraordinary reduction. Form 1120 Schedule C; Form 8993 if § 250
// also claimed. Coordinates with § 951A (GILTI / post-OBBBA NCTI residual),
// § 250 (FDII / GILTI / NCTI deduction), § 1297 (PFIC mutually exclusive),
// § 1248 (gain on CFC stock sale recharacterized as dividend eligible
// for § 245A DRD), § 243 (domestic DRD parallel framework), § 59A (BEAT),
// § 56A (CAMT — § 245A-deductible amount excluded from CAMT AFSI per
// § 56A(c)(2)(C)).

async fn section_245a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_245a::Section245aInput>,
) -> Result<Json<traderview_expense::section_245a::Section245aResult>, ApiError> {
    Ok(Json(traderview_expense::section_245a::check(&b)))
}

// ── § 246 General Rules for DRD ──────────────────────────────────────
// Mounted at /api/calc/section-246 (iter 548). Pure compute. § 246 sets
// out the general limitations on the dividends-received deduction (DRD)
// claimable under § 243 + § 245(a) + § 245A + (indirectly) § 246A.
// Three core limitations:
//
// § 246(c) HOLDING PERIOD: common stock must be held for MORE THAN 45
// days during the 91-day testing window beginning 45 days before the
// ex-dividend date. Preferred stock with dividends for periods
// EXCEEDING 366 days must be held for MORE THAN 90 days during the
// 181-day testing window. § 246(c)(4) tolling: days with substantially
// diminished risk of loss (short sale, equity put, contractual hedge)
// do NOT count toward the holding period.
//
// § 246(b) TAXABLE-INCOME CAP: DRD limited to 50% / 65% / 100% of
// taxable income before DRD, matching the § 243 ownership-tier DRD
// percentage. § 246(b)(2) NOL exception: cap inapplicable if it would
// create or increase a net operating loss; resulting NOL becomes
// subject to § 172 carryforward rules.
//
// § 246(a) DRD EXCLUSIONS: no DRD for dividends from (1) § 501 / § 521
// tax-exempt corps, (2) non-qualified RIC distributions (not § 854
// pass-through), (3) non-capital-gain REIT distributions, (4) § 901(j)
// sanctioned-country foreign corps, (5) debt-financed portfolio stock
// under § 246A.
//
// Six-mode severity ladder: NotApplicable,
// Section246AHoldingPeriodFailedDrdFullyDisallowed,
// Section246AExcludedIssuerDrdDenied,
// Section246BTaxableIncomeCapAppliedReducedDrd,
// Section246BTwoNolExceptionFullDrdPreserved,
// FullDrdAllowedHoldingPeriodAndCapsMet.
//
// Coordinates with § 243 (50% / 65% / 100% DRD by ownership tier),
// § 245(a) US-source-portion DRD on foreign-corp dividends, § 245A
// 100% participation DRD, § 246A (iter 530) debt-financed portfolio
// stock DRD reduction, § 854 RIC pass-through framework, § 172 NOL
// carryover, § 901(j) sanctioned-country denial.

async fn section_246_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_246::Section246DividendsReceivedDeductionLimitsInput>,
) -> Result<
    Json<traderview_expense::section_246::Section246DividendsReceivedDeductionLimitsOutput>,
    ApiError,
> {
    Ok(Json(traderview_expense::section_246::check(&b)))
}

// ── § 246A DRD Reduction for Debt-Financed Portfolio Stock ──────────
// Mounted at /api/calc/section-246a (iter 530). Pure compute. § 246A
// substitutes a reduced DRD percentage for the base § 243 (domestic
// dividend) or § 245(a) (US-source foreign-corp dividend) DRD percentage
// when the corporate dividend recipient holds the underlying stock with
// portfolio indebtedness during the base period. Reverse-engineers the
// DRD downward to neutralize the financing arbitrage between simultaneous
// interest deduction and DRD on the same economic return.
//
// § 246A(a) formula: substituted DRD % = base DRD % × (100% - average
// indebtedness %). Base DRD percentages: 50% for less than 20% ownership;
// 65% for 20%-up-to-50% ownership. § 246A(b) "portfolio stock" = corporate
// shareholder owns less than 50% of issuer; 50%+ ownership is NOT portfolio
// stock and § 246A inapplicable. § 246A(c) "average indebtedness percentage"
// = average portfolio indebtedness divided by average adjusted basis of
// stock during base period. § 246A(d)(3) CAP: reduction cannot exceed
// interest deduction (including deductible short-sale expense) allocable
// to the dividend.
//
// § 246A(e) exceptions: (1) § 243(b) qualifying dividends from
// affiliated-group members (80% or more ownership per § 1504), (2) SBIC
// dividends under Small Business Investment Act of 1958. § 245A 100%
// participation DRD on foreign-source dividend is NOT subject to § 246A
// reduction by statutory structure and Treas. Reg. § 1.245A-3.
//
// Seven-mode severity ladder: NotApplicable (50%+ ownership not portfolio
// stock), Section245aFullParticipationDrdPreservedNoReduction,
// Section243bAffiliatedQualifyingDividendPreservedNoReduction,
// SbicExemptionPreservesFullDrd, Section246aReductionApplied,
// Section246aReductionCappedByInterestDeductionAllocable,
// InvalidInputAverageIndebtednessExceedsOneHundred.
//
// Coordinates with § 243 (50% / 65% / 100% base DRD tiers), § 245(a)
// (US-source DRD on foreign-corp dividend), § 245A (100% participation
// DRD — distinct branch not reduced), § 246 (general DRD limits incl.
// § 246(c) 46/91-day holding-period rule + § 246(b) taxable-income cap),
// § 265 (interest-expense disallowance for tax-exempt income — parallel
// logic), § 1504 (affiliated group), § 56A (CAMT — DRD effect on AFSI).

async fn section_246a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_246a::Section246aDebtFinancedPortfolioStockInput>,
) -> Result<
    Json<traderview_expense::section_246a::Section246aDebtFinancedPortfolioStockOutput>,
    ApiError,
> {
    Ok(Json(traderview_expense::section_246a::check(&b)))
}

// ── §250 GILTI/FDII (NCTI/FDDEI post-OBBBA 2025) deduction ──────────
// Mounted at /api/calc/section-250. TCJA §14202 50% GILTI / 37.5%
// FDII deductions; OBBBA 2025 amendments effective tax years after
// 2025-12-31 rename to NCTI/FDDEI, reduce deductions to 40%/33.34%,
// eliminate DTIR/NDTIR (QBAI 10% return), raise FTC from 80% to 90%.

async fn section_250_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_250::Section250Input>,
) -> Result<Json<traderview_expense::section_250::Section250Result>, ApiError> {
    if b.gilti_ncti_income_dollars < 0
        || b.fdii_fddei_income_dollars < 0
        || b.qbai_dollars < 0
        || b.deemed_paid_foreign_taxes_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_250::compute(&b)))
}

// ── § 56A Corporate Alternative Minimum Tax (CAMT) ─────────────────
// Mounted at /api/calc/section-56a (iter 498). Pure compute. IRA 2022
// Pub. L. 117-169 § 10101; 15% minimum tax on adjusted financial statement
// income (AFSI) for applicable corporations effective for taxable years
// beginning after December 31, 2022. § 59(k)(1) applicable-corporation
// test: corporation (not S corp / RIC / REIT) with three-year-average AFSI
// exceeding $1B for years ending after December 31, 2021. § 59(k)(2)
// FPMG (Foreign-Parented Multinational Group) aggregation: US-resident
// member's AFSI test includes ALL FPMG members plus § 52 single-employer
// aggregation; US member applicable when FPMG aggregate > $1B AND US
// member's own three-year-average AFSI >= $100M safe-harbor floor.
// § 56A(c) sixteen AFSI adjustments to GAAP/IFRS book net income (federal
// tax back-out, defined benefit pension, qualified depreciation via § 168
// not book, cooperative dividends, CFC distributions, wholly-owned
// disregarded entity, consolidated tax group, etc.). § 56A(d) FSNOL
// limited to 80% of AFSI parallel to § 172 regular-tax NOL. § 38(c)(6)(E)
// general business credit usable against CAMT up to 75% of tentative
// minimum tax. § 53(c)-(d) CAMT credit carryforward indefinite against
// future regular tax. Form 4626 attached to Form 1120. Coordination with
// § 4501 (1% stock buyback excise — same IRA 2022 package), § 481
// (accounting method change AFSI restatement), § 55 (general AMT
// framework), § 53 (AMT credit), § 38 (general business credit), § 59A
// (BEAT — inbound FPMG members).

async fn section_56a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_56a::Section56aInput>,
) -> Result<Json<traderview_expense::section_56a::Section56aResult>, ApiError> {
    Ok(Json(traderview_expense::section_56a::check(&b)))
}

// ── §59A BEAT (Base Erosion and Anti-Abuse Tax) ─────────────────────
// Mounted at /api/calc/section-59a. TCJA §14401 BEAT for large
// multinationals: $500M 3-yr avg gross receipts gate, 3% base
// erosion percentage gate (2% banks/dealers), rate 5%→10%→10.5%
// post-OBBBA (was scheduled 12.5% under TCJA, repealed by OBBBA);
// banks/dealers +1% surcharge throughout; S corps/REITs/RICs
// categorically excluded under §59A(e)(2).

async fn section_59a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_59a::Section59aInput>,
) -> Result<Json<traderview_expense::section_59a::Section59aResult>, ApiError> {
    if b.gross_receipts_year_minus_1_dollars < 0
        || b.gross_receipts_year_minus_2_dollars < 0
        || b.gross_receipts_year_minus_3_dollars < 0
        || b.base_erosion_payments_dollars < 0
        || b.total_deductions_dollars < 0
        || b.nol_deduction_dollars < 0
        || b.regular_tax_liability_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs except taxable_income must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_59a::compute(&b)))
}

// ── §641 imposition of tax on trusts and estates ────────────────────
// Mounted at /api/calc/section-641. § 641(a) tax imposed on
// taxable income of estates, trusts, bankruptcy estates; paid on
// Form 1041. § 641(b) taxable income computed as for individual
// subject to § 642 + § 643 DNI + § 651/§ 661 distribution
// deductions. § 641(c) ESBT S corp portion at top 37 % rate with
// NO § 651/§ 661 distribution deduction. § 641(d) ESBT non-S-corp
// portion at normal trust rules. § 1(e) compressed four-bracket
// rate schedule (10/24/35/37). 2025 brackets (Rev. Proc. 2024-40):
// 10 % $0-$3,150; 24 % $3,150-$11,450; 35 % $11,450-$15,650; 37 %
// above $15,650. 2026 top bracket ~ $16,000. § 642(b) exemption:
// $600 estate / $300 complex trust / $100 simple trust (NOT
// inflation-indexed). § 1411(a)(2) NIIT 3.8 % on undistributed
// NII above top bracket threshold (same compressed level). TCJA
// 2017 rate structure made permanent by OBBBA 2025 § 70411.
// Trader-critical: trust top bracket $15,650 vs individual single
// $626,350 = ≈ 40× compression — trust planning + DNI optimization
// dominates after-tax outcome.

async fn section_641_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_641::Section641Input>,
) -> Result<Json<traderview_expense::section_641::Section641Result>, ApiError> {
    Ok(Json(traderview_expense::section_641::compute(&b)))
}

// ── §642 special rules for trust/estate credits + deductions ────────
// Mounted at /api/calc/section-642. § 642(a)(1) FTC cross-ref.
// § 642(b)(1) $600 estate exemption + § 642(b)(2)(A)/(B) $300
// complex trust exemption + § 642(b)(3) $100 simple trust exemption
// (NOT indexed for inflation). § 642(c)(1) UNLIMITED charitable
// contribution deduction from gross taxable income (tax-exempt
// interest excluded) pursuant to governing instrument — NO AGI
// FLOOR like § 170 individual. § 642(c)(2) following-year payment
// election. § 642(c)(3) remainder interest cross-ref to § 170(f)
// (3). § 642(c)(4) tax-exempt interest adjustment per Treas. Reg.
// § 1.642(c)-3. § 642(d) NOL cross-ref. § 642(e) depreciation +
// depletion apportioned between fiduciary and beneficiaries on
// income basis. § 642(f) amortization cross-ref to § 178/§ 169.
// § 642(g) double deduction disallowed — same expense cannot be
// claimed on BOTH income tax (§ 162/§ 212) AND estate tax (§ 2053/
// § 2054). § 642(h) unused loss carryovers + excess deductions
// pass to beneficiaries on termination. § 642(i) § 673 grantor
// trust cross-ref. OBBBA 2025 Pease limitation reinstatement
// (2026) — applicability to § 642(c) under analysis.

async fn section_642_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_642::Section642Input>,
) -> Result<Json<traderview_expense::section_642::Section642Result>, ApiError> {
    Ok(Json(traderview_expense::section_642::compute(&b)))
}

// ── §643 trust/estate DNI + accounting income definition ────────────
// Mounted at /api/calc/section-643. § 643(a) DNI definition: trust
// taxable income modified by — (1) no distribution deduction; (2)
// no personal exemption; (3) capital gains EXCLUDED from DNI when
// allocated to corpus and not distributed/required-to-be-distributed/
// set aside for § 642(c) charity; (5) tax-exempt income INCLUDED in
// DNI net of allocable expenses; (6) foreign trust modifications;
// (7) deemed-distributed amounts excluded. § 643(b) trust accounting
// income (FAI) defined for §§ 651/661 distribution limits + marital
// deduction + pooled income funds + charitable remainder trusts.
// § 643(c) beneficiary definition. § 643(d) coordination with
// §§ 651/661. § 643(e) property distributed in kind. § 643(f)
// multiple trusts aggregated for tax-avoidance principal purpose.
// § 643(g) estimated tax election. § 643(h) foreign trust
// distributions. Treas. Reg. § 1.643(b)-1 final regs (Dec 30, 2003)
// allow POWER TO ADJUST under state Uniform Principal and Income
// Act equivalents. DNI ceiling: tier-1 mandatory + tier-2
// discretionary; excess = tax-free corpus return. Trader-critical
// because 2025 trust top bracket 37 % at ~$15,650 makes DNI
// optimization the single most impactful trust planning decision.

async fn section_643_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_643::Section643Input>,
) -> Result<Json<traderview_expense::section_643::Section643Result>, ApiError> {
    Ok(Json(traderview_expense::section_643::compute(&b)))
}

// ── §651 simple trust distribution deduction ────────────────────────
// Mounted at /api/calc/section-651. § 651(a) simple trust deduction
// = amount of income required to be distributed currently if
// THREE requirements satisfied: (1) all income required current
// distribution; (2) NO § 642(c) charitable provision in trust
// instrument; (3) NO corpus distribution made during taxable year.
// § 651(b) deduction CAPPED at DNI (tax-exempt interest portion
// excluded). § 652(a) beneficiary includes amount required to be
// distributed WHETHER OR NOT ACTUALLY DISTRIBUTED; if income
// required > DNI, ratable allocation to beneficiaries per second
// sentence. § 652(b) character per Treas. Reg. § 1.652(b)-2.
// § 652(c) different taxable year handling: beneficiary includes
// based on trust's taxable year ending in beneficiary's taxable
// year. Treas. Reg. § 1.651(a)-1 + § 1.651(a)-2 + § 1.651(b)-1 +
// § 1.652(b)-2 implementing regs. Any failure of the three simple-
// trust requirements converts trust to COMPLEX under § 661.
// Trader-critical for family-office QPRT remainder + GRAT
// remainder + CLAT post-charitable phase + income-only family
// trusts.

async fn section_651_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_651::Section651Input>,
) -> Result<Json<traderview_expense::section_651::Section651Result>, ApiError> {
    Ok(Json(traderview_expense::section_651::compute(&b)))
}

// ── §661 complex trust distribution deduction ────────────────────────
// Mounted at /api/calc/section-661. § 661(a)(1) tier-1 mandatory
// distributions + § 661(a)(2) tier-2 discretionary distributions;
// total CAPPED at DNI under § 661(c) + § 643(a). § 661(b) character
// preservation — same proportion of each DNI class as that class
// bears to total DNI flows to beneficiaries. § 661(c) coordination
// with § 642(c) charitable contribution deduction. § 662(a)(1)/(2)
// beneficiary inclusion mirrors tier priority. § 662(b) beneficiary
// character parallels § 661(b). § 663(a)(1) specific bequest
// excluded; § 663(a)(2) capital gains allocated to corpus excluded.
// § 663(b) 65-day rule election: distributions within first 65 days
// of next year may be elected as made on last day of prior year.
// § 663(c) separate share rule: substantially separate and
// independent shares treated as separate trusts for § 661/§ 662/
// § 663(b). Treas. Reg. § 1.661(c)-2 + § 1.663(c)-1 to § 1.663(c)-5
// implementing regs. Conduit principle: trust deduction +
// beneficiary inclusion = no double tax; shifts income from
// compressed 37 % trust bracket (~$15,650 for 2025) to potentially
// lower individual brackets.

async fn section_661_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_661::Section661Input>,
) -> Result<Json<traderview_expense::section_661::Section661Result>, ApiError> {
    Ok(Json(traderview_expense::section_661::compute(&b)))
}

// ── §671 grantor trust general attribution rule ──────────────────────
// Mounted at /api/calc/section-671. § 671 general rule: when
// grantor (or another person under § 678) treated as owner of any
// portion of a trust under §§ 673-679, items of income, deductions,
// and credits attributable to that portion are INCLUDED in
// grantor's/owner's taxable income and credits — trust transparent
// for income tax purposes while separately respected for gift/
// estate tax (the "intentionally defective" feature). § 672
// definitions. § 673 reversionary interest > 5 % triggers (post-
// 1986; Tax Reform Act of 1986). § 674 power to control beneficial
// enjoyment (broad + § 674(b)/(c) exceptions). § 675 administrative
// powers — § 675(4)(C) substitution power is most common modern
// IDGT trigger. § 676 power to revoke. § 677 income for grantor's
// or spouse's benefit. § 678 person other than grantor treated as
// owner (BDIT); § 678(b)(2) 5x5 safe harbor — lapse up to greater
// of $5,000 or 5 % of corpus not treated as release. § 679 foreign
// trusts with US beneficiaries (anti-deferral). 26 CFR Part 1
// Subpart E implementing regs. IRS Rev. Rul. 2023-2 confirms
// grantor's death does NOT trigger basis step-up on IDGT assets.

async fn section_671_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_671::Section671Input>,
) -> Result<Json<traderview_expense::section_671::Section671Result>, ApiError> {
    Ok(Json(traderview_expense::section_671::compute(&b)))
}

// ── §673 reversionary interests / GRAT GRUT foundation ────────────────
// Mounted at /api/calc/section-673. § 673 is the FIRST substantive
// grantor-trust trigger in the §§ 671-679 progression (after § 672
// definitions; before § 674 beneficial enjoyment, § 675 administrative
// powers — iter 644, § 676 power to revoke — iter 646, § 677 income
// for benefit of grantor — iter 642, § 678 person other than grantor
// — iter 640, § 679 foreign trusts). § 673(a) general rule: grantor
// treated as owner of any portion in which grantor has reversionary
// interest in corpus or income if value of interest exceeds 5 %
// of trust portion value at inception. Tax Reform Act of 1986
// (Public Law 99-514) replaced prior "10-year rule" (Clifford trust
// era) with 5 % present-value test — effectively eliminated
// Clifford-style short-term grantor trust income shifting. § 673(b)
// minor lineal descendant exception: grantor not treated as owner
// solely by reason of reversion taking effect on death of lineal-
// descendant beneficiary before beneficiary attains age 21. § 673(c)
// special valuation rule: reversion value computed assuming MAXIMUM
// EXERCISE OF DISCRETION in favor of grantor (anti-avoidance).
// § 673(d) postponement rule: postponement of reacquisition date
// treated as NEW TRANSFER IN TRUST commencing with postponement
// effective date. § 673 is foundational substantive trigger for
// GRAT (Grantor Retained Annuity Trust) and GRUT (Grantor Retained
// Unitrust) grantor-trust status — retained annuity/unitrust
// interest value under § 7520 AFR tables creates reversionary
// interest exceeding 5 % threshold. Walton v. Comm'r 115 T.C. 589
// (2000) zeroed-out GRAT precedent. Treas. Reg. § 1.673(a)-1 +
// § 1.673(c)-1 implementing regs.

async fn section_673_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_673::Section673Input>,
) -> Result<Json<traderview_expense::section_673::Section673Result>, ApiError> {
    Ok(Json(traderview_expense::section_673::compute(&b)))
}

// ── §674 power to control beneficial enjoyment ────────────────────────
// Mounted at /api/calc/section-674. § 674 is the SECOND substantive
// grantor-trust trigger in §§ 671-679 progression (after § 673
// reversionary — iter 648). Completes the grantor-trust suite
// alongside § 675 (administrative — iter 644), § 676 (revocation —
// iter 646), § 677 (income for benefit — iter 642), § 678 (BDIT —
// iter 640), § 679 (foreign trusts — iter 650). § 674(a) general
// rule: grantor treated as owner where beneficial enjoyment subject
// to power of disposition exercisable by grantor or nonadverse
// party without adverse-party consent. § 674 is the BROADEST single
// Subpart E statute — its sweeping general rule is mitigated by
// three layers of exceptions: § 674(b) eight enumerated exceptions
// exercisable by any person (support of dependent; after-event;
// testamentary; charitable beneficiaries; distribute corpus;
// withhold income temporarily; withhold during legal disability;
// allocate corpus and income); § 674(c) independent trustee
// exception (at least HALF of trustees independent + grantor not a
// trustee); § 674(d) ascertainable standard exception (trustee not
// grantor or spouse + power limited by HEMS or similar). § 672(c)
// related or subordinate party definition controls § 674(c)
// independent trustee analysis (spouse if living with grantor;
// father/mother/issue/brother/sister; employee of grantor;
// corporation/employee where grantor + trust hold significant
// voting control; subordinate employee where grantor is executive).
// § 674(b)(5)/(6)/(7) + § 674(c) + § 674(d) exceptions DEFEATED by
// any power to add beneficiaries (except after-born/after-adopted
// children).

async fn section_674_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_674::Section674Input>,
) -> Result<Json<traderview_expense::section_674::Section674Result>, ApiError> {
    Ok(Json(traderview_expense::section_674::compute(&b)))
}

// ── §675 administrative powers / IDGT substitution power ──────────────
// Mounted at /api/calc/section-675. § 675 third grantor-trust trigger
// in §§ 671-679 progression after § 673 (reversionary) + § 674
// (beneficial enjoyment); precedes § 676 (revocation), § 677 (income
// for benefit; built iter 642), § 678 (third-party owner; built iter
// 640). § 675(1) power to deal for LESS THAN ADEQUATE CONSIDERATION.
// § 675(2) power to borrow corpus or income without adequate interest
// OR security — exception when trustee other than grantor acts under
// general lending power. § 675(3) grantor actually borrowed and not
// repaid before beginning of taxable year — exception for loans with
// adequate interest AND security by independent trustee (not grantor,
// not related/subordinate party). § 675(4) general powers of admin
// exercisable in NONFIDUCIARY capacity without fiduciary approval:
// (A) power to vote securities where grantor + trust hold significant
// voting control (10 % presumed significant under Treas. Reg.
// § 1.675-1); (B) power to control investment of trust funds in
// significant corporate holdings; (C) POWER TO SUBSTITUTE TRUST
// CORPUS for equivalent value ("swap power") — THE classic modern
// IDGT trigger; mere inclusion creates grantor trust status without
// further action; Rev. Rul. 2008-22 confirms no § 2036/§ 2038 estate
// inclusion when exercised in fiduciary capacity (trustee challenge +
// no value-shifting among beneficiaries); Rev. Rul. 2011-28 same for
// § 2042 in life insurance trusts. Paul Hood Swap Power Monograph
// (LISI) + Kitces Utilizing Swap Powers in Irrevocable Trusts =
// foundational practitioner references. § 672(a) adverse party +
// § 672(b) nonadverse + § 672(c) related/subordinate party definitions
// determine exception eligibility for § 675(2) and § 675(3).

async fn section_675_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_675::Section675Input>,
) -> Result<Json<traderview_expense::section_675::Section675Result>, ApiError> {
    Ok(Json(traderview_expense::section_675::compute(&b)))
}

// ── §676 power to revoke / revocable trust grantor trust ──────────────
// Mounted at /api/calc/section-676. § 676 is the fourth grantor-trust
// trigger in §§ 671-679 progression after § 673 (reversionary), § 674
// (beneficial enjoyment), § 675 (administrative powers — built iter
// 644). Precedes § 677 (income for grantor — iter 642), § 678
// (third-party owner / BDIT — iter 640), § 679 (foreign trusts).
// § 676 is the BROADEST and most commonly triggered grantor-trust
// rule because the standard revocable living trust used in routine
// estate planning falls squarely within § 676(a). § 676(a) general
// rule: grantor treated as owner of any portion where power to REVEST
// title in grantor is exercisable at any time by grantor OR
// nonadverse party OR both. 26 CFR § 1.676(a)-1 — power form
// irrelevant: revoke, terminate, alter, amend, appoint all trigger
// § 676(a) if exercise revests title in grantor. Adverse party
// exception: if power exercisable only with adverse-party consent
// (substantial adverse beneficial interest per § 672(a)), § 676 does
// NOT trigger. § 672(e) (enacted 1986) — spouse never adverse for
// § 671-679 purposes. § 676(b) postponement exception: § 676(a)
// inapplicable to power whose exercise can only affect beneficial
// enjoyment for period commencing after event such that grantor
// would not be treated as owner under § 673 (5 % reversionary rule)
// if power were reversionary interest. 26 CFR § 1.676(b)-1 — postpone-
// ment exception parallels § 673 5 % threshold. Post-event rule:
// grantor MAY be treated as owner post-event unless power relinquished
// (the "ticking clock"). Treas. Reg. § 1.673(d)-1 postponement
// period rule.

async fn section_676_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_676::Section676Input>,
) -> Result<Json<traderview_expense::section_676::Section676Result>, ApiError> {
    Ok(Json(traderview_expense::section_676::compute(&b)))
}

// ── §677 income for benefit of grantor / spouse ───────────────────────
// Mounted at /api/calc/section-677. § 677(a) third grantor-trust rule
// after § 673 (reversionary), § 674 (beneficial enjoyment), § 675
// (administrative powers), and § 676 (power to revoke); precedes § 678
// (person other than grantor; built in iter 640). § 677(a)(1) grantor
// treated as owner if income is or may be distributed to grantor or
// grantor's spouse; § 677(a)(2) same for income held or accumulated
// for future distribution to grantor or spouse; § 677(a)(3) same for
// income applied to payment of premiums on policies of insurance on
// life of grantor or spouse — except policies irrevocably payable for
// charitable purposes under § 170(c). § 677(a) spouse rule applies
// only to property transferred in trust after October 9, 1969 and
// only during period of marriage of grantor to beneficiary spouse —
// "§ 677 The Ghost That Haunts the Divorced Grantor" (Higgs Fletcher
// & Mack ABA 2021-09-23 presentation). § 677(b) discharge of legal
// obligation — grantor treated as owner whose income may be applied
// in discharge of legal obligation of grantor or spouse (includes
// state-law support obligations for minor children). § 672(e)
// (enacted 1986) — grantor treated as holding any power or interest
// held by spouse at time of creation; spouse is NEVER adverse for
// § 677 purposes. ILIT (Irrevocable Life Insurance Trust) discretion
// to pay premium alone creates § 677(a)(3) trigger under "may be"
// standard. 26 CFR § 1.677(a)-1 + § 1.677(b)-1 implementing regs.
// Grantor reports trust income on Form 1040 via § 671 flow-through;
// trust files Form 1041 with grantor-trust statement attached.

async fn section_677_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_677::Section677Input>,
) -> Result<Json<traderview_expense::section_677::Section677Result>, ApiError> {
    Ok(Json(traderview_expense::section_677::compute(&b)))
}

// ── §678 person other than grantor as owner / BDIT foundation ─────────
// Mounted at /api/calc/section-678. § 678(a)(1) person other than
// grantor treated as owner with respect to portion over which
// person has power exercisable solely by self to vest corpus or
// income in self (Crummey/withdrawal power). § 678(a)(2) post-
// release/modification attribution if retained control matches
// §§ 671-677 grantor criteria. § 678(b) exception when grantor
// otherwise treated as owner under §§ 671-679 (grantor's grantor
// trust status preempts). § 678(c) HEMS ascertainable standard
// (Health, Education, Maintenance, Support) safe from § 678(a)(1).
// § 678(d) timely renunciation/disclaimer escape. § 678(e) cross-
// reference to § 2514(e) 5x5 lapse safe harbor — lapse treated as
// gift only to extent exceeds GREATER of $5,000 or 5 % of trust
// corpus. BDIT (Beneficiary Defective Inheritor Trust): 3rd-party
// irrevocable trust funded with initial $5,000; beneficiary holds
// Crummey power lapsing within 5x5 → beneficiary as § 678 owner
// for income tax; trust assets escape beneficiary's estate. BDOT
// (Beneficiary Deemed Owner Trust): income-only variant. PLR
// 200949012 (Dec 4, 2009) confirms BDIT structure under § 678.
// Oshins NAEPC Journal foundational article; ACTEC Foundation
// podcast on § 678 mysteries; Griffin Bridgers HEMS safety
// analysis.

async fn section_678_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_678::Section678Input>,
) -> Result<Json<traderview_expense::section_678::Section678Result>, ApiError> {
    Ok(Json(traderview_expense::section_678::compute(&b)))
}

// ── §679 foreign trusts with US beneficiaries / anti-deferral ─────────
// Mounted at /api/calc/section-679. § 679 is the LAST substantive
// grantor-trust trigger in the §§ 671-679 progression after § 673
// (reversionary — iter 648), § 674 (beneficial enjoyment), § 675
// (administrative powers — iter 644), § 676 (power to revoke — iter
// 646), § 677 (income for benefit of grantor — iter 642), § 678
// (person other than grantor — iter 640). § 679(a)(1) general rule:
// US person who directly or indirectly transfers property to foreign
// trust treated as OWNER of portion attributable to property if any
// US beneficiary exists in that taxable year. § 679(a)(2) US
// beneficiary presumption — Secretary may treat any foreign trust
// as having US beneficiary unless transferor rebuts by documenting
// no part of income or corpus may benefit US person. § 679(a)(3)
// transfer-at-death exception — transfers by reason of death of
// transferor excepted; foreign grantor trust status terminates at
// US settlor death. § 679(a)(4) outbound trust migration — domestic
// trust that becomes foreign during transferor's life treated as if
// transferor transferred to foreign trust on migration date.
// § 679(a)(5) 5-year pre-immigration lookback — nonresident alien
// with residency starting date within 5 years after transfer deemed
// to have transferred property on residency starting date.
// § 679(c) 5-year beneficiary lookback — beneficiary not treated as
// US person if first became US person more than 5 years after
// transfer. Form 3520-A (Annual Information Return of Foreign Trust
// With US Owner) due 15th day of 3rd month after end of trust's
// taxable year; Foreign Grantor Trust Owner Statement flows to
// grantor's Form 1040 via § 671 attribution. Form 3520 (Annual
// Return To Report Transactions With Foreign Trusts and Receipt of
// Certain Foreign Gifts) required of US transferor/owner. § 6048
// reporting penalty: greater of $10,000 OR 35 % of gross reportable
// amount. Small Business Job Protection Act of 1996 (PL 104-188)
// significant amendments. Treasury Proposed Regs (May 8, 2024;
// 89 FR 39440) modernize § 679 + § 6048 implementing rules.

async fn section_679_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_679::Section679Input>,
) -> Result<Json<traderview_expense::section_679::Section679Result>, ApiError> {
    Ok(Json(traderview_expense::section_679::compute(&b)))
}

// ── §67(g) TCJA misc itemized deduction suspension ────────────────────
// Mounted at /api/calc/section-67g. § 67(g) added by Tax Cuts and Jobs
// Act of 2017 § 11045 (Pub. L. 115-97, December 22, 2017); originally
// scheduled to sunset after December 31, 2025; One Big Beautiful Bill
// Act of 2025 (H.R. 1, signed July 4, 2025) made § 67(g) PERMANENT.
// Trader-critical because § 67(g) is the single most important reason
// traders ELECT trader status under § 475(f) — without trader status,
// investment expenses (advisory fees, custody fees, subscription fees,
// home office, trading platform fees, education, travel) become NON-
// DEDUCTIBLE under § 67(g) because they are § 212 expenses subject to
// the suspended 2%-of-AGI floor. With § 475(f) trader status, expenses
// qualify as § 162 trade-or-business deductions on Schedule C and
// ESCAPE § 67(g) suspension entirely. § 67(b) exempts 12 categories
// (§ 163 interest + § 164 taxes + § 165(a) casualty + § 170 charity +
// § 213 medical + § 691(c) IRD + § 215 alimony + § 217 moving (armed
// forces) + § 1341 claim of right + gambling losses + § 642(c)
// trust/estate charity + § 7702B(a)(2) qualified long-term care).
// § 67(e) preserves estate/trust administration expense deductibility
// per Treas. Reg. § 1.67-4 + IRS Notice 2018-61. Companion to § 475(f)
// (trader election) + § 162 (Schedule C) + § 212 (production-of-income)
// + § 1411 (NIIT 3.8% surtax — also exempts trader business income).

async fn section_67g_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_67g::Section67gInput>,
) -> Result<Json<traderview_expense::section_67g::Section67gResult>, ApiError> {
    Ok(Json(traderview_expense::section_67g::check(&b)))
}

// ── §6045 broker information reporting (Form 1099-B / 1099-DA) ───────
// Mounted at /api/calc/section-6045. §6045(a) requires brokers (anyone
// in ordinary-course-of-business standing ready to effect sales for
// others) to file Form 1099-B (securities + barter) or new Form 1099-DA
// (digital assets, eff. 2025-01-01). §6045(g) bifurcates into COVERED
// (broker required to report adjusted basis) vs NON-COVERED (gross
// proceeds only). Acquisition cutoffs per Treas. Reg. § 1.6045-1(a)(15):
// stock 2011-01-01 + mutual fund/DRIP 2012-01-01 + less-complex debt
// 2014-01-01 + more-complex debt 2016-01-01 + digital asset 2026-01-01
// (NEW under IIJA § 80603 amending § 6045; requires continuous broker-
// account holding). NO DE MINIMIS — even one cent triggers reporting.

// ── §6041 information at source + 1099-NEC / 1099-MISC reporting ────
// Mounted at /api/calc/section-6041. § 6041(a) trade-or-business
// persons making payments of $2,000+ (post-OBBBA) to another
// person must file Form 1099. § 6041(c) corporate exception
// (general); attorneys + medical/health payments NOT exempt
// despite corporate payee status. § 6041(h) TIN requirement
// (W-9); § 3406 backup withholding at 24 % when payee fails to
// provide TIN. § 6041A direct sales of $5,000+ + nonemployee
// remuneration subject to same OBBBA threshold. § 6721 failure
// to file ($60 ≤ 30 days, $130 ≤ Aug 1, $330 after Aug 1;
// intentional disregard $660+). § 6722 failure to furnish
// payee statement. § 6723 other reporting failures ($50). P.L.
// 119-21 OBBBA § 70433 raised § 6041(a) + § 6041A(a)(2) threshold
// from $600 to $2,000 effective Jan 1, 2026; inflation indexed
// from 2027 with 2025 base, rounded to nearest $100. Forms:
// 1099-NEC (non-employee comp; due Jan 31), 1099-MISC (rents/
// prizes/awards; due Feb 28 paper / Mar 31 electronic), 1042-S
// (foreign payee FATCA Chapter 3/4), W-9 (US payee TIN), W-8BEN /
// W-8BEN-E (foreign payee).

async fn section_6041_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6041::Section6041Input>,
) -> Result<Json<traderview_expense::section_6041::Section6041Result>, ApiError> {
    Ok(Json(traderview_expense::section_6041::compute(&b)))
}

// ── §6109 identifying numbers / TIN / EIN / ITIN / SSN ────────────────
// Mounted at /api/calc/section-6109. § 6109(a)(1) payer required to
// file information return must include payee's correct TIN. § 6109(a)(2)
// payee must furnish correct TIN to payer when payment is reportable
// on information return. § 6109(a)(3) person making return or
// statement under § 6011 must include identifying number assigned to
// such person. TIN types: SSN issued by Social Security Administration
// to US citizens/residents eligible to work; EIN issued by IRS to
// entities + sole proprietors; ITIN issued by IRS to foreign persons
// not eligible for SSN; ATIN issued by IRS during pending US-domestic
// adoptions. Form W-9 (US persons), Form W-8BEN (foreign individuals),
// Form W-8BEN-E (foreign entities), Form W-8IMY (intermediaries),
// Form W-8ECI (effectively connected income). § 3406 backup withholding
// at 24 % when payee fails to furnish valid TIN, after B-Notice TIN
// mismatch, IRS notification of underreporting, or W-9 exemption
// certification failure. § 6721 information return penalty: $50 per
// return ($250,000 large filer max / $100,000 small business max);
// $100 per return with NO MAXIMUM for intentional disregard. B-Notice
// cure process: first B-Notice 30 business days; second B-Notice
// requires IRS or SSA validation document within 3-year lookback.
// Treas. Reg. § 301.6109-1. IRS IRM 5.19.3 Backup Withholding Program.

async fn section_6109_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6109::Section6109Input>,
) -> Result<Json<traderview_expense::section_6109::Section6109Result>, ApiError> {
    Ok(Json(traderview_expense::section_6109::compute(&b)))
}

// ── §6042 returns regarding payments of dividends (1099-DIV) ────────
// Mounted at /api/calc/section-6042. § 6042(a)(1) — every person who
// makes dividend payments aggregating $10 or more (or who receives
// as nominee) shall make a return. § 6042(b) — dividend defined per
// § 316 (corporate distributions out of E&P) + § 852 RIC + § 857
// REIT + stockbroker substitute payments; EXCLUDES exempt-interest
// dividends (§ 852(b)(5)) and § 3406 backup-withheld amounts.
// § 6042(c) — written statement to recipient by January 31. § 6042
// (d)(1) substitute dividend payments by broker on short sales
// reportable; § 6042(d)(2) uncertain payments rule — entire amount
// treated as dividend. Form 1099-DIV box breakdown: 1a ordinary +
// 1b qualified (§ 1(h)(11) 60-day holding period) + 2a-d capital
// gain (including § 1202 QSBS + § 1250 unrecaptured) + 3 return of
// capital + 5 § 199A REIT/PTP (20% deduction) + 8 foreign tax (§
// 901 FTC) + 12 exempt-interest + 13 specified private activity
// bond (AMT). Trader-critical: § 1411 NIIT 3.8% on dividends + §
// 988 foreign currency ADR conversions. Companion to § 6041 + §
// 6045 + § 6049 + § 6050W + § 3406 + § 1(h)(11) + § 199A + § 1411
// + § 1202 + § 988 + § 901.
async fn section_6042_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6042::Section6042Input>,
) -> Result<Json<traderview_expense::section_6042::Section6042Result>, ApiError> {
    Ok(Json(traderview_expense::section_6042::check(&b)))
}

async fn section_6045_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6045::Section6045Input>,
) -> Result<Json<traderview_expense::section_6045::Section6045Result>, ApiError> {
    if b.proceeds_cents < 0 {
        return Err(ApiError::BadRequest(
            "proceeds_cents must be non-negative".into(),
        ));
    }
    if !(1990..=2100).contains(&b.acquisition_year)
        || !(1..=12).contains(&b.acquisition_month)
        || !(1..=31).contains(&b.acquisition_day)
        || !(1990..=2100).contains(&b.transaction_year)
    {
        return Err(ApiError::BadRequest(
            "acquisition + transaction dates must be valid".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6045::compute(&b)))
}

// ── §6049 returns regarding payments of interest (1099-INT/OID) ─────
// Mounted at /api/calc/section-6049. § 6049(a) — every person who
// makes interest payments aggregating $10 or more (or who receives
// as nominee) shall make a return. § 6049(b) — interest defined:
// registered-form obligations + bank deposits + savings institution
// interest + insurance company-held + OID per § 1272 + broker-dealer
// custodial + Treasury obligations + municipal bond tax-exempt (§
// 103(a)). § 6049(c) — written statement to recipient by January 31.
// § 6049(d) — nominee/middleman pass-through; broker (§ 6045(c)) is
// middleman. § 6049(e) — backup withholding under § 3406 triggers
// reporting IRRESPECTIVE of $10 threshold. Form 1099-INT + Form 1099-
// OID. Trader-relevant sources: Treasury (T-bills + TIPS + Series I
// — federal tax/state-exempt); municipal bonds (federal tax-exempt
// § 103(a)); corporate bonds (taxable); zero-coupon (OID); money
// market funds; bank deposit; brokerage cash-balance; § 988 foreign
// currency. Companion to § 6041 + § 6042 + § 6045 + § 6045A + § 6045B
// + § 6050W + § 3406 backup withholding + § 103(a) tax-exempt muni +
// § 1272 OID + § 988 foreign currency.
async fn section_6049_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6049::Section6049Input>,
) -> Result<Json<traderview_expense::section_6049::Section6049Result>, ApiError> {
    Ok(Json(traderview_expense::section_6049::check(&b)))
}

// ── §6050I cash transaction reporting (Form 8300) ────────────────────
// Mounted at /api/calc/section-6050i. §6050I(a) requires any person
// engaged in a trade or business who receives more than $10,000 in
// cash in one transaction (or two or more related transactions within
// 24 hours) to report to IRS AND FinCEN within 15 days on Form 8300.
// §6050I(d) cash definition includes currency + cashier's checks +
// money orders + bank drafts WITH FACE AMOUNT ≤ $10,000 (personal
// checks and wire transfers are NOT cash). IIJA §80603 added digital
// assets effective 2024-01-01 BUT IRS Announcement 2024-04
// SUSPENDED implementation — digital assets currently EXCLUDED from
// § 6050I cash pending IRS regulations. §6721 intentional-disregard
// penalty = greater of $250K or aggregate; §7203 willful-failure
// criminal exposure.

async fn section_6050i_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6050i::Section6050IInput>,
) -> Result<Json<traderview_expense::section_6050i::Section6050IResult>, ApiError> {
    if b.single_instrument_face_amount_cents < 0 || b.aggregate_related_24_hour_amount_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6050i::compute(&b)))
}

// ── §6050W payment-settlement-entity 1099-K reporting threshold ──────
// Mounted at /api/calc/section-6050w. Two PSE categories with different
// thresholds: §6050W(d)(1) merchant acquiring entity (Stripe, Square,
// traditional card processors) — NO de minimis; every dollar reportable.
// §6050W(d)(3) Third-Party Settlement Organization (PayPal, Venmo, Cash
// App, Zelle, eBay, Etsy, StubHub, Airbnb) — bouncing-ball threshold.
// OBBBA §70432 (eff. 2025-01-01) RETROACTIVELY restored the original
// $20,000 AND 200 transactions strict-greater-than threshold for 2025+,
// superseding the ARPA $600 nominal and IRS Notice 2024-85 transitional
// $5K/$2,500. Historical years 2022 (ARPA $600), 2023 (delayed to $20K/200
// per Notice 2023-74), 2024 (transitional $5K per Notice 2024-85) pinned
// for pre-2025 accuracy.

async fn section_6050w_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6050w::Section6050WInput>,
) -> Result<Json<traderview_expense::section_6050w::Section6050WResult>, ApiError> {
    if b.gross_amount_cents < 0 {
        return Err(ApiError::BadRequest(
            "gross_amount_cents must be non-negative".into(),
        ));
    }
    if !(1990..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest("year must be in [1990, 2100]".into()));
    }
    Ok(Json(traderview_expense::section_6050w::compute(&b)))
}

// ── §6651 failure-to-file / failure-to-pay penalty ──────────────────
// ── §6212 statutory notice of deficiency (SNOD / 90-day letter) ─────
// Mounted at /api/calc/section-6212. § 6212(a) Secretary authority to
// issue SNOD by CERTIFIED or REGISTERED mail when § 6211 deficiency
// exists; § 6212(b) load-bearing LAST KNOWN ADDRESS rule (Treas. Reg.
// § 301.6212-2) — mailing to wrong address renders SNOD INVALID and
// any subsequent assessment also invalid; § 6212(c) generally ONE
// SNOD per taxable year (exceptions: fraud, substantial omission,
// § 6861 jeopardy assessment, bankruptcy); § 6212(d) rescission with
// taxpayer's WRITTEN consent — SNOD treated as if never issued + §
// 6212(c) one-per-year limit does not bar subsequent re-issued SNOD;
// § 6213(a) petition deadline 90 days (or 150 days if taxpayer
// address outside US) + restraint on assessment during petition
// window and while petition pending; Hopkins v. Commissioner (T.C.
// 2024) — taxpayer may equitably rely on stated "last day to file"
// date even when incorrect. Natural sibling to section_6213 (Tax
// Court petition deadline + restrictions on assessment), section_6501
// (ASED), section_6502 (CSED).

async fn section_6212_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6212::Section6212Input>,
) -> Result<Json<traderview_expense::section_6212::Section6212Result>, ApiError> {
    Ok(Json(traderview_expense::section_6212::check(&b)))
}

// ── §6213 Tax Court petition 90-day rule ────────────────────────────
// Mounted at /api/calc/section-6213. § 6213(a) 90-day standard period
// (150 days if notice addressed to person outside US) for filing
// petition with Tax Court for redetermination of deficiency. Weekend/
// DC-holiday-at-last-day extension. § 6213(a) last sentence — petition
// timely if filed on or before Secretary-specified date in notice of
// deficiency (even if later than 90/150 days). § 6213(c) failure to
// file → assessment on notice and demand. Hallmark Research
// Collective (159 T.C. No. 6, 2022) holds deadline JURISDICTIONAL;
// Culp v. Commissioner (75 F.4th 196, 3d Cir. 2023) holds non-
// jurisdictional — circuit split. Trader-relevant when receiving IRS
// notice asserting § 475(f) MTM election was untimely or TTS criteria
// not satisfied — 90-day clock starts on notice mailing date.

async fn section_6213_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6213::Section6213Input>,
) -> Result<Json<traderview_expense::section_6213::Section6213Result>, ApiError> {
    if b.days_from_mailing_to_petition > 100_000 {
        return Err(ApiError::BadRequest(
            "days_from_mailing_to_petition looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6213::compute(&b)))
}

// ── §6201 Assessment authority ──────────────────────────────────────
// Mounted at /api/calc/section-6201. Foundational grant of IRS power
// to determine and assess tax liability. § 6201(a)(1) taxes shown on
// return; § 6201(a)(2) stamp taxes; § 6201(a)(3) erroneous prepayment
// credits assessed as math/clerical error WITHOUT § 6213(b)(2)
// abatement availability; § 6201(b) deficiency restriction
// cross-references § 6213(a) (SNOD + 90-day Tax Court window
// prerequisite); § 6201(c) child compensation assessment;
// § 6201(d) (RRA 98 § 3201) burden-shifting rule — Secretary bears
// burden of producing reasonable and probative information beyond
// information return itself when taxpayer asserts reasonable dispute
// AND fully cooperates; trader-critical for 1099-B + 1099-K + K-1
// disputes. Procedural predicate for § 6203 (method of assessment)
// + § 6303 (notice and demand) + § 6321 (lien) + § 6331 (levy).
async fn section_6201_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6201::Section6201Input>,
) -> Result<Json<traderview_expense::section_6201::Section6201Result>, ApiError> {
    Ok(Json(traderview_expense::section_6201::check(&b)))
}

// ── §6203 Method of assessment ──────────────────────────────────────
// Mounted at /api/calc/section-6203. Mechanical procedure by which
// IRS assessment under § 6201 becomes effective. § 6203 — assessment
// made by recording liability of taxpayer in office of Secretary;
// upon request of taxpayer, Secretary shall furnish taxpayer copy
// of record of assessment. 26 CFR § 301.6203-1 — assessment officer
// signs summary record providing (1) identification of taxpayer,
// (2) character of liability, (3) taxable period if applicable, (4)
// amount of assessment. Form 23-C signed assessment certificate
// (internal IRS document, NOT released to taxpayers); Form 4340
// Certificate of Assessments is the document IRS provides per Rev.
// Rul. 2007-21 and is presumptive evidence per March v. IRS, 335
// F.3d 1186 (10th Cir. 2003). Trader-procedural-critical because
// no lawful § 6321 lien attachment OR § 6331 levy authority engages
// without valid § 6203 record of assessment.
async fn section_6203_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6203::Section6203Input>,
) -> Result<Json<traderview_expense::section_6203::Section6203Result>, ApiError> {
    Ok(Json(traderview_expense::section_6203::check(&b)))
}

// ── §6303 notice and demand for tax ─────────────────────────────────
// Mounted at /api/calc/section-6303. § 6303(a) — Secretary shall,
// within 60 days after assessment under § 6203, give notice to each
// person liable for unpaid tax stating amount and demanding payment.
// § 6303(a) manner of delivery — (1) left at dwelling; (2) left at
// usual place of business; or (3) sent by mail to last known address.
// Certified mail NOT required. § 6303(a) failure to give notice
// within 60 days does NOT invalidate notice. § 6303(b) — if tax
// assessed BEFORE last date prescribed for payment, demand shall not
// be made until AFTER such date (except jeopardy finding under
// § 6861/§ 6862 with § 7429 review). Foundational predicate for
// § 6321 lien attachment + § 6331 levy authority (10-day neglect
// rule begins after notice and demand). Trader-relevant because no
// lawful IRS lien, levy, or seizure may proceed without proper
// § 6303 notice and demand. 26 CFR § 301.6303-1.

async fn section_6303_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6303::Section6303Input>,
) -> Result<Json<traderview_expense::section_6303::Section6303Result>, ApiError> {
    Ok(Json(traderview_expense::section_6303::check(&b)))
}

// ── §6304 Fair Tax Collection Practices ─────────────────────────────
// Mounted at /api/calc/section-6304. RRA 98 § 3466-imported FDCPA
// (15 USC § 1692) protections: § 6304(a) communications (8 a.m.-9
// p.m. local time default convenient window; represented-taxpayer
// bypass prohibition under § 7521; workplace-contact restriction
// when employer prohibits); § 6304(b) harassment and abuse
// prohibitions (threats, obscene language, repeated phone ringing,
// anonymous calls without identity disclosure); § 6304(c) civil
// damages via § 7433 (capped $1M reckless/intentional, $100K
// negligence). Trader-relevant when revenue officer or § 6306
// private collection agency contractor uses abusive collection
// tactics against trader-taxpayer.
async fn section_6304_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6304::Section6304Input>,
) -> Result<Json<traderview_expense::section_6304::Section6304Result>, ApiError> {
    Ok(Json(traderview_expense::section_6304::check(&b)))
}

// ── §6306 Qualified Tax Collection Contracts (PCAs) ─────────────────
// Mounted at /api/calc/section-6306. American Jobs Creation Act of
// 2004 § 881-added private collection agency program; made
// MANDATORY by FAST Act of 2015 § 32102 for inactive tax
// receivables. § 6306(c) inactive defined as (1) removed from
// active inventory, (2) > 2 years post-assessment unassigned, or
// (3) > 365 days no contact on assigned receivable. § 6306(d) 8+
// exclusion categories (pending OIC/IA, innocent spouse, deceased,
// under 18, combat zone, identity theft, examination/litigation/
// criminal/levy, appeals, disability/SSI under § 223 or title XVI,
// AGI ≤ 200% federal poverty level per Taxpayer First Act of 2019
// § 1205). § 6306(b) 7-year installment agreement cap. § 6306(e)
// PCA restrictions (no enforcement, no § 7521(b)(2) audio
// recording). § 6306(f) § 6304 + § 7433 + FDCPA extend to PCA
// contractor. § 6306(j) 25%/25%/50% revenue split. Trader-relevant
// for old IRS receivables that become inactive and get assigned
// to one of four authorized PCAs (CBE Group, Coast Professional,
// ConServe, Pioneer Credit Recovery).
async fn section_6306_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6306::Section6306Input>,
) -> Result<Json<traderview_expense::section_6306::Section6306Result>, ApiError> {
    Ok(Json(traderview_expense::section_6306::check(&b)))
}

// ── §6320 Collection Due Process (CDP) for liens ────────────────────
// Mounted at /api/calc/section-6320. Parallel framework to § 6330
// (CDP for levies). § 6320(a)(2)(B) 5-business-day notice deadline
// (Letter 3172) after NFTL filing + § 6320(a)(3)(B) 30-day CDP
// request window starting day AFTER 5-business-day notice period +
// § 6320(b)(1) fair CDP hearing right + § 6320(c) issues considered
// (incorporates § 6330(c) collection alternatives + spousal defenses
// + underlying-liability gating + lien-specific § 6323(j) WITHDRAWAL,
// § 6325(d) SUBORDINATION, § 6325(b) DISCHARGE) + § 6320(d) Tax Court
// review (incorporates § 6330(d)(1) 30-day petition window). Key
// difference vs § 6330 — lien REMAINS in place during CDP review;
// no automatic withdrawal. Boechler v. Commissioner (596 U.S. 199,
// 2022) holding likely extends via § 6320(d) incorporation of
// § 6330(d)(1). Trader-relevant when receiving Letter 3172 after IRS
// files Notice of Federal Tax Lien.

async fn section_6320_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6320::Section6320Input>,
) -> Result<Json<traderview_expense::section_6320::Section6320Result>, ApiError> {
    if b.business_days_from_nftl_filing_to_notice > 10_000
        || b.days_from_notice_to_cdp_request > 100_000
        || b.days_from_determination_to_tax_court_petition > 100_000
    {
        return Err(ApiError::BadRequest(
            "day inputs out of plausible range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6320::compute(&b)))
}

// ── §6321 lien for taxes (foundational IRS general tax lien) ────────
// Mounted at /api/calc/section-6321. § 6321 three-element test for
// automatic federal tax lien arising by operation of law: (1)
// assessment by IRS under § 6201 + (2) notice and demand for payment
// under § 6303 + (3) taxpayer neglects or refuses to pay after demand.
// When all three present, lien arises AUTOMATICALLY upon ALL property
// and rights to property of taxpayer (real + personal + tangible +
// intangible), relating back to assessment date. NFTL filing under §
// 6323(f) is NOT required for lien to ATTACH (only for priority
// against third parties under § 6323). § 6322 lien continues until
// liability satisfied OR becomes unenforceable by lapse of time
// (paired with § 6502 10-year CSED). Drye v. United States, 528 U.S.
// 49 (1999) — lien attaches to whatever interest state law gives
// taxpayer; United States v. Craft, 535 U.S. 274 (2002) — tenancy by
// entirety property still subject to lien. Trader-relevant for trader-
// landlords facing automatic lien exposure on rental property
// holdings. Foundational lien-constellation companion to § 6322 +
// § 6323 + § 6325 + § 6334 (exempt property) + § 7426 (third-party
// wrongful levy) + § 7433 (unauthorized collection damages). IRM
// 5.17.2.

async fn section_6321_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6321::Section6321Input>,
) -> Result<Json<traderview_expense::section_6321::Section6321Result>, ApiError> {
    Ok(Json(traderview_expense::section_6321::check(&b)))
}

// ── §6323 federal tax lien validity / priority against third parties ─
// Mounted at /api/calc/section-6323. § 6323(a) four protected classes
// — lien NOT valid against (1) purchaser; (2) holder of security
// interest; (3) mechanic's lienor; (4) judgment lien creditor UNTIL
// NFTL filed under § 6323(f); first-in-time wins among NFTL filing
// and competing perfection. § 6323(b) ten superpriorities — priority
// OVER federal tax lien EVEN AFTER NFTL filed when interest arose
// without actual notice: (1) securities; (2) motor vehicles; (3)
// retail purchase; (4) casual sale; (5) possessory lien; (6) real
// property tax/special assessment; (7) residential mechanic's lien
// (repair/improvement); (8) attorney's lien; (9) insurance contracts;
// (10) passbook loans. § 6323(c)+(d) 45-day window for commercial
// transactions financing agreements + after-acquired personal
// property without actual notice. § 6323(g) NFTL refiling required
// every 10 years (paired with § 6502 CSED). Trader-relevant for
// trader-landlords whose rental property holdings interact with
// mortgages + judgment liens + mechanics' liens + secured creditors.
// Foundational lien-priority constellation companion to § 6321 (lien
// attachment) + § 6322 (period of lien) + § 6325 (release) + § 6334
// (exempt property) + § 7426 (third-party wrongful levy). Rev. Rul.
// 2003-108; IRM 5.12.1, 5.12.2, 5.12.7, 5.12.8.

async fn section_6323_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6323::Section6323Input>,
) -> Result<Json<traderview_expense::section_6323::Section6323Result>, ApiError> {
    Ok(Json(traderview_expense::section_6323::check(&b)))
}

// ── §6325 release of lien or discharge of property ──────────────────
// Mounted at /api/calc/section-6325. § 6325(a) RELEASE — Secretary
// SHALL issue certificate of release within 30 days upon (1) full
// satisfaction OR legally unenforceable OR (2) bond accepted;
// extinguishes lien entirely. § 6325(b) DISCHARGE of specific
// property: (b)(1) double-value rule (property ≥ 2× (lien + senior
// liens)); (b)(2)(A) partial payment for US net interest; (b)(2)(B)
// no-value determination; (b)(3) proceeds substituted; (b)(4)
// purchaser deposit. § 6325(d) SUBORDINATION: (d)(1) payment for
// subordinated amount OR (d)(2) ultimate collection facilitated
// (typical trader-landlord mortgage refinance to extract equity for
// IRS payment). § 6325(e) NON-ATTACHMENT certificate (confusion-of-
// names cases). § 6325(f) — certificates are CONCLUSIVE. Trader-
// relevant for trader-landlords seeking to (a) extinguish lien upon
// full payment, (b) discharge individual rental property for sale
// or refinance, or (c) subordinate IRS lien to allow junior
// financing. Completes lien constellation: § 6321 + § 6322 + § 6323
// + § 6325 + § 6334 + § 7426. 26 CFR § 301.6325-1; IRM 5.12.10; IRS
// Pub. 783 (Discharge); IRS Pub. 784 (Subordination).

async fn section_6325_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6325::Section6325Input>,
) -> Result<Json<traderview_expense::section_6325::Section6325Result>, ApiError> {
    Ok(Json(traderview_expense::section_6325::check(&b)))
}

// ── §6330 Collection Due Process (CDP) for levies ───────────────────
// Mounted at /api/calc/section-6330. § 6330(a) 30-day pre-levy notice
// + § 6330(b) right to fair CDP hearing before IRS Appeals + § 6330(c)
// matters at hearing (collection alternatives — § 6159 installment
// agreement / § 7122 offer in compromise / currently not collectible
// — + spousal defenses + underlying-liability challenge if no prior
// opportunity) + § 6330(d)(1) 30-day Tax Court petition + § 6330(e)
// collection suspension during pending review + § 6330(f) jeopardy /
// state refund / Federal contractor / disqualified employment tax
// levy exceptions. Boechler v. Commissioner (596 U.S. 199, 2022)
// UNANIMOUSLY held § 6330(d)(1) deadline is NON-jurisdictional and
// SUBJECT TO equitable tolling — sharp contrast to § 6213(a)
// deficiency petition deadline (Hallmark Research Collective).
// Trader-relevant when receiving IRS Final Notice of Intent to Levy
// (Letter 1058 / LT-11).

async fn section_6330_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6330::Section6330Input>,
) -> Result<Json<traderview_expense::section_6330::Section6330Result>, ApiError> {
    if b.days_from_final_notice_to_cdp_request > 100_000
        || b.days_from_determination_to_tax_court_petition > 100_000
    {
        return Err(ApiError::BadRequest(
            "day inputs out of plausible range (>100000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6330::compute(&b)))
}

// ── §6331 levy and distraint authority ──────────────────────────────
// Mounted at /api/calc/section-6331. § 6331(a) — Secretary may levy
// upon property of taxpayer who has failed to pay tax within 10 days
// after § 6303 notice and demand. § 6331(d) — 30-day pre-levy notice
// required (in person, dwelling/place of business, or certified/
// registered mail to last known address). § 6331(e) continuous wage
// levy — attaches to (1) wages earned but not yet paid; (2) advances
// subsequent to levy; (3) wages becoming payable subsequent to levy;
// continues until released. § 6331(h) — continuous levy on up to 15%
// of specified federal payments (Social Security + federal employee
// retirement). § 6331(j) jeopardy levy exception — 30-day pre-levy
// notice DOES NOT apply if Secretary finds collection in jeopardy
// (paired with § 6861/§ 6862 jeopardy assessment + § 7429 judicial
// review). § 6331(k) — no levy while (1) innocent spouse relief
// request under § 6015 pending OR (2) CDP hearing under § 6330
// pending. Foundational levy statute. Trader-relevant for any
// taxpayer facing IRS levy threat. Pair with § 6321 (lien) + § 6323
// (priority) + § 6325 (release) + § 6334 (exempt property). 26 CFR
// § 301.6331-1; IRM 5.17.3.

async fn section_6331_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6331::Section6331Input>,
) -> Result<Json<traderview_expense::section_6331::Section6331Result>, ApiError> {
    Ok(Json(traderview_expense::section_6331::check(&b)))
}

// ── §6332 surrender of property subject to levy ─────────────────────
// Mounted at /api/calc/section-6332. § 6332(a) any person in
// possession of property subject to levy must surrender upon demand
// by Secretary. § 6332(c) 21-day bank hold: banks surrender deposits
// ONLY AFTER 21 days after service of levy (error-correction window).
// § 6332(b) wage/salary cross-references § 6331(e) continuous wage
// levy. § 6332(d)(1) personal liability — failure to surrender =
// liability equal to value of property NOT surrendered, capped at
// tax + costs + § 6621 underpayment interest. § 6332(d)(2) 50%
// additional penalty for failure WITHOUT REASONABLE CAUSE; NO credit
// against underlying tax. § 6332(e) discharge safe harbor —
// compliant surrender DISCHARGES third party from any obligation to
// delinquent taxpayer. Trader-relevant on both sides: trader-traders
// facing IRS levy on brokerage accounts (broker as third party);
// trader-landlords as third-party levy recipients (employers,
// vendors). Pair with § 6331 (levy authority) + § 6321 (lien) +
// § 6303 (notice and demand) + § 6334 (exempt property) + § 7426
// (third-party wrongful levy INVERSE pathway). 26 CFR § 301.6332-1;
// IRM 5.17.3.

async fn section_6332_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6332::Section6332Input>,
) -> Result<Json<traderview_expense::section_6332::Section6332Result>, ApiError> {
    Ok(Json(traderview_expense::section_6332::check(&b)))
}

// ── §6334 property exempt from levy ─────────────────────────────────
// Mounted at /api/calc/section-6334. § 6334(a) thirteen enumerated
// exemption categories: (1) wearing apparel + school books; (2) fuel
// + provisions + furniture + household ≤ $11,980 (2026 indexed); (3)
// books + tools of trade ≤ $5,990 (2026 indexed); (4) unemployment;
// (5) undelivered mail; (6) annuity/pension; (7) workmen's comp; (8)
// child support; (9) wage minimum exemption; (10) military disability;
// (11) public assistance; (12) Job Training Partnership Act; (13)
// residence in small-deficiency cases (unpaid tax ≤ $5,000). §
// 6334(d)(4)(B) — 2026 wage exemption parameter $5,300. § 6334(e)(1)
// — principal residence (§ 121) requires district court judge or
// magistrate WRITTEN approval before levy; district courts have
// EXCLUSIVE jurisdiction. § 6334(e)(2) — self-employed assets +
// non-rental residential real property require IRS area director
// approval. Trader-relevant for tools-of-trade exemption (trading
// rigs / monitors / books), wage exemption, principal-residence
// judicial-approval gate. Companion to § 7421 (Anti-Injunction Act +
// § 7426 wrongful-levy exception) + § 7433 (civil damages for
// unauthorized collection) + § 7430 (litigation costs) + § 7811
// (TAOs). Rev. Proc. 2025-32 + Pub. L. 119-21 (OBBBA).

async fn section_6334_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6334::Section6334Input>,
) -> Result<Json<traderview_expense::section_6334::Section6334Result>, ApiError> {
    Ok(Json(traderview_expense::section_6334::check(&b)))
}

// ── §6402 refund offsets / Treasury Offset Program ──────────────────
// Mounted at /api/calc/section-6402. § 6402 statutory hierarchy
// applies overpayments to debts in priority order: § 6402(a) IRS
// internal revenue tax (IRS handles directly), § 6402(c)(1) past-due
// child support ASSIGNED to a State, § 6402(d) federal agency non-tax
// debt (student loans etc.), § 6402(c)(2) child support NOT assigned
// to a State, § 6402(e) state income tax, § 6402(f) state unemployment
// compensation, § 6402(g) state TANF. § 6402(n) injured spouse rule
// (Form 8379) protects non-debtor spouse's share of joint refund.
// Centralized administration: Treasury Offset Program (TOP) under
// Bureau of the Fiscal Service since 1999.

async fn section_6402_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6402::Section6402Input>,
) -> Result<Json<traderview_expense::section_6402::Section6402Result>, ApiError> {
    if b.injured_spouse_share_bps > 100_000 {
        return Err(ApiError::BadRequest(
            "injured_spouse_share_bps out of plausible range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6402::compute(&b)))
}

// ── §7502 timely mailing treated as timely filing ───────────────────
// Mounted at /api/calc/section-7502. § 7502(a) US postmark = filing
// date when postmark within prescribed period + envelope properly
// addressed + sufficient postage; § 7502(c)(1) registered mail prima
// facie evidence; § 7502(c)(2) certified mail registration = postmark
// date; § 7502(f) designated PDS per Notice 2016-30 (FedEx First/
// Priority/Standard Overnight, 2 Day, International Priority/First/
// Economy; UPS Next Day Air variants, 2nd Day Air variants, Worldwide
// Express; DHL Express 9:00/10:30/12:00, Worldwide, Envelope, Import
// Express variants). Non-designated services (FedEx Ground, UPS
// Ground, FedEx Home Delivery) DO NOT qualify. Electronic filing
// governed by 26 CFR § 301.7502-1(d) e-file acknowledgment timestamp.
// Anderson v. United States (9th Cir. 1992) — § 7502 displaces common-
// law mailbox rule. Critical paired with § 6213(a) Hallmark jurisdic-
// tional deadlines and § 6330(d) Boechler equitable tolling.

async fn section_7502_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7502::Section7502Input>,
) -> Result<Json<traderview_expense::section_7502::Section7502Result>, ApiError> {
    Ok(Json(traderview_expense::section_7502::compute(&b)))
}

// ── §7503 weekend/holiday extension rule ────────────────────────────
// Mounted at /api/calc/section-7503. § 7503 — when last day for
// performing any act falls on Saturday, Sunday, or legal holiday,
// performance is timely if performed on next succeeding business
// day. Legal holiday defined: (1) legal holiday in District of
// Columbia (5 USC § 6103 — includes Juneteenth since 2021) AND (2)
// statewide legal holiday in State where office located outside DC
// but within internal revenue district. DC Emancipation Day (April
// 16, Rev. Rul. 2015-13) regularly extends federal tax filing
// deadline by 1 business day when April 15 falls on weekend. § 7503
// stacks with § 7502 timely-mailing rule. Applies to taxpayer acts
// (return filing + payment + § 6213 Tax Court petition + § 6511
// refund claim + elections) AND Commissioner acts (§ 6212 SNOD +
// § 6303 notice and demand + § 6851 termination notice).
async fn section_7503_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7503::Section7503Input>,
) -> Result<Json<traderview_expense::section_7503::Section7503Result>, ApiError> {
    Ok(Json(traderview_expense::section_7503::check(&b)))
}

// ── §7508A presidentially-declared disaster deadline postponement ───
// Mounted at /api/calc/section-7508a. § 7508A(a) Secretary's
// discretionary postponement (up to ONE YEAR / 365 days) for taxpayers
// affected by federally declared disaster (Stafford Act / 42 USC §
// 5121 et seq.) or significant fire. § 7508A(b) terroristic or
// military action postponement. § 7508A(c) special rules for pensions
// + retirement plan loan repayments. § 7508A(d) MANDATORY 60-day
// postponement period for federally declared disasters with specified
// incident date declared after December 20, 2019 (Taxpayer Certainty
// and Disaster Tax Relief Act of 2019, Pub. L. 116-94 Div. Q § 205);
// runs CONCURRENTLY with Secretary's discretionary postponement if
// Secretary period ≥ 60 days. Disaster area defined under § 1033(h)
// (3) = area eligible for federal assistance under Stafford Act.
// Postponed acts include filing returns + paying tax + filing amended
// returns + Tax Court petitions + § 6511 refund claims + § 6212 SNOD
// responses + § 6213 deficiency challenges. Trader-relevant for any
// trader in federally-declared disaster area (CA wildfires + FL
// hurricanes + TX flooding + tornado disasters) needing to extend
// filing/payment/refund-claim deadlines. Procedural-companion to §
// 7421 + § 7426 + § 7433 + § 7430 + § 6212 + § 6213 + § 6511. 26 CFR
// § 301.7508A-1.

async fn section_7508a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7508a::Section7508AInput>,
) -> Result<Json<traderview_expense::section_7508a::Section7508AResult>, ApiError> {
    Ok(Json(traderview_expense::section_7508a::check(&b)))
}

// ── §7508 combat zone / contingency operation postponement ──────────
// Mounted at /api/calc/section-7508. Trader-relevant for active-duty
// military traders + military spouses serving in combat zones,
// contingency operations, or qualified hazardous duty areas. § 7508(a)
// IRS DISREGARDS time during (1) combat zone service (Executive Order
// designated), (2) Secretary of Defense designated contingency
// operation outside US, or (3) qualified hazardous duty area, PLUS
// 180 days after last day in such area / operation / qualified
// hospitalization. Hospitalization INSIDE the United States capped at
// 5 years (1825 days); hospitalization OUTSIDE not capped. § 7508(b)
// military spouse extension. § 7508(c) qualified hazardous duty area
// includes Sinai Peninsula. Postponed acts include filing returns +
// paying tax + § 6511 refund claims + § 6212 SNOD + § 6213
// deficiency. Distinct from § 7508A presidentially-declared disaster
// postponement (different qualifying event). 26 CFR § 301.7508-1; IRS
// Notice 2003-21; IRS Form 15109; IRS Pub. 3 Armed Forces' Tax Guide.

async fn section_7508_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7508::Section7508Input>,
) -> Result<Json<traderview_expense::section_7508::Section7508Result>, ApiError> {
    Ok(Json(traderview_expense::section_7508::check(&b)))
}

// ── §7811 Taxpayer Assistance Orders (TAOs) ─────────────────────────
// Mounted at /api/calc/section-7811. § 7811(a)(1) National Taxpayer
// Advocate may issue TAO on Form 911 application if taxpayer suffering
// or about to suffer significant hardship. § 7811(a)(2) four enumerated
// hardship categories: (A) immediate adverse action, (B) delay > 30
// days, (C) significant costs, (D) irreparable injury. § 7811(b) TAO
// may order IRS to release levied property OR cease/take/refrain from
// action. § 7811(c) modification or rescission limited to NTA /
// Commissioner / Deputy Commissioner. § 7811(d) statute of limitations
// suspended during application + decision period. § 7811(e) TAO
// INDEPENDENT of other remedies (CDP, Tax Court, refund litigation).
// Trader-relevant for IRS administrative actions causing hardship.

async fn section_7811_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7811::Section7811Input>,
) -> Result<Json<traderview_expense::section_7811::Section7811Result>, ApiError> {
    Ok(Json(traderview_expense::section_7811::compute(&b)))
}

// ── §7521 procedures involving taxpayer interviews ──────────────────
// Mounted at /api/calc/section-7521. § 7521(a)(1) taxpayer recording
// right (advance request + own equipment + own expense). § 7521(a)(2)
// IRS recording requires advance notice + reimbursable transcript on
// taxpayer request. § 7521(b)(1)(A) explanation of audit process for
// tax determination interviews; § 7521(b)(1)(B) explanation of
// collection process for collection interviews. § 7521(c) right to
// representation via attorney / CPA / enrolled agent / enrolled
// actuary / authorized rep with Form 2848 power of attorney + IRS
// MUST suspend interview when taxpayer requests representation
// consultation. § 7521(c) administrative-summons exception bars
// suspension right. § 7521(c) delay bypass with Immediate Supervisor
// consent. Trader-relevant for audit / collection / examination
// interviews — paired with § 7811 (TAOs) and § 6330/§ 6320 (CDP).

async fn section_7521_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7521::Section7521Input>,
) -> Result<Json<traderview_expense::section_7521::Section7521Result>, ApiError> {
    Ok(Json(traderview_expense::section_7521::compute(&b)))
}

// ── §7522 content of tax due, deficiency, and other notices ─────────
// Mounted at /api/calc/section-7522. Added by Taxpayer Bill of
// Rights of 1988 (TBOR 1, Pub. L. 100-647 § 6233). § 7522(a) any
// covered notice shall describe the basis for, and identify the
// amounts (if any) of, any tax due + interest + additional amounts
// + additions to tax + assessable penalties; SAFE HARBOR —
// inadequate description shall NOT INVALIDATE such notice.
// § 7522(b)(1) applies to § 6155 + § 6212 + § 6303 notices (CP14,
// Letter 1058, Letter 3171/5071C SNOD); § 7522(b)(2) applies to
// CP2000 Automated Underreporter notices generated from 1099-B +
// 1099-K + K-1 information return matching (30-day response / 60-
// day outside US); § 7522(b)(3) applies to Letter 525 first
// proposed-deficiency 30-day letter with IRS Independent Office of
// Appeals review opportunity (Taxpayer First Act of 2019 § 1001
// redesignation). Trader-procedural-critical content-disclosure
// layer over § 6201 + § 6203 + § 6212 + § 6303.
async fn section_7522_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7522::Section7522Input>,
) -> Result<Json<traderview_expense::section_7522::Section7522Result>, ApiError> {
    Ok(Json(traderview_expense::section_7522::check(&b)))
}

// ── §7525 federally authorized tax practitioner privilege ───────────
// Mounted at /api/calc/section-7525. § 7525(a)(1) extends attorney-
// client common-law privilege to CPA / EA / attorney / enrolled actuary
// / enrolled retirement plan agent (FATP under 31 USC § 330 / Circular
// 230). § 7525(a)(3)(A) noncriminal-only — categorically EXCLUDED from
// criminal tax matters (grand jury, indictment, IRS-CI referral) and
// state / local tax matters. § 7525(b) written tax-shelter-promotion
// communications categorically excluded (§ 6662(d)(2)(C)(ii) shelter
// definition). United States v. Frederick, 182 F.3d 496 (7th Cir.
// 1999) — return-preparation work NOT covered. Trader-relevant for
// protecting CPA / EA communications on M2M (§ 475(f)), straddle
// (§ 1092), § 1256 60/40, § 1091 wash sale, qualified trader status,
// § 988 / § 1297 / § 6038D international advice. Paired with § 7521
// (interview procedure) and § 7811 (TAOs).

async fn section_7525_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7525::Section7525Input>,
) -> Result<Json<traderview_expense::section_7525::Section7525Result>, ApiError> {
    Ok(Json(traderview_expense::section_7525::check(&b)))
}

// ── §7201 attempt to evade or defeat tax (apex criminal felony) ─────
// Mounted at /api/calc/section-7201. Apex criminal tax statute —
// 5-year FELONY with $250K individual / $500K corporation fine
// (18 U.S.C. § 3571 Criminal Fines Improvement Act supersedes
// § 7201 original $100K cap). Four-element test (BEYOND REASONABLE
// DOUBT burden on government): (1) existence of tax deficiency
// (additional tax owed) + (2) WILLFULNESS voluntary intentional
// violation of known duty + (3) AFFIRMATIVE ACT of evasion (Spies
// doctrine — omissions alone insufficient + mere failure to file
// or pay does NOT satisfy) + (4) SUBSTANTIAL amount. Spies v.
// United States, 317 U.S. 492 (1943) enumerates 7 affirmative-act
// indicia: double set of books + false entries + false invoices +
// destruction of records + concealment of assets + covering up
// income sources + handling affairs to avoid usual records. Two
// forms (Sansone v. United States, 380 U.S. 343 (1965)): evasion
// of ASSESSMENT (false return, hidden income) vs evasion of
// PAYMENT (concealment after assessment, transfers to nominees).
// Cheek v. United States, 498 U.S. 192 (1991) good-faith
// misunderstanding (subjective belief test) defeats willfulness.
// § 6531 criminal SOL 6 years. Spies-Daly doctrine permits
// PARALLEL civil § 6663 75% prosecution + § 6501(c)(1)/(c)(2)
// UNLIMITED ASED + § 7491 burden shifts do NOT apply. Pairs with
// section_7206 (3-year felony / tax perjury) + section_6663 (civil
// fraud 75%). IRM 9.1.3.

async fn section_7201_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7201::Section7201Input>,
) -> Result<Json<traderview_expense::section_7201::Section7201Result>, ApiError> {
    Ok(Json(traderview_expense::section_7201::check(&b)))
}

// ── §7202 willful failure to collect or pay over tax (felony) ───────
// Mounted at /api/calc/section-7202. Criminal FELONY (5-year
// imprisonment cap) + $250K individual / $500K corporation fine
// (18 U.S.C. § 3571 supersedes § 7202's original $10K cap).
// Criminal counterpart to § 6672 Trust Fund Recovery Penalty (civil
// 100%). Same conduct triggers BOTH § 7202 felony and § 6672
// 100% civil penalty per Spies-Daly doctrine. Four-element test
// (BEYOND REASONABLE DOUBT): (1) duty to collect / account for /
// pay over tax + (2) WILLFUL failure (voluntary intentional
// violation of known legal duty + no evil/bad intent required) +
// (3) amount required to be withheld and paid + (4) defendant was
// a RESPONSIBLE PERSON (status, duty, AND authority to avoid
// default; same standard as § 6672). Trust fund taxes reached:
// § 3402 income withholding + § 3101 employee FICA (Social
// Security + Medicare) + § 3301 FUTA; NOT REACHED: § 3111 employer
// FICA match. Cheek v. United States, 498 U.S. 192 (1991)
// good-faith subjective belief defeats willfulness. § 6531
// criminal SOL 6 years. Parallel civil: § 6672 TFRP (100%) +
// § 6651(a)(2) failure-to-pay (0.5%/month) + 11 U.S.C. § 523(a)(7)
// NONDISCHARGEABLE in bankruptcy. § 7491 burden shifts do NOT
// apply to criminal cases. IRM 9.1.3 + IRM 8.25.1. Pairs with
// section_6672 (civil counterpart) + section_7201 (felony evasion)
// + section_7203 (misdemeanor failure to file) + section_7206
// (felony perjury). Critical trader-business operational risk for
// any entity with W-2 employees.

async fn section_7202_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7202::Section7202Input>,
) -> Result<Json<traderview_expense::section_7202::Section7202Result>, ApiError> {
    Ok(Json(traderview_expense::section_7202::check(&b)))
}

// ── §7203 willful failure to file / pay / supply info (misdemeanor) ─
// Mounted at /api/calc/section-7203. Criminal MISDEMEANOR (1-year
// imprisonment cap) + $100K individual / $200K corporation fine
// (18 U.S.C. § 3571 supersedes § 7203 original $25K/$100K caps).
// Distinct from § 7201 5-year felony (requires affirmative acts)
// and § 7206 3-year felony (perjury). § 7203 reaches MERE
// OMISSIONS (failure to file return + failure to pay tax +
// failure to supply information + failure to keep records);
// Spies v. United States, 317 U.S. 492 (1943) — willful omission
// COUPLED with affirmative acts elevates to § 7201. Three-element
// test (BEYOND REASONABLE DOUBT): (1) required by law to file /
// pay / supply / keep records + (2) failure at time required +
// (3) WILLFULNESS. Cheek v. United States, 498 U.S. 192 (1991) —
// genuine good-faith subjective belief negates willfulness EVEN
// IF OBJECTIVELY UNREASONABLE; NOT a defense for constitutional
// challenges or tax-protester arguments. § 6050I FELONY
// EXCEPTION (cash reporting >$10K) elevates to 5 YEARS
// imprisonment. § 6531 criminal SOL 6 years. § 6651(a)(1) civil
// failure-to-file penalty (5%/month up to 25%) + § 6651(a)(2)
// civil failure-to-pay penalty (0.5%/month) PARALLEL prosecution
// permitted per Spies-Daly. § 6501(c)(3) UNLIMITED ASED when no
// return filed. § 7491 burden shifts do NOT apply to criminal
// cases. IRM 9.1.3. Completes criminal tax statute trio with
// section_7201 (felony 5-year apex) and section_7206 (3-year
// felony / perjury).

async fn section_7203_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7203::Section7203Input>,
) -> Result<Json<traderview_expense::section_7203::Section7203Result>, ApiError> {
    Ok(Json(traderview_expense::section_7203::check(&b)))
}

// ── §7212 attempts to interfere with administration (felony) ────────
// Mounted at /api/calc/section-7212. Criminal FELONY (3-year cap)
// + $250K individual / $500K corporation fine (18 U.S.C. § 3571
// supersedes original $5K cap). Two clauses: officer-specific
// (corruptly OR by force/threats endeavors to intimidate or
// impede IRS officer/employee in official capacity) + omnibus
// (any other way corruptly OR by force/threats obstructs or
// impedes due administration of Title 26). § 7212(a) threats-only
// downgrade: when offense committed ONLY by threats of force
// (no actual force + no corrupt act), 1-year misdemeanor +
// $3K fine. Marinello v. United States, 138 S. Ct. 1101 (2018)
// — omnibus clause requires NEXUS to known pending OR reasonably
// foreseeable proceeding (routine non-compliance with tax code
// requirements absent nexus does NOT constitute § 7212 violation).
// 'Corrupt' = act performed with INTENTION TO SECURE UNLAWFUL
// BENEFIT. § 6531 general 3-year criminal SOL applies (not 6).
// Spies-Daly parallel civil § 6663 fraud + § 6672 TFRP + § 6501
// (c)(1) UNLIMITED ASED for fraud. IRM 9.1.3.

async fn section_7212_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7212::Section7212Input>,
) -> Result<Json<traderview_expense::section_7212::Section7212Result>, ApiError> {
    Ok(Json(traderview_expense::section_7212::check(&b)))
}

// ── §7216 disclosure or use by preparers (criminal misdemeanor) ─────
// Mounted at /api/calc/section-7216. Pairs with § 6713 civil
// penalty ($250/disclosure + $10K annual cap). Criminal misdemeanor
// 1-year + $100K individual / $200K corporation fine (18 U.S.C. §
// 3571 supersedes original $1K cap) for preparer who knowingly or
// recklessly discloses or uses tax return info for purpose other
// than preparing return. § 7216(b) exceptions: taxpayer consent +
// non-consent permissible disclosures under 26 CFR § 301.7216-2.
// Consent must comply with 26 CFR § 301.7216-3 + Rev. Proc.
// 2013-14 (written + signed before disclosure + specific recipient
// + specific purpose + duration + prominent and separate).
// Identity-theft enhancement up to $100,000 separate from § 3571.
// § 6531 general 3-year criminal SOL. § 6713(a) no-fault civil
// penalty does not require knowing or reckless conduct. Pairs with
// section_6531 + section_6713 (civil counterpart). IRM 25.5.1 +
// IRM 4.10 preparer penalty procedural manuals.

async fn section_7216_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7216::Section7216Input>,
) -> Result<Json<traderview_expense::section_7216::Section7216Result>, ApiError> {
    Ok(Json(traderview_expense::section_7216::check(&b)))
}

// ── §7206 fraud and false statements (criminal felony) ──────────────
// Mounted at /api/calc/section-7206. Five enumerated criminal tax
// offenses: § 7206(1) tax perjury (workhorse statute) — willfully
// makes and subscribes return / statement / document containing
// declaration under penalty of perjury knowing it false as to
// material matter; § 7206(2) aiding or assisting preparation of
// false document — reaches return preparers, advisors, third
// parties even when taxpayer-signer innocent; § 7206(3) fraudulent
// bonds + permits + entries; § 7206(4) removal or concealment of
// taxed goods with intent to defraud; § 7206(5) compromises and
// closing agreement fraud under § 7121 / § 7122. Penalties: up to
// 3 YEARS imprisonment + fine $250K individual / $500K corporation
// (18 U.S.C. § 3571 Criminal Fines Improvement Act supersedes §
// 7206's original $100K cap) + costs of prosecution. § 7206(1)
// five-element test: (1) made and subscribed + (2) false as to
// material matter + (3) declaration under penalty of perjury + (4)
// did not believe true + (5) willful with specific intent.
// Cheek v. United States, 498 U.S. 192 (1991) good-faith
// misunderstanding defeats willfulness (subjective belief test).
// § 6531 criminal SOL: 6 years for § 7206(1)/(2)/(3)/(4); 3 years
// for § 7206(5). Spies-Daly doctrine permits parallel civil §
// 6663 75% fraud penalty + § 6501(c)(1) UNLIMITED ASED + § 6501
// (c)(2) UNLIMITED ASED for willful evasion. § 7491 burden shifts
// do NOT apply — government bears BEYOND REASONABLE DOUBT burden.
// IRM 9.1.3 Criminal Statutory Provisions and Common Law.

async fn section_7206_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7206::Section7206Input>,
) -> Result<Json<traderview_expense::section_7206::Section7206Result>, ApiError> {
    Ok(Json(traderview_expense::section_7206::check(&b)))
}

// ── §7207 fraudulent returns/statements/other documents (misdemeanor)
// Mounted at /api/calc/section-7207. Pairs with § 7206 (felony perjury
// alternative when document signed under penalties of perjury) and §
// 7434 (civil damages for fraudulent information return). Criminal
// MISDEMEANOR 1-year cap + $100K individual / $200K corporation fine
// (18 U.S.C. § 3571 supersedes § 7207's original $10K / $50K caps).
// Three-element test BEYOND REASONABLE DOUBT: (1) delivery or
// disclosure to IRS officer or employee of list/return/account/
// statement/other document + (2) document false or fraudulent as to
// material matter + (3) willfully or with knowledge of falsity.
// Broader scope than § 7206 (covers documents NOT signed under
// penalties of perjury) but lower penalty. Cheek v. United States,
// 498 U.S. 192 (1991) good-faith subjective belief defeats willfulness.
// § 6531(2) 6-year SOL (enumerated 6-year offense). Typical fact
// patterns: fabricated receipts during audit + altered K-1 + fraudulent
// supporting documents + false Form 433 collection information
// statement. Spies-Daly parallel civil § 7434 + § 6663 + § 6501(c)(1)
// UNLIMITED ASED. IRM 9.1.3.

async fn section_7207_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7207::Section7207Input>,
) -> Result<Json<traderview_expense::section_7207::Section7207Result>, ApiError> {
    Ok(Json(traderview_expense::section_7207::check(&b)))
}

// ── §7434 civil damages for fraudulent information return ──────────
// Mounted at /api/calc/section-7434. Trader-relevant CIVIL remedy
// when third party (employer / broker / payor) willfully files
// fraudulent W-2 / 1099 / other information return against
// taxpayer. § 7434(a) cause of action; § 7434(b) damages — greater
// of $5,000 OR actual damages + court costs + court's-discretion
// attorney fees; § 7434(d) statute of limitations — later of 6
// years from filing OR 1 year from reasonable discovery; § 7434(e)
// plaintiff must provide complaint copy to Secretary (IRS notice).
// Derolf v. Risinger Bros. misclassification carveout — most
// courts hold misclassification (1099 instead of W-2) without
// amount misstatement does NOT support § 7434 claim; plaintiff
// must allege FRAUDULENT AMOUNT MISSTATEMENT. Trader-relevant
// scenarios: broker files incorrect 1099-B inflating proceeds;
// employer files false W-2; payor files fake 1099-NEC; retaliatory
// false W-2 / 1099 from former employer. Civil judgment provides
// collateral-estoppel leverage in Tax Court / refund litigation
// arising from fraudulent 1099 / W-2 deficiency notice. Distinct
// from criminal statutes (§§ 7201 / 7202 / 7203 / 7206), civil
// fraud (§ 6663), and TFRP (§ 6672).

async fn section_7434_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7434::Section7434Input>,
) -> Result<Json<traderview_expense::section_7434::Section7434Result>, ApiError> {
    Ok(Json(traderview_expense::section_7434::check(&b)))
}

// ── §7463 disputes involving $50,000 or less (Tax Court small case) ─
// Mounted at /api/calc/section-7463. § 7463(a) — Tax Court small
// case procedure for petitions where amount in dispute does not
// exceed $50,000 per taxable year (income), per estate (estate
// tax), per calendar year (gift tax), or per period/event (excise
// tax); proceedings at option of taxpayer concurred by Tax Court
// BEFORE the hearing. § 7463(b) — decision NOT REVIEWED IN ANY
// OTHER COURT and NOT TREATED AS PRECEDENT. § 7463(c) — taxpayer
// or Secretary may discontinue designation before final decision.
// § 7463(d) — proceedings under Tax Court Rules 170-175; as
// informally as possible; any evidence with probative value
// admissible. § 7463(f) — also available for § 6320 (CDP-lien),
// § 6330 (CDP-levy), § 6015 (innocent spouse), § 7436 (worker
// classification) under $50,000. Procedural tradeoff: faster +
// cheaper + informal + pro se friendly BUT no appeal + no
// precedential value. Trader-relevant for smaller audit
// deficiencies seeking faster + cheaper Tax Court resolution.
async fn section_7463_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7463::Section7463Input>,
) -> Result<Json<traderview_expense::section_7463::Section7463Result>, ApiError> {
    Ok(Json(traderview_expense::section_7463::check(&b)))
}

// ── §7491 burden of proof shifts to Secretary ───────────────────────
// Mounted at /api/calc/section-7491. § 7491(a)(1) general burden
// shift on factual issues under Subtitle A income tax / B estate-
// gift-GST when taxpayer introduces CREDIBLE EVIDENCE (court would
// find sufficient to base decision on); § 7491(a)(2) three
// threshold conditions (A) substantiation + (B) records maintained
// + (C) cooperation with reasonable IRS requests for witnesses /
// information / documents / meetings / interviews; § 7491(a)(2)(C)
// net worth limitation — corporations + partnerships + trusts with
// net worth EXCEEDING $7,000,000 EXCLUDED from (a)(1) shifting
// (individuals + estates unlimited); § 7491(b) statistical
// reconstruction burden — Secretary bears burden on any income
// item reconstructed by statistical methods from unrelated
// taxpayers (BLS surveys, market-segment analysis) for INDIVIDUAL
// Subtitle A; § 7491(c) penalty PRODUCTION burden (not persuasion)
// for any penalty or addition to tax including § 6651, § 6662, §
// 6663, § 6672. Enacted under IRS Restructuring and Reform Act of
// 1998 (Pub. L. No. 105-206). Cross-references § 7454(a) fraud +
// accumulated earnings burden + § 6664(c) reasonable cause
// defense. Highly relevant to trader-tax controversy on § 1256
// MTM, § 988 currency, § 1202 QSBS, § 475(f) trader-tax-status.

async fn section_7491_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7491::Section7491Input>,
) -> Result<Json<traderview_expense::section_7491::Section7491Result>, ApiError> {
    Ok(Json(traderview_expense::section_7491::check(&b)))
}

// ── §7430 awarding of costs and certain fees ────────────────────────
// Mounted at /api/calc/section-7430. § 7430(a) court may award
// reasonable administrative + litigation costs to prevailing party
// against the IRS. § 7430(b)(1) exhaustion of administrative remedies
// required. § 7430(c)(4)(A) prevailing party = substantially prevailed
// on amount or most significant issue. § 7430(c)(4)(B) IRS substantial
// justification defense defeats prevailing party status. § 7430(c)(4)
// (D) + 28 U.S.C. § 2412(d)(2)(B) net worth limits — individual ≤ $2M,
// business entity ≤ $7M, 500-employee ceiling. § 7430(c)(7) qualified
// offer rule — taxpayer treated as prevailing party if QO liability ≥
// judgment. § 7430(c)(1)(B)(iii) hourly cap — 2026: $260/hour per
// Rev. Proc. 2025-32. Trader-relevant when prevailing against IRS in
// Tax Court or refund litigation.

async fn section_7430_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7430::Section7430Input>,
) -> Result<Json<traderview_expense::section_7430::Section7430Result>, ApiError> {
    if b.employee_count_at_filing > 1_000_000_000 {
        return Err(ApiError::BadRequest(
            "employee_count_at_filing looks invalid".into(),
        ));
    }
    if b.attorney_hours_billed > 1_000_000 {
        return Err(ApiError::BadRequest(
            "attorney_hours_billed looks invalid (>1000000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_7430::compute(&b)))
}

// ── §7421 Anti-Injunction Act (AIA) ─────────────────────────────────
// Mounted at /api/calc/section-7421. § 7421(a) general bar — "no suit
// for the purpose of restraining the assessment or collection of any
// tax shall be maintained in any court by any person." Eleven
// statutory exceptions: §§ 6015(e) + 6212(a)+(c) + 6213(a) + 6232(c)
// + 6330(e)(1) + 6331(i) + 6672(c) + 6694(c) + 7426(a)+(b)(1) +
// 7429(b) + 7436. Enochs v. Williams Packing, 370 U.S. 1 (1962)
// judicial 2-prong exception: (1) government cannot ultimately
// prevail AND (2) equity jurisdiction exists; BOTH required
// conjunctively. CIC Services v. IRS, 593 U.S. 209 (2021) — pre-
// enforcement challenge to IRS reporting requirement / regulation is
// NOT a suit to restrain assessment or collection within § 7421(a).
// Trader-procedural-critical: default answer for TRO/preliminary
// injunction against IRS levy/lien/assessment = NEVER. Default
// pathway: pay tax + file refund claim under § 6402/§ 7422 + refund
// suit. Paired with § 7521 (interview procedure) + § 7525 (FATP
// privilege) + § 7811 (TAOs) + § 7430 (litigation costs).

async fn section_7421_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7421::Section7421Input>,
) -> Result<Json<traderview_expense::section_7421::Section7421Result>, ApiError> {
    Ok(Json(traderview_expense::section_7421::check(&b)))
}

// ── §7422 civil actions for refund ──────────────────────────────────
// Mounted at /api/calc/section-7422. Completes refund-procedure
// constellation. Four pre-suit requirements: (1) Flora full-payment
// rule (Flora v. United States, 362 U.S. 145 (1960) — taxpayer must
// FULLY PAY assessment before suing in district court / Court of
// Federal Claims); (2) administrative claim filed under § 6511
// (within later of 3 years after return filing or 2 years after
// payment); (3) § 6532(a) 6-month wait period (180 days from admin
// claim filing, unless IRS issues disallowance sooner); (4) §
// 6532(a) 2-year filing window after IRS mails notice of
// disallowance. § 7422(e) concurrent jurisdiction limitation: if
// Secretary mails notice of deficiency BEFORE hearing, proceedings
// stayed during Tax Court petition window + 60 days; if taxpayer
// files Tax Court petition, district court / Court of Federal
// Claims loses jurisdiction to extent acquired by Tax Court.
// Jurisdiction: district court (28 USC § 1346(a)(1)) concurrent
// with Court of Federal Claims (28 USC § 1491). Pair with § 7421
// AIA exception (refund-after-payment is AIA-exception pathway) +
// § 7508A disaster postponement of § 6511 deadlines + § 7430
// litigation costs.

async fn section_7422_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7422::Section7422Input>,
) -> Result<Json<traderview_expense::section_7422::Section7422Result>, ApiError> {
    Ok(Json(traderview_expense::section_7422::check(&b)))
}

// ── §7426 third-party wrongful levy + surplus + substituted proceeds ─
// Mounted at /api/calc/section-7426. § 7426(a)(1) wrongful levy — any
// person OTHER than the assessed taxpayer with interest or lien on
// property wrongfully levied; civil action in district court. §
// 7426(a)(2) surplus proceeds — claimant interest JUNIOR to United
// States, entitled to excess sale proceeds. § 7426(a)(3) substituted
// sales proceeds — fund substituted for property under agreement. §
// 7426(c) SOL — 2 years (730 days) for wrongful levy post-12/22/2017
// TCJA Pub. L. 115-97 § 11071; pre-TCJA 9 months (274 days). § 7426(h)
// civil damages for unauthorized collection: lesser of $1,000,000
// (reckless/intentional) / $100,000 (negligence) OR actual damages +
// costs; mirrors § 7433 framework. § 7421(a) Anti-Injunction Act
// exception — § 7426(a) + (b)(1) statutorily excepted. Trader-relevant
// when IRS levies on third-party property (joint accounts + nominee
// accounts + community-property third-party interests + trader's co-
// owner / lender / lien-holder rights in seized rental property).
// Procedural-companion to § 7421 + § 7433 + § 7430 + § 6334. Pair with
// IRS Pub. 4528 (Making an Administrative Wrongful Levy Claim) + IRM
// 34.5.3 (Suits Brought Against the United States).

async fn section_7426_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7426::Section7426Input>,
) -> Result<Json<traderview_expense::section_7426::Section7426Result>, ApiError> {
    Ok(Json(traderview_expense::section_7426::check(&b)))
}

// ── §7429 review of jeopardy levy or assessment procedures ──────────
// Mounted at /api/calc/section-7429. Trader-relevant when IRS believes
// collection is in jeopardy (taxpayer planning to flee, conceal
// assets, dispose of assets to evade collection) and invokes jeopardy
// assessment under § 6861 (income/estate/gift tax) + § 6862 (other
// taxes) + immediate collection. § 7429(a) administrative review
// framework: (1) IRS provides written statement within 5 days of
// jeopardy assessment/levy; (2) taxpayer requests administrative
// review within 30 days; (3) IRS responds within 15 calendar days.
// § 7429(b) judicial review: filed within 90 days from earlier of (a)
// district director's notice of determination or (b) 16th day after
// administrative review request; DISTRICT COURT has EXCLUSIVE
// jurisdiction (no Tax Court alternative); court determines within
// 20 calendar days whether (1) assessment is REASONABLE and (2)
// amount is APPROPRIATE; extension up to 40 additional calendar
// days available for reasonable grounds (combined 60-day maximum).
// § 7421(a)(11) Anti-Injunction Act exception — § 7429(b) judicial
// review is one of 11 statutory exceptions to AIA bar. Procedural-
// companion to § 7421 + § 7426 + § 7433 + § 7430 + § 6321 + § 6323 +
// § 6325 + § 6334. 26 CFR § 301.7429-3; IRM 5.1.4; IRM 5.17.15; IRM
// 8.24.2.

async fn section_7429_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7429::Section7429Input>,
) -> Result<Json<traderview_expense::section_7429::Section7429Result>, ApiError> {
    Ok(Json(traderview_expense::section_7429::check(&b)))
}

// ── §7433 civil damages for unauthorized collection actions ─────────
// Mounted at /api/calc/section-7433. § 7433(a) cause of action — IRS
// officer or employee recklessly OR intentionally OR by reason of
// negligence disregards any IRC provision or regulation in connection
// with collection of federal tax. § 7433(b)(1) damages cap: lesser of
// $1,000,000 (reckless or intentional) / $100,000 (negligence) OR sum
// of actual direct economic damages + costs of action. § 7433(d)(1)
// exhaustion of administrative remedies required. § 7433(d)(2)
// mitigation reduction. § 7433(d)(3) 2-year SOL from accrual.
// § 7433A parallel regime for qualified tax collection contractors.
// Trader-relevant for wrongful levy beyond statutory limits + lien
// without notice + collection during § 6330 CDP appeal + § 6331/
// § 6332/§ 6334 violations. Companion to § 7421 (Anti-Injunction
// Act + § 7426 wrongful levy exception) + § 7430 (litigation costs)
// + § 7521 (interview procedure) + § 7525 (FATP privilege) + § 7811
// (TAOs).

async fn section_7433_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7433::Section7433Input>,
) -> Result<Json<traderview_expense::section_7433::Section7433Result>, ApiError> {
    Ok(Json(traderview_expense::section_7433::check(&b)))
}

// ── §162(f) fines and penalties nondeductibility ────────────────────
// Mounted at /api/calc/section-162f. § 162(f)(1) general rule (post-
// TCJA Dec 22, 2017) — no deduction for amounts paid to government in
// relation to violation or investigation. § 162(f)(2) restitution /
// remediation / compliance exception requires BOTH § 162(f)(2)(B)
// identification (court order or settlement agreement explicitly
// identifies amount + purpose) AND § 162(f)(2)(A) establishment
// (taxpayer establishes payment was for identified purpose). § 162(f)
// (3) routine investigation / court costs unaffected. § 162(f)(5) qui
// tam payments to relators outside § 162(f)(1) prohibition. § 6050X
// Form 1098-F reporting at $50K threshold. § 162(q) separate sexual-
// harassment-NDA restriction. TCJA § 13306 grandfathers pre-December
// 22, 2017 binding orders. Trader-relevant for FINRA / SEC / CFTC /
// exchange disciplinary fines.

// ── § 162(a) Trade or Business Expenses (FOUNDATIONAL) ───────────────
// Mounted at /api/calc/section-162a. Pure compute; FOUNDATIONAL
// deduction provision; Welch v. Helvering, 290 U.S. 111 (1933)
// four-element test (ordinary + necessary + carrying on trade or
// business + not capital expenditure); § 162(a)(1) reasonable
// compensation (subject to § 162(m) iter 446 $1M cap + § 280G
// iter 444 golden parachute); § 162(a)(2) traveling expenses
// (subject to § 274 iter 454 specific limits — meals 50% + entertainment
// disallowed + $25 gift cap + foreign convention reasonableness +
// luxury water travel cap); § 162(a)(3) rentals/other payments;
// § 162(c) illegal payment exceptions; § 162(e) lobbying; § 162(f)
// fines and penalties; INDOPCO 503 U.S. 79 (1992) § 263 long-term-
// benefit capitalization; Higgins 312 U.S. 212 (1941) investor
// vs trade or business; § 475(f) iter 458 trader mark-to-market
// election converts to trade or business; § 280E cannabis
// trafficking complete disallowance; § 183 hobby loss profit
// motive (3-of-5 year presumption + 9-factor Treas. Reg.
// § 1.183-2(b) test).

async fn section_162a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_162a::Section162aInput>,
) -> Result<Json<traderview_expense::section_162a::Section162aResult>, ApiError> {
    Ok(Json(traderview_expense::section_162a::check(&b)))
}

// ── §162(e) Lobbying and Political Expenditure Disallowance ────────
// Mounted at /api/calc/section-162e. § 162(e)(1) four categories of
// non-deductible expenditures: (A) influencing legislation; (B)
// political campaign participation; (C) grassroots lobbying; (D)
// direct communication with covered executive branch official.
// § 162(e)(4) covered executive branch official definition. TCJA
// 2017 § 13308 (Public Law 115-97, signed December 22, 2017)
// STRUCK former § 162(e)(2) local-legislation exception and former
// § 162(e)(7) Indian tribal government special rule; redesignated
// (3)-(6) and (8) as (2)-(5) and (6). De minimis $2,000 in-house
// lobbying exception under § 162(e)(5)(B) RETAINED — does NOT
// apply to professional lobbyists or organizational dues.
async fn section_162e_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_162e::Section162EInput>,
) -> Result<Json<traderview_expense::section_162e::Section162EResult>, ApiError> {
    Ok(Json(traderview_expense::section_162e::check(&b)))
}

async fn section_162f_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_162f::Section162fInput>,
) -> Result<Json<traderview_expense::section_162f::Section162fResult>, ApiError> {
    if b.payment_amount_cents < 0 {
        return Err(ApiError::BadRequest(
            "payment_amount_cents must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_162f::compute(&b)))
}

// ── § 162(l) Self-Employed Health Insurance Above-The-Line Deduction ─
// Mounted at /api/calc/section-162l (iter 508). Pure compute. § 162(l)
// allows sole proprietors, partners, S corp >2% shareholders, and
// single-member LLC owners to deduct medical/dental/long-term care
// premiums above the line on Schedule 1 line 17 — bypasses § 213 7.5%
// AGI floor. Six business structures: SoleProprietorOrSingleMemberLlc
// (Sch C net profit minus 1/2 SE tax minus SE retirement = earned
// income limit), PartnershipPartner (K-1 box 14 code A minus 1/2 SE
// tax minus SE retirement), SCorporationShareholderOver2Pct (W-2 box 1
// wages = limit; premiums must be in W-2 box 1 per Notice 2008-1 +
// excluded from FICA under § 3121(a)(2)(B)), SCorporationShareholder-
// TwoPctOrLess (ordinary § 106 employer exclusion not § 162(l)),
// CCorporationShareholder (not eligible), W2EmployeeOnly (not eligible).
// Three premium types: MedicalDentalHealth, QualifiedLongTermCare (subject
// to § 213(d)(10) age-based dollar caps via § 7702B(b)), MedicarePart-
// ABCD (deductible per CCA 201228037 July 13 2012). § 162(l)(2)(B)
// double-coverage prohibition disallows deduction month-by-month when
// taxpayer or spouse eligible for subsidized employer-sponsored plan.
// § 162(l)(2)(B) plan-establishment requirement: plan must be in name
// of business or owner per Notice 2008-1 + Rev. Proc. 79-46. Form 7206
// dedicated computation form effective tax years beginning after Dec
// 31 2022. Six-mode severity ladder including DeductionLimitedByEarned-
// Income + DoubleCoverageDisallowedForMonth + PlanNotEstablishedThrough-
// Business + DeductionAllowedFullPremium. Coordinates with § 213
// (itemized medical), § 106 (employer health exclusion), § 7702B (LTC),
// § 401(c)(2) (earned income), § 1372 (S corp fringe), § 4980H (ACA
// employer mandate), § 36B (Marketplace premium tax credit).

async fn section_162l_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_162l::Section162lInput>,
) -> Result<Json<traderview_expense::section_162l::Section162lResult>, ApiError> {
    Ok(Json(traderview_expense::section_162l::check(&b)))
}

// ── § 162(m) $1M public-company executive comp deduction limit ───────
// Mounted at /api/calc/section-162m. Pure compute; § 162(m)(1)
// $1,000,000 annual cap on covered-employee remuneration deductible
// to publicly held corporation; § 162(m)(2) PUBLICLY HELD = SEC § 12
// registration OR § 15(d) reporting (TCJA-expanded to include
// foreign private issuers); § 162(m)(3) COVERED EMPLOYEE = CEO +
// CFO + top 3 most highly compensated officers + ONCE-COVERED-
// ALWAYS-COVERED (post-2016 status survives departure + death);
// post-2026 ARPA FIVE (Pub. L. 117-2 § 9708) adds next 5 most
// highly compensated (NOT necessarily officers; retested
// annually); TCJA 2017 § 13601 eliminated performance-based and
// commission exceptions; pre-TCJA written binding contract on
// November 2, 2017 transition rule preserves former § 162(m)(4)(C)
// performance-based exception; § 162(m) and § 280G can apply
// simultaneously on same compensation.

async fn section_162m_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_162m::Section162mInput>,
) -> Result<Json<traderview_expense::section_162m::Section162mResult>, ApiError> {
    Ok(Json(traderview_expense::section_162m::check(&b)))
}

// ── §6404 abatement of interest + tax + penalties ───────────────────
// Mounted at /api/calc/section-6404. § 6404(a) general abatement
// authority for excessive / post-SOL / erroneously assessed; §
// 6404(b) no statutory taxpayer RIGHT for interest / additions to
// tax / additional amounts / assessable penalties (relies on
// discretionary IRS authority OR § 6404(e)/(f)/(g)); § 6404(c)
// small tax balance ($5 or less); § 6404(e)(1) UNREASONABLE ERROR
// OR DELAY by IRS employee — error/delay after written IRS contact
// + taxpayer did not contribute + act was MINISTERIAL (procedural/
// mechanical, no judgment) OR MANAGERIAL (administrative, loss of
// records or personnel discretion); legal-judgment delays NOT
// abatable; § 6404(e)(2) erroneous refund check $50K cap; § 6404(f)
// erroneous written advice three-element test (written request +
// accurate facts + reasonable reliance); § 6404(g) 36-month
// interest suspension for individuals when IRS fails to notify
// within 1,095 days of timely return filing (21-day grace); §
// 6404(h) Tax Court review of § 6404(e) refusals for abuse of
// discretion within 180 days; Treas. Reg. § 301.6404-2; IRM 20.2.7
// Abatement and Suspension of Underpayment Interest; Form 843 +
// § 6511 lookback (3 years from return OR 2 years from payment).

async fn section_6404_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6404::Section6404Input>,
) -> Result<Json<traderview_expense::section_6404::Section6404Result>, ApiError> {
    Ok(Json(traderview_expense::section_6404::check(&b)))
}

// ── §6425 corporate quick-refund procedure Form 4466 ──────────────
// Mounted at /api/calc/section-6425. § 6425 is the procedural
// companion to § 6655 (corporate estimated tax underpayment penalty
// — built iter 676) and § 6621 (interest rate determination — built
// iter 674). Allows C corporation that has overpaid quarterly
// estimated taxes to obtain ACCELERATED REFUND (within 45 days)
// rather than waiting for standard tax-return refund cycle. § 6425(a)
// right to adjustment via Form 4466 (Corporation Application for
// Quick Refund of Overpayment of Estimated Tax). § 6425(b) filing
// deadline: 15th day of 4th month after close of taxable year AND
// before tax return filed (whichever earlier); April 15 for calendar-
// year corps. § 6425(b)(2) computation: adjustment = estimated tax
// PAID − estimated income tax LIABILITY. § 6425(b)(3) MINIMUM
// THRESHOLD: adjustment must meet BOTH (a) 10 PERCENT of estimated
// income tax liability AND (b) $500 (conjunctive double-test).
// § 6425(c) IRS PROCESSING WINDOW: 45 DAYS to examine, determine
// adjustment, credit, refund. Not a refund claim under § 6511 —
// distinct procedural track. § 6655(h) EXCESSIVE ADJUSTMENT INTEREST
// CHARGE: if quick refund proves excessive, interest at § 6621
// underpayment rate accrues from refund date through 15th day of 4th
// month following year-end. Applies only to C corporations; S corps,
// partnerships, individuals, trusts, estates not eligible. 14-mode
// severity ladder × 2 corporation types × 4 application statuses × 4
// compliance aspects × 3 excessive-adjustment statuses × variable
// estimated tax / liability / adjustment / IRS processing days / FSTR
// inputs. Sibling cluster: section_6621 (built iter 674; underpayment
// rate cited by § 6655(h)), section_6601 (general underpayment
// interest), section_6611 (overpayment interest), section_6622 (daily
// compounding), section_6655 (built iter 676; corporate estimated
// tax underpayment penalty + § 6655(h) excessive-adjustment interest
// cross-reference), section_6654 (individual estimated tax — parallel
// individual provision; no individual analog to § 6425 quick refund),
// section_6511 (refund-claim limitations period — DISTINCT from
// § 6425), section_6662 (accuracy-related penalty), section_7206
// (fraud and false statements; willful fraudulent Form 4466
// exposure).

async fn section_6425_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6425::Section6425Input>,
) -> Result<Json<traderview_expense::section_6425::Section6425Result>, ApiError> {
    Ok(Json(traderview_expense::section_6425::compute(&b)))
}

// ── §6411 tentative carryback adjustment Form 1139 / Form 1045 ────
// Mounted at /api/calc/section-6411. § 6411 is the companion to
// § 6425 (corporate quick refund for estimated tax overpayments —
// built iter 684) and follows a similar quick-refund processing
// model with a 90-day IRS examination window (vs § 6425's 45-day
// window). Form 1139 (Corporation Application for Tentative Refund)
// is the operative form for corporations; Form 1045 (Application for
// Tentative Refund) is the analog for individuals, estates, and
// trusts. § 6411(a) right to tentative adjustment for NOL carryback
// (§ 172), net capital loss carryback (§ 1212), or unused business
// credit carryback (§ 39). Filing deadline: 12 MONTHS after end of
// taxable year of loss / unused credit; income tax return for loss /
// credit year must be filed no later than Form 1139 / Form 1045
// filing date. § 6411(b) IRS examination window: 90 DAYS to make
// limited examination for omissions and computational errors,
// determine adjustment, credit / refund. § 6411(c) consolidated
// return special rules under Treas. Reg. § 1.1502-78. § 6411(d) claim
// of right tentative refund under § 1341(b)(1) added by Public Law
// 95-628 of 1978; available within 12 months from last day of
// taxable year of repayment. NOT a refund claim under § 6511 —
// distinct procedural track. § 6213(b)(3) subsequent disallowance:
// excessive tentative refund treated as deficiency assessable
// WITHOUT normal § 6213(a) Tax Court petition right; interest at
// § 6601 underpayment rate accrues from original refund date.
// CARES Act 2020 (Public Law 116-136, March 27, 2020) temporarily
// allowed corporations to carry back NOLs arising in 2018, 2019, or
// 2020 to each of 5 preceding taxable years; sunset for losses
// arising in 2021+. 16-mode severity ladder × 5 taxpayer types × 3
// application form statuses × 5 carryback categories × 3 income tax
// return statuses × 6 compliance aspects × variable IRS processing
// days / consolidated return / subsequent disallowance flags.
// Sibling cluster: section_6425 (built iter 684; corporate estimated
// tax quick refund — 45-day vs § 6411's 90-day window), section_6601
// (general underpayment interest), section_6611 (overpayment
// interest), section_6621 (built iter 674; underpayment rate),
// section_6655 (built iter 676; corporate estimated tax penalty),
// section_6511 (refund claim limitations — DISTINCT from § 6411),
// section_6213b3 (special rule for tentative carryback assessments),
// section_172 (NOL), section_1212 (net capital loss), section_39
// (unused business credit), section_1341 (claim of right),
// section_6662 (accuracy-related penalty), section_7206 (fraud and
// false statements; willful fraudulent Form 1139 / 1045 exposure).

async fn section_6411_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6411::Section6411Input>,
) -> Result<Json<traderview_expense::section_6411::Section6411Result>, ApiError> {
    Ok(Json(traderview_expense::section_6411::compute(&b)))
}

async fn section_6418_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6418::Section6418Input>,
) -> Result<Json<traderview_expense::section_6418::Section6418Result>, ApiError> {
    Ok(Json(traderview_expense::section_6418::compute(&b)))
}

async fn section_6417_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6417::Section6417Input>,
) -> Result<Json<traderview_expense::section_6417::Section6417Result>, ApiError> {
    Ok(Json(traderview_expense::section_6417::compute(&b)))
}

// ── §6531 periods of limitation on criminal prosecutions ────────────
// Mounted at /api/calc/section-6531. Cross-cutting reference statute
// that determines criminal SOL for ALL Title 26 criminal tax
// prosecutions. General rule: 3 YEARS from commission of offense.
// 6-YEAR exception for enumerated offenses: § 7201 evasion + § 7202
// trust fund failure + § 7203 failure to FILE/PAY (NOT failure to
// keep records or supply info) + § 7206(1) filing false return +
// § 7206(2) aiding false return + § 7207 fraudulent returns/
// statements + § 7212(b) rescue of seized property + § 7214 unlawful
// acts of revenue officers + 18 U.S.C. § 371 Klein conspiracy.
// 3-year SOL: § 7203 records/info + § 7205 false withholding
// exemption + § 7206(3)/(4)/(5) + § 7212(a) general obstruction +
// all other Title 26 offenses. § 6531(4) carveout: 6-year for
// failure to file does NOT apply to partnership Form 1065 + exempt
// org Form 990 + S-corp Form 1120-S returns under Part III
// Subchapter A Chapter 61 (3-year SOL applies). Final-paragraph
// tolling: defendant outside US or fugitive tolls SOL until 6
// months after return/surrender. Toussie v. United States, 397
// U.S. 112 (1970) continuing-offense doctrine narrowed but
// affirmative-act-doctrine cases survive — SOL runs from LAST
// affirmative act for § 7201. § 6531 SOL is JURISDICTIONAL. DOJ
// Criminal Tax Manual § 7.00 + IRM 25.6.2.1. Pairs with section_
// 7201 + section_7202 + section_7203 + section_7206 + section_7212.

async fn section_6531_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6531::Section6531Input>,
) -> Result<Json<traderview_expense::section_6531::Section6531Result>, ApiError> {
    Ok(Json(traderview_expense::section_6531::check(&b)))
}

// ── §6532 periods of limitation on refund + wrongful-levy suits ─────
// Mounted at /api/calc/section-6532. § 6532(a) taxpayer refund suit
// under § 7422 — 6-month floor + 2-year ceiling from notice of
// disallowance mailed certified/registered; § 6532(a)(2) written
// extension; § 6532(a)(3) reconsideration does NOT extend;
// § 6532(a)(4) waiver of certified-mail requirement runs from waiver
// filing. § 6532(b) US erroneous refund suit under § 7405 — 2 years
// standard; 5 years if refund induced by FRAUD OR MISREPRESENTATION
// OF A MATERIAL FACT. § 6532(c)(1) third-party wrongful levy suit
// under § 7426 — 2 YEARS from date of levy (TCJA 2017 § 11071
// EXTENDED prior 9-month period to 2 years, effective for levies
// made after December 22, 2017); § 6532(c)(2) § 6343(b)
// administrative-claim extension to SOONER of (A) 12 months from
// claim filing OR (B) 6 months from IRS disallowance. Trader-
// critical for every refund-suit scenario (NOL § 172/§ 475(f)
// carryback, § 1256 60/40 mark-to-market amended return, § 1091
// wash-sale recomputation, § 988 currency loss restatement) and
// every third-party broker-account wrongful-levy scenario. Sibling
// cluster: § 7422 + § 7426 + § 7405 + § 6511 + § 6343(b).

async fn section_6532_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6532::Section6532Input>,
) -> Result<Json<traderview_expense::section_6532::Section6532Result>, ApiError> {
    if b.written_extension_days_added > 36_500 {
        return Err(ApiError::BadRequest(
            "written_extension_days_added out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6532::check(&b)))
}

// ── §6501 limitations on assessment + collection (ASED) ─────────────
// Mounted at /api/calc/section-6501. § 6501(a) 3-year default ASED
// from filing date; § 6501(b)(1) early-filed return deemed filed on
// statutory due date; § 6501(c)(1) UNLIMITED for false/fraudulent
// return with intent to evade tax (clear-and-convincing burden);
// § 6501(c)(2) UNLIMITED for willful attempt to evade; § 6501(c)(3)
// UNLIMITED for no return filed (3-year clock starts only upon
// filing); § 6501(c)(4) Form 872 consent extension + IRM 25.6.22
// three-rights disclosure requirement; § 6501(e)(1)(A)(i) 6-year for
// >25% gross-income omission; § 6501(e)(1)(B) 6-year for basis
// overstatement (post-2015 Surface Transportation Act amendment
// overruling Home Concrete & Supply v. United States, 132 S. Ct.
// 1836 (2012)). Trader-critical defensive shield against IRS audit
// reach-back on wash-sale disallowances, § 1256 mark-to-market,
// § 988 currency, § 1202 QSBS holding-period determinations.

async fn section_6501_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6501::Section6501Input>,
) -> Result<Json<traderview_expense::section_6501::Section6501Result>, ApiError> {
    Ok(Json(traderview_expense::section_6501::check(&b)))
}

// ── §6502 collection after assessment (CSED) ────────────────────────
// Mounted at /api/calc/section-6502. § 6502(a)(1) 10-year base CSED
// from date of assessment — after CSED, IRS BARRED from collecting
// via levy (§ 6331), lien (§ 6321), or court proceeding. Six
// independent suspension triggers each extend CSED: § 6502(a)(2)
// installment agreement + 90 days post-expiration; § 6331(k)(1) OIC
// suspended from submission through accept/reject/withdraw/return
// + ADDITIONAL 30 days if rejected; § 6330(e)(1) CDP hearing
// request suspends through conclusion + 90-day floor if < 90 days
// remain on CSED; § 6503(h) bankruptcy automatic stay + 6 months
// after stay terminates; § 7508(a) military combat zone + 180 days;
// § 6503(c) taxpayer continuously absent from US 6+ months +
// absence period + return + 6 months. Overlapping suspensions run
// CONCURRENTLY not cumulatively per IRM 5.1.19.3.4. Natural sibling
// to section_6501 (ASED — 3/6/unlimited assessment statute) and
// section_7811 (TAOs).

async fn section_6502_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6502::Section6502Input>,
) -> Result<Json<traderview_expense::section_6502::Section6502Result>, ApiError> {
    Ok(Json(traderview_expense::section_6502::check(&b)))
}

// ── §6511 limitations on credit or refund ───────────────────────────
// Mounted at /api/calc/section-6511. §6511(a) general 3-year-from-
// filing or 2-year-from-payment whichever later; §6511(b)(2) 3-year
// or 2-year lookback rule on refund amount; §6511(d)(1) 7-year bad
// debt / worthless security (§§ 166, 832(c), 165(g)); §6511(d)(2)(A)
// NOL/capital loss carryback period ends 3 years after due date of
// LOSS-year return; §6511(d)(3)(A) 10-year foreign tax credit;
// §6511(h) financial-disability suspension flagged for caller. Rev.
// Rul. 2020-8 (suspending Rev. Rul. 71-533) flagged for FTC carryback
// from NOL carryback open question. Trader-critical for Form 1040-X
// amended returns claiming missed § 475(f) MTM elections, missed
// § 901 FTCs, worthless-security losses, or NOL carrybacks.

async fn section_6511_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6511::Section6511Input>,
) -> Result<Json<traderview_expense::section_6511::Section6511Result>, ApiError> {
    if b.return_tax_year < 1900 || b.return_tax_year > 2200 {
        return Err(ApiError::BadRequest("return_tax_year out of range".into()));
    }
    if let Some(y) = b.carryback_loss_year {
        if !(1900..=2200).contains(&y) {
            return Err(ApiError::BadRequest(
                "carryback_loss_year out of range".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_6511::compute(&b)))
}

// ── §6601 interest on underpayment + §6621 rate + §6622 compounding ─
// Mounted at /api/calc/section-6601. § 6601(a) interest from last
// date prescribed for payment under § 6601(b)(1) (extension to file
// does not extend time to pay) until paid. § 6622(a) daily
// compounding. § 6621(a)(2) underpayment rate = federal short-term
// rate + 3%; § 6621(c) large corporate underpayment rate = federal
// short-term rate + 5% (after applicable date — generally 30 days
// after IRS notice). Quarterly rates published via Revenue Ruling.
// 2026 Q1 (Rev. Rul. 2025-22): 7% underpayment / 9% large corporate.
// 2026 Q2 (Rev. Rul. 2026-5): 6% underpayment / 8% large corporate.
// Trader-relevant when amended return / audit produces additional
// tax — interest runs from ORIGINAL April 15 due date regardless of
// extension to file. § 6601 interest is non-deductible personal
// interest under § 163(h) for individuals but deductible business
// interest under § 163(a) for sole-proprietor traders.

async fn section_6601_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6601::Section6601Input>,
) -> Result<Json<traderview_expense::section_6601::Section6601Result>, ApiError> {
    if b.rate_quarter == 0 || b.rate_quarter > 4 {
        return Err(ApiError::BadRequest(
            "rate_quarter must be 1, 2, 3, or 4".into(),
        ));
    }
    if b.days_outstanding > 1_000_000 {
        return Err(ApiError::BadRequest(
            "days_outstanding looks invalid (>1000000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6601::compute(&b)))
}

// ── §6611 interest on overpayments (companion to §6601) ─────────────
// Mounted at /api/calc/section-6611. § 6611(a) general overpayment
// interest at § 6621 rate. § 6611(b)(2) refund interest from
// overpayment date to 30 days before refund check (§ 6611(b)(1) credit
// path). § 6611(e)(1) 45-day SAFE HARBOR — refund within 45 days of
// return-due date triggers ZERO interest. § 6611(e)(2) parallel 45-day
// safe harbor for refund claims (Form 1040-X). § 6611(e)(3) IRS-
// initiated adjustment SUBTRACTS 45 days from interest period. § 6621(a)
// (1) overpayment rates: individual FST + 3%, corporate FST + 2%,
// corporate > $10K GATT rate FST + 0.5% (Pub. L. 103-465 § 713). § 6622
// (a) daily compounding. 2026 Q1 (Rev. Rul. 2025-22): 7% individual /
// 6% corporate / 4.5% GATT. 2026 Q2 (Rev. Rul. 2026-5): 6% / 5% / 3.5%.
// Interest received treated as gross income under § 61(a)(4).

async fn section_6611_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6611::Section6611Input>,
) -> Result<Json<traderview_expense::section_6611::Section6611Result>, ApiError> {
    if b.rate_quarter == 0 || b.rate_quarter > 4 {
        return Err(ApiError::BadRequest(
            "rate_quarter must be 1, 2, 3, or 4".into(),
        ));
    }
    if b.days_from_overpayment_to_refund > 1_000_000 {
        return Err(ApiError::BadRequest(
            "days_from_overpayment_to_refund looks invalid (>1000000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6611::compute(&b)))
}

// ── §6621 determination of interest rate / federal short-term rate ──
// Mounted at /api/calc/section-6621. § 6621 is the master rate-
// setting provision feeding every § 6601 underpayment-interest and
// every § 6611 overpayment-interest computation, driven by the
// § 6621(b) federal short-term rate (FSTR) determined quarterly
// under § 1274(d) AFR methodology, rounded to nearest full percent
// (or next highest if multiple of 0.5 percent). § 6621(a)(1)
// overpayment rate = FSTR + 3 pp (individuals); FSTR + 2 pp
// (corporations); FSTR + 0.5 pp for the portion of a corporate
// overpayment exceeding $10,000 in any taxable period. § 6621(a)(2)
// underpayment rate = FSTR + 3 pp (all taxpayers). § 6621(c) large
// corporate underpayment rate = FSTR + 5 pp (substituting "5
// percentage points" for "3 percentage points" in (a)(2)); applies
// only to C corporations whose underpayment EXCEEDS $100,000 in a
// taxable period (strict greater-than statutory boundary); rate
// applies only to interest accruing AFTER the applicable date =
// 30 DAYS following the earlier of proposed deficiency notice or
// formal notice of deficiency. § 6621(d) net zero rate for
// overlapping periods of equivalent overpayments and underpayments
// by the SAME taxpayer of Title 26 tax. § 6622 cross-reference:
// interest under § 6601 and § 6611 is COMPOUNDED DAILY; the
// module's linear-rate computations are simplifications of the
// actual daily-compounded interest. Eight-mode severity ladder ×
// three taxpayer types (individual / C corp / other entity) × three
// amount statuses (overpayment / underpayment / none) × three large-
// corporate applicable-date statuses × two net-zero overlapping
// statuses × variable FSTR / amount / years input. Sibling cluster:
// section_6601 (interest on underpayments — primary consumer of the
// § 6621 rate), section_6611 (interest on overpayments — primary
// consumer of § 6621(a)(1) rate), section_6651 (failure to file /
// failure to pay penalty — runs alongside § 6601 interest accrual),
// section_6654 (individual estimated tax underpayment penalty —
// uses § 6621 rate), section_1274 (AFR determination — defines
// federal short-term rate methodology), section_1258 (conversion
// transactions — uses § 6621(b) federal short-term rate compounded
// daily for indefinite-term applicable rate), section_1260
// (constructive ownership transactions — uses § 6601 + § 6621 for
// interest charge on ordinary recharacterization).

async fn section_6621_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6621::Section6621Input>,
) -> Result<Json<traderview_expense::section_6621::Section6621Result>, ApiError> {
    Ok(Json(traderview_expense::section_6621::compute(&b)))
}

// Mounted at /api/calc/section-6651. §6651(a)(1) FTF 5%/month / 25%
// max; §6651(a)(2) FTP 0.5%/month / 25% max; §6651(c)(1) FTF reduced
// by FTP for overlap months (net 4.5%/month FTF + 0.5%/month FTP =
// 5%/month combined); §6651(f) fraud 15%/month / 75% max; §6651(g)
// minimum-penalty floor for returns > 60 days late (lesser of
// inflation-adjusted amount Rev. Proc. 2025 = $510 or 100% tax);
// §6651(h) installment-rate 0.25%/month when timely-filed-with-
// extension + §6159 agreement; reasonable-cause defense (NOT for fraud).

async fn section_6651_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6651::Section6651Input>,
) -> Result<Json<traderview_expense::section_6651::Section6651Result>, ApiError> {
    if b.tax_required_dollars < 0 || b.minimum_penalty_inflation_adjusted_dollars < 0 {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6651::compute(&b)))
}

// ── §6654 individual estimated-tax underpayment penalty ─────────────
// Mounted at /api/calc/section-6654. §6654(d)(1)(B)(i) 90%-current-year
// safe harbor; §6654(d)(1)(B)(ii) 100%-prior-year safe harbor;
// §6654(d)(1)(C) 110% high-AGI uplift when prior-year AGI > $150,000
// ($75,000 MFS); §6654(e)(1) $1,000 de minimis exception; required
// installment = (lesser of the two safe-harbor amounts) ÷ 4; underpayment
// per quarter accrues at the § 6621(a)(2) federal-short-term-rate + 3
// percentage points (2026 Q1 = 7%, Q2 = 6%). Out of scope: §6654(d)(2)
// annualized-income exception, §6654(i) farmer/fisherman two-thirds
// rule, §6654(e)(3) retired/disabled waiver.

async fn section_6654_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6654::Section6654Input>,
) -> Result<Json<traderview_expense::section_6654::Section6654Result>, ApiError> {
    for p in &b.quarterly_payments_cents {
        if *p < 0 {
            return Err(ApiError::BadRequest(
                "quarterly_payments_cents must be non-negative".into(),
            ));
        }
    }
    Ok(Json(traderview_expense::section_6654::compute(&b)))
}

// ── §6655 corporate estimated tax underpayment penalty ─────────────
// Mounted at /api/calc/section-6655. § 6655 is the corporate
// parallel to § 6654 (individual estimated tax) and uses the § 6621
// underpayment rate determined by § 6621(b) federal short-term rate
// + 3 pp (or + 5 pp for large corporate underpayment > $100,000
// under § 6621(c)) — see section_6621 (built iter 674). § 6655(a)
// imposes the addition to tax computed by applying the § 6621
// underpayment rate to the underpayment for the underpayment period
// (from installment due date until earlier of April 15 of following
// year or payment date). § 6655(c) four required installments due
// April 15, June 15, September 15, and December 15. § 6655(d)(1)(A)
// each installment = 25 PERCENT of required annual payment; required
// annual payment = LESSER of (i) 100 % current-year tax or (ii)
// 100 % preceding-year tax (subject to availability). § 6655(d)(2)
// LARGE CORPORATION (taxable income ≥ $1,000,000 in any of 3
// preceding taxable years under § 6655(g)(2)) may NOT use prior-year
// safe harbor after the first installment; large corp must pay
// 100 % current-year tax through installments 2-4 but may use
// prior-year for installment 1. § 6655(e) annualized income
// installment method (using 3, 3, 6, 9-month annualization) and
// adjusted seasonal installment method (70 %+ income in same 6
// months in each of 3 preceding years) provide alternative lower-
// installment safe harbors. § 6655(f) small-underpayment exception:
// no penalty if total tax shown on return is LESS THAN $500 (strict
// less-than statutory boundary). § 6655(g)(1) "tax" includes regular
// income tax under § 11, AMT under § 55, and BEAT under § 59A,
// reduced by applicable credits. § 6655(h) excessive § 6425
// quick-refund interest at § 6621 rate. Nine-mode severity ladder
// × 2 corporation types × 4 installment quarters × 4 safe-harbor
// methods × variable current-year tax / preceding-year tax / payment /
// FSTR / underpayment-period inputs. Sibling cluster: section_6621
// (underpayment rate; primary input to § 6655 penalty computation
// — built iter 674), section_6601 (general underpayment interest),
// section_6611 (overpayment interest), section_6622 (daily
// compounding cross-reference), section_6651 (failure-to-file /
// failure-to-pay penalty — independent civil penalty layer),
// section_6654 (individual estimated tax underpayment — parallel
// individual provision; § 6655 is corporate counterpart).

async fn section_6655_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6655::Section6655Input>,
) -> Result<Json<traderview_expense::section_6655::Section6655Result>, ApiError> {
    Ok(Json(traderview_expense::section_6655::compute(&b)))
}

// ── §6662 accuracy-related penalty ──────────────────────────────────
// Mounted at /api/calc/section-6662. §6662(a) 20% baseline on
// portion of underpayment attributable to misconduct; §6662(h) 40%
// for gross valuation misstatement (claimed ≥ 200% correct);
// §6662(b) 8 categories (negligence, substantial understatement,
// valuation misstatement, etc.); §6662(d) substantial-understatement
// threshold (greater of 10% of correct tax or $5k individual /
// $10k corporate, capped at $10M); §6664(c) reasonable-cause-and-
// good-faith defense (UNAVAILABLE for §6662(b)(6) economic substance
// + §6662(b)(7) undisclosed foreign asset); no stacking.

async fn section_6662_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6662::Section6662Input>,
) -> Result<Json<traderview_expense::section_6662::Section6662Result>, ApiError> {
    if b.underpayment_dollars < 0
        || b.correct_tax_required_dollars < 0
        || b.claimed_value_dollars < 0
        || b.correct_value_for_valuation_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6662::compute(&b)))
}

// ── §448 small business gross receipts test + cascade exemptions ────
// Mounted at /api/calc/section-448. §448(a) mandatory accrual for
// C-corps / partnerships with C-corp partner; §448(b)(3) small
// business exception when §448(c) 3-year average gross receipts ≤
// inflation-indexed threshold ($25M TCJA base; $30M 2024 / $31M 2025
// / $32M 2026); §448(a)(3) tax shelter disqualification; §448(c)(2)
// §52(a)/(b) aggregation. Cascade exemptions: §263A UNICAP, §471
// inventory, §163(j) business interest, §460 long-term contracts.

// ── § 446 general rule for methods of accounting ─────────────────
// Mounted at /api/calc/section-446. Foundational accounting-method
// provision. § 446(a) book-tax conformity default. § 446(b)
// Secretary's authority to override when method does not clearly
// reflect income. § 446(c) permissible methods: (1) cash; (2)
// accrual; (3) other permitted (§ 475 MTM, § 453 installment, § 460
// long-term contracts); (4) hybrid combinations. § 446(d) different
// methods for different trades. § 446(e) Form 3115 consent required
// for method changes. § 446(f) cross-reference to § 460 long-term
// contracts. Treas. Reg. § 1.446-1(a)(2) consistency requirement
// (year-to-year treatment of gross profit and deductions). Sibling
// cluster: § 461(h) economic performance (accrual deduction
// timing), § 471 inventory requirement, § 475 trader/dealer MTM,
// § 453 installment method, § 460 long-term contracts, § 448
// limitation on cash method, § 481 accounting-method-change
// adjustments.

async fn section_446_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_446::Section446Input>,
) -> Result<Json<traderview_expense::section_446::Section446Output>, ApiError> {
    Ok(Json(traderview_expense::section_446::check(&b)))
}

async fn section_448_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_448::Section448Input>,
) -> Result<Json<traderview_expense::section_448::Section448Result>, ApiError> {
    if b.gross_receipts_year_minus_1_dollars < 0
        || b.gross_receipts_year_minus_2_dollars < 0
        || b.gross_receipts_year_minus_3_dollars < 0
        || b.aggregated_gross_receipts_year_minus_1_dollars < 0
        || b.aggregated_gross_receipts_year_minus_2_dollars < 0
        || b.aggregated_gross_receipts_year_minus_3_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all gross receipts inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_448::compute(&b)))
}

// ── §444 fiscal year election ───────────────────────────────────────
// Mounted at /api/calc/section-444. §444(a) election availability for
// partnerships, S-corps, and PSCs; §444(b)(2) 3-month deferral cap
// (only Sept 30 / Oct 31 / Nov 30 fiscal year ends qualify when
// required year is calendar); §7519 required payment for partnerships
// and S-corps (Form 8752, due May 15); §280H deduction limitations
// for PSCs.

async fn section_444_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_444::Section444Input>,
) -> Result<Json<traderview_expense::section_444::Section444Result>, ApiError> {
    if b.net_income_for_election_year_dollars < 0 {
        return Err(ApiError::BadRequest(
            "net_income_for_election_year_dollars must be >= 0".into(),
        ));
    }
    if b.required_tax_year_end_month == 0 || b.required_tax_year_end_month > 12 {
        return Err(ApiError::BadRequest(
            "required_tax_year_end_month must be 1..=12".into(),
        ));
    }
    if b.proposed_fiscal_year_end_month == 0 || b.proposed_fiscal_year_end_month > 12 {
        return Err(ApiError::BadRequest(
            "proposed_fiscal_year_end_month must be 1..=12".into(),
        ));
    }
    Ok(Json(traderview_expense::section_444::compute(&b)))
}

// ── §3406 backup withholding ────────────────────────────────────────
// Mounted at /api/calc/section-3406. §3406(a)(1)(A) TIN-not-furnished
// trigger; §3406(a)(1)(B) IRS-notified-incorrect-TIN trigger (BWH-B
// program, CP 2100 / CP 2100A); §3406(a)(1)(C) notified-payee
// underreporting trigger (BWH-C, interest/dividend only);
// §3406(a)(1)(D) payee-certification-failure trigger; §3406(b)(1)(A)
// 24% rate (4th lowest §1(c) rate, post-TCJA).

async fn section_3406_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_3406::Section3406Input>,
) -> Result<Json<traderview_expense::section_3406::Section3406Result>, ApiError> {
    if b.payment_amount_dollars < 0 {
        return Err(ApiError::BadRequest(
            "payment_amount_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_3406::compute(&b)))
}

// ── §305 stock dividend distribution classification ─────────────────
// Mounted at /api/calc/section-305. §305(a) general rule excludes
// stock-on-stock distributions from gross income; §305(b) 5
// taxable exceptions (in lieu of money / disproportionate / common-
// and-preferred / on preferred stock / convertible preferred w/o
// safe harbor); §305(c) deemed distributions from capital-structure
// events; §307(a) basis allocation between old and new shares when
// §305(a) applies; §301 distribution treatment when taxable
// (dividend up to E&P + basis recovery + capital gain).

async fn section_305_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_305::Section305Input>,
) -> Result<Json<traderview_expense::section_305::Section305Result>, ApiError> {
    Ok(Json(traderview_expense::section_305::compute(&b)))
}

// ── § 302 Distributions in Redemption of Stock ──────────────────────
// Mounted at /api/calc/section-302 (iter 556). Pure compute. § 302
// determines whether a corporation's redemption of its own stock is
// treated as a § 1001 SALE OR EXCHANGE (capital gain or loss) or as a
// § 301 DISTRIBUTION (ordinary dividend up to E&P + basis recovery
// + capital gain on excess). Four § 302(b) tests:
//
// § 302(b)(1) not essentially equivalent to a dividend (Davis NEED-test):
// facts-and-circumstances meaningful-reduction analysis.
//
// § 302(b)(2) substantially disproportionate (mechanical 50/80): post-
// redemption voting < 50% AND post-redemption interest < 80% of
// pre-redemption interest (both voting and combined voting/non-voting).
//
// § 302(b)(3) complete termination: redemption terminates ALL
// shareholder interest. § 302(c)(2) family-attribution WAIVER available
// if (A) no interest other than creditor immediately after, (B) no
// reacquisition within 10 years (other than bequest/inheritance), and
// (C) signed IRS notification agreement filed.
//
// § 302(b)(4) partial liquidation (non-corporate shareholder): § 302(e)
// requires 5+ years of active trade or business + distribution of
// discontinued segment proceeds.
//
// § 302(c)(1): § 318(a) attribution rules apply to determine ownership.
// § 302(d): redemption defaults to § 301 distribution if all § 302(b)
// tests fail.
//
// Six-mode severity ladder: NotApplicable,
// Section302BTestSatisfiedSaleOrExchangeTreatment,
// Section302B2SubstantiallyDisproportionate50_80TestFailed,
// Section302B3CompleteTerminationWithAttributionWaiver,
// Section302B3CompleteTerminationAttributionDefeatsFailedWaiver,
// Section302DDefaultDistributionTreatmentSection301.
//
// Coordinates with § 318 (iter 552 — constructive ownership engine),
// § 304 (iter 554 — related-corp redemption recharacterization),
// § 311 (iter 550 — corp-level recognition on distribution), § 312
// E&P computation, § 1001 + § 1222 capital-gain treatment, § 1(h)(11)
// qualified-dividend rate, § 331 corporate-liquidation regime, § 332
// parent-sub liquidation.

async fn section_302_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_302::Section302RedemptionTestInput>,
) -> Result<Json<traderview_expense::section_302::Section302RedemptionTestOutput>, ApiError> {
    Ok(Json(traderview_expense::section_302::check(&b)))
}

// ── § 304 Redemption Through Use of Related Corporations ────────────
// Mounted at /api/calc/section-304 (iter 554). Pure compute. § 304
// prevents brother-sister and parent-subsidiary stock sales between
// commonly-controlled corporations from being recharacterized as
// capital-gain sales when the substance of the transaction is a
// dividend distribution from accumulated E&P. Closes the "extraction"
// tactic where a controlling shareholder converts what is economically
// a dividend into a § 1001 exchange.
//
// § 304(a)(1) brother-sister acquisitions: if one or more persons is
// in control of each of two corporations and sells stock of the
// issuing corp to the acquiring corp for property, the transaction is
// recharacterized as a § 301 distribution by acquiring corp.
//
// § 304(a)(2) parent-subsidiary acquisitions: subsidiary purchases
// parent stock from parent's shareholder → § 301 distribution from
// subsidiary.
//
// § 304(b)(1) control: 50% vote OR value (with § 318 constructive
// ownership applied).
//
// § 304(b)(2) E&P stacking order: distribution treated as first paid
// by acquiring corp E&P, then by issuing corp E&P. § 301(c)
// dividend/basis-recovery/capital-gain split applies.
//
// § 304(b)(3) foreign-corporation rules: § 1248 deemed-dividend rules
// coordinate with § 304. § 304(b)(4) anti-avoidance rules (Notice
// 2006-85 + Notice 2007-9 + 2012 final regs) prevent partnership-
// interposition circumvention.
//
// Six-mode severity ladder: NotApplicable,
// Section304InapplicableUnrelatedArmsLengthSale,
// Section304InapplicableNotInControlOfBothCorporations,
// Section304ARecharacterizationDividendUpToTotalEep,
// Section304BasisRecoveryAfterEepExhausted,
// Section304CapitalGainAfterBasisRecovered.
//
// Coordinates with § 318 (iter 552 — constructive ownership driving
// control determination), § 301 (distribution character + basis +
// capital gain split), § 311 (iter 550 — corp-level recognition on
// distribution), § 312 E&P computation, § 1(h)(11) qualified-dividend
// rate, § 1222 holding-period capital-gain character, § 1248 foreign-
// corp deemed-dividend, § 245A foreign-source DRD where applicable.

async fn section_304_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_304::Section304RelatedCorpRedemptionInput>,
) -> Result<Json<traderview_expense::section_304::Section304RelatedCorpRedemptionOutput>, ApiError>
{
    Ok(Json(traderview_expense::section_304::check(&b)))
}

// ── § 311 Taxability of Corporation on Distribution in Kind ─────────
// Mounted at /api/calc/section-311 (iter 550). Pure compute. § 311
// governs gain/loss recognition by the DISTRIBUTING corporation on
// distributions of property in kind to shareholders. The Tax Reform
// Act of 1986 repealed the General Utilities doctrine and added
// § 311(b), which requires the distributing corporation to recognize
// gain on appreciated-property distributions as if the property had
// been sold at fair market value. § 311(a) continues to deny LOSS
// recognition on depreciated-property distributions — losses are
// preserved only through § 336 liquidation distributions, sales, or
// worthlessness under § 165(g).
//
// § 311(a) general rule: no gain or loss to distributing corporation
// on distribution of property with respect to its stock (subject to
// § 311(b)).
//
// § 311(b)(1) appreciated property: gain recognized as if sold at FMV.
// § 311(b)(2) liability floor (cross-reference to § 336(b)): FMV deemed
// not less than the amount of liability assumed by shareholder OR to
// which the property is subject. § 311(b)(3) partnership/trust
// anti-loss rule: gain computed without regard to losses attributable
// to property contributed for principal purpose of recognizing loss
// on the distribution.
//
// § 311 INAPPLICABLE to (a) § 332 parent-subsidiary liquidations,
// (b) § 336 corporate liquidations, (c) § 355 corporate-division
// distributions — each governed by its own gain/loss-recognition regime.
//
// Seven-mode severity ladder: NotApplicable,
// Section311InapplicableOtherDistributionRegime,
// Section311ANoGainOrLossNeutralDistribution,
// Section311ANoLossOnDepreciatedPropertyDistribution,
// Section311BGainRecognitionAppreciatedProperty,
// Section311BWithLiabilityAssumptionFmvFloorAdjustment,
// Section311B3PartnershipTrustAntiLossDisallowance.
//
// Coordinates with § 312 E&P computation (recognized gain increases
// E&P), § 301(b)/(c)/(d) distributee dividend / basis / capital-gain
// treatment, § 332/337 parent-sub liquidation regime, § 336 general
// liquidation, § 355 corporate-division non-recognition, § 1374 S-corp
// built-in-gain tax (10-year window), § 1245/§ 1250 depreciation
// recapture, § 165(g) worthless-security loss preservation, § 267
// related-party loss deferral.

async fn section_311_route(
    _u: AuthUser,
    Json(b): Json<
        traderview_expense::section_311::Section311CorporateDistributionGainRecognitionInput,
    >,
) -> Result<
    Json<traderview_expense::section_311::Section311CorporateDistributionGainRecognitionOutput>,
    ApiError,
> {
    Ok(Json(traderview_expense::section_311::check(&b)))
}

// ── § 312 Effect on Earnings and Profits ─────────────────────────────
// Mounted at /api/calc/section-312 (iter 558). Pure compute. § 312
// governs the computation and adjustment of a corporation's E&P. E&P
// is the touchstone for distribution character determination throughout
// subchapter C: § 301(c) classifies distributions as dividends to extent
// of E&P, basis recovery thereafter, capital gain on excess. § 302
// redemption analysis turns on E&P when redemption defaults to § 301
// distribution. § 304 brother-sister stacks acquiring then issuing E&P.
//
// § 312(a) general rule: E&P decreased by cash + principal of
// obligations + adjusted basis of distributed property.
//
// § 312(b) appreciated-property distribution: E&P increased by § 311(b)
// recognized gain, then decreased by FMV (net effect equals decrease
// by basis + gain accrual).
//
// § 312(c) liability-assumption modifier: E&P decrease reduced by
// liability assumed by shareholder OR to which property is subject.
//
// § 312(d) stock dividend: § 305(a) non-taxable does not reduce E&P;
// § 305(b) taxable does (by FMV).
//
// § 312(k)(3) ADS straight-line depreciation: § 168(g)(2) Alternative
// Depreciation System applies for E&P (vs accelerated for taxable
// income); creates permanent timing difference.
//
// § 312(n) special adjustments to align E&P with economic income:
// (n)(1) construction-period interest + (n)(2) intangible drilling +
// (n)(3) circulation/organizational amortization + (n)(4) LIFO + (n)(5)
// installment sales (full recognition in year of sale) + (n)(6)
// completed-contract + (n)(7) stock redemptions + (n)(8) foreign
// corporation PTEP under § 959 + § 961.
//
// Nine-mode severity ladder: NotApplicable,
// EepDecreaseCashOrObligation,
// EepDecreaseDistributedPropertyBasis,
// EepIncreaseAppreciationThenDecreaseFmv,
// EepDecreaseAdjustedByLiabilityAssumed,
// EepUnchangedNonTaxableStockDividendSection305A,
// EepReducedTaxableStockDividendSection305B,
// EepAdjustedAdsStraightLineSection312K3,
// EepFullRecognitionSection312N5InstallmentSale.
//
// Coordinates with § 301 distribution character + § 302 redemption
// (iter 556) + § 304 related-corp (iter 554) + § 311 corporate-level
// recognition (iter 550) + § 305 stock-dividend rules + § 307 basis
// allocation + § 168(g)(2) ADS + § 959 + § 961 PTEP.

async fn section_312_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_312::Section312EepAdjustmentInput>,
) -> Result<Json<traderview_expense::section_312::Section312EepAdjustmentOutput>, ApiError> {
    Ok(Json(traderview_expense::section_312::check(&b)))
}

// ── § 318 Constructive Ownership of Stock ────────────────────────────
// Mounted at /api/calc/section-318 (iter 552). Pure compute. § 318
// attributes stock ownership from one taxpayer to another for purposes
// of related-party and ownership-percentage testing throughout the
// Code. Drives stock-redemption analysis (§ 302), brother-sister
// redemption recharacterization (§ 304), accumulated-earnings tax
// (§ 531), personal-holding-company tax (§ 542), corporate-attribution-
// to-shareholder rules, qualified-personal-service-corporation testing.
//
// § 318(a)(1) family attribution: individual treated as owning stock
// owned by spouse (unless legally separated), children, grandchildren,
// and parents. § 318 family attribution does NOT extend to SIBLINGS or
// grandparents (compare § 267 which DOES include siblings).
//
// § 318(a)(2) entity-to-owner attribution: proportional for
// partnership/estate, trust (actuarial under § 7520), and corporation
// (50%+ shareholders).
//
// § 318(a)(3) owner-to-entity attribution: in FULL for partnership/
// estate, trust, and corporation (50%+ shareholder).
//
// § 318(a)(4) option attribution: holder of call option, warrant, or
// convertible debenture treated as owning the underlying stock.
// § 318(a)(5)(D) option attribution takes priority over family
// attribution when both could apply.
//
// § 318(a)(5)(B) family-to-family re-attribution DISALLOWED.
// § 318(a)(5)(C) entity-bounce re-attribution DISALLOWED.
//
// Nine-mode severity ladder: NotApplicable,
// NoAttributionUnderSection318,
// Section318A1FamilyAttributionApplies,
// Section318A2EntityToOwnerProportionalAttribution,
// Section318A3OwnerToEntityFullAttribution,
// Section318A4OptionAttributionApplies,
// Section318A5BReAttributionDisallowedFamilyToFamily,
// Section318A5CReAttributionDisallowedEntityBounce,
// SiblingNotIncludedInSection318FamilyAttribution.

async fn section_318_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_318::Section318ConstructiveOwnershipInput>,
) -> Result<Json<traderview_expense::section_318::Section318ConstructiveOwnershipOutput>, ApiError>
{
    Ok(Json(traderview_expense::section_318::check(&b)))
}

// ── §331 shareholder gain/loss in corporate complete liquidation ─
// Mounted at /api/calc/section-331. §331(a) treats liquidating
// distribution as in full payment for stock (§1001 exchange);
// §331(b) §301 dividend rules inapplicable; capital character when
// stock is capital asset (§1221); §332 corporate-parent 80%/80%
// (§1504(a)(2)) non-recognition exception + §334(b) carryover
// basis; §334(a) shareholder basis in non-cash property = FMV;
// partial liquidations fall to §302 redemption analysis.

async fn section_331_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_331::Section331Input>,
) -> Result<Json<traderview_expense::section_331::Section331Result>, ApiError> {
    if b.adjusted_basis_in_stock_dollars < 0
        || b.cash_received_dollars < 0
        || b.fmv_non_cash_property_received_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_331::compute(&b)))
}

// ── §332 complete liquidations of subsidiaries ──────────────────────
// Mounted at /api/calc/section-332. §332(a) parent corporation no
// gain/loss recognition on receipt of property in complete liquidation
// of subsidiary IF 4-prong test satisfied: (1) §332(b)(2) 80% voting
// power AND (2) 80% value (§1504(a)(2) test) AND (3) continuous 80%
// ownership maintained from plan-adoption date through final
// distribution AND (4) complete liquidation (all property distributed,
// all stock cancelled). §337(a) parallel subsidiary non-recognition.
// §334(b)(1) parent takes carryover basis (NOT FMV). Failing any
// prong falls to §331/§336 FMV recognition.

async fn section_332_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_332::Section332Input>,
) -> Result<Json<traderview_expense::section_332::Section332Result>, ApiError> {
    if b.fmv_of_property_distributed_cents < 0
        || b.subsidiary_adjusted_basis_cents < 0
        || b.parent_basis_in_subsidiary_stock_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if b.voting_power_owned_bp > 10_000 || b.value_owned_bp > 10_000 {
        return Err(ApiError::BadRequest(
            "voting_power_owned_bp and value_owned_bp must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_332::compute(&b)))
}

// ── §1234A character of gain/loss on right termination ─────────────
// Mounted at /api/calc/section-1234a. §1234A(1) treats gain/loss on
// cancellation, lapse, expiration, or other termination of a right or
// obligation with respect to property that is (or would be on
// acquisition) a capital asset as gain/loss from the sale of a
// capital asset; holding period of the RIGHT governs §1222 character.
// §1234A(2) routes character of terminated §1256 contracts to the
// §1256(a)(3) 60/40 split, ignoring holding period. §1234A excludes
// securities futures contracts — §1234B governs those. Ordinary
// underlying property is OUTSIDE §1234A scope (§ 165 / § 1231 govern).

async fn section_1234a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1234a::Section1234AInput>,
) -> Result<Json<traderview_expense::section_1234a::Section1234AResult>, ApiError> {
    Ok(Json(traderview_expense::section_1234a::compute(&b)))
}

// ── §1234B character of gain/loss on securities futures contracts ─
// Mounted at /api/calc/section-1234b. §1234B(a) character mirrors
// underlying property (capital underlying → capital character;
// ordinary underlying → ordinary character). §1234B(b) — gain/loss
// on sale/exchange/termination of a securities futures contract TO
// SELL property is treated as SHORT-TERM CAPITAL regardless of
// holding period (parallels § 1233 short-sale rule). §1256(b)(1)(E)
// override — DEALER securities futures contracts are § 1256 contracts
// and get the § 1256(a)(3) 60/40 split, BEFORE § 1234B engages.
// §1234B(c) defines SFC via Securities Exchange Act § 3(a)(55)(A).
// §1234B(d) — SFC is not a commodity futures contract.

async fn section_1234b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1234b::Section1234BInput>,
) -> Result<Json<traderview_expense::section_1234b::Section1234BResult>, ApiError> {
    Ok(Json(traderview_expense::section_1234b::compute(&b)))
}

// ── §263(g) capitalization of interest + carrying charges on straddles ─
// Mounted at /api/calc/section-263g. §263(g)(1) general rule disallows
// current deduction for interest + carrying charges allocable to
// personal property that is part of a §1092(c) straddle; disallowed
// amount is chargeable to the capital account (basis) of the straddle
// property — timing-only, not permanent. §263(g)(2) defines interest
// and carrying charges as the EXCESS of (A) interest-on-indebtedness +
// other carrying costs (storage / insurance / transport) OVER (B)
// interest received + ordinary income from property + dividends net of
// §243 DRD + security loan fee payments includible in gross income.
// §263(g)(3) exempts §1256(e) hedging transactions (bona fide hedge of
// inventory / ordinary obligations / borrowings; identified before
// close of day entered into). §263(g)(4) provides coordination rules
// with §263(h) short-sale + §1277/§1282 market-discount/OID rules.

async fn section_263g_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_263g::Section263GInput>,
) -> Result<Json<traderview_expense::section_263g::Section263GResult>, ApiError> {
    if b.interest_on_indebtedness_cents < 0
        || b.carrying_costs_cents < 0
        || b.interest_received_cents < 0
        || b.ordinary_income_from_property_cents < 0
        || b.dividends_received_cents < 0
        || b.dividend_received_deduction_cents < 0
        || b.security_loan_fees_received_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_263g::compute(&b)))
}

// ── § 264 Insurance-Related Interest + Premium Disallowance ─────────
// Mounted at /api/calc/section-264 (iter 542). Pure compute. § 264
// disallows four categories of insurance-related deductions: (1)
// § 264(a)(1) premiums on life insurance covering officers/employees
// where the taxpayer is the beneficiary; (2) § 264(a)(2) interest on
// debt to purchase or carry SINGLE-PREMIUM life, endowment, or annuity
// contracts (per se non-deductible); (3) § 264(a)(3) interest on debt
// under plans of systematic borrowing against cash-value buildup (added
// by Tax Reform Act of 1986); (4) § 264(f) pro-rata interest
// disallowance for businesses owning life insurance on owners/employees
// (the "inside-buildup" regime added by Taxpayer Relief Act of 1997).
//
// § 264(c) FOUR EXCEPTIONS to § 264(a)(3) systematic-borrowing rule:
//   (c)(1) 4-of-7 — no part of 4 of first 7 annual premiums paid through debt;
//   (c)(2) trade-or-business — debt in connection with unrelated trade or business;
//   (c)(3) unforeseen-loss — substantial unforeseen income loss or obligation increase;
//   (c)(4) de minimis — interest paid ≤ $100 for the taxable year.
//
// § 264(a)(2) SINGLE-PREMIUM RULE has NO § 264(c) exceptions — disallowance
// is per se permanent. Single-premium definition: substantially all
// premiums paid within 4 years OR amount deposited with insurer for
// substantial future premiums.
//
// § 264(f) PRO-RATA DISALLOWANCE formula: disallowance = total interest
// × (avg unborrowed policy cash value / avg total assets). § 264(f)(4)
// exceptions: (A) owner-employee 20%+ policies, (E) key-person policies
// limited to 20 individuals max, $50K aggregate de minimis threshold.
//
// Ten-mode severity ladder: NotApplicable,
// Section264A2SinglePremiumInterestDisallowedPerSe,
// Section264A3SystematicBorrowingInterestDisallowed,
// Section264cFourOfSevenExceptionPreservesDeduction,
// Section264cTradeOrBusinessExceptionPreservesDeduction,
// Section264cUnforeseenLossExceptionPreservesDeduction,
// Section264cDeMinimisInterestExceptionPreservesDeduction,
// Section264fProRataInterestDisallowanceApplied,
// Section264f4ExceptionAppliesNoProRataDisallowance,
// NonSinglePremiumNoSystematicBorrowingNoDisallowance.
//
// Coordinates with § 163(j) business-interest limit (separate cap on
// remaining interest), § 265 (iter 532 — tax-exempt-income interest
// disallowance parallel), § 246A (iter 530 — debt-financed portfolio
// stock DRD reduction), § 101(j) employer-owned life insurance EOLI
// notice + consent requirements enacted by Pension Protection Act of
// 2006, § 7702 modified endowment contract rules.

async fn section_264_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_264::Section264InsuranceInterestDisallowanceInput>,
) -> Result<
    Json<traderview_expense::section_264::Section264InsuranceInterestDisallowanceOutput>,
    ApiError,
> {
    Ok(Json(traderview_expense::section_264::check(&b)))
}

// ── § 265 Expenses + Interest Relating to Tax-Exempt Income ──────────
// Mounted at /api/calc/section-265 (iter 532). Pure compute. § 265
// disallows deductions for expenses (§ 265(a)(1)) and interest
// (§ 265(a)(2)) allocable to wholly tax-exempt income, preventing
// double-dipping when a taxpayer borrows to acquire or carry tax-exempt
// obligations (municipal bonds, exempt-interest dividends from RIC under
// § 852(b)(5)) while claiming an interest deduction that would offset
// taxable income.
//
// § 265(a)(2) tracing test (Wisconsin Cheeseman v. United States,
// 7th Cir. 1968): disallowance applies when debt is "incurred or
// continued" to purchase or carry tax-exempt obligations. Direct-tracing
// standard for individuals + non-financial corporations; IRS bears
// burden of proving the tracing connection. Mere co-existence of debt
// and tax-exempt holdings is insufficient. Rev. Proc. 72-18 sets out
// the framework + safe harbors; Rev. Proc. 87-53 amplifies for
// non-financial corporations.
//
// § 265(b) BANK / FINANCIAL INSTITUTION REGIME (Tax Reform Act of 1986
// effective Aug 7 1986): MECHANICAL pro-rata disallowance based on
// ratio of average adjusted bases of tax-exempt obligations to average
// adjusted bases of all taxpayer assets. NO tracing test — formula
// applies regardless of debt source. Effective only for obligations
// acquired AFTER Aug 7 1986; pre-Aug-7-1986 obligations grandfathered.
//
// § 265(b)(3) BANK-QUALIFIED OBLIGATIONS ("qualified tax-exempt
// obligations"): issuer must reasonably anticipate issuing no more than
// $10,000,000 of tax-exempt obligations during the calendar year.
// § 291(e) reduces the bank-qualified disallowance to 20% (vs 100% for
// non-bank-qualified). Bank-qualified bonds are attractive small-issuer
// securities sold at lower rates than non-bank-qualified.
//
// Dealer-in-tax-exempt-obligations safe harbor (Rev. Proc. 72-18 § 7):
// dealers carrying inventory of tax-exempt obligations are EXEMPT from
// § 265 disallowance for interest expense on inventory-line debt.
//
// Nine-mode severity ladder: NotApplicable, NoTaxExemptIncomeNoDisallow-
// ance, UnrelatedDebtNoSectionTwoSixFiveADisallowance, IndividualDirect-
// TracedFullDisallowance, NonFinancialCorpDirectTracedFullDisallowance,
// BankNonBankQualifiedHundredPctProRataDisallowance,
// BankQualifiedTwentyPctSection291EDisallowance,
// BankPreAugust1986GrandfatheredNoDisallowance,
// DealerInTaxExemptObligationsSafeHarborNoDisallowance.
//
// Coordinates with § 163(j) interest-deduction limitation (excess
// business interest expense), § 246A debt-financed-portfolio-stock DRD
// reduction (iter 530 — parallel debt-financing disallowance logic),
// § 291(e) bank preference items (20% bank-qualified disallowance),
// § 852(b)(5) RIC exempt-interest dividends, § 103 municipal bond
// interest exclusion (source of the tax-exempt income).

async fn section_265_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_265::Section265TaxExemptInterestDisallowanceInput>,
) -> Result<
    Json<traderview_expense::section_265::Section265TaxExemptInterestDisallowanceOutput>,
    ApiError,
> {
    Ok(Json(traderview_expense::section_265::check(&b)))
}

// ── §1276 market-discount-bond ordinary-income recharacterization ─
// Mounted at /api/calc/section-1276. §1276(a)(1) general rule: gain
// on disposition of any market discount bond is ordinary income up
// to accrued market discount. §1276(a)(2): non-sale dispositions
// (gift, distribution) treated as realizing FMV. §1276(a)(3): partial
// principal payment ordinary up to accrued. §1276(a)(4): amount
// treated as INTEREST for purposes of the Code (with carve-outs for
// §§ 103, 871(a), 881, 1441, 1442, 6049). §1276(b)(1) ratable accrual
// default = market_discount × (days_held / total_days). §1276(b)(2)
// constant-yield election uses §1272(a) OID formula (caller supplies
// computed accrual). §1278(a)(2)(A) defines market discount = stated
// redemption − basis (clamped at zero). §1278(b) current-inclusion
// election lets taxpayer recognize annually; prior-year accrual
// subtracts from §1276 disposition cap to avoid double inclusion.

async fn section_1276_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1276::Section1276Input>,
) -> Result<Json<traderview_expense::section_1276::Section1276Result>, ApiError> {
    if b.purchase_price_cents < 0
        || b.stated_redemption_at_maturity_cents < 0
        || b.realized_amount_cents < 0
        || b.constant_yield_accrual_cents < 0
        || b.prior_years_accrual_already_taxed_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1276::compute(&b)))
}

// ── §1277 deferral of interest deduction on market-discount bonds ─
// Mounted at /api/calc/section-1277. Direct companion to §1276.
// §1277(a) general rule: net direct interest expense (NDIE) on
// indebtedness to purchase/carry a market discount bond is deductible
// in the current year ONLY to the extent it exceeds the portion of
// market discount allocable to the days during the taxable year on
// which the taxpayer held the bond. §1277(b)(1) net-interest-income
// carryover recovery: disallowed amount recovered in later year up
// to net interest income on that bond. §1277(b)(2) disposition
// terminal recovery: all remaining deferred amount recovered in the
// disposition year. §1277(c) NDIE definition = excess of interest
// paid/accrued on indebtedness OVER interest (incl. OID under
// §1272(a)) includible in gross income on the bond. §1278(b)
// current-inclusion election exempts taxpayer from §1277 deferral
// because matching market-discount income is recognized currently.
// §1277(d) was struck out entirely by Pub. L. 103-66 (1993).

async fn section_1277_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1277::Section1277Input>,
) -> Result<Json<traderview_expense::section_1277::Section1277Result>, ApiError> {
    if b.interest_on_indebtedness_cents < 0
        || b.interest_income_on_bond_cents < 0
        || b.accrued_market_discount_for_year_cents < 0
        || b.net_interest_income_for_year_cents < 0
        || b.prior_year_disallowed_carryover_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1277::compute(&b)))
}

// ── §1278 market-discount-bond definitions + § 1278(b) election ───
// Mounted at /api/calc/section-1278. The definitional + election
// module that both §1276 (ordinary-income recharacterization) and
// §1277 (interest-deduction deferral) cross-reference. §1278(a)(1)
// market discount bond definition with carve-outs for U.S. savings
// bonds, short-term obligations (≤ 1 year to maturity), and §453B
// installment obligations. §1278(a)(2)(A) market discount = stated
// redemption price at maturity − basis at acquisition. §1278(a)(2)(B)
// OID bonds use REVISED ISSUE PRICE (acquisition-date OID-adjusted
// basis) in lieu of stated redemption price. §1278(a)(2)(C) DE
// MINIMIS rule — raw discount STRICTLY LESS THAN ¼ of 1% of stated
// redemption × complete years to maturity is treated as ZERO.
// §1278(b)(1) current-inclusion election — switches off §1276
// disposition recharacterization AND §1277 interest deferral.
// §1278(b)(2) election scope to all market discount bonds acquired
// during or after year of election. §1278(b)(3) election IRREVOCABLE
// absent Secretary's consent.

async fn section_1278_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1278::Section1278Input>,
) -> Result<Json<traderview_expense::section_1278::Section1278Result>, ApiError> {
    if b.stated_redemption_price_cents < 0
        || b.revised_issue_price_cents < 0
        || b.purchase_price_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1278::compute(&b)))
}

// ── §1271 retirement of debt instruments treated as sale/exchange ─
// Mounted at /api/calc/section-1271. §1271(a)(1) general rule
// amounts received on retirement of any debt instrument considered
// amounts received in exchange therefor — default capital character.
// §1271(a)(2) intent-to-call OID instruments — gain up to OID
// (reduced by §1271(c) prior-year inclusions) recharacterized as
// ordinary income; carve-outs for tax-exempt obligations and
// premium-buyers. §1271(a)(3) short-term government obligations
// ≤1 year to maturity — gain up to ratable share of acquisition
// discount recharacterized as ordinary. §1271(a)(4) short-term
// nongovernment obligations — gain up to ratable share of OID
// recharacterized as ordinary. §1271(b) natural-person issuer
// exception — § 1271 does not apply to obligations issued by
// natural persons before June 9, 1997. §1271(c) no double
// inclusion — § 1271, § 1272, and § 1286 do not require inclusion
// of amounts previously includible in gross income.

async fn section_1271_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1271::Section1271Input>,
) -> Result<Json<traderview_expense::section_1271::Section1271Result>, ApiError> {
    if b.purchase_price_cents < 0
        || b.redemption_amount_cents < 0
        || b.original_issue_discount_cents < 0
        || b.oid_previously_included_cents < 0
        || b.acquisition_discount_cents < 0
        || b.ratable_short_term_accrual_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1271::compute(&b)))
}

// ── §1272 current inclusion of original issue discount (OID) ──────
// Mounted at /api/calc/section-1272. §1272(a)(1) general rule —
// holder must include sum of daily portions of OID in gross income
// each year regardless of cash received (phantom income). § 1272(a)(2)
// carve-outs: (A) tax-exempt obligations; (B) U.S. savings bonds;
// (C) short-term obligations ≤ 1 year (§ 1281 + § 1283 govern);
// (D) natural-person small loans ≤ $10,000 not for tax avoidance.
// § 1272(a)(3) daily-portion ratable allocation by days held.
// § 1272(a)(6) prepayable mortgage-backed / REMIC special PV
// methodology. § 1272(a)(7) acquisition-premium reduction —
// secondary-market basis above adjusted issue price reduces daily-
// portion by fraction (basis − AIP) / (stated redemption − AIP).
// Companion to § 1271 (retirement; § 1271(c) no double inclusion)
// and § 1273 (OID definition).

async fn section_1272_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1272::Section1272Input>,
) -> Result<Json<traderview_expense::section_1272::Section1272Result>, ApiError> {
    if b.adjusted_issue_price_start_of_year_cents < 0
        || b.adjusted_issue_price_end_of_year_cents < 0
        || b.acquisition_premium_cents < 0
        || b.stated_redemption_minus_aip_at_acquisition_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1272::compute(&b)))
}

// ── §1273 OID definition + issue price determination ─────────────
// Mounted at /api/calc/section-1273. Definitional anchor for the
// OID cluster. § 1273(a)(1) OID = excess of stated redemption price
// at maturity over issue price. § 1273(a)(2) SRPM = amount fixed by
// last modification of purchase agreement. § 1273(a)(3) DE MINIMIS
// — raw OID strictly less than ¼ of 1% × SRPM × complete years to
// maturity treated as ZERO (same factor as § 1278(a)(2)(C) market
// discount). § 1273(b)(1) publicly offered cash issue = initial
// offering price to public. § 1273(b)(2) non-public cash = price
// paid by first buyer. § 1273(b)(3) traded debt (issued for property
// where debt OR property is publicly traded) = FMV of debt
// instrument. § 1273(b)(4) residual case = SRPM minus OID (caller-
// supplied OID typically from § 1274 AFR imputation). § 1273(b)(5)
// "property" includes services and right to use property.

async fn section_1273_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1273::Section1273Input>,
) -> Result<Json<traderview_expense::section_1273::Section1273Result>, ApiError> {
    if b.stated_redemption_price_at_maturity_cents < 0
        || b.initial_public_offering_price_cents < 0
        || b.first_buyer_price_cents < 0
        || b.fmv_of_debt_instrument_cents < 0
        || b.residual_oid_amount_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1273::compute(&b)))
}

// ── §1274 issue price for debt instruments issued for property ──
// Mounted at /api/calc/section-1274. § 1274 is the foundational
// anti-abuse rule that prevents seller-financed property transactions
// from concealing imputed interest as disguised principal. § 1274(a)
// general rule: issue price = (1) STATED PRINCIPAL AMOUNT if there is
// adequate stated interest (stated interest rate ≥ AFR) OR (2)
// IMPUTED PRINCIPAL AMOUNT under § 1274(b) if not adequate stated
// interest. § 1274(b) imputed principal amount = sum of PRESENT
// VALUES of ALL PAYMENTS due under debt instrument, computed using
// AFR COMPOUNDED SEMIANNUALLY as of sale/exchange date. § 1274(b)(3)
// exception for potentially abusive situations (tax shelters,
// nonrecourse financing, unusually long terms): FAIR MARKET VALUE of
// property received governs. § 1274(c) applicability scope: applies
// if (1) some payments due MORE THAN 6 MONTHS after sale AND (2)
// total payments exceed stated principal or imputed principal.
// § 1274(c)(3) key exceptions: (A) farm sales ≤ $1,000,000 by
// individuals/small businesses; (B) principal residence sales;
// (C) total payments ≤ $250,000 aggregate; (D) publicly traded debt
// under § 1273(b); (E) patent transfers under § 1235. § 1274(d) AFR
// three-tier framework: SHORT-TERM AFR (term ≤ 3 years); MID-TERM
// AFR (term > 3 but ≤ 9 years); LONG-TERM AFR (term > 9 years);
// IRS publishes AFRs monthly based on Treasury obligation yields;
// taxpayer may elect LOWEST of three monthly AFRs from 3 months
// preceding binding contract (three-month lookback rule). § 1274(e)
// sale-leaseback adjustment: 110 PERCENT of AFR compounded
// semiannually when property sold and leased back. Cross-references:
// § 483 unstated-interest rules apply to transactions excluded from
// § 1274 (farms < $1M, principal residences, < $250,000); § 1273
// OID determination feeds from § 1274 issue price ((SRPM − issue
// price) = OID). Eleven-mode severity ladder × 2 transaction types ×
// 6 exception statuses × 2 payment-timing statuses × 3 debt-term
// categories × variable stated interest / AFR / principal / FMV /
// sale-leaseback / abusive-situation inputs. Sibling cluster:
// section_1271 (treatment of amounts received on retirement of debt
// instrument), section_1272 (current OID inclusion mechanics),
// section_1273 (general OID determination — § 1274 issue price
// feeds § 1273(a)(1) OID), section_1276 (market discount accrual),
// section_1277 / section_1278 (market discount deferred deductions),
// section_1281 (current inclusion on short-term obligations),
// section_1286 (stripped bonds — built iter 672; § 1273(a)(3) de
// minimis rule incorporated by § 1286), section_1287 (anti-bearer-
// bond rule), section_1258 (conversion transactions — uses § 1274(d)
// AFR for indefinite-term applicable rate), section_1260
// (constructive ownership transactions — uses § 6601 + § 6621 +
// § 1274(d) AFR for interest charge), section_6621 (built iter 674;
// federal short-term rate determined under § 1274(d) methodology),
// section_7872 (below-market loans — uses § 1274(d) AFR for imputed
// interest computation).

async fn section_1274_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1274::Section1274Input>,
) -> Result<Json<traderview_expense::section_1274::Section1274Result>, ApiError> {
    Ok(Json(traderview_expense::section_1274::compute(&b)))
}

// ── §1275 other definitions and special rules / OID anchor ──
// Mounted at /api/calc/section-1275. § 1275 is the definitional
// anchor for the OID statutory cluster (§§ 163(e), 1271-1275
// inclusive) and its implementing regulations at Treas. Reg.
// §§ 1.1275-1 through 1.1275-7. § 1275(a)(1)(A) defines "debt
// instrument" as any instrument constituting indebtedness under
// federal income tax principles (including CDs and loans).
// § 1275(a)(1)(B) excludes (i) annuity contracts depending on life
// expectancy with no disqualifying provisions (cash surrender, secured
// loan availability, max payout, decreasing payouts); (ii) annuity
// contracts issued by foreign insurers subject to subchapter L tax.
// § 1275(a)(2) issue date = date of first issue. § 1275(a)(3) issue
// price cross-references § 1273(b) (cash-sold / publicly offered) and
// § 1274 (debt-for-property). § 1275(b) personal-use property loan
// exception: for any loan between natural persons NOT issued in
// connection with trade/business of lender, OID rules of §§ 1272 and
// 1273 do NOT apply to borrower; borrower governed by cash
// receipts/disbursements method. § 1275(c) information requirements:
// Secretary requires all OID information sent to holders + reported
// on Form 1099-OID / Form 1099-INT under § 6049; failure triggers
// § 6721/§ 6722 penalties. § 1275(d) anti-abuse regs: Secretary
// prescribes regulations to prevent avoidance by variable-rate /
// contingent / convertible recharacterizations (Treas. Reg.
// §§ 1.1275-2 through 1.1275-7). § 1.1275-1 adjusted issue price =
// issue price + OID previously included − payments other than QSI.
// § 1.1275-2(a) OID payment allocation: each payment is FIRST a
// payment of OID to extent of accrued OID not allocated to prior
// payments, SECOND a payment of principal — prevents tax-motivated
// repayment-ordering schemes. Transition rule: current § 1.1275-1
// definitions apply to debt instruments issued ON OR AFTER
// March 13, 2001. Annuity grandfather date April 7, 1995 exempts
// pre-grandfather contracts from certain disqualifying-provision
// limits under Notice FI-33-94. Fourteen-mode severity ladder ×
// 4 instrument types × 6 compliance aspects × 3 payment-allocation
// orders × 5 annuity disqualifying-provision statuses × 2 transition-
// date statuses × variable adjusted-issue-price / info-return /
// personal-use-loan inputs. Sibling cluster: section_163e (issuer-
// side OID deduction; parallel to § 1272 holder-side inclusion),
// section_1271 (retirement of debt instrument), section_1272
// (current OID inclusion), section_1273 (general OID determination
// — § 1275(a)(3) cross-references § 1273(b)), section_1274 (issue
// price for debt-for-property exchanges — built iter 678; § 1275(a)(3)
// cross-references § 1274), section_1276 (market discount accrual),
// section_1277 / section_1278 (market discount deferred deductions),
// section_1281 (current inclusion on short-term obligations),
// section_1286 (built iter 672 — stripped bonds; § 1275(c)(3) info
// reporting on stripped bonds), section_1287 (anti-bearer-bond rule),
// section_6049 (info reporting on OID + interest; Form 1099-OID /
// Form 1099-INT), section_6621 (built iter 674 — federal short-term
// rate determined under § 1274(d) methodology that § 1275(a)(3)
// cross-references).

async fn section_1275_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1275::Section1275Input>,
) -> Result<Json<traderview_expense::section_1275::Section1275Result>, ApiError> {
    Ok(Json(traderview_expense::section_1275::compute(&b)))
}

// ── §1281 current inclusion of acquisition discount on short-term ─
// Mounted at /api/calc/section-1281. Bookend to OID cluster: § 1272
// governs long-term OID; § 1281 governs short-term obligations
// (≤ 1 year). § 1272(a)(2)(C) and § 1271(a)(3)/(a)(4) cross-reference
// § 1281. Critical distinction: § 1281 applies ONLY to specific
// holder categories — § 1281(b)(1)(A) accrual-method taxpayers +
// (B) dealers + (C) banks (§ 581) + (D) RICs + common trust funds +
// (E) § 1256(e)(2) hedging-transaction-identified + (F) stripped-
// bond strippers + § 1281(b)(2) pass-thru entities. Cash-method
// individual investors are OUTSIDE § 1281 scope and defer to
// § 1271(a)(3)/(a)(4) ratable accrual at disposition. § 1281(c)
// cross-references § 1283(c) for nongovernmental obligation OID-
// only limitation. § 1283(a)(1) short-term obligation definition
// (≤ 1 year to maturity); § 1283(a)(2) acquisition discount = SRPM
// minus basis at acquisition.

async fn section_1281_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1281::Section1281Input>,
) -> Result<Json<traderview_expense::section_1281::Section1281Result>, ApiError> {
    Ok(Json(traderview_expense::section_1281::compute(&b)))
}

// ── §1283 short-term obligation + acquisition discount definitions ─
// Mounted at /api/calc/section-1283. Definitional anchor for short-
// term obligation cluster. § 1281 (current inclusion) and § 1282
// (interest deduction deferral) both cross-reference § 1283 for
// underlying terms. § 1283(a)(1) defines short-term obligation as
// any bond/debenture/note/certificate with fixed maturity ≤ 1 year
// from date of issue (with tax-exempt carve-out). § 1283(a)(2)
// acquisition discount = SRPM minus basis. § 1283(b)(1) daily-
// portion ratable accrual = total discount divided by days from
// acquisition to maturity inclusive. § 1283(b)(2) constant-yield
// election parallels § 1272(a)(3) OID rules. § 1283(c) nongovern-
// mental obligations substitute OID for acquisition discount.
// § 1283(d) basis increased by § 1281 prior-year inclusion.

async fn section_1283_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1283::Section1283Input>,
) -> Result<Json<traderview_expense::section_1283::Section1283Result>, ApiError> {
    if b.stated_redemption_price_at_maturity_cents < 0
        || b.basis_at_acquisition_cents < 0
        || b.oid_amount_for_nongovernmental_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1283::compute(&b)))
}

// ── §1286 tax treatment of stripped bonds / coupon-stripping OID ──
// Mounted at /api/calc/section-1286. § 1286 was enacted as successor
// to predecessor § 1232B by the Tax Equity and Fiscal Responsibility
// Act of 1982 (Public Law 97-248) and applies to stripped bonds and
// stripped coupons PURCHASED AFTER JULY 1, 1982. § 1286(a) treats
// the purchaser of a stripped bond or coupon as having received a
// bond ORIGINALLY ISSUED on the purchase date with ORIGINAL ISSUE
// DISCOUNT equal to (STATED REDEMPTION PRICE AT MATURITY − RATABLE
// SHARE OF PURCHASE PRICE). § 1286(b) requires the person stripping
// coupons to include in gross income immediately before disposition
// (1) interest accrued on the bond while held by such person and not
// previously included AND (2) accrued market discount; basis is
// increased by the amount included then ALLOCATED between bond and
// coupons in proportion to FAIR MARKET VALUES at the time of
// disposition. § 1286(c)/(d) tax-exempt stripped obligation rules,
// added by Tax Reform Act of 1986 § 1879 effective for any purchase
// or sale AFTER JUNE 10, 1987, split the OID into a TAX-EXEMPT
// PORTION limited to the OID accruing at a YIELD equal to the LOWER
// of (A) the coupon rate or (B) the stripped obligation's yield to
// maturity, and a NON-EXEMPT PORTION subject to ordinary OID
// inclusion. § 1273(a)(3) DE MINIMIS OID RULE (incorporated by
// § 1286): if OID is LESS THAN 0.25 % × STATED REDEMPTION PRICE AT
// MATURITY × NUMBER OF COMPLETE YEARS TO MATURITY, OID is treated
// as ZERO and no current inclusion is required — a narrow escape
// for trivial OID amounts that does NOT apply to zero-coupon
// Treasury STRIPS or TIPS strips where OID is the entire return.
// 13-mode severity ladder × 3 taxpayer roles (stripper / purchaser /
// uninvolved) × 4 obligation types × 2 purchase-date statuses ×
// 3 tax-exempt amendment-date statuses × 4 stripper actions ×
// 5 purchaser actions × variable SRPM / purchase price / years to
// maturity inputs. Computes both the OID amount (SRPM − ratable
// share) and the § 1273(a)(3) de minimis threshold using u128
// saturating arithmetic. Sibling cluster: section_1271 (treatment
// of amounts received on retirement of debt instrument), section_
// 1272 (current OID inclusion mechanics), section_1273 (general
// OID determination + de minimis rule incorporated by § 1286),
// section_1274 (issue price determination for debt-for-property
// exchanges), section_1276 (market discount accrual), section_
// 1277 (deferred deduction on market-discount bond holding costs),
// section_1278 (market discount definitions), section_1281
// (current inclusion of interest on short-term obligations),
// section_1282 (deferred deductions on short-term obligation
// holding costs), section_1283 (definitions for net-direct-
// interest carry), section_1287 (anti-bearer-bond ordinary-income
// rule for registration-required obligations — TEFRA companion
// to § 4701 issuer excise tax), section_4701 (TEFRA issuer excise
// tax on registration-required obligations not in registered
// form).

async fn section_1286_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1286::Section1286Input>,
) -> Result<Json<traderview_expense::section_1286::Section1286Result>, ApiError> {
    Ok(Json(traderview_expense::section_1286::compute(&b)))
}

// ── §1287 anti-bearer-bond rule / denial of capital gain treatment ────
// Mounted at /api/calc/section-1287. § 1287(a) converts capital gain
// on disposition of registration-required obligation NOT in registered
// form to ORDINARY INCOME (unless issuer paid § 4701 tax). Enacted
// as part of TEFRA (Public Law 97-248 § 310) effective for obligations
// issued after December 31, 1982. § 1287(b)(1) "registration-required
// obligation" defined per § 163(f)(2) — exempts obligations with
// maturity ≤ 1 year, not offered to public, or foreign-targeted
// meeting Treasury conditions. § 1287(b)(2) "registered form"
// defined per § 163(f). § 4701 separate 1 % per year issuer excise
// tax on face amount; if paid, § 1287(a) parenthetical exception
// applies. Treas. Reg. § 1.165-12(c) holder exception requires no
// actual knowledge of registration failure with reasonable care.
// Pre-TEFRA obligations issued before Jan 1, 1983 grandfathered
// outside § 1287 regardless of registration status. September 2017
// Treasury Proposed Regulations (Federal Register 82 FR 43773)
// updated registered form definition. Cross-reference with § 1276
// market discount ordinary income treatment. Trader-relevant for
// family offices and international portfolio operators with legacy
// bearer bond holdings, off-shore-issued bonds, or non-registered
// debt instruments.

async fn section_1287_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1287::Section1287Input>,
) -> Result<Json<traderview_expense::section_1287::Section1287Result>, ApiError> {
    Ok(Json(traderview_expense::section_1287::compute(&b)))
}

// ── §1288 treatment of OID on tax-exempt obligations (muni OID) ──
// Mounted at /api/calc/section-1288. § 1288 is the closing companion
// to the OID statutory cluster (§§ 1271-1275 + § 1286 + § 1287) — the
// muni-bond rule ensuring OID on tax-exempt obligations (state and
// local bonds + other § 103-exempt instruments) accrues into the
// holder's adjusted basis WITHOUT being included in gross income.
// § 1288(a) general rule: OID treated as accruing in manner provided
// by § 1272(a) FOR PURPOSES OF DETERMINING ADJUSTED BASIS (with
// § 1272(a)(7) adjustments); same accrual method applies for
// interest-deduction purposes BUT without § 1272(a)(7) adjustments.
// § 1288(b)(1) NO DE MINIMIS RULE: OID on tax-exempt obligation
// determined under § 1273(a) WITHOUT § 1273(a)(3) — the § 1273(a)(3)
// de minimis OID rule (0.25 % × SRPM × years to maturity) DOES NOT
// APPLY to tax-exempt obligations; every dollar of OID accrues into
// adjusted basis no matter how small. § 1288(b)(2) AFR adjustments:
// Secretary prescribes regulations adjusting AFRs under § 483 and
// § 1274 to take into account tax-exemption benefit; tax-exempt AFR
// is typically lower than standard taxable AFR. § 1288(b)(3) tax-
// exempt obligation = § 1275(a)(3) cross-references § 103.
// § 1288(b)(4) short-term obligations (≤ 1 year): rules similar to
// § 1283(b). Effect: § 1288 RECONCILES § 103 gross-income exclusion
// with basis adjustment so disposition gain/loss properly computed;
// without § 1288, holders could claim basis step-up without OID
// accrual, generating artificial losses. Effective: Public Law 98-369
// § 41(c) (Deficit Reduction Act of 1984) on July 18, 1984; applies
// to obligations ISSUED AFTER September 3, 1982 AND ACQUIRED AFTER
// March 1, 1984. Eleven-mode severity ladder × 3 obligation
// classifications × 2 issuance-date statuses × 2 acquisition-date
// statuses × 4 compliance aspects × 3 de minimis application
// statuses × 4 gross-income / basis statuses. Sibling cluster:
// section_1271 (retirement of debt instrument), section_1272 (current
// OID inclusion; § 1272(a)(7) basis adjustment cross-reference),
// section_1273 (general OID determination; § 1288(b)(1) excludes
// § 1273(a)(3) de minimis rule), section_1274 (built iter 678 —
// issue price for debt-for-property; § 1288(b)(2) AFR adjustment),
// section_1275 (built iter 680 — other definitions; § 1288(b)(3)
// tax-exempt obligation definition cross-reference), section_1276
// (market discount accrual), section_1277 / section_1278 (market
// discount deferred deductions), section_1281 (current inclusion on
// short-term obligations), section_1282 (deferred deductions on
// short-term obligation holding costs), section_1283 (short-term
// obligation definitions; § 1288(b)(4) cross-reference), section_
// 1286 (built iter 672 — stripped bonds; § 1286(c)/(d) tax-exempt-
// stripped-obligation rules), section_1287 (anti-bearer-bond rule),
// section_103 (interest on certain state and local bonds — tax
// exemption foundation), section_6049 (information reporting on
// Form 1099-OID).

async fn section_1288_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1288::Section1288Input>,
) -> Result<Json<traderview_expense::section_1288::Section1288Result>, ApiError> {
    Ok(Json(traderview_expense::section_1288::compute(&b)))
}

// ── §1282 short-term obligation interest-deduction deferral ──────
// Mounted at /api/calc/section-1282. Direct short-term-obligation
// companion to section_1277 (long-term market-discount interest
// deferral parallel). § 1282(a) general rule defers net direct
// interest expense (NDIE) on indebtedness incurred to purchase or
// carry short-term obligation to extent of daily portions of
// acquisition discount allocable to days held in year. § 1282(b)(1)
// exception for § 1281 holders (already including discount
// currently — accrual + dealer + bank + RIC + hedging + stripper +
// pass-thru). § 1282(b)(2) election to apply § 1281 to all
// short-term obligations triggers § 1282(b) exception. § 1282(c)
// cross-reference to § 1277 long-term rules. § 1282(d) § 1283(c)
// nongovernmental OID substitution. Companion to section_1281
// (current inclusion mandate) + section_1283 (definitions).

async fn section_1282_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1282::Section1282Input>,
) -> Result<Json<traderview_expense::section_1282::Section1282Result>, ApiError> {
    if b.interest_expense_on_indebtedness_cents < 0 || b.interest_income_includible_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1282::compute(&b)))
}

// ── §7704 publicly traded partnership corporate treatment ────────
// Mounted at /api/calc/section-7704. Trader-critical for any
// investor holding master limited partnerships (MLPs) or PTPs.
// §7704(a) general rule treats PTP as CORPORATION losing pass-
// through status — unless §7704(c) exception applies. §7704(b)
// PTP definition has two prongs: (1) interests traded on
// established securities market OR (2) readily tradable on
// secondary market. §7704(c)(1) requires continuous compliance
// with 90% test every taxable year beginning after 1987-12-31.
// §7704(c)(2) 90% qualifying-income test. §7704(d)(1) seven
// qualifying-income categories: (A) interest + (B) dividends +
// (C) real property rents + (D) gain from real property + (E)
// mineral/natural-resource income + (F) qualifying capital asset
// gain + (G) commodities income. §7704(e) inadvertent-termination
// relief requires all three prongs: (i) inadvertent failure +
// (ii) corrective steps within reasonable time + (iii) agreement
// to required adjustments.

async fn section_7704_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_7704::Section7704Input>,
) -> Result<Json<traderview_expense::section_7704::Section7704Result>, ApiError> {
    if b.gross_income_total_cents < 0 || b.qualifying_income_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_7704::compute(&b)))
}

// ── §6045B issuer Form 8937 organizational action basis reporting ─
// Mounted at /api/calc/section-6045b. § 6045B requires issuers of
// specified securities to report organizational actions affecting
// basis to the IRS via Form 8937 within fixed deadline. § 6045B(a)
// return must describe the action AND include quantitative effect
// on basis. § 6045B(b) deadline = earlier of (1) 45 days after
// action OR (2) January 15 of year following calendar year of
// action. § 6045B(c) issuer must furnish written statement to
// nominees and holders by January 15 of following year. § 6045B(d)
// specified security defined by § 6045(g)(3). § 6045B(e) PUBLIC
// WEBSITE WAIVER via Treas. Reg. § 1.6045B-1(a)(3) — issuer is
// deemed to satisfy IRS filing duty by posting completed signed
// Form 8937 on public website for at least 10 YEARS. Companion to
// section_6045 (broker Form 1099-B downstream reporting).

async fn section_6045b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6045b::Section6045BInput>,
) -> Result<Json<traderview_expense::section_6045b::Section6045BResult>, ApiError> {
    if b.days_since_action > 100_000 || b.website_posting_duration_years > 100 {
        return Err(ApiError::BadRequest(
            "counters look invalid (>threshold)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6045b::check(&b)))
}

// ── §6045A broker-to-broker custody transfer statement ────────────
// Mounted at /api/calc/section-6045a. § 6045A requires the
// transferring broker (or other applicable person) to furnish a
// written information statement to the receiving broker within 15
// days of the transfer. Receiving broker uses statement to populate
// Form 1099-B basis reporting under § 6045 on eventual sale.
// § 6045A(a) general rule + § 6045A(b)(1) broker definition via
// § 6045(c)(1) + § 6045A(b)(2) other person per Secretary + § 6045A(c)
// 15-day deadline + § 6045A(d) digital-asset transfer return regime
// added by Infrastructure Investment and Jobs Act of 2021 Pub. L.
// 117-58 § 80603 effective post-2025-12-31 — broker transferring
// digital asset to non-broker account must make return showing
// transfer info. Treas. Reg. § 1.6045A-1 statement content: basis,
// acquisition date, wash-sale flag per § 1091. Companion to
// section_6045 (downstream Form 1099-B) and section_6045b (upstream
// issuer Form 8937).

async fn section_6045a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6045a::Section6045AInput>,
) -> Result<Json<traderview_expense::section_6045a::Section6045AResult>, ApiError> {
    if b.days_since_transfer > 100_000 {
        return Err(ApiError::BadRequest(
            "days_since_transfer looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6045a::check(&b)))
}

// ── §1297 PFIC classification income + asset tests ───────────────
// Mounted at /api/calc/section-1297. Trader-critical for any
// investor holding foreign mutual funds + foreign ETFs + foreign
// stock. § 1297(a)(1) 75% income test — 75% or more of gross income
// is passive income. § 1297(a)(2) 50% asset test — 50% or more of
// average assets produce passive income. EITHER test triggers PFIC
// status which subjects shareholder to § 1291 punitive excess-
// distribution + interest-charge regime unless QEF (§ 1295) or
// mark-to-market (§ 1296) election made. § 1297(b)(1) passive
// income = § 954(c) foreign personal holding company income.
// § 1297(b)(2) exceptions — (A) active banking + (B) active
// insurance + (C) related-party allocable income. § 1297(c) 25%
// look-through rule — foreign corp owning 25%+ of subsidiary by
// value is treated as holding proportionate share of subsidiary's
// assets and income. § 1297(d) once-a-PFIC qualified portion
// exception with § 1298(b)(1) purging election.

async fn section_1297_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1297::Section1297Input>,
) -> Result<Json<traderview_expense::section_1297::Section1297Result>, ApiError> {
    if b.gross_income_total_cents < 0 || b.passive_income_cents < 0 || b.avg_total_assets_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if b.avg_passive_assets_bp > 10_000 {
        return Err(ApiError::BadRequest(
            "avg_passive_assets_bp must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1297::compute(&b)))
}

// ── §1298 PFIC attribution + special rules + annual reporting ────
// Mounted at /api/calc/section-1298. Direct companion to section_1297
// (which cross-references § 1298(b)(1) purging election in § 1297(d)).
// § 1298(a)(2) 50% value corporation attribution; § 1298(a)(3)
// partnership/estate/trust proportionate attribution; § 1298(a)(4)
// options attribution per regulations; § 1298(b)(1) purging election
// under § 1291(d)(2) — pay current tax on accumulated PFIC gain to
// shed PFIC taint going forward; § 1298(b)(6) PLEDGE-AS-SECURITY
// DEEMED DISPOSITION — using PFIC stock as security for loan
// triggers deemed sale under § 1291; § 1298(f) annual Form 8621
// reporting required for every U.S. PFIC shareholder.

async fn section_1298_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1298::Section1298Input>,
) -> Result<Json<traderview_expense::section_1298::Section1298Result>, ApiError> {
    if b.pfic_stock_value_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if b.upstream_value_ownership_bp > 10_000 {
        return Err(ApiError::BadRequest(
            "upstream_value_ownership_bp must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1298::compute(&b)))
}

// ── §6038D Form 8938 foreign financial asset reporting ───────────
// Mounted at /api/calc/section-6038d. Trader-critical for anyone
// with offshore brokerage accounts + foreign mutual fund holdings
// (PFICs covered by § 1297/1298) + foreign retirement accounts +
// foreign-issued bonds + interests in foreign entities. § 6038D(a)
// requires individuals to attach Form 8938 if aggregate value of
// specified foreign financial assets exceeds threshold under
// § 6038D(b). Treas. Reg. § 1.6038D-2 tiers thresholds by filing
// status + residency. § 6038D(c) required information includes
// institution name + address + account number + issuer info +
// maximum value of asset during taxable year. § 6038D(d) $10,000
// initial penalty per failure to disclose. § 6038D(e) continuing
// $10,000 per 30-day period after 90-day IRS notice grace, capped
// at $50,000. § 6038D(g) reasonable-cause-AND-not-willful-neglect
// exception. Distinct from FinCEN Form 114 (FBAR) Bank Secrecy Act
// filing under 31 U.S.C. § 5314 with separate threshold and
// penalty regime.

// ── §6020 returns prepared for or executed by Secretary ─────────────
// Mounted at /api/calc/section-6020. § 6020(a) voluntary preparation
// (taxpayer consents + discloses + signs); SFR signed by taxpayer
// counts as filed return and starts § 6501 ASED. § 6020(b)(1)
// involuntary preparation when taxpayer fails to make return or
// makes false/fraudulent return; Secretary makes return from own
// knowledge and testimony. § 6020(b)(2) — Secretary-prepared return
// is PRIMA FACIE GOOD AND SUFFICIENT for all legal purposes. § 6020
// (b) SFR does NOT satisfy Beard test (Beard v. Commissioner, 82
// T.C. 766 (1984), aff'd 793 F.2d 139 (6th Cir. 1986)) prong 4
// (executed under penalties of perjury BY TAXPAYER); § 6501 ASED
// NEVER STARTS on § 6020(b) SFR — IRS may assess at any time
// forever. Late-filed valid return AFTER SFR starts § 6501 ASED
// clock. 26 CFR § 301.6020-1 + Form 13496 — § 6020(b) return must
// identify taxpayer + contain sufficient info + purport to be a
// return. Trader-relevant because non-filing trader receives § 6020
// (b) SFR with worst-case computations (no Schedule C deductions +
// no § 475(f) M2M + no § 1091 wash sale + no cost basis on 1099-B).
async fn section_6020_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6020::Section6020Input>,
) -> Result<Json<traderview_expense::section_6020::Section6020Result>, ApiError> {
    Ok(Json(traderview_expense::section_6020::check(&b)))
}

// ── §6035 Basis Info to Persons Acquiring Property From Decedent ────
// Mounted at /api/calc/section-6035. Public Law 114-41 § 2004 (signed
// July 31, 2015) added § 1014(f) basis consistency rule + § 6035
// executor reporting via Form 8971 + Schedule A within 30 days of
// estate tax return filing; supplemental Form 8971 due within 30
// days of final value determination or discovery of incorrect info.
// § 6662(k) 20% accuracy-related penalty on inconsistent estate
// basis underpayment. § 6721 + § 6722 information return / payee
// statement failure penalties (base $250 per failure). Final
// regulations published Federal Register September 17, 2024 —
// eliminated the zero-basis rule for unreported property; modified
// 'acquiring' definition for § 6035(a)(1) timing; eliminated
// subsequent-transfer reporting except for trustees. Companion to
// § 1014 (cost basis at death) and § 1014e (transferred-basis rule).
async fn section_6035_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6035::Section6035Input>,
) -> Result<Json<traderview_expense::section_6035::Section6035Result>, ApiError> {
    Ok(Json(traderview_expense::section_6035::check(&b)))
}

// ── §6038A Form 5472 25%-foreign-owned domestic corp + DRE ─────────
// Mounted at /api/calc/section-6038a. § 6038A(a) requires every
// 25%-foreign-owned domestic corp + foreign corp engaged in US
// trade/business to file Form 5472 reporting related-party
// transactions. § 6038A(c)(1) 25% threshold = direct or indirect
// foreign ownership of voting power OR total value at ANY TIME
// during taxable year. Treas. Reg. § 1.6038A-1(c) per T.D. 9796
// (December 13, 2016) effective tax years beginning 2017-01-01 —
// foreign-owned US single-member LLC disregarded entities treated as
// DOMESTIC CORPORATIONS for limited § 6038A purposes (Form 5472
// filed as attachment to pro-forma Form 1120). § 6038A(d)(1) BASE
// PENALTY $25,000 per taxable year per reporting corporation;
// § 6038A(d)(2) CONTINUATION PENALTY $25,000 per 30-day period (or
// fraction) after 90-day IRS notification — UNCAPPED; § 6038A(d)(3)
// reasonable cause defense under Treas. Reg. § 1.6038A-4(b).
// § 6501(c)(8) — § 6501 assessment SOL does NOT start running until
// required § 6038A return is filed, keeping ASED OPEN INDEFINITELY.
// Trader-critical for foreign-owned DE/WY/NV trading LLCs, foreign
// hedge fund US-LLC conduits, jointly-owned US LLCs with foreign
// family/business partners, and § 475(f) MTM-elected entities with
// intra-family transfers. Sibling cluster: § 6038D + § 6038 +
// § 6038B + § 6038C + § 6501(c)(8).

async fn section_6038a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6038a::Section6038aInput>,
) -> Result<Json<traderview_expense::section_6038a::Section6038aResult>, ApiError> {
    if b.max_foreign_ownership_bps > 10_000 {
        return Err(ApiError::BadRequest(
            "max_foreign_ownership_bps must be ≤ 10000 (100%)".into(),
        ));
    }
    if b.days_since_irs_notification > 100_000 {
        return Err(ApiError::BadRequest(
            "days_since_irs_notification out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6038a::check(&b)))
}

// ── §6038B Form 926 / Form 8865 transfer to foreign corp + partnership ─
// Mounted at /api/calc/section-6038b. § 6038B(a)(1)(A) Form 926
// transfers to foreign corp (§ 332/§ 351/§ 354/§ 355/§ 356/§ 361
// exchanges); § 6038B(a)(1)(B) Form 8865 § 721 contribution to
// foreign partnership. § 6038B(b)(1) BASE PENALTY = 10% of FMV at
// time of transfer; § 6038B(b)(1)(A) CAPPED at $100,000;
// § 6038B(b)(1)(B) INTENTIONAL DISREGARD removes cap. § 6038B(b)(2)
// failure forces § 367 gain recognition AS IF property sold at FMV
// (in addition to monetary penalty). § 6038B(c) reasonable cause
// defense under Treas. Reg. § 1.6038B-1(f)(3) / § 1.6038B-2(j)(3).
// § 367(d) intangibles trigger DEEMED-SALE treatment requiring
// annual commensurate-with-income inclusion. § 721(c) gain-deferral
// method under Treas. Reg. § 1.721(c)-3 available for related-party
// foreign partnership transfers (multi-year reporting + remedial
// allocations). § 6501(c)(8) — § 6501 assessment SOL OPEN
// INDEFINITELY on non-filing. Trader-critical for cryptocurrency
// transfers to foreign exchanges/wallets (Notice 2014-21 property
// classification), intangible asset transfers (trading algorithms,
// proprietary models, IP), § 351 contributions to foreign-
// incorporated trading entities, § 721 contributions to foreign
// partnership trading vehicles, and master/feeder/parallel fund
// structures. Sibling cluster: § 6038A + § 6038D + § 367 + § 721 +
// § 721(c) + § 6501(c)(8).

async fn section_6038b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6038b::Section6038bInput>,
) -> Result<Json<traderview_expense::section_6038b::Section6038bResult>, ApiError> {
    if b.ownership_pct_after_transfer_bps > 10_000 {
        return Err(ApiError::BadRequest(
            "ownership_pct_after_transfer_bps must be ≤ 10000 (100%)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6038b::check(&b)))
}

// ── §6038C foreign corp engaged in US trade or business — Form 5472 ─
// Mounted at /api/calc/section-6038c. § 6038C(a) — foreign corp
// engaged in US T/B at any time during taxable year SHALL furnish
// information described in § 6038A(b) (related party + reportable
// transactions) AND maintain records prescribed by regulations.
// § 6038C(b) — penalties of § 6038A apply (cross-reference):
// $25,000 base + $25,000/30-day continuation (UNCAPPED after 90-day
// notification) + reasonable cause defense. § 6038C(c) — LIMITED
// AGENT authorization rule: rules apply to any transaction with
// foreign-person related party UNLESS related party AGREES to
// authorize reporting corp as limited agent for § 7602 (examination)
// + § 7603 (service of summons) + § 7604 (enforcement of summons)
// purposes. § 6038C(d) — terms 'related party' + 'foreign person' +
// 'records' have same meaning as § 6038A(c) (cross-reference).
// § 864(b)(2) trading safe harbor — foreign person NOT a dealer who
// trades for own account through resident broker/agent does NOT
// have US T/B; if safe harbor qualifies, NO § 6038C exposure.
// § 882 — foreign corp engaged in US T/B taxed on ECI; § 6038C
// provides reporting backbone. § 6501(c)(8) — § 6501 ASED OPEN
// INDEFINITELY on non-filing. Anti-avoidance backstop closing
// foreign-corp reporting cluster with § 6038A + § 6038B. Trader-
// critical for foreign hedge fund LPs with US branch, foreign
// proprietary trading firms with US-based traders (potential loss
// of § 864(b)(2) safe harbor), foreign brokerages with US
// permanent establishment, foreign trader-managed family offices
// with US ECI. Statutory origin: Omnibus Budget Reconciliation Act
// of 1990 § 11315 (Pub. L. 101-508, November 5, 1990).

async fn section_6038c_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6038c::Section6038cInput>,
) -> Result<Json<traderview_expense::section_6038c::Section6038cResult>, ApiError> {
    if b.days_since_irs_notification > 100_000 {
        return Err(ApiError::BadRequest(
            "days_since_irs_notification out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6038c::check(&b)))
}

async fn section_6038d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6038d::Section6038DInput>,
) -> Result<Json<traderview_expense::section_6038d::Section6038DResult>, ApiError> {
    if b.aggregate_value_year_end_cents < 0 || b.aggregate_value_any_time_during_year_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if b.days_since_irs_notice > 100_000 {
        return Err(ApiError::BadRequest(
            "days_since_irs_notice looks invalid (>100000)".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6038d::compute(&b)))
}

// ── § 6039 corporate reporting of ISO exercises and ESPP transfers ──
// Mounted at /api/calc/section-6039. Corporate reporting obligation
// under § 6039(a)(1) (Form 3921 — one return per ISO exercise) and
// § 6039(a)(2) (Form 3922 — one return per ESPP first-transfer where
// purchase price < FMV at grant or not fixed/determinable at grant).
// Employee statement deadline § 6039(b): January 31. IRS deadline:
// February 28 paper, March 31 electronic. Mandatory e-filing threshold
// (Treas. Reg. § 301.6011-2 as amended by T.D. 9972 effective filings
// in 2024): 10 or more aggregate information returns. Penalties under
// § 6721 cross-referenced by § 6039(c): $60/form late ≤ 30 days,
// $120/form late 31 days through August 1, $310/form after August 1
// or complete failure (2025 amounts); intentional disregard under
// § 6721(e) carries NO maximum penalty. Companion to § 422 (ISO) and
// § 423 (ESPP) — together they pin employee tax treatment + corporate
// reporting + employee basis tracking for statutory option plans.

async fn section_6039_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6039::Section6039Input>,
) -> Result<Json<traderview_expense::section_6039::Section6039Output>, ApiError> {
    Ok(Json(traderview_expense::section_6039::check(&b)))
}

// ── § 6011 reportable transaction disclosure (Form 8886) ──────────
// Mounted at /api/calc/section-6011. Treas. Reg. § 1.6011-4(b)
// designates five reportable-transaction categories: listed
// transactions (b)(2); confidential transactions (b)(3) with
// $250K corporate / $50K noncorporate fee thresholds; transactions
// with contractual protection (b)(4); loss transactions (b)(5)
// with $2M individual / $10M entity single-year thresholds;
// transactions of interest (b)(6). Failure to disclose triggers
// § 6707A penalty — 75% of tax reduction, floored at $5K
// individual / $10K entity, capped at $100K individual / $200K
// entity for listed transactions. Companion to § 6111 material
// advisor disclosure (Form 8918), § 6112 advisor list maintenance,
// and § 6662A 20%/30% reportable-transaction-understatement penalty.

async fn section_6011_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6011::Section6011Input>,
) -> Result<Json<traderview_expense::section_6011::Section6011Result>, ApiError> {
    if b.fee_paid_to_advisor_cents < 0
        || b.single_year_loss_claimed_cents < 0
        || b.multi_year_loss_total_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6011::compute(&b)))
}

// ── § 6111 material advisor disclosure (Form 8918) ──────────────────
// Mounted at /api/calc/section-6111. Direct sibling to § 6011
// (taxpayer-side Form 8886). § 6111(b)(1) two-prong test: (A)
// provided material aid/assistance/advice for reportable
// transaction AND (B) gross income exceeds threshold ($50K
// natural-person / $250K other under Treas. Reg.
// § 301.6111-3(b)(3)). Filing deadline: last day of month
// following calendar-quarter-end (§ 301.6111-3(e)). § 6707
// penalties: $50K non-listed; greater of $200K or 50% of gross
// income for listed transactions, reduced to $50K for
// unintentional failures per § 6707(b)(1) flush. Statute of
// limitations: 3 years from Form 8918 filing; unlimited if no
// return filed. Companion to § 6112 (advisor list maintenance),
// § 6707A (taxpayer penalty), § 6662A (reportable-transaction-
// understatement accuracy penalty on underlying tax).

async fn section_6111_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6111::Section6111Input>,
) -> Result<Json<traderview_expense::section_6111::Section6111Result>, ApiError> {
    if b.gross_income_from_transaction_cents < 0 {
        return Err(ApiError::BadRequest(
            "gross_income_from_transaction_cents must be non-negative".into(),
        ));
    }
    if b.days_late_after_quarter_end < 0 || b.days_late_after_quarter_end > 100_000 {
        return Err(ApiError::BadRequest(
            "days_late_after_quarter_end out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6111::compute(&b)))
}

// ── § 6112 material advisor list maintenance ────────────────────────
// Mounted at /api/calc/section-6112. Sixth member of the
// disclosure-regime cluster (§ 6011 + § 6111 + § 6707 + § 6707A
// + § 6662A + § 6112). § 6112(a) requires material advisors to
// maintain a list of all persons advised on a reportable
// transaction; § 6112(b)(1)(A) requires production within 20
// BUSINESS DAYS of IRS written request. Treas. Reg.
// § 301.6112-1(b)(2) defines three required list components:
// itemized statement + detailed transaction description + copies
// of documents. § 6708(a) imposes $10,000-per-day penalty for
// each day after the 20-business-day deadline; reasonable cause
// excused on day-by-day basis per § 301.6708-1(c). The only
// per-day-accruing penalty in the disclosure-regime cluster.

async fn section_6112_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6112::Section6112Input>,
) -> Result<Json<traderview_expense::section_6112::Section6112Result>, ApiError> {
    if b.business_days_since_request < 0 || b.business_days_since_request > 100_000 {
        return Err(ApiError::BadRequest(
            "business_days_since_request out of range".into(),
        ));
    }
    if b.days_with_reasonable_cause < 0 || b.days_with_reasonable_cause > 100_000 {
        return Err(ApiError::BadRequest(
            "days_with_reasonable_cause out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6112::compute(&b)))
}

// ── § 6662A reportable-transaction-understatement penalty ──────────
// Mounted at /api/calc/section-6662a. Direct sibling to § 6011
// (taxpayer Form 8886), § 6111 (advisor Form 8918), § 6707
// (advisor penalty), § 6707A (taxpayer disclosure penalty).
// § 6662A taxes the SUBSTANTIVE tax position taken on the return
// when a reportable transaction is involved — not just the
// disclosure failure standalone. § 6662A(a) 20% baseline rate;
// § 6662A(c) 30% enhanced rate when transaction was not
// adequately disclosed under § 6011 regulations. § 6662A(b)(1)
// understatement = (income increase × highest tax rate) +
// credit decrease. § 6664(d) reasonable-cause exception requires
// ALL THREE prongs: (A) adequate disclosure; (B) substantial
// authority; (C) more-likely-than-not belief. § 6662A(e)(2)(A)
// coordination prevents stacking with § 6662 on the same
// understatement.

async fn section_6662a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6662a::Section6662AInput>,
) -> Result<Json<traderview_expense::section_6662a::Section6662AResult>, ApiError> {
    if b.highest_tax_rate_bps > 10_000 || b.highest_tax_rate_bps < -10_000 {
        return Err(ApiError::BadRequest(
            "highest_tax_rate_bps out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6662a::compute(&b)))
}

// ── § 6663 civil fraud penalty (75%) ────────────────────────────────
// Mounted at /api/calc/section-6663. § 6663(a) 75% penalty on portion
// of underpayment attributable to fraud; § 6663(b) burden-shift rule —
// once IRS proves any portion as fraud, ENTIRE underpayment treated as
// fraud unless taxpayer carves out by preponderance; § 6663(c) joint
// return innocent spouse exception — penalty does not apply to spouse
// whose conduct did not contribute (cross-reference § 6015); § 6662(b)(7)
// non-stacking with accuracy-related penalty (mutually exclusive on
// same dollar); § 7454(a) IRS bears CLEAR AND CONVINCING burden of
// proof (heightened standard greater than preponderance, less than
// beyond reasonable doubt); Spies v. United States, 317 U.S. 492 (1943)
// badges of fraud doctrine (9-badge enumeration); § 6501(c)(1) UNLIMITED
// ASED when fraud established; § 6651(f) parallel 75% failure-to-file
// penalty; 11 U.S.C. § 523(a)(1)(C) NONDISCHARGEABLE in personal
// bankruptcy; Spies-Daly doctrine permits parallel civil + criminal
// prosecution under § 7201 / § 7206; IRM 25.1.6 Civil Fraud procedural
// manual. § 6664(c)(1) reasonable-cause defense theoretically applies
// but rarely succeeds. Natural sibling to section_6664 + section_6501 +
// section_6502 + section_6212.

async fn section_6663_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6663::Section6663Input>,
) -> Result<Json<traderview_expense::section_6663::Section6663Result>, ApiError> {
    Ok(Json(traderview_expense::section_6663::check(&b)))
}

// ── § 6664 reasonable cause + good faith defense ────────────────────
// Mounted at /api/calc/section-6664. § 6664(c)(1) general rule —
// no penalty under § 6662 or § 6663 may be imposed for any portion
// of an underpayment where taxpayer shows reasonable cause AND
// good faith. § 6664(c)(2) economic-substance strict-liability
// bar — defense NOT available for transactions lacking economic
// substance under § 7701(o); § 6662(b)(6) + § 6662(i) impose 20%
// (40% non-disclosed) strict-liability penalty with no escape.
// § 6664(d) reportable-transaction heightened defense for § 6662A
// requires ALL THREE elements: (A) adequate disclosure per
// § 6664(d)(3)(A); (B) substantial authority per § 6664(d)(3)(B);
// (C) reasonable belief more-likely-than-not per § 6664(d)(3)(C).
// Treas. Reg. § 1.6664-4 implementing regulation — facts-and-
// circumstances analysis (education, sophistication, business
// experience, advisor reliance with complete + accurate facts).
// Treas. Reg. § 1.6662-3(c)(2) regulation-invalidity adequate-
// disclosure rule. Cross-cutting defense applies to section_6662
// (accuracy) + section_6662a (reportable) + section_6663 (civil
// fraud). Highly relevant for aggressive § 1256 MTM / § 988 / §
// 1202 / § 475(f) trader positions.

async fn section_6664_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6664::Section6664Input>,
) -> Result<Json<traderview_expense::section_6664::Section6664Result>, ApiError> {
    Ok(Json(traderview_expense::section_6664::check(&b)))
}

// ── § 6672 Trust Fund Recovery Penalty (TFRP) ───────────────────────
// Mounted at /api/calc/section-6672. 100% PERSONAL liability on
// responsible persons for unpaid trust fund portion of employment
// taxes (§ 3402 income tax withholding + § 3101 employee FICA; NOT
// § 3111 employer FICA + § 3301 FUTA). Two-prong test: (1)
// responsible person = significant (not exclusive) control over
// finances OR officer/director/designated status OR check-signing /
// payment authority; (2) willfulness = knew taxes due OR reckless
// disregard OR used available funds to pay other creditors (no
// evil intent required). § 6672(b)(1) IRS MUST send preliminary
// notice (Letter 1153 + Form 2751) at least 60 days before
// assessment. § 6672(d) joint and several liability + state-law
// contribution claim among co-responsible persons. 11 U.S.C. §
// 523(a)(7) NONDISCHARGEABLE in personal bankruptcy + § 507(a)(8)(C)
// priority claim. Critical trader-business operational risk for
// LLC / S-corp / C-corp with W-2 employees.

async fn section_6672_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6672::Section6672Input>,
) -> Result<Json<traderview_expense::section_6672::Section6672Result>, ApiError> {
    Ok(Json(traderview_expense::section_6672::check(&b)))
}

// ── § 6694 tax return preparer penalties ────────────────────────────
// Mounted at /api/calc/section-6694. § 6694(a) unreasonable-
// position penalty: greater of $1,000 OR 50% of preparer fee.
// Three trigger paths under § 6694(a)(2): (A) undisclosed +
// no substantial authority; (B) disclosed but no reasonable
// basis; (C) tax shelter / § 6662A reportable transaction
// without more-likely-than-not standard. § 6694(a)(3) reasonable-
// cause + good-faith exception. § 6694(b) willful or reckless
// conduct: greater of $5,000 OR 75% of fee; no reasonable-cause
// exception. § 6694(b)(3) no-stacking — (b) replaces (a) when
// both trigger. Sibling preparer + promoter penalty cluster:
// § 6695 + § 6700 + § 6701. Taxpayer-side companions: § 6662
// + § 6662A + § 6707A.

async fn section_6694_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6694::Section6694Input>,
) -> Result<Json<traderview_expense::section_6694::Section6694Result>, ApiError> {
    if b.preparer_fee_cents < 0 || b.preparer_fee_cents > 1_000_000_000_000 {
        return Err(ApiError::BadRequest(
            "preparer_fee_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6694::compute(&b)))
}

// ── § 6695 preparer information return penalties ───────────────────
// Mounted at /api/calc/section-6695. Direct sibling to § 6694
// (substantive position penalty) — covers PROCEDURAL failures
// by the preparer. Five per-failure subsections (a)-(e) at $60
// each (2025; max $31,500/year per subsection): copy to taxpayer,
// signature, PTIN, retention, info return. Higher-tier
// per-failure $635: § 6695(f) refund check negotiation; § 6695(g)
// due diligence on credits (EITC, CTC/ACTC/ODC, AOTC, HOH) —
// max combined per return = $2,540 ($635 × 4). Treas. Reg.
// § 1.6695-2 requires Form 8867 + worksheet + knowledge
// requirement + 3-year retention. 2025 amounts per Rev. Proc.
// 2024-40 inflation adjustments.

async fn section_6695_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6695::Section6695Input>,
) -> Result<Json<traderview_expense::section_6695::Section6695Result>, ApiError> {
    if b.per_failure_penalty_cents < 0 || b.per_failure_penalty_cents > 1_000_000 {
        return Err(ApiError::BadRequest(
            "per_failure_penalty_cents out of range".into(),
        ));
    }
    if b.annual_max_cap_cents < 0 || b.annual_max_cap_cents > 100_000_000_000 {
        return Err(ApiError::BadRequest(
            "annual_max_cap_cents out of range".into(),
        ));
    }
    if b.higher_tier_penalty_cents < 0 || b.higher_tier_penalty_cents > 10_000_000 {
        return Err(ApiError::BadRequest(
            "higher_tier_penalty_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6695::compute(&b)))
}

// ── §6695A appraiser penalty for substantial/gross valuation ──
// misstatements. Mounted at /api/calc/section-6695a. Added by
// Pension Protection Act of 2006 § 1219 to penalize appraisers
// whose appraisals support substantial or gross valuation
// misstatements. § 6695A(a) — penalty imposed when appraiser knew
// or reasonably should have known appraisal would be used on
// return AND claimed value results in § 6662(e) substantial (≥
// 150%) OR § 6662(g) estate/gift understatement (≤ 65%) OR §
// 6662(h) gross (≥ 200%) valuation misstatement. § 6695A(b) —
// penalty equals LESSER OF (1) greater of 10% of underpayment or
// $1,000 AND (2) 125% of gross income received from appraisal.
// § 6695A(c) good-faith exception — no penalty if appraiser
// establishes value was MORE LIKELY THAN NOT (51% confidence) the
// proper value. Effective dates: general rule after August 17,
// 2006; facade easement special rule after July 25, 2006. Trader-
// relevant for art donations + conservation easements (§ 170(h))
// + facade easements + § 1031 real estate + partnership interest
// valuations + syndicated conservation easement deductions
// (§ 6707A listed transaction crossover) + estate/gift tax
// valuations.
async fn section_6695a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6695a::Section6695AInput>,
) -> Result<Json<traderview_expense::section_6695a::Section6695AResult>, ApiError> {
    Ok(Json(traderview_expense::section_6695a::check(&b)))
}

// ── § 6700 promoter penalties for abusive tax shelter promotion ────
// Mounted at /api/calc/section-6700. Third member of the preparer
// + promoter penalty cluster (after § 6694 + § 6695). Two-prong
// structure: § 6700(a)(1) promoter status (organizes/sells plan
// or arrangement) + § 6700(a)(2)(A) false/fraudulent statement
// with scienter (50% gross income penalty per AJCA 2004), or
// § 6700(a)(2)(B) + § 6700(b)(1) gross valuation overstatement
// exceeding 200% threshold with direct relationship to
// deduction/credit ($1,000 floor or lesser-of-gross-income).
// Penalty applies REGARDLESS of participant reliance or actual
// underreporting. Sibling cluster: § 6694 + § 6695 + § 6701
// (aiding/abetting) + § 7408 (injunction remedy). Effective
// since January 1, 1990; substantially amended by AJCA 2004
// Pub. L. 108-357 § 818.

async fn section_6700_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6700::Section6700Input>,
) -> Result<Json<traderview_expense::section_6700::Section6700Result>, ApiError> {
    if b.gross_income_from_activity_cents < 0
        || b.gross_income_from_activity_cents > 1_000_000_000_000
    {
        return Err(ApiError::BadRequest(
            "gross_income_from_activity_cents out of range".into(),
        ));
    }
    if b.stated_value_cents > 1_000_000_000_000 || b.correct_value_cents > 1_000_000_000_000 {
        return Err(ApiError::BadRequest(
            "stated_value_cents or correct_value_cents out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6700::compute(&b)))
}

// ── § 6701 aiding and abetting understatement of tax liability ─────
// Mounted at /api/calc/section-6701. Fourth and final member of
// the preparer + promoter penalty cluster (§ 6694 + § 6695 +
// § 6700 + § 6701). § 6701 captures the broadest range of
// conduct — any person who aids, assists, procures, or advises
// preparation of a document they KNOW would result in
// understatement of another's tax. Three-element test under
// § 6701(a): (1) aid/assist/procure/advise; (2) material-matter
// knowledge; (3) understatement-knowledge. § 6701(b)(1)
// penalties: $1,000 non-corporate / $10,000 corporate per
// document. § 6701(b)(2) one-per-taxpayer-per-period limit.
// § 6701(f) coordination — § 6701 supersedes § 6694(a)/(b) on
// same document. § 7408 injunction remedy available.

async fn section_6701_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6701::Section6701Input>,
) -> Result<Json<traderview_expense::section_6701::Section6701Result>, ApiError> {
    if b.number_of_documents < 0 || b.number_of_documents > 1_000_000 {
        return Err(ApiError::BadRequest(
            "number_of_documents out of range".into(),
        ));
    }
    if b.number_of_distinct_taxpayers < 0 || b.number_of_distinct_taxpayers > 1_000_000 {
        return Err(ApiError::BadRequest(
            "number_of_distinct_taxpayers out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6701::compute(&b)))
}

// ── §6707A reportable transaction penalty ───────────────────────────
// Mounted at /api/calc/section-6707a. § 6707A(b)(1) base = 75% of
// decrease in tax shown on return as result of transaction. §
// 6707A(b)(2) maximum: listed transaction = $200,000 entity /
// $100,000 natural person; other reportable = $50,000 entity /
// $10,000 natural person. § 6707A(b)(3) minimum: $10,000 entity /
// $5,000 natural person. § 6707A(c)(2) listed transaction =
// substantially similar to transaction specifically identified by
// Secretary as tax avoidance under § 6011. Trader-relevant for
// partnerships caught in syndicated conservation easements + micro-
// captive § 831(b) insurance + monetized installment sales +
// abusive § 6011 reportable transactions. Stacks on top of § 6662A
// accuracy-related penalty. CIC Services v. IRS, 593 U.S. 209 (2021)
// — pre-enforcement challenge to § 6707A reporting requirements NOT
// barred by § 7421(a) Anti-Injunction Act.

// ── §6707 material advisor failure to furnish reportable transaction info ─
// Mounted at /api/calc/section-6707. § 6707(a) — material advisor
// required to file Form 8918 under § 6111 with respect to ANY
// reportable transaction must do so on or before deadline OR file
// return with complete/accurate information; failure = penalty.
// § 6707(b)(1) OTHER REPORTABLE TRANSACTIONS: flat $50,000 base
// penalty. § 6707(b)(2) LISTED TRANSACTIONS: GREATER of $200,000 OR
// 50% of gross income from aid/assistance/advice; 50% rate
// SUBSTITUTED with 75% when failure or act is INTENTIONAL.
// § 6707(c)(1) Commissioner may rescind penalty for non-listed
// transactions if rescission promotes tax compliance and effective
// tax administration. § 6707(c)(2) LISTED TRANSACTIONS NOT
// ELIGIBLE FOR RESCISSION — strict liability. § 6707(c)(3) NO
// JUDICIAL REVIEW of denial of rescission. § 6664(d) reasonable
// cause defense AVAILABLE for non-listed but NOT for listed
// transactions. Enacted by American Jobs Creation Act of 2004 § 815
// (Pub. L. 108-357, October 22, 2004). Trader-critical for material
// advisors on basket option contracts (Notice 2015-73), conservation
// easement syndications (Notice 2017-10), micro-captive insurance
// (Notice 2016-66), § 643 distribution-tier-out trusts, STARS
// foreign-tax-credit shelters. Sibling cluster: § 6011 (Form 8886
// taxpayer disclosure) + § 6111 (Form 8918 material advisor) +
// § 6112 (advisor list maintenance) + § 6707A (taxpayer return
// disclosure penalty) + § 6662A (reportable-transaction
// understatement accuracy penalty) + IRM 20.1.13.

async fn section_6707_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6707::Section6707Input>,
) -> Result<Json<traderview_expense::section_6707::Section6707Result>, ApiError> {
    Ok(Json(traderview_expense::section_6707::check(&b)))
}

async fn section_6707a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6707a::Section6707AInput>,
) -> Result<Json<traderview_expense::section_6707a::Section6707AResult>, ApiError> {
    Ok(Json(traderview_expense::section_6707a::check(&b)))
}

// ── §6713 civil penalty for disclosure or use of information by ─────
// return preparers. Mounted at /api/calc/section-6713. Civil
// companion to § 7216 criminal penalty. § 6713(a) — $250 per
// disclosure/use; $10,000 annual cap. § 6713(b) — § 7216(b)
// exceptions apply: court order/subpoena + preparer in firm +
// assisting firm (e-filing) + bookkeeping + quality/peer review +
// professional liability insurance + tax authority investigation +
// other federal law + taxpayer written consent. § 6713 strict
// liability (no knowing/reckless requirement) vs § 7216 criminal
// requires KNOWING OR RECKLESS conduct (misdemeanor + 1 year + $1,000
// fine + costs). Both penalties may apply concurrently to the same
// disclosure. Trader-relevant for preparer monetizing/sharing trader
// financial data (1099-Bs + § 475(f) M2M + cost basis + § 1091 wash
// sale + § 1256 60/40 + § 988 + § 6038D). Rev. Proc. 2013-14 +
// AICPA sample consent forms. Companion to § 7216 criminal +
// § 7525 FATP + § 6694 preparer substantive + § 6695 preparer
// procedural + Circular 230 § 10.50.
// ── §6708 material advisor failure to maintain list of advisees ─────
// Mounted at /api/calc/section-6708. § 6708(a)(1) — material advisor
// required to maintain § 6112 list FAILS to make list available
// upon WRITTEN IRS request within 20 BUSINESS DAYS = $10,000 PER DAY
// for each day after 20th day, NO STATUTORY MAXIMUM. § 6708(a)(2)
// reasonable cause exception (distinct from § 6664(d)). Treas. Reg.
// § 301.6708-1(c)(3)(ii) extension request requires reason + period
// required + good-faith-effort description. § 6112(b) cross-reference:
// list content (advisee identifiers + transaction ID + timing +
// amount + tax treatment); 30 CALENDAR DAYS preparation period; 7
// YEARS retention; separate list per transaction; one list for
// substantially similar transactions. Coordination: § 6707 penalizes
// failure to FILE Form 8918 disclosure; § 6708 penalizes failure to
// MAINTAIN AND PRODUCE the § 6112 list — TWO INDEPENDENT penalties
// on same material advisor for same transaction. Enacted by American
// Jobs Creation Act of 2004 § 815 (Pub. L. 108-357, October 22,
// 2004). Trader-critical for material advisors on basket option
// contracts (Notice 2015-73), conservation easement syndications
// (Notice 2017-10), micro-captive insurance (Notice 2016-66), § 643
// distribution-tier-out trusts, STARS foreign-tax-credit shelters.

async fn section_6708_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6708::Section6708Input>,
) -> Result<Json<traderview_expense::section_6708::Section6708Result>, ApiError> {
    if b.business_days_since_irs_request > 100_000 {
        return Err(ApiError::BadRequest(
            "business_days_since_irs_request out of range".into(),
        ));
    }
    Ok(Json(traderview_expense::section_6708::check(&b)))
}

async fn section_6713_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6713::Section6713Input>,
) -> Result<Json<traderview_expense::section_6713::Section6713Result>, ApiError> {
    Ok(Json(traderview_expense::section_6713::check(&b)))
}

// ── §6721 failure to file correct information returns ────────────────
// Mounted at /api/calc/section-6721. § 6721(a) failure to file
// correct information return on or before required filing date,
// failure to include all required information, or including
// incorrect information triggers per-return penalty. § 6721(b)(1)
// Tier 1: corrected within 30 days = $50 per return base ($60
// inflation-adjusted 2026 under Rev. Proc. 2025-32); max $500,000
// annual. § 6721(b)(2) Tier 2: corrected after 30 days but by Aug
// 1 = $100 per return base ($130 for 2026); max $1,500,000 annual.
// § 6721(a)(1) Tier 3: not corrected by Aug 1 = $250 per return
// base ($340 for 2026); max $3,000,000 annual base ($4,191,500 for
// 2026 large filer). § 6721(d) small business exception (average
// annual gross receipts ≤ $5,000,000 for 3-year lookback): reduced
// max ($175,000 / $500,000 / $1,000,000 base; $1,397,000 for 2026
// Tier 3 small business). § 6721(e) intentional disregard: § 6721
// (b)/(c)/(d) NOT apply; penalty GREATER OF $500 per return base
// ($680 for 2026) OR 10 % of aggregate amount required to be
// reported; NO MAXIMUM. § 6724 reasonable cause waiver. Treas. Reg.
// § 301.6721-1. § 6722 parallel payee statement penalty. § 6723
// $50 per other reporting failure. IRS IRM 20.1.7. Companion to
// § 6109 (TIN requirements — iter 656) and § 6041 (information
// reporting — iter 620).

async fn section_6721_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6721::Section6721Input>,
) -> Result<Json<traderview_expense::section_6721::Section6721Result>, ApiError> {
    Ok(Json(traderview_expense::section_6721::compute(&b)))
}

// ── §6722 failure to furnish correct payee statements ────────────────
// Mounted at /api/calc/section-6722. § 6722 is the structural parallel
// to § 6721 (iter 658) for the payer-side obligation to furnish a
// correct payee statement (e.g., recipient copy of Form 1099-B, DIV,
// INT, K, NEC, MISC, K-1, W-2) to the payee — distinct from § 6721
// obligation to file with the IRS. Same per-statement amounts
// ($50/$100/$250 base; $60/$130/$340 for 2026 under Rev. Proc.
// 2025-32), same small business exception ($5M gross receipts
// threshold), same intentional disregard rule with NO MAXIMUM. Key
// difference: § 6722(e) intentional disregard uses 10 PERCENT of
// aggregate amount for MOST payee statements OR 5 PERCENT for
// CERTAIN SPECIFIED STATEMENTS (vs § 6721 uniform 10 percent).
// § 6722(a) general rule; § 6722(b)(1) Tier 1 (≤30 days); § 6722(b)(2)
// Tier 2 (≤Aug 1); § 6722(a)(1) Tier 3 (uncorrected); § 6722(d) small
// business exception; § 6722(e) intentional disregard. § 6721/§ 6722
// stacking commonly doubles penalty exposure for late or omitted
// 1099 issuance cycles. § 6724 reasonable cause waiver. Treas. Reg.
// § 301.6722-1. IRS IRM 20.1.7.

async fn section_6722_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6722::Section6722Input>,
) -> Result<Json<traderview_expense::section_6722::Section6722Result>, ApiError> {
    Ok(Json(traderview_expense::section_6722::compute(&b)))
}

// ── §6723 failure to comply with other information reporting reqs ────
// Mounted at /api/calc/section-6723. Catch-all penalty for OTHER
// specified information reporting requirements not covered by § 6721
// (failure to file with IRS — iter 658) or § 6722 (failure to furnish
// payee statement — iter 660). § 6723 general rule: $50 per failure
// with $100,000 annual maximum. NO TIER STRUCTURE, NO INFLATION
// ADJUSTMENT, NO SMALL BUSINESS EXCEPTION, NO SAFE HARBOR. Treas.
// Reg. § 301.6723-1 "specified information reporting requirement"
// includes: (1) TIN-furnishing under § 6109(a); (2) §§ 6038A/6038B
// ancillary filing (Forms 5471, 8865); (3) § 6041A direct-sales
// notice; (4) § 6042(c)(2) corporate shareholder listing; (5)
// magnetic media filing requirements. Common triggers: failure to
// provide correct TIN on Form W-4; failure of payee to provide Form
// W-9 to payor on request; failure to file in magnetic media (10+
// returns); failure to provide TIN for § 6042(c)(2). § 6724
// reasonable cause waiver. Completes the § 6109/§ 6721/§ 6722/
// § 6723 information-return compliance set.

async fn section_6723_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6723::Section6723Input>,
) -> Result<Json<traderview_expense::section_6723::Section6723Result>, ApiError> {
    Ok(Json(traderview_expense::section_6723::compute(&b)))
}

// ── §6724 waiver / definitions / reasonable cause defense ─────────────
// Mounted at /api/calc/section-6724. § 6724 completes the § 6721/
// § 6722/§ 6723/§ 6724 information-return penalty quartet by
// providing the reasonable-cause waiver standard, de minimis failure
// exception, and statutory definitions of "information return"/
// "payee statement"/"specified information reporting requirement"
// referenced by all three preceding penalty sections. § 6724(a)
// reasonable cause waiver: no penalty under §§ 6721, 6722, or 6723
// if failure due to REASONABLE CAUSE and NOT WILLFUL NEGLECT.
// Treas. Reg. § 301.6724-1 two-prong test: (1) significant
// mitigating factors OR events beyond filer's control (impediment);
// AND (2) filer acted in responsible manner before AND after
// failure. Examples of impediments: natural disaster, IRS systems
// failure, death/serious illness, fire/casualty destroying records.
// § 6724(b) payment of penalty on notice and demand same as tax.
// § 6724(c) DE MINIMIS EXCEPTION: § 6721 and § 6722 penalties NOT
// imposed on failures corrected by Aug 1 if number does NOT exceed
// GREATER OF (i) 10 OR (ii) 0.5 % of total returns/statements;
// applies AFTER reasonable cause analysis; NOT available for § 6723.
// § 6724(d) definitions: (d)(1) "information return" Forms 1098,
// 1099, 3921, 3922, 5498, W-2G, 1097, W-2, W-3, 5471, 8865;
// (d)(2) "payee statement"; (d)(3) "specified information reporting
// requirement" for § 6723. IRS Pub 1586 reasonable cause guidance.
// Rev. Proc. 2025-22 TIN solicitation procedures. IRS IRM 20.1.7.

async fn section_6724_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6724::Section6724Input>,
) -> Result<Json<traderview_expense::section_6724::Section6724Result>, ApiError> {
    Ok(Json(traderview_expense::section_6724::compute(&b)))
}

// ── §6851 termination assessment of income tax ──────────────────────
// Mounted at /api/calc/section-6851. Emergency procedure by which IRS
// may TERMINATE a taxpayer's taxable year mid-year when Secretary
// finds taxpayer designing to depart from US, conceal property, or
// jeopardize collection. § 6851(a)(1) — three triggers: (A)
// departing/removing property; (B) concealing self/property; (C)
// other jeopardizing act including corporate liquidation. § 6851(a)
// (2) — tax computed as if terminated period were taxable year AND
// by placing entire tax base on annual basis (annualization). §
// 6851(b) — SNOD within 60 days after LATER of full-year return due
// date or taxpayer's filing date. § 6851(c) — amounts collected
// treated as collected on date of entire-year assessment. § 6851(d)
// cross-references § 7429 review + § 6863 stay + § 6213(a) Tax
// Court petition right. § 6851 vs § 6861 distinction: § 6851
// terminates CURRENT/preceding taxable year BEFORE return due date;
// § 6861 jeopardy-assesses EXISTING DEFICIENCY AFTER return filing.
// § 6851 + § 6863 interaction: 10-day payment requirement unless
// bond filed. Companion to § 6852 + § 6861 + § 6863 + § 7429
// jeopardy/termination constellation.
async fn section_6851_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6851::Section6851Input>,
) -> Result<Json<traderview_expense::section_6851::Section6851Result>, ApiError> {
    Ok(Json(traderview_expense::section_6851::check(&b)))
}

// ── §6861 jeopardy assessment of income, estate, gift, and certain ──
// excise taxes. Mounted at /api/calc/section-6861. § 6861(a) emergency
// authority — if Secretary believes assessment or collection of
// deficiency will be jeopardized by delay, Secretary shall
// immediately assess deficiency together with interest, additional
// amounts, and additions to tax; § 6861(b) 60-day SNOD mailing
// requirement when assessment precedes § 6212(a) SNOD; § 6861(f)
// IMMEDIATE § 6321 lien attachment + § 6331 levy authority (no
// 10-day neglect rule); § 6861(g) abatement if Tax Court determines
// deficiency less than jeopardy assessment. § 7429 review procedures
// — § 7429(a)(1)(A) Chief Counsel for IRS personal written approval
// required; § 7429(a)(1)(B) 5-day written statement requirement;
// § 7429(a)(2) 30-day administrative review window; § 7429(b)(1)
// 90-day judicial review in district court; § 7429(g) burden split
// — Secretary bears burden on reasonableness, taxpayer bears burden
// on amount appropriateness. Companion to § 6201 (assessment
// authority), § 6203 (method of assessment), § 6212 (SNOD), § 6303
// (notice and demand), § 6321 (lien), § 6331 (levy), § 6863 (stay
// of collection), § 7522 (content of notices).
async fn section_6861_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6861::Section6861Input>,
) -> Result<Json<traderview_expense::section_6861::Section6861Result>, ApiError> {
    Ok(Json(traderview_expense::section_6861::check(&b)))
}

// ── §6862 jeopardy assessment of taxes other than income, estate, ───
// gift, and certain excise taxes. Mounted at /api/calc/section-6862.
// § 6862(a) — if Secretary believes collection of any tax (OTHER than
// income tax + estate tax + gift tax + chapter 41/42/43/44 excise
// taxes) will be jeopardized by delay, Secretary shall immediately
// assess tax; whether or not due date for return and payment has
// expired; immediately due and payable. § 6862(b) — § 6331(a) levy
// without regard to 10-day notice requirement. In-scope taxes:
// employment (§ 3402 + § 3111 + § 3301 + § 3406) + excise non-
// chapter-41-44 (alcohol § 5001 + tobacco § 5701 + fuel § 4081 +
// manufacturer § 4221 + communications § 4251 + air transportation
// § 4261) + foreign withholding (§§ 1441-1446 + FATCA chapter 4).
// § 7429 procedural cluster — Chief Counsel personal approval + 5-
// day SPECIFIC FACTS AND REASONS statement (not mere conclusions) +
// 30-day administrative + 90-day judicial review + burden split. §
// 6862 + § 6863(b)(3)(A) sale prohibition SPECIFICALLY applies to
// § 6862(a) (not § 6861). Companion to § 6851 + § 6861 + § 6863 +
// § 7429 jeopardy/termination cluster.
async fn section_6862_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6862::Section6862Input>,
) -> Result<Json<traderview_expense::section_6862::Section6862Result>, ApiError> {
    Ok(Json(traderview_expense::section_6862::check(&b)))
}

// ── §6863 stay of collection of jeopardy assessments ────────────────
// Mounted at /api/calc/section-6863. Procedural pressure-relief valve
// when § 6861 (income/estate/gift jeopardy), § 6862 (other-tax
// jeopardy), § 6851 (income tax termination), or § 6852 (qualified-
// person termination) assessment has been imposed. § 6863(a) bond
// to stay collection — taxpayer may stay collection by filing bond
// in amount equal to amount of stay desired (capped at jeopardy
// amount + interest); § 6863(b)(1) bond filed before § 6213(a)
// petition triggers further condition requiring payment if petition
// not filed within 90-day window (150 days outside US); § 6863(b)(2)
// bond proportionately reduced upon final Tax Court decision if
// determining amount less than jeopardy assessment AND taxpayer
// requests; § 6863(b)(3)(A) § 6862(a) seized property may NOT be
// sold pending § 7429(b) district court judgment; § 6863(b)(3)(B)
// three sale exceptions (taxpayer consent + excessive conservation
// costs + perishable); § 6863(g) court abatement authority when
// assessment unreasonable OR amount inappropriate. Companion to
// § 6851 + § 6852 + § 6861 + § 6862 jeopardy/termination
// assessments; preserves § 6213(a) Tax Court petition right;
// subject to § 7429 review framework.
async fn section_6863_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_6863::Section6863Input>,
) -> Result<Json<traderview_expense::section_6863::Section6863Result>, ApiError> {
    Ok(Json(traderview_expense::section_6863::check(&b)))
}

// ── §336 gain/loss on property distributed in complete liquidation ─
// Mounted at /api/calc/section-336. §336(a) FMV sale treatment of
// distributed property; §336(b) liability ≥ FMV adjustment;
// §336(d)(1) related-party loss disallowance (>50% ownership);
// §336(d)(2) 5-year anti-tax-avoidance built-in loss disallowance;
// §336(d)(3) §332 subsidiary 80%+ parent full nonrecognition.

async fn section_336_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_336::Section336Input>,
) -> Result<Json<traderview_expense::section_336::Section336Result>, ApiError> {
    if b.distributed_property_fmv_dollars < 0
        || b.distributed_property_adjusted_basis_dollars < 0
        || b.built_in_loss_at_contribution_dollars < 0
        || b.liability_amount_on_property_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_336::compute(&b)))
}

// ── §351 corporate formation non-recognition ────────────────────────
// Mounted at /api/calc/section-351. §351(a) non-recognition when
// transferors meet §368(c) 80% voting + 80% nonvoting control test
// immediately after exchange; §351(b)(1) boot gain recognition;
// §351(b)(2) loss never recognized; §351(d) services exclusion;
// §357(a) liabilities not boot; §357(b) tax-avoidance full-boot;
// §357(c) excess-liability-over-basis gain; §358(a) substituted
// stock basis; §362(a) corp carryover basis + gain.

async fn section_351_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_351::Section351Input>,
) -> Result<Json<traderview_expense::section_351::Section351Result>, ApiError> {
    if b.property_adjusted_basis_dollars < 0
        || b.property_fmv_dollars < 0
        || b.stock_fmv_received_dollars < 0
        || b.boot_received_dollars < 0
        || b.liabilities_assumed_by_corp_dollars < 0
        || b.control_group_voting_pct_bp > 10_000
        || b.control_group_nonvoting_pct_bp > 10_000
    {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs and control percentages ≤ 100% (10,000bp) required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_351::compute(&b)))
}

// ── §451(b) AFS conformity / all-events test acceleration ───────────
// Mounted at /api/calc/section-451b. TCJA P.L. 115-97 §13221 added
// §451(b) requiring accrual taxpayers with AFS to recognize income
// no later than when recognized on AFS; §451(b)(3) AFS hierarchy
// (SEC-filed > audited > certified); §451(b) cost offset election
// (TD 9941 eff. 2020-12-21); §451(c) 1-year advance payment deferral.

// ── §45L New Energy Efficient Home Credit (EPAct 2005 + IRA + OBBBA) ─
// Mounted at /api/calc/section-45l. Originally added by Section 1332
// of the Energy Policy Act of 2005 (Public Law 109-58, 119 Stat.
// 594), signed by President Bush on August 8, 2005. Substantially
// expanded by Section 13304 of the Inflation Reduction Act of 2022
// (Public Law 117-169, 136 Stat. 1818), signed by President Biden on
// August 16, 2022; effective for homes acquired after December 31,
// 2022. Single-family / manufactured: $2,500 (ENERGY STAR) or
// $5,000 (DOE Zero Energy Ready Home / Efficient New Homes).
// Multifamily per dwelling unit: $500 base (ENERGY STAR) /
// $1,000 base (ZERH) → $2,500 / $5,000 with prevailing wage under
// § 45L(g). § 45L UNIQUE among IRA credits — requires ONLY
// prevailing wage, NO apprenticeship. Form 8908 (Rev. Dec 2025)
// required. TERMINATED by One Big Beautiful Bill Act of 2025
// (Public Law 119-21, signed July 4, 2025) for homes acquired after
// June 30, 2026 (accelerating original IRA 2032 sunset).
async fn section_45l_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_45l::Section45LInput>,
) -> Result<Json<traderview_expense::section_45l::Section45LResult>, ApiError> {
    Ok(Json(traderview_expense::section_45l::check(&b)))
}

// ── §45W Qualified Commercial Clean Vehicle Credit (IRA + OBBBA) ───
// Mounted at /api/calc/section-45w. Added by Section 13403 of the
// Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat.
// 1818), signed by President Biden on August 16, 2022. Credit equal
// to LESSER of (1) 15 % of vehicle basis (30 % if not powered by
// gas/diesel ICE) or (2) incremental cost. Per-vehicle maximum
// $7,500 (GVWR < 14,000 lbs) / $40,000 (GVWR ≥ 14,000 lbs). C-corps
// + pass-through entities eligible; § 6417 direct pay election for
// tax-exempt entities. Battery capacity ≥ 7 kWh (light-duty) /
// 15 kWh (heavy-duty). Retail Price Equivalent (RPE) for incremental
// cost determination (proposed regs January 14, 2025). Form 8936-A.
// TERMINATED by Section 70503 of One Big Beautiful Bill Act of 2025
// (Public Law 119-21, signed July 4, 2025) for vehicles acquired
// after September 30, 2025 (parallel § 30D new clean vehicle
// termination; accelerates original IRA 2032 sunset by 7+ years).
async fn section_45q_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_45q::Section45QInput>,
) -> Result<Json<traderview_expense::section_45q::Section45QResult>, ApiError> {
    Ok(Json(traderview_expense::section_45q::check(&b)))
}

async fn section_45u_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_45u::Section45uInput>,
) -> Result<Json<traderview_expense::section_45u::Section45uResult>, ApiError> {
    Ok(Json(traderview_expense::section_45u::check(&b)))
}

async fn section_45v_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_45v::Section45VInput>,
) -> Result<Json<traderview_expense::section_45v::Section45VResult>, ApiError> {
    Ok(Json(traderview_expense::section_45v::check(&b)))
}

async fn section_45w_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_45w::Section45WInput>,
) -> Result<Json<traderview_expense::section_45w::Section45WResult>, ApiError> {
    Ok(Json(traderview_expense::section_45w::check(&b)))
}

async fn section_45x_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_45x::Section45XInput>,
) -> Result<Json<traderview_expense::section_45x::Section45XResult>, ApiError> {
    Ok(Json(traderview_expense::section_45x::check(&b)))
}

async fn section_45y_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_45y::Section45YInput>,
) -> Result<Json<traderview_expense::section_45y::Section45YResult>, ApiError> {
    Ok(Json(traderview_expense::section_45y::check(&b)))
}

async fn section_45z_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_45z::Section45ZInput>,
) -> Result<Json<traderview_expense::section_45z::Section45ZResult>, ApiError> {
    Ok(Json(traderview_expense::section_45z::check(&b)))
}

async fn section_47_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_47::Section47Input>,
) -> Result<Json<traderview_expense::section_47::Section47Result>, ApiError> {
    Ok(Json(traderview_expense::section_47::check(&b)))
}

async fn section_48_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_48::Section48Input>,
) -> Result<Json<traderview_expense::section_48::Section48Result>, ApiError> {
    Ok(Json(traderview_expense::section_48::check(&b)))
}

async fn section_48c_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_48c::Section48cInput>,
) -> Result<Json<traderview_expense::section_48c::Section48cResult>, ApiError> {
    Ok(Json(traderview_expense::section_48c::check(&b)))
}

async fn section_48e_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_48e::Section48EInput>,
) -> Result<Json<traderview_expense::section_48e::Section48EResult>, ApiError> {
    Ok(Json(traderview_expense::section_48e::check(&b)))
}

async fn section_51_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_51::Section51Input>,
) -> Result<Json<traderview_expense::section_51::Section51Result>, ApiError> {
    Ok(Json(traderview_expense::section_51::check(&b)))
}

async fn section_451b_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_451b::Section451bInput>,
) -> Result<Json<traderview_expense::section_451b::Section451bResult>, ApiError> {
    if b.afs_revenue_recognized_for_item_dollars < 0
        || b.classic_all_events_test_amount_dollars < 0
        || b.costs_incurred_to_date_for_cost_offset_dollars < 0
        || b.advance_payment_received_current_year_dollars < 0
        || b.afs_advance_payment_recognized_current_year_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_451b::compute(&b)))
}

// ── §451(c) Special Rule for Advance Payments (TCJA 2017) ──────────
// Mounted at /api/calc/section-451c. Added by Section 13221 of the
// Tax Cuts and Jobs Act of 2017 (Public Law 115-97), signed December
// 22, 2017. Accrual-method taxpayer 1-year deferral framework for
// advance payments. § 451(c)(1)(A) full inclusion in year of receipt;
// § 451(c)(1)(B) AFS deferral method election (AFS revenue in year of
// receipt + remainder in succeeding taxable year, max 1-year defer).
// § 451(c)(4)(A) advance payment definition (goods + services + IP
// use + software + gift cards + subscriptions + warranties +
// memberships + property use ancillary to services + loyalty
// programs per Treas. Reg. § 1.451-8(a)(1)). Treas. Reg. § 1.451-8
// proposed September 9, 2019 (84 FR 47175); finalized January 6,
// 2021 (T.D. 9941; 86 FR 810); obsoletes Rev. Proc. 2004-34 +
// Notice 2018-35 for tax years beginning on or after January 1,
// 2021. Rev. Proc. 2021-34 method change procedures. Treas. Reg.
// § 1.451-8(f) accelerates deferred balance on cessation /
// bankruptcy / certain M&A transactions.
async fn section_451c_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_451c::Section451CInput>,
) -> Result<Json<traderview_expense::section_451c::Section451CResult>, ApiError> {
    Ok(Json(traderview_expense::section_451c::check(&b)))
}

// ── MLP K-1 UBTI tracker for IRAs ─────────────────────────────────────
// Mounted at /api/calc/mlp-ubti. Aggregates K-1 line items into
// Unrelated Business Taxable Income, applies §512(b) exclusions for
// passive items, §514 debt-financed inclusion, §512(b)(12) $1k
// specific deduction, and trust-rate tax per §511(b)(2).

async fn mlp_ubti_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::mlp_ubti::MlpUbtiInput>,
) -> Result<Json<traderview_expense::mlp_ubti::MlpUbtiResult>, ApiError> {
    Ok(Json(traderview_expense::mlp_ubti::compute(&b)))
}

// ── §168(g) Alternative Depreciation System (ADS) ────────────────────
// Mounted at /api/calc/section-168g. Pure compute; computes the
// annual ADS deduction for a property at a given year, with a GDS
// comparison so callers can sum up multi-property differences and
// feed into the §163(j) tradeoff analyzer.

// ── § 168 MACRS general depreciation ─────────────────────────────
// Mounted at /api/calc/section-168. Foundational depreciation
// provision under Tax Reform Act of 1986. § 168(a) MACRS formula:
// method × recovery period × convention. § 168(b)(1) 200% DB
// default for 3/5/7/10-year property; § 168(b)(2)(B) 150% DB for
// 15/20-year property; § 168(b)(3) straight-line required for
// residential rental (27.5 years) and nonresidential real (39
// years); § 168(b)(5) elective straight-line. § 168(c) recovery
// periods. § 168(d)(1) half-year default for personal property;
// § 168(d)(2) mid-month for real property; § 168(d)(3) mid-quarter
// triggered when > 40% of personal property placed in last quarter.
// § 168(e)(2)(A) residential rental 80% dwelling-income threshold;
// § 168(e)(2)(B) nonresidential real. § 168(g) ADS straight-line
// over class life. § 168(k) bonus depreciation (separate iter
// module). Sibling cluster: § 168(e)(6), § 168(g), § 168(k),
// § 179 (expensing), § 167 (general depreciation), § 197
// (intangibles amortization).

async fn section_168_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_168::Section168Input>,
) -> Result<Json<traderview_expense::section_168::Section168Output>, ApiError> {
    Ok(Json(traderview_expense::section_168::check(&b)))
}

async fn section_168g_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_168g::Section168gInput>,
) -> Result<Json<traderview_expense::section_168g::Section168gResult>, ApiError> {
    if b.depreciable_basis < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "depreciable_basis must be >= 0".into(),
        ));
    }
    if !(1..=12).contains(&b.placed_in_service_month) {
        return Err(ApiError::BadRequest(
            "placed_in_service_month must be 1..12".into(),
        ));
    }
    Ok(Json(traderview_expense::section_168g::compute(&b)))
}

// ── §168(k) bonus depreciation (post-OBBBA 100% permanent) ──────────
// Mounted at /api/calc/section-168k. § 168(k)(1) additional first-year
// depreciation deduction; § 168(k)(2) qualified property (MACRS ≤ 20
// years + no prior use); § 168(k)(6) pre-OBBBA TCJA phasedown rate
// schedule (100% 2018-2022, 80% 2023, 60% 2024, 40% 2025, 20% 2026,
// 0% 2027+); OBBBA § 70302 permanently restores 100% for property
// acquired AND placed in service after 2025-01-19 — eliminating the
// TCJA phasedown's 2026-2027 step-down years. Transition election
// permits 40% (60% long-production/aircraft) for FYE-after-2025-01-19
// year. Used property eligible if no prior use by taxpayer (TCJA
// 2017 expansion preserved by OBBBA). Distinct from § 179 expensing
// which has dollar caps; § 168(k) has no dollar cap, no income limit,
// no phaseout — works alongside § 179 (§ 179 first, then § 168(k) on
// remainder, then MACRS).

async fn section_168k_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_168k::Section168KInput>,
) -> Result<Json<traderview_expense::section_168k::Section168KResult>, ApiError> {
    if b.property_cost_cents < 0 {
        return Err(ApiError::BadRequest(
            "property_cost_cents must be non-negative".into(),
        ));
    }
    if !(1990..=2100).contains(&b.acquisition_year)
        || !(1..=12).contains(&b.acquisition_month)
        || !(1..=31).contains(&b.acquisition_day)
        || !(1990..=2100).contains(&b.placed_in_service_year)
        || !(1..=12).contains(&b.placed_in_service_month)
        || !(1..=31).contains(&b.placed_in_service_day)
    {
        return Err(ApiError::BadRequest(
            "acquisition + placed_in_service dates must be valid".into(),
        ));
    }
    Ok(Json(traderview_expense::section_168k::compute(&b)))
}

// ── §163(j)(7)(B) electing-RPTB tradeoff analyzer ────────────────────
// Mounted at /api/calc/section-163j-tradeoff. Pure compute; turns
// annual depreciation sacrificed + annual interest disallowed into
// after-tax net benefit using the taxpayer's marginal rate.

async fn section_163j_tradeoff_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_168g::Section163jTradeoffInput>,
) -> Result<Json<traderview_expense::section_168g::Section163jTradeoffResult>, ApiError> {
    if b.marginal_federal_rate < Decimal::ZERO || b.marginal_federal_rate > Decimal::ONE {
        return Err(ApiError::BadRequest(
            "marginal_federal_rate must be 0..1".into(),
        ));
    }
    if b.annual_depreciation_sacrificed < Decimal::ZERO
        || b.annual_interest_disallowed_under_163j < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "depreciation / interest amounts must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_168g::analyze_tradeoff(&b)))
}

// ── §164 SALT deduction cap (TCJA + OBBBA expansion) ─────────────────
// Mounted at /api/calc/section-164. TCJA §164(b)(6) capped SALT at
// $10K ($5K MFS) for 2018-2024. OBBBA §70413 (eff. 2025-01-01)
// temporarily expanded the cap to $40K ($20K MFS) for 2025 with annual
// 1% compounded growth through 2029; 30% high-income phaseout above
// $500K MAGI ($250K MFS) with threshold also growing 1%/yr; statutory
// $10K ($5K MFS) floor — phaseout never drives the cap below the floor.
// Automatic sunset to TCJA $10K cap in 2030. Out of scope: pass-through-
// entity (PTET) workaround state-level elections.

async fn section_164_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_164::Section164Input>,
) -> Result<Json<traderview_expense::section_164::Section164Result>, ApiError> {
    if b.salt_paid_cents < 0 {
        return Err(ApiError::BadRequest(
            "salt_paid_cents must be non-negative".into(),
        ));
    }
    if !(1900..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest("year must be in [1900, 2100]".into()));
    }
    Ok(Json(traderview_expense::section_164::compute(&b)))
}

// ── §165(h) personal casualty loss deduction ─────────────────────────
// Mounted at /api/calc/section-165h. Three time-windowed regimes:
// pre-TCJA (≤2017) any sudden-unexpected-identifiable event qualifies
// subject to $100 per-event + 10% AGI floors; TCJA window 2018-2025
// §165(h)(5) suspends personal casualty losses EXCEPT for federally
// declared disasters (FEMA); OBBBA §70423 (eff. tax years after
// 2025-12-31) makes TCJA suspension PERMANENT for non-disaster losses
// AND EXPANDS qualifying events to include state-declared disasters
// (natural catastrophes hurricane/tornado/storm/earthquake or any
// fire/flood/explosion the state deems severe). Per-event $500 floor
// + no 10% AGI floor for congressionally designated qualified-disaster
// losses. Loss = lesser of (basis, FMV decline) − insurance, capped at 0.

async fn section_165h_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_165h::Section165HInput>,
) -> Result<Json<traderview_expense::section_165h::Section165HResult>, ApiError> {
    if b.adjusted_basis_cents < 0
        || b.decline_in_fmv_cents < 0
        || b.insurance_reimbursement_cents < 0
        || b.agi_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1900..=2100).contains(&b.year) {
        return Err(ApiError::BadRequest("year must be in [1900, 2100]".into()));
    }
    Ok(Json(traderview_expense::section_165h::compute(&b)))
}

// ── §25C Energy Efficient Home Improvement Credit (OBBBA term 12/31/25)
// Mounted at /api/calc/section-25c. IRA 2022 30% credit for energy-
// efficiency improvements with layered cap structure totaling up to
// $3,200/year. General $1,200 envelope (§25C(b)(1)) with sub-caps:
// $600 windows+skylights (§25C(b)(2)(A)) + $250/door / $500 aggregate
// doors (§25C(b)(2)(B)) + $600/item energy property (§25C(b)(2)(C)) +
// $150 home energy audit (§25C(b)(2)(D)) + insulation no sub-cap.
// Heat pump SEPARATE $2,000 cap (§25C(b)(3)) above and beyond the
// general $1,200. NONREFUNDABLE no carryforward (distinct from §25D).
// OBBBA §70425 ACCELERATED termination to property PLACED IN SERVICE
// after 2025-12-31 — wiping out IRA's 2032 sunset.

async fn section_25c_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_25c::Section25CInput>,
) -> Result<Json<traderview_expense::section_25c::Section25CResult>, ApiError> {
    if b.windows_skylights_cost_cents < 0
        || b.doors_cost_cents < 0
        || b.insulation_cost_cents < 0
        || b.energy_property_cost_cents < 0
        || b.heat_pump_cost_cents < 0
        || b.home_energy_audit_cost_cents < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1990..=2100).contains(&b.placed_in_service_year)
        || !(1..=12).contains(&b.placed_in_service_month)
        || !(1..=31).contains(&b.placed_in_service_day)
    {
        return Err(ApiError::BadRequest(
            "placed_in_service date must be a valid year/month/day".into(),
        ));
    }
    Ok(Json(traderview_expense::section_25c::compute(&b)))
}

// ── §25D Residential Clean Energy Credit (OBBBA termination 12/31/25) ─
// Mounted at /api/calc/section-25d. IRA 2022 30% credit for qualifying
// clean energy property installed at taxpayer's residence (primary +
// secondary homes; NOT pure rentals). Qualifying property §25D(d):
// solar electric + solar water heater + fuel cell + small wind +
// geothermal heat pump + battery storage with capacity ≥ 3 kWh under
// §25D(d)(6) added 2023 (biomass terminated end of 2022 moved to §25C).
// Nonrefundable with §25D(c) INDEFINITE carryforward to succeeding
// years. OBBBA §70426 ACCELERATED termination to expenditures made
// after December 31, 2025 — wiping out 2026-2034 step-down years
// originally scheduled under IRA.

async fn section_25d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_25d::Section25DInput>,
) -> Result<Json<traderview_expense::section_25d::Section25DResult>, ApiError> {
    if b.qualifying_property_cost_cents < 0 || b.current_year_tax_liability_cents < 0 {
        return Err(ApiError::BadRequest(
            "non-negative cents inputs required".into(),
        ));
    }
    if !(1990..=2100).contains(&b.expenditure_year)
        || !(1..=12).contains(&b.expenditure_month)
        || !(1..=31).contains(&b.expenditure_day)
    {
        return Err(ApiError::BadRequest(
            "expenditure date must be a valid year/month/day".into(),
        ));
    }
    Ok(Json(traderview_expense::section_25d::compute(&b)))
}

// ── §25E Previously-Owned Clean Vehicle Credit (IRA 2022 / OBBBA) ──
// Mounted at /api/calc/section-25e. Added by Section 13402 of the
// Inflation Reduction Act of 2022 (Public Law 117-169, 136 Stat.
// 1818), signed by President Biden on August 16, 2022. Credit equal
// to LESSER of $4,000 or 30 % of sale price for previously-owned
// clean vehicles meeting statutory requirements (sale price
// ≤ $25,000; model year ≥ 2 years older than purchase year;
// battery capacity ≥ 7 kWh; GVWR < 14,000 lbs; purchased from
// licensed dealer; first transfer since August 16, 2022 to non-
// original-owner; modified AGI ≤ filing-status threshold of $75K
// single / $112.5K HoH / $150K MFJ; once-per-3-years limit). Credit
// transfer election to dealer for vehicles acquired after December
// 31, 2023 under § 25E(f). TERMINATED by One Big Beautiful Bill Act
// of 2025 (Public Law 119-21, 139 Stat. 72, signed July 4, 2025) —
// available ONLY for vehicles acquired before October 1, 2025.
async fn section_25e_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_25e::Section25EInput>,
) -> Result<Json<traderview_expense::section_25e::Section25EResult>, ApiError> {
    Ok(Json(traderview_expense::section_25e::check(&b)))
}

// ── §30D Clean Vehicle Credit (post-OBBBA termination 2025-09-30) ────
// Mounted at /api/calc/section-30d. IRA 2022 bifurcated $7,500 credit:
// $3,750 critical-minerals (§30D(e)(1)) + $3,750 battery-components
// (§30D(e)(2)). MSRP caps §30D(f)(11): $55K cars / $80K SUVs+trucks+vans.
// MAGI hard-cutoff §30D(f)(10): $150K Single/MFS + $225K HoH + $300K MFJ.
// OBBBA §70424 (eff. 2025-09-30) TERMINATED §30D for vehicles acquired
// after September 30, 2025 — accelerating the IRA's 2032 sunset by 7+
// years. IRS binding-contract carve-out: written binding contract +
// payment ≤ 2025-09-30 preserves credit even if vehicle placed in
// service later. Out of scope: §25E used clean vehicle credit (also
// terminated 2025-09-30); §30D(g) transfer-to-dealer election.

// ── §30C Alternative Fuel Vehicle Refueling Property Credit ────────
// Mounted at /api/calc/section-30c. Originally added by Section
// 1342 of the Energy Policy Act of 2005 (Public Law 109-58, 119
// Stat. 594), signed by President Bush on August 8, 2005.
// Substantially expanded by Section 13404 of the Inflation
// Reduction Act of 2022 (Public Law 117-169, 136 Stat. 1818),
// signed by President Biden on August 16, 2022. 6 % base rate /
// 30 % with PWA (prevailing wage + apprenticeship) via 5×
// multiplier under § 30C(g)(1) for business property (or BOC
// exception). $100,000 per-item cap for depreciable business
// property under § 30C(e)(6); $1,000 for residential property.
// Eligible census tract requirement under § 30C(c) (low-income
// community per § 45D(e) NMTC definition OR non-urban per
// Treasury guidance under Notice 2024-20). Treas. Reg. § 1.30C-1
// final regulations published September 19, 2024. TERMINATED by
// One Big Beautiful Bill Act of 2025 (Public Law 119-21, 139
// Stat. 72, signed July 4, 2025) for property placed in service
// after June 30, 2026 (accelerating original IRA 2032 sunset by
// more than 6 years).
async fn section_30c_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_30c::Section30CInput>,
) -> Result<Json<traderview_expense::section_30c::Section30CResult>, ApiError> {
    Ok(Json(traderview_expense::section_30c::check(&b)))
}

async fn section_30d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_30d::Section30DInput>,
) -> Result<Json<traderview_expense::section_30d::Section30DResult>, ApiError> {
    if b.msrp_cents < 0 || b.modified_agi_cents < 0 {
        return Err(ApiError::BadRequest(
            "msrp_cents and modified_agi_cents must be non-negative".into(),
        ));
    }
    if !(1990..=2100).contains(&b.acquisition_year)
        || !(1..=12).contains(&b.acquisition_month)
        || !(1..=31).contains(&b.acquisition_day)
    {
        return Err(ApiError::BadRequest(
            "acquisition date must be a valid year/month/day".into(),
        ));
    }
    Ok(Json(traderview_expense::section_30d::compute(&b)))
}

// ── §1296 PFIC mark-to-market election ───────────────────────────────
// Mounted at /api/calc/section-1296. Pure compute; annual mark of
// marketable PFIC stock as ordinary income, with loss limited to
// prior unreversed inclusions.

async fn section_1296_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1296::Section1296Input>,
) -> Result<Json<traderview_expense::section_1296::Section1296Result>, ApiError> {
    if b.adjusted_basis_year_start < Decimal::ZERO
        || b.fair_market_value_year_end < Decimal::ZERO
        || b.prior_unreversed_inclusions < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "all dollar inputs must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1296::compute(&b)))
}

// ── §1341 claim-of-right doctrine — lesser-of deduction vs credit ────
// Mounted at /api/calc/section-1341. Codifies the claim-of-right
// doctrine: when income reported in a prior year is restored in a later
// year and exceeds the §1341(a)(3) $3,000 threshold, taxpayer chooses
// the LESSER of Method A (§1341(a)(4) deduction) and Method B (§1341(a)(5)
// refundable credit = current-year tax without relief minus prior-year
// tax decrease that would have resulted had the now-repaid income been
// excluded). §1341(b)(2) mandates the lesser; no election required.

async fn section_1341_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_1341::Section1341Input>,
) -> Result<Json<traderview_expense::section_1341::Section1341Result>, ApiError> {
    if b.repayment_amount_cents < 0 {
        return Err(ApiError::BadRequest(
            "repayment_amount_cents must be non-negative".into(),
        ));
    }
    Ok(Json(traderview_expense::section_1341::compute(&b)))
}

// ── §988 forex transaction character ─────────────────────────────────
// Mounted at /api/calc/section-988. Pure compute; classifies forex /
// FX-denominated debt / forex derivatives into ordinary / capital /
// §1256 60/40 / personal-use-excluded character based on transaction
// kind + §988(a)(1)(B) capital election + §988(c)(1)(D)(i) kick-out
// election + personal-use flag.

async fn section_988_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_988::Section988Input>,
) -> Result<Json<traderview_expense::section_988::Section988Result>, ApiError> {
    Ok(Json(traderview_expense::section_988::compute(&b)))
}

// ── §267 related-party loss disallowance ─────────────────────────────
// Mounted at /api/calc/section-267. Pure compute; §267(a)(1) disallows
// the loss when seller and buyer are related per the §267(b) 10-category
// list. §267(d) preserves the disallowance for buyer's subsequent gain
// reduction (capped at buyer's actual gain).

async fn section_267_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_267::Section267Input>,
) -> Result<Json<traderview_expense::section_267::Section267Result>, ApiError> {
    if b.realized_loss < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "realized_loss must be >= 0 (pass loss as positive number)".into(),
        ));
    }
    if b.buyer_purchase_price < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "buyer_purchase_price must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_267::compute(&b)))
}

// ── § 269 Acquisitions Made to Evade or Avoid Income Tax ─────────────
// Mounted at /api/calc/section-269 (iter 536). Pure compute. § 269 gives
// Treasury authority to DISALLOW any deduction, credit, or other
// allowance when (1) a person acquires CONTROL of a corporation OR a
// corporation acquires the property of another corporation with
// carry-over basis, AND (2) the principal purpose of the acquisition was
// to evade or avoid federal income tax by securing a benefit the
// acquirer would not otherwise enjoy. Primary anti-loss-trafficking
// weapon operating ALONGSIDE the mechanical § 382 NOL cap (post-1986);
// § 269 reaches non-NOL benefits (general business credits, accelerated
// depreciation, foreign tax credits, charitable contribution carryovers)
// that § 382 does not capture.
//
// § 269(a) CONTROL THRESHOLD: at least 50% combined voting power OR
// at least 50% of total value of all classes of stock.
//
// § 269(a)(1): stock acquisition triggering control analysis.
// § 269(a)(2): asset acquisition with carry-over basis transaction.
// § 269(b): § 332 parent-subsidiary liquidation within 2-year window
// after acquisition.
// § 269(c): rebuttable presumption when purchase price substantially
// disproportionate to asset value (excluding tax-benefit value). Strong
// business-purpose evidence (≥ 75% in our framework) rebuts.
//
// § 382(l)(5) BANKRUPTCY COORDINATION: Treas. Reg. § 1.269-3(d)
// per se presumption that ownership change is for tax-avoidance
// principal purpose UNLESS corporation carries on more than an
// insignificant amount of active trade or business during AND after
// the title 11 case.
//
// Eight-mode severity ladder: NotApplicable,
// NoControlAcquisitionSection269Inapplicable,
// BonaFideBusinessPurposeNoDisallowance,
// Section382L5BankruptcyActiveBusinessMaintainedNoDisallowance,
// Section269ATaxBenefitDisallowanceApplied,
// Section269BNolCarryoverDisallowanceApplied,
// Section269CPriceDisproportionatePresumptionApplied,
// Section382L5PerSePresumptionDisallowanceApplied.
//
// Coordinates with § 382 NOL annual cap, § 383 general-business-credit
// cap, § 384 built-in-gain limit (5-year recognition period), § 332
// parent-sub liquidation, § 368 reorganization framework, § 7874
// anti-inversion.

async fn section_269_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_269::Section269AcquisitionsToEvadeTaxInput>,
) -> Result<Json<traderview_expense::section_269::Section269AcquisitionsToEvadeTaxOutput>, ApiError>
{
    Ok(Json(traderview_expense::section_269::check(&b)))
}

// ── § 269A Personal Service Corporation Tax-Avoidance Allocation ─────
// Mounted at /api/calc/section-269a (iter 544). Pure compute. § 269A
// gives Treasury authority to ALLOCATE income, deductions, credits,
// exclusions, and other allowances between a personal service
// corporation (PSC) and its employee-owner(s) when the corporation is
// formed or used to avoid or evade federal income tax. Targets the
// classic "incorporate-and-zero-out" tactic where an individual
// provides services to a single client, forms a wholly-owned PSC to
// receive the income, then zeroes out the PSC's taxable income via
// fringe benefits, deferred compensation, and qualified retirement
// contributions that the individual could not claim if reporting the
// income directly on Schedule C.
//
// § 269A(a) STATUTORY TEST — both conditions must be satisfied:
//   (1) SUBSTANTIALLY ALL of the PSC's services performed for ONE
//       other corporation, partnership, or entity;
//   (2) The principal purpose of forming or availing of the PSC is to
//       AVOID or EVADE federal income tax by securing the benefit of
//       any expense, deduction, credit, exclusion, or other allowance
//       the employee-owner could not otherwise claim.
//
// § 269A(b)(1) PSC DEFINITION: corporation whose principal activity is
// performance of personal services AND those services are substantially
// performed by employee-owners.
//
// § 269A(b)(2) EMPLOYEE-OWNER DEFINITION: any employee who owns, on any
// day during the taxable year, MORE THAN 10% of the outstanding stock
// of the PSC.
//
// § 269A(b)(3) RELATED-PERSONS aggregation: all related persons within
// the meaning of § 144(a)(3) are treated as one entity for the "one
// entity" test under § 269A(a)(1). Effective for taxable years
// beginning after December 31, 1982.
//
// Six-mode severity ladder: NotApplicable,
// NotPersonalServiceCorporationSection269AInapplicable,
// MultipleClientsFailsOneEntityTestNoAllocation,
// BonaFideBusinessPurposeNoAllocation,
// NoEmployeeOwnerThresholdSatisfiedNoAllocation,
// Section269AAllocationApplied.
//
// Coordinates with § 269 (iter 536 — broader anti-tax-avoidance
// acquisition disallowance), § 482 (related-party transfer pricing),
// § 162 (reasonable compensation reallocation), § 444 (PSC fiscal-year
// election restrictions), § 280H (PSC accumulated earnings + minimum
// distribution rules), § 199A (Specified Service Trade or Business
// limitation parallel).

async fn section_269a_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_269a::Section269APersonalServiceCorpAllocationInput>,
) -> Result<
    Json<traderview_expense::section_269a::Section269APersonalServiceCorpAllocationOutput>,
    ApiError,
> {
    Ok(Json(traderview_expense::section_269a::check(&b)))
}

// ── § 274 Meals, Entertainment, Gift, Travel deduction limits ────────
// Mounted at /api/calc/section-274. Pure compute; § 274(a)
// ENTERTAINMENT fully disallowed post-TCJA 2017 § 13304 (Pub. L.
// 115-97); § 274(k) BUSINESS MEALS 50% subject to (1) not lavish/
// extravagant; (2) taxpayer or employee present; § 274(n) general
// 50% limit; § 274(o) PER SE entertainment facilities (country
// clubs, sporting events, golf, yachts, etc.) no deduction;
// § 274(b) GIFT limit $25 per recipient per year with $4
// promotional-item exception; § 274(d) substantiation (amount +
// time/place + business purpose + business relationship) with
// complete-denial on failure (Sanford v. Commissioner, 50 T.C.
// 823 (1968); COHAN RULE rejected); § 274(h) FOREIGN CONVENTION
// 4-part reasonableness test; § 274(m) LUXURY WATER TRAVEL 2×
// federal per diem daily cap (~$1,140/day 2026); temporary 100%
// restaurant meal exception 2021-2022 expired (Pub. L. 116-260
// § 210); OBBBA 2026 § 274(o) updates (Pub. L. 119-21 § 70202)
// modified employer-convenience meal deduction.

async fn section_274_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_274::Section274Input>,
) -> Result<Json<traderview_expense::section_274::Section274Result>, ApiError> {
    Ok(Json(traderview_expense::section_274::check(&b)))
}

// ── § 279 Interest on Corporate Acquisition Indebtedness ─────────────
// Mounted at /api/calc/section-279 (iter 534). Pure compute. § 279(a)
// disallows the corporate interest deduction for amounts exceeding
// $5,000,000 per tax year on "corporate acquisition indebtedness" —
// debt incurred to acquire stock or assets of another corporation that
// satisfies all four § 279(b) statutory criteria. Targets debt-financed
// leveraged-buyout (LBO) transactions and "junk-bond" subordinated
// convertible takeover financings of the 1970s-1980s. Disallowance is
// PERMANENT (not capitalized, no carryforward).
//
// § 279(b) FOUR-PRONG DEFINITION (all four must be satisfied):
//   (1) ISSUED AFTER Oct 9, 1969 to provide consideration for acquisition
//       of stock or assets of another corporation;
//   (2) SUBORDINATED to claims of trade creditors generally OR expressly
//       subordinated to a substantial amount of unsecured indebtedness;
//   (3) CONVERTIBLE into stock of the issuer OR issued as part of an
//       investment unit with stock-purchase warrants or options;
//   (4) Issuer DEBT:EQUITY > 2:1 OR projected EBITDA fails 3:1
//       INTEREST-COVERAGE test for three-taxable-year averaging period.
//
// § 279(d) PRE-1969 EXEMPTION: obligations issued on or before Oct 9,
// 1969 grandfathered. § 279(g) SMALL-ISSUER SAFE HARBOR: applies only to
// issuer with total acquisition interest above $5M. § 279(h)
// COMPENSATION-STOCK SAFE HARBOR: stock acquired as compensation
// (employee stock-purchase plan, § 83 transfer for services) NOT an
// acquisition for § 279.
//
// Eight-mode severity ladder: NotApplicable,
// PreOctober1969GrandfatheredNoDisallowance,
// Section279HCompensationStockSafeHarborNoDisallowance,
// NotSubordinatedFailsSection279DefinitionNoDisallowance,
// NeitherConvertibleNorWarrantPackagedFailsSection279DefinitionNoDisallowance,
// DebtEquityAndCoverageSafeHarborPassesNoDisallowance,
// SmallIssuerUnderFiveMillionThresholdNoDisallowance,
// Section279AInterestDisallowanceApplied.
//
// Coordinates with § 163(j) business-interest limit (separate cap on
// remaining interest), § 385 debt-equity classification (whether the
// instrument is debt vs equity), § 269 acquisitions-to-evade-tax
// disallowance, § 7874 anti-inversion.

async fn section_279_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_279::Section279CorporateAcquisitionIndebtednessInput>,
) -> Result<
    Json<traderview_expense::section_279::Section279CorporateAcquisitionIndebtednessOutput>,
    ApiError,
> {
    Ok(Json(traderview_expense::section_279::check(&b)))
}

// ── §469(c)(7) Real Estate Professional Status qualification ─────────
// Mounted at /api/calc/reps-qualification. Pure compute; checks the
// 750-hour test, the >50%-of-personal-services test, and material
// participation. Returns whether REPS is met (flips rental losses from
// passive to non-passive in §469 PAL).

async fn reps_qualification_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::reps_qualification::RepsInput>,
) -> Result<Json<traderview_expense::reps_qualification::RepsResult>, ApiError> {
    if b.other_personal_services_hours < Decimal::ZERO {
        return Err(ApiError::BadRequest(
            "other_personal_services_hours must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::reps_qualification::compute(&b)))
}

// ── §121 home sale exclusion ──────────────────────────────────────────
// Mounted at /api/calc/section-121. Pure compute; up to $250k single /
// $500k MFJ of gain on principal-residence sale excluded with the 2-of-5
// year ownership + use tests, §121(b)(4) hardship pro-rata, §121(b)(5)
// non-qualified-use proportional reduction, and §121(d)(6) post-1997
// depreciation recapture.

async fn section_121_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_121::Section121Input>,
) -> Result<Json<traderview_expense::section_121::Section121Result>, ApiError> {
    if b.sale_price < Decimal::ZERO
        || b.selling_costs < Decimal::ZERO
        || b.depreciation_post_1997 < Decimal::ZERO
    {
        return Err(ApiError::BadRequest(
            "sale_price / selling_costs / depreciation_post_1997 must be >= 0".into(),
        ));
    }
    if b.non_qualified_use_days_post_2008 > b.total_ownership_days_post_2008
        && b.total_ownership_days_post_2008 > 0
    {
        return Err(ApiError::BadRequest(
            "non_qualified_use_days_post_2008 must be <= total_ownership_days_post_2008".into(),
        ));
    }
    Ok(Json(traderview_expense::section_121::compute(&b)))
}

// ── §121(d) divorce special rules ───────────────────────────────────
// Mounted at /api/calc/section-121d. §121(d)(2) holding-period
// tacking from §1041(a) transferor spouse; §121(d)(3)(A) use
// attribution via former-spouse occupation under divorce or
// separation instrument. Lets divorced spouse meet 2-year ownership
// and 2-year use tests even after years of non-occupation.

async fn section_121d_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_121d::Section121dInput>,
) -> Result<Json<traderview_expense::section_121d::Section121dResult>, ApiError> {
    if b.gain_realized_on_sale_dollars < 0 {
        return Err(ApiError::BadRequest(
            "gain_realized_on_sale_dollars must be >= 0".into(),
        ));
    }
    Ok(Json(traderview_expense::section_121d::compute(&b)))
}

// ── §132 fringe benefits exclusion ───────────────────────────────────
// Mounted at /api/calc/section-132. §132(a) 8 fringe-benefit
// exclusion categories: (1) no-additional-cost + (2) qualified
// employee discount (services 20% / goods gross-profit %) + (3)
// working condition + (4) de minimis + (5) qualified transportation
// (§132(f) 2026 $340/mo each for parking and transit) + (6)
// qualified moving PERMANENTLY SUSPENDED by OBBBA 2025 P.L. 119-21
// except armed forces / intelligence community + (7) retirement
// planning + (8) military base realignment.

async fn section_132_route(
    _u: AuthUser,
    Json(b): Json<traderview_expense::section_132::Section132Input>,
) -> Result<Json<traderview_expense::section_132::Section132Result>, ApiError> {
    if b.fringe_value_dollars < 0
        || b.parking_monthly_cap_dollars < 0
        || b.transit_monthly_cap_dollars < 0
    {
        return Err(ApiError::BadRequest(
            "non-negative dollar inputs required".into(),
        ));
    }
    Ok(Json(traderview_expense::section_132::compute(&b)))
}

#[derive(Deserialize)]
struct TaxRebalanceBody {
    holdings: Vec<traderview_core::tax_aware_rebalance::Holding>,
    strategy: traderview_core::tax_lot_optimizer::LotStrategy,
    tax_rate: Decimal,
    band: Decimal,
}

/// Tax-aware rebalance: trade toward target weights while choosing sell
/// lots to minimize realized gain (or harvest losses), reporting the gain
/// and estimated tax the rebalance triggers.
async fn tax_aware_rebalance_route(
    _u: AuthUser,
    Json(b): Json<TaxRebalanceBody>,
) -> Json<traderview_core::tax_aware_rebalance::Plan> {
    Json(traderview_core::tax_aware_rebalance::plan(
        &b.holdings,
        b.strategy,
        b.tax_rate,
        b.band,
    ))
}

/// Financial order of operations: allocate a month's available savings
/// down the priority ladder (match, debt, emergency, tax-advantaged…),
/// remainder to taxable.
async fn savings_waterfall_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::savings_waterfall::WaterfallInput>,
) -> Json<traderview_core::savings_waterfall::WaterfallPlan> {
    Json(traderview_core::savings_waterfall::plan(&b))
}

/// House hacking: net rental income against the carrying cost to show
/// what you actually pay to live in a small multi-unit you partly rent.
async fn house_hacking_route(
    _u: AuthUser,
    Json(b): Json<traderview_db::house_hacking::HouseHackInput>,
) -> Json<traderview_db::house_hacking::HouseHackResult> {
    Json(traderview_db::house_hacking::compute(&b))
}

/// BRRRR: net the cash-out refinance against total cash invested to show
/// the cash left in the deal, post-refi cash flow, and cash-on-cash.
async fn brrrr_route(
    _u: AuthUser,
    Json(b): Json<traderview_db::brrrr::BrrrrInput>,
) -> Json<traderview_db::brrrr::BrrrrResult> {
    Json(traderview_db::brrrr::compute(&b))
}

/// 401(k) per-paycheck maximizer: the even deferral to hit the annual
/// limit and the match forfeited by front-loading without a true-up.
async fn paycheck_401k_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::paycheck_401k::Paycheck401kInput>,
) -> Json<traderview_core::paycheck_401k::Paycheck401kResult> {
    Json(traderview_core::paycheck_401k::compute(&b))
}

/// Guyton-Klinger guardrails: one year's dynamic-withdrawal decision —
/// inflation raise, capital-preservation cut, or prosperity raise.
async fn guyton_klinger_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::guyton_klinger::GuardrailInput>,
) -> Json<traderview_core::guyton_klinger::GuardrailDecision> {
    Json(traderview_core::guyton_klinger::decide(&b))
}

/// IRMAA: the 2026 Medicare Part B/D income-surcharge tier for a MAGI and
/// filing status, with the surcharge amounts and headroom to the next cliff.
async fn irmaa_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::irmaa::IrmaaInput>,
) -> Json<traderview_core::irmaa::IrmaaResult> {
    Json(traderview_core::irmaa::compute(&b))
}

/// Break-even / CVP: contribution margin, break-even units + revenue,
/// target-profit volume, and margin of safety for a small business.
async fn break_even_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::break_even::BreakEvenInput>,
) -> Json<traderview_core::break_even::BreakEvenResult> {
    Json(traderview_core::break_even::analyze(&b))
}

/// Lease generator: assembles a residential lease agreement and computes
/// the term, prorated first month, and move-in total from the terms.
async fn lease_generator_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::lease_generator::LeaseInput>,
) -> Json<traderview_core::lease_generator::LeaseDocument> {
    Json(traderview_core::lease_generator::generate(&b))
}

/// Invoice generator: builds a business invoice from line items, computing
/// the subtotal, discount, tax on the discounted subtotal, total, and due date.
async fn invoice_generator_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::invoice_generator::InvoiceInput>,
) -> Json<traderview_core::invoice_generator::InvoiceDocument> {
    Json(traderview_core::invoice_generator::generate(&b))
}

/// Landlord notice: generates a Michigan SCAO landlord-tenant notice
/// (DC 100a nonpayment demand / DC 100c notice to quit) with the comply-by date.
async fn landlord_notice_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::landlord_notice::NoticeInput>,
) -> Json<traderview_core::landlord_notice::NoticeDocument> {
    Json(traderview_core::landlord_notice::generate(&b))
}

/// Security-deposit itemization: deductions, balance returned or owed, and the
/// statutory return deadline, assembled into a printable statement.
async fn security_deposit_itemization_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::security_deposit_itemization::DepositInput>,
) -> Json<traderview_core::security_deposit_itemization::DepositStatement> {
    Json(traderview_core::security_deposit_itemization::generate(&b))
}

/// Promissory note: amortizes the loan (monthly payment, total interest,
/// maturity) and assembles the note's operative clauses.
async fn promissory_note_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::promissory_note::NoteInput>,
) -> Json<traderview_core::promissory_note::PromissoryNote> {
    Json(traderview_core::promissory_note::generate(&b))
}

/// Rent increase notice: new rent (percent or flat), change amount/percent,
/// and the effective date from the service date + notice period.
async fn rent_increase_notice_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::rent_increase_notice::RentIncreaseInput>,
) -> Json<traderview_core::rent_increase_notice::RentIncreaseNotice> {
    Json(traderview_core::rent_increase_notice::generate(&b))
}

/// Demand for payment: totals principal + interest + fees and computes the
/// pay-by date, assembled into a formal demand letter.
async fn demand_for_payment_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::demand_for_payment::DemandInput>,
) -> Json<traderview_core::demand_for_payment::DemandLetter> {
    Json(traderview_core::demand_for_payment::generate(&b))
}

/// Lease renewal / extension: new end date and rent change, assembled into a
/// renewal agreement.
async fn lease_renewal_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::lease_renewal::RenewalInput>,
) -> Json<traderview_core::lease_renewal::RenewalAgreement> {
    Json(traderview_core::lease_renewal::generate(&b))
}

/// Bill of sale: sales tax + total consideration, assembled into a transfer
/// document with condition and title clauses.
async fn bill_of_sale_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::bill_of_sale::BillOfSaleInput>,
) -> Json<traderview_core::bill_of_sale::BillOfSale> {
    Json(traderview_core::bill_of_sale::generate(&b))
}

/// Rent receipt: records a payment against rent due, computing any balance or
/// overpayment credit and the paid-in-full flag.
async fn rent_receipt_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::rent_receipt::ReceiptInput>,
) -> Json<traderview_core::rent_receipt::RentReceipt> {
    Json(traderview_core::rent_receipt::generate(&b))
}

/// Independent contractor agreement: fee (fixed or hourly with estimate) and
/// the 1099 / IP / confidentiality clauses.
async fn contractor_agreement_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::contractor_agreement::ContractorInput>,
) -> Json<traderview_core::contractor_agreement::ContractorAgreement> {
    Json(traderview_core::contractor_agreement::generate(&b))
}

/// Notice of entry: earliest lawful entry date from the service date + notice
/// period, with the purpose and time window.
async fn notice_of_entry_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::notice_of_entry::EntryInput>,
) -> Json<traderview_core::notice_of_entry::EntryNotice> {
    Json(traderview_core::notice_of_entry::generate(&b))
}

/// Lease termination letter: move-out date from the service date + notice
/// period, with wording for the landlord or the tenant.
async fn lease_termination_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::lease_termination::TerminationInput>,
) -> Json<traderview_core::lease_termination::TerminationLetter> {
    Json(traderview_core::lease_termination::generate(&b))
}

/// Non-disclosure agreement: expiration from effective date + term, one-way or
/// mutual, with the operative confidentiality clauses.
async fn nda_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::nda_agreement::NdaInput>,
) -> Json<traderview_core::nda_agreement::NdaAgreement> {
    Json(traderview_core::nda_agreement::generate(&b))
}

/// Pet addendum: up-front charges total and the new monthly rent with pet rent,
/// assembled into a lease addendum.
async fn pet_addendum_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::pet_addendum::PetAddendumInput>,
) -> Json<traderview_core::pet_addendum::PetAddendum> {
    Json(traderview_core::pet_addendum::generate(&b))
}

/// Move-in/out inspection checklist: area-by-area condition record with a
/// needs-attention count.
async fn inspection_checklist_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::inspection_checklist::ChecklistInput>,
) -> Json<traderview_core::inspection_checklist::InspectionChecklist> {
    Json(traderview_core::inspection_checklist::generate(&b))
}

/// Estimate / quote: shared line-item math plus a valid-until date.
async fn estimate_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::estimate::EstimateInput>,
) -> Json<traderview_core::estimate::EstimateDocument> {
    Json(traderview_core::estimate::generate(&b))
}

/// Purchase order: shared line-item math + shipping + expected delivery date.
async fn purchase_order_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::purchase_order::PurchaseOrderInput>,
) -> Json<traderview_core::purchase_order::PurchaseOrder> {
    Json(traderview_core::purchase_order::generate(&b))
}

/// Sublease agreement: end date, rent markup/discount vs the master lease.
async fn sublease_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::sublease_agreement::SubleaseInput>,
) -> Json<traderview_core::sublease_agreement::SubleaseAgreement> {
    Json(traderview_core::sublease_agreement::generate(&b))
}

/// Roommate agreement: rent and deposit split (weighted) across roommates.
async fn roommate_agreement_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::roommate_agreement::RoommateInput>,
) -> Json<traderview_core::roommate_agreement::RoommateAgreement> {
    Json(traderview_core::roommate_agreement::generate(&b))
}

/// Commercial (NNN) lease: base + triple-net charges, gross monthly rent, end
/// date.
async fn commercial_lease_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::commercial_lease::CommercialLeaseInput>,
) -> Json<traderview_core::commercial_lease::CommercialLease> {
    Json(traderview_core::commercial_lease::generate(&b))
}

/// Lease guaranty / co-signer: total rent over the term + guaranty clauses.
async fn guaranty_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::guaranty_agreement::GuarantyInput>,
) -> Json<traderview_core::guaranty_agreement::GuarantyAgreement> {
    Json(traderview_core::guaranty_agreement::generate(&b))
}

/// Equipment rental: rental total + deposit + return date.
async fn equipment_rental_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::equipment_rental::EquipmentRentalInput>,
) -> Json<traderview_core::equipment_rental::EquipmentRental> {
    Json(traderview_core::equipment_rental::generate(&b))
}

/// LLC operating agreement: member ownership-% split from capital + clauses.
async fn llc_operating_agreement_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::llc_operating_agreement::LlcInput>,
) -> Json<traderview_core::llc_operating_agreement::LlcOperatingAgreement> {
    Json(traderview_core::llc_operating_agreement::generate(&b))
}

/// Lead-based paint disclosure: pre-1978 applicability + lessor disclosure.
async fn lead_paint_disclosure_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::lead_paint_disclosure::LeadPaintInput>,
) -> Json<traderview_core::lead_paint_disclosure::LeadPaintDisclosure> {
    Json(traderview_core::lead_paint_disclosure::generate(&b))
}

/// Employment offer letter: per-paycheck breakdown + offer clauses.
async fn offer_letter_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::offer_letter::OfferInput>,
) -> Json<traderview_core::offer_letter::OfferLetter> {
    Json(traderview_core::offer_letter::generate(&b))
}

/// Severance agreement: severance pay + total payout, release clauses.
async fn severance_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::severance_agreement::SeveranceInput>,
) -> Json<traderview_core::severance_agreement::SeveranceAgreement> {
    Json(traderview_core::severance_agreement::generate(&b))
}

/// Sales commission agreement: projected commission + draw and clauses.
async fn commission_agreement_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::commission_agreement::CommissionInput>,
) -> Json<traderview_core::commission_agreement::CommissionAgreement> {
    Json(traderview_core::commission_agreement::generate(&b))
}

/// PTO accrual policy: annual accrual (hours/days) + policy clauses.
async fn pto_policy_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::pto_policy::PtoPolicyInput>,
) -> Json<traderview_core::pto_policy::PtoPolicy> {
    Json(traderview_core::pto_policy::generate(&b))
}

/// Expense reimbursement: itemized + mileage → total reimbursement.
async fn expense_reimbursement_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::expense_reimbursement::ReimbursementInput>,
) -> Json<traderview_core::expense_reimbursement::ReimbursementRequest> {
    Json(traderview_core::expense_reimbursement::generate(&b))
}

/// Timesheet: regular + overtime hours → gross pay.
async fn timesheet_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::timesheet::TimesheetInput>,
) -> Json<traderview_core::timesheet::Timesheet> {
    Json(traderview_core::timesheet::generate(&b))
}

/// Pay stub: auto FICA + withholding + deductions → net pay.
async fn pay_stub_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::pay_stub::PayStubInput>,
) -> Json<traderview_core::pay_stub::PayStub> {
    Json(traderview_core::pay_stub::generate(&b))
}

/// Rental application: income-to-rent qualification + application clauses.
async fn rental_application_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::rental_application::RentalApplicationInput>,
) -> Json<traderview_core::rental_application::RentalApplication> {
    Json(traderview_core::rental_application::generate(&b))
}

/// Cease and desist letter: comply-by deadline + demand clauses.
async fn cease_desist_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::cease_desist::CeaseDesistInput>,
) -> Json<traderview_core::cease_desist::CeaseDesistLetter> {
    Json(traderview_core::cease_desist::generate(&b))
}

/// Employee disciplinary write-up: level → consequence + escalation.
async fn employee_writeup_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::employee_writeup::WriteupInput>,
) -> Json<traderview_core::employee_writeup::EmployeeWriteup> {
    Json(traderview_core::employee_writeup::generate(&b))
}

/// Real-estate purchase agreement: down payment + loan + earnest % and clauses.
async fn purchase_agreement_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::purchase_agreement::PurchaseAgreementInput>,
) -> Json<traderview_core::purchase_agreement::PurchaseAgreement> {
    Json(traderview_core::purchase_agreement::generate(&b))
}

/// Seller's closing statement: commission + payoff + tax proration → net.
async fn closing_statement_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::closing_statement::ClosingInput>,
) -> Json<traderview_core::closing_statement::ClosingStatement> {
    Json(traderview_core::closing_statement::generate(&b))
}

/// Lease-option (rent-to-own): rent credits + net price at exercise.
async fn lease_option_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::lease_option::LeaseOptionInput>,
) -> Json<traderview_core::lease_option::LeaseOption> {
    Json(traderview_core::lease_option::generate(&b))
}

/// Land contract (contract for deed): amortized installment sale + clauses.
async fn land_contract_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::land_contract::LandContractInput>,
) -> Json<traderview_core::land_contract::LandContract> {
    Json(traderview_core::land_contract::generate(&b))
}

/// Assignment of lease: months remaining + obligation transferred.
async fn lease_assignment_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::lease_assignment::LeaseAssignmentInput>,
) -> Json<traderview_core::lease_assignment::LeaseAssignment> {
    Json(traderview_core::lease_assignment::generate(&b))
}

/// Seller's property disclosure: known-defect statement with counts.
async fn seller_disclosure_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::seller_disclosure::SellerDisclosureInput>,
) -> Json<traderview_core::seller_disclosure::SellerDisclosure> {
    Json(traderview_core::seller_disclosure::generate(&b))
}

/// Earnest money receipt: deposit %, balance at closing, escrow terms.
async fn earnest_money_receipt_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::earnest_money_receipt::EarnestMoneyInput>,
) -> Json<traderview_core::earnest_money_receipt::EarnestMoneyReceipt> {
    Json(traderview_core::earnest_money_receipt::generate(&b))
}

/// Stock subscription: investment + resulting ownership % and clauses.
async fn stock_subscription_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::stock_subscription::StockSubscriptionInput>,
) -> Json<traderview_core::stock_subscription::StockSubscription> {
    Json(traderview_core::stock_subscription::generate(&b))
}

/// Convertible note: accrued interest + discount/cap conversion price + shares.
async fn convertible_note_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::convertible_note::ConvertibleNoteInput>,
) -> Json<traderview_core::convertible_note::ConvertibleNote> {
    Json(traderview_core::convertible_note::generate(&b))
}

/// Cap table: per-holder ownership % of fully-diluted shares.
async fn cap_table_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::cap_table::CapTableInput>,
) -> Json<traderview_core::cap_table::CapTable> {
    Json(traderview_core::cap_table::generate(&b))
}

/// Board resolution: quorum + vote tally → passed/failed.
async fn board_resolution_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::board_resolution::BoardResolutionInput>,
) -> Json<traderview_core::board_resolution::BoardResolution> {
    Json(traderview_core::board_resolution::generate(&b))
}

/// SAFE: discount/cap conversion + shares (no interest, no maturity).
async fn safe_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::safe_agreement::SafeInput>,
) -> Json<traderview_core::safe_agreement::SafeAgreement> {
    Json(traderview_core::safe_agreement::generate(&b))
}

/// Stock option grant (ISO/NSO): cliff+monthly vesting, exercise spread, ISO AMT.
async fn option_grant_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::option_grant::OptionGrantInput>,
) -> Json<traderview_core::option_grant::OptionGrant> {
    Json(traderview_core::option_grant::generate(&b))
}

/// RSU grant: cliff+monthly vesting, vest value (ordinary income), sell-to-cover.
async fn rsu_grant_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::rsu_grant::RsuGrantInput>,
) -> Json<traderview_core::rsu_grant::RsuGrant> {
    Json(traderview_core::rsu_grant::generate(&b))
}

/// Statement of account: aggregate outstanding invoices into aging buckets.
async fn statement_of_account_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::statement_of_account::StatementInput>,
) -> Json<traderview_core::statement_of_account::StatementOfAccount> {
    Json(traderview_core::statement_of_account::generate(&b))
}

/// Warrant agreement: cashless net exercise, intrinsic value, loan coverage.
async fn warrant_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::warrant_agreement::WarrantInput>,
) -> Json<traderview_core::warrant_agreement::WarrantAgreement> {
    Json(traderview_core::warrant_agreement::generate(&b))
}

/// Earnout agreement: contingent consideration (rate × excess over threshold, capped).
async fn earnout_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::earnout_agreement::EarnoutInput>,
) -> Json<traderview_core::earnout_agreement::EarnoutAgreement> {
    Json(traderview_core::earnout_agreement::generate(&b))
}

/// Royalty/license agreement: earned vs minimum-guarantee royalty, advance recoupment.
async fn royalty_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::royalty_agreement::RoyaltyInput>,
) -> Json<traderview_core::royalty_agreement::RoyaltyAgreement> {
    Json(traderview_core::royalty_agreement::generate(&b))
}

/// CAM reconciliation: pro-rata-by-sqft share of actual CAM vs estimates paid.
async fn cam_reconciliation_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::cam_reconciliation::CamInput>,
) -> Json<traderview_core::cam_reconciliation::CamReconciliation> {
    Json(traderview_core::cam_reconciliation::generate(&b))
}

/// Percentage rent: base + rate × sales over the (natural or stated) breakpoint.
async fn percentage_rent_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::percentage_rent::PercentageRentInput>,
) -> Json<traderview_core::percentage_rent::PercentageRentStatement> {
    Json(traderview_core::percentage_rent::generate(&b))
}

/// CPI rent adjustment: index-ratio escalation bounded by a floor/ceiling collar.
async fn cpi_rent_adjustment_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::cpi_rent_adjustment::CpiRentInput>,
) -> Json<traderview_core::cpi_rent_adjustment::CpiRentAdjustment> {
    Json(traderview_core::cpi_rent_adjustment::generate(&b))
}

/// Security-deposit interest: simple or annually-compounded interest over the tenancy.
async fn deposit_interest_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::deposit_interest::DepositInterestInput>,
) -> Json<traderview_core::deposit_interest::DepositInterest> {
    Json(traderview_core::deposit_interest::generate(&b))
}

/// Lease buyout: PV of remaining rent + concessions + fee − reletting recovery.
async fn lease_buyout_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::lease_buyout::LeaseBuyoutInput>,
) -> Json<traderview_core::lease_buyout::LeaseBuyout> {
    Json(traderview_core::lease_buyout::generate(&b))
}

/// Operating-expense escalation: base-year stop + occupancy gross-up, pro-rata share.
async fn opex_escalation_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::opex_escalation::OpexEscalationInput>,
) -> Json<traderview_core::opex_escalation::OpexEscalation> {
    Json(traderview_core::opex_escalation::generate(&b))
}

/// Leasing commission: tiered rate per lease-year on an escalating rent stream.
async fn leasing_commission_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::leasing_commission::LeasingCommissionInput>,
) -> Json<traderview_core::leasing_commission::LeasingCommission> {
    Json(traderview_core::leasing_commission::generate(&b))
}

/// Holdover rent: penalty-multiple daily rate over the holdover days, premium.
async fn holdover_rent_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::holdover_rent::HoldoverInput>,
) -> Json<traderview_core::holdover_rent::HoldoverRent> {
    Json(traderview_core::holdover_rent::generate(&b))
}

/// Prorated rent: partial-month rent on actual calendar-month days.
async fn prorated_rent_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::prorated_rent::ProratedRentInput>,
) -> Json<traderview_core::prorated_rent::ProratedRent> {
    Json(traderview_core::prorated_rent::generate(&b))
}

/// TI allowance reconciliation: per-sqft allowance vs actual cost → overage/unused.
async fn ti_allowance_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::ti_allowance::TiAllowanceInput>,
) -> Json<traderview_core::ti_allowance::TiAllowance> {
    Json(traderview_core::ti_allowance::generate(&b))
}

/// 1099-NEC summary: total contractor pay, reporting threshold, backup withholding.
async fn contractor_1099_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::contractor_1099::Contractor1099Input>,
) -> Json<traderview_core::contractor_1099::Contractor1099> {
    Json(traderview_core::contractor_1099::generate(&b))
}

/// PTO balance: earned − used, capped, with payout value.
async fn pto_balance_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::pto_balance::PtoBalanceInput>,
) -> Json<traderview_core::pto_balance::PtoBalance> {
    Json(traderview_core::pto_balance::generate(&b))
}

/// Wage garnishment: CCPA cap — lesser of % of disposable or amount above 30× min wage.
async fn wage_garnishment_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::wage_garnishment::GarnishmentInput>,
) -> Json<traderview_core::wage_garnishment::WageGarnishment> {
    Json(traderview_core::wage_garnishment::generate(&b))
}

/// Final paycheck: waiting-time penalty (daily wage × days late, capped).
async fn final_paycheck_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::final_paycheck::FinalPaycheckInput>,
) -> Json<traderview_core::final_paycheck::FinalPaycheck> {
    Json(traderview_core::final_paycheck::generate(&b))
}

/// Break premium: one hour of pay per missed meal/rest break day (§226.7).
async fn break_premium_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::break_premium::BreakPremiumInput>,
) -> Json<traderview_core::break_premium::BreakPremium> {
    Json(traderview_core::break_premium::generate(&b))
}

/// Reporting-time pay: clamped half-shift minimum guarantee, additional owed.
async fn reporting_time_pay_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::reporting_time_pay::ReportingTimeInput>,
) -> Json<traderview_core::reporting_time_pay::ReportingTimePay> {
    Json(traderview_core::reporting_time_pay::generate(&b))
}

/// Split-shift premium: one hour at minimum wage, offset by earnings above minimum.
async fn split_shift_premium_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::split_shift_premium::SplitShiftInput>,
) -> Json<traderview_core::split_shift_premium::SplitShiftPremium> {
    Json(traderview_core::split_shift_premium::generate(&b))
}

/// Workers' comp premium: payroll × class rate per code, summed, × experience mod.
async fn workers_comp_premium_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::workers_comp_premium::WorkersCompInput>,
) -> Json<traderview_core::workers_comp_premium::WorkersCompPremium> {
    Json(traderview_core::workers_comp_premium::generate(&b))
}

/// Allowance for doubtful accounts: aging-method bad-debt reserve, net realizable AR.
async fn allowance_doubtful_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::allowance_doubtful::AllowanceInput>,
) -> Json<traderview_core::allowance_doubtful::AllowanceDoubtful> {
    Json(traderview_core::allowance_doubtful::generate(&b))
}

/// Depreciation schedule: straight-line or DDB period-by-period book value.
async fn depreciation_schedule_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::depreciation_schedule::DepreciationScheduleInput>,
) -> Json<traderview_core::depreciation_schedule::DepreciationSchedule> {
    Json(traderview_core::depreciation_schedule::generate(&b))
}

/// Fixed-asset disposal: gain/loss with §1245 ordinary recapture and §1231 split.
async fn asset_disposal_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::asset_disposal::AssetDisposalInput>,
) -> Json<traderview_core::asset_disposal::AssetDisposal> {
    Json(traderview_core::asset_disposal::generate(&b))
}

/// Cash flow statement (indirect method): derive CFO/CFI/CFF and net change.
async fn cash_flow_statement_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::cash_flow_statement::CashFlowInput>,
) -> Json<traderview_core::cash_flow_statement::CashFlowStatement> {
    Json(traderview_core::cash_flow_statement::generate(&b))
}

/// Income statement: revenue → gross profit → EBIT → pre-tax → net income, margins.
async fn income_statement_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::income_statement::IncomeStatementInput>,
) -> Json<traderview_core::income_statement::IncomeStatement> {
    Json(traderview_core::income_statement::generate(&b))
}

/// Bank reconciliation: adjusted bank vs adjusted book balance, difference.
async fn bank_reconciliation_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::bank_reconciliation::BankReconciliationInput>,
) -> Json<traderview_core::bank_reconciliation::BankReconciliation> {
    Json(traderview_core::bank_reconciliation::generate(&b))
}

/// Trial balance: list accounts by debit/credit, verify totals are equal.
async fn trial_balance_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::trial_balance::TrialBalanceInput>,
) -> Json<traderview_core::trial_balance::TrialBalance> {
    Json(traderview_core::trial_balance::generate(&b))
}

/// Fix-and-flip: the 70% rule max-allowable-offer plus the full deal P&L
/// (holding, financing, selling costs → net profit, cash-on-cash, annualized).
async fn fix_and_flip_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::fix_and_flip::FlipInput>,
) -> Json<traderview_core::fix_and_flip::FlipResult> {
    Json(traderview_core::fix_and_flip::analyze(&b))
}

/// Cash conversion cycle: DSO + DIO − DPO and the operating cycle, the days
/// a dollar is tied up between paying suppliers and collecting from customers.
async fn cash_conversion_cycle_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::cash_conversion_cycle::CccInput>,
) -> Json<traderview_core::cash_conversion_cycle::CccResult> {
    Json(traderview_core::cash_conversion_cycle::analyze(&b))
}

/// Profit First: splits real revenue across Profit / Owner's Pay / Tax /
/// OpEx by the target allocation band (or custom percentages).
async fn profit_first_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::profit_first::ProfitFirstInput>,
) -> Json<traderview_core::profit_first::ProfitFirstResult> {
    Json(traderview_core::profit_first::analyze(&b))
}

/// Markup vs margin: from cost + one of {price, markup%, margin%}, returns
/// price, profit, and both markup% (of cost) and margin% (of price).
async fn markup_margin_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::markup_margin::MarkupInput>,
) -> Json<traderview_core::markup_margin::MarkupResult> {
    Json(traderview_core::markup_margin::analyze(&b))
}

/// Economic order quantity: the Wilson EOQ, order cadence, ordering/holding
/// cost split, and the reorder point for an inventory item.
async fn inventory_eoq_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::inventory_eoq::EoqInput>,
) -> Json<traderview_core::inventory_eoq::EoqResult> {
    Json(traderview_core::inventory_eoq::analyze(&b))
}

/// Rent vs sell: end-of-horizon wealth from selling now and investing the
/// proceeds vs holding the rental (appreciation + reinvested cash flow).
async fn rent_vs_sell_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::rent_vs_sell::RentVsSellInput>,
) -> Json<traderview_core::rent_vs_sell::RentVsSellResult> {
    Json(traderview_core::rent_vs_sell::analyze(&b))
}

/// Depreciation recapture: splits a rental's gain into unrecaptured § 1250
/// gain (max 25%) and LTCG, with the tax on each and the effective rate.
async fn depreciation_recapture_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::depreciation_recapture::RecaptureInput>,
) -> Json<traderview_core::depreciation_recapture::RecaptureResult> {
    Json(traderview_core::depreciation_recapture::analyze(&b))
}

/// § 1031 like-kind exchange: boot (cash + net mortgage relief), recognized
/// vs deferred gain, tax now, and the replacement property's carryover basis.
async fn like_kind_exchange_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::like_kind_exchange::ExchangeInput>,
) -> Json<traderview_core::like_kind_exchange::ExchangeResult> {
    Json(traderview_core::like_kind_exchange::analyze(&b))
}

/// True cost of hire: fully-loaded W-2 cost (payroll tax + benefits + match
/// + workers' comp + overhead) vs a 1099 contractor, with burden + hourly.
async fn cost_of_hire_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::cost_of_hire::CostOfHireInput>,
) -> Json<traderview_core::cost_of_hire::CostOfHireResult> {
    Json(traderview_core::cost_of_hire::analyze(&b))
}

/// Invoice factoring: advance, fee, reserve, net proceeds, and the
/// annualized effective APR of selling a receivable.
async fn invoice_factoring_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::invoice_factoring::FactoringInput>,
) -> Json<traderview_core::invoice_factoring::FactoringResult> {
    Json(traderview_core::invoice_factoring::analyze(&b))
}

/// LTV:CAC — customer lifetime value, acquisition cost, the ratio (3:1 rule),
/// and CAC payback months.
async fn ltv_cac_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::ltv_cac::LtvCacInput>,
) -> Json<traderview_core::ltv_cac::LtvCacResult> {
    Json(traderview_core::ltv_cac::analyze(&b))
}

/// Burn rate & runway: gross/net burn, how many months the cash lasts, and
/// months to break-even given revenue growth.
async fn burn_rate_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::burn_rate::BurnInput>,
) -> Json<traderview_core::burn_rate::BurnResult> {
    Json(traderview_core::burn_rate::analyze(&b))
}

/// QLAC: caps the premium at the SECURE 2.0 limit and computes the RMD
/// reduction from excluding it from the account's RMD base.
async fn qlac_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::qlac::QlacInput>,
) -> Json<traderview_core::qlac::QlacResult> {
    Json(traderview_core::qlac::analyze(&b))
}

/// Spousal IRA: per-spouse contribution limits (with catch-up) and whether
/// the couple's combined earned income covers both contributions.
async fn spousal_ira_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::spousal_ira::SpousalIraInput>,
) -> Json<traderview_core::spousal_ira::SpousalIraResult> {
    Json(traderview_core::spousal_ira::analyze(&b))
}

/// Pension survivor election: cost of survivor protection (single-life − J&S),
/// the survivor's continued benefit, and the pension-max comparison.
async fn pension_survivor_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::pension_survivor::PensionSurvivorInput>,
) -> Json<traderview_core::pension_survivor::PensionSurvivorResult> {
    Json(traderview_core::pension_survivor::analyze(&b))
}

/// Social Security PIA: the progressive 90/32/15 bend-point formula turning
/// AIME into the full-retirement-age benefit, with the tier breakdown.
async fn ss_pia_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::social_security_pia::PiaInput>,
) -> Json<traderview_core::social_security_pia::PiaResult> {
    Json(traderview_core::social_security_pia::analyze(&b))
}

/// HSA triple-tax: HSA vs a taxable account over a horizon — the dollar
/// value of deductible-in / tax-free-growth / tax-free-out treatment.
async fn hsa_triple_tax_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::hsa_triple_tax::HsaInput>,
) -> Json<traderview_core::hsa_triple_tax::HsaResult> {
    Json(traderview_core::hsa_triple_tax::analyze(&b))
}

/// Age-based allocation: the rule-of-N equity glidepath (equity % = N − age),
/// the dollar split, and the glidepath at 10-year steps.
async fn age_allocation_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::age_based_allocation::AllocationInput>,
) -> Json<traderview_core::age_based_allocation::AllocationResult> {
    Json(traderview_core::age_based_allocation::analyze(&b))
}

/// Roth bracket-fill: the conversion that tops off a tax bracket (headroom to
/// the ceiling, capped at the balance) and the tax it triggers.
async fn roth_bracket_fill_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::roth_bracket_fill::BracketFillInput>,
) -> Json<traderview_core::roth_bracket_fill::BracketFillResult> {
    Json(traderview_core::roth_bracket_fill::analyze(&b))
}

/// Mortgage points: the bought-down rate, points cost, payment savings, and
/// the months to break even on buying discount points.
async fn mortgage_points_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::mortgage_points::PointsInput>,
) -> Json<traderview_core::mortgage_points::PointsResult> {
    Json(traderview_core::mortgage_points::analyze(&b))
}

/// APR ↔ APY: nominal vs effective annual rate at a compounding frequency,
/// plus the continuous-compounding ceiling.
async fn apr_apy_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::apr_apy::AprApyInput>,
) -> Json<traderview_core::apr_apy::AprApyResult> {
    Json(traderview_core::apr_apy::analyze(&b))
}

/// Blended debt rate: the balance-weighted average APR across debts, total
/// monthly interest, and a consolidation-loan comparison.
async fn blended_debt_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::blended_debt::BlendedDebtInput>,
) -> Json<traderview_core::blended_debt::BlendedDebtResult> {
    Json(traderview_core::blended_debt::analyze(&b))
}

/// Dividend coverage: payout ratio, earnings coverage, retention, and an
/// optional FCF payout — whether a stock's dividend is sustainable.
async fn dividend_coverage_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::dividend_coverage::DividendInput>,
) -> Json<traderview_core::dividend_coverage::DividendResult> {
    Json(traderview_core::dividend_coverage::analyze(&b))
}

/// SPIA: the guaranteed monthly income a single-premium immediate annuity
/// pays from a lump sum, with the payout rate and total received.
async fn spia_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::spia::SpiaInput>,
) -> Json<traderview_core::spia::SpiaResult> {
    Json(traderview_core::spia::analyze(&b))
}

/// Debt yield & loan sizing: the commercial-RE lender ratios (debt yield,
/// LTV, LTC) and the max loan each allows, with the binding constraint.
async fn debt_yield_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::debt_yield::DebtYieldInput>,
) -> Json<traderview_core::debt_yield::DebtYieldResult> {
    Json(traderview_core::debt_yield::analyze(&b))
}

/// Price-to-rent: home price ÷ annual rent and the gross rental yield, with
/// the buy/borderline/rent verdict for a market.
async fn price_to_rent_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::price_to_rent::PriceToRentInput>,
) -> Json<traderview_core::price_to_rent::PriceToRentResult> {
    Json(traderview_core::price_to_rent::analyze(&b))
}

/// Years to FI: the FI number (expenses / SWR) and the years for current
/// savings + the annual surplus to reach it at an expected return.
async fn years_to_fi_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::years_to_fi::YearsToFiInput>,
) -> Json<traderview_core::years_to_fi::YearsToFiResult> {
    Json(traderview_core::years_to_fi::analyze(&b))
}

/// Gross rent multiplier: price ÷ gross annual rent, with vacancy/credit loss
/// and other income rolled into effective gross income and an effective GRM.
async fn grm_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::gross_rent_multiplier::GrmInput>,
) -> Json<traderview_core::gross_rent_multiplier::GrmResult> {
    Json(traderview_core::gross_rent_multiplier::analyze(&b))
}

/// Seller financing: the carryback note — amortized payment, balloon balance,
/// and the seller's interest income.
async fn seller_financing_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::seller_financing::SellerFinancingInput>,
) -> Json<traderview_core::seller_financing::SellerFinancingResult> {
    Json(traderview_core::seller_financing::analyze(&b))
}

/// Expense ratio drag: the dollars a fund's expense ratio costs over a
/// horizon vs a zero-fee fund (gross vs net-return future value).
async fn expense_drag_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::expense_ratio_drag::ExpenseDragInput>,
) -> Json<traderview_core::expense_ratio_drag::ExpenseDragResult> {
    Json(traderview_core::expense_ratio_drag::analyze(&b))
}

/// Car lease payment: the depreciation + finance fee breakdown from cap cost,
/// residual, and money factor, with the equivalent APR.
async fn lease_payment_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::lease_money_factor::LeaseInput>,
) -> Json<traderview_core::lease_money_factor::LeaseResult> {
    Json(traderview_core::lease_money_factor::analyze(&b))
}

/// Real return: the Fisher inflation-adjusted return (exact + shortcut),
/// after-tax real return, and the principal's purchasing power over time.
async fn real_return_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::real_return::RealReturnInput>,
) -> Json<traderview_core::real_return::RealReturnResult> {
    Json(traderview_core::real_return::analyze(&b))
}

/// CD early-withdrawal penalty: interest earned minus the months-of-interest
/// penalty, net proceeds, annualized yield, and whether principal is lost.
async fn cd_penalty_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::cd_early_withdrawal::CdInput>,
) -> Json<traderview_core::cd_early_withdrawal::CdResult> {
    Json(traderview_core::cd_early_withdrawal::analyze(&b))
}

/// Yield on cost: a dividend against your cost basis vs the current price,
/// with a projected YOC at the dividend growth rate.
async fn yield_on_cost_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::yield_on_cost::YieldOnCostInput>,
) -> Json<traderview_core::yield_on_cost::YieldOnCostResult> {
    Json(traderview_core::yield_on_cost::analyze(&b))
}

/// Trade expectancy: the per-trade edge from win rate + avg win/loss, the
/// reward:risk ratio, the break-even win rate, and expectancy in R.
async fn trade_expectancy_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::trade_expectancy::ExpectancyInput>,
) -> Json<traderview_core::trade_expectancy::ExpectancyResult> {
    Json(traderview_core::trade_expectancy::analyze(&b))
}

/// Wage converter: hourly ↔ salary across week / two weeks / month / year.
async fn wage_converter_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::wage_converter::WageInput>,
) -> Json<traderview_core::wage_converter::WageResult> {
    Json(traderview_core::wage_converter::analyze(&b))
}

/// Sales tax / VAT: add tax to a net price or extract it from a gross total.
async fn sales_tax_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::sales_tax::SalesTaxInput>,
) -> Json<traderview_core::sales_tax::SalesTaxResult> {
    Json(traderview_core::sales_tax::analyze(&b))
}

/// Bond accrued interest + dirty price between coupon dates.
async fn accrued_interest_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::accrued_interest::AccruedInput>,
) -> Json<traderview_core::accrued_interest::AccruedResult> {
    Json(traderview_core::accrued_interest::analyze(&b))
}

/// Stock-split position adjuster: scale shares, price, and basis by a ratio.
async fn stock_split_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::stock_split::SplitInput>,
) -> Json<traderview_core::stock_split::SplitResult> {
    Json(traderview_core::stock_split::analyze(&b))
}

/// T-bill yields: bank-discount, money-market, coupon-equivalent, effective.
async fn tbill_yield_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::tbill_yield::TbillInput>,
) -> Json<traderview_core::tbill_yield::TbillResult> {
    Json(traderview_core::tbill_yield::analyze(&b))
}

/// Debt-service coverage ratio + max loan that clears a target DSCR.
async fn dscr_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::dscr::DscrInput>,
) -> Json<traderview_core::dscr::DscrResult> {
    Json(traderview_core::dscr::analyze(&b))
}

/// Graham number, margin of safety, P/E×P/B test, and net-net screen.
async fn graham_number_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::graham_number::GrahamInput>,
) -> Json<traderview_core::graham_number::GrahamResult> {
    Json(traderview_core::graham_number::analyze(&b))
}

/// Take-home pay: gross paycheck to net, per period and per year.
async fn take_home_paycheck_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::take_home_paycheck::PaycheckInput>,
) -> Json<traderview_core::take_home_paycheck::PaycheckResult> {
    Json(traderview_core::take_home_paycheck::analyze(&b))
}

/// Enterprise value + EV/EBITDA, EV/Sales, and EBITDA margin.
async fn ev_ebitda_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::ev_ebitda::EvEbitdaInput>,
) -> Json<traderview_core::ev_ebitda::EvEbitdaResult> {
    Json(traderview_core::ev_ebitda::analyze(&b))
}

/// Holding-period return: price + income return, annualized over days held.
async fn holding_period_return_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::holding_period_return::HprInput>,
) -> Json<traderview_core::holding_period_return::HprResult> {
    Json(traderview_core::holding_period_return::analyze(&b))
}

/// Altman Z-Score — five-ratio bankruptcy-distress model + zone.
async fn altman_z_score_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::altman_z_score::AltmanInput>,
) -> Json<traderview_core::altman_z_score::AltmanResult> {
    Json(traderview_core::altman_z_score::analyze(&b))
}

/// Piotroski F-Score — 9-point financial-strength test.
async fn piotroski_f_score_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::piotroski_f_score::PiotroskiInput>,
) -> Json<traderview_core::piotroski_f_score::PiotroskiResult> {
    Json(traderview_core::piotroski_f_score::analyze(&b))
}

/// GMROI — gross-margin return on inventory + turnover and days of inventory.
async fn gmroi_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::gmroi::GmroiInput>,
) -> Json<traderview_core::gmroi::GmroiResult> {
    Json(traderview_core::gmroi::analyze(&b))
}

/// Roth IRA contribution limit after the MAGI phase-out.
async fn roth_contribution_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::roth_contribution::RothInput>,
) -> Json<traderview_core::roth_contribution::RothResult> {
    Json(traderview_core::roth_contribution::analyze(&b))
}

/// Interest- and fixed-charge-coverage ratios.
async fn interest_coverage_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::interest_coverage::CoverageInput>,
) -> Json<traderview_core::interest_coverage::CoverageResult> {
    Json(traderview_core::interest_coverage::analyze(&b))
}

/// Capital-gains tax — long-term 0/15/20 stacking or short-term ordinary.
async fn capital_gains_tax_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::capital_gains_tax::CapGainsInput>,
) -> Json<traderview_core::capital_gains_tax::CapGainsResult> {
    Json(traderview_core::capital_gains_tax::analyze(&b))
}

/// Traditional IRA deduction after the MAGI phase-out.
async fn traditional_ira_deduction_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::traditional_ira_deduction::TradIraInput>,
) -> Json<traderview_core::traditional_ira_deduction::TradIraResult> {
    Json(traderview_core::traditional_ira_deduction::analyze(&b))
}

/// Rule of 40 — revenue growth + profit margin vs the 40% bar.
async fn rule_of_40_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::rule_of_40::RuleOf40Input>,
) -> Json<traderview_core::rule_of_40::RuleOf40Result> {
    Json(traderview_core::rule_of_40::analyze(&b))
}

/// WACC — blended after-tax cost of capital (optional CAPM cost of equity).
async fn wacc_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::wacc::WaccInput>,
) -> Json<traderview_core::wacc::WaccResult> {
    Json(traderview_core::wacc::analyze(&b))
}

/// DuPont ROE decomposition (five-step).
async fn dupont_roe_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::dupont_roe::DupontInput>,
) -> Json<traderview_core::dupont_roe::DupontResult> {
    Json(traderview_core::dupont_roe::analyze(&b))
}

/// Taxation of Social Security benefits (Pub 915 provisional-income tiers).
async fn ss_taxation_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::ss_taxation::SsTaxInput>,
) -> Json<traderview_core::ss_taxation::SsTaxResult> {
    Json(traderview_core::ss_taxation::analyze(&b))
}

/// NPV / IRR for a cash-flow series, with payback and profitability index.
async fn npv_irr_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::npv_irr::NpvIrrInput>,
) -> Json<traderview_core::npv_irr::NpvIrrResult> {
    Json(traderview_core::npv_irr::analyze(&b))
}

/// Degree of operating, financial, and combined leverage.
async fn leverage_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::leverage_dol_dfl::LeverageInput>,
) -> Json<traderview_core::leverage_dol_dfl::LeverageResult> {
    Json(traderview_core::leverage_dol_dfl::analyze(&b))
}

/// Two-asset portfolio risk/return + diversification benefit (Markowitz).
async fn two_asset_portfolio_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::two_asset_portfolio::TwoAssetInput>,
) -> Json<traderview_core::two_asset_portfolio::TwoAssetResult> {
    Json(traderview_core::two_asset_portfolio::analyze(&b))
}

/// Mortgage recast — re-amortize after a lump-sum, same term, lower payment.
async fn mortgage_recast_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::mortgage_recast::RecastInput>,
) -> Json<traderview_core::mortgage_recast::RecastResult> {
    Json(traderview_core::mortgage_recast::analyze(&b))
}

/// Tax-equivalent yield — muni vs taxable bond on an after-tax basis.
async fn tax_equivalent_yield_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::tax_equivalent_yield::TeyInput>,
) -> Json<traderview_core::tax_equivalent_yield::TeyResult> {
    Json(traderview_core::tax_equivalent_yield::analyze(&b))
}

/// PMI removal timeline — months until the balance reaches 80% / 78% LTV.
async fn pmi_removal_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::pmi_removal::PmiInput>,
) -> Json<traderview_core::pmi_removal::PmiResult> {
    Json(traderview_core::pmi_removal::analyze(&b))
}

/// Free cash flow + FCF margin, yield, and cash-conversion quality.
async fn free_cash_flow_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::free_cash_flow::FcfInput>,
) -> Json<traderview_core::free_cash_flow::FcfResult> {
    Json(traderview_core::free_cash_flow::analyze(&b))
}

/// Credit-card minimum-payment trap vs a fixed payment.
async fn credit_card_payoff_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::credit_card_payoff::CardInput>,
) -> Json<traderview_core::credit_card_payoff::CardResult> {
    Json(traderview_core::credit_card_payoff::analyze(&b))
}

/// Bond pricing — price a coupon bond from its yield to maturity.
async fn bond_pricing_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::bond_pricing::BondPriceInput>,
) -> Json<traderview_core::bond_pricing::BondPriceResult> {
    Json(traderview_core::bond_pricing::analyze(&b))
}

/// Cash-out refinance — max loan at LTV, cash out, new payment, equity left.
async fn cash_out_refinance_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::cash_out_refinance::CashOutInput>,
) -> Json<traderview_core::cash_out_refinance::CashOutResult> {
    Json(traderview_core::cash_out_refinance::analyze(&b))
}

/// Income-statement margin waterfall — gross / operating / pre-tax / net.
async fn margin_analysis_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::margin_analysis::MarginInput>,
) -> Json<traderview_core::margin_analysis::MarginResult> {
    Json(traderview_core::margin_analysis::analyze(&b))
}

/// Bonus gross-up — the gross payment needed to net a target after tax.
async fn bonus_grossup_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::bonus_grossup::GrossUpInput>,
) -> Json<traderview_core::bonus_grossup::GrossUpResult> {
    Json(traderview_core::bonus_grossup::analyze(&b))
}

/// Lease cost with escalations + concessions → total, NPV, effective rent.
async fn rent_escalation_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::rent_escalation::RentEscalationInput>,
) -> Json<traderview_core::rent_escalation::RentEscalationResult> {
    Json(traderview_core::rent_escalation::analyze(&b))
}

/// True loan APR including upfront fees.
async fn loan_apr_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::loan_apr::LoanAprInput>,
) -> Json<traderview_core::loan_apr::LoanAprResult> {
    Json(traderview_core::loan_apr::analyze(&b))
}

/// Primary-home sale capital gain after the §121 exclusion.
async fn home_sale_exclusion_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::home_sale_exclusion::HomeSaleInput>,
) -> Json<traderview_core::home_sale_exclusion::HomeSaleResult> {
    Json(traderview_core::home_sale_exclusion::analyze(&b))
}

/// Life-insurance needs analysis (DIME) — coverage gap net of existing.
async fn life_insurance_needs_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::life_insurance_needs::LifeInsuranceInput>,
) -> Json<traderview_core::life_insurance_needs::LifeInsuranceResult> {
    Json(traderview_core::life_insurance_needs::analyze(&b))
}

/// Car affordability — the 20/4/10 rule worked back to a max car price.
async fn car_affordability_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::car_affordability::CarAffordInput>,
) -> Json<traderview_core::car_affordability::CarAffordResult> {
    Json(traderview_core::car_affordability::analyze(&b))
}

/// Disability-insurance needs — monthly benefit gap net of group LTD.
async fn disability_insurance_needs_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::disability_insurance_needs::DisabilityInput>,
) -> Json<traderview_core::disability_insurance_needs::DisabilityResult> {
    Json(traderview_core::disability_insurance_needs::analyze(&b))
}

/// True hourly wage — net pay after job costs over all job-related hours.
async fn true_hourly_wage_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::true_hourly_wage::TrueWageInput>,
) -> Json<traderview_core::true_hourly_wage::TrueWageResult> {
    Json(traderview_core::true_hourly_wage::analyze(&b))
}

/// Property tax — annual/monthly tax from value, assessment ratio, mill rate.
async fn property_tax_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::property_tax::PropertyTaxInput>,
) -> Json<traderview_core::property_tax::PropertyTaxResult> {
    Json(traderview_core::property_tax::analyze(&b))
}

/// Rental NOI — net operating income from rental income statement line items.
async fn rental_noi_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::rental_noi::RentalNoiInput>,
) -> Json<traderview_core::rental_noi::RentalNoiResult> {
    Json(traderview_core::rental_noi::analyze(&b))
}

/// Mortgage affordability — max home price under the 28/36 rule.
async fn mortgage_affordability_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::mortgage_affordability::AffordabilityInput>,
) -> Json<traderview_core::mortgage_affordability::AffordabilityResult> {
    Json(traderview_core::mortgage_affordability::analyze(&b))
}

/// Overtime pay — weekly/annual gross from regular, OT, and double-time hours.
async fn overtime_pay_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::overtime_pay::OvertimeInput>,
) -> Json<traderview_core::overtime_pay::OvertimeResult> {
    Json(traderview_core::overtime_pay::analyze(&b))
}

/// Solar payback — net cost, payback years, lifetime savings, ROI.
async fn solar_payback_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::solar_payback::SolarInput>,
) -> Json<traderview_core::solar_payback::SolarResult> {
    Json(traderview_core::solar_payback::analyze(&b))
}

/// Portfolio longevity — years a nest egg lasts under inflation-adjusted draws.
async fn portfolio_longevity_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::portfolio_longevity::LongevityInput>,
) -> Json<traderview_core::portfolio_longevity::LongevityResult> {
    Json(traderview_core::portfolio_longevity::analyze(&b))
}

/// Second income — net household benefit after taxes, childcare, work costs.
async fn second_income_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::second_income::SecondIncomeInput>,
) -> Json<traderview_core::second_income::SecondIncomeResult> {
    Json(traderview_core::second_income::analyze(&b))
}

/// Break-even occupancy — occupancy needed to cover opex + debt service.
async fn breakeven_occupancy_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::breakeven_occupancy::BreakevenOccupancyInput>,
) -> Json<traderview_core::breakeven_occupancy::BreakevenOccupancyResult> {
    Json(traderview_core::breakeven_occupancy::analyze(&b))
}

/// Rent affordability — max rent under the 30% and debt-adjusted rules.
async fn rent_affordability_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::rent_affordability::RentAffordInput>,
) -> Json<traderview_core::rent_affordability::RentAffordResult> {
    Json(traderview_core::rent_affordability::analyze(&b))
}

/// Real raise — whether a pay raise beats inflation (purchasing-power change).
async fn real_raise_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::real_raise::RealRaiseInput>,
) -> Json<traderview_core::real_raise::RealRaiseResult> {
    Json(traderview_core::real_raise::analyze(&b))
}

/// SDE business valuation — seller's discretionary earnings × multiple.
async fn sde_valuation_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::sde_valuation::SdeInput>,
) -> Json<traderview_core::sde_valuation::SdeResult> {
    Json(traderview_core::sde_valuation::analyze(&b))
}

/// Freelance rate — the hourly rate a contractor must charge to net a target.
async fn freelance_rate_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::freelance_rate::FreelanceInput>,
) -> Json<traderview_core::freelance_rate::FreelanceResult> {
    Json(traderview_core::freelance_rate::analyze(&b))
}

/// Preferred stock valuation — fair value and current yield (perpetuity).
async fn preferred_stock_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::preferred_stock::PreferredInput>,
) -> Json<traderview_core::preferred_stock::PreferredResult> {
    Json(traderview_core::preferred_stock::analyze(&b))
}

/// Margin interest — carry cost of a margin loan + break-even return.
async fn margin_interest_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::margin_interest::MarginInterestInput>,
) -> Json<traderview_core::margin_interest::MarginInterestResult> {
    Json(traderview_core::margin_interest::analyze(&b))
}

/// QBI deduction — IRC § 199A 20% pass-through deduction.
async fn qbi_deduction_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::qbi_deduction::QbiInput>,
) -> Json<traderview_core::qbi_deduction::QbiResult> {
    Json(traderview_core::qbi_deduction::analyze(&b))
}

/// Federal estate tax — taxable estate above the unified exclusion.
async fn estate_tax_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::estate_tax::EstateTaxInput>,
) -> Json<traderview_core::estate_tax::EstateTaxResult> {
    Json(traderview_core::estate_tax::analyze(&b))
}

/// Marriage penalty / bonus — joint tax vs two single filers.
async fn marriage_penalty_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::marriage_penalty::MarriagePenaltyInput>,
) -> Json<traderview_core::marriage_penalty::MarriagePenaltyResult> {
    Json(traderview_core::marriage_penalty::analyze(&b))
}

/// Standard vs itemized deduction — whichever deduction is larger.
async fn standard_vs_itemized_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::standard_vs_itemized::StdVsItemizedInput>,
) -> Json<traderview_core::standard_vs_itemized::StdVsItemizedResult> {
    Json(traderview_core::standard_vs_itemized::analyze(&b))
}

/// Up / down capture ratio — gains captured in up markets vs losses in down.
async fn capture_ratio_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::capture_ratio::CaptureRatioInput>,
) -> Json<traderview_core::capture_ratio::CaptureRatioResult> {
    Json(traderview_core::capture_ratio::analyze(&b))
}

/// Rental total return — the four-component decomposition on cash invested.
async fn rental_total_return_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::rental_total_return::RentalReturnInput>,
) -> Json<traderview_core::rental_total_return::RentalReturnResult> {
    Json(traderview_core::rental_total_return::analyze(&b))
}

/// Economic Value Added — economic profit above the capital charge.
async fn economic_value_added_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::economic_value_added::EvaInput>,
) -> Json<traderview_core::economic_value_added::EvaResult> {
    Json(traderview_core::economic_value_added::analyze(&b))
}

/// Modified internal rate of return.
async fn mirr_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::mirr::MirrInput>,
) -> Json<traderview_core::mirr::MirrResult> {
    Json(traderview_core::mirr::analyze(&b))
}

/// Equivalent annual cost — level annual cost over an asset's life.
async fn equivalent_annual_cost_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::equivalent_annual_cost::EacInput>,
) -> Json<traderview_core::equivalent_annual_cost::EacResult> {
    Json(traderview_core::equivalent_annual_cost::analyze(&b))
}

/// Multi-product break-even — weighted-average contribution margin CVP.
async fn multi_product_breakeven_route(
    _u: AuthUser,
    Json(b): Json<traderview_core::multi_product_breakeven::MultiProductInput>,
) -> Json<traderview_core::multi_product_breakeven::MultiProductResult> {
    Json(traderview_core::multi_product_breakeven::analyze(&b))
}
