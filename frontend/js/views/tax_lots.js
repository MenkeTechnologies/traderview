// Tax-lot tracker — FIFO/LIFO matching, ST/LT classification, wash-sale flag.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderTaxLots(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    if (!acct) {
        mount.innerHTML = `<p class="boot">No account selected. Create one on the Accounts tab first.</p>`;
        return;
    }
    const now = new Date();
    const years = [];
    for (let y = now.getFullYear(); y >= now.getFullYear() - 5; y--) years.push(y);

    mount.innerHTML = `
        <h1 class="view-title">// TAX LOTS — ${esc(acct.broker)} · ${esc(acct.name)}</h1>
        <p class="muted small">Lot-by-lot accounting derived from your executions. Methods supported:
            <strong>FIFO</strong> (default — IRS default for equities), <strong>LIFO</strong>.
            Wash-sale flag: per IRC §1091, any loss realized within ±30 days of a buy of the
            same symbol is disallowed for tax purposes (the disallowed amount is added back
            to net gain). Long-term = held ≥ 365 days. Short/cover events are passed through
            but not lot-matched in v1.</p>

        <form id="tx-form" class="inline-form">
            <label>Year
                <select name="year">
                    ${years.map(y => `<option value="${y}" ${y === now.getFullYear() ? 'selected' : ''}>${y}</option>`).join('')}
                </select>
            </label>
            <label>Method
                <select name="method">
                    <option value="fifo" selected>FIFO</option>
                    <option value="lifo">LIFO</option>
                </select>
            </label>
            <button class="primary" type="submit">Build report</button>
        </form>

        <div id="tx-out"><p class="muted small">Pick a year + method and run.</p></div>
    `;
    mount.querySelector('#tx-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const out = mount.querySelector('#tx-out');
        if (!out) return;
        out.innerHTML = '<div class="boot">running…</div>';
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
            <div class="card"><div class="label">Net total (year ${r.year})</div>
                <div class="value ${r.net_total >= 0 ? 'pos' : 'neg'}">$${fmt(r.net_total)}</div></div>
            <div class="card"><div class="label">Short-term net</div>
                <div class="value ${r.net_short_term >= 0 ? 'pos' : 'neg'}">$${fmt(r.net_short_term)}</div>
                <div class="small muted">${r.short_term_gain >= 0 ? '+' : ''}$${fmt(r.short_term_gain)} / -$${fmt(r.short_term_loss)}</div></div>
            <div class="card"><div class="label">Long-term net</div>
                <div class="value ${r.net_long_term >= 0 ? 'pos' : 'neg'}">$${fmt(r.net_long_term)}</div>
                <div class="small muted">${r.long_term_gain >= 0 ? '+' : ''}$${fmt(r.long_term_gain)} / -$${fmt(r.long_term_loss)}</div></div>
            <div class="card"><div class="label">Wash-sale disallowed</div>
                <div class="value ${r.wash_sale_total > 0 ? 'warn' : ''}">$${fmt(r.wash_sale_total)}</div>
                <div class="small muted">added back to net</div></div>
            <div class="card"><div class="label">Realized events</div>
                <div class="value">${r.realized_count}</div>
                <div class="small muted">proceeds $${fmt(r.total_proceeds)} / basis $${fmt(r.total_basis)}</div></div>
            <div class="card"><div class="label">Open lots</div>
                <div class="value">${r.open_lot_count}</div>
                <div class="small muted">basis $${fmt(r.open_basis)}</div></div>
        </div>

        ${r.skipped_short_events > 0 ? `
            <p class="muted small">${r.skipped_short_events} short/cover or oversold event(s)
            were skipped (short-side lot tracking is a v2 feature).</p>` : ''}

        <div class="chart-panel">
            <h2>Realized events (${r.realized.length}) — method: ${r.method.toUpperCase()}</h2>
            ${r.realized.length === 0
                ? '<p class="muted small">No closed lots in this year.</p>'
                : `<table class="trades">
                    <thead><tr>
                        <th>Symbol</th><th>Acquired</th><th>Disposed</th><th>Days</th><th>Term</th>
                        <th>Qty</th><th>Basis</th><th>Proceeds</th><th>Gain/Loss</th><th>Wash</th>
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
            <h2>Open lots (${r.open_lots.length})</h2>
            ${r.open_lots.length === 0
                ? '<p class="muted small">No open lots.</p>'
                : `<table class="trades">
                    <thead><tr>
                        <th>Symbol</th><th>Acquired</th><th>Held</th><th>Term</th>
                        <th>Qty</th><th>Cost/sh</th><th>Basis</th>
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
            <p class="muted small">Holding-period clock on open lots is current as of now — closing
                these positions today would realize at their displayed term.</p>
        </div>
    `;
}
