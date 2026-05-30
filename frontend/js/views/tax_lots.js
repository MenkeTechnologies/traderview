// Tax-lot tracker — FIFO/LIFO matching, ST/LT classification, wash-sale flag.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { applyUiI18n } from '../i18n.js';

export async function renderTaxLots(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) {
        mount.innerHTML = `<p data-i18n="view.tax_lots.hint.no_account_selected_create_one_on_the_accounts_tab" class="boot">No account selected. Create one on the Accounts tab first.</p>`;
        return;
    }
    const now = new Date();
    const years = [];
    for (let y = now.getFullYear(); y >= now.getFullYear() - 5; y--) years.push(y);

    mount.innerHTML = `
        <h1 class="view-title">// TAX LOTS — ${esc(acct.broker)} · ${esc(acct.name)}</h1>
        <p class="muted small" data-i18n="view.tax_lots.hint.intro">Lot-by-lot accounting derived from your executions. Methods supported: FIFO (default — IRS default for equities), LIFO. Wash-sale flag: per IRC §1091, any loss realized within ±30 days of a buy of the same symbol is disallowed for tax purposes (the disallowed amount is added back to net gain). Long-term = held ≥ 365 days. Short/cover events are passed through but not lot-matched in v1.</p>

        <form id="tx-form" class="inline-form">
            <label><span data-i18n="view.tax_lots.label.year">Year</span>
                <select name="year">
                    ${years.map(y => `<option value="${y}" ${y === now.getFullYear() ? 'selected' : ''}>${y}</option>`).join('')}
                </select>
            </label>
            <label><span data-i18n="view.tax_lots.label.method">Method</span>
                <select name="method">
                    <option data-i18n="view.tax_lots.opt.fifo" value="fifo" selected>FIFO</option>
                    <option data-i18n="view.tax_lots.opt.lifo" value="lifo">LIFO</option>
                </select>
            </label>
            <button data-i18n="view.tax_lots.btn.build_report" class="primary" type="submit">Build report</button>
        </form>

        <div id="tx-out"><p data-i18n="view.tax_lots.hint.pick_a_year_method_and_run" class="muted small">Pick a year + method and run.</p></div>
    `;
    mount.querySelector('#tx-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const out = mount.querySelector('#tx-out');
        if (!out) return;
        out.innerHTML = '<div class="boot" data-i18n="common.status.running">running…</div>';
        try {
            const r = await api.taxLots(state.accountId, fd.get('year'), fd.get('method'));
            if (!viewIsCurrent(tok)) return;
            const outNow = mount.querySelector('#tx-out');
            if (outNow) renderReport(r, outNow);
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const outNow = mount.querySelector('#tx-out');
            if (outNow) outNow.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
}

function renderReport(r, out) {
    out.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label"><span data-i18n="view.tax_lots.card.net_total">Net total</span> (year ${r.year})</div>
                <div class="value ${r.net_total >= 0 ? 'pos' : 'neg'}">$${fmt(r.net_total)}</div></div>
            <div class="card"><div class="label" data-i18n="view.tax_lots.card.short_term_net">Short-term net</div>
                <div class="value ${r.net_short_term >= 0 ? 'pos' : 'neg'}">$${fmt(r.net_short_term)}</div>
                <div class="small muted">${r.short_term_gain >= 0 ? '+' : ''}$${fmt(r.short_term_gain)} / -$${fmt(r.short_term_loss)}</div></div>
            <div class="card"><div class="label" data-i18n="view.tax_lots.card.long_term_net">Long-term net</div>
                <div class="value ${r.net_long_term >= 0 ? 'pos' : 'neg'}">$${fmt(r.net_long_term)}</div>
                <div class="small muted">${r.long_term_gain >= 0 ? '+' : ''}$${fmt(r.long_term_gain)} / -$${fmt(r.long_term_loss)}</div></div>
            <div class="card"><div class="label" data-i18n="view.tax_lots.card.wash_sale_disallowed">Wash-sale disallowed</div>
                <div class="value ${r.wash_sale_total > 0 ? 'warn' : ''}">$${fmt(r.wash_sale_total)}</div>
                <div class="small muted">${esc(t('view.tax_lots.card.added_back_to_net'))}</div></div>
            <div class="card"><div class="label" data-i18n="view.tax_lots.card.realized_events">Realized events</div>
                <div class="value">${r.realized_count}</div>
                <div class="small muted">${esc(t('view.tax_lots.card.proceeds_basis', { proceeds: fmt(r.total_proceeds), basis: fmt(r.total_basis) }))}</div></div>
            <div class="card"><div class="label" data-i18n="view.tax_lots.card.open_lots">Open lots</div>
                <div class="value">${r.open_lot_count}</div>
                <div class="small muted">${esc(t('view.tax_lots.card.basis_only', { basis: fmt(r.open_basis) }))}</div></div>
        </div>

        ${r.skipped_short_events > 0 ? `
            <p class="muted small">${esc(t('view.tax_lots.hint.skipped_short', { n: r.skipped_short_events }))}</p>` : ''}

        <div class="chart-panel">
            <h2>${esc(t('view.tax_lots.h2.realized', { count: r.realized.length, method: r.method.toUpperCase() }))}</h2>
            ${r.realized.length === 0
                ? '<p data-i18n="view.tax_lots.hint.no_closed_lots_in_this_year" class="muted small">No closed lots in this year.</p>'
                : `<table class="trades">
                    <thead><tr>
                        <th data-i18n="view.tax_lots.th.symbol">Symbol</th><th data-i18n="view.tax_lots.th.acquired">Acquired</th><th data-i18n="view.tax_lots.th.disposed">Disposed</th><th data-i18n="view.tax_lots.th.days">Days</th><th data-i18n="view.tax_lots.th.term">Term</th>
                        <th data-i18n="view.tax_lots.th.qty">Qty</th><th data-i18n="view.tax_lots.th.basis">Basis</th><th data-i18n="view.tax_lots.th.proceeds">Proceeds</th><th data-i18n="view.tax_lots.th.gain_loss">Gain/Loss</th><th data-i18n="view.tax_lots.th.wash">Wash</th>
                    </tr></thead>
                    <tbody>
                        ${r.realized.map(rv => `<tr>
                            <td>${esc(rv.symbol)}</td>
                            <td class="small">${rv.acquired_at.slice(0, 10)}</td>
                            <td class="small">${rv.disposed_at.slice(0, 10)}</td>
                            <td>${rv.holding_days}</td>
                            <td class="${rv.long_term ? 'pos' : ''}">${rv.long_term ? 'LT' : 'ST'}</td>
                            <td>${fmt(rv.qty)}</td>
                            <td>$${fmt(rv.cost_basis)}</td>
                            <td>$${fmt(rv.proceeds)}</td>
                            <td class="${rv.gain_loss >= 0 ? 'pos' : 'neg'}">$${fmt(rv.gain_loss)}</td>
                            <td class="${rv.wash_sale_disallowed > 0 ? 'warn' : 'muted'}">${rv.wash_sale_disallowed > 0 ? '$' + fmt(rv.wash_sale_disallowed) : '—'}</td>
                        </tr>`).join('')}
                    </tbody>
                </table>`}
        </div>

        <div class="chart-panel">
            <h2>${esc(t('view.tax_lots.h2.open_lots', { count: r.open_lots.length }))}</h2>
            ${r.open_lots.length === 0
                ? '<p data-i18n="view.tax_lots.hint.no_open_lots" class="muted small">No open lots.</p>'
                : `<table class="trades">
                    <thead><tr>
                        <th data-i18n="view.tax_lots.th.symbol_2">Symbol</th><th data-i18n="view.tax_lots.th.acquired_2">Acquired</th><th data-i18n="view.tax_lots.th.held">Held</th><th data-i18n="view.tax_lots.th.term_2">Term</th>
                        <th data-i18n="view.tax_lots.th.qty_2">Qty</th><th data-i18n="view.tax_lots.th.cost_sh">Cost/sh</th><th data-i18n="view.tax_lots.th.basis_2">Basis</th>
                    </tr></thead>
                    <tbody>
                        ${r.open_lots.map(l => `<tr>
                            <td>${esc(l.symbol)}</td>
                            <td class="small">${l.acquired_at.slice(0, 10)}</td>
                            <td>${l.holding_days}d</td>
                            <td class="${l.long_term ? 'pos' : ''}">${l.long_term ? 'LT' : 'ST'}</td>
                            <td>${fmt(l.qty_remaining)}</td>
                            <td>$${fmt(l.cost_per_share)}</td>
                            <td>$${fmt(l.cost_basis)}</td>
                        </tr>`).join('')}
                    </tbody>
                </table>`}
            <p data-i18n="view.tax_lots.hint.holding_period_clock_on_open_lots_is_current_as_of" class="muted small">Holding-period clock on open lots is current as of now — closing
                these positions today would realize at their displayed term.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.tax_lots.h2.realized_chart">Realized gain/loss per event</h2>
            <div id="tl-chart" style="width:100%;height:240px"></div>
        </div>
    `;
    try { applyUiI18n(out); } catch (_) {}
    renderRealizedChart(r.realized);
}

function renderRealizedChart(realized) {
    const el = document.getElementById('tl-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const events = Array.isArray(realized) ? realized.filter(e => Number.isFinite(e.gain_loss)) : [];
    if (events.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.tax_lots.empty_chart">${esc(t('view.tax_lots.empty_chart'))}</div>`;
        return;
    }
    const gain = events.map(e => e.gain_loss);
    const xs = gain.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.tax_lots.chart.event_idx') },
            { label: t('view.tax_lots.chart.gain_loss'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.tax_lots.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, gain, zero], el);
}
