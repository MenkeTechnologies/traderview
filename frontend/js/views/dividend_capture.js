// Dividend capture / arb scanner — for a symbol list with upcoming
// ex-dividends, rank by max(long_capture_edge, short_arb_edge).
// Long-capture = annual_yield × retention_pct − tx_friction.
// Short-arb = annual_yield × (1 − retention_pct) − borrow_proxy − tx.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const DEFAULT_SYMS = 'KO,PEP,JNJ,PFE,XOM,CVX,VZ,T,IBM,F,GM,MO,PM,ABBV,KMI,O,STAG,VICI,MAIN,AGNC,NLY,EPD,MMP,UPS,WBA,LMT,RTX,LOW,HD,MCD';

export async function renderDividendCapture(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.dividend_capture.title">// DIVIDEND CAPTURE / ARB SCANNER</span></h1>
        <p class="muted small" data-i18n-html="view.dividend_capture.intro">
            Two strategies in one ranking. <strong>Long capture</strong> buys before
            ex-div, sells after — empirically the price drop captures only ~60-80% of
            the dividend, leaving a ~30% retention edge per cycle. <strong>Short arb</strong>
            shorts the price-drop portion, pays borrow + tx friction. Borrow cost is
            proxied from short_pct_float (no paid SLB feed): &lt;5%→0.25%, 5-15%→2%,
            15-30%→8%, &gt;30%→25% annualised. Ranking by best variant — high-yield
            easy-to-borrow names favour short arb; high-yield HTB names favour long.
            Default universe is a hand-picked liquid-dividend-payer list.
        </p>
        <div class="chart-panel">
            <div class="dc-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <label style="flex:1 1 400px">
                    <span data-i18n="view.dividend_capture.label.symbols">symbols (comma-separated)</span>
                    <input type="text" id="dc-symbols" value="${DEFAULT_SYMS}" style="width:100%;text-transform:uppercase">
                </label>
                <button class="btn btn-sm primary" id="dc-scan" data-shortcut="r" data-i18n="view.dividend_capture.btn.scan">⚡ Scan</button>
                <span class="muted small" id="dc-meta"></span>
            </div>
            <table class="trades" id="dc-table">
                <thead><tr>
                    <th data-i18n="view.dividend_capture.th.rank">#</th>
                    <th data-i18n="view.dividend_capture.th.symbol">Symbol</th>
                    <th data-i18n="view.dividend_capture.th.best">Best Edge</th>
                    <th data-i18n="view.dividend_capture.th.ex_date">Ex-Date</th>
                    <th data-i18n="view.dividend_capture.th.days">Days</th>
                    <th data-i18n="view.dividend_capture.th.yield">Annual Yield</th>
                    <th data-i18n="view.dividend_capture.th.long">Long Edge</th>
                    <th data-i18n="view.dividend_capture.th.short">Short Edge</th>
                    <th data-i18n="view.dividend_capture.th.borrow">Borrow</th>
                    <th data-i18n="view.dividend_capture.th.short_pct">% Short</th>
                </tr></thead>
                <tbody><tr><td colspan="10" class="muted" data-i18n="view.dividend_capture.empty.hint">Enter symbols and click Scan — fetches Yahoo dividends + Finnhub short stats per name.</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#dc-scan').addEventListener('click', () => runScan(mount));
}

async function runScan(mount) {
    const tbody = mount.querySelector('#dc-table tbody');
    const meta = mount.querySelector('#dc-meta');
    const symbols = mount.querySelector('#dc-symbols').value.trim();
    if (!symbols) {
        tbody.innerHTML = `<tr><td colspan="10" class="muted">${esc(t('view.dividend_capture.empty.no_symbols'))}</td></tr>`;
        return;
    }
    tbody.innerHTML = `<tr><td colspan="10" class="muted">${esc(t('view.dividend_capture.status.scanning'))}</td></tr>`;
    if (meta) meta.textContent = '';
    try {
        const rows = await api(`/dividend-capture/scan?symbols=${encodeURIComponent(symbols)}`);
        if (!rows.length) {
            tbody.innerHTML = `<tr><td colspan="10" class="muted">${esc(t('view.dividend_capture.empty.no_rows'))}</td></tr>`;
            return;
        }
        if (meta) meta.textContent = t('view.dividend_capture.meta.summary').replace('{n}', rows.length);
        tbody.innerHTML = rows.map((r, i) => {
            const variant = r.long_capture_edge_pct >= r.short_arb_edge_pct ? 'LONG' : 'SHORT';
            const bestCls = r.best_edge_pct >= 2.0 ? 'pos' : r.best_edge_pct >= 0.5 ? '' : 'muted';
            return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td class="muted">${i + 1}</td>
                <td><strong style="color:var(--accent)">${esc(r.symbol)}</strong></td>
                <td class="${bestCls}"><strong>+${r.best_edge_pct.toFixed(2)}%</strong> <span class="muted small">(${variant})</span></td>
                <td>${esc(r.ex_dividend_date || '—')}</td>
                <td>${r.days_to_ex == null ? '—' : r.days_to_ex + 'd'}</td>
                <td>${r.annual_yield_pct.toFixed(2)}%</td>
                <td class="${r.long_capture_edge_pct >= 0 ? 'pos' : 'neg'}">${r.long_capture_edge_pct >= 0 ? '+' : ''}${r.long_capture_edge_pct.toFixed(2)}%</td>
                <td class="${r.short_arb_edge_pct >= 0 ? 'pos' : 'neg'}">${r.short_arb_edge_pct >= 0 ? '+' : ''}${r.short_arb_edge_pct.toFixed(2)}%</td>
                <td class="muted">${r.borrow_cost_proxy_pct.toFixed(2)}%</td>
                <td class="muted">${r.short_pct_float == null ? '—' : (r.short_pct_float * 100).toFixed(1) + '%'}</td>
            </tr>`;
        }).join('');
    } catch (e) {
        tbody.innerHTML = `<tr><td colspan="10" class="muted">${esc(String(e))}</td></tr>`;
    }
}
