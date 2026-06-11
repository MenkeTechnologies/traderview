// DCF intrinsic-value calculator — two-stage discounted cash flow.
// All math runs server-side (POST /calc/dcf); this view is the form +
// result table. Optionally pre-fills FCF/shares from fundamentals when
// a symbol is given in the hash (#dcf/AAPL).

import { api } from '../api.js';
import { esc } from '../util.js';
import { t, applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const FIELDS = [
    { key: 'fcf_usd',             label: 'Free cash flow (TTM, $)',  def: 100_000_000_000 },
    { key: 'growth_pct',          label: 'Growth rate (%/yr)',       def: 8 },
    { key: 'growth_years',        label: 'Growth years',             def: 5 },
    { key: 'terminal_growth_pct', label: 'Terminal growth (%)',      def: 2.5 },
    { key: 'discount_rate_pct',   label: 'Discount rate (%)',        def: 10 },
    { key: 'net_debt_usd',        label: 'Net debt ($, neg = cash)', def: 0 },
    { key: 'shares_outstanding',  label: 'Shares outstanding',       def: 15_000_000_000 },
    { key: 'current_price',       label: 'Current price ($, optional)', def: '' },
];

export async function renderDcfValuation(mount, _state, sym) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.dcf.h1" class="view-title">// DCF INTRINSIC VALUE</h1>
        <p class="muted small" data-i18n="view.dcf.subtitle">
            Two-stage discounted cash flow: explicit growth years + Gordon terminal value,
            discounted to present, minus net debt, per share.
        </p>
        <div class="chart-panel">
            <form id="dcf-form" class="dcf-form">
                ${FIELDS.map(f => `
                    <label class="dcf-field">
                        <span class="dcf-label">${esc(f.label)}</span>
                        <input name="${f.key}" type="number" step="any" value="${f.def}">
                    </label>`).join('')}
                <button type="submit" class="primary" data-i18n="view.dcf.btn.compute">Compute</button>
            </form>
        </div>
        <div class="chart-panel">
            <div id="dcf-result" class="muted" data-i18n="view.dcf.hint.fill_form">Fill the form and hit Compute.</div>
        </div>
    `;
    try { applyUiI18n(mount); } catch (_) {}

    // Pre-fill from fundamentals when a symbol rides in the hash.
    if (sym) {
        api.symbolFundamentals(sym).then(f => {
            if (!viewIsCurrent(tok) || !f) return;
            const set = (k, v) => {
                const inp = mount.querySelector(`input[name="${k}"]`);
                if (inp && v != null && isFinite(v)) inp.value = v;
            };
            set('fcf_usd', f.free_cash_flow ?? f.freeCashflow);
            set('shares_outstanding', f.shares_outstanding ?? f.sharesOutstanding);
        }).catch(() => {});
        api.quote(sym).then(q => {
            if (!viewIsCurrent(tok) || !q) return;
            const inp = mount.querySelector('input[name="current_price"]');
            if (inp && q.price) inp.value = q.price;
        }).catch(() => {});
    }

    mount.querySelector('#dcf-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {};
        for (const f of FIELDS) {
            const raw = fd.get(f.key);
            if (f.key === 'current_price') {
                body[f.key] = raw === '' ? null : Number(raw);
            } else if (f.key === 'growth_years') {
                body[f.key] = parseInt(raw, 10) || 5;
            } else {
                body[f.key] = Number(raw) || 0;
            }
        }
        const out = mount.querySelector('#dcf-result');
        out.textContent = '…';
        try {
            const r = await api.calcDcf(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(out, r);
        } catch (err) {
            out.innerHTML = `<span class="neg">${esc(err.message || String(err))}</span>`;
        }
    });
}

function renderResult(el, r) {
    const fmtB = (v) => {
        const a = Math.abs(v);
        if (a >= 1e12) return (v / 1e12).toFixed(2) + 'T';
        if (a >= 1e9) return (v / 1e9).toFixed(2) + 'B';
        if (a >= 1e6) return (v / 1e6).toFixed(2) + 'M';
        return v.toFixed(0);
    };
    const verdictCls = { undervalued: 'pos', overvalued: 'neg', fair: 'neutral' }[r.verdict] || '';
    const rows = (r.yearly || []).map(y => `
        <tr><td>${y.year}</td>
            <td>$${fmtB(y.fcf_usd)}</td>
            <td>$${fmtB(y.present_value_usd)}</td></tr>`).join('');
    el.innerHTML = `
        <div class="cards">
            <div class="card"><div class="label">Intrinsic / share</div>
                <div class="value">$${r.intrinsic_per_share.toFixed(2)}</div>
                ${r.upside_pct != null ? `<div class="small ${r.upside_pct >= 0 ? 'pos' : 'neg'}">${(r.upside_pct >= 0 ? '+' : '') + r.upside_pct.toFixed(1)}% ${esc(r.verdict || '')}</div>` : ''}
            </div>
            <div class="card"><div class="label">Equity value</div><div class="value">$${fmtB(r.equity_value_usd)}</div></div>
            <div class="card"><div class="label">PV stage 1</div><div class="value">$${fmtB(r.pv_stage1_usd)}</div></div>
            <div class="card"><div class="label">PV terminal</div><div class="value">$${fmtB(r.pv_terminal_usd)}</div></div>
        </div>
        ${r.verdict ? `<p class="dcf-verdict ${verdictCls}">${esc(r.verdict.toUpperCase())}</p>` : ''}
        <table class="gs-table">
            <thead><tr><th>Year</th><th>Projected FCF</th><th>Present value</th></tr></thead>
            <tbody>${rows}</tbody>
        </table>
    `;
}
