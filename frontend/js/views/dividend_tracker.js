// Dividend total-return tracker. For income-oriented investors the
// wealth measure is total return (price + reinvested dividends), not
// price return. Surfaces per-position price return, total return (DRIP),
// trailing-12m dividend, yield-on-cost, current yield, forward 12m
// income estimate; plus portfolio totals.

import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { t } from '../i18n.js';

export async function renderDividendTracker(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.dividend_tracker.title">// DIVIDEND TOTAL-RETURN TRACKER</span></h1>
        <p class="muted small" data-i18n-html="view.dividend_tracker.intro">
            Pulls Yahoo's per-symbol dividend history for every long paper position,
            applies dividend reinvestment (DRIP) since opened_at, and surfaces:
            <strong>total return</strong> (price + reinvested dividends),
            <strong>yield-on-cost</strong> (annual div × shares / original cost basis —
            climbs as dividends grow), <strong>current yield</strong>, and
            <strong>forward 12-month income</strong>. For a 3% yielder bought 20 years
            ago whose dividend has grown 4×, yield-on-cost can be 12% while current
            yield is still 3%.
        </p>
        <div class="chart-panel">
            <div style="display:flex;gap:12px;align-items:center;margin-bottom:8px">
                <button class="btn btn-sm primary" id="dt-refresh" data-shortcut="r" data-i18n="view.dividend_tracker.btn.refresh">⚡ Refresh</button>
                <span class="muted small" id="dt-meta"></span>
            </div>
            <div id="dt-summary"></div>
            <h2 style="margin-top:1rem" data-i18n="view.dividend_tracker.h2.positions">Per-position</h2>
            <table class="trades" id="dt-positions">
                <thead><tr>
                    <th data-i18n="view.dividend_tracker.th.symbol">Symbol</th>
                    <th data-i18n="view.dividend_tracker.th.shares">Shares</th>
                    <th data-i18n="view.dividend_tracker.th.cost">Cost/sh</th>
                    <th data-i18n="view.dividend_tracker.th.price">Price</th>
                    <th data-i18n="view.dividend_tracker.th.price_return">Price RTN%</th>
                    <th data-i18n="view.dividend_tracker.th.total_return">Total RTN%</th>
                    <th data-i18n="view.dividend_tracker.th.ttm_div">TTM Div/sh</th>
                    <th data-i18n="view.dividend_tracker.th.current_yield">Cur Yield</th>
                    <th data-i18n="view.dividend_tracker.th.yoc">YoC</th>
                    <th data-i18n="view.dividend_tracker.th.fwd_income">Fwd 12m Income</th>
                </tr></thead>
                <tbody><tr><td colspan="10" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
        </div>
    `;
    mount.querySelector('#dt-refresh').addEventListener('click', () => fetchAndRender(mount));
    await fetchAndRender(mount);
}

async function fetchAndRender(mount) {
    const summary = mount.querySelector('#dt-summary');
    const tbody = mount.querySelector('#dt-positions tbody');
    const meta = mount.querySelector('#dt-meta');
    summary.innerHTML = `<p class="muted">${esc(t('view.dividend_tracker.status.fetching'))}</p>`;
    if (meta) meta.textContent = '';
    try {
        const r = await api('/dividend-tracker/report');
        if (!r || !r.positions || !r.positions.length) {
            summary.innerHTML = `<p class="muted">${esc(t('view.dividend_tracker.empty.no_positions'))}</p>`;
            tbody.innerHTML = '';
            return;
        }
        const yocCls = r.weighted_yield_on_cost_pct >= 4 ? 'pos'
            : r.weighted_yield_on_cost_pct >= 2 ? '' : 'muted';
        summary.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px">
                <div><div class="muted small">${esc(t('view.dividend_tracker.field.fwd_income'))}</div>
                    <strong class="pos">$${r.total_forward_12m_income_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.dividend_tracker.field.weighted_yoc'))}</div>
                    <strong class="${yocCls}">${r.weighted_yield_on_cost_pct.toFixed(2)}%</strong></div>
                <div><div class="muted small">${esc(t('view.dividend_tracker.field.weighted_cy'))}</div>
                    <strong>${r.weighted_current_yield_pct.toFixed(2)}%</strong></div>
                <div><div class="muted small">${esc(t('view.dividend_tracker.field.positions'))}</div>
                    <strong>${r.positions.length}</strong></div>
            </div>
        `;
        tbody.innerHTML = r.positions.map(p => {
            const prCls = p.price_return_pct >= 0 ? 'pos' : 'neg';
            const trCls = p.total_return_pct >= 0 ? 'pos' : 'neg';
            const yocCls = p.yield_on_cost_pct >= p.current_yield_pct + 1 ? 'pos' : '';
            return `<tr>
                <td><strong>${esc(p.symbol)}</strong></td>
                <td>${p.qty.toFixed(2)}</td>
                <td>$${p.cost_basis_per_share.toFixed(2)}</td>
                <td>$${p.current_price.toFixed(2)}</td>
                <td class="${prCls}">${p.price_return_pct >= 0 ? '+' : ''}${p.price_return_pct.toFixed(2)}%</td>
                <td class="${trCls}"><strong>${p.total_return_pct >= 0 ? '+' : ''}${p.total_return_pct.toFixed(2)}%</strong></td>
                <td>$${p.trailing_12m_div_per_share.toFixed(2)}</td>
                <td>${p.current_yield_pct.toFixed(2)}%</td>
                <td class="${yocCls}">${p.yield_on_cost_pct.toFixed(2)}%</td>
                <td class="pos">$${p.forward_12m_income_usd.toFixed(2)}</td>
            </tr>`;
        }).join('');
        if (meta) meta.textContent = t('view.dividend_tracker.meta.generated').replace('{t}', fmtDateTime(r.generated_at));
    } catch (e) {
        summary.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
