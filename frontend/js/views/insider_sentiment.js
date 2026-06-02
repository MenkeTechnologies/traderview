// Insider Sentiment — Finnhub /stock/insider-sentiment.
// Monthly MSPR (Monthly Share Purchase Ratio) score that aggregates insider
// transactions into a sentiment number. > 0 = net bullish, < 0 = net bearish.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '' };

export async function renderInsiderSentiment(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.is.h1.title">// INSIDER SENTIMENT</span></h1>
        <p class="muted small" data-i18n="view.is.hint.intro">
            Monthly Share Purchase Ratio (MSPR) aggregates insider transactions into a
            single sentiment number. &gt; 0 = net bullish, &lt; 0 = net bearish. Pair with
            the Insider Transactions view for the underlying trades.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="is-form">
                <label><span data-i18n="view.is.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="NVDA" required></label>
                <button class="primary" type="submit" data-i18n="view.is.btn.load">Load</button>
            </form>
            <div id="is-chart" style="width:100%;height:240px;margin-top:10px"></div>
            <div id="is-table" style="margin-top:10px"></div>
        </div>
    `;
    document.getElementById('is-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const tEl = document.getElementById('is-table');
    const cEl = document.getElementById('is-chart');
    if (tEl) tEl.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`;
    try {
        const to = new Date();
        const from = new Date(to);
        from.setFullYear(from.getFullYear() - 2);
        const data = await api.symbolInsiderSentiment(state.symbol, fmtDay(from), fmtDay(to));
        if (!viewIsCurrent(tok)) return;
        const rows = data?.data || [];
        if (!rows.length) {
            tEl.innerHTML = `<p class="muted" data-i18n="view.is.empty">No sentiment data (or premium required).</p>`;
            if (cEl) cEl.innerHTML = '';
            return;
        }
        const sorted = [...rows].sort((a, b) =>
            (a.year - b.year) || (a.month - b.month));
        const last = sorted[sorted.length - 1];
        const mspr = Number(last.mspr || 0);
        const cls = mspr > 0.3 ? 'pos' : mspr < -0.3 ? 'neg' : '';
        tEl.innerHTML = `
            <div class="cards">
                <div class="card ${cls}">
                    <div class="label" data-i18n="view.is.card.latest_mspr">Latest MSPR</div>
                    <div class="value">${mspr.toFixed(3)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.is.card.month">Month</div>
                    <div class="value">${esc(`${last.year}-${String(last.month).padStart(2, '0')}`)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.is.card.change">Net share change</div>
                    <div class="value ${Number(last.change || 0) >= 0 ? 'pos' : 'neg'}">
                        ${Number(last.change || 0).toLocaleString()}
                    </div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.is.card.months_tracked">Months tracked</div>
                    <div class="value">${rows.length}</div>
                </div>
            </div>
            <table class="trades" style="margin-top:10px">
                <thead><tr>
                    <th data-i18n="view.is.th.month">Month</th>
                    <th data-i18n="view.is.th.mspr">MSPR</th>
                    <th data-i18n="view.is.th.change">Net shares</th>
                </tr></thead>
                <tbody>${[...sorted].reverse().slice(0, 24).map(r => {
                    const m = Number(r.mspr || 0);
                    const c = m > 0.3 ? 'pos' : m < -0.3 ? 'neg' : '';
                    return `<tr>
                        <td>${esc(`${r.year}-${String(r.month).padStart(2, '0')}`)}</td>
                        <td class="${c}">${m.toFixed(3)}</td>
                        <td>${Number(r.change || 0).toLocaleString()}</td>
                    </tr>`;
                }).join('')}</tbody>
            </table>
        `;
        if (cEl && window.uPlot) {
            cEl.innerHTML = '';
            const xs = sorted.map(r =>
                new Date(`${r.year}-${String(r.month).padStart(2, '0')}-15`).getTime() / 1000);
            const ys = sorted.map(r => Number(r.mspr || 0));
            new window.uPlot({
                title: '', width: cEl.clientWidth || 800, height: 240,
                scales: { x: { time: true }, y: { auto: true } },
                series: [
                    {},
                    { label: 'MSPR', stroke: '#00e5ff', width: 1.5,
                      fill: 'rgba(0,229,255,0.08)' },
                ],
                axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 50 }],
                legend: { show: false },
            }, [xs, ys], cEl);
        }
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        if (tEl) tEl.innerHTML = `<p class="muted neg">${esc(t('view.is.error.load', { msg: e.message || e }))}</p>`;
        showToast(t('view.is.toast.failed'), { level: 'error' });
    }
}

function fmtDay(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}
