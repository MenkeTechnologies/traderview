// Modified internal rate of return (MIRR) — IRR with explicit finance and
// reinvestment rates, via /calc/mirr. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => (n == null ? '—' : (n < 0 ? '−$' : '$') + Math.abs(Number(n)).toLocaleString(undefined, { maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%');

function parseFlows(raw) {
    return String(raw || '')
        .split(/[\s,]+/)
        .map((s) => s.trim())
        .filter((s) => s.length)
        .map(Number)
        .filter((n) => Number.isFinite(n));
}

export async function renderMirr(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mirr.h1.title">// MODIFIED IRR (MIRR)</span></h1>
        <p class="muted small" data-i18n="view.mirr.hint.intro">
            Plain IRR assumes interim cash flows are reinvested at the IRR itself — usually unrealistic.
            MIRR discounts the outflows at a finance rate and compounds the inflows forward at a
            reinvestment rate, giving one unambiguous return. Enter the period cash flows (index 0 is
            today, usually the negative outlay). Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.mirr.h2.inputs">The project</h2>
            <form id="mirr-form" class="inline-form">
                <label><span data-i18n="view.mirr.label.flows">Cash flows (comma-separated)</span>
                    <input type="text" name="cash_flows_usd" value="-1000, 300, 400, 500, 600" required></label>
                <label><span data-i18n="view.mirr.label.finance">Finance rate (%)</span>
                    <input type="number" step="0.1" name="finance_rate_pct" value="10" required></label>
                <label><span data-i18n="view.mirr.label.reinvest">Reinvestment rate (%)</span>
                    <input type="number" step="0.1" name="reinvestment_rate_pct" value="12" required></label>
            </form>
        </div>
        <div id="mirr-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#mirr-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            cash_flows_usd: parseFlows(fd.get('cash_flows_usd')),
            finance_rate_pct: Number(fd.get('finance_rate_pct')) || 0,
            reinvestment_rate_pct: Number(fd.get('reinvestment_rate_pct')) || 0,
        };
        try {
            const r = await api.calcMirr(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.mirr.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#mirr-result');
    const hasMirr = r.mirr_pct != null;
    const cls = hasMirr && r.mirr_pct >= 0 ? 'pos' : 'neg';
    const mirrCell = hasMirr ? pct(r.mirr_pct) : t('view.mirr.undefined');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.mirr.h2.result">The return</h2>
            <div class="cards">
                <div class="card ${cls}"><div class="label" data-i18n="view.mirr.card.mirr">MIRR</div>
                    <div class="value ${cls}">${mirrCell}</div></div>
                <div class="card"><div class="label" data-i18n="view.mirr.card.fv">FV of inflows</div>
                    <div class="value">${money(r.future_value_inflows_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.mirr.card.pv">PV of outflows</div>
                    <div class="value">${money(r.present_value_outflows_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.mirr.row.periods">Periods</td><td>${r.n_periods}</td></tr>
                    <tr><td data-i18n="view.mirr.row.pv">PV of outflows (finance rate)</td><td>${money(r.present_value_outflows_usd)}</td></tr>
                    <tr><td data-i18n="view.mirr.row.fv">FV of inflows (reinvest rate)</td><td>${money(r.future_value_inflows_usd)}</td></tr>
                    <tr><td data-i18n="view.mirr.row.net">Net undiscounted</td><td>${money(r.net_undiscounted_usd)}</td></tr>
                    <tr class="emph ${cls}"><td data-i18n="view.mirr.row.mirr">MIRR</td><td>${mirrCell}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
