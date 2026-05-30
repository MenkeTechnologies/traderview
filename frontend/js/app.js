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
import { renderImportView } from './views/import.js';
import { renderPlans } from './views/plans.js';
import { renderTags } from './views/tags.js';
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
import { renderHalts } from './views/halts.js';
import { renderLauncher } from './views/launcher.js';
import { renderLiveScanner } from './views/live_scanner.js';
import { renderCatalysts } from './views/catalysts.js';
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
import { installHotkeyEngine, reloadHotkeys } from './hotkey_engine.js';
import { renderExpensesView } from './views/expenses.js';

export const state = {
    mode: 'web',
    accountId: null,
    accounts: [],
    me: null,
    view: 'dashboard',
};

async function boot() {
    await initApi();
    try {
        const cfg = await api.config();
        state.mode = cfg.mode;
    } catch (_) { /* server may not be reachable yet */ }
    try {
        const me = await api.me();
        state.me = me;
        const userStrip = document.getElementById('user-strip');
        if (userStrip) {
            userStrip.textContent = me.is_local ? t('app.user.local') : (me.email || me.display_name || '');
        }
        await loadAccounts();
        renderAccountStrip();
        await dispatch();
        hideAuthScreen();
        // Boot background engines once authenticated. installHotkeyEngine
        // already calls reloadHotkeys() internally — don't double-fetch.
        startAlertEngine();
        installHotkeyEngine();
        requestNotifPermission();
        startWs();
        wireWsStatusIndicator();
        wireKillSwitchIndicator();
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
    if (state.accounts.length && !state.accountId) state.accountId = state.accounts[0].id;
}

function renderAccountStrip() {
    const strip = document.getElementById('account-strip');
    if (!strip) return;
    if (state.accounts.length === 0) {
        strip.innerHTML = `<span class="muted">${esc(t('app.account_strip.no_account'))}</span>`;
        return;
    }
    const options = state.accounts.map(a => `
        <option value="${a.id}" ${a.id === state.accountId ? 'selected' : ''}>${a.broker} · ${a.name}</option>
    `).join('');
    strip.innerHTML = `<select id="account-select" class="account-select">${options}</select>`;
    const sel = strip.querySelector('#account-select');
    if (sel) sel.addEventListener('change', (e) => {
        state.accountId = e.target.value;
        dispatch();
    });
}

function bindTabs() {
    document.querySelectorAll('.tab').forEach(btn => {
        btn.addEventListener('click', () => {
            window.location.hash = btn.dataset.view;
            // Auto-close mobile drawer after picking a tab.
            closeNavDrawer();
        });
    });
    window.addEventListener('hashchange', dispatch);
    bindNavToggle();
    installShortcuts();
    installCommandPalette();
    installToasts();
    installDialog();
    installContextMenu();
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
    const [view, ...rest] = hash.split('/');
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
            case 'halts':          await renderHalts(mount, state); break;
            case 'live-scanner':   await renderLiveScanner(mount, state); break;
            case 'catalysts':      await renderCatalysts(mount, state); break;
            case 'webull':         await renderWebull(mount, state); break;
            case 'vol-surface':    await renderVolSurface(mount, state); break;
            case 'walk-forward':   await renderWalkForward(mount, state); break;
            case 'tax-lots':       await renderTaxLots(mount, state); break;
            case 'expenses':       await renderExpensesView(mount); break;
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
            case 'import':      await renderImportView(mount, state); break;
            case 'plans':       await renderPlans(mount, state); break;
            case 'tags':        await renderTags(mount, state); break;
            case 'mentorship':  await renderMentorship(mount, state); break;
            case 'community':   if (rest.length === 2) await renderCommunityThread(mount, state, rest[0], rest[1]);
                                else await renderCommunity(mount, state, rest[0]); break;
            case 'shares':      await renderShares(mount, state); break;
            case 'shared':      await renderSharedTrade(mount, state, rest[0]); break;
            case 'accounts':    await renderAccounts(mount, state, () => { renderAccountStrip(); }); break;
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
document.addEventListener('DOMContentLoaded', () => {
    bindTabs();
    boot();
});
