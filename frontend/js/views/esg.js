// ESG Scores — current + historical from Finnhub /stock/esg + /stock/historical-esg.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '' };

export async function renderEsg(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.esg.h1.title">// ESG SCORES</span></h1>
        <p class="muted small" data-i18n="view.esg.hint.intro">
            Environmental / Social / Governance scores. ESG-mandated funds increasingly
            screen on these; sudden score drops correlate with institutional sell flows.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="esg-form">
                <label><span data-i18n="view.esg.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="MSFT" required></label>
                <button class="primary" type="submit" data-i18n="view.esg.btn.load">Load</button>
            </form>
        </div>
        <div class="panel-grid">
            <div class="chart-panel"><h2 data-i18n="view.esg.h2.current">Current scores</h2>
                <div id="esg-current"></div></div>
            <div class="chart-panel"><h2 data-i18n="view.esg.h2.historical">Historical scores</h2>
                <div id="esg-historical"></div></div>
        </div>
    `;
    document.getElementById('esg-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const curEl = document.getElementById('esg-current');
    const histEl = document.getElementById('esg-historical');
    if (curEl) curEl.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    if (histEl) histEl.innerHTML = `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;
    try {
        const [cur, hist] = await Promise.all([
            api.symbolEsg(state.symbol).catch(() => null),
            api.symbolEsgHistorical(state.symbol).catch(() => null),
        ]);
        if (!viewIsCurrent(tok)) return;
        renderCurrent(curEl, cur);
        renderHistorical(histEl, hist);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.esg.toast.failed'), { level: 'error' });
    }
}

function renderCurrent(el, data) {
    if (!el) return;
    if (!data || typeof data !== 'object' || !Object.keys(data).length) {
        el.innerHTML = `<p class="muted" data-i18n="view.esg.empty.current">No ESG data.</p>`;
        return;
    }
    const total = Number(data.totalESGScore || 0);
    const env = Number(data.environmentScore || 0);
    const soc = Number(data.socialScore || 0);
    const gov = Number(data.governanceScore || 0);
    const cls = s => s >= 70 ? 'pos' : s <= 30 ? 'neg' : '';
    el.innerHTML = `
        <div class="cards">
            <div class="card ${cls(total)}">
                <div class="label" data-i18n="view.esg.card.total">Total ESG</div>
                <div class="value">${total.toFixed(1)}</div></div>
            <div class="card ${cls(env)}">
                <div class="label" data-i18n="view.esg.card.env">Environment</div>
                <div class="value">${env.toFixed(1)}</div></div>
            <div class="card ${cls(soc)}">
                <div class="label" data-i18n="view.esg.card.soc">Social</div>
                <div class="value">${soc.toFixed(1)}</div></div>
            <div class="card ${cls(gov)}">
                <div class="label" data-i18n="view.esg.card.gov">Governance</div>
                <div class="value">${gov.toFixed(1)}</div></div>
            <div class="card">
                <div class="label" data-i18n="view.esg.card.risk_level">Risk level</div>
                <div class="value">${esc(data.ESGRiskLevel || '—')}</div></div>
            <div class="card">
                <div class="label" data-i18n="view.esg.card.controversy">Controversy</div>
                <div class="value">${data.controversyLevel ?? '—'}</div></div>
        </div>
    `;
}

function renderHistorical(el, data) {
    if (!el) return;
    const rows = data?.data || (Array.isArray(data) ? data : []);
    if (!rows.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.esg.empty.historical">No historical scores.</p>`;
        return;
    }
    const sorted = [...rows].sort((a, b) => (a.year || 0) - (b.year || 0));
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.esg.th.year">Year</th>
            <th data-i18n="view.esg.th.total">Total</th>
            <th data-i18n="view.esg.th.env">Env</th>
            <th data-i18n="view.esg.th.soc">Soc</th>
            <th data-i18n="view.esg.th.gov">Gov</th>
        </tr></thead>
        <tbody>${sorted.map(r => `
            <tr>
                <td>${r.year ?? '—'}</td>
                <td>${num(r.totalESGScore)}</td>
                <td>${num(r.environmentScore)}</td>
                <td>${num(r.socialScore)}</td>
                <td>${num(r.governanceScore)}</td>
            </tr>
        `).join('')}</tbody>
    </table>`;
}

function num(v) {
    if (v == null || !Number.isFinite(Number(v))) return '—';
    return Number(v).toFixed(1);
}
