// Main entry. Wires tabs, runs auth bootstrap, dispatches to view modules.

import { api, initApi, ApiError } from './api.js';
import { esc } from './util.js';
import { showAuthScreen, hideAuthScreen } from './auth.js';
import { installSymbolHotkey } from './symbol_hotkey_install.js';
import { getGlobalSymbol } from './_global_symbol.js';
import { renderDashboard } from './views/dashboard.js';
import { renderTradesView } from './views/trades.js';
import { renderTradeDetail } from './views/trade_detail.js';
import { renderJournalView } from './views/journal.js';
import { renderCalendar } from './views/calendar.js';
import { renderReports } from './views/reports.js';
import { renderCharts } from './views/charts.js';
import { renderMultichart } from './views/multichart.js';
import { renderSqueezeScanner } from './views/squeeze_scanner.js';
import { renderIpoCalendar } from './views/ipo_calendar.js';
import { renderTopNews } from './views/top_news.js';
import { renderFinnhubPattern } from './views/finnhub_pattern.js';
import { renderFinnhubSr } from './views/finnhub_sr.js';
import { renderFinnhubAggregate } from './views/finnhub_aggregate.js';
import { renderForexRates } from './views/forex_rates.js';
import { renderEconomicCalendar } from './views/economic_calendar.js';
import { renderSymbolChanges } from './views/symbol_changes.js';
import { renderEtfProfile } from './views/etf_profile.js';
import { renderLobbying } from './views/lobbying.js';
import { renderCongressionalTrading } from './views/congressional_trading.js';
import { renderFinnhubSearch } from './views/finnhub_search.js';
import { renderFdaCalendar } from './views/fda_calendar.js';
import { renderMarketStatus } from './views/market_status.js';
import { renderIndexConstituents } from './views/index_constituents.js';
import { renderInsiderTransactionsFinnhub } from './views/insider_transactions_finnhub.js';
import { renderNewsSentiment } from './views/news_sentiment.js';
import { renderPriceTarget } from './views/price_target.js';
import { renderEstimatesDashboard } from './views/estimates_dashboard.js';
import { renderCryptoMarkets } from './views/crypto_markets.js';
import { renderHistoricalMarketCap } from './views/historical_market_cap.js';
import { renderEarningsCallLive } from './views/earnings_call_live.js';
import { renderSupplyChain } from './views/supply_chain.js';
import { renderEsg } from './views/esg.js';
import { renderSectorHeatmap } from './views/sector_heatmap.js';
import { renderBondYieldCurve } from './views/bond_yield_curve.js';
import { renderUnusualOptions } from './views/unusual_options.js';
import { renderSubscriptions } from './views/subscriptions.js';
import { renderRevenueBreakdown } from './views/revenue_breakdown.js';
import { renderEarningsQuality } from './views/earnings_quality.js';
import { renderQuarterlyTax } from './views/quarterly_tax.js';
import { renderMileageLog } from './views/mileage_log.js';
import { renderFilingsBrowser } from './views/filings_browser.js';
import { renderInsiderSentiment } from './views/insider_sentiment.js';
import { renderInstitutional13F } from './views/institutional_13f.js';
import { renderSection179 } from './views/section_179.js';
import { renderRetirementMax } from './views/retirement_max.js';
import { renderSplitsHistory } from './views/splits_history.js';
import { renderMutualFund } from './views/mutual_fund.js';
import { renderUsptoPatents } from './views/uspto_patents.js';
import { renderHomeOffice } from './views/home_office.js';
import { renderIncome1099 } from './views/income_1099.js';
import { renderMealDeduction } from './views/meal_deduction.js';
import { renderBizCategorizer } from './views/biz_categorizer.js';
import { renderDepreciation } from './views/depreciation.js';
import { renderTravelPerDiem } from './views/travel_per_diem.js';
import { renderQbi199A } from './views/qbi_199a.js';
import { renderStateTax } from './views/state_tax.js';
import { renderScorpCalc } from './views/scorp_calc.js';
import { renderNolTracker } from './views/nol_tracker.js';
import { renderAugustaRule } from './views/augusta_rule.js';
import { renderCharitablePlanner } from './views/charitable_planner.js';
import { renderFbar8938 } from './views/fbar_8938.js';
import { renderSec1256 } from './views/sec_1256.js';
import { renderWashSaleTracker } from './views/wash_sale_tracker.js';
import { renderHsaMax } from './views/hsa_max.js';
import { renderForex988 } from './views/forex_988.js';
import { renderRdCredit } from './views/rd_credit.js';
import { renderMtmElection } from './views/mtm_election.js';
import { renderTtsQualification } from './views/tts_qualification.js';
import { renderQsbs1202 } from './views/qsbs_1202.js';
import { renderEducationCredits } from './views/education_credits.js';
import { renderAccountablePlan } from './views/accountable_plan.js';
import { renderDcfsa } from './views/dcfsa.js';
import { renderEvCredit } from './views/ev_credit.js';
import { renderForeignTaxCredit } from './views/foreign_tax_credit.js';
import { renderRothLadder } from './views/roth_ladder.js';
import { renderGiftTax } from './views/gift_tax.js';
import { renderCleanEnergy25D } from './views/clean_energy_25d.js';
import { renderSaversCredit } from './views/savers_credit.js';
import { renderInheritedIraRmd } from './views/inherited_ira_rmd.js';
import { renderQcdTracker } from './views/qcd_tracker.js';
import { renderNuaStrategy } from './views/nua_strategy.js';
import { renderKiddieTax } from './views/kiddie_tax.js';
import { renderQozTracker } from './views/qoz_tracker.js';
import { renderRollover529Roth } from './views/rollover_529_roth.js';
import { renderSeHealthDeduction } from './views/se_health_deduction.js';
import { renderMegaBackdoorRoth } from './views/mega_backdoor_roth.js';
import { renderCostSeg } from './views/cost_seg.js';
import { renderPassiveLoss } from './views/passive_loss.js';
import { renderSection1031 } from './views/section_1031.js';
import { renderInstallmentSale } from './views/installment_sale.js';
import { renderStrLoophole } from './views/str_loophole.js';
import { renderAmtCalc } from './views/amt_calc.js';
import { renderIsoExercise } from './views/iso_exercise.js';
import { renderNsoExercise } from './views/nso_exercise.js';
import { renderRsuVestTracker } from './views/rsu_vest_tracker.js';
import { renderEsppCalc } from './views/espp_calc.js';
import { renderBackdoorRoth } from './views/backdoor_roth.js';
import { renderCrossBrokerWash } from './views/cross_broker_wash.js';
import { renderAbleAccount } from './views/able_account.js';
import { renderConservationEasement } from './views/conservation_easement.js';
import { renderLihtc } from './views/lihtc.js';
import { renderMlpK1 } from './views/mlp_k1.js';
import { renderHistoricRehab } from './views/historic_rehab.js';
import { renderDisabledAccess } from './views/disabled_access.js';
import { renderFilm181 } from './views/film_181.js';
import { renderPartialDisposition } from './views/partial_disposition.js';
import { renderTtsScorer } from './views/tts_scorer.js';
import { renderSection475f } from './views/section_475f.js';
import { renderSection195 } from './views/section_195.js';
import { renderSection1244 } from './views/section_1244.js';
import { renderSection280f } from './views/section_280f.js';
import { renderSection197 } from './views/section_197.js';
import { renderSection274 } from './views/section_274.js';
import { renderSolo401k } from './views/solo_401k.js';
import { renderGrat } from './views/grat.js';
import { renderSection469 } from './views/section_469.js';
import { renderSepIra } from './views/sep_ira.js';
import { renderSection72t } from './views/section_72t.js';
import { renderDaf } from './views/daf.js';
import { renderSlat } from './views/slat.js';
import { renderSection121 } from './views/section_121.js';
import { renderSection1361 } from './views/section_1361.js';
import { renderSection162l } from './views/section_162l.js';
import { renderSection7872 } from './views/section_7872.js';
import { renderCrut } from './views/crut.js';
import { renderIlit } from './views/ilit.js';
import { renderSection168k } from './views/section_168k.js';
import { renderSection168 } from './views/section_168.js';
import { renderSection263a } from './views/section_263a.js';
import { renderSection6654 } from './views/section_6654.js';
import { renderSection911 } from './views/section_911.js';
import { renderSection1411 } from './views/section_1411.js';
import { renderSection280e } from './views/section_280e.js';
import { renderSection165g } from './views/section_165g.js';
import { renderSection2010c } from './views/section_2010c.js';
import { renderSection174 } from './views/section_174.js';
import { renderSection691 } from './views/section_691.js';
import { renderSection25a } from './views/section_25a.js';
import { renderSection221 } from './views/section_221.js';
import { renderCrat } from './views/crat.js';
import { renderResidencyDaycount } from './views/residency_daycount.js';
import { renderSection36b } from './views/section_36b.js';
import { renderSimpleIra } from './views/simple_ira.js';
import { renderDefinedBenefit } from './views/defined_benefit.js';
import { renderSection213 } from './views/section_213.js';
import { renderSection6038 } from './views/section_6038.js';
import { renderSection408d3 } from './views/section_408d3.js';
import { renderSection162m } from './views/section_162m.js';
import { renderSection4975 } from './views/section_4975.js';
import { renderSection4980h } from './views/section_4980h.js';
import { renderSection263Tpr } from './views/section_263_tpr.js';
import { renderSection6048 } from './views/section_6048.js';
import { renderSection6038a } from './views/section_6038a.js';
import { renderSection7702a } from './views/section_7702a.js';
import { renderCryptoStaking } from './views/crypto_staking.js';
import { renderSection1042 } from './views/section_1042.js';
import { renderSection1259 } from './views/section_1259.js';
import { renderSection1296Pfic } from './views/section_1296_pfic.js';
import { renderSection12451250 } from './views/section_1245_1250.js';
import { renderSection351721 } from './views/section_351_721.js';
import { renderSection172 } from './views/section_172.js';
import { renderSection1035 } from './views/section_1035.js';
import { renderSection6707a } from './views/section_6707a.js';
import { renderSection280g } from './views/section_280g.js';
import { renderSection1374 } from './views/section_1374.js';
import { renderSection871m } from './views/section_871m.js';
import { renderSection1402 } from './views/section_1402.js';
import { renderSection1276 } from './views/section_1276.js';
import { renderSection1233 } from './views/section_1233.js';
import { renderSection6166 } from './views/section_6166.js';
import { renderSection4941 } from './views/section_4941.js';
import { renderSection1092 } from './views/section_1092.js';
import { renderSection6038b } from './views/section_6038b.js';
import { renderSection2056 } from './views/section_2056.js';
import { renderSection4940 } from './views/section_4940.js';
import { renderSection7345 } from './views/section_7345.js';
import { renderSection1212 } from './views/section_1212.js';
import { renderSection4980d } from './views/section_4980d.js';
import { renderSection6663 } from './views/section_6663.js';
import { renderSection6694 } from './views/section_6694.js';
import { renderSection6045b } from './views/section_6045b.js';
import { renderSection529 } from './views/section_529.js';
import { renderSection530 } from './views/section_530.js';
import { renderSection401kHardship } from './views/section_401k_hardship.js';
import { renderSection72p } from './views/section_72p.js';
import { renderSection401a9 } from './views/section_401a9.js';
import { renderSection6015 } from './views/section_6015.js';
import { renderSection6651 } from './views/section_6651.js';
import { renderSection1014 } from './views/section_1014.js';
import { renderSection23 } from './views/section_23.js';
import { renderSection32Eic } from './views/section_32_eic.js';
import { renderSection4942 } from './views/section_4942.js';
import { renderSection4960 } from './views/section_4960.js';
import { renderSection6213 } from './views/section_6213.js';
import { renderSection6321 } from './views/section_6321.js';
import { renderSection6331 } from './views/section_6331.js';
import { renderSection4943 } from './views/section_4943.js';
import { renderSection4944 } from './views/section_4944.js';
import { renderSection4945 } from './views/section_4945.js';
import { renderSection6664 } from './views/section_6664.js';
import { renderSection7430 } from './views/section_7430.js';
import { renderSection6502 } from './views/section_6502.js';
import { renderSection7122 } from './views/section_7122.js';
import { renderSection6159 } from './views/section_6159.js';
import { renderSection7811 } from './views/section_7811.js';
import { renderSection6724 } from './views/section_6724.js';
import { renderSection24Ctc } from './views/section_24_ctc.js';
import { renderSection21Cdcc } from './views/section_21_cdcc.js';
import { renderSection71Alimony } from './views/section_71_alimony.js';
import { renderSection152 } from './views/section_152.js';
import { renderSection7508a } from './views/section_7508a.js';
import { renderSection132 } from './views/section_132.js';
import { renderSection127 } from './views/section_127.js';
import { renderSection125 } from './views/section_125.js';
import { renderSection165c3 } from './views/section_165c3.js';
import { renderSection119 } from './views/section_119.js';
import { renderSection25c } from './views/section_25c.js';
import { renderSection45l } from './views/section_45l.js';
import { renderSection179d } from './views/section_179d.js';
import { renderSection86 } from './views/section_86.js';
import { renderSection219 } from './views/section_219.js';
import { renderSection415 } from './views/section_415.js';
import { renderSection414v } from './views/section_414v.js';
import { renderSection481a } from './views/section_481a.js';
import { renderSection168g } from './views/section_168g.js';
import { renderSection416 } from './views/section_416.js';
import { renderSection446 } from './views/section_446.js';
import { renderSection451 } from './views/section_451.js';
import { renderSection461 } from './views/section_461.js';
import { renderSection471 } from './views/section_471.js';
import { renderSection482 } from './views/section_482.js';
import { renderSection901 } from './views/section_901.js';
import { renderSection951 } from './views/section_951.js';
import { renderSection951A } from './views/section_951a.js';
import { renderSection250 } from './views/section_250.js';
import { renderSection163j } from './views/section_163j.js';
import { renderSection59A } from './views/section_59a.js';
import { renderSection245A } from './views/section_245a.js';
import { renderSection7874 } from './views/section_7874.js';
import { renderSection4501 } from './views/section_4501.js';
import { renderSection877A } from './views/section_877a.js';
import { renderSection1291 } from './views/section_1291.js';
import { renderSection1295 } from './views/section_1295.js';
import { renderSection897 } from './views/section_897.js';
import { renderSection1445 } from './views/section_1445.js';
import { renderSection754 } from './views/section_754.js';
import { renderSection302 } from './views/section_302.js';
import { renderSection332 } from './views/section_332.js';
import { renderSection338 } from './views/section_338.js';
import { renderSection368 } from './views/section_368.js';
import { renderSection1248 } from './views/section_1248.js';
import { renderSection355 } from './views/section_355.js';
import { renderSection311 } from './views/section_311.js';
import { renderSection1366 } from './views/section_1366.js';
import { renderSection1368 } from './views/section_1368.js';
import { renderSection41 } from './views/section_41.js';
import { renderSection1033 } from './views/section_1033.js';
import { renderSection1400z } from './views/section_1400z.js';
import { renderSection357 } from './views/section_357.js';
import { renderSection305 } from './views/section_305.js';
import { renderSection336 } from './views/section_336.js';
import { renderSection30D } from './views/section_30d.js';
import { renderSection45Q } from './views/section_45q.js';
import { renderSection48 } from './views/section_48.js';
import { renderSection38 } from './views/section_38.js';
import { renderSection6050W } from './views/section_6050w.js';
import { renderSection45X } from './views/section_45x.js';
import { renderSection45V } from './views/section_45v.js';
import { renderSection48C } from './views/section_48c.js';
import { renderSection25D } from './views/section_25d.js';
import { renderSection1446F } from './views/section_1446f.js';
import { renderSection42 } from './views/section_42.js';
import { renderSection83 } from './views/section_83.js';
import { renderSection409A } from './views/section_409a.js';
import { renderSection457 } from './views/section_457.js';
import { renderSection30C } from './views/section_30c.js';
import { renderSection45W } from './views/section_45w.js';
import { renderSection47 } from './views/section_47.js';
import { renderSection51 } from './views/section_51.js';
import { renderSection25E } from './views/section_25e.js';
import { renderSection460 } from './views/section_460.js';
import { renderSection467 } from './views/section_467.js';
import { renderSection79 } from './views/section_79.js';
import { renderSection105 } from './views/section_105.js';
import { renderSection269 } from './views/section_269.js';
import { renderSection1239 } from './views/section_1239.js';
import { renderSection7701o } from './views/section_7701o.js';
import { renderSection106 } from './views/section_106.js';
import { renderSection1015 } from './views/section_1015.js';
import { renderSection444 } from './views/section_444.js';
import { renderSection904 } from './views/section_904.js';
import { renderSection7701B } from './views/section_7701b.js';
import { renderSection1041 } from './views/section_1041.js';
import { renderSection871A } from './views/section_871a.js';
import { renderSection367D } from './views/section_367d.js';
import { renderSection165D } from './views/section_165d.js';
import { renderSection1377 } from './views/section_1377.js';
import { renderSection162F } from './views/section_162f.js';
import { renderSection162C } from './views/section_162c.js';
import { renderSection1297 } from './views/section_1297.js';
import { renderSection6655 } from './views/section_6655.js';
import { renderSection6700 } from './views/section_6700.js';
import { renderSection6325 } from './views/section_6325.js';
import { renderSection1298 } from './views/section_1298.js';
import { renderSection6072 } from './views/section_6072.js';
import { renderSection162a1 } from './views/section_162a1.js';
import { renderSection367A } from './views/section_367a.js';
import { renderSection882 } from './views/section_882.js';
import { renderSection884 } from './views/section_884.js';
import { renderSection6045 } from './views/section_6045.js';
import { renderSection263C } from './views/section_263c.js';
import { renderSection412 } from './views/section_412.js';
import { renderSection1362 } from './views/section_1362.js';
import { renderSection414 } from './views/section_414.js';
import { renderSection472 } from './views/section_472.js';
import { renderSection901J } from './views/section_901j.js';
import { renderSection248 } from './views/section_248.js';
import { renderSection6041 } from './views/section_6041.js';
import { renderSection6049 } from './views/section_6049.js';
import { renderSection6051 } from './views/section_6051.js';
import { renderSection1273 } from './views/section_1273.js';
import { renderSection7702 } from './views/section_7702.js';
import { renderSection6033 } from './views/section_6033.js';
import { renderSection7491 } from './views/section_7491.js';
import { renderSection483 } from './views/section_483.js';
import { renderSection4973 } from './views/section_4973.js';
import { renderSection6695 } from './views/section_6695.js';
import { renderSection2503 } from './views/section_2503.js';
import { renderSection2055 } from './views/section_2055.js';
import { renderSection511 } from './views/section_511.js';
import { renderSection2032 } from './views/section_2032.js';
import { renderSection4974 } from './views/section_4974.js';
import { renderSection6111 } from './views/section_6111.js';
import { renderSection4972 } from './views/section_4972.js';
import { renderSection6039 } from './views/section_6039.js';
import { renderSection894 } from './views/section_894.js';
import { renderSection2518 } from './views/section_2518.js';
import { renderSection199A } from './views/section_199a.js';
import { renderSection461L } from './views/section_461l.js';
import { renderSection165 } from './views/section_165.js';
import { renderSection408A } from './views/section_408a.js';
import { renderSection962 } from './views/section_962.js';
import { renderSection280C } from './views/section_280c.js';
import { renderSection1202 } from './views/section_1202.js';
import { renderSection170 } from './views/section_170.js';
import { renderSection351 } from './views/section_351.js';
import { renderSection6038D } from './views/section_6038d.js';
import { renderSection752 } from './views/section_752.js';
import { renderSection1245 } from './views/section_1245.js';
import { renderSection164 } from './views/section_164.js';
import { renderSection24 } from './views/section_24.js';
import { renderSection6045A } from './views/section_6045a.js';
import { renderSection743 } from './views/section_743.js';
import { renderSection1250 } from './views/section_1250.js';
import { renderSection1231 } from './views/section_1231.js';
import { renderSection871 } from './views/section_871.js';
import { renderSection731 } from './views/section_731.js';
import { renderSection6011 } from './views/section_6011.js';
import { renderSection382 } from './views/section_382.js';
import { renderSection1234 } from './views/section_1234.js';
import { renderSection32 } from './views/section_32.js';
import { renderSection707 } from './views/section_707.js';
import { renderSection736 } from './views/section_736.js';
import { renderSection1058 } from './views/section_1058.js';
import { renderSection269A } from './views/section_269a.js';
import { renderSection318 } from './views/section_318.js';
import { renderSection481 } from './views/section_481.js';
import { renderSection6221 } from './views/section_6221.js';
import { renderSection956 } from './views/section_956.js';
import { renderSection1296 } from './views/section_1296.js';
import { renderSection1059 } from './views/section_1059.js';
import { renderSection304 } from './views/section_304.js';
import { renderSection6112 } from './views/section_6112.js';
import { renderSection6601 } from './views/section_6601.js';
import { renderSection475 } from './views/section_475.js';
import { renderSection988 } from './views/section_988.js';
import { renderSection303 } from './views/section_303.js';
import { renderSection129 } from './views/section_129.js';
import { renderSection134 } from './views/section_134.js';
import { renderSection6672 } from './views/section_6672.js';
import { renderSection6662 } from './views/section_6662.js';
import { renderSection67 } from './views/section_67.js';
import { renderSection421 } from './views/section_421.js';
import { renderSection989 } from './views/section_989.js';
import { renderSection4958 } from './views/section_4958.js';
import { renderSection102 } from './views/section_102.js';
import { renderSection362 } from './views/section_362.js';
import { renderSection6330 } from './views/section_6330.js';
import { renderSection6404 } from './views/section_6404.js';
import { renderImportView } from './views/import.js';
import { renderPlans } from './views/plans.js';
import { renderTags } from './views/tags.js';
import { renderNoteTemplates } from './views/note_templates.js';
import { renderMentorship } from './views/mentorship.js';
import { renderCommunity, renderCommunityThread } from './views/community.js';
import { renderShares, renderSharedTrade } from './views/shares.js';
import { renderAccounts } from './views/accounts.js';
import { renderSettings } from './views/settings.js';
import { renderSearch } from './views/search.js';
import { renderNewTrade } from './views/new_trade.js';
import { renderWatchlists } from './views/watchlists.js';
import { renderResearch } from './views/research.js';
import { renderScreener } from './views/screener.js';
import { renderTopSignals } from './views/top_signals.js';
import { renderScanners } from './views/scanners.js';
import { renderSectors } from './views/sectors.js';
import { renderPaper } from './views/paper.js';
import { renderRisk } from './views/risk.js';
import { renderAlerts } from './views/alerts.js';
import { renderHotkeys } from './views/hotkeys.js';
import { renderReplay } from './views/replay.js';
import { renderTape } from './views/tape.js';
import { renderEarningsIv } from './views/earnings_iv.js';
import { renderDisclosures } from './views/disclosures.js';
import { renderSentiment } from './views/sentiment.js';
import { renderHeatmap } from './views/heatmap.js';
import { renderOptions } from './views/options_chain.js';
import { renderOptionPayoff } from './views/option_payoff.js';
import { renderVolSmile } from './views/vol_smile.js';
import { renderMonteCarlo } from './views/monte_carlo.js';
import { renderPortfolioAllocator } from './views/portfolio_allocator.js';
import { renderVarCalculator } from './views/var_calculator.js';
import { renderSeriesSmoother } from './views/series_smoother.js';
import { renderPatternDiscovery } from './views/pattern_discovery.js';
import { renderExecutionScheduler } from './views/execution_scheduler.js';
import { renderRegimeDetector } from './views/regime_detector.js';
import { renderAmericanOption } from './views/american_option.js';
import { renderFxOption } from './views/fx_option.js';
import { renderForwardVolCurve } from './views/forward_vol_curve.js';
import { renderYieldCurvePca } from './views/yield_curve_pca.js';
import { renderDividendCalendar } from './views/dividend_calendar.js';
import { renderSignalDecomposition } from './views/signal_decomposition.js';
import { renderRrButterfly } from './views/rr_butterfly.js';
import { renderCovDenoiser } from './views/cov_denoiser.js';
import { renderMicroprice } from './views/microprice.js';
import { renderDtw } from './views/dtw.js';
import { renderHurst } from './views/hurst.js';
import { renderBocpd } from './views/bocpd.js';
import { renderVasicek } from './views/vasicek.js';
import { renderOptimalF } from './views/optimal_f.js';
import { renderKalmanBeta } from './views/kalman_beta.js';
import { renderPairTrade } from './views/pair_trade.js';
import { renderIvSolver } from './views/iv_solver.js';
import { renderGreeksProfile } from './views/greeks_profile.js';
import { renderSecondOrderGreeks } from './views/second_order_greeks.js';
import { renderAlmgrenChriss } from './views/almgren_chriss.js';
import { renderImplementationShortfall } from './views/implementation_shortfall.js';
import { renderDeflatedSharpe } from './views/deflated_sharpe.js';
import { renderVpin } from './views/vpin.js';
import { renderCupAndHandle } from './views/cup_and_handle.js';
import { renderIvRank } from './views/iv_rank.js';
import { renderMarketImpact } from './views/market_impact.js';
import { renderLiquidity } from './views/liquidity.js';
import { renderSpreadTracker } from './views/spread_tracker.js';
import { renderIntradayHeatmap } from './views/intraday_heatmap.js';
import { renderIvBacktest } from './views/iv_backtest.js';
import { renderOrderBookImbalance } from './views/order_book_imbalance.js';
import { renderCusum } from './views/cusum.js';
import { renderOrderFlow } from './views/order_flow.js';
import { renderVwapSlippage } from './views/vwap_slippage.js';
import { renderPerSymbolSlippage } from './views/per_symbol_slippage.js';
import { renderOrderStaleness } from './views/order_staleness.js';
import { renderOpenType } from './views/open_type.js';
import { renderMarketProfile } from './views/market_profile.js';
import { renderOiChange } from './views/oi_change.js';
import { renderPyramid } from './views/pyramid.js';
import { renderHaReversal } from './views/ha_reversal.js';
import { renderThreeBarReversal } from './views/three_bar_reversal.js';
import { renderRangeExpansion } from './views/range_expansion.js';
import { renderAlligator } from './views/alligator.js';
import { renderDemarker } from './views/demarker.js';
import { renderMurreyMath } from './views/murrey_math.js';
import { renderDemarkPivots } from './views/demark_pivots.js';
import { renderCypherPattern } from './views/cypher_pattern.js';
import { renderDashboards } from './views/dashboards.js';
import { renderTwap } from './views/twap.js';
import { renderNewsEvent } from './views/news_event.js';
import { renderStopLossBestOf } from './views/stop_loss_best_of.js';
import { renderSqueezeAlerts } from './views/squeeze_alerts.js';
import { renderFootprint } from './views/footprint.js';
import { renderStressTest } from './views/stress_test.js';
import { renderChandelierStop } from './views/chandelier_stop.js';
import { renderTripleScreen } from './views/triple_screen.js';
import { renderAlertRules } from './views/alert_rules.js';
import { renderDailyLossLimit } from './views/daily_loss_limit.js';
import { renderDrawdownThrottle } from './views/drawdown_throttle.js';
import { renderGoalTracker } from './views/goal_tracker.js';
import { renderTradePlanChecklist } from './views/trade_plan_checklist.js';
import { renderRegimeEquity } from './views/regime_equity.js';
import { renderVolStopClose } from './views/vol_stop_close.js';
import { renderTimeInForce } from './views/time_in_force.js';
import { renderClustersTradeFeatures } from './views/clusters_trade_features.js';
import { renderClustersCorrelation } from './views/clusters_correlation.js';
import { renderSetupsBySetup } from './views/setups_by_setup.js';
import { renderCohortTilt } from './views/cohort_tilt.js';
import { renderChoppiness } from './views/choppiness.js';
import { renderVarEstimator } from './views/var_estimator.js';
import { renderKelly } from './views/kelly.js';
import { renderMcTrades } from './views/mc_trades.js';
import { renderKeyboardShortcuts } from './views/keyboard_shortcuts.js';
import { renderCommissionOptimizer } from './views/commission_optimizer.js';
import { renderMarginRunway } from './views/margin_runway.js';
import { renderRiskParity } from './views/risk_parity.js';
import { renderRiskOnOff } from './views/risk_on_off.js';
import { renderRiskReward } from './views/risk_reward.js';
import { renderTaxLossHarvest } from './views/tax_loss_harvest.js';
import { renderWashSale } from './views/wash_sale.js';
import { renderBuyingPower } from './views/buying_power.js';
import { renderMarginCall } from './views/margin_call.js';
import { renderVixTermStructure } from './views/vix_term_structure.js';
import { renderCurrencyExposure } from './views/currency_exposure.js';
import { renderBondDuration } from './views/bond_duration.js';
import { renderCarryScore } from './views/carry_score.js';
import { renderYieldCurve } from './views/yield_curve.js';
import { renderCostBasis } from './views/cost_basis.js';
import { renderStopLossBacktest } from './views/stop_loss_backtest.js';
import { renderFuturesRoll } from './views/futures_roll.js';
import { renderHeatmapDowHour } from './views/heatmap_dow_hour.js';
import { renderAtrCone } from './views/atr_cone.js';
import { renderRoundLevels } from './views/round_levels.js';
import { renderKylesLambda } from './views/kyles_lambda.js';
import { renderHawkesIntensity } from './views/hawkes_intensity.js';
import { renderKagiChart } from './views/kagi_chart.js';
import { renderRiskParitySolver } from './views/risk_parity_solver.js';
import { renderVolumeAtPrice } from './views/volume_at_price.js';
import { renderHerfindahl } from './views/herfindahl.js';
import { renderRollSpread } from './views/roll_spread.js';
import { renderThreeLineBreak } from './views/three_line_break.js';
import { renderMomentumCrash } from './views/momentum_crash.js';
import { renderEffectiveSpread } from './views/effective_spread.js';
import { renderWeightedMidprice } from './views/weighted_midprice.js';
import { renderMarginalVar } from './views/marginal_var.js';
import { renderRangeBar } from './views/range_bar.js';
import { renderTickBar } from './views/tick_bar.js';
import { renderVolumeBar } from './views/volume_bar.js';
import { renderDollarBar } from './views/dollar_bar.js';
import { renderActiveShare } from './views/active_share.js';
import { renderBrinson } from './views/brinson.js';
import { renderEquivolume } from './views/equivolume.js';
import { renderImbalanceBar } from './views/imbalance_bar.js';
import { renderBlackLitterman } from './views/black_litterman.js';
import { renderAdfTest } from './views/adf_test.js';
import { renderAroon } from './views/aroon.js';
import { renderAmihud } from './views/amihud.js';
import { renderBreadthThrust } from './views/breadth_thrust.js';
import { renderBollingerSqueeze } from './views/bollinger_squeeze.js';
import { renderBalanceOfPower } from './views/balance_of_power.js';
import { renderAnchoredMomentum } from './views/anchored_momentum.js';
import { renderAcf } from './views/acf.js';
import { renderBeta } from './views/beta.js';
import { renderBrierScore } from './views/brier_score.js';
import { renderBipowerVariation } from './views/bipower_variation.js';
import { renderBootstrapPnl } from './views/bootstrap_pnl.js';
import { renderBlockBootstrap } from './views/block_bootstrap.js';
import { renderAdNormality } from './views/ad_normality.js';
import { renderArchLm } from './views/arch_lm.js';
import { renderAlma } from './views/alma.js';
import { renderAlphatrend } from './views/alphatrend.js';
import { renderAtrChannel } from './views/atr_channel.js';
import { renderAtrTrailStop } from './views/atr_trail_stop.js';
import { renderAdl } from './views/adl.js';
import { renderAsi } from './views/asi.js';
import { renderAdOscillator } from './views/ad_oscillator.js';
import { renderBetaShrink } from './views/beta_shrink.js';
import { renderBartlett } from './views/bartlett.js';
import { renderBidAskVol } from './views/bid_ask_vol.js';
import { renderBbw } from './views/bbw.js';
import { renderBbwp } from './views/bbwp.js';
import { renderBbPercentB } from './views/bb_pb.js';
import { renderBbd } from './views/bbd.js';
import { renderBbOsc } from './views/bb_osc.js';
import { renderBorrowRate } from './views/borrow_rate.js';
import { renderBpTest } from './views/bp_test.js';
import { renderBurke } from './views/burke.js';
import { renderCamarilla } from './views/camarilla.js';
import { renderBgTest } from './views/bg_test.js';
import { renderCsi } from './views/csi.js';
import { renderCarhart4 } from './views/carhart4.js';
import { renderCsm } from './views/csm.js';
import { renderChaikinOsc } from './views/chaikin_osc.js';
import { renderCdmi } from './views/cdmi.js';
import { renderCks } from './views/cks.js';
import { renderCmo } from './views/cmo.js';
import { renderCti } from './views/cti.js';
import { renderCvi } from './views/cvi.js';
import { renderChandelier } from './views/chandelier.js';
import { renderCholesky } from './views/cholesky.js';
import { renderAbcPattern } from './views/abc_pattern.js';
import { renderAbsorption } from './views/absorption.js';
import { renderFavoritesManager } from './views/favorites_manager.js';
import { installShortcuts, setScope } from './shortcuts.js';
import { installCommandPalette } from './command_palette.js';
import { installToasts } from './toast.js';
import { installDialog } from './dialog.js';
import { installNoSpellcheck } from './_no_spellcheck.js';
import { installSymbolAutocomplete } from './_symbol_autocomplete.js';
import { installContextMenu, registerContextItems } from './context_menu.js';
import { SYMBOL_ITEMS, SYMBOL_AWARE_SCOPES, ALL_SCOPED_ITEMS } from './_context_menu.js';
import { installTooltips, upgradeTooltips, autoApplyTooltips } from './tooltip.js';
import { bootI18n, applyUiI18n, t } from './i18n.js';
import { renderCrypto } from './views/crypto.js';
import { renderBacktest } from './views/backtest.js';
import { renderEconomy } from './views/economy.js';
import { renderPairs } from './views/pairs.js';
import { renderShortInterest } from './views/short_interest.js';
import { renderDarkpool } from './views/darkpool.js';
import { renderVol } from './views/vol.js';
import { renderWebhooks } from './views/webhooks.js';
import { renderBreadth } from './views/breadth.js';
import { renderFearGreed } from './views/fear_greed.js';
import { renderPremarket } from './views/premarket.js';
import { renderAfterHours } from './views/after_hours.js';
import { renderHalts } from './views/halts.js';
import { renderLauncher } from './views/launcher.js';
import { renderLiveScanner } from './views/live_scanner.js';
import { renderCatalysts } from './views/catalysts.js';
import { renderCatalystCorrelations } from './views/catalyst_correlations.js';
import { renderUoaStream } from './views/uoa_stream.js';
import { renderGammaSqueeze } from './views/gamma_squeeze.js';
import { renderHtbRanker } from './views/htb_ranker.js';
import { renderBreadthDivergence } from './views/breadth_divergence.js';
import { renderRvolAccel } from './views/rvol_accel.js';
import { renderInsiderStream } from './views/insider_stream.js';
import { renderInsiderClusters } from './views/insider_clusters.js';
import { renderEarningsRevisions } from './views/earnings_revisions.js';
import { renderSectorTiming } from './views/sector_timing.js';
import { renderMarketGammaRegime } from './views/market_gamma_regime.js';
import { renderScannerBacktest } from './views/scanner_backtest.js';
import { renderConfluenceAutotrade } from './views/confluence_autotrade.js';
import { renderPortfolioExposure } from './views/portfolio_exposure.js';
import { renderDividendTracker } from './views/dividend_tracker.js';
import { renderMagicFormula } from './views/magic_formula.js';
import { renderPaperRebalance } from './views/paper_rebalance.js';
import { renderPaperTaxLossHarvest } from './views/paper_tax_loss_harvest.js';
import { renderSectorRotationStrategy } from './views/sector_rotation_strategy.js';
import { renderDcaSimulator } from './views/dca_simulator.js';
import { renderDividendAristocrats } from './views/dividend_aristocrats.js';
import { renderPermanentPortfolio } from './views/permanent_portfolio.js';
import { renderCapeIndicator } from './views/cape_indicator.js';
import { renderFireCalculator } from './views/fire_calculator.js';
import { renderEmergencyFund } from './views/emergency_fund.js';
import { renderNetWorthTracker } from './views/net_worth_tracker.js';
import { renderPersonalBalanceSheet } from './views/personal_balance_sheet.js';
import { renderPersonalCashFlow } from './views/personal_cash_flow.js';
import { renderFinancialRatios } from './views/financial_ratios.js';
import { renderSavingsRate } from './views/savings_rate.js';
import { renderSinkingFund } from './views/sinking_fund.js';
import { renderZeroBasedBudget } from './views/zero_based_budget.js';
import { renderFiftyThirtyTwenty } from './views/fifty_thirty_twenty.js';
import { renderEnvelopeBudget } from './views/envelope_budget.js';
import { renderDebtAvalanche } from './views/debt_avalanche.js';
import { renderDebtSnowball } from './views/debt_snowball.js';
import { renderCreditUtilization } from './views/credit_utilization.js';
import { renderAutoLoan } from './views/auto_loan.js';
import { renderMortgageAmortization } from './views/mortgage_amortization.js';
import { renderMortgageRefinance } from './views/mortgage_refinance.js';
import { renderRentVsBuy } from './views/rent_vs_buy.js';
import { renderHeloc } from './views/heloc.js';
import { renderHomeMaintenance } from './views/home_maintenance.js';
import { renderStudentLoanPayoff } from './views/student_loan_payoff.js';
import { renderPslfTracker } from './views/pslf_tracker.js';
import { renderCollege529 } from './views/college_529.js';
import { renderFafsaEfc } from './views/fafsa_efc.js';
import { renderCarTco } from './views/car_tco.js';
import { renderDrawdownCutoff } from './views/drawdown_cutoff.js';
import { renderPead } from './views/pead.js';
import { renderSentimentVelocity } from './views/sentiment_velocity.js';
import { renderConfluence } from './views/confluence.js';
import { renderVrp } from './views/vrp.js';
import { renderPairsCoint } from './views/pairs_coint.js';
import { renderIpoLockups } from './views/ipo_lockups.js';
import { renderIvTerm } from './views/iv_term.js';
import { renderSp500Predict } from './views/sp500_predict.js';
import { renderDividendCapture } from './views/dividend_capture.js';
import { renderMultiBroker } from './views/multi_broker.js';
import { renderWebull } from './views/webull.js';
import { renderVolSurface } from './views/vol_surface.js';
import { renderWalkForward } from './views/walk_forward.js';
import { renderTaxLots } from './views/tax_lots.js';
import { renderCompare } from './views/compare.js';
import { renderExports } from './views/exports.js';
import { renderAiSettings } from './views/journal_ai.js';
import { renderDeveloper } from './views/api_tokens.js';
import { renderBoards } from './views/boards.js';
import { renderNews } from './views/news.js';
import { renderEarningsCal } from './views/earnings_cal.js';
import { renderPositionSize } from './views/position_size.js';
import { renderLivePositions } from './views/live_positions.js';
import { renderCorrMatrix } from './views/corr_matrix.js';
import { renderStrategyAlerts } from './views/strategy_alerts.js';
import { renderAlgo } from './views/algo.js';
import { renderRebalance } from './views/rebalance.js';
import { renderSectorRotation } from './views/sector_rotation.js';
import { renderTapeReplay } from './views/tape_replay.js';
import { renderBacktestPresets } from './views/backtest_presets.js';
import { renderMoodAnalytics } from './views/mood_analytics.js';
import { renderDiscipline } from './views/discipline.js';
import { renderGoals } from './views/goals.js';
import { renderRDist } from './views/r_distribution.js';
import { renderTradeReviews } from './views/trade_reviews.js';
import { renderEquityForecast } from './views/equity_forecast.js';
import { renderFillQuality } from './views/fill_quality.js';
import { renderCustomIndicators } from './views/custom_indicators.js';
import { renderTradeCompare } from './views/trade_compare.js';
import { renderCsvWizard } from './views/csv_wizard.js';
import { renderAccountsOverview } from './views/accounts_overview.js';
import { renderTutorial } from './views/tutorial.js';
import { renderTaxWorkshop } from './views/tax_workshop.js';
import { renderRiskGate } from './views/risk_gate.js';
import { spinnerHTML } from './spinner.js';
import { startAlertEngine, requestNotifPermission } from './alert_engine.js';
import { startWs, on as onWsEvent } from './ws.js';
import { installHotkeyEngine } from './hotkey_engine.js';
import { renderExpensesView } from './views/expenses.js';
import { renderExpenseDashboard } from './views/expense_dashboard.js';
import { renderExpenseCalendar } from './views/expense_calendar.js';
import { renderBusinessCompare } from './views/business_compare.js';
import { renderBrokerCompare } from './views/broker_compare.js';
import { renderBrokersManage } from './views/brokers_manage.js';
import { renderBusinessesManage } from './views/businesses_manage.js';
import { renderToastHistory } from './views/toast_history.js';
import { renderLogViewer } from './views/log_viewer.js';
import { renderReceipts } from './views/receipts.js';
import { renderPurchases } from './views/purchases.js';
import { renderCategorize } from './views/categorize.js';
import { renderTaxWizard } from './views/file_taxes.js';
import { renderBudget } from './views/budget.js';

export const state = {
    mode: 'web',
    accountId: null,
    accounts: [],
    me: null,
    view: 'dashboard',
};

let uiWired = false;

export async function mountApp({ cfg, me, accounts }) {
    state.mode = cfg.mode ?? state.mode;
    state.me = me;
    state.accounts = accounts;
    if (state.accounts.length && !state.accountId) state.accountId = state.accounts[0].id;

    // Version chip — dynamic, single-sourced from workspace.package.version
    // via src-tauri/build.rs writing _version.js (loaded synchronously in
    // index.html before app.js). Falls back to the cfg.version field
    // already on hand from entry.js's api.config() call, then to fetches.
    // mountApp is the real entry path used by entry.js; boot() below only
    // fires on the tv:authed event after a login redirect.
    {
        const verEl = document.getElementById('tv-version');
        const setVer = (v) => { if (verEl && v) verEl.textContent = `v${v}`; };
        if (window.__TRADERVIEW_VERSION__) setVer(window.__TRADERVIEW_VERSION__);
        else if (cfg && cfg.version) setVer(cfg.version);
        else {
            try {
                const j = await fetch('./version.json', { cache: 'no-store' }).then(r => r.json());
                setVer(j.version);
            } catch (e) { console.warn('tv-version: all sources failed', e); }
        }
    }

    if (!uiWired) {
        bindTabs();
        uiWired = true;
    }

    const userStrip = document.getElementById('user-strip');
    if (userStrip) {
        userStrip.textContent = me.is_local ? t('app.user.local') : (me.email || me.display_name || '');
    }
    renderAccountStrip();
    // Mount broker selector + thread changes through dispatch so every
    // trade view re-renders against the new broker filter.
    try {
        const brokerCtx = await import('./broker_context.js');
        const host = document.getElementById('broker-strip');
        if (host) await brokerCtx.mountBrokerSelector(host);
        // When broker changes, the visible accounts change too — re-render
        // the account strip BEFORE dispatching so the dashboard widget
        // fetch goes out against the freshly-picked account.
        brokerCtx.onChange(() => { renderAccountStrip(); dispatch(); });
    } catch (e) {
        console.warn('broker selector mount failed', e);
    }
    await dispatch();
    hideAuthScreen();
    startAlertEngine();
    installHotkeyEngine();
    requestNotifPermission();
    startWs();
    wireWsStatusIndicator();
    wireKillSwitchIndicator();
}

// Strip WebKit's autocorrect / autocapitalize / spell-check from every
// filter or search input in the app — the suggestion overlay flags valid
// domain terms (vpin, kagi, tlb, renko, …) as misspellings, and silent
// autocorrect substitutes a typed token mid-filter ("vpin" → "pin"). We
// scope to: explicit `type="search"` inputs, anything carrying the
// opt-in `data-filter` attribute, and inputs whose placeholder starts
// with "filter" / "search" (case-insensitive). Real text-entry fields
// (journal, notes, names) keep spell-check on.
const FILTER_INPUT_SELECTOR =
    'input[type="search"], input[data-filter], textarea[data-filter]';
function isPlaceholderFilter(el) {
    const p = (el.placeholder || '').toLowerCase().trim();
    return p.startsWith('filter') || p.startsWith('search') || p.startsWith('find');
}
function disableAutocorrectOn(el) {
    if (el.dataset && el.dataset.tvNoAutocorrect === '1') return;
    el.setAttribute('autocorrect', 'off');
    el.setAttribute('autocapitalize', 'off');
    el.setAttribute('spellcheck', 'false');
    if (!el.hasAttribute('autocomplete')) el.setAttribute('autocomplete', 'off');
    if (el.dataset) el.dataset.tvNoAutocorrect = '1';
}
function applyFilterNoAutocorrect(root) {
    if (!root || !root.querySelectorAll) return;
    root.querySelectorAll(FILTER_INPUT_SELECTOR).forEach(disableAutocorrectOn);
    // Placeholder heuristic — catches the inputs not explicitly tagged
    // (e.g. dashboards.js's `placeholder="filter views…"`).
    root.querySelectorAll('input:not([type="search"]), textarea').forEach(el => {
        if (isPlaceholderFilter(el)) disableAutocorrectOn(el);
    });
}
function installFilterNoAutocorrectObserver() {
    applyFilterNoAutocorrect(document);
    const obs = new MutationObserver((muts) => {
        for (const m of muts) {
            for (const n of m.addedNodes) {
                if (n.nodeType !== 1) continue; // only Element nodes
                // The added node itself might be a filter input…
                if (n.matches && n.matches(FILTER_INPUT_SELECTOR)) disableAutocorrectOn(n);
                else if (n.matches && (n.tagName === 'INPUT' || n.tagName === 'TEXTAREA')
                         && isPlaceholderFilter(n)) disableAutocorrectOn(n);
                // …or a subtree containing one.
                if (n.querySelectorAll) applyFilterNoAutocorrect(n);
            }
        }
    });
    obs.observe(document.body, { childList: true, subtree: true });
}

async function boot() {
    installFilterNoAutocorrectObserver();
    try {
        await initApi();
    } catch (e) {
        const appEl = document.getElementById('app');
        if (appEl) {
            appEl.innerHTML = `<p class="boot">${t('boot.failed_connect', { err: e.message || String(e) })}</p>`;
        }
        return;
    }
    // Version chip — dynamic, never hardcoded in HTML. Try the backend
    // first (returns CARGO_PKG_VERSION via /api/config); if that fails
    // or omits version, fall back to the bundled frontend's package.json.
    // Either path always wins over a stale literal in index.html.
    const verEl = document.getElementById('tv-version');
    let versionSet = false;
    const setVersion = (v) => {
        if (verEl && v) {
            verEl.textContent = `v${v}`;
            versionSet = true;
        }
    };
    // Highest-priority source: window.__TRADERVIEW_VERSION__ set by
    // _version.js (a tiny build-emitted script loaded synchronously in
    // index.html). This bypasses every fetch failure mode below — if
    // _version.js loaded, the chip resolves before boot() awaits a thing.
    if (typeof window !== 'undefined' && window.__TRADERVIEW_VERSION__) {
        setVersion(window.__TRADERVIEW_VERSION__);
    }
    try {
        const cfg = await api.config();
        state.mode = cfg.mode;
        if (cfg.version) setVersion(cfg.version);
        else console.warn('tv-version: /api/config returned no version field', cfg);
    } catch (e) {
        console.warn('tv-version: /api/config failed, falling back to package.json', e);
    }
    // Fallback 1: version.json — emitted by src-tauri/build.rs every
    // build, single-sourced from workspace.package.version. Always fresh
    // for a built binary even if /api/config is unreachable. Plain name
    // (no leading dot) so the Tauri asset bundler includes it.
    if (verEl && !versionSet) {
        try {
            const v = await fetch('./version.json', { cache: 'no-store' }).then(r => r.json());
            setVersion(v.version);
        } catch (e) {
            console.warn('tv-version: version.json fallback failed', e);
        }
    }
    // Fallback 2: frontend package.json — only useful in pure-web mode
    // where there's no Tauri build step writing .version.json.
    if (verEl && !versionSet) {
        try {
            const pkg = await fetch('./package.json', { cache: 'no-store' }).then(r => r.json());
            setVersion(pkg.version);
        } catch (e) {
            console.warn('tv-version: package.json fallback failed; chip stays at HTML placeholder', e);
        }
    }
    try {
        const me = await api.me();
        const accounts = await api.accounts();
        await mountApp({ cfg: { mode: state.mode }, me, accounts });
    } catch (e) {
        if (e instanceof ApiError && e.status === 401 && state.mode === 'web') {
            showAuthScreen();
        } else {
            const appEl = document.getElementById('app');
            if (appEl) appEl.innerHTML = `<p class="boot">${t('boot.failed_connect', { err: e.message })}</p>`;
        }
    }
}

async function loadAccounts() {
    state.accounts = await api.accounts();
    // If the previously-active account UUID is no longer in the list (the
    // user deleted it from the Accounts page), drop the stale pointer.
    // Otherwise downstream calls — Import, Trades, Reports — would attach
    // it to requests and the backend returns NotFound ("error: not found"
    // in the import widget). Re-seed to the first available account so
    // the app continues to work without a manual page refresh.
    if (state.accountId && !state.accounts.some(a => a.id === state.accountId)) {
        state.accountId = '';
    }
    if (state.accounts.length && !state.accountId) state.accountId = state.accounts[0].id;
}

function renderAccountStrip() {
    const strip = document.getElementById('account-strip');
    if (!strip) return;
    // Filter by the active broker — every account lives under exactly one
    // broker (accounts.broker_id FK), so when a broker is picked we only
    // surface that broker's accounts. "All brokers" shows everything.
    let brokerId = null;
    try { brokerId = globalThis.__tvActiveBroker?.() || null; } catch {}
    const visible = brokerId
        ? state.accounts.filter(a => a.broker_id === brokerId)
        : state.accounts;
    // Re-seed accountId if the previously-selected account isn't in the
    // filtered set anymore — happens when the user switches broker.
    if (state.accountId && !visible.some(a => a.id === state.accountId)) {
        state.accountId = visible.length ? visible[0].id : '';
    }
    const newOpt = `<option value="__new__">${esc(t('app.account_strip.add_new'))}…</option>`;
    if (visible.length === 0) {
        // Empty state — still expose the "+ New account" path so users can
        // create one without leaving the page.
        strip.innerHTML = `<select id="account-select" class="account-select">
            <option disabled selected>${esc(t(state.accounts.length === 0
                ? 'app.account_strip.no_account'
                : 'app.account_strip.no_account_for_broker'))}</option>
            ${newOpt}
        </select>`;
    } else {
        const options = visible.map(a => `
            <option value="${a.id}" ${a.id === state.accountId ? 'selected' : ''}>${a.broker} · ${a.name}</option>
        `).join('');
        strip.innerHTML = `<select id="account-select" class="account-select">${options}${newOpt}</select>`;
    }
    const sel = strip.querySelector('#account-select');
    if (sel) sel.addEventListener('change', async (e) => {
        if (e.target.value === '__new__') {
            // Reset the visible selection so cancelling the wizard doesn't
            // leave the dropdown stuck on "+ New account".
            sel.value = state.accountId || '';
            try {
                const w = await import('./setup_wizard.js');
                // Default the wizard's broker pick to the currently-active
                // broker so the new account lands under it without an
                // extra click.
                let defaultSlug = null;
                if (brokerId) {
                    const ctx = await import('./broker_context.js');
                    const list = await ctx.listBrokers();
                    defaultSlug = list.find(b => b.id === brokerId)?.slug || null;
                }
                const created = await w.openSetupWizard({ kind: 'account', defaultBrokerSlug: defaultSlug });
                if (created) {
                    await loadAccounts();
                    state.accountId = created.id;
                    renderAccountStrip();
                    dispatch();
                }
            } catch (err) {
                console.warn('account wizard failed', err);
            }
            return;
        }
        state.accountId = e.target.value;
        dispatch();
    });
}

// Tabs that always live in the primary strip — `+ New Trade` is the
// primary CTA, `⚙️ Settings` is the always-on escape hatch.
const PINNED_PRIMARY_VIEWS = new Set(['new-trade', 'settings']);

// Hard cap on how many redistributable tabs are allowed in the primary
// strip. Without this, ultrawide screens pull everything out of MORE
// and the topbar feels like a wall of buttons. Adjust to taste —
// this is the "max tabs that feel calm" number.
const MAX_PRIMARY_TABS = 10;

function getRedistributableItems() {
    const nav = document.getElementById('primary-nav');
    const moreMenu = document.getElementById('nav-more-menu');
    if (!nav || !moreMenu) return [];
    return [
        ...nav.querySelectorAll(':scope > [data-view]'),
        ...moreMenu.querySelectorAll(':scope > [data-view]'),
    ].filter((el) => !PINNED_PRIMARY_VIEWS.has(el.dataset.view));
}

/**
 * Fit as many tabs as the viewport will hold into the primary strip,
 * push the rest into the MORE menu. Runs on boot + window resize +
 * drag-reorder.
 *
 * Width math: the previous attempt used `broker-strip.left` as the
 * upstream boundary, but `.tabs` has `flex: 1 1 auto` — adding items
 * to nav grows .tabs and slides broker-strip rightward, so the
 * boundary moved WITH the content and the algorithm never found an
 * overflow point. This version sums every NON-`.tabs` topbar child's
 * `offsetWidth` (those are stable, set by their content) and
 * subtracts from the topbar's total inner width to get a fixed cap.
 */
function recomputeTabOverflow() {
    const nav = document.getElementById('primary-nav');
    const topbar = document.querySelector('.topbar');
    const moreMenu = document.getElementById('nav-more-menu');
    const moreWrap = document.querySelector('.tab-more-wrap');
    const moreBtn = document.getElementById('nav-more-btn');
    if (!nav || !topbar || !moreMenu || !moreWrap || !moreBtn) return;

    const items = getRedistributableItems();
    if (items.length === 0) { moreWrap.style.display = 'none'; return; }

    // 1. Park every redistributable item in the primary nav so each
    // has a stable `.tab` layout for measurement.
    for (const item of items) {
        if (item.classList.contains('tab-more-item')) {
            item.classList.remove('tab-more-item');
            item.classList.add('tab');
        }
        if (item.parentNode !== nav || item.nextSibling !== moreWrap) {
            nav.insertBefore(item, moreWrap);
        }
    }
    moreWrap.style.display = '';

    // 2. Sum every non-`.tabs` topbar child's intrinsic width. Those
    // are set by content (broker label, account select, locale code,
    // ⌘K, ?, theme/crt/neon, ws status, ticker, user strip, brand,
    // version) and don't depend on how many tabs are in nav.
    const TOPBAR_GAP = 12;
    const TOPBAR_PADDING = 14 * 2;
    let othersWidth = 0;
    let othersCount = 0;
    for (const child of topbar.children) {
        if (child === nav) continue;
        if (child.offsetParent === null) continue; // display:none — skip
        othersWidth += child.offsetWidth;
        othersCount += 1;
    }
    const tabsBudget = topbar.clientWidth
        - TOPBAR_PADDING
        - othersWidth
        - (othersCount * TOPBAR_GAP);

    // 3. Reserve room inside .tabs for the always-on bits: MORE wrap
    // and ⚙️ Settings. Both sit at the end of nav.
    const moreWidth = moreBtn.offsetWidth || 80;
    const settingsBtn = nav.querySelector('[data-view="settings"]');
    const settingsWidth = settingsBtn ? settingsBtn.offsetWidth : 32;
    const NAV_GAP = 2;
    const itemsAvailable = tabsBudget - moreWidth - settingsWidth - (NAV_GAP * 3) - 4;

    // 4. Walk items, accumulate intrinsic widths, stop at first
    // overflow. Cap by MAX_PRIMARY_TABS so wide viewports stay calm.
    let cumWidth = 0;
    let splitIdx = items.length;
    for (let i = 0; i < items.length; i++) {
        if (i >= MAX_PRIMARY_TABS) { splitIdx = i; break; }
        const w = items[i].offsetWidth + NAV_GAP;
        if (cumWidth + w > itemsAvailable) { splitIdx = i; break; }
        cumWidth += w;
    }

    // 5. Move the overflow into MORE.
    for (let i = splitIdx; i < items.length; i++) {
        const item = items[i];
        item.classList.remove('tab');
        item.classList.add('tab-more-item');
        moreMenu.appendChild(item);
    }

    moreWrap.style.display = splitIdx >= items.length ? 'none' : '';
}

let _overflowRaf = 0;
function scheduleTabOverflowRecompute() {
    if (_overflowRaf) return;
    _overflowRaf = requestAnimationFrame(() => {
        _overflowRaf = 0;
        recomputeTabOverflow();
    });
}

function bindTabs() {
    // Topbar-wide delegation. Per-element handlers broke after
    // recomputeTabOverflow() because items shuttled between primary
    // nav and MORE menu kept their original (out-of-context) listener:
    // a `.tab-more-item` promoted to primary would still try to close
    // a menu that was never open; a `.tab` demoted to MORE would
    // navigate but not close the menu. Delegation on the topbar
    // resolves both — the handler reads class + container at click
    // time, not at bind time.
    const topbar = document.querySelector('.topbar');
    if (topbar) {
        topbar.addEventListener('click', (e) => {
            // 1) MORE button toggles the dropdown.
            const moreBtn = e.target.closest('.tab-more');
            if (moreBtn && topbar.contains(moreBtn)) {
                e.stopPropagation();
                const menu = document.getElementById('nav-more-menu');
                if (!menu) return;
                const open = menu.classList.toggle('hidden');
                moreBtn.setAttribute('aria-expanded', String(!open));
                // Menu is `position: fixed`; anchor by LEFT (not right) so
                // drag-reorder can put MORE anywhere without stranding the
                // dropdown at the far right of the viewport. Cap inside
                // the viewport with a scrollable max-height.
                if (!open) {
                    const r = moreBtn.getBoundingClientRect();
                    const MIN_W = 220;
                    const left = Math.min(r.left, window.innerWidth - MIN_W - 8);
                    const maxH = Math.max(120, window.innerHeight - r.bottom - 24);
                    menu.style.left = `${Math.max(8, left)}px`;
                    menu.style.right = 'auto';
                    menu.style.top = `${r.bottom + 4}px`;
                    menu.style.maxHeight = `${maxH}px`;
                    menu.style.overflowY = 'auto';
                }
                return;
            }
            // 2) Any [data-view] target navigates and (if it was inside
            // the MORE menu) closes the menu.
            const item = e.target.closest('[data-view]');
            if (!item || !topbar.contains(item)) return;
            window.location.hash = item.dataset.view;
            const menu = document.getElementById('nav-more-menu');
            if (menu && !menu.classList.contains('hidden')) {
                menu.classList.add('hidden');
                const trigger = document.getElementById('nav-more-btn');
                if (trigger) trigger.setAttribute('aria-expanded', 'false');
            }
            closeNavDrawer();
        });
    }
    // Outside click closes the menu.
    document.addEventListener('click', (e) => {
        const menu = document.getElementById('nav-more-menu');
        const wrap = document.querySelector('.tab-more-wrap');
        if (!menu || menu.classList.contains('hidden')) return;
        if (!wrap || wrap.contains(e.target)) return;
        menu.classList.add('hidden');
        const trigger = document.getElementById('nav-more-btn');
        if (trigger) trigger.setAttribute('aria-expanded', 'false');
    });
    window.addEventListener('hashchange', dispatch);
    bindNavToggle();
    installShortcuts();

    // Trello-style reorder for the primary nav. Persists in localStorage so
    // users can put the tabs they actually use first.
    const nav = document.getElementById('primary-nav');
    if (nav) {
        import('./drag_reorder.js').then(({ initDragReorder }) => {
            // Exclude `.tab-more` from the reorderable set — it has no
            // data-view, so it sorts by textContent and was landing at the
            // start of the strip when an old saved order replayed against
            // the new tab roster. Bumping the prefs key to `_v2` also
            // invalidates the stale 19-tab layout from before the
            // restructure.
            initDragReorder(nav, '.tab:not(.tab-more)', 'primary_nav_order_v2', {
                direction: 'horizontal',
                getKey: (el) => el.dataset.view || el.textContent.trim(),
                toastKey: 'toast.reordered_tabs',
                onReorder: scheduleTabOverflowRecompute,
            });
            // More-dropdown items: same draggability so users can pin
            // their favorites to the top of the menu. Key bumped to _v2
            // so the older saved order (which predates business-compare /
            // broker-compare) doesn't strand the new entries off-screen.
            const moreMenu = document.getElementById('nav-more-menu');
            if (moreMenu) {
                initDragReorder(moreMenu, '.tab-more-item', 'nav_more_menu_order_v2', {
                    direction: 'vertical',
                    getKey: (el) => el.dataset.view || el.textContent.trim(),
                    toastKey: 'toast.reordered_more_menu',
                    onReorder: scheduleTabOverflowRecompute,
                });
            }
        }).catch(() => { /* drag_reorder optional — not fatal */ });
    }

    // Pack the primary strip up to MAX_PRIMARY_TABS or the viewport's
    // capacity (whichever is smaller); rest goes to MORE. Runs once
    // after layout settles + on every resize.
    window.addEventListener('resize', scheduleTabOverflowRecompute);
    requestAnimationFrame(() => requestAnimationFrame(recomputeTabOverflow));
    installCommandPalette();
    installToasts();
    installDialog();
    installContextMenu();
    installNoSpellcheck();
    installSymbolAutocomplete();
    // Register a per-scope item for the launcher recents block so users can
    // wipe their navigation history without leaving the page.
    registerContextItems('launcher-recents', [
        { id: 'clear_recents', labelKey: 'ctxmenu.clear_recents',
          actionKey: 'tv:clear-recents', section: 'view' },
    ]);
    // Symbol-aware views: right-click → Copy SYMBOL / Charts for SYMBOL / etc.
    // The mount carries `data-context-scope=<view>`, so each scope just needs
    // the same SYMBOL_ITEMS set registered.
    for (const scope of SYMBOL_AWARE_SCOPES) {
        registerContextItems(scope, SYMBOL_ITEMS);
    }
    // Per-scope item sets registered from a single source of truth so
    // future row scopes are 1-line additions in ALL_SCOPED_ITEMS.
    for (const [scope, items] of ALL_SCOPED_ITEMS) {
        registerContextItems(scope, items);
    }
    installTooltips();
    installSymbolHotkey();
    void bootI18n('en').then(() => {
        applyUiI18n();
        const picker = document.getElementById('locale-picker');
        if (picker) {
            const saved = (typeof localStorage !== 'undefined') ? localStorage.getItem('tv-locale-v1') : null;
            if (saved) {
                picker.value = saved;
                // Apply saved locale on boot so it doesn't snap back to en
                // after the first applyUiI18n pass above.
                if (saved !== 'en') {
                    void (async () => {
                        try {
                            const { loadLocale } = await import('./i18n.js');
                            await loadLocale(saved);
                        } catch (_) { /* missing catalog — fall back to en */ }
                    })();
                }
            }
            // Remember the prior selection so we can revert on failed load.
            let priorLocale = picker.value;
            picker.addEventListener('change', async (e) => {
                const locale = e.target.value;
                const labelText = picker.options[picker.selectedIndex].text;
                const { loadLocale } = await import('./i18n.js');
                const keyCount = await loadLocale(locale);
                const toast = await import('./toast.js');
                const i18n  = await import('./i18n.js');
                if (keyCount === 0) {
                    // Failure: missing catalog / network error. Revert + toast.
                    picker.value = priorLocale;
                    toast.showToast(
                        i18n.t('toast.locale_failed', { locale: labelText }),
                        { level: 'error' });
                    return;
                }
                priorLocale = locale;
                try { localStorage.setItem('tv-locale-v1', locale); } catch (_) {}
                toast.showToast(
                    i18n.t('toast.locale_changed', { locale: labelText }),
                    { level: 'success' });
            });
        }
    });
    // Bridge: hash-based help/tutorial action from the new registry.
    window.addEventListener('tv:open-help', () => { window.location.hash = 'keyboard-shortcuts'; });
    window.addEventListener('tv:go-home',   () => { window.location.hash = 'launcher'; });
    // View-scoped: `n` in trades scope → new trade route.
    window.addEventListener('tv:trades-new', () => { window.location.hash = 'new-trade'; });
    // View-scoped: `r` in dashboard scope → re-render via hashchange.
    window.addEventListener('tv:dashboard-refresh', () => {
        window.dispatchEvent(new HashChangeEvent('hashchange'));
    });
    // View-scoped: `n` in journal scope → focus the body textarea.
    // Works on both /journal/<date> and trade-detail's journal block
    // (they use #body and #journal-body respectively).
    window.addEventListener('tv:journal-focus-body', () => {
        const el = document.getElementById('body') || document.getElementById('journal-body');
        if (el && typeof el.focus === 'function') { el.focus(); el.select?.(); }
    });
    // View-scoped: `n` in watchlists scope → focus add-symbol input.
    window.addEventListener('tv:watchlists-focus-add', () => {
        const el = document.querySelector('#add-sym input[name="symbol"]');
        if (el && typeof el.focus === 'function') { el.focus(); el.select?.(); }
    });
    // View-scoped: `n` in alert-rules scope → focus new-rule name input.
    window.addEventListener('tv:alert-rules-focus-new', () => {
        const el = document.getElementById('ar-new-name');
        if (el && typeof el.focus === 'function') { el.focus(); el.select?.(); }
    });
    // View-scoped: `c` in rebalance scope → trigger Compute plan button.
    window.addEventListener('tv:rebalance-compute', () => {
        const el = document.getElementById('rb-go');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `t` in rebalance scope → focus the targets JSON editor.
    window.addEventListener('tv:rebalance-focus-targets', () => {
        const el = document.getElementById('rb-targets');
        if (el && typeof el.focus === 'function') { el.focus(); el.select?.(); }
    });
    // View-scoped: `n` in strategy-alerts scope → focus rule name input.
    window.addEventListener('tv:strategy-alerts-focus-name', () => {
        const el = document.querySelector('#sa-form input[name="name"]');
        if (el && typeof el.focus === 'function') { el.focus(); el.select?.(); }
    });
    // View-scoped: `e` in strategy-alerts scope → trigger Evaluate now.
    window.addEventListener('tv:strategy-alerts-evaluate-now', () => {
        const el = document.getElementById('sa-eval-now');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `n` in accounts scope → focus the account-name input.
    window.addEventListener('tv:accounts-focus-name', () => {
        const el = document.querySelector('#acct-form input[name="name"]');
        if (el && typeof el.focus === 'function') { el.focus(); el.select?.(); }
    });
    // View-scoped: `r` in live/trades/journal/watchlists/webull/charts
    // scopes — each refreshes the active view via hashchange.
    const refreshNow = () => window.dispatchEvent(new HashChangeEvent('hashchange'));
    window.addEventListener('tv:live-refresh',       refreshNow);
    window.addEventListener('tv:trades-refresh',     refreshNow);
    window.addEventListener('tv:journal-refresh',    refreshNow);
    window.addEventListener('tv:watchlists-refresh', refreshNow);
    window.addEventListener('tv:webull-refresh',     refreshNow);
    window.addEventListener('tv:charts-refresh',     refreshNow);
    window.addEventListener('tv:accounts-overview-refresh', refreshNow);
    window.addEventListener('tv:discipline-refresh',        refreshNow);
    window.addEventListener('tv:replay-refresh',            refreshNow);
    window.addEventListener('tv:mood-refresh',              refreshNow);
    // View-scoped: `r` in forecast scope → submit Run-forecast form.
    window.addEventListener('tv:forecast-run', () => {
        const form = document.getElementById('ef-form');
        if (form && typeof form.requestSubmit === 'function') form.requestSubmit();
    });
    // View-scoped: `a` in cohort-tilt scope → click Aggregate.
    window.addEventListener('tv:cohort-tilt-run', () => {
        const el = document.getElementById('ct-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in setups-by-setup scope → click Analyze.
    window.addEventListener('tv:setups-by-setup-run', () => {
        const el = document.getElementById('sbs-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in second-order-greeks scope → click Compute.
    window.addEventListener('tv:second-order-greeks-run', () => {
        const el = document.getElementById('sg-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in forward-vol scope → click Bootstrap.
    window.addEventListener('tv:forward-vol-run', () => {
        const el = document.getElementById('fv-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in yield-curve-pca scope → click Decompose.
    window.addEventListener('tv:yield-curve-pca-run', () => {
        const el = document.getElementById('yp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `f` in dividend-calendar scope → click Fetch dividends.
    window.addEventListener('tv:dividend-calendar-run', () => {
        const el = document.getElementById('dc-horizon');
        if (el) el.dispatchEvent(new Event('change'));
    });
    // View-scoped: `d` in signal-decomposition scope → click Decompose.
    window.addEventListener('tv:signal-decomposition-run', () => {
        const el = document.getElementById('sd-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in rr-butterfly scope → click Compute.
    window.addEventListener('tv:rr-butterfly-run', () => {
        const el = document.getElementById('rr-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in cov-denoiser scope → click Denoise.
    window.addEventListener('tv:cov-denoiser-run', () => {
        const el = document.getElementById('cd-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in almgren-chriss scope → click Solve.
    window.addEventListener('tv:almgren-chriss-run', () => {
        const el = document.getElementById('ac-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `f` in almgren-chriss scope → click Plot frontier.
    window.addEventListener('tv:almgren-chriss-frontier', () => {
        const el = document.getElementById('ac-frontier');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in implementation-shortfall scope → click Analyze.
    window.addEventListener('tv:implementation-shortfall-run', () => {
        const el = document.getElementById('is-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in spread-tracker scope → click Analyze.
    window.addEventListener('tv:spread-tracker-run', () => {
        const el = document.getElementById('st-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in spread-tracker scope → click Load demo.
    window.addEventListener('tv:spread-tracker-demo', () => {
        const el = document.getElementById('st-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in open-type scope → click Classify.
    window.addEventListener('tv:open-type-run', () => {
        const el = document.getElementById('ot-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in market-profile scope → click Build TPO.
    window.addEventListener('tv:market-profile-run', () => {
        const el = document.getElementById('mp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in market-profile scope → click Load demo.
    window.addEventListener('tv:market-profile-demo', () => {
        const el = document.getElementById('mp-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in pyramid scope → click Build plan.
    window.addEventListener('tv:pyramid-run', () => {
        const el = document.getElementById('py-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in ha-reversal scope → click Detect.
    window.addEventListener('tv:ha-reversal-run', () => {
        const el = document.getElementById('ha-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in ha-reversal scope → click Load demo.
    window.addEventListener('tv:ha-reversal-demo', () => {
        const el = document.getElementById('ha-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in three-bar-reversal scope → click Detect.
    window.addEventListener('tv:three-bar-reversal-run', () => {
        const el = document.getElementById('tbr-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in three-bar-reversal scope → click Load demo.
    window.addEventListener('tv:three-bar-reversal-demo', () => {
        const el = document.getElementById('tbr-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in range-expansion scope → click Detect.
    window.addEventListener('tv:range-expansion-run', () => {
        const el = document.getElementById('re-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in range-expansion scope → click Load demo.
    window.addEventListener('tv:range-expansion-demo', () => {
        const el = document.getElementById('re-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in alligator scope → click Compute.
    window.addEventListener('tv:alligator-run', () => {
        const el = document.getElementById('al-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in alligator scope → click Load demo.
    window.addEventListener('tv:alligator-demo', () => {
        const el = document.getElementById('al-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in demarker scope → click Compute.
    window.addEventListener('tv:demarker-run', () => {
        const el = document.getElementById('dm-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in demarker scope → click Load demo.
    window.addEventListener('tv:demarker-demo', () => {
        const el = document.getElementById('dm-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in murrey-math scope → click Compute.
    window.addEventListener('tv:murrey-math-run', () => {
        const el = document.getElementById('mm-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in murrey-math scope → click Load demo.
    window.addEventListener('tv:murrey-math-demo', () => {
        const el = document.getElementById('mm-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in demark-pivots scope → click Compute.
    window.addEventListener('tv:demark-pivots-run', () => {
        const el = document.getElementById('dp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in cypher-pattern scope → click Detect.
    window.addEventListener('tv:cypher-pattern-run', () => {
        const el = document.getElementById('cy-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in cypher-pattern scope → click Load demo.
    window.addEventListener('tv:cypher-pattern-demo', () => {
        const el = document.getElementById('cy-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in footprint scope → click Build footprint.
    window.addEventListener('tv:footprint-run', () => {
        const el = document.getElementById('fp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in footprint scope → click Load demo.
    window.addEventListener('tv:footprint-demo', () => {
        const el = document.getElementById('fp-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `r` in stress-test scope → click Run stress test.
    window.addEventListener('tv:stress-test-run', () => {
        const el = document.getElementById('st-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in stress-test scope → click Load demo.
    window.addEventListener('tv:stress-test-demo', () => {
        const el = document.getElementById('st-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in chandelier-stop scope → click Compute.
    window.addEventListener('tv:chandelier-stop-run', () => {
        const el = document.getElementById('cs-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in chandelier-stop scope → click Load demo.
    window.addEventListener('tv:chandelier-stop-demo', () => {
        const el = document.getElementById('cs-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in triple-screen scope → click Evaluate.
    window.addEventListener('tv:triple-screen-run', () => {
        const el = document.getElementById('ts-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in daily-loss-limit scope → click Evaluate.
    window.addEventListener('tv:daily-loss-limit-run', () => {
        const el = document.getElementById('dl-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in drawdown-throttle scope → click Evaluate.
    window.addEventListener('tv:drawdown-throttle-run', () => {
        const el = document.getElementById('dt-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in goal-tracker scope → click Evaluate.
    window.addEventListener('tv:goal-tracker-run', () => {
        const el = document.getElementById('gt-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in trade-plan-checklist scope → click Evaluate.
    window.addEventListener('tv:trade-plan-checklist-run', () => {
        const el = document.getElementById('tpc-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in regime-equity scope → click Analyze.
    window.addEventListener('tv:regime-equity-run', () => {
        const el = document.getElementById('re-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in vol-stop-close scope → click Compute.
    window.addEventListener('tv:vol-stop-close-run', () => {
        const el = document.getElementById('vsc-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in time-in-force scope → click Evaluate.
    window.addEventListener('tv:time-in-force-run', () => {
        const el = document.getElementById('tif-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `n` in time-in-force scope → click Snap now.
    window.addEventListener('tv:time-in-force-snap-now', () => {
        const el = document.getElementById('tif-now-snap');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in clusters-trade-features scope → click Analyze.
    window.addEventListener('tv:clusters-trade-features-run', () => {
        const el = document.getElementById('cl-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in clusters-correlation scope → click Cluster.
    window.addEventListener('tv:clusters-correlation-run', () => {
        const el = document.getElementById('cc-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in choppiness scope → click Compute.
    window.addEventListener('tv:choppiness-run', () => {
        const el = document.getElementById('cp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in var-estimator scope → click Compute.
    window.addEventListener('tv:var-estimator-run', () => {
        const el = document.getElementById('var-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in mc-trades scope → click Simulate.
    window.addEventListener('tv:mc-trades-run', () => {
        const el = document.getElementById('mct-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in commission-optimizer scope → click Evaluate.
    window.addEventListener('tv:commission-optimizer-run', () => {
        const el = document.getElementById('co-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in margin-runway scope → click Compute.
    window.addEventListener('tv:margin-runway-run', () => {
        const el = document.getElementById('mr-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in risk-parity scope → click Allocate.
    window.addEventListener('tv:risk-parity-run', () => {
        const el = document.getElementById('rp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in risk-on-off scope → click Evaluate.
    window.addEventListener('tv:risk-on-off-run', () => {
        const el = document.getElementById('ro-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in risk-reward scope → click Compute.
    window.addEventListener('tv:risk-reward-run', () => {
        const el = document.getElementById('rr-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in tax-loss-harvest scope → click Suggest.
    window.addEventListener('tv:tax-loss-harvest-run', () => {
        const el = document.getElementById('tlh-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in wash-sale scope → click Detect.
    window.addEventListener('tv:wash-sale-run', () => {
        const el = document.getElementById('ws-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in buying-power scope → click Compute.
    window.addEventListener('tv:buying-power-run', () => {
        const el = document.getElementById('bp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in margin-call scope → click Evaluate.
    window.addEventListener('tv:margin-call-run', () => {
        const el = document.getElementById('mc-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in vix-term-structure scope → click Analyze.
    window.addEventListener('tv:vix-term-structure-run', () => {
        const el = document.getElementById('vix-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in currency-exposure scope → click Analyze.
    window.addEventListener('tv:currency-exposure-run', () => {
        const el = document.getElementById('ce-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in bond-duration scope → click Compute duration.
    window.addEventListener('tv:bond-duration-run', () => {
        const el = document.getElementById('bd-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in bond-duration scope → click Build coupon bond.
    window.addEventListener('tv:bond-duration-build', () => {
        const el = document.getElementById('bd-build');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in carry-score scope → click Score.
    window.addEventListener('tv:carry-score-run', () => {
        const el = document.getElementById('cs-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in yield-curve scope → click Classify.
    window.addEventListener('tv:yield-curve-run', () => {
        const el = document.getElementById('yc-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in cost-basis scope → click Compute.
    window.addEventListener('tv:cost-basis-run', () => {
        const el = document.getElementById('cb-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `o` in cost-basis scope → click Use tax-optimal.
    window.addEventListener('tv:cost-basis-opt', () => {
        const el = document.getElementById('cb-opt');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in stop-loss-backtest scope → click Simulate.
    window.addEventListener('tv:stop-loss-backtest-run', () => {
        const el = document.getElementById('slb-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in futures-roll scope → click Build schedule.
    window.addEventListener('tv:futures-roll-run', () => {
        const el = document.getElementById('fr-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in heatmap-dow-hour scope → click Build heatmap.
    window.addEventListener('tv:heatmap-dow-hour-run', () => {
        const el = document.getElementById('hh-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `p` in atr-cone scope → click Project.
    window.addEventListener('tv:atr-cone-run', () => {
        const el = document.getElementById('ac-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in round-levels scope → click Detect.
    window.addEventListener('tv:round-levels-run', () => {
        const el = document.getElementById('rl-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in kyles-lambda scope → click Compute λ.
    window.addEventListener('tv:kyles-lambda-run', () => {
        const el = document.getElementById('kl-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in hawkes scope → click Compute λ(t).
    window.addEventListener('tv:hawkes-run', () => {
        const el = document.getElementById('hk-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in kagi scope → click Build Kagi.
    window.addEventListener('tv:kagi-run', () => {
        const el = document.getElementById('kg-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in risk-parity-solver scope → click Solve.
    window.addEventListener('tv:risk-parity-solver-run', () => {
        const el = document.getElementById('rps-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in vap scope → click Build profile.
    window.addEventListener('tv:volume-at-price-run', () => {
        const el = document.getElementById('vap-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in hhi scope → click Compute HHI.
    window.addEventListener('tv:herfindahl-run', () => {
        const el = document.getElementById('hh-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in roll-spread scope → click Compute spread.
    window.addEventListener('tv:roll-spread-run', () => {
        const el = document.getElementById('rs-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in tlb scope → click Build TLB.
    window.addEventListener('tv:three-line-break-run', () => {
        const el = document.getElementById('tlb-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `m` in mcp scope → click Manage.
    window.addEventListener('tv:momentum-crash-run', () => {
        const el = document.getElementById('mcp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in eff-spread scope → click Analyze.
    window.addEventListener('tv:effective-spread-run', () => {
        const el = document.getElementById('es-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in wmp scope → click Compute microprice.
    window.addEventListener('tv:weighted-midprice-run', () => {
        const el = document.getElementById('wmp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in mvar scope → click Analyze.
    window.addEventListener('tv:marginal-var-run', () => {
        const el = document.getElementById('mv-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in range-bar scope → click Build bars.
    window.addEventListener('tv:range-bar-run', () => {
        const el = document.getElementById('rb-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in tick-bar scope → click Build bars.
    window.addEventListener('tv:tick-bar-run', () => {
        const el = document.getElementById('tb-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in vol-bar scope → click Build bars.
    window.addEventListener('tv:volume-bar-run', () => {
        const el = document.getElementById('vb-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in abc-pattern scope → click Detect.
    window.addEventListener('tv:abc-pattern-run', () => {
        const el = document.getElementById('abc-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in absorption scope → click Detect.
    window.addEventListener('tv:absorption-run', () => {
        const el = document.getElementById('abs-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in acf scope → click Compute ACF.
    window.addEventListener('tv:acf-run', () => {
        const el = document.getElementById('ac-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in active-share scope → click Compute.
    window.addEventListener('tv:active-share-run', () => {
        const el = document.getElementById('as-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in ad-oscillator scope → click Compute.
    window.addEventListener('tv:ad-oscillator-run', () => {
        const el = document.getElementById('ao-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in adf scope → click Run ADF.
    window.addEventListener('tv:adf-test-run', () => {
        const el = document.getElementById('adf-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in ad-normality scope → click Test.
    window.addEventListener('tv:ad-normality-run', () => {
        const el = document.getElementById('adn-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in adl scope → click Compute.
    window.addEventListener('tv:adl-run', () => {
        const el = document.getElementById('ad-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in alma scope → click Compute.
    window.addEventListener('tv:alma-run', () => {
        const el = document.getElementById('al-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in amihud scope → click Compute Amihud.
    window.addEventListener('tv:amihud-run', () => {
        const el = document.getElementById('am-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in arch-lm scope → click Test.
    window.addEventListener('tv:arch-lm-run', () => {
        const el = document.getElementById('arl-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in alphatrend scope → click Compute.
    window.addEventListener('tv:alphatrend-run', () => {
        const el = document.getElementById('at-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in aroon scope → click Compute Aroon.
    window.addEventListener('tv:aroon-run', () => {
        const el = document.getElementById('ar-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in anchored-momentum scope → click Compute.
    window.addEventListener('tv:anchored-momentum-run', () => {
        const el = document.getElementById('am-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in atr-channel scope → click Compute.
    window.addEventListener('tv:atr-channel-run', () => {
        const el = document.getElementById('ac-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in bop scope → click Compute BOP.
    window.addEventListener('tv:balance-of-power-run', () => {
        const el = document.getElementById('bp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in asi scope → click Compute.
    window.addEventListener('tv:asi-run', () => {
        const el = document.getElementById('as-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in atr-trailing-stop scope → click Compute.
    window.addEventListener('tv:atr-trailing-stop-run', () => {
        const el = document.getElementById('ts-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in bartlett scope → click Test.
    window.addEventListener('tv:bartlett-run', () => {
        const el = document.getElementById('bt-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in bollinger-percent-b scope → click Compute.
    window.addEventListener('tv:bb-pb-run', () => {
        const el = document.getElementById('pb-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in bollinger-band-width scope → click Compute.
    window.addEventListener('tv:bbw-run', () => {
        const el = document.getElementById('bw-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in bollinger-oscillators scope → click Compute.
    window.addEventListener('tv:bb-osc-run', () => {
        const el = document.getElementById('bo-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in bollinger-bandwidth-percentile scope → click Compute.
    window.addEventListener('tv:bbwp-run', () => {
        const el = document.getElementById('bp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in bollinger-band-distance scope → click Compute.
    window.addEventListener('tv:bbd-run', () => {
        const el = document.getElementById('bd-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in beta-shrinkage scope → click Shrink.
    window.addEventListener('tv:beta-shrink-run', () => {
        const el = document.getElementById('bs-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in breusch-godfrey scope → click Test.
    window.addEventListener('tv:bg-test-run', () => {
        const el = document.getElementById('bg-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in beta scope → click Estimate β.
    window.addEventListener('tv:beta-run', () => {
        const el = document.getElementById('bt-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in bid-ask-volume-ratio scope → click Compute.
    window.addEventListener('tv:bid-ask-vol-run', () => {
        const el = document.getElementById('bv-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in bpv scope → click Compute BPV.
    window.addEventListener('tv:bipower-variation-run', () => {
        const el = document.getElementById('bv-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in borrow-rate-indicator scope → click Compute.
    window.addEventListener('tv:borrow-rate-run', () => {
        const el = document.getElementById('br-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in black-litterman scope → click Solve posterior.
    window.addEventListener('tv:black-litterman-run', () => {
        const el = document.getElementById('bl-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `r` in block-bootstrap scope → click Resample.
    window.addEventListener('tv:block-bootstrap-run', () => {
        const el = document.getElementById('bb-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `r` in bootstrap-pnl scope → click Resample.
    window.addEventListener('tv:bootstrap-pnl-run', () => {
        const el = document.getElementById('bp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in bb-squeeze scope → click Detect squeeze.
    window.addEventListener('tv:bollinger-squeeze-run', () => {
        const el = document.getElementById('bs-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in breusch-pagan scope → click Test.
    window.addEventListener('tv:bp-test-run', () => {
        const el = document.getElementById('bp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in breadth-thrust scope → click Detect thrust.
    window.addEventListener('tv:breadth-thrust-run', () => {
        const el = document.getElementById('bt-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `r` in market-breadth scope → click Refresh.
    window.addEventListener('tv:market-breadth-refresh', () => {
        const el = document.getElementById('mb-refresh');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `n` in developer scope → focus token-name input.
    window.addEventListener('tv:developer-focus-name', () => {
        const el = document.querySelector('#tok-form input[name="name"]');
        if (el && typeof el.focus === 'function') { el.focus(); el.select?.(); }
    });
    // View-scoped: `g` in developer scope → submit token-form (Generate).
    window.addEventListener('tv:developer-generate', () => {
        const form = document.getElementById('tok-form');
        if (form && typeof form.requestSubmit === 'function') form.requestSubmit();
    });
    // View-scoped: `r` in backtest scope → submit backtest form (Run).
    window.addEventListener('tv:backtest-run', () => {
        const form = document.getElementById('bt-form');
        if (form && typeof form.requestSubmit === 'function') form.requestSubmit();
    });
    // View-scoped: `n` in backtest-presets scope → focus preset-name input.
    window.addEventListener('tv:backtest-presets-focus-name', () => {
        const el = document.querySelector('#bp-form input[name="name"]');
        if (el && typeof el.focus === 'function') { el.focus(); el.select?.(); }
    });
    // View-scoped: `u` in csv-wizard scope → open the file picker.
    window.addEventListener('tv:csv-wizard-upload', () => {
        const el = document.getElementById('cw-file');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `n` in boards scope (list view) → focus board-name input.
    window.addEventListener('tv:boards-focus-name', () => {
        const el = document.querySelector('#b-new input[name="name"]');
        if (el && typeof el.focus === 'function') { el.focus(); el.select?.(); }
    });
    // View-scoped: `p` in import scope → click the dropzone (opens file picker).
    window.addEventListener('tv:import-pick-file', () => {
        const el = document.getElementById('drop');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `u` in import scope → click the Upload button.
    window.addEventListener('tv:import-upload', () => {
        const el = document.getElementById('go');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in ai scope → submit AI-settings form.
    window.addEventListener('tv:ai-save', () => {
        const form = document.getElementById('ai-form');
        if (form && typeof form.requestSubmit === 'function') form.requestSubmit();
    });
    // View-scoped: `n` in community scope → focus new-thread title input.
    window.addEventListener('tv:community-focus-title', () => {
        const el = document.querySelector('#thread-form input[name="title"]');
        if (el && typeof el.focus === 'function') { el.focus(); el.select?.(); }
    });
    // View-scoped: `n` in goals scope → focus new-goal name input.
    window.addEventListener('tv:goals-focus-name', () => {
        const el = document.querySelector('#g-form input[name="name"]');
        if (el && typeof el.focus === 'function') { el.focus(); el.select?.(); }
    });
    // View-scoped: `s` in journal scope → click Save button.
    window.addEventListener('tv:journal-save', () => {
        const el = document.getElementById('save');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `n` in hotkeys scope → focus binding-name input.
    window.addEventListener('tv:hotkeys-focus-name', () => {
        const el = document.querySelector('#hk-form input[name="name"]');
        if (el && typeof el.focus === 'function') { el.focus(); el.select?.(); }
    });
    // View-scoped: `c` in hotkeys scope → click Capture combo button.
    window.addEventListener('tv:hotkeys-capture', () => {
        const el = document.getElementById('capture');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in paper scope → submit order-ticket form.
    window.addEventListener('tv:paper-submit', () => {
        const form = document.getElementById('ord-form');
        if (form && typeof form.requestSubmit === 'function') form.requestSubmit();
    });
    // View-scoped: `r` in screener scope → submit screener form (Run).
    window.addEventListener('tv:screener-run', () => {
        const form = document.getElementById('sc-form');
        if (form && typeof form.requestSubmit === 'function') form.requestSubmit();
    });
    // View-scoped: `n` in dashboards scope → focus new-dashboard name input.
    window.addEventListener('tv:dashboards-focus-new', () => {
        const el = document.getElementById('db-new-name');
        if (el && typeof el.focus === 'function') { el.focus(); el.select?.(); }
    });
    // View-scoped: `e` in dashboards scope → toggle Edit-layout button.
    window.addEventListener('tv:dashboards-toggle-edit', () => {
        const el = document.getElementById('db-edit');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in new-trade scope → submit execution form (Add).
    window.addEventListener('tv:new-trade-add', () => {
        const form = document.getElementById('ex-form');
        if (form && typeof form.requestSubmit === 'function') form.requestSubmit();
    });
    // View-scoped: `r` in research scope → submit search form if in
    // search-mode, else re-fetch the active research page.
    window.addEventListener('tv:research-action', () => {
        const form = document.getElementById('rs-form');
        if (form && typeof form.requestSubmit === 'function') {
            form.requestSubmit();
        } else {
            window.dispatchEvent(new HashChangeEvent('hashchange'));
        }
    });
    // View-scoped: `r` in economy scope → submit calendar form (Load).
    window.addEventListener('tv:economy-load', () => {
        const form = document.getElementById('ec-form');
        if (form && typeof form.requestSubmit === 'function') form.requestSubmit();
    });
    // View-scoped: `r` in earnings-cal scope → submit refresh-view form.
    window.addEventListener('tv:earnings-cal-refresh', () => {
        const form = document.getElementById('e-form');
        if (form && typeof form.requestSubmit === 'function') form.requestSubmit();
    });
    // View-scoped: `p` in earnings-cal scope → click Poll-now Yahoo button.
    window.addEventListener('tv:earnings-cal-poll', () => {
        const el = document.getElementById('e-poll');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `r` in monte-carlo scope → click Run button.
    window.addEventListener('tv:monte-carlo-run', () => {
        const el = document.getElementById('mc-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in kelly scope → click Compute-static button.
    window.addEventListener('tv:kelly-compute-static', () => {
        const el = document.getElementById('kl-run-static');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in kelly scope → click Compute-dynamic button.
    window.addEventListener('tv:kelly-compute-dynamic', () => {
        const el = document.getElementById('kl-run-dyn');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in risk scope → submit limits form.
    window.addEventListener('tv:risk-save', () => {
        const form = document.getElementById('risk-form');
        if (form && typeof form.requestSubmit === 'function') form.requestSubmit();
    });
    // View-scoped: `r` in darkpool scope — submit the rank form in list
    // mode, else hashchange-refresh the symbol-detail page.
    window.addEventListener('tv:darkpool-rank', () => {
        const form = document.getElementById('rf');
        if (form && typeof form.requestSubmit === 'function') {
            form.requestSubmit();
        } else {
            window.dispatchEvent(new HashChangeEvent('hashchange'));
        }
    });
    // View-scoped: `c` in var-calculator scope → click Compute button.
    window.addEventListener('tv:var-calculator-compute', () => {
        const el = document.getElementById('vc-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in portfolio-allocator scope → click Allocate.
    window.addEventListener('tv:portfolio-allocator-run', () => {
        const el = document.getElementById('pa-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in live-scanner scope → submit configure form.
    window.addEventListener('tv:live-scanner-connect', () => {
        const form = document.getElementById('ls-config');
        if (form && typeof form.requestSubmit === 'function') form.requestSubmit();
    });
    // View-scoped: `v` in live-scanner scope → toggle voice-alert checkbox.
    window.addEventListener('tv:live-scanner-toggle-voice', () => {
        const el = document.getElementById('ls-voice');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `r` in top-signals scope → submit refresh form.
    window.addEventListener('tv:top-signals-refresh', () => {
        const form = document.getElementById('top-form');
        if (form && typeof form.requestSubmit === 'function') form.requestSubmit();
    });
    // View-scoped: `a` in pair-trade-calc scope → click Analyze button.
    window.addEventListener('tv:pair-trade-analyze', () => {
        const el = document.getElementById('pt-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `f` in vol-smile scope → click Fit button.
    window.addEventListener('tv:vol-smile-fit', () => {
        const el = document.getElementById('vs-fit');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `r` in option-payoff scope → click Recalculate.
    window.addEventListener('tv:option-payoff-recalc', () => {
        const el = document.getElementById('op-recalc');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in series-smoother scope → click Smooth.
    window.addEventListener('tv:series-smoother-run', () => {
        const el = document.getElementById('ss-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in pattern-discovery scope → click Discover.
    window.addEventListener('tv:pattern-discovery-run', () => {
        const el = document.getElementById('pd-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in execution-scheduler scope → click Schedule.
    window.addEventListener('tv:execution-scheduler-run', () => {
        const el = document.getElementById('es-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in regime-detector scope → click Detect.
    window.addEventListener('tv:regime-detector-run', () => {
        const el = document.getElementById('rd-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `p` in american-option scope → click Price.
    window.addEventListener('tv:american-option-price', () => {
        const el = document.getElementById('ao-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `p` in fx-option scope → click Price.
    window.addEventListener('tv:fx-option-price', () => {
        const el = document.getElementById('fx-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in greeks-profile scope → click Compute.
    window.addEventListener('tv:greeks-profile-compute', () => {
        const el = document.getElementById('gp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in iv-solver scope → click Solve IV.
    window.addEventListener('tv:iv-solver-solve', () => {
        const el = document.getElementById('iv-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `r` in kalman-beta scope → click Run.
    window.addEventListener('tv:kalman-beta-run', () => {
        const el = document.getElementById('kb-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in optimal-f scope → click Compute.
    window.addEventListener('tv:optimal-f-compute', () => {
        const el = document.getElementById('of-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `w` in dtw scope → click Warp.
    window.addEventListener('tv:dtw-warp', () => {
        const el = document.getElementById('dt-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in hurst scope → click Estimate.
    window.addEventListener('tv:hurst-estimate', () => {
        const el = document.getElementById('hu-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in bocpd scope → click Detect.
    window.addEventListener('tv:bocpd-detect', () => {
        const el = document.getElementById('bo-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in vasicek scope → click Simulate.
    window.addEventListener('tv:vasicek-simulate', () => {
        const el = document.getElementById('va-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in microprice scope → click Compute.
    window.addEventListener('tv:microprice-compute', () => {
        const el = document.getElementById('mp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in vpin scope → click Compute VPIN.
    window.addEventListener('tv:vpin-compute', () => {
        const el = document.getElementById('vp-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in vpin scope → click Load demo.
    window.addEventListener('tv:vpin-demo', () => {
        const el = document.getElementById('vp-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in deflated-sharpe scope → click Deflate.
    window.addEventListener('tv:deflated-sharpe-compute', () => {
        const el = document.getElementById('ds-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `s` in deflated-sharpe scope → click Trials sweep.
    window.addEventListener('tv:deflated-sharpe-sweep', () => {
        const el = document.getElementById('ds-sweep');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in cup-and-handle scope → click Detect.
    window.addEventListener('tv:cup-and-handle-detect', () => {
        const el = document.getElementById('ch-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in cup-and-handle scope → click Load demo.
    window.addEventListener('tv:cup-and-handle-demo', () => {
        const el = document.getElementById('ch-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in iv-rank scope → click Compute.
    window.addEventListener('tv:iv-rank-compute', () => {
        const el = document.getElementById('iv-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in iv-rank scope → click Load demo.
    window.addEventListener('tv:iv-rank-demo', () => {
        const el = document.getElementById('iv-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in market-impact scope → click Analyze.
    window.addEventListener('tv:market-impact-analyze', () => {
        const el = document.getElementById('mi-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in market-impact scope → click Load demo.
    window.addEventListener('tv:market-impact-demo', () => {
        const el = document.getElementById('mi-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in liquidity scope → click Analyze.
    window.addEventListener('tv:liquidity-analyze', () => {
        const el = document.getElementById('lq-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in liquidity scope → click Load demo.
    window.addEventListener('tv:liquidity-demo', () => {
        const el = document.getElementById('lq-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in intraday-heatmap scope → click Build heatmap.
    window.addEventListener('tv:intraday-heatmap-build', () => {
        const el = document.getElementById('ih-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in intraday-heatmap scope → click Load demo.
    window.addEventListener('tv:intraday-heatmap-demo', () => {
        const el = document.getElementById('ih-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `b` in iv-backtest scope → click Backtest.
    window.addEventListener('tv:iv-backtest-run', () => {
        const el = document.getElementById('ib-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in iv-backtest scope → click Load demo.
    window.addEventListener('tv:iv-backtest-demo', () => {
        const el = document.getElementById('ib-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in order-book-imbalance scope → click Compute.
    window.addEventListener('tv:obi-compute', () => {
        const el = document.getElementById('obi-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `d` in cusum scope → click Detect.
    window.addEventListener('tv:cusum-detect', () => {
        const el = document.getElementById('cu-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in cusum scope → click Auto-fit mean/stdev.
    window.addEventListener('tv:cusum-autofit', () => {
        const el = document.getElementById('cu-autofit');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `c` in order-flow scope → click Classify.
    window.addEventListener('tv:order-flow-classify', () => {
        const el = document.getElementById('of-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in order-flow scope → click Load demo.
    window.addEventListener('tv:order-flow-demo', () => {
        const el = document.getElementById('of-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in vwap-slippage scope → click Analyze.
    window.addEventListener('tv:vwap-slippage-analyze', () => {
        const el = document.getElementById('vw-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in vwap-slippage scope → click Load demo.
    window.addEventListener('tv:vwap-slippage-demo', () => {
        const el = document.getElementById('vw-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `a` in per-symbol-slippage scope → click Aggregate.
    window.addEventListener('tv:per-symbol-slippage-run', () => {
        const el = document.getElementById('ps-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in per-symbol-slippage scope → click Load demo.
    window.addEventListener('tv:per-symbol-slippage-demo', () => {
        const el = document.getElementById('ps-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `e` in order-staleness scope → click Evaluate.
    window.addEventListener('tv:order-staleness-evaluate', () => {
        const el = document.getElementById('os-run');
        if (el && typeof el.click === 'function') el.click();
    });
    // View-scoped: `l` in order-staleness scope → click Load demo.
    window.addEventListener('tv:order-staleness-demo', () => {
        const el = document.getElementById('os-demo');
        if (el && typeof el.click === 'function') el.click();
    });
    // Quick-nav globals — Cmd/Ctrl+Option/Alt+<letter> → hash route.
    window.addEventListener('tv:nav-trades',      () => { window.location.hash = 'trades'; });
    window.addEventListener('tv:nav-journal',     () => { window.location.hash = 'journal'; });
    window.addEventListener('tv:nav-dashboard',   () => { window.location.hash = 'dashboard'; });
    window.addEventListener('tv:nav-watchlists',  () => { window.location.hash = 'watchlists'; });
    window.addEventListener('tv:nav-charts',      () => { window.location.hash = 'charts'; });
    window.addEventListener('tv:nav-live',        () => { window.location.hash = 'live'; });
    window.addEventListener('tv:nav-reports',     () => { window.location.hash = 'reports'; });
    window.addEventListener('tv:nav-scanner',     () => { window.location.hash = 'live-scanner'; });
    // Session-added expense/tax/budget nav.
    window.addEventListener('tv:nav-expenses',    () => { window.location.hash = 'expenses'; });
    window.addEventListener('tv:nav-receipts',    () => { window.location.hash = 'receipts'; });
    window.addEventListener('tv:nav-purchases',   () => { window.location.hash = 'purchases'; });
    window.addEventListener('tv:nav-categorize',  () => { window.location.hash = 'categorize'; });
    window.addEventListener('tv:nav-file-taxes',  () => { window.location.hash = 'file-taxes'; });
    window.addEventListener('tv:nav-budget',      () => { window.location.hash = 'budget'; });
    // More-menu nav targets that previously had no shortcut.
    window.addEventListener('tv:nav-accounts',       () => { window.location.hash = 'accounts'; });
    window.addEventListener('tv:nav-calendar',       () => { window.location.hash = 'calendar'; });
    window.addEventListener('tv:nav-catalysts',      () => { window.location.hash = 'catalysts'; });
    window.addEventListener('tv:nav-dashboards',     () => { window.location.hash = 'dashboards'; });
    window.addEventListener('tv:nav-goals',          () => { window.location.hash = 'goals'; });
    window.addEventListener('tv:nav-halts',          () => { window.location.hash = 'halts'; });
    window.addEventListener('tv:nav-after-hours',    () => { window.location.hash = 'after-hours'; });
    window.addEventListener('tv:nav-note-templates', () => { window.location.hash = 'note-templates'; });
    window.addEventListener('tv:nav-reviews',        () => { window.location.hash = 'reviews'; });
    window.addEventListener('tv:nav-risk-gate',      () => { window.location.hash = 'risk-gate'; });
    window.addEventListener('tv:nav-search',         () => { window.location.hash = 'search'; });
    window.addEventListener('tv:nav-tags',           () => { window.location.hash = 'tags'; });
    window.addEventListener('tv:nav-webull',         () => { window.location.hash = 'webull'; });
    // Toast on HUD toggles so keyboard-only users see feedback (the
    // visible change can be subtle in some scheme combos).
    window.addEventListener('tv:hud-toggled', (e) => {
        void (async () => {
            try {
                const toast = await import('./toast.js');
                const i18n  = await import('./i18n.js');
                const d = e && e.detail || {};
                const kind = d.kind;
                const on = !!d.on;
                if (!['theme', 'crt', 'neon'].includes(kind)) return;
                // theme reports `on=true` for LIGHT (since light is the "on" state in the emitter)
                // but the user-facing label should say "Light" / "Dark".
                let msg;
                if (kind === 'theme') {
                    msg = i18n.t(on ? 'toast.theme_light' : 'toast.theme_dark');
                } else {
                    msg = i18n.t(on ? `toast.${kind}_on` : `toast.${kind}_off`);
                }
                toast.showToast(msg, { level: 'success' });
            } catch (_) { /* toast/i18n unavailable */ }
        })();
    });
    window.addEventListener('tv:cycle-locale', () => {
        const picker = document.getElementById('locale-picker');
        if (!picker || picker.options.length === 0) return;
        const next = (picker.selectedIndex + 1) % picker.options.length;
        picker.selectedIndex = next;
        picker.dispatchEvent(new Event('change'));
    });
    window.addEventListener('tv:open-settings', () => { window.location.hash = 'settings'; });
    window.addEventListener('tv:focus-search', () => {
        // Per-view priority: explicit chip opt-in via data-shortcut →
        // launcher / shortcuts / palette inputs → type=search →
        // placeholder hinting search/filter/find. Bails silently if none.
        // The data-shortcut='focus_search' selector lets any view opt
        // an input in by adding the chip — same attribute that surfaces
        // the keybind in the tooltip via augmentShortcutTitles.
        const candidates = [
            'input[data-shortcut="focus_search"]:not([disabled])',
            '#launcher-q', '#ks-filter', '#palette-input',
            'input[type=search]:not([disabled])',
            'input:not([disabled])[placeholder*="search" i]',
            'input:not([disabled])[placeholder*="filter" i]',
            'input:not([disabled])[placeholder*="find" i]',
        ];
        for (const sel of candidates) {
            const el = document.querySelector(sel);
            if (el && typeof el.focus === 'function') {
                el.focus();
                if (typeof el.select === 'function') el.select();
                return;
            }
        }
    });
    window.addEventListener('tv:clear-recents', () => {
        void (async () => {
            try {
                const r = await import('./_recents_storage.js');
                const toast = await import('./toast.js');
                const i18n = await import('./i18n.js');
                r.saveState(r.clearRecents(r.loadState()));
                toast.showToast(i18n.t('toast.recents_cleared'), { level: 'success' });
                // Re-paint launcher if we're on it.
                if ((window.location.hash || '').replace(/^#/, '').split('/')[0] === 'launcher') {
                    window.dispatchEvent(new HashChangeEvent('hashchange'));
                }
            } catch (_) { /* storage / toast unavailable */ }
        })();
    });
}

// Cmd+K + ? are now owned by ./shortcuts.js → tv:open-palette /
// tv:open-help events. ./command_palette.js handles the overlay.

function bindNavToggle() {
    bindTopPaletteButton();
    const btn = document.getElementById('navToggle');
    if (!btn) return;
    btn.addEventListener('click', () => {
        const open = document.body.classList.toggle('nav-open');
        btn.setAttribute('aria-expanded', open ? 'true' : 'false');
    });
    // Close drawer if the viewport widens back past the breakpoint, so the
    // drawer doesn't leave the body in `nav-open` state when switching to
    // desktop layout.
    const mql = window.matchMedia('(min-width: 901px)');
    const onWidthChange = (e) => { if (e.matches) closeNavDrawer(); };
    if (mql.addEventListener) mql.addEventListener('change', onWidthChange);
    else if (mql.addListener) mql.addListener(onWidthChange);
    // Tap outside the drawer closes it.
    document.addEventListener('click', (e) => {
        if (!document.body.classList.contains('nav-open')) return;
        const tabs = document.querySelector('.tabs');
        if (!tabs.contains(e.target) && !btn.contains(e.target)) {
            closeNavDrawer();
        }
    });
}

// Topbar palette button — used to live as `onclick=` in index.html, which
// the release-build CSP refuses to execute. Bind via JS instead.
function bindTopPaletteButton() {
    const btn = document.getElementById('topPalette');
    if (!btn) return;
    btn.addEventListener('click', () => {
        window.dispatchEvent(new CustomEvent('tv:open-palette'));
    });
}

function wireWsStatusIndicator() {
    const dot = document.getElementById('wsStatus');
    if (!dot) return;
    const set = (cls, title) => {
        dot.className = `ws-status ${cls}`;
        dot.title = title;
    };
    set('warn', 'connecting…');
    onWsEvent('_open',  () => set('on',  'real-time stream connected'));
    onWsEvent('_close', () => set('off', 'real-time stream disconnected — reconnecting'));
    onWsEvent('ping',   () => set('on',  `real-time stream alive @ ${new Date().toLocaleTimeString(undefined, { hour12: false })}`));
}

/// Topbar 🛑 indicator. Polls /api/risk-gate/kill-switch every 30s and
/// shows the icon only when the switch is ACTIVE — so the user can always
/// see when trading is halted, regardless of which view they're on.
function wireKillSwitchIndicator() {
    const el = document.getElementById('killSwitchTop');
    if (!el) return;
    const tick = async () => {
        try {
            const s = await api.riskKillSwitchState();
            el.style.display = s.active ? 'inline' : 'none';
        } catch (_) { /* stay quiet on transient failures */ }
    };
    tick();
    setInterval(tick, 30_000);
}

function closeNavDrawer() {
    document.body.classList.remove('nav-open');
    const btn = document.getElementById('navToggle');
    if (btn) btn.setAttribute('aria-expanded', 'false');
}

export function go(view, params = '') {
    window.location.hash = view + (params ? `/${params}` : '');
}

// Re-export the view-token machinery so the 80+ views that already import
// it from app.js keep working. Implementation lives in `view_token.js` so
// `node --test` can unit-test the semantics without pulling in the DOM.
import { bumpViewToken, currentViewToken, viewIsCurrent } from './view_token.js';
export { currentViewToken, viewIsCurrent };

export async function dispatch() {
    // Invalidate every captured token from the previous view — pending awaits,
    // queued WS reconnects, and setInterval ticks will see a stale token and
    // skip the work that would otherwise reach into the wrong view's DOM.
    bumpViewToken();
    const hash = (window.location.hash || '#launcher').slice(1);
    let [view, ...rest] = hash.split('/');
    // Strip optional `?...` query string off the LAST segment so a view
    // like `#file-taxes?year=2025&section=income` routes to `file-taxes`,
    // not the whole string. Individual views read `location.hash` and
    // parse their own query params (e.g. file_taxes.js reads ?year=).
    if (rest.length === 0) {
        const q = view.indexOf('?');
        if (q >= 0) view = view.slice(0, q);
    } else {
        const last = rest[rest.length - 1];
        const q = last.indexOf('?');
        if (q >= 0) rest[rest.length - 1] = last.slice(0, q);
    }
    // Popout mode — a route like `#popout/<viewId>/<...rest>` renders the
    // target view ALONE on the page (no topbar, no account strip, no
    // launcher chrome) so the user can tear it off into its own
    // window for multi-monitor work. Strip the `popout/` prefix and
    // toggle a body class that the stylesheet uses to hide chrome.
    if (view === 'popout' && rest.length > 0) {
        document.body.classList.add('popout-mode');
        view = rest.shift();
    } else {
        document.body.classList.remove('popout-mode');
    }
    // Track recents (pure, localStorage-backed). Best-effort; never blocks
    // the dispatch path. Skipped views (launcher, keyboard-shortcuts, …)
    // are filtered inside push().
    try {
        const r = await import('./_recents_storage.js');
        r.saveState(r.push(r.loadState(), view));
    } catch (_) { /* storage unavailable */ }
    // For symbol-aware views, fall back to the global ticker store when
    // the URL doesn't carry one. Lets the user type a ticker once on
    // any page, then navigate freely — every symbol-aware view picks
    // it up automatically.
    const sym = () => rest[0] || getGlobalSymbol() || '';
    state.view = view;
    document.querySelectorAll('.tab').forEach(b =>
        b.classList.toggle('active', b.dataset.view === view)
    );
    // "More ▾" trigger lights up when the active view lives inside its
    // dropdown — same visual signal the user gets on a primary tab.
    const moreTrigger = document.getElementById('nav-more-btn');
    if (moreTrigger) {
        const inMenu = !!document.querySelector(
            `.tab-more-item[data-view="${view}"]`
        );
        moreTrigger.classList.toggle('active', inMenu);
    }
    const mount = document.getElementById('app');
    // View-wide context-menu scope: every right-click inside #app now
    // resolves to the current view's slug via nearestScope() walk-up.
    // Inner `data-context-scope` on chart-panel elements still wins
    // for granular scopes (it's the closer ancestor).
    mount.setAttribute('data-context-scope', view);
    // Shortcut scope follows the active view so future view-specific
    // bindings (registered with scope: view) can fire here without
    // bleeding into other views.
    setScope(view);
    // Browser tab title: "TraderView • <localized view label>". Falls
    // back to the view slug for routes without a tile (trade/X, etc.).
    if (typeof document !== 'undefined') {
        const labelKey = `tile.${view}.label`;
        const lab = t(labelKey);
        const label = (lab && lab !== labelKey) ? lab : view;
        document.title = `TraderView • ${label}`;
    }
    mount.innerHTML = spinnerHTML(t('common.loading_view', { view }));
    try {
        switch (view) {
            case 'launcher':    await renderLauncher(mount, state); break;
            case 'dashboard':   await renderDashboard(mount, state); break;
            case 'trades':      await renderTradesView(mount, state); break;
            case 'new-trade':   await renderNewTrade(mount, state); break;
            case 'search':      await renderSearch(mount, state); break;
            case 'watchlists':  await renderWatchlists(mount, state); break;
            case 'research':    await renderResearch(mount, state, sym()); break;
            case 'screener':    await renderScreener(mount, state); break;
            case 'top-signals': await renderTopSignals(mount, state); break;
            case 'scanners':    await renderScanners(mount, state); break;
            case 'sectors':     await renderSectors(mount, state); break;
            case 'paper':       await renderPaper(mount, state); break;
            case 'risk':        await renderRisk(mount, state); break;
            case 'alerts':      await renderAlerts(mount, state); break;
            case 'hotkeys':     await renderHotkeys(mount, state); break;
            case 'replay':      await renderReplay(mount, state, sym()); break;
            case 'tape':        await renderTape(mount, state); break;
            case 'earnings-iv': await renderEarningsIv(mount, state, sym()); break;
            case 'disclosures': await renderDisclosures(mount, state); break;
            case 'sentiment':   await renderSentiment(mount, state, sym()); break;
            case 'heatmap':     await renderHeatmap(mount, state); break;
            case 'options':     await renderOptions(mount, state, sym()); break;
            case 'option-payoff': await renderOptionPayoff(mount, state); break;
            case 'vol-smile':     await renderVolSmile(mount, state); break;
            case 'monte-carlo':   await renderMonteCarlo(mount, state); break;
            case 'portfolio-allocator': await renderPortfolioAllocator(mount, state); break;
            case 'var-calculator': await renderVarCalculator(mount, state); break;
            case 'series-smoother': await renderSeriesSmoother(mount, state); break;
            case 'pattern-discovery': await renderPatternDiscovery(mount, state); break;
            case 'execution-scheduler': await renderExecutionScheduler(mount, state); break;
            case 'regime-detector': await renderRegimeDetector(mount, state); break;
            case 'american-option': await renderAmericanOption(mount, state); break;
            case 'fx-option':       await renderFxOption(mount, state); break;
            case 'forward-vol':     await renderForwardVolCurve(mount, state); break;
            case 'yield-curve-pca': await renderYieldCurvePca(mount, state); break;
            case 'dividend-calendar': await renderDividendCalendar(mount, state); break;
            case 'signal-decomposition': await renderSignalDecomposition(mount, state); break;
            case 'rr-butterfly':    await renderRrButterfly(mount, state); break;
            case 'cov-denoiser':    await renderCovDenoiser(mount, state); break;
            case 'microprice':      await renderMicroprice(mount, state); break;
            case 'dtw':             await renderDtw(mount, state); break;
            case 'hurst':           await renderHurst(mount, state); break;
            case 'bocpd':           await renderBocpd(mount, state); break;
            case 'vasicek':         await renderVasicek(mount, state); break;
            case 'optimal-f':       await renderOptimalF(mount, state); break;
            case 'kalman-beta':     await renderKalmanBeta(mount, state); break;
            case 'pair-trade-calc': await renderPairTrade(mount, state); break;
            case 'iv-solver':       await renderIvSolver(mount, state); break;
            case 'greeks-profile':  await renderGreeksProfile(mount, state); break;
            case 'second-order-greeks': await renderSecondOrderGreeks(mount, state); break;
            case 'almgren-chriss':  await renderAlmgrenChriss(mount, state); break;
            case 'implementation-shortfall': await renderImplementationShortfall(mount, state); break;
            case 'deflated-sharpe': await renderDeflatedSharpe(mount, state); break;
            case 'vpin':            await renderVpin(mount, state); break;
            case 'cup-and-handle':  await renderCupAndHandle(mount, state); break;
            case 'iv-rank':         await renderIvRank(mount, state); break;
            case 'market-impact':   await renderMarketImpact(mount, state); break;
            case 'liquidity':       await renderLiquidity(mount, state); break;
            case 'spread-tracker':  await renderSpreadTracker(mount, state); break;
            case 'intraday-heatmap': await renderIntradayHeatmap(mount, state); break;
            case 'iv-backtest':     await renderIvBacktest(mount, state); break;
            case 'order-book-imbalance': await renderOrderBookImbalance(mount, state); break;
            case 'cusum':           await renderCusum(mount, state); break;
            case 'order-flow':      await renderOrderFlow(mount, state); break;
            case 'vwap-slippage':   await renderVwapSlippage(mount, state); break;
            case 'per-symbol-slippage': await renderPerSymbolSlippage(mount, state); break;
            case 'order-staleness': await renderOrderStaleness(mount, state); break;
            case 'open-type':       await renderOpenType(mount, state); break;
            case 'market-profile':  await renderMarketProfile(mount, state); break;
            case 'oi-change':       await renderOiChange(mount, state); break;
            case 'pyramid':         await renderPyramid(mount, state); break;
            case 'ha-reversal':     await renderHaReversal(mount, state); break;
            case 'three-bar-reversal': await renderThreeBarReversal(mount, state); break;
            case 'range-expansion':    await renderRangeExpansion(mount, state); break;
            case 'alligator':          await renderAlligator(mount, state); break;
            case 'demarker':           await renderDemarker(mount, state); break;
            case 'murrey-math':        await renderMurreyMath(mount, state); break;
            case 'demark-pivots':      await renderDemarkPivots(mount, state); break;
            case 'cypher-pattern':     await renderCypherPattern(mount, state); break;
            case 'dashboards':         await renderDashboards(mount, state); break;
            case 'twap':               await renderTwap(mount, state); break;
            case 'news-event':         await renderNewsEvent(mount, state); break;
            case 'stop-loss-best-of':  await renderStopLossBestOf(mount, state); break;
            case 'squeeze-alerts':     await renderSqueezeAlerts(mount, state); break;
            case 'footprint':          await renderFootprint(mount, state); break;
            case 'stress-test':        await renderStressTest(mount, state); break;
            case 'chandelier-stop':    await renderChandelierStop(mount, state); break;
            case 'triple-screen':      await renderTripleScreen(mount, state); break;
            case 'alert-rules':        await renderAlertRules(mount, state); break;
            case 'daily-loss-limit':   await renderDailyLossLimit(mount, state); break;
            case 'drawdown-throttle':  await renderDrawdownThrottle(mount, state); break;
            case 'goal-tracker':       await renderGoalTracker(mount, state); break;
            case 'trade-plan-checklist': await renderTradePlanChecklist(mount, state); break;
            case 'regime-equity':      await renderRegimeEquity(mount, state); break;
            case 'vol-stop-close':     await renderVolStopClose(mount, state); break;
            case 'time-in-force':      await renderTimeInForce(mount, state); break;
            case 'clusters-trade-features': await renderClustersTradeFeatures(mount, state); break;
            case 'clusters-correlation': await renderClustersCorrelation(mount, state); break;
            case 'setups-by-setup':    await renderSetupsBySetup(mount, state); break;
            case 'cohort-tilt':        await renderCohortTilt(mount, state); break;
            case 'choppiness':         await renderChoppiness(mount, state); break;
            case 'var-estimator':      await renderVarEstimator(mount, state); break;
            case 'kelly':              await renderKelly(mount, state); break;
            case 'mc-trades':          await renderMcTrades(mount, state); break;
            case 'keyboard-shortcuts': await renderKeyboardShortcuts(mount, state); break;
            case 'commission-optimizer': await renderCommissionOptimizer(mount, state); break;
            case 'margin-runway':      await renderMarginRunway(mount, state); break;
            case 'risk-parity':        await renderRiskParity(mount, state); break;
            case 'risk-on-off':        await renderRiskOnOff(mount, state); break;
            case 'risk-reward':        await renderRiskReward(mount, state); break;
            case 'tax-loss-harvest':   await renderTaxLossHarvest(mount, state); break;
            case 'wash-sale':          await renderWashSale(mount, state); break;
            case 'buying-power':       await renderBuyingPower(mount, state); break;
            case 'margin-call':        await renderMarginCall(mount, state); break;
            case 'vix-term-structure': await renderVixTermStructure(mount, state); break;
            case 'currency-exposure': await renderCurrencyExposure(mount, state); break;
            case 'bond-duration':     await renderBondDuration(mount, state); break;
            case 'carry-score':       await renderCarryScore(mount, state); break;
            case 'yield-curve':       await renderYieldCurve(mount, state); break;
            case 'cost-basis':        await renderCostBasis(mount, state); break;
            case 'stop-loss-backtest': await renderStopLossBacktest(mount, state); break;
            case 'futures-roll':      await renderFuturesRoll(mount, state); break;
            case 'heatmap-dow-hour':  await renderHeatmapDowHour(mount, state); break;
            case 'atr-cone':          await renderAtrCone(mount, state); break;
            case 'round-levels':      await renderRoundLevels(mount, state); break;
            case 'kyles-lambda':      await renderKylesLambda(mount, state); break;
            case 'hawkes':            await renderHawkesIntensity(mount, state); break;
            case 'kagi':              await renderKagiChart(mount, state); break;
            case 'risk-parity-solver': await renderRiskParitySolver(mount, state); break;
            case 'volume-at-price':    await renderVolumeAtPrice(mount, state); break;
            case 'herfindahl':         await renderHerfindahl(mount, state); break;
            case 'roll-spread':        await renderRollSpread(mount, state); break;
            case 'three-line-break':   await renderThreeLineBreak(mount, state); break;
            case 'momentum-crash':     await renderMomentumCrash(mount, state); break;
            case 'effective-spread':   await renderEffectiveSpread(mount, state); break;
            case 'weighted-midprice':  await renderWeightedMidprice(mount, state); break;
            case 'marginal-var':       await renderMarginalVar(mount, state); break;
            case 'range-bar':          await renderRangeBar(mount, state); break;
            case 'tick-bar':           await renderTickBar(mount, state); break;
            case 'volume-bar':         await renderVolumeBar(mount, state); break;
            case 'dollar-bar':         await renderDollarBar(mount, state); break;
            case 'active-share':       await renderActiveShare(mount, state); break;
            case 'brinson':            await renderBrinson(mount, state); break;
            case 'equivolume':         await renderEquivolume(mount, state); break;
            case 'imbalance-bar':      await renderImbalanceBar(mount, state); break;
            case 'black-litterman':    await renderBlackLitterman(mount, state); break;
            case 'adf-test':           await renderAdfTest(mount, state); break;
            case 'aroon':              await renderAroon(mount, state); break;
            case 'amihud':             await renderAmihud(mount, state); break;
            case 'breadth-thrust':     await renderBreadthThrust(mount, state); break;
            case 'bb-squeeze':         await renderBollingerSqueeze(mount, state); break;
            case 'balance-of-power':   await renderBalanceOfPower(mount, state); break;
            case 'anchored-momentum':  await renderAnchoredMomentum(mount, state); break;
            case 'acf':                await renderAcf(mount, state); break;
            case 'beta':               await renderBeta(mount, state); break;
            case 'brier-score':        await renderBrierScore(mount, state); break;
            case 'bipower-variation':  await renderBipowerVariation(mount, state); break;
            case 'bootstrap-pnl':      await renderBootstrapPnl(mount, state); break;
            case 'block-bootstrap':    await renderBlockBootstrap(mount, state); break;
            case 'ad-normality':       await renderAdNormality(mount, state); break;
            case 'arch-lm':            await renderArchLm(mount, state); break;
            case 'alma':               await renderAlma(mount, state); break;
            case 'alphatrend':         await renderAlphatrend(mount, state); break;
            case 'atr-channel':        await renderAtrChannel(mount, state); break;
            case 'atr-trailing-stop':  await renderAtrTrailStop(mount, state); break;
            case 'adl':                await renderAdl(mount, state); break;
            case 'asi':                await renderAsi(mount, state); break;
            case 'ad-oscillator':      await renderAdOscillator(mount, state); break;
            case 'beta-shrinkage':     await renderBetaShrink(mount, state); break;
            case 'bartlett-variance':  await renderBartlett(mount, state); break;
            case 'bid-ask-volume-ratio': await renderBidAskVol(mount, state); break;
            case 'bollinger-band-width': await renderBbw(mount, state); break;
            case 'bollinger-bandwidth-percentile': await renderBbwp(mount, state); break;
            case 'bollinger-percent-b': await renderBbPercentB(mount, state); break;
            case 'bollinger-band-distance': await renderBbd(mount, state); break;
            case 'bollinger-oscillators': await renderBbOsc(mount, state); break;
            case 'borrow-rate-indicator': await renderBorrowRate(mount, state); break;
            case 'breusch-pagan':      await renderBpTest(mount, state); break;
            case 'burke-ratio':        await renderBurke(mount, state); break;
            case 'camarilla-pivots':   await renderCamarilla(mount, state); break;
            case 'breusch-godfrey':    await renderBgTest(mount, state); break;
            case 'candle-strength-index': await renderCsi(mount, state); break;
            case 'carhart-4':          await renderCarhart4(mount, state); break;
            case 'centered-smoothed-momentum': await renderCsm(mount, state); break;
            case 'chaikin-oscillator': await renderChaikinOsc(mount, state); break;
            case 'chande-dynamic-momentum': await renderCdmi(mount, state); break;
            case 'chande-kroll-stop':  await renderCks(mount, state); break;
            case 'chande-momentum-oscillator': await renderCmo(mount, state); break;
            case 'chande-trend-index': await renderCti(mount, state); break;
            case 'chande-volatility-index': await renderCvi(mount, state); break;
            case 'chandelier-exit':    await renderChandelier(mount, state); break;
            case 'cholesky':           await renderCholesky(mount, state); break;
            case 'abc-pattern':        await renderAbcPattern(mount, state); break;
            case 'absorption':         await renderAbsorption(mount, state); break;
            case 'favorites':          await renderFavoritesManager(mount, state); break;
            case 'crypto':      await renderCrypto(mount, state); break;
            case 'backtest':    await renderBacktest(mount, state); break;
            case 'economy':     await renderEconomy(mount, state); break;
            case 'pairs':       await renderPairs(mount, state); break;
            case 'short-interest': await renderShortInterest(mount, state, sym()); break;
            case 'darkpool':       await renderDarkpool(mount, state, sym()); break;
            case 'vol':            await renderVol(mount, state); break;
            case 'webhooks':       await renderWebhooks(mount, state); break;
            case 'breadth':        await renderBreadth(mount, state); break;
            case 'fear-greed':     await renderFearGreed(mount, state); break;
            case 'premarket':      await renderPremarket(mount, state); break;
            case 'after-hours':    await renderAfterHours(mount, state); break;
            case 'halts':          await renderHalts(mount, state); break;
            case 'live-scanner':   await renderLiveScanner(mount, state); break;
            case 'catalysts':      await renderCatalysts(mount, state); break;
            case 'catalyst-correlations': await renderCatalystCorrelations(mount, state); break;
            case 'uoa-stream':     await renderUoaStream(mount, state); break;
            case 'gamma-squeeze':  await renderGammaSqueeze(mount, state); break;
            case 'htb-ranker':     await renderHtbRanker(mount, state); break;
            case 'breadth-divergence': await renderBreadthDivergence(mount, state); break;
            case 'rvol-accel':     await renderRvolAccel(mount, state); break;
            case 'insider-stream': await renderInsiderStream(mount, state); break;
            case 'insider-clusters': await renderInsiderClusters(mount, state); break;
            case 'earnings-revisions': await renderEarningsRevisions(mount, state); break;
            case 'sector-timing':  await renderSectorTiming(mount, state); break;
            case 'market-gamma':   await renderMarketGammaRegime(mount, state); break;
            case 'scanner-backtest': await renderScannerBacktest(mount, state); break;
            case 'confluence-autotrade': await renderConfluenceAutotrade(mount, state); break;
            case 'portfolio-exposure': await renderPortfolioExposure(mount, state); break;
            case 'dividend-tracker': await renderDividendTracker(mount, state); break;
            case 'magic-formula': await renderMagicFormula(mount, state); break;
            case 'paper-rebalance': await renderPaperRebalance(mount, state); break;
            case 'paper-tax-loss-harvest': await renderPaperTaxLossHarvest(mount, state); break;
            case 'sector-rotation-strategy': await renderSectorRotationStrategy(mount, state); break;
            case 'dca-simulator': await renderDcaSimulator(mount, state); break;
            case 'dividend-aristocrats': await renderDividendAristocrats(mount, state); break;
            case 'permanent-portfolio': await renderPermanentPortfolio(mount, state); break;
            case 'cape-indicator': await renderCapeIndicator(mount, state); break;
            case 'fire-calculator': await renderFireCalculator(mount, state); break;
            case 'emergency-fund': await renderEmergencyFund(mount, state); break;
            case 'net-worth-tracker': await renderNetWorthTracker(mount, state); break;
            case 'personal-balance-sheet': await renderPersonalBalanceSheet(mount, state); break;
            case 'personal-cash-flow': await renderPersonalCashFlow(mount, state); break;
            case 'financial-ratios': await renderFinancialRatios(mount, state); break;
            case 'savings-rate': await renderSavingsRate(mount, state); break;
            case 'sinking-fund': await renderSinkingFund(mount, state); break;
            case 'zero-based-budget': await renderZeroBasedBudget(mount, state); break;
            case 'fifty-thirty-twenty': await renderFiftyThirtyTwenty(mount, state); break;
            case 'envelope-budget': await renderEnvelopeBudget(mount, state); break;
            case 'debt-avalanche': await renderDebtAvalanche(mount, state); break;
            case 'debt-snowball': await renderDebtSnowball(mount, state); break;
            case 'credit-utilization': await renderCreditUtilization(mount, state); break;
            case 'auto-loan': await renderAutoLoan(mount, state); break;
            case 'mortgage-amortization': await renderMortgageAmortization(mount, state); break;
            case 'mortgage-refinance': await renderMortgageRefinance(mount, state); break;
            case 'rent-vs-buy': await renderRentVsBuy(mount, state); break;
            case 'heloc': await renderHeloc(mount, state); break;
            case 'home-maintenance': await renderHomeMaintenance(mount, state); break;
            case 'student-loan-payoff': await renderStudentLoanPayoff(mount, state); break;
            case 'pslf-tracker': await renderPslfTracker(mount, state); break;
            case 'college-529': await renderCollege529(mount, state); break;
            case 'fafsa-efc': await renderFafsaEfc(mount, state); break;
            case 'car-tco': await renderCarTco(mount, state); break;
            case 'drawdown-cutoff': await renderDrawdownCutoff(mount, state); break;
            case 'pead':           await renderPead(mount, state); break;
            case 'sentiment-velocity': await renderSentimentVelocity(mount, state); break;
            case 'confluence':     await renderConfluence(mount, state); break;
            case 'vrp':            await renderVrp(mount, state); break;
            case 'pairs-coint':    await renderPairsCoint(mount, state); break;
            case 'ipo-lockups':    await renderIpoLockups(mount, state); break;
            case 'iv-term':        await renderIvTerm(mount, state); break;
            case 'sp500-predict':  await renderSp500Predict(mount, state); break;
            case 'dividend-capture': await renderDividendCapture(mount, state); break;
            case 'multi-broker':   await renderMultiBroker(mount, state); break;
            case 'webull':         await renderWebull(mount, state); break;
            case 'vol-surface':    await renderVolSurface(mount, state); break;
            case 'walk-forward':   await renderWalkForward(mount, state); break;
            case 'tax-lots':       await renderTaxLots(mount, state); break;
            case 'expenses':       await renderExpensesView(mount); break;
            case 'expense-dashboard': await renderExpenseDashboard(mount); break;
            case 'expense-calendar':  await renderExpenseCalendar(mount); break;
            case 'business-compare':  await renderBusinessCompare(mount); break;
            case 'broker-compare':    await renderBrokerCompare(mount); break;
            case 'brokers':           await renderBrokersManage(mount); break;
            case 'businesses':        await renderBusinessesManage(mount); break;
            case 'toast-history':     await renderToastHistory(mount); break;
            case 'log-viewer':        await renderLogViewer(mount); break;
            case 'receipts':       await renderReceipts(mount, state); break;
            case 'purchases':      await renderPurchases(mount, state); break;
            case 'categorize':     await renderCategorize(mount, state); break;
            case 'file-taxes':     await renderTaxWizard(mount, state); break;
            case 'budget':         await renderBudget(mount, state); break;
            case 'compare':        await renderCompare(mount, state); break;
            case 'exports':        await renderExports(mount, state); break;
            case 'ai':             await renderAiSettings(mount, state); break;
            case 'developer':      await renderDeveloper(mount, state); break;
            case 'boards':         await renderBoards(mount, state, rest[0] || ''); break;
            case 'news':           await renderNews(mount, state); break;
            case 'earnings-cal':   await renderEarningsCal(mount, state); break;
            case 'sizing':         await renderPositionSize(mount, state); break;
            case 'live':           await renderLivePositions(mount, state); break;
            case 'correlation':    await renderCorrMatrix(mount, state); break;
            case 'strategy-alerts': await renderStrategyAlerts(mount, state); break;
            case 'algo':           await renderAlgo(mount, state); break;
            case 'rebalance':      await renderRebalance(mount, state); break;
            case 'sector-rotation': await renderSectorRotation(mount, state); break;
            case 'tape-replay':    await renderTapeReplay(mount, state, sym()); break;
            case 'backtest-presets': await renderBacktestPresets(mount, state, rest[0] || ''); break;
            case 'mood':           await renderMoodAnalytics(mount, state); break;
            case 'discipline':     await renderDiscipline(mount, state); break;
            case 'goals':          await renderGoals(mount, state); break;
            case 'r-dist':         await renderRDist(mount, state); break;
            case 'reviews':        await renderTradeReviews(mount, state); break;
            case 'forecast':       await renderEquityForecast(mount, state); break;
            case 'fill-quality':   await renderFillQuality(mount, state); break;
            case 'custom-indicators': await renderCustomIndicators(mount, state); break;
            case 'trade-compare':     await renderTradeCompare(mount, state); break;
            case 'csv-wizard':        await renderCsvWizard(mount, state); break;
            case 'accounts-overview': await renderAccountsOverview(mount, state); break;
            case 'trade':       await renderTradeDetail(mount, state, rest[0]); break;
            case 'journal':     await renderJournalView(mount, state, rest[0]); break;
            case 'calendar':    await renderCalendar(mount, state); break;
            case 'reports':     await renderReports(mount, state, rest[0] || 'overview'); break;
            case 'charts':      await renderCharts(mount, state, sym()); break;
            case 'multichart': await renderMultichart(mount, state, sym()); break;
            case 'squeeze-scanner': await renderSqueezeScanner(mount, state); break;
            case 'ipo-calendar':    await renderIpoCalendar(mount, state); break;
            case 'top-news':        await renderTopNews(mount, state); break;
            case 'finnhub-pattern':       await renderFinnhubPattern(mount, state, sym()); break;
            case 'finnhub-sr':            await renderFinnhubSr(mount, state, sym()); break;
            case 'finnhub-aggregate':     await renderFinnhubAggregate(mount, state, sym()); break;
            case 'forex-rates':           await renderForexRates(mount, state); break;
            case 'economic-calendar':     await renderEconomicCalendar(mount, state); break;
            case 'symbol-changes':        await renderSymbolChanges(mount, state); break;
            case 'etf-profile':           await renderEtfProfile(mount, state, sym()); break;
            case 'lobbying':              await renderLobbying(mount, state, sym()); break;
            case 'congressional-trading': await renderCongressionalTrading(mount, state, sym()); break;
            case 'finnhub-search':        await renderFinnhubSearch(mount, state); break;
            case 'fda-calendar':          await renderFdaCalendar(mount, state); break;
            case 'market-status':         await renderMarketStatus(mount, state); break;
            case 'index-constituents':    await renderIndexConstituents(mount, state); break;
            case 'insider-finnhub':       await renderInsiderTransactionsFinnhub(mount, state, sym()); break;
            case 'news-sentiment':        await renderNewsSentiment(mount, state, sym()); break;
            case 'price-target':          await renderPriceTarget(mount, state, sym()); break;
            case 'estimates-dashboard':   await renderEstimatesDashboard(mount, state, sym()); break;
            case 'crypto-markets':        await renderCryptoMarkets(mount, state); break;
            case 'historical-market-cap': await renderHistoricalMarketCap(mount, state, sym()); break;
            case 'earnings-call-live':    await renderEarningsCallLive(mount, state); break;
            case 'supply-chain':          await renderSupplyChain(mount, state, sym()); break;
            case 'esg':                   await renderEsg(mount, state, sym()); break;
            case 'sector-heatmap':        await renderSectorHeatmap(mount, state); break;
            case 'bond-yield-curve':      await renderBondYieldCurve(mount, state); break;
            case 'unusual-options':       await renderUnusualOptions(mount, state, sym()); break;
            case 'subscriptions':         await renderSubscriptions(mount, state); break;
            case 'revenue-breakdown':     await renderRevenueBreakdown(mount, state, sym()); break;
            case 'earnings-quality':      await renderEarningsQuality(mount, state, sym()); break;
            case 'quarterly-tax':         await renderQuarterlyTax(mount, state); break;
            case 'mileage-log':           await renderMileageLog(mount, state); break;
            case 'filings-browser':       await renderFilingsBrowser(mount, state, sym()); break;
            case 'insider-sentiment':     await renderInsiderSentiment(mount, state, sym()); break;
            case 'institutional-13f':     await renderInstitutional13F(mount, state); break;
            case 'section-179':           await renderSection179(mount, state); break;
            case 'retirement-max':        await renderRetirementMax(mount, state); break;
            case 'splits-history':        await renderSplitsHistory(mount, state, sym()); break;
            case 'mutual-fund':           await renderMutualFund(mount, state, sym()); break;
            case 'uspto-patents':         await renderUsptoPatents(mount, state, sym()); break;
            case 'home-office':           await renderHomeOffice(mount, state); break;
            case 'income-1099':           await renderIncome1099(mount, state); break;
            case 'meal-deduction':        await renderMealDeduction(mount, state); break;
            case 'biz-categorizer':       await renderBizCategorizer(mount, state); break;
            case 'depreciation':          await renderDepreciation(mount, state); break;
            case 'travel-per-diem':       await renderTravelPerDiem(mount, state); break;
            case 'qbi-199a':              await renderQbi199A(mount, state); break;
            case 'state-tax':             await renderStateTax(mount, state); break;
            case 'scorp-calc':            await renderScorpCalc(mount, state); break;
            case 'nol-tracker':           await renderNolTracker(mount, state); break;
            case 'augusta-rule':          await renderAugustaRule(mount, state); break;
            case 'charitable-planner':    await renderCharitablePlanner(mount, state); break;
            case 'fbar-8938':             await renderFbar8938(mount, state); break;
            case 'sec-1256':              await renderSec1256(mount, state); break;
            case 'wash-sale-tracker':     await renderWashSaleTracker(mount, state); break;
            case 'hsa-max':               await renderHsaMax(mount, state); break;
            case 'forex-988':             await renderForex988(mount, state); break;
            case 'rd-credit':             await renderRdCredit(mount, state); break;
            case 'mtm-election':          await renderMtmElection(mount, state); break;
            case 'tts-qualification':     await renderTtsQualification(mount, state); break;
            case 'qsbs-1202':             await renderQsbs1202(mount, state); break;
            case 'education-credits':     await renderEducationCredits(mount, state); break;
            case 'accountable-plan':      await renderAccountablePlan(mount, state); break;
            case 'dcfsa':                 await renderDcfsa(mount, state); break;
            case 'ev-credit':             await renderEvCredit(mount, state); break;
            case 'foreign-tax-credit':    await renderForeignTaxCredit(mount, state); break;
            case 'roth-ladder':           await renderRothLadder(mount, state); break;
            case 'gift-tax':              await renderGiftTax(mount, state); break;
            case 'clean-energy-25d':      await renderCleanEnergy25D(mount, state); break;
            case 'savers-credit':         await renderSaversCredit(mount, state); break;
            case 'inherited-ira-rmd':     await renderInheritedIraRmd(mount, state); break;
            case 'qcd-tracker':           await renderQcdTracker(mount, state); break;
            case 'nua-strategy':          await renderNuaStrategy(mount, state); break;
            case 'kiddie-tax':            await renderKiddieTax(mount, state); break;
            case 'qoz-tracker':           await renderQozTracker(mount, state); break;
            case '529-roth':              await renderRollover529Roth(mount, state); break;
            case 'se-health-deduction':   await renderSeHealthDeduction(mount, state); break;
            case 'mega-backdoor-roth':    await renderMegaBackdoorRoth(mount, state); break;
            case 'cost-seg':              await renderCostSeg(mount, state); break;
            case 'passive-loss':          await renderPassiveLoss(mount, state); break;
            case 'section-1031':          await renderSection1031(mount, state); break;
            case 'installment-sale':      await renderInstallmentSale(mount, state); break;
            case 'str-loophole':          await renderStrLoophole(mount, state); break;
            case 'amt-calc':              await renderAmtCalc(mount, state); break;
            case 'iso-exercise':          await renderIsoExercise(mount, state); break;
            case 'nso-exercise':          await renderNsoExercise(mount, state); break;
            case 'rsu-vest-tracker':      await renderRsuVestTracker(mount, state); break;
            case 'espp-calc':             await renderEsppCalc(mount, state); break;
            case 'backdoor-roth':         await renderBackdoorRoth(mount, state); break;
            case 'cross-broker-wash':     await renderCrossBrokerWash(mount, state); break;
            case 'able-account':          await renderAbleAccount(mount, state); break;
            case 'conservation-easement': await renderConservationEasement(mount, state); break;
            case 'lihtc':                 await renderLihtc(mount, state); break;
            case 'mlp-k1':                await renderMlpK1(mount, state); break;
            case 'historic-rehab':        await renderHistoricRehab(mount, state); break;
            case 'disabled-access':       await renderDisabledAccess(mount, state); break;
            case 'film-181':              await renderFilm181(mount, state); break;
            case 'partial-disposition':   await renderPartialDisposition(mount, state); break;
            case 'tts-scorer':            await renderTtsScorer(mount, state); break;
            case 'section-475f':          await renderSection475f(mount, state); break;
            case 'section-195':           await renderSection195(mount, state); break;
            case 'section-1244':          await renderSection1244(mount, state); break;
            case 'section-280f':          await renderSection280f(mount, state); break;
            case 'section-197':           await renderSection197(mount, state); break;
            case 'section-274':           await renderSection274(mount, state); break;
            case 'solo-401k':             await renderSolo401k(mount, state); break;
            case 'grat':                  await renderGrat(mount, state); break;
            case 'section-469':           await renderSection469(mount, state); break;
            case 'sep-ira':               await renderSepIra(mount, state); break;
            case 'section-72t':           await renderSection72t(mount, state); break;
            case 'daf':                   await renderDaf(mount, state); break;
            case 'slat':                  await renderSlat(mount, state); break;
            case 'section-121':           await renderSection121(mount, state); break;
            case 'section-1361':          await renderSection1361(mount, state); break;
            case 'section-162l':          await renderSection162l(mount, state); break;
            case 'section-7872':          await renderSection7872(mount, state); break;
            case 'crut':                  await renderCrut(mount, state); break;
            case 'ilit':                  await renderIlit(mount, state); break;
            case 'section-168k':          await renderSection168k(mount, state); break;
            case 'section-168':           await renderSection168(mount, state); break;
            case 'section-263a':          await renderSection263a(mount, state); break;
            case 'section-6654':          await renderSection6654(mount, state); break;
            case 'section-911':           await renderSection911(mount, state); break;
            case 'section-1411':          await renderSection1411(mount, state); break;
            case 'section-280e':          await renderSection280e(mount, state); break;
            case 'section-165g':          await renderSection165g(mount, state); break;
            case 'section-2010c':         await renderSection2010c(mount, state); break;
            case 'section-174':           await renderSection174(mount, state); break;
            case 'section-691':           await renderSection691(mount, state); break;
            case 'section-25a':           await renderSection25a(mount, state); break;
            case 'section-221':           await renderSection221(mount, state); break;
            case 'crat':                  await renderCrat(mount, state); break;
            case 'residency-daycount':    await renderResidencyDaycount(mount, state); break;
            case 'section-36b':           await renderSection36b(mount, state); break;
            case 'simple-ira':            await renderSimpleIra(mount, state); break;
            case 'defined-benefit':       await renderDefinedBenefit(mount, state); break;
            case 'section-213':           await renderSection213(mount, state); break;
            case 'section-6038':          await renderSection6038(mount, state); break;
            case 'section-408d3':         await renderSection408d3(mount, state); break;
            case 'section-162m':          await renderSection162m(mount, state); break;
            case 'section-4975':          await renderSection4975(mount, state); break;
            case 'section-4980h':         await renderSection4980h(mount, state); break;
            case 'section-263-tpr':       await renderSection263Tpr(mount, state); break;
            case 'section-6048':          await renderSection6048(mount, state); break;
            case 'section-6038a':         await renderSection6038a(mount, state); break;
            case 'section-7702a':         await renderSection7702a(mount, state); break;
            case 'crypto-staking':        await renderCryptoStaking(mount, state); break;
            case 'section-1042':          await renderSection1042(mount, state); break;
            case 'section-1259':          await renderSection1259(mount, state); break;
            case 'section-1296-pfic':     await renderSection1296Pfic(mount, state); break;
            case 'section-1245-1250':     await renderSection12451250(mount, state); break;
            case 'section-351-721':       await renderSection351721(mount, state); break;
            case 'section-172':           await renderSection172(mount, state); break;
            case 'section-1035':          await renderSection1035(mount, state); break;
            case 'section-6707a':         await renderSection6707a(mount, state); break;
            case 'section-280g':          await renderSection280g(mount, state); break;
            case 'section-1374':          await renderSection1374(mount, state); break;
            case 'section-871m':          await renderSection871m(mount, state); break;
            case 'section-1402':          await renderSection1402(mount, state); break;
            case 'section-1276':          await renderSection1276(mount, state); break;
            case 'section-1233':          await renderSection1233(mount, state); break;
            case 'section-6166':          await renderSection6166(mount, state); break;
            case 'section-4941':          await renderSection4941(mount, state); break;
            case 'section-1092':          await renderSection1092(mount, state); break;
            case 'section-6038b':         await renderSection6038b(mount, state); break;
            case 'section-2056':          await renderSection2056(mount, state); break;
            case 'section-4940':          await renderSection4940(mount, state); break;
            case 'section-7345':          await renderSection7345(mount, state); break;
            case 'section-1212':          await renderSection1212(mount, state); break;
            case 'section-4980d':         await renderSection4980d(mount, state); break;
            case 'section-6663':          await renderSection6663(mount, state); break;
            case 'section-6694':          await renderSection6694(mount, state); break;
            case 'section-6045b':         await renderSection6045b(mount, state); break;
            case 'section-529':           await renderSection529(mount, state); break;
            case 'section-530':           await renderSection530(mount, state); break;
            case 'section-401k-hardship': await renderSection401kHardship(mount, state); break;
            case 'section-72p':           await renderSection72p(mount, state); break;
            case 'section-401a9':         await renderSection401a9(mount, state); break;
            case 'section-6015':          await renderSection6015(mount, state); break;
            case 'section-6651':          await renderSection6651(mount, state); break;
            case 'section-1014':          await renderSection1014(mount, state); break;
            case 'section-23':            await renderSection23(mount, state); break;
            case 'section-32-eic':        await renderSection32Eic(mount, state); break;
            case 'section-4942':          await renderSection4942(mount, state); break;
            case 'section-4960':          await renderSection4960(mount, state); break;
            case 'section-6213':          await renderSection6213(mount, state); break;
            case 'section-6321':          await renderSection6321(mount, state); break;
            case 'section-6331':          await renderSection6331(mount, state); break;
            case 'section-4943':          await renderSection4943(mount, state); break;
            case 'section-4944':          await renderSection4944(mount, state); break;
            case 'section-4945':          await renderSection4945(mount, state); break;
            case 'section-6664':          await renderSection6664(mount, state); break;
            case 'section-7430':          await renderSection7430(mount, state); break;
            case 'section-6502':          await renderSection6502(mount, state); break;
            case 'section-7122':          await renderSection7122(mount, state); break;
            case 'section-6159':          await renderSection6159(mount, state); break;
            case 'section-7811':          await renderSection7811(mount, state); break;
            case 'section-6724':          await renderSection6724(mount, state); break;
            case 'section-24-ctc':        await renderSection24Ctc(mount, state); break;
            case 'section-21-cdcc':       await renderSection21Cdcc(mount, state); break;
            case 'section-71-alimony':    await renderSection71Alimony(mount, state); break;
            case 'section-152':           await renderSection152(mount, state); break;
            case 'section-7508a':         await renderSection7508a(mount, state); break;
            case 'section-132':           await renderSection132(mount, state); break;
            case 'section-127':           await renderSection127(mount, state); break;
            case 'section-125':           await renderSection125(mount, state); break;
            case 'section-165c3':         await renderSection165c3(mount, state); break;
            case 'section-119':           await renderSection119(mount, state); break;
            case 'section-25c':           await renderSection25c(mount, state); break;
            case 'section-45l':           await renderSection45l(mount, state); break;
            case 'section-179d':          await renderSection179d(mount, state); break;
            case 'section-86':            await renderSection86(mount, state); break;
            case 'section-219':           await renderSection219(mount, state); break;
            case 'section-415':           await renderSection415(mount, state); break;
            case 'section-414v':          await renderSection414v(mount, state); break;
            case 'section-481a':          await renderSection481a(mount, state); break;
            case 'section-168g':          await renderSection168g(mount, state); break;
            case 'section-416':           await renderSection416(mount, state); break;
            case 'section-446':           await renderSection446(mount, state); break;
            case 'section-451':           await renderSection451(mount, state); break;
            case 'section-461':           await renderSection461(mount, state); break;
            case 'section-471':           await renderSection471(mount, state); break;
            case 'section-482':           await renderSection482(mount, state); break;
            case 'section-901':           await renderSection901(mount, state); break;
            case 'section-951':           await renderSection951(mount, state); break;
            case 'section-951a':          await renderSection951A(mount, state); break;
            case 'section-250':           await renderSection250(mount, state); break;
            case 'section-163j':          await renderSection163j(mount, state); break;
            case 'section-59a':           await renderSection59A(mount, state); break;
            case 'section-245a':          await renderSection245A(mount, state); break;
            case 'section-7874':          await renderSection7874(mount, state); break;
            case 'section-4501':          await renderSection4501(mount, state); break;
            case 'section-877a':          await renderSection877A(mount, state); break;
            case 'section-1291':          await renderSection1291(mount, state); break;
            case 'section-1295':          await renderSection1295(mount, state); break;
            case 'section-897':           await renderSection897(mount, state); break;
            case 'section-1445':          await renderSection1445(mount, state); break;
            case 'section-754':           await renderSection754(mount, state); break;
            case 'section-302':           await renderSection302(mount, state); break;
            case 'section-332':           await renderSection332(mount, state); break;
            case 'section-338':           await renderSection338(mount, state); break;
            case 'section-368':           await renderSection368(mount, state); break;
            case 'section-1248':          await renderSection1248(mount, state); break;
            case 'section-355':           await renderSection355(mount, state); break;
            case 'section-311':           await renderSection311(mount, state); break;
            case 'section-1366':          await renderSection1366(mount, state); break;
            case 'section-1368':          await renderSection1368(mount, state); break;
            case 'section-41':            await renderSection41(mount, state); break;
            case 'section-1033':          await renderSection1033(mount, state); break;
            case 'section-1400z':         await renderSection1400z(mount, state); break;
            case 'section-357':           await renderSection357(mount, state); break;
            case 'section-305':           await renderSection305(mount, state); break;
            case 'section-336':           await renderSection336(mount, state); break;
            case 'section-30d':           await renderSection30D(mount, state); break;
            case 'section-45q':           await renderSection45Q(mount, state); break;
            case 'section-48':            await renderSection48(mount, state); break;
            case 'section-38':            await renderSection38(mount, state); break;
            case 'section-6050w':         await renderSection6050W(mount, state); break;
            case 'section-45x':           await renderSection45X(mount, state); break;
            case 'section-45v':           await renderSection45V(mount, state); break;
            case 'section-48c':           await renderSection48C(mount, state); break;
            case 'section-25d':           await renderSection25D(mount, state); break;
            case 'section-1446f':         await renderSection1446F(mount, state); break;
            case 'section-42':            await renderSection42(mount, state); break;
            case 'section-83':            await renderSection83(mount, state); break;
            case 'section-409a':          await renderSection409A(mount, state); break;
            case 'section-457':           await renderSection457(mount, state); break;
            case 'section-30c':           await renderSection30C(mount, state); break;
            case 'section-45w':           await renderSection45W(mount, state); break;
            case 'section-47':            await renderSection47(mount, state); break;
            case 'section-51':            await renderSection51(mount, state); break;
            case 'section-25e':           await renderSection25E(mount, state); break;
            case 'section-460':           await renderSection460(mount, state); break;
            case 'section-467':           await renderSection467(mount, state); break;
            case 'section-79':            await renderSection79(mount, state); break;
            case 'section-105':           await renderSection105(mount, state); break;
            case 'section-269':           await renderSection269(mount, state); break;
            case 'section-1239':          await renderSection1239(mount, state); break;
            case 'section-7701o':         await renderSection7701o(mount, state); break;
            case 'section-106':           await renderSection106(mount, state); break;
            case 'section-1015':          await renderSection1015(mount, state); break;
            case 'section-444':           await renderSection444(mount, state); break;
            case 'section-904':           await renderSection904(mount, state); break;
            case 'section-7701b':         await renderSection7701B(mount, state); break;
            case 'section-1041':          await renderSection1041(mount, state); break;
            case 'section-871a':          await renderSection871A(mount, state); break;
            case 'section-367d':          await renderSection367D(mount, state); break;
            case 'section-165d':          await renderSection165D(mount, state); break;
            case 'section-1377':          await renderSection1377(mount, state); break;
            case 'section-162f':          await renderSection162F(mount, state); break;
            case 'section-162c':          await renderSection162C(mount, state); break;
            case 'section-1297':          await renderSection1297(mount, state); break;
            case 'section-6655':          await renderSection6655(mount, state); break;
            case 'section-6700':          await renderSection6700(mount, state); break;
            case 'section-6325':          await renderSection6325(mount, state); break;
            case 'section-1298':          await renderSection1298(mount, state); break;
            case 'section-6072':          await renderSection6072(mount, state); break;
            case 'section-162a1':         await renderSection162a1(mount, state); break;
            case 'section-367a':          await renderSection367A(mount, state); break;
            case 'section-882':           await renderSection882(mount, state); break;
            case 'section-884':           await renderSection884(mount, state); break;
            case 'section-6045':          await renderSection6045(mount, state); break;
            case 'section-263c':          await renderSection263C(mount, state); break;
            case 'section-412':           await renderSection412(mount, state); break;
            case 'section-1362':          await renderSection1362(mount, state); break;
            case 'section-414':           await renderSection414(mount, state); break;
            case 'section-472':           await renderSection472(mount, state); break;
            case 'section-901j':          await renderSection901J(mount, state); break;
            case 'section-248':           await renderSection248(mount, state); break;
            case 'section-6041':          await renderSection6041(mount, state); break;
            case 'section-6049':          await renderSection6049(mount, state); break;
            case 'section-6051':          await renderSection6051(mount, state); break;
            case 'section-1273':          await renderSection1273(mount, state); break;
            case 'section-7702':          await renderSection7702(mount, state); break;
            case 'section-6033':          await renderSection6033(mount, state); break;
            case 'section-7491':          await renderSection7491(mount, state); break;
            case 'section-483':           await renderSection483(mount, state); break;
            case 'section-4973':          await renderSection4973(mount, state); break;
            case 'section-6695':          await renderSection6695(mount, state); break;
            case 'section-2503':          await renderSection2503(mount, state); break;
            case 'section-2055':          await renderSection2055(mount, state); break;
            case 'section-511':           await renderSection511(mount, state); break;
            case 'section-2032':          await renderSection2032(mount, state); break;
            case 'section-4974':          await renderSection4974(mount, state); break;
            case 'section-6111':          await renderSection6111(mount, state); break;
            case 'section-4972':          await renderSection4972(mount, state); break;
            case 'section-6039':          await renderSection6039(mount, state); break;
            case 'section-894':           await renderSection894(mount, state); break;
            case 'section-2518':          await renderSection2518(mount, state); break;
            case 'section-199a':          await renderSection199A(mount, state); break;
            case 'section-461l':          await renderSection461L(mount, state); break;
            case 'section-165':           await renderSection165(mount, state); break;
            case 'section-408a':          await renderSection408A(mount, state); break;
            case 'section-962':           await renderSection962(mount, state); break;
            case 'section-280c':          await renderSection280C(mount, state); break;
            case 'section-1202':          await renderSection1202(mount, state); break;
            case 'section-170':           await renderSection170(mount, state); break;
            case 'section-351':           await renderSection351(mount, state); break;
            case 'section-6038d':         await renderSection6038D(mount, state); break;
            case 'section-752':           await renderSection752(mount, state); break;
            case 'section-1245':          await renderSection1245(mount, state); break;
            case 'section-164':           await renderSection164(mount, state); break;
            case 'section-24':            await renderSection24(mount, state); break;
            case 'section-6045a':         await renderSection6045A(mount, state); break;
            case 'section-743':           await renderSection743(mount, state); break;
            case 'section-1250':          await renderSection1250(mount, state); break;
            case 'section-1231':          await renderSection1231(mount, state); break;
            case 'section-871':           await renderSection871(mount, state); break;
            case 'section-731':           await renderSection731(mount, state); break;
            case 'section-6011':          await renderSection6011(mount, state); break;
            case 'section-382':           await renderSection382(mount, state); break;
            case 'section-1234':          await renderSection1234(mount, state); break;
            case 'section-32':            await renderSection32(mount, state); break;
            case 'section-707':           await renderSection707(mount, state); break;
            case 'section-736':           await renderSection736(mount, state); break;
            case 'section-1058':          await renderSection1058(mount, state); break;
            case 'section-269a':          await renderSection269A(mount, state); break;
            case 'section-318':           await renderSection318(mount, state); break;
            case 'section-481':           await renderSection481(mount, state); break;
            case 'section-6221':          await renderSection6221(mount, state); break;
            case 'section-956':           await renderSection956(mount, state); break;
            case 'section-1296':          await renderSection1296(mount, state); break;
            case 'section-1059':          await renderSection1059(mount, state); break;
            case 'section-304':           await renderSection304(mount, state); break;
            case 'section-6112':          await renderSection6112(mount, state); break;
            case 'section-6601':          await renderSection6601(mount, state); break;
            case 'section-475':           await renderSection475(mount, state); break;
            case 'section-988':           await renderSection988(mount, state); break;
            case 'section-303':           await renderSection303(mount, state); break;
            case 'section-129':           await renderSection129(mount, state); break;
            case 'section-134':           await renderSection134(mount, state); break;
            case 'section-6672':          await renderSection6672(mount, state); break;
            case 'section-6662':          await renderSection6662(mount, state); break;
            case 'section-67':            await renderSection67(mount, state); break;
            case 'section-421':           await renderSection421(mount, state); break;
            case 'section-989':           await renderSection989(mount, state); break;
            case 'section-4958':          await renderSection4958(mount, state); break;
            case 'section-102':           await renderSection102(mount, state); break;
            case 'section-362':           await renderSection362(mount, state); break;
            case 'section-6330':          await renderSection6330(mount, state); break;
            case 'section-6404':          await renderSection6404(mount, state); break;
            case 'import':      await renderImportView(mount, state); break;
            case 'plans':       await renderPlans(mount, state); break;
            case 'tags':        await renderTags(mount, state); break;
            case 'note-templates': await renderNoteTemplates(mount); break;
            case 'mentorship':  await renderMentorship(mount, state); break;
            case 'community':   if (rest.length === 2) await renderCommunityThread(mount, state, rest[0], rest[1]);
                                else await renderCommunity(mount, state, rest[0]); break;
            case 'shares':      await renderShares(mount, state); break;
            case 'shared':      await renderSharedTrade(mount, state, rest[0]); break;
            case 'accounts':    await renderAccounts(mount, state, async () => { await loadAccounts(); renderAccountStrip(); }); break;
            case 'settings':    await renderSettings(mount, state); break;
            case 'tutorial':    await renderTutorial(mount, state); break;
            case 'tax-workshop': await renderTaxWorkshop(mount, state); break;
            case 'risk-gate':   await renderRiskGate(mount, state); break;
            default:            mount.innerHTML = `<p class="boot">${t('boot.unknown_view', { view })}</p>`;
        }
    } catch (e) {
        mount.innerHTML = `<p class="boot">${t('boot.view_error', { err: e.message })}</p>`;
        console.error(e);
    }
    // Translate any `data-i18n*` attributes the view just emitted.
    try { applyUiI18n(mount); } catch { /* i18n optional */ }
    // Upgrade any `data-tip` attributes the view emitted to native titles.
    try { upgradeTooltips(mount); } catch { /* tooltip optional */ }
    // Auto-derive a `title` for every interactive element that didn't
    // declare a `data-tip` — guarantees hover discoverability everywhere.
    try { autoApplyTooltips(mount); } catch { /* tooltip optional */ }
}

// View-renderer registry — exposed so the Dashboards view can mount any
// of these inside a tile. Only includes views that don't need URL
// params (rest[]) past the global symbol, since tile-context has none.
export const viewRenderers = {
    // Markets / accounts / expenses — top-level tiles that work
    // standalone inside a custom dashboard.
    'crypto-markets':      (m, s) => renderCryptoMarkets(m, s),
    'expenses':            (m) => renderExpensesView(m),
    'webull':              (m, s) => renderWebull(m, s),
    // Pattern / indicator detectors.
    'ha-reversal':         (m, s) => renderHaReversal(m, s),
    'three-bar-reversal':  (m, s) => renderThreeBarReversal(m, s),
    'range-expansion':     (m, s) => renderRangeExpansion(m, s),
    'alligator':           (m, s) => renderAlligator(m, s),
    'demarker':            (m, s) => renderDemarker(m, s),
    'murrey-math':         (m, s) => renderMurreyMath(m, s),
    'demark-pivots':       (m, s) => renderDemarkPivots(m, s),
    'cypher-pattern':      (m, s) => renderCypherPattern(m, s),
    'cup-and-handle':      (m, s) => renderCupAndHandle(m, s),
    'cusum':               (m, s) => renderCusum(m, s),
    // Microstructure / TCA.
    'vpin':                (m, s) => renderVpin(m, s),
    'order-book-imbalance': (m, s) => renderOrderBookImbalance(m, s),
    'order-flow':          (m, s) => renderOrderFlow(m, s),
    'open-type':           (m, s) => renderOpenType(m, s),
    'market-profile':      (m, s) => renderMarketProfile(m, s),
    'oi-change':           (m, s) => renderOiChange(m, s),
    'almgren-chriss':      (m, s) => renderAlmgrenChriss(m, s),
    'implementation-shortfall': (m, s) => renderImplementationShortfall(m, s),
    'market-impact':       (m, s) => renderMarketImpact(m, s),
    'liquidity':           (m, s) => renderLiquidity(m, s),
    'spread-tracker':      (m, s) => renderSpreadTracker(m, s),
    'intraday-heatmap':    (m, s) => renderIntradayHeatmap(m, s),
    'vwap-slippage':       (m, s) => renderVwapSlippage(m, s),
    'twap':                (m, s) => renderTwap(m, s),
    'news-event':          (m, s) => renderNewsEvent(m, s),
    'stop-loss-best-of':   (m, s) => renderStopLossBestOf(m, s),
    'squeeze-alerts':      (m, s) => renderSqueezeAlerts(m, s),
    'footprint':           (m, s) => renderFootprint(m, s),
    'stress-test':         (m, s) => renderStressTest(m, s),
    'chandelier-stop':     (m, s) => renderChandelierStop(m, s),
    'triple-screen':       (m, s) => renderTripleScreen(m, s),
    'alert-rules':         (m, s) => renderAlertRules(m, s),
    'daily-loss-limit':    (m, s) => renderDailyLossLimit(m, s),
    'drawdown-throttle':   (m, s) => renderDrawdownThrottle(m, s),
    'goal-tracker':        (m, s) => renderGoalTracker(m, s),
    'trade-plan-checklist':(m, s) => renderTradePlanChecklist(m, s),
    'regime-equity':       (m, s) => renderRegimeEquity(m, s),
    'vol-stop-close':      (m, s) => renderVolStopClose(m, s),
    'time-in-force':       (m, s) => renderTimeInForce(m, s),
    'clusters-trade-features': (m, s) => renderClustersTradeFeatures(m, s),
    'clusters-correlation': (m, s) => renderClustersCorrelation(m, s),
    'setups-by-setup':     (m, s) => renderSetupsBySetup(m, s),
    'cohort-tilt':         (m, s) => renderCohortTilt(m, s),
    'choppiness':          (m, s) => renderChoppiness(m, s),
    'var-estimator':       (m, s) => renderVarEstimator(m, s),
    'kelly':               (m, s) => renderKelly(m, s),
    'mc-trades':           (m, s) => renderMcTrades(m, s),
    'keyboard-shortcuts':  (m, s) => renderKeyboardShortcuts(m, s),
    'commission-optimizer': (m, s) => renderCommissionOptimizer(m, s),
    'margin-runway':       (m, s) => renderMarginRunway(m, s),
    'risk-parity':         (m, s) => renderRiskParity(m, s),
    'risk-on-off':         (m, s) => renderRiskOnOff(m, s),
    'risk-reward':         (m, s) => renderRiskReward(m, s),
    'tax-loss-harvest':    (m, s) => renderTaxLossHarvest(m, s),
    'wash-sale':           (m, s) => renderWashSale(m, s),
    'buying-power':        (m, s) => renderBuyingPower(m, s),
    'margin-call':         (m, s) => renderMarginCall(m, s),
    'vix-term-structure':  (m, s) => renderVixTermStructure(m, s),
    'currency-exposure':   (m, s) => renderCurrencyExposure(m, s),
    'bond-duration':       (m, s) => renderBondDuration(m, s),
    'carry-score':         (m, s) => renderCarryScore(m, s),
    'yield-curve':         (m, s) => renderYieldCurve(m, s),
    'cost-basis':          (m, s) => renderCostBasis(m, s),
    'stop-loss-backtest':  (m, s) => renderStopLossBacktest(m, s),
    'futures-roll':        (m, s) => renderFuturesRoll(m, s),
    'heatmap-dow-hour':    (m, s) => renderHeatmapDowHour(m, s),
    'atr-cone':            (m, s) => renderAtrCone(m, s),
    'round-levels':        (m, s) => renderRoundLevels(m, s),
    'kyles-lambda':        (m, s) => renderKylesLambda(m, s),
    'hawkes':              (m, s) => renderHawkesIntensity(m, s),
    'kagi':                (m, s) => renderKagiChart(m, s),
    'risk-parity-solver':  (m, s) => renderRiskParitySolver(m, s),
    'volume-at-price':     (m, s) => renderVolumeAtPrice(m, s),
    'herfindahl':          (m, s) => renderHerfindahl(m, s),
    'roll-spread':         (m, s) => renderRollSpread(m, s),
    'three-line-break':    (m, s) => renderThreeLineBreak(m, s),
    'momentum-crash':      (m, s) => renderMomentumCrash(m, s),
    'effective-spread':    (m, s) => renderEffectiveSpread(m, s),
    'weighted-midprice':   (m, s) => renderWeightedMidprice(m, s),
    'marginal-var':        (m, s) => renderMarginalVar(m, s),
    'range-bar':           (m, s) => renderRangeBar(m, s),
    'tick-bar':            (m, s) => renderTickBar(m, s),
    'volume-bar':          (m, s) => renderVolumeBar(m, s),
    'dollar-bar':          (m, s) => renderDollarBar(m, s),
    'active-share':        (m, s) => renderActiveShare(m, s),
    'brinson':             (m, s) => renderBrinson(m, s),
    'equivolume':          (m, s) => renderEquivolume(m, s),
    'imbalance-bar':       (m, s) => renderImbalanceBar(m, s),
    'black-litterman':     (m, s) => renderBlackLitterman(m, s),
    'adf-test':            (m, s) => renderAdfTest(m, s),
    'aroon':               (m, s) => renderAroon(m, s),
    'amihud':              (m, s) => renderAmihud(m, s),
    'breadth-thrust':      (m, s) => renderBreadthThrust(m, s),
    'bb-squeeze':          (m, s) => renderBollingerSqueeze(m, s),
    'balance-of-power':    (m, s) => renderBalanceOfPower(m, s),
    'anchored-momentum':   (m, s) => renderAnchoredMomentum(m, s),
    'acf':                 (m, s) => renderAcf(m, s),
    'beta':                (m, s) => renderBeta(m, s),
    'brier-score':         (m, s) => renderBrierScore(m, s),
    'bipower-variation':   (m, s) => renderBipowerVariation(m, s),
    'bootstrap-pnl':       (m, s) => renderBootstrapPnl(m, s),
    'block-bootstrap':     (m, s) => renderBlockBootstrap(m, s),
    'ad-normality':        (m, s) => renderAdNormality(m, s),
    'arch-lm':             (m, s) => renderArchLm(m, s),
    'alma':                (m, s) => renderAlma(m, s),
    'alphatrend':          (m, s) => renderAlphatrend(m, s),
    'atr-channel':         (m, s) => renderAtrChannel(m, s),
    'atr-trailing-stop':   (m, s) => renderAtrTrailStop(m, s),
    'adl':                 (m, s) => renderAdl(m, s),
    'asi':                 (m, s) => renderAsi(m, s),
    'ad-oscillator':       (m, s) => renderAdOscillator(m, s),
    'beta-shrinkage':      (m, s) => renderBetaShrink(m, s),
    'bartlett-variance':   (m, s) => renderBartlett(m, s),
    'bid-ask-volume-ratio': (m, s) => renderBidAskVol(m, s),
    'bollinger-band-width': (m, s) => renderBbw(m, s),
    'bollinger-bandwidth-percentile': (m, s) => renderBbwp(m, s),
    'bollinger-percent-b':  (m, s) => renderBbPercentB(m, s),
    'bollinger-band-distance': (m, s) => renderBbd(m, s),
    'bollinger-oscillators': (m, s) => renderBbOsc(m, s),
    'borrow-rate-indicator': (m, s) => renderBorrowRate(m, s),
    'breusch-pagan':       (m, s) => renderBpTest(m, s),
    'burke-ratio':         (m, s) => renderBurke(m, s),
    'camarilla-pivots':    (m, s) => renderCamarilla(m, s),
    'breusch-godfrey':     (m, s) => renderBgTest(m, s),
    'candle-strength-index': (m, s) => renderCsi(m, s),
    'carhart-4':           (m, s) => renderCarhart4(m, s),
    'centered-smoothed-momentum': (m, s) => renderCsm(m, s),
    'chaikin-oscillator':  (m, s) => renderChaikinOsc(m, s),
    'chande-dynamic-momentum': (m, s) => renderCdmi(m, s),
    'chande-kroll-stop':   (m, s) => renderCks(m, s),
    'chande-momentum-oscillator': (m, s) => renderCmo(m, s),
    'chande-trend-index':  (m, s) => renderCti(m, s),
    'chande-volatility-index': (m, s) => renderCvi(m, s),
    'chandelier-exit':     (m, s) => renderChandelier(m, s),
    'cholesky':            (m, s) => renderCholesky(m, s),
    'abc-pattern':         (m, s) => renderAbcPattern(m, s),
    'absorption':          (m, s) => renderAbsorption(m, s),
    'favorites':           (m, s) => renderFavoritesManager(m, s),
    'per-symbol-slippage': (m, s) => renderPerSymbolSlippage(m, s),
    'order-staleness':     (m, s) => renderOrderStaleness(m, s),
    // Options analytics.
    'iv-rank':             (m, s) => renderIvRank(m, s),
    'iv-backtest':         (m, s) => renderIvBacktest(m, s),
    'greeks-profile':      (m, s) => renderGreeksProfile(m, s),
    'second-order-greeks': (m, s) => renderSecondOrderGreeks(m, s),
    // Risk / sizing.
    'deflated-sharpe':     (m, s) => renderDeflatedSharpe(m, s),
    'pyramid':             (m, s) => renderPyramid(m, s),
};

window.addEventListener('tv:authed', () => boot());
