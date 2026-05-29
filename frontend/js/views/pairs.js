// Correlation matrix + per-pair spread/z-score analyzer.
import { api } from '../api.js';
import { barChart } from '../charts.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n } from '../i18n.js';

export async function renderPairs(mount) {
    const tok = currentViewToken();
    const lists = await api.watchlists();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.pairs.h1.pairs_correlation" class="view-title">// PAIRS / CORRELATION</h1>
        <p data-i18n="view.pairs.hint.pearson_correlation_matrix_over_log_returns_per_pa" class="muted small">Pearson correlation matrix over log-returns + per-pair OLS spread &amp; z-score
        (mean-reversion stat-arb signal — |z| ≥ 2 → consider fading).</p>

        <div class="chart-panel">
            <form id="cf" class="inline-form">
                <label><span data-i18n="view.pairs.label.symbols">Symbols</span>
                    <input name="symbols" placeholder="AAPL,MSFT,GOOGL,NVDA,META,AMZN,TSLA"
                           data-i18n-placeholder="view.pairs.placeholder.symbols" required style="min-width:340px;text-transform:uppercase">
                </label>
                <label><span data-i18n="view.pairs.label.or_watchlist">or watchlist</span>
                    <select name="wl">
                        <option data-i18n="view.pairs.opt.ignore" value="">— ignore —</option>
                        ${lists.map(w => `<option value="${w.id}">${esc(w.name)}</option>`).join('')}
                    </select>
                </label>
                <label><span data-i18n="view.pairs.label.days">Days</span>
                    <input name="days" type="number" value="90" style="width:80px"></label>
                <button data-i18n="view.pairs.btn.run" class="primary" type="submit">Run</button>
            </form>
        </div>

        <div id="cmatrix"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.pairs.h2.pair_analyzer">Pair analyzer</h2>
            <form id="pf" class="inline-form">
                <input name="a" placeholder="A (KO)" data-i18n-placeholder="view.pairs.placeholder.a" required style="width:90px;text-transform:uppercase">
                <input name="b" placeholder="B (PEP)" data-i18n-placeholder="view.pairs.placeholder.b" required style="width:90px;text-transform:uppercase">
                <label><span data-i18n="view.pairs.label.days_180">Days</span>
                    <input name="days" type="number" value="180" style="width:80px"></label>
                <button data-i18n="view.pairs.btn.analyze" class="primary" type="submit">Analyze</button>
            </form>
            <div id="pair-out"></div>
        </div>
    `;
    mount.querySelector('#cf').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        let syms = fd.get('symbols').toUpperCase();
        const wid = fd.get('wl');
        if (wid) {
            try {
                const ws = await api.watchlistSymbols(wid);
                if (!viewIsCurrent(tok)) return;
                if (ws.length) syms = ws.join(',');
            } catch (_) {}
        }
        const days = Number(fd.get('days') || 90);
        const cm = mount.querySelector('#cmatrix');
        if (cm) cm.innerHTML = '<div class="boot" data-i18n="view.pairs.status.computing_matrix">computing matrix…</div>';
        try {
            const r = await api.correlationMatrix(syms, days);
            if (!viewIsCurrent(tok)) return;
            renderMatrix(r, mount);
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const cm2 = mount.querySelector('#cmatrix');
            if (cm2) cm2.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
    mount.querySelector('#pf').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const a = fd.get('a').toUpperCase();
        const b = fd.get('b').toUpperCase();
        const days = Number(fd.get('days') || 180);
        const el = mount.querySelector('#pair-out');
        if (el) el.innerHTML = '<div class="boot" data-i18n="common.status.analyzing">analyzing…</div>';
        try {
            const r = await api.pairAnalysis(a, b, days);
            if (!viewIsCurrent(tok)) return;
            const el2 = mount.querySelector('#pair-out');
            if (el2) renderPairOut(el2, a, b, r, mount);
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const el2 = mount.querySelector('#pair-out');
            if (el2) el2.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
}

function renderMatrix(r, mount) {
    const heat = (c) => {
        const intensity = Math.min(1, Math.abs(c));
        if (c >= 0) return `rgba(35, 209, 96, ${0.15 + intensity * 0.65})`;
        return `rgba(255, 56, 96, ${0.15 + intensity * 0.65})`;
    };
    const html = `<div class="chart-panel">
        <h2>${r.symbols.length}×${r.symbols.length} correlation · ${r.days}d · ${r.samples} samples</h2>
        <table class="corr-matrix">
            <thead><tr><th></th>${r.symbols.map(s => `<th>${esc(s)}</th>`).join('')}</tr></thead>
            <tbody>${r.symbols.map((row, i) => `<tr>
                <th>${esc(row)}</th>
                ${r.matrix[i].map((c, j) => `<td style="background:${heat(c)}">${c.toFixed(2)}</td>`).join('')}
            </tr>`).join('')}</tbody>
        </table>
    </div>`;
    const cm = mount.querySelector('#cmatrix');
    if (cm) cm.innerHTML = html;
}

function renderPairOut(el, a, b, r, mount) {
    const zCls = r.latest_zscore > 2 ? 'neg' : r.latest_zscore < -2 ? 'pos' : '';
    const reco = r.latest_zscore > 2  ? `SHORT ${esc(a)} / LONG ${esc(b)}`
              : r.latest_zscore < -2  ? `LONG ${esc(a)} / SHORT ${esc(b)}`
              : 'NEUTRAL — wait for |z| ≥ 2';
    el.innerHTML = `
        <div class="cards" style="margin-top:10px">
            <div class="card"><div class="label" data-i18n="view.pairs.card.correlation">Correlation</div><div class="value">${r.correlation.toFixed(3)}</div></div>
            <div class="card"><div class="label">β</div><div class="value">${r.beta.toFixed(3)}</div></div>
            <div class="card"><div class="label">α</div><div class="value">${r.alpha.toFixed(3)}</div></div>
            <div class="card"><div class="label" data-i18n="view.pairs.card.mean_spread">Mean spread</div><div class="value">${fmt(r.mean_spread)}</div></div>
            <div class="card"><div class="label">σ spread</div><div class="value">${fmt(r.stdev_spread)}</div></div>
            <div class="card"><div class="label" data-i18n="view.pairs.card.latest_spread">Latest spread</div><div class="value">${fmt(r.latest_spread)}</div></div>
            <div class="card"><div class="label" data-i18n="view.pairs.card.latest_zscore">Latest z-score</div>
                <div class="value ${zCls}">${r.latest_zscore.toFixed(2)}</div></div>
            <div class="card"><div class="label" data-i18n="view.pairs.card.samples">Samples</div><div class="value">${r.samples}</div></div>
        </div>
        <div class="chart-panel"><h2 data-i18n="view.pairs.h2.trade_signal">Trade signal</h2><p><strong>${reco}</strong></p></div>
        <div class="chart-panel"><h2 data-i18n="view.pairs.h2.spread_series">Spread series</h2><div id="sp-chart"></div></div>
        <div class="chart-panel"><h2 data-i18n="view.pairs.h2.z_score">Z-score</h2><div id="z-chart"></div></div>
    `;
    try { applyUiI18n(el); } catch (_) {}
    const labels = r.spread_series.map((_, i) => String(i));
    const spChart = mount.querySelector('#sp-chart');
    const zChart = mount.querySelector('#z-chart');
    if (spChart) barChart(spChart, labels, r.spread_series, { color: '#00e5ff' });
    if (zChart) barChart(zChart,  labels, r.zscore_series, { color: '#b86bff' });
}
