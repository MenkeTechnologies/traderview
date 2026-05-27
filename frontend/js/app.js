// Main entry. Wires tabs, runs auth bootstrap, dispatches to view modules.

import { api, initApi, ApiError } from './api.js';
import { showAuthScreen, hideAuthScreen } from './auth.js';
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
import { startAlertEngine, requestNotifPermission } from './alert_engine.js';
import { startWs, on as onWsEvent } from './ws.js';
import { installHotkeyEngine, reloadHotkeys } from './hotkey_engine.js';

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
        document.getElementById('user-strip').textContent =
            me.is_local ? 'local user' : (me.email || me.display_name || '');
        await loadAccounts();
        renderAccountStrip();
        await dispatch();
        hideAuthScreen();
        // Boot background engines once authenticated.
        startAlertEngine();
        installHotkeyEngine();
        reloadHotkeys();
        requestNotifPermission();
        startWs();
        wireWsStatusIndicator();
    } catch (e) {
        if (e instanceof ApiError && e.status === 401 && state.mode === 'web') {
            showAuthScreen();
        } else {
            document.getElementById('app').innerHTML =
                `<p class="boot">Failed to connect: ${e.message}</p>`;
        }
    }
}

async function loadAccounts() {
    state.accounts = await api.accounts();
    if (state.accounts.length && !state.accountId) state.accountId = state.accounts[0].id;
}

function renderAccountStrip() {
    const strip = document.getElementById('account-strip');
    if (state.accounts.length === 0) {
        strip.innerHTML = '<span class="muted">no account</span>';
        return;
    }
    const options = state.accounts.map(a => `
        <option value="${a.id}" ${a.id === state.accountId ? 'selected' : ''}>${a.broker} · ${a.name}</option>
    `).join('');
    strip.innerHTML = `<select id="account-select" class="account-select">${options}</select>`;
    document.getElementById('account-select').addEventListener('change', (e) => {
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
}

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

function closeNavDrawer() {
    document.body.classList.remove('nav-open');
    const btn = document.getElementById('navToggle');
    if (btn) btn.setAttribute('aria-expanded', 'false');
}

export function go(view, params = '') {
    window.location.hash = view + (params ? `/${params}` : '');
}

export async function dispatch() {
    const hash = (window.location.hash || '#dashboard').slice(1);
    const [view, ...rest] = hash.split('/');
    state.view = view;
    document.querySelectorAll('.tab').forEach(b =>
        b.classList.toggle('active', b.dataset.view === view)
    );
    const mount = document.getElementById('app');
    mount.innerHTML = '<div class="boot">loading…</div>';
    try {
        switch (view) {
            case 'dashboard':   await renderDashboard(mount, state); break;
            case 'trades':      await renderTradesView(mount, state); break;
            case 'new-trade':   await renderNewTrade(mount, state); break;
            case 'search':      await renderSearch(mount, state); break;
            case 'watchlists':  await renderWatchlists(mount, state); break;
            case 'research':    await renderResearch(mount, state, rest[0] || ''); break;
            case 'screener':    await renderScreener(mount, state); break;
            case 'top-signals': await renderTopSignals(mount, state); break;
            case 'scanners':    await renderScanners(mount, state); break;
            case 'sectors':     await renderSectors(mount, state); break;
            case 'paper':       await renderPaper(mount, state); break;
            case 'risk':        await renderRisk(mount, state); break;
            case 'alerts':      await renderAlerts(mount, state); break;
            case 'hotkeys':     await renderHotkeys(mount, state); break;
            case 'replay':      await renderReplay(mount, state, rest[0]); break;
            case 'tape':        await renderTape(mount, state); break;
            case 'earnings-iv': await renderEarningsIv(mount, state, rest[0]); break;
            case 'disclosures': await renderDisclosures(mount, state); break;
            case 'sentiment':   await renderSentiment(mount, state, rest[0]); break;
            case 'heatmap':     await renderHeatmap(mount, state); break;
            case 'options':     await renderOptions(mount, state, rest[0]); break;
            case 'crypto':      await renderCrypto(mount, state); break;
            case 'backtest':    await renderBacktest(mount, state); break;
            case 'economy':     await renderEconomy(mount, state); break;
            case 'pairs':       await renderPairs(mount, state); break;
            case 'short-interest': await renderShortInterest(mount, state, rest[0]); break;
            case 'darkpool':       await renderDarkpool(mount, state, rest[0]); break;
            case 'vol':            await renderVol(mount, state); break;
            case 'webhooks':       await renderWebhooks(mount, state); break;
            case 'breadth':        await renderBreadth(mount, state); break;
            case 'fear-greed':     await renderFearGreed(mount, state); break;
            case 'premarket':      await renderPremarket(mount, state); break;
            case 'vol-surface':    await renderVolSurface(mount, state); break;
            case 'walk-forward':   await renderWalkForward(mount, state); break;
            case 'tax-lots':       await renderTaxLots(mount, state); break;
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
            case 'tape-replay':    await renderTapeReplay(mount, state, rest[0] || ''); break;
            case 'backtest-presets': await renderBacktestPresets(mount, state, rest[0] || ''); break;
            case 'mood':           await renderMoodAnalytics(mount, state); break;
            case 'discipline':     await renderDiscipline(mount, state); break;
            case 'goals':          await renderGoals(mount, state); break;
            case 'r-dist':         await renderRDist(mount, state); break;
            case 'reviews':        await renderTradeReviews(mount, state); break;
            case 'forecast':       await renderEquityForecast(mount, state); break;
            case 'fill-quality':   await renderFillQuality(mount, state); break;
            case 'custom-indicators': await renderCustomIndicators(mount, state); break;
            case 'trade':       await renderTradeDetail(mount, state, rest[0]); break;
            case 'journal':     await renderJournalView(mount, state, rest[0]); break;
            case 'calendar':    await renderCalendar(mount, state); break;
            case 'reports':     await renderReports(mount, state, rest[0] || 'overview'); break;
            case 'charts':      await renderCharts(mount, state, rest[0] || ''); break;
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
            default:            mount.innerHTML = `<p class="boot">Unknown view: ${view}</p>`;
        }
    } catch (e) {
        mount.innerHTML = `<p class="boot">Error: ${e.message}</p>`;
        console.error(e);
    }
}

window.addEventListener('tv:authed', () => boot());
document.addEventListener('DOMContentLoaded', () => {
    bindTabs();
    boot();
});
