// Earnings Quality Score — Finnhub /stock/earnings-quality-score.
// Score 1-10 across multiple quality dimensions: profitability, growth,
// cash generation, capital efficiency, leverage. Surfaces "fake" earnings
// (e.g. capital-light businesses with healthy free cash flow vs. heavy GAAP).

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const FREQS = ['annual', 'quarterly'];
let state = { symbol: '', freq: 'annual' };

export async function renderEarningsQuality(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.eqs.h1.title">// EARNINGS QUALITY SCORE</span></h1>
        <p class="muted small" data-i18n="view.eqs.hint.intro">
            Score 1-10 across profitability, growth, cash generation, capital efficiency, leverage.
            Companies with low score + high GAAP EPS often see post-earnings disappointment.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="eqs-form">
                <label><span data-i18n="view.eqs.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="AAPL" required></label>
                <label><span data-i18n="view.eqs.label.freq">Frequency</span>
                    <select name="freq">${FREQS.map(f =>
                        `<option value="${f}" ${f === state.freq ? 'selected' : ''}>${esc(t('view.estimates.freq.' + f))}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" type="submit" data-i18n="view.eqs.btn.load">Load</button>
            </form>
            <div id="eqs-result" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('eqs-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        state.freq = fd.get('freq') || 'annual';
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const el = document.getElementById('eqs-result');
    if (el) el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const data = await api.symbolEarningsQualityScore(state.symbol, state.freq);
        if (!viewIsCurrent(tok)) return;
        const rows = data?.data || [];
        if (!rows.length) {
            el.innerHTML = `<p class="muted" data-i18n="view.eqs.empty">No quality scores available.</p>`;
            return;
        }
        const latest = rows[rows.length - 1] || {};
        const dims = [
            { key: 'profitability',     label: t('view.eqs.dim.profitability') },
            { key: 'growth',            label: t('view.eqs.dim.growth') },
            { key: 'cashGeneration',    label: t('view.eqs.dim.cash_gen') },
            { key: 'capitalAllocation', label: t('view.eqs.dim.cap_alloc') },
            { key: 'leverage',          label: t('view.eqs.dim.leverage') },
        ];
        const overall = Number(latest.score || 0);
        const cls = s => s >= 7 ? 'pos' : s <= 3 ? 'neg' : '';
        el.innerHTML = `
            <div class="cards">
                <div class="card ${cls(overall)}">
                    <div class="label" data-i18n="view.eqs.card.overall">Overall</div>
                    <div class="value">${overall.toFixed(1)}/10</div>
                </div>
                ${dims.map(d => {
                    const v = Number(latest[d.key] || 0);
                    return `<div class="card ${cls(v)}">
                        <div class="label">${esc(d.label)}</div>
                        <div class="value">${v.toFixed(1)}</div>
                    </div>`;
                }).join('')}
            </div>
            <table class="trades" style="margin-top:10px">
                <thead><tr>
                    <th data-i18n="view.eqs.th.period">Period</th>
                    <th data-i18n="view.eqs.th.overall">Overall</th>
                    ${dims.map(d => `<th>${esc(d.label)}</th>`).join('')}
                </tr></thead>
                <tbody>${rows.slice(-12).reverse().map(r => `
                    <tr>
                        <td>${esc(r.period || '—')}</td>
                        <td class="${cls(Number(r.score || 0))}">${Number(r.score || 0).toFixed(1)}</td>
                        ${dims.map(d => `<td>${Number(r[d.key] || 0).toFixed(1)}</td>`).join('')}
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (el) el.innerHTML = `<p class="muted neg">${esc(t('view.eqs.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.eqs.toast.failed'), { level: 'error' });
    }
}
