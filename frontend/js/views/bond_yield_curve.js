// Bond Yield Curve — Finnhub /bond/yield-curve.
// Shows term-structure for major sovereign curves (US Treasury, UK Gilts, etc.).
// Inversion detector for recession signal.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const CURVES = [
    { value: '10YTRY', label: 'US 10Y Treasury' },
    { value: '2YTRY',  label: 'US 2Y Treasury' },
    { value: '5YTRY',  label: 'US 5Y Treasury' },
    { value: '30YTRY', label: 'US 30Y Treasury' },
    { value: 'UK10Y',  label: 'UK 10Y Gilt' },
    { value: 'DE10Y',  label: 'Germany 10Y Bund' },
    { value: 'JP10Y',  label: 'Japan 10Y JGB' },
];

let state = { code: '10YTRY' };

export async function renderBondYieldCurve(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.bond_yc.h1.title">// BOND YIELD CURVE</span></h1>
        <p class="muted small" data-i18n="view.bond_yc.hint.intro">
            Sovereign yield curves. Inversion (short rate > long rate) is a classic recession
            signal. Watch 2s10s spread on US Treasury for the canonical inversion indicator.
        </p>
        <div class="chart-panel">
            <div class="inline-form">
                <label><span data-i18n="view.bond_yc.label.code">Curve</span>
                    <select id="byc-code">${CURVES.map(c =>
                        `<option value="${c.value}" ${c.value === state.code ? 'selected' : ''}>${esc(c.label)}</option>`
                    ).join('')}</select>
                </label>
                <button class="primary" id="byc-refresh" type="button" data-i18n="view.bond_yc.btn.refresh">Refresh</button>
            </div>
            <div id="byc-chart" style="width:100%;height:280px;margin-top:10px"></div>
            <div id="byc-table" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('byc-code').addEventListener('change', e => {
        state.code = e.target.value;
        void load(tok);
    });
    document.getElementById('byc-refresh').addEventListener('click', () => void load(tok));
    await load(tok);
}

async function load(tok) {
    const tableEl = document.getElementById('byc-table');
    const chartEl = document.getElementById('byc-chart');
    if (tableEl) tableEl.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const data = await api.finnhubBondYieldCurve(state.code);
        if (!viewIsCurrent(tok)) return;
        const rows = data?.data || [];
        if (!rows.length) {
            tableEl.innerHTML = `<p class="muted" data-i18n="view.bond_yc.empty">No yield data (may require premium).</p>`;
            if (chartEl) chartEl.innerHTML = '';
            return;
        }
        const sorted = [...rows].sort((a, b) => String(a.d).localeCompare(String(b.d)));
        const last = sorted[sorted.length - 1];
        const yest = sorted.length > 1 ? sorted[sorted.length - 2] : null;
        const delta = yest ? Number(last.v) - Number(yest.v) : 0;
        const cls = delta > 0 ? 'pos' : delta < 0 ? 'neg' : '';
        tableEl.innerHTML = `
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.bond_yc.card.latest">Latest</div>
                    <div class="value">${Number(last.v).toFixed(3)}%</div>
                </div>
                <div class="card ${cls}">
                    <div class="label" data-i18n="view.bond_yc.card.day_change">Δ vs prior</div>
                    <div class="value">${delta >= 0 ? '+' : ''}${delta.toFixed(3)} bp</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.bond_yc.card.as_of">As of</div>
                    <div class="value">${esc(last.d || '—')}</div>
                </div>
            </div>
            <details style="margin-top:10px">
                <summary data-i18n="view.bond_yc.summary.history">Full history (${sorted.length} points)</summary>
                <table class="trades" style="margin-top:6px">
                    <thead><tr>
                        <th data-i18n="view.bond_yc.th.date">Date</th>
                        <th data-i18n="view.bond_yc.th.yield">Yield %</th>
                    </tr></thead>
                    <tbody>${sorted.slice(-200).reverse().map(r =>
                        `<tr><td class="muted">${esc(r.d)}</td><td>${Number(r.v).toFixed(3)}</td></tr>`
                    ).join('')}</tbody>
                </table>
            </details>
        `;
        if (chartEl && window.uPlot) {
            chartEl.innerHTML = '';
            const xs = sorted.map(r => new Date(r.d).getTime() / 1000);
            const ys = sorted.map(r => Number(r.v));
            new window.uPlot({
                title: '', width: chartEl.clientWidth || 800, height: 280,
                scales: { x: { time: true }, y: { auto: true } },
                series: [
                    {},
                    { label: t('view.bond_yc.label.yield'),
                      stroke: '#f0b800', width: 1.5, fill: 'rgba(240,184,0,0.10)' },
                ],
                axes: [
                    { stroke: '#aab', size: 28 },
                    { stroke: '#aab', size: 50,
                      values: (_u, splits) => splits.map(v => v.toFixed(2) + '%') },
                ],
                legend: { show: false },
            }, [xs, ys], chartEl);
        }
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (tableEl) tableEl.innerHTML = `<p class="muted neg">${esc(t('view.bond_yc.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.bond_yc.toast.failed'), { level: 'error' });
    }
}
