// Stock Splits History — Finnhub /stock/split.
// Per-symbol split log + computes pre/post-split share count for accounting.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '' };

export async function renderSplitsHistory(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.splits.h1.title">// STOCK SPLITS HISTORY</span></h1>
        <p class="muted small" data-i18n="view.splits.hint.intro">
            Per-symbol historical splits + reverse splits. Reverse splits often signal
            distressed companies trying to maintain listing — typically bearish setup
            for short-term traders. Forward splits historically bullish (NVDA, TSLA).
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="sp-form">
                <label><span data-i18n="view.splits.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="AAPL" required></label>
                <button class="primary" type="submit" data-i18n="view.splits.btn.load">Load</button>
            </form>
            <div id="sp-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('sp-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('sp-result');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const to = new Date();
        const from = new Date(to);
        from.setFullYear(from.getFullYear() - 30);
        const data = await api.symbolSplits(state.symbol, fmtDay(from), fmtDay(to));
        if (!viewIsCurrent(tok)) return;
        const rows = Array.isArray(data) ? data : (data?.data || []);
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.splits.empty">No splits on record.</p>`;
            return;
        }
        const sorted = [...rows].sort((a, b) =>
            String(b.date || '').localeCompare(String(a.date || '')));
        // Compute cumulative multiplier from earliest to today.
        const ascending = [...rows].sort((a, b) =>
            String(a.date || '').localeCompare(String(b.date || '')));
        let cumulative = 1;
        const cumMap = new Map();
        for (const r of ascending) {
            const f = Number(r.fromFactor) || 1;
            const tFac = Number(r.toFactor) || 1;
            cumulative *= (tFac / f);
            cumMap.set(r.date, cumulative);
        }
        const totalMultiplier = cumulative;
        // Test: 100 shares pre-history = how many today?
        const hypotheticalGrew = 100 * totalMultiplier;
        el.innerHTML = `
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.splits.card.total_splits">Total splits</div>
                    <div class="value">${rows.length}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.splits.card.multiplier">Cumulative multiplier</div>
                    <div class="value">${totalMultiplier.toFixed(2)}×</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.splits.card.hypothetical">100 sh in ${rows.length > 0 ? new Date(ascending[0].date).getFullYear() : '?'}  →  today</div>
                    <div class="value">${hypotheticalGrew.toLocaleString(undefined, { maximumFractionDigits: 0 })} sh</div>
                </div>
            </div>
            <table class="trades" style="margin-top:10px">
                <thead><tr>
                    <th data-i18n="view.splits.th.date">Date</th>
                    <th data-i18n="view.splits.th.ratio">Ratio</th>
                    <th data-i18n="view.splits.th.type">Type</th>
                    <th data-i18n="view.splits.th.cumulative">Cumulative</th>
                </tr></thead>
                <tbody>${sorted.map(r => {
                    const f = Number(r.fromFactor) || 1;
                    const tFac = Number(r.toFactor) || 1;
                    const ratio = tFac / f;
                    const isReverse = ratio < 1;
                    return `<tr>
                        <td>${esc(r.date || '—')}</td>
                        <td>${tFac}:${f}</td>
                        <td class="${isReverse ? 'neg' : 'pos'}">${esc(isReverse ? t('view.splits.type.reverse') : t('view.splits.type.forward'))}</td>
                        <td class="muted">${(cumMap.get(r.date) || 1).toFixed(2)}×</td>
                    </tr>`;
                }).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.splits.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.splits.toast.failed'), { level: 'error' });
    }
}

function fmtDay(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}
