// Position sizing calculator — fixed-fractional + R-based + Kelly,
// correlation-drag aware, can pull Kelly inputs from account history.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderPositionSize(mount, state) {
    const tok = currentViewToken();
    const acct = state.accounts.find(a => a.id === state.accountId);
    mount.innerHTML = `
        <h1 data-i18n="view.position_size.h1.position_sizing" class="view-title">// POSITION SIZING</h1>
        <p data-i18n="view.position_size.hint.three_industry_methods_side_by_side_risk_per_share" class="muted small">Three industry methods side-by-side. Risk-per-share is computed
            from the entry/stop distance per side. Correlation drag halves your share count
            at ρ=1 (one perfectly-correlated open position), reduces by 33% at ρ=0.5, and so on.
            Kelly defaults to half-Kelly because full Kelly is brutal in real drawdowns.</p>

        <div class="chart-panel">
            <form id="ps-form" class="inline-form">
                <label>Side
                    <select name="side">
                        <option data-i18n="view.position_size.opt.long" value="long" selected>long</option>
                        <option data-i18n="view.position_size.opt.short" value="short">short</option>
                    </select>
                </label>
                <label><span data-i18n="view.position_size.label.entry">Entry</span>  <input name="entry" type="number" step="any" required value="50" style="width:90px;"></label>
                <label><span data-i18n="view.position_size.label.stop">Stop</span>   <input name="stop"  type="number" step="any" required value="48" style="width:90px;"></label>
                <label><span data-i18n="view.position_size.label.equity">Equity</span> <input name="equity" type="number" step="any" required value="100000" style="width:120px;"></label>
                <label><span data-i18n="view.position_size.label.risk_pct">Risk % per trade</span>
                    <input name="risk_pct" type="number" step="0.01" min="0.01" max="10" value="1" style="width:80px;">
                </label>
                <label><span data-i18n="view.position_size.label.r_dollars">R ($)</span>
                    <input name="r_dollars" type="number" step="any" min="0" value="500" style="width:100px;">
                </label>
                <label><span data-i18n="view.position_size.label.max_position_pct">Max position %</span>
                    <input name="max_pos_pct" type="number" step="1" min="0" max="100" value="25" style="width:80px;">
                </label>
                <label><span data-i18n="view.position_size.label.correlation_drag">Correlation drag</span>
                    <input name="corr_drag" type="number" step="0.05" min="0" max="5" value="0" style="width:80px;">
                </label>
            </form>
            <form id="ps-kelly" class="inline-form" style="margin-top:8px;">
                <span class="muted small" data-i18n="view.position_size.kelly_inputs">Kelly inputs:</span>
                <label><span data-i18n="view.position_size.label.win_rate">Win rate</span> <input name="win_rate" type="number" step="0.01" min="0" max="1" value="0.55" style="width:80px;"></label>
                <label><span data-i18n="view.position_size.label.avg_win">Avg win</span>  <input name="avg_win"  type="number" step="any" min="0" value="1.5" style="width:90px;"></label>
                <label><span data-i18n="view.position_size.label.avg_loss">Avg loss</span> <input name="avg_loss" type="number" step="any" min="0" value="1.0" style="width:90px;"></label>
                <label><span data-i18n="view.position_size.label.fractional">Fractional</span> <input name="frac_k" type="number" step="0.05" min="0" max="1" value="0.5" style="width:80px;"></label>
                <button data-i18n="view.position_size.btn.pull_from_history" type="button" class="btn" id="ps-fill-history" ${acct ? '' : 'disabled'}
                        title="${acct ? 'Pull win-rate + avg win/loss from this account history' : 'no account selected'}">
                    Pull from history
                </button>
                <span class="muted small" id="ps-fill-status"></span>
            </form>
            <div style="margin-top:8px;">
                <button data-i18n="view.position_size.btn.compute" class="primary" id="ps-go">Compute</button>
                <span class="muted small" id="ps-status"></span>
            </div>
        </div>

        <div id="ps-out"></div>
    `;

    mount.querySelector('#ps-go').addEventListener('click', () => compute(mount, tok));
    mount.querySelector('#ps-fill-history').addEventListener('click', async () => {
        if (!acct) return;
        const status = mount.querySelector('#ps-fill-status');
        if (status) status.textContent = 'fetching…';
        try {
            const r = await api.positionSizeWinRate(acct.id);
            if (!viewIsCurrent(tok)) return;
            const wr = mount.querySelector('#ps-kelly [name=win_rate]');
            const aw = mount.querySelector('#ps-kelly [name=avg_win]');
            const al = mount.querySelector('#ps-kelly [name=avg_loss]');
            if (wr) wr.value = r.win_rate.toFixed(4);
            if (aw) aw.value = r.avg_win.toFixed(2);
            if (al) al.value = r.avg_loss.toFixed(2);
            const s2 = mount.querySelector('#ps-fill-status');
            if (s2) s2.textContent = `loaded ${r.wins}W / ${r.losses}L (${r.samples} closed)`;
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const s2 = mount.querySelector('#ps-fill-status');
            if (s2) s2.textContent = 'error: ' + e.message;
        }
    });
    await compute(mount, tok);
}

async function compute(mount, tok) {
    const f1 = mount.querySelector('#ps-form');
    const f2 = mount.querySelector('#ps-kelly');
    if (!f1 || !f2) return;
    const body = {
        side: f1.side.value,
        entry: Number(f1.entry.value),
        stop:  Number(f1.stop.value),
        equity: Number(f1.equity.value),
        correlation_drag: Number(f1.corr_drag.value) || 0,
        max_position_pct: (Number(f1.max_pos_pct.value) || 0) / 100,
        fixed_fractional: { risk_pct: (Number(f1.risk_pct.value) || 1) / 100 },
        r_based:          { risk_dollars: Number(f1.r_dollars.value) || 0 },
        kelly:            {
            win_rate: Number(f2.win_rate.value),
            avg_win:  Number(f2.avg_win.value),
            avg_loss: Number(f2.avg_loss.value),
            fractional_kelly: Number(f2.frac_k.value),
        },
        recommended_method: 'fixed_fractional',
    };
    const status = mount.querySelector('#ps-status');
    if (status) status.textContent = 'computing…';
    try {
        const r = await api.positionSize(body);
        if (!viewIsCurrent(tok)) return;
        render(r, mount);
        const s2 = mount.querySelector('#ps-status');
        if (s2) s2.textContent = '';
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const s2 = mount.querySelector('#ps-status');
        if (s2) s2.textContent = 'error: ' + e.message;
    }
}

function render(r, mount) {
    const out = mount.querySelector('#ps-out');
    if (!out) return;
    const sizing = (s, color) => {
        if (!s) return `<div class="card"><div class="label">${color}</div>
            <div class="muted small">n/a</div></div>`;
        const cap = s.capped_by_position_pct ? ' (capped)' : '';
        return `<div class="card" style="border-left-color:${color};">
            <div class="label">${esc(s.method)}</div>
            <div class="value">${s.shares.toLocaleString()} sh${cap}</div>
            <div class="small">$${fmt(s.notional)} notional · ${(s.position_pct_of_equity * 100).toFixed(2)}% of equity</div>
            <div class="small ${s.risk_dollars > 0 ? 'neg' : 'muted'}">$${fmt(s.risk_dollars)} at risk</div>
            <div class="muted small">corr × ${s.correlation_multiplier.toFixed(2)}</div>
            <div class="muted small" style="margin-top:4px;">${esc(s.note)}</div>
        </div>`;
    };
    out.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.position_size.h2.inputs">Inputs</h2>
            <div class="muted small">
                ${r.inputs.side} · entry $${fmt(r.inputs.entry)} · stop $${fmt(r.inputs.stop)} ·
                equity $${fmt(r.inputs.equity)} ·
                <strong>risk/share $${fmt(r.risk_per_share)}</strong>
            </div>
            ${r.warnings.length === 0 ? '' :
                `<ul class="muted small" style="margin-top:6px;">
                    ${r.warnings.map(w => `<li>${esc(w)}</li>`).join('')}
                </ul>`}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.position_size.h2.sized_by_method">Sized by method</h2>
            <div class="cards">
                ${sizing(r.fixed_fractional, '#00e5ff')}
                ${sizing(r.r_based, '#7af0a8')}
                ${sizing(r.kelly, '#ff7a1f')}
            </div>
        </div>
        ${r.recommended ? `<div class="chart-panel">
            <h2 data-i18n="view.position_size.h2.recommended">Recommended</h2>
            <div class="cards">${sizing(r.recommended, '#ffd24a')}</div>
        </div>` : ''}
    `;
}
