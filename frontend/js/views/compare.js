// Stock comparison — side-by-side fundamentals + RS chart for 2-4 symbols.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const COLORS = ['#00e5ff', '#ff7a1f', '#7af0a8', '#ff1f7a'];

const ROWS = [
    { group: 'Profile', metrics: [
        { key: 'name', label: 'Name', kind: 'text' },
        { key: 'sector', label: 'Sector', kind: 'text' },
        { key: 'industry', label: 'Industry', kind: 'text' },
        { key: 'price', label: 'Price', kind: 'money' },
        { key: 'market_cap', label: 'Market cap', kind: 'big' },
        { key: 'enterprise_value', label: 'Enterprise value', kind: 'big' },
        { key: 'beta', label: 'Beta', kind: 'num' },
    ]},
    { group: 'Valuation (lower = cheaper)', metrics: [
        { key: 'trailing_pe',  label: 'Trailing P/E',  kind: 'num',   lower: true },
        { key: 'forward_pe',   label: 'Forward P/E',   kind: 'num',   lower: true },
        { key: 'peg_ratio',    label: 'PEG ratio',     kind: 'num',   lower: true },
        { key: 'price_to_book',label: 'Price / book',  kind: 'num',   lower: true },
        { key: 'price_to_sales', label: 'Price / sales', kind: 'num', lower: true },
        { key: 'ev_to_ebitda', label: 'EV / EBITDA',   kind: 'num',   lower: true },
    ]},
    { group: 'Profitability (higher = stronger)', metrics: [
        { key: 'profit_margin',    label: 'Profit margin',    kind: 'pct', lower: false },
        { key: 'operating_margin', label: 'Operating margin', kind: 'pct', lower: false },
        { key: 'return_on_equity', label: 'ROE',              kind: 'pct', lower: false },
        { key: 'return_on_assets', label: 'ROA',              kind: 'pct', lower: false },
    ]},
    { group: 'Growth (higher = faster)', metrics: [
        { key: 'revenue_growth',  label: 'Revenue growth (YoY)',  kind: 'pct', lower: false },
        { key: 'earnings_growth', label: 'Earnings growth (YoY)', kind: 'pct', lower: false },
        { key: 'revenue_per_share', label: 'Revenue / share', kind: 'money', lower: false },
    ]},
    { group: 'Balance sheet', metrics: [
        { key: 'debt_to_equity',  label: 'Debt / equity (%)', kind: 'num', lower: true },
        { key: 'current_ratio',   label: 'Current ratio',     kind: 'num', lower: false },
        { key: 'quick_ratio',     label: 'Quick ratio',       kind: 'num', lower: false },
        { key: 'free_cashflow',   label: 'Free cashflow',     kind: 'big', lower: false },
        { key: 'total_cash_per_share', label: 'Cash / share', kind: 'money', lower: false },
    ]},
    { group: 'Income to holders', metrics: [
        { key: 'dividend_yield', label: 'Dividend yield', kind: 'pct', lower: false },
        { key: 'payout_ratio',   label: 'Payout ratio',   kind: 'pct', lower: true  },
    ]},
    { group: 'Price action', metrics: [
        { key: 'fifty_two_week_high', label: '52-wk high', kind: 'money' },
        { key: 'fifty_two_week_low',  label: '52-wk low',  kind: 'money' },
        { key: 'fifty_day_avg',       label: '50-d avg',   kind: 'money' },
        { key: 'two_hundred_day_avg', label: '200-d avg',  kind: 'money' },
    ]},
    { group: 'Returns', metrics: [
        { key: 'return_1d', label: '1-day',  kind: 'pct', lower: false },
        { key: 'return_1w', label: '1-week', kind: 'pct', lower: false },
        { key: 'return_1m', label: '1-month', kind: 'pct', lower: false },
        { key: 'return_3m', label: '3-month', kind: 'pct', lower: false },
        { key: 'return_6m', label: '6-month', kind: 'pct', lower: false },
        { key: 'return_1y', label: '1-year',  kind: 'pct', lower: false },
    ]},
];

export async function renderCompare(mount) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.compare.h1.stock_comparison" class="view-title">// STOCK COMPARISON</h1>
        <p data-i18n="view.compare.hint.side_by_side_fundamentals_multi_horizon_returns_25" class="muted small">Side-by-side fundamentals + multi-horizon returns + 252-bar relative-strength
            overlay (each line rebased to 100 at the window start). Up to 4 symbols, comma-separated.
            Best metric in each row is highlighted in green; worst in red.</p>

        <form id="cmp-form" class="inline-form">
            <input name="symbols" placeholder="AAPL,MSFT,GOOGL,AMZN" value="AAPL,MSFT,GOOGL,AMZN"
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
        out.innerHTML = '<div class="boot">fetching…</div>';
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
            <h2>${esc(t('view.compare.h2.fundamental', { count: r.rows.length }))}</h2>
            ${renderTable(r.rows)}
            <p class="muted small">${esc(t('view.compare.hint.fetched', { time: new Date(r.fetched_at).toLocaleTimeString(undefined, { hour12: false }) }))}</p>
        </div>
    `;
    renderRsSvg(r.rows, mount);
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
            html += `<tr><td class="small">${esc(m.label)}</td>`;
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
            <text x="${w / 2}" y="${h - 10}" text-anchor="middle" fill="#9aa0c8" font-size="11">bar index (${maxLen} bars ≈ 1 year)</text>
        </svg>
    `;
}
