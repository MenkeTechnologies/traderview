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
import { startAlertEngine, requestNotifPermission } from './alert_engine.js';
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
        });
    });
    window.addEventListener('hashchange', dispatch);
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
