// Stock comparison — side-by-side fundamentals + RS chart for 2-4 symbols.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const COLORS = ['#00e5ff', '#ff7a1f', '#7af0a8', '#ff1f7a'];

function _m(key, kind, lower) {
    const o = { key, kind, get label() { return t(`view.compare.metric.${key}`); } };
    if (lower !== undefined) o.lower = lower;
    return o;
}
function _g(slug, metrics) {
    return { get group() { return t(`view.compare.group.${slug}`); }, metrics };
}

const ROWS = [
    _g('profile', [
        _m('name', 'text'),
        _m('sector', 'text'),
        _m('industry', 'text'),
        _m('price', 'money'),
        _m('market_cap', 'big'),
        _m('enterprise_value', 'big'),
        _m('beta', 'num'),
    ]),
    _g('valuation', [
        _m('trailing_pe',     'num',   true),
        _m('forward_pe',      'num',   true),
        _m('peg_ratio',       'num',   true),
        _m('price_to_book',   'num',   true),
        _m('price_to_sales',  'num',   true),
        _m('ev_to_ebitda',    'num',   true),
    ]),
    _g('profitability', [
        _m('profit_margin',    'pct', false),
        _m('operating_margin', 'pct', false),
        _m('return_on_equity', 'pct', false),
        _m('return_on_assets', 'pct', false),
    ]),
    _g('growth', [
        _m('revenue_growth',    'pct',   false),
        _m('earnings_growth',   'pct',   false),
        _m('revenue_per_share', 'money', false),
    ]),
    _g('balance_sheet', [
        _m('debt_to_equity',        'num',   true),
        _m('current_ratio',         'num',   false),
        _m('quick_ratio',           'num',   false),
        _m('free_cashflow',         'big',   false),
        _m('total_cash_per_share',  'money', false),
    ]),
    _g('income', [
        _m('dividend_yield', 'pct', false),
        _m('payout_ratio',   'pct', true),
    ]),
    _g('price_action', [
        _m('fifty_two_week_high', 'money'),
        _m('fifty_two_week_low',  'money'),
        _m('fifty_day_avg',       'money'),
        _m('two_hundred_day_avg', 'money'),
    ]),
    _g('returns', [
        _m('return_1d', 'pct', false),
        _m('return_1w', 'pct', false),
        _m('return_1m', 'pct', false),
        _m('return_3m', 'pct', false),
        _m('return_6m', 'pct', false),
        _m('return_1y', 'pct', false),
    ]),
];

export async function renderCompare(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.compare.h1.stock_comparison" class="view-title">// STOCK COMPARISON</h1>
        <p data-i18n="view.compare.hint.side_by_side_fundamentals_multi_horizon_returns_25" class="muted small">Side-by-side fundamentals + multi-horizon returns + 252-bar relative-strength
            overlay (each line rebased to 100 at the window start). Up to 4 symbols, comma-separated.
            Best metric in each row is highlighted in green; worst in red.</p>

        <form id="cmp-form" class="inline-form">
            <input name="symbols" data-shortcut="focus_search" placeholder="AAPL,MSFT,GOOGL,AMZN" value="AAPL,MSFT,GOOGL,AMZN"
                   required style="min-width:340px;text-transform:uppercase">
            <button data-i18n="view.compare.btn.compare" class="primary" type="submit">Compare</button>
        </form>

        <div id="cmp-out"><p data-i18n="view.compare.hint.enter_2_4_tickers_and_run" class="muted small">Enter 2-4 tickers and run.</p></div>
    `;
    mount.querySelector('#cmp-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const syms = e.target.symbols.value.trim().toUpperCase();
        const out = mount.querySelector('#cmp-out');
        if (!out) return;
        out.innerHTML = '<div class="boot" data-i18n="common.status.fetching">fetching…</div>';
        try {
            const r = await api.compare(syms);
            if (!viewIsCurrent(tok)) return;
            const out2 = mount.querySelector('#cmp-out');
            if (out2) renderReport(r, out2, mount);
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const out2 = mount.querySelector('#cmp-out');
            if (out2) out2.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
}

function renderReport(r, out, mount) {
    if (r.rows.length < 2) {
        out.innerHTML = `<p class="boot">${esc(t('view.compare.hint.too_few', { count: r.rows.length }))}</p>`;
        return;
    }
    out.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.compare.h2.relative_strength_252_bar_normalized_to_100">Relative strength — 252-bar normalized to 100</h2>
            <div id="cmp-rs"></div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.compare.h2.returns_chart">Returns by window (per symbol)</h2>
            <div id="cmp-returns-chart" style="width:100%;height:240px"></div>
        </div>
        <div class="chart-panel">
            <h2>${esc(t('view.compare.h2.fundamental', { count: r.rows.length }))}</h2>
            ${renderTable(r.rows)}
            <p class="muted small">${esc(t('view.compare.hint.fetched', { time: new Date(r.fetched_at).toLocaleTimeString(undefined, { hour12: false }) }))}</p>
        </div>
    `;
    renderRsSvg(r.rows, mount);
    renderReturnsChart(r.rows);
}

function renderReturnsChart(rows) {
    const el = document.getElementById('cmp-returns-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const windows = ['return_1d', 'return_1w', 'return_1m', 'return_3m', 'return_6m', 'return_1y'];
    const labels = ['1d', '1w', '1m', '3m', '6m', '1y'];
    const xs = labels.map((_, i) => i + 1);
    const symbolSeries = rows.map(r => windows.map(w => {
        const v = r[w];
        return (typeof v === 'number' && Number.isFinite(v)) ? v * 100 : null;
    }));
    const hasData = symbolSeries.some(s => s.some(v => v != null));
    if (!hasData) {
        el.innerHTML = `<div class="muted" data-i18n="view.compare.empty_chart">${esc(t('view.compare.empty_chart'))}</div>`;
        return;
    }
    const zero = xs.map(() => 0);
    const series = [
        { label: t('view.compare.chart.window') },
        ...rows.map((r, i) => ({
            label: r.symbol,
            stroke: COLORS[i] || '#aab',
            width: 1.6,
            points: { show: true, size: 8, fill: COLORS[i] || '#aab', stroke: COLORS[i] || '#aab' },
        })),
        { label: t('view.compare.chart.zero'),
          stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
    ];
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series,
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ...symbolSeries, zero], el);
}

function renderTable(rows) {
    let html = `<table class="trades" style="table-layout:fixed;">
        <thead><tr><th data-i18n="view.compare.th.metric">Metric</th>${rows.map((row, i) => `<th style="color:${COLORS[i]}">${esc(row.symbol)}</th>`).join('')}</tr></thead>
        <tbody>`;
    for (const grp of ROWS) {
        html += `<tr><td colspan="${rows.length + 1}" class="muted small" style="background:#1a1d2e;">${esc(grp.group)}</td></tr>`;
        for (const m of grp.metrics) {
            const values = rows.map(r => r[m.key]);
            const { best, worst } = bestWorst(values, m.lower);
            const lk = `view.compare.metric.${m.key}.label`;
            const lv = t(lk);
            const labelTr = (lv && lv !== lk) ? lv : m.label;
            html += `<tr><td class="small">${esc(labelTr)}</td>`;
            for (let i = 0; i < rows.length; i++) {
                const v = values[i];
                let cls = '';
                if (typeof v === 'number' && m.lower !== undefined) {
                    if (i === best && rows.length > 1) cls = 'pos';
                    else if (i === worst && rows.length > 1) cls = 'neg';
                }
                html += `<td class="${cls}">${formatCell(v, m.kind)}</td>`;
            }
            html += '</tr>';
        }
    }
    html += '</tbody></table>';
    return html;
}

function bestWorst(values, lowerIsBetter) {
    const nums = values.map((v, i) => [v, i]).filter(([v]) => typeof v === 'number' && Number.isFinite(v));
    if (nums.length < 2) return { best: -1, worst: -1 };
    nums.sort((a, b) => a[0] - b[0]);
    if (lowerIsBetter) return { best: nums[0][1], worst: nums[nums.length - 1][1] };
    return { best: nums[nums.length - 1][1], worst: nums[0][1] };
}

function formatCell(v, kind) {
    if (v == null || (typeof v === 'number' && !Number.isFinite(v))) return '—';
    if (kind === 'text') return esc(String(v));
    if (kind === 'pct')  return (v * 100).toFixed(2) + '%';
    if (kind === 'money') return '$' + fmt(v, v < 10 ? 4 : 2);
    if (kind === 'big')   return abbreviate(v);
    return fmt(v, 2);
}

function abbreviate(v) {
    const abs = Math.abs(v);
    if (abs >= 1e12) return '$' + (v / 1e12).toFixed(2) + 'T';
    if (abs >= 1e9)  return '$' + (v / 1e9).toFixed(2) + 'B';
    if (abs >= 1e6)  return '$' + (v / 1e6).toFixed(2) + 'M';
    return '$' + fmt(v);
}

function renderRsSvg(rows, mount) {
    const series = rows.filter(r => r.normalized_closes && r.normalized_closes.length > 0);
    const rsEl = mount.querySelector('#cmp-rs');
    if (!rsEl) return;
    if (!series.length) {
        rsEl.innerHTML = '<p data-i18n="view.compare.hint.no_cached_bars_for_any_symbol_populate_the_prices_" class="muted small">no cached bars for any symbol — populate the prices cache first.</p>';
        return;
    }
    const w = 1000, h = 320, pad = 50;
    const maxLen = Math.max(...series.map(s => s.normalized_closes.length));
    const allVals = series.flatMap(s => s.normalized_closes.map(p => p.value));
    const yMin = Math.min(...allVals);
    const yMax = Math.max(...allVals);
    const sx = (i, total) => pad + (i / Math.max(total - 1, 1)) * (w - 2 * pad);
    const sy = (y) => h - pad - (y - yMin) / Math.max(yMax - yMin, 1e-9) * (h - 2 * pad);
    const paths = series.map((s, idx) => {
        const pts = s.normalized_closes;
        const d = pts.map((p, i) => (i ? 'L' : 'M') + sx(i, maxLen) + ',' + sy(p.value)).join(' ');
        return `<path d="${d}" stroke="${COLORS[rows.indexOf(s)]}" stroke-width="2" fill="none"/>`;
    }).join('');
    const baseY = sy(100);
    const legend = rows.map((r, i) => {
        const last = r.normalized_closes?.[r.normalized_closes.length - 1]?.value;
        const lastTxt = last == null ? '—' : last.toFixed(1);
        return `<g><rect x="${pad + 12 + i * 160}" y="${pad - 18}" width="10" height="10" fill="${COLORS[i]}"/>
            <text x="${pad + 26 + i * 160}" y="${pad - 9}" fill="#cfd2e8" font-size="12">${esc(r.symbol)} (${lastTxt})</text></g>`;
    }).join('');
    rsEl.innerHTML = `
        <svg viewBox="0 0 ${w} ${h}" width="100%" style="display:block;">
            <line x1="${pad}" y1="${h - pad}" x2="${w - pad}" y2="${h - pad}" stroke="#444"/>
            <line x1="${pad}" y1="${pad}" x2="${pad}" y2="${h - pad}" stroke="#444"/>
            <line x1="${pad}" y1="${baseY}" x2="${w - pad}" y2="${baseY}" stroke="#666" stroke-dasharray="3,3"/>
            <text x="${pad + 4}" y="${baseY - 4}" fill="#9aa0c8" font-size="10">100 = window start</text>
            ${paths}
            ${legend}
            <text x="${w / 2}" y="${h - 10}" text-anchor="middle" fill="#9aa0c8" font-size="11">${esc(t('view.compare.svg.x_axis', { bars: maxLen }))}</text>
        </svg>
    `;
}
