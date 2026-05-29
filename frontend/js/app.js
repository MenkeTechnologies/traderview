// Main entry. Wires tabs, runs auth bootstrap, dispatches to view modules.

import { api, initApi, ApiError } from './api.js';
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
import { installShortcuts } from './shortcuts.js';
import { installCommandPalette } from './command_palette.js';
import { bootI18n, applyUiI18n } from './i18n.js';
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
            userStrip.textContent = me.is_local ? 'local user' : (me.email || me.display_name || '');
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
            if (appEl) appEl.innerHTML = `<p class="boot">Failed to connect: ${e.message}</p>`;
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
        strip.innerHTML = '<span class="muted">no account</span>';
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
    installSymbolHotkey();
    void bootI18n('en').then(() => {
        applyUiI18n();
        const picker = document.getElementById('locale-picker');
        if (picker) {
            const saved = (typeof localStorage !== 'undefined') ? localStorage.getItem('tv-locale-v1') : null;
            if (saved) picker.value = saved;
            picker.addEventListener('change', async (e) => {
                const { loadLocale } = await import('./i18n.js');
                await loadLocale(e.target.value);
            });
        }
    });
    // Bridge: hash-based help/tutorial action from the new registry.
    window.addEventListener('tv:open-help', () => { window.location.hash = 'tutorial'; });
}

// Cmd+K + ? are now owned by ./shortcuts.js → tv:open-palette /
// tv:open-help events. ./command_palette.js handles the overlay.

function bindNavToggle() {
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
    mount.innerHTML = spinnerHTML(`loading ${view}…`);
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
            default:            mount.innerHTML = `<p class="boot">Unknown view: ${view}</p>`;
        }
    } catch (e) {
        mount.innerHTML = `<p class="boot">Error: ${e.message}</p>`;
        console.error(e);
    }
    // Translate any `data-i18n*` attributes the view just emitted.
    try { applyUiI18n(mount); } catch { /* i18n optional */ }
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
