// Finnhub Support / Resistance — per-symbol S/R levels via /scan/support-resistance.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const RESOLUTIONS = ['1', '5', '15', '30', '60', 'D', 'W', 'M'];
let state = { symbol: '', resolution: 'D' };

export async function renderFinnhubSr(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.finnhub_sr.h1.title">// SUPPORT / RESISTANCE</span></h1>
        <p class="muted small" data-i18n="view.finnhub_sr.hint.intro">
            Finnhub /scan/support-resistance — algorithmically detected price levels.
            Higher resolution = stronger zones (weekly/monthly levels matter more than 1-min).
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="fsr-form">
                <label><span data-i18n="view.finnhub_sr.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="AAPL" required></label>
                <label><span data-i18n="view.finnhub_sr.label.resolution">Resolution</span>
                    <select name="resolution">${RESOLUTIONS.map(r =>
                        `<option value="${r}" ${r === state.resolution ? 'selected' : ''}>${r}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" type="submit" data-i18n="view.finnhub_sr.btn.scan">Scan</button>
            </form>
            <div id="fsr-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('fsr-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        state.resolution = fd.get('resolution') || 'D';
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('fsr-result');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const data = await api.symbolScanSr(state.symbol, state.resolution);
        if (!viewIsCurrent(tok)) return;
        const levels = data?.levels || [];
        if (!levels.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.finnhub_sr.empty">No S/R levels at this resolution.</p>`;
            return;
        }
        const sorted = [...levels].sort((a, b) => b - a);
        el.innerHTML = `<table class="trades">
            <thead><tr>
                <th data-i18n="view.finnhub_sr.th.rank">#</th>
                <th data-i18n="view.finnhub_sr.th.level">Level</th>
            </tr></thead>
            <tbody>${sorted.map((lvl, i) =>
                `<tr><td>${i + 1}</td><td>$${lvl.toFixed(2)}</td></tr>`
            ).join('')}</tbody>
        </table>`;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.finnhub_sr.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.finnhub_sr.toast.failed'), { level: 'error' });
    }
}
