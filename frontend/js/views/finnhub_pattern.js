// Finnhub Pattern Recognition — fetches detected technical patterns
// (head & shoulders, double tops/bottoms, triangles, etc.) on a chosen
// symbol + resolution. Pure consumer of /scan/pattern.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const RESOLUTIONS = ['1', '5', '15', '30', '60', 'D', 'W', 'M'];
let state = { symbol: '', resolution: 'D', rows: [] };

export async function renderFinnhubPattern(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.finnhub_pattern.h1.title">// PATTERN RECOGNITION</span></h1>
        <p class="muted small" data-i18n="view.finnhub_pattern.hint.intro">
            Finnhub /scan/pattern detector — head &amp; shoulders, double tops/bottoms,
            triangles, flags. Useful as a quick second opinion vs. your own indicators.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="fp-form">
                <label><span data-i18n="view.finnhub_pattern.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="AAPL" required></label>
                <label><span data-i18n="view.finnhub_pattern.label.resolution">Resolution</span>
                    <select name="resolution">${RESOLUTIONS.map(r =>
                        `<option value="${r}" ${r === state.resolution ? 'selected' : ''}>${r}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" type="submit" data-i18n="view.finnhub_pattern.btn.scan">Scan</button>
            </form>
            <div id="fp-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('fp-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        state.resolution = fd.get('resolution') || 'D';
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('fp-result');
    if (el) el.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const data = await api.symbolScanPattern(state.symbol, state.resolution);
        if (!viewIsCurrent(tok)) return;
        const points = data?.points || [];
        if (!points.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.finnhub_pattern.empty">No patterns detected at this resolution.</p>`;
            return;
        }
        el.innerHTML = `<table class="trades">
            <thead><tr>
                <th data-i18n="view.finnhub_pattern.th.pattern">Pattern</th>
                <th data-i18n="view.finnhub_pattern.th.type">Type</th>
                <th data-i18n="view.finnhub_pattern.th.status">Status</th>
                <th data-i18n="view.finnhub_pattern.th.entry">Entry</th>
                <th data-i18n="view.finnhub_pattern.th.target">Target</th>
                <th data-i18n="view.finnhub_pattern.th.stop">Stop</th>
                <th data-i18n="view.finnhub_pattern.th.profit">Profit</th>
            </tr></thead>
            <tbody>${points.map(p => `
                <tr>
                    <td>${esc(p.patternname || p.patterntype || '—')}</td>
                    <td class="muted">${esc(p.patterntype || '—')}</td>
                    <td>${esc(p.status || '—')}</td>
                    <td>${p.entry != null ? p.entry.toFixed(2) : '—'}</td>
                    <td>${p.profit1 != null ? p.profit1.toFixed(2) : '—'}</td>
                    <td>${p.stoploss != null ? p.stoploss.toFixed(2) : '—'}</td>
                    <td class="${(p.profit_percent ?? 0) >= 0 ? 'pos' : 'neg'}">${p.profit_percent != null ? p.profit_percent.toFixed(2) + '%' : '—'}</td>
                </tr>
            `).join('')}</tbody>
        </table>`;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.finnhub_pattern.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.finnhub_pattern.toast.failed'), { level: 'error' });
    }
}
