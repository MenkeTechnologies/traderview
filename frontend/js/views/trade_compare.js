// Trade comparison — pick 2-4 closed trades, diff stats + overlay
// normalized P/L curves on a single SVG.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const COLORS = ['#00e5ff', '#ff7a1f', '#7af0a8', '#ff1f7a'];

let selectedIds = [];

export async function renderTradeCompare(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) { mount.innerHTML = `<p data-i18n="view.trade_compare.hint.no_account_selected" class="boot">No account selected.</p>`; return; }
    mount.innerHTML = `
        <h1 class="view-title">// TRADE COMPARISON — ${esc(acct.broker)} · ${esc(acct.name)}</h1>
        <p class="muted small" data-i18n="view.trade_compare.hint.intro">Pick 2-4 closed trades from the picker. The right pane shows side-by-side stats and a normalized P/L overlay where the x-axis is t ∈ [0..1] from open to close (so a 5-min scalp and a 6-month swing can be compared on the same chart) and the y-axis is % return vs entry. Auto-picks bar interval per trade duration (1m / 5m / 1h / 1d) so each curve has 50-200 points.</p>

        <div style="display:grid;grid-template-columns:340px 1fr;gap:10px;">
            <div class="chart-panel">
                <h2 data-i18n="view.trade_compare.h2.picker">Picker</h2>
                <input id="tc-search" placeholder="filter by symbol…" data-i18n-placeholder="view.trade_compare.placeholder.filter" style="width:100%;margin-bottom:8px;">
                <div id="tc-picker" style="max-height:500px;overflow:auto;"><div class="tv-spinner-wrap"><div class="tv-spinner"></div><div class="tv-spinner-text" data-i18n="common.loading">loading…</div></div></div>
            </div>
            <div id="tc-result"><p data-i18n="view.trade_compare.hint.select_2_4_trades_from_the_picker" class="muted small">Select 2-4 trades from the picker.</p></div>
        </div>
    `;
    try {
        const trades = await api.trades(acct.id, { status: 'closed', limit: 300 });
        if (!viewIsCurrent(tok)) return;
        renderPicker(trades, mount, tok);
        const searchEl = mount.querySelector('#tc-search');
        if (searchEl) searchEl.addEventListener('input', (e) => {
            const q = e.target.value.toLowerCase().trim();
            const filtered = q ? trades.filter(t => t.symbol.toLowerCase().includes(q)) : trades;
            renderPicker(filtered, mount, tok);
        });
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const picker = mount.querySelector('#tc-picker');
        if (picker) picker.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function renderPicker(trades, mount, tok) {
    const el = mount.querySelector('#tc-picker');
    if (!el) return;
    if (!trades.length) { el.innerHTML = '<p data-i18n="view.trade_compare.hint.no_closed_trades" class="muted small">no closed trades</p>'; return; }
    el.innerHTML = trades.map(t => {
        const sel = selectedIds.includes(t.id);
        const cls = (t.net_pnl ?? 0) >= 0 ? 'pos' : 'neg';
        return `<label data-context-scope="trade-row" data-id="${esc(t.id)}"
                    style="display:flex;gap:6px;align-items:center;padding:4px 2px;border-bottom:1px solid var(--border);cursor:pointer;${sel ? 'background:rgba(0,229,255,0.10);' : ''}">
            <input type="checkbox" class="tc-pick" data-id="${t.id}" ${sel ? 'checked' : ''}>
            <span style="flex:1;font-size:11px;">
                <strong>${esc(t.symbol)}</strong>
                <span class="muted">${esc(t.side)} · ${new Date(t.opened_at).toLocaleDateString()}</span>
            </span>
            <span class="${cls}" style="font-size:11px;">${
                t.net_pnl != null ? '$' + fmt(Number(t.net_pnl), 0) : '—'
            }</span>
        </label>`;
    }).join('');
    el.querySelectorAll('.tc-pick').forEach(cb => {
        cb.addEventListener('change', async () => {
            const id = cb.dataset.id;
            if (cb.checked) {
                if (selectedIds.length >= 4) {
                    showToast(t('view.trade_compare.alert.max_four'), { level: 'error' });
                    cb.checked = false; return;
                }
                selectedIds.push(id);
            } else {
                selectedIds = selectedIds.filter(x => x !== id);
            }
            // Re-render picker for highlight + re-run compare if >=2 selected.
            renderPicker(trades, mount, tok);
            if (selectedIds.length >= 2) await runCompare(mount, tok);
            else {
                const r = mount.querySelector('#tc-result');
                if (r) r.innerHTML = '<p data-i18n="view.trade_compare.hint.select_at_least_2_trades" class="muted small">Select at least 2 trades.</p>';
            }
        });
    });
}

async function runCompare(mount, tok) {
    const out = mount.querySelector('#tc-result');
    if (!out) return;
    out.innerHTML = '<div class="boot" data-i18n="common.status.comparing">comparing…</div>';
    try {
        const r = await api.tradeCompare(selectedIds);
        if (!viewIsCurrent(tok)) return;
        const outNow = mount.querySelector('#tc-result');
        if (outNow) render(r, outNow);
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const outNow = mount.querySelector('#tc-result');
        if (outNow) outNow.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}

function render(r, out) {
    if (r.rows.length < 2) { out.innerHTML = `<p class="boot">${esc(t('view.trade_compare.too_few', { count: r.rows.length }))}</p>`; return; }
    out.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.trade_compare.h2.normalized_p_l_overlay_return_vs_entry">Normalized P/L overlay (% return vs entry)</h2>
            ${overlaySvg(r.rows)}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.trade_compare.h2.mfe_mae_chart">MFE vs MAE per trade</h2>
            <div id="tc-chart" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.trade_compare.h2.side_by_side_stats">Side-by-side stats</h2>
            ${statsTable(r.rows)}
        </div>
    `;
    renderMfeMaeChart(r.rows);
}

function renderMfeMaeChart(rows) {
    const el = document.getElementById('tc-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (rows || []).filter(r => Number.isFinite(Number(r.mfe)) || Number.isFinite(Number(r.mae)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.trade_compare.empty_chart">${esc(t('view.trade_compare.empty_chart'))}</div>`;
        return;
    }
    const labels = valid.map(r => r.symbol);
    const mfe = valid.map(r => Number.isFinite(Number(r.mfe)) ? Number(r.mfe) : null);
    const mae = valid.map(r => Number.isFinite(Number(r.mae)) ? Number(r.mae) : null);
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.trade_compare.chart.trade_idx') },
            { label: t('view.trade_compare.chart.mfe'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.trade_compare.chart.mae'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.trade_compare.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, mfe, mae, zero], el);
}

function overlaySvg(rows) {
    const W = 1000, H = 320, padL = 56, padR = 10, padT = 14, padB = 28;
    const innerW = W - padL - padR, innerH = H - padT - padB;
    const allY = rows.flatMap(r => r.curve.map(p => p.pnl_pct));
    if (!allY.length) return '<p data-i18n="view.trade_compare.hint.no_curve_data" class="muted small">no curve data</p>';
    const yMin = Math.min(0, ...allY) * 1.05;
    const yMax = Math.max(0, ...allY) * 1.05;
    const sx = (t) => padL + t * innerW;
    const sy = (y) => padT + (1 - (y - yMin) / Math.max(yMax - yMin, 1e-9)) * innerH;
    const lines = rows.map((r, i) => {
        if (!r.curve.length) return '';
        const color = COLORS[i % COLORS.length];
        const d = r.curve.map((p, j) => (j ? 'L' : 'M') + sx(p.t).toFixed(1) + ',' + sy(p.pnl_pct).toFixed(1)).join(' ');
        const finalPt = r.curve[r.curve.length - 1];
        return `<path d="${d}" stroke="${color}" stroke-width="1.8" fill="none" opacity="0.9"/>
                <circle cx="${sx(finalPt.t)}" cy="${sy(finalPt.pnl_pct)}" r="3" fill="${color}"/>`;
    }).join('');
    const zeroY = sy(0);
    const yLabels = [0, 0.25, 0.5, 0.75, 1].map(t => {
        const y = padT + t * innerH;
        const v = yMax - t * (yMax - yMin);
        return `<text x="${padL - 4}" y="${y + 3}" text-anchor="end" fill="#9aa0c8" font-size="10">${v.toFixed(2)}%</text>`;
    }).join('');
    const legend = rows.map((r, i) =>
        `<g transform="translate(${padL + 8 + i * 200}, ${padT - 2})">
            <line x1="0" y1="0" x2="14" y2="0" stroke="${COLORS[i % COLORS.length]}" stroke-width="2"/>
            <text x="18" y="3" fill="#cfd2e8" font-size="10">${esc(r.symbol)} · ${esc(r.side)} · ${r.net_pnl != null ? '$' + fmt(r.net_pnl, 0) : ''}</text>
         </g>`).join('');
    return `<svg viewBox="0 0 ${W} ${H}" width="100%" style="display:block;">
        <rect x="${padL}" y="${padT}" width="${innerW}" height="${innerH}" fill="#0d0d22" stroke="#222"/>
        <line x1="${padL}" y1="${zeroY}" x2="${padL + innerW}" y2="${zeroY}" stroke="#666" stroke-dasharray="3,3"/>
        ${lines}
        ${yLabels}
        ${legend}
        <text x="${padL + innerW / 2}" y="${H - 8}" text-anchor="middle" fill="#9aa0c8" font-size="11">${esc(t('view.trade_compare.svg.x_axis'))}</text>
    </svg>`;
}

function statsTable(rows) {
    const rowsHtml = (cells) => `<tr>${cells.map(c => `<td>${c}</td>`).join('')}</tr>`;
    const num = (v, decs = 2) => v == null ? '—' : Number(v).toFixed(decs);
    const usd = (v) => v == null ? '—' : '$' + fmt(v);
    const cls = (v) => v == null ? '' : v >= 0 ? 'pos' : 'neg';
    const td = (val, c = '') => `<td class="${c}">${val}</td>`;
    const headerRow = rows.map((r, i) =>
        `<th style="color:${COLORS[i % COLORS.length]}">${esc(r.symbol)} · ${esc(r.side)}</th>`).join('');
    const metricRow = (label, fn) =>
        `<tr><td>${esc(label)}</td>${rows.map(fn).join('')}</tr>`;
    const cmpRow = (label, fn) =>
        `<tr><td>${esc(label)}</td>${rows.map((r) => {
            const { v, c } = fn(r);
            return td(v, c);
        }).join('')}</tr>`;
    return `<table class="trades">
        <thead><tr><th data-i18n="view.trade_compare.th.metric">Metric</th>${headerRow}</tr></thead>
        <tbody>
            ${metricRow(t('view.trade_compare.row.opened'),   r => td(new Date(r.opened_at).toLocaleString()))}
            ${metricRow(t('view.trade_compare.row.closed'),   r => td(r.closed_at ? new Date(r.closed_at).toLocaleString() : '—'))}
            ${metricRow(t('view.trade_compare.row.hold_h'),   r => td((r.hold_seconds / 3600).toFixed(2)))}
            ${metricRow(t('view.trade_compare.row.bar_interval'), r => td(r.bar_interval))}
            ${metricRow(t('view.trade_compare.row.qty'),      r => td(fmt(r.qty, 0)))}
            ${metricRow(t('view.trade_compare.row.entry'),    r => td(num(r.entry_avg)))}
            ${metricRow(t('view.trade_compare.row.exit'),     r => td(num(r.exit_avg)))}
            ${metricRow(t('view.trade_compare.row.stop'),     r => td(num(r.stop_loss)))}
            ${metricRow(t('view.trade_compare.row.target'),   r => td(num(r.initial_target)))}
            ${cmpRow(t('view.trade_compare.row.gross_pnl'),   r => ({ v: usd(r.gross_pnl), c: cls(r.gross_pnl) }))}
            ${metricRow(t('view.trade_compare.row.fees'),     r => td(usd(r.fees)))}
            ${cmpRow(t('view.trade_compare.row.net_pnl'),     r => ({ v: usd(r.net_pnl), c: cls(r.net_pnl) }))}
            ${cmpRow(t('view.trade_compare.row.mfe'),         r => ({ v: usd(r.mfe), c: 'pos' }))}
            ${cmpRow(t('view.trade_compare.row.mae'),         r => ({ v: usd(r.mae), c: 'neg' }))}
            ${metricRow(t('view.trade_compare.row.risk_d'),   r => td(usd(r.risk_amount)))}
            ${cmpRow(t('view.trade_compare.row.r_multiple'),  r => ({
                v: r.r_multiple == null ? '—' : (r.r_multiple >= 0 ? '+' : '') + r.r_multiple.toFixed(2) + 'R',
                c: cls(r.r_multiple),
            }))}
        </tbody>
    </table>`;
}
